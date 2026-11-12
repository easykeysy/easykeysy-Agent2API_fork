//! when （ UTC+8）—— to for 「」 1 。
//!
//! ── Why need has this 1 （issue #138）────────────────────────────
//! to 「」 is ** up **： in when before after 
//! （CodeArts by UTC+8 、 by 、Qoder 
//! 「daily 100 Credits」 10:00（UTC+8））。 can can in NAS /
//! Docker / VPS up ， that when not is UTC+8 —— when 
//! will 「」 ： up 00:01 when to in 
//! when 15:01， 「 already to 」 by error 。
//!
//! ── Why is not is when ─────────────────────────────
//! in when no has when ，`Asia/Shanghai` in 1970 after as +08:00 ——
//! 1 ， and not tzdata / when config。
//! `core::degrade` and `providers::codearts::welfare` already is 1 
//! （）； is to this 1 。
//!
//! ── and `request_stats::clock` （， has ）────────
//! requeststatistics by ** when **（ is 「 」）， that 
//! by `clock` ； to ** up **， use 。 not can ：
//! to will issue #138。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic， 1 Option 。

use chrono::{DateTime, FixedOffset, Offset};

/// UTC+8 （seconds）。
pub const OFFSET_SECONDS: i32 = 8 * 3600;

/// UTC+8 when for 。
///
/// 8 when in `FixedOffset` （±24h），`east_opt` not will failure；
/// not use unwrap is as release none panic —— to UTC ，
/// down can can 8 when ， abort 。
pub fn offset() -> FixedOffset {
    FixedOffset::east_opt(OFFSET_SECONDS).unwrap_or_else(|| chrono::Utc.fix())
}

/// ms when → when when ； when to None。
pub fn date_time_of_ms(ms: i64) -> Option<DateTime<FixedOffset>> {
    DateTime::from_timestamp_millis(ms).map(|utc| utc.with_timezone(&offset()))
}

/// when current when （ when and 1 ：`logging::now_ms`）。
pub fn now() -> DateTime<FixedOffset> {
    date_time_of_ms(crate::server::logging::now_ms())
        .unwrap_or_else(|| chrono::Utc::now().with_timezone(&offset()))
}

/// `YYYY-MM-DD`（， when ）。
pub fn date_key(now: DateTime<FixedOffset>) -> String {
    now.format("%Y-%m-%d").to_string()
}

/// ms when in when 1 （`YYYY-MM-DD`）； when to None。
pub fn date_key_of_ms(ms: i64) -> Option<String> {
    date_time_of_ms(ms).map(date_key)
}

/// when （`YYYY-MM-DD`）。
pub fn today_key() -> String {
    date_key(now())
}
