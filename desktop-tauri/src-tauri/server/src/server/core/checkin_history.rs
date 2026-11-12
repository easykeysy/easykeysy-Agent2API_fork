//! to —— to in 「 to 」 。
//!
//! ── Why need need ────────────────────────────────────────────
//! when to has `autoCheckin.lastResult` 1 （ down 1 ）， can to 
//! 「 after 1 」 ； to in when need （ when / / ），
//! to total config.json `checkinHistory`。
//!
//! ── （id=None）， to not ─────────────────
//! when is 「 1 」 total； to is / up ，
//! already `checkinAt` in daily to ， when will 
//! 「 09:12 -C」 this 。
//!
//! ── Why core not is api ────────────────────────────────
//! is `billing::checkin::run_checkin`（ when to and use that 1 ），
//! in core ；`core::auto_checkin` config already is （ module header
//! 「config」）， use 1 ：`config::update_raw_field` and ，
//! config.json its 。
//!
//! ── ────────────────────────────────────────────────────
//! [`MAX_ENTRIES`]（20）： 1 when 1 + if ，20 
//! ； will config.json ， no has 。

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::auto_checkin;
use crate::server::logging;

/// config.json to 
const CONFIG_KEY: &str = "checkinHistory";
/// （ new in before ， ）
pub const MAX_ENTRIES: usize = 20;

/// （ new in before ）。 / to empty —— and auto_checkin raw_object
/// 1 ： old config.json no has this ，must is default。
pub fn list() -> Vec<Value> {
    match config::current().raw().get(CONFIG_KEY) {
        Some(Value::Array(items)) => items.clone(),
        _ => Vec::new(),
    }
}

/// 1 to total。`summary` is `run_checkin` return（ results），
/// this need 1 ，`results` not ——
/// failure and `autoCheckin.lastResult` 1 （`（error）`， before 5 ），
/// to in and when to is 1 failure。
///
/// failure、 not to up ： not not this to 
/// （ already in up to ）。
pub fn record(summary: &Value, reason: &str) {
    let number = |key: &str| summary.get(key).and_then(Value::as_u64).unwrap_or(0);
    let failures: Vec<String> = summary
        .get("results")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(crate::server::core::billing::checkin::failure_of)
                .collect()
        })
        .unwrap_or_default();
    let now_ms = logging::now_ms();
    let entry = json!({
        "at": now_ms,
        // by when （ up ， core::beijing）： 
        // 「」 and to 「」must is 1 ，otherwise down when will 
        "date": auto_checkin::today_key(),
        "reason": reason,
        "succeeded": number("succeeded"),
        // （WorkBuddy ）： and succeeded statistics —— 
        // success not checkinAt， need 1 ， when this 1 
        // 「 that 」
        "active": number("active"),
        "total": number("total"),
        "skipped": number("skipped"),
        "failed": failures.iter().take(5).cloned().collect::<Vec<_>>(),
        "failedCount": failures.len(),
    });
    let mut entries = list();
    entries.insert(0, entry);
    entries.truncate(MAX_ENTRIES);
    write(entries);
}

/// and `checkinHistory`（ and auto_checkin::write_state 1 ：
/// `update_raw_field` by ，config.json its ）
fn write(entries: Vec<Value>) {
    config::update_raw_field(CONFIG_KEY, Value::Array(entries));
}
