//! for （ for `/v1/chat/completions` ）。
//!
//! POST /v1/responses Responses API（OpenAI new ；Codex etc client in use ）
//! POST /v1/messages Anthropic Messages API（Claude Code etc client in use ）
//!
//! ── this ────────────────────────────────────────
//! is ****： down request Chat Completions， to and 
//! `/v1/chat/completions` **** （、、、
//! 、 not ）， down 。 then in 
//! `core::protocol` ， in `api::pipeline` —— file
//! 「HTTP 」： body、、 by response。
//!
//! ── Why not 「supports」──────────────────
//! `core::protocol` module header 。 1 ：all up 
//! already 1 as Chat， in 1 （2 ） in 6 
//! （12 ）， and add up when not 。
//!
//! ── requestWhy up ──────────────────────────
//! up is can （ in up up is 「 after return」 
//! ）。 down need ， `stream:true` request up ，
//! after JSON —— and `/v1/chat/completions` path 1 。
//! as ， to body `stream` as true，「 down need not need 」
//! by handler （`ForwardRequest.stream` is down ，
//! as up / ， `provider_loop` `ctx.stream`）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::compact_routing::{self, RequestFormat};
use crate::server::core::key_scope::KeyScope;
use crate::server::core::protocol::{anthropic, responses};
use crate::server::core::upstream::cancellation;
use crate::server::core::upstream::continuation::{self, Continuation};
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::{ForwardOutcome, ForwardRequest};
use crate::server::errors::GatewayError;
use crate::server::logging;
use crate::server::ServerState;

use super::disconnect_guard::DisconnectGuard;
use super::pipeline::{
    self, error_response, json_response, record_early_failure, sse_response, RecordContext,
};

/// request JSON for （failure to 400 + ）。
///
/// `client` by use from request（failure need can 「 is 」，
/// `record_early_failure` ）。
fn parse_object(
    state: &ServerState,
    started_at: i64,
    path: &str,
    body: &Bytes,
    source: &crate::server::request_stats::RequestSource,
) -> Result<Value, Response> {
    let parsed = serde_json::from_slice::<Value>(body).ok();
    let Some(payload) = parsed.filter(Value::is_object) else {
        let error = GatewayError::bad_request("请求体必须是 JSON 对象");
        record_early_failure(state, started_at, "", "", source, &error);
        logging::verbose("[Model]", &format!("← POST {path} 请求体不是 JSON 对象"));
        return Err(error.payload_response());
    };
    Ok(payload)
}

/// 1 before ： already Chat request to 。
///
/// `telemetry` by ** use **create（with_id id） after ： in 
/// （`record_started`） need in before ， need need id and ——
/// that in endpoint ， create up to endpoint
/// （ and `/v1/chat/completions` for ）。
///
/// `scope` is request in Key limit（R9，`None` = unlimited）：
/// **** that 1 （ when by ），**** that 1 in use 
/// `pipeline::resolve_model` already （ that —— not ：
/// is ， is ）。
async fn forward_chat(
    state: &ServerState,
    payload: Value,
    client_headers: &HeaderMap,
    stream: bool,
    dedupe_key: String,
    scope: Option<KeyScope>,
    telemetry: Arc<RequestTelemetry>,
) -> Result<ForwardOutcome, GatewayError> {
    // up down （ `autoContinue` **default**， when not clone）。
    // in this before is has ： this ， `payload`
    // already is **Chat **（`chat_from_responses` / `chat_from_anthropic` after ），
    // need assistant / user in Chat up has ——
    // to down that 1 。
    let continuation = continuation::plan().map(|max_rounds| Continuation {
        service: state.upstream().clone(),
        base_body: payload.clone(),
        stream,
        client_headers: client_headers.clone(),
        telemetry: telemetry.clone(),
        allowed_providers: scope.clone(),
        max_rounds,
    });
    let outcome = state
        .upstream()
        .forward(ForwardRequest {
            body: payload,
            stream,
            dedupe_key,
            client_headers: client_headers.clone(),
            telemetry: telemetry.clone(),
            allowed_providers: scope,
            // not is ： not ，
            pinned_account: None,
            // for request「 already in in 」（`ignore_model_gate`
            // is ， `ForwardRequest` ）
            ignore_model_gate: false,
            proxy_override: None,
        })
        .await;
    // ： in down before （must in 
    // `ResponsesStream` / `AnthropicStream` to before ，otherwise will before 
    // 、 after ）； then in this 。
    match (continuation, outcome) {
        (Some(context), Ok(ForwardOutcome::Stream { status, stream })) => {
            Ok(ForwardOutcome::Stream { status, stream: context.stream(stream) })
        }
        (Some(context), Ok(ForwardOutcome::Completion { body })) => {
            Ok(ForwardOutcome::Completion { body: context.complete(body).await })
        }
        (_, result) => result,
    }
}

// ─── POST /v1/responses ─────────────────────────────────────

/// POST /v1/responses
pub async fn responses_endpoint(
    State(state): State<ServerState>,
    headers: HeaderMap,
    // R9： in request 「 in Key limit」（`None` = unlimited，
    // `core::key_scope` module header）。 use `Option<Extension<_>>` not is 
    // `Extension<_>` —— after in down not to will request。
    key_scope: Option<Extension<KeyScope>>,
    key_identity: Option<Extension<crate::server::request_stats::ApiKeyIdentity>>,
    body: Bytes,
) -> Response {
    let scope = key_scope.map(|Extension(scope)| scope);
    let started_at = logging::now_ms();
    // requestclient（UA ， `core::client`）：failurepath and use 
    let source = pipeline::request_source(&headers, key_identity);
    let path = "/v1/responses";
    let raw = match parse_object(&state, started_at, path, &body, &source) {
        Ok(value) => value,
        Err(response) => return response,
    };
    // before ，。
    let compact_client = compact_routing::client_for_request(&source.behavior_client, &raw, RequestFormat::Responses);
    // request 1 ： need use request（instructions / tools / …），
    // down will payload Chat 
    let original = raw.clone();
    // down etc and down 1 when （client****：reasoning.effort
    // in will to chat_body reasoning_effort， —— 
    // client_reasoning_of ）
    let client_reasoning = pipeline::client_reasoning_of(&raw);
    let stream = raw.get("stream").and_then(Value::as_bool).unwrap_or(false);

    // （ has in this ）
    let mut chat_body = match responses::chat_from_responses(&raw) {
        Ok(body) => body,
        Err(message) => {
            let error = GatewayError::bad_request(message);
            record_early_failure(&state, started_at, &pipeline::model_field_text(&raw), &client_reasoning, &source, &error);
            return error.payload_response();
        }
    };
    if !chat_body
        .get("messages")
        .map(Value::is_array)
        .unwrap_or(false)
    {
        let error = GatewayError::bad_request("input 必须能转换成对话消息");
        record_early_failure(&state, started_at, "", &client_reasoning, &source, &error);
        return error.payload_response();
    }

    // （default / / directory）。
    // is this Key available（None = unlimited）， /v1/chat/completions
    let requested_model = match super::request_prepare::resolve_client_model(&state, &mut chat_body, scope.as_ref(), compact_client).await {
        Ok(model) => model,
        Err(error) => {
            record_early_failure(&state, started_at, &pipeline::model_field_text(&raw), &client_reasoning, &source, &error);
            return error.payload_response();
        }
    };
    let user_agent = pipeline::user_agent_of(&headers);
    logging::verbose(
        "[Model]",
        &format!(
            "← POST {path} model={} stream={stream} bytes={} ua={user_agent}",
            if requested_model.is_empty() { "(未指定)" } else { &requested_model },
            body.len(),
        ),
    );
    pipeline::write_debug_files(&body, "POST", path, &user_agent);

    // up （ by module header）
    // scope **** that 1 （ when by ）；
    // that 1 already in up （ forward_chat ）。
    // telemetry in this create（ not by forward_chat ）： in need in 
    // before ， down record_started。
    let telemetry = Arc::new(RequestTelemetry::with_id());
    let client_model = pipeline::model_field_text(&raw);
    // 「 in 」（ and /v1/chat/completions 1 when ：success、
    // before ； `RequestStats::record_started`）
    let telemetry_id = telemetry.snapshot().id;
    super::request_prepare::record_started(
        state.request_stats(),
        &telemetry_id,
        started_at,
        &requested_model,
        &client_model,
        &client_reasoning,
        &source,
        false,
    ).await;
    // in （ and /v1/chat/completions 1 when and by ，
    // `pipeline::live_row_sink`）
    telemetry.set_live_sink(pipeline::live_row_sink(
        state.request_stats(),
        telemetry_id.clone(),
        started_at,
    ));
    // + （ and /v1/chat/completions 1 when 
    // and 1 ： after can this request；handler axum when 
    // by Drop 408 。 down complete / handoff）
    if let Some(token) = cancellation::register(&telemetry_id) {
        telemetry.set_cancel_token(token);
    }
    let mut guard = DisconnectGuard::new(state.request_stats(), telemetry_id.clone());
    let outcome = forward_chat(
        &state,
        chat_body,
        &headers,
        stream,
        pipeline::sha256_hex(&body),
        scope,
        telemetry.clone(),
    )
    .await;
    let context = RecordContext {
        stats: state.request_stats(),
        telemetry,
        started_at,
        model: requested_model.clone(),
        // down clientrequest（ before model ）
        client_model,
        client_reasoning,
        status: 200,
        // down request：client that 1 （ before ）——only
        // when （`capture_raw_body`）。response in after ，
        // by RecordingStream in when 
        raw_request: pipeline::capture_raw_body(&body),
        raw_response: None,
        // ： not is （ `RecordContext::is_test`）
        is_test: false,
        // client：（ `RecordContext::client`）
        source: source.clone(),
    };

    match outcome {
        Ok(ForwardOutcome::Stream { status, stream: source }) => {
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
            let model = requested_model.clone();
            let request = original.clone();
            // up down （ must is client to that ）
            let context = RecordContext { status: i64::from(status.as_u16()), ..context };
            if stream {
                let mut machine = responses::ResponsesStream::new(&model, &request);
                let transformed = pipeline::transformed_stream(
                    source,
                    context,
                    pipeline::terminal::RESPONSES,
                    move |chunk| machine.push(chunk),
                );
                // to response（ not ； to ）
                guard.handoff();
                return sse_response(status, transformed);
            }
            // down need ：， JSON
            let mut collector = responses::ResponsesCollector::new();
            collect_stream(source, &mut |chunk| collector.push(chunk)).await;
            collector.finish();
            let body = collector.into_response(&model, &request);
            // response in before （ JSON ；failure or 
            // to None， `capture_raw_json`）
            let context = RecordContext {
                raw_response: pipeline::capture_raw_json(&body),
                ..context
            };
            pipeline::record_entry(&context, None);
            // already ： and 
            guard.complete();
            json_response(body)
        }
        Ok(ForwardOutcome::Completion { body: chat }) => {
            // up path（ not this ： need ， CatPaw etc 
            // has can can to Completion）。， not request。
            let response = responses::responses_from_chat(&chat, &requested_model, &original);
            let context = RecordContext {
                raw_response: pipeline::capture_raw_json(&response),
                ..context
            };
            pipeline::record_entry(&context, None);
            // already ： and 
            guard.complete();
            json_response(response)
        }
        Err(error) => {
            let message = error.message.clone();
            // in ：`error_response` will 1 message request
            // （ `api::chat` ）。
            logging::console_line("[Model]", &format!("❌ {message}"));
            let response = error_response(context, &error, |status, error| {
                let code = anthropic::responses_error_code(status);
                GatewayError::with_status(i32::from(status), error.message.clone())
                    .with_code(code)
                    .payload_response()
            });
            // already （ 408）： and 
            guard.complete();
            response
        }
    }
}

// ─── POST /v1/messages ──────────────────────────────────────

/// POST /v1/messages
pub async fn messages_endpoint(
    State(state): State<ServerState>,
    headers: HeaderMap,
    // R9： responses_endpoint（`Option` + not to = unlimited）
    key_scope: Option<Extension<KeyScope>>,
    key_identity: Option<Extension<crate::server::request_stats::ApiKeyIdentity>>,
    body: Bytes,
) -> Response {
    let scope = key_scope.map(|Extension(scope)| scope);
    let started_at = logging::now_ms();
    // requestclient（UA ， `core::client`）：failurepath and use 
    let source = pipeline::request_source(&headers, key_identity);
    let path = "/v1/messages";
    let raw = match parse_object(&state, started_at, path, &body, &source) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let stream = raw.get("stream").and_then(Value::as_bool).unwrap_or(false);

    let compact_client = compact_routing::client_for_request(&source.behavior_client, &raw, RequestFormat::Anthropic);
    // 
    let mut chat_body = match anthropic::chat_from_anthropic(&raw) {
        Ok(body) => body,
        Err(message) => {
            let error = GatewayError::bad_request(message);
            record_early_failure(&state, started_at, &pipeline::model_field_text(&raw), "", &source, &error);
            return anthropic_error_response(&error);
        }
    };
    // is this Key available（None = unlimited）， and 。
    // down etc from ** after chat **：Anthropic thinking/output_config →
    // reasoning_effort is （anthropic_effort）， use and 
    let client_reasoning = pipeline::client_reasoning_of(&chat_body);
    let requested_model = match super::request_prepare::resolve_client_model(&state, &mut chat_body, scope.as_ref(), compact_client).await {
        Ok(model) => model,
        Err(error) => {
            record_early_failure(&state, started_at, &pipeline::model_field_text(&raw), &client_reasoning, &source, &error);
            return anthropic_error_response(&error);
        }
    };
    let user_agent = pipeline::user_agent_of(&headers);
    logging::verbose(
        "[Model]",
        &format!(
            "← POST {path} model={} stream={stream} bytes={} ua={user_agent}",
            if requested_model.is_empty() { "(未指定)" } else { &requested_model },
            body.len(),
        ),
    );
    pipeline::write_debug_files(&body, "POST", path, &user_agent);

    // scope **** that 1 （ when by ）；
    // that 1 already in up （ forward_chat ）。
    // telemetry in this create（ and responses_endpoint ： in need in 
    // before ）
    let telemetry = Arc::new(RequestTelemetry::with_id());
    let client_model = pipeline::model_field_text(&raw);
    // `message_start` ： up usage need etc that to ， by 
    // request 1 （ `/v1/messages/count_tokens`， issue #437）。
    // must in this —— `chat_body` down will move `forward_chat`。
    let input_estimate = anthropic::estimate_tokens_for_body(&chat_body);
    // 「 in 」（ and /v1/chat/completions 1 when ：success、
    // before ； `RequestStats::record_started`）
    let telemetry_id = telemetry.snapshot().id;
    super::request_prepare::record_started(
        state.request_stats(),
        &telemetry_id,
        started_at,
        &requested_model,
        &client_model,
        &client_reasoning,
        &source,
        false,
    ).await;
    // in （ and /v1/chat/completions 1 when and by ，
    // `pipeline::live_row_sink`）
    telemetry.set_live_sink(pipeline::live_row_sink(
        state.request_stats(),
        telemetry_id.clone(),
        started_at,
    ));
    // + （ and /v1/chat/completions 1 when 
    // and 1 ： after can this request；handler axum when 
    // by Drop 408 。 down complete / handoff）
    if let Some(token) = cancellation::register(&telemetry_id) {
        telemetry.set_cancel_token(token);
    }
    let mut guard = DisconnectGuard::new(state.request_stats(), telemetry_id.clone());
    let outcome = forward_chat(
        &state,
        chat_body,
        &headers,
        stream,
        pipeline::sha256_hex(&body),
        scope,
        telemetry.clone(),
    )
    .await;
    let context = RecordContext {
        stats: state.request_stats(),
        telemetry,
        started_at,
        model: requested_model.clone(),
        // down clientrequest（ before model ）
        client_model,
        client_reasoning,
        status: 200,
        // down request：client that 1 （ before ）——only
        // when （`capture_raw_body`）。response in after ，
        // by RecordingStream in when 
        raw_request: pipeline::capture_raw_body(&body),
        raw_response: None,
        // ： not is （ `RecordContext::is_test`）
        is_test: false,
        // client：（ `RecordContext::client`）
        source: source.clone(),
    };

    match outcome {
        Ok(ForwardOutcome::Stream { status, stream: source }) => {
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
            let model = requested_model.clone();
            let context = RecordContext { status: i64::from(status.as_u16()), ..context };
            if stream {
                let mut machine = anthropic::AnthropicStream::new(&model, Some(input_estimate));
                let transformed = pipeline::transformed_stream(
                    source,
                    context,
                    pipeline::terminal::ANTHROPIC,
                    move |chunk| machine.push(chunk),
                );
                // to response（ not ； to ）
                guard.handoff();
                return sse_response(status, transformed);
            }
            let mut collector = anthropic::AnthropicCollector::new();
            collect_stream(source, &mut |chunk| collector.push(chunk)).await;
            collector.finish();
            let body = collector.into_response(&model);
            // response in before （ JSON ；failure or 
            // to None， `capture_raw_json`）
            let context = RecordContext {
                raw_response: pipeline::capture_raw_json(&body),
                ..context
            };
            pipeline::record_entry(&context, None);
            // already ： and 
            guard.complete();
            json_response(body)
        }
        Ok(ForwardOutcome::Completion { body: chat }) => {
            let response = anthropic::anthropic_from_chat(&chat, &requested_model);
            let context = RecordContext {
                raw_response: pipeline::capture_raw_json(&response),
                ..context
            };
            pipeline::record_entry(&context, None);
            // already ： and 
            guard.complete();
            json_response(response)
        }
        Err(error) => {
            let message = error.message.clone();
            // in ：`error_response` will 1 message request
            // （ `api::chat` ）。
            logging::console_line("[Model]", &format!("❌ {message}"));
            let response = error_response(context, &error, |status, error| {
                anthropic_error_response_with_status(status, &error.message)
            });
            // already （ 408）： and 
            guard.complete();
            response
        }
    }
}

/// 1 （ use ）。
///
/// up already `ReasoningCoalescer`， this to is Chat SSE。
async fn collect_stream(
    mut source: Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
    sink: &mut impl FnMut(&[u8]),
) {
    use futures::StreamExt;
    while let Some(item) = source.next().await {
        match item {
            Ok(chunk) => sink(&chunk),
            // up in ： already （ will already has ），
            // and path「 not use already to 」 1 to 
            Err(error) => {
                logging::verbose("[Model]", &format!("聚合时上游中断: {error}"));
                break;
            }
        }
    }
}

/// Anthropic errorresponse（`{type:"error", error:{type, message}}`）
fn anthropic_error_response(error: &GatewayError) -> Response {
    anthropic_error_response_with_status(error.http_status().as_u16(), &error.message)
}

fn anthropic_error_response_with_status(status: u16, message: &str) -> Response {
    let kind = anthropic::error_kind_for_status(status);
    let body = json!({
        "type": "error",
        "error": { "type": kind, "message": message },
    });
    let code = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = json_response(body);
    *response.status_mut() = code;
    response
}