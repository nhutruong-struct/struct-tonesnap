# Maintaining ToneSnap

The maintainer's side of [CONTRIBUTING.md](../CONTRIBUTING.md): how the repository is set up, how to triage,
review, merge and release.

## Repository setup

| Setting | Value | Why |
|---|---|---|
| Default branch | `develop` | Pull requests target it without anyone having to change the base |
| `develop` ruleset | Pull request required (1 code-owner approval, stale approvals dismissed, approval of the last push, conversations resolved); `check` and `dependency-review` green on a branch that is up to date with `develop`; no new high or critical CodeQL alert; squash merge only; no force push, no deletion | One commit per change, every change reviewed, tested and scanned |
| `master` ruleset | Same, but merge commits only | Releases keep `develop`'s history, so the next release merges cleanly |
| Tag ruleset | Only admins create, move or delete `v*` tags | A release cannot be tagged by anyone else |
| Ruleset bypass | Repository admins, **through pull requests only** | A sole maintainer cannot approve their own PR; the bypass lets them merge it, but never lets anyone push directly |
| Merge buttons | Squash and merge commit; rebase off; auto-merge on; "Update branch" suggested | Squash for `develop`, merge commit for releases |
| Head branches | Deleted automatically after the merge | No stale branches; protected branches are never deleted |
| Actions | `GITHUB_TOKEN` read-only by default; every action pinned to a commit SHA; workflows from all outside contributors need approval | A fork's PR cannot run code with write access, nor run before someone looks at it; a hijacked action tag cannot change what runs |
| Security | Private vulnerability reporting, Dependabot alerts and security updates, secret scanning with push protection, CodeQL, dependency review, OpenSSF Scorecard, weekly fuzzing of the redaction | See [SECURITY.md](../SECURITY.md) and [OpenSSF Scorecard](#openssf-scorecard) |
| Community | Issues (forms only), Discussions, no wiki | Questions and ideas go to Discussions; issues stay actionable |

Change these in **Settings → Rules → Rulesets** and **Settings → General**; update this table in the same PR.

### OpenSSF Scorecard

Scorecard runs weekly and on every push to `develop`; each check scoring below 10 is an alert under
**Security → Code scanning**. Where the open checks stand:

| Check | Status | Why |
|---|---|---|
| Branch-Protection | Dismissed (won't fix) | The rulesets cover everything else; the rest needs 2 approvals and no admin bypass, which one maintainer cannot meet. Scorecard reads the rulesets with the default token, so no extra token is needed |
| Code-Review | Dismissed (won't fix) | Counts changes approved by someone other than the author; the maintainer cannot approve their own PRs |
| SAST | Clears with time | The first commit predates CodeQL; it leaves Scorecard's window of the last 30 commits after about 25 more merges |
| Maintained | Clears with time | Scores 0 for repositories younger than 90 days (until 2027-01-16), then needs activity most weeks |
| Fuzzing | Fixed | `src-tauri/fuzz/` with `.github/workflows/fuzz.yml` |
| CII-Best-Practices | Dismissed after registering | The [OpenSSF Best Practices](https://www.bestpractices.dev) passing badge scores 5; only gold scores 10, and gold needs at least two maintainers |

Revisit the dismissed alerts when a second maintainer joins: a second approver allows 2 required approvals,
no admin bypass, and reviewed PRs.

To fuzz locally (needs the nightly toolchain and `cargo install cargo-fuzz`), from `src-tauri`:

```bash
cargo +nightly fuzz run redact -- -max_total_time=60
```

## Triage

New issues arrive labelled `needs triage`. For each one:

1. Is it a question or an open-ended idea? Convert it to a discussion (**Convert to discussion** in the issue
   sidebar).
2. Is it a duplicate? Link the original, label `duplicate` and close.
3. Can it be reproduced or acted on? If information is missing, ask for it and wait; close it after ~30 days of
   silence with a polite note.
4. Otherwise replace `needs triage` with the type (`bug`, `enhancement`, `documentation`, `app compatibility`)
   and an `area:` label. Add `good first issue` (small, well described, no Win32 knowledge needed) or
   `help wanted` when outside help is welcome.
5. Check that the reporter did not post an API key or private text. If they did, edit it out, and ask them to
   revoke the key.

App compatibility reports also go into the compatibility table of [PLAN.md](PLAN.md).

## Reviewing a pull request

1. **Fork PRs from first-time contributors:** read the diff, especially `.github/`, `scripts/` and
   `package.json`, before clicking **Approve and run** for the workflows.
2. Check the change against the project rules in [CLAUDE.md](../CLAUDE.md), privacy first: no user text in
   logs, no new network host, no secrets, Rust ⇄ TypeScript contracts on both sides, strings in both languages.
3. Check that it has tests, and that the docs (`docs/specs/`, `docs/PLAN.md`) follow the behavior.
4. **Security review**: approve only code you understand. Look hard at anything that runs at build, install
   or CI time and could ship to every user:
   - `.github/workflows/` (new actions must be pinned to a SHA; no `pull_request_target`; no untrusted
     `${{ github.event.* }}` inside `run:`), `scripts/`, `.claude/`, `dev.ps1`, `build.rs`;
   - `package.json` scripts and lifecycle hooks, new dependencies, lockfile entries that point to a registry
     or Git URL other than npm / crates.io, `tauri.conf.json` and `capabilities/` (new permissions, CSP);
   - obfuscated or minified code, encoded strings, binary files, downloads at runtime, new network hosts,
     process spawning, registry or file writes outside the app's folders.
   When in doubt, ask for the change to be split or explained. CodeQL and dependency review help, but they do
   not replace reading the diff.
5. Significant changes to `platform/win`, `capture.rs`, `redact.rs`, `llm/` or the capabilities: also run the
   `tonesnap-reviewer` agent or review against the same list by hand.
6. Prompt changes: try `scripts/bench-cases.json` on a real provider (`/smoke-test`) and compare.
7. Request changes with concrete suggestions; approve when it is ready.

## Merging

- Into `develop`: **Squash and merge**. Edit the title into a good imperative summary (it becomes the commit
  message) and trim the body to what is useful. The branch is deleted automatically.
- Your own PRs: CI must still be green. Merging uses the admin bypass for the missing approval; never use it
  to merge a red build.
- Dependabot PRs: merge when CI is green and the changelog has nothing alarming. The `windows` crate is pinned
  to Tauri's version and is ignored by Dependabot; update it together with Tauri.

## Releasing

There are no hotfix branches yet: fixes land in `develop` and ship with the next release.

1. On a `chore/release-x.y.z` branch, bump the version in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json` (let cargo update `Cargo.lock`), move the `Unreleased` entries of
   `CHANGELOG.md` into a section for the new version, update `docs/PLAN.md`, and merge it into `develop` as usual.
2. Open a pull request from `develop` into `master` titled `Release vX.Y.Z`, wait for CI, and merge it with
   **Create a merge commit**.
3. Tag the merge commit on `master`. The tag starts `.github/workflows/release.yml`, which checks that the tag
   matches the app version and is on `master`, builds the installers, attests their provenance and publishes
   the release with the `CHANGELOG.md` section as notes:

   ```bash
   git fetch origin
   ```

   ```bash
   git tag -a vX.Y.Z origin/master -m "ToneSnap vX.Y.Z"
   ```

   ```bash
   git push origin vX.Y.Z
   ```

4. Watch the **Release** workflow. Besides the installers it publishes their `.sig` files and `latest.json`,
   which every installed ToneSnap reads to update itself ([specs/updates.md](specs/updates.md)): once the
   release is published, the installed apps start installing it within hours. Never attach an installer built on a personal machine; if the workflow
   fails, fix it through a PR and push the tag again only if no release was published (releases are immutable).
   Merging `master` back into `develop` is not needed: the release PR's merge commit is the only commit `master`
   has that `develop` lacks, and it changes no files. Announce the release in Discussions → Announcements.

### Update signing key

The installers are signed for the in-app updater with a minisign key made by `pnpm tauri signer generate`.
The public key is `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`; the private key and its password
are the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets of the `release`
environment, whose deployment rule allows only `v*` tags. Keep an offline copy of the private key: without
it, no installed ToneSnap accepts another update, and a new key reaches users only through a manual install.
Never commit it, and never sign on a personal machine.

## Security reports

Reports arrive as draft advisories under **Security → Advisories**. Acknowledge within 7 days, discuss the fix
in the advisory (it has a private fork for that), publish the advisory with the release that fixes it, and
credit the reporter.

## Moderation

Apply the [Code of Conduct](../CODE_OF_CONDUCT.md). Hide off-topic or abusive comments, lock heated threads, and
use **Block user** for repeated abuse. Pin important discussions and keep the Announcements category for
releases and project news.
