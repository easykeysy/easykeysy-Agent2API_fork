//! `request_daily` （ by ） ** SQL** —— from `sql.rs` 1 。
//!
//! ── Whyfile ────────────────────────────────────────
//! `sql.rs` when （`requests`） and （`request_daily`） ，
//! 「 in 」 and 「」 after 800。 that 1 ：
//! 1 、 1 for 、 1 --， and 1 is 
//! 「 in `Db` that 」。 after `sql.rs` ，file ——
//! 「 by `fold_into_daily` 」 this not 
//! in this 1 file（ can as `pub(super)`， use and before ）。
//!
//! ── JSON Why ──────────────────────────────
//! `model_tokens` / `provider_stats` / `account_stats` is JSON ，
//! must**、 in Rust 、**（ `sql.rs` module header 
//! 「 --」）：total and is 1 ， UPDATE 
//! total will 「 and = total」。 1 has 1 、
//! ， 1 -- can 。

use std::collections::BTreeMap;

use rusqlite::{params, Connection};

use super::record::{AccountAccum, DailyEntry, ModelAccum, ProviderAccum};
use super::source::{ApiKeyAccum, ClientAccum};

/// `request_daily` （all SELECT by this ，`decode_daily`
/// ； not `SELECT *` by `sql::REQUEST_COLUMNS`）
const DAILY_COLUMNS: &str = "date, requests, successful, tokens, cache_hit_tokens, \
     cache_input_tokens, model_tokens, provider_stats, account_stats, client_stats, api_key_stats, \
     source_stats_version";

/// JSON → `Vec<T>`。failure or not is when empty。
///
/// and old file when to 1 （、）： this is ****
/// ， 1 not this not —— empty is that 1 1 ，
/// total in 。
fn decode_accum<T: serde::de::DeserializeOwned>(text: &str) -> Vec<T> {
    serde_json::from_str(text).unwrap_or_default()
}

/// JSON ← `Vec<T>`。failure to `'[]'`（ and DDL default 1 ，
/// is 「 this 1 no has 」 in has 1 ）。
fn encode_accum<T: serde::Serialize>(items: &[T]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}

/// → `DailyEntry`
fn decode_daily(row: &rusqlite::Row<'_>) -> rusqlite::Result<DailyEntry> {
    let model_tokens: String = row.get(6)?;
    let provider_stats: String = row.get(7)?;
    let account_stats: String = row.get(8)?;
    Ok(DailyEntry {
        date: row.get(0)?,
        requests: row.get(1)?,
        successful: row.get(2)?,
        tokens: row.get(3)?,
        cache_hit_tokens: row.get(4)?,
        cache_input_tokens: row.get(5)?,
        model_tokens: decode_accum::<ModelAccum>(&model_tokens),
        provider_stats: decode_accum::<ProviderAccum>(&provider_stats),
        account_stats: decode_accum::<AccountAccum>(&account_stats),
        client_stats: decode_accum::<ClientAccum>(&row.get::<_, String>(9)?),
        api_key_stats: decode_accum::<ApiKeyAccum>(&row.get::<_, String>(10)?),
        source_stats_version: row.get(11)?,
    })
}

/// → `BTreeMap`（ need is this ）。
///
/// 「」 this in this ： statistics this ，
/// after not will empty。
pub(super) fn select_daily_map(conn: &Connection) -> rusqlite::Result<BTreeMap<String, DailyEntry>> {
    let sql = format!("SELECT {DAILY_COLUMNS} FROM request_daily ORDER BY date ASC");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut out = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let day = decode_daily(row)?;
        out.insert(day.date.clone(), day);
    }
    Ok(out)
}

/// 1 （`record` -- 「」）
pub(super) fn select_daily_row(
    conn: &Connection,
    date: &str,
) -> rusqlite::Result<Option<DailyEntry>> {
    let sql = format!("SELECT {DAILY_COLUMNS} FROM request_daily WHERE date = ?1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![date])?;
    match rows.next()? {
        Some(row) => Ok(Some(decode_daily(row)?)),
        None => Ok(None),
    }
}

/// （ in ）。
///
/// `date` is ，「 1 」 is not is ——
/// this is by （ `schema.rs`）。 JSON by 
/// [`encode_accum`] ，total `DailyEntry` 。
pub(super) fn upsert_daily(conn: &Connection, day: &DailyEntry) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO request_daily (date, requests, successful, tokens, cache_hit_tokens, \
         cache_input_tokens, model_tokens, provider_stats, account_stats, client_stats, api_key_stats, \
         source_stats_version) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12) \
         ON CONFLICT(date) DO UPDATE SET requests = excluded.requests, \
         successful = excluded.successful, tokens = excluded.tokens, \
         cache_hit_tokens = excluded.cache_hit_tokens, \
         cache_input_tokens = excluded.cache_input_tokens, \
         model_tokens = excluded.model_tokens, provider_stats = excluded.provider_stats, \
         account_stats = excluded.account_stats, client_stats = excluded.client_stats, \
         api_key_stats = excluded.api_key_stats, source_stats_version = excluded.source_stats_version",
        params![
            day.date,
            day.requests,
            day.successful,
            day.tokens,
            day.cache_hit_tokens,
            day.cache_input_tokens,
            encode_accum(&day.model_tokens),
            encode_accum(&day.provider_stats),
            encode_accum(&day.account_stats),
            encode_accum(&day.client_stats),
            encode_accum(&day.api_key_stats),
            day.source_stats_version,
        ],
    )?;
    Ok(())
}

/// 1 （ after that 1 not ）
pub(super) fn delete_daily(conn: &Connection, date: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM request_daily WHERE date = ?1", params![date])
}

/// `date < cutoff_key` （ when ； when ）。
/// use ：`record` and `prune` （ by use ）。
pub(super) fn delete_daily_before(conn: &Connection, cutoff_key: &str) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM request_daily WHERE date < ?1", params![cutoff_key])
}

/// （ `dailyDays` ）
pub(super) fn count_daily(conn: &Connection) -> rusqlite::Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM request_daily", [], |row| row.get(0))?;
    Ok(count.max(0) as usize)
}

/// empty（`clear()` use ）
pub(super) fn delete_all_daily(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM request_daily", [])
}
