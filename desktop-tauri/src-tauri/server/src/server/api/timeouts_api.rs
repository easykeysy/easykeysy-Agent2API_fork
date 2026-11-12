//! GET/PUT /api/timeouts —— up request when （ / etc response / empty / response）。
//!
//! ── for （ and OmniProxy set 1 1 for ）──
//! - `connectTimeoutSeconds`： up TCP/TLS or （ to that ）
//! —— in client `connect_timeout`（ `core::egress`）。
//! - `headersTimeoutSeconds`：request → response to 
//! —— in `upstream::request` etc limit。
//! - `streamIdleTimeoutSeconds`：response empty
//! —— in `upstream::ForwardStream` when （ to new ）。
//! - `bodyTimeoutSeconds`：response total
//! —— in `upstream::aggregate` （ 1 when ， not ）。
//!
//! 1 in config（ `config.rs`）， by set
//! 「 use → request when 」 HTTP —— and /api/retry 1 ：
//! 、、return after 、none use （ not ）。
//! after for down 1 request（request， not ）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{
    self, TimeoutPatch, TimeoutSettings, KEY_TIMEOUT_BODY_SECONDS, KEY_TIMEOUT_CONNECT_SECONDS,
    KEY_TIMEOUT_HEADERS_SECONDS, KEY_TIMEOUT_STREAM_IDLE_SECONDS, TIMEOUT_MAX_SECONDS,
    TIMEOUT_MIN_SECONDS,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/timeouts
pub async fn get_timeouts(State(_state): State<ServerState>) -> Response {
    ok_json(timeouts_json(config::timeout_settings()))
}

/// PUT /api/timeouts —— body `{connectTimeoutSeconds?, headersTimeoutSeconds?,
/// streamIdleTimeoutSeconds?, bodyTimeoutSeconds?}`
///
/// （ ，null ）。 after ：config
/// → return** after ** （ before use response new ， not GET 1 ）。
pub async fn put_timeouts(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // and （ and put_retry 1 ）： has 1 when 
    // not ，「 when 、empty no 」 set
    let mut patch = TimeoutPatch::default();
    let mut targets = [
        (
            KEY_TIMEOUT_CONNECT_SECONDS,
            object.get(KEY_TIMEOUT_CONNECT_SECONDS),
            &mut patch.connect_seconds,
        ),
        (
            KEY_TIMEOUT_HEADERS_SECONDS,
            object.get(KEY_TIMEOUT_HEADERS_SECONDS),
            &mut patch.headers_seconds,
        ),
        (
            KEY_TIMEOUT_STREAM_IDLE_SECONDS,
            object.get(KEY_TIMEOUT_STREAM_IDLE_SECONDS),
            &mut patch.stream_idle_seconds,
        ),
        (
            KEY_TIMEOUT_BODY_SECONDS,
            object.get(KEY_TIMEOUT_BODY_SECONDS),
            &mut patch.body_seconds,
        ),
    ];
    let mut updated = false;
    for (key, value, slot) in targets.iter_mut() {
        let Some(value) = value else {
            continue;
        };
        // null as 「 this 1 not 」： before when as null
        if value.is_null() {
            continue;
        }
        match parse_seconds(key, value) {
            Ok(number) => {
                **slot = Some(number);
                updated = true;
            }
            Err(message) => return errors::management_error(400, message),
        }
    }

    if updated && !config::set_timeouts(patch) {
        // failure： already update（）， after will to old ——
        // must use ，otherwise「set」 will 
        logging::log("[Config]", "⚠️  请求超时设置写入失败，本次运行内仍生效");
    }

    let settings = config::timeout_settings();
    if updated {
        logging::log(
            "[Config]",
            &format!(
                "请求超时已更新: 连接 {} 秒 / 等待响应 {} 秒 / 流空闲 {} 秒 / 非流式 {} 秒",
                settings.connect_seconds,
                settings.headers_seconds,
                settings.stream_idle_seconds,
                settings.body_seconds,
            ),
        );
    }
    ok_json(timeouts_json(settings))
}

/// when response（GET and PUT use ， and config.rs 1 ——
/// and `retry_api::retry_json` 1 ： use not is ）。
/// for before is ****（`settings-panel.js` TIMEOUT_FIELDS for ）。
fn timeouts_json(settings: TimeoutSettings) -> Value {
    json!({
        KEY_TIMEOUT_CONNECT_SECONDS: settings.connect_seconds,
        KEY_TIMEOUT_HEADERS_SECONDS: settings.headers_seconds,
        KEY_TIMEOUT_STREAM_IDLE_SECONDS: settings.stream_idle_seconds,
        KEY_TIMEOUT_BODY_SECONDS: settings.body_seconds,
    })
}

/// seconds ：must is 1–3600 ，otherwise to can 400 。
/// and `retry_api::parse_bounded_int` 1 （ JSON ，
/// `30.0` this ； `"30"` as ）。
fn parse_seconds(key: &str, value: &Value) -> Result<i64, String> {
    let number = match value {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                .map(|raw| raw as i64)
        }),
        _ => None,
    };
    match number {
        Some(number) if (TIMEOUT_MIN_SECONDS..=TIMEOUT_MAX_SECONDS).contains(&number) => Ok(number),
        _ => Err(format!(
            "{key} 必须是 {TIMEOUT_MIN_SECONDS}-{TIMEOUT_MAX_SECONDS} 的整数（收到: {value}）"
        )),
    }
}
