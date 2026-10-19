<div align="center">

<img src="src-tauri/icons/128x128.png" alt="ToneSnap icon" width="96" height="96">

# ToneSnap

**Select text in any app, press a hotkey, and get it back in the tone you need.**

[Website](https://nhutruong-struct.github.io/struct-tonesnap/) · [Discussions](https://github.com/nhutruong-struct/struct-tonesnap/discussions) · [Roadmap](docs/PLAN.md) · [Contributing](CONTRIBUTING.md)

[![CI](https://github.com/nhutruong-struct/struct-tonesnap/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/nhutruong-struct/struct-tonesnap/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform: Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-0078d4.svg)](#status)
[![Discussions](https://img.shields.io/github/discussions/nhutruong-struct/struct-tonesnap)](https://github.com/nhutruong-struct/struct-tonesnap/discussions)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/nhutruong-struct/struct-tonesnap/badge)](https://scorecard.dev/viewer/?uri=github.com/nhutruong-struct/struct-tonesnap)
[![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Sponsor](https://img.shields.io/badge/sponsor-%E2%9D%A4-db61a2.svg)](https://github.com/sponsors/nhutruong-struct)

</div>

ToneSnap is a small desktop app for people who write English at work, especially Vietnamese speakers. Select a
sentence in Slack, Outlook, Teams, a browser or any other app, press `Ctrl+Alt+E` (or click the floating icon next
to the selection), and a popup suggests several versions, each in a different tone. **Apply** pastes the version you pick back in place of the selection.

## Features

- **Works in any app.** Reads the selection through UI Automation, falls back to the clipboard, and to on-device
  OCR when neither works. Your clipboard is always restored.
- **Translate or rewrite, in Vietnamese, English or Japanese.** Pick the target language (Vietnamese by default)
  and a mode, **Translate** (close to the original) or **Rewrite** (rephrased freely), right in the popup.
- **Several tones at once:** concise, natural and formal.
- **Apply in place.** Replaces the selection in the original app when the field is editable; **Copy** otherwise.
- **Privacy first.** Emails, phone numbers, IDs, card numbers, URLs, API keys and your own terms are redacted
  before anything is sent, and restored for display. Password fields and apps you block are never read.
  Nothing is logged or stored, and there is no telemetry. See [docs/specs/privacy.md](docs/specs/privacy.md).
- **Bring your own key.** Google Gemini (free tier, default), Groq (free tier), OpenRouter (free models) or the
  Claude API (paid). Keys are kept in Windows Credential Manager.
- **Vietnamese and English interface**, light and dark themes, start with Windows, tray menu.

## Status

ToneSnap runs on **Windows 10/11**; macOS support is planned. Download the installer from the
[latest release](https://github.com/nhutruong-struct/struct-tonesnap/releases/latest), or build it from source
as described below. The installers are not code-signed yet, so Windows SmartScreen asks for confirmation the
first time (**More info → Run anyway**). See the [changelog](CHANGELOG.md) for what is in each version.

## Getting started

To just use ToneSnap, run the installer from the [latest release](https://github.com/nhutruong-struct/struct-tonesnap/releases/latest)
and go to [Choose an AI provider](#choose-an-ai-provider). The rest of this section is for building from source.

### Requirements

- Windows 10 or 11
- [Rust](https://rustup.rs/) (stable, MSVC toolchain) and Visual Studio Build Tools with
  "Desktop development with C++"
- [Node.js](https://nodejs.org/) 20 or newer and [pnpm](https://pnpm.io/) 10
- [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) (preinstalled on Windows 11)

### Run in development

```bash
pnpm install
```

```bash
pwsh -File dev.ps1
```

`dev.ps1` starts Vite and the Tauri app; Rust changes rebuild and restart it. Run `pwsh -File dev.ps1 -Check`
to verify the toolchain.

### Build an installer

```bash
pnpm tauri build
```

The installers are written to `src-tauri/target/release/bundle/`.

### Choose an AI provider

Open **Settings** from the tray icon, pick a provider and save its API key:

| Provider | Cost | Get a key |
|---|---|---|
| Google Gemini (default) | Free tier | [aistudio.google.com/apikey](https://aistudio.google.com/apikey) |
| Groq | Free tier | [console.groq.com/keys](https://console.groq.com/keys) |
| OpenRouter | Free models | [openrouter.ai/keys](https://openrouter.ai/keys) |
| Claude API | Paid | [console.anthropic.com](https://console.anthropic.com/settings/keys) |

Free tiers have daily limits, and the provider may use what you send to improve its products. There is no
local model: the (redacted) text always goes to the provider you choose.

## Usage

1. Select some text in any app.
2. Press `Ctrl+Alt+E`, or click the ToneSnap icon that appears next to the selection.
3. Read the suggestions, then click **Apply** on the one you like (or **Copy**).

The hotkey, the trigger mode, the tones, the target language and mode, and the blocked apps can be changed in
Settings. `Ctrl+Alt+Space` is avoided by default because Claude desktop's Quick Entry uses it.

## How it works

```
Trigger (hotkey / floating icon)
  → Selection + context: UI Automation → clipboard → on-device OCR
  → Safety: skip password fields and blocked apps, redact sensitive data (reversibly)
  → AI call to the provider you chose, streamed
  → Popup: one version per tone
  → Apply: refocus the original app → paste → restore the previous clipboard
```

ToneSnap is built with [Tauri 2](https://tauri.app/): a Rust core (Win32, UI Automation, OCR, providers) and a
React + TypeScript interface.

| Path | Role |
|---|---|
| `src-tauri/src/capture.rs` | Selection and context: UI Automation → clipboard → OCR |
| `src-tauri/src/platform/win/` | Win32: UIA, clipboard snapshot/restore, SendInput, OCR, focus, floating icon |
| `src-tauri/src/redact.rs` | Reversible redaction of sensitive data before sending |
| `src-tauri/src/prompt.rs`, `src-tauri/prompts/` | System prompt and output JSON schema |
| `src-tauri/src/llm/` | Providers: the Claude Messages API and an OpenAI-compatible client for Gemini, Groq and OpenRouter |
| `src-tauri/src/lib.rs` | App setup; one module per concern: `session`, `request`, `window`, `trigger`, `tray`, `commands`, `state` |
| `src-tauri/src/i18n.rs`, `src/lib/i18n.ts` | Interface strings in Vietnamese and English |
| `src/popup/`, `src/settings/` | The two windows |

More documentation:

- [docs/PLAN.md](docs/PLAN.md): decisions, roadmap and the app compatibility table
- [docs/specs/](docs/specs/README.md): how each built feature behaves, rule by rule
- [docs/HARNESS.md](docs/HARNESS.md): quality gates, hooks and the Claude Code setup

## Contributing

Contributions are welcome: bug reports, compatibility reports for the apps you use, ideas, translations, docs
and code. Please read [CONTRIBUTING.md](CONTRIBUTING.md) first; it explains the branch model, the checks every
pull request must pass, and the project rules (privacy above all).

- Questions and ideas: [GitHub Discussions](https://github.com/nhutruong-struct/struct-tonesnap/discussions)
- Bugs and feature requests: [Issues](https://github.com/nhutruong-struct/struct-tonesnap/issues/new/choose)
- Security problems: see [SECURITY.md](SECURITY.md); please do not open a public issue.

Everyone taking part is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Support the project

ToneSnap is free and MIT licensed, and it will stay that way. If it saves you time and you would like to help,
you can [sponsor the project on GitHub](https://github.com/sponsors/nhutruong-struct). Support is entirely
voluntary; it helps pay for development time and for testing on real apps and machines.

## License

ToneSnap is released under the [MIT License](LICENSE).
