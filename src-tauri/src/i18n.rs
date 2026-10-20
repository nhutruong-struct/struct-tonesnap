//! Interface strings rendered by the Rust side (the tray menu). The React UI keeps its own
//! dictionaries in `src/lib/i18n.ts`; both must offer the same languages. These two files are
//! the only places in the repository where Vietnamese text belongs.

use crate::settings::{self, Settings};

pub struct TrayLabels {
    pub settings: &'static str,
    pub quit: &'static str,
}

pub fn tray_labels(settings: &Settings) -> TrayLabels {
    match settings::ui_language(settings) {
        "vi" => TrayLabels {
            settings: "Cài đặt…",
            quit: "Thoát ToneSnap",
        },
        _ => TrayLabels {
            settings: "Settings…",
            quit: "Quit ToneSnap",
        },
    }
}

/// The tray item that installs a new version.
pub fn tray_update_label(settings: &Settings, version: &str) -> String {
    match settings::ui_language(settings) {
        "vi" => format!("Cập nhật lên {version} và khởi động lại"),
        _ => format!("Update to {version} & Restart"),
    }
}
