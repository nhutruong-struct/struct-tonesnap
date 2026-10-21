//! The popup and Settings windows: where the popup goes, showing and hiding them, and closing
//! the popup once the user has moved on to another app.

use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

use crate::geometry::Rect;
use crate::platform;
use crate::state::state;

pub const POPUP: &str = "popup";
pub const SETTINGS: &str = "settings";
const POPUP_WIDTH: f64 = 540.0;
const POPUP_HEIGHT: f64 = 500.0;
const NOTICE_HEIGHT: f64 = 120.0;

/// A short message the popup shows instead of suggestions.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeKind {
    NoSelection,
    PasswordField,
    BlockedApp,
    ApplyFailed,
    AccessibilityDenied,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Notice {
    kind: NoticeKind,
    /// The blocked app's name for `BlockedApp`, empty otherwise.
    detail: String,
}

pub fn show_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS) {
        // The page reloads saved values so unsaved edits from last time do not linger.
        let _ = app.emit_to(SETTINGS, "settings-opened", ());
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Hides the popup and stops the request it was waiting for. A hidden popup never shows that
/// session again (it reopens only for a new session or a notice), so letting the request run
/// on would only use up the provider's request quota or bill the API for text nobody sees.
pub fn dismiss_popup(app: &AppHandle) {
    state(app).cancel_request();
    if let Some(w) = app.get_webview_window(POPUP) {
        let _ = w.hide();
    }
}

/// Shows the popup for a new session next to the selection, or at the pointer when the app
/// does not say where the selection is.
pub fn show_popup(app: &AppHandle, selection: Option<Rect>, cursor: (i32, i32)) {
    place_and_show(app, selection, cursor, POPUP_HEIGHT);
}

/// Shows a notice in a small popup at the pointer.
pub fn show_notice(app: &AppHandle, kind: NoticeKind, detail: String) {
    let _ = app.emit_to(POPUP, "notice", Notice { kind, detail });
    place_and_show(app, None, platform::cursor(), NOTICE_HEIGHT);
}

fn place_and_show(app: &AppHandle, selection: Option<Rect>, cursor: (i32, i32), height: f64) {
    let Some(win) = app.get_webview_window(POPUP) else {
        return;
    };
    let point = selection.map(|b| (b.left, b.bottom)).unwrap_or(cursor);
    let monitor = platform::monitor_at(point.0, point.1);
    let anchor =
        selection.unwrap_or_else(|| Rect::around(cursor, 0, (12.0 * monitor.scale) as i32));
    let frame = popup_frame(anchor, monitor.work, monitor.scale, height);

    let _ = win.set_position(PhysicalPosition::new(frame.left, frame.top));
    let _ = win.set_size(PhysicalSize::new(
        frame.width() as u32,
        frame.height() as u32,
    ));
    state(app).mark_popup_shown();
    let _ = win.show();
    if !platform::focus_popup(&win) {
        let _ = win.set_focus();
    }
}

/// Where the popup goes: just below the anchor, or above it when there is no room below, kept
/// inside the monitor's work area.
fn popup_frame(anchor: Rect, work: Rect, scale: f64, height: f64) -> Rect {
    let w = (POPUP_WIDTH * scale) as i32;
    let h = (height * scale) as i32;
    let gap = (10.0 * scale) as i32;

    let mut y = anchor.bottom + gap;
    if y + h > work.bottom {
        y = anchor.top - gap - h;
    }
    if y < work.top {
        y = (work.bottom - h).max(work.top);
    }
    let x = anchor
        .left
        .clamp(work.left, (work.right - w).max(work.left));
    Rect {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

/// Called when the popup loses focus. Closes it only once another app has really come to the
/// front.
pub fn on_popup_blurred(app: &AppHandle) {
    // Ignore the focus churn that happens while the popup is being shown.
    if state(app).popup_shown_within(Duration::from_millis(400)) {
        return;
    }
    // Dragging or clicking inside the popup briefly moves focus between the webview and its
    // frame, so look again a moment later.
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        if !own_window_in_front(&app) {
            dismiss_popup(&app);
        }
    });
}

/// Whether the popup or Settings is in front. Tauri's own focus state is the fallback; it is
/// less reliable because focus moves between a webview and its frame.
fn own_window_in_front(app: &AppHandle) -> bool {
    platform::own_window_in_front().unwrap_or_else(|| {
        app.webview_windows()
            .values()
            .any(|w| w.is_focused().unwrap_or(false))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notices_serialize_like_the_typescript_type() {
        let kinds = [
            NoticeKind::NoSelection,
            NoticeKind::PasswordField,
            NoticeKind::BlockedApp,
            NoticeKind::ApplyFailed,
        ]
        .map(|k| serde_json::to_value(k).unwrap());
        assert_eq!(
            kinds,
            [
                "no_selection",
                "password_field",
                "blocked_app",
                "apply_failed"
            ]
        );
        let notice = Notice {
            kind: NoticeKind::BlockedApp,
            detail: "KeePass".into(),
        };
        assert_eq!(
            serde_json::to_value(notice).unwrap(),
            serde_json::json!({ "kind": "blocked_app", "detail": "KeePass" })
        );
    }

    const WORK: Rect = Rect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1040,
    };

    fn selection_at(top: i32) -> Rect {
        Rect {
            left: 300,
            top,
            right: 500,
            bottom: top + 20,
        }
    }

    #[test]
    fn goes_below_the_selection_when_there_is_room() {
        let frame = popup_frame(selection_at(100), WORK, 1.0, POPUP_HEIGHT);
        assert_eq!((frame.left, frame.top), (300, 130));
        assert_eq!((frame.width(), frame.height()), (540, 500));
    }

    #[test]
    fn goes_above_the_selection_near_the_bottom() {
        let frame = popup_frame(selection_at(900), WORK, 1.0, POPUP_HEIGHT);
        assert_eq!(frame.bottom, 890);
    }

    #[test]
    fn stays_inside_the_work_area() {
        let wide = Rect {
            left: 1800,
            top: 100,
            right: 1900,
            bottom: 120,
        };
        let frame = popup_frame(wide, WORK, 1.0, POPUP_HEIGHT);
        assert_eq!(frame.right, WORK.right);

        let tall = popup_frame(selection_at(400), WORK, 2.0, POPUP_HEIGHT);
        assert!(tall.top >= WORK.top && tall.height() == 1000);
    }
}
