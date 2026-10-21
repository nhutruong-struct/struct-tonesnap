//! The OS integration behind one platform-neutral surface. Orchestration code calls these
//! functions; on Windows they forward to `win`, on macOS to `mac`, elsewhere to inert stubs.
//! A function returning `Option` answers `None` where the platform cannot tell, so the caller
//! falls back to what Tauri knows.

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

use crate::geometry::Rect;

/// A monitor, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    /// The part not covered by the taskbar.
    pub work: Rect,
    #[cfg_attr(
        not(windows),
        allow(
            dead_code,
            reason = "only the Windows OCR area uses it until the macOS OCR step"
        )
    )]
    pub bounds: Rect,
    /// Effective DPI scale: 1.0 at 96 DPI.
    pub scale: f64,
}
