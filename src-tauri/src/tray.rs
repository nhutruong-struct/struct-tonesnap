//! The tray icon and its menu.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Wry};

use crate::settings::Settings;
use crate::{i18n, update, window};

const TRAY: &str = "main";

pub fn build(app: &App, settings: &Settings) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY)
        .tooltip("ToneSnap")
        .menu(&menu(app.handle(), settings)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => window::show_settings(app),
            "update" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move { update::install_now(&app).await });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::show_settings(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// The menu in the interface language, led by an install item while a new version is offered.
fn menu(app: &AppHandle, settings: &Settings) -> tauri::Result<Menu<Wry>> {
    let labels = i18n::tray_labels(settings);
    let open = MenuItem::with_id(app, "settings", labels.settings, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    match update::offered_version(app) {
        Some(version) => {
            let label = i18n::tray_update_label(settings, &version);
            let install = MenuItem::with_id(app, "update", label, true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            Menu::with_items(app, &[&install, &separator, &open, &quit])
        }
        None => Menu::with_items(app, &[&open, &quit]),
    }
}

/// Rebuilds the menu after the interface language or the offered update changed.
pub fn refresh(app: &AppHandle, settings: &Settings) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY), menu(app, settings)) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Hides the icon just before the updater ends the process.
pub fn hide(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY) {
        let _ = tray.set_visible(false);
    }
}

/// Shows the icon again when the installer could not be started after all.
pub fn show(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id(TRAY) {
        let _ = tray.set_visible(true);
    }
}
