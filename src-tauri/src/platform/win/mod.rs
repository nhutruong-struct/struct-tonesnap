pub mod clipboard;
mod com;
pub mod input;
pub mod ocr;
pub mod selection_icon;
#[cfg(test)]
mod test_window;
pub mod uia;
pub mod window;

use std::thread::sleep;
use std::time::{Duration, Instant};

pub use window::{cursor, monitor_at};

/// Whether the foreground window belongs to ToneSnap (the popup or Settings).
pub fn own_window_in_front() -> Option<bool> {
    Some(window::is_own_window(window::foreground()))
}

/// Puts `text` on the clipboard for the user to paste wherever they like.
pub fn copy_text(text: &str) {
    clipboard::set_text(text, false);
}

/// Brings `hwnd` back to the foreground and pastes `text` over its selection, then restores
/// the user's clipboard. Returns false when the target window could not be focused.
pub fn paste_into(hwnd: isize, text: &str) -> bool {
    let start = Instant::now();
    while window::foreground() != hwnd {
        if start.elapsed() > Duration::from_millis(800) {
            return false;
        }
        window::focus(hwnd);
        sleep(Duration::from_millis(40));
    }
    // Give the app a moment to restore its caret and selection.
    sleep(Duration::from_millis(80));

    let snapshot = clipboard::snapshot();
    if !clipboard::set_text(text, true) {
        return false;
    }
    input::wait_modifiers_released(Duration::from_millis(400));
    input::send_ctrl(input::VK_V);
    // Electron apps read the clipboard asynchronously; restoring too early pastes the old content.
    sleep(Duration::from_millis(500));
    clipboard::restore(&snapshot);
    true
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
    use super::test_window::{Options, TestWindow};
    use super::*;

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
        assert!(paste_into(window.hwnd, "there"), "paste_into failed");
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
