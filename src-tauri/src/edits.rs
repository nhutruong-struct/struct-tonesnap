//! The smallest replacements that turn a text into its rewrite, so an app that edits in place
//! (Word) leaves the unchanged words, and their formatting, alone.

/// Replaces `len` UTF-16 units of the old text, from `start`, with `text`. UTF-16 because that
/// is how Word counts positions.
#[derive(Debug, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub len: usize,
    pub text: String,
}

/// Above this many token pairs the comparison costs too much memory and time; the part that
/// differs is then replaced in one piece.
const MAX_PAIRS: usize = 1_000_000;

/// Above this many edits each one costs a call into the app; the span from the first to the
/// last is then replaced in one piece.
const MAX_EDITS: usize = 100;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Word,
    Space,
    Other,
}

fn kind(c: char) -> Kind {
    // Combining diacritics belong to their word (Vietnamese typed in decomposed form).
    if c.is_alphanumeric() || matches!(c, '\u{300}'..='\u{36f}') {
        Kind::Word
    } else if c.is_whitespace() && c != '\n' {
        Kind::Space
    } else {
        // A line break stays a token of its own: in Word it is the paragraph mark that holds
        // the paragraph's formatting, so changed indentation must not replace it.
        Kind::Other
    }
}

/// Words, runs of spaces, and every other character on its own.
fn tokens(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut previous = None;
    for (i, c) in text.char_indices() {
        let kind = kind(c);
        if i > start && (kind == Kind::Other || Some(kind) != previous) {
            out.push(&text[start..i]);
            start = i;
        }
        previous = Some(kind);
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Index pairs of the longest common subsequence of `a` and `b`, in order.
fn common(a: &[&str], b: &[&str]) -> Vec<(usize, usize)> {
    let width = b.len() + 1;
    // `table[i * width + j]`: length of the longest common subsequence of `a[i..]` and `b[j..]`.
    let mut table = vec![0u32; (a.len() + 1) * width];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            table[i * width + j] = if a[i] == b[j] {
                table[(i + 1) * width + j + 1] + 1
            } else {
                table[(i + 1) * width + j].max(table[i * width + j + 1])
            };
        }
    }
    let mut pairs = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if table[(i + 1) * width + j] >= table[i * width + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

/// The replacements that turn `old` into `new`, ordered from the start of `old`. Empty when the
/// texts are equal.
pub fn edits(old: &str, new: &str) -> Vec<Edit> {
    let a = tokens(old);
    let b = tokens(new);
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let suffix = a[prefix..]
        .iter()
        .rev()
        .zip(b[prefix..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (a_end, b_end) = (a.len() - suffix, b.len() - suffix);
    let middle = if (a_end - prefix) * (b_end - prefix) > MAX_PAIRS {
        Vec::new()
    } else {
        common(&a[prefix..a_end], &b[prefix..b_end])
    };
    let matches = (0..prefix)
        .map(|k| (k, k))
        .chain(middle.into_iter().map(|(i, j)| (i + prefix, j + prefix)))
        .chain((0..suffix).map(|k| (a_end + k, b_end + k)))
        // A last pair past both ends closes the final gap.
        .chain([(a.len(), b.len())]);

    // UTF-16 offset of every token of `old`, and of its end.
    let mut offsets = Vec::with_capacity(a.len() + 1);
    let mut offset = 0;
    for token in &a {
        offsets.push(offset);
        offset += utf16_len(token);
    }
    offsets.push(offset);

    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (next_i, next_j) in matches {
        if next_i > i || next_j > j {
            out.push(Edit {
                start: offsets[i],
                len: offsets[next_i] - offsets[i],
                text: b[j..next_j].concat(),
            });
        }
        i = next_i + 1;
        j = next_j + 1;
    }
    if out.len() > MAX_EDITS {
        // Both texts are the same before the first edit and after the last one.
        let first = out[0].start;
        let last = out.last().map_or(first, |e| e.start + e.len);
        let units: Vec<u16> = new.encode_utf16().collect();
        let tail = utf16_len(old) - last;
        return vec![Edit {
            start: first,
            len: last - first,
            text: String::from_utf16_lossy(&units[first..units.len() - tail]),
        }];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What applying `edits` to `old` gives, from the end so earlier offsets stay valid, the way
    /// the app applies them.
    fn apply(old: &str, edits: &[Edit]) -> String {
        let mut units: Vec<u16> = old.encode_utf16().collect();
        for edit in edits.iter().rev() {
            units.splice(
                edit.start..edit.start + edit.len,
                edit.text.encode_utf16().collect::<Vec<_>>(),
            );
        }
        String::from_utf16(&units).unwrap()
    }

    fn edit(start: usize, len: usize, text: &str) -> Edit {
        Edit {
            start,
            len,
            text: text.into(),
        }
    }

    #[test]
    fn equal_texts_need_no_edit() {
        assert!(edits("pls check my PR", "pls check my PR").is_empty());
        assert!(edits("", "").is_empty());
    }

    #[test]
    fn only_the_changed_word_is_replaced() {
        assert_eq!(
            edits("The quick brown fox", "The quick red fox"),
            [edit(10, 5, "red")]
        );
    }

    #[test]
    fn insertions_and_deletions_touch_nothing_else() {
        assert_eq!(
            edits("Please review it", "Please review it today"),
            [edit(16, 0, " today")]
        );
        assert_eq!(
            edits("Please kindly review it", "Please review it"),
            [edit(7, 7, "")]
        );
    }

    #[test]
    fn line_breaks_survive_a_change_of_indentation() {
        let old = "Steps:\n  - build\n  - test";
        let new = "Steps:\n    - build\n    - test";
        let edits = edits(old, new);
        assert!(edits.iter().all(|e| !old
            .encode_utf16()
            .skip(e.start)
            .take(e.len)
            .any(|u| u == u16::from(b'\n'))));
        assert_eq!(apply(old, &edits), new);
    }

    #[test]
    fn offsets_count_utf16_units() {
        let old = "Anh gửi 🙂 báo cáo nhé";
        let new = "Anh gửi 🙂 bản báo cáo nhé";
        let edits = edits(old, new);
        // "Anh gửi " is 8 units and the emoji 2.
        assert_eq!(edits, [edit(11, 0, "bản ")]);
        assert_eq!(apply(old, &edits), new);
    }

    #[test]
    fn decomposed_diacritics_stay_with_their_word() {
        let old = "Chu\u{301}c mư\u{300}ng";
        assert_eq!(tokens(old), ["Chu\u{301}c", " ", "mư\u{300}ng"]);
    }

    #[test]
    fn a_rewrite_applied_from_the_end_gives_the_new_text() {
        let cases = [
            (
                "pls check my PR",
                "Could you please review my pull request?",
            ),
            (
                "Hi team,\n\nthe build is broken.\nI fix it.",
                "Hi team,\n\nThe build is broken.\nI am fixing it.\n",
            ),
            ("a b c d e", "e d c b a"),
            ("", "new text"),
            ("old text", ""),
        ];
        for (old, new) in cases {
            assert_eq!(apply(old, &edits(old, new)), new, "{old:?} → {new:?}");
        }
    }

    #[test]
    fn too_many_edits_become_one() {
        let old: String = (0..300).map(|i| format!("w{i} ")).collect();
        let new: String = (0..300).map(|i| format!("v{i} ")).collect();
        let edits = edits(&old, &new);
        assert_eq!(edits.len(), 1);
        assert_eq!(apply(&old, &edits), new);
    }

    #[test]
    fn a_huge_difference_is_replaced_in_one_piece() {
        let old = "x ".repeat(1500);
        let new = "y ".repeat(1500);
        assert_eq!(apply(&old, &edits(&old, &new)), new);
    }
}
