//! On-device OCR: ScreenCaptureKit takes the pixels of one screen area and Vision recognizes
//! the text. The image never leaves the machine and is not stored. Needs the Screen Recording
//! permission and macOS 14 or later.

use std::ptr::NonNull;
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::AnyThread;
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
use objc2_foundation::{NSArray, NSDictionary, NSError};
use objc2_screen_capture_kit::{
    SCContentFilter, SCDisplay, SCScreenshotManager, SCShareableContent, SCStreamConfiguration,
};
use objc2_vision::{
    VNImageRequestHandler, VNRecognizeTextRequest, VNRequest, VNRequestTextRecognitionLevel,
};

use crate::geometry::Rect;

/// How long each asynchronous system call may take.
const TIMEOUT: Duration = Duration::from_secs(3);

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> u8;
}

/// Whether the user allowed ToneSnap under Privacy & Security → Screen Recording.
pub fn screen_capture_allowed() -> bool {
    // SAFETY: takes no arguments and only reads the permission state.
    unsafe { CGPreflightScreenCaptureAccess() != 0 }
}

/// Carries an object out of a system callback that runs on another thread.
struct Handoff<T>(T);

// SAFETY: the wrapped ScreenCaptureKit and Core Graphics objects are immutable results that the
// callback gives up; only the receiving thread uses them afterwards.
unsafe impl<T> Send for Handoff<T> {}

/// Reads a small patch of the screen once, in the background. Vision's first recognition after
/// a boot can take a minute while it prepares its model, which a capture must not wait for.
pub fn warm_up() {
    std::thread::spawn(|| {
        let _ = recognize(
            Rect {
                left: 0,
                top: 0,
                right: 240,
                bottom: 80,
            },
            1.0,
        );
    });
}

/// The text lines recognized in `area` (physical pixels, `scale` per point), top to bottom.
/// `None` when the permission is missing, the screen cannot be captured or nothing is read.
pub fn recognize(area: Rect, scale: f64) -> Option<Vec<String>> {
    if area.width() <= 0 || area.height() <= 0 || !screen_capture_allowed() {
        return None;
    }
    let image = capture(area, scale)?;
    read_text(&image)
}

fn capture(area: Rect, scale: f64) -> Option<CFRetained<CGImage>> {
    let points = CGRect {
        origin: CGPoint {
            x: f64::from(area.left) / scale,
            y: f64::from(area.top) / scale,
        },
        size: CGSize {
            width: f64::from(area.width()) / scale,
            height: f64::from(area.height()) / scale,
        },
    };
    let display = display_holding(points.origin.x + points.size.width / 2.0, points.origin.y)?;
    // SAFETY: `display` is a live SCDisplay; the frame is a plain value read.
    let frame = unsafe { display.0.frame() };

    // SAFETY: the filter, configuration and block are all live for the calls below; the sizes and
    // rectangle are plain values in the units the setters document (points, display-relative).
    unsafe {
        let filter = SCContentFilter::initWithDisplay_excludingWindows(
            SCContentFilter::alloc(),
            &display.0,
            &NSArray::new(),
        );
        let config = SCStreamConfiguration::new();
        config.setSourceRect(CGRect {
            origin: CGPoint {
                x: points.origin.x - frame.origin.x,
                y: points.origin.y - frame.origin.y,
            },
            size: points.size,
        });
        config.setWidth(usize::try_from(area.width()).ok()?);
        config.setHeight(usize::try_from(area.height()).ok()?);
        config.setShowsCursor(false);

        let (tx, rx) = mpsc::channel();
        let block = RcBlock::new(move |image: *mut CGImage, _error: *mut NSError| {
            let image = NonNull::new(image).map(|p| Handoff(CFRetained::retain(p)));
            let _ = tx.send(image);
        });
        SCScreenshotManager::captureImageWithFilter_configuration_completionHandler(
            &filter,
            &config,
            Some(&block),
        );
        rx.recv_timeout(TIMEOUT).ok()?.map(|h| h.0)
    }
}

/// The display whose frame holds the point (global points, y down).
fn display_holding(x: f64, y: f64) -> Option<Handoff<Retained<SCDisplay>>> {
    let (tx, rx) = mpsc::channel();
    let block = RcBlock::new(
        move |content: *mut SCShareableContent, _error: *mut NSError| {
            // SAFETY: the pointer is null or a live object for the duration of the callback; the
            // displays are retained before it returns.
            let displays = unsafe { content.as_ref() }
                .map(|content| unsafe { content.displays() }.to_vec())
                .unwrap_or_default();
            let _ = tx.send(Handoff(displays));
        },
    );
    // SAFETY: the block lives through the call and the system retains it for the callback.
    unsafe { SCShareableContent::getShareableContentWithCompletionHandler(&block) };
    let displays = rx.recv_timeout(TIMEOUT).ok()?.0;
    displays.into_iter().find_map(|display| {
        // SAFETY: a plain value read of a live display.
        let f = unsafe { display.frame() };
        let inside = x >= f.origin.x
            && x < f.origin.x + f.size.width
            && y >= f.origin.y
            && y < f.origin.y + f.size.height;
        inside.then_some(Handoff(display))
    })
}

fn read_text(image: &CGImage) -> Option<Vec<String>> {
    // SAFETY: the handler gets a live image and an empty options dictionary; `request` is kept
    // until the results were read.
    unsafe {
        let handler = VNImageRequestHandler::initWithCGImage_options(
            VNImageRequestHandler::alloc(),
            image,
            &NSDictionary::new(),
        );
        let request = VNRecognizeTextRequest::new();
        request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
        request.setUsesLanguageCorrection(true);
        request.setAutomaticallyDetectsLanguage(true);
        let base: &VNRequest = &request;
        handler
            .performRequests_error(&NSArray::from_slice(&[base]))
            .ok()?;

        let mut lines: Vec<(f64, String)> = request
            .results()?
            .iter()
            .filter_map(|observation| {
                let best = observation.topCandidates(1).firstObject()?;
                Some((
                    observation.boundingBox().origin.y,
                    best.string().to_string(),
                ))
            })
            .collect();
        // Vision's y grows upward from the bottom of the image.
        lines.sort_by(|a, b| b.0.total_cmp(&a.0));
        let lines: Vec<String> = lines.into_iter().map(|(_, text)| text).collect();
        (!lines.is_empty()).then_some(lines)
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Reads whatever text is on the screen; it asserts only that some was found and never prints
    /// it, since the screen holds the user's own content.
    #[test]
    #[ignore = "macOS integration: needs the Screen Recording permission and text on screen"]
    fn recognizes_text_on_the_screen_without_leaking_it() {
        assert!(screen_capture_allowed(), "Screen Recording is not allowed");
        let area = Rect {
            left: 0,
            top: 0,
            right: 2400,
            bottom: 1200,
        };
        let lines = recognize(area, 2.0).expect("the screen was captured and read");
        assert!(lines.iter().any(|line| line.trim().len() > 3));
    }
}
