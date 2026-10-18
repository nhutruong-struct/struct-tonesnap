---
name: feature
description: Implement a ToneSnap work item (a new feature, a bug fix, an unticked item in docs/PLAN.md) following the project workflow: read the plan, plan briefly, change in small steps, run the checks, update PLAN.md, hand over manual test steps. Use when the user wants to continue the plan, build a feature or fix a bug in the app.
argument-hint: "<task description or PLAN.md item>"
---

# Implement a work item

Task: $ARGUMENTS

## 1. Scope
- Without a task: list the unticked items of the current phase in `docs/PLAN.md` and ask the user which one to do.
- Find the item in `docs/PLAN.md`. If the work is new, ask whether to add it to the plan, and put it in the right phase.
- Name the contracts the change touches (`CLAUDE.md` → Invariants): `Settings`, tones, commands, events, capabilities, i18n, prompt.

## 2. Plan
- Read the relevant page in `docs/specs/` and the relevant code (the code map is in `CLAUDE.md`).
- Write a plan of 3–7 bullets listing the files to change.
- For large work (new dependency, new permission, UX change, prompt change, more than ~200 lines): present the plan and **wait for the user's approval** before editing.

## 3. Implement
- Change in small steps. After each meaningful step, run `node scripts/harness/check.mjs`.
- Follow `CLAUDE.md` → Code rules. The PostToolUse hook reports violations right after each edit; fix them immediately.
- New interface text: add it to `src/lib/i18n.ts` (or `src-tauri/src/i18n.rs`) in both `vi` and `en`. Everything else is written in English.
- New setting: add it to `settings.rs` (including `Default`), `api.ts` and the Settings UI.
- New command or event: register it on both sides. New window or plugin API in the frontend: add its permission to the capabilities.
- New pure logic: add unit tests in the same file.

## 4. Finish
- Run `node scripts/harness/check.mjs --full`; everything must pass.
- Significant changes to `platform/win`, `capture.rs`, `redact.rs`, `llm/` or capabilities: ask the `tonesnap-reviewer` agent for a review and address its findings.
- Prompt or model change: try the sample cases in `scripts/bench-cases.json` on a real provider (`/smoke-test`) and compare with the previous answers.
- Update `docs/PLAN.md`: tick the item and add any new decision to the Decisions table.
- Update `docs/specs/`: describe the new or changed behavior as numbered rules in the matching page (new rules take the next free ID), with the tests that verify them.
- Win32, UI, hotkey or clipboard changes: write concrete manual test steps (which app, what to select, what to press, what to expect) for the user. Never send synthetic keystrokes or mouse input to their desktop.
- Report to the user in their language: what changed (with file links), what was verified automatically, and what they need to try.
