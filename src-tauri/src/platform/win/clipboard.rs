//! Clipboard snapshot/restore so reading the selection (Ctrl+C) and applying a suggestion
//! (Ctrl+V) leave the user's clipboard as it was.

use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
    GetClipboardSequenceNumber, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use super::input;

pub struct Snapshot(Vec<(u32, Vec<u8>)>);

/// Formats whose handle is not an HGLOBAL (GDI objects, metafiles, owner-drawn, private ranges).
/// Bitmaps survive through CF_DIB / CF_DIBV5, which are HGLOBALs.
fn is_copyable(format: u32) -> bool {
    !matches!(format, 2 | 3 | 9 | 14 | 0x80..=0x8F | 0x200..=0x3FF)
}

struct Opened;

impl Opened {
    fn open() -> Option<Self> {
        for _ in 0..20 {
            // SAFETY: OpenClipboard takes no pointers; a failed call leaves nothing to release.
            if unsafe { OpenClipboard(None) }.is_ok() {
                return Some(Opened);
            }
            sleep(Duration::from_millis(15));
        }
        None
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        // SAFETY: an `Opened` exists only after OpenClipboard succeeded on this thread, so this
        // close is balanced.
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

unsafe fn read_global(handle: HANDLE) -> Option<Vec<u8>> {
    let global = HGLOBAL(handle.0);
    let size = GlobalSize(global);
    if size == 0 {
        return None;
    }
    let ptr = GlobalLock(global) as *const u8;
    if ptr.is_null() {
        return None;
    }
    let bytes = std::slice::from_raw_parts(ptr, size).to_vec();
    let _ = GlobalUnlock(global);
    Some(bytes)
}

unsafe fn write_global(format: u32, data: &[u8]) -> bool {
    let Ok(global) = GlobalAlloc(GMEM_MOVEABLE, data.len().max(1)) else {
        return false;
    };
    let ptr = GlobalLock(global) as *mut u8;
    if ptr.is_null() {
        let _ = GlobalFree(Some(global));
        return false;
    }
    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
    let _ = GlobalUnlock(global);
    if SetClipboardData(format, Some(HANDLE(global.0))).is_err() {
        let _ = GlobalFree(Some(global));
        return false;
    }
    true
}

pub fn snapshot() -> Snapshot {
    let mut items = Vec::new();
    let Some(_guard) = Opened::open() else {
        return Snapshot(items);
    };
    // SAFETY: the clipboard is open (`_guard`), so the handles GetClipboardData returns stay
    // valid until it closes; `is_copyable` keeps only HGLOBAL formats, which `read_global` needs.
    unsafe {
        let mut format = 0u32;
        loop {
            format = EnumClipboardFormats(format);
            if format == 0 {
                break;
            }
            if !is_copyable(format) {
                continue;
            }
            if let Ok(handle) = GetClipboardData(format) {
                if let Some(bytes) = read_global(handle) {
                    items.push((format, bytes));
                }
            }
        }
    }
    Snapshot(items)
}

pub fn restore(snapshot: &Snapshot) {
    let Some(_guard) = Opened::open() else { return };
    // SAFETY: the clipboard is open (`_guard`), as EmptyClipboard and SetClipboardData require.
    unsafe {
        let _ = EmptyClipboard();
        for (format, bytes) in &snapshot.0 {
            write_global(*format, bytes);
        }
    }
}

/// Puts `text` on the clipboard. `transient` keeps it out of clipboard history and cloud sync,
/// for content that only exists long enough to be pasted.
pub fn set_text(text: &str, transient: bool) -> bool {
    let Some(_guard) = Opened::open() else {
        return false;
    };
    // SAFETY: the clipboard is open (`_guard`). The byte view covers exactly the bytes of
    // `wide`, which outlives it; `w!` strings are static and NUL-terminated.
    unsafe {
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2);
        let ok = write_global(CF_UNICODETEXT.0 as u32, bytes);
        if transient {
            for name in [
                w!("ExcludeClipboardContentFromMonitorProcessing"),
                w!("CanIncludeInClipboardHistory"),
                w!("CanUploadToCloudClipboard"),
            ] {
                let format = RegisterClipboardFormatW(name);
                if format != 0 {
                    write_global(format, &0u32.to_le_bytes());
                }
            }
        }
        ok
    }
}

pub fn get_text() -> Option<String> {
    let _guard = Opened::open()?;
    // SAFETY: the clipboard is open (`_guard`) and CF_UNICODETEXT data is an HGLOBAL, as
    // `read_global` needs.
    unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32).ok()?;
        let bytes = read_global(handle)?;
        let wide: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
        Some(String::from_utf16_lossy(&wide[..end]))
    }
}

/// Copies the foreground app's selection with a synthetic Ctrl+C and returns it, leaving the
/// clipboard as it was. `None` when nothing was copied (no selection, or the app ignored Ctrl+C).
pub fn copy_selection() -> Option<String> {
    // SAFETY: GetClipboardSequenceNumber takes no arguments.
    let before = unsafe { GetClipboardSequenceNumber() };
    let snapshot = snapshot();
    input::send_ctrl(input::VK_C);

    let start = Instant::now();
    let mut changed = false;
    while start.elapsed() < Duration::from_millis(450) {
        sleep(Duration::from_millis(15));
        // SAFETY: GetClipboardSequenceNumber takes no arguments.
        if unsafe { GetClipboardSequenceNumber() } != before {
            changed = true;
            break;
        }
    }
    if !changed {
        return None;
    }
    // Some apps publish several formats one after another; let them finish.
    sleep(Duration::from_millis(40));
    let text = get_text();
    restore(&snapshot);
    text.filter(|t| !t.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_that_are_not_memory_are_skipped() {
        for format in [2, 3, 9, 14, 0x80, 0x8F, 0x200, 0x3FF] {
            assert!(!is_copyable(format), "format {format}");
        }
        for format in [1, 8, 13, 17, 0xC000] {
            assert!(is_copyable(format), "format {format}");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Puts the user's clipboard back however the test ends.
    struct KeepUserClipboard(Snapshot);

    impl Drop for KeepUserClipboard {
        fn drop(&mut self) {
            restore(&self.0);
        }
    }

    fn has_format(snapshot: &Snapshot, name: windows::core::PCWSTR) -> bool {
        // SAFETY: `name` comes from `w!`, a static NUL-terminated string.
        let format = unsafe { RegisterClipboardFormatW(name) };
        snapshot.0.iter().any(|(f, _)| *f == format)
    }

    fn clipboard_is(expected: &str) -> bool {
        get_text().as_deref() == Some(expected)
    }

    /// Everything this test writes is transient (kept out of Win+V history and the cloud
    /// clipboard), and failures never print the clipboard: the user may copy something
    /// meanwhile.
    #[test]
    #[ignore = "win32 integration: run by check.mjs --full (briefly replaces the clipboard)"]
    fn text_round_trips_and_a_snapshot_restores_every_format() {
        let _user = KeepUserClipboard(snapshot());

        assert!(set_text("ToneSnap clipboard test ✓", true));
        assert!(
            clipboard_is("ToneSnap clipboard test ✓"),
            "text did not round-trip"
        );
        let saved = snapshot();
        assert!(has_format(&saved, w!("CanIncludeInClipboardHistory")));
        assert!(has_format(&saved, w!("CanUploadToCloudClipboard")));
        assert!(has_format(
            &saved,
            w!("ExcludeClipboardContentFromMonitorProcessing")
        ));

        assert!(set_text("ToneSnap clipboard test, second", true));
        restore(&saved);
        assert!(
            clipboard_is("ToneSnap clipboard test ✓"),
            "restore did not bring the text back"
        );
        assert!(has_format(&snapshot(), w!("CanIncludeInClipboardHistory")));
    }
}
