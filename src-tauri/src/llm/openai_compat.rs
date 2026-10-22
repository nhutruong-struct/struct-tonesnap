//! Chat Completions API with SSE streaming, shared by every provider that speaks the OpenAI
//! dialect: Gemini, Groq and OpenRouter. They differ only in the endpoint and in how they can be
//! told to answer with JSON.

use serde_json::{json, Value};

use super::{error_body, read_frames, Error, Outcome, Prompt, Read, Result, CLIENT};

/// How the provider is asked to answer with JSON that matches the output schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonMode {
    /// `response_format: json_schema`, enforced by the provider.
    Schema,
    /// `response_format: json_object` plus the schema written into the system prompt. For
    /// providers that cannot stream a `json_schema` answer (Groq) or only support it on some
    /// models (OpenRouter).
    Object,
}

#[derive(Debug)]
pub struct Endpoint {
    pub url: &'static str,
    pub json_mode: JsonMode,
    /// `reasoning_effort` to send, for providers whose default models think for a long time.
    pub reasoning_effort: Option<&'static str>,
}

pub static GEMINI: Endpoint = Endpoint {
    url: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
    json_mode: JsonMode::Schema,
    reasoning_effort: Some("low"),
};

pub static GROQ: Endpoint = Endpoint {
    url: "https://api.groq.com/openai/v1/chat/completions",
    json_mode: JsonMode::Object,
    reasoning_effort: None,
};

pub static OPENROUTER: Endpoint = Endpoint {
    url: "https://openrouter.ai/api/v1/chat/completions",
    json_mode: JsonMode::Object,
    reasoning_effort: None,
};

fn body(endpoint: &Endpoint, model: &str, prompt: &Prompt) -> Value {
    let mut system = prompt.system.to_string();
    let response_format = match endpoint.json_mode {
        JsonMode::Schema => json!({
            "type": "json_schema",
            "json_schema": { "name": "tone_suggestions", "strict": true, "schema": prompt.schema }
        }),
        JsonMode::Object => {
            system.push_str(
                "\n\nAnswer with one JSON object and nothing else (no code fences) that \
                 matches this JSON Schema:\n",
            );
            system.push_str(&prompt.schema.to_string());
            json!({ "type": "json_object" })
        }
    };
    let mut body = json!({
        "model": model,
        "stream": true,
        // Short structured output; the headroom covers a reasoning model's thinking.
        "max_tokens": 4000,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": prompt.user },
        ],
        "response_format": response_format,
    });
    if let Some(effort) = endpoint.reasoning_effort {
        body["reasoning_effort"] = json!(effort);
    }
    body
}

pub async fn stream(
    endpoint: &Endpoint,
    api_key: &str,
    model: &str,
    prompt: &Prompt,
    on_text: impl FnMut(&str),
    cancelled: impl Fn() -> bool,
) -> Result<Outcome> {
    stream_to(
        endpoint,
        endpoint.url,
        api_key,
        model,
        prompt,
        on_text,
        cancelled,
    )
    .await
}

/// `stream` against a given endpoint URL; tests point it at a local server.
async fn stream_to(
    endpoint: &Endpoint,
    url: &str,
    api_key: &str,
    model: &str,
    prompt: &Prompt,
    mut on_text: impl FnMut(&str),
    cancelled: impl Fn() -> bool,
) -> Result<Outcome> {
    let resp = CLIENT
        .post(url)
        .bearer_auth(api_key)
        .json(&body(endpoint, model, prompt))
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let raw = resp.text().await.unwrap_or_default();
        return Err(http_error(status.as_u16(), &raw));
    }

    let mut outcome = Outcome::default();
    // Servers end lines with "\n" or "\r\n"; every `data:` line holds one complete JSON chunk.
    let read = read_frames(resp, b"\n", cancelled, |line| {
        let Some(data) = line.trim().strip_prefix("data:") else {
            return Ok(());
        };
        let data = data.trim();
        if data == "[DONE]" {
            // The end marker also proves the answer is whole when no chunk named a reason.
            outcome.stop_reason.get_or_insert_with(|| "end_turn".into());
            return Ok(());
        }
        handle_chunk(&serde_json::from_str(data)?, &mut outcome, &mut on_text)
    })
    .await?;
    outcome.cancelled = read == Read::Cancelled;
    Ok(outcome)
}

/// Maps an error response onto the codes the popup translates (the Claude vocabulary), since
/// these providers each name their errors differently.
fn http_error(status: u16, raw: &str) -> Error {
    let (_, message) = error_body(raw);
    let kind = match status {
        // Gemini answers a wrong key with 400.
        400 if message.contains("API key not valid") => "authentication_error",
        401 => "authentication_error",
        402 | 403 => "permission_error",
        404 => "not_found_error",
        429 => "rate_limit_error",
        500..=599 => "overloaded_error",
        _ => "http_error",
    };
    Error::api(status, kind, message)
}

fn handle_chunk(data: &Value, outcome: &mut Outcome, on_text: &mut impl FnMut(&str)) -> Result<()> {
    // OpenRouter reports a failure after the stream has started as a chunk with `error`.
    if let Some(err) = data.get("error").filter(|e| !e.is_null()) {
        return Err(Error::api(
            0,
            "stream_error",
            err["message"].as_str().unwrap_or("stream error"),
        ));
    }
    if outcome.model.is_none() {
        outcome.model = data["model"].as_str().map(str::to_string);
    }
    if data["usage"].is_object() {
        outcome.usage = data["usage"].clone();
    }
    let choice = &data["choices"][0];
    if let Some(text) = choice["delta"]["content"]
        .as_str()
        .filter(|t| !t.is_empty())
    {
        outcome.text.push_str(text);
        on_text(text);
    }
    if let Some(reason) = choice["finish_reason"].as_str() {
        outcome.stop_reason = Some(stop_reason(reason).to_string());
    }
    Ok(())
}

/// Normalizes `finish_reason` to the Claude vocabulary `llm::finish` understands.
fn stop_reason(finish_reason: &str) -> &str {
    match finish_reason {
        "length" => "max_tokens",
        "content_filter" => "refusal",
        _ => "end_turn",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::test_server::{closed_port_url, Reply, TestServer};

    fn prompt() -> Prompt {
        Prompt {
            system: "system rules",
            user: "<selection>\nhi\n</selection>".into(),
            schema: json!({ "type": "object" }),
        }
    }

    fn sse(chunks: &[Value]) -> String {
        let mut out: String = chunks.iter().map(|c| format!("data: {c}\n\n")).collect();
        out.push_str("data: [DONE]\n\n");
        out
    }

    fn delta(text: &str) -> Value {
        json!({ "model": "m-1", "choices": [{ "index": 0, "delta": { "content": text } }] })
    }

    fn finish(reason: &str) -> Value {
        json!({ "choices": [{ "index": 0, "delta": {}, "finish_reason": reason }],
                "usage": { "prompt_tokens": 10, "completion_tokens": 5 } })
    }

    fn run(endpoint: &Endpoint, base: &str, cancelled: bool) -> (Result<Outcome>, Vec<String>) {
        let mut deltas = Vec::new();
        let url = format!("{base}/chat/completions");
        let result = tauri::async_runtime::block_on(stream_to(
            endpoint,
            &url,
            "test-key",
            "m-1",
            &prompt(),
            |t| deltas.push(t.to_string()),
            || cancelled,
        ));
        (result, deltas)
    }

    #[test]
    fn streams_text_and_collects_the_outcome() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[
                // A reasoning model's thinking arrives in another field and is not shown.
                json!({ "choices": [{ "delta": { "reasoning": "hmm" } }] }),
                delta("Chào "),
                delta("bạn"),
                finish("stop"),
            ]),
        )]);
        let (result, deltas) = run(&GEMINI, &server.url, false);
        let outcome = result.unwrap();
        assert_eq!(deltas, ["Chào ", "bạn"]);
        assert_eq!(outcome.text, "Chào bạn");
        assert_eq!(outcome.model.as_deref(), Some("m-1"));
        assert_eq!(outcome.stop_reason.as_deref(), Some("end_turn"));
        assert_eq!(
            outcome.usage,
            json!({ "prompt_tokens": 10, "completion_tokens": 5 })
        );

        let request = &server.finish()[0];
        assert_eq!(request.path, "/chat/completions");
        assert_eq!(request.header("authorization"), Some("Bearer test-key"));
        let body = &request.body;
        assert_eq!(body["model"], "m-1");
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "system rules");
        assert_eq!(body["messages"][1]["content"], prompt().user);
        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(body["response_format"]["json_schema"]["strict"], true);
        assert_eq!(
            body["response_format"]["json_schema"]["schema"],
            prompt().schema
        );
        assert_eq!(body["reasoning_effort"], "low");
    }

    #[test]
    fn object_mode_puts_the_schema_in_the_system_prompt() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[delta("{}"), finish("stop")]),
        )]);
        let (result, _) = run(&GROQ, &server.url, false);
        assert_eq!(result.unwrap().text, "{}");
        let body = &server.finish()[0].body;
        assert_eq!(body["response_format"], json!({ "type": "json_object" }));
        let system = body["messages"][0]["content"].as_str().unwrap();
        assert!(system.starts_with("system rules"));
        assert!(system.contains(r#"{"type":"object"}"#));
        assert!(body.get("reasoning_effort").is_none());
    }

    #[test]
    fn every_provider_streams_with_room_for_a_reasoning_model() {
        for endpoint in [&GEMINI, &GROQ, &OPENROUTER] {
            let body = body(endpoint, "m-1", &prompt());
            assert_eq!(body["stream"], true, "{}", endpoint.url);
            assert_eq!(body["max_tokens"], 4000, "{}", endpoint.url);
        }
    }

    #[test]
    fn crlf_line_endings_are_understood() {
        let crlf = sse(&[delta("ok"), finish("stop")]).replace('\n', "\r\n");
        let server = TestServer::start(vec![Reply::stream("text/event-stream", &crlf)]);
        let (result, deltas) = run(&OPENROUTER, &server.url, false);
        assert_eq!(result.unwrap().text, "ok");
        assert_eq!(deltas, ["ok"]);
    }

    #[test]
    fn openrouter_comments_are_ignored() {
        let body = format!(": OPENROUTER PROCESSING\n\n{}", sse(&[delta("ok")]));
        let server = TestServer::start(vec![Reply::stream("text/event-stream", &body)]);
        let (result, _) = run(&OPENROUTER, &server.url, false);
        assert_eq!(result.unwrap().text, "ok");
    }

    #[test]
    fn finish_reasons_use_the_claude_vocabulary() {
        assert_eq!(stop_reason("stop"), "end_turn");
        assert_eq!(stop_reason("length"), "max_tokens");
        assert_eq!(stop_reason("content_filter"), "refusal");
    }

    #[test]
    fn http_errors_map_to_the_ui_codes() {
        for (status, body, kind) in [
            (
                401,
                r#"{"error":{"message":"bad key"}}"#,
                "authentication_error",
            ),
            (
                400,
                r#"[{"error":{"code":400,"message":"API key not valid. Please pass a valid API key."}}]"#,
                "authentication_error",
            ),
            (
                403,
                r#"{"error":{"message":"no access"}}"#,
                "permission_error",
            ),
            (
                404,
                r#"{"error":{"message":"no such model"}}"#,
                "not_found_error",
            ),
            (
                429,
                r#"{"error":{"message":"slow down"}}"#,
                "rate_limit_error",
            ),
            (
                503,
                r#"[{"error":{"message":"overloaded"}}]"#,
                "overloaded_error",
            ),
            (400, r#"{"error":{"message":"bad request"}}"#, "http_error"),
        ] {
            let server = TestServer::start(vec![Reply::json(status, body)]);
            let err = run(&GEMINI, &server.url, false).0.unwrap_err();
            assert_eq!(err.kind(), kind, "status {status}");
            assert_eq!(err.status(), Some(status));
        }
    }

    #[test]
    fn the_error_message_comes_from_the_body() {
        let server = TestServer::start(vec![Reply::json(
            429,
            r#"[{"error":{"code":429,"message":"Quota exceeded","status":"RESOURCE_EXHAUSTED"}}]"#,
        )]);
        let err = run(&GEMINI, &server.url, false).0.unwrap_err();
        assert_eq!(err.message(), "Quota exceeded");
    }

    #[test]
    fn errors_without_a_json_body_keep_the_raw_text() {
        let server = TestServer::start(vec![Reply::json(502, "Bad Gateway")]);
        let err = run(&GROQ, &server.url, false).0.unwrap_err();
        assert_eq!(err.kind(), "overloaded_error");
        assert_eq!(err.message(), "Bad Gateway");
    }

    #[test]
    fn an_error_chunk_in_the_stream_fails_the_request() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[
                delta("partial"),
                json!({ "error": { "code": 502, "message": "upstream died" } }),
            ]),
        )]);
        let (result, deltas) = run(&OPENROUTER, &server.url, false);
        let err = result.unwrap_err();
        assert_eq!(deltas, ["partial"]);
        assert_eq!(err.kind(), "stream_error");
        assert_eq!(err.status(), Some(0));
        assert_eq!(err.message(), "upstream died");
    }

    #[test]
    fn a_cancelled_request_stops_quietly() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[delta("never shown")]),
        )]);
        let (result, deltas) = run(&GEMINI, &server.url, true);
        assert!(deltas.is_empty());
        let outcome = result.unwrap();
        assert_eq!(outcome.text, "");
        assert!(outcome.cancelled);
    }

    #[test]
    fn the_done_marker_ends_an_answer_without_a_finish_reason() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &sse(&[delta("{}")]),
        )]);
        let (result, _) = run(&GROQ, &server.url, false);
        assert_eq!(result.unwrap().stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn a_stream_cut_before_the_end_has_no_stop_reason() {
        let server = TestServer::start(vec![Reply::stream(
            "text/event-stream",
            &format!("data: {}\n\n", delta("{\"variants\":")),
        )]);
        let (result, _) = run(&GEMINI, &server.url, false);
        let outcome = result.unwrap();
        assert_eq!(outcome.stop_reason, None);
        assert!(matches!(
            super::super::finish(outcome),
            Err(Error::Incomplete)
        ));
    }

    #[test]
    fn an_unreachable_api_is_a_network_error() {
        let (result, _) = run(&GEMINI, &closed_port_url(), false);
        assert_eq!(result.unwrap_err().kind(), "network");
    }
}
