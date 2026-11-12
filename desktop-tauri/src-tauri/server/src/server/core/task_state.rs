//! after **** and （、）。
//!
//! ── this ─────────────────────────────────────
//! before after in （`scheduled_tasks` `RUNS`）： 1 
//! ， to 「 in 」， is ** will up 1 ** ——
//! 1 minutes，GitHub （60 / when ， by IP ） 1 
//! ；、 new 、，。
//!
//! 「 up / up success / down / failure」 1 `kv`
//! （ `backgroundTaskState`， `db::schema` ）， is ：
//! - ** not **：10:00 update（ 20 minutes），10:05、10:10 
//! not ， etc to 10:20；
//! - ** 1 **： / that 1 in after 1 ， not 
//! 1 ；
//! - **failure has **：failure by （limit 1 ），and** not **
//! —— otherwise「 can 」 etc no has 。 need not need by use 
//! （ [`ManualBackoff`]）。
//!
//! ── ：request** before ** 1 ─────────────────────
//! `claim` in return before `lastAttemptAt` / `nextRunAt` 。 by ：
//! in success after ，「request in / failure / up when 」 this path will down 
//! 1 「 no 」 ， down —— that is need as 。
//! is 「failure that 1 use 」， by failure（`retry_at`）。
//!
//! ── （lease）： not will ────────────────────
//! 1 when 1 request（ + 90 seconds）。**
//! when ** not is 1 ： / after no has ， has to 
//! will —— will in 「 in 」。 `RunGuard` 1 
//! ，（`finish`） and in （`Drop`） will 。
//!
//! ── Why not before 、 1 for ────────────────
//! `kv` must `db::schema::RESERVED_KV_KEYS`（configexcluding
//! 「 not 」 ）， 1 1 need ， for ；
//! 1 for 1 ， and add not schema that 。 is 「
//! 」， in 1 （`Db::with_mut` + `IMMEDIATE`）， not will 
//! update —— is （ 1 ， up 30 seconds 1 ）。
//!
//! ── use 1 ──────────────────────────────────────
//! and and （ and identifier ）， use 1 
//! configdirectory and 。「 to → use 」must in 1 
//! （`TransactionBehavior::Immediate` ），otherwise will in 1 
//! ms 1 request。
//!
//! ── ： not ────────────────────────────────
//! and `config::sql` / `providers::catalog_cache` 1 ：`Db::with*` is 
//! 1 ， `logging::log` / `verbose` need 1 `logs` 
//! —— `std::sync::Mutex` not can ， in will 。 
//! 1 in （`now_ms` when ， not IO）。

mod storage;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use self::storage::change;
use crate::server::db::Db;
use crate::server::logging;

/// when ： 30 seconds 1 ，。
///
/// Why not is 「」： 1 can can request，
/// minutes is 。 need 「 that 1 not will 」，expiration time by 
/// not after ； is ** **， that when 90 seconds after 。
const LEASE_MS: i64 = 90_000;

/// failure limit（ 1 ）： up unavailable when need 1 ，
/// otherwise「 up in etc 」 will need need can 。
const MAX_BACKOFF_MS: i64 = 24 * 60 * 60_000;
static DB: OnceLock<Option<Db>> = OnceLock::new();
static SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TaskState {
    pub last_attempt_at: i64,
    pub last_run_at: i64,
    pub last_success_at: i64,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
    pub next_run_at: i64,
    pub retry_at: i64,
    pub failures: u32,
    pub value: Option<Value>,
    interval_ms: i64,
    lease_until: i64,
    owner: String,
}

impl TaskState {
    pub fn running(&self) -> bool {
        !self.owner.is_empty() && self.lease_until > logging::now_ms()
    }

    pub fn due_at(&self) -> i64 {
        self.next_run_at.max(self.retry_at).max(self.lease_until)
    }

    pub fn waiting_message(&self) -> String {
        self.describe_wait(false)
    }

    /// **failure** when use that 1 （ [`ManualBackoff::Bypass`]）。
    ///
    /// in 「 up 1 Why no success」， not 「 in not can 」—— this when 
    /// this 1 is in or request。 [`waiting_message`] 
    /// will use 「 when after 」， 1 seconds after can 1 。
    pub fn waiting_message_bypassing(&self) -> String {
        self.describe_wait(true)
    }

    fn describe_wait(&self, backoff_bypassed: bool) -> String {
        if self.running() {
            return "任务正在执行中，请稍候".to_string();
        }
        let seconds = ((self.retry_at - logging::now_ms()).max(0) + 999) / 1000;
        if seconds > 0 && !backoff_bypassed {
            format!("请求处于冷却期，请在 {seconds} 秒后重试")
        } else {
            "刚刚已执行，请稍后重试".to_string()
        }
    }

    fn adjust_clock(&mut self, now: i64) {
        if self.last_attempt_at <= now + 60_000 {
            return;
        }
        // when when remaining etc ， not can old when none。
        let shift = self.last_attempt_at - now;
        self.last_attempt_at = now;
        self.last_run_at = (self.last_run_at - shift).max(0);
        self.last_success_at = (self.last_success_at - shift).max(0);
        self.next_run_at = (self.next_run_at - shift).max(now);
        self.retry_at = (self.retry_at - shift).max(0);
        self.lease_until = (self.lease_until - shift).max(0);
    }
}

pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn database() -> Result<&'static Db, String> {
    DB.get()
        .and_then(Option::as_ref)
        .ok_or_else(|| "任务状态数据库不可用，未发送后台请求".to_string())
}

pub fn read(key: &str) -> Result<TaskState, String> {
    Ok(read_all()?.get(key).cloned().unwrap_or_default())
}

pub fn read_all() -> Result<HashMap<String, TaskState>, String> {
    storage::read_all()
}

pub fn store_value(key: &str, value: Value) -> Result<(), String> {
    change(key, |state| state.value = Some(value)).map(|_| ())
}

pub(crate) fn update_value(key: &str, update: impl FnOnce(&mut Value)) -> Result<Value, String> {
    change(key, |state| update(state.value.get_or_insert(Value::Null)))
        .map(|state| state.value.unwrap_or(Value::Null))
}

/// 1 **** request（ use that 1 ）： when 1 
/// ，「、 to or after 1 」。
///
/// in that 1 （valid） not —— that is ， by `RunGuard`
/// in when 1 。failure not ： not this up before 。
pub fn note_external_run(key: &str, interval_ms: i64) -> Result<(), String> {
    change(key, |state| {
        if state.running() {
            return;
        }
        let now = logging::now_ms();
        state.adjust_clock(now);
        state.last_attempt_at = now;
        state.next_run_at = now.saturating_add(interval_ms).max(state.retry_at);
    })
    .map(|_| ())
}

/// as 「 in to 」（ use in up from to when use ）。
///
/// and `note_external_run` is ** need not need **： that 1 is 「 1 」，
/// this 1 is 「 use need this 」—— after before 「 1 」
/// ， not up when （ has failure：`due_at` ）。
pub fn schedule_now(key: &str) -> Result<(), String> {
    change(key, |state| {
        let now = logging::now_ms();
        state.adjust_clock(now);
        state.next_run_at = now;
    })
    .map(|_| ())
}

/// 1 **failure**（`retry_at` ）， not and when 。
///
/// use ： use 「 up failure before already 」—— for updatecheck is /
/// GitHub （ is 1 quota when ， by IP 、
/// by use ， after old not use ）。 use can 1 ， not is 
/// for up 「checkupdate」 by empty etc minutes。
///
/// （`next_run_at`）** not **： when by ， this 
/// 「 in not 」 that 。`adjust_clock` and its 1 1 ， when 
/// old when none。
pub fn clear_cooldown(key: &str) -> Result<(), String> {
    change(key, |state| {
        let now = logging::now_ms();
        state.adjust_clock(now);
        state.retry_at = 0;
    })
    .map(|_| ())
}

/// 1 / as ， not or new when 。
pub fn reschedule(key: &str, interval_ms: i64) -> Result<(), String> {
    change(key, |state| {
        let now = logging::now_ms();
        state.adjust_clock(now);
        state.interval_ms = interval_ms;
        let anchor = state.last_run_at.max(state.last_attempt_at);
        state.next_run_at = if anchor > 0 { anchor.saturating_add(interval_ms) } else { now };
        state.next_run_at = state.next_run_at.max(state.retry_at);
    })
    .map(|_| ())
}

pub enum Claim {
    Acquired(RunGuard),
    Deferred(TaskState),
}

/// ** not failure**（[`claim`] ）。
///
/// in when in ， and has by —— not in `manual` ：
/// - [`ManualBackoff::Respect`]： is ** up quota when **
/// （checkupdate GitHub by IP 、 by use ）。 before 1 
/// will 1 403， when new to ， not use need etc ；
/// - [`ManualBackoff::Bypass`]： is ** up 1 Why no success**（directory by 
/// up 401 / 5xx ； by failure）。 use by down 
/// 「get」/「」 is 「 in 1 」， and by by is 
/// as that Reason（ new 、、 ）—— old 
/// ， up will up 1 error， is by 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManualBackoff {
    Respect,
    Bypass,
}

/// to and （ is `Ok`：Deferred not is error， is this not this ）。
///
/// `manual`（ use ）；** is failure 1 ** by `backoff`
/// （ [`ManualBackoff`]， use ）。
///
/// in and request** not **： before 1 when （ or ）
/// and up ， after is 「 1 1 seconds not 」 —— can ， and and 
/// not ： use by when ， up is this ， not is that when 。
pub fn claim(
    key: &str,
    interval_ms: i64,
    manual: bool,
    backoff: ManualBackoff,
    min_gap_ms: i64,
) -> Result<Claim, String> {
    let now = logging::now_ms();
    let owner = format!(
        "{}-{now}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    );
    let mut acquired = false;
    let state = change(key, |state| {
        state.adjust_clock(now);
        let earliest = state.last_attempt_at.saturating_add(min_gap_ms);
        let cooling = state.retry_at > now && !(manual && backoff == ManualBackoff::Bypass);
        if state.running() || now < earliest || cooling || (!manual && now < state.next_run_at) {
            return;
        }
        acquired = true;
        state.last_attempt_at = now;
        state.interval_ms = interval_ms;
        state.next_run_at = now.saturating_add(interval_ms);
        state.lease_until = now.saturating_add(LEASE_MS);
        state.owner = owner.clone();
    })?;
    if !acquired {
        return Ok(Claim::Deferred(state));
    }
    let heartbeat_key = key.to_string();
    let heartbeat_owner = owner.clone();
    let heartbeat = crate::spawn_task(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let result = change(&heartbeat_key, |state| {
                if state.owner == heartbeat_owner {
                    state.lease_until = logging::now_ms().saturating_add(LEASE_MS);
                }
            });
            if !matches!(result, Ok(state) if state.owner == heartbeat_owner) {
                break;
            }
        }
    });
    Ok(Claim::Acquired(RunGuard {
        key: key.to_string(),
        owner,
        heartbeat,
        finished: false,
    }))
}

pub struct RunGuard {
    key: String,
    owner: String,
    heartbeat: tokio::task::JoinHandle<()>,
    finished: bool,
}

impl RunGuard {
    pub fn finish(
        mut self,
        success: bool,
        summary: String,
        value: Option<Value>,
        retry_at: i64,
        interval_ms: i64,
    ) -> Result<TaskState, String> {
        self.heartbeat.abort();
        let result = change(&self.key, |state| {
            if state.owner != self.owner {
                return;
            }
            let now = logging::now_ms();
            state.last_run_at = now;
            state.last_result = Some(summary.clone());
            state.interval_ms = interval_ms;
            if success {
                state.last_success_at = now;
                state.last_error = None;
                state.failures = 0;
                state.retry_at = retry_at;
            } else {
                state.last_error = Some(summary);
                state.failures = state.failures.saturating_add(1);
                let multiplier = 1_i64 << state.failures.saturating_sub(1).min(10);
                let delay = interval_ms.max(60_000).saturating_mul(multiplier).min(MAX_BACKOFF_MS);
                state.retry_at = retry_at.max(now.saturating_add(delay));
            }
            if let Some(value) = value {
                state.value = Some(value);
            }
            state.next_run_at = now.saturating_add(interval_ms).max(state.retry_at);
            state.owner.clear();
            state.lease_until = 0;
        });
        self.finished = result.is_ok();
        result
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        self.heartbeat.abort();
        if self.finished {
            return;
        }
        let _ = change(&self.key, |state| {
            if state.owner == self.owner {
                state.owner.clear();
                state.lease_until = 0;
                state.last_result = Some("上次执行中断，保留原有排期".to_string());
            }
        });
    }
}
