# Security policy

ToneSnap reads text from other apps and sends it to an AI provider, so privacy and security bugs are taken
seriously.

## Supported versions

ToneSnap is in early development. Security fixes go to the `develop` branch and ship with the next release;
only the latest release on `master` is supported.

## Reporting a vulnerability

**Please do not report security problems in public issues, discussions or pull requests.**

Report them privately through GitHub:
[**Report a vulnerability**](https://github.com/nhutruong-struct/struct-tonesnap/security/advisories/new)
(Security tab → *Report a vulnerability*).

Please include:

- what an attacker could do, and under which conditions,
- the steps to reproduce it (a proof of concept if you have one),
- the version or commit you tested, and your Windows version.

Do not include real API keys or other people's data in the report.

## What to expect

- An acknowledgement within **7 days**.
- An assessment and, if confirmed, a plan for a fix within **30 days**. Complex problems may take longer; you
  will be kept informed.
- Credit in the security advisory once the fix is released, unless you prefer to stay anonymous.

## How the code and releases are protected

ToneSnap runs on users' machines with access to what they type, so a malicious change must not be able to
reach a release unnoticed:

- **No direct pushes.** `develop` and `master` change only through pull requests that a code owner approves;
  a new push after the approval needs a new approval. Force pushes and branch deletion are blocked.
- **Automated checks on every pull request:** the build and tests, [CodeQL](https://codeql.github.com/)
  static analysis of the Rust, TypeScript and workflow code, dependency review (known vulnerabilities and
  licenses of any new dependency), `cargo deny` (crates.io only, no known advisories) and `pnpm audit`.
  A pull request that adds a high or critical code-scanning alert cannot be merged.
- **Secret scanning with push protection** blocks commits that contain API keys or tokens.
- **Pinned CI.** Every GitHub Action is pinned to a full commit SHA, workflows get a read-only token by
  default, and workflows from outside contributors run only after a maintainer approves them.
- **Dependabot** proposes dependency and security updates, which go through the same review.
- **Release tags** (`v*`) can be created only by maintainers and by the release workflow.
- **[OpenSSF Scorecard](https://scorecard.dev/viewer/?uri=github.com/nhutruong-struct/struct-tonesnap)**
  checks these practices weekly.

### Verifying a download

Only download ToneSnap from the
[Releases page](https://github.com/nhutruong-struct/struct-tonesnap/releases) of this repository. Installers
will be built by GitHub Actions from a commit on `master`, never on a personal machine, and published
with a [build provenance attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations).
Once releases exist, you can check that an installer was built from this repository:

```bash
gh attestation verify ToneSnap_x.y.z_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

## Scope

Examples of problems we want to hear about:

- user text leaving the device other than in the redacted request to the chosen provider, or sensitive data
  that the redaction misses in a way that matters,
- text read from password fields or blocked apps,
- API keys written anywhere other than the OS credential store,
- the clipboard not being restored, or text pasted into the wrong window,
- code execution or privilege problems in the app, its installer or its Tauri capabilities.

Out of scope: the AI providers' own services and their data policies, and attacks that need someone who
already controls your Windows account.
