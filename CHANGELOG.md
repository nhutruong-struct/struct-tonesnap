# Changelog

All notable changes to ToneSnap are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- The tray menu's *Update to {version} & Restart* item has an icon, like the other items.

## [1.1.0] - 2026-10-20

### Install

Download `ToneSnap_1.1.0_x64-setup.exe` (installer) or `ToneSnap_1.1.0_x64_en-US.msi` (for managed deployment)
below. If you use 1.0.0, install this version by hand once: 1.0.0 cannot update itself, and later versions arrive
by themselves. The installers are still **not code-signed**, so Windows SmartScreen shows "Windows protected your
PC" the first time. Choose **More info → Run anyway** if the file came from this page. You can check that a file
was built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.1.0_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Added

- Choose the language of the versions (Vietnamese by default, English or Japanese) and a mode, **Translate**
  (stays close to the original, the default) or **Rewrite** (rephrases freely), in the popup. Settings holds the
  defaults. Every version is written in the chosen language, whatever language the selection is in.
- Automatic updates from the GitHub releases. By default ToneSnap downloads a new version in the background
  and installs it, then restarts, only while you are not using it (no popup or Settings window open). Choose
  **Ask before updating** in Settings → Updates to be told about new versions (in the tray and in Settings)
  and install them with **Update & Restart**. Updates are signed and verified before they are installed; the
  check sends only the app version and platform.
- Settings → AI suggests models for Gemini, Groq and OpenRouter in a list (any other model name still works),
  and a **Get Key** button opens the provider's page for creating an API key.
- Uninstalling with the `.exe` installer and **Delete the application data** ticked also deletes your API keys
  from Windows Credential Manager. The README explains how to uninstall and what stays behind.

### Changed

- New installs show the floating icon next to a selection by default; `Ctrl+Alt+E` still works, and Settings can
  switch to the hotkey only. Existing settings keep their mode.
- The tray menu items have icons.
- Tones are now **Concise**, **Natural** and **Formal**.

### Removed

- The explanations (a note per version and the list of mistakes in the original) and their setting: they made
  every answer longer and slower.

### Fixed

- Apply is no longer offered for selected text on web pages that cannot be edited (Chrome, Edge): the popup shows
  Copy only.

## [1.0.0] - 2026-10-20

The first public release of ToneSnap for Windows 10 and 11.

### Install

Download `ToneSnap_1.0.0_x64-setup.exe` (installer) or `ToneSnap_1.0.0_x64_en-US.msi` (for managed deployment)
below. The installers are **not code-signed yet**, so Windows SmartScreen shows "Windows protected your PC" the
first time. Choose **More info → Run anyway** if the file came from this page. You can check that a file was
built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.0.0_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Features

- Select text in any app, press `Ctrl+Alt+E` or click the floating icon next to the selection, and get
  English versions in several tones (minimal, concise, natural, polite, formal, friendly) with a short
  explanation in Vietnamese or English.
- **Apply** replaces the selection in the original app and restores your clipboard; **Copy** copies a version.
  Keys `1`–`9`, `R` and `Esc` work in the popup.
- Reads the selection with UI Automation, falls back to the clipboard, and uses on-device OCR (Windows.Media.Ocr)
  for context when an app exposes no text. Indentation of multi-line selections is kept in Chromium apps such as
  Slack; the floating icon also works in Google Docs.
- AI providers with your own API key: Google Gemini (free tier, default), Groq (free tier), OpenRouter (free
  models) and the Claude API (paid). Keys are stored in Windows Credential Manager.
- Privacy: emails, phone numbers, IDs, card numbers, URLs, API keys and your own terms are redacted before
  sending and restored on screen; password fields and blocked apps (password managers by default) are never
  read; preview mode shows what will be sent; no telemetry, nothing logged or stored.
- Settings window with a Vietnamese and English interface, light and dark themes, a tray menu, a configurable
  hotkey with conflict detection, and "Start ToneSnap when I sign in to Windows".
- Works behind corporate TLS inspection: requests use the Windows TLS stack.

### Known limitations

- Windows only; macOS is planned.
- The installers are not code-signed (see above), and there is no auto-update yet: watch the repository's
  releases to hear about new versions.
- Several apps in the [compatibility table](https://github.com/nhutruong-struct/struct-tonesnap/blob/develop/docs/PLAN.md#compatibility-table-filled-during-phase-0)
  are not verified yet (Word, Outlook, Teams, PDF readers). Reports are welcome:
  [app compatibility report](https://github.com/nhutruong-struct/struct-tonesnap/issues/new?template=app_compatibility.yml).
- Free tiers have daily limits, and the provider may use what you send to improve its products.

[Unreleased]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.0...develop
[1.1.0]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/nhutruong-struct/struct-tonesnap/releases/tag/v1.0.0
