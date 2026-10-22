# Maintaining ToneSnap

The maintainer's side of [CONTRIBUTING.md](../CONTRIBUTING.md): how the repository is set up, how to triage,
review, merge and release.

## Repository setup

| Setting | Value | Why |
|---|---|---|
| Default branch | `develop` | Pull requests target it without anyone having to change the base |
| `develop` ruleset | Pull request required (1 code-owner approval, stale approvals dismissed, approval of the last push, conversations resolved); `check` and `dependency-review` green on a branch that is up to date with `develop`; no new high or critical CodeQL alert; squash merge only; no force push, no deletion | One commit per change, every change reviewed, tested and scanned |
| `master` ruleset | Same, but merge commits only | Releases keep `develop`'s history, so the next release merges cleanly |
| Tag ruleset | Only admins and the GitHub Actions app (bypass actor) create `v*` tags; nobody moves or deletes them | The release workflow tags the commit it publishes; a release cannot be tagged by anyone else |
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
2. Check the change against the project rules in [AGENTS.md](../AGENTS.md) and `.claude/rules/`, privacy first: no user text in
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

There are no hotfix branches yet: fixes land in `develop` and ship with the next release. The `/release` skill
(`.claude/skills/release/`) walks an agent through these steps with `gh`, asking before each action on GitHub.

1. On a `chore/release-x.y.z` branch, bump the version in `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json` (let cargo update `Cargo.lock`), move the `Unreleased` entries of
   `CHANGELOG.md` into a `## [vX.Y.Z] - date` section, update `docs/PLAN.md`, and merge it into `develop` as usual.
   A version is always written as its tag, with the `v`: the changelog headings and links, the PR title, the
   release title (`ToneSnap vX.Y.Z`) and the announcement. The `harness` gate fails on a changelog version
   without it.
2. Bring the README and the website up to date with the release. Read `README.md` and `site/index.html`
   against the new `CHANGELOG.md` section: features, supported platforms, tones, languages, the mock popup,
   default settings, install, update and uninstall steps, file names, and the intro video in `site/media/`
   (re-render it when the popup or the flow changed, see PLAN.md). Fix what no longer matches in a `docs/…` pull
   request into `develop`. The site deploys from `develop` (`.github/workflows/pages.yml`), so it goes live as
   soon as that PR is merged. Merge it before the release PR.
3. Settle what the release changes on GitHub:
   - **Issues:** every issue the release resolves (the `Closes #n` of its PRs, and open issues the changelog
     answers) gets a comment naming the release and is closed; partly done ones say what is left and stay open.
     The issue forms still offer the right platforms and versions.
   - **Project:** on the **ToneSnap Roadmap** project, the shipped issues are **Done**.
   - **Announcement:** draft the post for Discussions → Announcements ("ToneSnap vX.Y.Z is released").
4. Open a pull request from `develop` into `master` titled `Release vX.Y.Z` with the release template
   (`gh pr create --base master --head develop --template release.md`, or `?template=release.md` on the compare
   page). Fill in the version PR, the site PR (or why the site needed no change), the shipped issues and the
   announcement draft, and tick each step as it is done. CI's `check` fails on a PR into `master` while the title
   does not match the version in `tauri.conf.json`, a box is unticked or a placeholder is left (the `release-pr`
   gate, `scripts/harness/release-pr.mjs`). When it is green, merge it with **Create a merge commit**.
5. The merge starts `.github/workflows/release.yml` on `master`. There is no tag to push: the workflow reads
   the version from `src-tauri/tauri.conf.json`, checks that `CHANGELOG.md` has a `## [vX.Y.Z]` section, builds the
   installers, attests their provenance and publishes the release `vX.Y.Z`, titled `ToneSnap vX.Y.Z`, with that section as notes, which
   creates the tag on the merge commit. The notes end with a **Contributors** list: the GitHub accounts that
   authored a commit since the previous release, bots excluded. A merge into `master` whose version already has a release (the version
   was not bumped) builds nothing and says so in the run's summary.
6. Watch the **Release** workflow. Besides the installers it publishes `SHA256SUMS.txt` and `latest.json`,
   which every installed ToneSnap reads to update itself ([specs/updates.md](specs/updates.md)): once the
   release is published, the installed apps start installing it within hours. Never attach an installer built
   on a personal machine. If the workflow fails before publishing, fix it through a PR into `develop` and
   re-run the workflow on `master` from the Actions tab (a draft left by the failed run is replaced); if a
   release was already published, it is immutable, so the fix ships as the next version.
   Merging `master` back into `develop` is not needed: the release PR's merge commit is the only commit `master`
   has that `develop` lacks, and it changes no files.
7. Once the release is published, post the drafted announcement in Discussions → Announcements.

### Update signing key

The installers are signed for the in-app updater with a minisign key made by `pnpm tauri signer generate`.
The public key is `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`; the private key and its password
are the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets of the `release`
environment, whose deployment rule allows only the `master` branch. Keep an offline copy of the private key: without
it, no installed ToneSnap accepts another update, and a new key reaches users only through a manual install.
Never commit it, and never sign on a personal machine.

### Release app (tags)

The ruleset **Protect release tags** lets only repository admins and the **ToneSnap Release** GitHub App create
`v*` tags; the workflow's own `GITHUB_TOKEN` cannot, so publishing a release with it fails with "Cannot create
ref due to creations being restricted". The `publish` job therefore gets a short-lived token of the app
(`actions/create-github-app-token`) and uses it only for the command that publishes the draft, which creates the
tag. The app is installed on this repository alone, with the **Contents: Read and write** permission and no
webhook. Its credentials are the `RELEASE_APP_CLIENT_ID` variable and the `RELEASE_APP_PRIVATE_KEY` secret of the
`release` environment. If it is ever lost or the key leaks, delete the key in the app's settings, generate a new
one and replace the secret. To release without the app, create the `vX.Y.Z` tag on the merge commit as an
admin and re-run the failed jobs: a release on an existing tag needs no ref creation.

## macOS

The app runs on macOS 14 or later, Apple Silicon (`./dev.sh` in development; the `check (macOS)` job of `ci.yml` lints and tests the macOS layer on every pull request). A release carries `ToneSnap_<version>_aarch64.dmg` and the signed update archive `ToneSnap_<version>_aarch64.app.tar.gz` next to the Windows installers, and `latest.json` lists `darwin-aarch64` for the in-app updater.

How a release is made (`release.yml`): `build macOS app` builds `ToneSnap.app` without any secret (it also runs on pull requests, as a check); `sign and notarize macOS app`, in the `release` environment, runs only `scripts/release/sign-macos.sh`, which signs with the Developer ID certificate and the hardened runtime, notarizes the app and the disk image with Apple, staples both and writes the `.dmg` and the `.app.tar.gz`; the `sign for the updater` job signs the archive with the Tauri key like the installers.

### Apple credentials (one time)

1. **Certificate.** In the Apple Developer account (Account Holder role) create a *Developer ID Application* certificate from a certificate signing request, install it in Keychain Access, and export it with its private key as a `.p12` (a password of your own).
2. **Notarization key.** In App Store Connect → Users and Access → Integrations → *Team Keys*, create a key with the *Developer* role and download its `.p8` once.
3. **Secrets** in the `release` environment of the repository (`gh secret set NAME --env release`):

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | `base64 -i certificate.p12`, the whole output |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12` password |
| `APPLE_SIGNING_IDENTITY` | the certificate's name, e.g. `Developer ID Application: Name (TEAMID)` |
| `APPLE_API_KEY_ID` | the key's id |
| `APPLE_API_ISSUER_ID` | the issuer id shown above the keys list |
| `APPLE_API_KEY_P8` | the contents of the `.p8` file |

A release stops with a message naming the secret when one is missing. The Tauri update key (`TAURI_SIGNING_PRIVATE_KEY`) is the one already used for Windows.

### Trying it on your own Mac

With the certificate in your login keychain and a notarization profile (`xcrun notarytool store-credentials tonesnap-notary --apple-id … --team-id …`):

```bash
pnpm tauri build --bundles app
```

```bash
APPLE_SIGNING_IDENTITY="Developer ID Application: Name (TEAMID)" NOTARY_PROFILE=tonesnap-notary scripts/release/sign-macos.sh src-tauri/target/release/bundle/macos/ToneSnap.app 1.2.3 out
```

Add `SKIP_NOTARIZE=1` to sign and package without Apple's service (for example with an *Apple Development* certificate). The script ends with `spctl` checks that the app and the disk image are accepted by Gatekeeper.

Permissions on a Mac belong to the app's code signature: a build signed with the same certificate keeps its Accessibility and Screen Recording permissions across updates, while an unsigned or re-signed build has to be allowed again under Privacy & Security.

## Security reports

Reports arrive as draft advisories under **Security → Advisories**. Acknowledge within 7 days, discuss the fix
in the advisory (it has a private fork for that), publish the advisory with the release that fixes it, and
credit the reporter.

## Moderation

Apply the [Code of Conduct](../CODE_OF_CONDUCT.md). Hide off-topic or abusive comments, lock heated threads, and
use **Block user** for repeated abuse. Pin important discussions and keep the Announcements category for
releases and project news.
