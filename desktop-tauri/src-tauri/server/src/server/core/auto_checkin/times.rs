//! to when + （） when 。
//!
//! ── Why need need 「 when 」──────────────────────────────────
//! to not is 「 can in 」： up daily has ， no when use to is 
//! 「 down 」（ in ）， can 。（2026-10-29）Qoder 
//! in 00:01 after 1 to 10:00 ， to down 7 「」；
//! use to is 「 is 00:01， 7 」。
//!
//! | | （ when ） | |
//! |---|---|---|
//! | Qoder（） | 10:00 | `providers::qoder::checkin` module header（） |
//! | ZCode（） | 19:00 | ：daily 19:00 can |
//!
//! when 「 + 1 minutes」： in that 1 seconds when ， up can can no down 。
//!
//! ── （ before ）──────────────────────────────────
//! up `providerTimes[]`（ use ）> （）> `time`。
//! is config****： no old by when ， has by when 。
//! use up that 1 ， not 。
//!
//! ── （）──────────────────────────────────────────
//! by `max(retryTime, to + etc )` ， and to 。
//! default 00:01–00:10、 10 minutes when ， 00:20 ，Qoder 10:20、ZCode 19:20。
//! not will before in etc ， not will when 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use std::collections::BTreeMap;

use chrono::{DateTime, FixedOffset};

use crate::server::core::auto_checkin::{normalize_time, today_at};
use crate::server::core::beijing;
use crate::server::core::checkin_schedules::Schedule;

/// not when ； from after 。
pub const DEFAULT_RETRY_TIME: &str = "00:10";
pub const DEFAULT_SPREAD_MINUTES: i64 = 9;

pub fn valid_range(start: &str, end: &str) -> bool {
    matches!((today_at_minutes(start), today_at_minutes(end)), (Some(start), Some(end)) if end >= start)
}

pub fn end_with_spread(start: &str, spread: i64) -> String {
    format_minutes((today_at_minutes(start).unwrap_or(1) + spread.max(0)).min(1439))
}

pub fn default_time_end(start: &str) -> String {
    end_with_spread(start, DEFAULT_SPREAD_MINUTES)
}

pub fn spread_minutes(schedule: &Schedule) -> i64 {
    (today_at_minutes(&schedule.time_end).unwrap_or(10)
        - today_at_minutes(&schedule.time).unwrap_or(1)).max(0)
}

pub fn effective_end(schedule: &Schedule, provider: &str) -> String {
    end_with_spread(&effective_time(schedule, provider), spread_minutes(schedule))
}

pub fn provider_window(schedule: &Schedule, provider: &str, now: DateTime<FixedOffset>) -> Option<(i64, i64)> {
    Some((today_at(&effective_time(schedule, provider), now)?.timestamp_millis(),
        today_at(&effective_end(schedule, provider), now)?.timestamp_millis()))
}

/// daily when （ when ， after 1 minutes）。
///
/// 「 when 」 1 。** has ** ——
/// by 「 that when 」 this ， etc to when 。
pub const PROVIDER_TIMES: [(&str, &str); 4] = [
    ("qoder", "10:01"),
    ("qoder-intl", "10:01"),
    ("zcode", "19:01"),
    ("zcode-intl", "19:01"),
];

/// when （ no has then None = when ）。
pub fn provider_time(provider: &str) -> Option<&'static str> {
    PROVIDER_TIMES
        .iter()
        .find(|(id, _)| *id == provider)
        .map(|(_, time)| *time)
}

/// 1 in ** when **： > > time。
pub fn effective_time(schedule: &Schedule, provider: &str) -> String {
    schedule
        .provider_times
        .get(provider)
        .filter(|time| !time.is_empty())
        .cloned()
        .or_else(|| provider_time(provider).map(str::to_string))
        .unwrap_or_else(|| schedule.time.clone())
}

/// in `date` that is already 
fn fired_on(map: &BTreeMap<String, String>, provider: &str, date: &str) -> bool {
    map.get(provider).is_some_and(|value| value == date)
}

/// is already （****）。
///
/// （`lastFiredProviders`） as empty = before old ： that when has 
/// 1 `lastFiredDate`， 1 is 「」。 this 
/// （ has ），must in —— otherwise that 1 will already 
/// 1 。
pub(crate) fn main_fired(schedule: &Schedule, provider: &str, today: &str) -> bool {
    if schedule.account_plan.date == today {
        return !schedule.account_plan.pending_for(provider);
    }
    if !schedule.last_fired_providers.is_empty() {
        return fired_on(&schedule.last_fired_providers, provider, today);
    }
    schedule.last_fired_date.as_deref() == Some(today)
}

/// this ： when already to 、 no 。
///
/// this is `tick` —— not has 「 1 1 」 ， by 
/// ** **， is 1 can in 00:01 / 10:01 / 19:01
/// 1 。
pub fn due_providers(
    schedule: &Schedule,
    now: DateTime<FixedOffset>,
    today: &str,
) -> Vec<String> {
    schedule
        .providers
        .iter()
        .filter(|provider| !main_fired(schedule, provider, today))
        .filter(|provider| time_passed(&effective_time(schedule, provider), now))
        .cloned()
        .collect()
}

/// when ：`max(config when , this to + etc )`。
/// and use ； as 0 when etc 1 minutes， then 。
pub fn provider_retry_time(schedule: &Schedule, provider: &str) -> Option<String> {
    if schedule.retry_time.is_empty() {
        return None;
    }
    let base = today_at_minutes(&schedule.retry_time)?;
    let main = today_at_minutes(&effective_end(schedule, provider))?;
    // ； 0（） when to 1 minutes —— and 「 etc 
    // 」 is ， after not this 
    let gap = schedule.retry_gap_minutes.max(1);
    let target = base.max(main + gap);
    (target < 1440).then(|| format_minutes(target))
}

/// this ： this when already to 、 no 。
///
/// when by [`provider_retry_time`] （ already 「 no not 」 this ： 1 
/// when ）。 this not —— that will 。
pub fn retry_due_providers(
    schedule: &Schedule,
    now: DateTime<FixedOffset>,
    today: &str,
) -> Vec<String> {
    schedule
        .providers
        .iter()
        .filter(|provider| crate::server::core::plugin_host::serves_provider(provider))
        .filter(|provider| !fired_on(&schedule.last_retried_providers, provider, today))
        .filter(|provider| main_fired(schedule, provider, today))
        .filter(|provider| {
            provider_retry_time(schedule, provider)
                .is_some_and(|time| time_passed(&time, now))
        })
        .cloned()
        .collect()
}

/// no （** no to **）—— to 「」 use 。
///
/// and [`due_providers`] in 「 not when 」： that is （ to ），
/// this is （ is ）。 = this returnempty。
pub fn pending_providers(schedule: &Schedule, today: &str) -> Vec<String> {
    schedule
        .providers
        .iter()
        .filter(|provider| !main_fired(schedule, provider, today))
        .cloned()
        .collect()
}

/// ** this no ** （ when already 、 is empty ）—— 「」 use 。
///
/// and [`pending_providers`] need ： up 08:00 1 00:01 ，
/// Qoder（10:01） no to ，** not is **「」 is 「 etc 」—— in 1 will 
/// use as 。
pub fn overdue_providers(
    schedule: &Schedule,
    now: DateTime<FixedOffset>,
    today: &str,
) -> Vec<String> {
    due_providers(schedule, now, today)
}

/// down 1 will when （ms）—— 「 down 」 use 。
///
/// ： no and when to → ； already / when already → 。
/// （ [`retry_next_at`]）。。
pub fn next_run_at(
    schedule: &Schedule,
    now: DateTime<FixedOffset>,
    today: &str,
) -> Option<i64> {
    let mut earliest: Option<i64> = if schedule.account_plan.date == today {
        schedule.account_plan.next_at().map(|at| at.max(now.timestamp_millis()))
    } else { None };
    for provider in &schedule.providers {
        if !crate::server::core::plugin_host::serves_provider(provider) { continue; }
        let fired = main_fired(schedule, provider, today);
        if !fired && schedule.account_plan.date == today { continue; }
        let Some(at) = next_occurrence(&effective_time(schedule, provider), fired, now) else {
            continue;
        };
        earliest = Some(earliest.map_or(at, |current: i64| current.min(at)));
    }
    if let Some(retry) = retry_next_at(schedule, now, today) {
        earliest = Some(earliest.map_or(retry, |current: i64| current.min(retry)));
    }
    earliest
}

/// down 1 when （ms）；（`retryTime` as empty） to None。
///
/// 「 has 」（ no and already ）→ ；otherwise to 。
pub fn retry_next_at(
    schedule: &Schedule,
    now: DateTime<FixedOffset>,
    today: &str,
) -> Option<i64> {
    if schedule.retry_time.is_empty() {
        return None;
    }
    // ， that 「 no 」 ； already to 
    let mut earliest: Option<i64> = None;
    for provider in &schedule.providers {
        if !crate::server::core::plugin_host::serves_provider(provider) { continue; }
        let Some(time) = provider_retry_time(schedule, provider) else {
            continue;
        };
        let Some(target) = today_at(&time, now) else {
            continue;
        };
        let pending = !fired_on(&schedule.last_retried_providers, provider, today);
        if pending && !main_fired(schedule, provider, today) && target <= now { continue; }
        let next = if pending {
            // already to no （ etc down 1 tick）→ is in 
            if target > now { target.timestamp_millis() } else { now.timestamp_millis() }
        } else {
            (target + chrono::Duration::days(1)).timestamp_millis()
        };
        earliest = Some(earliest.map_or(next, |current: i64| current.min(next)));
    }
    earliest
}

/// `HH:MM` → minutes（return None）
fn today_at_minutes(time_text: &str) -> Option<i64> {
    let normalized = normalize_time(Some(&serde_json::Value::String(time_text.to_string()))).ok()?;
    let (hour, minute) = normalized.split_once(':')?;
    Some(hour.parse::<i64>().ok()? * 60 + minute.parse::<i64>().ok()?)
}

/// minutes → `HH:MM`（ when by 24 when ）
fn format_minutes(total: i64) -> String {
    let wrapped = total.rem_euclid(24 * 60);
    format!("{:02}:{:02}", wrapped / 60, wrapped % 60)
}

/// HH:MM is already （ when by 「 no 」 —— can not ）
fn time_passed(time_text: &str, now: DateTime<FixedOffset>) -> bool {
    today_at(time_text, now).is_some_and(|target| now >= target)
}

/// when down 1 ：`fired`（ already ） or when already → 。
fn next_occurrence(
    time_text: &str,
    fired: bool,
    now: DateTime<FixedOffset>,
) -> Option<i64> {
    let target = today_at(time_text, now)?;
    let next = if fired || target <= now {
        target + chrono::Duration::days(1)
    } else {
        target
    };
    Some(next.timestamp_millis())
}

/// 1 `providerTimes`（`{providerId: "HH:MM"}`）。
///
/// already to （`checkin_providers()`，）；empty「 this 1 
/// 、 to / when 」，**** as empty not is —— will 
/// 「」「 no 」， use not 。
pub fn normalize_overrides(value: Option<&serde_json::Value>) -> BTreeMap<String, String> {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        return BTreeMap::new();
    };
    let known = crate::server::core::auto_checkin::checkin_providers();
    let mut map = BTreeMap::new();
    for (provider, raw) in object {
        if !known.iter().any(|id| id == provider) {
            continue;
        }
        let text = raw.as_str().map(str::trim).unwrap_or("");
        if text.is_empty() {
            map.insert(provider.clone(), String::new());
            continue;
        }
        if let Ok(normalized) = normalize_time(Some(raw)) {
            map.insert(provider.clone(), normalized);
        }
    }
    map
}

/// 1 `retryTime`： → ；empty → （returnempty）； → 。
pub fn normalize_retry_time(value: Option<&serde_json::Value>) -> String {
    match value {
        None => DEFAULT_RETRY_TIME.to_string(),
        Some(serde_json::Value::String(text)) if text.trim().is_empty() => String::new(),
        Some(raw) => normalize_time(Some(raw)).unwrap_or_else(|_| DEFAULT_RETRY_TIME.to_string()),
    }
}

/// when → need （ and `auto_checkin` 1 ，only use ）
pub fn now() -> DateTime<FixedOffset> {
    beijing::now()
}
