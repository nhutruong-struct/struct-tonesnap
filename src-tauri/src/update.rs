//! New versions from the GitHub releases, through the Tauri updater plugin. ToneSnap checks a
//! minute after it starts and then every six hours. In auto mode a new version is downloaded in
//! the background and installed once ToneSnap is idle, so the user is never interrupted; in
//! manual mode ToneSnap only says that it exists (tray and Settings) until the user installs it.
//! Installing runs the signed installer, which closes ToneSnap and starts the new version.
//!
//! The requests carry only the app's version and platform, never user text.

use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::utils::{config::BundleType, platform::bundle_type};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::settings::UpdateMode;
use crate::state::state;
use crate::{tray, window};

/// Leaves the first minute to starting up.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(60);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);
/// Sooner after a failure, such as on a laptop that started offline.
const RETRY_INTERVAL: Duration = Duration::from_secs(30 * 60);
/// How often a downloaded update looks for an idle moment to install.
const IDLE_POLL: Duration = Duration::from_secs(30);
/// How long the popup must have been gone before ToneSnap counts as idle: a user who just used
/// it may be about to use it again.
const QUIET_PERIOD: Duration = Duration::from_secs(120);

/// Where updating stands, shown in Settings.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateStatus {
    /// No check has run yet.
    #[default]
    Idle,
    Checking,
    UpToDate,
    /// A newer version exists and is not downloaded yet.
    Available {
        version: String,
    },
    Downloading {
        version: String,
    },
    /// Downloaded and verified, waiting for an idle moment (auto) or for the user (manual).
    Ready {
        version: String,
    },
    /// The installer is starting; ToneSnap is about to close.
    Installing {
        version: String,
    },
    /// The last check, download or install failed; the next check tries again. `detail` is the
    /// raw error, shown after the translated message for `step`.
    Failed {
        step: FailedStep,
        detail: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailedStep {
    Check,
    Download,
    Install,
}

impl UpdateStatus {
    /// The version the user can install now, which the tray offers.
    fn offered_version(&self) -> Option<&str> {
        match self {
            Self::Available { version } | Self::Ready { version } => Some(version),
            _ => None,
        }
    }
}

/// What the `update-status` event and the `update_info` command send to Settings.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub current_version: String,
    pub status: UpdateStatus,
    /// Installed with the MSI, so updates need administrator rights and always wait for the user.
    pub msi_install: bool,
    /// False where releases carry no installer for the in-app updater (macOS, for now).
    pub supported: bool,
}

/// The release the last check found, with its installer once downloaded.
struct Release {
    update: Update,
    installer: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct Updates {
    status: Mutex<UpdateStatus>,
    release: Mutex<Option<Release>>,
    /// A check, download or install is running.
    busy: AtomicBool,
}

/// Clears `Updates::busy` when the work that set it ends, however it ends.
struct Busy<'a>(&'a AtomicBool);

impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.0.store(false, SeqCst);
    }
}

impl Updates {
    fn status(&self) -> UpdateStatus {
        self.status.lock().unwrap().clone()
    }

    fn try_begin(&self) -> Option<Busy<'_>> {
        (!self.busy.swap(true, SeqCst)).then_some(Busy(&self.busy))
    }
}

/// What the user is doing with ToneSnap right now.
#[derive(Clone, Copy, Debug)]
struct Activity {
    popup_visible: bool,
    settings_visible: bool,
    capturing: bool,
    popup_recent: bool,
}

/// Idle means the user is not using ToneSnap: no popup or Settings window, no capture running,
/// and no popup in the last few minutes.
fn is_idle(activity: Activity) -> bool {
    !(activity.popup_visible
        || activity.settings_visible
        || activity.capturing
        || activity.popup_recent)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Wait,
    Download,
    Install,
}

/// The next step for the release in `status`. Auto mode downloads it at once and installs it
/// when ToneSnap is idle; manual mode waits for the user, then downloads and installs it straight
/// away. A user's request also installs at once in auto mode.
fn next_step(status: &UpdateStatus, mode: UpdateMode, user_asked: bool, idle: bool) -> Step {
    let auto = mode == UpdateMode::Auto;
    match status {
        UpdateStatus::Available { .. } if auto || user_asked => Step::Download,
        UpdateStatus::Ready { .. } if user_asked || (auto && idle) => Step::Install,
        _ => Step::Wait,
    }
}

/// The mode that applies. An MSI install updates through a Windows prompt for administrator
/// rights, which must not appear while the user is away or busy, and which would leave ToneSnap
/// closed if they declined it; so it always waits for the user.
fn effective_mode(mode: UpdateMode, msi_install: bool) -> UpdateMode {
    if msi_install {
        UpdateMode::Manual
    } else {
        mode
    }
}

/// How long after the last check the next one is due. Only a failed check is retried sooner: a
/// release that failed to download or install would fail the same way, and auto mode would fetch
/// it again each time.
fn check_interval(status: &UpdateStatus) -> Duration {
    match status {
        UpdateStatus::Failed {
            step: FailedStep::Check,
            ..
        } => RETRY_INTERVAL,
        _ => CHECK_INTERVAL,
    }
}

fn is_msi_install() -> bool {
    bundle_type() == Some(BundleType::Msi)
}

pub fn info(app: &AppHandle) -> UpdateInfo {
    UpdateInfo {
        current_version: app.package_info().version.to_string(),
        status: app.state::<Updates>().status(),
        msi_install: is_msi_install(),
        supported: updates_supported(),
    }
}

/// The release workflow publishes updater installers for Windows only; a macOS build would
/// look for a platform the manifest does not list and report a failed check every time.
fn updates_supported() -> bool {
    cfg!(not(target_os = "macos"))
}

/// The version the tray offers to install, if any.
pub fn offered_version(app: &AppHandle) -> Option<String> {
    app.try_state::<Updates>()?
        .status()
        .offered_version()
        .map(String::from)
}

/// Checks on a schedule and installs a downloaded update when ToneSnap is idle. Release builds
/// only: a development build would replace itself with the released installer.
pub fn start(app: &AppHandle) {
    if cfg!(debug_assertions) || !updates_supported() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK_DELAY);
        let mut last_check: Option<Instant> = None;
        loop {
            let interval = check_interval(&app.state::<Updates>().status());
            let check = last_check.is_none_or(|at| at.elapsed() >= interval);
            if check {
                last_check = Some(Instant::now());
            }
            tauri::async_runtime::block_on(run(&app, check, false));
            std::thread::sleep(IDLE_POLL);
        }
    });
}

/// The user's *Check for Updates*: checks now, then steps as the mode allows.
pub async fn check_now(app: &AppHandle) {
    run(app, true, false).await;
}

/// The user's *Update & Restart*: downloads the release if needed and installs it now.
pub async fn install_now(app: &AppHandle) {
    run(app, false, true).await;
}

/// A new update mode applies at once: switching to auto starts downloading an announced release.
pub fn settings_changed(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { run(&app, false, false).await });
}

/// Checks for a new release (when `check`), then takes every step allowed now. Only one run
/// works at a time; one that finds another running does nothing.
async fn run(app: &AppHandle, check: bool, user_asked: bool) {
    if !updates_supported() {
        return;
    }
    let updates = app.state::<Updates>();
    let Some(_busy) = updates.try_begin() else {
        return;
    };
    if check {
        find(app).await;
    }
    loop {
        let mode = effective_mode(state(app).settings().update_mode, is_msi_install());
        match next_step(&updates.status(), mode, user_asked, is_idle(activity(app))) {
            Step::Wait => return,
            Step::Download => {
                if !download(app).await {
                    return;
                }
            }
            Step::Install => {
                // The capture flag keeps the hotkey from starting a capture (which may hold the
                // clipboard) that the exiting process would cut off. Once the installer has
                // started, the flag is never released.
                if state(app).try_begin_capture() {
                    install(app);
                    state(app).end_capture();
                }
                return;
            }
        }
    }
}

fn activity(app: &AppHandle) -> Activity {
    let visible = |label| {
        app.get_webview_window(label)
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false)
    };
    let state = state(app);
    Activity {
        popup_visible: visible(window::POPUP),
        settings_visible: visible(window::SETTINGS),
        capturing: state.is_capturing(),
        popup_recent: state.popup_shown_within(QUIET_PERIOD),
    }
}

async fn find(app: &AppHandle) {
    let updates = app.state::<Updates>();
    // The newest release found is downloaded already; it only waits to be installed.
    if matches!(updates.status(), UpdateStatus::Ready { .. }) {
        return;
    }
    set_status(app, UpdateStatus::Checking);
    let tray_app = app.clone();
    let checked = match app
        .updater_builder()
        // The process ends without Tauri's cleanup, which would leave a dead tray icon.
        .on_before_exit(move || tray::hide(&tray_app))
        .build()
    {
        Ok(updater) => updater.check().await,
        Err(e) => Err(e),
    };
    match checked {
        Ok(Some(update)) => {
            let version = update.version.clone();
            *updates.release.lock().unwrap() = Some(Release {
                update,
                installer: None,
            });
            set_status(app, UpdateStatus::Available { version });
        }
        Ok(None) => {
            *updates.release.lock().unwrap() = None;
            set_status(app, UpdateStatus::UpToDate);
        }
        Err(e) => fail(app, FailedStep::Check, e),
    }
}

/// Downloads the release found by the last check; the plugin verifies its signature.
async fn download(app: &AppHandle) -> bool {
    let updates = app.state::<Updates>();
    let update = updates
        .release
        .lock()
        .unwrap()
        .as_ref()
        .map(|r| r.update.clone());
    let Some(update) = update else {
        return false;
    };
    let version = update.version.clone();
    set_status(
        app,
        UpdateStatus::Downloading {
            version: version.clone(),
        },
    );
    match update.download(|_, _| {}, || {}).await {
        Ok(installer) => {
            if let Some(release) = updates.release.lock().unwrap().as_mut() {
                release.installer = Some(installer);
            }
            set_status(app, UpdateStatus::Ready { version });
            true
        }
        Err(e) => {
            fail(app, FailedStep::Download, e);
            false
        }
    }
}

fn install(app: &AppHandle) {
    let release = app.state::<Updates>().release.lock().unwrap().take();
    let Some(Release {
        update,
        installer: Some(installer),
    }) = release
    else {
        // Cannot happen (Ready always has an installer), but Ready must not stay behind without
        // one: it would stop the checks for good.
        set_status(app, UpdateStatus::Idle);
        return;
    };
    set_status(
        app,
        UpdateStatus::Installing {
            version: update.version.clone(),
        },
    );
    window::dismiss_popup(app);
    // Starts the installer and ends this process; the installer starts the new version.
    if let Err(e) = update.install(installer) {
        // The icon was hidden just before the installer was to start.
        tray::show(app);
        fail(app, FailedStep::Install, e);
    }
}

fn fail(app: &AppHandle, step: FailedStep, error: tauri_plugin_updater::Error) {
    eprintln!("update {step:?} failed: {error}");
    *app.state::<Updates>().release.lock().unwrap() = None;
    set_status(
        app,
        UpdateStatus::Failed {
            step,
            detail: error.to_string(),
        },
    );
}

fn set_status(app: &AppHandle, status: UpdateStatus) {
    let updates = app.state::<Updates>();
    let previous = std::mem::replace(&mut *updates.status.lock().unwrap(), status.clone());
    let _ = app.emit("update-status", info(app));
    if previous.offered_version() != status.offered_version() {
        tray::refresh(app, &state(app).settings());
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const AUTO: UpdateMode = UpdateMode::Auto;
    const MANUAL: UpdateMode = UpdateMode::Manual;

    fn available() -> UpdateStatus {
        UpdateStatus::Available {
            version: "1.1.0".into(),
        }
    }

    fn ready() -> UpdateStatus {
        UpdateStatus::Ready {
            version: "1.1.0".into(),
        }
    }

    fn failed() -> UpdateStatus {
        UpdateStatus::Failed {
            step: FailedStep::Check,
            detail: "offline".into(),
        }
    }

    #[test]
    fn auto_mode_downloads_at_once_and_installs_only_when_idle() {
        assert_eq!(next_step(&available(), AUTO, false, false), Step::Download);
        assert_eq!(next_step(&ready(), AUTO, false, false), Step::Wait);
        assert_eq!(next_step(&ready(), AUTO, false, true), Step::Install);
    }

    #[test]
    fn manual_mode_waits_for_the_user_even_when_idle() {
        assert_eq!(next_step(&available(), MANUAL, false, true), Step::Wait);
        assert_eq!(next_step(&ready(), MANUAL, false, true), Step::Wait);
    }

    #[test]
    fn a_user_request_downloads_and_installs_at_once_in_either_mode() {
        for mode in [AUTO, MANUAL] {
            assert_eq!(next_step(&available(), mode, true, false), Step::Download);
            assert_eq!(next_step(&ready(), mode, true, false), Step::Install);
        }
    }

    #[test]
    fn there_is_nothing_to_do_without_a_release() {
        let nothing = [
            UpdateStatus::Idle,
            UpdateStatus::Checking,
            UpdateStatus::UpToDate,
            UpdateStatus::Downloading {
                version: "1.1.0".into(),
            },
            UpdateStatus::Installing {
                version: "1.1.0".into(),
            },
            failed(),
        ];
        for status in nothing {
            assert_eq!(
                next_step(&status, AUTO, true, true),
                Step::Wait,
                "{status:?}"
            );
        }
    }

    #[test]
    fn an_msi_install_always_waits_for_the_user() {
        assert_eq!(effective_mode(AUTO, true), MANUAL);
        assert_eq!(effective_mode(MANUAL, true), MANUAL);
        assert_eq!(effective_mode(AUTO, false), AUTO);
        assert_eq!(effective_mode(MANUAL, false), MANUAL);
    }

    #[test]
    fn a_failed_check_is_retried_sooner() {
        assert_eq!(check_interval(&failed()), RETRY_INTERVAL);
        assert_eq!(check_interval(&UpdateStatus::UpToDate), CHECK_INTERVAL);
        for step in [FailedStep::Download, FailedStep::Install] {
            let status = UpdateStatus::Failed {
                step,
                detail: "bad signature".into(),
            };
            assert_eq!(check_interval(&status), CHECK_INTERVAL, "{step:?}");
        }
        assert!(RETRY_INTERVAL < CHECK_INTERVAL);
    }

    #[test]
    fn idle_means_no_window_no_capture_and_no_recent_popup() {
        let quiet = Activity {
            popup_visible: false,
            settings_visible: false,
            capturing: false,
            popup_recent: false,
        };
        assert!(is_idle(quiet));
        assert!(!is_idle(Activity {
            popup_visible: true,
            ..quiet
        }));
        assert!(!is_idle(Activity {
            settings_visible: true,
            ..quiet
        }));
        assert!(!is_idle(Activity {
            capturing: true,
            ..quiet
        }));
        assert!(!is_idle(Activity {
            popup_recent: true,
            ..quiet
        }));
    }

    #[test]
    fn the_tray_offers_only_a_release_that_can_be_installed() {
        assert_eq!(available().offered_version(), Some("1.1.0"));
        assert_eq!(ready().offered_version(), Some("1.1.0"));
        assert_eq!(UpdateStatus::UpToDate.offered_version(), None);
        let downloading = UpdateStatus::Downloading {
            version: "1.1.0".into(),
        };
        assert_eq!(downloading.offered_version(), None);
    }

    #[test]
    fn only_one_run_works_at_a_time() {
        let updates = Updates::default();
        let first = updates.try_begin();
        assert!(first.is_some());
        assert!(updates.try_begin().is_none());
        drop(first);
        assert!(updates.try_begin().is_some());
    }

    #[test]
    fn only_macos_has_no_in_app_updates_yet() {
        assert_eq!(updates_supported(), !cfg!(target_os = "macos"));
    }

    #[test]
    fn the_status_serializes_like_the_typescript_type() {
        let info = UpdateInfo {
            current_version: "1.0.0".into(),
            status: ready(),
            msi_install: false,
            supported: true,
        };
        assert_eq!(
            serde_json::to_value(info).unwrap(),
            json!({
                "currentVersion": "1.0.0",
                "status": { "kind": "ready", "version": "1.1.0" },
                "msiInstall": false,
                "supported": true
            })
        );
        assert_eq!(
            serde_json::to_value(UpdateStatus::UpToDate).unwrap(),
            json!({ "kind": "up_to_date" })
        );
        assert_eq!(
            serde_json::to_value(failed()).unwrap(),
            json!({ "kind": "failed", "step": "check", "detail": "offline" })
        );
        assert_eq!(
            serde_json::to_value(UpdateStatus::default()).unwrap(),
            json!({ "kind": "idle" })
        );
    }
}
