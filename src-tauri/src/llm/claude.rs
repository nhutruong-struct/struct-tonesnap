//! Claude Messages API over raw HTTP with SSE streaming (Rust has no official SDK).

use serde_json::{json, Value};

use super::{error_body, read_frames, Error, Outcome, Prompt, Read, Result, CLIENT};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

/// Haiku 4.5 rejects `effort`; every newer model accepts it.
fn supports_effort(model: &str) -> bool {
    !model.starts_with("claude-haiku-4-5")
}

/// Models that accept the `fallbacks: "default"` refusal fallback on the Claude API.
fn supports_default_fallbacks(model: &str) -> bool {
    matches!(
        model,
        "claude-opus-5-5" | "claude-opus-5" | "claude-sonnet-5-5"
    )
}

fn body(model: &str, effort: &str, prompt: &Prompt) -> Value {
    let mut output_config = json!({
        "format": { "type": "json_schema", "schema": prompt.schema }
    });
    if supports_effort(model) {
        output_config["effort"] = json!(effort);
    }
    let mut body = json!({
        "model": model,
        // Short structured output; the headroom covers adaptive thinking at low effort.
        "max_tokens": 8000,
        "stream": true,
        "system": [{
            "type": "text",
            "text": prompt.system,
            "cache_control": { "type": "ephemeral" }
        }],
        "messages": [{ "role": "user", "content": prompt.user }],
        "output_config": output_config
    });
    if supports_default_fallbacks(model) {
        body["fallbacks"] = json!("default");
    }
    body
}

pub async fn stream(
    api_key: &str,
    model: &str,
    effort: &str,
    prompt: &Prompt,
    on_text: impl FnMut(&str),
    cancelled: impl Fn() -> bool,
) -> Result<Outcome> {
    stream_to(API_URL, api_key, model, effort, prompt, on_text, cancelled).await
}

/// `stream` against a given Messages endpoint; tests point it at a local server.
async fn stream_to(
    url: &str,
    api_key: &str,
    model: &str,
    effort: &str,
    prompt: &Prompt,
    mut on_text: impl FnMut(&str),
    cancelled: impl Fn() -> bool,
) -> Result<Outcome> {
    let mut http = CLIENT
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json");
    if supports_default_fallbacks(model) {
        http = http.header("anthropic-beta", FALLBACK_BETA);
    }
    let resp = http.json(&body(model, effort, prompt)).send().await?;

    let status = resp.status();
    if !status.is_success() {
        let raw = resp.text().await.unwrap_or_default();
        let (kind, message) = error_body(&raw);
        return Err(Error::api(
            status.as_u16(),
            kind.as_deref().unwrap_or("http_error"),
            message,
        ));
    }

    let mut outcome = Outcome::default();
    // SSE events are separated by a blank line.
    let read = read_frames(resp, b"\n\n", cancelled, |event| {
        for line in event.lines() {
            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data: Value = serde_json::from_str(data.trim())?;
            handle_event(&data, &mut outcome, &mut on_text)?;
        }
        Ok(())
    })
    .await?;
    outcome.cancelled = read == Read::Cancelled;
    Ok(outcome)
}

fn handle_event(data: &Value, outcome: &mut Outcome, on_text: &mut impl FnMut(&str)) -> Result<()> {
    match data["type"].as_str().unwrap_or_default() {
        "message_start" => {
            outcome.model = data["message"]["model"].as_str().map(str::to_string);
            outcome.usage = data["message"]["usage"].clone();
        }
        "content_block_delta" => {
            if data["delta"]["type"] == "text_delta" {
                if let Some(text) = data["delta"]["text"].as_str() {
                    outcome.text.push_str(text);
                    on_text(text);
                }
            }
        }
        "message_delta" => {
            if let Some(reason) = data["delta"]["stop_reason"].as_str() {
                outcome.stop_reason = Some(reason.to_string());
            }
            if let Some(usage) = data["usage"].as_object() {
                for (k, v) in usage {
                    outcome.usage[k] = v.clone();
                }
            }
        }
        "error" => {
            let err = &data["error"];
            return Err(Error::api(
                0,
                err["type"].as_str().unwrap_or("stream_error"),
                err["message"].as_str().unwrap_or("stream error"),
            ));
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::llm::test_server::{closed_port_url, Reply, TestServer};

    fn prompt() -> Prompt {
        Prompt {
            system: "system rules",
            user: "<selection>\nhi\n</selection>".into(),
            schema: json!({ "type": "object" }),
        }
    }

    fn sse(events: &[Value]) -> String {
        events
            .iter()
            .map(|e| format!("event: {}\ndata: {e}\n\n", e["type"].as_str().unwrap()))
            .collect()
    }

    fn run(base: &str, model: &str, cancelled: bool) -> (Result<Outcome>, Vec<String>) {
        let mut deltas = Vec::new();
        let url = format!("{base}/v1/messages");
        let result = tauri::async_runtime::block_on(stream_to(
            &url,
            "test-key",
            model,
            "low",
            &prompt(),
            |t| deltas.push(t.to_string()),
            || cancelled,
        ));
        (result, deltas)
    }

    fn text_delta(text: &str) -> Value {
        json!({ "type": "content_block_delta", "index": 0,
                "delta": { "type": "text_delta", "text": text } })
    }

    #[test]
    fn streams_text_and_collects_the_outcome() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[
                json!({ "type": "message_start", "message": {
                    "model": "claude-opus-5-5", "usage": { "input_tokens": 10 } } }),
                json!({ "type": "content_block_start", "index": 0,
                        "content_block": { "type": "text", "text": "" } }),
                json!({ "type": "content_block_delta", "index": 0,
                        "delta": { "type": "thinking_delta", "thinking": "hmm" } }),
                text_delta("Chào "),
                text_delta("bạn"),
                json!({ "type": "message_delta", "delta": { "stop_reason": "end_turn" },
                        "usage": { "output_tokens": 5 } }),
                json!({ "type": "message_stop" }),
            ]),
        )]);
        let (result, deltas) = run(&server.url, "claude-opus-5-5", false);
        let outcome = result.unwrap();
        assert_eq!(deltas, ["Chào ", "bạn"]);
        assert_eq!(outcome.text, "Chào bạn");
        assert_eq!(outcome.model.as_deref(), Some("claude-opus-5-5"));
        assert_eq!(outcome.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(
            outcome.usage,
            json!({ "input_tokens": 10, "output_tokens": 5 })
        );

        let requests = server.finish();
        let request = &requests[0];
        assert_eq!(request.path, "/v1/messages");
        assert_eq!(request.header("x-api-key"), Some("test-key"));
        assert_eq!(request.header("anthropic-version"), Some("2023-06-01"));
        assert_eq!(request.header("anthropic-beta"), Some(FALLBACK_BETA));
        let body = &request.body;
        assert_eq!(body["model"], "claude-opus-5-5");
        assert_eq!(body["stream"], true);
        assert_eq!(body["system"][0]["text"], "system rules");
        assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
        assert_eq!(body["messages"][0]["content"], prompt().user);
        assert_eq!(body["output_config"]["effort"], "low");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], prompt().schema);
        assert_eq!(body["fallbacks"], "default");
    }

    #[test]
    fn only_the_models_that_accept_it_get_the_default_fallbacks() {
        for model in ["claude-opus-5-5", "claude-opus-5", "claude-sonnet-5-5"] {
            assert!(supports_default_fallbacks(model), "{model}");
        }
        for model in ["claude-haiku-4-5", "claude-sonnet-4-5", "some-other-model"] {
            assert!(!supports_default_fallbacks(model), "{model}");
        }
        assert!(!supports_effort("claude-haiku-4-5"));
        assert!(supports_effort("claude-sonnet-5-5"));
    }

    #[test]
    fn the_request_leaves_room_for_thinking_and_caches_the_system_prompt() {
        let body = body("claude-sonnet-5-5", "medium", &prompt());
        assert_eq!(body["max_tokens"], 8000);
        assert_eq!(body["output_config"]["effort"], "medium");
        assert_eq!(body["system"][0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn haiku_gets_neither_effort_nor_fallbacks() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[text_delta("ok")]),
        )]);
        let (result, _) = run(&server.url, "claude-haiku-4-5", false);
        assert_eq!(result.unwrap().text, "ok");
        let request = &server.finish()[0];
        assert_eq!(request.header("anthropic-beta"), None);
        assert!(request.body["output_config"].get("effort").is_none());
        assert!(request.body.get("fallbacks").is_none());
    }

    #[test]
    fn http_errors_keep_the_api_error_type() {
        let server = TestServer::start(vec![Reply::json(
            401,
            r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#,
        )]);
        let (result, _) = run(&server.url, "claude-opus-5-5", false);
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "authentication_error");
        assert_eq!(err.status(), Some(401));
        assert_eq!(err.message(), "invalid x-api-key");
    }

    #[test]
    fn errors_without_a_json_body_are_http_errors() {
        let server = TestServer::start(vec![Reply::json(502, "Bad Gateway")]);
        let (result, _) = run(&server.url, "claude-opus-5-5", false);
        let err = result.unwrap_err();
        assert_eq!(err.kind(), "http_error");
        assert_eq!(err.status(), Some(502));
        assert_eq!(err.message(), "Bad Gateway");
    }

    #[test]
    fn error_events_in_the_stream_fail_the_request() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[
                text_delta("partial"),
                json!({ "type": "error", "error": {
                    "type": "overloaded_error", "message": "Overloaded" } }),
            ]),
        )]);
        let (result, deltas) = run(&server.url, "claude-opus-5-5", false);
        let err = result.unwrap_err();
        assert_eq!(deltas, ["partial"]);
        assert_eq!(err.kind(), "overloaded_error");
        assert_eq!(err.status(), Some(0));
    }

    #[test]
    fn a_cancelled_request_stops_quietly() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[text_delta("never shown")]),
        )]);
        let (result, deltas) = run(&server.url, "claude-opus-5-5", true);
        assert!(deltas.is_empty());
        let outcome = result.unwrap();
        assert_eq!(outcome.text, "");
        assert!(outcome.cancelled);
    }

    #[test]
    fn a_stream_cut_before_the_stop_reason_has_none() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[text_delta("{\"variants\":{\"formal\":")]),
        )]);
        let (result, _) = run(&server.url, "claude-opus-5-5", false);
        let outcome = result.unwrap();
        assert_eq!(outcome.stop_reason, None);
        assert!(!outcome.cancelled);
        assert!(matches!(
            super::super::finish(outcome),
            Err(Error::Incomplete)
        ));
    }

    #[test]
    fn a_body_that_is_not_json_is_a_bad_response() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            "data: <html>proxy login</html>\n\n",
        )]);
        let (result, _) = run(&server.url, "claude-opus-5-5", false);
        assert_eq!(result.unwrap_err().kind(), "bad_response");
    }

    #[test]
    fn an_unreachable_api_is_a_network_error() {
        let (result, _) = run(&closed_port_url(), "claude-opus-5-5", false);
        assert_eq!(result.unwrap_err().kind(), "network");
    }
}
