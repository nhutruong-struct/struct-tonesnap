use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

/// How the popup is triggered. The hotkey works in every mode except `Off`.
/// `Icon` shows a floating icon after a selection; `Auto` is not implemented yet.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerMode {
    Hotkey,
    Icon,
    Auto,
    Off,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// Paid Claude API.
    Claude,
    /// Google Gemini API; has a free tier.
    Gemini,
    /// Groq; has a free tier.
    Groq,
    /// OpenRouter; has free models.
    Openrouter,
}

/// How new versions from the GitHub releases are installed (see `update.rs`).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateMode {
    /// Download in the background and install once ToneSnap is idle.
    Auto,
    /// Only say that a new version exists; the user installs it.
    Manual,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// "auto" (follow the OS), "vi" or "en".
    pub ui_language: String,
    pub trigger_mode: TriggerMode,
    pub hotkey: String,
    pub auto_delay_ms: u32,
    pub auto_min_chars: u32,
    pub provider: ProviderKind,
    /// Claude model id.
    pub model: String,
    pub effort: String,
    pub gemini_model: String,
    pub groq_model: String,
    pub openrouter_model: String,
    /// Tone ids in display order, see `prompt::TONES`.
    pub tones: Vec<String>,
    /// "vi", "en" or "off".
    pub explain_language: String,
    /// Characters of surrounding text to send on each side of the selection.
    pub context_chars: u32,
    pub ocr_fallback: bool,
    pub redact: bool,
    pub custom_sensitive_terms: Vec<String>,
    pub preview_before_send: bool,
    /// Lower-case process names where ToneSnap never reads text.
    pub blocked_apps: Vec<String>,
    pub update_mode: UpdateMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_language: "auto".into(),
            trigger_mode: TriggerMode::Icon,
            // Ctrl+Alt+Space belongs to Claude desktop's Quick Entry. Ctrl+Alt+E only shadows
            // Word's euro-sign shortcut, which our users rarely need.
            hotkey: "ctrl+alt+e".into(),
            auto_delay_ms: 400,
            auto_min_chars: 8,
            provider: ProviderKind::Gemini,
            model: "claude-opus-5-5".into(),
            effort: "low".into(),
            gemini_model: "gemini-3.8-flash".into(),
            groq_model: "openai/gpt-oss-20b".into(),
            openrouter_model: "openai/gpt-oss-20b:free".into(),
            // Fewer tones mean a shorter answer, which is faster and stays within the free
            // tiers' token limits.
            tones: ["minimal", "concise", "polite"].map(String::from).to_vec(),
            explain_language: "vi".into(),
            context_chars: 600,
            ocr_fallback: true,
            redact: true,
            custom_sensitive_terms: Vec::new(),
            preview_before_send: false,
            blocked_apps: [
                "keepass.exe",
                "keepassxc.exe",
                "1password.exe",
                "bitwarden.exe",
                "lastpass.exe",
            ]
            .map(String::from)
            .to_vec(),
            update_mode: UpdateMode::Auto,
        }
    }
}

impl Settings {
    /// The model requests go to with the chosen provider.
    pub fn active_model(&self) -> &str {
        match self.provider {
            ProviderKind::Claude => &self.model,
            ProviderKind::Gemini => &self.gemini_model,
            ProviderKind::Groq => &self.groq_model,
            ProviderKind::Openrouter => &self.openrouter_model,
        }
    }

    /// Whether ToneSnap must never read text from this process (a lower-case `name.exe`).
    pub fn is_blocked_app(&self, process: &str) -> bool {
        self.blocked_apps
            .iter()
            .any(|b| b.eq_ignore_ascii_case(process))
    }
}

/// The interface language to use: the explicit choice, or the OS locale for "auto".
pub fn ui_language(settings: &Settings) -> &'static str {
    match settings.ui_language.as_str() {
        "vi" => "vi",
        "en" => "en",
        _ => match sys_locale::get_locale() {
            Some(locale) if locale.to_lowercase().starts_with("vi") => "vi",
            _ => "en",
        },
    }
}

/// Why the saved settings could not be used as they were. Shown in the Settings window until
/// the next successful save.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SettingsIssue {
    /// These fields held values ToneSnap does not understand and fell back to their defaults.
    InvalidFields { fields: Vec<String> },
    /// The file is not settings JSON. It was moved to `backup` so saving does not overwrite
    /// it, and the defaults are in use.
    Broken { backup: String },
    /// The file could not be read (locked, no permission) or, when broken, not moved aside. It
    /// was left as it is and the defaults are in use; saving replaces it.
    Unreadable,
}

fn settings_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    Ok(app.path().app_config_dir()?.join("settings.json"))
}

pub fn load(app: &AppHandle) -> (Settings, Option<SettingsIssue>) {
    match settings_path(app) {
        Ok(path) => load_from(&path),
        Err(_) => (Settings::default(), None),
    }
}

pub fn save(app: &AppHandle, settings: &Settings) -> anyhow::Result<()> {
    save_to(&settings_path(app)?, settings)
}

fn load_from(path: &Path) -> (Settings, Option<SettingsIssue>) {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        // First run.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return (Settings::default(), None)
        }
        // Possibly temporary (another program holds the file): leave it untouched.
        Err(_) => return (Settings::default(), Some(SettingsIssue::Unreadable)),
    };
    let parsed = String::from_utf8(bytes).ok().and_then(|text| parse(&text));
    match parsed {
        Some((settings, invalid)) if invalid.is_empty() => (settings, None),
        Some((settings, fields)) => (settings, Some(SettingsIssue::InvalidFields { fields })),
        None => {
            let issue = match set_aside(path) {
                Some(backup) => SettingsIssue::Broken {
                    backup: backup.display().to_string(),
                },
                None => SettingsIssue::Unreadable,
            };
            (Settings::default(), Some(issue))
        }
    }
}

/// Moves a broken settings file to the first free `settings.broken[-N].json` beside it, so an
/// older backup is never overwritten.
fn set_aside(path: &Path) -> Option<PathBuf> {
    let backup = (1..100)
        .map(|n| match n {
            1 => path.with_file_name("settings.broken.json"),
            n => path.with_file_name(format!("settings.broken-{n}.json")),
        })
        .find(|candidate| !candidate.exists())?;
    std::fs::rename(path, &backup).ok()?;
    Some(backup)
}

/// Reads settings from JSON, one field at a time: a field with a value ToneSnap does not
/// understand falls back to its default instead of discarding every other setting. Returns the
/// settings and the names of the fields that fell back, or `None` when the text is not a JSON
/// object at all.
fn parse(text: &str) -> Option<(Settings, Vec<String>)> {
    let Ok(Value::Object(saved)) = serde_json::from_str::<Value>(text) else {
        return None;
    };
    let mut merged = serde_json::to_value(Settings::default()).ok()?;
    let mut invalid = Vec::new();
    for (key, value) in saved {
        let previous = merged.get(&key).cloned();
        merged[&key] = value;
        if serde_json::from_value::<Settings>(merged.clone()).is_err() {
            match previous {
                Some(default) => merged[&key] = default,
                None => {
                    if let Some(fields) = merged.as_object_mut() {
                        fields.remove(&key);
                    }
                }
            }
            invalid.push(key);
        }
    }
    let settings = serde_json::from_value(merged).unwrap_or_default();
    Some((settings, invalid))
}

/// Writes a sibling file first and swaps it in, so a crash mid-write cannot leave a truncated
/// settings file behind.
fn save_to(path: &Path, settings: &Settings) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let partial = path.with_extension("json.partial");
    std::fs::write(&partial, serde_json::to_string_pretty(settings)?)?;
    std::fs::rename(&partial, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system temp folder, removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("tonesnap-settings-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn file(&self) -> PathBuf {
            self.0.join("settings.json")
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_missing_file_means_defaults_without_an_issue() {
        let dir = TempDir::new("missing");
        let (settings, issue) = load_from(&dir.file());
        assert_eq!(settings.hotkey, Settings::default().hotkey);
        assert_eq!(issue, None);
    }

    #[test]
    fn saved_settings_round_trip_and_leave_no_partial_file() {
        let dir = TempDir::new("round-trip");
        let settings = Settings {
            hotkey: "ctrl+alt+r".into(),
            trigger_mode: TriggerMode::Hotkey,
            tones: vec!["polite".into()],
            ..Settings::default()
        };
        save_to(&dir.file(), &settings).unwrap();
        let (loaded, issue) = load_from(&dir.file());
        assert_eq!(issue, None);
        assert_eq!(loaded.hotkey, "ctrl+alt+r");
        assert_eq!(loaded.trigger_mode, TriggerMode::Hotkey);
        assert_eq!(loaded.tones, ["polite"]);
        assert!(!dir.file().with_extension("json.partial").exists());
    }

    #[test]
    fn an_invalid_field_falls_back_alone() {
        let (settings, invalid) = parse(
            r#"{ "hotkey": "ctrl+alt+r", "triggerMode": "telepathy", "contextChars": "lots",
                 "someOldField": 1 }"#,
        )
        .unwrap();
        assert_eq!(settings.hotkey, "ctrl+alt+r");
        assert_eq!(settings.trigger_mode, Settings::default().trigger_mode);
        assert_eq!(settings.context_chars, Settings::default().context_chars);
        // serde_json keeps object keys sorted.
        assert_eq!(invalid, ["contextChars", "triggerMode"]);
    }

    #[test]
    fn a_broken_file_is_set_aside_without_overwriting_older_backups() {
        let dir = TempDir::new("broken");
        let broken = |backup: &str| {
            Some(SettingsIssue::Broken {
                backup: dir.0.join(backup).display().to_string(),
            })
        };

        std::fs::write(dir.file(), "{ not json").unwrap();
        let (settings, issue) = load_from(&dir.file());
        assert_eq!(settings.hotkey, Settings::default().hotkey);
        assert_eq!(issue, broken("settings.broken.json"));
        assert!(!dir.file().exists());

        std::fs::write(dir.file(), [0xff, 0xfe, 0x00]).unwrap();
        assert_eq!(load_from(&dir.file()).1, broken("settings.broken-2.json"));
        assert_eq!(
            std::fs::read_to_string(dir.0.join("settings.broken.json")).unwrap(),
            "{ not json"
        );
        assert_eq!(
            std::fs::read(dir.0.join("settings.broken-2.json")).unwrap(),
            [0xff, 0xfe, 0x00]
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_is_left_alone() {
        let dir = TempDir::new("unreadable");
        // A directory where the file should be cannot be read as one.
        std::fs::create_dir(dir.file()).unwrap();
        let (settings, issue) = load_from(&dir.file());
        assert_eq!(settings.hotkey, Settings::default().hotkey);
        assert_eq!(issue, Some(SettingsIssue::Unreadable));
        assert!(dir.file().is_dir());
        assert!(!dir.0.join("settings.broken.json").exists());
    }

    #[test]
    fn a_json_value_that_is_not_an_object_is_unreadable() {
        assert!(parse("[1, 2]").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn issues_serialize_like_the_typescript_type() {
        let invalid = SettingsIssue::InvalidFields {
            fields: vec!["triggerMode".into()],
        };
        assert_eq!(
            serde_json::to_value(invalid).unwrap(),
            serde_json::json!({ "kind": "invalid_fields", "fields": ["triggerMode"] })
        );
        let broken = SettingsIssue::Broken {
            backup: "C:\\settings.broken.json".into(),
        };
        assert_eq!(
            serde_json::to_value(broken).unwrap(),
            serde_json::json!({ "kind": "broken", "backup": "C:\\settings.broken.json" })
        );
        assert_eq!(
            serde_json::to_value(SettingsIssue::Unreadable).unwrap(),
            serde_json::json!({ "kind": "unreadable" })
        );
    }

    #[test]
    fn settings_serialize_in_camel_case_with_snake_case_enums() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert_eq!(value["triggerMode"], "icon");
        assert_eq!(value["provider"], "gemini");
        assert_eq!(value["geminiModel"], "gemini-3.8-flash");
        assert_eq!(value["openrouterModel"], "openai/gpt-oss-20b:free");
        assert_eq!(value["previewBeforeSend"], false);
        assert_eq!(value["updateMode"], "auto");
        assert!(value.get("trigger_mode").is_none());
    }

    #[test]
    fn the_active_model_follows_the_provider() {
        let settings = |provider| Settings {
            provider,
            ..Settings::default()
        };
        let default = Settings::default();
        assert_eq!(default.active_model(), default.gemini_model);
        assert_eq!(settings(ProviderKind::Claude).active_model(), default.model);
        assert_eq!(
            settings(ProviderKind::Groq).active_model(),
            default.groq_model
        );
        assert_eq!(
            settings(ProviderKind::Openrouter).active_model(),
            default.openrouter_model
        );
    }

    #[test]
    fn blocked_apps_match_case_insensitively() {
        let settings = Settings::default();
        assert!(settings.is_blocked_app("KeePass.exe"));
        assert!(!settings.is_blocked_app("notepad.exe"));
    }

    #[test]
    fn the_update_mode_round_trips_and_an_unknown_one_falls_back() {
        let (settings, invalid) = parse(r#"{ "updateMode": "manual" }"#).unwrap();
        assert_eq!(settings.update_mode, UpdateMode::Manual);
        assert!(invalid.is_empty());
        let (settings, invalid) = parse(r#"{ "updateMode": "nightly" }"#).unwrap();
        assert_eq!(settings.update_mode, UpdateMode::Auto);
        assert_eq!(invalid, ["updateMode"]);
    }

    #[test]
    fn a_saved_ollama_provider_falls_back_to_the_default() {
        let (settings, invalid) = parse(r#"{ "provider": "ollama", "ollamaModel": "x" }"#).unwrap();
        assert_eq!(settings.provider, ProviderKind::Gemini);
        // Unknown fields are ignored, not reported.
        assert_eq!(invalid, ["provider"]);
    }
}
