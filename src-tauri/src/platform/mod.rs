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
pub use mac::{focus_popup, init, prepare_popup};

/// Per-platform app setup, run once at startup.
#[cfg(not(target_os = "macos"))]
pub fn init(_app: &tauri::AppHandle) {}

/// Gives the popup window the traits it needs to appear without stealing focus.
#[cfg(not(target_os = "macos"))]
pub fn prepare_popup(_window: &tauri::WebviewWindow) {}

/// Gives the shown popup the keyboard; false when the caller should focus it the usual way.
#[cfg(not(target_os = "macos"))]
pub fn focus_popup(_window: &tauri::WebviewWindow) -> bool {
    false
}

use crate::geometry::Rect;

/// A monitor, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    /// The part not covered by the taskbar.
    pub work: Rect,
    pub bounds: Rect,
    /// Effective DPI scale: 1.0 at 96 DPI.
    pub scale: f64,
}
