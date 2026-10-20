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

#[cfg(test)]
mod tests {
    use super::*;

    fn in_language(code: &str) -> Settings {
        Settings {
            ui_language: code.into(),
            ..Settings::default()
        }
    }

    #[test]
    fn the_tray_menu_follows_the_interface_language() {
        let vi = tray_labels(&in_language("vi"));
        assert_eq!((vi.settings, vi.quit), ("Cài đặt…", "Thoát ToneSnap"));
        let en = tray_labels(&in_language("en"));
        assert_eq!((en.settings, en.quit), ("Settings…", "Quit ToneSnap"));
    }

    #[test]
    fn the_update_item_names_the_version_in_both_languages() {
        assert_eq!(
            tray_update_label(&in_language("en"), "1.2.0"),
            "Update to 1.2.0 & Restart"
        );
        assert_eq!(
            tray_update_label(&in_language("vi"), "1.2.0"),
            "Cập nhật lên 1.2.0 và khởi động lại"
        );
    }
}
