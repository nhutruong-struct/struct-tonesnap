//! Reads the selection, its surrounding text and whether the control is editable via
//! UI Automation, without touching the clipboard.

use std::time::{Duration, Instant};

use windows::core::{IUnknown, Interface};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetLBound, SafeArrayGetUBound,
    SafeArrayUnaccessData,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, CUIAutomation8, IUIAutomation, IUIAutomation2, IUIAutomationElement,
    IUIAutomationTextPattern, IUIAutomationTextRange, IUIAutomationValuePattern,
    TextPatternRangeEndpoint_End, TextPatternRangeEndpoint_Start, TextUnit_Character,
    TextUnit_Paragraph, UIA_EditControlTypeId, UIA_IsReadOnlyAttributeId, UIA_TextPatternId,
    UIA_ValuePatternId, UiaGetReservedMixedAttributeValue,
};

use super::com::on_mta_thread;
use crate::geometry::Rect;

#[derive(Debug, Default)]
pub struct UiaCapture {
    pub selection: String,
    pub before: String,
    pub after: String,
    pub editable: Option<bool>,
    /// Union of the selection's bounding rectangles, in physical screen pixels.
    pub bounds: Option<Rect>,
    pub is_password: bool,
}

pub fn capture(context_chars: i32) -> UiaCapture {
    // SAFETY: UI Automation needs COM, which `on_mta_thread` initializes; every COM object is
    // released before that thread uninitializes it.
    on_mta_thread(move || unsafe { capture_inner(context_chars) }.unwrap_or_default())
        .unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionState {
    Selected,
    /// The focused control exposes its text and has nothing selected.
    Empty,
    /// The app does not say (common in Electron apps); callers fall back to heuristics.
    Unknown,
    Password,
}

/// Cheap check used before showing the floating icon. `at` is where the selection gesture ended,
/// in physical screen pixels.
pub fn selection_state(at: (i32, i32)) -> SelectionState {
    // SAFETY: as in `capture`, the call runs on a thread with COM initialized.
    on_mta_thread(move || unsafe { selection_state_inner(at) }.unwrap_or(SelectionState::Unknown))
        .unwrap_or(SelectionState::Unknown)
}

unsafe fn selection_state_inner(at: (i32, i32)) -> windows::core::Result<SelectionState> {
    selection_state_of(&automation()?.GetFocusedElement()?, Some(at))
}

unsafe fn selection_state_of(
    focused: &IUIAutomationElement,
    at: Option<(i32, i32)>,
) -> windows::core::Result<SelectionState> {
    if is_password(focused) {
        return Ok(SelectionState::Password);
    }
    // Some apps keep focus on a hidden input while the user selects somewhere else: Google Docs
    // parks its keyboard target off-screen and draws the selection on a canvas. What that input
    // says about its own selection says nothing about a gesture outside it.
    if let Some((x, y)) = at {
        if !element_rect(focused).is_some_and(|rect| rect.contains(x, y)) {
            return Ok(SelectionState::Unknown);
        }
    }
    let Ok(pattern) = focused.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
    else {
        return Ok(SelectionState::Unknown);
    };
    let ranges = pattern.GetSelection()?;
    if ranges.Length()? < 1 {
        return Ok(SelectionState::Empty);
    }
    let text = ranges.GetElement(0)?.GetText(64)?.to_string();
    Ok(if text.trim().is_empty() {
        SelectionState::Empty
    } else {
        SelectionState::Selected
    })
}

unsafe fn automation() -> windows::core::Result<IUIAutomation> {
    match CoCreateInstance::<_, IUIAutomation>(&CUIAutomation8, None, CLSCTX_INPROC_SERVER) {
        Ok(a) => {
            // A hung target app must not freeze the hotkey.
            if let Ok(a2) = a.cast::<IUIAutomation2>() {
                let _ = a2.SetConnectionTimeout(1500);
                let _ = a2.SetTransactionTimeout(2000);
            }
            Ok(a)
        }
        Err(_) => CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER),
    }
}

unsafe fn capture_inner(context_chars: i32) -> windows::core::Result<UiaCapture> {
    let automation = automation()?;
    let focused = automation.GetFocusedElement()?;
    capture_element(&automation, &focused, context_chars)
}

unsafe fn capture_element(
    automation: &IUIAutomation,
    focused: &IUIAutomationElement,
    context_chars: i32,
) -> windows::core::Result<UiaCapture> {
    if is_password(focused) {
        return Ok(UiaCapture {
            is_password: true,
            ..Default::default()
        });
    }
    let editable = editable_state(focused);

    // Browsers expose the text pattern on the document rather than the focused node.
    let walker = automation.ControlViewWalker()?;
    let mut element = focused.clone();
    let mut pattern: Option<IUIAutomationTextPattern> = None;
    for _ in 0..5 {
        if let Ok(p) = element.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId) {
            pattern = Some(p);
            break;
        }
        match walker.GetParentElement(&element) {
            Ok(parent) => element = parent,
            Err(_) => break,
        }
    }
    let Some(pattern) = pattern else {
        return Ok(UiaCapture {
            editable,
            ..Default::default()
        });
    };

    let ranges = pattern.GetSelection()?;
    if ranges.Length()? < 1 {
        return Ok(UiaCapture {
            editable,
            ..Default::default()
        });
    }
    let range = ranges.GetElement(0)?;
    // A focused element that says nothing about editing (a Chromium group, say) leaves the
    // question to the selected text itself.
    let editable = editable.or_else(|| range_editable(&range));
    let selection = range.GetText(-1)?.to_string();
    if selection.trim().is_empty() {
        return Ok(UiaCapture {
            editable,
            ..Default::default()
        });
    }
    // Chromium (seen in Slack) moves the spaces that start a line elsewhere when a range of
    // several lines is read at once, but keeps them in a one-paragraph range.
    let selection = if may_have_lost_indentation(&selection) {
        let parts = paragraphs(&range).ok().flatten();
        with_indentation(&selection, parts.as_deref())
    } else {
        selection
    };

    let before = range.Clone()?;
    before.MoveEndpointByRange(
        TextPatternRangeEndpoint_End,
        &range,
        TextPatternRangeEndpoint_Start,
    )?;
    before.MoveEndpointByUnit(
        TextPatternRangeEndpoint_Start,
        TextUnit_Character,
        -context_chars,
    )?;
    let after = range.Clone()?;
    after.MoveEndpointByRange(
        TextPatternRangeEndpoint_Start,
        &range,
        TextPatternRangeEndpoint_End,
    )?;
    after.MoveEndpointByUnit(
        TextPatternRangeEndpoint_End,
        TextUnit_Character,
        context_chars,
    )?;

    Ok(UiaCapture {
        selection,
        before: before
            .GetText(-1)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        after: after.GetText(-1).map(|s| s.to_string()).unwrap_or_default(),
        editable,
        bounds: bounding_rect(&range),
        is_password: false,
    })
}

/// Limits on the paragraph read, which runs before the popup appears. Each paragraph costs
/// about nine calls into the other app; a longer or slower selection keeps its single read.
const MAX_PARAGRAPHS: usize = 100;
const PARAGRAPH_READ_BUDGET: Duration = Duration::from_millis(300);

/// The selection read one paragraph at a time, each part cut to the selection; `None` when the
/// selection has more than `MAX_PARAGRAPHS` paragraphs or reading them takes longer than
/// `PARAGRAPH_READ_BUDGET`.
unsafe fn paragraphs(
    selection: &IUIAutomationTextRange,
) -> windows::core::Result<Option<Vec<String>>> {
    let deadline = Instant::now() + PARAGRAPH_READ_BUDGET;
    let paragraph = selection.Clone()?;
    paragraph.MoveEndpointByRange(
        TextPatternRangeEndpoint_End,
        selection,
        TextPatternRangeEndpoint_Start,
    )?;
    paragraph.ExpandToEnclosingUnit(TextUnit_Paragraph)?;
    let mut texts = Vec::new();
    while paragraph.CompareEndpoints(
        TextPatternRangeEndpoint_Start,
        selection,
        TextPatternRangeEndpoint_End,
    )? < 0
    {
        if texts.len() == MAX_PARAGRAPHS || Instant::now() > deadline {
            return Ok(None);
        }
        let part = paragraph.Clone()?;
        if part.CompareEndpoints(
            TextPatternRangeEndpoint_Start,
            selection,
            TextPatternRangeEndpoint_Start,
        )? < 0
        {
            part.MoveEndpointByRange(
                TextPatternRangeEndpoint_Start,
                selection,
                TextPatternRangeEndpoint_Start,
            )?;
        }
        if part.CompareEndpoints(
            TextPatternRangeEndpoint_End,
            selection,
            TextPatternRangeEndpoint_End,
        )? > 0
        {
            part.MoveEndpointByRange(
                TextPatternRangeEndpoint_End,
                selection,
                TextPatternRangeEndpoint_End,
            )?;
        }
        texts.push(part.GetText(-1)?.to_string());
        if paragraph.Move(TextUnit_Paragraph, 1)? == 0 {
            break;
        }
        paragraph.ExpandToEnclosingUnit(TextUnit_Paragraph)?;
    }
    Ok(Some(texts))
}

/// Text on several lines, none after the first starting with whitespace, may have lost its
/// indentation; text that has some kept it. Blank lines say nothing either way.
fn may_have_lost_indentation(text: &str) -> bool {
    let later: Vec<&str> = text
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .collect();
    !later.is_empty()
        && !later
            .iter()
            .any(|line| line.starts_with(char::is_whitespace))
}

/// The words of each line, blank lines included: what two readings of the same text share when
/// only the spaces differ.
fn line_words(text: &str) -> Vec<Vec<&str>> {
    text.trim_matches(['\r', '\n'])
        .lines()
        .map(|line| line.split_whitespace().collect())
        .collect()
}

/// How many whitespace characters start the lines of `text`, all lines together.
fn indentation(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .sum()
}

/// The paragraph read, joined by line breaks, when it gives back indentation the single read
/// lost: it must hold the same words on the same lines and more spaces at their start, so a
/// reading that splits, joins or drops anything never replaces the single read. The single
/// read's final line breaks are kept: Apply replaces them too.
fn with_indentation(single: &str, paragraphs: Option<&[String]>) -> String {
    let Some(paragraphs) = paragraphs else {
        return single.to_string();
    };
    let joined = paragraphs
        .iter()
        .map(|p| p.trim_end_matches(['\r', '\n', '\u{2028}', '\u{2029}']))
        .collect::<Vec<_>>()
        .join("\n");
    if line_words(&joined) != line_words(single) || indentation(&joined) <= indentation(single) {
        return single.to_string();
    }
    let ending = &single[single.trim_end_matches(['\r', '\n']).len()..];
    joined.trim_end_matches(['\r', '\n']).to_string() + ending
}

/// Where the element is on screen, in physical pixels.
unsafe fn element_rect(element: &IUIAutomationElement) -> Option<Rect> {
    let r = element.CurrentBoundingRectangle().ok()?;
    Some(Rect {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    })
}

unsafe fn is_password(element: &IUIAutomationElement) -> bool {
    element
        .CurrentIsPassword()
        .map(|b| b.as_bool())
        .unwrap_or(false)
}

unsafe fn editable_state(element: &IUIAutomationElement) -> Option<bool> {
    if let Ok(value) = element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
    {
        if let Ok(read_only) = value.CurrentIsReadOnly() {
            return Some(!read_only.as_bool());
        }
    }
    match element.CurrentControlType() {
        Ok(ct) if ct == UIA_EditControlTypeId => Some(true),
        _ => None,
    }
}

/// Whether the selected text can be edited, from its read-only text attribute. Chromium (Slack,
/// Chrome, Edge) answers "not read-only" inside an editable field and "mixed" for page text
/// that cannot be edited. An app that does not support the attribute gives no answer.
unsafe fn range_editable(range: &IUIAutomationTextRange) -> Option<bool> {
    let value = range.GetAttributeValue(UIA_IsReadOnlyAttributeId).ok()?;
    if let Ok(read_only) = bool::try_from(&value) {
        return Some(!read_only);
    }
    let answer = IUnknown::try_from(&value).ok()?;
    let mixed = UiaGetReservedMixedAttributeValue().ok()?;
    (answer == mixed).then_some(false)
}

unsafe fn bounding_rect(range: &IUIAutomationTextRange) -> Option<Rect> {
    let array = range.GetBoundingRectangles().ok()?;
    if array.is_null() {
        return None;
    }
    let lower = SafeArrayGetLBound(array, 1).ok();
    let upper = SafeArrayGetUBound(array, 1).ok();
    let mut data = std::ptr::null_mut();
    let mut rect: Option<Rect> = None;
    if let (Some(lower), Some(upper)) = (lower, upper) {
        if SafeArrayAccessData(array, &mut data).is_ok() {
            let count = (upper - lower + 1).max(0) as usize;
            let values = std::slice::from_raw_parts(data as *const f64, count);
            // Each rectangle is (left, top, width, height).
            for r in values.as_chunks::<4>().0 {
                if r[2] <= 0.0 && r[3] <= 0.0 {
                    continue;
                }
                let next = Rect {
                    left: r[0] as i32,
                    top: r[1] as i32,
                    right: (r[0] + r[2]) as i32,
                    bottom: (r[1] + r[3]) as i32,
                };
                rect = Some(rect.map_or(next, |acc| acc.union(next)));
            }
            let _ = SafeArrayUnaccessData(array);
        }
    }
    let _ = SafeArrayDestroy(array);
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How Chromium reads a nested list from Slack in one call: the spaces that start the nested
    /// lines are not at the start of those lines.
    const SINGLE: &str = "+ PR #4522 => Waiting for review.    \n+ Re-checked the checklist.    \n+ Created PR #4532.";
    const NESTED: &str =
        "+ PR #4522 => Waiting for review.\n    + Re-checked the checklist.\n    + Created PR #4532.";

    fn parts(text: &str) -> Vec<String> {
        text.split_inclusive('\n').map(String::from).collect()
    }

    #[test]
    fn the_paragraph_read_brings_back_the_indentation() {
        assert_eq!(with_indentation(SINGLE, Some(&parts(NESTED))), NESTED);
    }

    #[test]
    fn paragraphs_may_end_with_any_line_break() {
        let parts = vec![
            "+ PR #4522 => Waiting for review.\r\n".to_string(),
            "    + Re-checked the checklist.\u{2029}".to_string(),
            "    + Created PR #4532.".to_string(),
        ];
        assert_eq!(with_indentation(SINGLE, Some(&parts)), NESTED);
    }

    #[test]
    fn a_paragraph_read_that_differs_is_ignored() {
        let rewrapped = parts("+ PR #4522 => Waiting\n    for review.\n    + Re-checked the checklist.\n    + Created PR #4532.");
        let short = parts("+ PR #4522 => Waiting for review.\n    + Re-checked the checklist.");
        let reordered = parts("    + Re-checked the checklist.\n+ PR #4522 => Waiting for review.\n    + Created PR #4532.");
        for paragraphs in [rewrapped, short, reordered] {
            assert_eq!(with_indentation(SINGLE, Some(&paragraphs)), SINGLE);
        }
        assert_eq!(with_indentation(SINGLE, None), SINGLE);
    }

    #[test]
    fn the_final_line_break_of_the_selection_is_kept() {
        let single = format!("{SINGLE}\n");
        assert_eq!(
            with_indentation(&single, Some(&parts(&single.replace("    \n", "\n    ")))),
            format!("{NESTED}\n")
        );
    }

    #[test]
    fn a_paragraph_read_with_no_more_indentation_keeps_the_single_read() {
        let windows_text = "Line one\r\nLine two";
        assert_eq!(
            with_indentation(windows_text, Some(&parts("Line one\nLine two"))),
            windows_text
        );
    }

    #[test]
    fn which_text_may_have_lost_its_indentation() {
        assert!(may_have_lost_indentation(SINGLE));
        assert!(may_have_lost_indentation("a\n \nb"));
        assert!(may_have_lost_indentation("  a\nb"));
        assert!(!may_have_lost_indentation(NESTED));
        assert!(!may_have_lost_indentation("a single line"));
        assert!(!may_have_lost_indentation("a\n \n"));
    }
}

#[cfg(test)]
mod integration_tests {
    use windows::Win32::Foundation::HWND;

    use super::super::test_window::{Options, TestWindow};
    use super::*;

    fn element_of<T: Send + 'static>(
        window: &TestWindow,
        f: impl FnOnce(&IUIAutomation, &IUIAutomationElement) -> T + Send + 'static,
    ) -> T {
        let hwnd = window.hwnd;
        // SAFETY: runs on a COM thread; `hwnd` is the test window, alive for the whole call.
        on_mta_thread(move || unsafe {
            let automation = automation().unwrap();
            let element = automation.ElementFromHandle(HWND(hwnd as *mut _)).unwrap();
            f(&automation, &element)
        })
        .unwrap()
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn reads_the_selection_and_its_context() {
        let window = TestWindow::edit("The quick brown fox jumps", Options::default());
        window.select(4, 15);
        // SAFETY: `element_of` calls this on a COM thread with a live element.
        let (ui, state) = element_of(&window, |automation, element| unsafe {
            (
                capture_element(automation, element, 4).unwrap(),
                selection_state_of(element, None).unwrap(),
            )
        });
        assert_eq!(ui.selection, "quick brown");
        assert_eq!(ui.before, "The ");
        assert_eq!(ui.after, " fox");
        assert_eq!(ui.editable, Some(true));
        assert!(!ui.is_password);
        assert_eq!(state, SelectionState::Selected);
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn reads_a_multi_line_selection_paragraph_by_paragraph() {
        // No line after the first is indented, so the paragraph read runs. The selection starts
        // and ends inside a line, so the first and last paragraphs are cut to it.
        let text = "alpha\r\nbeta\r\ngamma";
        let selected = "pha\r\nbeta\r\ngam";
        let window = TestWindow::edit(
            text,
            Options {
                multiline: true,
                ..Options::default()
            },
        );
        window.select(2, text.len() - 2);
        // SAFETY: `element_of` calls this on a COM thread with a live element.
        let (ui, parts) = element_of(&window, |automation, element| unsafe {
            let pattern = element
                .GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)
                .unwrap();
            let range = pattern.GetSelection().unwrap().GetElement(0).unwrap();
            (
                capture_element(automation, element, 10).unwrap(),
                paragraphs(&range).unwrap().unwrap(),
            )
        });
        // The parts cover the selection exactly, in more than one piece; where they split is up
        // to the app.
        assert_eq!(parts.concat(), selected);
        assert!(parts.len() > 1, "{parts:?}");
        // An Edit control loses no spaces in a single read, so the paragraphs add nothing and
        // the single read is kept as it is.
        assert_eq!(ui.selection, selected);
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn nothing_selected_is_empty() {
        let window = TestWindow::edit("Nothing selected here", Options::default());
        window.select(3, 3);
        // SAFETY: `element_of` calls this on a COM thread with a live element.
        let (ui, state) = element_of(&window, |automation, element| unsafe {
            (
                capture_element(automation, element, 10).unwrap(),
                selection_state_of(element, None).unwrap(),
            )
        });
        assert_eq!(ui.selection, "");
        assert_eq!(state, SelectionState::Empty);
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn a_gesture_outside_the_focused_element_is_not_judged_by_it() {
        let window = TestWindow::edit("Nothing selected here", Options::default());
        window.select(3, 3);
        // SAFETY: `element_of` calls this on a COM thread with a live element.
        let (inside, outside) = element_of(&window, |_, element| unsafe {
            let rect = element_rect(element).unwrap();
            let middle_y = (rect.top + rect.bottom) / 2;
            (
                selection_state_of(element, Some((rect.left + 10, middle_y))).unwrap(),
                selection_state_of(element, Some((rect.right + 50, middle_y))).unwrap(),
            )
        });
        assert_eq!(inside, SelectionState::Empty);
        assert_eq!(outside, SelectionState::Unknown);
    }

    #[test]
    #[ignore = "win32 integration: run by check.mjs --full"]
    fn password_fields_are_refused() {
        let window = TestWindow::edit(
            "hunter2",
            Options {
                password: true,
                ..Options::default()
            },
        );
        window.select(0, 7);
        // SAFETY: `element_of` calls this on a COM thread with a live element.
        let (ui, state) = element_of(&window, |automation, element| unsafe {
            (
                capture_element(automation, element, 10).unwrap(),
                selection_state_of(element, None).unwrap(),
            )
        });
        assert!(ui.is_password);
        assert_eq!(ui.selection, "");
        assert_eq!(state, SelectionState::Password);
    }
}
