# ToneSnap specifications

These pages describe what the app **does today**: its behavior, business rules and the reasons behind them. They are written for two readers:

- a person who wants to know how a feature behaves without reading the code;
- an AI agent that must understand the logic before changing it.

`docs/PLAN.md` says what is decided and what is planned; these specs say how the built parts behave. When the two disagree, the code is the truth and the spec is a bug to fix.

## Pages

| Page | Covers | Prefix |
|---|---|---|
| [app-shell.md](app-shell.md) | Startup, tray, single instance, windows, quitting, interface language, uninstalling | `APP` |
| [trigger.md](trigger.md) | What opens the popup: the hotkey, the floating icon, trigger modes | `TRG` |
| [capture.md](capture.md) | Reading the selection and its context: UI Automation, clipboard, OCR; refusals | `CAP` |
| [privacy.md](privacy.md) | What leaves the machine, redaction, preview, clipboard hygiene, secrets | `PRV` |
| [ai-request.md](ai-request.md) | Prompt, output schema, the Gemini, Groq, OpenRouter and Claude providers, streaming, errors | `AI` |
| [popup.md](popup.md) | The popup: placement, states, suggestions, keys, notices, errors, closing | `POP` |
| [apply-copy.md](apply-copy.md) | Putting a suggestion back into the app, or onto the clipboard | `APL` |
| [settings.md](settings.md) | Every setting, the Settings window, saving and loading, the API key | `SET` |
| [ui-design.md](ui-design.md) | How both windows look: design tokens, controls, popup and Settings layout | `UI` |
| [contracts.md](contracts.md) | Rust ⇄ TypeScript commands, events, data shapes and error codes | `CON` |

## The product in one paragraph

ToneSnap sits in the Windows tray. The user selects text in any app (English, Vietnamese or a mix) and presses a hotkey (default `Ctrl+Alt+E`) or clicks a small floating icon. ToneSnap reads the selection and some surrounding text, masks sensitive values, and asks a cloud AI model (Gemini's free tier by default; Groq, OpenRouter or the paid Claude API as options) for one English version per configured tone, with short explanations of the mistakes in Vietnamese (or English). The popup streams the versions in as they are written. **Apply** pastes the chosen version over the selection in the original app; **Copy** puts it on the clipboard.

## End-to-end flow

```
Hotkey pressed / floating icon clicked                         trigger.md
  → foreground app checks: own window? blocked app?            capture.md CAP-01..03
  → read selection: UI Automation, else Ctrl+C                 capture.md CAP-04..09, CAP-17
  → refuse password fields / empty selections                  capture.md CAP-05, CAP-08
  → context: UIA text around it, else on-device OCR            capture.md CAP-10..14
  → redact sensitive values (reversible)                       privacy.md PRV-10..
  → Session created and sent to the popup; popup shown         popup.md POP-01..
  → preview mode? wait for Send : start the request            privacy.md PRV-20.., popup.md
  → provider streams JSON; popup parses it as it arrives       ai-request.md, popup.md
  → Apply: refocus the app, paste, restore the clipboard       apply-copy.md
```

## Glossary

| Term | Meaning |
|---|---|
| Selection | The text the user highlighted in another app. |
| Context | Text just before and after the selection (from UI Automation), or text recognized on screen near it (OCR). Used only to understand the situation; never rewritten. |
| Session | One use of ToneSnap: the captured selection, its redacted payload and the window to paste back into. Only the latest session exists, in memory. |
| Payload | Exactly what is sent to the model: app name, redacted context and selection, tones, explanation language. |
| Request | One call to the model for a session. Pressing R starts a new request for the same session; a new request cancels the running one. |
| Tone | One style of rewrite: `minimal`, `concise`, `natural`, `polite`, `formal`, `friendly`. |
| Variant | The model's version of the selection in one tone, plus a one-line note. |
| Issue | A mistake in the original English, with the fix and an explanation. |
| Notice | A short message the popup shows instead of suggestions (nothing selected, password field, blocked app, Apply failed). |
| Placeholder | A token such as `[EMAIL_1]` that replaces a sensitive value before sending; restored before display and Apply. |
| Provider | Which cloud service runs the model: `gemini`, `groq`, `openrouter` (free tiers) or `claude` (Anthropic API, paid). Each has its own API key. |
| UIA | Microsoft UI Automation, the accessibility API used to read text from other apps. |

## Conventions

- Each rule has a stable ID (`CAP-07`). New rules get the next free number in their page; a removed rule keeps its number, marked *Removed*, so references stay valid.
- **Status** lines say whether a part is implemented. Anything not built yet is listed under "Not implemented" with a pointer to the `docs/PLAN.md` item.
- Numbers (timeouts, sizes, limits) are the values in the code at the time of writing; each page links the source file.
- "Verified by" names the automated tests that pin a rule. Rules without one depend on other apps, the global hotkey or the mouse hook and are checked by hand (`/smoke-test`).

## Keeping the specs current

- A change that alters behavior updates the matching spec in the same change, like the contracts in `CLAUDE.md`.
- A finished `docs/PLAN.md` item is described here before it is ticked.
- Specs describe behavior and reasons, not code structure; the code map lives in `CLAUDE.md`.
