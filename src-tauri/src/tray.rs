//! The tray icon and its menu.

use tauri::image::Image;
use tauri::menu::{IconMenuItem, Menu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use crate::settings::Settings;
use crate::{i18n, window};

/// The menu items whose labels follow the interface language.
struct TrayMenu {
    settings: IconMenuItem<Wry>,
    quit: IconMenuItem<Wry>,
}

// Drawn by `scripts/make-tray-icons.mjs`.
const SETTINGS_ICON: &[u8] = include_bytes!("../icons/tray/settings.png");
const QUIT_ICON: &[u8] = include_bytes!("../icons/tray/quit.png");

fn menu_item(
    app: &App,
    id: &str,
    label: &str,
    icon: &'static [u8],
) -> tauri::Result<IconMenuItem<Wry>> {
    IconMenuItem::with_id(
        app,
        id,
        label,
        true,
        Some(Image::from_bytes(icon)?),
        None::<&str>,
    )
}

pub fn build(app: &App, settings: &Settings) -> tauri::Result<()> {
    let labels = i18n::tray_labels(settings);
    let open = menu_item(app, "settings", labels.settings, SETTINGS_ICON)?;
    let quit = menu_item(app, "quit", labels.quit, QUIT_ICON)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_icons_are_valid_images_of_the_same_size() {
        let settings = Image::from_bytes(SETTINGS_ICON).unwrap();
        let quit = Image::from_bytes(QUIT_ICON).unwrap();
        assert_eq!((settings.width(), settings.height()), (20, 20));
        assert_eq!((quit.width(), quit.height()), (20, 20));
    }
}
