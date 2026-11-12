//! config **** and ：`Map` 、。
//!
//! ── Why 1 ──────────────────────────────────────────
//! `mod.rs` is 「config 」（、 setter、）， this 1 
//! is ** not ** 。 `build()`（ when 
//! 1 ） and `set_*` use ， and 、 and file no has —— after 
//! `mod.rs` not as this and to 900 up （ file
//! ）， and 「config has this 1 」 this in file up can 。
//!
//! ── 1 （ before after 1 ）───────────────────────
//! config is ** to ** （ before is `config.json`， in available `sqlite3`
//! that 1 ）， must「 / not / when 
//! 」， and 1 ：** 1 to default， not 、 not **。
//! - not ： 1 not this not ；
//! - not ： 5000 3650 will use as 「set」， to 
//! default and to not 1 ，。
//! in PUT already by 1 （ that is 400 ）——
//! use （`RETENTION_MIN_DAYS` etc ）， not will 「 60
//! 」 this 。
//!
//! ── Why in this ──────────────────────────────────
//! 「configfile > > default」 this by `mod.rs` `build()`
//! ， `env_api_key()` / `env_text()` is —— and 
//! 1 「 1 」 this 1 ， will 
//! 1 。

use serde_json::{Map, Value};

use super::types::*;

/// API Key（ new ， old ；empty，emptynot configured）
pub(super) fn env_api_key() -> Option<String> {
    env_api_key_value()
}

/// by 1 valid（`AGENT2API_PROXY_API_KEY` > old ）
pub(crate) fn env_api_key_value() -> Option<String> {
    for name in ["AGENT2API_PROXY_API_KEY", "WORKBUDDY_PROXY_API_KEY"] {
        if let Ok(value) = std::env::var(name) {
            let trimmed = value.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

/// empty
pub(super) fn env_text(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// ，default as 1，0 as 。
///
/// has config** no has ** `captchaEnabled`（set from ） when —— 
/// 「config > > default」module header。 `0` 1 by 
/// ： is / ， can 1 。
pub(super) fn env_captcha_enabled() -> bool {
    std::env::var("AGENT2API_CAPTCHA_ENABLED")
        .map(|value| value.trim() != "0")
        .unwrap_or(true)
}

/// `AGENT2API_STRIP_NEWLINE_KEEPALIVE` （**default**）。
///
/// and `env_captcha_enabled` " 0 " not ： this is ** down to client
/// **（ content ）， is 1 ，
/// `1` / `true` / `on`（ignore and empty）， its 1 ——
/// includingempty。`AGENT2API_STRIP_NEWLINE_KEEPALIVE=` this " empty" compose
/// in Docker （set is this ）， by 0 will 。
pub(super) fn env_strip_newline_keepalive() -> bool {
    switch_on_from_env_text(std::env::var("AGENT2API_STRIP_NEWLINE_KEEPALIVE").ok().as_deref())
}

/// `AGENT2API_AUTO_CONTINUE` （**default**）。
///
/// and `AGENT2API_STRIP_NEWLINE_KEEPALIVE` use 1 （[`switch_on_from_env_text`]），
/// to 1 ： this is ** up **、 is ，empty / 1 
/// —— can can no （ use in up 1 has ）， not need 1 compose 
/// use 。
pub(super) fn env_auto_continue() -> bool {
    switch_on_from_env_text(std::env::var("AGENT2API_AUTO_CONTINUE").ok().as_deref())
}

/// `AGENT2API_AUTO_CONTINUE_MAX` 。
///
/// and file 1 （[`int_in_range`]： / ** no **， not ），
/// `=0`、`=9`、`=abc`、`=3.5` and 「 no this 」 `None`，
/// by [`auto_continue_rounds_from`] to default。
pub(super) fn env_auto_continue_max_rounds() -> Option<i64> {
    bounded_int_from_env_text(
        std::env::var("AGENT2API_AUTO_CONTINUE_MAX").ok().as_deref(),
        AUTO_CONTINUE_MIN_ROUNDS,
        AUTO_CONTINUE_MAX_ROUNDS,
    )
}

/// 「 」 1 ： `1` / `true` / `on`
/// （ignore and empty）， its 1 。
///
/// ── Why ────────────────────────────────────────────
/// this then has （、）， must**default** and 
/// not 「 0 」： 1 ， will 1 （e.g. `yes`） 1 not ，
/// use in compose 1 will to —— this not ， can 1 。
///
/// parameter is 「 」 not is ： need can 。
/// `std::env::set_var` in 1 can ， and use （ 
/// `core::providers::trae::forward::forward_at` ），「」 by use 
/// `None` ， not 。
pub(super) fn switch_on_from_env_text(raw: Option<&str>) -> bool {
    matches!(
        raw.map(|value| value.trim().to_ascii_lowercase()).as_deref(),
        Some("1") | Some("true") | Some("on")
    )
}

/// → ： / empty / / is `None`。
///
/// and [`int_in_range`] 1 to （** not **）， is is not is JSON
/// ：`3.0` in file（JSON 3 and 3.0 is 1 ）， in not ——
/// that is in shell ，`"3"` is ， down `3.0` not ，
/// will `AGENT2API_AUTO_CONTINUE_MAX=3.O`（ O） this 。
pub(super) fn bounded_int_from_env_text(raw: Option<&str>, min: i64, max: i64) -> Option<i64> {
    let text = raw.map(str::trim).filter(|text| !text.is_empty())?;
    let value = text.parse::<i64>().ok()?;
    (min..=max).contains(&value).then_some(value)
}

/// JSON → ： not / / is `None`
/// （** not **， by module header）。
///
/// ── Why this 1 is `pub(crate)` not is `pub(super)` ──────────
/// must use 1 —— module header that 「 not will 60 
/// 」 is this ， not is ，including** 1 **：
/// `30.0` this ，（`api::auto_continue` PUT ）must，
/// otherwise before `3.0` will 400， 1 。
pub(crate) fn int_in_range(value: &Value, min: i64, max: i64) -> Option<i64> {
    let parsed = match value {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                .map(|raw| raw as i64)
        }),
        _ => None,
    };
    parsed.filter(|value| (min..=max).contains(value))
}

/// from JSON 「 」，** not default**： / not /
/// / 1 `None`， use down 1 （ or default）。
///
/// in [`int_in_range`]（ use ）。
pub(super) fn bounded_int_option(map: &Map<String, Value>, key: &str, min: i64, max: i64) -> Option<i64> {
    map.get(key).and_then(|value| int_in_range(value, min, max))
}

/// from JSON 「 」： / not / / 
/// 1 to `default`。
///
/// ****： config.json when ， will 
/// as 「set」，default and use in set to not 1 when 。
/// in PUT already by 1 （ that is 400 ）。
///
/// `30.0` this （ and `parse_retention_value` 1 ）：JSON 
/// `30` and `30.0` is ， as after to default will its 
/// （ use file when 30 30.0 is can can ）。
pub(super) fn bounded_int_field(map: &Map<String, Value>, key: &str, default: i64, min: i64, max: i64) -> i64 {
    bounded_int_option(map, key, min, max).unwrap_or(default)
}

/// from JSON empty
pub(super) fn string_field(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// from JSON （`bounded_int_field` ）
pub(super) fn days_field(map: &Map<String, Value>, key: &str, default: i64) -> i64 {
    bounded_int_field(map, key, default, RETENTION_MIN_DAYS, RETENTION_MAX_DAYS)
}

/// or set to default。
pub(super) fn retention_from(map: &Map<String, Value>) -> RetentionSettings {
    let defaults = RetentionSettings::default();
    RetentionSettings {
        log_days: days_field(map, KEY_LOG_RETENTION_DAYS, defaults.log_days),
        log_count: bounded_int_field(
            map,
            KEY_LOG_RETENTION_COUNT,
            defaults.log_count,
            RETENTION_MIN_COUNT,
            RETENTION_MAX_COUNT,
        ),
        request_days: days_field(map, KEY_REQUEST_RETENTION_DAYS, defaults.request_days),
        request_count: bounded_int_field(
            map,
            KEY_REQUEST_RETENTION_COUNT,
            defaults.request_count,
            RETENTION_MIN_COUNT,
            RETENTION_MAX_COUNT,
        ),
        daily_days: days_field(map, KEY_DAILY_RETENTION_DAYS, defaults.daily_days),
    }
}

// ─── when （scheduledTasks.*）──────────────────

/// from `scheduledTasks` 1 for ； / not empty for 
/// （ is `enabled` use default、`interval` use default， and 「 use no 」 etc ）。
pub(super) fn task_object(map: &Map<String, Value>, key: &str) -> Map<String, Value> {
    map.get(KEY_SCHEDULED_TASKS)
        .and_then(Value::as_object)
        .and_then(|tasks| tasks.get(key))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// from 1 for 。
///
/// and `days_field` 1 （****、、 in 
/// `min..=max` ）， is by use to ： 
/// （ after by minutes、 before new by seconds）， use 1 will its in 1 。
pub(super) fn interval_field(task: &Map<String, Value>, default: i64, min: i64, max: i64) -> i64 {
    let parsed = task.get("interval").and_then(|value| match value {
        Value::Number(number) => number.as_i64().or_else(|| {
            number
                .as_f64()
                .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                .map(|raw| raw as i64)
        }),
        _ => None,
    });
    parsed.filter(|value| (min..=max).contains(value)).unwrap_or(default)
}

/// from 1 for 。
///
/// ** by **： this in before is none （ 10 minutes、
/// before 10 seconds、 new /v1/models ），
/// up config.json no has `scheduledTasks` —— if 「 no 」「」，
/// use no ， not 。（`PUT /api/scheduled-tasks`）
/// then need 。
pub(super) fn task_enabled(task: &Map<String, Value>, default: bool) -> bool {
    task.get("enabled").and_then(Value::as_bool).unwrap_or(default)
}

/// by JSON （ use default）。
/// old config can can `usageQuery` （ already ）：
/// by **unknownignore**， not to is not in ，set when will 。
pub(super) fn scheduled_from(map: &Map<String, Value>) -> ScheduledSettings {
    let defaults = ScheduledSettings::default();
    let task = |key: &str, interval: i64, min: i64, max: i64| {
        let object = task_object(map, key);
        IntervalTask {
            enabled: task_enabled(&object, true),
            interval: interval_field(&object, interval, min, max),
        }
    };
    ScheduledSettings {
        credential_maintenance: task(
            KEY_CREDENTIAL_MAINTENANCE,
            defaults.credential_maintenance.interval,
            INTERVAL_MIN_MINUTES,
            INTERVAL_MAX_MINUTES,
        ),
        model_refresh: task(
            KEY_MODEL_REFRESH,
            defaults.model_refresh.interval,
            INTERVAL_MIN_MINUTES,
            INTERVAL_MAX_MINUTES,
        ),
        logs_auto_refresh: task(
            KEY_LOGS_AUTO_REFRESH,
            defaults.logs_auto_refresh.interval,
            INTERVAL_MIN_SECONDS,
            INTERVAL_MAX_SECONDS,
        ),
        requests_auto_refresh: task(
            KEY_REQUESTS_AUTO_REFRESH,
            defaults.requests_auto_refresh.interval,
            INTERVAL_MIN_SECONDS,
            INTERVAL_MAX_SECONDS,
        ),
        report_auto_refresh: task(
            KEY_REPORT_AUTO_REFRESH,
            defaults.report_auto_refresh.interval,
            INTERVAL_MIN_SECONDS,
            INTERVAL_MAX_SECONDS,
        ),
        update_check: task(
            KEY_UPDATE_CHECK,
            defaults.update_check.interval,
            INTERVAL_MIN_MINUTES,
            INTERVAL_MAX_MINUTES,
        ),
    }
}

// ─── requestset （retryCount / retryCrossProviderCount / retryIntervalSeconds / noRetryStatusCodes）──

/// by JSON requestset（ use default）
pub(super) fn retry_from(map: &Map<String, Value>) -> RetrySettings {
    let defaults = RetrySettings::default();
    RetrySettings {
        count: bounded_int_field(
            map,
            KEY_RETRY_COUNT,
            defaults.count,
            RETRY_MIN_COUNT,
            RETRY_MAX_COUNT,
        ),
        account_switch_count: bounded_int_field(
            map,
            KEY_RETRY_ACCOUNT_SWITCH_COUNT,
            defaults.account_switch_count,
            RETRY_MIN_COUNT,
            RETRY_MAX_COUNT,
        ),
        interval_seconds: bounded_int_field(
            map,
            KEY_RETRY_INTERVAL_SECONDS,
            defaults.interval_seconds,
            RETRY_MIN_INTERVAL_SECONDS,
            RETRY_MAX_INTERVAL_SECONDS,
        ),
        no_retry_codes: no_retry_codes_field(map, KEY_RETRY_NO_RETRY_CODES),
    }
}

/// 「error」 。
///
/// ── ────────────────────────────────────────────
/// - / not is → **default**（[402]）：config from no this 1 is ，
/// default this in this when ；（） and 「 no 」none，
/// by 1 （default，module header）。
/// - empty → **empty**： this is use （error），
/// must and 「 no 」。
/// - （ / 100–599）→ ** this **：
/// already （400）， can to this ，default
/// 。： contains，none，
/// 1 config.json can 。
fn no_retry_codes_field(map: &Map<String, Value>, key: &str) -> std::sync::Arc<[u16]> {
    let Some(Value::Array(items)) = map.get(key) else {
        return std::sync::Arc::from(DEFAULT_NO_RETRY_CODES);
    };
    let mut codes: Vec<u16> = items
        .iter()
        .filter_map(|item| {
            let number = item.as_i64().or_else(|| {
                item.as_f64()
                    .filter(|raw| raw.is_finite() && raw.fract() == 0.0)
                    .map(|raw| raw as i64)
            });
            u16::try_from(number?)
                .ok()
                .filter(|code| (RETRY_CODE_MIN..=RETRY_CODE_MAX).contains(code))
        })
        .collect();
    codes.sort_unstable();
    codes.dedup();
    std::sync::Arc::from(codes)
}

// ─── up request when （，seconds）────────────────────────

/// by JSON when （ use default；default，module header）。
///
/// and default `types.rs`——「 1~3600」 in （timeouts_api）
/// is 400 ， in this is default： 0 or 99999 when ， can to 
/// 30/300 not need to 1 （0 ms etc use this ）。
pub(super) fn timeouts_from(map: &Map<String, Value>) -> TimeoutSettings {
    let defaults = TimeoutSettings::default();
    TimeoutSettings {
        connect_seconds: bounded_int_field(
            map,
            KEY_TIMEOUT_CONNECT_SECONDS,
            defaults.connect_seconds,
            TIMEOUT_MIN_SECONDS,
            TIMEOUT_MAX_SECONDS,
        ),
        headers_seconds: bounded_int_field(
            map,
            KEY_TIMEOUT_HEADERS_SECONDS,
            defaults.headers_seconds,
            TIMEOUT_MIN_SECONDS,
            TIMEOUT_MAX_SECONDS,
        ),
        stream_idle_seconds: bounded_int_field(
            map,
            KEY_TIMEOUT_STREAM_IDLE_SECONDS,
            defaults.stream_idle_seconds,
            TIMEOUT_MIN_SECONDS,
            TIMEOUT_MAX_SECONDS,
        ),
        body_seconds: bounded_int_field(
            map,
            KEY_TIMEOUT_BODY_SECONDS,
            defaults.body_seconds,
            TIMEOUT_MIN_SECONDS,
            TIMEOUT_MAX_SECONDS,
        ),
    }
}

// ─── etc （， and seconds）──────────────────────────

/// by JSON etc （ use default；default）。
///
/// and `timeouts_from` 1 ： in （`queue_api`） is 400 ，
/// in this is default —— 1 when ， can to 2 / ，
/// not need to 1 （e.g. will ）。
pub(super) fn queue_from(map: &Map<String, Value>) -> QueueSettings {
    let defaults = QueueSettings::default();
    QueueSettings {
        max_waits: bounded_int_field(
            map,
            KEY_QUEUE_MAX_WAITS,
            defaults.max_waits,
            QUEUE_MIN_MAX_WAITS,
            QUEUE_MAX_MAX_WAITS,
        ),
        wait_seconds: bounded_int_field(
            map,
            KEY_QUEUE_WAIT_SECONDS,
            defaults.wait_seconds,
            QUEUE_MIN_WAIT_SECONDS,
            QUEUE_MAX_WAIT_SECONDS,
        ),
    }
}

// ─── ZCode （： / / limit）──────

/// by JSON （ use default；default）。
///
/// and `queue_from` 1 ： in （`zcode_captcha_pool_api`） is 
/// 400 ， in this is default —— 1 when ， can to ，
/// not need to 1 （e.g. 0 ms will empty）。
pub(super) fn captcha_pool_from(map: &Map<String, Value>) -> CaptchaPoolSettings {
    let defaults = CaptchaPoolSettings::default();
    CaptchaPoolSettings {
        target: bounded_int_field(
            map,
            KEY_CAPTCHA_POOL_TARGET,
            defaults.target,
            CAPTCHA_POOL_MIN_TARGET,
            CAPTCHA_POOL_MAX_TARGET,
        ),
        poll_ms: bounded_int_field(
            map,
            KEY_CAPTCHA_POOL_POLL_MS,
            defaults.poll_ms,
            CAPTCHA_POOL_MIN_POLL_MS,
            CAPTCHA_POOL_MAX_POLL_MS,
        ),
        max_per_round: bounded_int_field(
            map,
            KEY_CAPTCHA_POOL_MAX_PER_ROUND,
            defaults.max_per_round,
            CAPTCHA_POOL_MIN_MAX_PER_ROUND,
            CAPTCHA_POOL_MAX_MAX_PER_ROUND,
        ),
    }
}

// ─── by （providerRoute，，）───────────

// ─── will （ + / / ）──────────────

/// by JSON will set（ use default；default）。
///
/// ：** has `false` **， / （`"on"` / `1`） by 
/// —— and `debug_mode` that 。 is 「default、 can 1 」：config no has this 
/// when 1 （ `KEY_IMAGE_COMPRESS_ENABLED` ）。
///
/// parameter： in （`api::image_compress`） is 400 ， in this is default
/// —— 1 when ， can to 2048 / 80 / 512（OmniProxy default），
/// not need to 1 （e.g. as 0 will 1 ）。
pub(super) fn image_compress_from(map: &Map<String, Value>) -> ImageCompressSettings {
    let defaults = ImageCompressSettings::default();
    ImageCompressSettings {
        enabled: map
            .get(KEY_IMAGE_COMPRESS_ENABLED)
            .and_then(Value::as_bool)
            .unwrap_or(true),
        max_edge: bounded_int_field(
            map,
            KEY_IMAGE_COMPRESS_MAX_EDGE,
            defaults.max_edge,
            IMAGE_COMPRESS_MIN_MAX_EDGE,
            IMAGE_COMPRESS_MAX_MAX_EDGE,
        ),
        quality: bounded_int_field(
            map,
            KEY_IMAGE_COMPRESS_QUALITY,
            defaults.quality,
            IMAGE_COMPRESS_MIN_QUALITY,
            IMAGE_COMPRESS_MAX_QUALITY,
        ),
        threshold_kb: bounded_int_field(
            map,
            KEY_IMAGE_COMPRESS_THRESHOLD_KB,
            defaults.threshold_kb,
            IMAGE_COMPRESS_MIN_THRESHOLD_KB,
            IMAGE_COMPRESS_MAX_THRESHOLD_KB,
        ),
    }
}

// ─── by （ / / / / ）──────────────

/// by JSON by set（ use default；default）。
///
/// and `timeouts_from` 1 ： in （`routing_api`） is 400 ，
/// in this is default —— 1 when ， can to default as not need 
/// to 1 （e.g. 0 will 1 failure）。
pub(super) fn routing_from(map: &Map<String, Value>) -> RoutingSettings {
    let defaults = RoutingSettings::default();
    let strategy = map
        .get(KEY_ROUTING_STRATEGY)
        .and_then(Value::as_str)
        .map(RouteStrategy::from_id)
        .unwrap_or(defaults.strategy);
    let sticky = map
        .get(KEY_ROUTING_STICKY)
        .and_then(Value::as_str)
        .map(StickyMode::from_id)
        .unwrap_or(defaults.sticky);
    let bool_field = |key: &str, default: bool| -> bool {
        map.get(key).and_then(Value::as_bool).unwrap_or(default)
    };
    RoutingSettings {
        strategy,
        sticky,
        failover_backoff: bool_field(KEY_ROUTING_FAILOVER_BACKOFF, defaults.failover_backoff),
        sink: bool_field(KEY_ROUTING_SINK, defaults.sink),
        breaker_enabled: bool_field(KEY_ROUTING_BREAKER_ENABLED, defaults.breaker_enabled),
        breaker_threshold: bounded_int_field(
            map,
            KEY_ROUTING_BREAKER_THRESHOLD,
            defaults.breaker_threshold,
            ROUTING_BREAKER_MIN_THRESHOLD,
            ROUTING_BREAKER_MAX_THRESHOLD,
        ),
        breaker_timeout_seconds: bounded_int_field(
            map,
            KEY_ROUTING_BREAKER_TIMEOUT,
            defaults.breaker_timeout_seconds,
            ROUTING_BREAKER_MIN_TIMEOUT,
            ROUTING_BREAKER_MAX_TIMEOUT,
        ),
        breaker_success_threshold: bounded_int_field(
            map,
            KEY_ROUTING_BREAKER_SUCCESS,
            defaults.breaker_success_threshold,
            ROUTING_BREAKER_MIN_SUCCESS,
            ROUTING_BREAKER_MAX_SUCCESS,
        ),
    }
}

// ─── （autoContinue / autoContinueMaxRounds）────────

/// by JSON 「」：file > > 。
///
/// and `build()` `strip_newline_keepalive` that 1 1 （** has 
/// `true` / `false` this 1 **， `"on"` / `1` no 、 to ），
/// is that in use ， this in [`resolve_auto_continue`]
/// —— and 「 has this 1 」 1 （module header）。
pub(super) fn auto_continue_from(map: &Map<String, Value>) -> bool {
    resolve_auto_continue(
        map.get(KEY_AUTO_CONTINUE).and_then(Value::as_bool),
        env_auto_continue(),
    )
}

/// 「file / → 」：file as ，**including `false`**。
///
/// （ not is in [`auto_continue_from`] ）：
/// this need can ， not can （ [`switch_on_from_env_text`]
/// ）。 is 「must is can 」—— 
/// `file || ` is 「file」， use in up not 。
pub(super) fn resolve_auto_continue(from_file: Option<bool>, from_env: bool) -> bool {
    from_file.unwrap_or(from_env)
}

/// by JSON limit：file > > default。
///
/// and `routing_from` 1 （ in `api::auto_continue` is 400 ，
/// in this is default）， 1 is —— and `env_auto_continue`
/// that for ：Docker need from ， need from config， not this 1 can 
/// 1 not can 。
///
/// Note**file to **， not is default： this 1 「unavailable」 
/// is 「 this 1 no to available」（ and etc ）， and module header 1 。
pub(super) fn auto_continue_rounds_from(map: &Map<String, Value>) -> i64 {
    resolve_rounds(
        bounded_int_option(
            map,
            KEY_AUTO_CONTINUE_MAX_ROUNDS,
            AUTO_CONTINUE_MIN_ROUNDS,
            AUTO_CONTINUE_MAX_ROUNDS,
        ),
        env_auto_continue_max_rounds(),
    )
}

/// 「file / → 」： not to availabledefault。
///
/// `Option` （ not is in [`auto_continue_rounds_from`]
/// ）： need can ， not can （
/// [`switch_on_from_env_text`] ）。default in this 1 ，
/// 「 not 、 is to down 1 」 this then has 1 1 。
pub(super) fn resolve_rounds(from_file: Option<i64>, from_env: Option<i64>) -> i64 {
    from_file.or(from_env).unwrap_or(DEFAULT_AUTO_CONTINUE_MAX_ROUNDS)
}

#[cfg(test)]
mod tests {
    //! ： 「 1 / true / on」、 「default」、
    //! and 「file > > default」 this 。
    //!
    //! ── Why this use not ────────────────────────
    //! `std::env::set_var` in 1 can ， and use （
    //! `core::providers::trae::forward::forward_at` ），
    //! 「 no 」 is has —— `None`。 
    //! parameter （[`switch_on_from_env_text`] / [`bounded_int_from_env_text`] /
    //! [`resolve_auto_continue`] / [`resolve_rounds`]），`env_*` is to 
    //! up that 1 。

    use super::*;
    use serde_json::json;

    /// 1 to 。
    fn map_of(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect()
    }

    /// `1` / `true` / `on`（ignore and empty）。
    ///
    /// ：「 0 」（`env_captcha_enabled` that ）。 by will 
    /// `AGENT2API_AUTO_CONTINUE=`（compose set ）****，
    /// is 1 no use up 。
    #[test]
    fn the_env_switch_only_accepts_one_true_or_on() {
        for on in ["1", "true", "on", " TRUE ", "On", "ON "] {
            assert!(switch_on_from_env_text(Some(on)), "{on} 应当算开启（大小写与首尾空白都要被吃掉）");
        }
        for off in ["", " ", "0", "yes", "10", "2", "no", "开启", "trueX", "one"] {
            assert!(!switch_on_from_env_text(Some(off)), "{off} 不该被当成开启");
        }
        assert!(!switch_on_from_env_text(None), "环境变量没设（None）必须是关");
    }

    /// ： has **** and in 。
    ///
    /// ： `parse::<f64>()` （ is `3.7` 3）， or when 
    /// to 8 —— after will as 99 。
    #[test]
    fn the_env_integer_must_be_a_plain_integer_in_range() {
        assert_eq!(bounded_int_from_env_text(Some("1"), 1, 8), Some(1), "下限要收");
        assert_eq!(bounded_int_from_env_text(Some(" 8 "), 1, 8), Some(8), "上限要收，空白要吃掉");
        for unusable in ["", " ", "0", "9", "-1", "3.0", "3.5", "abc", "8.", "+"] {
            assert_eq!(
                bounded_int_from_env_text(Some(unusable), 1, 8),
                None,
                "{unusable} 不是可用值，必须交回 None 让下一级兜（而不是夹紧）"
            );
        }
        assert_eq!(bounded_int_from_env_text(None, 1, 8), None);
    }

    /// file → **default， not **（`bounded_int_field` ）。
    ///
    /// ：`clamp(min, max)`。 99 if to set 8，
    /// will as 「8 is to 」， 99 from this not 。
    #[test]
    fn an_out_of_range_file_value_falls_back_to_the_default_not_the_bound() {
        let too_many = map_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(99))]);
        assert_eq!(
            bounded_int_option(&too_many, KEY_AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS),
            None,
            "越界必须表达成「这一级没给出可用值」，夹紧就不叫回落了"
        );
        assert_eq!(
            bounded_int_field(&too_many, KEY_AUTO_CONTINUE_MAX_ROUNDS, DEFAULT_AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS),
            DEFAULT_AUTO_CONTINUE_MAX_ROUNDS,
            "回落的是默认值 3，不是上限 8"
        );
        // up and 「 」
        let zero = map_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(0))]);
        assert_eq!(bounded_int_field(&zero, KEY_AUTO_CONTINUE_MAX_ROUNDS, DEFAULT_AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS), DEFAULT_AUTO_CONTINUE_MAX_ROUNDS, "0 在范围外（想关去关开关）");
        let text = map_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!("3"))]);
        assert_eq!(bounded_int_option(&text, KEY_AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS), None, "字符串数字不算（与写侧同一口径）");
        // `3.0` 3：JSON 3 and 3.0 is 1 ，file when 3.0 can can 
        let floaty = map_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(3.0))]);
        assert_eq!(bounded_int_option(&floaty, KEY_AUTO_CONTINUE_MAX_ROUNDS, AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS), Some(3));
    }

    /// ：file as （**including false and 1**），otherwise，otherwisedefault。
    ///
    /// ： `file || ` —— up is 「file」，
    /// use in up not （ already is false， that `on` 1 ）。
    #[test]
    fn the_file_value_beats_the_env_even_when_it_says_off() {
        assert!(!resolve_auto_continue(Some(false), true), "文件里的 false 必须盖过环境变量的开");
        assert!(resolve_auto_continue(Some(true), false), "文件里的 true 必须盖过环境变量的关");
        assert!(resolve_auto_continue(None, true), "文件没配时环境变量说话");
        assert!(!resolve_auto_continue(None, false), "两级都没配 → 默认关");

        assert_eq!(resolve_rounds(Some(1), Some(8)), 1, "文件里的 1 必须盖过环境变量的 8");
        assert_eq!(resolve_rounds(Some(8), None), 8);
        assert_eq!(resolve_rounds(None, Some(8)), 8, "文件不可用时环境变量说话");
        assert_eq!(resolve_rounds(None, None), DEFAULT_AUTO_CONTINUE_MAX_ROUNDS, "两级都不可用才落默认");
        // file in this already is None（ up use ）， up as 「 to down 1 」
        assert_eq!(resolve_rounds(None, Some(1)), 1);
    }

    /// ：`*_from(&map)` must and that 1 to 1 。
    ///
    /// ：`auto_continue_rounds_from` （e.g.limit use 
    /// `AUTO_CONTINUE_MAX_ROUNDS - 1`）， that will 8 in 「 no 」——
    /// down not ， is `parse` module header not 。
    #[test]
    fn the_map_level_helpers_follow_the_same_rules_as_the_pure_ones() {
        let at_the_top = map_of(&[
            (KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(8)),
            (KEY_AUTO_CONTINUE, json!(true)),
        ]);
        assert_eq!(
            auto_continue_rounds_from(&at_the_top),
            8,
            "上限边界值 8 必须能用（读写两侧共用同一对范围常量）"
        );
        assert!(auto_continue_from(&at_the_top), "文件写了 true 就必须是开");

        let off = map_of(&[
            (KEY_AUTO_CONTINUE, json!(false)),
            (KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(1)),
        ]);
        assert!(!auto_continue_from(&off));
        assert_eq!(auto_continue_rounds_from(&off), 1, "下限边界值 1 必须能用");

        // that 1 etc 「 no 」： / not down 1 can 
        let junk = map_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(0)), (KEY_AUTO_CONTINUE, json!("true"))]);
        assert_eq!(
            auto_continue_rounds_from(&junk),
            resolve_rounds(None, env_auto_continue_max_rounds()),
            "文件里的 0 不可用，落点必须与「文件没这项」逐字相同"
        );
        assert_eq!(
            auto_continue_from(&junk),
            resolve_auto_continue(None, env_auto_continue()),
            "文件里的字符串 true 不可用，落点必须与「文件没这项」逐字相同"
        );
    }
}
