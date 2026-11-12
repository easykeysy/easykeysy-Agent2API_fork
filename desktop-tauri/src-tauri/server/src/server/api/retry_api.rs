//! GET/PUT /api/retry —— requestset（ / / error）。
//!
//! for up when error and failure「 1 」，
//! （ `config.rs` ）：
//! - `retryCount`： in ** 1 ** up ；
//! - `retryCrossProviderCount`：failure after ** **（ by ，
//! not —— is old ， `config.rs`）。
//! - `noRetryStatusCodes`： up （ not in 1 ，
//! by down 1 ， `config.rs` `KEY_RETRY_NO_RETRY_CODES`）。
//!
//! `upstream::provider_loop`（ in `send_with_retry`，
//! in `attempt_queue`）。
//!
//! 1 in config.json（ `config.rs`）， by set
//! 「 use → request」 HTTP —— and /api/retention 1 ：
//! 、、return after 。
//!
//! and /api/retention ： this ** no has use **（ not 、 not ），
//! after for down 1 failurerequest（， not ）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{
    self, RetryPatch, RetrySettings, KEY_RETRY_ACCOUNT_SWITCH_COUNT, KEY_RETRY_COUNT,
    KEY_RETRY_INTERVAL_SECONDS, KEY_RETRY_NO_RETRY_CODES, RETRY_CODE_MAX, RETRY_CODE_MIN,
    RETRY_MAX_COUNT, RETRY_MAX_INTERVAL_SECONDS, RETRY_MAX_NO_RETRY_CODES, RETRY_MIN_COUNT,
    RETRY_MIN_INTERVAL_SECONDS,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/retry
pub async fn get_retry(State(_state): State<ServerState>) -> Response {
    ok_json(retry_json(config::retry_settings()))
}

/// PUT /api/retry —— body `{retryCount?, retryCrossProviderCount?, retryIntervalSeconds?, noRetryStatusCodes?}`
///
/// （ ，null ）。 after ： config.json
/// → return** after ** （ before use response new ， not GET 1 ）。
pub async fn put_retry(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // ── ： 1 （ and put_retention 1 ）─────────
    // has 1 when not ，「、 no 」 set
    let mut patch = RetryPatch::default();
    let mut targets = [
        (
            KEY_RETRY_COUNT,
            object.get(KEY_RETRY_COUNT),
            RETRY_MIN_COUNT,
            RETRY_MAX_COUNT,
            &mut patch.count,
        ),
        (
            KEY_RETRY_ACCOUNT_SWITCH_COUNT,
            object.get(KEY_RETRY_ACCOUNT_SWITCH_COUNT),
            RETRY_MIN_COUNT,
            RETRY_MAX_COUNT,
            &mut patch.account_switch_count,
        ),
        (
            KEY_RETRY_INTERVAL_SECONDS,
            object.get(KEY_RETRY_INTERVAL_SECONDS),
            RETRY_MIN_INTERVAL_SECONDS,
            RETRY_MAX_INTERVAL_SECONDS,
            &mut patch.interval_seconds,
        ),
    ];
    let mut updated = false;
    for (key, value, min, max, slot) in targets.iter_mut() {
        let Some(value) = value else {
            continue;
        };
        // null as 「 this 1 not 」： before when as null
        if value.is_null() {
            continue;
        }
        match parse_bounded_int(key, value, *min, *max) {
            Ok(number) => {
                **slot = Some(number);
                updated = true;
            }
            Err(message) => return errors::management_error(400, message),
        }
    }

    // ── ：error（）─────────────────────
    // and 1 ：， not 。 null / 
    // （ this 1 not ）；empty is = empty（error）。
    if let Some(value) = object.get(KEY_RETRY_NO_RETRY_CODES) {
        if !value.is_null() {
            match parse_no_retry_codes(value) {
                Ok(codes) => {
                    patch.no_retry_codes = Some(codes);
                    updated = true;
                }
                Err(message) => return errors::management_error(400, message),
            }
        }
    }

    if updated && !config::set_retry(patch) {
        // failure： already update（）， after will to old ——
        // must use ，otherwise「set」 will 
        logging::log("[Config]", "⚠️  请求重试设置写入 config.json 失败，本次运行内仍生效");
    }

    let settings = config::retry_settings();
    if updated {
        logging::log(
            "[Config]",
            &format!(
                "请求重试已更新: 同一账号 {} 次 / 最多换 {} 个账号 / 间隔 {} 秒 / 直接换号错误码 {:?}",
                settings.count,
                settings.account_switch_count,
                settings.interval_seconds,
                settings.no_retry_codes,
            ),
        );
    }
    ok_json(retry_json(settings))
}

/// response（GET and PUT use ， and config.rs 1 ——
/// and `stats_api::retention_json` 1 ： use not is ）。
///
/// for before is ****（`settings-panel.js` RETRY_FIELDS for ），
/// its in is old `retryCrossProviderCount`（config， config.rs）。
fn retry_json(settings: RetrySettings) -> Value {
    json!({
        KEY_RETRY_COUNT: settings.count,
        KEY_RETRY_ACCOUNT_SWITCH_COUNT: settings.account_switch_count,
        KEY_RETRY_INTERVAL_SECONDS: settings.interval_seconds,
        KEY_RETRY_NO_RETRY_CODES: settings.no_retry_codes.to_vec(),
    })
}

/// 「error」 ：、 100–599、 50 。
///
/// and `parse_bounded_int` 1 ： JSON （ `402.0` this ）；
/// has 1 400（ not ）， before ，。
fn parse_no_retry_codes(value: &Value) -> Result<Vec<u16>, String> {
    let Some(items) = value.as_array() else {
        return Err(format!("{KEY_RETRY_NO_RETRY_CODES} 必须是状态码数组（收到: {value}）"));
    };
    if items.len() > RETRY_MAX_NO_RETRY_CODES {
        return Err(format!(
            "{KEY_RETRY_NO_RETRY_CODES} 最多 {} 项（收到 {} 项）",
            RETRY_MAX_NO_RETRY_CODES,
            items.len()
        ));
    }
    let mut codes = Vec::with_capacity(items.len());
    for item in items {
        let number = item.as_i64().or_else(|| {
            item.as_f64()
                .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                .map(|raw| raw as i64)
        });
        let Some(number) = number else {
            return Err(format!("{KEY_RETRY_NO_RETRY_CODES} 里必须是整数（收到: {item}）"));
        };
        if !(i64::from(RETRY_CODE_MIN)..=i64::from(RETRY_CODE_MAX)).contains(&number) {
            return Err(format!(
                "{KEY_RETRY_NO_RETRY_CODES} 里的值必须是 {}-{} 的 HTTP 状态码（收到: {number}）",
                RETRY_CODE_MIN, RETRY_CODE_MAX
            ));
        }
        codes.push(number as u16);
    }
    codes.sort_unstable();
    codes.dedup();
    Ok(codes)
}

/// ：must is min–max ，otherwise to can 400 。
///
/// and `stats_api::parse_days` 1 ： JSON （ `5.0` this 
/// ， JS number ）； `"5"` as 。
fn parse_bounded_int(key: &str, value: &Value, min: i64, max: i64) -> Result<i64, String> {
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
        Some(number) if (min..=max).contains(&number) => Ok(number),
        _ => Err(format!("{key} 必须是 {min}-{max} 的整数（收到: {value}）")),
    }
}
