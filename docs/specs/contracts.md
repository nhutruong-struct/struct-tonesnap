# Rust ⇄ TypeScript contracts

> **Status:** implemented (Phase 0). Checked by the harness (`tones`, `settings`, `commands`, `events`, `capabilities` rules in `scripts/harness/invariants.mjs`) and by JSON-shape tests on the Rust side.
> **Source:** `src-tauri/src/commands.rs`, `lib.rs` (`generate_handler!`), `session.rs`, `request.rs`, `window.rs`, `settings.rs`, `prompt.rs`, `llm/openai_compat.rs`, `src/lib/api.ts`, `src/lib/tones.ts`.

The two windows talk to the Rust core only through the commands and events below. Components never call `invoke` or `listen` directly: they use the typed wrappers `api.*` and `events.*` in `src/lib/api.ts`.

- **CON-01** A change to any shape here changes both sides in the same change.
- **CON-02** Rust serializes struct fields in camelCase and enum values in snake_case. The model's own output (`Suggestions`) keeps the schema's snake_case names.

## Commands

- **CON-10** Synchronous commands run on Tauri's main thread, so they only touch memory. Anything that waits (files, the credential store, the network, the clipboard, focus) is `async` or hands off to its own thread.

| Command | Arguments | Result | How it runs | Called by |
|---|---|---|---|---|
| `get_settings` | none | `Settings` | sync, memory | both windows |
| `save_settings` | `newSettings: Settings` | nothing, or rejects with `CommandError` | async; hotkey part on the main thread | Settings |
| `hotkey_error` | none | `CommandError \| null` | sync | Settings |
| `settings_issue` | none | `SettingsIssue \| null` | sync | Settings |
| `has_api_key` | `provider` | `boolean` | async (credential store) | Settings |
| `set_api_key` | `provider`, `key: string` (empty deletes) | nothing, or rejects with a message | async | Settings |
| `send_request` | none | nothing | sync; the request runs on the async runtime | popup (Send, R, Retry) |
| `apply_text` | `text: string` | nothing, or rejects with `"no session"` | sync; pasting on its own thread | popup |
| `copy_text` | `text: string` | nothing | sync; closes the popup and cancels its request; clipboard on its own thread | popup |
| `hide_popup` | none | nothing | sync; also cancels the request | popup |
| `open_settings` | none | nothing | sync; closes the popup and cancels its request | popup |
| `update_info` | none | `UpdateInfo` | sync, memory | Settings |
| `check_for_updates` | none | nothing; the result arrives as `update-status` | async (network) | Settings |
| `install_update` | none | nothing; ToneSnap exits once the installer starts | async (network) | Settings |

- **CON-11** The Settings window also calls the autostart plugin's own commands through the wrappers `api.launchAtLogin` and `api.setLaunchAtLogin`: `plugin:autostart|is_enabled`, `plugin:autostart|enable`, `plugin:autostart|disable` (permission `autostart:default`, Settings window only).

## Events

- **CON-20** Event names are literals on both sides so the harness can match every `listen` to an `emit`.

| Event | Sent to | Payload | When |
|---|---|---|---|
| `session` | popup | `Session` | A capture succeeded |
| `notice` | popup | `Notice` | A capture was refused, or Apply failed |
| `request-started` | popup | `RequestEvent` (ids only) | A request began |
| `request-delta` | popup | `RequestEvent` + `text` | Each text chunk |
| `request-done` | popup | `RequestEvent` + `stopReason`, `model`, `usage`, `firstTokenMs`, `totalMs` | The answer is complete. A cancelled stream also ends here, with no stop reason; by then the popup shows a newer request (and ignores it, POP-11) or is hidden |
| `request-failed` | popup | `RequestEvent` + `errorKind`, `errorStatus`, `errorMessage`, timings | The request failed. `no_api_key` carries no timings |
| `settings-opened` | settings | none | The Settings window is being shown |
| `settings-changed` | every window | `Settings` | Settings were saved |
| `update-status` | every window | `UpdateInfo` | The update status changed ([updates.md](updates.md)) |

## Data shapes

### Session (Rust → popup)

| Field | Type | Meaning |
|---|---|---|
| `id` | number | Increments per capture |
| `app` | string | Friendly app name (CAP-03) |
| `source` | `"uia" \| "clipboard"` | Where the selection came from |
| `contextSource` | `"uia" \| "ocr" \| "none"` | Where the context came from |
| `editable` | `boolean \| null` | Whether the field accepts Apply; `null` = unknown |
| `selection` | string | The original selection, **before** redaction |
| `translated` | boolean | The selection looks Vietnamese (AI-12) |
| `payload` | `Payload` | Exactly what is sent to the model |
| `redactions` | `[placeholder, original][]` | Pairs that restore the suggestions |
| `preview` | boolean | Wait for Send before requesting |
| `model` | string | The model the request will use |

The target window handle stays in Rust and is never serialized.

### Payload

`app`, `contextBefore`, `selection`, `contextAfter`, `screenText` (all strings, redacted), `tones` (`ToneId[]`, the requested tones in order), `explanationLanguage` (`"Vietnamese"`, `"English"` or `"none"`).

### RequestEvent

`sessionId` and `requestId` (always), then optional `text`, `stopReason`, `model`, `firstTokenMs`, `totalMs`, `usage`, `errorKind`, `errorStatus`, `errorMessage`. Fields that do not apply to an event are `null`.

### Notice

`kind`: `no_selection` \| `password_field` \| `blocked_app` \| `apply_failed`; `detail`: the blocked app's name for `blocked_app`, empty otherwise.

### Suggestions (model output, parsed in the popup)

`detected_language`, `variants` (keyed by tone id, each `{ text, note }`), `issues` (`{ original, fix, explanation }[]`). While streaming, any part may be missing (AI-20..22).

### Settings

See [settings.md](settings.md). Enum values: `uiLanguage` `auto|vi|en`, `triggerMode` `hotkey|icon|auto|off`, `provider` `claude|gemini|groq|openrouter`, `explainLanguage` `vi|en|off`, `updateMode` `auto|manual`.

### InstalledModel

`name`, `sizeBytes`, `parameterSize` (e.g. `"4.7B"`).

### CommandError

`{ kind, hotkey, detail }` with `kind` one of `hotkey_invalid`, `hotkey_taken`, `save_failed`. `hotkey` names the hotkey for the hotkey errors (empty otherwise); `detail` is the raw message.

### UpdateInfo

`{ currentVersion, status, msiInstall }`. `status.kind` is `idle`, `checking`, `up_to_date`, `available`, `downloading`, `ready`, `installing` (these four with `version`) or `failed` (with `step`: `check`, `download` or `install`, and `detail`, the raw error). `msiInstall` is true when the app was installed with the MSI (UPD-24).

### SettingsIssue

`{ kind: "invalid_fields", fields: string[] }`, `{ kind: "broken", backup: string }` or `{ kind: "unreadable" }` (SET-31..34).

## Tone ids

- **CON-40** The six tone ids are listed identically, in the same order, in `prompt.rs TONES`, the `ToneId` type in `api.ts` and `TONE_IDS` in `tones.ts`: `minimal`, `concise`, `natural`, `polite`, `formal`, `friendly`. Labels and descriptions live in the i18n dictionaries (`tone.<id>`, `tone.<id>.desc`).

## Request error codes

- **CON-50** `errorKind` values the popup knows how to explain ([popup.md](popup.md) POP-60): `no_api_key`, `authentication_error`, `permission_error`, `not_found_error`, `rate_limit_error`, `overloaded_error`, `refusal`, `max_tokens`, `network`. Any other value (`http_error`, `stream_error`, other API error types) gets the generic "Error {status}" with the raw message.
- **CON-51** `errorStatus` is the HTTP status, `0` when there was none (an error inside the stream), and `null` for `refusal`, `max_tokens`, `network` and `no_api_key`.

**Verified by:** Rust JSON-shape tests: `session.rs` (Session keys and enum values), `request.rs` (RequestEvent field names, error fields), `window.rs` (Notice), `commands.rs` (CommandError), `settings.rs` (SettingsIssue, Settings casing), `update.rs` (UpdateInfo); the harness cross-file rules; `src/test/fixtures.ts` mirrors the Rust defaults for the UI tests.
