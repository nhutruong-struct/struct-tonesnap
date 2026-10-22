//! The foreground app, the pointer and the monitors, in the physical pixels the rest of the app
//! uses: points times the scale of the screen they are on, y growing downward from the top of
//! the primary screen. With screens of different scales these spaces do not line up exactly; the
//! popup moves to native placement with the `NSPanel` work.

use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSScreen, NSWorkspace};
use objc2_foundation::NSRect;

use crate::geometry::Rect;
use crate::platform::Monitor;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreate(source: *const std::ffi::c_void) -> super::cf::CFTypeRef;
    fn CGEventGetLocation(event: super::cf::CFTypeRef) -> Point;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A rectangle in points, origin at the top left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PointRect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl PointRect {
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.left && p.x < self.right && p.y >= self.top && p.y < self.bottom
    }

    fn scaled(&self, scale: f64) -> Rect {
        Rect {
            left: (self.left * scale).round() as i32,
            top: (self.top * scale).round() as i32,
            right: (self.right * scale).round() as i32,
            bottom: (self.bottom * scale).round() as i32,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Screen {
    frame: PointRect,
    /// Without the menu bar and the Dock.
    visible: PointRect,
    scale: f64,
}

impl Screen {
    fn monitor(&self) -> Monitor {
        Monitor {
            work: self.visible.scaled(self.scale),
            bounds: self.frame.scaled(self.scale),
            scale: self.scale,
        }
    }
}

/// Cocoa measures y upward from the bottom of the primary screen; the rest of the app measures
/// it downward from the top.
fn from_cocoa(rect: NSRect, primary_height: f64) -> PointRect {
    PointRect {
        left: rect.origin.x,
        top: primary_height - (rect.origin.y + rect.size.height),
        right: rect.origin.x + rect.size.width,
        bottom: primary_height - rect.origin.y,
    }
}

/// The screen holding a physical point, or the nearest one.
fn screen_at_physical(screens: &[Screen], x: i32, y: i32) -> Option<Screen> {
    let distance = |r: Rect| {
        let dx = (r.left - x).max(0).max(x - r.right + 1) as i64;
        let dy = (r.top - y).max(0).max(y - r.bottom + 1) as i64;
        dx * dx + dy * dy
    };
    screens
        .iter()
        .copied()
        .min_by_key(|s| distance(s.frame.scaled(s.scale)))
}

fn screen_at_point(screens: &[Screen], p: Point) -> Option<Screen> {
    screens
        .iter()
        .copied()
        .find(|s| s.frame.contains(p))
        .or_else(|| screens.first().copied())
}

/// Runs `f` on the main thread, which AppKit requires for screens, and waits for it.
pub(super) fn on_main<R: Send>(f: impl FnOnce(MainThreadMarker) -> R + Send) -> Option<R> {
    if let Some(mtm) = MainThreadMarker::new() {
        return Some(f(mtm));
    }
    let mut result = None;
    DispatchQueue::main().exec_sync(|| result = MainThreadMarker::new().map(f));
    result
}

fn screens() -> Vec<Screen> {
    on_main(|mtm| {
        let list = NSScreen::screens(mtm);
        // The first screen is the one with the menu bar; its height is the origin of every y.
        let Some(primary) = list.iter().next() else {
            return Vec::new();
        };
        let primary_height = primary.frame().size.height;
        list.iter()
            .map(|s| Screen {
                frame: from_cocoa(s.frame(), primary_height),
                visible: from_cocoa(s.visibleFrame(), primary_height),
                scale: s.backingScaleFactor(),
            })
            .collect()
    })
    .unwrap_or_default()
}

/// The pointer in points, as Core Graphics reports it (origin top left).
pub(super) fn pointer() -> Point {
    // SAFETY: a null source makes an event stamped with the current state; we own the result and
    // release it once the location is read.
    unsafe {
        let Some(event) = super::cf::Cf::from_owned(CGEventCreate(std::ptr::null())) else {
            return Point::default();
        };
        CGEventGetLocation(event.as_ptr())
    }
}

/// The scale of the screen under a point.
pub(super) fn scale_at(p: Point) -> f64 {
    screen_at_point(&screens(), p).map_or(1.0, |s| s.scale)
}

/// The part of the screen under `p` not covered by the menu bar and the Dock, and the height
/// of the primary screen (the origin of Cocoa's upward y), both in points. Main thread.
pub(super) fn visible_area(p: Point) -> Option<(PointRect, f64)> {
    let screens = screens();
    let primary_height = on_main(|mtm| {
        NSScreen::screens(mtm)
            .iter()
            .next()
            .map(|s| s.frame().size.height)
    })??;
    screen_at_point(&screens, p).map(|s| (s.visible, primary_height))
}

pub fn cursor() -> (i32, i32) {
    let p = pointer();
    let scale = scale_at(p);
    ((p.x * scale).round() as i32, (p.y * scale).round() as i32)
}

pub fn monitor_at(x: i32, y: i32) -> Monitor {
    screen_at_physical(&screens(), x, y)
        .map(|s| s.monitor())
        .unwrap_or_else(|| super::super::stub::monitor_at(x, y))
}

/// The app in front of the others.
#[derive(Debug)]
pub struct Frontmost {
    pub pid: i32,
    /// Lower-case executable name, e.g. `1password`; what `blockedApps` matches.
    pub process: String,
    /// What the user knows the app as, e.g. `Microsoft Word`.
    pub name: String,
}

impl Frontmost {
    pub fn is_own(&self) -> bool {
        self.pid as u32 == std::process::id()
    }
}

pub fn frontmost() -> Option<Frontmost> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let process = app
        .executableURL()
        .and_then(|url| url.lastPathComponent())
        .map(|name| name.to_string().to_lowercase())
        .unwrap_or_default();
    let name = app
        .localizedName()
        .map(|name| name.to_string())
        .unwrap_or_else(|| process.clone());
    Some(Frontmost {
        pid: app.processIdentifier(),
        process,
        name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_foundation::{NSPoint, NSSize};

    fn rect(x: f64, y: f64, w: f64, h: f64) -> NSRect {
        NSRect {
            origin: NSPoint { x, y },
            size: NSSize {
                width: w,
                height: h,
            },
        }
    }

    #[test]
    fn cocoa_rectangles_are_flipped_to_grow_downward() {
        // A 1440x900 primary screen with a 24-point menu bar and a 60-point Dock at the bottom.
        let visible = from_cocoa(rect(0.0, 60.0, 1440.0, 816.0), 900.0);
        assert_eq!((visible.top, visible.bottom), (24.0, 840.0));
        assert_eq!((visible.left, visible.right), (0.0, 1440.0));
    }

    #[test]
    fn a_screen_above_the_primary_gets_negative_y() {
        let above = from_cocoa(rect(0.0, 900.0, 1440.0, 900.0), 900.0);
        assert_eq!((above.top, above.bottom), (-900.0, 0.0));
    }

    fn screen(x: f64, scale: f64) -> Screen {
        let frame = PointRect {
            left: x,
            top: 0.0,
            right: x + 1000.0,
            bottom: 800.0,
        };
        Screen {
            frame,
            visible: PointRect { top: 24.0, ..frame },
            scale,
        }
    }

    #[test]
    fn physical_coordinates_multiply_points_by_the_screen_scale() {
        let monitor = screen(0.0, 2.0).monitor();
        assert_eq!(monitor.scale, 2.0);
        assert_eq!(monitor.bounds.right, 2000);
        assert_eq!((monitor.work.top, monitor.work.bottom), (48, 1600));
    }

    #[test]
    fn the_monitor_for_a_point_is_the_one_holding_it_else_the_nearest() {
        let screens = [screen(0.0, 1.0), screen(1000.0, 1.0)];
        let at = |x, y| screen_at_physical(&screens, x, y).expect("there are screens");
        assert_eq!(at(1500, 100).frame.left, 1000.0);
        assert_eq!(at(10, 10).frame.left, 0.0);
        // Past the right edge, closest to the second screen; far to the left, to the first.
        assert_eq!(at(5000, 100).frame.left, 1000.0);
        assert_eq!(at(-300, 100).frame.left, 0.0);
    }

    #[test]
    fn the_pointer_falls_back_to_the_first_screen_when_none_holds_it() {
        let screens = [screen(0.0, 2.0), screen(1000.0, 1.0)];
        let outside = Point { x: -50.0, y: 10.0 };
        assert_eq!(
            screen_at_point(&screens, outside).map(|s| s.scale),
            Some(2.0)
        );
        let on_second = Point { x: 1200.0, y: 10.0 };
        assert_eq!(
            screen_at_point(&screens, on_second).map(|s| s.scale),
            Some(1.0)
        );
    }
}
