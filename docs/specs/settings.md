# Settings

> **Status:** implemented (Phase 0).
> **Look and feel:** [ui-design.md](ui-design.md).
> **Source:** `src-tauri/src/settings.rs`, `commands.rs` (`save_settings`), `trigger.rs`, `secrets.rs`, `src/settings/` (`useSettingsForm.ts`, `SettingsPage.tsx`, `SettingsNav.tsx`, `sections/`), `src/lib/api.ts` (`Settings`).

## Fields

The same `Settings` object exists in Rust and TypeScript, serialized in camelCase with snake_case enum values ([contracts.md](contracts.md) CON-02).

| Field | Default | Settings control | Effect |
|---|---|---|---|
| `uiLanguage` | `auto` | Interface → language (`auto`, `vi`, `en`) | APP-30 |
| `triggerMode` | `icon` | Trigger → four options | [trigger.md](trigger.md) |
| `hotkey` | `ctrl+alt+e` | Trigger → press a combination | TRG-10..17 |
| `autoDelayMs` | `400` | Trigger → number 100–3000, step 50, enabled only in Auto mode | [trigger.md](trigger.md) TRG-41 |
| `autoMinChars` | `8` | Trigger → number 1–200 (held to that range when used), enabled only in Auto mode | [trigger.md](trigger.md) TRG-44 |
| `provider` | `gemini` | AI → Gemini, Groq, OpenRouter or Claude API | [ai-request.md](ai-request.md) |
| `model` | `claude-opus-5-5` | AI (Claude) → Opus 5.5 / Sonnet 5.5 / Haiku 4.5 | AI-35 |
| `effort` | `low` | AI (Claude) → low / medium / high; disabled for Haiku 4.5 | AI-36 |
| `geminiModel` | `gemini-3.8-flash` | AI (Gemini) → model name, with suggestions | AI-30 |
| `groqModel` | `openai/gpt-oss-20b` | AI (Groq) → model name, with suggestions | AI-30 |
| `openrouterModel` | `openai/gpt-oss-20b:free` | AI (OpenRouter) → model name, with suggestions | AI-30 |
| `targetLanguage` | `vi` | Language and tones → default target language (`vi`, `en`, `ja`) | AI-01, POP-25 |
| `mode` | `translate` | Language and tones → default mode (`translate`, `rewrite`) | AI-14, POP-25 |
| `tones` | `concise, natural, formal` | Language and tones → one checkbox per tone | AI-05 |
| `contextChars` | `600` | Context → number 0–3000, step 100 | CAP-10, CAP-14 |
| `ocrFallback` | `true` | Context → checkbox | CAP-11 |
| `redact` | `true` | Privacy → checkbox | PRV-10 |
| `customSensitiveTerms` | empty | Privacy → one term per line | PRV-12 |
| `previewBeforeSend` | `false` | Privacy → checkbox | PRV-20 |
| `blockedApps` | `keepass.exe`, `keepassxc.exe`, `1password.exe`, `bitwarden.exe`, `lastpass.exe` | Privacy → one process name per line | CAP-02 |
| `updateMode` | `auto` | Updates → install automatically when idle, or ask first | [updates.md](updates.md) UPD-20..23 |

Two choices are **not** in this file: the API keys (one per provider) live in the credential store (SET-50), and starting at sign-in is kept by Windows (SET-60).

## Editing in the Settings window

- **SET-01** Edits stay in the form until *Save Settings*; nothing applies earlier, except the interface language preview (APP-32) and the API key (SET-50).
- **SET-02** Tones are saved in the canonical order whatever order they were ticked in, so the popup is predictable.
- **SET-03** The two list fields are edited as one entry per line; on save, lines are trimmed and empty ones dropped, and blocked app names are lower-cased.
- **SET-04** Choosing a provider shows that provider's key field (with where to get a key) and its model. Gemini, Groq and OpenRouter take any model name, with a few suggestions in a combobox (UI-15); Claude offers a fixed list and the effort. For the free providers the form warns that a free tier may let the provider use the content. Switching provider starts from that provider's own key state; a key being typed is not carried over. The *Get Key* button next to the key field opens that provider's key page in the default browser (`opener:default` permission, http/https only).
- **SET-05** The hotkey field flags a known conflict ([trigger.md](trigger.md) TRG-13) and a registration error.
- **SET-06** The window shows one section at a time, chosen in a sidebar: Interface, AI connection, Popup trigger, Language and tones, Context, Privacy, Updates. Every section stays mounted, so edits and drafts in a section that is not open are kept and saved with the rest ([ui-design.md](ui-design.md) UI-31).

## Saving

- **SET-20** *Save Settings* applies the new settings in this order:
  1. switch the hotkey (on the main thread). If the new one cannot be registered, the old one is restored and the save stops with `hotkey_invalid` or `hotkey_taken`: **nothing is written** and the window stays open showing the error;
  2. write the file. A failure registers the old hotkey again, so the hotkey in use always matches the saved settings, and stops with `save_failed`, shown next to the Save button;
  3. start or stop the floating icon;
  4. keep the new settings in memory, clear any loading problem (SET-34), tell both windows (`settings-changed`), update the tray menu and apply the update mode ([updates.md](updates.md) UPD-23);
  5. change starting at sign-in, if the user changed it (SET-61).
  The window then hides.
- **SET-21** The file is written to a sibling `settings.json.partial` first and then renamed over `settings.json`, so a crash mid-write cannot leave a truncated file.
- **SET-22** New settings apply from the next trigger. A request already running keeps the settings it started with.

## Loading

Settings live in `settings.json` in the app's config directory (on Windows, `%APPDATA%\com.tonesnap.app\`).

- **SET-30** No file (first run): the defaults, with no warning.
- **SET-31** The file is read **field by field**. A field whose value ToneSnap does not understand (a wrong type, an unknown enum value) falls back to its default alone; every other field keeps its saved value. Unknown fields (e.g. from an older version) are ignored and disappear at the next save.
- **SET-32** A **broken** file (not UTF-8, or not a JSON object) is moved aside to the first free name among `settings.broken.json`, `settings.broken-2.json`, `settings.broken-3.json`… (so an older backup is never overwritten and the next save cannot destroy it), and the defaults are used.
- **SET-33** A file that exists but **cannot be read** right now (another program holds it, no permission), or a broken file that could not be moved aside, is left untouched and the defaults are used. Saving replaces it.
- **SET-34** Each problem is shown as a warning banner in Settings, and Settings opens at startup (APP-03), until the next successful save. The banner names the fields that fell back (`invalid_fields`), the path of the kept copy (`broken`), or says the file could not be read (`unreadable`).

## Opening and reopening

- **SET-40** The Settings window opens from the tray, a second launch, the popup's ⚙ button, an error's *Open Settings* button, or by itself at startup (APP-03).
- **SET-41** Every time it opens it reloads the saved settings, the hotkey error and the loading problem, and resets every section and the open section (back to Interface), so unsaved edits and drafts (such as a half-typed API key) from last time do not linger.
- **SET-42** Closing the window without saving discards the edits.

## Starting with Windows

- **SET-60** *Start ToneSnap when I sign in to Windows* (Interface section) makes Windows launch ToneSnap at sign-in, so it waits in the tray, ready to use. It is off until the user turns it on. Windows keeps the choice (an entry under the user's `Run` registry key, written by the autostart plugin), so `settings.json` does not hold it. The `.exe` uninstaller removes the entry; the `.msi` one does not ([app-shell.md](app-shell.md) APP-40, APP-42).
- **SET-61** The checkbox shows what Windows reports each time the window opens. A change applies only when *Save Settings* succeeds, and only if it differs from what Windows reported. If Windows refuses the change, the settings are still saved, the save bar says the start-up choice could not be changed, and the window stays open.
- **SET-62** When Windows cannot report the state, the option is not shown.
- **SET-63** The entry points to the running executable: enabling it from a development build registers the development binary.

## Permissions (macOS)

- **SET-70** On macOS Settings has a *Permissions* section, second in the sidebar, for the two permissions the OS makes the user grant: *Accessibility* (read the selection, press Cmd+C and Cmd+V; required) and *Screen Recording* (OCR context; optional). Each row shows *Allowed* or *Not allowed*; a missing one has *Allow…* (the system's prompt, which the system shows once per app) and *Open System Settings* (the Privacy & Security pane of that permission). Elsewhere the section does not exist (`get_permissions` answers `required: false`).
- **SET-71** The status is read when the window opens, every 1.5 s while it is open and when it regains focus, since the user grants it in another window. A missing Accessibility permission puts a red dot on the section and makes it the first page shown, decided once when the status first arrives, so granting it does not move the page. Screen Recording takes effect only after ToneSnap is reopened, which the row says. The Accessibility row's hint says what to do in System Settings, and its status turns to *Allowed* by itself.
- **SET-72** In development the permission belongs to the program that launched ToneSnap (the terminal or editor running `./dev.sh`), not to the `tonesnap` binary; a bundled app is its own entry.

## API key

- **SET-50** Each provider has its own key. It is entered in a password field and stored with *Save Key*, immediately and independently of *Save Settings*. The value is trimmed; it goes to Windows Credential Manager ([privacy.md](privacy.md) PRV-40), never into the settings.
- **SET-51** The form only knows whether a key exists. With a key stored, the field says "Saved (enter a new key to replace it)" and a *Remove* button deletes it.

**Verified by:** `settings.rs` tests (missing file, save/load round trip without a leftover partial file, the defaults are Vietnamese, translate and all three tones, an unknown language falls back alone, an invalid field falls back alone, a broken file is set aside without overwriting older backups, a file that cannot be read is left alone, a non-object is broken, issue JSON, camelCase JSON, the active model follows the provider, a saved Ollama provider falls back to the default, blocked apps, a file written before the newer fields keeps its values and loads without a warning, enum values are spelled like the TypeScript types, the prompt's names for languages and modes, an explicit interface language), `secrets.rs` tests (one account and variable per provider), `commands.rs` tests (error JSON; the save order: switch, then write, and the old hotkey put back when the write fails), `src/settings/SettingsPage.test.tsx` (one section open at a time with edits kept, a hotkey problem marked in the sidebar, saved values shown, settings warning, cleaned lists and canonical tone order on save, the default language and mode are saved, hotkey capture and conflict, a taken hotkey keeps the window open, a failed write is reported, a successful save clears the warning, each provider keeps its own key and model and Gemini is the default, a saved key is shown and can be removed, the Claude key goes to the credential store, Haiku disables effort, reopening drops unsaved edits; starting with Windows changes only on save and only when changed, a refused change is reported and keeps the window open, the option is hidden when Windows cannot tell; the trigger modes with Auto unavailable and its delay off, leaving a tone out, the context size and screen fallback, masking and preview, the Claude model and effort, the interface language is saved), `src/settings/useSettingsForm.test.ts` (the hotkey, save and settings-file messages in both languages). Permissions: `src/settings/sections/PermissionsSection.test.tsx` (allowed and missing rows, the buttons ask the system and open the right pane, the status is read again on focus), the `permissions (macOS)` tests of `SettingsPage.test.tsx` (no section without permissions, the first look goes to a missing Accessibility permission and stays, no jump when it is allowed), `platform/mod.rs` tests (JSON shape and names). The system prompts and System Settings panes are manual.
