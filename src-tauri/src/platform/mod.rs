//! The OS integration behind one platform-neutral surface. Orchestration code calls these
//! functions; on Windows they forward to `win`, on macOS to `mac`, elsewhere to inert stubs.
//! `win` and `mac` are private, so nothing outside this module can bypass the surface.
//! A function returning `Option` answers `None` where the platform cannot tell, so the caller
//! falls back to what Tauri knows.

#[cfg(any(windows, target_os = "macos"))]
mod gesture;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(not(any(windows, target_os = "macos")))]
mod stub;
#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))]
use stub as imp;
#[cfg(windows)]
use win as imp;

use std::thread::sleep;
use std::time::{Duration, Instant};

pub use imp::{
    copy_selection, cursor, foreground, has_accessibility, monitor_at, own_window_in_front,
    read_focused, recognize_text, selection_icon, wait_modifiers_released,
};
#[cfg(target_os = "macos")]
pub use mac::{focus_popup, init, prepare_popup, refresh};

/// Per-platform app setup, run once at startup.
#[cfg(not(target_os = "macos"))]
pub fn init(_app: &tauri::AppHandle) {}

/// Re-reads what the OS tells about the keyboard layout; called on the main thread before a capture.
#[cfg(not(target_os = "macos"))]
pub fn refresh() {}

/// Gives the popup window the traits it needs to appear without stealing focus.
#[cfg(not(target_os = "macos"))]
pub fn prepare_popup(_window: &tauri::WebviewWindow) {}

/// Gives the shown popup the keyboard; false when the caller should focus it the usual way.
#[cfg(not(target_os = "macos"))]
pub fn focus_popup(_window: &tauri::WebviewWindow) -> bool {
    false
}

use serde::{Deserialize, Serialize};

use crate::geometry::Rect;

/// The app a capture reads from and Apply pastes back into: a window handle on Windows, a
/// process id on macOS. Only the platform layer looks inside.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Target(isize);

/// The app in front when ToneSnap was triggered.
#[derive(Debug)]
pub struct Foreground {
    pub target: Target,
    /// Lower-case executable name, e.g. `winword.exe` or `1password`; what `blockedApps` matches.
    pub process: String,
    /// What the user knows the app as, where the OS says (macOS); otherwise the caller names it.
    pub name: Option<String>,
    /// ToneSnap itself: one of its windows is in front or holds the keyboard.
    pub own: bool,
}

/// What the accessibility API (UI Automation, the macOS Accessibility API) tells about the
/// focused control. Empty fields mean it did not say.
#[derive(Debug, Default)]
pub struct Accessible {
    pub selection: String,
    pub before: String,
    pub after: String,
    /// `None` when the control does not say.
    pub editable: Option<bool>,
    /// The selection's rectangle, in physical screen pixels.
    pub bounds: Option<Rect>,
    pub is_password: bool,
}

/// Puts `text` on the clipboard for the user to paste wherever they like.
pub fn copy_text(text: &str) {
    imp::clipboard::set_text(text, false);
}

/// Brings `target` back to the front and pastes `text` over its selection, then restores the
/// user's clipboard. False when the target could not be brought forward or the clipboard could
/// not be written.
pub fn paste_into(target: Target, text: &str) -> bool {
    let start = Instant::now();
    while !imp::is_in_front(target) {
        if start.elapsed() > Duration::from_millis(800) {
            return false;
        }
        imp::bring_forward(target);
        sleep(Duration::from_millis(40));
    }
    // Give the app a moment to restore its caret and selection.
    sleep(Duration::from_millis(80));

    let snapshot = imp::clipboard::snapshot();
    if !imp::clipboard::set_text(text, true) {
        return false;
    }
    imp::wait_modifiers_released(Duration::from_millis(400));
    imp::send_paste();
    // Electron apps read the clipboard asynchronously; restoring too early pastes the old content.
    sleep(Duration::from_millis(500));
    imp::clipboard::restore(&snapshot);
    true
}

/// A permission the OS makes the user grant before ToneSnap can work (macOS only).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read the selection and press Cmd+C and Cmd+V.
    Accessibility,
    /// Capture the screen for OCR context.
    ScreenRecording,
}

/// Where the permissions stand. Without a permission system (`required` false) both are granted.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub required: bool,
    pub accessibility: bool,
    pub screen_recording: bool,
}

#[cfg(target_os = "macos")]
pub use mac::permissions::{
    open_settings as open_permission_settings, request as request_permission,
};

/// Where the permissions stand.
#[cfg(target_os = "macos")]
pub fn permissions() -> Permissions {
    mac::permissions::permissions()
}

/// Where the permissions stand: there is nothing to grant on this platform.
#[cfg(not(target_os = "macos"))]
pub fn permissions() -> Permissions {
    Permissions {
        required: false,
        accessibility: true,
        screen_recording: true,
    }
}

/// Shows the system prompt for a permission.
#[cfg(not(target_os = "macos"))]
pub fn request_permission(_permission: Permission) {}

/// Opens the system's settings page for a permission.
#[cfg(not(target_os = "macos"))]
pub fn open_permission_settings(_permission: Permission) {}

/// A monitor, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    /// The part not covered by the taskbar.
    pub work: Rect,
    pub bounds: Rect,
    /// Effective DPI scale: 1.0 at 96 DPI.
    pub scale: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_reach_the_ui_in_camel_case() {
        let json = serde_json::to_value(Permissions {
            required: true,
            accessibility: false,
            screen_recording: true,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "required": true, "accessibility": false, "screenRecording": true })
        );
    }

    #[test]
    fn a_permission_is_named_in_snake_case_by_the_ui() {
        let parse = |name: &str| serde_json::from_str::<Permission>(&format!("\"{name}\""));
        assert_eq!(parse("accessibility").unwrap(), Permission::Accessibility);
        assert_eq!(
            parse("screen_recording").unwrap(),
            Permission::ScreenRecording
        );
        assert!(parse("camera").is_err());
    }

    #[test]
    fn nothing_is_required_where_the_os_has_no_permission_system() {
        if cfg!(not(target_os = "macos")) {
            assert!(!permissions().required);
        }
    }
}
