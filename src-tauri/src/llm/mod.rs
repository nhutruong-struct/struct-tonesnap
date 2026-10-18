//! Model providers behind one streaming interface: the Claude API (paid) and the providers that
//! speak the OpenAI dialect (Gemini, Groq, OpenRouter; free tiers). All receive the same system
//! prompt and JSON output schema.

mod claude;
mod openai_compat;
#[cfg(test)]
mod test_server;

use futures_util::StreamExt;
use serde_json::Value;

pub use openai_compat::{Endpoint, GEMINI, GROQ, OPENROUTER};

pub enum Provider<'a> {
    Claude {
        api_key: &'a str,
        model: &'a str,
        effort: &'a str,
    },
    /// Gemini, Groq or OpenRouter, told apart by `endpoint`.
    Compat {
        endpoint: &'static Endpoint,
        api_key: &'a str,
        model: &'a str,
    },
}

pub struct Prompt {
    pub system: &'static str,
    pub user: String,
    pub schema: Value,
}

#[derive(Debug, Default)]
pub struct Outcome {
    pub text: String,
    pub model: Option<String>,
    /// Normalized to the Claude vocabulary: "end_turn", "max_tokens", "refusal".
    pub stop_reason: Option<String>,
    pub usage: Value,
}

/// Why a request failed. `kind()` is the code the UI translates.
#[derive(Debug)]
pub enum Error {
    /// The provider answered with an error. `status` is 0 when there was no HTTP status, e.g.
    /// an error event inside the stream.
    Api {
        status: u16,
        kind: String,
        message: String,
    },
    /// The model declined to process the text.
    Refusal,
    /// The output hit the token limit and is incomplete.
    MaxTokens,
    /// The connection failed or the stream could not be read.
    Network(String),
}

impl Error {
    fn api(status: u16, kind: &str, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            kind: kind.to_string(),
            message: message.into(),
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Self::Api { kind, .. } => kind,
            Self::Refusal => "refusal",
            Self::MaxTokens => "max_tokens",
            Self::Network(_) => "network",
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Api { message, .. } => message.clone(),
            other => other.to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api {
                status,
                kind,
                message,
            } => write!(f, "{status} ({kind}): {message}"),
            Self::Network(message) => f.write_str(message),
            other => f.write_str(other.kind()),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Network(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Streams the response, calling `on_text` for every text delta. Stops early (returning
/// what was received so far) when `cancelled()` turns true. A refusal or truncated output is
/// an error, so a successful outcome is always complete.
pub async fn stream(
    provider: Provider<'_>,
    prompt: Prompt,
    on_text: impl FnMut(&str),
    cancelled: impl Fn() -> bool,
) -> Result<Outcome> {
    let outcome = match provider {
        Provider::Claude {
            api_key,
            model,
            effort,
        } => claude::stream(api_key, model, effort, &prompt, on_text, cancelled).await?,
        Provider::Compat {
            endpoint,
            api_key,
            model,
        } => openai_compat::stream(endpoint, api_key, model, &prompt, on_text, cancelled).await?,
    };
    finish(outcome)
}

/// Turns a refusal or a truncated answer into an error.
fn finish(outcome: Outcome) -> Result<Outcome> {
    match outcome.stop_reason.as_deref() {
        Some("refusal") => Err(Error::Refusal),
        Some("max_tokens") => Err(Error::MaxTokens),
        _ => Ok(outcome),
    }
}

/// Reads a streamed response body and hands each complete `separator`-terminated frame to
/// `on_frame`. Returns early, without error, once `cancelled()` turns true.
async fn read_frames(
    resp: reqwest::Response,
    separator: &[u8],
    cancelled: impl Fn() -> bool,
    mut on_frame: impl FnMut(&str) -> Result<()>,
) -> Result<()> {
    let mut bytes = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = bytes.next().await {
        if cancelled() {
            return Ok(());
        }
        buf.extend_from_slice(&chunk?);
        while let Some(frame) = next_frame(&mut buf, separator) {
            on_frame(&frame)?;
        }
    }
    Ok(())
}

/// Removes the first complete frame (with its separator) from `buf`. Frames are split on
/// bytes, so a multi-byte character cut between two chunks is never decoded half-way.
fn next_frame(buf: &mut Vec<u8>, separator: &[u8]) -> Option<String> {
    let pos = buf.windows(separator.len()).position(|w| w == separator)?;
    let frame: Vec<u8> = buf.drain(..pos + separator.len()).collect();
    Some(String::from_utf8_lossy(&frame).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_wait_for_their_separator() {
        let mut buf = b"data: a\n\ndata: b".to_vec();
        assert_eq!(
            next_frame(&mut buf, b"\n\n").as_deref(),
            Some("data: a\n\n")
        );
        assert_eq!(next_frame(&mut buf, b"\n\n"), None);
        buf.extend_from_slice(b"\n\n");
        assert_eq!(
            next_frame(&mut buf, b"\n\n").as_deref(),
            Some("data: b\n\n")
        );
        assert!(buf.is_empty());
    }

    fn stopped(reason: Option<&str>) -> Outcome {
        Outcome {
            stop_reason: reason.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn only_complete_answers_succeed() {
        assert!(matches!(
            finish(stopped(Some("refusal"))),
            Err(Error::Refusal)
        ));
        assert!(matches!(
            finish(stopped(Some("max_tokens"))),
            Err(Error::MaxTokens)
        ));
        assert!(finish(stopped(Some("end_turn"))).is_ok());
        // A cancelled stream has no stop reason and is not an error.
        assert!(finish(stopped(None)).is_ok());
    }

    #[test]
    fn error_kinds_match_the_ui_codes() {
        assert_eq!(
            Error::api(404, "not_found_error", "x").kind(),
            "not_found_error"
        );
        assert_eq!(Error::Refusal.kind(), "refusal");
        assert_eq!(Error::Refusal.message(), "refusal");
        assert_eq!(Error::Network("reset".into()).message(), "reset");
        assert_eq!(Error::MaxTokens.status(), None);
    }
}
