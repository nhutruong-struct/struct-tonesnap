//! Model providers behind one streaming interface: the Claude API (paid) and the providers that
//! speak the OpenAI dialect (Gemini, Groq, OpenRouter; free tiers). All receive the same system
//! prompt and JSON output schema.

mod claude;
mod openai_compat;
#[cfg(test)]
mod test_server;

use std::sync::LazyLock;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::Value;

pub use openai_compat::{Endpoint, GEMINI, GROQ, OPENROUTER};

/// How long a response may send nothing before it counts as stalled. Thinking models stay well
/// under it: Claude sends pings while it thinks, and the other providers get a low reasoning
/// effort.
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// One client for every provider, so connections and TLS sessions are reused.
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .read_timeout(IDLE_TIMEOUT)
        .build()
        .expect("http client")
});

/// Error details from a provider's error body: Claude, OpenAI, OpenRouter and Gemini (which
/// wraps the error in an array) all put them under `error`. `message` falls back to the raw body.
fn error_body(raw: &str) -> (Option<String>, String) {
    let parsed: Value = serde_json::from_str(raw).unwrap_or(Value::Null);
    let error = &parsed.get(0).unwrap_or(&parsed)["error"];
    let message = error["message"]
        .as_str()
        .map_or_else(|| raw.to_string(), str::to_string);
    (error["type"].as_str().map(str::to_string), message)
}

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
    /// The caller cancelled the request before the answer ended.
    pub cancelled: bool,
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
    /// The stream ended before the model finished its answer, e.g. a proxy closed the
    /// connection.
    Incomplete,
    /// The provider could not be reached in time, or stopped sending in the middle.
    Timeout,
    /// The provider sent something that is not the JSON its API promises.
    BadResponse(String),
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
            Self::Incomplete => "incomplete",
            Self::Timeout => "timeout",
            Self::BadResponse(_) => "bad_response",
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
            Self::Network(message) | Self::BadResponse(message) => f.write_str(message),
            other => f.write_str(other.kind()),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else {
            Self::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::BadResponse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Streams the response, calling `on_text` for every text delta. Stops early (returning
/// what was received so far, marked `cancelled`) when `cancelled()` turns true. A refusal, a
/// truncated output or a stream that ends early is an error, so a successful outcome that was
/// not cancelled is always complete.
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

/// Turns a refusal, a truncated answer or an answer without an end into an error. Only the
/// provider's stop reason proves the answer is whole: a connection can also close cleanly in
/// the middle of it.
fn finish(outcome: Outcome) -> Result<Outcome> {
    if outcome.cancelled {
        return Ok(outcome);
    }
    match outcome.stop_reason.as_deref() {
        Some("refusal") => Err(Error::Refusal),
        Some("max_tokens") => Err(Error::MaxTokens),
        Some(_) => Ok(outcome),
        None => Err(Error::Incomplete),
    }
}

/// How reading a streamed body ended.
#[derive(Debug, PartialEq, Eq)]
enum Read {
    /// The server closed the stream.
    Ended,
    /// `cancelled()` turned true first.
    Cancelled,
}

/// Reads a streamed response body and hands each complete `separator`-terminated frame to
/// `on_frame`. Returns early, without error, once `cancelled()` turns true.
async fn read_frames(
    resp: reqwest::Response,
    separator: &[u8],
    cancelled: impl Fn() -> bool,
    mut on_frame: impl FnMut(&str) -> Result<()>,
) -> Result<Read> {
    let mut bytes = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = bytes.next().await {
        if cancelled() {
            return Ok(Read::Cancelled);
        }
        buf.extend_from_slice(&chunk?);
        while let Some(frame) = next_frame(&mut buf, separator) {
            on_frame(&frame)?;
        }
    }
    Ok(Read::Ended)
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
    }

    #[test]
    fn a_stream_that_ends_without_a_stop_reason_is_incomplete() {
        assert!(matches!(finish(stopped(None)), Err(Error::Incomplete)));
    }

    #[test]
    fn a_cancelled_stream_is_not_an_error() {
        let cancelled = Outcome {
            cancelled: true,
            ..Default::default()
        };
        assert!(finish(cancelled).is_ok());
    }

    #[test]
    fn error_bodies_give_their_type_and_message() {
        let claude = r#"{"type":"error","error":{"type":"overloaded_error","message":"busy"}}"#;
        assert_eq!(
            error_body(claude),
            (Some("overloaded_error".into()), "busy".into())
        );
        let gemini = r#"[{"error":{"code":400,"message":"API key not valid"}}]"#;
        assert_eq!(error_body(gemini), (None, "API key not valid".into()));
        assert_eq!(error_body("Bad Gateway"), (None, "Bad Gateway".into()));
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
        assert_eq!(Error::Incomplete.kind(), "incomplete");
        assert_eq!(Error::Timeout.kind(), "timeout");
        let bad = Error::from(serde_json::from_str::<Value>("{").unwrap_err());
        assert_eq!(bad.kind(), "bad_response");
    }
}
