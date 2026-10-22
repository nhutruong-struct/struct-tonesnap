# Capture: selection and context

> **Status:** implemented on Windows (Phase 0). Word COM and the browser extension are Phase 2. On macOS (Phase 3) the selection and context are read through the Accessibility API; the clipboard fallback and OCR are not built yet (CAP-18 to CAP-23).
> **Source:** `src-tauri/src/capture.rs` (the steps, the same on every platform), `platform/mod.rs` (the facade it calls: `foreground`, `read_focused`, `copy_selection`, `recognize_text`); Windows: `platform/win/mod.rs`, `uia.rs`, `clipboard.rs`, `input.rs`, `ocr.rs`, `window.rs`, `com.rs`; macOS: `platform/mac/mod.rs`, `ax.rs`, `window.rs`, `cf.rs`, `ocr.rs`.

Capture reads what the user selected in the foreground app and some text around it. It tries the cheapest and most precise source first: UI Automation, then the clipboard for the selection, then on-device OCR for context. `capture.rs` runs these steps once for every platform; each platform only answers the facade's questions (CAP-18 to CAP-23 for macOS).

## Result

A capture produces either a **Captured** record or a refusal.

| Field | Meaning |
|---|---|
| `app` | Friendly app name (CAP-03) |
| `selection` | The selected text as read; for a multi-line selection, with its indentation read paragraph by paragraph when the single read lost it (CAP-17) |
| `before`, `after` | Text around the selection from UI Automation, up to `contextChars` each |
| `screen_text` | Text recognized on screen near the selection, when OCR was used |
| `editable` | `true`/`false` when the control says, `null` when it does not |
| `bounds` | The selection's screen rectangle, when the app reports it |
| `cursor` | Pointer position at capture time |
| `source` | Where the selection came from: `uia` or `clipboard` |
| `context_source` | Where the context came from: `uia`, `ocr` or `none` |
| `target` | The app to paste back into, kept for Apply and never sent to the UI: an opaque `platform::Target` (the foreground window on Windows, the process id on macOS) |

| Refusal | What the user sees |
|---|---|
| `OwnWindow` | Nothing: the popup closes |
| `BlockedApp(name)` | Notice "ToneSnap is turned off in {app}" |
| `PasswordField` | Notice "ToneSnap never reads password fields" |
| `NoSelection` | Notice "Nothing is selected…" |
| `AccessibilityDenied` | macOS only: notice "ToneSnap does not have the Accessibility permission…" (the system prompt is shown the first time) |

## Before reading anything

- **CAP-01** If the foreground window belongs to ToneSnap (the popup or Settings), the capture stops and the popup closes, stopping its request. Pressing the hotkey while the popup is in front therefore closes it.
- **CAP-02** If the foreground process is in `blockedApps` (case-insensitive executable name, e.g. `keepass.exe`), the capture stops **before any text is read** and a notice names the app. Default list: `keepass.exe`, `keepassxc.exe`, `1password.exe`, `bitwarden.exe`, `lastpass.exe`.
- **CAP-03** The app name is derived from the executable: a fixed table maps common apps (`winword.exe` → Microsoft Word, `outlook.exe`/`olk.exe` → Outlook, `ms-teams.exe`/`teams.exe` → Microsoft Teams, `slack.exe` → Slack, `chrome.exe` → Google Chrome, `msedge.exe` → Microsoft Edge, Zalo, Telegram, Discord, Notion, Obsidian, VS Code, Office apps, Acrobat, Notepad, Firefox); any other app shows its executable name without `.exe`. The model uses it to judge the register ([ai-request.md](ai-request.md) AI-02).
- **CAP-04** Before reading, ToneSnap waits up to 600 ms for Ctrl, Shift, Alt and Win to be released. The user is usually still holding the hotkey's modifiers; a synthetic Ctrl+C sent then would arrive as, for example, Ctrl+Alt+C.

## Selection

- **CAP-05** UI Automation is asked first, for the focused element. If the element is a password field, the capture is refused at once with nothing read.
- **CAP-06** ToneSnap looks for the Text pattern on the focused element and up to 5 of its ancestors (many apps put it on a container). The selection is the first selected range; empty or whitespace-only counts as no selection.
- **CAP-07** Editability, in this order: the focused element's Value pattern read-only flag when present; otherwise an Edit control counts as editable; otherwise the selected text's own read-only text attribute (`UIA_IsReadOnlyAttributeId`): "not read-only" means editable, and "mixed" means not editable; otherwise unknown (`null`). Unknown still offers Apply. The attribute matters in Chromium (Chrome, Edge, Slack): the focused element of a page, or of a rich-text field (a `contenteditable` group), has no Value pattern, but the text range answers "not read-only" inside an editable field and "mixed" for page text that cannot be edited, so Apply is no longer offered for the latter. Seen with a read-only probe on Edge (2026-10-19): a paragraph, a textarea, a `contenteditable` div and an input gave "mixed", "not read-only", "not read-only", "not read-only" for the text range of their elements.
- **CAP-08** If UI Automation gives no selection, ToneSnap copies it: it snapshots the clipboard, sends Ctrl+C, waits up to 450 ms for the clipboard to change (plus 40 ms for apps that publish several formats), reads the text and **restores the previous clipboard**. If nothing changed or the copied text is blank, the capture is refused with `NoSelection`. The automatic popup never takes this step: when UI Automation gives no selection, or a shorter one than `autoMinChars`, it leaves the selection to the floating icon ([trigger.md](trigger.md) TRG-43, TRG-44).
- **CAP-17** Indentation. Chromium, as seen in Slack (other Chromium-based apps likely behave the same, not yet checked), moves the spaces that start a line elsewhere when a range of several lines is read in one call, but keeps them when the range is read one paragraph at a time. So when the single read has more than one line and no later line starts with whitespace, ToneSnap also reads the selection paragraph by paragraph (each part cut to the selection) and joins the parts with line breaks. That read runs before the popup appears, so it stops at 100 paragraphs or 300 ms, and the single read is kept. It uses that text only when it has the same words on the same lines and more whitespace at the start of its lines than the single read; otherwise the single read is kept, so a provider that splits paragraphs differently changes nothing. The single read's final line breaks are kept, since Apply replaces them too. This is all UI Automation: no keystroke, no clipboard. Found with a read-only probe on Slack (2026-10-08): the single read had no line indented, the paragraph read had the four spaces of each nested line.
- **CAP-09** UI Automation calls time out (connection 1.5 s, transaction 2 s) so a hung app cannot freeze ToneSnap.

## Context

Context helps the model fit the rewrite to the conversation. It is never rewritten.

- **CAP-10** UI Automation context is used when the selection came from UI Automation and the text around it is not blank: up to `contextChars` characters before (the last ones) and after (the first ones). Default 600.
- **CAP-11** Otherwise, if `ocrFallback` is on (default), ToneSnap captures part of the screen and recognizes it on the device. The image never leaves the machine and is not stored; only the recognized text is used, after redaction.
- **CAP-12** OCR area: the selection's rectangle padded by 420 × 200 logical pixels on each side, or, when the app does not report the rectangle, a band 840 px to each side and 200 px above and below the pointer. Always clipped to the monitor. Pixels are scaled by the monitor's DPI.
- **CAP-13** The image is upscaled 2× before recognition when it stays within the OCR engine's size limit (small UI text recognizes much better). The engine is English (`en-US`), falling back to the user's profile languages.
- **CAP-14** OCR output is trimmed to the part around the selection: the first 24 non-space characters of the selection are searched for in the recognized text, ignoring case and whitespace (OCR text differs slightly from the real text). If found (at least 6 characters to match), `contextChars` characters before the match start and after its end are kept; if not, the first `2 × contextChars` characters. A blank result means no context.
- **CAP-15** When neither source gives context, `context_source` is `none` and the request is made with the selection alone.

## Thread safety

- **CAP-16** COM (UI Automation, OCR) runs on short-lived dedicated MTA threads, never on Tauri's threads. The process MTA is kept alive for the app's lifetime, because tearing it down while the `windows` crate caches the OCR factory made a second OCR on a new thread crash the app (fixed 2026-10-07).

## macOS (Phase 3, in progress)

The same record and refusals as above, filled from `platform/mac/`.

- **CAP-18** The frontmost app is read first (`NSWorkspace`). If it is ToneSnap, the capture stops as in CAP-01. The process name is the lower-case executable name (`1password`, `textedit`); `blockedApps` matching ignores a trailing `.exe` on both sides, so CAP-02's list also blocks the macOS apps. The app name is the localized name the user sees. `target` holds the process id.
- **CAP-19** The focused element of that app is asked through the Accessibility API (`AXFocusedUIElement`). An element with the subrole `AXSecureTextField` is refused as a password field before anything is read. Calls time out after 1.5 s. The first time an app is read, ToneSnap sets its `AXManualAccessibility` attribute so Electron and Chromium apps build their accessibility tree; the first read right after may still find nothing, and CAP-22 covers it. Without the Accessibility permission the capture is refused before anything is read with `AccessibilityDenied`, and the system's prompt is shown the first time it happens ([settings.md](settings.md) SET-70).
- **CAP-20** The selection is `AXSelectedText`; when that is empty but `AXSelectedTextRange` is not, the text of the range. Context is cut from `AXValue` around `AXSelectedTextRange`, which counts UTF-16 units, so a cut never splits a surrogate pair, and a range that does not fit the text gives no context. Documents over 500 000 units give none. Without context, `context_source` is `none` (or OCR, CAP-23). `source` is reported as `uia` on both systems.
- **CAP-21** Editable is `true` when `AXValue` can be set; `false` only for a text field, text area or combo box that refuses it; otherwise unknown. Bounds come from `AXBoundsForRange` of the selection, converted to physical pixels (points times the screen's scale). 
- **CAP-22** Like CAP-04 and CAP-08: before reading, ToneSnap waits up to 600 ms for Shift, Control, Option and Command to be released. If the Accessibility API gives no selection, it snapshots the clipboard, posts Cmd+C, waits up to 450 ms for the pasteboard's change count to move (plus 40 ms), reads the text and restores every item and type. `source` is then `clipboard`. A copy of nothing or of blank text is `NoSelection`.
- **CAP-23** OCR context on macOS (`platform/mac/ocr.rs`), under the same conditions as CAP-11 and with the same area and trimming (CAP-12, CAP-14): ScreenCaptureKit (`SCScreenshotManager`, macOS 14 or later) captures the area of the display holding it, without the cursor and at the display's pixel scale, and Vision (`VNRecognizeTextRequest`, accurate level, language correction, automatic language detection) reads it; lines are ordered top to bottom. Without the Screen Recording permission (`CGPreflightScreenCaptureAccess`, checked without prompting), or when a call takes over 3 s, there is no OCR context. Vision's first recognition after a boot can take about a minute, so a one-time recognition of a small patch runs in the background at startup when the permission is already granted. The image never leaves the machine and is not stored.

**Verified by:** `capture.rs` tests (app names, context cut by characters, OCR trimming, OCR area with and without bounds, near the edge; the capture steps against a fake desktop: the accessibility API first, context cut to `contextChars`, the clipboard and OCR filling in, OCR off, nothing selected, a password field neither read nor copied, Auto mode never copying and leaving short selections alone, a blocked app named and left alone, our own window, a missing permission, the name the OS gives), `uia.rs` tests (the paragraph read brings back indentation, ends with any line break, a read that differs or adds nothing is ignored, the final line break is kept, which text may have lost its indentation) and integration tests on a real Edit control (selection with context, a multi-line selection starting and ending inside lines is read paragraph by paragraph in several parts that cover it exactly, and keeps the single read, nothing selected, password refused), `ocr.rs` integration tests (rendered text recognized, a second call on a new thread after a failure, empty rectangle), `clipboard.rs` (format filtering, snapshot and restore), `platform/win/mod.rs` input tests on request (Ctrl+C copies the selection and keeps the clipboard; nothing copied without a selection). macOS: `platform/mac/ax.rs` tests (context on each side, ends of the text, a range that does not fit, UTF-16 units and surrogate pairs, Vietnamese text), `cf.rs` (string round trip), `window.rs` (Cocoa to top-down coordinates, scale, the monitor for a point), `settings.rs` (macOS executables match `blockedApps`), `ocr.rs` integration test (the screen is captured and read; run with `--ignored`; the first run took 62 s while Vision prepared its model, later runs under a second). The Accessibility reads on real apps are manual: libtest cannot run the AppKit calls on the main thread, so they were checked with the app itself on a TextEdit document (2026-10-21: a drag-selected range, its context and editability). `platform/mac/mod.rs` `input_tests` (on request, `cargo test input_tests -- --ignored`): Cmd+C returns the selection and Cmd+V replaces it, the clipboard kept both times. Reading real third-party apps, including Slack's paragraphs (CAP-17), is manual and recorded in the compatibility table of `docs/PLAN.md`.

## Not implemented

- Per-app integrations (Phase 2): Word COM for formatting-preserving replace, a browser extension over native messaging for Gmail / Google Docs.
- Sending the screenshot itself to the model (Phase 2, off by default, with sensitive regions masked).
