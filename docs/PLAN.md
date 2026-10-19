# ToneSnap: implementation plan

> Working name. A desktop app (Windows first, macOS later): select English (or Vietnamese) text in any app → a
> popup suggests several English versions in different tones (concise, polite, formal, friendly…) with
> explanations in Vietnamese → Apply replaces the selection in place when the field is editable.
>
> How the built parts behave, rule by rule: [specs/](specs/README.md).

## Decisions

| Topic | Decision |
|---|---|
| Stack | Tauri 2: Rust core (UIA/AX, OCR, clipboard, API calls) + React/TS interface |
| First audience | Individuals and small teams; users bring their own API key |
| Order | Phase 0 → 1 → 2 → 3 (macOS) → 4 (enterprise) → 5 (release) |
| AI (2026-10-18) | Cloud providers only, chosen in Settings with one API key each (Credential Manager): **Gemini** (free tier, default), **Groq** (free tier), **OpenRouter** (free models) and **Claude API** (paid). They share one prompt and one JSON schema; Gemini, Groq and OpenRouter use one OpenAI-compatible client (`llm/openai_compat.rs`). The local Ollama provider was removed (it needed a GPU and a multi-GB model); its benchmark script and the `/prompt-bench` skill were removed too; the reports stay in `bench/` as history. Free tiers may let the provider train on the content: Settings warns about it |
| Claude model | `claude-opus-5-5` at effort `low`; switchable to Sonnet 5.5 / Haiku 4.5 |
| Development process | Harness: `CLAUDE.md`, `scripts/harness/` (quality gates, Rust ⇄ TS contracts, code rules), Claude Code hooks and the `/check` `/feature` `/smoke-test` skills ([details](HARNESS.md)) |
| Repository language | Everything is written in English. Vietnamese appears only in the interface dictionaries (`src/lib/i18n.ts`, `src-tauri/src/i18n.rs`), in Vietnamese test inputs and in recorded model output |
| Code structure (2026-10-07 refactor) | Rust: one module per concern (`session`, `request`, `window`, `trigger`, `tray`, `commands`, `state`); OS calls go through the `platform` facade so macOS (Phase 3) only adds an implementation; provider errors are a typed `llm::Error`. Commands that wait on I/O are async. React: each window is a state hook plus small components; Rust events are subscribed through typed wrappers in `api.ts` |
| Testing (2026-10-08) | Automate everything that can be; test by hand only what depends on other apps, the global hotkey or the mouse hook. Layers: Rust unit and contract tests, provider tests against a local HTTP server, Win32 integration tests on windows the test owns (`--full`), key-press tests on request (`--only=input`), vitest + Testing Library with Tauri's `mockIPC` for the UI. Windows tests are written on Windows, macOS tests on a Mac |
| Code control (2026-10-08) | Tools enforce the code rules: Clippy lints in `Cargo.toml` (a `// SAFETY:` comment on every `unsafe` block, no `dbg!`/`println!`/`todo!`/`panic!`, `#[allow]` with a reason), ESLint (type-aware, React hooks, no unhandled promises), Prettier for TS/CSS/JSON (rustfmt for Rust), LF line endings (`.gitattributes`, `.editorconfig`), and `cargo deny` + `pnpm audit` for vulnerabilities and licenses (permissive plus MPL-2.0). The hooks format each edited file; `check.mjs` runs every tool |
| Settings file | Loaded field by field: a value ToneSnap does not understand resets only that field; a damaged file is moved to `settings.broken.json` (then `-2`, `-3`… so older backups survive); a file that cannot be read right now (locked) is left untouched. Settings opens with a warning until the next save. Saving writes a temporary file and renames it |
| Specifications (2026-10-08) | Implemented behavior is written down in `docs/specs/`, one page per area, each rule with a stable ID (`CAP-07`) and the tests that pin it. This plan keeps decisions and the roadmap; a change in behavior updates its spec in the same change |
| Start with Windows (2026-10-08) | Off until the user ticks it in Settings. Windows keeps the choice (the autostart plugin's `Run` registry entry), so it is not in `settings.json`; Settings reads it on open and changes it on Save. The `ANTHROPIC_API_KEY` environment fallback works only in debug builds, so a developer's key is never billed silently |
| Default trigger (2026-10-18) | `triggerMode` defaults to `icon`: the floating icon shows next to a selection and the hotkey still works. A settings file that already holds a `triggerMode` keeps it; only new installs and fields that fell back to the default change |
| Get Key button (2026-10-19) | The API key card has a *Get Key* button that opens the selected provider's key page in the default browser through `opener:default`. Those pages (`aistudio.google.com`, `console.groq.com`, `console.anthropic.com`) are listed in `BROWSER_LINK_HOSTS` in the harness config, apart from `ALLOWED_HOSTS`: the app sends nothing to them. |
| Interface design (2026-10-08) | One design system in `src/styles/` (tokens on a 4 px grid, accent taken from the app icon, light and dark), hand-drawn checkbox, radio and switch with a keyboard-only focus ring, a card per tone in the popup, and a Settings window with a sidebar and one section at a time. No new dependency: system font, inline SVG icons. Rules in [specs/ui-design.md](specs/ui-design.md) |
| Open source (2026-10-19) | Public on GitHub as `nhutruong-struct/struct-tonesnap` under the MIT License, starting from a fresh history. `develop` (default) and `master` are protected by rulesets: pull requests only, CI `check` green, squash into `develop`, merge commits into `master` for releases; head branches are deleted after the merge. Contributor process in `CONTRIBUTING.md`, maintainer process in `docs/MAINTAINING.md`; questions and ideas go to GitHub Discussions |
| Contributor style enforcement (2026-10-19) | CI `check` is the gate (same name, so the rulesets did not change) and a fork PR runs the same gates. Added to it: ESLint for `scripts/`, the project rules over every text file in the repository (secrets and English-only now cover `.github/`, `site/`, root docs), a `whitespace` rule (the `.editorconfig` rules for files no formatter touches), a `suppress` rule (`eslint-disable` needs a reason; `prettier-ignore` and `rustfmt::skip` banned), a PR-title check (English, imperative, under 72 characters; run from `ci.yml` on `pull_request` with the `edited` trigger) and failure hints with line annotations. For contributors: `pnpm run fix`, optional `.githooks/` (`pnpm run hooks:install`), style configuration under CODEOWNERS. Markdown stays unformatted on purpose (hand-aligned tables); the Win32 integration tests stay local. Details in [HARNESS.md](HARNESS.md#enforcement-layers) |
| Releases (2026-10-20) | Version 1.0.0 is the first public release. Installers (NSIS and MSI) are built only by `.github/workflows/release.yml` from a `v*` tag on `master`, with build provenance attestations and SHA-256 checksums, and the release notes come from `CHANGELOG.md`. Not code-signed yet (issue #25): SmartScreen warns on first run |
| OpenSSF Scorecard (2026-10-19) | Fix what one maintainer can: branches must be up to date before merging, and the redaction is fuzzed with cargo-fuzz (`src-tauri/fuzz/`, `fuzz.yml`, weekly and on PRs that touch it). Branch-Protection (2 approvals, no admin bypass) and Code-Review (approval by someone other than the author) cannot be met alone and are dismissed until a second maintainer joins; SAST and Maintained clear with time. Details in [MAINTAINING.md](MAINTAINING.md#openssf-scorecard) |

## Architecture

```
Trigger (hotkey / floating icon / automatic popup)
  → Selection + context: UIA → (Word COM / browser extension) → on-device screenshot + OCR → clipboard
  → Safety: skip password fields and blocked apps, redact sensitive data (reversibly)
  → AI call: Gemini | Groq | OpenRouter | Claude API (own key) | enterprise gateway (Phase 4)
  → Popup: one version per tone + explanations
  → Apply: refocus the original app → paste through the clipboard → restore the previous clipboard
```

## Context: four layers

1. **UI Automation / AX**: text around the selection, its position, whether the field is editable.
2. **Per-app integrations** (Phase 2): Word COM, a browser extension over native messaging.
3. **On-device screenshot + OCR** (Windows.Media.Ocr / Apple Vision): the image never leaves the machine; only the redacted text is sent.
4. **Image sent to the AI** (Phase 2, off by default): sensitive regions are masked first.

## Data protection

- Nothing is stored by default. User text is never logged.
- Sensitive data is redacted before sending and restored for display: emails, phone numbers, national IDs, card numbers, IPs/URLs, API keys/tokens, and the user's own list of terms.
- Password fields are skipped. Apps can be blocked.
- API keys live in Windows Credential Manager / macOS Keychain.
- A "preview what is sent" mode.
- Selected text is treated strictly as data. The AI only returns JSON in a fixed schema and cannot take any action.
- Enterprise (Phase 4): a self-hosted gateway (SSO, policies, a second redaction pass, quotas, metadata-only logs);
  providers are Claude API (ZDR) / Bedrock / Vertex AI / Foundry / an internal model. Policies are pushed through GPO/Intune/MDM.
  The Fable models require 30-day data retention, so enterprise mode does not offer them.

## Roadmap

| Phase | Scope | Done when |
|---|---|---|
| 0. Technical spike (Windows) | Hotkey → read text (UIA + clipboard) → OCR → popup → stream → Apply. App compatibility table | The app × capability table is filled. First version appears in ≤ 2 s |
| 1. Windows MVP | Tray icon, complete Settings, icon / automatic popup modes (mouse hook), Vietnamese → English, redaction, a 100-sentence evaluation set | Usable every day in ≥ 80% of the listed apps |
| 2. Richer context | Word COM, browser extension, per-app tones, glossary, encrypted history, image input | Replacing text in Word keeps formatting. Gmail / Google Docs work |
| 3. macOS | AX, NSPanel, Vision OCR, ScreenCaptureKit, permission onboarding, notarization | Parity with Windows |
| 4. Enterprise | Gateway (Docker), SSO, policies, GPO/MDM, security documentation | Internal pilot with 5–10 people |
| 5. Release | Code signing, auto-update, MSI/DMG installers, beta | Public installer available |

## Phase 0 progress

- [x] Tauri 2 skeleton + tray icon + single instance
- [x] Global hotkey (default `Ctrl+Alt+E`; `Ctrl+Alt+Space` is taken by Claude desktop's Quick Entry). Settings shows an error when another app holds the hotkey
- [x] Read the selection with UI Automation (context, position, editability), fall back to Ctrl+C and restore the previous clipboard
- [x] On-device OCR (Windows.Media.Ocr) for context when UIA cannot read the app
- [x] Reversible redaction of sensitive data + unit tests
- [x] Claude call: streaming, JSON output schema, `fallbacks: "default"` for refusals
- [x] Popup: one version per tone, explanations, Apply/Copy, keys 1–9 / R / Esc, preview-before-send mode
- [x] Settings: API key (Credential Manager), model, effort, popup trigger, tones, context, privacy
- [x] Bilingual interface, Vietnamese / English (follows Windows by default), including the tray menu
- [x] Use the Windows TLS stack so the app works behind corporate TLS inspection (Netskope, Zscaler…)
- [ ] Fill in the compatibility table below by testing on a real machine
- [x] Add Ollama (local, free) as the default provider, Claude as an option (superseded: removed on 2026-10-18)
- [x] Benchmark 3 small local models and choose qwen3.5:4b (history; no local model any more)
- [x] Replace Ollama with cloud providers: Gemini (default), Groq, OpenRouter, Claude; one API key per provider chosen in Settings
- [ ] Measure quality and latency of the free-tier models (Gemini, Groq, OpenRouter) on the real prompt
- [x] JSON schema pins exactly the requested tones; a reminder line states the output languages (fixes missing versions, untranslated Vietnamese input, explanations in the wrong language)
- [ ] Measure latency of Opus 5.5 / Sonnet 5.5 / Haiku 4.5 (needs an API key)
- [x] Icon mode: selecting text shows the ToneSnap icon next to the pointer (low-level mouse hook + UIA confirmation; the icon is a layered window that never takes focus)
- [x] ToneSnap brand icon for the app, the tray and the floating icon
- [ ] Automatic popup on selection (reuse the icon mode's detection)
- [x] Development harness: quality gates, Rust ⇄ TS contract checks, code rules, hooks, skills; repository converted to English
- [x] Clean-architecture refactor (one module per concern, platform facade, typed errors, async I/O commands) and automated tests: 86 Rust tests (9 Win32 integration, 2 key-press on request), 46 frontend tests
- [x] Fix: a second OCR on a new thread could crash the app after an earlier OCR failed early (the COM apartment was torn down under a cached WinRT factory); the MTA is now kept alive
- [x] Settings survive a damaged settings file (see Decisions)
- [x] "Start ToneSnap when I sign in to Windows" in Settings (see Decisions)
- [x] Interface redesign: design tokens and shared controls, popup cards and key hints, Settings sidebar with row-style cards and switches (55 frontend tests)
- [x] The popup no longer quotes the original selection: the user has just selected it in the other app, and preview mode still shows what will be sent (POP-22)
- [x] Multi-line selections keep their indentation in Chromium apps (Slack): a single UI Automation read moves the spaces that start a line, a paragraph-by-paragraph read keeps them, so ToneSnap reads paragraph by paragraph when the single read lost indentation (CAP-17; no keystroke, no clipboard; confirmed with the hotkey in Slack on 2026-10-08); the prompt keeps indentation ([report](bench/local-models-2026-10-07.md): 11/11 valid, 53/53 Vietnamese explanations)
- [x] The floating icon shows in Google Docs: an empty selection reported by the focused element is believed only when the selection gesture ended inside that element (TRG-21)
- [x] Fixes from the specs review: default tones are the three the local model was benchmarked on; a failed settings write puts the old hotkey back; every way of closing the popup stops its request (Copy and Open Settings used to let it run); the unused `current_session` command is gone

## Compatibility table (filled during Phase 0)

| App | Reads selection | Reads context | Knows if editable | Apply | Notes |
|---|---|---|---|---|---|
| Word | | | | | |
| Outlook | | | | | |
| Slack | ✓ UIA (indentation kept by the paragraph read, CAP-17) | ✓ UIA (surrounding text) | Apply offered (editable or unknown, not told apart) | ✓ | Message composer, 2026-10-08 |
| Chrome (textarea / contenteditable) | | | | | |
| Edge | | | | | |
| Google Docs (Edge) | ✓ via clipboard (UI Automation sees no selection) | | | ✓ | Floating icon shows (TRG-21), 2026-10-08 |
| Edge / Acrobat PDF | | | | | |
| Notepad | | | | | |
| Teams | | | | | |
