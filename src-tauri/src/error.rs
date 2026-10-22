//! The error commands return to the UI. Its own module, so the state and the hotkey code can
//! use it without depending on the command layer.

use serde::Serialize;

/// Error codes the UI translates; `detail` carries the raw message.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub kind: &'static str,
    /// The hotkey the error is about, for the hotkey errors.
    pub hotkey: String,
    detail: String,
}

impl CommandError {
    pub fn hotkey(kind: &'static str, hotkey: &str, detail: String) -> Self {
        Self {
            kind,
            hotkey: hotkey.to_string(),
            detail,
        }
    }

    pub fn save_failed(detail: String) -> Self {
        Self {
            kind: "save_failed",
            hotkey: String::new(),
            detail,
        }
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.kind, self.detail)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn errors_serialize_like_the_typescript_type() {
        let taken = CommandError::hotkey("hotkey_taken", "ctrl+alt+e", "in use".into());
        assert_eq!(
            serde_json::to_value(&taken).unwrap(),
            json!({ "kind": "hotkey_taken", "hotkey": "ctrl+alt+e", "detail": "in use" })
        );
        let failed = CommandError::save_failed("disk full".into());
        assert_eq!(
            serde_json::to_value(&failed).unwrap(),
            json!({ "kind": "save_failed", "hotkey": "", "detail": "disk full" })
        );
        assert_eq!(taken.to_string(), "hotkey_taken (in use)");
    }
}
