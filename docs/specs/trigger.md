# Triggers

> **Status:** Hotkey, Icon and Off modes implemented (Phase 0). Auto mode not implemented.
> **Source:** `src-tauri/src/trigger.rs`, `session.rs` (`on_trigger`), `platform/win/selection_icon/`, `platform/gesture.rs`, `platform/mac/selection_icon.rs`, `src/lib/hotkey.ts`, `src/settings/sections/TriggerSection.tsx`.

A trigger starts one use of ToneSnap: capture the selection, then show the popup. There are two triggers, the global hotkey and a click on the floating icon, and both do exactly the same thing.

## Trigger modes

The `triggerMode` setting decides which triggers are active.

| Mode | Hotkey | Floating icon | Status |
|---|---|---|---|
| `hotkey` | on | off | implemented |
| `icon` (default) | on | shown after a selection | implemented |
| `auto` | on | off | **not implemented**: the option is disabled in Settings ("Coming soon") |
| `off` | off | off | implemented: ToneSnap is paused |

- **TRG-01** The hotkey works in every mode except Off.
- **TRG-02** A saved `auto` mode (only possible by editing the settings file) behaves like `hotkey`.

## Common rules

- **TRG-03** One capture at a time: a trigger that arrives while a capture is still running is ignored. The guard is released however the capture ends.
- **TRG-04** Any trigger first hides the floating icon if it is showing.
- **TRG-05** The capture runs on its own thread, never on the UI thread, because it waits on the keyboard, the clipboard and UI Automation.

## Hotkey

- **TRG-10** Default: `Ctrl+Alt+E`. `Ctrl+Alt+Space` was avoided because Claude desktop's Quick Entry uses it; `Ctrl+Alt+E` only shadows Word's euro-sign shortcut.
- **TRG-11** Stored as a normalized lower-case string, modifiers first in the order `ctrl`, `alt`, `shift`, `super`, then the key: `ctrl+alt+e`. Aliases are folded (`control`→`ctrl`, `option`→`alt`, `cmd`/`command`/`win`/`meta`→`super`) and the `Key`/`Digit` prefixes of key codes are dropped. Shown to the user as `Ctrl+Alt+E` (`super` shows as `Win`).
- **TRG-12** In Settings the hotkey box is read-only; the user clicks it and presses a combination. A combination is accepted only with at least one modifier and one non-modifier key. The physical key (`KeyboardEvent.code`) is used, so the keyboard layout does not change the stored key.
- **TRG-13** Known conflicts are warned about but not blocked. Low-level keyboard hooks in other apps cannot be detected by Windows, so ToneSnap keeps a short list: `Ctrl+Alt+Space` (Claude desktop), `Ctrl+Space` (IDEs, Windows IME), `Alt+Space` (PowerToys Run, ChatGPT, Copilot), `Ctrl+Shift+Space`, `Ctrl+Shift+E`, `Ctrl+Alt+R`, `Ctrl+Alt+C`, `Ctrl+Alt+T` (Word).
- **TRG-14** Registering replaces every hotkey ToneSnap holds. Failures are typed: `hotkey_invalid` (the string cannot be parsed) and `hotkey_taken` (another app registered it). The error is remembered and shown as a banner in Settings until a registration succeeds.
- **TRG-15** Changing the hotkey on save: the new hotkey is registered first. If that fails, the old one is registered again (the user is never left without a hotkey), the save is rejected with the hotkey error, and nothing is written. See [settings.md](settings.md) SET-20.
- **TRG-16** Hotkey registration always runs on the main thread. Called from another thread, the global-shortcut plugin can deadlock against its own key handler.
- **TRG-17** The trigger fires on key press (not release). Capture then waits for the modifiers to be released ([capture.md](capture.md) CAP-04).

## Floating icon (Icon mode)

The icon lets mouse users open ToneSnap without the keyboard. Only the mouse is watched; the keyboard is never hooked.

- **TRG-20** A low-level mouse hook watches left-button presses and releases. A release counts as a **selection gesture** when:
  - the pointer moved more than 6 physical pixels in either direction between press and release (a drag), or the press was a double or triple click within the system's double-click time and distance; **and**
  - the pointer was the text I-beam at the press (or, if not, at the release).
- **TRG-21** After a gesture ToneSnap waits 120 ms (so the app finishes updating its selection), then confirms on another thread. The icon is **not** shown when the foreground window is ToneSnap's own, when the app is blocked ([capture.md](capture.md) CAP-02), or when UI Automation says nothing is selected or the field is a password. "Nothing is selected" is believed only when the gesture ended inside the focused element: Google Docs keeps focus on an input parked off-screen (checked 2026-10-08: about 15,000 px above the screen) and draws the selection on a canvas, so that input always reports an empty selection. When the gesture ended outside the focused element, or UI Automation cannot tell, the icon is shown.
- **TRG-22** The icon is 26 × 26 logical pixels (scaled to the monitor's DPI), placed 10 px right and 14 px below where the button was released, kept inside the monitor's work area.
- **TRG-23** The icon never takes focus, so the app keeps its selection. Clicking it hides it and then triggers exactly like the hotkey.
- **TRG-24** The icon hides when: the user presses the left button elsewhere, presses the right or middle button, or scrolls (except over the icon); the foreground window changes away from the app that owned the selection; it has been shown for 6 seconds without the pointer over it; any trigger fires.
- **TRG-25** The hook callback returns immediately and posts work elsewhere, because Windows removes low-level hooks that stall input.
- **TRG-26** Saving settings starts or stops the icon to match the mode. The blocked-app list is read at every selection, so changes apply without a restart.

## Floating icon on macOS

- **TRG-30** A global `NSEvent` monitor watches mouse events only (left press and release, right and other presses, scroll), which needs no permission; the keyboard is never watched. Gestures are told apart by the same `gesture.rs` as on Windows, in points: a drag of more than 6 points, or two clicks within the system double-click interval and 4 points.
- **TRG-31** After a gesture ToneSnap waits 120 ms, then asks the Accessibility API (`selection_state`): a non-empty `AXSelectedText` or selected range shows the icon; a password field or an empty selection inside the focused control's frame does not; when the app does not say, the icon shows only if the pointer was the text I-beam on release. The I-beam is read from the system cursor (`NSCursor.currentSystemCursor`, deprecated by Apple): if a future macOS stops reporting it, only apps that report their selection get the icon, and the hotkey still works. Our own windows and apps in `blockedApps` never get the icon.
- **TRG-32** The icon is a borderless non-activating `NSPanel` of 26 points at the status window level, on every Space and over full-screen apps, placed 10 points right and 14 below where the mouse was released and kept inside the screen's visible area. It never becomes the key window, so the app keeps its selection. Its click is seen by a local monitor, hides the icon and triggers the capture like the hotkey (TRG-23).
- **TRG-33** The icon hides on a press elsewhere, a right or other button press, a scroll, when another app comes to the front, or after 6 s unhovered, checked every 150 ms.

**Verified by:** `state.rs` test (only one capture at a time, TRG-03), `src/lib/hotkey.test.ts` (normalizing, display, known conflicts), `src/settings/SettingsPage.test.tsx` (capturing a new hotkey, the conflict warning, a taken hotkey keeps the window open), `selection_icon/gesture.rs` tests (drag, single click, double click within the system limits, presses on the icon), `uia.rs` integration test (a gesture outside the focused element is not judged by it), `selection_icon/paint.rs` tests (resampling). Registering the global hotkey and the mouse hook in other apps are checked by hand (`/smoke-test`).

## Not implemented

- **Auto mode** (`docs/PLAN.md`: "Automatic popup on selection"). Planned to reuse the icon's gesture detection. The settings already exist: `autoDelayMs` (default 400 ms, shown but disabled in Settings) and `autoMinChars` (default 8, no control yet). Settings lists it as *Coming soon*.
