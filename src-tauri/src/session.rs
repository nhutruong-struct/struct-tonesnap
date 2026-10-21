//! One use of ToneSnap: capture the selection, redact it, show the popup, then ask the model.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::capture::{self, CaptureError, Captured, ContextSource, SelectionSource};
use crate::platform;
use crate::prompt::{self, Payload};
use crate::redact::Redactor;
use crate::request;
use crate::settings::Settings;
use crate::state::state;
use crate::window::{self, NoticeKind, POPUP};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: u64,
    /// The window Apply pastes into.
    #[serde(skip)]
    pub hwnd: isize,
    pub app: String,
    pub source: SelectionSource,
    pub context_source: ContextSource,
    pub editable: Option<bool>,
    /// The original selection, before redaction.
    pub selection: String,
    /// Exactly what is sent to the model.
    pub payload: Payload,
    /// (placeholder, original) pairs used to restore suggestions.
    pub redactions: Vec<(String, String)>,
    pub preview: bool,
    pub model: String,
}

/// Handles the hotkey or a click on the floating icon. The capture runs on its own thread
/// because it waits on the keyboard, the clipboard and UI Automation.
pub fn on_trigger(app: AppHandle) {
    // The hotkey and the icon click arrive on the main thread, the one place the layout can be read.
    platform::refresh();
    platform::selection_icon::hide();
    let Some(capturing) = CaptureInProgress::begin(&app) else {
        return;
    };
    let settings = state(&app).settings();
    std::thread::spawn(move || {
        let _capturing = capturing;
        match capture::capture(&settings) {
            Ok(captured) => start(&app, &settings, captured),
            Err(CaptureError::OwnWindow) => window::dismiss_popup(&app),
            Err(CaptureError::NoSelection) => {
                window::show_notice(&app, NoticeKind::NoSelection, String::new())
            }
            Err(CaptureError::PasswordField) => {
                window::show_notice(&app, NoticeKind::PasswordField, String::new())
            }
            Err(CaptureError::BlockedApp(name)) => {
                window::show_notice(&app, NoticeKind::BlockedApp, name)
            }
            Err(CaptureError::AccessibilityDenied) => {
                ask_for_accessibility_once();
                window::show_notice(&app, NoticeKind::AccessibilityDenied, String::new())
            }
        }
    });
}

/// Shows the system's Accessibility prompt, once per run: at startup when the permission is
/// missing, or at the first capture that needs it. The prompt also adds ToneSnap to the list in
/// System Settings and has a button that opens it. Later refusals only show the notice, and
/// Settings has the buttons for the rest.
pub fn ask_for_accessibility_once() {
    static ASKED: AtomicBool = AtomicBool::new(false);
    if !ASKED.swap(true, Ordering::SeqCst) {
        platform::request_permission(platform::Permission::Accessibility);
    }
}

/// Ignores triggers while a capture is running, and clears that flag however the capture ends.
struct CaptureInProgress(AppHandle);

impl CaptureInProgress {
    fn begin(app: &AppHandle) -> Option<Self> {
        state(app).try_begin_capture().then(|| Self(app.clone()))
    }
}

impl Drop for CaptureInProgress {
    fn drop(&mut self) {
        state(&self.0).end_capture();
    }
}

fn start(app: &AppHandle, settings: &Settings, captured: Captured) {
    let (payload, redactions) = redacted_payload(settings, &captured);
    let session = Session {
        id: state(app).next_session_id(),
        hwnd: captured.hwnd,
        app: captured.app,
        source: captured.source,
        context_source: captured.context_source,
        editable: captured.editable,
        selection: captured.selection,
        payload,
        redactions,
        preview: settings.preview_before_send,
        model: settings.active_model().to_string(),
    };
    state(app).set_session(session.clone());

    let _ = app.emit_to(POPUP, "session", &session);
    window::show_popup(app, captured.bounds, captured.cursor);
    if !session.preview {
        request::start(app, session);
    }
}

/// What the model gets: the captured text with sensitive values replaced by placeholders
/// (unless redaction is off), plus the pairs that undo the replacement.
fn redacted_payload(settings: &Settings, captured: &Captured) -> (Payload, Vec<(String, String)>) {
    let mut redactor = Redactor::new(&settings.custom_sensitive_terms);
    let mut clean = |text: &str| {
        if settings.redact {
            redactor.apply(text)
        } else {
            text.to_string()
        }
    };
    let payload = Payload {
        app: captured.app.clone(),
        context_before: clean(&captured.before),
        selection: clean(&captured.selection),
        context_after: clean(&captured.after),
        screen_text: clean(&captured.screen_text),
        target_language: settings.target_language,
        mode: settings.mode,
        tones: prompt::requested_tones(&settings.tones),
    };
    (payload, redactor.entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{Mode, TargetLanguage};

    fn captured(selection: &str, before: &str) -> Captured {
        Captured {
            hwnd: 0,
            app: "Slack".into(),
            selection: selection.into(),
            before: before.into(),
            after: String::new(),
            screen_text: String::new(),
            editable: Some(true),
            bounds: None,
            cursor: (0, 0),
            source: SelectionSource::Uia,
            context_source: ContextSource::Uia,
        }
    }

    #[test]
    fn payload_is_redacted_consistently_across_fields() {
        let settings = Settings::default();
        let (payload, redactions) =
            redacted_payload(&settings, &captured("mail an@corp.vn", "from an@corp.vn"));
        assert_eq!(payload.selection, "mail [EMAIL_1]");
        assert_eq!(payload.context_before, "from [EMAIL_1]");
        assert_eq!(
            redactions,
            vec![("[EMAIL_1]".to_string(), "an@corp.vn".to_string())]
        );
    }

    #[test]
    fn payload_starts_with_the_languages_and_mode_from_the_settings() {
        let settings = Settings {
            target_language: TargetLanguage::Ja,
            mode: Mode::Rewrite,
            ..Settings::default()
        };
        let (payload, _) = redacted_payload(&settings, &captured("pls review", ""));
        assert_eq!(payload.target_language, TargetLanguage::Ja);
        assert_eq!(payload.mode, Mode::Rewrite);
    }

    #[test]
    fn payload_requests_the_configured_tones_the_model_knows() {
        let settings = Settings {
            tones: vec!["formal".into(), "pirate".into()],
            ..Settings::default()
        };
        let (payload, _) = redacted_payload(&settings, &captured("pls review", ""));
        assert_eq!(payload.tones, ["formal"]);
    }

    #[test]
    fn payload_keeps_the_text_when_redaction_is_off() {
        let settings = Settings {
            redact: false,
            ..Settings::default()
        };
        let (payload, redactions) = redacted_payload(&settings, &captured("mail an@corp.vn", ""));
        assert_eq!(payload.selection, "mail an@corp.vn");
        assert!(redactions.is_empty());
    }

    #[test]
    fn sessions_serialize_like_the_typescript_type() {
        let settings = Settings::default();
        let captured = captured("Anh gửi báo cáo nhé", "");
        let (payload, redactions) = redacted_payload(&settings, &captured);
        let session = Session {
            id: 4,
            hwnd: 1234,
            app: captured.app.clone(),
            source: SelectionSource::Clipboard,
            context_source: ContextSource::None,
            editable: None,
            selection: captured.selection.clone(),
            payload,
            redactions,
            preview: false,
            model: settings.active_model().to_string(),
        };
        let value = serde_json::to_value(&session).unwrap();
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "app",
                "contextSource",
                "editable",
                "id",
                "model",
                "payload",
                "preview",
                "redactions",
                "selection",
                "source"
            ]
        );
        assert_eq!(value["source"], "clipboard");
        assert_eq!(value["contextSource"], "none");
        assert_eq!(value["payload"]["contextBefore"], "");
        assert!(value["payload"].get("explanationLanguage").is_none());
        assert_eq!(value["payload"]["targetLanguage"], "vi");
        assert_eq!(value["payload"]["mode"], "translate");
        assert_eq!(value["payload"]["tones"], serde_json::json!(settings.tones));
    }
}
