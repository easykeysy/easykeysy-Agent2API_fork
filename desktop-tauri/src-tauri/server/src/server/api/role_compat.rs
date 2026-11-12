//! ：`GET/PUT /api/developer-role-compat`。
//!
//! ── this ──────────────────────────────────────────
//! after in before request `messages` `role: "developer"` 
//! as `role: "system"`（ `core::upstream::payload` send_body 1 
//! ）：OpenAI new API use developer system ， OpenAI
//! up （ path） not this ， will 400。
//!
//! ── Why has 1 ──────────────────────────────────────
//! is **none** then （developer and system in OpenAI up etc ，
//! OpenAI is this ）， no has 、 no has use can ；
//! and `/api/sanitize`、`/api/debug` 1 —— GET 、PUT 、response is 
//! new ，set that use 1 。
//!
//! ── ────────────────────────────────────────────────
//! `config` `developerRoleCompat` （ 1 `kv` config），default。
//! request， down 1 request， not 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/developer-role-compat —— current
pub async fn get_role_compat(State(_state): State<ServerState>) -> Response {
    ok_json(role_compat_json())
}

/// PUT /api/developer-role-compat —— body `{developerRoleCompat: true|false}`
///
/// config， down 1 request（request， not ）。
pub async fn put_role_compat(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let key = config::KEY_DEVELOPER_ROLE_COMPAT;
    let Some(value) = object.get(key) else {
        return errors::management_error(400, format!("缺少 {key}"));
    };
    let Some(enabled) = value.as_bool() else {
        return errors::management_error(400, format!("{key} 必须是 true 或 false"));
    };
    if !config::set_developer_role_compat(enabled) {
        logging::log(
            "[Config]",
            "⚠️  开发者角色兼容设置写入失败，本次运行内仍生效",
        );
    }
    logging::log(
        "[Config]",
        if enabled {
            "开发者角色兼容已开启：转发前把 developer 角色消息改写为 system"
        } else {
            "开发者角色兼容已关闭：developer 角色消息将原样发往上游"
        },
    );
    ok_json(role_compat_json())
}

/// （GET and PUT use ， before use response new ）
fn role_compat_json() -> Value {
    json!({
        config::KEY_DEVELOPER_ROLE_COMPAT: config::current().developer_role_compat(),
    })
}
