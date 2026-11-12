//! GET/PUT /api/queue —— etc set（ and when ）。
//!
//! ── this set ──────────────────────────────────────────
//! has up in when ** not 、**： 1 「 not can ， N seconds after 
//! 」（Qoder use is this ， 10605）。 is by up 
//! when etc 1 will 1 request， etc not up to client。
//!
//! - `queueMaxWaits`： etc （0 = not etc ，return 503）；
//! - `queueWaitSeconds`： etc seconds（0 = up ， up no to when 
//! to 15 seconds）。
//!
//! in config（ `config.rs` `KEY_QUEUE_*`）， by set
//! 「 use → etc 」 HTTP —— and /api/timeouts 1 ：
//! 、、return after 、none use （ not ）。
//! after for down 1 request（ to ）。
//!
//! ── and 「 etc response when 」 （set 1 ）───────
//! etc in before ， is 「 etc response when 」 that ：set 1 
//! 「 1 request」。 not will as etc response when 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{
    self, QueuePatch, QueueSettings, KEY_QUEUE_MAX_WAITS, KEY_QUEUE_WAIT_SECONDS,
    QUEUE_MAX_MAX_WAITS, QUEUE_MAX_WAIT_SECONDS, QUEUE_MIN_MAX_WAITS, QUEUE_MIN_WAIT_SECONDS,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/queue
pub async fn get_queue(State(_state): State<ServerState>) -> Response {
    ok_json(queue_json(config::queue_settings()))
}

/// PUT /api/queue —— body `{queueMaxWaits?, queueWaitSeconds?}`
///
/// （ ，null ）。 after ：config
/// → return** after ** （ before use response new ， not GET 1 ）。
pub async fn put_queue(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // and （ and put_timeouts 1 ）： has 1 when 
    // not ，「、 when no 」 set
    let mut patch = QueuePatch::default();
    let mut updated = false;
    let targets = [
        (
            KEY_QUEUE_MAX_WAITS,
            object.get(KEY_QUEUE_MAX_WAITS),
            QUEUE_MIN_MAX_WAITS,
            QUEUE_MAX_MAX_WAITS,
            "排队等待次数",
            &mut patch.max_waits,
        ),
        (
            KEY_QUEUE_WAIT_SECONDS,
            object.get(KEY_QUEUE_WAIT_SECONDS),
            QUEUE_MIN_WAIT_SECONDS,
            QUEUE_MAX_WAIT_SECONDS,
            "排队等待秒数",
            &mut patch.wait_seconds,
        ),
    ];
    for (key, value, min, max, label, slot) in targets {
        let Some(value) = value else {
            continue;
        };
        // null as 「 this 1 not 」： before when as null
        if value.is_null() {
            continue;
        }
        match parse_bounded_int(key, value, min, max, label) {
            Ok(number) => {
                *slot = Some(number);
                updated = true;
            }
            Err(message) => return errors::management_error(400, message),
        }
    }

    if updated && !config::set_queue(patch) {
        // failure： already update（）， after will to old ——
        // must use ，otherwise「set」 will 
        logging::log("[Config]", "⚠️  排队等待设置写入失败，本次运行内仍生效");
    }

    let settings = config::queue_settings();
    if updated {
        let wait = if settings.wait_seconds > 0 {
            format!("{} 秒", settings.wait_seconds)
        } else {
            "跟随上游建议".to_string()
        };
        logging::log(
            "[Config]",
            &format!("排队等待已更新: 最多 {} 次 / 单次 {wait}", settings.max_waits),
        );
    }
    ok_json(queue_json(settings))
}

/// response（GET and PUT use ， use not is ——
/// and `timeouts_api::timeouts_json` 1 ， for before is ）。
fn queue_json(settings: QueueSettings) -> Value {
    json!({
        KEY_QUEUE_MAX_WAITS: settings.max_waits,
        KEY_QUEUE_WAIT_SECONDS: settings.wait_seconds,
    })
}

/// ：must is ，otherwise to can 400 。
/// and `timeouts_api::parse_seconds` 1 （ JSON ， `2.0`
/// this ； `"2"` as ）。
fn parse_bounded_int(
    key: &str,
    value: &Value,
    min: i64,
    max: i64,
    label: &str,
) -> Result<i64, String> {
    let number = match value {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                .map(|raw| raw as i64)
        }),
        _ => None,
    };
    let Some(number) = number else {
        return Err(format!("{label}（{key}）必须是整数"));
    };
    if !(min..=max).contains(&number) {
        return Err(format!("{label}（{key}）必须在 {min}~{max} 之间"));
    }
    Ok(number)
}
