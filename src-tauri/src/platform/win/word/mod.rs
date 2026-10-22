//! Microsoft Word through its COM object model: the capture reads the selection with Word's own
//! offsets, and Apply replaces only the words that changed, so everything else keeps its
//! formatting. Late-bound `IDispatch` calls, so no type library is needed. Every call runs on a
//! dedicated MTA thread and waits a bounded time, because a busy Word does not answer.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::RecvTimeoutError;
use std::sync::Arc;
use std::time::Duration;

use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::IDispatch;
use windows::Win32::UI::Accessibility::AccessibleObjectFromWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO, OBJID_NATIVEOM,
};

use super::com;
use super::dispatch::{retry, unexpected, Object, Value};
use crate::edits;

mod text;

use text::{Field, Mapped};

/// How long a capture waits for Word before it goes on without it.
const READ_LIMIT: Duration = Duration::from_millis(1500);
/// How long Apply waits for Word to start editing before it reports a failure.
const REPLACE_LIMIT: Duration = Duration::from_millis(3000);

/// `Selection.StoryType`: the main text, not a header, footnote or text box, whose offsets
/// `Document.Range` would read in the main text instead.
const WD_MAIN_TEXT_STORY: i32 = 1;
/// `Selection.Type`: one contiguous range, not a column block whose `Start..End` covers more.
const WD_SELECTION_NORMAL: i32 = 2;

/// Where the selection was in a Word document, so Apply can edit it in place.
#[derive(Clone, Debug)]
pub struct Anchor {
    /// The document pane (`_WwG` window) that had the keyboard.
    pane: isize,
    start: i32,
    end: i32,
    /// The paragraph marks the user selected after `end`, left out of the text (`text::map`).
    marks: usize,
    /// The selection as the model got it.
    text: String,
}

impl Anchor {
    /// What to paste when the document could not be edited in place: Word's selection still
    /// holds the paragraph marks the text left out, so they are pasted back.
    pub fn pasted(&self, text: &str) -> String {
        format!("{text}{}", "\n".repeat(self.marks))
    }
}

#[cfg(test)]
impl Anchor {
    /// An anchor for tests outside the platform layer.
    pub fn example() -> Self {
        Self {
            pane: 0,
            start: 0,
            end: 15,
            marks: 0,
            text: "pls check my PR".into(),
        }
    }
}

/// The selection and its context, read from Word.
pub struct Read {
    pub anchor: Anchor,
    pub before: String,
    pub after: String,
}

impl Read {
    pub fn selection(&self) -> &str {
        &self.anchor.text
    }
}

/// What Apply did to the document.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The new text is in place.
    Edited,
    /// Nothing was changed: paste instead.
    Untouched,
    /// Word did not start editing in time (and never will), or an edit failed after others
    /// were made (and undone).
    Failed,
}

/// Steps of an Apply, shared by the caller and the thread talking to Word, so that a late
/// answer from Word cannot start editing after the caller has given up.
const PENDING: u8 = 0;
const EDITING: u8 = 1;
const ABANDONED: u8 = 2;

/// Set while a thread is waiting on Word. A hung Word would otherwise collect one blocked thread
/// per capture; while one waits, Word is left alone.
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);

struct InFlight;

impl InFlight {
    fn claim() -> Option<Self> {
        IN_FLIGHT
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            .then_some(Self)
    }
}

impl Drop for InFlight {
    fn drop(&mut self) {
        IN_FLIGHT.store(false, Ordering::SeqCst);
    }
}

/// `doc.Range(start, end)`.
fn range(doc: &Object, start: i32, end: i32) -> windows::core::Result<Object> {
    doc.call("Range", vec![Value::int(start), Value::int(end)])?
        .as_object()
        .ok_or_else(unexpected)
}

/// The Word document pane that has the keyboard in the thread of `window`. `None` when the
/// keyboard is in the ribbon, a task pane or a dialog, where Word's selection is not the text
/// the user selected.
fn focused_pane(window: isize) -> Option<isize> {
    // SAFETY: `info` is a local with `cbSize` set and `class` a local buffer whose length the call
    // is given; a stale window only makes the calls fail.
    unsafe {
        let thread = GetWindowThreadProcessId(HWND(window as *mut _), None);
        let mut info = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        GetGUIThreadInfo(thread, &mut info).ok()?;
        let mut class = [0u16; 16];
        let len = GetClassNameW(info.hwndFocus, &mut class).max(0) as usize;
        (String::from_utf16_lossy(&class[..len]) == "_WwG").then_some(info.hwndFocus.0 as isize)
    }
}

/// Word's `Window` object for a document pane. Reached through the window rather than the
/// running-object table, so it is the right document even with several Word processes.
fn document_window(pane: isize) -> windows::core::Result<Object> {
    let mut raw = std::ptr::null_mut();
    // SAFETY: `raw` is a local the call fills with an IDispatch it AddRefs; ownership passes to
    // the `IDispatch` built from it. Called on an MTA thread.
    unsafe {
        AccessibleObjectFromWindow(
            HWND(pane as *mut _),
            OBJID_NATIVEOM.0 as u32,
            &IDispatch::IID,
            &mut raw,
        )?;
        if raw.is_null() {
            return Err(unexpected());
        }
        Ok(Object(IDispatch::from_raw(raw)))
    }
}

fn utf16_len(text: &str) -> i32 {
    text.encode_utf16().count() as i32
}

/// The fields inside `range` (a link is one), by position. `None` when one shows its code
/// instead of its result, so the text would not be what the user reads.
fn fields(range: &Object) -> windows::core::Result<Option<Vec<Field>>> {
    let fields = range.object("Fields")?;
    let mut out = Vec::new();
    for i in 1..=fields.int("Count")? {
        let field = fields
            .call("Item", vec![Value::int(i)])?
            .as_object()
            .ok_or_else(unexpected)?;
        if field.flag("ShowCodes")? {
            return Ok(None);
        }
        let code = field.object("Code")?;
        out.push(Field {
            // The opening mark sits just before the code, the separator just after it.
            start: code.int("Start")? - 1,
            separator: code.int("End")?,
            end: field.object("Result")?.int("End")?,
        });
    }
    Ok(Some(out))
}

/// The text of `start..end` mapped to its positions (`text::map`).
fn mapped(doc: &Object, start: i32, end: i32) -> windows::core::Result<Option<Mapped>> {
    let selected = range(doc, start, end)?;
    if has_revisions(&selected)? {
        return Ok(None);
    }
    let raw = selected.text("Text")?;
    Ok(fields(&selected)?.and_then(|fields| text::map(start, end, &raw, fields)))
}

/// Word's text around the selection, as context: nothing has to map back to offsets.
fn context_text(raw: &str) -> String {
    raw.chars()
        .filter_map(|c| match c {
            '\r' | '\u{b}' | '\u{c}' => Some('\n'),
            '\u{7}' => Some('\t'),
            '\t' => Some('\t'),
            c if c < ' ' => None,
            c => Some(c),
        })
        .collect()
}

/// Text as Word stores it: `\r` ends a paragraph.
fn word_text(text: &str) -> String {
    text.replace('\n', "\r")
}

/// Whether the range holds tracked changes. Deleted text then stays in `Range.Text` with its
/// positions, so the text is not what the user reads.
fn has_revisions(range: &Object) -> windows::core::Result<bool> {
    Ok(range.object("Revisions")?.int("Count")? > 0)
}

/// The selection in the Word document that has the keyboard in `window`, with `context_chars`
/// of text on each side. `None` when the keyboard is elsewhere, Word does not answer in time
/// (or another call is still waiting on it), or the selection cannot be edited in place.
pub fn read(window: isize, context_chars: usize) -> Option<Read> {
    let pane = focused_pane(window)?;
    let busy = InFlight::claim()?;
    let n = i32::try_from(context_chars).unwrap_or(i32::MAX);
    com::on_mta_thread_within(READ_LIMIT, move || {
        let _busy = busy;
        retry(|| {
            let window = document_window(pane)?;
            read_selection(&window.object("Selection")?, &window.object("Document")?, n)
        })
        .ok()
        .flatten()
    })
    .flatten()
    .map(|selected| Read {
        anchor: Anchor {
            pane,
            start: selected.start,
            end: selected.end,
            marks: selected.marks,
            text: selected.text,
        },
        before: selected.before,
        after: selected.after,
    })
}

struct Selected {
    start: i32,
    end: i32,
    marks: usize,
    text: String,
    before: String,
    after: String,
}

fn read_selection(
    selection: &Object,
    doc: &Object,
    n: i32,
) -> windows::core::Result<Option<Selected>> {
    if selection.int("StoryType")? != WD_MAIN_TEXT_STORY
        || selection.int("Type")? != WD_SELECTION_NORMAL
    {
        return Ok(None);
    }
    let start = selection.int("Start")?;
    let Some(mapped) = mapped(doc, start, selection.int("End")?)? else {
        return Ok(None);
    };
    let end = mapped.end;
    let last = doc.object("Content")?.int("End")?;
    let before = range(doc, start.saturating_sub(n).max(0), start)?.text("Text")?;
    let after = range(doc, end, end.saturating_add(n).min(last))?.text("Text")?;
    Ok(Some(Selected {
        start,
        end,
        marks: mapped.marks,
        text: mapped.text,
        before: context_text(&before),
        after: context_text(&after),
    }))
}

/// Puts `text` in place of the anchored selection, changing only the words that differ, as one
/// undo step. `Untouched` when Word cannot be reached or the selection's text has changed since
/// the capture. Once Word has started editing, waits for it to finish whatever the time.
pub fn replace(anchor: &Anchor, text: &str) -> Outcome {
    let Some(busy) = InFlight::claim() else {
        return Outcome::Untouched;
    };
    let (anchor, text) = (anchor.clone(), text.to_string());
    let step = Arc::new(AtomicU8::new(PENDING));
    let worker = step.clone();
    let result = com::start_on_mta_thread(move || {
        let _busy = busy;
        retry(|| {
            let window = document_window(anchor.pane)?;
            replace_in(
                &window.object("Application")?,
                &window.object("Document")?,
                &anchor,
                &text,
                &worker,
            )
        })
        .unwrap_or(Outcome::Untouched)
    });
    match result.recv_timeout(REPLACE_LIMIT) {
        Ok(outcome) => outcome,
        Err(RecvTimeoutError::Timeout) => {
            if step
                .compare_exchange(PENDING, ABANDONED, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                // Word will not edit now: the user gets the text to paste.
                Outcome::Failed
            } else {
                // Word is editing: only its answer says how the document ends up.
                result.recv().unwrap_or(Outcome::Failed)
            }
        }
        Err(RecvTimeoutError::Disconnected) => Outcome::Failed,
    }
}

fn replace_in(
    app: &Object,
    doc: &Object,
    anchor: &Anchor,
    text: &str,
    step: &AtomicU8,
) -> windows::core::Result<Outcome> {
    let Anchor { start, end, .. } = *anchor;
    let Some(mapped) = mapped(doc, start, end)?.filter(|m| m.end == end && m.text == anchor.text)
    else {
        return Ok(Outcome::Untouched);
    };
    // The model may answer with Windows line breaks; Word gets one paragraph mark for each.
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let Some(changes) = mapped.changes(&edits::edits(&anchor.text, &text)) else {
        return Ok(Outcome::Untouched);
    };
    let tracking = doc.flag("TrackRevisions").unwrap_or(false);

    // One undo step for the whole Apply (Word 2010 and later).
    let record = retry(|| {
        let undo = app.object("UndoRecord")?;
        undo.call("StartCustomRecord", vec![Value::text("ToneSnap")])?;
        Ok(undo)
    })
    .ok();
    let end_record = || {
        if let Some(undo) = &record {
            let _ = undo.call("EndCustomRecord", Vec::new());
        }
    };
    if step
        .compare_exchange(PENDING, EDITING, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // The caller gave up waiting and told the user; nothing may change now.
        end_record();
        return Ok(Outcome::Untouched);
    }
    // From the end, so the offsets of the edits still to come stay valid.
    let mut applied = 0;
    let mut failed = false;
    for change in changes.iter().rev() {
        let new = word_text(&change.text);
        match retry(|| range(doc, change.from, change.to)?.put("Text", Value::text(&new))) {
            Ok(()) => applied += 1,
            Err(_) => {
                failed = true;
                break;
            }
        }
    }
    end_record();
    if failed {
        if applied == 0 {
            return Ok(Outcome::Untouched);
        }
        let steps = if record.is_some() { 1 } else { applied };
        let _ = doc.call("Undo", vec![Value::int(steps)]);
        return Ok(Outcome::Failed);
    }

    // The caret after the new text, as after a paste. Tracked deletions stay in the document,
    // so then the selection only grows by what was inserted.
    let grown: i32 = changes
        .iter()
        .map(|c| utf16_len(&c.text) - if tracking { 0 } else { c.to - c.from })
        .sum();
    if let Ok(caret) = range(doc, end + grown, end + grown) {
        let _ = caret.call("Select", Vec::new());
    }
    Ok(Outcome::Edited)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_keeps_line_breaks_and_drops_word_marks() {
        assert_eq!(
            context_text("a\rb\u{b}c\u{c}d\u{7}e\u{13}"),
            "a\nb\nc\nd\te"
        );
    }

    #[test]
    fn new_paragraphs_are_written_as_word_paragraph_marks() {
        assert_eq!(word_text("one\ntwo\n"), "one\rtwo\r");
    }

    #[test]
    fn a_paste_instead_puts_back_the_paragraph_marks_left_out() {
        let anchor = Anchor {
            marks: 1,
            ..Anchor::example()
        };
        assert_eq!(anchor.pasted("Please review"), "Please review\n");
        assert_eq!(Anchor::example().pasted("Please review"), "Please review");
    }
}

/// Tests against a real, invisible Word. They take no focus and send no input, but need Word
/// installed and start it, so they run only on request: `cargo test word_tests -- --ignored`.
#[cfg(test)]
mod word_tests {
    use windows::core::w;
    use windows::Win32::System::Com::{CLSIDFromProgID, CoCreateInstance, CLSCTX_LOCAL_SERVER};

    use super::*;

    /// An invisible Word with one new document, closed without saving however the test ends.
    struct Word {
        app: Object,
        doc: Object,
    }

    impl Word {
        fn start(text: &str) -> Self {
            // SAFETY: runs on an MTA thread (`on_mta_thread`); the ProgID is a static literal.
            let app: IDispatch = unsafe {
                let clsid = CLSIDFromProgID(w!("Word.Application")).expect("Word is installed");
                CoCreateInstance(&clsid, None, CLSCTX_LOCAL_SERVER).expect("Word starts")
            };
            let app = Object(app);
            let doc = app
                .object("Documents")
                .and_then(|docs| docs.call("Add", Vec::new()))
                .ok()
                .and_then(|v| v.as_object())
                .expect("a new document");
            doc.object("Content")
                .and_then(|content| content.put("Text", Value::text(text)))
                .expect("the text is written");
            Self { app, doc }
        }

        fn text(&self) -> String {
            self.doc
                .object("Content")
                .and_then(|c| c.text("Text"))
                .expect("the text")
        }

        fn font(&self, start: i32, end: i32, property: &str, value: Option<i32>) -> i32 {
            let font = range(&self.doc, start, end)
                .and_then(|r| r.object("Font"))
                .expect("the font");
            if let Some(value) = value {
                font.put(property, Value::int(value)).expect("set the font");
            }
            font.int(property).expect("read the font")
        }

        fn select(&self, start: i32, end: i32) -> Object {
            range(&self.doc, start, end)
                .and_then(|r| r.call("Select", Vec::new()))
                .expect("select");
            self.app.object("Selection").expect("the selection")
        }

        fn anchor(&self, start: i32, end: i32, text: &str) -> Anchor {
            Anchor {
                pane: 0,
                start,
                end,
                marks: 0,
                text: text.into(),
            }
        }

        fn replace(&self, anchor: &Anchor, text: &str) -> Outcome {
            replace_in(&self.app, &self.doc, anchor, text, &AtomicU8::new(PENDING))
                .expect("Word answers")
        }

        fn caret(&self) -> (i32, i32) {
            let selection = self.app.object("Selection").expect("the selection");
            (
                selection.int("Start").expect("start"),
                selection.int("End").expect("end"),
            )
        }
    }

    /// `item(collection, index)`, for Word's collections.
    fn item(parent: &Object, collection: &str, index: i32) -> Object {
        parent
            .object(collection)
            .and_then(|c| c.call("Item", vec![Value::int(index)]))
            .ok()
            .and_then(|v| v.as_object())
            .expect("the item")
    }

    impl Drop for Word {
        fn drop(&mut self) {
            // 0: wdDoNotSaveChanges.
            let _ = self.doc.call("Close", vec![Value::int(0)]);
            let _ = self.app.call("Quit", vec![Value::int(0)]);
        }
    }

    const TRUE: i32 = -1;

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn reads_the_selection_without_its_paragraph_mark() {
        com::on_mta_thread(|| {
            let word = Word::start("Hi team,\rThe build is broken.");
            // "The build is broken." and its paragraph mark.
            let selection = word.select(9, 30);
            let read = read_selection(&selection, &word.doc, 100)
                .expect("Word answers")
                .expect("a selection");
            assert_eq!((read.start, read.end, read.marks), (9, 29, 1));
            assert_eq!(read.text, "The build is broken.");
            assert_eq!(read.before, "Hi team,\n");
            assert_eq!(read.after, "\n");
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn a_header_is_left_to_ui_automation() {
        com::on_mta_thread(|| {
            let word = Word::start("Body text that must not be read");
            // 1: wdHeaderFooterPrimary.
            let header = item(&item(&word.doc, "Sections", 1), "Headers", 1)
                .object("Range")
                .expect("the header");
            header
                .put("Text", Value::text("Header text"))
                .expect("the header text");
            header
                .call("Select", Vec::new())
                .expect("select the header");
            let selection = word.app.object("Selection").expect("the selection");
            assert!(read_selection(&selection, &word.doc, 100)
                .expect("Word answers")
                .is_none());
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn with_track_changes_the_caret_ends_after_the_new_text_and_revisions_are_not_read() {
        com::on_mta_thread(|| {
            let word = Word::start("The quick brown fox");
            word.doc
                .put("TrackRevisions", Value::flag(true))
                .expect("track changes");
            let anchor = word.anchor(0, 19, "The quick brown fox");
            assert_eq!(word.replace(&anchor, "The quick red fox"), Outcome::Edited);
            // "brown" stays as a tracked deletion next to the inserted "red".
            assert_eq!(word.caret(), (22, 22));

            let selection = word.select(0, 22);
            assert!(read_selection(&selection, &word.doc, 100)
                .expect("Word answers")
                .is_none());
            assert_eq!(
                word.replace(&word.anchor(0, 22, "The quick brownred fox"), "x"),
                Outcome::Untouched
            );
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn unchanged_words_keep_their_formatting_and_one_undo_reverts() {
        com::on_mta_thread(|| {
            let word = Word::start("The quick brown fox");
            assert_eq!(word.font(4, 9, "Bold", Some(TRUE)), TRUE);
            assert_eq!(word.font(10, 15, "Italic", Some(TRUE)), TRUE);
            let anchor = word.anchor(0, 19, "The quick brown fox");

            assert_eq!(
                word.replace(&anchor, "The quick red fox jumps"),
                Outcome::Edited
            );
            assert_eq!(word.text(), "The quick red fox jumps\r");
            assert_eq!(word.caret(), (23, 23), "the caret ends after the new text");
            assert_eq!(word.font(4, 9, "Bold", None), TRUE, "quick stays bold");
            assert_eq!(
                word.font(10, 13, "Italic", None),
                TRUE,
                "red replaces an italic word"
            );
            assert_eq!(word.font(17, 23, "Italic", None), 0, "jumps follows fox");

            word.doc.call("Undo", vec![Value::int(1)]).expect("undo");
            assert_eq!(word.text(), "The quick brown fox\r");
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn a_selection_edited_since_the_capture_is_left_alone() {
        com::on_mta_thread(|| {
            let word = Word::start("The quick brown fox");
            let anchor = word.anchor(0, 19, "The slow brown fox!");
            assert_eq!(word.replace(&anchor, "The red fox"), Outcome::Untouched);
            assert_eq!(word.text(), "The quick brown fox\r");
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn nothing_changes_once_the_caller_has_given_up() {
        com::on_mta_thread(|| {
            let word = Word::start("The quick brown fox");
            let anchor = word.anchor(0, 19, "The quick brown fox");
            let outcome = replace_in(
                &word.app,
                &word.doc,
                &anchor,
                "The red fox",
                &AtomicU8::new(ABANDONED),
            );
            assert_eq!(outcome.ok(), Some(Outcome::Untouched));
            assert_eq!(word.text(), "The quick brown fox\r");
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn a_link_keeps_its_address_when_words_around_and_inside_it_change() {
        com::on_mta_thread(|| {
            let word = Word::start("see the release notes now");
            let notes = range(&word.doc, 8, 21).expect("the link text");
            word.doc
                .object("Hyperlinks")
                .and_then(|links| {
                    links.call(
                        "Add",
                        vec![
                            Value::object(&notes),
                            Value::text("https://example.com/notes"),
                        ],
                    )
                })
                .expect("the link");
            let end = word
                .doc
                .object("Content")
                .and_then(|c| c.int("End"))
                .expect("the end");

            let selection = word.select(0, end);
            let read = read_selection(&selection, &word.doc, 100)
                .expect("Word answers")
                .expect("the link's code and marks map to positions");
            assert_eq!(read.text, "see the release notes now");
            let anchor = Anchor {
                pane: 0,
                start: read.start,
                end: read.end,
                marks: read.marks,
                text: read.text,
            };
            assert_eq!(
                word.replace(&anchor, "read the release summary today"),
                Outcome::Edited
            );
            assert_eq!(word.text(), "read the release summary today\r");
            let link = item(&word.doc, "Hyperlinks", 1);
            assert_eq!(
                link.text("TextToDisplay").ok().as_deref(),
                Some("release summary")
            );
            assert_eq!(
                link.text("Address").ok().as_deref(),
                Some("https://example.com/notes")
            );
        })
        .expect("the test thread ran");
    }

    #[test]
    #[ignore = "starts Microsoft Word: run with cargo test word_tests -- --ignored"]
    fn windows_line_breaks_in_the_answer_add_one_paragraph_each() {
        com::on_mta_thread(|| {
            let word = Word::start("one\rtwo");
            let anchor = word.anchor(0, 7, "one\ntwo");
            assert_eq!(word.replace(&anchor, "One\r\nTwo"), Outcome::Edited);
            assert_eq!(word.text(), "One\rTwo\r");
            assert_eq!(word.caret(), (7, 7));
        })
        .expect("the test thread ran");
    }
}
