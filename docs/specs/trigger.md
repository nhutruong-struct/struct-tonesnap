# Triggers

> **Status:** Hotkey, Icon, Auto and Off modes implemented (Phase 0).
> **Source:** `src-tauri/src/trigger.rs`, `session.rs` (`on_trigger`, `on_selection`), `platform/mod.rs` (`Selection`), `platform/win/selection_icon/`, `platform/gesture.rs`, `platform/mac/selection_icon.rs`, `src/lib/hotkey.ts`, `src/settings/sections/TriggerSection.tsx`.

A trigger starts one use of ToneSnap: capture the selection, then show the popup. The global hotkey and a click on the floating icon do exactly the same thing; Auto mode starts the same flow from the selection itself, with the limits in TRG-40 to TRG-46.

## Trigger modes

The `triggerMode` setting decides which triggers are active.

| Mode | Hotkey | Floating icon | Status |
|---|---|---|---|
| `hotkey` | on | off | implemented |
| `icon` (default) | on | shown after a selection | implemented |
| `auto` | on | shown when the popup does not open by itself (TRG-44) | implemented |
| `off` | off | off | implemented: ToneSnap is paused |

- **TRG-01** The hotkey works in every mode except Off.
- **TRG-02** Icon and Auto mode share one selection watcher (TRG-20 to TRG-33); saving settings starts it in either mode and stops it in the others.

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
- **TRG-31** After a gesture ToneSnap waits 120 ms, then asks the Accessibility API (`selection_state`): a non-empty `AXSelectedText` or selected range shows the icon; a password field or an empty selection inside the focused control's frame does not; when the app does not say, the icon shows only if the pointer was the text I-beam at the press or the release, as in TRG-20 (a drag often starts on the text and ends past it, over an arrow). Edge and Chrome never say: they expose the browser's own controls to the Accessibility API but not the page, and refuse both `AXManualAccessibility` and `AXEnhancedUserInterface` (Edge 155, 2026-10-23), so a selection on a web page gets the icon from the I-beam alone and its click reads it through the clipboard (CAP-22). The I-beam is read from the system cursor (`NSCursor.currentSystemCursor`, deprecated by Apple): if a future macOS stops reporting it, only apps that report their selection get the icon, and the hotkey still works. Our own windows and apps in `blockedApps` never get the icon.
- **TRG-32** The icon is a borderless non-activating `NSPanel` of 26 points at the status window level, on every Space and over full-screen apps, placed 10 points right and 14 below where the mouse was released and kept inside the screen's visible area. It never becomes the key window, so the app keeps its selection. Its click is seen by a local monitor, hides the icon and triggers the capture like the hotkey (TRG-23).
- **TRG-33** The icon hides on a press elsewhere, a right or other button press, a scroll, when another app comes to the front, or after 6 s unhovered, checked every 150 ms.

## Automatic popup (Auto mode)

Auto mode is Icon mode that skips the click when it can: the same gestures, the same vetoes, and the icon whenever the popup does not open by itself.

- **TRG-40** Only selections the icon would get are considered: a mouse gesture (TRG-20, TRG-30) confirmed as for the icon (TRG-21, TRG-31), so never in ToneSnap's own windows, blocked apps or password fields. Selections made with the keyboard are never seen, because the keyboard is never watched.
- **TRG-41** The popup waits `autoDelayMs` after the mouse button is released (default 400 ms; values outside 100–3000 are held to that range). After a double click it waits at least the system's double-click time, so a triple click can still extend the selection to the paragraph.
- **TRG-42** Any mouse button press (including the side buttons) or scroll, vertical or horizontal, during the wait, or another app coming to the front, cancels it: neither the popup nor the icon appears. A press or a new front app while the selection is being read cancels the popup too. A new selection then starts its own wait.
- **TRG-43** The selection is read only through the accessibility API (UI Automation, the macOS Accessibility API). Auto mode never sends Ctrl+C / Cmd+C: nobody asked for it, and in a terminal Ctrl+C interrupts the running program ([capture.md](capture.md) CAP-08). It also opens only when the watcher's check found the selection where the gesture ended (`Selected` in TRG-21, TRG-31): when that check could not tell (the gesture ended outside the focused control, as on Google Docs' canvas, or the app does not say), the focused control may still hold an older selection the user did not mean to send, so the icon is shown at once instead.
- **TRG-44** When the accessibility API reports no selection (many Electron apps), or a selection of fewer than `autoMinChars` characters after trimming (default 8, so a double-clicked word does not open it; values outside 1–200 are held to that range), the floating icon is shown instead, once the wait is over, and works as in Icon mode.
- **TRG-45** A capture already running (TRG-03) cancels the automatic popup. Its failures (a blocked app, a password field) show no notice, since the user did not ask; the hotkey still shows them. A missing Accessibility permission (macOS) shows the icon, whose click explains it.
- **TRG-46** The popup takes the keyboard as usual: by the time it opens the mouse button is up and the wait is over, so the selection is complete.

**Verified by:** `state.rs` test (only one capture at a time, TRG-03), `src/lib/hotkey.test.ts` (normalizing, display, known conflicts), `src/settings/SettingsPage.test.tsx` (capturing a new hotkey, the conflict warning, a taken hotkey keeps the window open, the Auto mode fields are enabled and saved only with it), `platform/gesture.rs` tests (drag, single click, double click within the system limits, presses on the icon, only a selection made by clicking waits for a triple click), `platform/mod.rs` test (the wait after a double click, a press ends the selection's wait, TRG-41/42), `capture.rs` tests (Auto mode never copies and leaves unreadable or short selections to the icon, TRG-43/44), `settings.rs` tests (the delay and minimum length ranges), `uia.rs` integration test (a gesture outside the focused element is not judged by it), `selection_icon/paint.rs` tests (resampling), `platform/mac/selection_icon.rs` tests (an app that does not say gets the icon only for a gesture over text; what the app reports wins over the pointer). Registering the global hotkey and the mouse hook in other apps, and the automatic popup in other apps (TRG-40 to TRG-46), are checked by hand (`/smoke-test`).
