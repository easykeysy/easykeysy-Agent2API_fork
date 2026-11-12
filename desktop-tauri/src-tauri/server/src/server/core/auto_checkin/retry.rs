//! in ****（`lastResult.retryAccounts`）+ 。
//!
//! ── not ， ──────────────────────────────────────
//! 1. ** etc **（ `due_accounts` / `merge`）： up 1 to 「 in 
//! 」 ， etc to `retryAt`（ or config ） to 1 。
//! is 「 no / up 1 when no down 」， and 「 no 」none。
//! 2. ****（[`unsigned_accounts`]）： to this ，`checkinAt`
//! not in that —— / failure when ， this 1 。
//!
//! ── Why not retryAccounts ─────────────────────────
//! `retryAccounts` 「 up no 」 ； 429 / failure is 
//! **failed**， not that （ `retry_at` before return）。 need 
//! is this —— can is 「 up no has 」 this 。
//!
//! ── and can ──────────────────────────────────────────
//! `RETRY_GAP_MS` in is ； up `retryGapMinutes` / `retryLimit` will 
//! （ `checkin_schedules::Schedule`）。`retryLimit = 0` or 
//! `retryGapMinutes = 0` 「 not 」—— use need 「 up 」 when need can 
//! ， before that is 1 10 minutes。
//!
//! ── `retryLimit` is ** limit**（2026-10 ）────────────────
//! in this before `retryLimit` use （`<= 0` ）， from no ：
//! 1 need 1 to 「 in 」， will by `retryGapMinutes` 1 ——
//! Qoder from 10:01 1 to up ， to 「」，
//! is 「 N 」， and as for not up 。
//!
//! in by `lastResult.retryAttempts`（`{accountId: already }`）：
//! · has ****（[`Round`]）—— and not is 「」，
//! will use 3 1 ；
//! · to `retryLimit` after not `retryAccounts`（），
//! `due_accounts_scoped` 1 （）—— before down old no has ，
//! will that 1 by as ；
//! · and ，（ and `retryAccounts` 1 ）。
//! use by [`exhausted_accounts`] to ，「 not 、
//! 」， not is from 、「 no 」。

use std::collections::BTreeMap;

use serde_json::Value;

use crate::server::core::checkin_schedules::Schedule;
use crate::server::core::{account_store::AccountStore, beijing, billing::checkin};

/// （ no when use ）
const RETRY_GAP_MS: i64 = 10 * 60 * 1000;

/// 1 to ： has 「」 1 。
///
/// ── Whymust（ this before ）─────────────────────────
/// up `retryLimit` is 「 」。 is 1 
/// to 、 is 「 no up 1 」， not is 「」； ，
/// use 3 1 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Round {
    /// etc （`retryAccounts` to that ）。
    Window,
    /// / / ：， not 。
    Other,
}

/// 1 （ / limit / ）。
///
/// 1 struct is as use can ：`merge(..., RetryConfig::of(schedule, round))`
/// `merge(..., gap_ms, limit, round)` 1 「 is 」 for 。
#[derive(Clone, Copy, Debug)]
pub(super) struct RetryConfig {
    /// （ms），0 = 。
    pub(super) gap_ms: i64,
    /// ，0 = 。
    pub(super) limit: i64,
    /// this 1 ： has [`Round::Window`] 。
    pub(super) round: Round,
}

impl RetryConfig {
    pub(super) fn of(schedule: Option<&Schedule>, round: Round) -> Self {
        Self {
            gap_ms: gap_ms_of(schedule),
            limit: limit_of(schedule),
            round,
        }
    }
}

/// **** that `lastResult`（ / to `None`）。
///
/// old 1 ：otherwise will 1 up ，
/// will 。
fn today_of(previous: Option<&Value>) -> Option<&Value> {
    previous.filter(|value| {
        value.get("date").and_then(Value::as_str) == Some(beijing::today_key().as_str())
    })
}

/// 「 already 」（`lastResult.retryAttempts`，`{accountId: }`）。
/// and 1 0（config not this 、 limit）。
fn attempts_of(previous: &Value) -> BTreeMap<String, i64> {
    previous
        .get("retryAttempts")
        .and_then(Value::as_object)
        .map(|map| {
            map.iter()
                .filter_map(|(id, value)| value.as_i64().map(|count| (id.clone(), count.max(0))))
                .collect()
        })
        .unwrap_or_default()
}

/// this （`None` = no has can ）。
///
/// `previous` is up 1 `lastResult`；`gap_ms` / `limit` config
/// （ `checkin_schedules::Schedule` `retryGapMinutes` / `retryLimit`）。
/// has **** ： old 1 （otherwise will 
/// 1 up ）。
///
/// use in this 1 ：（[`merge`]） already ， that 
/// path「 after 」； before down old is no ，
/// will that 1 by as 1 （2026-10 ）。
pub(super) fn due_accounts_scoped(
    previous: Option<&Value>,
    now: i64,
    gap_ms: i64,
    limit: i64,
) -> Option<Vec<String>> {
    if gap_ms <= 0 || limit <= 0 {
        return None;
    }
    let previous = today_of(previous)?;
    let attempts = attempts_of(previous);
    let retry = previous.get("retryAccounts").and_then(Value::as_object)?;
    let due: Vec<String> = retry
        .iter()
        .filter(|(id, at)| {
            attempts.get(*id).map(|count| *count < limit).unwrap_or(true)
                && at.as_i64().is_some_and(|at| at <= now)
        })
        .map(|(id, _)| id.clone())
        .collect();
    // empty to None： use （`tick`）empty， to `Some([])` will 
    // 1 「 has 1 not use 」 
    (!due.is_empty()).then_some(due)
}

/// （ms）：`retryGapMinutes` ；0 = 。
pub(super) fn gap_ms_of(schedule: Option<&Schedule>) -> i64 {
    match schedule {
        Some(schedule) if schedule.retry_gap_minutes <= 0 => 0,
        Some(schedule) => schedule.retry_gap_minutes.saturating_mul(60_000),
        None => RETRY_GAP_MS,
    }
}

/// limit：0 = 。
pub(super) fn limit_of(schedule: Option<&Schedule>) -> i64 {
    schedule.map(|schedule| schedule.retry_limit).unwrap_or(3)
}

fn retry_at(store: &AccountStore, row: &Value, now: i64, gap_ms: i64) -> Option<i64> {
    if row
        .get("error")
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
        || checkin::day_completed(row)
    {
        return None;
    }
    let claim = row.get("claim")?;
    if !checkin::claim_pending(claim) {
        return None;
    }
    if let Some(at) = claim.get("retryAt") {
        return at
            .as_i64()
            .filter(|at| *at > 0)
            .map(|at| at.max(now + 30_000));
    }
    let id = row.get("id").and_then(Value::as_str)?;
    let provider = store.account_provider_of(id)?;
    let qoder = crate::server::core::providers::qoder::endpoints::Region::Global.provider_id();
    if provider == qoder {
        // Qoder by in `retryAt` to （10:00 that ）； this 
        // is no to when ： etc to 10:00， and `providers::qoder::checkin`
        // `daily_window_start` 1 。
        //
        // **「 no 」**： no to `retryAt`， is already 
        // 「 this no has can 」（ `qoder::checkin::no_claim` ），
        // this when 「 10 minutes」 will 1 to up —— 2026-10 that 
        // 「」 is this 。
        let window = beijing::now()
            .date_naive()
            .and_hms_opt(10, 0, 0)?
            .and_local_timezone(beijing::offset())
            .single()?
            .timestamp_millis();
        return (now < window).then(|| window.max(now + gap_ms));
    }
    (provider == "codearts").then_some(now + gap_ms)
}

/// 1 and `lastResult`： + 。
///
/// ── has （ and 1 ）────────────────────────
/// · `retryAccounts`：`{accountId: down can when }` —— 「 need 1 」；
/// · `retryAttempts`：`{accountId: already }` —— 「 will 」。
/// has no 「」， has no in 「 need not need 」。
///
/// in [`Round::Window`] up （ [`Round`]）： / 
/// `retried`（ that is 「 」）， not is 「」。
pub(super) fn merge(
    store: &AccountStore,
    mut summary: Value,
    result: &Value,
    previous: Option<&Value>,
    retried: Option<&[String]>,
    config: RetryConfig,
) -> Value {
    let Some(fields) = summary.as_object_mut() else {
        return summary;
    };
    let now = crate::server::logging::now_ms();
    // ── ：**** from up 1 ────────────────────
    // 1 1 will （ when not ）， 1 that 。 if 
    // from empty， up 1 down 「 etc 」 will （CodeArts that 
    // will will ）。 is old ， 
    // that id 。。
    let previous_today = today_of(previous);
    let mut retry = previous_today
        .and_then(|value| value.get("retryAccounts"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut attempts = previous_today.map(attempts_of).unwrap_or_default();
    let retry_enabled = config.gap_ms > 0 && config.limit > 0;
    if !retry_enabled { retry.clear(); }
    if let Some(retried) = retried {
        for id in retried {
            retry.remove(id);
            // has ；`limit <= 0`（） when not this 
            if config.round == Round::Window && config.limit > 0 {
                let count = attempts.entry(id.clone()).or_insert(0);
                *count = count.saturating_add(1);
            }
        }
    }
    if let (Some(previous), Some(_)) = (previous_today, retried) {
        for key in ["succeeded", "active", "failedCount"] {
            let before = previous.get(key).and_then(Value::as_u64).unwrap_or(0);
            let current = fields.get(key).and_then(Value::as_u64).unwrap_or(0);
            fields.insert(key.to_string(), Value::from(before.saturating_add(current)));
        }
        for key in ["total", "skipped"] {
            let before = previous.get(key).and_then(Value::as_u64).unwrap_or(0);
            let current = fields.get(key).and_then(Value::as_u64).unwrap_or(0);
            fields.insert(key.to_string(), Value::from(before.saturating_add(current)));
        }
        let mut failures = previous
            .get("failed")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        failures.extend(
            fields
                .get("failed")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
        failures.truncate(5);
        fields.insert("failed".to_string(), Value::Array(failures));
    }
    if let Some(rows) = result.get("results").and_then(Value::as_array) {
        for row in rows {
            let Some(id) = row.get("id").and_then(Value::as_str) else {
                continue;
            };
            // 1 from old ， by `retry_at` need not need ——
            // otherwise「 up in etc 、 this already up 」 will in 。
            // （ before is ，`retried` ； in by ，
            // that `retried` is None，must this 。）
            retry.remove(id);
            if checkin::day_completed(row) {
                // up ： 1 and ，otherwise will 1 
                // 「 already use 」—— already ， not is no will 
                attempts.remove(id);
                continue;
            }
            // already to limit： not （ this is 「 N 」 
            // ）。 to ， is `retryExhausted`。
            if !retry_enabled || attempts.get(id).is_some_and(|count| *count >= config.limit) {
                continue;
            }
            if let Some(at) = retry_at(store, row, now, config.gap_ms) {
                retry.insert(id.to_string(), Value::from(at));
            }
        }
    }
    // ：「 in 」 and 「 already use 」。 already up / already 
    // not need need （ is `checkinAt`， not this ）。
    attempts.retain(|id, count| retry.contains_key(id) || *count >= config.limit);
    fields.insert("retryAccounts".to_string(), Value::Object(retry));
    fields.insert(
        "retryAttempts".to_string(),
        serde_json::to_value(&attempts).unwrap_or(Value::Null),
    );
    summary
}

pub(super) fn next_at(result: Option<&Value>, today: &str, limit: i64) -> Option<i64> {
    let result = result.filter(|value| value.get("date").and_then(Value::as_str) == Some(today))?;
    let attempts = attempts_of(result);
    result
        .get("retryAccounts")
        .and_then(Value::as_object)?
        .iter()
        // use not will ， not this when 「 down 」
        .filter(|(id, _)| attempts.get(*id).map(|count| *count < limit).unwrap_or(true))
        .filter_map(|(_, at)| at.as_i64())
        .min()
}

/// ** already use ** id（「 not 、」）。
///
/// must to ， not can this from ：`retryCount`（ need 
/// ） will ， use to is 「 no 」， is 「 no 」。
///
/// use `>=` not is `>`： in ** to **limit that 1 ，
/// 「」 and 「 not in 」 is 1 when 。
pub(super) fn exhausted_accounts(previous: Option<&Value>, limit: i64) -> Vec<String> {
    if limit <= 0 {
        return Vec::new();
    }
    let Some(previous) = today_of(previous) else {
        return Vec::new();
    };
    attempts_of(previous)
        .into_iter()
        .filter(|(_, count)| *count >= limit)
        .map(|(id, _)| id)
        .collect()
}

/// ** no up id**（ ）。
///
/// is `checkinAt` is in （ when ）—— and 「 already to 」、
/// to in `checkedInToday` 1 ， not will to 。
///
/// not `available` / `enabled`： that is ， and to none（
/// `billing::checkin` module header「 to not enabled」）。 this not this by 
/// `resolve_checkin_targets` `supports_checkin` and 。
pub(super) fn unsigned_accounts(store: &AccountStore) -> Vec<String> {
    store
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|account| !crate::server::core::account_store::checkin_state::checked_in_today(account))
                .filter_map(|account| account.get("id").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
