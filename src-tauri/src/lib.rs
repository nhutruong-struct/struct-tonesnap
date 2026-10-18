mod capture;
mod commands;
mod geometry;
mod i18n;
mod llm;
mod platform;
mod prompt;
mod redact;
mod request;
mod secrets;
mod session;
mod settings;
mod state;
mod tray;
mod trigger;
mod window;

use tauri::{App, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show_settings(app)
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        session::on_trigger(app.clone());
                    }
                })
                .build(),
        )
        .setup(setup)
        .on_window_event(|win, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = win.hide();
            }
            WindowEvent::Focused(false) if win.label() == window::POPUP => {
                window::on_popup_blurred(win.app_handle())
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::hotkey_error,
            commands::settings_issue,
            commands::has_api_key,
            commands::set_api_key,
            commands::send_request,
            commands::apply_text,
            commands::copy_text,
            commands::hide_popup,
            commands::open_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Keep running in the tray when the last window is hidden or closed.
            if let RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    let (settings, issue) = settings::load(&handle);
    let settings_need_attention = issue.is_some();
    app.manage(AppState::new(settings.clone(), issue));
    tray::build(app, &settings)?;
    trigger::init_icon(&handle)?;
    trigger::apply_icon_mode(&settings);
    if let Err(e) = trigger::apply_hotkey(&handle, &settings) {
        eprintln!("hotkey {}: {e}", e.hotkey);
        window::show_settings(&handle);
    }
    let missing_key = secrets::api_key(settings.provider).is_none();
    if settings_need_attention || missing_key {
        window::show_settings(&handle);
    }
    Ok(())
}
