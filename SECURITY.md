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
