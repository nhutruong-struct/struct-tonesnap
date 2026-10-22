//! Commands the two windows invoke through the typed wrappers in `src/lib/api.ts`.
//!
//! Synchronous commands run on Tauri's main thread, so they only touch in-memory state. Anything
//! that waits (files, the credential store, the network, the clipboard) is an async command or
//! hands off to its own thread.

use tauri::{AppHandle, Emitter};

use crate::error::CommandError;
use crate::settings::{self, Mode, ProviderKind, Settings, SettingsIssue, TargetLanguage};
use crate::state::state;
use crate::update::{self, UpdateInfo};
use crate::window::{self, NoticeKind};
use crate::{platform, request, secrets, tray, trigger};

/// Runs `f` on the main thread and waits for its result. The wait is short: the main thread
/// only runs `f`.
fn on_main_thread<T: Send + 'static>(
    app: &AppHandle,
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

/// Runs a hotkey change on the main thread, where the global-shortcut plugin needs it.
fn hotkey_on_main_thread(
    app: &AppHandle,
    change: impl FnOnce(&AppHandle) -> Result<(), CommandError> + Send + 'static,
) -> Result<(), CommandError> {
    let main_app = app.clone();
    on_main_thread(app, move || change(&main_app))
        .unwrap_or_else(|err| Err(CommandError::save_failed(err)))
}

/// Switches to the new hotkey, then writes the settings file. A failed write registers the old
/// hotkey again, so the hotkey in use always matches the saved settings. A hotkey that cannot
/// be registered stops the save before anything is written.
fn switch_then_write(
    switch_hotkey: impl FnOnce() -> Result<(), CommandError>,
    write: impl FnOnce() -> Result<(), CommandError>,
    restore_hotkey: impl FnOnce(),
) -> Result<(), CommandError> {
    switch_hotkey()?;
    write().inspect_err(|_| restore_hotkey())
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    state(&app).settings()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, new_settings: Settings) -> Result<(), CommandError> {
    let old = state(&app).settings();
    let (from, to, previous) = (old.clone(), new_settings.clone(), old.clone());
    switch_then_write(
        || hotkey_on_main_thread(&app, move |app| trigger::switch_hotkey(app, &from, &to)),
        || {
            settings::save(&app, &new_settings)
                .map_err(|e| CommandError::save_failed(e.to_string()))
        },
        || {
            let _ = hotkey_on_main_thread(&app, move |app| trigger::apply_hotkey(app, &previous));
        },
    )?;
    trigger::apply_icon_mode(&new_settings);
    state(&app).set_saved_settings(new_settings.clone());
    // Once the new settings are in memory, so an update status change in between cannot rebuild
    // the menu in the old language.
    tray::refresh(&app, &new_settings);
    let _ = app.emit("settings-changed", &new_settings);
    update::settings_changed(&app);
    Ok(())
}

#[tauri::command]
pub fn hotkey_error(app: AppHandle) -> Option<CommandError> {
    state(&app).hotkey_error()
}

#[tauri::command]
pub fn settings_issue(app: AppHandle) -> Option<SettingsIssue> {
    state(&app).settings_issue()
}

#[tauri::command]
pub async fn has_api_key(provider: ProviderKind) -> bool {
    secrets::api_key(provider).is_some()
}

#[tauri::command]
pub async fn set_api_key(provider: ProviderKind, key: String) -> Result<(), String> {
    secrets::set_api_key(provider, key.trim()).map_err(|e| e.to_string())
}

/// Sends the current session (after preview) or asks again for new suggestions, in the target
/// language and mode the popup shows.
#[tauri::command]
pub fn send_request(app: AppHandle, target_language: TargetLanguage, mode: Mode) {
    if let Some(session) = state(&app).set_session_options(target_language, mode) {
        request::start(&app, session);
    }
}

#[tauri::command]
pub fn apply_text(app: AppHandle, text: String) -> Result<(), String> {
    let (target, anchor) = state(&app)
        .session()
        .map(|s| (s.target, s.anchor))
        .ok_or("no session")?;
    window::dismiss_popup(&app);
    std::thread::spawn(move || {
        if !platform::apply(target, anchor.as_ref(), &text) {
            platform::copy_text(&text);
            window::show_notice(&app, NoticeKind::ApplyFailed, String::new());
        }
    });
    Ok(())
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) {
    window::dismiss_popup(&app);
    // The clipboard can stay locked by another app for a moment.
    std::thread::spawn(move || platform::copy_text(&text));
}

/// Where the OS permissions stand; Settings polls it while the user grants them.
#[tauri::command]
pub fn get_permissions() -> platform::Permissions {
    platform::permissions()
}

/// Shows the system prompt for a permission. Off the main thread: the system call can wait on
/// the user's session.
#[tauri::command]
pub async fn request_permission(permission: platform::Permission) {
    platform::request_permission(permission);
}

#[tauri::command]
pub async fn open_permission_settings(permission: platform::Permission) {
    platform::open_permission_settings(permission);
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    window::dismiss_popup(&app);
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    window::dismiss_popup(&app);
    window::show_settings(&app);
}

#[tauri::command]
pub fn update_info(app: AppHandle) -> UpdateInfo {
    update::info(&app)
}

#[tauri::command]
pub async fn check_for_updates(app: AppHandle) {
    update::check_now(&app).await;
}

/// Downloads the offered version if needed, then installs it; ToneSnap closes and the new
/// version starts.
#[tauri::command]
pub async fn install_update(app: AppHandle) {
    update::install_now(&app).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `switch_then_write` with steps that record themselves in order.
    fn save_steps(
        hotkey_ok: bool,
        write_ok: bool,
    ) -> (Result<(), CommandError>, Vec<&'static str>) {
        let steps = std::cell::RefCell::new(Vec::new());
        let result = switch_then_write(
            || {
                steps.borrow_mut().push("switch");
                hotkey_ok.then_some(()).ok_or_else(|| {
                    CommandError::hotkey("hotkey_taken", "ctrl+alt+e", "in use".into())
                })
            },
            || {
                steps.borrow_mut().push("write");
                write_ok
                    .then_some(())
                    .ok_or_else(|| CommandError::save_failed("disk full".into()))
            },
            || steps.borrow_mut().push("restore"),
        );
        (result, steps.into_inner())
    }

    #[test]
    fn a_save_switches_the_hotkey_then_writes() {
        let (result, steps) = save_steps(true, true);
        assert!(result.is_ok());
        assert_eq!(steps, ["switch", "write"]);
    }

    #[test]
    fn a_hotkey_that_cannot_be_registered_writes_nothing() {
        let (result, steps) = save_steps(false, true);
        assert_eq!(result.unwrap_err().kind, "hotkey_taken");
        assert_eq!(steps, ["switch"]);
    }

    #[test]
    fn a_failed_write_puts_the_old_hotkey_back() {
        let (result, steps) = save_steps(true, false);
        assert_eq!(result.unwrap_err().kind, "save_failed");
        assert_eq!(steps, ["switch", "write", "restore"]);
    }
}
