# ToneSnap: agent guide

A Tauri 2 desktop app (Rust core, React/TS interface), Windows first, macOS later. The user selects text in any app and clicks the floating icon or presses a hotkey (default `Ctrl+Alt+E`). A popup streams one version per tone in the chosen language and mode (translate or rewrite); Apply pastes the chosen version back into the original app.

- `docs/PLAN.md`: decisions, roadmap, progress, compatibility table (the single source of truth).
- `docs/specs/`: how built features behave, one page per area. Read the relevant page before changing behavior.
- `docs/HARNESS.md`: checks, hooks, skills, and the code map.
- Rust and TypeScript code rules: `.claude/rules/rust.md` and `.claude/rules/typescript.md` (Claude loads them when it opens matching files).

## Commands

| Task | Command |
|---|---|
| Run the app in development | `pwsh -File dev.ps1` (in the background; Rust changes rebuild and restart the app) |
| Fast check / full check (before calling work done) | `node scripts/harness/check.mjs` / `--full` (adds the build and Win32 integration tests) |
| Format and auto-fix | `node scripts/harness/check.mjs --fix` |
| Key-press tests (take focus: ask the user first) | `node scripts/harness/check.mjs --only=input` |
| Dependency audit (needs network) | `node scripts/harness/check.mjs --only=deps` |

`cargo` lives in `~/.cargo/bin` and may be missing from PATH; the harness scripts and `dev.ps1` add it.

## Invariants

★ = checked by `scripts/harness/invariants.mjs`; the Stop hook does not let a turn end while one fails.

1. **Privacy ★.** Never log, store or send user text anywhere except the AI request. Every provider call goes through the redacted `Payload` (`redact.rs`). Screenshots are OCR'd on the device only. No telemetry. `eprintln!` only for diagnostics without user text.
2. **Rust ⇄ TS contracts ★.** Change both sides in the same change: `Settings` (`settings.rs` ⇄ `api.ts`, camelCase); tones (`prompt.rs TONES` ⇄ `api.ts ToneId` ⇄ `tones.ts TONE_IDS`); commands (`#[tauri::command]` + `generate_handler!` ⇄ `invoke`); events (`emit`/`emit_to` ⇄ `listen`); permissions in `src-tauri/capabilities/` for every window or plugin API the frontend calls.
3. **i18n ★.** Every interface string lives in `src/lib/i18n.ts` (or `src-tauri/src/i18n.rs` for Rust-rendered text) in both `vi` and `en`. Button labels use title case and never wrap.
4. **Prompt.** `src-tauri/prompts/system.md` and `output_schema.json` are the single source (`include_str!`). After changing them or `prompt::user_turn`/`prompt::reminder`, run `scripts/bench-cases.json` on a real provider (`/smoke-test`) and compare with the previous answers.
5. **Win32.** COM runs on dedicated (MTA) threads. Never block Tauri's main thread. Always restore the clipboard. The floating icon never takes focus. ★ The `windows` crate stays on Tauri's version (currently 0.62).
6. **Network ★.** `reqwest` uses the OS TLS stack (corporate TLS inspection puts its root certificates there); never rustls. Network calls only to the hosts in `scripts/harness/config.mjs` (AI providers, GitHub for updates, localhost for tests). A new endpoint is a privacy decision: ask the user first.
7. **Secrets ★.** API keys live only in the credential store, through `secrets.rs`. Never in files, logs or tests.
8. **Hands off ★:** `src-tauri/gen/`, lockfiles, `node_modules/`, `target/`, `dist/`.

## Rules for every file

- ★ The repository is written in English. Vietnamese only in `src/lib/i18n.ts`, `src-tauri/src/i18n.rs`, Vietnamese test inputs (`#[cfg(test)]` modules, `scripts/bench-cases.json`) and `docs/bench/`.
- ★ LF, no trailing spaces (Markdown excepted), one final newline, spaces for indentation. No lint or format suppressions: `eslint-disable` needs a reason after ` -- `; `prettier-ignore` and `rustfmt::skip` are banned.
- Ask before adding or removing a dependency; it must pass `deny.toml` and `pnpm audit`. Prefer the standard library and what is already in use.
- Test automatically whatever can be, at the lowest layer that shows the behavior. Only what depends on other apps, the global hotkey or the mouse hook is left to manual steps or `/smoke-test`.
- Match the surrounding style. Comments explain why. No dead or commented-out code, no unrelated refactors. Split files over 800 lines.

## Workflow

1. Read `docs/PLAN.md` and the relevant files first. For large work (new dependency, new permission, UX change, more than ~200 lines), present a short plan and ask.
2. Work in small steps; run `node scripts/harness/check.mjs`.
3. **Never send synthetic keystrokes or mouse input to the user's desktop without asking**, including the `input` gate.
4. Update `docs/PLAN.md` (tick items, add decisions, fill the compatibility table) and the matching `docs/specs/` page.
5. Odd frontend errors after quick edits usually mean Vite kept a stale module: restart `dev.ps1`.
6. Git: `master` and `develop` change only through pull requests ([CONTRIBUTING.md](CONTRIBUTING.md)). Branch from `develop` (`feat/…`, `fix/…`, `docs/…`, `chore/…`) and target `develop`. Commit only when the user asks, or on a `feat/…`/`fix/…`/`chore/…` branch while executing a plan the user approved (see Superpowers). Never commit to `develop` or `master`, never push or open a PR unasked. Commit messages: English, imperative, summary under ~72 characters.
7. Releases follow [docs/MAINTAINING.md](docs/MAINTAINING.md) → Releasing (`/release`). Before the `develop` → `master` release PR, bring the README and the website (`site/`) up to date through a PR into `develop`, close the shipped issues with a comment, move them to Done on the Roadmap project and draft the Discussions announcement. The release PR uses `.github/PULL_REQUEST_TEMPLATE/release.md`; CI fails until every box is ticked. Each of these GitHub actions needs the user's yes.

## Superpowers

The Superpowers plugin is enabled for this repository (`.claude/settings.json`). This file wins over any plugin skill. Where they differ:

- **Specs and plans** go to `docs/superpowers/specs/` and `docs/superpowers/plans/` (gitignored scratch, English). They never replace `docs/PLAN.md` and `docs/specs/`: the last plan task is always to update those and tick the item.
- **Approval:** an approved brainstorming spec plus plan satisfies "present a short plan and ask". A new dependency, permission, endpoint or prompt change still needs its own explicit yes.
- **Worktrees** live in `.claude/worktrees/<name>/` (the hooks resolve them; run `pnpm install` there). Not `.worktrees/` or elsewhere.
- **Tests:** TDD at the lowest layer that shows the behavior. Win32, hotkey and mouse-hook behavior stays manual (`/smoke-test`); never fake input on the user's desktop.
- **Per task:** `check.mjs` (fast) before each commit; `--full` before saying the work is done, together with the Definition of done below.
- **Finishing a branch:** offer only "open a PR into `develop`", "keep" or "discard". No local merge into `develop` or `master`.
- **Review:** for the areas named in the Definition of done, request `tonesnap-reviewer` in addition to the plugin's reviewer.

## Definition of done

- `node scripts/harness/check.mjs --full` passes.
- New interface text exists in Vietnamese and English.
- `docs/PLAN.md` and `docs/specs/` describe the change.
- Automated tests cover it; manual steps only for what automation cannot reach.
- Prompt or model changes: the bench cases ran on a real provider and quality did not drop.
- Significant changes to `platform/win`, `capture.rs`, `redact.rs`, `llm/` or capabilities: reviewed by the `tonesnap-reviewer` agent.

## Communication

Talk to the user in the language they write in (usually Vietnamese). Everything written into the repository is in English.
