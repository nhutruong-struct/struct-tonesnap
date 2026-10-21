//! macOS integration. Selection, context, the foreground app, the clipboard and paste are real;
//! the popup, OCR and the floating icon still use the stubs until their Phase 3 steps land.

mod ax;
mod cf;
pub mod clipboard;
pub mod input;
mod window;

use std::thread::sleep;
use std::time::{Duration, Instant};

use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

pub use ax::capture;
pub use window::{cursor, frontmost, monitor_at};

// Not built for macOS yet: `selection_icon` (NSPanel).
pub use super::stub::selection_icon;

/// Whether the foreground app is ToneSnap (the popup or Settings).
pub fn own_window_in_front() -> Option<bool> {
    Some(frontmost()?.is_own())
}

/// Puts `text` on the clipboard for the user to paste wherever they like.
pub fn copy_text(text: &str) {
    clipboard::set_text(text, false);
}

/// Brings the app with process id `pid` back to the front and pastes `text` over its selection,
/// then restores the user's clipboard. Returns false when the app could not be brought forward.
pub fn paste_into(pid: isize, text: &str) -> bool {
    let start = Instant::now();
    while frontmost().map(|f| f.pid as isize) != Some(pid) {
        if start.elapsed() > Duration::from_millis(800) {
            return false;
        }
        if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid as i32)
        {
            app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
        }
        sleep(Duration::from_millis(40));
    }
    // Give the app a moment to restore its caret and selection.
    sleep(Duration::from_millis(80));

    let snapshot = clipboard::snapshot();
    if !clipboard::set_text(text, true) {
        return false;
    }
    input::wait_modifiers_released(Duration::from_millis(400));
    input::send_command(input::KEY_V);
    // Electron apps read the clipboard asynchronously; restoring too early pastes the old content.
    sleep(Duration::from_millis(500));
    clipboard::restore(&snapshot);
    true
}
