pub mod clipboard;
mod com;
mod dispatch;
pub mod input;
pub mod ocr;
pub mod selection_icon;
#[cfg(test)]
mod test_window;
pub mod uia;
pub mod window;
pub mod word;

pub use clipboard::copy_selection;
pub use input::wait_modifiers_released;
pub use window::{cursor, monitor_at};

use super::{Accessible, Anchor, Foreground, InPlace, Target};
use crate::geometry::Rect;

/// Whether the foreground window belongs to ToneSnap (the popup or Settings).
pub fn own_window_in_front() -> Option<bool> {
    Some(window::is_own_window(window::foreground()))
}

/// UI Automation needs no permission on Windows.
pub fn has_accessibility() -> bool {
    true
}

/// The foreground window. Windows names no app, so the caller derives the name from the process.
pub fn foreground() -> Option<Foreground> {
    let hwnd = window::foreground();
    Some(Foreground {
        target: Target(hwnd),
        process: window::process_name(hwnd).unwrap_or_default(),
        name: None,
        own: window::is_own_window(hwnd),
    })
}

/// The focused control as UI Automation sees it, whichever window holds it. In a Word
/// document, the selection and its context come from Word itself, with the anchor Apply needs
/// to edit in place; the bounds still come from UI Automation.
pub fn read_focused(target: Target, context_chars: usize) -> Accessible {
    let ui = uia::capture(context_chars as i32);
    let mut focused = Accessible {
        selection: ui.selection,
        before: ui.before,
        after: ui.after,
        editable: ui.editable,
        bounds: ui.bounds,
        is_password: ui.is_password,
        anchor: None,
    };
    if window::process_name(target.0).as_deref() == Some("winword.exe") {
        if let Some(read) = word::read(target.0, context_chars) {
            focused.selection = read.selection().to_string();
            focused.before = read.before;
            focused.after = read.after;
            focused.anchor = Some(read.anchor);
        }
    }
    focused
}

/// Windows' OCR works in physical pixels, so the scale is not needed.
pub fn recognize_text(area: Rect, _scale: f64) -> Option<Vec<String>> {
    ocr::recognize(area).ok()
}

pub(super) fn is_in_front(target: Target) -> bool {
    window::foreground() == target.0
}

pub(super) fn bring_forward(target: Target) {
    window::focus(target.0);
}

pub(super) fn edit_in_place(anchor: &Anchor, text: &str) -> InPlace {
    match word::replace(anchor, text) {
        word::Outcome::Edited => InPlace::Edited,
        word::Outcome::Failed => InPlace::Failed,
        word::Outcome::Untouched => InPlace::Paste(anchor.pasted(text)),
    }
}

pub(super) fn send_paste() {
    input::send_ctrl(input::VK_V);
}

/// Tests that send real key presses and move focus. They run only on request
/// (`check.mjs --only=input`), never from hooks or `--full`: keys typed meanwhile could land in
/// another window.
///
/// Every key press is preceded by a check that the test window is still in front, and
/// assertions never print clipboard contents: if the user switches windows, a failure must not
/// reveal what another app had selected.
#[cfg(test)]
mod input_tests {
    use std::time::Duration;

    use super::test_window::{Options, TestWindow};
    use super::*;
    use crate::platform::paste_into;

    /// Puts the user's clipboard back however the test ends.
    struct KeepUserClipboard(clipboard::Snapshot);

    impl Drop for KeepUserClipboard {
        fn drop(&mut self) {
            clipboard::restore(&self.0);
        }
    }

    fn focused_window(text: &str) -> TestWindow {
        let window = TestWindow::edit(
            text,
            Options {
                focusable: true,
                ..Options::default()
            },
        );
        assert!(
            window::focus(window.hwnd),
            "could not focus the test window"
        );
        window
    }

    /// Stops the test before a key press if another window has come to the front.
    fn still_in_front(window: &TestWindow) {
        input::wait_modifiers_released(Duration::from_millis(2000));
        assert!(
            window::foreground() == window.hwnd,
            "another window came to the front; stopped before sending keys"
        );
    }

    fn clipboard_is(expected: &str) -> bool {
        clipboard::get_text().as_deref() == Some(expected)
    }

    #[test]
    #[ignore = "sends key presses and takes focus: run with check.mjs --only=input"]
    fn copies_the_selection_and_pastes_over_it_keeping_the_clipboard() {
        let _user = KeepUserClipboard(clipboard::snapshot());
        assert!(clipboard::set_text("what the user had copied", true));
        let window = focused_window("Hello world");
        window.select(6, 11);

        still_in_front(&window);
        let copied = clipboard::copy_selection();
        assert!(
            copied.as_deref() == Some("world"),
            "Ctrl+C did not return the selection"
        );
        assert!(
            clipboard_is("what the user had copied"),
            "the clipboard was not restored after Ctrl+C"
        );

        still_in_front(&window);
        assert!(
            paste_into(Target(window.hwnd), "there"),
            "paste_into failed"
        );
        assert!(
            window.text() == "Hello there",
            "Ctrl+V did not replace the selection"
        );
        assert!(
            clipboard_is("what the user had copied"),
            "the clipboard was not restored after Ctrl+V"
        );
    }

    #[test]
    #[ignore = "sends key presses and takes focus: run with check.mjs --only=input"]
    fn nothing_is_copied_without_a_selection() {
        let _user = KeepUserClipboard(clipboard::snapshot());
        let window = focused_window("No selection");
        window.select(2, 2);
        still_in_front(&window);
        assert!(
            clipboard::copy_selection().is_none(),
            "Ctrl+C returned text without a selection"
        );
    }
}
