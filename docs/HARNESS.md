# ToneSnap development harness

The harness is the frame around development:

- **Context:** tells the agent where the project stands and what it must respect.
- **Gates:** catch mistakes right after a change.
- **Code rules:** constrain how code gets written.
- **Hooks:** block dangerous actions.
- **Workflows:** standard steps for recurring work.

It works the same whether you write the code yourself or let Claude Code do it.

## Components

| Component | File | Role |
|---|---|---|
| Context | `AGENTS.md` | Commands, invariants, rules for every file, workflow, definition of done. Claude Code (v2.1.277 or later) and other coding agents read it at the start of every session; kept short because it is sent with every request |
| Code rules by language | `.claude/rules/rust.md`, `.claude/rules/typescript.md` | Rust and TypeScript rules and test conventions. Their `paths:` frontmatter makes Claude load each one only when it opens a matching file |
| Code map | This page, [Code map](#code-map) | Where each part of the app lives |
| Gates | `scripts/harness/check.mjs` | Runs the gates: project rules, rustfmt, tsc, clippy, unit tests, vite build |
| Project rules | `scripts/harness/invariants.mjs` | Rules compilers cannot see (below) |
| Configuration | `scripts/harness/config.mjs` | Protected paths, secret patterns, risky commands, permission map, allowed hosts, language allowlist |
| Hooks | `scripts/harness/hooks.mjs` + `.claude/settings.json` | Wire the harness into Claude Code's loop |
| Git hooks | `.githooks/pre-commit`, `.githooks/pre-push` | Optional: run the gates before a commit or push (see Enforcement layers) |
| PR title rules | `scripts/harness/pr-title.mjs` | The pull request title becomes the commit message; CI checks its form |
| Workflows | `.claude/skills/*/SKILL.md` | `/check`, `/feature`, `/smoke-test` |
| Review | `.claude/agents/tonesnap-reviewer.md` | Read-only agent that reviews against the invariants and code rules |
| Superpowers | `enabledPlugins` in `.claude/settings.json`, `AGENTS.md` → Superpowers | Trial of the Superpowers plugin (brainstorming, plans, TDD, subagents). `AGENTS.md` decides where its specs go (`docs/superpowers/`, gitignored), where worktrees live, and when it may commit. `/feature` stays as the project's own workflow for comparison |
| State | `.claude/.state/harness.json` | Last check result and the source groups changed since (not committed); each worktree keeps its own |

## Code map

| Path | Role |
|---|---|
| `src-tauri/src/lib.rs` | App setup only: plugins, managed state, window events, the command list |
| `src-tauri/src/session.rs`, `request.rs` | Trigger → capture → redacted `Session` → popup; streaming a request into `request-*` events |
| `src-tauri/src/window.rs`, `tray.rs`, `trigger.rs` | Popup/Settings windows and placement; tray menu; hotkey and floating icon |
| `src-tauri/src/commands.rs`, `state.rs` | Every `#[tauri::command]` and `CommandError`; `AppState` (settings, session, request cancellation) |
| `src-tauri/src/capture.rs` | Selection and context: UIA → clipboard → OCR |
| `src-tauri/src/platform/` | `mod.rs` is the platform-neutral facade (stubs in `stub.rs`); `gesture.rs` is the selection-gesture logic shared by both icons; `mac/` holds the macOS layer (`ax` Accessibility reads, `window` pointer, monitors and frontmost app, `cf` Core Foundation, `clipboard`, `input`, `panel`, `selection_icon`); `win/` holds all Win32/COM: `uia`, `clipboard`, `input`, `ocr`, `window`, `com` (MTA threads), `selection_icon` (floating icon + mouse hook) |
| `src-tauri/src/geometry.rs` | `Rect` in physical pixels, shared by capture, placement and the platform layer |
| `src-tauri/src/redact.rs` | Reversible redaction of sensitive data |
| `src-tauri/src/prompt.rs` + `src-tauri/prompts/` | System prompt, output JSON schema, user turn |
| `src-tauri/src/llm/` | AI providers: `claude.rs` (paid) and `openai_compat.rs` (Gemini, Groq, OpenRouter; free tiers) |
| `src-tauri/src/update.rs` | Updates from the GitHub releases (Tauri updater plugin): schedule, update mode, install when idle |
| `src-tauri/src/settings.rs`, `secrets.rs` | Settings (JSON in the app config dir), API keys (Credential Manager, one per provider) |
| `src-tauri/windows/hooks.nsh` | NSIS uninstaller hook: deletes the API keys when *Delete the application data* is ticked |
| `src-tauri/src/i18n.rs` | Strings rendered by Rust (tray menu) |
| `src/popup/`, `src/settings/` | The two windows: a state hook (`usePopupState`, `useSettingsForm`) plus small components (`settings/sections/`) |
| `src/lib/api.ts` | Types, typed `invoke` wrappers for every command and typed `events` subscriptions |
| `src/lib/i18n.ts` | Every interface string, in Vietnamese and English |
| `src/test/` | Frontend test helpers: a fake backend on Tauri's `mockIPC`, fixtures matching the Rust defaults |
| `scripts/harness/` | Checks and hooks (this page) |
| `scripts/release/` | `updater-manifest.mjs`: builds the updater's `latest.json` for `release.yml` |
| `site/` | The website (GitHub Pages, `pages.yml`); `site/media/` holds the intro video and its poster |

## Enforcement layers

The same checks run in four places, from the quickest feedback to the one that cannot be skipped:

| Layer | When | What | Can be skipped? |
|---|---|---|---|
| Editor | While typing | `.editorconfig` (LF, spaces, final newline), the recommended VS Code extensions | Yes |
| Claude Code hooks | Around each edit and turn (`.claude/settings.json`) | Formats edited files, reports project-rule violations, blocks bad edits, and re-runs the gates before a turn may end | Only for people not using Claude Code |
| Git hooks (optional) | `git commit` and `git push` (`.githooks/`, enabled with `pnpm run hooks:install`) | `pre-commit`: project rules, rustfmt, Prettier. `pre-push`: every fast gate | Yes (`--no-verify`) |
| CI | Every pull request, forks included | The `check` job: PR title, then `check.mjs` (below). It is a required status check in both rulesets | **No** |

CI is the real gate. The first three layers exist so that a contributor meets the failure at their desk, with the
fix at hand (`pnpm run fix`), rather than in a red pull request. What the `check` job runs:

- the PR title rules (`pr-title.mjs`; the title becomes the squashed commit message), on `pull_request` events,
  skipped for Dependabot whose titles are generated;
- the frontend build (`vite build`), because `tauri-build` embeds `dist/`;
- `check.mjs`, the fast gates below, with every failure printed with a hint, as a line annotation on the pull
  request (`::error file=…,line=…`) and in the job summary.

The gates run the code of the pull request itself, so a pull request could weaken them. `.github/CODEOWNERS`
makes the owner review every change under `scripts/`, `.github/`, `.githooks/` and the lint and format
configuration (`eslint.config.js`, `.prettierrc.json`, `.prettierignore`, `.editorconfig`, `tsconfig*.json`,
`src-tauri/clippy.toml`, `src-tauri/Cargo.toml`), and workflows from outside contributors need approval before
they run. Not in CI: the Win32 integration tests (they need an interactive desktop), the key-press tests, and
`check.mjs --full`'s `deps` gate, which `audit.yml` runs when dependency files change.

## How one Claude turn passes through the harness

```
Session opens ── SessionStart ──▶ open PLAN.md items and the last check result
   │
   ├─ About to edit / run a command ── PreToolUse ──▶ deny when:
   │      · the file is a lockfile, src-tauri/gen, node_modules, target or dist
   │      · the new content contains a real API key or token
   │      · the edit adds Vietnamese text outside the i18n files
   │    ask the user first for: Cargo.toml / package.json edits, cargo add / pnpm add / npm install <pkg>,
   │      force push, git reset --hard, publish, recursive deletes of source folders…
   │
   ├─ After an edit / command ── PostToolUse ──▶ rustfmt the edited .rs file, report project-rule
   │      violations in the edited file, record which groups changed (rust / web / harness)
   │
   └─ About to finish ── Stop ──▶ re-run the gates for the changed groups
          · pass: "all gates passed"
          · fail: the turn may not end; the errors go back to Claude (at most 3 times, then it stops and tells you)
```

A turn that only answers a question changes nothing, so the Stop hook exits immediately.

### Git worktrees

Parallel Claude sessions each work in a git worktree under `.claude/worktrees/<name>/`. Claude Code starts the hooks from the main checkout (`CLAUDE_PROJECT_DIR`) even then, so the hooks work out the session's own checkout from the `cwd` in the hook payload (`locate` in `config.mjs`):

- An edited file is judged by its path inside the checkout that holds it, so a worktree's `src/lib/i18n.ts` is `src/lib/i18n.ts` for the protected paths, the language allowlist and the manifest rule.
- SessionStart, PostToolUse and Stop read and write that checkout's `.claude/.state/harness.json`, and Stop runs that checkout's own `check.mjs`, so one session is never blocked by another session's unfinished edits.
- The gates need `node_modules` in the worktree (`pnpm install` there). PostToolUse borrows the main checkout's Prettier when the worktree has none.

## Gates

| Gate | Runs when this changed | Time (warm build) |
|---|---|---|
| `invariants` | always | < 0.2 s |
| `rustfmt` | rust | ~0.1 s |
| `prettier` (`--check`) | web, harness | ~0.7 s |
| `harness` (`node --test scripts/harness`: tests of the rules below) | harness | < 0.5 s |
| `clippy` (`-D warnings` + the lints in `Cargo.toml`) | rust | ~3–8 s |
| `test` (`cargo test --lib`) | rust | ~3 s |
| `tsc` | web | ~1 s |
| `eslint` (type-aware, React hooks, `--max-warnings 0`; `scripts/` and `eslint.config.js` get the recommended Node rules) | web, harness | ~3 s |
| `vitest` | web | ~2 s |
| `build` (vite) | only with `--full` | ~0.5 s |
| `win32` | only with `--full` | ~0.5 s |
| `deps` (`cargo deny check`, `pnpm audit`; needs network) | only with `--full` | ~10 s |
| `input` | only with `--only=input` | ~3 s |
| `pr-title` (`PR_TITLE="…"`; CI sets it) | only with `--only=pr-title` | instant |

The first run after `target/` is deleted takes a few minutes because everything rebuilds.

## Tests

Test automatically whatever can be; test by hand only what cannot.

| Layer | Where | What it covers |
|---|---|---|
| Rust unit tests | `#[cfg(test)] mod tests` next to the code | Redaction rules, prompt and schema (language, mode, tones), settings loading (bad fields, unreadable files, atomic save), request cancellation and the capture guard in `AppState`, the update decisions in `update.rs` (what to do next, when ToneSnap is idle, retry timing), tray strings, popup placement, OCR area, app names, selection-gesture detection, icon resampling, stop reasons, and the JSON shapes Rust sends to the UI (Session, Notice, RequestEvent, CommandError, SettingsIssue, UpdateInfo) |
| Provider tests | `llm/claude.rs`, `llm/openai_compat.rs`, with `llm/test_server.rs` | The providers against a local HTTP server: streaming split across chunks, request headers and body, HTTP and in-stream errors, cancellation, unreachable servers |
| Release scripts | `scripts/release/*.test.mjs` | The updater manifest: keys, URLs and missing signatures |
| Win32 integration (`win32` gate) | `mod integration_tests`, `#[ignore]`d | UI Automation on a real off-screen Edit control (selection, context, a multi-line selection read paragraph by paragraph, password, empty), on-device OCR of rendered text (including a second call on a new thread), clipboard snapshot and restore, process and monitor queries. Briefly replaces the clipboard and restores it; never takes focus |
| Input (`input` gate) | `mod input_tests`, `#[ignore]`d | Ctrl+C capture and Apply (Ctrl+V) into a focused test window, with the clipboard restored. It takes focus and sends key presses, so it runs only when the user asks for it, while nobody is typing |
| Frontend (`vitest` gate) | `src/**/*.test.ts(x)`, helpers in `src/test/` | Pure logic (hotkeys, i18n parity between languages, redaction restore, streaming suggestions) and the two windows rendered against a fake backend built on Tauri's `mockIPC`: event streams, keys, language and mode, Apply/Copy, errors, notices, preview, every Settings section (including Updates), saving settings, the API key, reopening |

**Still manual** (`/smoke-test`): the global hotkey through Tauri, the low-level mouse hook and floating icon in other apps, focus and paste behavior in real third-party apps (the compatibility table), and model quality (try `scripts/bench-cases.json` on a real provider). macOS tests are written on a Mac.

## Lint and format

| Tool | Config | Covers |
|---|---|---|
| Clippy lints | `[lints.clippy]` in `src-tauri/Cargo.toml`, `src-tauri/clippy.toml` | `// SAFETY:` comment on every `unsafe` block; no `dbg!`, `println!`, `todo!`, `unimplemented!`, `panic!` (tests may panic); `#[allow]` needs a `reason` |
| ESLint | `eslint.config.js` | `src/`: type-aware TypeScript rules, React hooks (dependencies, no setState in effect bodies), promises that are neither awaited nor marked `void`, exhaustive `switch`, no `console`. `scripts/` and `eslint.config.js`: ESLint's recommended rules |
| Prettier | `.prettierrc.json`, `.prettierignore` | Formatting of TS, CSS, HTML, JSON, YAML (workflows, templates), the website and the harness scripts. Markdown, `docs/`, `.claude/` and `src-tauri/` are left alone (rustfmt formats Rust; prose and tables are hand-aligned) |
| cargo-deny | `src-tauri/deny.toml` | RustSec advisories, permissive licenses only, crates.io as the only source, yanked or unmaintained direct dependencies |
| Line endings | `.gitattributes`, `.editorconfig` | LF everywhere; editors follow the formatters' indentation |

The PostToolUse hook runs rustfmt or Prettier on every edited file, so formatting never needs a separate step. `check.mjs --fix` runs `cargo fmt`, `eslint --fix`, the whitespace repairs and `prettier --write` over everything; `pnpm run fix` does that and re-runs the quick gates.

## Project rules

**Per-file rules.** They judge one file at a time, so the hooks can also run them on a single edit. They apply to every text file git would commit (tracked, or untracked and not ignored), wherever it lives: `.github/`, `site/`, the root Markdown files and the dot-files included. Lockfiles are exempt from the whitespace and secret rules, and `docs/bench/` (recorded model output) from the whitespace rule.

| Rule | Catches |
|---|---|
| `language` | Vietnamese text outside `src/lib/i18n.ts`, `src-tauri/src/i18n.rs`, Vietnamese test inputs and `docs/bench/` |
| `privacy` | `println!`, `dbg!`, `console.log`; `eprintln!` printing variables that hold user text |
| `code` | `unsafe` outside `platform/`; `.unwrap()` outside a mutex lock; `todo!`/`unimplemented!`/`panic!`; `#[allow]` without a reason; TypeScript `any`, `@ts-ignore`, an undescribed `@ts-expect-error`; `invoke` or `listen` outside `api.ts`; a warning for files over 800 lines |
| `network` | URLs to hosts other than the AI providers, GitHub (updates) and localhost (`ALLOWED_HOSTS` in `config.mjs`; the key pages the interface opens in the browser are `BROWSER_LINK_HOSTS`) |
| `secrets` | Real API keys or tokens anywhere in the tree |
| `whitespace` | What `.editorconfig` asks of files no formatter touches (Markdown, TOML, prompts, `src-tauri/` JSON, PowerShell…): CR line endings, trailing spaces (Markdown excepted), a missing or doubled final newline, tab indentation (Rust excepted). `--fix` repairs all but tabs |
| `suppress` | `eslint-disable` without a reason after ` -- `, inline ESLint rule settings, `prettier-ignore`, `rustfmt::skip`: ways to hide a problem from the gates |

**Cross-file rules.** They check the contracts between the Rust core and the React UI.

| Rule | Catches |
|---|---|
| `tones` | Tone lists that differ between `prompt.rs`, `api.ts` and `tones.ts` |
| `settings` | `Settings` fields or enum values that differ between Rust and TS |
| `commands` | `invoke("x")` without a matching command, or a command missing from `generate_handler!` |
| `events` | The UI listening for an event Rust never emits |
| `capabilities` | A window or plugin API used without its permission; a capability naming a window that does not exist |
| `setup` | `reqwest` switched to rustls (breaks behind TLS-inspecting proxies); the `windows` crate drifting from Tauri's version; dev port mismatch between Tauri and Vite |

Every rule was tested by planting the violation in a copy of the repository and confirming it is caught.

## Everyday use

```bash
npm run check
```

```bash
npm run check:full
```

```bash
node scripts/harness/check.mjs --only=invariants,tsc
```

```bash
node scripts/harness/check.mjs --fix
```

```bash
npm run check:input
```

```bash
pnpm run fix
```

```bash
pnpm run hooks:install
```

The first command is the fast check and the second the full one, to run before calling work done. The third runs selected gates only, and the fourth runs `cargo fmt` before checking. The key-press tests (`check:input`) take focus: start them yourself while you are not using the PC (Claude asks before running them). `pnpm run fix` formats everything and repairs whitespace, then re-runs the quick style gates; `pnpm run hooks:install` enables the optional git hooks.

In Claude Code:

- `/check`: run the checks and fix what fails.
- `/feature <task>`: implement an item following the workflow. Without a task, Claude lists the open PLAN items to choose from.
- `/smoke-test [app…]`: Claude guides, you operate the real apps, and Claude records the results in the compatibility table.

## Extending

- **Exceptions** (protected paths, allowed hosts, language allowlist, permission map): edit `scripts/harness/config.mjs`.
- **A per-file rule**: add a function to `FILE_RULES` in `invariants.mjs`, returning `[{ line, message, severity? }]`.
- **A cross-file rule**: add a function to `CROSS_RULES`, returning `{ errors, warnings }`.
- **A gate**: add it to `GATES` in `check.mjs` and map it to a source group in `GATES_FOR_GROUP` (`config.mjs`).
- Test every new rule: the per-file rules and the PR title rules have unit tests in `scripts/harness/invariants.test.mjs`, the hooks' path handling (worktrees included) in `scripts/harness/hooks.test.mjs` (the `harness` gate; build samples that would trip the rule from pieces so the test file passes its own checks), the cross-file rules by planting a violation in a copy of the repository.

## Troubleshooting

- **Hooks do not run:**
  - Open Claude Code in the `tonesnap` folder, where `.claude/settings.json` lives (a session in one of its worktrees works too).
  - Hooks are read when a session starts; open a new session after changing `settings.json`.
  - Type `/hooks` in a `claude` terminal to list them.
  - Hooks run under Git Bash. Without Git Bash, Claude Code falls back to PowerShell, where `${CLAUDE_PROJECT_DIR}` may not expand.
- **A check sits at "Blocking waiting for file lock":** `dev.ps1` is building; cargo continues when it finishes.
- **The Stop hook keeps blocking:** it blocks at most 3 times in a row, then lets the turn end and tells you. Run `npm run check` to see the errors.

## Limits

- Behavior that depends on other apps (their UI Automation support, focus and paste handling), the global hotkey and the mouse hook cannot be tested automatically; use `/smoke-test`. Everything else has automated tests (see Tests above).
- The hooks detect changes from file fingerprints (path, size, modification time) rather than git, so they also see edits made through shell commands and work before anything is committed.
