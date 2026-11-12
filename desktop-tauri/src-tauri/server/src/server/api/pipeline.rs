//! for ：、request、response、。
//!
//! ── Why need need this ──────────────────────────────────────
//! for for （`/v1/chat/completions`、`/v1/responses`、
//! `/v1/messages`）， ** in ** up ， its ：
//! 1. （default → → use / in ）
//! 2. request（ use request 1 ， need to after ）
//! 3. response（JSON / SSE and ）
//! this from `api/chat.rs` to this ， use 1 —— otherwise 1 
// （e.g.、default） need in file 1 ，
// will not 1 as 。
//!
//! ── and `api/chat.rs` ─────────────────────────────────
//! `chat.rs` is for （`/v1/chat/completions` handler）， in from 
//! ；`api/protocol.rs` new 。** not 
//! **（ not messages / input / content）， 1 in handler 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::Response;
use serde_json::Value;

use crate::server::config;
use crate::server::core::key_scope::{self, KeyScope};
use crate::server::core::limiter;
use crate::server::core::providers::catalog::{
    advertised_manifest_contains, default_model_catalog, default_model_usable,
    has_available_providers, model_blocked_everywhere, suggest_advertised,
};
use crate::server::core::upstream::usage::{self, RequestTelemetry, TelemetrySnapshot};
use crate::server::errors::GatewayError;
use crate::server::logging;
use crate::server::request_stats::{
    AttemptDetail, NewRequestEntry, RequestStats, RetryEvent, RunningProgress, SensitiveHit,
};
use crate::server::ServerState;

mod raw;
pub use raw::{capture_raw_body, capture_raw_json, raw_body_text};
use raw::RawCapture;

/// directory and file（Node `join(CONFIG_DIR, 'debug', ...)`）
const DEBUG_DIR: &str = "debug";
const DEBUG_REQUEST_FILE: &str = "last-request.json";
const DEBUG_META_FILE: &str = "last-request.meta.txt";

/// request error need limit；requesterror。
const ERROR_SUMMARY_CHARS: usize = 200;

/// client in / response before when error need 。
///
/// HTTP （2xx）， can this 「Why no has token」。
pub const STREAM_ABORTED: &str = "响应流未完整下发（客户端中断或服务退出）";

/// 1 「」 （ [`RecordingStream`] ）。
///
/// by ** use ** by that to （），
/// for ， not —— this is `api::pipeline` 
/// （module header：「 not 」）。
///
/// **must**（ 1 ）： in up is 1 
/// （`data:[DONE]\n` / `event: response.completed\n`）， up `\n` not will in 
/// —— can 1 `data: [DONE]` ， that in will 
/// 1 ** ** in success。 1 this ：JSON must
/// （`\n` in up is n），SSE `data:` not ，
/// 「 1 」 can can in up 。 this is ： can 
/// （ before as ， is 1 need ）， not can in success。
pub type TerminalFrames = &'static [&'static [u8]];

/// SSE （ that ）。
///
/// in this not is in use ： and is ** for **
/// ， in 1 can 1 「 for up 」。
pub mod terminal {
    /// Chat Completions：`data:[DONE]`， after can empty、 can as LF / CRLF。
    ///
    /// ── Why is not is that 1 ────────────────────────
    /// （2026-10-02，`requests`×`request_raw` ， no limit 
    /// 383 「response down 」）：**381 response has DONE **，
    /// is is `data:[DONE]\n\n` —— after ** no has that empty**。
    /// `data: [DONE]\n\n`， is **** CodeArts（1176 ， in 
    /// 98%） 1 by in ， 「failure token」 1 usage。
    /// `ReasoningCoalescer`（`upstream::sse`） path by that ，
    /// empty、 1 is in —— this Why this bug in this 1 up 。
    ///
    /// SSE after ** that 1 **empty is optional （ when 1 ），
    /// must， not is need up 。`\r\n` 1 to ： 1 
    /// is 1 ， 1 is 1 in 。
    ///
    /// need ** 1 **（ before need ）： that empty not is 1 ，
    /// 「DONE after no has empty」 will ， is （
    /// [`super::TerminalFrames`] ： can can in ）。
    pub const CHAT: super::TerminalFrames = &[
        b"data: [DONE]\n",
        b"data:[DONE]\n",
        b"data: [DONE]\r\n",
        b"data:[DONE]\r\n",
    ];
    /// Responses： `response.completed`，failure `response.failed`
    pub const RESPONSES: super::TerminalFrames =
        &[b"event: response.completed\n", b"event: response.failed\n"];
    /// Anthropic Messages：`message_stop`（success and error）
    pub const ANTHROPIC: super::TerminalFrames = &[b"event: message_stop\n"];
}

// ─── ───────────────────────────────────────────────

/// and request ，return** use ** 。
///
/// （ and before `/v1/chat/completions` 1 ）：
/// ① → default（config defaultModel → directory isDefault → directory）
/// ② by （ + ） → 404（`model_not_found`）
/// ③ ** no has ** → 404（，`model_not_found`）
///
/// ── ② and ③ Why is 404（）────────────────────
/// this is 「 this for （ down ） not in 」：② is 、
/// ③ is directory no has 。 before use 400 「request has 」， OpenAI 
/// for 「 not in 」 is **404 + code=model_not_found**（OmniProxy 
/// `fail(res, ..., 404, 'model_not_found')` 1 ，client and new 
/// ）。 use 404 1 not 1 ：R9 that is 404，
/// 1 code down path in 1 。
///
/// ── ③ Why as （「 no has 」）──────────────
/// client `/v1/models`； 1 that no has ，
/// 404， not is to up 1 and Reasonnone up 。 
/// （Cline by 、 no has available 、 ）** not 
/// can use ** —— and 1 ， not in 「 not to can 」
/// in 。 `catalog::advertised_manifest_contains`。
///
/// ****（ not this ， to to error）：
/// - client** no **（ ① default）：default is ，
/// not this ；
/// - ** 1 available no has **（ no ）： when as empty，
/// 1 「 not in 」 will 「 no has available，none can 」 this 
/// can 。 1 for down （ by that ）。
///
/// ── Why not in this （ OmniProxy ）──────
/// before is 「request in alias → payload target」：request
/// and up id when ， will **** that up （ by not to 
/// ）， old 。 after （、
/// ）， down to ** by **（`payload::send_body` →
/// `catalog::wire_target_for_provider`）：requestclient，
/// to 、 to target —— in 
/// `providers::router::route_for_forward`。
///
/// `payload` will （onlydefault）： after and use 1 request，
/// client body in use （ handler）。
///
/// ── `scope`： Key available（R9）────────────────────
/// `None` unlimited； use must Key 。
/// has scope when in ** this **， not is to ： is 
/// 「 this for this Key not in 」， and 「 not in directory」
/// （404 + `model_not_found`， OmniProxy 
/// `if (!keyAllowsModel(...)) return fail(res, \` '${publicModel}' unavailable\`, 404, 'model_not_found')`）——
/// 「 in 」 not this to down ： can not ， is 404 1 。
///
/// defaultmust to Key ；check not 。
pub fn resolve_model(
    state: &ServerState,
    payload: &mut Value,
    scope: Option<&KeyScope>,
) -> Result<String, GatewayError> {
    resolve_model_for(state, payload, scope, false)
}

pub fn resolve_image_model(
    state: &ServerState,
    payload: &mut Value,
    scope: Option<&KeyScope>,
) -> Result<String, GatewayError> {
    resolve_model_for(state, payload, scope, true)
}

fn resolve_model_for(
    state: &ServerState,
    payload: &mut Value,
    scope: Option<&KeyScope>,
    image_generation: bool,
) -> Result<String, GatewayError> {
    if let Some(object) = payload.as_object_mut() {
        object.remove(crate::server::core::compact_routing::ROUTE_FIELD);
        object.remove("_wb_route_group_id");
        object.remove("_wb_route_continue");
    }
    let requested = model_field_text(payload);
    // client is ****：default after this ， down 
    let client_named = !requested.is_empty();
    if !client_named {
        // client no → default。 
        // `default_model_catalog()`（ by 「 not default」 after ）：
        // 1 not when not model， by 「」（ up use default）
        let snapshot = config::current();
        let default_model = snapshot.default_model();
        // ：unlimited when `allowed` ， etc （ as ）
        let allowed = |name: &str| key_scope::allows_model(scope, name);
        let models = default_model_catalog();
        let fallback = if (config::routing_profiles().group_for_model(default_model).is_some() || default_model_usable(default_model)) && allowed(default_model) {
            Some(default_model.to_string())
        } else {
            models
                .iter()
                .find(|model| {
                    model.get("isDefault").map(value_is_truthy).unwrap_or(false)
                        && model
                            .get("id")
                            .map(crate::server::core::models::shape_value_text)
                            .map(|id| allowed(&id))
                            .unwrap_or(false)
                })
                .or_else(|| {
                    models.iter().find(|model| {
                        model
                            .get("id")
                            .map(crate::server::core::models::shape_value_text)
                            .map(|id| allowed(&id))
                            .unwrap_or(false)
                    })
                })
                .and_then(|model| model.get("id"))
                .filter(|id| !id.is_null())
                .map(|id| match id {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                })
        };
        if let Some(fallback) = fallback {
            if let Some(object) = payload.as_object_mut() {
                object.insert("model".to_string(), Value::String(fallback));
            }
        } else if scope.is_some_and(KeyScope::restricts_models) {
            // this Key limit、directory** no has 1 is **：
            // down will up default（），
            // in this 。 to this （ that ）。
            return Err(GatewayError::bad_request(
                "这把网关 Key 的可用模型列表里没有任何当前可用的模型：请到「网关 Key」页为它勾选可用模型",
            )
            .with_code("model_not_found"));
        }
    }
    let requested_model = model_field_text(payload);
    // empty by is already ；check，unavailableReason to 。
    if let Some(group) = config::routing_profiles().group_for_model(&requested_model) {
        if !key_scope::allows_model(scope, &requested_model) {
            return Err(GatewayError::with_status(404, "此路由组不在网关 Key 的可用模型列表里").with_code("model_not_found"));
        }
        if image_generation { return Err(GatewayError::bad_request("对话路由组不能用于图片生成")); }
        if let Some(object) = payload.as_object_mut() {
            object.insert("model".into(), group.model.clone().into());
            object.insert("_wb_route_group_id".into(), group.id.clone().into());
        }
        config::remember_request_model(&group.model);
        return Ok(group.model.clone());
    }
    // by after ，404 is 「 by （ + ）unavailable」
    if !requested_model.is_empty() && model_blocked_everywhere(&requested_model) {
        return Err(GatewayError::with_status(404, format!(
            "模型已在网关中关闭: {requested_model}。完整列表见 GET /v1/models"
        ))
        .with_code("model_not_found"));
    }
    // ③ no has → 400（「 no has 」；）
    if client_named && !requested_model.is_empty() {
        // ：** 1 available no has **（ no ） when —— 
        // as empty， 1 「 not in 」 will 「 no has available，none can 」
        // this can （ ）。
        // ── this active use ** by Key ** that 1 （）───────────
        // `/v1/models` by Key after （
        // `catalog::models_response`）， this use ： 
        // 「 in 、 this Key not 」 this 1 。
        // that **must by **（`provider_loop::forward_with_providers`
        // available）， as has has 、 can 「 
        // this Key 」； in this by 、 to to ，
        // in this use after 1 「 not in 」 for use has use 
        //（ after will ， need is Key available）。
        if has_available_providers(state.store())
            && !advertised_manifest_contains(state.store(), &requested_model)
        {
            let hint = suggest_advertised(state.store(), &requested_model, 5);
            let message = format!(
                "模型不存在: {requested_model}{}。完整列表见 GET /v1/models",
                if hint.is_empty() {
                    String::new()
                } else {
                    format!("（目录里相近的模型: {}）", hint.join("、"))
                },
            );
            return Err(GatewayError::with_status(404, message).with_code("model_not_found"));
        }
    }
    if !requested_model.is_empty() && !key_scope::allows_model(scope, &requested_model) {
        return Err(GatewayError::with_status(404, format!(
            "模型 '{requested_model}' 不可用：不在这把网关 Key 的可用模型列表里"
        )).with_code("model_not_found"));
    }
    if !image_generation && !requested_model.is_empty() {
        let providers = crate::server::core::providers::catalog::forwarding_providers(&requested_model);
        if !providers.is_empty() && providers.iter().all(|provider| {
            crate::server::core::providers::images::provider_model_is_image(provider, &requested_model)
        }) {
            return Err(GatewayError::bad_request(format!(
                "模型 '{requested_model}' 是图片生成模型，请使用 POST /v1/images/generations"
            )).with_code("image_generation_required"));
        }
    }
    // use （default already ），default in 
    if !requested_model.is_empty() && !image_generation {
        config::remember_request_model(&requested_model);
    }
    Ok(requested_model)
}

/// request model → / use 。
///
/// Node ：`if (!body.model)` （null/""/0/false 
/// ）， to directory（`get()` is `String(id).toLowerCase()`），
/// / for this will after and for —— is 
/// that ， and ** not payload **。
pub fn model_field_text(payload: &Value) -> String {
    let Some(value) = payload.get("model") else {
        return String::new();
    };
    if !value_is_truthy(value) {
        return String::new();
    }
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// down requestclient etc （request「 down etc 」；empty = no ）。
///
/// and [`model_field_text`] 1 when （ handler in payload after 1 ，
/// [`RecordContext`] 1 to ）。 is 
/// `model_rules::reasoning::read_client_level` —— that is ** and 、
/// **： when 「client no has 」 by resolver ，
/// （CatPaw empty will 400、Qoder to 1 as ） not this need 
/// 。Note payload before can can already [`resolve_model`] —— 
/// `model` ， etc not ， before after 1 。
pub fn client_reasoning_of(payload: &Value) -> String {
    crate::server::core::model_rules::read_client_level(payload).unwrap_or_default()
}

/// JS （`Boolean(x)`）
pub fn value_is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// from request User-Agent（ use ； to `-`）
pub fn user_agent_of(headers: &axum::http::HeaderMap) -> String {
    headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("-")
        .to_string()
}

pub fn request_source(
    headers: &axum::http::HeaderMap,
    identity: Option<axum::extract::Extension<crate::server::request_stats::ApiKeyIdentity>>,
) -> crate::server::request_stats::RequestSource {
    let behavior_client = crate::server::core::client::classify(
        headers.get(header::USER_AGENT).and_then(|value| value.to_str().ok()).unwrap_or(""),
    );
    let snapshot = crate::server::config::client_identification::current();
    let defaults = crate::server::core::client::ClientSettings::default();
    let (client, evidence) = crate::server::core::client::identify(
        crate::server::core::client::ClientEvidence::capture(headers),
        snapshot.settings.as_ref().unwrap_or(&defaults),
    );
    crate::server::request_stats::RequestSource {
        client,
        api_key: identity.map(|axum::extract::Extension(identity)| identity).unwrap_or_default(),
        evidence: Some(evidence),
        behavior_client,
    }
}

// ─── request（requeststatistics 1 ）────────────────────────

/// 1 request up down ： need to response， etc when 。
///
/// Why is 「」 not is use ：`RecordContext` will response，
/// response is `'static`（ need handler ）。
pub struct RecordContext {
    /// statistics（`Arc` ， and ServerState that is 1 ）
    pub stats: Arc<RequestStats>,
    /// usage / 
    pub telemetry: Arc<RequestTelemetry>,
    /// request when （ms Unix when ）
    pub started_at: i64,
    /// **request** （default、directory already ；** not **
    /// —— client when is ， `resolve_model` ）。
    /// request ； by ** not ** use （ will 
    /// ），statistics is `upstream_model`（empty，
    /// `fold_into_daily` `model_stat_key`）。
    pub model: String,
    /// ** down request **（clientrequest ， / default before ；
    /// client no when as empty）。request use and up （telemetry 
    /// `upstream_model`）「request → 」。
    pub client_model: String,
    /// ** down request**client etc （`max` etc ；empty = no ）。
    /// request to down `( etc )` after ； and up 
    /// `snapshot.upstream_reasoning` —— 「client need 」 and 「 
    /// 」 is （client no when can 1 ， can can and ）。
    /// when and [`Self::client_model`] （ payload after 1 ）。
    pub client_reasoning: String,
    /// down to client HTTP 
    pub status: i64,
    /// ** down request**（`request_raw` request；，None = ）。
    pub raw_request: Option<String>,
    /// **response**（`request_raw` response）。
    /// by ， in when from 。
    pub raw_response: Option<String>,
    /// this request is not is **** （default false，
    /// `request_stats::RequestEntry::is_test`）。
    ///
    /// has `api::model_test` true： and request 1 、
    /// ， 1 is 「 」—— when need can （excluding、
    /// request）。
    pub is_test: bool,
    /// statistics、 and already Key 。
    pub source: crate::server::request_stats::RequestSource,
}

/// before failure（body / / not in directory） when 。
///
/// this request** 1 no up **， attempts 0 will 1
/// （ is 「、 ≥1」）。 use 1 「 1 」，
/// this and 「 up N 」 in is ， status and error 。
///
/// Why this need ：`model_not_found` is clientconfigerror ，
/// not use in not to 「request in failure」， will as statistics。
///
/// `client` by use from request（`core::client::classify`）：failure
/// need need 「 is 」（client is ）； not to 
/// headers pathempty（ `RequestEntry::client`）。
pub fn record_early_failure(
    state: &ServerState,
    started_at: i64,
    model: &str,
    client_reasoning: &str,
    source: &crate::server::request_stats::RequestSource,
    error: &GatewayError,
) {
    let context = RecordContext {
        stats: state.request_stats(),
        telemetry: Arc::new(RequestTelemetry::new()),
        started_at,
        client_model: model.to_string(),
        client_reasoning: client_reasoning.to_string(),
        model: model.to_string(),
        // and `payload_response` 1 ： will 1 500
        status: i64::from(error.http_status().as_u16()),
        // before failure： no has id（telemetry is empty ）、 no has ——
        // request can can no success， no has 
        raw_request: None,
        raw_response: None,
        is_test: source.is_internal(),
        source: source.clone(),
    };
    record_entry(&context, Some(error.message.clone()));
}

/// in ： telemetry 1 that 「 in 」。
///
/// ── ──────────────────────────────────────────────
/// `record_started` in has id / ts / —— and 
/// in after ， that 1 not 「 in 、 is 、
/// already 」。 this in request before already ， is before no has 
/// （`requests` up has and path）。
///
/// ── Why is not is telemetry has `Arc<RequestStats>` ──
/// `core::upstream` not `request_stats`（ to `core/mod.rs` ），
/// 「 → 」 in （`record_entry` when is 1 
/// ， use down `stored_*` ）。 in this ，core in 
/// when 1 down 。
///
/// `started_at` by use ： in telemetry is ** for when **，
/// that 1 is for request ms（ `RequestEntry::first_response_ms`）——
/// can in this ，telemetry not request when 。
pub fn live_row_sink(
    stats: Arc<RequestStats>,
    id: String,
    started_at: i64,
) -> impl Fn(&TelemetrySnapshot) + Send + Sync {
    move |snapshot| {
        stats.update_running(
            &id,
            &RunningProgress {
                provider: snapshot.provider.clone().unwrap_or_default(),
                account_id: snapshot.account_id.clone(),
                account_name: snapshot.account_name.clone(),
                upstream_model: snapshot.upstream_model.clone(),
                upstream_reasoning: snapshot.upstream_reasoning.clone(),
                // and ： 1 no （0） by 1 ， by 
                // `TelemetrySnapshot::attempts` 
                attempts: snapshot.attempts.max(1),
                first_response_ms: snapshot.first_response_at.map(|at| (at - started_at).max(0)),
                // and ： as None can can is 「 in （ in ）、
                // 1 no 」—— that is **request when **（ down this 
                // started_at， and in when phase_started_at ），
                // use started_at 。 not this 1 will NULL，
                // by 「 in already 」 will or 
                // no has —— not this in 「 already no 」 request up 
                phase: snapshot.phase.as_str().to_string(),
                phase_started_at: snapshot.phase_started_at.or(Some(started_at)),
                attempt_details: stored_attempt_details(&snapshot.attempts_detail),
                route: snapshot.route.clone(),
                sensitive_hits: stored_sensitive_hits(&snapshot.sensitive_hits),
            },
        );
    }
}

/// telemetry → 。
///
/// （`record_entry`） and in use this 1 ： 1 ， will as 
/// 「 1 」 in and 。
fn stored_attempt_details(list: &[usage::AttemptDetail]) -> Vec<AttemptDetail> {
    list.iter()
        .map(|item| AttemptDetail {
            provider: item.provider.clone(),
            account: item.account.clone(),
            account_id: item.account_id.clone(),
            candidate_id: item.candidate_id.clone(),
            member_id: item.member_id.clone(),
            model: item.model.clone(),
            reasoning: item.reasoning.clone(),
            reason: item.reason.clone(),
            sticky_hit: item.sticky_hit,
            started_at: item.started_at,
            response_at: item.response_at,
            finished_at: item.finished_at,
            status: item.status,
            error: item.error.clone(),
            retries: item
                .retries
                .iter()
                .map(|retry| RetryEvent {
                    reason: retry.reason.clone(),
                    status: retry.status,
                    delay_ms: retry.delay_ms,
                })
                .collect(),
            notice: item.notice.clone(),
            // （None = this 1 no ； by usage ）
            body_bytes: item.body_bytes,
        })
        .collect()
}

/// telemetry in → （ and [`stored_attempt_details`] 1 by ）
fn stored_sensitive_hits(list: &[usage::SensitiveHit]) -> Vec<SensitiveHit> {
    list.iter()
        .map(|hit| SensitiveHit {
            word: hit.word.clone(),
            count: hit.count,
        })
        .collect()
}

/// 1 request。
///
/// ── Why not can request ─────────────────────────────────
/// `RequestStats::record` not return `Result`： for failure
/// `[Stats] statisticsfailure`， in `poisoned.into_inner()` use ，
/// failure this —— already 「statisticsfailure」「 1 」，
/// no has unwind path， this not need need `catch_unwind`。
///
/// ── ────────────────────────────────────────────────
/// ts request**** when （ not is when ）： need by 「 use when 
/// request」
/// durationMs - （、、 up etc 、 down ）
/// firstResponseMs to - ； no has to when as null
/// attempts up request； 1 no when 1
/// error Reason（），otherwise use use to 
pub fn record_entry(context: &RecordContext, fallback_error: Option<String>) {
    let mut snapshot = context.telemetry.snapshot();
    // when 1 ： durationMs and that 1 is 1 
    let finished_at = logging::now_ms();
    let duration_ms = finished_at - context.started_at;
    let first_response_ms = snapshot
        .first_response_at
        .map(|at| (at - context.started_at).max(0));
    let error = snapshot.error.clone().or(fallback_error).map(|text| {
        if context.raw_request.is_some() || context.raw_response.is_some()
            || context.telemetry.capture().is_some() || crate::server::core::debug_traffic::enabled()
        {
            text
        } else {
            truncate_chars(&text, ERROR_SUMMARY_CHARS)
        }
    });
    snapshot.finish_trace(finished_at, error.as_deref());
    let attempts = snapshot.attempts.max(1);
    let mut entry = NewRequestEntry::new(context.model.clone(), context.status);
    entry.ts = Some(context.started_at);
    // id： by （ `core::debug_traffic`）。
    // empty = this request no id（ before failure path）， before not 。
    // clone not is move：snapshot.id down to store_raw use （ 1 ）
    entry.id = snapshot.id.clone();
    entry.duration_ms = duration_ms;
    entry.first_response_ms = first_response_ms;
    entry.attempts = attempts;
    // this is String（ not is Option），empty is 「 no has 」 
    entry.account_id = snapshot.account_id;
    entry.account_name = snapshot.account_name;
    entry.provider = snapshot.provider;
    // down / up ： down is client（context ）， up is 
    // （telemetry ，empty = 1 no ）。empty when 
    // before model 1 —— old no has this ， 1 。
    entry.client_model = context.client_model.clone();
    entry.upstream_model = snapshot.upstream_model;
    // etc （ and up ）： down etc （context），
    // up etc （telemetry ）
    entry.client_reasoning = context.client_reasoning.clone();
    entry.upstream_reasoning = snapshot.upstream_reasoning;
    // （）： is 「 has has 」 ， in 
    // up （`core::upstream::usage` ）， this 。
    // · attemptDetails： up （provider / / / error need 
    // / / ），request「」 ；
    // before failure request no has （empty）。
    // · sensitiveHits： in and ， 1 。
    // and **、current**（ `RequestEntry` 
    // ）， this 1 —— not `From` impl is as can 
    // （ 「」 and 「」 1 ， will 1 
    // when 1 ， by not ）。
    // `stored_*` ： in （`live_row_sink`） use is 1 ，
    // 1 will in and 。
    entry.attempt_details = stored_attempt_details(&snapshot.attempts_detail);
    entry.sensitive_hits = stored_sensitive_hits(&snapshot.sensitive_hits);
    // ： has （ no to request is Null， before not that 1 ）
    entry.route = snapshot.route.unwrap_or(Value::Null);
    // ： ，excluding（ this ）
    entry.is_test = context.is_test;
    // client：request （`core::client::classify`），
    // not —— UA then has 1 
    entry.source = context.source.clone();
    entry.error = error;
    entry.prompt_tokens = snapshot.prompt_tokens;
    entry.completion_tokens = snapshot.completion_tokens;
    entry.total_tokens = snapshot.total_tokens;
    entry.cache_read_tokens = snapshot.cache_read_tokens;
    // Token limit （ record：entry already move）。
    // request current， Token not etc down 1 
    // 10 seconds new 。account_id as empty = no to （ before failure path），
    // tokens ≤ 0 in will —— failurerequest no has usage， is 。
    let limited_account = entry.account_id.clone();
    let limited_tokens = entry.total_tokens;
    let limited_model = if entry.upstream_model.is_empty() { entry.model.clone() } else { entry.upstream_model.clone() };
    let limited_request = entry.id.clone();
    let sent = snapshot.attempts > 0;
    context.stats.record(entry);
    if sent {
        limiter::counter::note(&limited_account, &limited_request, &limited_model,
            &snapshot.quota_channel, limited_tokens, context.started_at);
    }
    if !limited_account.is_empty() {
        limiter::note_request_tokens(&limited_account, limited_tokens, finished_at);
    }
    // （request_raw ）： and id、 when 。 record
    // 1 （ not path， by `RequestStats::store_raw`）；
    // in already by gate（ `capture_raw_body` / `capture_raw_json`
    // —— this is None，store_raw before return not ）；id as empty / 
    // empty in store_raw down ，failure —— is ，
    // 1 not can already request
    context.stats.store_raw(
        &snapshot.id,
        context.started_at,
        context.raw_request.as_deref(),
        context.raw_response.as_deref(),
    );
    logging::verbose(
        "[Stats]",
        &format!(
            "记一条请求: model={} status={} {duration_ms}ms 首响={} attempts={attempts} tokens={}+{}（缓存 {}）",
            if context.model.is_empty() { "(未指定)" } else { &context.model },
            context.status,
            first_response_ms
                .map(|value| format!("{value}ms"))
                .unwrap_or_else(|| "-".to_string()),
            snapshot.prompt_tokens,
            snapshot.completion_tokens,
            snapshot.cache_read_tokens,
        ),
    );
}

/// by （ use `…` ）。
pub fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out: String = text.chars().take(limit).collect();
    out.push('…');
    out
}

/// response。
///
/// ── Why 1 （ not is in handler ）────────────────────
/// request 「 use 」 is ** after 1 **， handler in 
/// response when return。 1 after ， durationMs down ，
/// client in 、 up this 「」 can 1 。
///
/// ── is ──────────────────────────────────────────
/// not ： for `Item` （ 1 「 is not is to None 」 
/// ）， not 、 not 、 not ， not error。
///
/// ── Why need 「」（`terminals`）─────────────────────
/// up that 「 to `None` success」 ， for ** by ** client is not ：
/// in to when already this 1 ，****， from not to EOF。
/// is `Drop` that 「 no to None」 will —— 1 **success** request 
/// 「response down 」， and 「failure token」 1 （`NewRequestEntry::normalize`）
/// usage 1 and 。 this not is ： need client this ，****。
///
/// to client this ：
/// - Codex Desktop（`/v1/responses`）： to `response.completed` ；
/// - to `data: [DONE]` Chat client（ Node 
/// `reader.cancel()` ）：`/v1/chat/completions` in 。
///
/// 1 ：** already down ** ⇒ this 1 in already ，
/// after client。 by use to （），
/// for 。
///
/// Note this not 「 to 」 ： in （client in before ）
/// by `STREAM_ABORTED` ，`error` by telemetry。
struct TerminalScan {
    /// 
    frames: TerminalFrames,
    /// chunk ： can can TCP 
    ///
    /// 「 - 1」 —— up that 1 ， not will 
    /// responsenone（ 1 SSE response KB）。
    window: Vec<u8>,
    /// already in （ etc ， in after not ）
    seen: bool,
}

impl TerminalScan {
    fn new(frames: TerminalFrames) -> Self {
        Self { frames, window: Vec::new(), seen: false }
    }

    /// 1 down ，return「 is in 」。
    fn push(&mut self, chunk: &[u8]) -> bool {
        if self.seen || self.frames.is_empty() {
            return false;
        }
        // = up + ： TCP when ， this up 
        self.window.extend_from_slice(chunk);
        let hit = self
            .frames
            .iter()
            .any(|frame| contains(&self.window, frame));
        // 「 - 1」 ： this not up 1 ，
        // will responsenone（ 1 SSE response KB）
        let longest = self.frames.iter().map(|frame| frame.len()).max().unwrap_or(0);
        let keep = longest.saturating_sub(1);
        if self.window.len() > keep {
            let drop = self.window.len() - keep;
            self.window.drain(..drop);
        }
        if hit {
            self.seen = true;
        }
        hit
    }
}

/// `haystack` is `needle`（empty needle ）。
///
/// ： 30 、 not ，
/// response this can ignore（ for down ， as 1 
/// or KMP is ）。
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack.windows(needle.len()).any(|window| window == needle)
}

pub struct RecordingStream {
    inner: Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
    /// up down ；`take()` 「 already 」（ 1 ）
    context: Option<RecordContext>,
    /// （`None` = this no ， etc before as ）
    terminal: Option<TerminalScan>,
    /// response（`request_raw` response， [`RawCapture`]）
    raw: RawCapture,
}

impl RecordingStream {
    /// （ [`RecordingStream`] ）。
    ///
    /// `frames` by use by that to （`terminal::CHAT` etc ）——
    /// **must** that ： etc before as （client by when 
    /// successrequest in ）， this not 「 not 」 。
    pub fn with_terminals(
        inner: Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
        context: RecordContext,
        frames: TerminalFrames,
    ) -> Self {
        let terminal = if frames.is_empty() { None } else { Some(TerminalScan::new(frames)) };
        // response and request 1 ： not （）——
        // this 1 ，request response as after not （ `RawCapture` ）
        let raw = RawCapture::new(crate::server::core::debug_traffic::enabled());
        Self { inner, context: Some(context), terminal, raw }
    }

    /// and empty up down （ etc ： use not ）。
    ///
    /// response in this ： （ to None / ） is 「」
    /// or 「 this 」 when ， after not has 。
    fn settle(&mut self, fallback_error: Option<String>) {
        if let Some(mut context) = self.context.take() {
            // path when raw_response as None， this use ；
            // if has response， that not this （ ）
            context.raw_response = self.raw.into_text();
            record_entry(&context, fallback_error);
            // in ： = request not in （ and 
            // DisconnectGuard::complete 1 ；settle by Drop 
            // use ，「client」「」「」 will to this ）
            crate::server::core::upstream::cancellation::unregister(&context.telemetry.id());
        }
    }

    /// is already down （`true` = this 1 in already ）
    fn terminal_seen(&self) -> bool {
        self.terminal.as_ref().is_some_and(|scan| scan.seen)
    }
}

impl futures::Stream for RecordingStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::StreamExt;
        // is Unpin， Unpin，get_mut is 
        let this = self.get_mut();
        // already （ up already ）→ not poll up ，
        if this.context.is_none() {
            return std::task::Poll::Ready(None);
        }
        let polled = this.inner.poll_next_unpin(cx);
        match &polled {
            std::task::Poll::Ready(Some(Ok(bytes))) => {
                // ：** down 1 ** to when 1 （ in up not is 
                // up up ， is path in as 1 ）
                if let Some(context) = &this.context {
                    context.telemetry.note_first_frame();
                }
                // response， not 。
                this.raw.push(bytes);
                // ： in client down when can can ， that 
                if let Some(scan) = &mut this.terminal {
                    if scan.push(bytes) {
                        logging::verbose(
                            "[Stats]",
                            "已下发协议收尾帧：此后客户端断开按正常收尾记账",
                        );
                    }
                }
            }
            std::task::Poll::Ready(Some(Err(_))) => {}
            std::task::Poll::Ready(None) => {}
            std::task::Poll::Pending => {}
        }
        if matches!(polled, std::task::Poll::Ready(None)) {
            this.settle(None);
        }
        polled
    }
}

impl Drop for RecordingStream {
    fn drop(&mut self) {
        // to this response** no has ** to None 。：
        // · already down → client by （）， not need ；
        // · otherwise is before （client in / ）→ 1 in need 。
        // 1 —— request， already to settle 。
        if self.terminal_seen() {
            self.settle(None);
        } else {
            self.settle(Some(STREAM_ABORTED.to_string()));
        }
    }
}

/// 1 response「 + 」 use 。
///
/// ── Why is 「、 after 」────────────────────────────
/// `RecordingStream` is and ，must to ** down ** 
/// （client ）。（Responses / Anthropic ）
/// in ****， is ：
/// → RecordingStream → axum Body
/// when is 「 after 1 to client」 when ， and client 1 。
///
/// `transform` is 1 「」 ： has ，
/// not 。
///
/// `terminals` is ** **， by use by that to 
/// （ [`RecordingStream`] ）—— for ， not 。
///
/// ── Why `Box<dyn Stream>` not is `S` ────────────────
/// need need `S: Unpin`（`flat_map` not Unpin），
/// use is `ForwardOutcome::Stream` to —— 
/// 1 parameter， not in use `Unpin` 。
pub fn transformed_stream(
    source: Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
    context: RecordContext,
    terminals: TerminalFrames,
    mut transform: impl FnMut(&[u8]) -> Vec<Bytes> + Send + 'static,
) -> Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin> {
    use futures::StreamExt;
    // up → after 
    let converted = source.flat_map(move |item| {
        let frames = match item {
            Ok(chunk) => transform(&chunk),
            // up ： error（ by axum ）
            Err(error) => return futures::stream::iter(vec![Err(error)]).boxed(),
        };
        futures::stream::iter(frames.into_iter().map(Ok)).boxed()
    });
    Box::new(RecordingStream::with_terminals(Box::new(converted), context, terminals))
}

// ─── response ───────────────────────────────────────────────

/// response：`Content-Type: application/json; charset=utf-8` + 200。
///
/// 200： up 2xx when already （ not will to this ），
/// and Node `res.writeHead(200, ...)` 1 。charset up ——
/// response has in （error/）。
pub fn json_response(body: Value) -> Response {
    let text = match serde_json::to_string(&body) {
        Ok(text) => text,
        Err(error) => {
            // failure can can is ： to 1 500， not panic
            let message = format!("响应序列化失败: {error}");
            logging::log("[Model]", &format!("❌ {message}"));
            return GatewayError::with_status(500, message).payload_response();
        }
    };
    let mut response = Response::new(Body::from(text));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response
}

/// response：`Content-Type: text/event-stream; charset=utf-8` +
/// `Cache-Control: no-cache` + `Connection: keep-alive`，。
///
/// `StatusCode` not is `u16`： use need 1 「 200」 1 
/// （ need use 1 ）， 1 in use 、 this down 。
pub fn sse_response(
    status: StatusCode,
    stream: Box<dyn futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
) -> Response {
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    response
}

/// 2xx error → by down errorresponse。
///
/// error not （OpenAI / Responses / Anthropic）， by 
/// use to ； 「 1 + 」 this 。
///
/// `convert` `(status, error)`，return this errorresponse。
pub fn error_response(
    context: RecordContext,
    error: &GatewayError,
    convert: impl FnOnce(u16, &GatewayError) -> Response,
) -> Response {
    let status = error.http_status().as_u16();
    // must and client to that 1 
    let recorded = RecordContext { status: i64::from(status), ..context };
    record_entry(&recorded, Some(error.message.clone()));
    convert(status, error)
}

// ─── ───────────────────────────────────────────────

/// ： body + 1 meta（；**failure not request**）。
///
/// and Node 1 ：directory not in when create， IO failure ——
/// is ， not can as failure use requestfailure。
pub fn write_debug_files(body: &[u8], method: &str, path: &str, user_agent: &str) {
    let dir = config::config_dir().join(DEBUG_DIR);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let _ = std::fs::write(dir.join(DEBUG_REQUEST_FILE), body);
    let meta = format!(
        "{} {method} {path} {}B ua={user_agent}",
        iso_timestamp(),
        body.len(),
    );
    let _ = std::fs::write(dir.join(DEBUG_META_FILE), meta);
}

/// ISO-8601 UTC when （ for Node `new Date().toISOString()`）
pub fn iso_timestamp() -> String {
    let millis = logging::now_ms();
    match chrono::DateTime::from_timestamp_millis(millis) {
        Some(utc) => utc.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(),
        None => String::new(),
    }
}

/// request sha256（）——， for Node 
/// `createHash('sha256').update(rawBody).digest('hex')`
pub fn sha256_hex(body: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(body);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    //! 。 this 「client before 」 not 1 success，
    //! to is not for ： ⇒ successrequest in 、 
    //! 「failure」usage（ up 1176 is this ）； ⇒ 
    //! in success（ not ）。 this 1 use need 。

    use super::{terminal, RecordContext, RecordingStream, TerminalFrames, TerminalScan};
    use crate::server::request_stats::Retention;

    /// to （CodeArts ， up down ）：
    /// after ** no has ** that empty。 `request_raw.response_body` 。
    const UNSPACED_DONE: &str = "data:[DONE]\n\n";
    /// that 1 （`ReasoningCoalescer` and ）empty。
    const SPACED_DONE: &str = "data: [DONE]\n\n";
    /// before Chat —— for use ： 1 ， old not must not 。
    const OLD_CHAT: TerminalFrames = &[b"data: [DONE]\n\n"];
    /// after 1 （ finish_reason and usage， and 1 ）
    const LAST_CHUNK: &str = r#"data: {"id":"c1","choices":[{"index":0,"finish_reason":"stop"}],"usage":{"prompt_tokens":71476}}"#;

    /// by this ，return「 is 」。
    fn seen_after(frames: TerminalFrames, chunks: &[&str]) -> bool {
        let mut scan = TerminalScan::new(frames);
        for chunk in chunks {
            scan.push(chunk.as_bytes());
        }
        scan.seen
    }

    /// （ up by ）。
    fn longest(frames: TerminalFrames) -> usize {
        frames.iter().map(|frame| frame.len()).max().unwrap_or(0)
    }

    /// this ：must。 after 1 is for —— 1 
    /// to before that ，must** not ** in ，otherwise that 1176 in not is this Reason。
    #[test]
    fn unspaced_done_counts_as_terminal() {
        assert!(seen_after(terminal::CHAT, &[LAST_CHUNK, UNSPACED_DONE]));
        let old_table_hits = seen_after(OLD_CHAT, &[LAST_CHUNK, UNSPACED_DONE]);
        assert!(!old_table_hits, "旧表若能命中，那 1176 行就不是这个原因");
    }

    /// empty not can as new —— that is before 1 in ，
    /// `ReasoningCoalescer` 。
    #[test]
    fn spaced_done_still_counts() {
        assert!(seen_after(terminal::CHAT, &[LAST_CHUNK, SPACED_DONE]));
        assert!(seen_after(OLD_CHAT, &[LAST_CHUNK, SPACED_DONE]), "旧表绿的不许变红");
    }

    /// CRLF （ up by SSE use `\r\n` when ）empty。
    #[test]
    fn crlf_line_endings_count() {
        assert!(seen_after(terminal::CHAT, &["data: [DONE]\r\n\r\n"]));
        assert!(seen_after(terminal::CHAT, &["data:[DONE]\r\n\r\n"]));
    }

    /// TCP need —— chunk is as in ；
    /// ， no （ to `data:[DON`）** not **， that is in 。
    #[test]
    fn only_a_whole_frame_counts_when_chunks_split() {
        let split = &[LAST_CHUNK, "\n\ndata:[DON", "E]\n\n"];
        assert!(seen_after(terminal::CHAT, split));
        assert!(seen_after(terminal::CHAT, &["\n\ndata:[DON", "E]\n\n"]));
        // 「 1 」 ： longest-1， this 
        assert!(seen_after(terminal::CHAT, &["data:[DONE]", "\n"]));
        let half_frame = seen_after(terminal::CHAT, &["data:[DON", "E]"]);
        assert!(!half_frame, "半截帧不算收尾");
    }

    /// ： `data: [DONE]` **** when not 。JSON 
    /// must（ up is n），SSE `data:` not ，
    /// need that not will in 。
    #[test]
    fn done_echoed_inside_content_is_not_a_terminal_frame() {
        let echoed = "data: {\"content\":\"示例：data: [DONE]\\n\\n 就是结束标记\"}\n\n";
        assert!(!seen_after(terminal::CHAT, &[echoed]));
        let unspaced = "data: {\"content\":\"示例：data:[DONE] 就是结束标记\"}\n\n";
        assert!(!seen_after(terminal::CHAT, &[unspaced]));
        // `data:[DONE]` after not is → not 
        let near_miss = &["data:[DONE", "\", \"x\":1}", "\n"];
        assert!(!seen_after(terminal::CHAT, near_miss));
    }

    /// must has ： 1 SSE response KB， etc response
    /// （ before this ， longest-1 ）。
    #[test]
    fn scan_window_stays_bounded() {
        let chunk = "data: {\"content\":\"很长的一段回答内容\"}\n\n";
        let mut scan = TerminalScan::new(terminal::CHAT);
        for _ in 0..2000 {
            scan.push(chunk.as_bytes());
        }
        let cap = longest(terminal::CHAT).saturating_sub(1);
        assert!(scan.window.len() <= cap, "窗口留了 {} 字节，上限 {}", scan.window.len(), cap);
    }

    /// not （ before ， not in empty）。
    /// this is 「 Chat」， is to —— 1 
    /// etc to that 。
    #[test]
    fn other_entries_keep_their_own_frames() {
        assert!(seen_after(terminal::RESPONSES, &["event: response.completed\n"]));
        assert!(seen_after(terminal::RESPONSES, &["event: response.failed\n"]));
        assert!(seen_after(terminal::ANTHROPIC, &["event: message_stop\n"]));
        // DONE is Chat ： to not this in 
        assert!(!seen_after(terminal::RESPONSES, &[UNSPACED_DONE, SPACED_DONE]));
        assert!(!seen_after(terminal::ANTHROPIC, &[UNSPACED_DONE, SPACED_DONE]));
    }

    // ── （ has ）──────────────────
    //
    // up that 「」。 and 0 token ****：
    // `Drop` `fallback_error`， up `NewRequestEntry::normalize` 
    // 「failure」。 this use 1 + 1 statistics for ，
    // 1 ： after ⇒ and has usage； before ⇒ 。

    use std::sync::Arc;

    use axum::body::Bytes;
    use futures::{Stream, StreamExt};
    use serde_json::Value;

    use crate::server::core::upstream::usage::RequestTelemetry;
    use crate::server::db::Db;
    use crate::server::request_stats::{RequestQuery, RequestStats};

    /// when RAII ：**return to use has **（ in create when 1 will 
    /// use failure or before return when directory down ， ）。
    struct TempDir(std::path::PathBuf);

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// 1 use statistics（ + + tag ）
    fn temp_stats(tag: &str) -> (Arc<RequestStats>, TempDir) {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let id = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("api-pipeline-{}-{}-{id}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&dir);
        let db = Db::open(&dir.join("agent2api.db")).expect("临时库应当能建起来");
        (Arc::new(RequestStats::with_db(Some(db), Retention::default)), TempDir(dir))
    }

    /// 1 up down ：usage by that （71476 / 56）
    fn context_for(stats: Arc<RequestStats>) -> (RecordContext, Arc<RequestTelemetry>) {
        let telemetry = Arc::new(RequestTelemetry::new());
        telemetry.report_usage(&serde_json::json!({
            "prompt_tokens": 71476, "completion_tokens": 56, "total_tokens": 71532
        }));
        let context = RecordContext {
            stats,
            telemetry: telemetry.clone(),
            started_at: crate::server::logging::now_ms(),
            model: "deepseek-v4.1-flash".to_string(),
            client_model: "deepseek-v4.1-flash".to_string(),
            client_reasoning: String::new(),
            // ： not is （ `RecordContext::is_test`）
            is_test: false,
            // client not and use ， to empty（none UA ）
            source: Default::default(),
            status: 200,
            raw_request: None,
            raw_response: None,
        };
        (context, telemetry)
    }

    /// 「 + + ** not to EOF**」—— after 1 after up ，
    /// is Qoder that client ： to ， by **client**。
    fn stream_ending_at(
        frames: Vec<&'static [u8]>,
    ) -> impl futures::Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin {
        futures::stream::iter(frames.into_iter().map(|frame| Ok(Bytes::from_static(frame))))
            .chain(futures::stream::pending::<Result<Bytes, std::io::Error>>())
    }

    /// that （ use has 1 ）
    fn only_row(stats: &RequestStats) -> serde_json::Value {
        let query = RequestQuery {
            offset: 0,
            limit: Some(5),
            model: None,
            provider: None,
            client: None,
            api_key: None,
            production_only: false,
            completed_only: false,
            status: None,
            start: None,
            end: None,
        };
        let page = stats.query_requests(&query);
        let entries = page.get("entries").and_then(Value::as_array);
        entries
            .and_then(|list| list.first().cloned())
            .unwrap_or_else(|| panic!("应当恰好落一条明细，实际读到 {page}"))
    }

    /// 1 「 already down this 、 after up not to EOF」 ， after in client ，
    /// return that 1 。 use in **** and **** up ， is 。
    async fn record_after_hangup(frames: &[&'static [u8]], terminals: TerminalFrames) -> Value {
        let (stats, _dir) = temp_stats("row");
        let (context, _telemetry) = context_for(stats.clone());
        let mut stream = RecordingStream::with_terminals(
            Box::new(stream_ending_at(frames.to_vec())),
            context,
            terminals,
        );
        // already （ can can in ）
        for _ in 0..frames.len() {
            assert!(stream.next().await.is_some(), "这一帧应当能取到");
        }
        // poll 1 ： up （`stream_ending_at` after pending）
        let still_pending =
            futures::future::poll_fn(|cx| match std::pin::Pin::new(&mut stream).poll_next(cx) {
                std::task::Poll::Ready(_) => std::task::Poll::Ready(false),
                std::task::Poll::Pending => std::task::Poll::Ready(true),
            })
            .await;
        assert!(still_pending, "客户端挂断前流必须还没有结束，否则走不到 Drop 那条分支");
        drop(stream);
        only_row(&stats)
    }

    /// ： `data:[DONE]`（ after no empty） already down ，client ⇒
    /// is 、usagemust。 this is that 1176 。
    #[tokio::test]
    async fn hangup_right_after_unspaced_done_records_success_with_usage() {
        let row =
            record_after_hangup(&[LAST_CHUNK.as_bytes(), UNSPACED_DONE.as_bytes()], terminal::CHAT).await;
        assert_eq!(row["status"], Value::from(200), "状态码照原样：{row}");
        assert!(row["error"].is_null(), "收尾帧之后挂断不该记成中断：{row}");
        assert_eq!(row["promptTokens"], Value::from(71476), "用量不能被清零：{row}");
        assert_eq!(row["completionTokens"], Value::from(56), "用量不能被清零：{row}");
    }

    /// 1 、 1 ， to ** before that ** ⇒ must is and 0 token。
    /// this is up 1 for ： no has ，「」 can can is as this not 。
    #[tokio::test]
    async fn the_old_needle_would_have_recorded_the_same_stream_as_an_abort() {
        let row = record_after_hangup(&[LAST_CHUNK.as_bytes(), UNSPACED_DONE.as_bytes()], OLD_CHAT).await;
        let error = row["error"].as_str().unwrap_or("");
        assert!(error.contains("未完整下发"), "旧表下这条必须仍是中断：{row}");
        assert_eq!(row["promptTokens"], Value::from(0), "旧口径的失败清零：{row}");
    }

    /// for ：client in ** before ** ⇒ is in 、 is 。
    /// this ， is 「 not 」 empty —— in 1 success 
    /// can when ， has for in up 。
    #[tokio::test]
    async fn hangup_before_the_terminal_frame_stays_an_abort() {
        let row = record_after_hangup(&[LAST_CHUNK.as_bytes()], terminal::CHAT).await;
        let error = row["error"].as_str().unwrap_or("");
        assert!(error.contains("未完整下发"), "真中断必须留摘要：{row}");
        assert_eq!(row["promptTokens"], Value::from(0), "失败的行按契约清零：{row}");
    }
}
