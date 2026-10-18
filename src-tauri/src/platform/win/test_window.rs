//! A real top-level Edit control for Win32 integration tests, owned by a thread that pumps its
//! messages (UI Automation and SendMessage need that). It never takes focus unless a test asks.

use std::sync::mpsc;
use std::thread::JoinHandle;

use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, GetWindowTextLengthW,
    GetWindowTextW, PostThreadMessageW, SendMessageW, ShowWindow, TranslateMessage, ES_AUTOHSCROLL,
    ES_MULTILINE, ES_PASSWORD, MSG, SW_SHOWNOACTIVATE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT,
    WS_BORDER, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

/// Defined here so the `Win32_UI_Controls` feature is not compiled in for one test constant.
const EM_SETSEL: u32 = 0x00B1;

#[derive(Clone, Copy, Default)]
pub struct Options {
    pub password: bool,
    /// Shown on screen and able to take focus (for tests that send keys); otherwise it sits
    /// off-screen and never activates.
    pub focusable: bool,
    /// Several lines, separated by `\r\n` in the text.
    pub multiline: bool,
}

pub struct TestWindow {
    pub hwnd: isize,
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

impl TestWindow {
    pub fn edit(text: &str, options: Options) -> Self {
        let (tx, rx) = mpsc::channel();
        let text = HSTRING::from(text);
        // SAFETY: the window is created, pumped and destroyed on this one thread; `text` lives
        // until the thread ends, so its pointer stays valid through CreateWindowExW.
        let thread = std::thread::spawn(move || unsafe {
            let module = GetModuleHandleW(None).ok();
            let mut style = WS_POPUP | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32);
            if options.password {
                style |= WINDOW_STYLE(ES_PASSWORD as u32);
            }
            if options.multiline {
                style |= WINDOW_STYLE(ES_MULTILINE as u32);
            }
            let (ex_style, x, y) = if options.focusable {
                (WS_EX_TOOLWINDOW, 200, 200)
            } else {
                (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, -32000, -32000)
            };
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(ex_style.0),
                w!("EDIT"),
                PCWSTR(text.as_ptr()),
                style,
                x,
                y,
                600,
                40,
                None,
                None,
                module.map(Into::into),
                None,
            )
            .expect("create the test Edit control");
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = tx.send((hwnd.0 as isize, GetCurrentThreadId()));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let _ = DestroyWindow(hwnd);
        });
        let (hwnd, thread_id) = rx.recv().expect("test window thread started");
        Self {
            hwnd,
            thread_id,
            thread: Some(thread),
        }
    }

    fn handle(&self) -> HWND {
        HWND(self.hwnd as *mut _)
    }

    /// Selects characters `start..end`.
    pub fn select(&self, start: usize, end: usize) {
        // SAFETY: EM_SETSEL carries two integers; the window stays alive while `self` exists.
        unsafe {
            SendMessageW(
                self.handle(),
                EM_SETSEL,
                Some(WPARAM(start)),
                Some(LPARAM(end as isize)),
            );
        }
    }

    pub fn text(&self) -> String {
        // SAFETY: `buf` holds the reported length plus the terminator, and GetWindowTextW writes
        // at most `buf.len()` characters.
        unsafe {
            let len = GetWindowTextLengthW(self.handle()).max(0) as usize;
            let mut buf = vec![0u16; len + 1];
            let copied = GetWindowTextW(self.handle(), &mut buf).max(0) as usize;
            String::from_utf16_lossy(&buf[..copied])
        }
    }
}

impl Drop for TestWindow {
    fn drop(&mut self) {
        // SAFETY: posting takes no pointers; the thread id is the window thread's own.
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
