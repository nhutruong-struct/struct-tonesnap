//! The tray icon and its menu.

use tauri::image::Image;
use tauri::menu::{IconMenuItem, Menu, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Wry};

use crate::settings::Settings;
use crate::{i18n, update, window};

const TRAY: &str = "main";

// Drawn by `scripts/make-tray-icons.mjs` at the size the menu shows them, so they are not rescaled
// into a blur: 16 pixels on Windows, 18 points (36 Retina pixels) on macOS.
#[cfg(target_os = "macos")]
macro_rules! menu_icon {
    ($name:literal) => {
        include_bytes!(concat!("../icons/tray/macos/", $name, ".png"))
    };
}
#[cfg(not(target_os = "macos"))]
macro_rules! menu_icon {
    ($name:literal) => {
        include_bytes!(concat!("../icons/tray/windows/", $name, ".png"))
    };
}
const SETTINGS_ICON: &[u8] = menu_icon!("settings");
const UPDATE_ICON: &[u8] = menu_icon!("update");
const QUIT_ICON: &[u8] = menu_icon!("quit");

fn menu_item(
    app: &AppHandle,
    id: &str,
    label: &str,
    icon: Option<&'static [u8]>,
) -> tauri::Result<IconMenuItem<Wry>> {
    let icon = icon.map(Image::from_bytes).transpose()?;
    IconMenuItem::with_id(app, id, label, true, icon, None::<&str>)
}

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
    let open = menu_item(app, "settings", labels.settings, Some(SETTINGS_ICON))?;
    let quit = menu_item(app, "quit", labels.quit, Some(QUIT_ICON))?;
    match update::offered_version(app) {
        Some(version) => {
            let label = i18n::tray_update_label(settings, &version);
            let install = menu_item(app, "update", &label, Some(UPDATE_ICON))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_icons_are_valid_images_of_the_size_the_menu_shows() {
        let size = if cfg!(target_os = "macos") { 36 } else { 16 };
        for bytes in [SETTINGS_ICON, UPDATE_ICON, QUIT_ICON] {
            let icon = Image::from_bytes(bytes).unwrap();
            assert_eq!((icon.width(), icon.height()), (size, size));
        }
    }
}
