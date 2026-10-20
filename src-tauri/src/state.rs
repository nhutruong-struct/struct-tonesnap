//! State shared by the commands, the capture thread and the request tasks.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use crate::commands::CommandError;
use crate::session::Session;
use crate::settings::{Mode, Settings, SettingsIssue, TargetLanguage};

pub struct AppState {
    settings: Mutex<Settings>,
    /// The selection the popup is working on.
    session: Mutex<Option<Session>>,
    session_seq: AtomicU64,
    /// Id of the current request. Bumping it cancels the running one.
    request_seq: AtomicU64,
    capturing: AtomicBool,
    popup_shown_at: Mutex<Instant>,
    /// Why the configured hotkey is not active, if it is not.
    hotkey_error: Mutex<Option<CommandError>>,
    /// What was wrong with the saved settings, until they are saved again.
    settings_issue: Mutex<Option<SettingsIssue>>,
}

pub fn state(app: &AppHandle) -> tauri::State<'_, AppState> {
    app.state::<AppState>()
}

impl AppState {
    pub fn new(settings: Settings, settings_issue: Option<SettingsIssue>) -> Self {
        Self {
            settings: Mutex::new(settings),
            session: Mutex::new(None),
            session_seq: AtomicU64::new(0),
            request_seq: AtomicU64::new(0),
            capturing: AtomicBool::new(false),
            popup_shown_at: Mutex::new(Instant::now()),
            hotkey_error: Mutex::new(None),
            settings_issue: Mutex::new(settings_issue),
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    /// Replaces the settings with ones that were just saved, which also settles any issue
    /// found when loading them.
    pub fn set_saved_settings(&self, settings: Settings) {
        *self.settings.lock().unwrap() = settings;
        *self.settings_issue.lock().unwrap() = None;
    }

    pub fn settings_issue(&self) -> Option<SettingsIssue> {
        self.settings_issue.lock().unwrap().clone()
    }

    pub fn session(&self) -> Option<Session> {
        self.session.lock().unwrap().clone()
    }

    pub fn set_session(&self, session: Session) {
        *self.session.lock().unwrap() = Some(session);
    }

    /// Sets the language and mode the next request uses for the current session, and returns
    /// the session; `None` when there is none.
    pub fn set_session_options(
        &self,
        target_language: TargetLanguage,
        mode: Mode,
    ) -> Option<Session> {
        let mut guard = self.session.lock().unwrap();
        let session = guard.as_mut()?;
        session.payload.target_language = target_language;
        session.payload.mode = mode;
        Some(session.clone())
    }

    pub fn next_session_id(&self) -> u64 {
        self.session_seq.fetch_add(1, SeqCst) + 1
    }

    /// Starts a new request, which supersedes (and so cancels) any running one.
    pub fn begin_request(&self) -> u64 {
        self.request_seq.fetch_add(1, SeqCst) + 1
    }

    pub fn is_current_request(&self, id: u64) -> bool {
        self.request_seq.load(SeqCst) == id
    }

    /// Stops the running request, if any: its stream ends at the next chunk.
    pub fn cancel_request(&self) {
        self.request_seq.fetch_add(1, SeqCst);
    }

    /// Marks a capture as running; false when one already is.
    pub fn try_begin_capture(&self) -> bool {
        !self.capturing.swap(true, SeqCst)
    }

    pub fn is_capturing(&self) -> bool {
        self.capturing.load(SeqCst)
    }

    pub fn end_capture(&self) {
        self.capturing.store(false, SeqCst);
    }

    pub fn mark_popup_shown(&self) {
        *self.popup_shown_at.lock().unwrap() = Instant::now();
    }

    pub fn popup_shown_within(&self, window: Duration) -> bool {
        self.popup_shown_at.lock().unwrap().elapsed() <= window
    }

    pub fn hotkey_error(&self) -> Option<CommandError> {
        self.hotkey_error.lock().unwrap().clone()
    }

    pub fn set_hotkey_error(&self, error: Option<CommandError>) {
        *self.hotkey_error.lock().unwrap() = error;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{ContextSource, SelectionSource};

    fn session() -> Session {
        Session {
            id: 1,
            hwnd: 0,
            app: "Slack".into(),
            source: SelectionSource::Uia,
            context_source: ContextSource::Uia,
            editable: Some(true),
            selection: "hello".into(),
            payload: Default::default(),
            redactions: Vec::new(),
            preview: false,
            model: "m".into(),
        }
    }

    #[test]
    fn a_new_request_supersedes_the_running_one() {
        let state = AppState::new(Settings::default(), None);
        let first = state.begin_request();
        assert!(state.is_current_request(first));
        let second = state.begin_request();
        assert!(!state.is_current_request(first));
        assert!(state.is_current_request(second));
    }

    #[test]
    fn cancelling_stops_the_running_request_without_starting_another() {
        let state = AppState::new(Settings::default(), None);
        let running = state.begin_request();
        state.cancel_request();
        assert!(!state.is_current_request(running));
        // The next request is still told apart from the cancelled one.
        assert!(state.is_current_request(state.begin_request()));
    }

    #[test]
    fn only_one_capture_runs_at_a_time() {
        let state = AppState::new(Settings::default(), None);
        assert!(!state.is_capturing());
        assert!(state.try_begin_capture());
        assert!(state.is_capturing());
        assert!(!state.try_begin_capture());
        state.end_capture();
        assert!(!state.is_capturing());
        assert!(state.try_begin_capture());
    }

    #[test]
    fn session_ids_count_up_from_one() {
        let state = AppState::new(Settings::default(), None);
        assert_eq!(state.next_session_id(), 1);
        assert_eq!(state.next_session_id(), 2);
    }

    #[test]
    fn saving_settings_settles_the_loading_issue() {
        let state = AppState::new(Settings::default(), Some(SettingsIssue::Unreadable));
        assert_eq!(state.settings_issue(), Some(SettingsIssue::Unreadable));
        let saved = Settings {
            hotkey: "ctrl+alt+r".into(),
            ..Settings::default()
        };
        state.set_saved_settings(saved);
        assert_eq!(state.settings().hotkey, "ctrl+alt+r");
        assert_eq!(state.settings_issue(), None);
    }

    #[test]
    fn the_popup_counts_as_recent_only_for_a_while() {
        let state = AppState::new(Settings::default(), None);
        state.mark_popup_shown();
        assert!(state.popup_shown_within(Duration::from_secs(60)));
        std::thread::sleep(Duration::from_millis(20));
        assert!(!state.popup_shown_within(Duration::from_millis(1)));
    }

    #[test]
    fn the_hotkey_error_is_kept_until_it_is_cleared() {
        let state = AppState::new(Settings::default(), None);
        assert!(state.hotkey_error().is_none());
        state.set_hotkey_error(Some(CommandError::hotkey(
            "hotkey_taken",
            "ctrl+alt+e",
            "in use".into(),
        )));
        assert_eq!(state.hotkey_error().unwrap().hotkey, "ctrl+alt+e");
        state.set_hotkey_error(None);
        assert!(state.hotkey_error().is_none());
    }

    #[test]
    fn the_popup_can_change_the_language_and_mode_of_the_current_session() {
        let state = AppState::new(Settings::default(), None);
        assert!(state
            .set_session_options(TargetLanguage::En, Mode::Rewrite)
            .is_none());

        state.set_session(session());
        let changed = state
            .set_session_options(TargetLanguage::En, Mode::Rewrite)
            .unwrap();
        assert_eq!(changed.payload.target_language, TargetLanguage::En);
        assert_eq!(changed.payload.mode, Mode::Rewrite);
        // The next request starts from what was chosen.
        assert_eq!(
            state.session().unwrap().payload.target_language,
            TargetLanguage::En
        );
    }
}
