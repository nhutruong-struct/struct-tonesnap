# Maintaining ToneSnap

The maintainer's side of [CONTRIBUTING.md](../CONTRIBUTING.md): how the repository is set up, how to triage,
review, merge and release.

## Repository setup

| Setting | Value | Why |
|---|---|---|
| Default branch | `develop` | Pull requests target it without anyone having to change the base |
| `develop` ruleset | Pull request required (1 approval, stale approvals dismissed, conversations resolved), `check` CI job green, squash merge only, no force push, no deletion | One commit per change, every change reviewed and tested |
| `master` ruleset | Same, but merge commits only | Releases keep `develop`'s history, so the next release merges cleanly |
| Ruleset bypass | Repository admins, **through pull requests only** | A sole maintainer cannot approve their own PR; the bypass lets them merge it, but never lets anyone push directly |
| Merge buttons | Squash and merge commit; rebase off; auto-merge on; "Update branch" suggested | Squash for `develop`, merge commit for releases |
| Head branches | Deleted automatically after the merge | No stale branches; protected branches are never deleted |
| Actions | `GITHUB_TOKEN` read-only by default; workflows from first-time contributors need approval | A fork's PR cannot run code with write access, nor run before someone looks at it |
| Security | Private vulnerability reporting, Dependabot alerts and security updates, secret scanning with push protection | See [SECURITY.md](../SECURITY.md) |
| Community | Issues (forms only), Discussions, no wiki | Questions and ideas go to Discussions; issues stay actionable |

Change these in **Settings → Rules → Rulesets** and **Settings → General**; update this table in the same PR.

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
4. Significant changes to `platform/win`, `capture.rs`, `redact.rs`, `llm/` or the capabilities: also run the
   `tonesnap-reviewer` agent or review against the same list by hand.
5. Prompt changes: try `scripts/bench-cases.json` on a real provider (`/smoke-test`) and compare.
6. Request changes with concrete suggestions; approve when it is ready.

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
   `src-tauri/tauri.conf.json`, update `docs/PLAN.md`, and merge it into `develop` as usual.
2. Open a pull request from `develop` into `master` titled `Release vX.Y.Z`, wait for CI, and merge it with
   **Create a merge commit**.
3. Tag the merge commit on `master` and publish the release:

   ```bash
   git fetch origin
   ```

   ```bash
   git tag -a vX.Y.Z origin/master -m "ToneSnap vX.Y.Z"
   ```

   ```bash
   git push origin vX.Y.Z
   ```

   ```bash
   gh release create vX.Y.Z --generate-notes --verify-tag
   ```

4. Build the installer (`pnpm tauri build`) and attach it to the release, then announce it in
   Discussions → Announcements.

## Security reports

Reports arrive as draft advisories under **Security → Advisories**. Acknowledge within 7 days, discuss the fix
in the advisory (it has a private fork for that), publish the advisory with the release that fixes it, and
credit the reporter.

## Moderation

Apply the [Code of Conduct](../CODE_OF_CONDUCT.md). Hide off-topic or abusive comments, lock heated threads, and
use **Block user** for repeated abuse. Pin important discussions and keep the Announcements category for
releases and project news.
