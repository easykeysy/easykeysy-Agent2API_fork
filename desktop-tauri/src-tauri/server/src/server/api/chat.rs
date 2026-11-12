//! for by （ for server.mjs 466-560、956-961 ）。
//!
//! POST /v1/chat/completions protected（Node this API Key）
//! GET /v1/models public（Node this ** not ** API Key ——
//! is ，client when in key before use ）
//!
//! （`/v1/responses`、`/v1/messages`） in `api::protocol` ，
//! and file use `api::pipeline` （ / / response），
//! in （ `core::protocol` module header）。
//!
//! ── handleModelRequest （，）──────────────
//! ① body must is JSON for （400「requestmust is JSON for 」）
//! ② must has messages （400「 messages 」）
//! ③ verbose ：method/path/model/stream/msgs/bytes/ua
//! ④ {config_dir}/debug/last-request.json + .meta.txt（failure not request）
//! ⑤ ： → defaultModel → directory isDefault → ；
//! not in directory → 400（）
//! ⑥ rememberRequestModel（default already ）
//! ⑦ ： / 
//!
//! ── error in ────────────────────────────────────────────
//! failure when ：headers no → errorPayload（OpenAI + 429 reset_at）；
//! headers already （ already ）→ by response（Node is 
//! `data: {"error":...}` + `data: [DONE]`，Rust etc forward ）。
//!
//! ── （） ──────────────────────────
//! file** not **：request by client to ，
//! request。 in ** 1 provider will before **， is by config
//! `sanitizeBlacklistFingerprints` （ `core::upstream::payload` 
//! `send_body`）—— when none is ， to is request。
//!
//! ── request（）──────────────────────────────────
//! file is `RequestStats::record` ** 1 use **（ `record_entry`）：
//! nonesuccess / failure / client in ， 1 use request** 1 ** ——
//! 429 1 request， not 。
//! to `RecordingStream`（ in handler return after ），
//! and before failure then in 。usage by `core::upstream` 。
//!
//! ── already to `api::pipeline`（ use ）──────────────
//! 、、response、 this in in that ，file
//! 「Chat 」 ：messages 、OpenAI error。
//! this `/v1/responses` and `/v1/messages` can use 1 ， not will 。

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::compact_routing::{self, RequestFormat};
use crate::server::core::upstream::cancellation;
use crate::server::core::upstream::continuation::{self, Continuation};
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::{ForwardOutcome, ForwardRequest};
use crate::server::errors::GatewayError;
use crate::server::http::raw_json;
use crate::server::logging;
use crate::server::ServerState;

use super::disconnect_guard::DisconnectGuard;
use super::pipeline::{
    self, json_response, model_field_text, record_early_failure, record_entry, sse_response,
    write_debug_files, RecordContext, RecordingStream,
};

/// POST /v1/chat/completions
pub async fn chat_completions(
    State(state): State<ServerState>,
    headers: HeaderMap,
    // R9： in request 「 in Key limit」。`Option` is ——
    // 、Environment variable key、 use no has ，
    // 「 not to = unlimited」 is need （ `core::key_scope` module header）。
    // axum for `Option<Extension<T>>` has ： not to is `None`， not will 
    // `Extension<T>` that request（ that will 500）。
    key_scope: Option<Extension<crate::server::core::key_scope::KeyScope>>,
    key_identity: Option<Extension<crate::server::request_stats::ApiKeyIdentity>>,
    body: Bytes,
) -> Response {
    let scope = key_scope.map(|Extension(scope)| scope);
    // request when （requeststatistics use ）。 in before ： need body and when 
    let started_at = logging::now_ms();
    // requestclient（UA ， `core::client`）： in headers move 
    // before ，failurepath and use 1 
    let request_source = pipeline::request_source(&headers, key_identity);
    // ① body must is JSON for （//null ）
    let parsed = serde_json::from_slice::<Value>(&body).ok();
    let Some(mut payload) = parsed.filter(Value::is_object) else {
        let error = GatewayError::bad_request("请求体必须是 JSON 对象");
        record_early_failure(&state, started_at, "", "", &request_source, &error);
        return error.payload_response();
    };
    // ② messages must is 
    if !payload
        .get("messages")
        .map(Value::is_array)
        .unwrap_or(false)
    {
        let error = GatewayError::bad_request("缺少 messages 数组");
        let model = model_field_text(&payload);
        let reasoning = pipeline::client_reasoning_of(&payload);
        record_early_failure(&state, started_at, &model, &reasoning, &request_source, &error);
        return error.payload_response();
    }

    let method = "POST";
    let path = "/v1/chat/completions";
    let stream = payload.get("stream").map(|value| value == &Value::Bool(true)).unwrap_or(false);
    let message_count = payload
        .get("messages")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let user_agent = pipeline::user_agent_of(&headers);

    // ③ ： 1 request，（）
    write_debug_files(&body, method, path, &user_agent);

    // ④ by （default / / directory，
    // and ）。 down request in before 1 ：
    // request 「 down 」 is client ， not is after 。
    // is this Key available（None = unlimited）
    let client_model = model_field_text(&payload);
    // down etc and down 1 when （payload before ； model，
    // before after 1 ， client_reasoning_of ）
    let client_reasoning = pipeline::client_reasoning_of(&payload);
    let compact_client = compact_routing::client_for_request(&request_source.behavior_client, &payload, RequestFormat::Chat);
    let requested_model = match super::request_prepare::resolve_client_model(&state, &mut payload, scope.as_ref(), compact_client).await {
        Ok(model) => model,
        Err(error) => {
            record_early_failure(
                &state,
                started_at,
                &model_field_text(&payload),
                &client_reasoning,
                &request_source,
                &error,
            );
            return error.payload_response();
        }
    };
    logging::verbose(
        "[Model]",
        &format!(
            "← {method} {path} model={} stream={stream} msgs={message_count} bytes={} ua={user_agent}",
            if requested_model.is_empty() { "(未指定)" } else { &requested_model },
            body.len(),
        ),
    );

    // ⑤ ：request to （ in provider before by use 
    // ， `core::upstream::payload` `send_body`）；**client
    // request** ， and 「 1 is 」none。
    let dedupe_key = pipeline::sha256_hex(&body);
    let telemetry = Arc::new(RequestTelemetry::with_id());
    // 「 in 」： already success、 will —— 1 status=0 ，
    // request in response no when can to this request（ when by `record` UPDATE
    // ， `RequestStats::record_started`）。failurepath
    // （record_early_failure） no has id， not 。id telemetry that 
    // —— and 、 use is 1 。
    let telemetry_id = telemetry.snapshot().id;
    super::request_prepare::record_started(
            state.request_stats(),
            &telemetry_id,
            started_at,
            &requested_model,
            &client_model,
            &client_reasoning,
            &request_source,
            false,
        ).await;
    // ── （add）────────────────────────────
    // in （ = up this id）， 「request」 by 
    // to this in request； etc select 。
    // `DisconnectGuard` （path / path settle）。
    if let Some(token) = cancellation::register(&telemetry_id) {
        telemetry.set_cancel_token(token);
    }
    // ── （add）──────────────────────────────────
    // client in response before when axum will handler， not to ——
    // Drop 1 408 （ `DisconnectGuard` ）。
    // pathmust `complete()` / `handoff()`， down 。
    let mut guard = DisconnectGuard::new(state.request_stats(), telemetry_id.clone());
    // in ： 1 「 in 」、 1 「 up 」 this 
    // in （、、 in ）—— 1 seconds， is this in 
    // can to ， not etc 。 in this 、core has ， by 
    // `pipeline::live_row_sink`
    telemetry.set_live_sink(pipeline::live_row_sink(
        state.request_stats(),
        telemetry_id,
        started_at,
    ));
    // down request in 1 （request_raw request， `capture_raw_body`）：
    // body is client ； after payload will （default）。
    // **only when ** —— when this is None（ for no has ）
    let raw_request = pipeline::capture_raw_body(&body);
    // up down （ `autoContinue` **default**）。 when this 1 clone
    // not —— `plan()` ， need not need request/request/。
    // 、 and usage and in `core::upstream::continuation`。
    let continuation = continuation::plan().map(|max_rounds| Continuation {
        service: state.upstream().clone(),
        base_body: payload.clone(),
        stream,
        client_headers: headers.clone(),
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
            client_headers: headers,
            telemetry: telemetry.clone(),
            // （ when by ）； already in 
            // `resolve_model` （ that ， not ）
            allowed_providers: scope,
            // not ： by 
            pinned_account: None,
            // for request「 already in in 」（`ignore_model_gate`
            // is ， `ForwardRequest` ）
            ignore_model_gate: false,
            proxy_override: None,
        })
        .await;
    let stats = state.request_stats();

    match outcome {
        Ok(ForwardOutcome::Stream { status, stream: source }) => {
            // ：** not can ** in this —— this is 「response already 」，
            // in down 。 1 ， by in / when 
            //
            // in `RecordingStream` ** before **： need client can 
            // （all when and usage）， must in down 
            // to before —— will Responses/Anthropic before 。
            let source = match &continuation {
                Some(context) => context.stream(source),
                None => source,
            };
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
            let context = RecordContext {
                stats,
                telemetry,
                started_at,
                model: requested_model.clone(),
                client_model: client_model.clone(),
                client_reasoning: client_reasoning.clone(),
                status: i64::from(status.as_u16()),
                // request already ；response by RecordingStream in when 
                raw_request,
                raw_response: None,
                // ： not is （ `RecordContext::is_test`）
                is_test: false,
                // client：（ `RecordContext::client`）
                source: request_source.clone(),
            };
            // Chat ：client to `data: [DONE]` is ，
            // that when will 、`Drop` not will to EOF（ `RecordingStream`）
            // to response： not （ Drop has settle），
            // to （ can 「request」）
            guard.handoff();
            sse_response(
                status,
                Box::new(RecordingStream::with_terminals(
                    source,
                    context,
                    pipeline::terminal::CHAT,
                )),
            )
            .into_response()
        }
        Ok(ForwardOutcome::Completion { body }) => {
            // ： in this （ already to ， and 1 is 
            // ）。 no when `continuation` is None，body 1 not 。
            let body = match continuation {
                Some(context) => context.complete(body).await,
                None => body,
            };
            // response in before 1 （ JSON ；failure or 
            // to None， that 1 1 not ）
            let raw_response = pipeline::capture_raw_json(&body);
            record_entry(
                &RecordContext {
                    stats,
                    telemetry,
                    started_at,
                    model: requested_model.clone(),
                    client_model: client_model.clone(),
                    client_reasoning: client_reasoning.clone(),
                    status: 200,
                    raw_request,
                    raw_response,
                    // ： not is 
                    is_test: false,
                    // client：
                    source: request_source.clone(),
                },
                None,
            );
            // already ： and （request not in ）
            guard.complete();
            json_response(body)
        }
        Err(error) => {
            // headers no （ no ）→ to OpenAI error。
            // this in ： 1 message by `record_entry` 
            // fallback_error request 「error」（ Key 、
            // unavailable this ）—— not as 1 failure 
            // request 1 。
            logging::console_line("[Model]", &format!("❌ {}", error.message));
            let status = i64::from(error.http_status().as_u16());
            let message = error.message.clone();
            record_entry(
                &RecordContext {
                    stats,
                    telemetry,
                    started_at,
                    model: requested_model.clone(),
                    client_model: client_model.clone(),
                    client_reasoning: client_reasoning.clone(),
                    status,
                    // request use that （failurerequest request is 
                    // ； when is None）；response by 
                    // （error need already in ）， not 
                    raw_request,
                    raw_response: None,
                    // ： not is 
                    is_test: false,
                    // client：
                    source: request_source.clone(),
                },
                Some(message),
            );
            // already （ 408）： and 
            guard.complete();
            error.payload_response()
        }
    }
}

/// GET /v1/models —— （Node this no checkApiKey）。
///
/// response**directory**（Agent2API §4.4）： and 「current has available」
/// provider ，（ before that ），OpenAI response not 。
/// for has workbuddy 1 and has use ， and before 
/// （ `core::providers::catalog` module header）。
///
/// need need 「available」， this use `state.store()` ——
/// is `Clone` Arc， and not in when request（ this is ，
/// `accounts_for_provider` 1 file）。
///
/// `/v1/models` not ， Key when can 。
/// Key when mustvalid， by 1 。
pub async fn list_models(State(state): State<ServerState>, headers: HeaderMap) -> Response {
    let scope = if headers.contains_key("x-api-key") || headers.contains_key(axum::http::header::AUTHORIZATION) {
        match super::model_access::authenticate(&headers, "", true) {
            Ok(access) => access.scope,
            Err(error) => return error.payload_response(),
        }
    } else { None };
    let body = crate::server::core::providers::catalog::models_response(state.store(), scope.as_ref());
    // new directory， not response（ for Node `void refreshModelCatalog()`）
    spawn_catalog_refresh(&state);
    // Anthropic client（Claude Code / Claude Desktop） 1 path，
    // Anthropic （`type:"model"` + `display_name` +
    // `has_more`/`first_id`/`last_id`）， to OpenAI 
    // `{object:"list"}` will not 。 use `anthropic-version` 
    // —— that is Anthropic SDK ，OpenAI client not will 。
    //
    // ── Why not /anthropic/v1/models ───────────────────────
    // Claude Code is ** `{ANTHROPIC_BASE_URL}/v1/models`**
    // （path， not will before ），path etc failure。
    // by request is this 1 can 。
    if headers.contains_key("anthropic-version") || headers.contains_key("x-api-key") {
        return json_response(anthropic_models_view(&body));
    }
    raw_json(body)
}

/// directory OpenAI Anthropic 。
///
/// for （Anthropic List Models response）：
/// `data[].id` ← `id`
/// `data[].type` ← `"model"`
/// `data[].display_name` ← `name`（ id）
/// `data[].created_at` ← `created`（Unix seconds ISO； to current when ）
/// `has_more` / `first_id` / `last_id` by 
///
/// （`maxInputTokens` etc ）****：Anthropic client will ignore not 
/// ， its before （） need need 。
fn anthropic_models_view(body: &Value) -> Value {
    let empty: Vec<Value> = Vec::new();
    let items = body.get("data").and_then(Value::as_array).unwrap_or(&empty);
    let mut data: Vec<Value> = Vec::with_capacity(items.len());
    for item in items {
        // Note： this need is ****， not is `model_field_text`
        // —— that is request `model` ， 1 will to empty
        let text_of = crate::server::core::protocol::string_value;
        let id = item.get("id").map(text_of).unwrap_or_default();
        let display = {
            let name = item.get("name").map(text_of).unwrap_or_default();
            if name.is_empty() { id.clone() } else { name }
        };
        let mut entry = match item.as_object() {
            Some(map) => map.clone(),
            None => serde_json::Map::new(),
        };
        entry.insert("id".to_string(), Value::String(id.clone()));
        entry.insert("type".to_string(), Value::String("model".to_string()));
        entry.insert("display_name".to_string(), Value::String(display));
        // created_at：Anthropic use ISO-8601 ；directory no has create when ，
        // use 「directory new when 」（ 1 ）， not to 
        if let Some(refreshed) = body.pointer("/meta/lastRefreshedAt").and_then(Value::as_i64) {
            if refreshed > 0 {
                if let Some(utc) = chrono::DateTime::from_timestamp_millis(refreshed) {
                    entry.insert(
                        "created_at".to_string(),
                        Value::String(utc.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                    );
                }
            }
        }
        data.push(Value::Object(entry));
    }
    let first = data.first().and_then(|item| item.get("id")).cloned().unwrap_or(Value::Null);
    let last = data.last().and_then(|item| item.get("id")).cloned().unwrap_or(Value::Null);
    json!({
        "data": data,
        "has_more": false,
        "first_id": first,
        "last_id": last,
    })
}

/// POST /v1/messages/count_tokens
///
/// Anthropic this is ****（Claude Code when will ）。
/// has up ， up no has for tokenizer ——
/// this to 1 **** and `estimated: true`：
/// by 「 ÷ 3.5」（ in ， and Anthropic 「 3.5 
/// 1 token」 1 ）。
///
/// and `message_start` ** use ** 1 
/// （`protocol::anthropic::estimate_tokens_for_text`）：client will this 
/// ， for not up new （ issue #437）。
///
/// ── Why not 404 ────────────────────────────────────────
/// Claude Code in down will up down ；404 will by 「unsupported」
/// （ as ， can can is can can is ）。 to 1 ，
/// to 404 「 can use 」。 can ， not need 。
pub async fn count_tokens(State(state): State<ServerState>, body: Bytes) -> Response {
    let parsed = serde_json::from_slice::<Value>(&body).ok();
    let Some(payload) = parsed.filter(Value::is_object) else {
        let error = GatewayError::bad_request("请求体必须是 JSON 对象");
        return error.payload_response();
    };
    // use Anthropic → Chat statistics： this 「 / / system」
    // ， and when 1 （failure then by JSON ）
    let text = match crate::server::core::protocol::anthropic::chat_from_anthropic(&payload) {
        Ok(chat) => serde_json::to_string(&chat).unwrap_or_default(),
        Err(_) => serde_json::to_string(&payload).unwrap_or_default(),
    };
    let tokens = crate::server::core::protocol::anthropic::estimate_tokens_for_text(&text);
    let _ = state;
    json_response(json!({
        "input_tokens": tokens,
        "estimated": true,
    }))
}

/// 1 after new directory（ not currentresponse）。
///
/// new ****（`providers::adapter::refresh_implemented`）：
/// already provider （workbuddy is /v3/config），
/// not 1 。W3 up after this 1 not use 。
///
/// must use `crate::spawn_task` not is `tokio::spawn`：
/// in axum handler use （ when up down already ），
/// 1 spawn ， after has to Tokio up down when panic
/// （release is panic=abort， that will ）。
pub fn spawn_catalog_refresh(state: &ServerState) {
    if !config::scheduled_settings().model_refresh.enabled {
        return;
    }
    let store = state.store().clone();
    crate::spawn_task(async move {
        crate::server::core::providers::adapter::refresh_implemented(&store).await;
    });
}