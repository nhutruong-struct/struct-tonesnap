//! Fuzzes the reversible redaction in `src/redact.rs`: `Redactor::apply` must never panic, and swapping
//! the placeholders back the way the popup does (`src/lib/redaction.ts`) must give the original text.
//!
//! Input layout: the first line holds custom sensitive terms separated by commas; the rest holds the captured
//! fields (context before, selection…) separated by NUL, all redacted by one `Redactor` as in a session.

#![no_main]

use libfuzzer_sys::fuzz_target;

// Compiled by path so the target builds without Tauri, which needs GTK and WebKit on Linux.
#[path = "../../src/redact.rs"]
mod redact;

use redact::Redactor;

/// Mirrors `restore` in `src/lib/redaction.ts`.
fn restore(text: &str, entries: &[(String, String)]) -> String {
    entries
        .iter()
        .fold(text.to_string(), |acc, (placeholder, original)| {
            acc.replace(placeholder.as_str(), original)
        })
}

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let (terms, body) = input.split_once('\n').unwrap_or(("", input));
    let terms: Vec<String> = terms.split(',').map(str::to_string).collect();
    let fields: Vec<&str> = body.split('\0').collect();

    let mut redactor = Redactor::new(&terms);
    let redacted: Vec<String> = fields.iter().map(|field| redactor.apply(field)).collect();

    // Text that already contains "[" can hold something shaped like a placeholder, which no restore can
    // tell apart from a real one; the round trip is only promised without it.
    if body.contains('[') {
        return;
    }
    for (field, clean) in fields.iter().zip(&redacted) {
        assert_eq!(restore(clean, &redactor.entries), *field);
    }
});
