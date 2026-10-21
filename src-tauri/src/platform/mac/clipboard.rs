//! Clipboard snapshot/restore so reading the selection (Cmd+C) and applying a suggestion
//! (Cmd+V) leave the user's clipboard as it was.

use std::thread::sleep;
use std::time::{Duration, Instant};

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{NSArray, NSData, NSString};

use super::input;

const TYPE_TEXT: &str = "public.utf8-plain-text";
/// Marker types clipboard managers honour: the content is short-lived and not to be kept.
const TYPE_TRANSIENT: &str = "org.nspasteboard.TransientType";
const TYPE_CONCEALED: &str = "org.nspasteboard.ConcealedType";

/// Every item on the clipboard with the bytes of each of its types.
pub struct Snapshot(Vec<Vec<(String, Vec<u8>)>>);

pub fn snapshot() -> Snapshot {
    let board = NSPasteboard::generalPasteboard();
    let items = board
        .pasteboardItems()
        .map(|a| a.to_vec())
        .unwrap_or_default();
    Snapshot(
        items
            .iter()
            .map(|item| {
                item.types()
                    .iter()
                    .filter_map(|kind| {
                        let data = item.dataForType(&kind)?;
                        Some((kind.to_string(), data.to_vec()))
                    })
                    .collect()
            })
            .collect(),
    )
}

pub fn restore(snapshot: &Snapshot) {
    let board = NSPasteboard::generalPasteboard();
    board.clearContents();
    let items: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = snapshot
        .0
        .iter()
        .filter(|types| !types.is_empty())
        .map(|types| {
            let item = NSPasteboardItem::new();
            for (kind, bytes) in types {
                item.setData_forType(&NSData::with_bytes(bytes), &NSString::from_str(kind));
            }
            ProtocolObject::from_retained(item)
        })
        .collect();
    if !items.is_empty() {
        board.writeObjects(&NSArray::from_retained_slice(&items));
    }
}

/// Puts `text` on the clipboard. `transient` keeps it out of clipboard managers' history, for
/// content that only exists long enough to be pasted.
pub fn set_text(text: &str, transient: bool) -> bool {
    let board = NSPasteboard::generalPasteboard();
    board.clearContents();
    let item = NSPasteboardItem::new();
    let mut ok = item.setString_forType(&NSString::from_str(text), &NSString::from_str(TYPE_TEXT));
    if transient {
        for kind in [TYPE_TRANSIENT, TYPE_CONCEALED] {
            item.setData_forType(&NSData::with_bytes(b""), &NSString::from_str(kind));
        }
    }
    let items = [ProtocolObject::from_retained(item)];
    ok &= board.writeObjects(&NSArray::from_retained_slice(&items));
    ok
}

pub fn get_text() -> Option<String> {
    let board = NSPasteboard::generalPasteboard();
    board
        .stringForType(&NSString::from_str(TYPE_TEXT))
        .map(|s| s.to_string())
}

/// Copies the frontmost app's selection with a synthetic Cmd+C and returns it, leaving the
/// clipboard as it was. `None` when nothing was copied (no selection, or the app ignored Cmd+C).
pub fn copy_selection() -> Option<String> {
    let board = NSPasteboard::generalPasteboard();
    let before = board.changeCount();
    let snapshot = snapshot();
    input::send_command(input::KEY_C);

    let start = Instant::now();
    let mut changed = false;
    while start.elapsed() < Duration::from_millis(450) {
        sleep(Duration::from_millis(15));
        if board.changeCount() != before {
            changed = true;
            break;
        }
    }
    if !changed {
        return None;
    }
    // Some apps publish several formats one after another; let them finish.
    sleep(Duration::from_millis(40));
    let text = get_text();
    restore(&snapshot);
    text.filter(|t| !t.trim().is_empty())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Puts the user's clipboard back however the test ends.
    struct KeepUserClipboard(Snapshot);

    impl Drop for KeepUserClipboard {
        fn drop(&mut self) {
            restore(&self.0);
        }
    }

    fn types_of_first_item() -> Vec<String> {
        snapshot()
            .0
            .first()
            .map(|item| item.iter().map(|(kind, _)| kind.clone()).collect())
            .unwrap_or_default()
    }

    /// Failures never print the clipboard: the user may copy something meanwhile.
    #[test]
    #[ignore = "macOS integration: briefly replaces the clipboard"]
    fn text_round_trips_and_a_snapshot_restores_every_type() {
        let _user = KeepUserClipboard(snapshot());

        assert!(set_text("ToneSnap clipboard test ✓", true));
        assert!(
            get_text().as_deref() == Some("ToneSnap clipboard test ✓"),
            "text did not round-trip"
        );
        let saved = snapshot();
        assert!(types_of_first_item().iter().any(|t| t == TYPE_TRANSIENT));

        assert!(set_text("ToneSnap clipboard test, second", false));
        restore(&saved);
        assert!(
            get_text().as_deref() == Some("ToneSnap clipboard test ✓"),
            "restore did not bring the text back"
        );
        assert!(types_of_first_item().iter().any(|t| t == TYPE_CONCEALED));
    }
}
