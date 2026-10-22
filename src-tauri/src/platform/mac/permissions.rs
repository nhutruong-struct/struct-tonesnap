//! The two permissions macOS makes the user grant: Accessibility (read the selection, press
//! keys) and Screen Recording (OCR). Neither can be granted by the app itself; it can only ask
//! the system to show its prompt and open the matching pane of System Settings.

use std::ffi::c_void;
use std::process::Command;

use super::cf::{CFTypeRef, Cf};
use super::{ax, ocr};
use crate::platform::{Permission, Permissions};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: CFTypeRef;
    fn AXIsProcessTrustedWithOptions(options: CFTypeRef) -> u8;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    fn CFDictionaryCreate(
        allocator: CFTypeRef,
        keys: *const CFTypeRef,
        values: *const CFTypeRef,
        count: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFTypeRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGRequestScreenCaptureAccess() -> u8;
}

pub fn permissions() -> Permissions {
    Permissions {
        required: true,
        accessibility: ax::is_trusted(),
        screen_recording: ocr::screen_capture_allowed(),
    }
}

/// Shows the system's prompt for `permission`. The system shows it once per app; after a
/// refusal the user has to use System Settings, so callers also offer `open_settings`.
pub fn request(permission: Permission) {
    match permission {
        Permission::Accessibility => {
            // SAFETY: the key is the framework's constant, the value the shared `true`; the
            // dictionary retains both with the standard callbacks and is released when `options`
            // drops, after the call.
            unsafe {
                let keys = [kAXTrustedCheckOptionPrompt];
                let values = [super::cf::boolean_true()];
                let options = Cf::from_owned(CFDictionaryCreate(
                    std::ptr::null(),
                    keys.as_ptr(),
                    values.as_ptr(),
                    1,
                    &raw const kCFTypeDictionaryKeyCallBacks,
                    &raw const kCFTypeDictionaryValueCallBacks,
                ));
                if let Some(options) = options {
                    AXIsProcessTrustedWithOptions(options.as_ptr());
                }
            }
        }
        Permission::ScreenRecording => {
            // SAFETY: takes no arguments; shows the system prompt when the choice is still open.
            unsafe { CGRequestScreenCaptureAccess() };
        }
    }
}

pub fn open_settings(permission: Permission) {
    let pane = match permission {
        Permission::Accessibility => "Privacy_Accessibility",
        Permission::ScreenRecording => "Privacy_ScreenCapture",
    };
    let url = format!("x-apple.systempreferences:com.apple.preference.security?{pane}");
    let _ = Command::new("open").arg(url).spawn();
}
