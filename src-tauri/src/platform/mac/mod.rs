//! macOS integration: the selection and its context through the Accessibility API, the
//! foreground app, the clipboard, Cmd+C and Cmd+V, OCR through Vision, and the permissions.

mod ax;
mod cf;
pub mod clipboard;
pub mod input;
pub mod ocr;
mod panel;
pub mod permissions;
pub mod selection_icon;
mod window;

use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};

pub use clipboard::copy_selection;
pub use input::refresh_layout as refresh;
pub use input::wait_modifiers_released;
use window::on_main;
pub use window::{cursor, frontmost, monitor_at};

use super::{Accessible, Anchor, Foreground, InPlace, Target};
use crate::geometry::Rect;

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
    input::refresh_layout();
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

/// Whether the user allowed ToneSnap under Privacy & Security → Accessibility.
pub fn has_accessibility() -> bool {
    ax::is_trusted()
}

/// The frontmost app, identified by its process id. The popup does not make ToneSnap the
/// active app, so one of our windows holding the keyboard also counts as ToneSnap.
pub fn foreground() -> Option<Foreground> {
    let front = frontmost()?;
    let own = front.is_own() || own_window_in_front() == Some(true);
    Some(Foreground {
        target: Target(front.pid as isize),
        process: front.process,
        name: Some(front.name),
        own,
    })
}

/// The focused control of the app `target` as the Accessibility API sees it.
pub fn read_focused(target: Target, context_chars: usize) -> Accessible {
    let ax = ax::capture(target.0 as i32, context_chars);
    Accessible {
        selection: ax.selection,
        before: ax.before,
        after: ax.after,
        editable: ax.editable,
        bounds: ax.bounds,
        is_password: ax.is_password,
        anchor: None,
    }
}

pub fn recognize_text(area: Rect, scale: f64) -> Option<Vec<String>> {
    ocr::recognize(area, scale)
}

pub(super) fn is_in_front(target: Target) -> bool {
    frontmost().map(|f| f.pid as isize) == Some(target.0)
}

pub(super) fn bring_forward(target: Target) {
    if let Some(app) =
        NSRunningApplication::runningApplicationWithProcessIdentifier(target.0 as i32)
    {
        app.activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
    }
}

pub(super) fn send_paste() {
    input::send_command(input::key_v());
}

/// No app is edited in place on macOS, so no anchor exists.
pub(super) fn edit_in_place(anchor: &Anchor, _text: &str) -> InPlace {
    match *anchor {}
}

/// Tests that send real key presses and move focus. They run only on request
/// (`cargo test input_tests -- --ignored`), never from hooks or CI: keys typed meanwhile could
/// land in another window. Select some text in a native text field (TextEdit) first and leave
/// that app in front; the test refuses to run while ToneSnap or the terminal is.
///
/// Assertions never print clipboard contents: the clipboard and the selection are the user's.
#[cfg(test)]
mod input_tests {
    use std::time::Duration;

    use super::*;
    use crate::platform::paste_into;

    /// Puts the user's clipboard back however the test ends.
    struct KeepUserClipboard(clipboard::Snapshot);

    impl Drop for KeepUserClipboard {
        fn drop(&mut self) {
            clipboard::restore(&self.0);
        }
    }

    fn clipboard_is(expected: &str) -> bool {
        clipboard::get_text().as_deref() == Some(expected)
    }

    #[test]
    #[ignore = "sends key presses and takes focus: select text in TextEdit first"]
    fn copies_the_selection_and_pastes_over_it_keeping_the_clipboard() {
        let _user = KeepUserClipboard(clipboard::snapshot());
        assert!(clipboard::set_text("what the user had copied", true));
        let front = frontmost().expect("some app is in front");
        assert!(!front.is_own(), "ToneSnap is in front");
        assert_eq!(front.process, "textedit", "put TextEdit in front");

        input::wait_modifiers_released(Duration::from_millis(2000));
        let copied = clipboard::copy_selection();
        assert!(copied.is_some(), "Cmd+C did not return the selection");
        assert!(
            clipboard_is("what the user had copied"),
            "the clipboard was not restored after Cmd+C"
        );

        assert!(paste_into(Target(front.pid as isize), "PASTED BY TONESNAP"));
        assert!(
            clipboard_is("what the user had copied"),
            "the clipboard was not restored after Cmd+V"
        );
    }
}
