//! when to **** by （ to in 「 when to 」 ）。
//!
//! ```text
//! GET /api/checkin-schedules （total + + ）
//! POST /api/checkin-schedules 1 （ by id upsert，{id?, label?, enabled?, time?, providers?}）
//! POST /api/checkin-schedules/{id}/delete delete 1 （DELETE ）
//! POST /api/checkin-schedules/{id}/run this 1 
//! ```
//!
//! ── Why and /api/auto-checkin is ─────────────────────
//! `autoCheckin.enabled` is **total**， （`api::auto_checkin`，
//! Node ，OpenAI error body）； is add config，
//! 。 and 1 will body —— is 。
//!
//! ── by and error ──────────────────────────────────────
//! and `/api/scheduled-tasks` ： is `any(...)`， in file
//! `dispatch`（ by will 「 already path + 」 to 405 ，
//! this 1 to 404；POST DELETE is as CORS has DELETE 
//! for ， and tasks PATCH/POST 1 ）。
//! error 1 **** 400（`{success:false,error}`）： no has need need for 
//! Node ， API 400 1 （ scheduled_tasks module header）。

use axum::body::Bytes;
use axum::extract::State;
use axum::http::Method;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::server::core::checkin_schedules;
use crate::server::errors::not_found_response;
use crate::server::http::{ok_json, parse_body, MAX_BODY_SIZE};
use crate::server::logging;
use crate::server::ServerState;

/// `/api/checkin-schedules*` （any、anypath）。
pub async fn entry(State(state): State<ServerState>, request: axum::extract::Request) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let body = match axum::body::to_bytes(request.into_body(), MAX_BODY_SIZE).await {
        Ok(bytes) => bytes,
        Err(error) => return bad_request(format!("请求体读取失败或过大: {error}")),
    };
    dispatch(&state, method, &path, &body).await
}

/// path（， and scheduled_tasks 1 ）。
async fn dispatch(state: &ServerState, method: Method, path: &str, body: &Bytes) -> Response {
    let rest = path
        .strip_prefix("/api/checkin-schedules")
        .unwrap_or_default()
        .trim_matches('/');

    if rest.is_empty() {
        if method == Method::GET {
            return ok_json(state.auto_checkin().state());
        }
        if method == Method::POST {
            return save(state, body).await;
        }
        return not_found(method.as_str(), path);
    }

    let mut segments = rest.splitn(2, '/');
    let id = segments.next().unwrap_or_default();
    let action = segments.next();

    match (method.clone(), action) {
        // DELETE is ；POST is （CORS ， tasks PATCH/POST）
        (Method::DELETE, None) | (Method::POST, Some("delete")) => remove(state, id),
        (Method::POST, Some("run")) => run(state, id).await,
        _ => not_found(method.as_str(), path),
    }
}

/// 1 （add or update）。successreturn****： already is after 1 
/// after （displayLabel / lastFiredToday / nextRunAt ），response as 
/// ， not 1 GET（ and /api/models/refresh return 1 by ）。
async fn save(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_body(body) {
        Ok(value) => value,
        Err(error) => return bad_request(error.message),
    };
    if !payload.is_object() {
        return bad_request("请求内容不是有效对象");
    }
    match checkin_schedules::upsert(&payload) {
        Ok(_) => ok_json(state.auto_checkin().state()),
        Err(message) => bad_request(message),
    }
}

/// delete 1 。delete default = empty（「 is 」，
/// checkin_schedules::remove ）。
fn remove(_state: &ServerState, id: &str) -> Response {
    match checkin_schedules::remove(id) {
        Ok(()) => ok_json(json!({ "success": true })),
        Err(message) => bad_request(message),
    }
}

/// 1 （ and to 1 fire path， and ）。
///
/// not in and 「 in in 」 to 400 —— `run_schedule_now` for return
/// None， in in by 。response and /api/auto-checkin/run 1 ：
/// `{...summary, state}`（state in after ，）。
async fn run(state: &ServerState, id: &str) -> Response {
    if !checkin_schedules::list().iter().any(|entry| entry.id == id) {
        return bad_request(format!("定时条目不存在：{id}"));
    }
    let Some(summary) = state.auto_checkin().run_schedule_now(id).await else {
        return bad_request("签到正在执行中，请稍候");
    };
    let merged = match summary {
        Value::Object(map) => {
            let mut merged = map;
            if let Value::Object(state_map) = state.auto_checkin().state() {
                for (key, value) in state_map {
                    merged.insert(key, value);
                }
            }
            Value::Object(merged)
        }
        other => other,
    };
    ok_json(merged)
}

/// 400（`{success:false,error}`）， and 1 （scheduled_tasks ）。
fn bad_request(message: impl Into<String>) -> Response {
    let message = message.into();
    logging::log("[Checkin]", &format!("❌ {message}"));
    (
        axum::http::StatusCode::BAD_REQUEST,
        Json(json!({ "success": false, "error": message })),
    )
        .into_response()
}

/// in 1 ： and （`Not found: <METHOD> <path>`）。
fn not_found(method: &str, path: &str) -> Response {
    not_found_response(method, path)
}
