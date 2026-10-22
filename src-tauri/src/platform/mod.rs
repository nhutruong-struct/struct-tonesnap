//! The OS integration behind one platform-neutral surface. Orchestration code calls these
//! functions; on Windows they forward to `win`, on macOS to `mac`, elsewhere to inert stubs.
//! A function returning `Option` answers `None` where the platform cannot tell, so the caller
//! falls back to what Tauri knows.

#[cfg(any(windows, target_os = "macos"))]
mod gesture;
#[cfg(target_os = "macos")]
pub mod mac;
#[cfg(not(windows))]
#[cfg_attr(
    target_os = "macos",
    allow(
        dead_code,
        reason = "macOS borrows only the stubs its Phase 3 steps have not replaced"
    )
)]
mod stub;
#[cfg(windows)]
pub mod win;

#[cfg(target_os = "macos")]
use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))]
use stub as imp;
#[cfg(windows)]
use win as imp;

pub use imp::{copy_text, cursor, monitor_at, own_window_in_front, paste_into, selection_icon};
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
