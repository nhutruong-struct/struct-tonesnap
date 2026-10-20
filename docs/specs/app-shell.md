# App shell

> **Status:** implemented (Phase 0, Windows). macOS is Phase 3.
> **Source:** `src-tauri/src/lib.rs`, `tray.rs`, `window.rs`, `i18n.rs`, `scripts/make-tray-icons.mjs`, `src/main.tsx`, `src/lib/i18n.ts`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`.

ToneSnap is a tray app: it has no main window, runs in the background and only shows the popup or the Settings window when needed. It can start when the user signs in to Windows (SET-60).

## Startup

- **APP-01** Only one ToneSnap runs at a time. Starting it again opens the Settings window of the running instance instead.
- **APP-02** On startup ToneSnap, in order: loads the settings (see [settings.md](settings.md) SET-30..), builds the tray icon, prepares the floating icon and starts it if the trigger mode is Icon, and registers the hotkey.
- **APP-03** The Settings window opens by itself at startup when the user needs to act:
  - the hotkey could not be registered (another app holds it, or it is invalid);
  - the saved settings had a problem (invalid fields, a broken file, or a file that could not be read);
  - no API key is stored for the chosen provider (so a first run opens Settings to ask for one).
  Otherwise ToneSnap starts silently in the tray.
- **APP-04** Both windows exist from the start but are hidden (`tauri.conf.json`). One web bundle serves both; the window label (`popup` or `settings`) decides which page renders.

## Tray

- **APP-10** The tray icon shows the tooltip "ToneSnap". A left click opens Settings. The right-click menu has two items, each with an icon: *Settings…* (a gear) and *Quit ToneSnap* (a power symbol). The icons are 20 px grey bitmaps drawn by `scripts/make-tray-icons.mjs`; Windows draws a menu icon at its pixel size and cannot tint it for a dark or light menu, so one neutral grey is used.
- **APP-11** Menu labels follow the interface language and are updated as soon as settings are saved.
- **APP-12** *Quit ToneSnap* exits at once.

## Windows and lifetime

- **APP-20** Closing a window (title-bar X, Alt+F4) only hides it. ToneSnap keeps running in the tray with no window open; only *Quit ToneSnap* ends it.
- **APP-21** Opening Settings (from the tray, a second launch, the popup's ⚙ button or an error's *Open Settings* button) restores it if minimized, shows it, focuses it, and tells the page to reload the saved values (SET-41).
- **APP-22** The popup is frameless, always on top, not resizable and not in the taskbar. It can be dragged by its header.
- **APP-23** The web pages cannot reach the network: the Content Security Policy allows only Tauri's IPC. Every network call is made by the Rust core ([privacy.md](privacy.md) PRV-04).
- **APP-24** Window permissions granted to the pages: dragging, setting the title and hiding the current window (both windows), opening links (`opener`), and autostart (Settings only, for SET-60).

## Interface language

- **APP-30** The interface is in Vietnamese or English. The `uiLanguage` setting is `auto`, `vi` or `en`; `auto` picks Vietnamese when the OS locale starts with `vi`, otherwise English. The tray (Rust) reads the OS locale; the pages read the browser language, which follows Windows.
- **APP-31** The popup uses the saved language and switches as soon as settings are saved (`settings-changed`).
- **APP-32** The Settings window follows the language currently picked in its form, before saving, so the user sees what they are choosing.
- **APP-33** Every interface string exists in both languages (`src/lib/i18n.ts`, `src-tauri/src/i18n.rs`); a missing translation is a TypeScript error. Strings may contain `{name}` placeholders, filled the same way in both languages.

**Verified by:** `tray.rs` test (the menu icons decode and share one size), `src/lib/i18n.test.ts` (both languages have the same keys and placeholders, every tone/trigger/notice has a label, language resolution), `src/settings/SettingsPage.test.tsx` ("the interface language preview follows the unsaved choice").

## Not implemented

- macOS (Phase 3): the platform layer returns inert stubs there, and capture always reports "nothing selected".
