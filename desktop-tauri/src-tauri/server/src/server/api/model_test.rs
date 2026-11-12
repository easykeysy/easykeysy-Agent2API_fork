//! API（ that 「」）：
//!
//!   POST /api/models/test   `{provider, model, account_id?, prompt?, system_prompt?,
//!                            reasoning?, stream?, protocol?, proxy?, test_id?}`
//! → 1 request 
//!
//! ── Why 1 ─────────────────────────────────────
//! and `model_manage.rs`（ then ， not ） and `chat.rs`（ for ）
//! not ： in （ need will / Key）、 need ** up **， and 
//! is that 。 in 1 will 「 this 1 to not for 」
//! ；file + （ and `models.rs` / `auto_checkin.rs` 1 ）
//! in by up can 。
//!
//! ── ：****， not 1 ─────────────
//! `UpstreamService::forward` is 1 —— token new 、429 and 、
//! 、、、 etc 、 and request
//! in that up 。 1 「」， is and 
//! ****（ OmniProxy in `modelTestRun.ts` this 1 ：
//! after 「 by 」）。
//!
//! ── and request not （ `ForwardRequest` ）──────
//! 1. ** this 1 **：`allowed_providers = KeyScope::provider_only(provider)`
//! —— 1 for in 1 ， not 「 this 1 」 
//! is ，none；
//! 2. ** this 1 **：`pinned_account`（ `core::upstream::rotate`），
//! and ** not ** —— is 「 this not 」， will ；
//! 3. ** not **：`dedupe_key` empty。 is 「 body 」，
//! need and for 1 body（ before 1 1 
//! request）， will etc 、 1 。
//! 4. ** **：`ignore_model_gate`。 this by use is 「、
//! need not need enabled」， is ； up 「 
//! 」 404 ， that will for in 。
//! 1 that ， by （ not 
//! → 、 up etc not ）—— is 「 this 
//! in this up 」， not is 「 after will 」。
//! 5. **supports by when **：`proxy_override`。 
//! config， can in when 「」 or in any，only for 
//! request、 not set。
//! 6. **supports for 1 **：`protocol`（`chat_completions` / `responses` /
//! `anthropic`）。 by down request、 `core::protocol` 
//! Chat ， to up after by and ，
//! 1 and is 。
//!
//! this ， its 1 and request 1 —— including** will **。
//!
//! ── ：request， excluding ────────────────────────────
//! request is 1 ， will request 1 down （`is_test = 1`）。
//! request（ can for 、 can ）， excluding（ 
//! will ）。 in 1 ： `record_started`，
//! `record_entry` 。
//!
//! ── client ──────────────────────────────────────────────
//! 与 `/v1/*` 1 ：`cancellation::register` this can 「request」
//! ，`DisconnectGuard` in handler axum when 1 408 
//! （otherwise that in will 1 in request）。 before 「 in 」= this 1 
//! = abort fetch， to is this 。
//!
//! ── ：failure is 2xx ─────────────────────────────
//! 「 this ， is failure」 and 「 this request no 」 is ：
//! up error、unavailable、unavailable this in response `status` / `error` ，
//! HTTP 1 200（ and `/api/models/refresh` 2xx 1 ）， before can to 
//! ， not is errorresponse。 2xx has 「request not 」
//! （ provider / model， or this not in this ）。

use std::sync::Arc;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::key_scope::KeyScope;
use crate::server::core::protocol::{anthropic, responses};
use crate::server::core::proxies::ResolvedProxy;
use crate::server::core::providers::catalog;
use crate::server::core::upstream::aggregate::aggregate_frame_stream;
use crate::server::core::upstream::cancellation;
use crate::server::core::upstream::sse::FramePolicy;
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::{ForwardOutcome, ForwardRequest};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

use super::disconnect_guard::DisconnectGuard;
use super::pipeline;

/// use empty when default（ before 1 ）。
///
/// 「」 not is 1 has ： need is **request** —— 、
/// up ，「 not 」 this 。
pub const DEFAULT_TEST_PROMPT: &str = "";
/// limit（ use and 1 ）。
///
/// and OmniProxy `MAX_TEST_PROMPT_LENGTH` ： not is ， 1 
/// no has （ is 「 not 」）， can can up / 1 。
const MAX_PROMPT_CHARS: usize = 4000;
/// **total**（ etc up and ）。
///
/// Why need has up （request no has ）：request etc by client
/// （ when can ）， is ** 1 etc ** —— no has up when ， 1 up 
/// will 1 ， use not this etc 、 not can not can 。
///
/// ── Why is 50 seconds（ this request when ）────────────
/// API use `workbuddyDesktop` HTTP， that has 
/// **60 seconds**total when （`src/gateway.rs` `REQUEST_TIMEOUT_MS`）。 if ，
/// to is 「 when 」—— 1 and Reasonnone 。
/// this up must in ：50 seconds ≤ 60 seconds，client to 1 
/// 「 when 」， not is 1 error。
const TEST_TIMEOUT: Duration = Duration::from_secs(50);

/// for （ 1 ， and `/v1/chat/completions`、`/v1/responses`、`/v1/messages` 1 1 for ）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestProtocol {
    ChatCompletions,
    Responses,
    Anthropic,
}

impl TestProtocol {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "chat" | "chat_completions" => Ok(Self::ChatCompletions),
            "responses" => Ok(Self::Responses),
            "anthropic" | "messages" => Ok(Self::Anthropic),
            other => Err(format!(
                "unsupported for {other}（optional chat_completions / responses / anthropic）"
            )),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat_completions",
            Self::Responses => "responses",
            Self::Anthropic => "anthropic",
        }
    }
}

/// from request 1 optional（empty、 to limit； by empty）。
fn text_field(body: &Value, key: &str, limit: usize) -> String {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .map(|text| pipeline::truncate_chars(text, limit))
        .unwrap_or_default()
}

/// Chat Completions down request。
///
/// ── etc （「」 and 「」 in this ）─────────
/// not `reasoning` when ** 1 etc not body**： up etc by 
/// by （`model_rules::reasoning` has ）； then 
/// `reasoning_effort` —— that is 「client 」， by has ****
/// （ not client etc ）。 is up that for 1 
/// path， 1 not need need 。
fn build_chat_body(
    model: &str,
    prompt: &str,
    system_prompt: &str,
    reasoning: &str,
    stream: bool,
) -> Value {
    let mut messages: Vec<Value> = Vec::new();
    if !system_prompt.is_empty() {
        messages.push(json!({ "role": "system", "content": system_prompt }));
    }
    messages.push(json!({ "role": "user", "content": prompt }));
    let mut body = json!({
        "model": model,
        "messages": messages,
        // down （`stream` as false when will stream:true up 、
        // JSON —— and client need when is 1 ）
        "stream": stream,
    });
    if !reasoning.is_empty() {
        if let Some(object) = body.as_object_mut() {
            object.insert(
                "reasoning_effort".to_string(),
                Value::String(reasoning.to_string()),
            );
        }
    }
    body
}

/// OpenAI Responses down request（ for `POST /v1/responses`）。
fn build_responses_body(
    model: &str,
    prompt: &str,
    system_prompt: &str,
    reasoning: &str,
    stream: bool,
) -> Value {
    let mut body = json!({
        "model": model,
        "input": prompt,
        "stream": stream,
    });
    if let Some(object) = body.as_object_mut() {
        if !system_prompt.is_empty() {
            object.insert(
                "instructions".to_string(),
                Value::String(system_prompt.to_string()),
            );
        }
        if !reasoning.is_empty() {
            object.insert("reasoning".to_string(), json!({ "effort": reasoning }));
        }
    }
    body
}

/// Anthropic Messages down request（ for `POST /v1/messages`）。
fn build_anthropic_body(
    model: &str,
    prompt: &str,
    system_prompt: &str,
    reasoning: &str,
    stream: bool,
) -> Value {
    let mut body = json!({
        "model": model,
        "max_tokens": 8192,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "stream": stream,
    });
    if let Some(object) = body.as_object_mut() {
        if !system_prompt.is_empty() {
            object.insert(
                "system".to_string(),
                Value::String(system_prompt.to_string()),
            );
        }
        if !reasoning.is_empty() {
            object.insert("thinking".to_string(), json!({ "type": "enabled" }));
            object.insert("output_config".to_string(), json!({ "effort": reasoning }));
        }
    }
    body
}

/// by 「 down request」 and 「 after Chat request」。
fn build_protocol_request(
    protocol: TestProtocol,
    model: &str,
    prompt: &str,
    system_prompt: &str,
    reasoning: &str,
    stream: bool,
) -> Result<(Value, Value), String> {
    match protocol {
        TestProtocol::ChatCompletions => {
            let chat = build_chat_body(model, prompt, system_prompt, reasoning, stream);
            Ok((chat.clone(), chat))
        }
        TestProtocol::Responses => {
            let req = build_responses_body(model, prompt, system_prompt, reasoning, stream);
            let chat = responses::chat_from_responses(&req)?;
            Ok((req, chat))
        }
        TestProtocol::Anthropic => {
            let req = build_anthropic_body(model, prompt, system_prompt, reasoning, stream);
            let mut chat = anthropic::chat_from_anthropic(&req)?;
            // use （ CatPaw max / minimal）
            // this ， reasoning_effort， use 
            if !reasoning.is_empty() {
                if let Some(object) = chat.as_object_mut() {
                    object.insert(
                        "reasoning_effort".to_string(),
                        Value::String(reasoning.to_string()),
                    );
                }
            }
            Ok((req, chat))
        }
    }
}

/// from chat response 1 （`choices[0].message.<key>`； no has then empty）。
fn message_field(completion: &Value, key: &str) -> String {
    completion
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get(key))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// from Responses response and need 。
fn extract_responses_fields(response: &Value) -> (String, String) {
    let mut reply = response
        .get("output_text")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut reasoning_parts: Vec<String> = Vec::new();
    let mut message_parts: Vec<String> = Vec::new();
    if let Some(output) = response.get("output").and_then(Value::as_array) {
        for item in output {
            match item.get("type").and_then(Value::as_str).unwrap_or("") {
                "reasoning" => {
                    if let Some(summary) = item.get("summary").and_then(Value::as_array) {
                        for part in summary {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                if !text.is_empty() {
                                    reasoning_parts.push(text.to_string());
                                }
                            }
                        }
                    }
                }
                "message" => {
                    if let Some(content) = item.get("content").and_then(Value::as_array) {
                        for part in content {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                if !text.is_empty() {
                                    message_parts.push(text.to_string());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if reply.is_empty() && !message_parts.is_empty() {
        reply = message_parts.join("");
    }
    (reply, reasoning_parts.join(""))
}

/// from Anthropic Messages response and 。
fn extract_anthropic_fields(response: &Value) -> (String, String) {
    let mut text_parts: Vec<String> = Vec::new();
    let mut thinking_parts: Vec<String> = Vec::new();
    if let Some(content) = response.get("content").and_then(Value::as_array) {
        for block in content {
            match block.get("type").and_then(Value::as_str).unwrap_or("") {
                "text" => {
                    if let Some(text) = block.get("text").and_then(Value::as_str) {
                        if !text.is_empty() {
                            text_parts.push(text.to_string());
                        }
                    }
                }
                "thinking" => {
                    if let Some(thinking) = block.get("thinking").and_then(Value::as_str) {
                        if !thinking.is_empty() {
                            thinking_parts.push(thinking.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    (text_parts.join(""), thinking_parts.join(""))
}

/// after Chat Completion by down and 。
fn translate_completion(
    protocol: TestProtocol,
    model: &str,
    protocol_request: &Value,
    status: i64,
    chat_completion: Value,
) -> TestOutcome {
    match protocol {
        TestProtocol::ChatCompletions => TestOutcome {
            status,
            error: None,
            reply: message_field(&chat_completion, "content"),
            reasoning_text: message_field(&chat_completion, "reasoning_content"),
            raw_response: pipeline::capture_raw_json(&chat_completion),
        },
        TestProtocol::Responses => {
            let response_body =
                responses::responses_from_chat(&chat_completion, model, protocol_request);
            let raw_response = pipeline::capture_raw_json(&response_body);
            if response_body.get("status").and_then(Value::as_str) == Some("failed") {
                let message = response_body
                    .pointer("/error/message")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                    .unwrap_or("Responses return failed ")
                    .to_string();
                return TestOutcome {
                    status: 502,
                    error: Some(message),
                    reply: String::new(),
                    reasoning_text: String::new(),
                    raw_response,
                };
            }
            let (reply, reasoning_text) = extract_responses_fields(&response_body);
            TestOutcome {
                status,
                error: None,
                reply,
                reasoning_text,
                raw_response,
            }
        }
        TestProtocol::Anthropic => {
            let response_body = anthropic::anthropic_from_chat(&chat_completion, model);
            let raw_response = pipeline::capture_raw_json(&response_body);
            let (reply, reasoning_text) = extract_anthropic_fields(&response_body);
            TestOutcome {
                status,
                error: None,
                reply,
                reasoning_text,
                raw_response,
            }
        }
    }
}

/// when （`proxy` ）：
/// - `""` / `"follow"`：config（`None`）
/// - `"direct"` / `"none"`：（`Some(None)`）
/// - `"pool:<id>"` or `"<id>"`：（`Some(Some(proxy))`）
fn parse_proxy_override(
    raw: &str,
) -> Result<(Option<Option<ResolvedProxy>>, Option<String>), String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("follow") {
        return Ok((None, None));
    }
    if trimmed.eq_ignore_ascii_case("direct") || trimmed.eq_ignore_ascii_case("none") {
        return Ok((Some(None), Some("".to_string())));
    }
    let proxy_id = trimmed.strip_prefix("pool:").unwrap_or(trimmed).trim();
    if proxy_id.is_empty() {
        return Err(" id".to_string());
    }
    let resolved = crate::server::core::proxy_pool::resolve_for_test(proxy_id)?;
    let label = if resolved.label.is_empty() {
        format!(
            "{} {}:{}",
            resolved.protocol.to_uppercase(),
            resolved.host,
            resolved.port.unwrap_or(0)
        )
    } else {
        resolved.label.clone()
    };
    Ok((Some(Some(resolved)), Some(label)))
}

/// this 1 （ 1 ）。
struct TestOutcome {
    status: i64,
    error: Option<String>,
    reply: String,
    reasoning_text: String,
    raw_response: Option<String>,
}

/// POST /api/models/test
pub async fn run_model_test(State(state): State<ServerState>, headers: HeaderMap, body: Bytes) -> Response {
    let access = match super::model_access::authenticate(&headers, super::model_access::MODEL_TEST, true) {
        Ok(access) => access,
        Err(error) => return error.payload_response(),
    };
    let payload = match parse_body(&body) {
        Ok(payload) => payload,
        Err(error) => return errors::management_error(400, error.message),
    };
    let provider = text_field(&payload, "provider", 64);
    let model = text_field(&payload, "model", 128);
    if provider.is_empty() {
        return errors::management_error(400, " provider");
    }
    if model.is_empty() {
        return errors::management_error(400, " model");
    }
    let key_scope = access.scope.clone().unwrap_or_default();
    if !key_scope.allows_model(&model) {
        let error = errors::GatewayError::with_status(404, format!(" '{model}' not in this Key available"))
            .with_code("model_not_found");
        pipeline::record_early_failure(&state, logging::now_ms(), &model, "", &access.source, &error);
        return error.payload_response();
    }
    let Some(test_scope) = key_scope.for_test_provider(&provider) else {
        let error = errors::GatewayError::with_status(403, format!(" '{provider}' not in this Key available"));
        pipeline::record_early_failure(&state, logging::now_ms(), &model, "", &access.source, &error);
        return error.payload_response();
    };
    // must** this 1 **： no has when ， will 「 not in this 
    // Key available」 —— that for none， this 
    // and down 1 （ is no new ，or up ）
    let carriers = catalog::providers_for_model(&model);
    if !carriers.iter().any(|id| id.eq_ignore_ascii_case(&provider)) {
        return errors::management_error(
            400,
            format!(
                "{provider} current no has {model}：「get」 new ， or no up "
            ),
        );
    }
    if crate::server::core::providers::images::provider_model_is_image(&provider, &model) {
        return errors::management_error(400, " /v1/images/generations use ，unsupported for ");
    }
    let protocol_raw = text_field(&payload, "protocol", 32);
    let protocol = match TestProtocol::parse(&protocol_raw) {
        Ok(protocol) => protocol,
        Err(message) => return errors::management_error(400, message),
    };
    let account_id = text_field(&payload, "account_id", 128);
    let proxy_raw = text_field(&payload, "proxy", 128);
    let (proxy_override, proxy_label) = match parse_proxy_override(&proxy_raw) {
        Ok(parsed) => parsed,
        Err(reason) => {
            // in unavailable（ Clash 、 already delete） when ，
            // by failure（HTTP 200 + status=400）return， before Reason
            return ok_json(json!({
                "success": false,
                "status": 400,
                "error": reason,
                "reply": "",
                "reasoning": "",
                "provider": provider,
                "model": model,
                "protocol": protocol.as_str(),
                "proxy": Value::Null,
                "account_id": if account_id.is_empty() { Value::Null } else { Value::String(account_id) },
                "account_name": Value::Null,
                "upstream_model": Value::Null,
                "upstream_reasoning": Value::Null,
                "duration_ms": 0,
                "ttfb_ms": Value::Null,
                "attempts": 1,
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0,
            }));
        }
    };
    let prompt = {
        let text = text_field(&payload, "prompt", MAX_PROMPT_CHARS);
        if text.is_empty() {
            DEFAULT_TEST_PROMPT.to_string()
        } else {
            text
        }
    };
    let system_prompt = text_field(&payload, "system_prompt", MAX_PROMPT_CHARS);
    let reasoning = text_field(&payload, "reasoning", 32);
    // by （ and before default 1 ）： to ， that is this 
    // has 1 
    let stream = payload.get("stream").and_then(Value::as_bool).unwrap_or(true);

    let (protocol_request, request_body) = match build_protocol_request(
        protocol,
        &model,
        &prompt,
        &system_prompt,
        &reasoning,
        stream,
    ) {
        Ok(pair) => pair,
        Err(message) => {
            return ok_json(json!({
                "success": false,
                "status": 400,
                "error": message,
                "reply": "",
                "reasoning": "",
                "provider": provider,
                "model": model,
                "protocol": protocol.as_str(),
                "proxy": proxy_label,
                "account_id": if account_id.is_empty() { Value::Null } else { Value::String(account_id) },
                "account_name": Value::Null,
                "upstream_model": Value::Null,
                "upstream_reasoning": Value::Null,
                "duration_ms": 0,
                "ttfb_ms": Value::Null,
                "attempts": 1,
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0,
            }));
        }
    };

    let started_at = logging::now_ms();
    // ── id by ** before ** to （`test_id`）， not to ─────────
    // Why： before need in when can ， is has 
    // 「request」（`POST /api/stats/requests/terminate?id=`， by id 
    // ）—— that need 1 id， （`invoke('api_request')`）
    // ** no has abort**： before none 1 already in invoke。 is id 
    // to before ： 1 id、request 1 up ， can use 1 
    // id （request that 1 id is ， for ）。
    //
    // `ensure_id` in id as empty when ， has before id when need use ** no id **
    // （`new`），otherwise will that 
    let given_id = text_field(&payload, "test_id", 64);
    let telemetry = Arc::new(RequestTelemetry::new());
    if !given_id.is_empty() {
        telemetry.ensure_id(&given_id);
    }
    let test_id = telemetry.id();
    // 「 in 」： and request 1 when （success、 before ）。
    // `is_test = true` 1 down —— excluding、request。
    // and Key 1 。
    super::request_prepare::record_started(state.request_stats(), &test_id, started_at, &model, &model, &reasoning, &access.source, true).await;
    if let Some(token) = cancellation::register(&test_id) {
        telemetry.set_cancel_token(token);
    }
    // ：handler （ before = abort fetch） when 408 。
    // pathmust `complete()`， down 
    let mut guard = DisconnectGuard::new(state.request_stats(), test_id.clone());
    // request down request by request（ for can ）；
    // and 1 ：only when （ `capture_raw_json`）
    let raw_request = pipeline::capture_raw_json(&protocol_request);

    let outcome = run_forward(
        &state,
        &telemetry,
        &account_id,
        protocol,
        &model,
        &protocol_request,
        request_body,
        stream,
        proxy_override,
        test_scope,
    )
    .await;
    let finished_at = logging::now_ms();
    let snapshot = telemetry.snapshot();
    let ttfb_ms = snapshot
        .first_response_at
        .map(|at| (at - started_at).max(0));

    // （request， by is_test excluding）： and /v1/* use is 1 
    // `record_entry`， is and request
    let context = pipeline::RecordContext {
        stats: state.request_stats(),
        telemetry: telemetry.clone(),
        started_at,
        model: model.clone(),
        client_model: model.clone(),
        client_reasoning: reasoning.clone(),
        status: outcome.status,
        raw_request,
        raw_response: outcome.raw_response.clone(),
        is_test: true,
        source: access.source.clone(),
    };
    pipeline::record_entry(&context, None);
    guard.complete();

    ok_json(json!({
        "success": outcome.error.is_none(),
        "status": outcome.status,
        "error": outcome.error,
        "reply": outcome.reply,
        "reasoning": outcome.reasoning_text,
        "provider": provider,
        "model": model,
        "protocol": protocol.as_str(),
        "proxy": proxy_label,
        "apiKeyId": access.source.api_key.id,
        "apiKeyName": access.source.api_key.name,
        "apiKeySource": access.source.api_key.source,
        // and （， telemetry ）： is 
        // request that ，「 / use 」 this down to 
        // and request not ，requestparameter has use 
        "account_id": snapshot.account_id,
        "account_name": snapshot.account_name,
        "upstream_model": snapshot.upstream_model,
        "upstream_reasoning": snapshot.upstream_reasoning,
        "duration_ms": finished_at - started_at,
        "ttfb_ms": ttfb_ms,
        "attempts": snapshot.attempts.max(1),
        "prompt_tokens": snapshot.prompt_tokens,
        "completion_tokens": snapshot.completion_tokens,
        "total_tokens": snapshot.total_tokens,
    }))
}

/// 1 ： + by [`TestOutcome`]。
///
/// and not in this ：，** that 1 must by use 
/// **（otherwise `ForwardOutcome::Stream` 1 ， this request no ）。
/// use is down 1 （`aggregate_frame_stream`），
/// after by （Chat Completions / Responses / Anthropic Messages）
/// and 。
#[allow(clippy::too_many_arguments)]
async fn run_forward(
    state: &ServerState,
    telemetry: &Arc<RequestTelemetry>,
    account_id: &str,
    protocol: TestProtocol,
    model: &str,
    protocol_request: &Value,
    request_body: Value,
    stream: bool,
    proxy_override: Option<Option<ResolvedProxy>>,
    test_scope: KeyScope,
) -> TestOutcome {
    let forward = state.upstream().forward(ForwardRequest {
        body: request_body,
        stream,
        // not （module header）： and when ， will etc 
        dedupe_key: String::new(),
        // empty： not is client ， not this to 
        // up （ will will ， `passthrough_session_headers`）
        client_headers: Default::default(),
        telemetry: telemetry.clone(),
        // this 1 （ 1 for can can in up ，module header）
        allowed_providers: Some(test_scope),
        // this 1 ， and not 
        pinned_account: if account_id.is_empty() {
            None
        } else {
            Some(account_id.to_string())
        },
        // enabled need can —— 「、 need not need enabled」 is this by use 。
        // up that 1 ， 「 already in in 」 not used for
        // （ `ForwardRequest::ignore_model_gate`）。
        ignore_model_gate: true,
        // when / （None = config）
        proxy_override,
    });
    // total in （）： is 1 etc ，must has up ，
    // otherwise 1 up will 1 （ TEST_TIMEOUT）
    match tokio::time::timeout(TEST_TIMEOUT, forward).await {
        Err(_elapsed) => TestOutcome {
            status: 504,
            error: Some(format!(" when （{} seconds no to ）", TEST_TIMEOUT.as_secs())),
            reply: String::new(),
            reasoning_text: String::new(),
            raw_response: None,
        },
        Ok(Err(error)) => TestOutcome {
            status: i64::from(error.status_code),
            error: Some(error.message),
            reply: String::new(),
            reasoning_text: String::new(),
            raw_response: None,
        },
        Ok(Ok(ForwardOutcome::Completion { body })) => {
            translate_completion(protocol, model, protocol_request, 200, body)
        }
        Ok(Ok(ForwardOutcome::Stream { status, stream: source })) => {
            let status = i64::from(status);
            // `ForwardOutcome::Stream` to is `Box<dyn Stream + Unpin>`， need 
            // is `BoxStream`（`Pin<Box<dyn Stream>>`）—— `boxed()` this 1 is 
            // this Unpin `'static` Pin ， not 
            use futures::StreamExt;
            match tokio::time::timeout(
                TEST_TIMEOUT,
                aggregate_frame_stream(source.boxed(), telemetry.clone(), FramePolicy::default()),
            )
            .await
            {
                Err(_elapsed) => TestOutcome {
                    status: 504,
                    error: Some(format!(
                        " when （{} seconds no to ）",
                        TEST_TIMEOUT.as_secs()
                    )),
                    reply: String::new(),
                    reasoning_text: String::new(),
                    raw_response: None,
                },
                Ok(Err(error)) => TestOutcome {
                    status: i64::from(error.status_code),
                    error: Some(error.message),
                    reply: String::new(),
                    reasoning_text: String::new(),
                    raw_response: None,
                },
                Ok(Ok(aggregated)) => {
                    translate_completion(protocol, model, protocol_request, status, aggregated.body)
                }
            }
        }
    }
}