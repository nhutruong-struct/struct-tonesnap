## What and why

<!-- What does this change do, and why? Link the issue it resolves. -->

Closes #

## How it was tested

<!-- Automated tests added or changed, and manual steps for what automation cannot reach
     (other apps, the global hotkey, the mouse hook): which apps, which Windows version. -->

## Screenshots

<!-- For UI changes: before and after, in light and dark. Remove private text. Delete this section otherwise. -->

## Checklist

- [ ] The PR targets `develop` and its title is an imperative English summary (it becomes the commit message).
- [ ] `node scripts/harness/check.mjs --full` passes locally.
- [ ] New or changed behavior has automated tests.
- [ ] New interface text is in `src/lib/i18n.ts` (or `src-tauri/src/i18n.rs`) in both Vietnamese and English.
- [ ] `docs/specs/` and `docs/PLAN.md` are updated when behavior changes.
- [ ] No user text is logged, stored or sent anywhere except the redacted AI request; no new network host.
- [ ] No API key, token or private data in the code, tests, screenshots or commit history.
- [ ] Dependencies were not added or removed, or this was agreed in the linked issue.
