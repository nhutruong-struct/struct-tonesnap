# Changelog

All notable changes to ToneSnap are listed here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/). A version is written with the `v` of its tag (`v1.2.0`).

## [Unreleased]

## [v1.2.0] - 2026-10-22

### Install

On Windows, download `ToneSnap_1.2.0_x64-setup.exe` (installer) or `ToneSnap_1.2.0_x64_en-US.msi` (for managed
deployment) below. If you use 1.1.0 or later, you do not need to: ToneSnap installs this version by itself (or asks
first, if you chose **Ask before updating**). If you use 1.0.0, install this version by hand once, because 1.0.0 cannot
update itself. The Windows installers are still **not code-signed**, so your browser and Windows SmartScreen ask
first: keep the file (**… → Keep → Show more → Keep anyway**), then choose **More info → Run anyway** if the file
came from this page.

On a Mac (macOS 14 or later, Apple Silicon), download `ToneSnap_1.2.0_aarch64.dmg`, open it and drag ToneSnap to
Applications. It is signed and notarized. At first start, the **Permissions** section of Settings asks for
**Accessibility** and, optionally, **Screen Recording** (on-device OCR context).

You can check that a file was built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.2.0_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Added

- macOS support (macOS 14 or later, Apple Silicon): reading the selection through the Accessibility API, a menu-bar
  app with a floating-icon mode, copy and paste with ⌘C and ⌘V, on-device OCR with Vision, a Permissions section in
  Settings, and keys in the Keychain. Releases include a signed and notarized `.dmg` and update the app in place, and
  the website has a Mac download button.
- **Auto** trigger mode: selecting text with the mouse opens the popup by itself after a short wait, with no click on
  the icon. It reads the selection only through the accessibility API and shows the icon instead when it cannot.
  Keyboard selections are not seen.

### Changed

- In Word, **Apply** edits the document in place and changes only the words that differ, so bold, italics, links and
  styles inside the selection are kept, and one Ctrl+Z undoes it.

### Fixed

- An answer cut off by the network or the provider is now an error instead of a version you could apply; a provider
  that stays silent for 60 seconds times out.
- A version that lost a masked value of the selection (an email address, for example) says which one and offers Copy
  only.
- The Windows uninstaller shows the ToneSnap icon instead of the generic installer icon.

## [v1.1.4] - 2026-10-21

### Install

Download `ToneSnap_1.1.4_x64-setup.exe` (installer) or `ToneSnap_1.1.4_x64_en-US.msi` (for managed deployment)
below. If you use 1.1.0 or later, you do not need to: ToneSnap installs this version by itself (or asks first, if you
chose **Ask before updating**). If you use 1.0.0, install this version by hand once, because 1.0.0 cannot update
itself. The installers are still **not code-signed**, so your browser and Windows SmartScreen ask first: keep the
file (**… → Keep → Show more → Keep anyway**), then choose **More info → Run anyway** if the file came from this
page. You can check that a file was built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.1.4_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Changed

- No change to the app. Releases are now published automatically when a version is merged into `master`, and their
  notes list the contributors.

## [v1.1.3] - 2026-10-21

### Install

Download `ToneSnap_1.1.3_x64-setup.exe` (installer) or `ToneSnap_1.1.3_x64_en-US.msi` (for managed deployment)
below. If you use 1.1.0 or later, you do not need to: ToneSnap installs this version by itself (or asks first, if you
chose **Ask before updating**). If you use 1.0.0, install this version by hand once, because 1.0.0 cannot update
itself. The installers are still **not code-signed**, so your browser and Windows SmartScreen ask first: keep the
file (**… → Keep → Show more → Keep anyway**), then choose **More info → Run anyway** if the file came from this
page. You can check that a file was built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.1.3_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Changed

- The `.exe` installer, its window and the uninstaller show the ToneSnap icon instead of the generic installer icon.

## [v1.1.2] - 2026-10-21

### Install

Download `ToneSnap_1.1.2_x64-setup.exe` (installer) or `ToneSnap_1.1.2_x64_en-US.msi` (for managed deployment)
below. If you use 1.1.0 or later, you do not need to: ToneSnap installs this version by itself (or asks first, if you
chose **Ask before updating**). If you use 1.0.0, install this version by hand once, because 1.0.0 cannot update
itself. The installers are still **not code-signed**, so your browser and Windows SmartScreen ask first: keep the
file (**… → Keep → Show more → Keep anyway**), then choose **More info → Run anyway** if the file came from this
page. You can check that a file was built by this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.1.2_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Changed

- The website's **Download for Windows** button downloads the installer directly, and its Install step explains
  the browser and SmartScreen prompts.
- The release lists only the installer, the MSI, `SHA256SUMS.txt` and `latest.json`; the `.sig` files are no longer
  attached, because `latest.json` carries the signatures.

## [v1.1.1] - 2026-10-20

### Install

Download `ToneSnap_1.1.1_x64-setup.exe` (installer) or `ToneSnap_1.1.1_x64_en-US.msi` (for managed deployment)
below. If you use 1.1.0, you do not need to: ToneSnap installs this version by itself (or asks first, if you chose
**Ask before updating**). If you use 1.0.0, install this version by hand once, because 1.0.0 cannot update itself.
The installers are still **not code-signed**, so Windows SmartScreen shows "Windows protected your PC" the first
time. Choose **More info → Run anyway** if the file came from this page. You can check that a file was built by
this repository's GitHub Actions from this tag:

```
gh attestation verify ToneSnap_1.1.1_x64-setup.exe --repo nhutruong-struct/struct-tonesnap
```

and compare its SHA-256 with `SHA256SUMS.txt`.

### Changed

- A few interface texts that still described rewriting only now say "translate or rewrite".
- The tray menu's *Update to {version} & Restart* item has an icon, like the other items.

### Fixed

- A settings file edited by hand that lists a tone twice no longer makes the provider reject the request: each
  tone is requested once, in the usual order.

## [v1.1.0] - 2026-10-20

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

## [v1.0.0] - 2026-10-20

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

[Unreleased]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.2.0...develop
[v1.2.0]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.4...v1.2.0
[v1.1.4]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.3...v1.1.4
[v1.1.3]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.2...v1.1.3
[v1.1.2]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.1...v1.1.2
[v1.1.1]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.1.0...v1.1.1
[v1.1.0]: https://github.com/nhutruong-struct/struct-tonesnap/compare/v1.0.0...v1.1.0
[v1.0.0]: https://github.com/nhutruong-struct/struct-tonesnap/releases/tag/v1.0.0
