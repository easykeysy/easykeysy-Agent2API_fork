//! when （ to / set use ）。
//!
//! ```text
//! GET /api/travel-schedule 、 when 、 already when 、 up 1 
//! POST /api/travel-schedule config { enabled?, times? }
//! POST /api/travel-schedule/run 1 （ up ）
//! ```
//!
//! ── Why is 1 ─────────────────────────────────────
//! and to is ** **（ is `travel_hours` /
//! `checkin_hours` and when ）： is not ， use can can 
//! to 、 not 。 1 will 「」「 to 」 use 。
//!
//! ── error ────────────────────────────────────────────────
//! and `/api/scheduled-tasks` 1 ：parameterfailure**** 400
//! `{success:false,error:"…"}`， not is auto-checkin that OpenAI body
//! （ that is Node ， is add API）。
//!
//! ── 「 in in 」────────────────────────────────────────────
//! run up in return 409； can unavailablereturn in ，failureReason。

use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::server::core::growth_scheduler;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// `GET /api/travel-schedule` —— currentconfig and 。
pub async fn get_state() -> Response {
    ok_json(growth_scheduler::state())
}

/// `POST /api/travel-schedule` —— and when （ ）。
pub async fn configure(body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return bad_request(error.message),
    };
    // empty for as 「 no has need need update 」： and /api/scheduled-tasks 1 
    let has_field = payload
        .as_object()
        .map(|object| object.contains_key("enabled") || object.contains_key("times"))
        .unwrap_or(false);
    if !has_field {
        return bad_request("没有需要更新的字段（enabled / times）");
    }
    match growth_scheduler::configure(&payload) {
        Ok(state) => {
            let enabled = state
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let times = state
                .get("times")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(" / ")
                })
                .unwrap_or_default();
            logging::log(
                "[Travel]",
                &format!(
                    "猫猫旅行排程已{}，时刻 {times}",
                    if enabled { "开启" } else { "关闭" }
                ),
            );
            ok_json(state)
        }
        Err(message) => bad_request(message),
    }
}

/// `POST /api/travel-schedule/run` —— 1 。
///
/// response need and new after ： by ， not 1 
/// （ and `/api/scheduled-tasks/{id}/run` 1 ）。
pub async fn run_now(State(state): State<ServerState>) -> Response {
    let store = state.store().clone();
    match growth_scheduler::run_now(&store).await {
        growth_scheduler::RunOutcome::Completed(summary) => ok_json(json!({
            "outcome": "completed",
            "summary": summary,
            "schedule": growth_scheduler::state(),
        })),
        growth_scheduler::RunOutcome::Skipped(message) => ok_json(json!({
            "outcome": "skipped",
            "message": message,
            "summary": Value::Null,
            "schedule": growth_scheduler::state(),
        })),
        growth_scheduler::RunOutcome::Busy => (
            StatusCode::CONFLICT,
            Json(json!({
                "success": false,
                "outcome": "busy",
                "error": "猫猫旅行巡检正在执行中，请稍候",
                "schedule": growth_scheduler::state(),
            })),
        )
            .into_response(),
        growth_scheduler::RunOutcome::Failed(error) => (
            u16::try_from(error.status_code)
                .ok()
                .and_then(|status| StatusCode::from_u16(status).ok())
                .unwrap_or(StatusCode::BAD_GATEWAY),
            Json(json!({
                "success": false,
                "outcome": "failed",
                "error": error.message,
                "code": error.code,
                "schedule": growth_scheduler::state(),
            })),
        )
            .into_response(),
    }
}

/// 400（`{success:false,error}`）， and 1 。
fn bad_request(message: impl Into<String>) -> Response {
    let message = message.into();
    logging::log("[Travel]", &format!("❌ {message}"));
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "success": false, "error": message })),
    )
        .into_response()
}
