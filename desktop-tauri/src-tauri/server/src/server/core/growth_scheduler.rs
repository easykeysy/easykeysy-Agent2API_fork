//! ： when 、 use config、 and 。
//!
//! default、daily when and by `schedules.list` 。
//! 、disabled or unsupported not path； and configupdate will 。
//! `growthSchedule` is already config， use already has and when 。

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::beijing;
use crate::server::core::plugin_host::{self, schedules};
use crate::server::errors::GatewayError;
use crate::server::logging;

/// config（config.json ）。
pub const CONFIG_KEY: &str = "growthSchedule";

const PROVIDER: &str = "workbuddy";
const SCHEDULE_ID: &str = "travel";

pub use schedules::MAX_TIMES;

/// ： and `auto_checkin` 30 seconds（ to after 30 seconds）。
pub const TICK_MS: u64 = 30_000;

/// when 1 as `HH:MM`。
fn normalize_time(value: Option<&Value>) -> Option<String> {
    schedules::normalize_time(value?.as_str()?)
}

/// use ； use already default。
pub fn settings() -> (bool, Vec<String>) {
    let declared = schedules::definition(PROVIDER, SCHEDULE_ID);
    let raw = config::current()
        .raw()
        .get(CONFIG_KEY)
        .cloned()
        .unwrap_or(Value::Null);
    let enabled = raw
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            declared
                .as_ref()
                .is_some_and(|definition| definition.enabled)
        });
    let mut times: Vec<String> = raw
        .get("times")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| normalize_time(Some(item)))
                .collect()
        })
        .unwrap_or_default();
    if times.is_empty() {
        times = declared
            .map(|definition| definition.times)
            .unwrap_or_default();
    }
    times.sort();
    times.dedup();
    (enabled, times)
}

/// config（`{enabled?, times?}`， ）。
///
/// and `auto_checkin` `write_state` ： and ，config.json its 
/// 。
pub fn configure(patch: &Value) -> Result<Value, String> {
    let (_, previous_times) = settings();
    let mut current = config::current()
        .raw()
        .get(CONFIG_KEY)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Some(value) = patch.get("enabled") {
        let Some(enabled) = value.as_bool() else {
            return Err("enabled 必须是布尔值".to_string());
        };
        current.insert("enabled".to_string(), Value::Bool(enabled));
    }
    if let Some(items) = patch.get("times") {
        let Some(items) = items.as_array() else {
            return Err("times 必须是时刻数组".to_string());
        };
        if items.len() > MAX_TIMES {
            return Err(format!("时刻最多 {MAX_TIMES} 条"));
        }
        let mut times: Vec<String> = Vec::new();
        for item in items {
            let Some(time) = normalize_time(Some(item)) else {
                return Err(format!(
                    "非法时刻「{}」（应为 HH:MM，00:00 - 23:59）",
                    item.as_str().unwrap_or("?")
                ));
            };
            if !times.contains(&time) {
                times.push(time);
            }
        }
        if times.is_empty() {
            return Err("至少保留一个时刻".to_string());
        }
        times.sort();
        if times != previous_times {
            current.insert("lastFiredDate".to_string(), Value::Null);
            current.insert("lastFiredTime".to_string(), Value::Null);
            current.remove("firedTimes");
        }
        current.insert(
            "times".to_string(),
            Value::Array(times.into_iter().map(Value::String).collect()),
        );
    }
    config::update_raw_field(CONFIG_KEY, Value::Object(current));
    schedules::wake();
    Ok(state())
}

/// for （set / to ）。
pub fn state() -> Value {
    let (enabled, times) = settings();
    let raw = config::current()
        .raw()
        .get(CONFIG_KEY)
        .cloned()
        .unwrap_or(Value::Null);
    let today = beijing::today_key();
    let last_fired_date = raw
        .get("lastFiredDate")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    // already ** when **（ already ）。
    // `firedTimes`： that is by when （ `run_once` ），
    // not to `lastFiredTime`（ when ， and when not is 1 ）。
    let fired_today: Vec<String> = if last_fired_date == today {
        raw.get("firedTimes")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let pause_reason = schedules::pause_reason(PROVIDER, SCHEDULE_ID);
    json!({
        "enabled": enabled,
        "available": pause_reason.is_none(),
        "pausedReason": pause_reason,
        "times": times,
        "firedToday": fired_today,
        "lastFiredDate": if last_fired_date.is_empty() { Value::Null } else { Value::String(last_fired_date) },
        "lastFiredTime": raw.get("lastFiredTime").cloned().unwrap_or(Value::Null),
        "lastResult": raw.get("lastResult").cloned().unwrap_or(Value::Null),
        "running": is_running(),
    })
}

// ─── ───────────────────────────────────────────────

/// is has 1 in （， and auto_checkin running ）。
fn running_flag() -> &'static AtomicBool {
    static RUNNING: AtomicBool = AtomicBool::new(false);
    &RUNNING
}

pub fn is_running() -> bool {
    running_flag().load(Ordering::SeqCst)
}

/// 「 in 」 ：Drop when （ for Go `defer`）。
struct RunGuard;

impl Drop for RunGuard {
    fn drop(&mut self) {
        running_flag().store(false, Ordering::SeqCst);
    }
}

fn acquire() -> Option<RunGuard> {
    if running_flag().swap(true, Ordering::SeqCst) {
        return None;
    }
    Some(RunGuard)
}

// ─── ───────────────────────────────────────────────────

/// in 、 can unavailable and failure not can and as 1 empty。
pub enum RunOutcome {
    Completed(Value),
    Skipped(String),
    Busy,
    Failed(GatewayError),
}

/// 1 ； by when ， not use when 。
pub async fn run_once(store: &AccountStore, reason: &str, plan_time: Option<&str>) -> RunOutcome {
    if let Some(message) = schedules::pause_reason(PROVIDER, SCHEDULE_ID) {
        return RunOutcome::Skipped(message);
    }
    let Some(definition) = schedules::definition(PROVIDER, SCHEDULE_ID) else {
        return RunOutcome::Skipped("当前提供商插件未申报这项排程，请更新提供商插件".to_string());
    };
    let Some(adapter) = plugin_host::lookup(PROVIDER) else {
        return RunOutcome::Skipped("未安装或未启用提供商插件".to_string());
    };
    let Some(_guard) = acquire() else {
        return RunOutcome::Busy;
    };
    logging::log("[Travel]", &format!("⏰ 猫猫旅行巡检开始（{reason}）"));
    let summary = match adapter
        .admin_action(store, "", &definition.action, &Value::Null)
        .await
    {
        Ok(summary) => summary,
        Err(error) if error.code.as_deref() == Some("plugin_updating") => {
            return RunOutcome::Skipped("提供商插件正在重新加载，本轮已跳过".to_string());
        }
        Err(error) if error.status_code == 501 => {
            let message = "当前提供商插件未实现这项排程，请更新提供商插件".to_string();
            schedules::suspend(PROVIDER, SCHEDULE_ID, message.clone());
            return RunOutcome::Skipped(message);
        }
        Err(error) => {
            // not failure； down when or can 。
            record_run(
                json!({
                    "at": logging::now_ms(),
                    "outcome": "failed",
                    "error": error.message,
                    "accounts": 0,
                    "actions": [],
                    "credit": 0,
                    "failed": 1,
                }),
                plan_time,
            );
            logging::log_with_level(
                "[Travel]",
                &format!("猫猫旅行巡检失败：{}", error.message),
                "error",
            );
            return RunOutcome::Failed(error);
        }
    };
    record_run(summary.clone(), plan_time);
    let action_count = summary
        .get("actions")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    let credit = summary.get("credit").and_then(Value::as_i64).unwrap_or(0);
    logging::log(
        "[Travel]",
        &format!("猫猫旅行巡检完成：{action_count} 个动作（+{credit}c）"),
    );
    RunOutcome::Completed(summary)
}

fn record_run(summary: Value, plan_time: Option<&str>) {
    // lastResult + 「 already when 」
    let today = beijing::today_key();
    let now_time = beijing::now().format("%H:%M").to_string();
    let raw = config::current()
        .raw()
        .get(CONFIG_KEY)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut fired: Vec<String> = raw
        .get("firedTimes")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    // then （firedTimes in 1 valid）
    if raw.get("lastFiredDate").and_then(Value::as_str) != Some(today.as_str()) {
        fired.clear();
    }
    // by ** when **（）； no has when ，
    // not —— not etc when already 。
    if let Some(plan_time) = plan_time.filter(|text| !text.is_empty()) {
        if !fired.iter().any(|item| item == plan_time) {
            fired.push(plan_time.to_string());
        }
    }
    let mut next = raw;
    next.insert("lastFiredDate".to_string(), Value::String(today));
    next.insert("lastFiredTime".to_string(), Value::String(now_time));
    next.insert(
        "firedTimes".to_string(),
        Value::Array(fired.into_iter().map(Value::String).collect()),
    );
    next.insert("lastResult".to_string(), summary);
    config::update_raw_field(CONFIG_KEY, Value::Object(next));
}

/// this when has no → Some( that 1 )，otherwise None。
fn due_time(now: chrono::DateTime<chrono::FixedOffset>) -> Option<String> {
    let (enabled, times) = settings();
    if !enabled || schedules::pause_reason(PROVIDER, SCHEDULE_ID).is_some() {
        return None;
    }
    let today = beijing::today_key();
    let raw = config::current()
        .raw()
        .get(CONFIG_KEY)
        .cloned()
        .unwrap_or(Value::Null);
    let fired: Vec<String> =
        if raw.get("lastFiredDate").and_then(Value::as_str) == Some(today.as_str()) {
            raw.get("firedTimes")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
    let now_text = now.format("%H:%M").to_string();
    // 「 already to and no 」 1 ： when when when 
    //（ 1 1 ， down tick down 1 —— and auto_checkin ）
    times
        .into_iter()
        .find(|time| time.as_str() <= now_text.as_str() && !fired.iter().any(|item| item == time))
}

/// （ by `ServerState::bootstrap` 1 ）。
///
/// `TICK_MS` 1 new ：set down 1 、 after ——
/// and `auto_checkin` / `scheduled_tasks` 1 （ that module header）。
pub fn spawn(store: AccountStore) {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    crate::spawn_task(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(TICK_MS)) => {},
                _ = schedules::changed() => {},
            }
            let now = beijing::now();
            let Some(plan_time) = due_time(now) else {
                continue;
            };
            let _ = run_once(
                &store,
                &format!("到点触发（{plan_time}）"),
                Some(&plan_time),
            )
            .await;
        }
    });
}

// ─── （ API ）─────────────────────────────

/// not use when 。
pub async fn run_now(store: &AccountStore) -> RunOutcome {
    run_once(store, "手动执行", None).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_time_accepts_and_rejects() {
        assert_eq!(
            normalize_time(Some(&json!("09:00"))),
            Some("09:00".to_string())
        );
        assert_eq!(
            normalize_time(Some(&json!("9:5"))),
            Some("09:05".to_string())
        );
        assert_eq!(
            normalize_time(Some(&json!(" 21:30 "))),
            Some("21:30".to_string())
        );
        // / / 
        assert_eq!(normalize_time(Some(&json!("24:00"))), None);
        assert_eq!(normalize_time(Some(&json!("12:60"))), None);
        assert_eq!(normalize_time(Some(&json!(930))), None);
        assert_eq!(normalize_time(None), None);
    }

    /// must by ** when ** for 。
    ///
    /// this is 1 bug：`run_once` by when （`now_time`）
    /// ， `due_time` when —— to after 30 seconds，
    /// not etc ， is tick 1 。
    #[test]
    fn due_time_matches_plan_time_not_actual_time() {
        let times = ["09:00".to_string(), "21:00".to_string()];
        let fired = vec!["09:00".to_string()];
        let now_text = "09:00".to_string();
        // 09:00 already → 21:00 no to → none can 
        let due = times.iter().find(|time| {
            time.as_str() <= now_text.as_str() && !fired.iter().any(|item| *item == **time)
        });
        assert_eq!(due, None);
        // if fired when （09:01）， will 「 no 」→ 
        let wrong_fired = vec!["09:01".to_string()];
        let due_wrong = times.iter().find(|time| {
            time.as_str() <= now_text.as_str() && !wrong_fired.iter().any(|item| *item == **time)
        });
        assert_eq!(due_wrong.map(|time| time.as_str()), Some("09:00"));
    }

    #[test]
    fn due_time_picks_earliest_unfired_past_slot() {
        let times = [
            "09:00".to_string(),
            "13:00".to_string(),
            "21:00".to_string(),
        ];
        // in 22:00、 1 no → 09:00（ 1 1 ）
        let now_text = "22:00".to_string();
        let fired: Vec<String> = Vec::new();
        let due = times.iter().find(|time| {
            time.as_str() <= now_text.as_str() && !fired.iter().any(|item| *item == **time)
        });
        assert_eq!(due.map(|time| time.as_str()), Some("09:00"));
        // 09:00 after → 13:00
        let fired = vec!["09:00".to_string()];
        let due = times.iter().find(|time| {
            time.as_str() <= now_text.as_str() && !fired.iter().any(|item| *item == **time)
        });
        assert_eq!(due.map(|time| time.as_str()), Some("13:00"));
    }
}
