//! when —— and `core::auto_checkin` 1 （`chrono` + `Local`）。
//!
//! Why 1 ： for 「 1 」「 1 when 」 must** has 1 **。
//! by 、 by 、 by ， need has 1 use UTC or 
//! ， will （e.g. 9 18 and 「」 not is 1 ）。
//! this ， and `chrono::Local`。
//!
//! and to （** has ，「 for 」**）──────────
//! this is ** when **（`DateTime<Local>`）—— is 「 
//! use 」， when is for 。 to use ** UTC+8**
//! （`core::beijing`）： that is up （ when ）。 not can ——
//! to will issue #138（ down when ），
//! UTC+8 will in when to 「」。
//! `format("%Y-%m-%d")` （** not can use UTC **：UTC+8 
//! by UTC will to before 1 ， will 1 ）。

use chrono::{DateTime, Local, NaiveDate, TimeZone, Timelike};

/// currentms Unix when （ use logging ，）
pub(super) fn now_ms() -> i64 {
    crate::server::logging::now_ms()
}

/// when 
pub(super) fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// ms when → when when 。
///
/// `single()` for when has ； has file will return None，
/// this when to epoch not is unwrap panic —— 1 not this is 
/// （`DateTime<Local>` is DST when 1 ， use `single()` not is 
/// `earliest()` is as when none， not need need 1 ）。
pub(super) fn ms_to_local(ts: i64) -> DateTime<Local> {
    Local
        .timestamp_millis_opt(ts)
        .single()
        .or_else(|| Local.timestamp_millis_opt(0).single())
        .unwrap_or_else(Local::now)
}

/// ms when in ** when ** 1 （、 by use ）
pub(super) fn day_of(ts: i64) -> NaiveDate {
    ms_to_local(ts).date_naive()
}

/// `YYYY-MM-DD`。
///
/// + when ， can use 
/// （ use `BTreeMap<String, _>` is ）。
pub(super) fn date_key(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}

/// `YYYY-MM-DDTHH`（ 24 when use ）。
/// `%H` before when `DateTime<Local>`， is 「14 」 is use 
/// when up 14 ， not is UTC 14 。
pub(super) fn hour_key(now: DateTime<Local>) -> String {
    now.format("%Y-%m-%dT%H").to_string()
}

/// （/seconds/seconds）。
/// `with_*` in when can can return None， this when and to when ——
/// available 1 not need 1 （ in none when ， not to ）。
pub(super) fn hour_floor(now: DateTime<Local>) -> DateTime<Local> {
    now.with_minute(0)
        .and_then(|value| value.with_second(0))
        .and_then(|value| value.with_nanosecond(0))
        .unwrap_or(now)
}

/// for ms when 。
///
/// Why need need ： by ts ， need by 「 N 」 when ，
/// 1 **** down can use `partition_point` 1 
/// （O(log n)、 not ）， ——
/// this in when will ，must。
///
/// when can can not in （`LocalResult::None`）， this when 
/// when （`earliest()`） not is 1 ； in none when ， not to 。
pub(super) fn local_midnight_ms(day: NaiveDate) -> i64 {
    day.and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).earliest())
        .map(|value| value.timestamp_millis())
        // ： not when to epoch（ will ）
        .unwrap_or(0)
}
