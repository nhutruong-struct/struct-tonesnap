//! macOS integration. Selection, context, the foreground app, the clipboard and paste are real;
//! OCR is the one piece still to come (Phase 3).

mod ax;
mod cf;
pub mod clipboard;
pub mod input;
pub mod ocr;
mod panel;
pub mod selection_icon;
mod window;

use std::thread::sleep;
use std::time::{Duration, Instant};

use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

pub use ax::capture;
use window::on_main;
pub use window::{cursor, frontmost, monitor_at};

/// Whether ToneSnap's popup or Settings is in front. The popup never makes ToneSnap the active
/// app, so the frontmost app alone does not tell: one of our windows holding the keyboard does.
pub fn own_window_in_front() -> Option<bool> {
    if frontmost()?.is_own() {
        return Some(true);
    }
    on_main(panel::has_key_window)
}

/// Runs ToneSnap as a menu-bar app: no Dock icon and no app menu, and prepares the OCR.
pub fn init(app: &tauri::AppHandle) {
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    if ocr::screen_capture_allowed() {
        ocr::warm_up();
    }
}

/// Makes the popup window a non-activating panel.
pub fn prepare_popup(window: &tauri::WebviewWindow) {
    let Ok(native) = window.ns_window() else {
        return;
    };
    let native = native as usize;
    let _ = window.run_on_main_thread(move || {
        if !panel::make_panel(native as *mut std::ffi::c_void) {
            eprintln!("the popup could not become a panel; it stays an ordinary window");
        }
    });
}

/// Gives the shown popup the keyboard without activating ToneSnap. False when the window is
/// not a panel, so the caller focuses it the usual way.
pub fn focus_popup(window: &tauri::WebviewWindow) -> bool {
    let Ok(native) = window.ns_window() else {
        return false;
    };
    let native = native as usize;
    window
        .run_on_main_thread(move || panel::focus_panel(native as *mut std::ffi::c_void))
        .is_ok()
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
