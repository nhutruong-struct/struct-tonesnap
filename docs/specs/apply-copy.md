# Apply and Copy

> **Status:** implemented on Windows (Phase 0). Formatting-preserving replace in Word is Phase 2.
> **Source:** `src-tauri/src/commands.rs` (`apply_text`, `copy_text`), `platform/win/mod.rs` (`paste_into`), `platform/win/clipboard.rs`, `platform/win/input.rs`, `platform/win/window.rs` (`focus`), `src/popup/Popup.tsx`.

Apply puts the chosen version back in place of the selection; Copy leaves it on the clipboard for the user to paste wherever they like. Both use the restored text (placeholders swapped back, [privacy.md](privacy.md) PRV-14) and only a ready variant ([popup.md](popup.md) POP-32).

## Apply

- **APL-01** Apply closes the popup and cancels the running request, then works on its own thread so the UI never waits.
- **APL-02** It brings back the window that was in front when the selection was captured: it restores the window if minimized and asks Windows to focus it. Windows only lets the process that received the last input change the foreground, so ToneSnap falls back to attaching to the current foreground thread's input queue. It retries every 40 ms for up to 800 ms.
- **APL-03** Once the window is in front, ToneSnap waits 80 ms (the app restores its caret and selection), snapshots the clipboard, puts the text on it as a transient entry ([privacy.md](privacy.md) PRV-31), waits up to 400 ms for modifier keys to be released, and sends Ctrl+V.
- **APL-04** It waits 500 ms before restoring the user's clipboard, because Electron apps (Slack, Teams…) read the clipboard asynchronously and would otherwise paste the old content.
- **APL-05** If the window could not be focused or the clipboard could not be written, the text is copied to the clipboard as a normal entry and the `apply_failed` notice tells the user to press Ctrl+V. In that case the clipboard is intentionally **not** restored.
- **APL-06** Apply relies on the app's own paste: the text replaces the selection only if the app still has it selected when it gets focus back. How each app behaves is recorded in the compatibility table of `docs/PLAN.md`.
- **APL-07** Apply needs a session; without one the command fails with "no session" and nothing happens.

## Copy

- **APL-10** Copy closes the popup and cancels the running request ([ai-request.md](ai-request.md) AI-54), then puts the text on the clipboard as a normal entry, so it appears in clipboard history like any copy the user makes.
- **APL-11** The clipboard write runs on its own thread and retries for about 300 ms (20 tries, 15 ms apart), because another app can hold the clipboard open for a moment.

## macOS

- **APL-30** Same flow with Cmd instead of Ctrl (`platform/mac/mod.rs`, `input.rs`, `clipboard.rs`): the app is found by the process id kept in `hwnd`, brought forward if needed (800 ms), the text is placed as a transient entry (`org.nspasteboard.TransientType` and `ConcealedType`, which clipboard managers skip), and Cmd+V is posted after the modifiers were released. The snapshot keeps every item and every type on the clipboard. Posting keys needs the Accessibility permission. The V key is the virtual key at the US position, so layouts that move V (Dvorak) paste with another letter's shortcut.

## Read-only fields

- **APL-20** When the capture reported the field as not editable, Apply is not offered and the number keys copy instead ([popup.md](popup.md) POP-35). When editability is unknown, Apply is offered. How editability is found, including the selected text's read-only attribute that tells editable web fields from page text: [capture.md](capture.md) CAP-07.

**Verified by:** `platform/win/mod.rs` input tests, run on request (`check.mjs --only=input`): Ctrl+V into a focused test window replaces the selection and the user's clipboard is kept. `clipboard.rs` integration test (snapshot and restore of every format). `src/popup/Popup.test.tsx` (Apply and Copy send the restored text; read-only copies). Real third-party apps are manual (`/smoke-test`).

## Not implemented

- Word COM replace that keeps formatting (Phase 2).
