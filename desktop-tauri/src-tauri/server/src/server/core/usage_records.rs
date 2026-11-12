//! ****（`account_usage_records` ） and use ****。
//!
//! ── this ─────────────────────────────────────
//! 「 when 」 after ， by to （`core::usage_query`
//! ）。 1 （ 1 1 ， id， is 
//! UPSERT） old kv as 「 this 1 to 」 ：
//! - **update**： to that 1 ， not ；
//! - ** IO**：（ not ）request need 1 ，
//! that `BalanceFact`（ id → success ）
//! seconds —— from JSON or not this 。
//!
//! ── failure up success （OmniProxy 1 ）──────────────
//! `remaining` in **success** when ：failure up success ，
//! not will as 「 this failure」 （「failure → → 402」 ）；
//! need need success and to up 。failure in `error` / `code`
//! ， and not 。failure（ before from success） no has can ，
//! `remaining` as NULL —— 「 not 」 1 ， not 。
//!
//! ── request** before ** 1 ─────────────────────────────
//! path in before `mark_attempt`（ and `core::task_state` claim 1 
//! ）：「request in 」 not will down 「 no 」 ， down 1 by ，
//! not will up 。 is 「failure that 1 use 」，
//! this is need （failure not 、 by use ）。
//!
//! ── use 1 ──────────────────────────────────────
//! and and 。 no has （ is ，
//! 1 can ignore）， `mark_attempt` 
//! to 「」， not will up 。
//!
//! ── ：（） not ──────────────────────
//! and `core::task_state` 1 ：`Db::with*` is 1 ，
//! `logging::log` need 1 `logs` —— will 。
//! 1 in 。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::server::core::account_store::store_util::js_truthy;
use crate::server::db::Db;
use crate::server::logging;

/// old kv （「 when 」 when ）。
/// in `db::schema::RESERVED_KV_KEYS` ： in （failure not ） when ，
/// config 「delete already not in 」 not can —— that is 1 can 。
const LEGACY_SNAPSHOT_KEY: &str = "usageQuerySnapshot";

static DB: OnceLock<Option<Db>> = OnceLock::new();

/// ： ****（ id → 1 success）。
#[derive(Clone, Copy, Debug)]
pub struct BalanceFact {
    /// 1 success （ and 1 ）。`None` = not 
    /// （unlimited、none、 not ）—— 1 。
    pub remaining: Option<f64>,
}

fn facts_slot() -> &'static Mutex<HashMap<String, BalanceFact>> {
    static FACTS: OnceLock<Mutex<HashMap<String, BalanceFact>>> = OnceLock::new();
    FACTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 1 （； not in when ）。
#[derive(Clone, Debug)]
pub struct UsageRecord {
    /// 1 **success** 1 JSON（ ）。
    pub usage: Option<Value>,
    /// 1 failure Reason（successempty）。
    pub error: Option<String>,
    /// failure can （「not configured」）。
    pub code: Option<String>,
    /// （success when from usage ；failure up success； not as NULL）。
    pub remaining: Option<f64>,
    pub unlimited: bool,
    /// 1 success / when （ms；0 = from ）。
    pub last_success_at: i64,
    pub last_attempt_at: i64,
}

/// 1 ****（success usage，failure error； not to = as ，
/// use 1 ）。
pub struct UsageOutcome {
    pub usage: Option<Value>,
    pub error: Option<String>,
    pub code: Option<String>,
}

pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
    migrate_legacy_snapshot();
    reload_facts();
}

fn database() -> Option<&'static Db> {
    DB.get().and_then(Option::as_ref)
}

// ─── ──────────────────────────────────────────────────────

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageRecord> {
    let usage_text: Option<String> = row.get("usage")?;
    Ok(UsageRecord {
        usage: usage_text.and_then(|text| serde_json::from_str(&text).ok()),
        error: row.get("error")?,
        code: row.get("code")?,
        remaining: row.get::<_, Option<f64>>("remaining")?,
        unlimited: row.get::<_, i64>("unlimited").unwrap_or(0) != 0,
        last_success_at: row.get("last_success_at").unwrap_or(0),
        last_attempt_at: row.get("last_attempt_at").unwrap_or(0),
    })
}

const RECORD_COLUMNS: &str =
    "usage, error, code, remaining, unlimited, last_success_at, last_attempt_at";

/// 1 （ no has = ： from ）。
pub fn load(account_id: &str) -> UsageRecord {
    let fallback = UsageRecord {
        usage: None,
        error: None,
        code: None,
        remaining: None,
        unlimited: false,
        last_success_at: 0,
        last_attempt_at: 0,
    };
    let Some(db) = database() else { return fallback };
    db.with(|conn| {
        conn.query_row(
            &format!(
                "SELECT {RECORD_COLUMNS} FROM account_usage_records WHERE account_id = ?1"
            ),
            [account_id],
            row_to_record,
        )
        .optional()
        .unwrap_or_default()
    })
    .flatten()
    .unwrap_or(fallback)
}

/// （ ）。unavailable when to empty —— up by 「 no 」。
pub fn load_all() -> HashMap<String, UsageRecord> {
    let Some(db) = database() else { return HashMap::new() };
    db.with(|conn| {
        let Ok(mut statement) = conn.prepare(&format!(
            "SELECT account_id, {RECORD_COLUMNS} FROM account_usage_records"
        )) else {
            return HashMap::new();
        };
        let rows = statement.query_map([], |row| {
            let account_id: String = row.get("account_id")?;
            Ok((account_id, row_to_record(row)?))
        });
        let Ok(rows) = rows else { return HashMap::new() };
        rows.filter_map(|item| item.ok()).collect::<HashMap<_, _>>()
    })
    .unwrap_or_default()
}

/// use （， use 1 Map）。
pub fn balance_facts() -> HashMap<String, BalanceFact> {
    facts_slot()
        .lock()
        .map(|facts| facts.clone())
        .unwrap_or_default()
}

// ─── ──────────────────────────────────────────────────────

/// 1 「」（path in **request before ** use ，module header）。
/// when ， not up 1 —— no ， old is new 。
pub fn mark_attempt(account_id: &str) {
    let now = logging::now_ms();
    let Some(db) = database() else { return };
    let failed = db.with_mut(|conn| {
        conn.execute(
            "INSERT INTO account_usage_records (account_id, last_attempt_at, updated_at)
             VALUES (?1, ?2, ?2)
             ON CONFLICT(account_id) DO UPDATE SET
               last_attempt_at = excluded.last_attempt_at,
               updated_at = excluded.updated_at",
            params![account_id, now],
        )
        .is_err()
    });
    if failed != Some(false) {
        logging::verbose("[Usage]", &format!("账号 {account_id} 余额查询占位失败（库里没记上本次尝试）"));
    }
}

/// 1 （success，failureReason —— module header ）。
/// update：success and has → new ；success not （unlimited / not ）
/// → （ not can 1 old in ）。
pub fn write_result(account_id: &str, outcome: &UsageOutcome) {
    if let Some(usage) = &outcome.usage {
        crate::server::core::limiter::sources::note(account_id, usage);
    }
    let now = logging::now_ms();
    let Some(db) = database() else { return };
    // return `( is success, success when )`：failureReason、 old 
    // （SQL not usage / remaining / last_success ）， by update。
    // `with_mut` 1 Option（ in = None），match before flatten。
    let written = db.with_mut(|conn| -> Option<(bool, Option<f64>)> {
        match outcome.usage.as_ref() {
            Some(usage) => {
                let (remaining, unlimited) = extract_remaining(usage);
                let usage_text = serde_json::to_string(usage).ok()?;
                conn.execute(
                    "INSERT INTO account_usage_records
                       (account_id, usage, error, code, remaining, unlimited,
                        last_success_at, last_attempt_at, updated_at)
                     VALUES (?1, ?2, NULL, NULL, ?3, ?4, ?5, ?5, ?5)
                     ON CONFLICT(account_id) DO UPDATE SET
                       usage = excluded.usage,
                       error = NULL,
                       code = NULL,
                       remaining = excluded.remaining,
                       unlimited = excluded.unlimited,
                       last_success_at = excluded.last_success_at,
                       last_attempt_at = excluded.last_attempt_at,
                       updated_at = excluded.updated_at",
                    params![account_id, usage_text, remaining, unlimited as i64, now],
                )
                .ok()?;
                Some((true, remaining))
            }
            None => {
                let error = outcome.error.clone().unwrap_or_else(|| "查询失败".to_string());
                conn.execute(
                    "INSERT INTO account_usage_records
                       (account_id, error, code, last_attempt_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)
                     ON CONFLICT(account_id) DO UPDATE SET
                       error = excluded.error,
                       code = excluded.code,
                       last_attempt_at = excluded.last_attempt_at,
                       updated_at = excluded.updated_at",
                    params![account_id, error, outcome.code, now],
                )
                .ok()?;
                Some((false, None))
            }
        }
    })
    .flatten();
    if matches!(written, Some((true, _))) {
        if let Some(usage) = &outcome.usage {
            crate::server::core::account_order::note_usage(account_id, usage);
        }
    }
    match written {
        // success and → new 
        Some((true, Some(remaining))) => {
            if let Ok(mut facts) = facts_slot().lock() {
                facts.insert(account_id.to_string(), BalanceFact { remaining: Some(remaining) });
            }
        }
        // success not → old （unlimited / after not old ）
        Some((true, None)) => {
            if let Ok(mut facts) = facts_slot().lock() {
                facts.remove(account_id);
            }
        }
        // failure（ or unavailable）→ not （ up success ，module header）
        _ => {}
    }
}

/// 「 already delete」 （ 1 ， 1 SQL）。
pub fn prune_orphans() {
    let Some(db) = database() else { return };
    db.with_mut(|conn| {
        conn.execute(
            "DELETE FROM account_usage_records
             WHERE account_id NOT IN (SELECT id FROM accounts)",
            [],
        )
        .ok()
    });
}

// ─── （ and ）─────────────────────

/// 1 JSON → `(remaining, unlimited)`。
///
/// and before （`accounts-panels.tsx` `usageSummary`）：
/// - workbuddy has `totalLeft` ，`unlimited` = ∞（ not ）；
/// - 1 `available`；
/// - not （ / ）→ `None`， by 「none」。
/// 「 by not is by provider 」 by and before 1 ：provider 
/// ， not 。
pub fn extract_remaining(usage: &Value) -> (Option<f64>, bool) {
    let Some(fields) = usage.as_object() else { return (None, false) };
    let unlimited = fields.get("unlimited").map(js_truthy).unwrap_or(false);
    if unlimited {
        return (None, true);
    }
    if let Some(total_left) = fields.get("totalLeft") {
        return (total_left.as_f64().filter(|value| value.is_finite()), false);
    }
    if let Some(available) = fields.get("available") {
        return (available.as_f64().filter(|value| value.is_finite()), false);
    }
    (None, false)
}

// ─── up config（ and use ）─────────

/// up down （seconds）：30 seconds ~ 1 。
/// （`account_store::apply_patch`） and use this 1 for —— 1 
/// 「 will 」。
pub const MIN_QUERY_INTERVAL_SECONDS: i64 = 30;
pub const MAX_QUERY_INTERVAL_SECONDS: i64 = 86_400;

/// up ** no has ** `usageQuery` / `lowBalance` config when 。
///
/// 「」must and when as for ： that when 「 when 」default
/// （ 10 minutes）， not in ； is ****
/// （ 1 minutes —— use ）， not is 「not configured = not 」——
/// otherwise after all will 。
///
/// not by provider （ [`default_low_balance_mode`]）：
/// use （ 1）—— when no has this ， is and 
/// ； Cline ** not **，Reason that 。
pub const DEFAULT_QUERY_INTERVAL_SECONDS: i64 = 60;
pub const DEFAULT_LOW_BALANCE_THRESHOLD: f64 = 1.0;

/// Cline provider id（ from ， and `account_store::CLINE_FREE_PROVIDER_ID`
/// —— id this ）。
const CLINE_FREE_PROVIDER: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::ClineFree,
);

/// provider 「 not 」（ up no has `lowBalance` config when use ）。
///
/// this ** old `lowBalance` ** ；limit up after 
/// then in `limiter::default_rules`。
///
/// ── ： ──────────────────────────────────
/// 「 this 1 」 provider ， in when （
/// `AccountFeatures::default_limiters`）。 old can 「 / not 」，
/// is ：** has 1 then and is ** → `skip`，otherwise → `off`
/// （Token then 、empty then not ， off 1 not will 「」）。
///
/// no （ old ） down ：
/// - **Cline （`cline-free`）→ `off`（ not ）**： credit 
/// 0 、 use is （ is ）， 1 will 
/// from 。 can not can use to up （ no when 402 will by 
/// error）， not 。
/// - ** its provider → `skip`（）**： no 、 has 、
/// ， not need need after （， not ）。
pub fn default_low_balance_mode(provider: &str) -> &'static str {
    if provider == crate::server::core::account_store::codearts_accounts::CODEARTS_PROVIDER_ID {
        return "skip";
    }
    if let Some(declared) = crate::server::core::plugin_host::account_features_of(provider) {
        if let Some(rules) = declared.default_limiters {
            let balance_skip = rules.iter().any(|rule| {
                rule.kind == agent2api_provider_protocol::DefaultLimiterKind::Balance
                    && rule.action == agent2api_provider_protocol::DefaultLimiterAction::Skip
            });
            return if balance_skip { "skip" } else { "off" };
        }
    }
    if provider == CLINE_FREE_PROVIDER {
        "off"
    } else {
        "skip"
    }
}

/// （mode + threshold）：（store_view） and 
/// 1 （store_crud） use ，「」 is 1 for 。
/// off （ and `apply_patch` off 1 ）。
pub fn default_low_balance(provider: &str) -> Value {
    if default_low_balance_mode(provider) == "off" {
        serde_json::json!({ "mode": "off", "threshold": 0.0 })
    } else {
        serde_json::json!({ "mode": "skip", "threshold": DEFAULT_LOW_BALANCE_THRESHOLD })
    }
}

/// config （seconds）。
///
/// not configured → （1 minutes）； `{enabled:false}` → `None`（ use 
/// must）； / （DB ）→ by ，
/// not as 1 。
pub fn query_interval_of(account: &Value) -> Option<i64> {
    let Some(config) = account.get("usageQuery") else {
        return Some(DEFAULT_QUERY_INTERVAL_SECONDS);
    };
    if config.get("enabled").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    match config.get("interval").and_then(Value::as_i64) {
        Some(interval)
            if (MIN_QUERY_INTERVAL_SECONDS..=MAX_QUERY_INTERVAL_SECONDS).contains(&interval) =>
        {
            Some(interval)
        }
        _ => Some(DEFAULT_QUERY_INTERVAL_SECONDS),
    }
}

/// 「 not use 」 。**limit valid then **（`core::limiter`）：
/// up has `limiters` when by its in disable then ； no has when by old 
/// `lowBalance` / provider —— and before 1 。**（ as empty）
/// not enabled use **： use is not ，must is use will 。
/// （ / has ） in then already ， this not 。
pub fn low_balance_disable_threshold(account: &Value) -> Option<f64> {
    crate::server::core::limiter::balance_disable_threshold(account)
}

/// is 「 not 」 in when （）。
///
/// **limit valid then **（`core::limiter::balance_skip_blocked`）： 1 
/// enabled skip then 「****（ etc available， and 
/// OmniProxy ）」。（none `limiters` ） by old config：Cline 
/// not （free 0 is ， will ）， its skip、 1。
/// not —— none、unlimited、 already —— 1 ：
/// is for 「 this no 」 ， not not can request。
pub fn balance_blocked(account: &Value, facts: &HashMap<String, BalanceFact>) -> bool {
    crate::server::core::limiter::balance_skip_blocked(account, facts)
}

// ─── old kv 1 ─────────────────────────────────

/// when （kv `usageQuerySnapshot`） after delete this 。
///
/// Why in when ： after not can empty —— old （failure）
/// is 「 that 」， after by 「 up 」， to 
/// down 1 to new 。
///
/// ：、、 in 1 ， in then 、 down 
/// 。failure verbose（ old in ， no ）， not 。
fn migrate_legacy_snapshot() {
    let Some(db) = database() else { return };
    let imported = db.with_mut(|conn| -> Result<usize, String> {
        let text: Option<String> = conn
            .query_row(
                "SELECT value FROM kv WHERE key = ?1",
                [LEGACY_SNAPSHOT_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let Some(text) = text else { return Ok(0) };
        let snapshot: Value = serde_json::from_str(&text)
            .map_err(|error| format!("旧余额快照格式错误: {error}"))?;
        let at = snapshot.get("at").and_then(Value::as_i64).unwrap_or(0);
        let rows = snapshot
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut imported = 0usize;
        for row in &rows {
            let Some(id) = row.get("id").and_then(Value::as_str) else { continue };
            if id.is_empty() {
                continue;
            }
            let usage = row.get("usage").filter(|value| !value.is_null()).cloned();
            match usage {
                Some(usage) => {
                    let (remaining, unlimited) = extract_remaining(&usage);
                    let usage_text = match serde_json::to_string(&usage) {
                        Ok(text) => text,
                        Err(_) => continue,
                    };
                    conn.execute(
                        "INSERT INTO account_usage_records
                           (account_id, usage, error, code, remaining, unlimited,
                            last_success_at, last_attempt_at, updated_at)
                         VALUES (?1, ?2, NULL, NULL, ?3, ?4, ?5, ?5, ?5)
                         ON CONFLICT(account_id) DO UPDATE SET
                           usage = excluded.usage,
                           error = NULL,
                           code = NULL,
                           remaining = excluded.remaining,
                           unlimited = excluded.unlimited,
                           last_success_at = excluded.last_success_at,
                           last_attempt_at = excluded.last_attempt_at,
                           updated_at = excluded.updated_at",
                        params![id, usage_text, remaining, unlimited as i64, at],
                    )
                    .map_err(|error| error.to_string())?;
                }
                None => {
                    let error = row
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("查询失败")
                        .to_string();
                    let code = row.get("code").and_then(Value::as_str);
                    conn.execute(
                        "INSERT INTO account_usage_records
                           (account_id, error, code, last_attempt_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?4)
                         ON CONFLICT(account_id) DO UPDATE SET
                           error = excluded.error,
                           code = excluded.code,
                           last_attempt_at = excluded.last_attempt_at,
                           updated_at = excluded.updated_at",
                        params![id, error, code, at],
                    )
                    .map_err(|error| error.to_string())?;
                }
            }
            imported += 1;
        }
        conn.execute("DELETE FROM kv WHERE key = ?1", [LEGACY_SNAPSHOT_KEY])
            .map_err(|error| error.to_string())?;
        Ok(imported)
    });
    match imported {
        Some(Ok(0)) => { /* no has old （ new or already ）：none can */ }
        Some(Ok(count)) => {
            logging::log("[Usage]", &format!("已把旧版余额快照的 {count} 条结果迁入新记录表"));
        }
        other => {
            logging::verbose(
                "[Usage]",
                &format!("旧余额快照迁移未完成（下次启动重试）：{other:?}"),
            );
        }
    }
}

/// when success（ ）。
///
/// down after will 1 （ ，must）—— 
/// is `pub(crate)` not is has ：`cloud_sync::download_locked` is 1 use 。
pub(crate) fn reload_facts() {
    let records = load_all();
    crate::server::core::account_order::reload_usage(&records);
    let facts: HashMap<String, BalanceFact> = records
        .into_iter()
        .filter(|(_, record)| record.usage.is_some())
        .filter_map(|(id, record)| {
            record
                .remaining
                .map(|remaining| (id, BalanceFact { remaining: Some(remaining) }))
        })
        .collect();
    if let Ok(mut slot) = facts_slot().lock() {
        *slot = facts;
    }
}

