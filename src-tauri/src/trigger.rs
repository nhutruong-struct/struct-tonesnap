//! What opens the popup: the global hotkey, in Icon mode the floating icon next to a fresh
//! selection, and in Auto mode the selection itself.

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::error::CommandError;
use crate::platform::selection_icon;
use crate::session;
use crate::settings::{Settings, TriggerMode};
use crate::state::state;

// The hotkey functions must run on the main thread. Called from another thread, the
// global-shortcut plugin keeps a lock while it waits for the main thread, and its own hotkey
// handler takes that lock on the main thread: a hotkey press at that moment deadlocks the app.

/// Registers the hotkey and records the outcome for the Settings window.
pub fn apply_hotkey(app: &AppHandle, settings: &Settings) -> Result<(), CommandError> {
    let result = register_hotkey(app, settings);
    state(app).set_hotkey_error(result.as_ref().err().cloned());
    result
}

/// Moves from the old settings' hotkey to the new one. If the new one cannot be registered, the
/// old one is restored instead of leaving none.
pub fn switch_hotkey(app: &AppHandle, old: &Settings, new: &Settings) -> Result<(), CommandError> {
    apply_hotkey(app, new).inspect_err(|_| {
        let _ = apply_hotkey(app, old);
    })
}

fn register_hotkey(app: &AppHandle, settings: &Settings) -> Result<(), CommandError> {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    if settings.trigger_mode == TriggerMode::Off {
        return Ok(());
    }
    let shortcut: Shortcut = settings
        .hotkey
        .parse()
        .map_err(|e| CommandError::hotkey("hotkey_invalid", &settings.hotkey, format!("{e}")))?;
    shortcuts
        .register(shortcut)
        .map_err(|e| CommandError::hotkey("hotkey_taken", &settings.hotkey, e.to_string()))
}

/// Gives the floating icon what it needs from the app: its image, what a click does, what Auto
/// mode does with a selection and which apps it must stay away from.
pub fn init_icon(app: &AppHandle) -> tauri::Result<()> {
    let image = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128.png"))?;
    selection_icon::set_image(image.rgba().to_vec(), image.width(), image.height());
    let click_app = app.clone();
    selection_icon::set_on_click(move || session::on_trigger(click_app.clone()));
    let selection_app = app.clone();
    selection_icon::set_on_selection(move |selection| {
        session::on_selection(&selection_app, selection)
    });
    let filter_app = app.clone();
    selection_icon::set_app_filter(move |process| {
        !state(&filter_app).settings().is_blocked_app(process)
    });
    Ok(())
}

/// Starts or stops the selection watcher behind the floating icon, which Auto mode shares. The
/// hotkey stays active in every mode but Off.
pub fn apply_icon_mode(settings: &Settings) {
    if matches!(settings.trigger_mode, TriggerMode::Icon | TriggerMode::Auto) {
        selection_icon::start();
    } else {
        selection_icon::stop();
    }
}
