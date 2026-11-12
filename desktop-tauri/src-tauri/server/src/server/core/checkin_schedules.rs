//! when to ****config（`autoCheckin.schedules` ）。
//!
//! ── ────────────────────────────────────────────────────
//! is 1 「 to 」 config：
//!   `{id, label, enabled, time, timeEnd, accountPlan, providers, providerTimes, retryTime,
//!     retryGapMinutes, retryLimit, lastFiredDate, lastFiredProviders,
//!     lastRetriedProviders, lastResult}`
//! - `id` （ by upsert / delete / ； is "default"）
//! - `label` （ can empty；empty when to `time`）
//! - `enabled` 
//! - `time` / `timeEnd` HH:MM， when 1 ； when 。
//! - `accountPlan` when and ， after use 。
//! - `providers` this need （`CHECKIN_PROVIDERS` ， 1 ）
//! - `providerTimes` （`{providerId: "HH:MM"}`；empty = ）。
//! use default， no or `time`。
//! - `retryTime` not when （ 00:10；empty = ）； not 。
//! - `retryGapMinutes` in after （ 10 minutes；0 = not ）
//! - `retryLimit` （ 3；0 = not ）
//!   - `lastFiredDate` / `lastFiredProviders` / `lastRetriedProviders` / `lastResult`
//! and up 1 ： before （ 1 1 1 ），
//! `lastFiredDate` is before （， `times::main_fired`）
//! total use `autoCheckin.enabled`（ `core::auto_checkin`）：all not 。
//!
//! ── ： when ， not file ─────────────────────────
//! `schedules` **** when （config）， use old （`time` / `providers` /
//! `lastFiredDate` / `lastResult`） 1 `id="default"` ：config
//! ， 。** in **（ `[]`） as ，
//! `[]` 「 use 」—— this is delete ：delete `[]`， not 。
//! 1 in ， old not （ not ， as ）。
//!
//! ── Why core ──────────────────────────────────────────
//! （`auto_checkin::tick` / `start`） and （`fire` ） in 
//! core；`core::auto_checkin` config already is （ module header「config」），
//! use 1 ：`update_raw_field` and ，config.json its 。

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::auto_checkin;
use crate::server::core::auto_checkin::times;

pub mod account_plan;

/// limit：， to not need need 
pub const MAX_SCHEDULES: usize = 10;
/// default id（config no has `schedules` when that ）
pub const DEFAULT_SCHEDULE_ID: &str = "default";
/// in limit（minutes）： down 1 ， no has 
pub const MAX_RETRY_GAP_MINUTES: i64 = 120;
/// limit： 1 + this ， before after 
pub const MAX_RETRY_LIMIT: i64 = 20;

/// 1 to （； ， and auto_checkin 1 ）
#[derive(Clone, Debug)]
pub struct Schedule {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub time: String,
    pub time_end: String,
    pub providers: Vec<String>,
    /// when （empty = ， to / when ）
    pub provider_times: BTreeMap<String, String>,
    /// when （empty = ）
    pub retry_time: String,
    /// in （minutes；0 = not ）
    pub retry_gap_minutes: i64,
    /// limit（0 = not ）
    pub retry_limit: i64,
    pub last_fired_date: Option<String>,
    /// （`{providerId: "YYYY-MM-DD"}`）
    pub last_fired_providers: BTreeMap<String, String>,
    /// 
    pub last_retried_providers: BTreeMap<String, String>,
    pub account_plan: account_plan::AccountPlan,
    pub last_result: Option<Value>,
}

/// ： use ， no to when （ when is 1 ， 1 can ）
pub fn display_label(schedule: &Schedule) -> &str {
    if schedule.label.is_empty() {
        &schedule.time
    } else {
        &schedule.label
    }
}

// ─── ───────────────────────────────────────────────────

/// is in （ `[]`）： in as ， not from old 
fn stored_key_exists() -> bool {
    match config::current().raw().get(auto_checkin::CONFIG_KEY) {
        Some(Value::Object(map)) => map.contains_key("schedules"),
        _ => false,
    }
}

/// （； not in to None）
fn stored_entries() -> Option<Vec<Value>> {
    auto_checkin::raw_object()
        .get("schedules")
        .and_then(Value::as_array)
        .cloned()
}

/// 1 ：id / label / enabled / time / providers 。
/// time DEFAULT_TIME（ not when ， and auto_checkin ）。
fn normalize_schedule(raw: &Value) -> Schedule {
    let id = raw
        .get("id")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or(DEFAULT_SCHEDULE_ID)
        .to_string();
    let label = raw
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let enabled = raw.get("enabled").and_then(Value::as_bool) == Some(true);
    let time = raw
        .get("time")
        .filter(|value| !value.is_null())
        .and_then(|value| auto_checkin::normalize_time(Some(value)).ok())
        .unwrap_or_else(auto_checkin::default_time);
    let time_end = raw.get("timeEnd")
        .and_then(|value| auto_checkin::normalize_time(Some(value)).ok())
        .filter(|end| times::valid_range(&time, end))
        .unwrap_or_else(|| times::default_time_end(&time));
    let providers = auto_checkin::normalize_providers(raw.get("providers"));
    Schedule {
        id,
        label,
        enabled,
        time,
        time_end,
        providers,
        provider_times: times::normalize_overrides(raw.get("providerTimes")),
        retry_time: times::normalize_retry_time(raw.get("retryTime")),
        retry_gap_minutes: retry_number(raw, "retryGapMinutes", 10, MAX_RETRY_GAP_MINUTES),
        retry_limit: retry_number(raw, "retryLimit", 3, MAX_RETRY_LIMIT),
        last_fired_date: raw
            .get("lastFiredDate")
            .and_then(Value::as_str)
            .map(str::to_string),
        last_fired_providers: date_map(raw.get("lastFiredProviders")),
        last_retried_providers: date_map(raw.get("lastRetriedProviders")),
        account_plan: account_plan::AccountPlan::from_value(raw.get("accountPlan")),
        last_result: raw.get("lastResult").filter(|value| value.is_object()).cloned(),
    }
}

/// ： / → ； to 0（0 = ）。
fn retry_number(raw: &Value, key: &str, default: i64, max: i64) -> i64 {
    raw.get(key)
        .and_then(Value::as_i64)
        .unwrap_or(default)
        .clamp(0, max)
}

/// `{providerId: "YYYY-MM-DD"}` 1 （empty）
fn date_map(value: Option<&Value>) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let Some(object) = value.and_then(Value::as_object) else {
        return map;
    };
    for (key, raw) in object {
        if let Some(text) = raw.as_str().filter(|text| !text.is_empty()) {
            map.insert(key.clone(), text.to_string());
        }
    }
    map
}

/// ： use old that default（`schedules` when ）
fn legacy_schedule() -> Schedule {
    legacy_schedule_of(&auto_checkin::raw_object())
}

fn legacy_schedule_of(raw: &Map<String, Value>) -> Schedule {
    normalize_schedule(&json!({
        "id": DEFAULT_SCHEDULE_ID,
        "enabled": raw.get("enabled").cloned().unwrap_or(Value::Null),
        "time": raw.get("time").cloned().unwrap_or(Value::Null),
        "timeEnd": raw.get("timeEnd").cloned().unwrap_or(Value::Null),
        "accountPlan": raw.get("accountPlan").cloned().unwrap_or(Value::Null),
        "providers": raw.get("providers").cloned().unwrap_or(Value::Null),
        "providerTimes": raw.get("providerTimes").cloned().unwrap_or(Value::Null),
        "retryTime": raw.get("retryTime").cloned().unwrap_or(Value::Null),
        "retryGapMinutes": raw.get("retryGapMinutes").cloned().unwrap_or(Value::Null),
        "retryLimit": raw.get("retryLimit").cloned().unwrap_or(Value::Null),
        "lastFiredDate": raw.get("lastFiredDate").cloned().unwrap_or(Value::Null),
        "lastFiredProviders": raw.get("lastFiredProviders").cloned().unwrap_or(Value::Null),
        "lastRetriedProviders": raw.get("lastRetriedProviders").cloned().unwrap_or(Value::Null),
        "lastResult": raw.get("lastResult").cloned().unwrap_or(Value::Null),
    }))
}

/// （ and ）。
/// `schedules` → old default； in （ `[]`）→ as 。
pub fn list() -> Vec<Schedule> {
    match stored_entries() {
        Some(items) if !items.is_empty() => items.iter().map(normalize_schedule).collect(),
        Some(_) => Vec::new(),
        None => vec![legacy_schedule()],
    }
}

// ─── ───────────────────────────────────────────────────

/// `autoCheckin.schedules`（ its ）
fn write_entries(entries: Vec<Value>) {
    auto_checkin::write_state(json!({ "schedules": entries }));
}

/// → （ lastFiredDate / lastResult， ）
fn to_value(schedule: &Schedule) -> Value {
    json!({
        "id": schedule.id,
        "label": schedule.label,
        "enabled": schedule.enabled,
        "time": schedule.time,
        "timeEnd": schedule.time_end,
        "accountPlan": schedule.account_plan,
        "providers": schedule.providers,
        "providerTimes": schedule.provider_times,
        "retryTime": schedule.retry_time,
        "retryGapMinutes": schedule.retry_gap_minutes,
        "retryLimit": schedule.retry_limit,
        "lastFiredDate": schedule.last_fired_date,
        "lastFiredProviders": schedule.last_fired_providers,
        "lastRetriedProviders": schedule.last_retried_providers,
        "lastResult": schedule.last_result,
    })
}

// ─── CRUD（ ）───────────────────────────────

/// add / update 1 （ by id upsert）。： when and 、 1 、
/// limit。return after that （ after 1 ， after return as ）。
///
/// `enabled` not is 「」 is 「」：（`auto_checkin::configure`
/// when /） not this disabled ； new no has can ， as 。
pub fn upsert(payload: &Value) -> Result<Schedule, String> {
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("schedule-{}", crate::server::logging::now_ms()));
    let time = auto_checkin::normalize_time(payload.get("time"))
        .map_err(|error| error.message)?;
    // ： （、 by ）； 1 not to none can 
    let list_value = payload.get("providers");
    let picked: Vec<String> = auto_checkin::checkin_providers()
        .into_iter()
        .filter(|id| {
            list_value
                .and_then(Value::as_array)
                .map(|items| items.iter().any(|item| item.as_str() == Some(id.as_str())))
                .unwrap_or(false)
        })
        .collect();
    if picked.is_empty() {
        return Err("至少勾选一家签到提供商".to_string());
    }
    let label = payload
        .get("label")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let entries = stored_entries().unwrap_or_default();
    let existing = entries.iter().position(|raw| {
        raw.get("id").and_then(Value::as_str) == Some(id.as_str())
    });
    if existing.is_none() && entries.len() >= MAX_SCHEDULES {
        return Err(format!("定时条目最多 {MAX_SCHEDULES} 条"));
    }
    let existing_entry = existing.map(|index| normalize_schedule(&entries[index]))
        .or_else(|| (!stored_key_exists() && id == DEFAULT_SCHEDULE_ID).then(legacy_schedule));
    let enabled = match payload.get("enabled").and_then(Value::as_bool) {
        Some(flag) => flag,
        // ：update； new default —— 1 **** when ，
        // 「」 in old （ default is ）， 1 and 
        None => match existing_entry.as_ref().map(|old| old.enabled) {
            Some(flag) => flag,
            None if !stored_key_exists() && id == DEFAULT_SCHEDULE_ID => {
                legacy_schedule().enabled
            }
            None => true,
        },
    };
    // update already has when ； new from 「 no 」
    let (last_fired_date, last_result) = existing_entry
        .as_ref()
        .map(|old| (old.last_fired_date.clone(), old.last_result.clone()))
        .unwrap_or((None, None));
    // ： 1 default when ， old 
    let (last_fired_date, last_result) = if last_fired_date.is_none() && !stored_key_exists() && id == DEFAULT_SCHEDULE_ID {
        let legacy = legacy_schedule();
        (legacy.last_fired_date, legacy.last_result)
    } else {
        (last_fired_date, last_result)
    };
    // need 「」： when not this already 1 
    let (last_fired_providers, last_retried_providers) = existing_entry
        .as_ref()
        .map(|old| (old.last_fired_providers.clone(), old.last_retried_providers.clone()))
        .unwrap_or_default();
    // ── when / ──────────────────────────────
    // ** payload to **：（ and `configure` that 
    // ） time/providers， not this 。
    let provider_times = match payload.get("providerTimes") {
        Some(value) => times::normalize_overrides(Some(value)),
        None => existing_entry
            .as_ref()
            .map(|old| old.provider_times.clone())
            .unwrap_or_default(),
    };
    let retry_time = match payload.get("retryTime") {
        Some(value) => times::normalize_retry_time(Some(value)),
        None => existing_entry
            .as_ref()
            .map(|old| old.retry_time.clone())
            .unwrap_or_else(|| times::DEFAULT_RETRY_TIME.to_string()),
    };
    let retry_gap_minutes = payload
        .get("retryGapMinutes")
        .and_then(Value::as_i64)
        .map(|value| value.clamp(0, MAX_RETRY_GAP_MINUTES))
        .or_else(|| existing_entry.as_ref().map(|old| old.retry_gap_minutes))
        .unwrap_or(10);
    let retry_limit = payload
        .get("retryLimit")
        .and_then(Value::as_i64)
        .map(|value| value.clamp(0, MAX_RETRY_LIMIT))
        .or_else(|| existing_entry.as_ref().map(|old| old.retry_limit))
        .unwrap_or(3);
    let time_end = match payload.get("timeEnd") {
        Some(value) => auto_checkin::normalize_time(Some(value)).map_err(|error| error.message)?,
        None => existing_entry.as_ref().map(|old| {
            times::end_with_spread(&time, times::spread_minutes(old))
        }).unwrap_or_else(|| times::default_time_end(&time)),
    };
    if !times::valid_range(&time, &time_end) {
        return Err("签到区间结束时间不能早于开始时间；暂不支持跨天区间".to_string());
    }
    let schedule = Schedule {
        id,
        label,
        enabled,
        time,
        time_end,
        providers: picked,
        provider_times,
        retry_time,
        retry_gap_minutes,
        retry_limit,
        last_fired_date,
        last_fired_providers,
        last_retried_providers,
        account_plan: existing_entry.as_ref().map(|old| old.account_plan.clone()).unwrap_or_default(),
        last_result,
    };
    let mut outcome = Err("定时条目保存失败".to_string());
    auto_checkin::update_state(|state| {
        let mut entries = state.get("schedules").and_then(Value::as_array).cloned().unwrap_or_default();
        let index = entries.iter().position(|raw| raw.get("id").and_then(Value::as_str) == Some(schedule.id.as_str()));
        if index.is_none() && entries.len() >= MAX_SCHEDULES {
            outcome = Err(format!("定时条目最多 {MAX_SCHEDULES} 条"));
            return;
        }
        let current = index.map(|index| normalize_schedule(&entries[index]))
            .or_else(|| (!state.contains_key("schedules") && schedule.id == DEFAULT_SCHEDULE_ID).then(|| legacy_schedule_of(state)));
        let mut schedule = schedule;
        if let Some(current) = current {
            if payload.get("enabled").is_none() { schedule.enabled = current.enabled; }
            if payload.get("providerTimes").is_none() { schedule.provider_times = current.provider_times; }
            if payload.get("retryTime").is_none() { schedule.retry_time = current.retry_time; }
            if payload.get("retryGapMinutes").is_none() { schedule.retry_gap_minutes = current.retry_gap_minutes; }
            if payload.get("retryLimit").is_none() { schedule.retry_limit = current.retry_limit; }
            schedule.last_fired_date = current.last_fired_date;
            schedule.last_fired_providers = current.last_fired_providers;
            schedule.last_retried_providers = current.last_retried_providers;
            schedule.account_plan = current.account_plan;
            schedule.last_result = current.last_result;
        }
        match index {
            Some(index) => entries[index] = to_value(&schedule),
            None => entries.push(to_value(&schedule)),
        }
        state.insert("schedules".to_string(), Value::Array(entries));
        outcome = Ok(schedule);
    });
    outcome
}

/// delete 1 。delete（ or after 1 ） `[]`： in ，
/// 「 from old 」 not —— is ， not this 。
pub fn remove(id: &str) -> Result<(), String> {
    let mut outcome = Err(format!("定时条目不存在：{id}"));
    auto_checkin::update_state(|state| {
        let mut entries = state.get("schedules").and_then(Value::as_array).cloned().unwrap_or_default();
        let before = entries.len();
        entries.retain(|raw| raw.get("id").and_then(Value::as_str) != Some(id));
        if entries.len() < before || (!state.contains_key("schedules") && id == DEFAULT_SCHEDULE_ID) {
            state.insert("schedules".to_string(), Value::Array(entries));
            outcome = Ok(());
        }
    });
    outcome
}

/// ：`fire` in before （， not will ）。
/// `date` to None = （Qoder 、 etc ）。
///
/// ：`schedules` not in when is ** old **（`autoCheckin.lastFiredDate`）
/// —— old not ， and before 1 。
pub fn mark_fired(id: &str, date: Option<&str>) {
    patch_entry(id, |schedule| {
        schedule.last_fired_date = date.map(str::to_string)
    });
}

/// down 「 this 」。
///
/// and [`mark_fired`] and ： that is before （ in and old 
/// ）， is —— 1 1 1 ， not by 
/// when （ `auto_checkin::times`）。
pub fn mark_fired_providers(id: &str, providers: &[String], date: &str) {
    if providers.is_empty() {
        return;
    }
    patch_entry(id, |schedule| {
        for provider in providers {
            schedule
                .last_fired_providers
                .insert(provider.clone(), date.to_string());
        }
        schedule.last_fired_date = Some(date.to_string());
    });
}

/// down 「 this 」。
pub fn mark_retried_providers(id: &str, providers: &[String], date: &str) {
    if providers.is_empty() {
        return;
    }
    patch_entry(id, |schedule| {
        for provider in providers {
            schedule
                .last_retried_providers
                .insert(provider.clone(), date.to_string());
        }
    });
}

/// up 1 ：success / failure 1 （ and auto_checkin lastResult
/// ）；`None` = 。
pub fn mark_result(id: &str, result: Option<Value>) {
    patch_entry(id, |schedule| {
        schedule.last_result = result.filter(|value| value.is_object())
    });
}

/// 1 （ `apply` ）。
/// not in （config） and is default when to old ， not 。
fn patch_entry<F: FnOnce(&mut Schedule)>(id: &str, apply: F) {
    auto_checkin::update_state(|state| {
        if !state.contains_key("schedules") {
            if id != DEFAULT_SCHEDULE_ID { return; }
            let mut schedule = legacy_schedule_of(state);
            apply(&mut schedule);
            let value = to_value(&schedule);
            for key in ["accountPlan", "lastFiredDate", "lastFiredProviders", "lastRetriedProviders", "lastResult"] {
                state.insert(key.to_string(), value[key].clone());
            }
            return;
        }
        let mut entries = state.get("schedules").and_then(Value::as_array).cloned().unwrap_or_default();
        let Some(index) = entries.iter().position(|raw| raw.get("id").and_then(Value::as_str) == Some(id)) else { return; };
        let mut schedule = normalize_schedule(&entries[index]);
        apply(&mut schedule);
        entries[index] = to_value(&schedule);
        state.insert("schedules".to_string(), Value::Array(entries));
    });
}

/* ─── 1 ： new default in ───────────────────
 *
 * and 「Why not can 」 in `auto_checkin::PROVIDERS_VERSION` up ；
 * this ：
 * · by `CHECKIN_PROVIDERS` 1 （ and upsert 1 ），
 * that in 、 up and not 1 ；
 * · config（ no ）** not **， `providers` ——
 * 「 when 、 not file」 is `checkin_schedules` has ， not 。 */

/// config 「 to already to 」（ = 1）
fn providers_version() -> i64 {
    auto_checkin::raw_object()
        .get("providersVersion")
        .and_then(Value::as_i64)
        .unwrap_or(1)
}

/// by 1 （ 、）
fn ordered_providers(picked: &[String]) -> Vec<String> {
    auto_checkin::checkin_providers()
        .into_iter()
        .filter(|id| picked.iter().any(|item| item == id))
        .collect()
}

/// 「config this 1 」 1 ， after down 。
/// return is （ use 1 ）。 etc ： to after return。
pub fn extend_providers_once() -> bool {
    let version = providers_version();
    if version >= auto_checkin::PROVIDERS_VERSION {
        return false;
    }
    let added = auto_checkin::providers_added_after(version);
    let mut changed = false;
    if added.is_empty() {
        // （update ）：， not 
        record_providers_version();
        return false;
    }
    match stored_entries() {
        Some(entries) => {
            let mut next = entries;
            for raw in next.iter_mut() {
                let mut schedule = normalize_schedule(raw);
                let mut providers = schedule.providers.clone();
                for id in &added {
                    if !providers.iter().any(|item| item == id) {
                        providers.push(id.clone());
                    }
                }
                if providers.len() != schedule.providers.len() {
                    schedule.providers = ordered_providers(&providers);
                    *raw = to_value(&schedule);
                    changed = true;
                }
            }
            if changed {
                write_entries(next);
            }
        }
        None => {
            let legacy = ordered_providers(
                &auto_checkin::normalize_providers(auto_checkin::raw_object().get("providers")),
            );
            let mut providers = legacy.clone();
            for id in &added {
                if !providers.iter().any(|item| item == id) {
                    providers.push(id.clone());
                }
            }
            if providers.len() != legacy.len() {
                auto_checkin::write_state(json!({ "providers": ordered_providers(&providers) }));
                changed = true;
            }
        }
    }
    record_providers_version();
    changed
}

/// down 「 to already to current」（ not its ）
fn record_providers_version() {
    auto_checkin::write_state(json!({ "providersVersion": auto_checkin::PROVIDERS_VERSION }));
}

/// daily to when only 1 ； use after not can new 。
pub fn extend_plugin_providers_once() -> bool {
    let available: Vec<String> = auto_checkin::checkin_providers()
        .into_iter()
        .filter(|id| !auto_checkin::CHECKIN_PROVIDERS.contains(&id.as_str()))
        .collect();
    let known: Vec<String> = auto_checkin::raw_object()
        .get("pluginProvidersSeen")
        .and_then(Value::as_array)
        .into_iter().flatten().filter_map(Value::as_str).map(str::to_string).collect();
    let added: Vec<String> = available.into_iter().filter(|id| !known.contains(id)).collect();
    if added.is_empty() { return false; }
    let mut changed = false;
    match stored_entries() {
        Some(mut entries) => {
            for raw in &mut entries {
                let mut schedule = normalize_schedule(raw);
                let mut row_changed = false;
                for id in &added {
                    if !schedule.providers.contains(id) {
                        schedule.providers.push(id.clone());
                        row_changed = true;
                    }
                }
                if row_changed {
                    if let Some(object) = raw.as_object_mut() {
                        object.insert("providers".into(), json!(schedule.providers));
                    }
                    changed = true;
                }
            }
            if changed { write_entries(entries); }
        }
        None => {
            let mut providers = legacy_schedule().providers;
            for id in &added {
                if !providers.contains(id) { providers.push(id.clone()); changed = true; }
            }
            if changed { auto_checkin::write_state(json!({ "providers": providers })); }
        }
    }
    let mut seen = known;
    seen.extend(added);
    auto_checkin::write_state(json!({ "pluginProvidersSeen": seen }));
    changed
}
