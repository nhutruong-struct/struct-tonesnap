//! Sends a session to the configured model and streams the answer to the popup as
//! `request-*` events.

use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::llm;
use crate::prompt;
use crate::secrets;
use crate::session::Session;
use crate::settings::{ProviderKind, Settings};
use crate::state::state;
use crate::window::POPUP;

/// Payload of every `request-*` event; fields that do not apply to an event stay `None`.
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestEvent {
    session_id: u64,
    request_id: u64,
    text: Option<String>,
    stop_reason: Option<String>,
    model: Option<String>,
    first_token_ms: Option<u128>,
    total_ms: Option<u128>,
    usage: Option<serde_json::Value>,
    error_kind: Option<String>,
    error_status: Option<u16>,
    error_message: Option<String>,
}

impl RequestEvent {
    fn new(session_id: u64, request_id: u64) -> Self {
        Self {
            session_id,
            request_id,
            ..Default::default()
        }
    }

    fn failed(self, err: &llm::Error) -> Self {
        Self {
            error_kind: Some(err.kind().to_string()),
            error_status: err.status(),
            error_message: Some(err.message()),
            ..self
        }
    }
}

/// Starts a request for `session`, superseding any running one. Returns at once; the request
/// runs on the async runtime.
pub fn start(app: &AppHandle, session: Session) {
    let st = state(app);
    let request_id = st.begin_request();
    let settings = st.settings();
    let _ = app.emit_to(
        POPUP,
        "request-started",
        RequestEvent::new(session.id, request_id),
    );
    let app = app.clone();
    tauri::async_runtime::spawn(async move { run(&app, &settings, &session, request_id).await });
}

async fn run(app: &AppHandle, settings: &Settings, session: &Session, request_id: u64) {
    let ids = RequestEvent::new(session.id, request_id);
    let provider_kind = settings.provider;
    // The credential store is blocking I/O.
    let key = tauri::async_runtime::spawn_blocking(move || secrets::api_key(provider_kind))
        .await
        .ok()
        .flatten();
    let Some(api_key) = key else {
        let ev = RequestEvent {
            error_kind: Some("no_api_key".into()),
            ..ids
        };
        let _ = app.emit_to(POPUP, "request-failed", ev);
        return;
    };

    let started = Instant::now();
    let st = state(app);
    let mut first_token_ms = None;
    let result = llm::stream(
        provider(settings, &api_key),
        llm::Prompt {
            system: prompt::SYSTEM_PROMPT,
            user: prompt::user_turn(&session.payload),
            schema: prompt::output_schema(&session.payload.tones),
        },
        |text| {
            first_token_ms.get_or_insert_with(|| started.elapsed().as_millis());
            let ev = RequestEvent {
                text: Some(text.to_string()),
                ..ids.clone()
            };
            let _ = app.emit_to(POPUP, "request-delta", ev);
        },
        || !st.is_current_request(request_id),
    )
    .await;

    let timed = RequestEvent {
        first_token_ms,
        total_ms: Some(started.elapsed().as_millis()),
        ..ids
    };
    match result {
        Ok(outcome) => {
            let ev = RequestEvent {
                stop_reason: outcome.stop_reason,
                model: outcome.model,
                usage: Some(outcome.usage),
                ..timed
            };
            let _ = app.emit_to(POPUP, "request-done", ev);
        }
        Err(err) => {
            let _ = app.emit_to(POPUP, "request-failed", timed.failed(&err));
        }
    }
}

fn provider<'a>(settings: &'a Settings, api_key: &'a str) -> llm::Provider<'a> {
    match settings.provider {
        ProviderKind::Claude => llm::Provider::Claude {
            api_key,
            model: &settings.model,
            effort: &settings.effort,
        },
        ProviderKind::Gemini => compat(&llm::GEMINI, api_key, &settings.gemini_model),
        ProviderKind::Groq => compat(&llm::GROQ, api_key, &settings.groq_model),
        ProviderKind::Openrouter => compat(&llm::OPENROUTER, api_key, &settings.openrouter_model),
    }
}

fn compat<'a>(
    endpoint: &'static llm::Endpoint,
    api_key: &'a str,
    model: &'a str,
) -> llm::Provider<'a> {
    llm::Provider::Compat {
        endpoint,
        api_key,
        model,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// Field names of `RequestEvent` in `src/lib/api.ts`.
    const TS_FIELDS: [&str; 11] = [
        "sessionId",
        "requestId",
        "text",
        "stopReason",
        "model",
        "firstTokenMs",
        "totalMs",
        "usage",
        "errorKind",
        "errorStatus",
        "errorMessage",
    ];

    #[test]
    fn events_serialize_with_the_typescript_field_names() {
        let value = serde_json::to_value(RequestEvent::new(3, 7)).unwrap();
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let mut expected = TS_FIELDS;
        expected.sort_unstable();
        assert_eq!(keys, expected);
        assert_eq!(value["sessionId"], 3);
        assert_eq!(value["requestId"], 7);
    }

    #[test]
    fn failures_carry_the_error_code_status_and_message() {
        let api = llm::Error::Api {
            status: 429,
            kind: "rate_limit_error".into(),
            message: "slow down".into(),
        };
        let value = serde_json::to_value(RequestEvent::new(1, 2).failed(&api)).unwrap();
        assert_eq!(value["errorKind"], "rate_limit_error");
        assert_eq!(value["errorStatus"], 429);
        assert_eq!(value["errorMessage"], "slow down");

        let value =
            serde_json::to_value(RequestEvent::new(1, 2).failed(&llm::Error::Refusal)).unwrap();
        assert_eq!(value["errorKind"], "refusal");
        assert_eq!(value["errorStatus"], json!(null));
    }

    #[test]
    fn the_provider_follows_the_settings() {
        let default = Settings::default();
        assert!(matches!(
            provider(&default, "key"),
            llm::Provider::Compat {
                endpoint,
                api_key: "key",
                model: "gemini-3.8-flash",
            } if endpoint.url == llm::GEMINI.url
        ));
        let groq = Settings {
            provider: ProviderKind::Groq,
            ..Settings::default()
        };
        assert!(matches!(
            provider(&groq, "key"),
            llm::Provider::Compat { endpoint, .. } if endpoint.url == llm::GROQ.url
        ));
        let openrouter = Settings {
            provider: ProviderKind::Openrouter,
            ..Settings::default()
        };
        assert!(matches!(
            provider(&openrouter, "key"),
            llm::Provider::Compat { endpoint, .. } if endpoint.url == llm::OPENROUTER.url
        ));
        let claude = Settings {
            provider: ProviderKind::Claude,
            effort: "high".into(),
            ..Settings::default()
        };
        assert!(matches!(
            provider(&claude, "key"),
            llm::Provider::Claude {
                api_key: "key",
                effort: "high",
                ..
            }
        ));
    }
}
