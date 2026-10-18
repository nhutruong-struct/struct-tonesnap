---
name: check
description: Run ToneSnap's checks (Rust ⇄ TS contracts, privacy, secrets, code rules, language, rustfmt, clippy, unit tests, tsc, vite build) and fix what fails. Use when the user asks to check, test or verify the build (in any language), or before reporting work as done.
argument-hint: "[--full | --changed | --only=invariants,tsc,clippy,test,rustfmt,build]"
allowed-tools: Bash(node scripts/harness/*) PowerShell(node scripts/harness/*)
---

# Check and fix

1. Run `node scripts/harness/check.mjs $ARGUMENTS`. Without arguments it runs the fast mode. Before reporting a feature as done, always run `--full`.
2. Fix every failing gate at its root:
   - `invariants`: read the message.
     - Contract rules: fix **both sides** (Rust and TS), or add the permission to `src-tauri/capabilities/`.
     - `code` rules: rewrite the code to follow `CLAUDE.md` → Code rules. Do not silence them.
     - `language`: write the text in English, or move interface text into the i18n files.
     - The rules live in `scripts/harness/invariants.mjs`, the exceptions in `scripts/harness/config.mjs`.
   - `rustfmt`: run `node scripts/harness/check.mjs --fix`.
   - `clippy`: apply the suggestion. Use `#[allow(...)]` only with a stated reason in a comment next to it.
   - `test`: fix the code. Change a test only when the test is wrong, and say why.
   - `tsc` / `build`: fix the types. Never reach for `any` or `@ts-ignore`.
3. Re-run until everything passes.
4. Report briefly: the gates that ran, the results, and what you fixed where (as `file:line`).

Never loosen a rule in `config.mjs` or `invariants.mjs` just to make a check pass. If you believe a rule is wrong, explain it to the user, propose the rule change, and wait for their agreement.
