//! Inert stand-ins for platforms without an OS layer yet (macOS is Phase 3).

use super::Monitor;
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

pub fn copy_text(_text: &str) {}

pub fn paste_into(_window: isize, _text: &str) -> bool {
    false
}

pub mod selection_icon {
    pub fn set_image(_rgba: Vec<u8>, _width: u32, _height: u32) {}
    pub fn set_on_click(_f: impl Fn() + Send + Sync + 'static) {}
    pub fn set_app_filter(_f: impl Fn(&str) -> bool + Send + Sync + 'static) {}
    pub fn start() {}
    pub fn stop() {}
    pub fn hide() {}
}
