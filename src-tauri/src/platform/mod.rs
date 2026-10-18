//! The OS integration behind one platform-neutral surface. Orchestration code calls these
//! functions; on Windows they forward to `win`, elsewhere to inert stubs until Phase 3 (macOS).
//! A function returning `Option` answers `None` where the platform cannot tell, so the caller
//! falls back to what Tauri knows.

#[cfg(not(windows))]
mod stub;
#[cfg(windows)]
pub mod win;

#[cfg(not(windows))]
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
    pub bounds: Rect,
    /// Effective DPI scale: 1.0 at 96 DPI.
    pub scale: f64,
}
