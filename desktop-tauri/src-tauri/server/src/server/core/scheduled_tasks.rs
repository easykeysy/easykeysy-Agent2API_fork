//! when and 。
//!
//! ── （ is ****， not is can ）─────────────────
//! - `Runner::Backend`：、directory new 、check。
//! after ， has 「 up / down / 」 this 。
//! - `Runner::Frontend`： / request / new 。 when in 
//! up （ in can when this ）， after and ，。
//!
//! ── 「 when 」， not ─────────────────────────────
//! in （`credential_maintenance` /
//! `providers::catalog_refresh` / `update::check`），file by config 
//! and —— and `auto_checkin` 1 。
//!
//! ── Why not in this ─────────────────────────────
//! already **config** （ and 「 not 」 in 
//! set）， and in `core::usage_query` / `core::usage_records`：
//! 「」 and this 「 1 」 not is 1 ，
//! `{enabled, interval}` will （ and to not in by ）。
//!
//! ── Why not in this ─────────────────────────────────
//! 「 up / up success / down / failure / in 」 to 
//! `core::task_state`（、、）。 tick 
//! from ****「 to 」， not is in `nextRunAt` —— before 
//! is after ， is to 「 in 」， up 。
//!
//! ── ：， not is 「」────────────────────────
//! and `auto_checkin` 1 ： `TICK_MS` 1 ， new 。 after 
//! is need ：
//! - **set down 1 **（ 1 tick）： in from 
//! 60 minutes to 1 minutes when ， need etc 60 minutes —— 「 already 」
//! no has ， is that not 1 ；
//! - ** / / when after **： when in not ，
//! 1 tick 1 down up when when up （ 1 ，
//! not 1 ）。
//!
//! ── to Why not in this ─────────────────────────────
//! is ****（ can 00:01 this when ）， and 「 etc 」 not is 
//! 1 ： when （JSON ）、（`lastFiredDate`）、 is 
//! has 。 `{enabled, interval}` will 。
//! up in 1 ， is `/api/auto-checkin`。

use std::time::Duration;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::config::IntervalTaskPatch;
use crate::server::core::account_store::AccountStore;
use crate::server::core::task_state::{self, Claim, TaskState};
use crate::server::core::update::UpdateManager;
use crate::server::logging;

/// 。
///
/// 10 seconds：backend down is 1 minutes（ `INTERVAL_MIN_MINUTES`），
/// 10 seconds 「 to after 10 seconds」—— use not to ，
/// empty is 10 seconds 1 （ in ）。 not ： no has 。
pub const TICK_MS: u64 = 10_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Runner {
    Backend,
    Frontend,
}

pub struct TaskDef {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub unit: &'static str,
    pub runner: Runner,
    pub min: i64,
    pub max: i64,
    pub default_interval: i64,
}

pub const TASK_CREDENTIAL_MAINTENANCE: &str = config::KEY_CREDENTIAL_MAINTENANCE;
pub const TASK_MODEL_REFRESH: &str = config::KEY_MODEL_REFRESH;
pub const TASK_UPDATE_CHECK: &str = config::KEY_UPDATE_CHECK;
pub const TASK_LOGS_AUTO_REFRESH: &str = config::KEY_LOGS_AUTO_REFRESH;
pub const TASK_REQUESTS_AUTO_REFRESH: &str = config::KEY_REQUESTS_AUTO_REFRESH;
pub const TASK_REPORT_AUTO_REFRESH: &str = config::KEY_REPORT_AUTO_REFRESH;

pub const TASKS: [TaskDef; 6] = [
    TaskDef {
        id: TASK_CREDENTIAL_MAINTENANCE,
        label: "凭证自动维护",
        description: "定期遍历账号，只刷新已过期或临期的凭证。重启后沿用上次排期，失败账号在冷却后重试；转发需要的按需续期不受此开关影响。",
        unit: "minutes",
        runner: Runner::Backend,
        min: config::INTERVAL_MIN_MINUTES,
        max: config::INTERVAL_MAX_MINUTES,
        default_interval: config::DEFAULT_CREDENTIAL_MAINTENANCE_MINUTES,
    },
    TaskDef {
        id: TASK_MODEL_REFRESH,
        label: "模型目录刷新",
        description: "定期拉取最新模型清单，重启沿用排期与缓存。客户端读取模型列表的后台刷新也遵守本任务开关、间隔和失败冷却；手动获取模型可提前刷新。同一轮顺带刷新 models.dev 模型元数据（能力位与思考档位，可在设置页单独关闭）。",
        unit: "minutes",
        runner: Runner::Backend,
        min: config::INTERVAL_MIN_MINUTES,
        max: config::INTERVAL_MAX_MINUTES,
        default_interval: config::DEFAULT_MODEL_REFRESH_MINUTES,
    },
    TaskDef {
        id: TASK_UPDATE_CHECK,
        label: "软件版本检查",
        description: "定期向 GitHub 查询最新发布版本，默认每 20 分钟一次。检查结果、排期和限流冷却跨重启保留；手动检查也会合并重复请求并遵守冷却。开关与间隔在「软件更新」面板的「更新设置」弹窗里配置（定时任务页不再渲染这一条）。",
        unit: "minutes",
        runner: Runner::Backend,
        min: config::INTERVAL_MIN_MINUTES,
        max: config::INTERVAL_MAX_MINUTES,
        default_interval: config::DEFAULT_UPDATE_CHECK_MINUTES,
    },
    TaskDef {
        id: TASK_LOGS_AUTO_REFRESH,
        label: "日志页自动刷新",
        description: "停留在「日志」页时按此间隔重新拉取系统事件；页面不可见时不请求。",
        unit: "seconds",
        runner: Runner::Frontend,
        min: config::INTERVAL_MIN_SECONDS,
        max: config::INTERVAL_MAX_SECONDS,
        default_interval: config::DEFAULT_LOGS_AUTO_REFRESH_SECONDS,
    },
    TaskDef {
        id: TASK_REQUESTS_AUTO_REFRESH,
        label: "请求日志页自动刷新",
        description: "停留在「请求日志」页时按此间隔重新拉取请求日志；页面不可见时不请求。",
        unit: "seconds",
        runner: Runner::Frontend,
        min: config::INTERVAL_MIN_SECONDS,
        max: config::INTERVAL_MAX_SECONDS,
        default_interval: config::DEFAULT_REQUESTS_AUTO_REFRESH_SECONDS,
    },
    TaskDef {
        id: TASK_REPORT_AUTO_REFRESH,
        label: "报表自动刷新",
        description: "停留在「报表」页时按此间隔重新拉取统计数据；页面不可见时不请求。",
        unit: "seconds",
        runner: Runner::Frontend,
        min: config::INTERVAL_MIN_SECONDS,
        max: config::INTERVAL_MAX_SECONDS,
        default_interval: config::DEFAULT_REPORT_AUTO_REFRESH_SECONDS,
    },
];

pub fn find(id: &str) -> Option<&'static TaskDef> {
    TASKS.iter().find(|task| task.id == id)
}

fn settings_of(settings: config::ScheduledSettings, id: &str) -> config::IntervalTask {
    match id {
        TASK_CREDENTIAL_MAINTENANCE => settings.credential_maintenance,
        TASK_MODEL_REFRESH => settings.model_refresh,
        TASK_UPDATE_CHECK => settings.update_check,
        TASK_LOGS_AUTO_REFRESH => settings.logs_auto_refresh,
        TASK_REQUESTS_AUTO_REFRESH => settings.requests_auto_refresh,
        TASK_REPORT_AUTO_REFRESH => settings.report_auto_refresh,
        _ => config::IntervalTask { enabled: false, interval: 0 },
    }
}

fn interval_ms(task: &TaskDef) -> i64 {
    let interval = settings_of(config::scheduled_settings(), task.id).interval.max(1);
    interval * if task.unit == "seconds" { 1000 } else { 60_000 }
}

fn state_key(id: &str) -> String {
    if id == TASK_UPDATE_CHECK {
        crate::server::core::update::global_check_key()
    } else {
        id.to_string()
    }
}

fn task_json(task: &TaskDef, state: Result<TaskState, String>) -> Value {
    let settings = settings_of(config::scheduled_settings(), task.id);
    let backend = task.runner == Runner::Backend;
    let error = state.as_ref().err().cloned();
    let state = state.unwrap_or_default();
    let timestamp = |at: i64| if at > 0 { json!(at) } else { Value::Null };
    json!({
        "id": task.id,
        "label": task.label,
        "description": task.description,
        "unit": task.unit,
        "runner": if backend { "backend" } else { "frontend" },
        "enabled": settings.enabled,
        "interval": settings.interval,
        "min": task.min,
        "max": task.max,
        "defaultInterval": task.default_interval,
        "running": backend && state.running(),
        "lastRunAt": timestamp(state.last_run_at),
        "lastAttemptAt": timestamp(state.last_attempt_at),
        "lastSuccessAt": timestamp(state.last_success_at),
        "lastResult": error.or_else(|| state.last_result.clone()),
        "lastError": state.last_error,
        "retryAt": timestamp(state.retry_at),
        "nextRunAt": if backend && settings.enabled { timestamp(state.due_at().max(logging::now_ms())) } else { Value::Null },
        "canRun": backend,
    })
}

pub fn list() -> Value {
    // 1 ： 「 when 」 20 seconds 1 ，
    // 1 （ + ） no has 。
    let states = task_state::read_all();
    json!({
        "tasks": TASKS
            .iter()
            .map(|task| {
                let state = match &states {
                    Ok(states) if task.runner == Runner::Backend => {
                        Ok(states.get(&state_key(task.id)).cloned().unwrap_or_default())
                    }
                    Ok(_) => Ok(TaskState::default()),
                    Err(error) => Err(error.clone()),
                };
                task_json(task, state)
            })
            .collect::<Vec<_>>(),
    })
}

pub fn task_by_id(id: &str) -> Value {
    find(id)
        .map(|task| {
            let state = if task.runner == Runner::Backend {
                task_state::read(&state_key(task.id))
            } else {
                Ok(TaskState::default())
            };
            task_json(task, state)
        })
        .unwrap_or(Value::Null)
}

pub fn configure(id: &str, patch: IntervalTaskPatch) -> Result<Value, String> {
    let task = find(id).ok_or_else(|| format!("未知的定时任务: {id}"))?;
    if patch.enabled.is_none() && patch.interval.is_none() {
        return Err("没有需要更新的字段".to_string());
    }
    if let Some(interval) = patch.interval {
        if !(task.min..=task.max).contains(&interval) {
            let unit = if task.unit == "seconds" { "秒" } else { "分钟" };
            return Err(format!("「{}」的间隔必须是 {}–{} {}", task.label, task.min, task.max, unit));
        }
    }
    let before = settings_of(config::scheduled_settings(), id);
    let plugins_before = (id == TASK_UPDATE_CHECK).then(crate::server::core::plugin_host::settings::current);
    let saved = config::set_scheduled_task(id, patch, task.min, task.max);
    let after = settings_of(config::scheduled_settings(), id);
    if let Some(before) = plugins_before {
        crate::server::core::plugin_host::settings::notify_changed(before);
    }
    if task.runner == Runner::Backend {
        if !before.enabled && after.enabled {
            // ： before 「 1 」 （ down 1 tick ），
            // not is by up when old etc —— that is use when 
            // 1 。failure（`due_at` ）。
            task_state::schedule_now(&state_key(id))?;
        } else if before.interval != after.interval {
            // ： by ** new ** from up new when （ not before 1 ——
            // need up has 「」 by ）
            task_state::reschedule(&state_key(id), interval_ms(task))?;
        }
        if id == TASK_MODEL_REFRESH && before.interval != after.interval {
            crate::server::core::providers::catalog_refresh::reschedule(interval_ms(task))?;
        }
    }
    if before != after {
        logging::log("[Tasks]", &format!("定时任务「{}」已{}，间隔 {} {}", task.label,
            if after.enabled { "开启" } else { "关闭" }, after.interval,
            if task.unit == "seconds" { "秒" } else { "分钟" }));
    }
    let mut result = task_by_id(id);
    result["saved"] = json!(saved);
    Ok(result)
}

pub async fn run_now(store: &AccountStore, update: &UpdateManager, id: &str) -> Result<String, String> {
    let task = find(id).ok_or_else(|| format!("未知的定时任务: {id}"))?;
    if task.runner != Runner::Backend {
        return Err(format!("「{}」由界面自己刷新，无法在后端立即执行", task.label));
    }
    run_backend(store, update, task, true).await
}

/// 「failure」 not this 1 failure（`RunGuard::finish` `success`）。
///
/// ── Why not ──────────────────────────────────────────────
/// `success = false` will by failure（ 24 when ， `core::task_state`
/// `RunGuard::finish`）。 after failure is ** by ** ： 1 
/// up （refreshToken 、 401） 1 failure 1 ，
/// is to when after ， its 1 old 。failure already 
/// and ， not need need 1 。
///
/// directory new ， and failure is ** by ** （ `TASK_MODEL_REFRESH`
/// use ）： 1 401 by to 24 when up ， up 
/// 「 in 」， is new can 。
///
/// ── Why「 1 no success」failure ───────────────────────────
/// failure up （、 、 up ）： this when ，
/// by up 。 is 「 has failure ** and ** 1 no success」。
fn round_succeeded(succeeded: usize, failed: usize) -> bool {
    failed == 0 || succeeded > 0
}

async fn run_backend(
    store: &AccountStore,
    update: &UpdateManager,
    task: &TaskDef,
    manual: bool,
) -> Result<String, String> {
    // update when set by ，must not is when 。
    if task.id == TASK_UPDATE_CHECK {
        let current = crate::server::core::update::CURRENT_VERSION;
        let info = if manual { update.check(current).await } else { update.check_scheduled(current).await }
            .map_err(|error| error.message)?;
        return Ok(if info.get("hasUpdate").and_then(Value::as_bool) == Some(true) {
            let summary = format!("发现新版本 {}（当前 {current}）", info["latestVersion"].as_str().unwrap_or(""));
            logging::log("[Update]", &summary);
            summary
        } else { "已完成版本检查".to_string() });
    }
    // not failure： by ** is **（ `ManualBackoff`）。
    // · 「 up 1 Why no success」 （directory new 、）——
    // use by by is that Reason（、 new 、 ），
    // old will by is ；
    // · 「 up quota when 」 （checkupdate）—— before 1 will 1 
    // 403 and when new to ， use need etc has use 。
    let backoff = match task.id {
        TASK_MODEL_REFRESH | TASK_CREDENTIAL_MAINTENANCE => task_state::ManualBackoff::Bypass,
        _ => task_state::ManualBackoff::Respect,
    };
    let guard = match task_state::claim(task.id, interval_ms(task), manual, backoff, 1_000)? {
        Claim::Acquired(guard) => guard,
        // Reason need ： Bypass （ new / ） in 
        // path up and not ， this when 「 in N seconds after 」 will use minutes
        // when after —— this 1 is in or 1 seconds 。
        Claim::Deferred(state) => {
            return Err(if manual && backoff == task_state::ManualBackoff::Bypass {
                state.waiting_message_bypassing()
            } else {
                state.waiting_message()
            })
        }
    };
    let (summary, success) = match task.id {
        TASK_CREDENTIAL_MAINTENANCE => {
            let results = crate::server::core::credential_maintenance::refresh_expiring_accounts(store).await;
            let (refreshed, skipped, failed) = crate::server::core::credential_maintenance::summarize(&results);
            let summary = format!("刷新 {refreshed} 个，跳过 {skipped} 个，失败 {failed} 个");
            if refreshed > 0 || failed > 0 {
                logging::log_with_level("[Maintenance]", &format!("凭证自动维护：{summary}"), if failed > 0 { "error" } else { "info" });
            }
            (summary, round_succeeded(refreshed, failed))
        }
        TASK_MODEL_REFRESH => {
            let results = if manual {
                crate::server::core::providers::adapter::refresh_implemented_forced(store, &serde_json::Map::new(), None).await
            } else {
                crate::server::core::providers::adapter::refresh_implemented(store).await
            };
            let count = |status: &str| results.iter().filter(|item| item["status"] == status).count();
            let (refreshed, skipped, failed) = (count("refreshed"), count("skipped"), count("failed"));
            // （ no to / no has available） in path up is ：
            // new and new will ， this when 「success 0 」failure，
            // this 1 to 1 「none can 」。
            let summary = if refreshed == 0 && failed == 0 {
                format!("本轮无需刷新（跳过 {skipped} 家：未到间隔或没有可用登录态）")
            } else {
                format!("成功 {refreshed} 家，跳过 {skipped} 家，失败 {failed} 家")
            };
            // models.dev 1 （ is 1 1 ： and 
            // 1 is 1 can use ），**** —— directory
            // TTL and none， 1 not this 1 not 。
            //
            // failure** not **： is ， as 
            // failure will that failure， need new ；
            // failureReason need （ use to ）， this is 「 not 」 and 「 not 」 
            // in 。
            let metadata = crate::server::core::model_meta::refresh(manual, interval_ms(task)).await;
            let metadata_note = match metadata.get("status").and_then(Value::as_str) {
                Some("refreshed") => format!(
                    "；models.dev 元数据已更新 {} 条",
                    metadata.get("count").and_then(Value::as_u64).unwrap_or(0)
                ),
                Some("failed") => format!(
                    "；models.dev 元数据刷新失败（{}）",
                    metadata.get("message").and_then(Value::as_str).unwrap_or("未知原因")
                ),
                // skipped（ to / set） not need ： that is ， is 
                _ => String::new(),
            };
            // and 1 （`round_succeeded`）： failure** not this **
            // 1 。 this failure is ** by ** —— 1 up 
            // （CatPaw that 401） 1 failure 1 ， is failure
            // by 1 to 24 when limit， its can 1 in old up ，
            // up 「 in ，N when after 」， is new can 。
            // failure already and ， not need need failure。
            //
            // 「 1 no success」failure： that / / up 
            // ， not will up 。
            (format!("{summary}{metadata_note}"), round_succeeded(refreshed, failed))
        }
        _ => return Err("未知后端任务".to_string()),
    };
    guard.finish(success, summary.clone(), None, 0, interval_ms(task))?;
    Ok(summary)
}

/// ： after by config （ by `ServerState::bootstrap`
/// 1 ， has this 1 ）。
///
/// ：config → to → 。all in 
/// （`core::task_state`），** not **： up 10:00 update
/// （ 20 minutes），10:05 / 10:10 not will ， etc to 10:20；
/// that 1 then in after 1 （ 1 ， not 1 ）。
///
/// tick 1 **** （ not is 1 ）： 1 
/// ，empty and before 「」 1 。unavailable when 
/// —— that when not can ， can this 1 not 。
pub fn spawn(store: AccountStore, update: UpdateManager) {
    crate::spawn_task(async move {
        // by ： from to 「 in 」（ is new 
        // 1 ， and before 1 ）， use 「 up + 」——
        // already to to ， this 1 up 。
        for task in TASKS.iter().filter(|task| task.runner == Runner::Backend) {
            if let Err(error) = task_state::reschedule(&state_key(task.id), interval_ms(task)) {
                logging::log("[Tasks]", &format!("恢复「{}」排期失败：{error}", task.label));
            }
        }
        loop {
            let Ok(states) = task_state::read_all() else {
                tokio::time::sleep(Duration::from_millis(TICK_MS)).await;
                continue;
            };
            for task in TASKS.iter().filter(|task| task.runner == Runner::Backend) {
                if !settings_of(config::scheduled_settings(), task.id).enabled {
                    continue;
                }
                // after can can ， by to claim 。
                let state = states.get(&state_key(task.id));
                // in （） or no to （failure / ）
                if state.is_some_and(|state| state.running() || logging::now_ms() < state.due_at()) {
                    continue;
                }
                // to ： 1 （`run_backend` by current new ）。
                // （ 1 to 、 or in ） not error， verbose。
                if let Err(error) = run_backend(&store, &update, task, false).await {
                    logging::verbose("[Tasks]", &format!("{}：{error}", task.label));
                }
            }
            tokio::time::sleep(Duration::from_millis(TICK_MS)).await;
        }
    });
}
