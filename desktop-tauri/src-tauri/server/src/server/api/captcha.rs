//! ：`GET/PUT /api/captcha`。
//!
//! ── this ──────────────────────────────────────────
//! / ALTCHA proof-of-work （ `server::altcha`）：
//! when ，must、 after 、request payload can ；
//! when not （challenge 400）。**default** ——
//! and is ， / can 1 not can 1 。
//!
//! ── Why ────────────────────────────────────────────
//! and headless use 1 after ， not ：
//! （ and after ），headless in up 。
//! before when can ， after etc 。 
//! ， by `ServerState.local_panel` 1 。
//!
//! ── （GET and PUT response）─────────────────────────────
//! ```text
//! { captchaDesktop: bool, captchaWeb: bool, localPanel: bool }
//! ```
//! `localPanel` is **** （ true / headless false），set
//! 「 in use this 1 」； is can ，PUT to its in 1 
//! （update， and `/api/retry` 1 ）。 old `captchaEnabled` 
//! （ when ）， no client / 。
//!
//! old `captchaEnabled` in when as default use ，
//! `config::captcha_switch_from` —— when use not will 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

fn captcha_json(state: &ServerState) -> Value {
    let (desktop, web) = config::current().captcha_settings();
    json!({
        "captchaDesktop": desktop,
        "captchaWeb": web,
        "localPanel": state.local_panel,
    })
}

/// GET /api/captcha —— + 
pub async fn get_captcha(State(state): State<ServerState>) -> Response {
    ok_json(captcha_json(&state))
}

/// PUT /api/captcha —— body `{captchaDesktop?: bool, captchaWeb?: bool}`（ or old 
/// `captchaEnabled` when ）。
pub async fn put_captcha(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    // ： to null and 「 no to 」（update）， to then ——
    // ignore will as success
    fn field(
        object: &serde_json::Map<String, Value>,
        key: &str,
    ) -> Result<Option<bool>, Response> {
        match object.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Bool(value)) => Ok(Some(*value)),
            Some(_) => Err(errors::management_error(
                400,
                format!("{key} 必须是布尔值"),
            )),
        }
    }
    let desktop = match field(object, config::KEY_CAPTCHA_DESKTOP) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let web = match field(object, config::KEY_CAPTCHA_WEB) {
        Ok(value) => value,
        Err(response) => return response,
    };
    // old client `captchaEnabled`： 1 （ old has 1 ）
    let legacy = match field(object, config::KEY_CAPTCHA_ENABLED) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let desktop = desktop.or(legacy);
    let web = web.or(legacy);
    if desktop.is_none() && web.is_none() {
        return errors::management_error(
            400,
            "缺少 captchaDesktop / captchaWeb 字段（旧客户端可用 captchaEnabled 同时设置两端）",
        );
    }
    if !config::set_captcha_patch(desktop, web) {
        return errors::management_error(500, "写入配置失败，请重试");
    }
    let (desktop_now, web_now) = config::current().captcha_settings();
    logging::log(
        "[Security]",
        &format!(
            "机器人校验：桌面端{}、网页端{}",
            if desktop_now { "开启" } else { "关闭" },
            if web_now { "开启" } else { "关闭" },
        ),
    );
    ok_json(captcha_json(&state))
}
