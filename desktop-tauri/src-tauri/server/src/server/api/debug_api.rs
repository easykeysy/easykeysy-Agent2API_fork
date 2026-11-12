//! ：`GET/PUT /api/debug`（） and `GET /api/debug/traffic`（）。
//!
//! ── this ──────────────────────────────────────────
//! after ** to up request（ + ） and up return response（ +
//! + ）** to 1 `debug_traffic` ，request 
//! 「」。 and `core::debug_traffic`。
//!
//! ── Why 1 ────────────────────────────────────
//! can KB（ 1 SSE response），request will 1 
//! response。 to `id`， by need —— and OmniProxy「 not return
//! raw、return」 1 。
//!
//! ── ────────────────────────────────────────────────
//! and its `{config_dir}/agent2api.db`（ `/api/storage`
//! ）， this and 。

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::debug_traffic;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/debug —— current and 
pub async fn get_debug(State(_state): State<ServerState>) -> Response {
    ok_json(debug_json())
}

/// PUT /api/debug —— body `{debugMode: true|false}`
///
/// config.json（ `debugMode`）， down 1 request（request，
/// not ）。 when in 1 —— will after 
/// ， use this in to this 。
pub async fn put_debug(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let Some(value) = object.get(config::KEY_DEBUG_MODE) else {
        return errors::management_error(400, format!("缺少 {}", config::KEY_DEBUG_MODE));
    };
    let Some(enabled) = value.as_bool() else {
        return errors::management_error(
            400,
            format!("{} 必须是 true 或 false", config::KEY_DEBUG_MODE),
        );
    };
    if !config::set_debug_mode(enabled) {
        logging::log("[Config]", "⚠️  调试模式设置写入 config.json 失败，本次运行内仍生效");
    }
    logging::log(
        "[Config]",
        if enabled {
            "调试模式已开启：转发时会把上游原始报文（凭据类头已脱敏）保存到本地"
        } else {
            "调试模式已关闭"
        },
    );
    ok_json(debug_json())
}

/// GET /api/debug/traffic?id=... —— 1 request 
///
/// not to （ no / already / id as empty） when to 404： before 
/// 「 this request no has 」 not is empty。
pub async fn get_traffic(State(_state): State<ServerState>, Query(params): Query<Params>) -> Response {
    let id = params.id.trim();
    if id.is_empty() {
        return errors::management_error(400, "缺少 id");
    }
    match debug_traffic::get(id) {
        Some(entry) => ok_json(debug_traffic::detail_payload(&entry)),
        None => errors::management_error(
            404,
            "该请求没有保存原始报文（调试模式当时未开启，或记录已超出保留条数）",
        ),
    }
}

#[derive(Deserialize)]
pub struct Params {
    #[serde(default)]
    id: String,
}

/// + （GET and PUT use ， before use response new ）
fn debug_json() -> Value {
    json!({
        config::KEY_DEBUG_MODE: config::current().debug_mode(),
        "count": debug_traffic::count(),
        "limit": debug_traffic::MAX_ENTRIES,
    })
}
