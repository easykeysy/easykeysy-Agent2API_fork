//! when by （set「 when 」 ）。
//!
//! GET /api/scheduled-tasks 、 and 
//! PATCH /api/scheduled-tasks/{id} 1 （`{enabled?, interval?}`）
//! POST /api/scheduled-tasks/{id}/run 1 ** after **
//!
//! ── Why and /api/auto-checkin is ─────────────────────
//! to is ****（ when + + ）， is 
//! 「 etc 」， config and not （ by 
//! `core::scheduled_tasks` module header）。 up in 「 when 」，
//! —— and 1 will 1 「 not use 」
//! 。 before in this 1 。
//!
//! ── by ────────────────────────────────────────────────
//! （none / / ） is `any(...)`， in file 
//! `dispatch` —— and `/api/accounts` 1 ：
//! axum by ，「 already path + 」 will to 405 ，
//! this 1 to 404。（ has id path），
//! 「`PATCH` not in CORS 」 ：POST 。
//!
//! ── error ────────────────────────────────────────────────
//! parameterfailure（unknown id / / no has need need update ）return**** 
//! 400 `{success:false,error:"…"}`， not is auto-checkin that OpenAI body。
//! by ： that is Node auto-checkin （ for ，
//! `api::auto_checkin` module header）， is add API， no has need need for 
//! old client —— and `/api/retention` 1 （ that failure is 400）。
//!
//! 「 in in 」（run up 1 already in ） is 400 ： is ** and 
//! ** not is request， has auto-checkin for 1 use 400，
//! this 1 ， before can by 1 。

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::server::config::IntervalTaskPatch;
use crate::server::core::scheduled_tasks;
use crate::server::errors::not_found_response;
use crate::server::http::{ok_json, parse_body, MAX_BODY_SIZE};
use crate::server::logging;
use crate::server::ServerState;

/// `/api/scheduled-tasks` （any、anypath）。
pub async fn entry(State(state): State<ServerState>, request: axum::extract::Request) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let body = match axum::body::to_bytes(request.into_body(), MAX_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(error) => return bad_request(format!("请求体读取失败或过大: {error}")),
    };
    dispatch(&state, method, &path, &body).await
}

/// path。
///
/// id path use **** not is axum `Path<String>`： by 
/// `any({*rest})` after not to parameter， this （ before + 1 id +
/// optional ）， 1 by 。
async fn dispatch(state: &ServerState, method: Method, path: &str, body: &Bytes) -> Response {
    // before after ：`""`（none）、`"credentialMaintenance"`、`"…/run"`
    let rest = path
        .strip_prefix("/api/scheduled-tasks")
        .unwrap_or_default()
        .trim_matches('/');

    if rest.is_empty() {
        if method == Method::GET {
            return ok_json(scheduled_tasks::list());
        }
        return not_found(method.as_str(), path);
    }

    // ：`{id}` and `{id}/run`
    let mut segments = rest.splitn(2, '/');
    let id = segments.next().unwrap_or_default();
    let action = segments.next();

    match (method.clone(), action) {
        // PATCH is ；POST is —— CORS no has PATCH
        // （ Node ， http.rs CORS_METHODS）， when will down 
        // PATCH， Tauri before can use 。
        (Method::PATCH, None) | (Method::POST, None) => configure(id, body),
        (Method::POST, Some("run")) => run_now(state, id).await,
        _ => not_found(method.as_str(), path),
    }
}

/// 1 （`{enabled?}` / `{interval?}`， ）
fn configure(id: &str, body: &Bytes) -> Response {
    // empty body as `{}` → 「 no has need need update 」 400（ and auto-checkin 1 ）
    let payload = match parse_body(body) {
        Ok(value) => value,
        Err(error) => return bad_request(error.message),
    };
    let mut patch = IntervalTaskPatch::default();
    if let Some(object) = payload.as_object() {
        if let Some(value) = object.get("enabled") {
            // and auto_checkin configure 1 ：`enabled !== undefined` when 
            // true， its （null / / 0） 1 as false， not is ignore。
            // before is input.checked， is 。
            patch.enabled = Some(value == &Value::Bool(true));
        }
        if let Some(value) = object.get("interval") {
            match value.as_i64().or_else(|| {
                value
                    .as_f64()
                    .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                    .map(|raw| raw as i64)
            }) {
                Some(interval) => patch.interval = Some(interval),
                // （ "abc" / null / ） 400：ignore will 
                // 「success」 no ，
                None => return bad_request(format!("「{id}」的间隔必须是整数")),
            }
        }
    }
    match scheduled_tasks::configure(id, patch) {
        Ok(task) => ok_json(task),
        Err(message) => bad_request(message),
    }
}

/// 1 after 
///
/// for `runner = backend` valid（ before new when in up ，
/// after not ）—— that return 400 and Reason。
///
/// response** new after that **（ new `lastRunAt` / `lastResult` /
/// `nextRunAt`）： by can ， not 1 GET
/// （ and `/api/models/refresh` return 1 by ）。
async fn run_now(state: &ServerState, id: &str) -> Response {
    let store = state.store().clone();
    match scheduled_tasks::run_now(&store, state.update(), id).await {
        Ok(summary) => ok_json(json!({
            "summary": summary,
            "task": scheduled_tasks::task_by_id(id),
        })),
        Err(message) => bad_request(message),
    }
}

/// 400（`{success:false,error}`）， and 1 。
///
/// not `errors::GatewayError::bad_request`： that will to OpenAI body，
/// and `/api/retention` not 1 —— this 「 API 400 is 
/// 1 」。
fn bad_request(message: impl Into<String>) -> Response {
    let message = message.into();
    logging::log("[Tasks]", &format!("❌ {message}"));
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "success": false, "error": message })),
    )
        .into_response()
}

/// in 1 ： and （`Not found: <METHOD> <path>`）。
/// before down no has need need 「path」 and 「」 ， to 1 404 。
fn not_found(method: &str, path: &str) -> Response {
    not_found_response(method, path)
}
