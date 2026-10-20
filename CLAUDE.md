# ToneSnap: agent guide

A Tauri 2 desktop app (Rust core, React/TS interface), Windows first, macOS later. The user selects text in any app and presses a hotkey (default `Ctrl+Alt+E`) or clicks the floating icon. A popup shows English versions in several tones with explanations in Vietnamese, and Apply pastes the chosen version back into the original app.

- `docs/PLAN.md` is the single source of truth for decisions, roadmap, progress and the compatibility table.
- `docs/specs/` describes how the built features behave (business rules, edge cases, error codes, Rust ⇄ TS shapes), one page per area. Read the relevant page before changing behavior.
- `docs/HARNESS.md` explains the harness: checks, hooks, skills.

## Commands

| Task | Command |
|---|---|
| Run the app in development | `pwsh -File dev.ps1` (run in the background; Rust changes rebuild and restart the app) |
| Fast check | `node scripts/harness/check.mjs` |
| Full check (before calling work done) | `node scripts/harness/check.mjs --full` (adds the build and the Win32 integration tests) |
| Frontend tests only | `pnpm test` |
| Key-press tests (Ctrl+C/Ctrl+V into a test window) | `node scripts/harness/check.mjs --only=input`; ask the user first, it takes focus |
| Project rules only | `node scripts/harness/invariants.mjs` |
| Format and auto-fix (rustfmt, ESLint, Prettier, whitespace) | `node scripts/harness/check.mjs --fix` (or `pnpm run fix`, which re-runs the quick gates) |
| Git hooks for contributors (optional) | `pnpm run hooks:install` (pre-commit: quick style gates; pre-push: the CI gates) |
| Dependency audit (cargo deny, pnpm audit; needs network) | `node scripts/harness/check.mjs --only=deps` |

`cargo` lives in `~/.cargo/bin` and may be missing from a terminal's PATH; the harness scripts and `dev.ps1` add it.

## Code map

| Path | Role |
|---|---|
| `src-tauri/src/lib.rs` | App setup only: plugins, managed state, window events, the command list |
| `src-tauri/src/session.rs`, `request.rs` | Trigger → capture → redacted `Session` → popup; streaming a request into `request-*` events |
| `src-tauri/src/window.rs`, `tray.rs`, `trigger.rs` | Popup/Settings windows and placement; tray menu; hotkey and floating icon |
| `src-tauri/src/commands.rs`, `state.rs` | Every `#[tauri::command]` and `CommandError`; `AppState` (settings, session, request cancellation) |
| `src-tauri/src/capture.rs` | Selection and context: UIA → clipboard → OCR |
| `src-tauri/src/platform/` | `mod.rs` is the platform-neutral facade (non-Windows stubs in `stub.rs`); `win/` holds all Win32/COM: `uia`, `clipboard`, `input`, `ocr`, `window`, `com` (MTA threads), `selection_icon` (floating icon + mouse hook) |
| `src-tauri/src/geometry.rs` | `Rect` in physical pixels, shared by capture, placement and the platform layer |
| `src-tauri/src/redact.rs` | Reversible redaction of sensitive data |
| `src-tauri/src/prompt.rs` + `src-tauri/prompts/` | System prompt, output JSON schema, user turn |
| `src-tauri/src/llm/` | AI providers: `claude.rs` (paid) and `openai_compat.rs` (Gemini, Groq, OpenRouter; free tiers) |
| `src-tauri/src/update.rs` | Updates from the GitHub releases (Tauri updater plugin): schedule, update mode, install when idle |
| `src-tauri/src/settings.rs`, `secrets.rs` | Settings (JSON in the app config dir), API key (Credential Manager) |
| `src-tauri/src/i18n.rs` | Strings rendered by Rust (tray menu) |
| `src/popup/`, `src/settings/` | The two windows: a state hook (`usePopupState`, `useSettingsForm`) plus small components (`settings/sections/`) |
| `src/lib/api.ts` | Types, typed `invoke` wrappers for every command and typed `events` subscriptions |
| `src/lib/i18n.ts` | Every interface string, in Vietnamese and English |
| `src/test/` | Frontend test helpers: a fake backend on Tauri's `mockIPC`, fixtures matching the Rust defaults |
| `scripts/harness/` | Checks and hooks (see `docs/HARNESS.md`) |

## Invariants

Items marked ★ are checked by `scripts/harness/invariants.mjs`; the Stop hook does not let a turn end while they fail.

1. **Privacy ★.** Never log, store or send user text anywhere except the AI request. Every provider call goes through the redacted `Payload` (`redact.rs`). Screenshots are only OCR'd on the device. No telemetry. No `println!`/`dbg!`/`console.log`; `eprintln!` is for diagnostics that contain no user text.
2. **Rust ⇄ TS contracts ★.** These five pairs must always match; change both sides in the same change:
   - `Settings`: `settings.rs` ⇄ `api.ts`, serialized in camelCase.
   - Tones: `prompt.rs TONES` ⇄ `api.ts ToneId` ⇄ `tones.ts TONE_IDS`.
   - Commands: `#[tauri::command]` + `generate_handler!` ⇄ `invoke`.
   - Events: `emit`/`emit_to` ⇄ `listen`.
   - Permissions in `src-tauri/capabilities/` for every window or plugin API the frontend calls.
3. **i18n ★.** Every interface string lives in `src/lib/i18n.ts` (or `src-tauri/src/i18n.rs` for Rust-rendered text) in both `vi` and `en`; TypeScript reports missing keys. Button labels use title case and never wrap.
4. **Prompt.** `src-tauri/prompts/system.md` and `output_schema.json` are the single source: the app embeds them with `include_str!`. After changing them or `prompt::user_turn`/`prompt::reminder`, try the sample cases in `scripts/bench-cases.json` on a real provider (`/smoke-test`) and compare with the previous answers.
5. **Win32.** COM runs on dedicated (MTA) threads. Never block Tauri's main thread. Always restore the clipboard. The floating icon never takes focus. ★ The `windows` crate stays on Tauri's version (currently 0.62).
6. **Network ★.** `reqwest` uses the OS TLS stack so it works behind corporate TLS inspection (Netskope, Zscaler…), whose root certificates live in the OS trust store; do not switch to rustls.
7. **Secrets ★.** API keys live only in the credential store, accessed through `secrets.rs`. Never in files, logs or tests.
8. **Hands off ★:** `src-tauri/gen/`, lockfiles, `node_modules/`, `target/`, `dist/`. The PreToolUse hook blocks edits.

## Code rules

Rules marked ★ are enforced by the `code`, `network`, `language`, `whitespace` and `suppress` rules in `invariants.mjs`; rules marked (clippy) or (ESLint) by the lints in `src-tauri/Cargo.toml` and `eslint.config.js`. The PostToolUse hook reports violations right after an edit and formats the file (rustfmt, Prettier); the Stop hook blocks the turn until the gates pass.

**Rust**
- ★ `unsafe` only in `src-tauri/src/platform/`. Release every handle on every path.
- (clippy) Every `unsafe` block is preceded by a `// SAFETY:` comment saying why it is sound: which handle is valid, which buffer is large enough, which thread has COM.
- ★ No `.unwrap()` except directly after `.lock()`. Use `?`, a fallback such as `unwrap_or_default()`, or `.expect("why this cannot fail")`.
- ★ (clippy) No `todo!`, `unimplemented!`, `panic!`, `dbg!` or `println!` in shipped code. Commands return `Result` with an error code the UI translates (see `CommandError`).
- ★ (clippy) `#[allow(...)]` only with a reason: `#[allow(lint, reason = "why")]`.
- Slow or blocking work (Win32, COM, file and network I/O) runs off Tauri's main thread. Synchronous `#[tauri::command]`s run on that thread, so a command that touches files, the credential store, the network or the clipboard is `async` or hands off to its own thread.
- Code outside `platform/` calls the platform-neutral functions in `platform/mod.rs` instead of adding `#[cfg(windows)]` branches.
- Test automatically whatever can be, at the lowest layer that shows the behavior; leave to manual testing only what depends on other apps, the global hotkey or the mouse hook. See "Tests" in `docs/HARNESS.md`.
  - New pure logic gets unit tests in the same file (`#[cfg(test)] mod tests`).
  - Data Rust sends to the UI gets a test of its JSON shape.
  - Provider code is tested against `llm/test_server.rs`; Win32 code against real windows the test owns (`platform/win/test_window.rs`) in `mod integration_tests` marked `#[ignore = "win32 integration: run by check.mjs --full"]`.
  - UI behavior gets a vitest test next to the component, with the backend faked by `src/test/backend.ts`.
  - Windows tests are written here; macOS tests are written on a Mac.

**TypeScript / React**
- ★ No `any`, `@ts-ignore` or `@ts-nocheck`. `@ts-expect-error` needs a description.
- (ESLint) A command or promise that is deliberately not awaited is marked `void api.x()`; no `setState` directly in an effect body; hook dependencies are complete; a `switch` over a union lists every case instead of `default`; no `console`.
- Formatting is Prettier's (`.prettierrc.json`, 120 columns); never format by hand. ESLint also covers `scripts/` and `eslint.config.js`.
- ★ `invoke` and `listen` only in `src/lib/api.ts`; components call its typed wrappers and `events` subscriptions. Keep each event name a literal in its `listen` call so the `events` check can see it.
- State shared between windows goes through Rust events, not through browser storage.

**Everywhere**
- ★ Network calls only to the hosts in `scripts/harness/config.mjs`: the providers (`api.anthropic.com`, `generativelanguage.googleapis.com`, `api.groq.com`, `openrouter.ai`), GitHub for updates (`github.com` and its asset hosts; version and platform only) and localhost (tests). A new endpoint is a privacy decision: ask the user first.
- ★ Text files use LF, no trailing spaces (Markdown excepted), one final newline and spaces for indentation (`whitespace`); lint and format suppressions are not allowed: `eslint-disable` needs a reason after ` -- `, and `prettier-ignore` and `rustfmt::skip` are banned (`suppress`).
- ★ **The repository is written in English**: code, comments, docs, skills, commit messages. Vietnamese is allowed only in `src/lib/i18n.ts`, `src-tauri/src/i18n.rs`, Vietnamese test inputs (`#[cfg(test)]` modules, `scripts/bench-cases.json`) and recorded model output in `docs/bench/` (history of the removed local-model benchmarks). The PreToolUse hook denies edits that add Vietnamese anywhere else.
- Dependencies: ask before adding or removing one. The hooks ask the user on edits to `Cargo.toml`/`package.json` and on `cargo add`, `pnpm add`, `npm install <pkg>`. Prefer the standard library and crates already in use. A new dependency must pass `deny.toml` (permissive licenses, crates.io only, no known vulnerabilities) and `pnpm audit`.
- Files over 800 lines get a warning; split them by responsibility.
- Match the surrounding style. Comments explain why, not what. No commented-out or dead code.
- Keep changes focused on the task: no unrelated refactors.

## Workflow

1. Read `docs/PLAN.md` and the relevant files before editing. For large work (new dependency, new permission, UX change, more than ~200 lines), present a short plan and ask first.
2. Work in small steps.
3. Run `node scripts/harness/check.mjs`. The Stop hook also re-runs the gates for the source groups you changed.
4. Add or update automated tests for the change (see Code rules). Only behavior that depends on other apps, the global hotkey or the mouse hook needs manual steps or `/smoke-test`. **Never send synthetic keystrokes or mouse input to the user's desktop without asking**, and that includes the `input` gate; they could land in the window the user is working in.
5. Update `docs/PLAN.md`: tick finished items, add new decisions to the Decisions table, fill in the compatibility table when real tests come in. Update the matching page in `docs/specs/` when behavior changes or a PLAN item is finished.
6. Odd frontend errors right after several quick edits usually mean Vite kept a stale module: restart `dev.ps1`.
7. Git: `master` and `develop` are protected and change only through pull requests ([CONTRIBUTING.md](CONTRIBUTING.md)). Work on a topic branch cut from `develop` (`feat/…`, `fix/…`, `docs/…`, `chore/…`) and open the PR against `develop`. Commit only when the user asks, and only with `check.mjs --full` passing. Commit messages are in English, in the imperative mood, with a summary line under ~72 characters.

## Definition of done

- `node scripts/harness/check.mjs --full` passes.
- New interface text exists in Vietnamese and English.
- `docs/PLAN.md` is updated, and `docs/specs/` describes the new or changed behavior.
- The change has automated tests; manual test steps are given only for what automation cannot reach (other apps, the global hotkey, the mouse hook).
- Prompt or model changes: the sample cases in `scripts/bench-cases.json` were tried on a real provider and quality did not drop.
- Significant changes to `platform/win`, `capture.rs`, `redact.rs`, `llm/` or capabilities: reviewed by the `tonesnap-reviewer` agent.

## Skills and agents

- `/check [--full]`: run the checks and fix what fails.
- `/feature <item>`: implement a PLAN item following the workflow above.
- `/smoke-test [app…]`: test with the user in real apps and fill in the compatibility table.
- `tonesnap-reviewer` agent: read-only review against the invariants and code rules.

## Environment

- Windows 11 with PowerShell 7; Git Bash is installed and runs the hooks.
- Corporate networks may inspect TLS or use a PAC proxy; the app and the harness must keep working there (see invariant 6).
- `Ctrl+Alt+Space` belongs to Claude desktop's Quick Entry, hence the `Ctrl+Alt+E` default.
- Default provider: Gemini's free tier (needs an API key from Google AI Studio). Groq, OpenRouter and Claude are options; there is no local model.

## Communication

Talk to the user in the language they write in (usually Vietnamese). Everything written into the repository is in English.
