//! Reversible redaction of sensitive values before text leaves the machine.
//!
//! Each distinct value becomes a placeholder such as `[EMAIL_1]`; the same value always maps
//! to the same placeholder within one session, so the model can keep references consistent.
//! The popup swaps placeholders back to the originals before showing or pasting a suggestion.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

struct Rule {
    kind: &'static str,
    re: Regex,
    accept: fn(&str) -> bool,
}

fn always(_: &str) -> bool {
    true
}

fn luhn(candidate: &str) -> bool {
    let digits: Vec<u32> = candidate.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&digits.len()) {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| {
            if i % 2 == 1 {
                let x = d * 2;
                if x > 9 {
                    x - 9
                } else {
                    x
                }
            } else {
                d
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

fn looks_like_token(candidate: &str) -> bool {
    candidate.chars().any(|c| c.is_ascii_digit())
        && candidate.chars().any(|c| c.is_ascii_alphabetic())
}

/// Ordered by priority. When matches overlap, the one that starts first wins; for matches that
/// start at the same place, the rule listed first wins (custom terms come before all of these).
static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
    let rule = |kind, pattern: &str, accept| Rule {
        kind,
        re: Regex::new(pattern).expect("valid redaction pattern"),
        accept,
    };
    vec![
        rule(
            "SECRET",
            r"\b(?:sk-ant-[A-Za-z0-9_\-]{10,}|sk-[A-Za-z0-9_\-]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|xox[abprs]-[A-Za-z0-9\-]{10,}|AKIA[0-9A-Z]{16}|AIza[0-9A-Za-z_\-]{35})",
            always,
        ),
        rule(
            "SECRET",
            r"\beyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}",
            always,
        ),
        rule(
            "URL",
            r#"\bhttps?://[^\s<>"'`]*[^\s<>"'`.,;:!?)\]]"#,
            always,
        ),
        rule(
            "EMAIL",
            r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,}\b",
            always,
        ),
        rule("CARD", r"\b(?:\d[ \-]?){12,18}\d\b", luhn),
        // Vietnamese citizen ID (CCCD) is 12 digits.
        rule("ID", r"\b\d{12}\b", always),
        rule("PHONE", r"(?:\+84|\b0)(?:[ .\-]?\d){9,10}\b", always),
        rule("PHONE", r"\+\d[\d \-().]{7,}\d", always),
        rule("IP", r"\b(?:\d{1,3}\.){3}\d{1,3}\b", always),
        rule("SECRET", r"\b[A-Za-z0-9_\-]{32,}\b", looks_like_token),
    ]
});

fn term_pattern(term: &str) -> Option<Regex> {
    let term = term.trim();
    if term.is_empty() {
        return None;
    }
    let is_word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    let start = if is_word(term.chars().next()) {
        r"\b"
    } else {
        ""
    };
    let end = if is_word(term.chars().last()) {
        r"\b"
    } else {
        ""
    };
    Regex::new(&format!("(?i){start}{}{end}", regex::escape(term))).ok()
}

pub struct Redactor {
    custom: Vec<Regex>,
    counters: HashMap<&'static str, usize>,
    by_value: HashMap<String, String>,
    /// (placeholder, original) pairs in the order they were created.
    pub entries: Vec<(String, String)>,
}

impl Redactor {
    pub fn new(custom_terms: &[String]) -> Self {
        Self {
            custom: custom_terms
                .iter()
                .filter_map(|t| term_pattern(t))
                .collect(),
            counters: HashMap::new(),
            by_value: HashMap::new(),
            entries: Vec::new(),
        }
    }

    fn placeholder(&mut self, kind: &'static str, value: &str) -> String {
        if let Some(existing) = self.by_value.get(value) {
            return existing.clone();
        }
        let n = self.counters.entry(kind).or_insert(0);
        *n += 1;
        let placeholder = format!("[{kind}_{n}]");
        self.by_value.insert(value.to_string(), placeholder.clone());
        self.entries.push((placeholder.clone(), value.to_string()));
        placeholder
    }

    pub fn apply(&mut self, text: &str) -> String {
        // (start, end, priority, kind)
        let mut matches: Vec<(usize, usize, usize, &'static str)> = Vec::new();
        for (priority, re) in self.custom.iter().enumerate() {
            for m in re.find_iter(text) {
                matches.push((m.start(), m.end(), priority, "TERM"));
            }
        }
        let offset = self.custom.len();
        for (i, rule) in RULES.iter().enumerate() {
            for m in rule.re.find_iter(text) {
                if (rule.accept)(m.as_str()) {
                    matches.push((m.start(), m.end(), offset + i, rule.kind));
                }
            }
        }
        matches.sort_by_key(|&(start, end, priority, _)| (start, priority, usize::MAX - end));

        let mut out = String::with_capacity(text.len());
        let mut cursor = 0;
        for (start, end, _, kind) in matches {
            if start < cursor {
                continue;
            }
            out.push_str(&text[cursor..start]);
            let placeholder = self.placeholder(kind, &text[start..end]);
            out.push_str(&placeholder);
            cursor = end;
        }
        out.push_str(&text[cursor..]);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_and_reuses_placeholders() {
        let mut r = Redactor::new(&["Project Falcon".to_string()]);
        let out = r.apply(
            "Mail an@corp.vn or call 0912 345 678 about project falcon, again an@corp.vn. Key sk-ant-abc123DEF456ghi789",
        );
        assert_eq!(
            out,
            "Mail [EMAIL_1] or call [PHONE_1] about [TERM_1], again [EMAIL_1]. Key [SECRET_1]"
        );
        assert_eq!(r.entries.len(), 4);
    }

    #[test]
    fn keeps_ordinary_text() {
        let mut r = Redactor::new(&[]);
        let text = "Please review PR 1234 before Friday, it's urgent.";
        assert_eq!(r.apply(text), text);
    }

    #[test]
    fn custom_term_does_not_match_inside_words() {
        let mut r = Redactor::new(&["Arent".to_string()]);
        assert_eq!(r.apply("parent of Arent"), "parent of [TERM_1]");
    }

    #[test]
    fn card_requires_luhn() {
        let mut r = Redactor::new(&[]);
        assert_eq!(r.apply("4111 1111 1111 1111"), "[CARD_1]");
        assert_eq!(r.apply("1234 5678 9012 3456"), "1234 5678 9012 3456");
    }

    #[test]
    fn url_excludes_trailing_punctuation() {
        let mut r = Redactor::new(&[]);
        assert_eq!(
            r.apply("See https://jira.corp.local/browse/ABC-1."),
            "See [URL_1]."
        );
    }

    #[test]
    fn redacts_phones_ids_and_ips() {
        let mut r = Redactor::new(&[]);
        assert_eq!(
            r.apply("Call +84 912 345 678 or 0912.345.678, ID 001203004567, host 10.0.12.7"),
            "Call [PHONE_1] or [PHONE_2], ID [ID_1], host [IP_1]"
        );
        assert_eq!(r.apply("Office +1 (415) 555-0100"), "Office [PHONE_3]");
    }

    #[test]
    fn redacts_tokens() {
        let mut r = Redactor::new(&[]);
        let jwt = "eyJhbGciOiJIUzI1.eyJzdWIiOiIxMjM0.c2lnbmF0dXJlMTIz";
        assert_eq!(r.apply(&format!("Bearer {jwt}")), "Bearer [SECRET_1]");
        assert_eq!(
            r.apply("token ghp_abcdefghijklmnopqrstuvwx and xoxb-abcdefghijkl"),
            "token [SECRET_2] and [SECRET_3]"
        );
        assert_eq!(
            r.apply("hash 3f9a1c2b7d8e4f6a0b1c2d3e4f5a6b7c8d9e0f1a"),
            "hash [SECRET_4]"
        );
        // Long words without digits are ordinary text.
        let word = "Supercalifragilisticexpialidocious_and_more";
        assert_eq!(r.apply(word), word);
    }

    #[test]
    fn the_match_that_starts_first_wins_an_overlap() {
        let mut r = Redactor::new(&["corp.vn".to_string()]);
        // The email inside the URL stays part of the URL.
        assert_eq!(
            r.apply("Open https://corp.vn/u?mail=an@corp.vn now"),
            "Open [URL_1] now"
        );
        // The email starts before the custom term inside it.
        assert_eq!(r.apply("an@corp.vn"), "[EMAIL_1]");
        // At the same start, custom terms come before the built-in rules.
        let mut r = Redactor::new(&["Falcon-2026".to_string()]);
        assert_eq!(r.apply("Falcon-2026"), "[TERM_1]");
    }

    #[test]
    fn placeholders_are_numbered_per_kind() {
        let mut r = Redactor::new(&[]);
        r.apply("a@x.vn b@x.vn 10.0.0.1");
        assert_eq!(
            r.entries,
            [
                ("[EMAIL_1]".to_string(), "a@x.vn".to_string()),
                ("[EMAIL_2]".to_string(), "b@x.vn".to_string()),
                ("[IP_1]".to_string(), "10.0.0.1".to_string()),
            ]
        );
    }

    #[test]
    fn blank_custom_terms_are_ignored() {
        let mut r = Redactor::new(&["  ".to_string(), String::new()]);
        assert_eq!(r.apply("nothing to hide"), "nothing to hide");
    }
}
