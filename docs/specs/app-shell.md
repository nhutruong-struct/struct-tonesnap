# App shell

> **Status:** implemented (Phase 0, Windows). macOS is Phase 3.
> **Source:** `src-tauri/src/lib.rs`, `tray.rs`, `window.rs`, `i18n.rs`, `scripts/make-tray-icons.mjs`, `src/main.tsx`, `src/lib/i18n.ts`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`.

ToneSnap is a tray app: it has no main window, runs in the background and only shows the popup or the Settings window when needed. It can start when the user signs in to Windows (SET-60).

## Startup

- **APP-01** Only one ToneSnap runs at a time. Starting it again opens the Settings window of the running instance instead.
- **APP-02** On startup ToneSnap, in order: loads the settings (see [settings.md](settings.md) SET-30..), builds the tray icon, prepares the floating icon and starts it if the trigger mode is Icon, registers the hotkey, and starts checking for updates ([updates.md](updates.md) UPD-10).
- **APP-03** The Settings window opens by itself at startup when the user needs to act:
  - the hotkey could not be registered (another app holds it, or it is invalid);
  - the saved settings had a problem (invalid fields, a broken file, or a file that could not be read);
  - no API key is stored for the chosen provider (so a first run opens Settings to ask for one).
  Otherwise ToneSnap starts silently in the tray.
- **APP-04** Both windows exist from the start but are hidden (`tauri.conf.json`). One web bundle serves both; the window label (`popup` or `settings`) decides which page renders.

## Tray

- **APP-10** The tray icon shows the tooltip "ToneSnap". A left click opens Settings. The right-click menu has two items, each with an icon: *Settings…* (a gear) and *Quit ToneSnap* (a power symbol). The icons are 20 px grey bitmaps drawn by `scripts/make-tray-icons.mjs`; Windows draws a menu icon at its pixel size and cannot tint it for a dark or light menu, so one neutral grey is used. While a new version can be installed, *Update to {version} & Restart* comes first, with a download arrow ([updates.md](updates.md) UPD-40).
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

## macOS

- **APP-50** ToneSnap runs as a menu-bar app (`ActivationPolicy::Accessory`): no Dock icon and no app menu. The tray icon sits in the menu bar. `./dev.sh` runs it in development.
- **APP-51** At startup the popup window becomes a non-activating panel (`platform/mac/panel.rs`): its class is swapped for an `NSPanel` subclass that can become the key window, with the non-activating style, status window level, and the join-all-Spaces and full-screen-auxiliary behaviors, so it floats over full-screen apps. Showing it makes it the key window without making ToneSnap the active app: the app the user selected text in stays in front with its caret and selection. If the class cannot be swapped safely (the panel would need more memory than the window has), the popup stays an ordinary window and a diagnostic line goes to stderr.
- **APP-52** "Is one of our windows in front" is answered by the frontmost app being ToneSnap or one of ToneSnap's windows being the key window (`NSApp.keyWindow`). Pressing the hotkey while the popup holds the keyboard therefore still closes it (CAP-01), and clicking another app closes the popup through the blur handling.
- **APP-53** The interface names macOS instead of Windows where it speaks of the system (macOS, Keychain, Vision, the menu bar, `⌘V`) and shows shortcuts with the Mac symbols (`⌃⌥E`). The strings are overrides in `i18n.ts`, checked for the same placeholders as the originals.

**Verified by:** `tray.rs` test (the menu icons decode and share one size), `i18n.rs` tests (the menu labels and the update item in both languages), `src/lib/i18n.test.ts` (both languages have the same keys and placeholders, every tone, trigger, notice, error, provider and update string has a label, button labels are in title case, language resolution), `src/settings/SettingsPage.test.tsx` ("the interface language preview follows the unsaved choice").

## Installing

- **APP-39** The `.exe` installer file, its window and the uninstaller carry the ToneSnap icon (`bundle.windows.nsis.installerIcon` in `tauri.conf.json`). Without that key NSIS falls back to its own generic icon.

## Uninstalling

- **APP-40** The `.exe` (NSIS) uninstaller always removes the "Start with Windows" `Run` entry (SET-60), except during an update. Tauri's installer template does this.
- **APP-41** Its *Delete the application data* checkbox removes the settings (`%APPDATA%\com.tonesnap.app`), the WebView data (`%LOCALAPPDATA%\com.tonesnap.app`) and every provider's API key in Credential Manager ([privacy.md](privacy.md) PRV-43). The folders are Tauri's template; the keys are `src-tauri/windows/hooks.nsh`. Left unticked, a reinstall finds the settings and keys again. An update (`/UPDATE`) and a silent or passive uninstall, which show no checkbox, keep everything.
- **APP-42** The `.msi` uninstaller removes only the installed files and shortcuts: the settings, WebView data, keys and `Run` entry stay. The README's Uninstall section lists them and how to remove them by hand; there is no WiX custom action for them.

**Verified by:** `secrets.rs` test "the uninstaller deletes every key" (`hooks.nsh` names every provider's Credential Manager target); the uninstall itself by hand (install the `.exe`, save a key, uninstall with the box ticked, check Credential Manager).

## Not implemented

- macOS (Phase 3): `platform/mac/` reads the selection, copies and pastes (APL-30) and places the popup; the popup and icon as `NSPanel`s, OCR, permission onboarding and signing are not built, so the popup is still an ordinary window there.
