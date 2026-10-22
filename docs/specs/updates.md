# Updates

> **Status:** implemented (Phase 5), Windows and macOS (Apple Silicon). The first release that can update itself is the one after 1.0.0:
> 1.0.0 has no updater, so its users install the next version by hand once.
> **Source:** `src-tauri/src/update.rs`, `tray.rs`, `commands.rs` (`update_info`, `check_for_updates`, `install_update`), `src-tauri/tauri.conf.json` (`plugins.updater`), `src/settings/sections/UpdatesSection.tsx`, `.github/workflows/release.yml`, `scripts/release/updater-manifest.mjs`, `scripts/release/sign-macos.sh`.

ToneSnap installs new versions from the GitHub releases of `nhutruong-struct/struct-tonesnap` with the official Tauri updater plugin, driven from Rust (no JavaScript API and no capability).

## Where updates come from

- **UPD-01** The app reads `https://github.com/nhutruong-struct/struct-tonesnap/releases/latest/download/latest.json`. The manifest names the new version and, per installer kind, the installer's URL and its minisign signature. GitHub serves the files from its asset hosts (`objects.githubusercontent.com`, `release-assets.githubusercontent.com`) after a redirect.
- **UPD-02** A request carries only what the URL and headers say: the app's version and platform. It never carries user text, settings or keys ([privacy.md](privacy.md) PRV-04). It uses the operating system's TLS stack and proxy settings, like the provider requests (PRV-03).
- **UPD-03** An installer is installed only when its signature verifies against the public key built into the app (`plugins.updater.pubkey`), and only when the signature itself names the announced version (`requireSignedVersion`), so a forged manifest cannot pair a high version number with an older genuine installer. Only a version newer than the running one counts as an update.
- **UPD-04** Each release publishes `ToneSnap_<version>_x64-setup.exe` (NSIS), `ToneSnap_<version>_x64_en-US.msi`, `SHA256SUMS.txt` and `latest.json` (which carries the signatures; the `.sig` files stay in the workflow). An app installed with the MSI updates with the MSI (`windows-x86_64-msi`); every other install updates with the NSIS installer (`windows-x86_64-nsis`, `windows-x86_64`).
- **UPD-05** The installers are built without the key. A separate `sign` job of `release.yml`, run only when a push to `master` carries a version without a release, signs them with `tauri signer sign --app-version` and the `TAURI_SIGNING_PRIVATE_KEY` secret of the `release` environment, in one step that runs nothing but the Tauri CLI, so no build script or dependency ever sees the key. A release run fails when the key is missing or `pubkey` is empty, and publishing fails when an installer has no signature.

## When ToneSnap checks

- **UPD-10** A release build checks one minute after it starts and then every six hours. A development build never checks by itself (it would replace itself with the released installer); *Check for Updates* still works there.
- **UPD-11** *Check for Updates* in Settings checks at once. While a version is downloaded and waiting to be installed, checks are skipped.
- **UPD-12** A failed check, download or install (offline, GitHub unreachable, a bad signature, an installer that would not start) is shown in Settings, translated by step, with the raw error after it, and logged without any user text. After a failed check the next one comes 30 minutes later instead of six hours; a failed download or install waits the full six hours, since the same release would fail again and auto mode would download it each time.
- **UPD-13** Only one check, download or install runs at a time; a request that arrives while one runs is ignored.

## Update mode

The `updateMode` setting ([settings.md](settings.md)) decides what happens once a new version is found.

- **UPD-20** **Auto** (default): the new version is downloaded in the background at once. It is installed once ToneSnap is idle: no popup and no Settings window open, no capture running, and no popup in the last two minutes. The check for an idle moment runs every 30 seconds, so the user is never interrupted mid-task.
- **UPD-24** An app installed with the MSI always behaves as Manual, whatever the setting says: its updates need administrator rights, and a Windows prompt for them must not appear while the user is away or busy (declining it would leave ToneSnap closed). Settings says so under the two modes.
- **UPD-21** **Manual**: ToneSnap only says that the version exists, in the tray and in Settings. Nothing is downloaded until the user clicks *Update & Restart*.
- **UPD-22** *Update & Restart* (Settings, or the tray item) installs at once in either mode, downloading first when needed.
- **UPD-23** A new mode applies as soon as Settings are saved: switching to Auto starts downloading a version that was only announced; switching to Manual stops a downloaded version from installing by itself.

## Installing

- **UPD-30** Installing first claims the capture flag, so the hotkey cannot start a capture (which may hold the clipboard) that the exit would cut off; if a capture is running, an automatic install waits for the next idle moment. It then closes the popup, hides the tray icon and runs the installer in passive mode (a progress bar, no questions; an MSI install asks for administrator rights). ToneSnap exits, and the installer starts the new version when it is done. If the installer cannot be started, the tray icon comes back and the failure is shown (UPD-12).
- **UPD-31** Settings and API keys are kept: they live in the app config folder and the credential store, which the installer does not touch.

## In the interface

- **UPD-40** The tray menu starts with *Update to {version} & Restart* while a version can be installed (found, or downloaded and waiting), in the interface language ([app-shell.md](app-shell.md) APP-10).
- **UPD-41** Settings has an *Updates* section: the two modes, the running version, the status (not checked yet, checking, up to date, available, downloading, downloaded and when it will be installed, installing, what failed with the error), a note for MSI installs (UPD-24), *Check for Updates* (disabled while working) and *Update & Restart* when a version can be installed. The buttons act at once; the mode is saved with *Save Settings*.
- **UPD-42** Every status change reaches Settings through the `update-status` event ([contracts.md](contracts.md)).
- **UPD-50** On macOS (Apple Silicon) the updater downloads the signed `ToneSnap_<version>_aarch64.app.tar.gz` listed under `darwin-aarch64` in `latest.json`, replaces the app bundle in place, and ToneSnap restarts itself (`update.rs` `install`); Windows hands the installer over and exits. The signature, the modes and the schedule are the same as above.

**Verified by:** `update.rs` tests (auto downloads at once and installs only when idle, manual waits for the user even when idle, a user's request downloads and installs in either mode, nothing to do without a release, an MSI install always waits for the user, a failed check is retried sooner but a failed download is not, what idle means, which status the tray offers, one run at a time, the status JSON), `settings.rs` tests (`updateMode` camelCase JSON, an unknown mode falls back), `src/settings/sections/UpdatesSection.test.tsx` (version and status shown, the mode is saved with the settings, Check for Updates and its disabled state, Update & Restart, the downloaded text follows the chosen mode, status events, the MSI note, an event is not overwritten by the first answer), `scripts/release/updater-manifest.test.mjs` (manifest keys and URLs, missing signatures), the harness `network` rule (the updater endpoints in `tauri.conf.json`). Installing a real update needs two signed releases and is tested by hand (see the pull request that added this page).
