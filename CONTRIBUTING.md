# Contributing to ToneSnap

Thank you for helping! This page explains how to propose a change and what happens after you do. Maintainers
follow [docs/MAINTAINING.md](docs/MAINTAINING.md) for their side of the process.

Issues and discussions may be written in **English or Vietnamese**. Everything that goes into the repository
(code, comments, docs, commit messages, pull request titles) is written in **English**.

By taking part you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

| You want to… | Do this |
|---|---|
| Ask a question or get help | Open a thread in [Discussions → Q&A](https://github.com/nhutruong-struct/struct-tonesnap/discussions/categories/q-a) |
| Suggest an idea | Start in [Discussions → Ideas](https://github.com/nhutruong-struct/struct-tonesnap/discussions/categories/ideas); it becomes an issue once it is agreed on |
| Report a bug | Open a [bug report](https://github.com/nhutruong-struct/struct-tonesnap/issues/new?template=bug_report.yml) |
| Tell us how ToneSnap works in an app you use | Open an [app compatibility report](https://github.com/nhutruong-struct/struct-tonesnap/issues/new?template=app_compatibility.yml) |
| Report a security problem | Follow [SECURITY.md](SECURITY.md). **Never** in a public issue |
| Fix a bug or build a feature | Read the rest of this page |

Good places to start: issues labelled
[`good first issue`](https://github.com/nhutruong-struct/struct-tonesnap/labels/good%20first%20issue) and
[`help wanted`](https://github.com/nhutruong-struct/struct-tonesnap/labels/help%20wanted).

## Before you write code

1. **Find or open an issue.** Small fixes (typos, an obvious bug with a one-line fix) can go straight to a pull
   request. For anything larger, comment on the issue first so we can agree on the approach and avoid duplicate
   work. A maintainer assigns the issue to you once you say you are working on it.
2. **Ask first** when the change:
   - adds or removes a dependency (Rust crate or npm package),
   - talks to a new network host (a new place user text could go is a privacy decision),
   - changes the prompt (`src-tauri/prompts/`), the UX, or is bigger than ~200 lines.
3. Read the relevant page in [docs/specs/](docs/specs/README.md) and the decisions in
   [docs/PLAN.md](docs/PLAN.md) before changing behavior.

## Branch model

| Branch | Purpose | How it changes |
|---|---|---|
| `develop` | Integration branch, the default branch. All pull requests target it | Pull requests only (squash merge) |
| `master` | The latest release | Release pull requests from `develop` only, by a maintainer |
| `feat/…`, `fix/…`, `docs/…`, `chore/…`, `refactor/…`, `test/…` | Your work | Deleted automatically after the merge |

Both `master` and `develop` are protected: nobody, maintainers included, pushes to them directly. Every change
goes through a pull request that passes CI and gets a review.

## Workflow

1. **Fork** the repository and clone your fork.

   ```bash
   git clone https://github.com/<your-name>/struct-tonesnap.git
   ```

   ```bash
   git remote add upstream https://github.com/nhutruong-struct/struct-tonesnap.git
   ```

2. **Create a branch** from the latest `develop`.

   ```bash
   git fetch upstream
   ```

   ```bash
   git switch -c fix/short-description upstream/develop
   ```

3. **Set up** the toolchain (see [README → Requirements](README.md#requirements)), then install and run:

   ```bash
   pnpm install
   ```

   ```bash
   pwsh -File dev.ps1
   ```

4. **Make the change** in small, focused commits. Add or update tests (see [Tests](#tests)) and update the docs
   when behavior changes: the matching page in `docs/specs/`, and `docs/PLAN.md` when you finish a roadmap item.

5. **Run the checks** until they pass:

   ```bash
   node scripts/harness/check.mjs --full
   ```

   `node scripts/harness/check.mjs --fix` formats the code (rustfmt, ESLint, Prettier) first.

6. **Push** your branch to your fork and open a **pull request against `develop`**. Fill in the template,
   link the issue (`Closes #123`) and keep the PR as a draft until it is ready for review.

7. **Review.** CI must be green, and a maintainer reviews the change. Answer comments by pushing new commits to
   the same branch (no need to force-push; the PR is squashed on merge). Resolve each conversation once it is
   addressed.

8. **Merge.** A maintainer squash-merges the PR into `develop`; the PR title becomes the commit message, and
   the branch is deleted. Your change ships with the next release from `develop` to `master`.

Keep your branch up to date with `develop` by merging or rebasing on `upstream/develop`, or with the
**Update branch** button on the pull request.

## Commit messages and PR titles

- English, imperative mood, a summary line under ~72 characters: `Restore the clipboard after OCR fallback`,
  not `restored clipboard` or `fix bug`.
- The PR title is used as the squashed commit message, so make it describe the whole change.
- Explain *why* in the body when it is not obvious.

## Project rules

The full list, with the reasons behind it, is in [CLAUDE.md](CLAUDE.md) (it guides human and AI contributors
alike). The ones that matter most:

- **Privacy.** Never log, store or send user text anywhere except the AI request, and only through the redacted
  payload (`redact.rs`). No telemetry. Screenshots are only OCR'd on the device.
- **Secrets.** API keys live only in the OS credential store (`secrets.rs`). Never commit a real key, not even
  in a test. If you commit one by mistake, revoke it at the provider immediately: deleting the commit is not
  enough.
- **Network.** Only the provider hosts listed in `scripts/harness/config.mjs` and localhost (tests).
- **Rust ⇄ TypeScript contracts.** Settings, tones, commands, events and capabilities must match on both sides
  in the same change.
- **Interface text** lives in `src/lib/i18n.ts` (or `src-tauri/src/i18n.rs`) in both Vietnamese and English.
- **Rust:** no `unwrap()` (except after `.lock()`), no `panic!`/`todo!`/`println!`/`dbg!`; `unsafe` only in
  `src-tauri/src/platform/`, each block with a `// SAFETY:` comment. Blocking work stays off Tauri's main thread.
- **TypeScript:** no `any`, `@ts-ignore` or `console`; `invoke` and `listen` only in `src/lib/api.ts`.
- **Hands off** generated files and lockfiles unless the change is a dependency update.

`node scripts/harness/check.mjs` enforces most of these automatically.

## Tests

Test automatically whatever can be, at the lowest layer that shows the behavior:

- Pure Rust logic: unit tests in the same file (`#[cfg(test)] mod tests`).
- Data Rust sends to the UI: a test of its JSON shape.
- Provider code: against `llm/test_server.rs`, never a real provider.
- Win32 code: against windows the test owns (`platform/win/test_window.rs`), in `mod integration_tests`, run by
  `check.mjs --full`.
- UI behavior: a vitest test next to the component, with the backend faked by `src/test/backend.ts`.
- Redaction (`redact.rs`): also fuzzed by `src-tauri/fuzz/` in CI; a change to how placeholders are made or
  restored updates the fuzz target too.

Only behavior that depends on other apps, the global hotkey or the mouse hook is tested by hand; describe the
manual steps you followed in the pull request.

> The key-press tests (`check.mjs --only=input`) send real `Ctrl+C`/`Ctrl+V` and take focus. Run them only
> while you are not using the PC.

## Using an AI coding assistant

The repository ships a [Claude Code](https://claude.com/claude-code) setup (`CLAUDE.md`, `.claude/`,
`scripts/harness/`) with hooks that run the checks and skills such as `/check` and `/feature`. You are welcome
to use it or any other assistant, but you are responsible for every line you submit: read and understand it,
and make sure it follows the rules above.

## License

By contributing, you agree that your contributions are licensed under the project's [MIT License](LICENSE).
