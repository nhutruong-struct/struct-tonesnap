//! Floating ToneSnap icon shown next to a fresh text selection ("Icon" trigger mode).
//!
//! A low-level mouse hook spots selection gestures: a drag or a double/triple click while the
//! cursor is the text I-beam. UI Automation then vetoes the obvious misses (nothing selected,
//! password fields). The icon is a native layered window that never takes focus, so the app
//! keeps its selection and clicking the icon behaves exactly like pressing the hotkey. In Auto
//! mode the confirmed selection is first offered to `set_on_selection`, which may open the
//! popup instead. Only the mouse is hooked; the keyboard is never watched.

use std::cell::RefCell;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
    GetCursorInfo, GetMessageW, GetSystemMetrics, KillTimer, LoadCursorW, PostMessageW,
    PostThreadMessageW, RegisterClassW, SetTimer, SetWindowPos, SetWindowsHookExW, ShowWindow,
    TranslateMessage, UnhookWindowsHookEx, CURSORINFO, HWND_TOPMOST, IDC_HAND, IDC_IBEAM,
    MA_NOACTIVATE, MSG, MSLLHOOKSTRUCT, SM_CXDOUBLECLK, SM_CYDOUBLECLK, SWP_NOACTIVATE,
    SWP_SHOWWINDOW, SW_HIDE, WH_MOUSE_LL, WM_APP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
    WM_MOUSEACTIVATE, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_TIMER,
    WM_XBUTTONDOWN, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

mod paint;

use super::uia::{self, SelectionState};
use super::window;
use crate::geometry::Rect;
use crate::platform::gesture::{DoubleClick, Gesture};
use crate::platform::{note_press, presses, Selection};

/// Posted by the hook after a selection gesture; wParam is `presses()` at the release, lParam
/// packs the cursor position.
const WM_CANDIDATE: u32 = WM_APP + 1;
/// Posted by the checker thread once the selection is confirmed.
const WM_SHOW_ICON: u32 = WM_APP + 2;
const WM_HIDE_ICON: u32 = WM_APP + 3;
const TIMER_ID: usize = 1;

const ICON_SIZE: f64 = 26.0;
/// Hidden again after this long if the user ignores it.
const IDLE_TIMEOUT: Duration = Duration::from_secs(6);

static THREAD_ID: Mutex<Option<u32>> = Mutex::new(None);
static ICON_HWND: AtomicIsize = AtomicIsize::new(0);
static ON_CLICK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static IMAGE: OnceLock<(Vec<u8>, u32, u32)> = OnceLock::new();
type AppFilter = Box<dyn Fn(&str) -> bool + Send + Sync>;
static APP_FILTER: OnceLock<AppFilter> = OnceLock::new();
type SelectionHandler = Box<dyn Fn(&Selection) -> bool + Send + Sync>;
static ON_SELECTION: OnceLock<SelectionHandler> = OnceLock::new();

/// Screen rectangle of the visible icon, so the hook can ignore clicks on it.
static VISIBLE: Mutex<Option<Visible>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct Visible {
    rect: Rect,
    /// The window that owned the selection; switching away hides the icon.
    owner: isize,
    shown_at: Instant,
}

thread_local! {
    static GESTURE: RefCell<Gesture> = RefCell::new(Gesture::default());
}

/// RGBA pixels of the icon to draw (any size; it is resampled per monitor DPI).
pub fn set_image(rgba: Vec<u8>, width: u32, height: u32) {
    let _ = IMAGE.set((rgba, width, height));
}

/// Called (on the hook thread) when the user clicks the icon.
pub fn set_on_click(f: impl Fn() + Send + Sync + 'static) {
    let _ = ON_CLICK.set(Box::new(f));
}

/// Called (on a worker thread, may block) with each confirmed selection before the icon shows.
/// True means it was handled and the icon stays hidden.
pub fn set_on_selection(handled: impl Fn(&Selection) -> bool + Send + Sync + 'static) {
    let _ = ON_SELECTION.set(Box::new(handled));
}

/// Decides, from the lower-case process name of the app holding the selection, whether the
/// icon may appear there. Asked on every selection, so it can follow changing settings.
pub fn set_app_filter(allowed: impl Fn(&str) -> bool + Send + Sync + 'static) {
    let _ = APP_FILTER.set(Box::new(allowed));
}

pub fn start() {
    let mut thread = THREAD_ID.lock().unwrap();
    if thread.is_some() {
        return;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    // SAFETY: `run` owns the new thread: it creates the window and hook there and releases both
    // before the thread ends.
    std::thread::spawn(move || unsafe { run(tx) });
    *thread = rx.recv().ok().flatten();
}

pub fn stop() {
    if let Some(id) = THREAD_ID.lock().unwrap().take() {
        // SAFETY: posting takes no pointers; a thread that already ended just drops the message.
        unsafe {
            let _ = PostThreadMessageW(id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

/// Hides the icon if it is showing (e.g. when the hotkey is used instead).
pub fn hide() {
    post(WM_HIDE_ICON, WPARAM(0), LPARAM(0));
}

fn post(msg: u32, wparam: WPARAM, lparam: LPARAM) {
    let hwnd = ICON_HWND.load(Ordering::SeqCst);
    if hwnd != 0 {
        // SAFETY: posting takes no pointers; if the window was just destroyed the call fails
        // harmlessly.
        unsafe {
            let _ = PostMessageW(Some(HWND(hwnd as *mut _)), msg, wparam, lparam);
        }
    }
}

fn pack(pt: POINT) -> LPARAM {
    LPARAM(((pt.x as u32 as u64) | ((pt.y as u32 as u64) << 32)) as isize)
}

fn unpack(lparam: LPARAM) -> POINT {
    let v = lparam.0 as u64;
    POINT {
        x: v as u32 as i32,
        y: (v >> 32) as u32 as i32,
    }
}

fn double_click() -> DoubleClick {
    // SAFETY: both calls only read system settings.
    unsafe {
        DoubleClick {
            time: Duration::from_millis(GetDoubleClickTime() as u64),
            dx: GetSystemMetrics(SM_CXDOUBLECLK),
            dy: GetSystemMetrics(SM_CYDOUBLECLK),
        }
    }
}

/// Whether the cursor is the text I-beam, i.e. the pointer is over selectable text.
fn cursor_is_ibeam() -> bool {
    // SAFETY: `info` is a local with `cbSize` set, as GetCursorInfo requires; the shared system
    // cursor from LoadCursorW must not be freed, and is not.
    unsafe {
        let mut info = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        let Ok(ibeam) = LoadCursorW(None, IDC_IBEAM) else {
            return false;
        };
        GetCursorInfo(&mut info).is_ok() && info.hCursor == ibeam
    }
}

unsafe fn run(ready: std::sync::mpsc::Sender<Option<u32>>) {
    let Ok(module) = GetModuleHandleW(None) else {
        let _ = ready.send(None);
        return;
    };
    let class = WNDCLASSW {
        lpfnWndProc: Some(wnd_proc),
        hInstance: module.into(),
        lpszClassName: w!("ToneSnapSelectionIcon"),
        hCursor: LoadCursorW(None, IDC_HAND).unwrap_or_default(),
        ..Default::default()
    };
    // Fails harmlessly with "class already exists" when the mode is toggled again.
    RegisterClassW(&class);
    let Ok(hwnd) = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
        w!("ToneSnapSelectionIcon"),
        w!("ToneSnap"),
        WS_POPUP,
        0,
        0,
        1,
        1,
        None,
        None,
        Some(module.into()),
        None,
    ) else {
        let _ = ready.send(None);
        return;
    };
    ICON_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
    let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(module.into()), 0).ok();
    let _ = ready.send(Some(GetCurrentThreadId()));

    let mut msg = MSG::default();
    while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    if let Some(hook) = hook {
        let _ = UnhookWindowsHookEx(hook);
    }
    *VISIBLE.lock().unwrap() = None;
    ICON_HWND.store(0, Ordering::SeqCst);
    let _ = DestroyWindow(hwnd);
}

/// Must return quickly: Windows drops low-level hooks that stall input. Real work is posted
/// to the icon window and done off the hook path.
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let pt = (*(lparam.0 as *const MSLLHOOKSTRUCT)).pt;
        let on_icon = VISIBLE
            .lock()
            .unwrap()
            .is_some_and(|v| v.rect.contains(pt.x, pt.y));
        match wparam.0 as u32 {
            WM_LBUTTONDOWN => {
                note_press();
                if !on_icon {
                    hide();
                }
                let ibeam = !on_icon && cursor_is_ibeam();
                GESTURE.with_borrow_mut(|g| {
                    g.press((pt.x, pt.y), Instant::now(), on_icon, ibeam, double_click())
                });
            }
            WM_LBUTTONUP => {
                let selected = GESTURE
                    .with_borrow_mut(|g| g.release((pt.x, pt.y), Instant::now(), cursor_is_ibeam));
                if selected {
                    // Counted here: a press can be handled before WM_CANDIDATE is.
                    post(WM_CANDIDATE, WPARAM(presses() as usize), pack(pt));
                }
            }
            WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_MOUSEWHEEL => {
                note_press();
                if !on_icon {
                    hide();
                }
            }
            WM_XBUTTONDOWN | WM_MOUSEHWHEEL => note_press(),
            _ => {}
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// Runs off the hook thread: UI Automation calls can take a while. `at` is where the gesture
/// ended. `None` when the icon must not show; otherwise `Selected` or `Unknown`.
fn confirmed_state(at: POINT) -> Option<SelectionState> {
    let fg = window::foreground();
    if fg == 0 || window::is_own_window(fg) {
        return None;
    }
    let process = window::process_name(fg).unwrap_or_default();
    if APP_FILTER.get().is_some_and(|allowed| !allowed(&process)) {
        return None;
    }
    match uia::selection_state((at.x, at.y)) {
        SelectionState::Empty | SelectionState::Password => None,
        state => Some(state),
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CANDIDATE => {
            // A later press cancels this selection anyway, so the settle time read now is still
            // this gesture's whenever it matters.
            let settle = GESTURE.with_borrow(|g| g.settle_time(double_click()));
            let mut selection = Selection::new(Instant::now(), settle, wparam.0 as u64);
            std::thread::spawn(move || {
                // Let the app finish updating its selection first.
                std::thread::sleep(Duration::from_millis(120));
                let Some(state) = confirmed_state(unpack(lparam)) else {
                    return;
                };
                selection.reported = state == SelectionState::Selected;
                if !ON_SELECTION
                    .get()
                    .is_some_and(|handled| handled(&selection))
                {
                    post(WM_SHOW_ICON, WPARAM(0), lparam);
                }
            });
            LRESULT(0)
        }
        WM_SHOW_ICON => {
            show(hwnd, unpack(lparam));
            LRESULT(0)
        }
        WM_HIDE_ICON => {
            hide_now(hwnd);
            LRESULT(0)
        }
        WM_TIMER => {
            let visible = *VISIBLE.lock().unwrap();
            if let Some(v) = visible {
                let (cx, cy) = window::cursor();
                let hovered = v.rect.contains(cx, cy);
                if window::foreground() != v.owner
                    || (!hovered && v.shown_at.elapsed() > IDLE_TIMEOUT)
                {
                    hide_now(hwnd);
                }
            }
            LRESULT(0)
        }
        // Clicking must not activate the icon, or the app would lose its selection.
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_LBUTTONDOWN => LRESULT(0),
        WM_LBUTTONUP => {
            hide_now(hwnd);
            if let Some(on_click) = ON_CLICK.get() {
                on_click();
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn hide_now(hwnd: HWND) {
    if VISIBLE.lock().unwrap().take().is_some() {
        let _ = KillTimer(Some(hwnd), TIMER_ID);
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/// Shows the icon just below and to the right of where the mouse was released.
unsafe fn show(hwnd: HWND, at: POINT) {
    let Some((rgba, width, height)) = IMAGE.get() else {
        return;
    };
    let monitor = window::monitor_at(at.x, at.y);
    let (work, scale) = (monitor.work, monitor.scale);
    let size = (ICON_SIZE * scale).round() as i32;
    let x = (at.x + (10.0 * scale) as i32).clamp(work.left, work.right - size);
    let y = (at.y + (14.0 * scale) as i32).clamp(work.top, work.bottom - size);

    let pixels = paint::resample_premultiplied_bgra(rgba, *width, *height, size as u32);
    if !paint::paint(hwnd, &pixels, size, POINT { x, y }) {
        return;
    }
    let _ = SetWindowPos(
        hwnd,
        Some(HWND_TOPMOST),
        x,
        y,
        size,
        size,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    *VISIBLE.lock().unwrap() = Some(Visible {
        rect: Rect {
            left: x,
            top: y,
            right: x + size,
            bottom: y + size,
        },
        owner: window::foreground(),
        shown_at: Instant::now(),
    });
    SetTimer(Some(hwnd), TIMER_ID, 150, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_round_trips_through_lparam() {
        for pt in [
            POINT { x: 0, y: 0 },
            POINT { x: -1920, y: 1080 },
            POINT { x: 3840, y: -5 },
        ] {
            let back = unpack(pack(pt));
            assert_eq!((back.x, back.y), (pt.x, pt.y));
        }
    }
}
