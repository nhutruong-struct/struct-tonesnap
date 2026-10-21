//! Reads the selection, its surrounding text and whether the control is editable through the
//! Accessibility API, without touching the clipboard. Needs the Accessibility permission.

use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::Mutex;

use super::cf::{CFRange, CFTypeRef, Cf};
use super::window::{scale_at, Point};
use crate::geometry::Rect;

type AxError = i32;
const AX_SUCCESS: AxError = 0;

// AXValueType
const VALUE_CF_RANGE: i32 = 4;
const VALUE_CG_RECT: i32 = 3;

/// Apps that never answer must not freeze ToneSnap.
const MESSAGING_TIMEOUT_SECONDS: f32 = 1.5;
/// Documents longer than this (in UTF-16 units) give no context rather than a slow copy.
const MAX_VALUE_UNITS: usize = 500_000;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CgRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXUIElementCreateSystemWide() -> CFTypeRef;
    fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
    fn AXUIElementSetMessagingTimeout(element: CFTypeRef, seconds: f32) -> AxError;
    fn AXUIElementCopyAttributeValue(
        element: CFTypeRef,
        attribute: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> AxError;
    fn AXUIElementCopyParameterizedAttributeValue(
        element: CFTypeRef,
        attribute: CFTypeRef,
        parameter: CFTypeRef,
        value: *mut CFTypeRef,
    ) -> AxError;
    fn AXUIElementSetAttributeValue(
        element: CFTypeRef,
        attribute: CFTypeRef,
        value: CFTypeRef,
    ) -> AxError;
    fn AXUIElementIsAttributeSettable(
        element: CFTypeRef,
        attribute: CFTypeRef,
        settable: *mut u8,
    ) -> AxError;
    fn AXValueCreate(kind: i32, value: *const c_void) -> CFTypeRef;
    fn AXValueGetValue(value: CFTypeRef, kind: i32, out: *mut c_void) -> u8;
    fn AXValueGetTypeID() -> usize;
    fn AXIsProcessTrusted() -> u8;
}

#[derive(Debug, Default)]
pub struct Captured {
    pub selection: String,
    pub before: String,
    pub after: String,
    pub editable: Option<bool>,
    /// The selection's rectangle, in physical screen pixels.
    pub bounds: Option<Rect>,
    pub is_password: bool,
}

/// Whether the user allowed ToneSnap under Privacy & Security → Accessibility.
pub fn is_trusted() -> bool {
    // SAFETY: takes no arguments.
    unsafe { AXIsProcessTrusted() != 0 }
}

/// Reads the focused element of the app with this process id. Everything is best effort: an
/// app that does not answer leaves the fields empty.
pub fn capture(pid: i32, context_chars: usize) -> Captured {
    if !is_trusted() {
        // Every call would fail with "API disabled"; the permission prompt is a later step.
        eprintln!("Accessibility permission is not granted; the selection cannot be read");
        return Captured::default();
    }
    let Some(focused) = focused_element(pid) else {
        return Captured::default();
    };
    if focused.subrole().as_deref() == Some("AXSecureTextField") {
        return Captured {
            is_password: true,
            ..Captured::default()
        };
    }

    let range = focused.range("AXSelectedTextRange");
    let mut selection = focused.string("AXSelectedText").unwrap_or_default();
    if selection.trim().is_empty() {
        if let Some(r) = range.filter(|r| r.length > 0) {
            selection = focused.string_for_range(r).unwrap_or_default();
        }
    }

    let (before, after) = match (range, focused.utf16("AXValue")) {
        (Some(r), Some(text)) if text.len() <= MAX_VALUE_UNITS => {
            context_around(&text, r, context_chars)
        }
        _ => Default::default(),
    };

    Captured {
        editable: focused.editable(),
        bounds: range.and_then(|r| focused.bounds_for_range(r)),
        selection,
        before,
        after,
        is_password: false,
    }
}

/// The text up to `n` UTF-16 units before and after `range` in `text`. A range that does not
/// fit the text (the app counts differently) gives nothing instead of a wrong slice.
fn context_around(text: &[u16], range: CFRange, n: usize) -> (String, String) {
    let (Ok(start), Ok(len)) = (
        usize::try_from(range.location),
        usize::try_from(range.length),
    ) else {
        return Default::default();
    };
    let Some(end) = start.checked_add(len).filter(|&end| end <= text.len()) else {
        return Default::default();
    };
    (
        decode(&text[start.saturating_sub(n)..start]),
        decode(&text[end..(end + n).min(text.len())]),
    )
}

/// UTF-16 to text, dropping the half of a surrogate pair that a cut left at an edge.
fn decode(mut units: &[u16]) -> String {
    if units.first().is_some_and(|u| (0xDC00..=0xDFFF).contains(u)) {
        units = &units[1..];
    }
    if units.last().is_some_and(|u| (0xD800..=0xDBFF).contains(u)) {
        units = &units[..units.len() - 1];
    }
    String::from_utf16_lossy(units)
}

/// Processes already asked to expose their accessibility tree.
static TREE_REQUESTED: Mutex<Option<HashSet<i32>>> = Mutex::new(None);

/// Electron and Chromium build their accessibility tree only for assistive tools, which they
/// recognise by this attribute. Asked once per process; the first read right after may still
/// find nothing, and the clipboard fallback covers it.
fn request_accessibility_tree(app: &Element, pid: i32) {
    let first_time = TREE_REQUESTED
        .lock()
        .unwrap()
        .get_or_insert_with(HashSet::new)
        .insert(pid);
    if !first_time {
        return;
    }
    let Some(name) = Cf::string("AXManualAccessibility") else {
        return;
    };
    // SAFETY: the element and name are live; the value is the shared `true` constant.
    unsafe {
        AXUIElementSetAttributeValue(app.0.as_ptr(), name.as_ptr(), super::cf::boolean_true());
    }
}

fn focused_element(pid: i32) -> Option<Element> {
    // SAFETY: both create calls return owned references (or null, which `from_owned` turns into
    // `None`); the timeout call takes the system-wide element, which sets it for every element.
    unsafe {
        if let Some(system) = Cf::from_owned(AXUIElementCreateSystemWide()) {
            AXUIElementSetMessagingTimeout(system.as_ptr(), MESSAGING_TIMEOUT_SECONDS);
        }
        let app = Element(Cf::from_owned(AXUIElementCreateApplication(pid))?);
        request_accessibility_tree(&app, pid);
        app.attribute("AXFocusedUIElement").map(Element)
    }
}

struct Element(Cf);

impl Element {
    fn attribute(&self, name: &str) -> Option<Cf> {
        let name = Cf::string(name)?;
        let mut value: CFTypeRef = std::ptr::null();
        // SAFETY: the element and name are live; `value` is a local the call writes and, on
        // success, hands us a reference to.
        unsafe {
            let status = AXUIElementCopyAttributeValue(self.0.as_ptr(), name.as_ptr(), &mut value);
            if status != AX_SUCCESS {
                return None;
            }
            Cf::from_owned(value)
        }
    }

    fn utf16(&self, name: &str) -> Option<Vec<u16>> {
        self.attribute(name)?.utf16()
    }

    fn string(&self, name: &str) -> Option<String> {
        self.utf16(name).map(|u| String::from_utf16_lossy(&u))
    }

    fn subrole(&self) -> Option<String> {
        self.string("AXSubrole")
    }

    fn range(&self, name: &str) -> Option<CFRange> {
        let value = self.attribute(name)?;
        let mut range = CFRange {
            location: 0,
            length: 0,
        };
        read_ax_value(&value, VALUE_CF_RANGE, &mut range)
            .then_some(range)
            .filter(|r| r.location >= 0 && r.length >= 0)
    }

    /// The text of a range, for apps that give the range but not `AXSelectedText`.
    fn string_for_range(&self, range: CFRange) -> Option<String> {
        let units = self.parameterized("AXStringForRange", range)?.utf16()?;
        Some(String::from_utf16_lossy(&units))
    }

    fn bounds_for_range(&self, range: CFRange) -> Option<Rect> {
        let value = self.parameterized("AXBoundsForRange", range)?;
        let mut rect = CgRect::default();
        if !read_ax_value(&value, VALUE_CG_RECT, &mut rect) || rect.width <= 0.0 {
            return None;
        }
        let scale = scale_at(Point {
            x: rect.x,
            y: rect.y,
        });
        Some(Rect {
            left: (rect.x * scale).round() as i32,
            top: (rect.y * scale).round() as i32,
            right: ((rect.x + rect.width) * scale).round() as i32,
            bottom: ((rect.y + rect.height) * scale).round() as i32,
        })
    }

    fn parameterized(&self, name: &str, range: CFRange) -> Option<Cf> {
        let name = Cf::string(name)?;
        let mut value: CFTypeRef = std::ptr::null();
        // SAFETY: `range` is a live local of the type the value kind names, and is copied;
        // the parameter, element and name are live; `value` is written on success.
        unsafe {
            let parameter =
                Cf::from_owned(AXValueCreate(VALUE_CF_RANGE, (&raw const range).cast()))?;
            let status = AXUIElementCopyParameterizedAttributeValue(
                self.0.as_ptr(),
                name.as_ptr(),
                parameter.as_ptr(),
                &mut value,
            );
            if status != AX_SUCCESS {
                return None;
            }
            Cf::from_owned(value)
        }
    }

    /// `true` when text can be typed into the control, `false` for a text field that refuses it,
    /// `None` when the control does not say (Apply is still offered).
    fn editable(&self) -> Option<bool> {
        let name = Cf::string("AXValue")?;
        let mut settable = 0u8;
        // SAFETY: the element and name are live; `settable` is a local the call writes.
        let status = unsafe {
            AXUIElementIsAttributeSettable(self.0.as_ptr(), name.as_ptr(), &mut settable)
        };
        if status != AX_SUCCESS {
            return None;
        }
        if settable != 0 {
            return Some(true);
        }
        let role = self.string("AXRole")?;
        matches!(role.as_str(), "AXTextField" | "AXTextArea" | "AXComboBox").then_some(false)
    }
}

/// Reads a typed `AXValue` into `out`; false when the value is another type.
fn read_ax_value<T>(value: &Cf, kind: i32, out: &mut T) -> bool {
    // SAFETY: the type id check makes `value` an AXValue; `out` is a live local that the caller
    // sized for `kind`.
    unsafe {
        value.type_id() == AXValueGetTypeID()
            && AXValueGetValue(value.as_ptr(), kind, (out as *mut T).cast()) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    fn range(location: isize, length: isize) -> CFRange {
        CFRange { location, length }
    }

    #[test]
    fn context_is_the_text_on_each_side_up_to_the_limit() {
        let text = units("Hello big world today");
        // "world" is at 10..15.
        let (before, after) = context_around(&text, range(10, 5), 4);
        assert_eq!((before.as_str(), after.as_str()), ("big ", " tod"));
    }

    #[test]
    fn context_stops_at_the_ends_of_the_text() {
        let text = units("Hello world");
        let (before, after) = context_around(&text, range(0, 5), 100);
        assert_eq!((before.as_str(), after.as_str()), ("", " world"));
        let (before, after) = context_around(&text, range(6, 5), 100);
        assert_eq!((before.as_str(), after.as_str()), ("Hello ", ""));
    }

    #[test]
    fn a_range_that_does_not_fit_the_text_gives_no_context() {
        let text = units("short");
        for r in [range(3, 10), range(9, 0), range(-1, 2), range(2, -1)] {
            assert_eq!(context_around(&text, r, 10), Default::default(), "{r:?}");
        }
    }

    #[test]
    fn ranges_count_utf16_units_and_never_split_a_pair() {
        // Each emoji is two units; a cut between them must not leave a broken character.
        let text = units("a😀b😀c");
        let (before, after) = context_around(&text, range(3, 1), 1);
        assert_eq!((before.as_str(), after.as_str()), ("", ""));
        let (before, after) = context_around(&text, range(3, 1), 2);
        assert_eq!((before.as_str(), after.as_str()), ("😀", "😀"));
    }

    #[test]
    fn vietnamese_text_keeps_its_marks() {
        let text = units("đường Nguyễn Huệ");
        let (before, after) = context_around(&text, range(6, 6), 5);
        assert_eq!((before.as_str(), after.as_str()), ("ường ", " Huệ"));
    }
}
