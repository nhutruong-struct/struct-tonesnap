use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::System::Threading::{
    AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId, OpenProcess,
    QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, IsIconic,
    IsWindow, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

use crate::geometry::Rect;
use crate::platform::Monitor;

impl From<RECT> for Rect {
    fn from(r: RECT) -> Self {
        Self {
            left: r.left,
            top: r.top,
            right: r.right,
            bottom: r.bottom,
        }
    }
}

fn hwnd(raw: isize) -> HWND {
    HWND(raw as *mut _)
}

pub fn foreground() -> isize {
    // SAFETY: GetForegroundWindow takes no arguments.
    unsafe { GetForegroundWindow().0 as isize }
}

pub fn is_own_window(raw: isize) -> bool {
    let mut pid = 0u32;
    // SAFETY: `pid` is a local the call writes; a stale or null window just leaves it at 0.
    unsafe {
        GetWindowThreadProcessId(hwnd(raw), Some(&mut pid));
        pid == GetCurrentProcessId()
    }
}

/// Lower-case executable name of the process that owns the window, e.g. `winword.exe`.
pub fn process_name(raw: isize) -> Option<String> {
    // SAFETY: `buf` and `len` describe the same local buffer, and the call never writes past
    // `len`; the process handle is closed on every path after it was opened.
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd(raw), Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().map(str::to_lowercase)
    }
}

pub fn cursor() -> (i32, i32) {
    let mut pt = POINT::default();
    // SAFETY: `pt` is a local the call writes.
    unsafe {
        let _ = GetCursorPos(&mut pt);
    }
    (pt.x, pt.y)
}

/// The monitor nearest to a point.
pub fn monitor_at(x: i32, y: i32) -> Monitor {
    // SAFETY: MONITOR_DEFAULTTONEAREST always yields a monitor; `info` has `cbSize` set and the
    // DPI outputs are locals.
    unsafe {
        let monitor = MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let (mut dpi_x, mut dpi_y) = (96u32, 96u32);
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
        Monitor {
            work: info.rcWork.into(),
            bounds: info.rcMonitor.into(),
            scale: dpi_x as f64 / 96.0,
        }
    }
}

/// Makes `raw` the foreground window. Windows only allows this for the process that received
/// the last input, so fall back to attaching to the current foreground thread's input queue.
pub fn focus(raw: isize) -> bool {
    let target = hwnd(raw);
    // SAFETY: the calls take window handles and thread ids by value (a stale handle makes them
    // fail, nothing more); an attached input queue is detached again before returning.
    unsafe {
        if !IsWindow(Some(target)).as_bool() {
            return false;
        }
        if IsIconic(target).as_bool() {
            let _ = ShowWindow(target, SW_RESTORE);
        }
        if SetForegroundWindow(target).as_bool() {
            return true;
        }
        let fg_thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
        let me = GetCurrentThreadId();
        let attached = fg_thread != me && AttachThreadInput(me, fg_thread, true).as_bool();
        let ok = SetForegroundWindow(target).as_bool();
        let _ = BringWindowToTop(target);
        if attached {
            let _ = AttachThreadInput(me, fg_thread, false);
        }
        ok
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::test_window::{Options, TestWindow};
    use super::*;

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn knows_its_own_windows_and_process() {
        let window = TestWindow::edit("x", Options::default());
        assert!(is_own_window(window.hwnd));
        let exe = std::env::current_exe().unwrap();
        let exe = exe.file_name().unwrap().to_string_lossy().to_lowercase();
        assert_eq!(process_name(window.hwnd).as_deref(), Some(exe.as_str()));
        assert_eq!(process_name(0), None);
        assert!(!focus(0));
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn monitors_have_a_work_area_inside_their_bounds() {
        let (x, y) = cursor();
        let monitor = monitor_at(x, y);
        assert!(monitor.scale >= 1.0);
        assert!(monitor.work.width() > 0 && monitor.work.height() > 0);
        assert_eq!(monitor.work.intersect(monitor.bounds), monitor.work);
        // Points far off every screen still map to the nearest monitor.
        assert!(monitor_at(-100_000, -100_000).work.width() > 0);
    }
}
