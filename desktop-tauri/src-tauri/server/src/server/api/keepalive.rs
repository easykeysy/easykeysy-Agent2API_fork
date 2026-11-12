//! 「」：`GET/PUT /api/keepalive-strip`。
//!
//! ── this ──────────────────────────────────────────
//! WorkBuddy and AutoClaw up will in 
//! 1 `delta.content` （ 1 in `: heartbeat` ，
//! that is ）。 58 WorkBuddy 200 request：20 
//! this （34.5%），request in 13.4%（ 1.2%–30.8%），
//! request 0； is —— 1 request need ， need 1 。
//! client by markdown when 1 is 1 `<br>`， is 1 1 。
//! after ， in ** down ** before this （ and " can not " to 
//! [`crate::server::core::upstream::sse::is_newline_keepalive`]）。
//!
//! ── Why is 「 can × 」 ───────────────────────
//! that `sse_strip_newline_chunks()` is 「 this up will not will this 」，
//! has ； is 「 this need not need as down 」。
//! and ， and ** can not can **： not will 1 no up 
//! （ [`crate::server::core::upstream::sse::FramePolicy::assembled`]）。
//!
//! ── Whydefault、Why need can in ───────────────────────
//! is " has 1 will 1 "， is " no " ——
//! this is 1 ， and in this that 1 request up （ 34.5%）。
//! not use ：
//! default， in config（`stripNewlineKeepalive`，set when 
//! `AGENT2API_STRIP_NEWLINE_KEEPALIVE` ）。 in is as ** can in up 
//! and when **： 1 down 、request down 、 not for ， not need need new 。
//! request， down 1 request， not 。
//!
//! ── and /api/sanitize、/api/debug ─────────────────────
//! 1 （GET / PUT / response is new ）， in default
//! and to ：default（ not will 400）、default（ can KB）、
//! default（ is down ）。set can use 1 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/keepalive-strip —— current
pub async fn get_keepalive_strip(State(_state): State<ServerState>) -> Response {
    ok_json(state_json())
}

/// PUT /api/keepalive-strip —— body `{stripNewlineKeepalive: true|false}`
///
/// config， down 1 request（request， not ）。
/// and 1 ： that 1 ， that down not is up ，
/// that 1 —— " this no " when ，must to this 。
pub async fn put_keepalive_strip(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let key = config::KEY_STRIP_NEWLINE_KEEPALIVE;
    let Some(value) = object.get(key) else {
        return errors::management_error(400, format!("缺少 {key}"));
    };
    let Some(enabled) = value.as_bool() else {
        return errors::management_error(400, format!("{key} 必须是 true 或 false"));
    };
    if !config::set_strip_newline_keepalive(enabled) {
        logging::log("[Config]", "⚠️  保活换行开关写入配置失败，本次运行内仍生效");
    }
    logging::log(
        "[Config]",
        if enabled {
            "保活换行丢弃已开启：WorkBuddy / AutoClaw 的下发帧将丢掉「整片只有换行」的 content 分片"
        } else {
            "保活换行丢弃已关闭：下发帧恢复为上游原样字节（保活换行会一起透传给客户端）"
        },
    );
    ok_json(state_json())
}

/// （GET and PUT use ， before use response new ）
fn state_json() -> Value {
    json!({
        config::KEY_STRIP_NEWLINE_KEEPALIVE: config::strip_newline_keepalive(),
    })
}
