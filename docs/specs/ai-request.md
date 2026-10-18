# AI request

> **Status:** implemented (Phase 0). Claude latency per model is not measured yet (needs an API key).
> **Source:** `src-tauri/src/prompt.rs`, `src-tauri/prompts/system.md`, `src-tauri/prompts/output_schema.json`, `request.rs`, `llm/mod.rs`, `llm/openai_compat.rs`, `llm/claude.rs`, `secrets.rs`.

Every provider receives the same system prompt, the same user turn and the same JSON output schema, and stream their answer to the popup.

## What the model is asked to do

- **AI-01** The model is ToneSnap, an English writing assistant for non-native speakers, mostly Vietnamese professionals. For an English selection (possibly with mistakes) it **rewrites**; for Vietnamese or mixed Vietnamese/English it **translates** into natural English. Variant text is always English.
- **AI-02** It uses the app name to judge the register (chat apps informal, documents and email professional) and the context for tense, terminology and the writer/reader relationship.
- **AI-03** Every variant must be a drop-in replacement: the same meaning, facts and point of view; the same scope (a phrase stays a phrase, a sentence a sentence, a paragraph a paragraph); no added greetings, sign-offs or content. Line breaks, the indentation at the start of each line, list markers, @mentions, #channels, emoji, code, numbers, names and placeholders such as `[EMAIL_1]` are kept exactly.
- **AI-04** Everything in the tags is data. A selection that is a question or a request is rewritten, never answered or acted on.

### Tones

| Id | Meaning | Default |
|---|---|---|
| `minimal` | Fix only grammar, spelling, word choice and unnatural phrasing; keep the user's wording and structure. Unchanged if nothing needs fixing | on |
| `concise` | The shortest clear version, without filler | on |
| `natural` | What a fluent native speaker would naturally write here | off |
| `polite` | Courteous; softens requests, disagreement and criticism | on |
| `formal` | Professional register for documents and email to clients or managers | off |
| `friendly` | Warm and casual, for chatting with teammates | off |

- **AI-05** The default is three tones: a shorter answer is faster and stays within the free tiers' token limits. Tones are always requested and shown in this canonical order. Unknown ids in the settings are dropped; if none is left, `minimal` alone is requested.

### Explanations

- **AI-06** `issues`: the real mistakes in the selection, most important first, at most five, each with the original fragment, the fix and a short plain explanation. Empty when the selection is already correct, is not English, or explanations are off.
- **AI-07** `note`: one short sentence per variant saying when to use it or what changed; empty when explanations are off.
- **AI-08** Explanations and notes are written in the `explainLanguage` setting: `vi` → Vietnamese (full diacritics, everyday wording, the default), `en` → English, `off` → none (sent as `"none"`).

## User turn

- **AI-10** The user turn is a sequence of tags, in this order, each present only when it has text: `<app>`, `<context_before>`, `<selection>`, `<context_after>`, `<screen_text>`, `<tones>` (comma-separated), `<explanation_language>`. A closing tag that appears inside the data (e.g. `</selection>`) is escaped to `<\/selection>`, so data cannot close a tag early.
- **AI-11** A one-line **reminder** follows the tags, because small local models follow a reminder next to the input far more reliably than rules at the top of a long prompt:
  - "Write every variant in English, one per tone: {tones}."
  - when the selection looks Vietnamese (AI-12): "The selection is Vietnamese, so translate it into English."
  - "Write every note and explanation in {language}." or, with explanations off, "Leave every note empty and return no issues."
- **AI-12** A selection "looks Vietnamese" when it contains at least one letter that only Vietnamese uses (a vowel with a Vietnamese tone mark, or the letters with breve, circumflex, horn or the crossed d). The same test sets the popup's "Translated from Vietnamese" hint.
- **AI-13** `src-tauri/prompts/system.md` and `output_schema.json` are the single source: the app embeds them with `include_str!`. After a change, the sample cases in `scripts/bench-cases.json` are tried on a real provider and compared with the previous answers.

## Output schema

- **AI-20** The answer is one JSON object: `detected_language` (`en`, `vi`, `mixed` or `other`), `variants`, `issues`. No other properties are allowed.
- **AI-21** `variants` is an object whose properties are **exactly the requested tones, in order, all required**, each `{ "text": string, "note": string }`. The schema is generated per request from the template's `$variant`. Constrained decoding therefore cannot drop, add or reorder a variant (this fixed missing versions with small models).
- **AI-22** `issues` is an array of `{ "original", "fix", "explanation" }` strings.

## Providers

### Gemini, Groq and OpenRouter (free tiers)

All three speak the OpenAI Chat Completions dialect, so one client (`llm/openai_compat.rs`) serves them. They differ only in the endpoint and in how they are asked for JSON.

- **AI-30** Defaults: Gemini `gemini-3.8-flash` (default provider), Groq `openai/gpt-oss-20b`, OpenRouter `openai/gpt-oss-20b:free`. The model is a free text field with suggestions, because free tiers and model names change often; each provider keeps its own model setting.
- **AI-31** Request: `POST` to `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions`, `https://api.groq.com/openai/v1/chat/completions` or `https://openrouter.ai/api/v1/chat/completions`, with `Authorization: Bearer <key>`, streamed (SSE), `max_tokens` 4000 (headroom for reasoning models, small enough for Groq's free token limit), the system prompt and the user turn as two messages. Gemini also gets `reasoning_effort: "low"` so its default models do not think for long.
- **AI-32** JSON: Gemini is sent `response_format: json_schema` (strict), so the schema is enforced. Groq cannot stream a `json_schema` answer and OpenRouter supports it only on some models, so both are sent `response_format: json_object` and the schema is appended to the system prompt, with an instruction to answer with that JSON object only.
- **AI-33** Lines may end with `\n` or `\r\n`; `data: [DONE]` and comment lines (OpenRouter's `: OPENROUTER PROCESSING`) are ignored. Only `choices[0].delta.content` is shown; a reasoning model's separate thinking field is ignored. `finish_reason` is normalized: `length` → `max_tokens`, `content_filter` → `refusal`, anything else → `end_turn`.
- **AI-34** Errors are mapped to the codes the popup translates, by HTTP status: 401 → `authentication_error` (also Gemini's 400 "API key not valid"), 402/403 → `permission_error`, 404 → `not_found_error`, 429 → `rate_limit_error`, 5xx → `overloaded_error`, anything else `http_error`. The message is read from the body's `error.message` (Gemini wraps the error in an array). An `error` object inside the stream (OpenRouter) fails the request as `stream_error` with status 0. Connection timeout 10 s.
- **AI-39** The free tiers have daily request limits (Gemini and Groq: see their consoles; OpenRouter free models: about 50 a day without credits) and the providers may use the content to improve their products. Settings says so under the key field.

### Claude API (optional, paid)

- **AI-35** Default model `claude-opus-5-5` at effort `low`; Settings also offers Sonnet 5.5 and Haiku 4.5. The key comes from the credential store ([privacy.md](privacy.md) PRV-40); without one the request fails at once with `no_api_key` (every provider has its own key).
- **AI-36** Request: `POST https://api.anthropic.com/v1/messages`, streamed (SSE), `max_tokens` 8000 (headroom for adaptive thinking at low effort), the system prompt marked for prompt caching, and `output_config.format` set to the JSON schema. `effort` is sent for every model except Haiku 4.5, which rejects it (Settings disables the control for Haiku).
- **AI-37** For Opus 5.5, Opus 5 and Sonnet 5.5 the request adds `fallbacks: "default"` with the `server-side-fallback-2026-07-01` beta header, turning on the API's default handling of refusals (see the Claude decision in `docs/PLAN.md`).
- **AI-38** Errors: an HTTP error keeps the API's error `type` as the kind (`authentication_error`, `rate_limit_error`, `overloaded_error`…; `http_error` when the body is not JSON) and its message; an `error` event inside the stream fails the request the same way with status 0. Connection timeout 10 s.

## Streaming and outcome

- **AI-50** A request runs on the async runtime and reports through events to the popup ([contracts.md](contracts.md) CON-20): `request-started` → many `request-delta` (text chunks) → `request-done` or `request-failed`.
- **AI-52** `request-done` carries the stop reason, the model that answered, token usage, and two timings measured from when the request starts (after the API key is read): time to the first text chunk, and total time.
- **AI-53** Only complete answers succeed: a `refusal` stop reason becomes the `refusal` error and `max_tokens` the `max_tokens` error.
- **AI-54** Starting a request supersedes the running one: each request gets a new id and the old stream stops at its next chunk, without an error. Closing the popup in any way (Esc, ✕, Apply, Copy, *Open Settings*, losing focus to another app, the hotkey while the popup is in front) also cancels the running request: a hidden popup never shows that session again, so the rest of the answer would only use up the request quota or bill the API.
- **AI-55** Network chunks are split into frames on bytes, so a multi-byte character split across chunks is never decoded half-way.

**Verified by:** `prompt.rs` tests (schema holds exactly the requested tones, unknown tones dropped, closing tags neutralized, the reminder for Vietnamese, only non-empty tags, explanations off, the template is valid for every tone), `llm/mod.rs` tests (frames, only complete answers succeed, error codes), `llm/openai_compat.rs` and `llm/claude.rs` tests against a local HTTP server (streaming split across chunks, headers and body for both JSON modes, Haiku without effort or fallbacks, CRLF lines and comment lines, error codes by HTTP status, in-stream errors, cancellation, unreachable servers), `secrets.rs` tests (one account and variable per provider), `request.rs` tests (event field names, error fields, provider choice). Model quality is checked by hand with `scripts/bench-cases.json` (the old local-model reports in `docs/bench/` are history).

## Not implemented

- Measured Claude latency for Opus 5.5 / Sonnet 5.5 / Haiku 4.5 (`docs/PLAN.md` open item).
- Measured quality and latency of the free-tier models (there is no automated benchmark; the local-model one was removed).
- Per-app tones and a glossary (Phase 2).
