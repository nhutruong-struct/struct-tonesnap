//! macOS integration. Selection, context and the foreground app are real (Accessibility API);
//! the popup, OCR, clipboard and paste still use the stubs until their Phase 3 steps land.

mod ax;
mod cf;
mod window;

pub use ax::capture;
pub use window::{cursor, frontmost, monitor_at};

// Not built for macOS yet: `copy_text`, `paste_into` (clipboard step), `selection_icon` (NSPanel).
pub use super::stub::{copy_text, paste_into, selection_icon};

/// Whether the foreground app is ToneSnap (the popup or Settings).
pub fn own_window_in_front() -> Option<bool> {
    Some(frontmost()?.is_own())
}
