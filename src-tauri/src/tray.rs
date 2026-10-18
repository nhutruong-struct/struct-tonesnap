//! The tray icon and its menu.

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use crate::settings::Settings;
use crate::{i18n, window};

/// The menu items whose labels follow the interface language.
struct TrayMenu {
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn build(app: &App, settings: &Settings) -> tauri::Result<()> {
    let labels = i18n::tray_labels(settings);
    let open = MenuItem::with_id(app, "settings", labels.settings, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", labels.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("ToneSnap")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => window::show_settings(app),
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
    app.manage(TrayMenu {
        settings: open,
        quit,
    });
    Ok(())
}

pub fn update_labels(app: &AppHandle, settings: &Settings) {
    if let Some(menu) = app.try_state::<TrayMenu>() {
        let labels = i18n::tray_labels(settings);
        let _ = menu.settings.set_text(labels.settings);
        let _ = menu.quit.set_text(labels.quit);
    }
}
