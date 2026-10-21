//! Floating ToneSnap icon shown next to a fresh text selection ("Icon" trigger mode).
//!
//! A global mouse monitor spots selection gestures: a drag or a double/triple click. The
//! Accessibility API then vetoes the obvious misses (nothing selected, password fields), and
//! the text I-beam stands in where an app does not say. The icon is a non-activating panel, so
//! the app keeps its selection and clicking the icon behaves exactly like pressing the hotkey.
//! Only mouse events are watched, which needs no permission; the keyboard is never watched.

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::sleep;
use std::time::{Duration, Instant};

use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{
    NSBackingStoreType, NSBitmapImageRep, NSColor, NSCursor, NSEvent, NSEventMask, NSEventType,
    NSImage, NSImageView, NSPanel, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use super::ax::{self, SelectionState};
use super::window::{self, pointer, Point};
use crate::platform::gesture::{DoubleClick, Gesture};

const ICON_SIZE: f64 = 26.0;
/// Hidden again after this long if the user ignores it.
const IDLE_TIMEOUT: Duration = Duration::from_secs(6);
/// The largest move between the clicks of a double click, in points.
const DOUBLE_CLICK_DISTANCE: i32 = 4;
/// `NSStatusWindowLevel`
const LEVEL_STATUS: isize = 25;
/// `CanJoinAllSpaces | FullScreenAuxiliary`
const COLLECTION_BEHAVIOR: usize = (1 << 0) | (1 << 8);

static ON_CLICK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static IMAGE: OnceLock<(Vec<u8>, u32, u32)> = OnceLock::new();
type AppFilter = Box<dyn Fn(&str) -> bool + Send + Sync>;
static APP_FILTER: OnceLock<AppFilter> = OnceLock::new();

static RUNNING: AtomicBool = AtomicBool::new(false);
static GESTURE: Mutex<Option<Gesture>> = Mutex::new(None);
/// Where the visible icon is, so the watcher can hide it.
static VISIBLE: Mutex<Option<Visible>> = Mutex::new(None);
/// The panel and the event monitors, created and dropped on the main thread only.
static ACTIVE: Mutex<Option<MainOnly<Active>>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct Visible {
    /// Left, top, right and bottom in points, y down.
    rect: (f64, f64, f64, f64),
    /// The app that owned the selection; switching away hides the icon.
    owner: i32,
    shown_at: Instant,
}

struct Active {
    panel: Retained<NSPanel>,
    global_monitor: Option<Retained<AnyObject>>,
    local_monitor: Option<Retained<AnyObject>>,
}

/// Holds objects that belong to the main thread.
struct MainOnly<T>(T);

// SAFETY: the wrapped objects are created, used and dropped inside closures that run on the main
// thread; other threads only move the wrapper without touching what is inside.
unsafe impl<T> Send for MainOnly<T> {}

/// RGBA pixels of the icon to draw (any size; it is resampled by the system).
pub fn set_image(rgba: Vec<u8>, width: u32, height: u32) {
    let _ = IMAGE.set((rgba, width, height));
}

/// Called (on the main thread) when the user clicks the icon.
pub fn set_on_click(f: impl Fn() + Send + Sync + 'static) {
    let _ = ON_CLICK.set(Box::new(f));
}

/// Decides, from the lower-case process name of the app holding the selection, whether the
/// icon may appear there. Asked on every selection, so it can follow changing settings.
pub fn set_app_filter(allowed: impl Fn(&str) -> bool + Send + Sync + 'static) {
    let _ = APP_FILTER.set(Box::new(allowed));
}

fn on_main_async(f: impl FnOnce(MainThreadMarker) + Send + 'static) {
    DispatchQueue::main().exec_async(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            f(mtm);
        }
    });
}

pub fn start() {
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(watch);
    on_main_async(|mtm| {
        let mut active = ACTIVE.lock().unwrap();
        if active.is_none() {
            *active = make_active(mtm).map(MainOnly);
        }
    });
}

pub fn stop() {
    if !RUNNING.swap(false, Ordering::SeqCst) {
        return;
    }
    *VISIBLE.lock().unwrap() = None;
    on_main_async(|_| {
        if let Some(MainOnly(active)) = ACTIVE.lock().unwrap().take() {
            active.panel.orderOut(None);
            for monitor in [&active.global_monitor, &active.local_monitor]
                .into_iter()
                .flatten()
            {
                // SAFETY: each token came from `addGlobal/LocalMonitor…` and is removed once.
                unsafe { NSEvent::removeMonitor(monitor) };
            }
        }
    });
}

/// Hides the icon if it is showing (e.g. when the hotkey is used instead).
pub fn hide() {
    if VISIBLE.lock().unwrap().take().is_some() {
        on_main_async(|_| {
            if let Some(MainOnly(active)) = ACTIVE.lock().unwrap().as_ref() {
                active.panel.orderOut(None);
            }
        });
    }
}

fn make_active(mtm: MainThreadMarker) -> Option<Active> {
    let panel = make_panel(mtm)?;

    let global = RcBlock::new(|event: NonNull<NSEvent>| {
        // SAFETY: AppKit passes a live event for the duration of the call.
        on_mouse(unsafe { event.as_ref() }.r#type());
    });
    let global_monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
        NSEventMask::LeftMouseDown
            | NSEventMask::LeftMouseUp
            | NSEventMask::RightMouseDown
            | NSEventMask::OtherMouseDown
            | NSEventMask::ScrollWheel,
        &global,
    );

    // Events inside our own windows never reach the global monitor; the icon's click comes here.
    let panel_ptr = Retained::as_ptr(&panel) as usize;
    let local = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: AppKit passes a live event; local handlers run on the main thread.
        let event_ref = unsafe { event.as_ref() };
        let ty = event_ref.r#type();
        if !matches!(ty, NSEventType::LeftMouseDown | NSEventType::LeftMouseUp) {
            return event.as_ptr();
        }
        let Some(mtm) = MainThreadMarker::new() else {
            return event.as_ptr();
        };
        let ours = event_ref
            .window(mtm)
            .is_some_and(|w| Retained::as_ptr(&w) as usize == panel_ptr);
        if !ours {
            return event.as_ptr();
        }
        if ty == NSEventType::LeftMouseUp {
            hide();
            if let Some(on_click) = ON_CLICK.get() {
                on_click();
            }
        }
        std::ptr::null_mut()
    });
    // SAFETY: the block returns the event it was given or null, as the API requires.
    let local_monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(
            NSEventMask::LeftMouseDown | NSEventMask::LeftMouseUp,
            &local,
        )
    };
    Some(Active {
        panel,
        global_monitor,
        local_monitor,
    })
}

fn make_panel(mtm: MainThreadMarker) -> Option<Retained<NSPanel>> {
    let (rgba, width, height) = IMAGE.get()?;
    let image = make_image(rgba, *width, *height)?;
    let frame = NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: NSSize {
            width: ICON_SIZE,
            height: ICON_SIZE,
        },
    };
    let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
    let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
        mtm.alloc(),
        frame,
        style,
        NSBackingStoreType::Buffered,
        false,
    );
    panel.setOpaque(false);
    panel.setBackgroundColor(Some(&NSColor::clearColor()));
    panel.setHasShadow(true);
    panel.setFloatingPanel(true);
    panel.setHidesOnDeactivate(false);
    panel.setBecomesKeyOnlyIfNeeded(true);
    // SAFETY: the panel stays alive in `ACTIVE`; it must not be freed when ordered out.
    unsafe { panel.setReleasedWhenClosed(false) };
    panel.setLevel(LEVEL_STATUS);
    panel.setCollectionBehavior(objc2_app_kit::NSWindowCollectionBehavior(
        COLLECTION_BEHAVIOR,
    ));
    panel.setContentView(Some(&NSImageView::imageViewWithImage(&image, mtm)));
    Some(panel)
}

/// The icon's pixels as an image drawn `ICON_SIZE` points wide.
fn make_image(rgba: &[u8], width: u32, height: u32) -> Option<Retained<NSImage>> {
    let (w, h) = (isize::try_from(width).ok()?, isize::try_from(height).ok()?);
    let row = usize::try_from(width).ok()? * 4;
    if rgba.len() < row * usize::try_from(height).ok()? {
        return None;
    }
    // SAFETY: with null planes the representation allocates its own buffer of `row * height`
    // bytes, which is exactly what is copied into it.
    unsafe {
        let rep = NSBitmapImageRep::initWithBitmapDataPlanes_pixelsWide_pixelsHigh_bitsPerSample_samplesPerPixel_hasAlpha_isPlanar_colorSpaceName_bytesPerRow_bitsPerPixel(
            NSBitmapImageRep::alloc(),
            std::ptr::null_mut(),
            w,
            h,
            8,
            4,
            true,
            false,
            &NSString::from_str("NSDeviceRGBColorSpace"),
            isize::try_from(row).ok()?,
            32,
        )?;
        let data = NonNull::new(rep.bitmapData())?;
        std::ptr::copy_nonoverlapping(rgba.as_ptr(), data.as_ptr(), row * height as usize);
        let image = NSImage::initWithSize(
            NSImage::alloc(),
            NSSize {
                width: ICON_SIZE,
                height: ICON_SIZE,
            },
        );
        image.addRepresentation(&rep);
        Some(image)
    }
}

fn double_click() -> DoubleClick {
    DoubleClick {
        time: Duration::from_secs_f64(NSEvent::doubleClickInterval()),
        dx: DOUBLE_CLICK_DISTANCE,
        dy: DOUBLE_CLICK_DISTANCE,
    }
}

/// Whether the pointer is the text I-beam, i.e. over selectable text. The system cursor is
/// compared by its image, since another app owns it.
#[allow(
    deprecated,
    reason = "the only way to see the pointer another app has set; without it the I-beam hint is false and only apps that report a selection get the icon"
)]
fn cursor_is_ibeam() -> bool {
    let Some(current) = NSCursor::currentSystemCursor() else {
        return false;
    };
    let current = current.image().TIFFRepresentation();
    [
        NSCursor::IBeamCursor(),
        NSCursor::IBeamCursorForVerticalLayout(),
    ]
    .iter()
    .any(|ibeam| ibeam.image().TIFFRepresentation() == current)
}

/// Runs on the main thread for every watched mouse event; it must return quickly.
fn on_mouse(kind: NSEventType) {
    let p = pointer();
    let at = (p.x as i32, p.y as i32);
    match kind {
        NSEventType::LeftMouseDown => {
            hide();
            GESTURE
                .lock()
                .unwrap()
                .get_or_insert_with(Gesture::default)
                // The I-beam is judged on release, once the pointer rests on the text.
                .press(at, Instant::now(), false, true, double_click());
        }
        NSEventType::LeftMouseUp => {
            let selected = GESTURE
                .lock()
                .unwrap()
                .get_or_insert_with(Gesture::default)
                .release(at, Instant::now(), || true);
            if selected {
                let ibeam = cursor_is_ibeam();
                std::thread::spawn(move || candidate(p, ibeam));
            }
        }
        NSEventType::RightMouseDown | NSEventType::OtherMouseDown | NSEventType::ScrollWheel => {
            hide();
        }
        _ => {}
    }
}

/// Runs off the main thread: Accessibility calls can take a while. `at` is where the gesture
/// ended.
fn candidate(at: Point, ibeam: bool) {
    // Let the app finish updating its selection first.
    sleep(Duration::from_millis(120));
    let Some(front) = window::frontmost() else {
        return;
    };
    if front.is_own()
        || APP_FILTER
            .get()
            .is_some_and(|allowed| !allowed(&front.process))
    {
        return;
    }
    let shown = match ax::selection_state(front.pid, at) {
        SelectionState::Selected => true,
        SelectionState::Unknown => ibeam,
        SelectionState::Empty | SelectionState::Password => false,
    };
    if shown && RUNNING.load(Ordering::SeqCst) {
        on_main_async(move |_| show(at, front.pid));
    }
}

/// Shows the icon just below and to the right of where the mouse was released. Main thread.
fn show(at: Point, owner: i32) {
    let guard = ACTIVE.lock().unwrap();
    let Some(MainOnly(active)) = guard.as_ref() else {
        return;
    };
    let Some((area, primary_height)) = window::visible_area(at) else {
        return;
    };
    let x = (at.x + 10.0).clamp(area.left, (area.right - ICON_SIZE).max(area.left));
    let top = (at.y + 14.0).clamp(area.top, (area.bottom - ICON_SIZE).max(area.top));
    active.panel.setFrameTopLeftPoint(NSPoint {
        x,
        y: primary_height - top,
    });
    active.panel.orderFrontRegardless();
    *VISIBLE.lock().unwrap() = Some(Visible {
        rect: (x, top, x + ICON_SIZE, top + ICON_SIZE),
        owner,
        shown_at: Instant::now(),
    });
}

/// Hides the icon when the user moves on: another app came to the front, or the icon was
/// ignored for a while.
fn watch() {
    while RUNNING.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(150));
        let visible = *VISIBLE.lock().unwrap();
        let Some(v) = visible else { continue };
        let p = pointer();
        let hovered = p.x >= v.rect.0 && p.x < v.rect.2 && p.y >= v.rect.1 && p.y < v.rect.3;
        let owner_in_front = window::frontmost().is_some_and(|f| f.pid == v.owner);
        if !owner_in_front || (!hovered && v.shown_at.elapsed() > IDLE_TIMEOUT) {
            hide();
        }
    }
}
