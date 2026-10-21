//! Synthetic Cmd+key presses and the wait for the user's own modifier keys. Posting events
//! needs the Accessibility permission.

use std::thread::sleep;
use std::time::{Duration, Instant};

use super::cf::{CFTypeRef, Cf};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventSourceCreate(state: i32) -> CFTypeRef;
    fn CGEventCreateKeyboardEvent(source: CFTypeRef, key: u16, down: u8) -> CFTypeRef;
    fn CGEventSetFlags(event: CFTypeRef, flags: u64);
    fn CGEventPost(tap: u32, event: CFTypeRef);
    fn CGEventSourceFlagsState(state: i32) -> u64;
}

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
