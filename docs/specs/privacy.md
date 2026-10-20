# Privacy and data handling

> **Status:** implemented (Phase 0). The enterprise gateway and its second redaction pass are Phase 4.
> **Source:** `src-tauri/src/redact.rs`, `session.rs` (`redacted_payload`), `secrets.rs`, `platform/win/clipboard.rs`, `llm/`, `src/lib/redaction.ts`, `src/popup/PreviewPanel.tsx`, `scripts/harness/` (the privacy, network and secrets rules).

Users select text from work chats, emails and documents. ToneSnap treats that text as confidential: it goes to exactly one place (the chosen model), masked, and nowhere else.

## Where user text goes

- **PRV-01** User text (the selection, its context, OCR text, suggestions) is sent only in the model request. It is never logged, written to disk, or sent anywhere else. There is no telemetry.
- **PRV-02** The request goes only to the host of the chosen provider: `generativelanguage.googleapis.com` (Gemini, the default), `api.groq.com` (Groq), `openrouter.ai` (OpenRouter) or `api.anthropic.com` (Claude). There is no local model: the text always leaves the PC for that provider.
- **PRV-03** Requests use the operating system's TLS stack, so they work behind TLS-inspecting proxies such as Netskope.
- **PRV-09** Free tiers (Gemini, Groq, OpenRouter free models) usually let the provider use the content to improve its products. Settings warns about this next to the key field; users with private text should use a paid plan.
- **PRV-04** No other host is contacted. The web pages cannot make network calls at all (APP-23), and the harness rejects code that names any other host. Adding an endpoint is a privacy decision that needs the user's approval.
- **PRV-05** The current session (original selection, payload, placeholder pairs) is kept in memory only, and only until the next session replaces it. Nothing survives a restart.
- **PRV-06** Screenshots for OCR are processed in memory on the device and discarded; only the recognized text, redacted, is sent ([capture.md](capture.md) CAP-11).
- **PRV-07** Blocked apps and password fields are never read ([capture.md](capture.md) CAP-02, CAP-05). The floating icon does not appear there either (TRG-21).
- **PRV-08** Selected text is data, never instructions. The prompt says so, closing tags inside the data are neutralized, and the model can only return JSON in a fixed schema: it cannot take any action ([ai-request.md](ai-request.md) AI-04, AI-10).

## Redaction

When `redact` is on (default), sensitive values are replaced by placeholders before the payload is built, and restored for display and Apply.

- **PRV-10** Redaction applies to the context before, the selection, the context after and the screen text. The app name, target language, mode and tones are not redacted. With `redact` off the text is sent as captured and there are no placeholders.
- **PRV-11** Each distinct value becomes `[KIND_n]`, numbered per kind in order of appearance (`[EMAIL_1]`, `[EMAIL_2]`, `[PHONE_1]`). One redactor serves the whole session, so the same value gets the same placeholder in every field and the model can keep references consistent.
- **PRV-12** What is detected, in priority order:

| Kind | Detected | Notes |
|---|---|---|
| `TERM` | The user's own terms (`customSensitiveTerms`) | Case-insensitive; matched as whole words when the term starts/ends with a letter, digit or `_` (so `Arent` does not match inside `parent`); blank terms ignored |
| `SECRET` | Known key and token formats | Anthropic/OpenAI-style `sk-` keys, GitHub tokens (`ghp_`, `github_pat_`…), Slack tokens (`xox?-`), AWS access keys (`AKIA…`), Google API keys (`AIza…`) |
| `SECRET` | JWTs | Three base64url segments starting with `eyJ` |
| `URL` | `http://` and `https://` links | Trailing punctuation (`.`, `,`, `)`…) stays outside |
| `EMAIL` | Email addresses | |
| `CARD` | 13–19 digits, optionally grouped by spaces or dashes | Only when the Luhn checksum passes |
| `ID` | Exactly 12 digits | Vietnamese citizen ID (CCCD) |
| `PHONE` | Vietnamese numbers: `+84` or a leading `0`, then 9–10 digits with optional separators | |
| `PHONE` | International numbers starting with `+` | |
| `IP` | IPv4 addresses | |
| `SECRET` | Any run of 32+ letters, digits, `_` or `-` that contains both letters and digits | Hashes, opaque tokens; long words without digits are left alone |

- **PRV-13** When matches overlap, the one that starts first wins; at the same start, the higher-priority rule wins (custom terms first); then the longer match. Example: an email inside a URL stays part of the URL.
- **PRV-14** Restoring swaps every placeholder back to its original value in the variant text. Apply and Copy always use the restored text. Placeholders are bracketed, so `[EMAIL_1]` and `[EMAIL_10]` cannot be confused.
- **PRV-15** The system prompt tells the model to keep placeholders exactly as written ([ai-request.md](ai-request.md) AI-03).
- **PRV-16** The popup header shows how many values were masked ("N sensitive items masked"). The popup does not show the original selection; the suggestions it shows have the masked values restored, locally (PRV-14).

## Preview before sending

- **PRV-20** With `previewBeforeSend` on, the popup shows exactly what will be sent (app, context before, selection, context after, screen text, all redacted) and **nothing is sent** until the user presses *Send* or Enter. *Cancel* or Esc closes the popup without sending. Empty context rows are hidden.

## Clipboard hygiene

- **PRV-30** Reading the selection with Ctrl+C restores the user's previous clipboard afterwards ([capture.md](capture.md) CAP-08). Apply does the same after pasting ([apply-copy.md](apply-copy.md) APL-04).
- **PRV-31** Text ToneSnap puts on the clipboard only to paste it is marked as transient: excluded from clipboard monitors, from Windows clipboard history and from cloud clipboard sync. Text the user copies on purpose (Copy, or the fallback when Apply fails) is a normal clipboard entry.
- **PRV-32** A snapshot keeps every clipboard format whose data is plain memory. Formats that are GDI handles, metafiles, owner-drawn or private are skipped; bitmaps survive through their DIB formats.
- **PRV-33** When the selection is read with Ctrl+C (CAP-08, only when UI Automation gives none), the app makes the copy, so its text goes through the clipboard like any copy and may stay in Windows clipboard history or cloud sync: ToneSnap cannot mark a copy the app makes as transient (PRV-31 covers only its own paste).

## Secrets

- **PRV-40** Each provider's API key is stored only in Windows Credential Manager (service `ToneSnap`, accounts `anthropic_api_key`, `gemini_api_key`, `groq_api_key`, `openrouter_api_key`). It is never written to the settings file, logs or tests, and the UI never reads it back: it only learns whether a key exists.
- **PRV-41** Debug builds fall back to the provider's environment variable (`ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `GROQ_API_KEY`, `OPENROUTER_API_KEY`) when no key is stored, for local development. Release builds ignore it: developer tools such as Claude Code read the same variable, and ToneSnap must not bill a key the user never gave it, or report it as saved in Settings.
- **PRV-42** Saving an empty key deletes the stored one; a missing key is not an error.
- **PRV-43** Uninstalling with the `.exe` installer and *Delete the application data* ticked deletes every provider's key ([app-shell.md](app-shell.md) APP-41). The uninstaller deletes each credential by its target name (`<account>.ToneSnap`, as the keyring crate files it) and never reads it. The `.msi` uninstaller leaves the keys (APP-42).

**Verified by:** `redact.rs` tests (placeholders reused, ordinary text kept, whole-word custom terms, Luhn, URL punctuation, phones/IDs/IPs, tokens, overlap priority, per-kind numbering, blank terms), `session.rs` tests (consistent placeholders across fields, redaction off), `src/lib/redaction.test.ts` (restore, prefix-sharing placeholders), `src/popup/Popup.test.tsx` (restored text shown and applied, preview sends on Enter), `src/settings/SettingsPage.test.tsx` (the API key goes to the credential store, never into the settings), `clipboard.rs` integration test (transient markers, restore of every format), the harness `privacy`, `network` and `secrets` rules.

## Not implemented

- Encrypted local history (Phase 2; nothing is stored today).
- Enterprise mode (Phase 4): a self-hosted gateway with SSO, policies, a second redaction pass and metadata-only logs; providers limited to zero-data-retention options.
