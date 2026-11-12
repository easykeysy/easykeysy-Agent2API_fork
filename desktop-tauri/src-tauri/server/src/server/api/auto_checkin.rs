//! when to by （ for server.mjs 726-746 ）。
//!
//! GET /api/auto-checkin 、 when 、 up 
//! POST /api/auto-checkin set { enabled?, time? }
//! POST /api/auto-checkin/run 1 
//!
//! ── error（ Node ）────────────────────────────
//! Node this in handler try ， AutoCheckinConfigError
//! errorPayload **OpenAI ** body：
//!   `{"error":{"message":"…","type":"invalid_request_error"}}`
//! not is API `{success:false,error}`。（2026-09-17，Node v1.0.1）：
//! POST /api/auto-checkin {"time":"99:99"} → 400 when （00:00 - 23:59）
//! POST /api/auto-checkin {} → 400 no has need need update 
//! this use `to_gateway_error().into_response()`，error 
//! （AutoCheckinConfigError 400， core::auto_checkin ）。
//!
//! ── run response ─────────────────────────────────────────
//! success：`{success:true, data:{...summary, state}}` —— summary 
//! （at/date/reason/succeeded/total/skipped/failed/failedCount） and state and 
//! （state in after ， state as ， Node for ）。
//! in in （runNow return null）： AutoCheckinConfigError → 400
//! 「 to in in ，」。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use crate::server::core::auto_checkin::AutoCheckinConfigError;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// configerror → response（OpenAI body， use error ）
fn config_error(error: AutoCheckinConfigError) -> Response {
    logging::log("[Checkin]", &format!("❌ {}", error.message));
    crate::server::errors::GatewayError::with_status(error.status_code, error.message)
        .into_response()
}

/// GET /api/auto-checkin —— to 
pub async fn get_state(State(state): State<ServerState>) -> Response {
    ok_json(state.auto_checkin().state())
}

/// POST /api/auto-checkin —— set（{ enabled?, time? }）
pub async fn configure(State(state): State<ServerState>, body: Bytes) -> Response {
    // empty body as {}（Node `|| '{}'`）； JSON in Node by JSON.parse
    // SyntaxError to errorPayload → 500 proxy_error（），
    // this 1 ： not 400 。
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => {
            return crate::server::errors::GatewayError::new(error.message).into_response()
        }
    };
    if !payload.is_object() {
        // Node: `configure(body)` for for to undefined → not patch
        // → 「 no has need need update 」（AutoCheckinConfigError → 400）
        return config_error(AutoCheckinConfigError {
            message: "没有需要更新的字段".to_string(),
            status_code: 400,
        });
    }
    match state.auto_checkin().configure(&payload) {
        Ok(data) => ok_json(data),
        Err(error) => config_error(error),
    }
}

/// POST /api/auto-checkin/run —— 1 
///
/// `run_now()` return None already has 1 in （）→ 400
/// 「 to in in ，」。Note to failurereturn None， that down 
/// lastResult already failureReason，Node none（path to 400）。
pub async fn run_now(State(state): State<ServerState>) -> Response {
    let Some(summary) = state.auto_checkin().run_now().await else {
        return config_error(AutoCheckinConfigError {
            message: "签到正在执行中，请稍候".to_string(),
            status_code: crate::server::core::auto_checkin::CHECKIN_BUSY_STATUS,
        });
    };
    // `{...summary, state}`：summary 、state after 
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
