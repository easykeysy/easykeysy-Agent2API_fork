//! to ****（`checkin_records` ，schema v11）。
//!
//! ── ────────────────────────────────────────────
//! before 「 to to to 」 has ：config.json `lastResult`
//! （ 1 ， down 1 ） and （、）。 to in 
//! 「 to 」 need is ** can **： 、、、
//! success is no 、failureerror is 、（）—— this is that 。
//!
//! ── （source） ─────────────────────────────────
//! `schedule` when / `makeup` when / `startup` /
//! `manual` and / `single` to / `activity`
//! and 。 before from `run_checkin` 1 
//! （`source` parameter），`activity` from `api::billing::run_activity`
//! （ that path not run_checkin， ， `record_activity`）。
//!
//! 「 to 」 （ [`Query::clause`]）： =
//! `manual` / `single` / `activity`， when = `schedule`， when = `makeup` /
//! `startup`；`makeup` this 1 up before no has ， by reason 
//! 「」 before 1 （ that ）。
//!
//! ── 1 ────────────────────────────────────────
//! outcome use to 1 for 
//! （`billing::checkin::{checkin_completed_today, claim_pending}`）——
//! 「 already 」「 etc 」 in and （`auto_checkin::tick`）
//! must 1 ， 1 。
//!
//! ── failure not ────────────────────────────────────────────
//! in to responsepath up ， up already to —— 1 not this that to 
//! （ and `run_checkin` `mark_checkin` ）。failure 1 
//! **verbose** 。
//!
//! ── ：（） not ──────────────────────
//! and `core::usage_records` 1 ：`Db::with*` is 1 ，
//! `logging::log` need 1 `logs` —— will 。

use std::sync::OnceLock;

use rusqlite::params;
use serde_json::{json, Value};

use crate::server::core::beijing;
use crate::server::core::billing::checkin::{
    activity_report_ok, checkin_completed_today, claim_pending, web_conversation_ok,
};
use crate::server::db::Db;
use crate::server::logging;

static DB: OnceLock<Option<Db>> = OnceLock::new();

/// （`outcome` ）
pub const OUTCOME_SUCCESS: &str = "success";
pub const OUTCOME_ALREADY: &str = "already";
pub const OUTCOME_PENDING: &str = "pending";
pub const OUTCOME_CAPTCHA: &str = "captcha";
pub const OUTCOME_FAILED: &str = "failed";

pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn database() -> Option<&'static Db> {
    DB.get().and_then(Option::as_ref)
}

// ─── ────────────────────────────────────────────────────

/// 「」： for （`pokeSucceeded`） or will 1 
/// —— （ will 「」， for 「 has 
/// use 」）， 1 `kept_active`， not 。
///
/// `activity_report_ok` / `web_conversation_ok` in `billing::checkin`
/// （ and `day_completed` ， that ），。
fn kept_active_of(row: &Value) -> bool {
    row.get("activity")
        .and_then(|activity| activity.get("pokeSucceeded"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || web_conversation_ok(row)
}

/// to （`run_checkin` ，`{id, name, claim, activity?,
/// error}` ）。`provider` by use from up （ no has this ，
/// need use ）。
///
/// outcome （ 1 ，module header）：
/// - `error` empty → **failed**（： / / ）；
/// - claim 「 already 」→ success（ to ） or already（ up already ）；
/// - need need → **captcha**（ not is none， not already ）；
/// - claim 「 in 」→ **pending**（ no has can ， etc ）；
/// - daily「 up success（ or will ）」→
/// **already** —— no is up （ not to ）， already 
/// this 1 （ / will ）， by up ；
/// failed will 1 （ before ）。 and 
/// `billing::checkin::day_completed` 。
/// - its → **failed**，error `error`， claim `msg`
/// （claim.success=false and failure，「expired」）。
pub fn record_row(row: &Value, provider: &str, source: &str, reason: &str) {
    let claim = row.get("claim").cloned().unwrap_or(Value::Null);
    let row_error = row
        .get("error")
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string);
    let (outcome, error) = if let Some(text) = row_error {
        (OUTCOME_FAILED, Some(text))
    } else if checkin_completed_today(&claim) {
        let outcome = if claim.get("success").and_then(Value::as_bool) == Some(true) {
            OUTCOME_SUCCESS
        } else {
            OUTCOME_ALREADY
        };
        (outcome, None)
    } else if activity_report_ok(row) || web_conversation_ok(row) {
        (OUTCOME_ALREADY, None)
    } else if claim.get("outcome").and_then(Value::as_str) == Some("captcha") {
        (OUTCOME_CAPTCHA, None)
    } else if claim_pending(&claim) {
        (OUTCOME_PENDING, None)
    } else {
        // failure and up no has error：failure in claim.msg （claim_result 
        // Err already message error， this is to 
        // success=false claim ， workbuddy-intl）
        let message = claim
            .get("msg")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .map(str::to_string);
        (OUTCOME_FAILED, message)
    };
    let kept_active = kept_active_of(row);
    insert(Entry {
        account_id: text_or(row.get("id"), ""),
        account_name: text_or(row.get("name"), &text_or(row.get("id"), "")),
        provider: provider.to_string(),
        source,
        reason,
        outcome,
        ok: outcome == OUTCOME_SUCCESS || outcome == OUTCOME_ALREADY,
        message: claim
            .get("msg")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(str::to_string),
        error,
        kept_active,
    });
}

/// / （`api::billing::run_activity` return，
/// `{id, name, mode, claim, activity}`）。
///
/// by mode ：`keepalive` 「 up / for / will 」 1 success
/// —— claim is `{success:false, msg:""}` ， claim
/// will failed， this 1 by is 「」，
/// 1 ；`full` / `claim` and to （claim），
/// 「 up success（ or will ）」 by `record_row` to already。
/// reason use （ 「/」）。
pub fn record_activity(row: &Value, provider: &str, mode: &str) {
    let reason = match mode {
        "keepalive" => "手动保活",
        "claim" => "手动领取活动",
        _ => "手动活动",
    };
    let claim = row.get("claim").cloned().unwrap_or(Value::Null);
    let kept_active = kept_active_of(row);
    let report_ok = activity_report_ok(row) || web_conversation_ok(row);
    let (outcome, error, ok) = if mode == "keepalive" {
        match kept_active || report_ok {
            true => (OUTCOME_SUCCESS, None, true),
            false => (
                OUTCOME_FAILED,
                Some(
                    claim
                        .get("msg")
                        .and_then(Value::as_str)
                        .unwrap_or("活跃保活失败")
                        .to_string(),
                ),
                false,
            ),
        }
    } else if checkin_completed_today(&claim) {
        let outcome = if claim.get("success").and_then(Value::as_bool) == Some(true) {
            OUTCOME_SUCCESS
        } else {
            OUTCOME_ALREADY
        };
        (outcome, None, true)
    } else if report_ok {
        (OUTCOME_ALREADY, None, true)
    } else if claim.get("outcome").and_then(Value::as_str) == Some("captcha") {
        (OUTCOME_CAPTCHA, None, false)
    } else if claim_pending(&claim) {
        (OUTCOME_PENDING, None, false)
    } else {
        (
            OUTCOME_FAILED,
            claim
                .get("msg")
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
                .map(str::to_string),
            false,
        )
    };
    insert(Entry {
        account_id: text_or(row.get("id"), ""),
        account_name: text_or(row.get("name"), &text_or(row.get("id"), "")),
        provider: provider.to_string(),
        source: "activity",
        reason,
        outcome,
        ok,
        message: claim
            .get("msg")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .map(str::to_string),
        error,
        kept_active,
    });
}

/// 1 （`insert` ； and 1 1 for ）
struct Entry<'a> {
    account_id: String,
    account_name: String,
    provider: String,
    source: &'a str,
    reason: &'a str,
    outcome: &'a str,
    ok: bool,
    message: Option<String>,
    error: Option<String>,
    kept_active: bool,
}

/// 1 （ 1 ）。failure verbose， not to up 
/// （module header「failure not 」）。
///
/// ****： not by 、 no has limit（ and request 1 to ）——
/// need use to 「empty / by delete」（`clear_all` /
/// `clear_where`）。 this will by 「request」，
/// that already request 1 。
fn insert(entry: Entry<'_>) {
    let Some(db) = database() else { return };
    let ts = logging::now_ms();
    let date = beijing::today_key();
    let failed = db
        .with_mut(|conn| -> Result<(), rusqlite::Error> {
            conn.execute(
                "INSERT INTO checkin_records
                   (ts, date, source, reason, account_id, account_name, provider,
                    outcome, ok, message, error, kept_active)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    ts,
                    date,
                    entry.source,
                    entry.reason,
                    entry.account_id,
                    entry.account_name,
                    entry.provider,
                    entry.outcome,
                    entry.ok as i64,
                    entry.message,
                    entry.error,
                    entry.kept_active as i64,
                ],
            )?;
            Ok(())
        })
        .map(|result| result.is_err())
        .unwrap_or(true);
    if failed {
        logging::verbose("[Checkin]", "签到记录未能落库（库不可用或写入失败）");
    }
}

/// （ to ）
fn text_or(value: Option<&Value>, fallback: &str) -> String {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

// ─── and delete ──────────────────────────────────────────────

/// （GET and DELETE use 1 ）
#[derive(Clone, Debug, Default)]
pub struct Query {
    pub outcome: Option<String>,
    /// to ：`manual` / `schedule` / `makeup`（empty = ）
    pub kind: Option<String>,
    pub provider: Option<String>,
    pub keyword: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl Query {
    /// is 1 （DELETE ： 1 no = ）
    pub fn has_filters(&self) -> bool {
        self.outcome.as_deref().is_some_and(|text| !text.is_empty())
            || self.kind.as_deref().is_some_and(|text| !text.is_empty())
            || self.provider.as_deref().is_some_and(|text| !text.is_empty())
            || self.keyword.as_deref().is_some_and(|text| !text.is_empty())
            || self.start.is_some()
            || self.end.is_some()
    }

    /// WHERE and parameter（ and delete use 1 —— 「」）
    fn clause(&self) -> (String, Vec<Value>) {
        let mut conditions: Vec<String> = Vec::new();
        let mut params_json: Vec<Value> = Vec::new();
        if let Some(outcome) = self.outcome.as_deref().filter(|text| !text.is_empty()) {
            conditions.push("outcome = ?".to_string());
            params_json.push(json!(outcome));
        }
        match self.kind.as_deref().map(str::trim).filter(|text| !text.is_empty()) {
            // to ： / 、 to 、 
            // and —— is in up 
            Some("manual") => conditions.push("source IN ('manual', 'single', 'activity')".to_string()),
            // when to ： when and 。 in `makeup` up 
            // before schedule， by reason 「」 before excluding（ that 
            // reason is 「（）」）； up after new not will in 
            // this 
            Some("schedule") => conditions.push(
                "source = 'schedule' AND reason NOT LIKE '补签%'".to_string(),
            ),
            // when ：（`makeup`） and old （`startup`），
            // by old schedule 
            Some("makeup") => conditions.push(
                "(source IN ('makeup', 'startup')
                  OR (source = 'schedule' AND reason LIKE '补签%'))"
                    .to_string(),
            ),
            _ => {}
        }
        if let Some(provider) = self.provider.as_deref().filter(|text| !text.is_empty()) {
            conditions.push("provider = ?".to_string());
            params_json.push(json!(provider));
        }
        if let Some(keyword) = self.keyword.as_deref().filter(|text| !text.is_empty()) {
            // / / error / Reason can 
            let pattern = format!("%{}%", keyword.replace('%', "\\%").replace('_', "\\_"));
            conditions.push(
                "(account_name LIKE ? ESCAPE '\\'
                  OR message LIKE ? ESCAPE '\\'
                  OR error LIKE ? ESCAPE '\\'
                  OR reason LIKE ? ESCAPE '\\')"
                    .to_string(),
            );
            for _ in 0..4 {
                params_json.push(json!(pattern));
            }
        }
        if let Some(start) = self.start {
            conditions.push("ts >= ?".to_string());
            params_json.push(json!(start));
        }
        if let Some(end) = self.end {
            conditions.push("ts <= ?".to_string());
            params_json.push(json!(end));
        }
        if conditions.is_empty() {
            (String::new(), params_json)
        } else {
            (format!(" WHERE {}", conditions.join(" AND ")), params_json)
        }
    }
}

/// 1 （`offset` ，`limit` 200、limit 1000）。
/// response `{records: [...], total}`：total is **** total（ use ），
/// records by when （ new in before ）。unavailable to empty（empty， not ）。
pub fn query(query: &Query, limit: usize, offset: usize) -> Value {
    let limit = limit.clamp(1, 1000);
    let Some(db) = database() else {
        return json!({ "records": [], "total": 0 });
    };
    let (where_clause, filter_params) = query.clause();
    let sql_rows = format!(
        "SELECT row_id, ts, date, source, reason, account_id, account_name,
                provider, outcome, ok, message, error, kept_active
         FROM checkin_records{where_clause}
         ORDER BY ts DESC, row_id DESC
         LIMIT {limit} OFFSET {offset}"
    );
    let sql_count = format!("SELECT COUNT(*) FROM checkin_records{where_clause}");
    let loaded = db
        .with(|conn| -> Result<Value, rusqlite::Error> {
            let params_from_json = |values: &[Value]| -> Vec<String> {
                values.iter().map(value_to_text).collect()
            };
            let total: i64 = conn.query_row(
                &sql_count,
                rusqlite::params_from_iter(params_from_json(&filter_params)),
                |row| row.get(0),
            )?;
            let mut statement = conn.prepare(&sql_rows)?;
            let rows = statement.query_map(
                rusqlite::params_from_iter(params_from_json(&filter_params)),
                |row| {
                    Ok(json!({
                        "rowId": row.get::<_, i64>("row_id")?,
                        "ts": row.get::<_, i64>("ts")?,
                        "date": row.get::<_, String>("date")?,
                        "source": row.get::<_, String>("source")?,
                        "reason": row.get::<_, String>("reason")?,
                        "accountId": row.get::<_, String>("account_id")?,
                        "accountName": row.get::<_, String>("account_name")?,
                        // is provider id，（`qoder-intl`
                        // → 「Qoder 」）： up this 1 ， id
                        // has 。 —— in ，
                        // not （ need ， and old need ）。
                        "provider": crate::server::core::providers::label_of(
                            &row.get::<_, String>("provider")?
                        ),
                        "providerId": row.get::<_, String>("provider")?,
                        "outcome": row.get::<_, String>("outcome")?,
                        "ok": row.get::<_, i64>("ok").unwrap_or(0) != 0,
                        "message": row.get::<_, Option<String>>("message")?,
                        "error": row.get::<_, Option<String>>("error")?,
                        "keptActive": row.get::<_, i64>("kept_active").unwrap_or(0) != 0,
                    }))
                },
            )?;
            let records: Vec<Value> = rows.filter_map(|item| item.ok()).collect();
            Ok(json!({ "records": records, "total": total }))
        })
        .and_then(|result| result.ok())
        .unwrap_or_else(|| json!({ "records": [], "total": 0 }));
    loaded
}

/// by delete（「empty」）；none use **must** by API 
/// `all=1` （ logs then ， by api::checkin_records）。
/// returndelete；unavailable to None（API to 500 ）。
pub fn clear_where(query: &Query) -> Option<usize> {
    let db = database()?;
    let (where_clause, filter_params) = query.clause();
    let sql = format!("DELETE FROM checkin_records{where_clause}");
    let params_from_json = |values: &[Value]| -> Vec<String> {
        values.iter().map(value_to_text).collect()
    };
    db.with_mut(|conn| {
        conn.execute(&sql, rusqlite::params_from_iter(params_from_json(&filter_params)))
            .ok()
    })
    .flatten()
}

/// empty（API already ）。returndelete。
pub fn clear_all() -> Option<usize> {
    let db = database()?;
    db.with_mut(|conn| conn.execute("DELETE FROM checkin_records", []).ok()).flatten()
}

/// rusqlite parameter is ToSql： this 1 （ / ）
fn value_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
