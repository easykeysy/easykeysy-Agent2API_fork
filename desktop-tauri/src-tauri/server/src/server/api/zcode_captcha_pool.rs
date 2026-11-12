//! GET/PUT /api/zcode/captcha-pool —— ****set。
//!
//! ── and `/api/zcode/captcha` （， not ）─────────
//! that is ：GET is ****，POST is 
//! （ `zcode_captcha.rs` module header）； this is **set**——、
//! 、limit（ `config::CaptchaPoolSettings`）。
//! GET stats response will this 1 and to （`target` / `pollMs` /
//! `maxPerRound`）， after down 1 by new ，** not use **。
//!
//! ── Why need set ────────────────────────────────────────
//! （3 / 2500 / 1）， is 「 1 use not 」 ：
//! 1 、120 seconds， for has 。 when not up 
//! ，、request 503 —— use need need can by request
//! （ / limit、）， need can in when 。
//! set ： is quota， 1 。
//!
//! and /api/queue 1 ：、、return** after ** 、
//! none use （ not ）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{
    self, CaptchaPoolPatch, CaptchaPoolSettings, KEY_CAPTCHA_POOL_MAX_PER_ROUND,
    KEY_CAPTCHA_POOL_POLL_MS, KEY_CAPTCHA_POOL_TARGET, CAPTCHA_POOL_MAX_MAX_PER_ROUND,
    CAPTCHA_POOL_MAX_POLL_MS, CAPTCHA_POOL_MAX_TARGET, CAPTCHA_POOL_MIN_MAX_PER_ROUND,
    CAPTCHA_POOL_MIN_POLL_MS, CAPTCHA_POOL_MIN_TARGET,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/zcode/captcha-pool
pub async fn get_captcha_pool(State(_state): State<ServerState>) -> Response {
    ok_json(captcha_pool_json(config::captcha_pool_settings()))
}

/// PUT /api/zcode/captcha-pool —— body `{zcodeCaptchaPoolTarget?, zcodeCaptchaPoolPollMs?, zcodeCaptchaPoolMaxPerRound?}`
///
/// （ ，null ）。 after ：config
/// → return** after ** （ before use response new ， not GET 1 ）。
pub async fn put_captcha_pool(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // and （ and put_queue 1 ）： has 1 when 
    // not ，「、 no 」 set
    let mut patch = CaptchaPoolPatch::default();
    let mut updated = false;
    let targets = [
        (
            KEY_CAPTCHA_POOL_TARGET,
            object.get(KEY_CAPTCHA_POOL_TARGET),
            CAPTCHA_POOL_MIN_TARGET,
            CAPTCHA_POOL_MAX_TARGET,
            "库存目标",
            &mut patch.target,
        ),
        (
            KEY_CAPTCHA_POOL_POLL_MS,
            object.get(KEY_CAPTCHA_POOL_POLL_MS),
            CAPTCHA_POOL_MIN_POLL_MS,
            CAPTCHA_POOL_MAX_POLL_MS,
            "轮询间隔",
            &mut patch.poll_ms,
        ),
        (
            KEY_CAPTCHA_POOL_MAX_PER_ROUND,
            object.get(KEY_CAPTCHA_POOL_MAX_PER_ROUND),
            CAPTCHA_POOL_MIN_MAX_PER_ROUND,
            CAPTCHA_POOL_MAX_MAX_PER_ROUND,
            "单轮铸造上限",
            &mut patch.max_per_round,
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

    if updated && !config::set_captcha_pool(patch) {
        // failure： already update（）， after will to old ——
        // must use ，otherwise「set」 will 
        logging::log("[Config]", "⚠️  验证码令牌池设置写入失败，本次运行内仍生效");
    }

    let settings = config::captcha_pool_settings();
    if updated {
        logging::log(
            "[Config]",
            &format!(
                "验证码令牌池已更新: 目标 {} 个 / 轮询 {}ms / 单轮最多 {} 个",
                settings.target, settings.poll_ms, settings.max_per_round
            ),
        );
    }
    ok_json(captcha_pool_json(settings))
}

/// response（GET and PUT use ， use not is ——
/// and `queue_api::queue_json` 1 ， for before is ）。
fn captcha_pool_json(settings: CaptchaPoolSettings) -> Value {
    json!({
        KEY_CAPTCHA_POOL_TARGET: settings.target,
        KEY_CAPTCHA_POOL_POLL_MS: settings.poll_ms,
        KEY_CAPTCHA_POOL_MAX_PER_ROUND: settings.max_per_round,
    })
}

/// ：must is ，otherwise to can 400 。
/// and `queue_api::parse_bounded_int` 1 （ JSON ， `2.0`
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
