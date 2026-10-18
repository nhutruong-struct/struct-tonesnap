---
name: smoke-test
description: Test ToneSnap with the user in real apps (Word, Outlook, Slack, Chrome, Edge, PDF, Notepad, Teams) and fill in the compatibility table in docs/PLAN.md.
argument-hint: "[app, app…] (default: the empty rows of the compatibility table)"
disable-model-invocation: true
---

# Real-app smoke test

ToneSnap's own Win32 code has automated tests (`check.mjs --full`, and `--only=input` for key presses; see "Tests" in `docs/HARNESS.md`). What they cannot reach is how real third-party apps, the global hotkey and the mouse hook behave, so this is the manual acceptance test for those. **You never send synthetic keystrokes or mouse input**: the user operates the apps, you guide and record.

Apps to test: $ARGUMENTS

1. **Prepare**
   - `node scripts/harness/check.mjs` passes.
   - Check whether the app is running (`Get-Process tonesnap`). If it is not, start `pwsh -File dev.ps1` in the background and wait for ``Running `target`` in its output.
   - Read the hotkey, trigger mode and provider from `%APPDATA%\com.tonesnap.app\settings.json`. Never print an API key.
2. **Pick the apps**: from the arguments, or the empty rows of the compatibility table in `docs/PLAN.md`.
3. **Test each app.** Give the user concrete steps in their language:
   - Open the app, type a sample sentence from `scripts/bench-cases.json` that suits it, and select it.
   - Press the hotkey, or click the floating icon in icon mode.
   - Read the popup chips: text source (read directly / via clipboard), context source (UIA / OCR / none), read-only.
   - Apply a version and check that the text was replaced in the right place.
   - Press `Ctrl+V` somewhere else to confirm the previous clipboard came back.
   - Note the timing shown at the bottom of the popup.
4. **Ask for results** with AskUserQuestion using `multiSelect`: which of these worked (reads the selection, has context, knows if editable, Apply in the right place, clipboard restored). Let the user add notes.
5. **Record** each result in the table in `docs/PLAN.md` as ✅ / ⚠️ / ❌ with a short English note (context source, timing, error).
6. **Summarize** the problems found and the follow-ups. Propose them as new PLAN items and wait for the user's agreement.
