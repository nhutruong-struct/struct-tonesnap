use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

pub const VK_C: VIRTUAL_KEY = VIRTUAL_KEY(0x43);
pub const VK_V: VIRTUAL_KEY = VIRTUAL_KEY(0x56);

/// The user is usually still holding the hotkey's modifiers when we start; a synthetic Ctrl+C
/// sent now would arrive as e.g. Ctrl+Alt+C. Wait for them to be released.
pub fn wait_modifiers_released(timeout: Duration) {
    let keys = [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN];
    let start = Instant::now();
    while start.elapsed() < timeout {
        // SAFETY: GetAsyncKeyState only reads key state for a virtual-key code.
        let held = keys
            .iter()
            .any(|k| unsafe { GetAsyncKeyState(k.0 as i32) } as u16 & 0x8000 != 0);
        if !held {
            return;
        }
        sleep(Duration::from_millis(10));
    }
}

fn key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Sends Ctrl+`vk` to the foreground window.
pub fn send_ctrl(vk: VIRTUAL_KEY) {
    let inputs = [
        key(VK_CONTROL, false),
        key(vk, false),
        key(vk, true),
        key(VK_CONTROL, true),
    ];
    // SAFETY: `inputs` is a live slice of fully initialized keyboard INPUTs, and the size passed
    // is that of one INPUT, as SendInput expects.
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
