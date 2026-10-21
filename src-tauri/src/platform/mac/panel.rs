//! The popup as a non-activating panel: it takes keyboard input without making ToneSnap the
//! active app, so the app the user selected text in keeps its caret and selection, and it can
//! float over full-screen apps.

use std::ffi::c_void;
use std::sync::OnceLock;

use objc2::ffi::object_setClass;
use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
use objc2::{class, msg_send, sel, MainThreadMarker};
use objc2_app_kit::NSApplication;

/// `NSWindowStyleMaskNonactivatingPanel`
const STYLE_NONACTIVATING_PANEL: usize = 1 << 7;
/// `NSWindowCollectionBehaviorCanJoinAllSpaces | FullScreenAuxiliary`
const COLLECTION_BEHAVIOR: usize = (1 << 0) | (1 << 8);
/// `NSStatusWindowLevel`: above ordinary floating windows and over full-screen apps.
const LEVEL_STATUS: isize = 25;

extern "C" fn yes(_this: &AnyObject, _sel: Sel) -> Bool {
    Bool::YES
}

extern "C" fn no(_this: &AnyObject, _sel: Sel) -> Bool {
    Bool::NO
}

/// A panel class that can take the keyboard (the stock one refuses when it has no title bar).
fn panel_class() -> Option<&'static AnyClass> {
    static CLASS: OnceLock<Option<&'static AnyClass>> = OnceLock::new();
    *CLASS.get_or_init(|| {
        let mut builder = ClassBuilder::new(c"ToneSnapPopupPanel", class!(NSPanel))?;
        // SAFETY: both methods are declared `-(BOOL)name` on NSWindow, which `yes` and `no`
        // match in arguments and return type.
        unsafe {
            builder.add_method(sel!(canBecomeKeyWindow), yes as extern "C" fn(_, _) -> _);
            builder.add_method(sel!(canBecomeMainWindow), no as extern "C" fn(_, _) -> _);
        }
        Some(builder.register())
    })
}

/// Turns the Tauri window at `ns_window` into a floating, non-activating panel. Main thread.
///
/// The window's class is swapped in place for a subclass of `NSPanel`. That is only done when
/// the new class needs no more memory than the window already has.
pub fn make_panel(ns_window: *mut c_void) -> bool {
    let Some(class) = panel_class() else {
        return false;
    };
    let object = ns_window.cast::<AnyObject>();
    // SAFETY: Tauri hands out the pointer of a live NSWindow, and this runs on the main thread
    // that owns it. The size check keeps the swapped class within the object's allocation; the
    // calls after the swap are NSWindow/NSPanel setters with matching argument types.
    unsafe {
        let Some(object_ref) = object.as_ref() else {
            return false;
        };
        if class.instance_size() > object_ref.class().instance_size() {
            return false;
        }
        object_setClass(object, class);
        let mask: usize = msg_send![object, styleMask];
        let () = msg_send![object, setStyleMask: mask | STYLE_NONACTIVATING_PANEL];
        let () = msg_send![object, setFloatingPanel: true];
        let () = msg_send![object, setBecomesKeyOnlyIfNeeded: false];
        let () = msg_send![object, setHidesOnDeactivate: false];
        let () = msg_send![object, setCollectionBehavior: COLLECTION_BEHAVIOR];
        let () = msg_send![object, setLevel: LEVEL_STATUS];
    }
    true
}

/// Makes the panel the key window without activating the app. Main thread.
pub fn focus_panel(ns_window: *mut c_void) {
    // SAFETY: as in `make_panel`; `makeKeyAndOrderFront:` takes a nullable sender.
    unsafe {
        let object = ns_window.cast::<AnyObject>();
        let () = msg_send![object, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
    }
}

/// Whether one of ToneSnap's own windows has the keyboard focus.
pub fn has_key_window(mtm: MainThreadMarker) -> bool {
    NSApplication::sharedApplication(mtm).keyWindow().is_some()
}
