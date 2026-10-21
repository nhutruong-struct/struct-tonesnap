//! Synthetic Cmd+key presses and the wait for the user's own modifier keys. Posting events
//! needs the Accessibility permission.

use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

use objc2::MainThreadMarker;

use super::cf::{CFTypeRef, Cf};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceCreate(state: i32) -> CFTypeRef;
    fn CGEventCreateKeyboardEvent(source: CFTypeRef, key: u16, down: u8) -> CFTypeRef;
    fn CGEventSetFlags(event: CFTypeRef, flags: u64);
    fn CGEventPost(tap: u32, event: CFTypeRef);
    fn CGEventSourceFlagsState(state: i32) -> u64;
}

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    static kTISPropertyUnicodeKeyLayoutData: CFTypeRef;
    fn TISCopyCurrentASCIICapableKeyboardLayoutInputSource() -> CFTypeRef;
    fn TISGetInputSourceProperty(source: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
    fn LMGetKbdType() -> u8;
    fn UCKeyTranslate(
        layout: *const u8,
        key_code: u16,
        key_action: u16,
        modifier_key_state: u32,
        keyboard_type: u32,
        options: u32,
        dead_key_state: *mut u32,
        max_len: usize,
        actual_len: *mut usize,
        chars: *mut u16,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFDataGetBytePtr(data: CFTypeRef) -> *const u8;
}

const KEY_ACTION_DISPLAY: u16 = 3;
const TRANSLATE_NO_DEAD_KEYS: u32 = 1;

const SOURCE_HID_SYSTEM_STATE: i32 = 1;
const SOURCE_COMBINED_SESSION_STATE: i32 = 0;
const TAP_HID: u32 = 0;

const FLAG_SHIFT: u64 = 0x0002_0000;
const FLAG_CONTROL: u64 = 0x0004_0000;
const FLAG_OPTION: u64 = 0x0008_0000;
const FLAG_COMMAND: u64 = 0x0010_0000;
const MODIFIERS: u64 = FLAG_SHIFT | FLAG_CONTROL | FLAG_OPTION | FLAG_COMMAND;

/// Virtual key codes of the keys labelled C and V on a US layout.
pub const KEY_C: u16 = 8;
pub const KEY_V: u16 = 9;

/// The key codes that type `c` and `v` on the current Latin layout, which is where macOS looks
/// for a Cmd shortcut's letter (Dvorak and AZERTY move them). `None` until first read.
static LAYOUT_KEYS: Mutex<Option<(u16, u16)>> = Mutex::new(None);

/// The key code for `letter`: the one `translate` says types it, else `fallback`.
fn find_key(translate: impl Fn(u16) -> Option<char>, letter: char, fallback: u16) -> u16 {
    (0..128u16)
        .find(|&code| translate(code) == Some(letter))
        .unwrap_or(fallback)
}

/// Reads where the layout puts C and V. The Text Input Sources calls may only run on the main
/// thread, so elsewhere this does nothing and the last reading (or the US codes) stays; it runs
/// at startup and before every capture, which both start on the main thread.
pub fn refresh_layout() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    // SAFETY: the Copy call returns an owned source (released by `Cf`); the property is the
    // layout data, which stays valid while the source lives, and `UCKeyTranslate` reads it with
    // buffers sized as it states.
    let keys = unsafe {
        let Some(source) = Cf::from_owned(TISCopyCurrentASCIICapableKeyboardLayoutInputSource())
        else {
            return;
        };
        let data = TISGetInputSourceProperty(source.as_ptr(), kTISPropertyUnicodeKeyLayoutData);
        if data.is_null() {
            return;
        }
        let layout = CFDataGetBytePtr(data);
        let kbd_type = u32::from(LMGetKbdType());
        let translate = |code: u16| {
            let mut dead = 0u32;
            let mut chars = [0u16; 4];
            let mut len = 0usize;
            let status = UCKeyTranslate(
                layout,
                code,
                KEY_ACTION_DISPLAY,
                0,
                kbd_type,
                TRANSLATE_NO_DEAD_KEYS,
                &mut dead,
                chars.len(),
                &mut len,
                chars.as_mut_ptr(),
            );
            (status == 0 && len == 1)
                .then(|| char::from_u32(u32::from(chars[0])))
                .flatten()
        };
        (
            find_key(translate, 'c', KEY_C),
            find_key(translate, 'v', KEY_V),
        )
    };
    *LAYOUT_KEYS.lock().unwrap() = Some(keys);
}

/// The key code to press for Cmd+C.
pub fn key_c() -> u16 {
    LAYOUT_KEYS.lock().unwrap().map_or(KEY_C, |(c, _)| c)
}

/// The key code to press for Cmd+V.
pub fn key_v() -> u16 {
    LAYOUT_KEYS.lock().unwrap().map_or(KEY_V, |(_, v)| v)
}

/// The user is usually still holding the hotkey's modifiers when we start; a synthetic Cmd+C
/// sent now would arrive as, for example, Ctrl+Option+Cmd+C. Wait for them to be released.
pub fn wait_modifiers_released(timeout: Duration) {
    let start = Instant::now();
    // SAFETY: the call takes a state id and reads the keyboard state.
    while unsafe { CGEventSourceFlagsState(SOURCE_COMBINED_SESSION_STATE) } & MODIFIERS != 0 {
        if start.elapsed() > timeout {
            return;
        }
        sleep(Duration::from_millis(15));
    }
}

/// Presses and releases Cmd+`key` in the app that has the keyboard focus.
pub fn send_command(key: u16) {
    // SAFETY: each create call returns an owned reference (or null, which `from_owned` turns
    // into `None`) that is released when its `Cf` drops, after the event was posted.
    unsafe {
        let Some(source) = Cf::from_owned(CGEventSourceCreate(SOURCE_HID_SYSTEM_STATE)) else {
            return;
        };
        for down in [1u8, 0] {
            let Some(event) =
                Cf::from_owned(CGEventCreateKeyboardEvent(source.as_ptr(), key, down))
            else {
                return;
            };
            CGEventSetFlags(event.as_ptr(), FLAG_COMMAND);
            CGEventPost(TAP_HID, event.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_is_the_one_that_types_the_letter_on_the_layout() {
        // On Dvorak the letter c sits where US keyboards have the i key (code 34).
        let dvorak = |code: u16| (code == 34).then_some('c');
        assert_eq!(find_key(dvorak, 'c', KEY_C), 34);
    }

    #[test]
    fn a_layout_without_the_letter_keeps_the_us_code() {
        assert_eq!(find_key(|_| None, 'v', KEY_V), KEY_V);
        assert_eq!(find_key(|_| Some('x'), 'v', KEY_V), KEY_V);
    }
}
