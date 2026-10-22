//! The OS integration behind one platform-neutral surface. Orchestration code calls these
//! functions; on Windows they forward to `win`, on macOS to `mac`, elsewhere to inert stubs.
//! `win` and `mac` are private, so nothing outside this module can bypass the surface.
//! A function returning `Option` answers `None` where the platform cannot tell, so the caller
//! falls back to what Tauri knows.

#[cfg(any(windows, target_os = "macos"))]
mod gesture;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(not(any(windows, target_os = "macos")))]
mod stub;
#[cfg(windows)]
mod win;

#[cfg(target_os = "macos")]
use mac as imp;
#[cfg(not(any(windows, target_os = "macos")))]
use stub as imp;
#[cfg(windows)]
use win as imp;

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub use imp::{
    copy_selection, cursor, foreground, has_accessibility, monitor_at, own_window_in_front,
    read_focused, recognize_text, selection_icon, wait_modifiers_released,
};
#[cfg(target_os = "macos")]
pub use mac::{focus_popup, init, prepare_popup, refresh};

/// Per-platform app setup, run once at startup.
#[cfg(not(target_os = "macos"))]
pub fn init(_app: &tauri::AppHandle) {}

/// Re-reads what the OS tells about the keyboard layout; called on the main thread before a capture.
#[cfg(not(target_os = "macos"))]
pub fn refresh() {}

/// Gives the popup window the traits it needs to appear without stealing focus.
#[cfg(not(target_os = "macos"))]
pub fn prepare_popup(_window: &tauri::WebviewWindow) {}

/// Gives the shown popup the keyboard; false when the caller should focus it the usual way.
#[cfg(not(target_os = "macos"))]
pub fn focus_popup(_window: &tauri::WebviewWindow) -> bool {
    false
}

use serde::{Deserialize, Serialize};

use crate::geometry::Rect;

/// The app a capture reads from and Apply pastes back into: a window handle on Windows, a
/// process id on macOS. Only the platform layer looks inside.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Target(isize);

/// The app in front when ToneSnap was triggered.
#[derive(Debug)]
pub struct Foreground {
    pub target: Target,
    /// Lower-case executable name, e.g. `winword.exe` or `1password`; what `blockedApps` matches.
    pub process: String,
    /// What the user knows the app as, where the OS says (macOS); otherwise the caller names it.
    pub name: Option<String>,
    /// ToneSnap itself: one of its windows is in front or holds the keyboard.
    pub own: bool,
}

/// Where an app that exposes its document model held the selection, so Apply can edit the text
/// in place and keep its formatting: Word on Windows. Only the platform layer looks inside.
#[cfg(windows)]
pub use win::word::Anchor;

/// No app is edited in place on this platform, so there is never an anchor.
#[cfg(not(windows))]
#[derive(Clone, Debug)]
pub enum Anchor {}

/// What the accessibility API (UI Automation, the macOS Accessibility API) tells about the
/// focused control. Empty fields mean it did not say.
#[derive(Debug, Default)]
pub struct Accessible {
    pub selection: String,
    pub before: String,
    pub after: String,
    /// `None` when the control does not say.
    pub editable: Option<bool>,
    /// The selection's rectangle, in physical screen pixels.
    pub bounds: Option<Rect>,
    pub is_password: bool,
    /// Set when the app itself gave the selection and Apply can edit it in place.
    pub anchor: Option<Anchor>,
}

/// Mouse presses and scrolls the selection watchers have seen, so work that waits after a
/// gesture can tell whether the user has moved on.
static PRESSES: AtomicU64 = AtomicU64::new(0);

/// Called by the selection watchers for every button press and scroll.
#[cfg_attr(
    not(any(windows, target_os = "macos")),
    allow(
        dead_code,
        reason = "only the Windows and macOS watchers see the mouse"
    )
)]
fn note_press() {
    PRESSES.fetch_add(1, Ordering::SeqCst);
}

/// The presses and scrolls seen so far, for `Selection::new`.
#[cfg_attr(
    not(any(windows, target_os = "macos")),
    allow(
        dead_code,
        reason = "only the Windows and macOS watchers see the mouse"
    )
)]
fn presses() -> u64 {
    PRESSES.load(Ordering::SeqCst)
}

/// A selection gesture the floating icon's detection has confirmed, offered to Auto mode
/// before the icon shows.
pub struct Selection {
    released: Instant,
    /// How long another click could still extend it (a triple click after a double click).
    settle: Duration,
    /// `presses()` when the gesture ended.
    presses: u64,
    /// The accessibility API reported a selection where the gesture ended. Set by the watcher
    /// before the selection is offered; false where the app did not say.
    reported: bool,
}

impl Selection {
    /// A gesture that ended at `released`, when `presses()` was `presses`.
    #[cfg_attr(
        not(any(windows, target_os = "macos")),
        allow(
            dead_code,
            reason = "only the Windows and macOS watchers see the mouse"
        )
    )]
    fn new(released: Instant, settle: Duration, presses: u64) -> Self {
        Self {
            released,
            settle,
            presses,
            reported: false,
        }
    }

    /// Whether the accessibility API said the selection is where the gesture ended. Without
    /// that, the focused control's selection may be an older one somewhere else.
    pub fn reported(&self) -> bool {
        self.reported
    }

    /// When the selection may be acted on: `delay` after the release, and not while another
    /// click could still extend it.
    pub fn ready_at(&self, delay: Duration) -> Instant {
        self.released + delay.max(self.settle)
    }

    /// False once the user has pressed a mouse button or scrolled since the gesture.
    pub fn untouched(&self) -> bool {
        presses() == self.presses
    }
}

/// Puts `text` on the clipboard for the user to paste wherever they like.
pub fn copy_text(text: &str) {
    imp::clipboard::set_text(text, false);
}

/// Puts `text` in place of the captured selection: edited in place where the capture left an
/// anchor (Word keeps the formatting of the words that did not change), pasted otherwise. False
/// when neither worked; the caller then offers the text to paste by hand.
pub fn apply(target: Target, anchor: Option<&Anchor>, text: &str) -> bool {
    let pasted = match anchor.map(|anchor| imp::edit_in_place(anchor, text)) {
        Some(InPlace::Edited) => {
            bring_to_front(target);
            return true;
        }
        Some(InPlace::Failed) => return false,
        Some(InPlace::Paste(pasted)) => pasted,
        None => text.to_string(),
    };
    paste_into(target, &pasted)
}

/// What editing in place did.
#[cfg_attr(
    not(windows),
    allow(dead_code, reason = "only Word on Windows is edited in place")
)]
enum InPlace {
    Edited,
    /// The app is told nothing more; the caller offers the text to paste by hand.
    Failed,
    /// Nothing changed: paste this instead.
    Paste(String),
}

/// Brings `target` back to the front, trying for up to 800 ms. False when it did not come.
fn bring_to_front(target: Target) -> bool {
    let start = Instant::now();
    while !imp::is_in_front(target) {
        if start.elapsed() > Duration::from_millis(800) {
            return false;
        }
        imp::bring_forward(target);
        sleep(Duration::from_millis(40));
    }
    true
}

/// Brings `target` back to the front and pastes `text` over its selection, then restores the
/// user's clipboard. False when the target could not be brought forward or the clipboard could
/// not be written.
pub fn paste_into(target: Target, text: &str) -> bool {
    if !bring_to_front(target) {
        return false;
    }
    // Give the app a moment to restore its caret and selection.
    sleep(Duration::from_millis(80));

    let snapshot = imp::clipboard::snapshot();
    if !imp::clipboard::set_text(text, true) {
        return false;
    }
    imp::wait_modifiers_released(Duration::from_millis(400));
    imp::send_paste();
    // Electron apps read the clipboard asynchronously; restoring too early pastes the old content.
    sleep(Duration::from_millis(500));
    imp::clipboard::restore(&snapshot);
    true
}

/// A permission the OS makes the user grant before ToneSnap can work (macOS only).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read the selection and press Cmd+C and Cmd+V.
    Accessibility,
    /// Capture the screen for OCR context.
    ScreenRecording,
}

/// Where the permissions stand. Without a permission system (`required` false) both are granted.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub required: bool,
    pub accessibility: bool,
    pub screen_recording: bool,
}

#[cfg(target_os = "macos")]
pub use mac::permissions::{
    open_settings as open_permission_settings, request as request_permission,
};

/// Where the permissions stand.
#[cfg(target_os = "macos")]
pub fn permissions() -> Permissions {
    mac::permissions::permissions()
}

/// Where the permissions stand: there is nothing to grant on this platform.
#[cfg(not(target_os = "macos"))]
pub fn permissions() -> Permissions {
    Permissions {
        required: false,
        accessibility: true,
        screen_recording: true,
    }
}

/// Shows the system prompt for a permission.
#[cfg(not(target_os = "macos"))]
pub fn request_permission(_permission: Permission) {}

/// Opens the system's settings page for a permission.
#[cfg(not(target_os = "macos"))]
pub fn open_permission_settings(_permission: Permission) {}

/// A monitor, in physical pixels.
#[derive(Clone, Copy, Debug)]
pub struct Monitor {
    /// The part not covered by the taskbar.
    pub work: Rect,
    pub bounds: Rect,
    /// Effective DPI scale: 1.0 at 96 DPI.
    pub scale: f64,
}

/// A 1920 × 1080 screen at scale 1, for when the OS names no monitor (macOS between display
/// changes) or the platform has no OS layer.
#[cfg(not(windows))]
fn fallback_monitor() -> Monitor {
    let screen = Rect {
        left: 0,
        top: 0,
        right: 1920,
        bottom: 1080,
    };
    Monitor {
        work: screen,
        bounds: screen,
        scale: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissions_reach_the_ui_in_camel_case() {
        let json = serde_json::to_value(Permissions {
            required: true,
            accessibility: false,
            screen_recording: true,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "required": true, "accessibility": false, "screenRecording": true })
        );
    }

    #[test]
    fn a_permission_is_named_in_snake_case_by_the_ui() {
        let parse = |name: &str| serde_json::from_str::<Permission>(&format!("\"{name}\""));
        assert_eq!(parse("accessibility").unwrap(), Permission::Accessibility);
        assert_eq!(
            parse("screen_recording").unwrap(),
            Permission::ScreenRecording
        );
        assert!(parse("camera").is_err());
    }

    #[test]
    fn a_selection_waits_for_a_possible_triple_click_and_ends_at_the_next_press() {
        let now = Instant::now();
        let dragged = Selection::new(now, Duration::ZERO, presses());
        let clicked = Selection::new(now, Duration::from_millis(500), presses());
        let delay = Duration::from_millis(400);
        assert_eq!(dragged.ready_at(delay), now + delay);
        assert_eq!(clicked.ready_at(delay), now + Duration::from_millis(500));

        assert!(dragged.untouched());
        note_press();
        assert!(!dragged.untouched());
        assert!(Selection::new(now, Duration::ZERO, presses()).untouched());
    }

    #[test]
    fn nothing_is_required_where_the_os_has_no_permission_system() {
        if cfg!(not(target_os = "macos")) {
            assert!(!permissions().required);
        }
    }
}
