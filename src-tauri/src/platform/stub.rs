//! Inert stand-ins for platforms without an OS layer (neither Windows nor macOS).

use std::time::Duration;

use super::{Accessible, Foreground, Monitor, Target};
use crate::geometry::Rect;

pub fn cursor() -> (i32, i32) {
    (0, 0)
}

pub fn monitor_at(_x: i32, _y: i32) -> Monitor {
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

pub fn own_window_in_front() -> Option<bool> {
    None
}

pub fn has_accessibility() -> bool {
    true
}

pub fn foreground() -> Option<Foreground> {
    None
}

pub fn wait_modifiers_released(_timeout: Duration) {}

pub fn read_focused(_target: Target, _context_chars: usize) -> Accessible {
    Accessible::default()
}

pub fn copy_selection() -> Option<String> {
    None
}

pub fn recognize_text(_area: Rect, _scale: f64) -> Option<Vec<String>> {
    None
}

pub fn is_in_front(_target: Target) -> bool {
    true
}

pub fn bring_forward(_target: Target) {}

pub fn send_paste() {}

pub mod clipboard {
    pub struct Snapshot;

    pub fn snapshot() -> Snapshot {
        Snapshot
    }

    pub fn restore(_snapshot: &Snapshot) {}

    pub fn set_text(_text: &str, _transient: bool) -> bool {
        false
    }
}

pub mod selection_icon {
    pub fn set_image(_rgba: Vec<u8>, _width: u32, _height: u32) {}
    pub fn set_on_click(_f: impl Fn() + Send + Sync + 'static) {}
    pub fn set_app_filter(_f: impl Fn(&str) -> bool + Send + Sync + 'static) {}
    pub fn start() {}
    pub fn stop() {}
    pub fn hide() {}
}
