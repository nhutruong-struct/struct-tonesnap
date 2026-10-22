# Popup

> **Status:** implemented (Phase 0).
> **Look and feel:** [ui-design.md](ui-design.md).
> **Source:** `src-tauri/src/window.rs`, `session.rs`, `src/popup/` (`Popup.tsx`, `usePopupState.ts`, `suggestions.ts`, `SuggestionList.tsx`, `PreviewPanel.tsx`, `ErrorPanel.tsx`), `src/lib/i18n.ts`.

The popup shows one version of the selection per tone as the model writes them, lets the user Apply or Copy one, and explains what went wrong when something does.

## Opening and placement

- **POP-01** After a successful capture, Rust creates a **session**, sends it to the popup (`session` event) and shows the popup; then it starts the request, unless preview mode is on ([privacy.md](privacy.md) PRV-20).
- **POP-02** Size: 540 × 500 logical pixels for suggestions, 540 × 120 for a notice, scaled by the monitor's DPI.
- **POP-03** The popup is anchored to the selection's rectangle when the app reports it, otherwise to the pointer. It goes 10 px **below** the anchor; if it would run past the bottom of the work area it goes **above**; if that runs past the top, it is pinned to the bottom of the work area. Horizontally it starts at the anchor's left edge, kept inside the work area. The monitor is the one under the selection (or the pointer). Notices always open at the pointer.
- **POP-04** The popup is shown and focused, so its keys work at once.

## States

The popup is driven entirely by events from Rust. Each session resets it.

| Phase | When | Shows |
|---|---|---|
| `idle` | No session yet | "Select some text, then press the hotkey." |
| `preview` | Session with preview mode on | The data to be sent, with *Send (Enter)* and *Cancel* |
| `streaming` | A request started | The variants filling in, "Writing…" |
| `done` | The request finished | All variants ready; timings and model in the footer |
| `error` | The request failed | An error panel with *Retry (R)* and, when useful, *Open Settings* |

- **POP-10** A notice replaces whatever the popup shows; the next session clears it.
- **POP-11** Events of a request the popup is not showing are ignored: `request-started` must belong to the current session, and every later event must belong to the current request. A replaced or cancelled request therefore cannot mix its text into the new one.
- **POP-12** `request-started` clears the previous text and outcome, so *Retry* and R start from a clean slate.

## Header and footer

- **POP-20** The header shows the app icon and name. Under it, chips show: where the selection came from ("Read directly" / "Via clipboard"), where the context came from ("Context: surrounding text" / "Context: screen OCR" / "No context"), how many values were masked (only when some were), and "Read-only – use Copy" when the field is not editable.
- **POP-21** Header buttons: the gear opens Settings (and hides the popup), the cross closes the popup.
- **POP-22** *Removed.* The popup no longer quotes the original selection: the user has just selected it in the other app. Preview mode still shows the text that will be sent (PRV-20).
- **POP-23** The footer lists the keys (POP-40) as key caps with their effect: `1–9` Apply, `Shift+1–9` Copy, `R` Regenerate, `Esc` Close. For a read-only field `1–9` reads Copy and `Shift+1–9` is left out; while previewing only `Esc` is shown. On the right: "Writing…" while streaming; `first token / total · model` (e.g. `0.8s / 2.4s · gemini-3.8-flash`) once done.
- **POP-24** *Removed.* The "translated from Vietnamese" hint made no sense once the target language can be chosen; the options bar (POP-25) says what the versions are.
- **POP-25** Under the chips, an options bar has two segmented controls: the **target language** (Vietnamese, English, Japanese) and the **mode** (Translate, Rewrite). A session opens with the defaults from Settings ([settings.md](settings.md)); the choice made here applies to this session only. Choosing another value updates the bar at once and sends a new request with both values (`send_request(targetLanguage, mode)`), which replaces the running one (POP-11); choosing the value already selected does nothing. While previewing, a choice is only stored and shown in the preview (rows *Target language* and *Mode*); *Send*, Enter, R and *Retry* always send the values on screen, so Rust never works from a stale choice.

## Suggestions

- **POP-30** The model's JSON is parsed while it streams; an incomplete document yields what is there so far, and broken output yields nothing (no crash).
- **POP-31** There is one row per **requested** tone, in order, numbered 1–9. A row shows a placeholder skeleton until the model starts that tone, then its text.
- **POP-32** A variant is **ready** (can be applied or copied) once the model has moved on to the next tone, or the request has finished (the last tone is therefore ready when the request finishes), and its text is not empty. Until then its buttons are disabled and its keys do nothing, so a half-written sentence is never pasted.
- **POP-33** Placeholders are restored in the text before display ([privacy.md](privacy.md) PRV-14).
- **POP-34** A ready variant that lost a masked value of the selection ([privacy.md](privacy.md) PRV-17) shows a warning under its text naming the lost values. Its *Apply* button is disabled and its number key does nothing; *Copy* and `Shift`+number still copy it, so the user can check it before using it.
- **POP-35** Each row has *Apply* and *Copy*. When the field is read-only (`editable === false`) only *Copy* is shown; when editability is unknown, *Apply* is offered.

## Keys

- **POP-40**

| Key | Effect |
|---|---|
| `1`–`9` | Apply that variant (Copy when the field is read-only) |
| `Shift` + `1`–`9` | Copy that variant |
| `R` (without Ctrl) | Ask again: a new request for the same session (not in preview) |
| `Enter` | In preview: send |
| `Esc` | Close the popup |

Number keys are matched by the physical top-row digit key, so they work with Shift and on any keyboard layout; the numeric keypad is not used.

## Notices

- **POP-50** A notice is a one-line message with a *Close* button, shown in the small popup at the pointer. It closes by itself after 3.5 s.

| Notice | Message (English) |
|---|---|
| `no_selection` | Nothing is selected. Select the text you want to translate or rewrite, then press the hotkey. |
| `password_field` | ToneSnap never reads password fields. |
| `blocked_app` | ToneSnap is turned off in {app} (blocked apps in Settings). |
| `apply_failed` | Could not put the text back into the app. It is on your clipboard; press Ctrl+V to paste. (⌘V on macOS) |
| `accessibility_denied` | ToneSnap does not have the Accessibility permission, so it cannot read the selected text. Open ToneSnap Settings, Permissions. (macOS only) |

## Errors

- **POP-60** A known error code shows a translated title and, when there is one, a translated detail; otherwise the provider's message. An unknown code shows "Error {status}" and the raw message.

| Code | Title | Detail | *Open Settings* |
|---|---|---|---|
| `no_api_key` | No API key yet | Open Settings to enter your API key. | yes |
| `authentication_error` | Invalid API key | Check the key in Settings. | yes |
| `permission_error` | This key cannot use this model | (message) | yes |
| `not_found_error` | Model not found | (message) | yes |
| `rate_limit_error` | Rate limited | Try again in a few seconds (R). | |
| `overloaded_error` | The AI service is overloaded | Try again in a few minutes (R). | |
| `refusal` | The AI declined to process this text | | |
| `max_tokens` | The result was too long and got cut off | Select a shorter passage and try again. | |
| `incomplete` | The connection dropped while the AI was answering | The answer is unfinished, so it is not offered. Try again (R). | |
| `timeout` | The AI is not responding | Check your network connection and try again (R). | |
| `bad_response` | The AI's answer could not be read | A proxy or firewall may have blocked the request. Try again (R). | |
| `network` | Connection error | (message) | |

- **POP-61** Every error offers *Retry (R)*. Errors the user can fix in Settings also offer *Open Settings*.

## Closing

- **POP-70** The popup closes on Esc, ✕, Apply, Copy, *Open Settings*, *Cancel* in preview, a notice's timeout, or pressing the hotkey while the popup is in front ([capture.md](capture.md) CAP-01).
- **POP-71** It also closes once the user has really moved to another app. Focus changes in the first 400 ms after showing are ignored (the popup is still settling), and a blur is checked again 200 ms later, because dragging or clicking inside the popup briefly moves focus between the web view and its frame. It stays open while the popup or the Settings window is in front.
- **POP-72** Every way of closing cancels the running request ([ai-request.md](ai-request.md) AI-54).

**Verified by:** `window.rs` tests (below the selection, above near the bottom, inside the work area, at 2× DPI; notice JSON), `session.rs` test (session JSON), `src/popup/suggestions.test.ts` (streaming parse, broken output, readiness rules, the masked values a final variant lost), `src/popup/Popup.test.tsx` (a variant that lost a masked value copies but does not apply, an answer cut off by the connection offers no version, the key hints in the footer, restored text, number keys, Apply button, read-only offers Copy only, a half-written variant cannot be applied, events of a replaced request ignored, the language and mode bar: starting values, a new language or mode asks again with both values, the same value sends nothing, a choice while previewing is sent with the data, R repeats what is on screen, a new session starts from its own values, Esc and R, preview sends on Enter, known and unknown errors, notices replace suggestions, close by themselves or with *Close*, and a new session clears one; the header chips and its Settings and Close buttons; the empty state; a rate limit offers only Retry, a missing key points to Settings, a connection error shows its message, a cut-off answer is explained, and retrying clears the error; a missing timing shows a dash and the model comes from the session; the preview lists its rows, hides empty context and Cancel sends nothing; a number key beyond the tones does nothing). Focus behavior against real apps is manual.
