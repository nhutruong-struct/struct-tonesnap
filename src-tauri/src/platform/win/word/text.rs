//! Word's text as the model gets it, and back. Each visible character of a selection has a
//! position in the document, but fields (a link is one) hide their code and three marks between
//! the visible characters. The map from text to positions lets edits of the text land on the
//! right positions, and keeps them out of a field's hidden parts.

use crate::edits::Edit;

/// A field inside the selection, by document position: `start` holds its opening mark,
/// `separator` the mark between its code and its result (the visible text), `end` its closing
/// mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub start: i32,
    pub separator: i32,
    pub end: i32,
}

impl Field {
    /// The field's code and marks, which the text does not show.
    fn hides(&self, position: i32) -> bool {
        (self.start..=self.separator).contains(&position) || position == self.end
    }

    /// Text inserted here would become part of the field.
    fn surrounds(&self, position: i32) -> bool {
        self.start < position && position <= self.end
    }
}

/// A replacement in document positions.
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub from: i32,
    pub to: i32,
    pub text: String,
}

/// A selection's visible text and the document position of each of its UTF-16 units.
#[derive(Debug)]
pub struct Mapped {
    /// Where the selection ends once the paragraph marks at its end are left out.
    pub end: i32,
    /// Those paragraph marks.
    pub marks: usize,
    /// The text as the model gets it: paragraph marks and line breaks become `\n`.
    pub text: String,
    start: i32,
    positions: Vec<i32>,
    fields: Vec<Field>,
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Maps Word's text of the range `start..end` (`raw`, as `Range.Text` gives it, without field
/// codes) to document positions. Paragraph marks at the end stay out, so Apply never merges
/// paragraphs. `None` when the text does not account for every position outside the fields
/// (hidden text, pictures), holds a mark other than a tab or a line break (table cells), a field
/// reaches outside the range, or nothing is left.
pub fn map(start: i32, end: i32, raw: &str, fields: Vec<Field>) -> Option<Mapped> {
    let marked = raw
        .chars()
        .any(|c| c < ' ' && !matches!(c, '\t' | '\r' | '\u{b}'));
    let outside = fields
        .iter()
        .any(|f| f.start < start || f.end >= end || f.separator <= f.start || f.end < f.separator);
    if marked || outside {
        return None;
    }
    let mut positions: Vec<i32> = (start..end)
        .filter(|&p| !fields.iter().any(|f| f.hides(p)))
        .collect();
    if positions.len() != utf16_len(raw) {
        return None;
    }
    let kept = raw.trim_end_matches(['\r', '\u{b}']);
    let kept_len = utf16_len(kept);
    if kept_len == 0 {
        return None;
    }
    // The first mark left out is visible, so it comes after every field.
    let trimmed_end = positions.get(kept_len).copied().unwrap_or(end);
    let marks = positions.len() - kept_len;
    positions.truncate(kept_len);
    Some(Mapped {
        end: trimmed_end,
        marks,
        text: kept
            .chars()
            .map(|c| if c == '\r' || c == '\u{b}' { '\n' } else { c })
            .collect(),
        start,
        positions,
        fields,
    })
}

impl Mapped {
    /// The document changes for edits of the text, in the same order. An edit inside a field's
    /// result changes it in place, so a link keeps its address and formatting. An edit that
    /// reaches across a field's edge grows to the field's whole result and replaces the field,
    /// its unchanged words included. Inserted text joins a field only between two of its
    /// characters. `None` when an insertion has nowhere to go.
    pub fn changes(&self, edits: &[Edit]) -> Option<Vec<Change>> {
        let results: Vec<(usize, usize)> = self
            .fields
            .iter()
            .map(|f| {
                (
                    self.positions.partition_point(|&p| p <= f.separator),
                    self.positions.partition_point(|&p| p < f.end),
                )
            })
            .collect();
        let mut regions: Vec<Region> = edits
            .iter()
            .enumerate()
            .map(|(i, e)| Region {
                start: e.start,
                end: e.start + e.len,
                edits: i..i + 1,
            })
            .collect();
        loop {
            let mut grew = false;
            for region in regions.iter_mut().filter(|r| r.start < r.end) {
                for &(rs, re) in &results {
                    let overlaps = region.start < re && rs < region.end;
                    let inside = rs <= region.start && region.end <= re;
                    if overlaps && !inside && (rs < region.start || region.end < re) {
                        region.start = region.start.min(rs);
                        region.end = region.end.max(re);
                        grew = true;
                    }
                }
            }
            let mut merged: Vec<Region> = Vec::with_capacity(regions.len());
            for region in regions {
                match merged.last_mut() {
                    Some(last) if last.overlaps(&region) => {
                        last.start = last.start.min(region.start);
                        last.end = last.end.max(region.end);
                        last.edits.end = region.edits.end;
                        grew = true;
                    }
                    _ => merged.push(region),
                }
            }
            regions = merged;
            if !grew {
                break;
            }
        }
        regions
            .iter()
            .map(|region| self.change(region, &edits[region.edits.clone()], &results))
            .collect()
    }

    fn change(
        &self,
        region: &Region,
        edits: &[Edit],
        results: &[(usize, usize)],
    ) -> Option<Change> {
        // The region's new text: its unchanged words and the edits' new text, in order.
        let units: Vec<u16> = self.text.encode_utf16().collect();
        let mut new = Vec::new();
        let mut at = region.start;
        for edit in edits {
            new.extend_from_slice(&units[at..edit.start]);
            new.extend(edit.text.encode_utf16());
            at = edit.start + edit.len;
        }
        new.extend_from_slice(&units[at..region.end]);
        let text = String::from_utf16_lossy(&new);

        if region.start == region.end {
            // Between the characters on each side, outside every field if there is room.
            let low = match region.start {
                0 => self.start,
                s => self.positions[s - 1] + 1,
            };
            let high = self
                .positions
                .get(region.start)
                .copied()
                .unwrap_or(self.end);
            let outside = (low..=high).find(|&p| !self.fields.iter().any(|f| f.surrounds(p)));
            // No room only between two characters of the same field: the text joins it.
            let at = outside.or((low == high).then_some(low))?;
            return Some(Change {
                from: at,
                to: at,
                text,
            });
        }
        let mut from = self.positions[region.start];
        let mut to = self.positions[region.end - 1] + 1;
        // A field whose whole result is replaced along with text around it goes entirely.
        for (f, &(rs, re)) in self.fields.iter().zip(results) {
            let covered = region.start <= rs && re <= region.end;
            let wider = region.start < rs || re < region.end;
            if (rs < re && covered && wider) || (from < f.start && f.end < to) {
                from = from.min(f.start);
                to = to.max(f.end + 1);
            }
        }
        Some(Change { from, to, text })
    }
}

/// A span of the text that one change replaces, and the edits it holds.
struct Region {
    start: usize,
    end: usize,
    edits: std::ops::Range<usize>,
}

impl Region {
    fn overlaps(&self, other: &Region) -> bool {
        let within = |r: &Region, p: usize| r.start < p && p < r.end;
        match (self.start == self.end, other.start == other.end) {
            (true, true) => false,
            (true, false) => within(other, self.start),
            (false, true) => within(self, other.start),
            (false, false) => self.start < other.end && other.start < self.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "see the release notes now" at position 100, with "release notes" a link whose code
    /// "HYPERLINK x" sits before it: opening mark at 108, code 109..120, separator 120, result
    /// 121..134, closing mark 134.
    const LINK: Field = Field {
        start: 108,
        separator: 120,
        end: 134,
    };

    fn linked() -> Mapped {
        map(100, 139, "see the release notes now", vec![LINK]).expect("maps")
    }

    fn edit(start: usize, len: usize, text: &str) -> Edit {
        Edit {
            start,
            len,
            text: text.into(),
        }
    }

    fn change(from: i32, to: i32, text: &str) -> Change {
        Change {
            from,
            to,
            text: text.into(),
        }
    }

    #[test]
    fn without_fields_positions_follow_the_text() {
        let mapped = map(10, 26, "Hi team,\r\u{b}build\t", Vec::new()).expect("maps");
        assert_eq!(mapped.text, "Hi team,\n\nbuild\t");
        assert_eq!((mapped.end, mapped.marks), (26, 0));
        assert_eq!(
            mapped.changes(&[edit(0, 2, "Dear")]),
            Some(vec![change(10, 12, "Dear")])
        );
    }

    #[test]
    fn paragraph_marks_at_the_end_stay_out_of_the_selection() {
        let mapped = map(0, 7, "Hello\r\r", Vec::new()).expect("maps");
        assert_eq!(
            (mapped.text.as_str(), mapped.end, mapped.marks),
            ("Hello", 5, 2)
        );
        assert!(map(0, 1, "\r", Vec::new()).is_none());
    }

    #[test]
    fn text_that_does_not_account_for_every_position_is_left_to_the_paste() {
        // Hidden text or a field Word did not report.
        assert!(map(0, 30, "see page 4", Vec::new()).is_none());
        // A table cell, a field mark, a picture.
        for raw in ["a\r\u{7}", "a\u{13}b", "a\u{1}"] {
            assert!(
                map(0, utf16_len(raw) as i32, raw, Vec::new()).is_none(),
                "{raw:?}"
            );
        }
        // A field that starts before the selection.
        assert!(map(110, 139, "release notes now", vec![LINK]).is_none());
    }

    #[test]
    fn offsets_are_utf16_units() {
        let mapped = map(3, 13, "Anh gửi 🙂", Vec::new()).expect("maps");
        assert_eq!(mapped.end, 13);
    }

    #[test]
    fn a_link_hides_its_code_and_marks() {
        let mapped = linked();
        assert_eq!(mapped.text, "see the release notes now");
        assert_eq!(mapped.positions[8], 121, "r of release");
        assert_eq!(mapped.positions[21], 135, "the space after the link");
    }

    #[test]
    fn words_around_a_link_change_without_touching_it() {
        let mapped = linked();
        // "see" → "read", "now" → "today".
        assert_eq!(
            mapped.changes(&[edit(0, 3, "read"), edit(22, 3, "today")]),
            Some(vec![change(100, 103, "read"), change(136, 139, "today")])
        );
    }

    #[test]
    fn a_word_inside_the_link_changes_in_place() {
        // "notes" → "summary": the link keeps its address.
        assert_eq!(
            linked().changes(&[edit(16, 5, "summary")]),
            Some(vec![change(129, 134, "summary")])
        );
        // The whole link text replaced: still in place.
        assert_eq!(
            linked().changes(&[edit(8, 13, "changelog")]),
            Some(vec![change(121, 134, "changelog")])
        );
    }

    #[test]
    fn an_edit_across_the_edge_of_a_link_replaces_the_whole_link_and_keeps_its_words() {
        // "the release" → "our": the link goes, " notes" stays as text.
        assert_eq!(
            linked().changes(&[edit(4, 11, "our")]),
            Some(vec![change(104, 135, "our notes")])
        );
        // "notes now" → "doc today": from inside the link to past it.
        assert_eq!(
            linked().changes(&[edit(16, 9, "doc today")]),
            Some(vec![change(108, 139, "release doc today")])
        );
    }

    #[test]
    fn text_inserted_between_two_characters_of_a_link_joins_it() {
        assert_eq!(
            linked().changes(&[edit(16, 0, "new ")]),
            Some(vec![change(129, 129, "new ")])
        );
    }

    #[test]
    fn inserted_text_stays_outside_the_link() {
        let mapped = linked();
        // Before "release": in front of the opening mark, not inside the result.
        assert_eq!(
            mapped.changes(&[edit(8, 0, "latest ")]),
            Some(vec![change(108, 108, "latest ")])
        );
        // After "notes": past the closing mark.
        assert_eq!(
            mapped.changes(&[edit(21, 0, " page")]),
            Some(vec![change(135, 135, " page")])
        );
    }

    #[test]
    fn edits_that_grow_over_the_same_link_become_one() {
        // "the release" → "our" and "notes now" → "doc today" both reach across the link.
        let changes = linked().changes(&[edit(4, 11, "our"), edit(16, 9, "doc today")]);
        assert_eq!(changes, Some(vec![change(104, 139, "our doc today")]));
    }
}
