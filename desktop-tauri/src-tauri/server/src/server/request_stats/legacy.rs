//! old file： to `db::migrate` use and 。
//!
//! ── Why this in not is ────────────────────
//! is 「 old file、 new 」 that 1 ， ，：
//! 1. ** is 1 **：`parse_requests_jsonl` /
//! `parse_daily_jsonl` must and `record.rs` serde ** use 1 **
//! —— 「file in 、 when 、」；
//! 2. ** is then 1 **： use `insert_request` /
//! `upsert_daily` and 1 ；** after not ** ——
//! requeststatistics（ no has 、 no has limit）， old file 
//! has ， and 「 not 」 to 1 ；
//! 3. ** need need `backfill` and `fold_into_daily`**（ is has / has ），
//! not to 。
//! this ****（`parse_*` / `legacy_*` / `import_legacy_*`），
//! to and 。
//!
//! ── this in when use ─────────────────────────────
//! is `pub(crate)`： 1 use is `db::migrate::requests` 。
//! 1 not will to （ already has 、 old file already ）。
//! use `pub(crate)` not is `pub(super)` is as in `db::migrate` ，
//! and `request_stats` not is 。

use std::collections::BTreeMap;

use super::record::{DailyEntry, RequestEntry};
use super::report::{push_account_accum, push_model_accum, push_provider_accum};
use super::daily;
use super::{backfill, sql};

/// old file（`requests.jsonl`），。
///
/// ** and old `load_requests` 1 **： trim、empty、failure、
/// `#[serde(default)]`（`id` empty、`attempts` 1、token 0…）。
/// this in not is ： is **** 1 
/// （`record.rs` serde is ），「file in 、 when 」。
pub(crate) fn parse_requests_jsonl(text: &str) -> Vec<RequestEntry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(item) = serde_json::from_str::<RequestEntry>(trimmed) {
            out.push(item);
        }
    }
    out
}

/// old file（`request-daily.jsonl`），。
///
/// ** and old `load_daily` 1 **，including that not need then ：
/// 1 （ and file、 down ） when by ** and **，
/// not is 「 after before 」—— will before 1 。 and 
/// `push_*_accum`， and total 1 up ， not will 
/// 「total、 1 」 this for not up 。
/// as empty （ not by ）。
pub(crate) fn parse_daily_jsonl(text: &str) -> BTreeMap<String, DailyEntry> {
    let mut out: BTreeMap<String, DailyEntry> = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(item) = serde_json::from_str::<DailyEntry>(trimmed) else {
            continue;
        };
        if item.date.is_empty() {
            continue;
        }
        match out.get_mut(&item.date) {
            Some(existing) => {
                existing.requests += item.requests;
                existing.successful += item.successful;
                existing.tokens += item.tokens;
                existing.cache_hit_tokens += item.cache_hit_tokens;
                existing.cache_input_tokens += item.cache_input_tokens;
                for acc in item.model_tokens {
                    push_model_accum(&mut existing.model_tokens, &acc.model, acc.tokens, acc.requests);
                }
                // provider and （ 1 when need up ，
                // otherwise「 by provider and = requests」 this for will ）
                for acc in item.provider_stats {
                    push_provider_accum(
                        &mut existing.provider_stats,
                        &acc.provider,
                        acc.requests,
                        acc.successful,
                        acc.tokens,
                    );
                }
                // （ by and provider that 1 ： and need not 、 need not not ）
                for acc in item.account_stats {
                    push_account_accum(
                        &mut existing.account_stats,
                        &acc.account_id,
                        &acc.account_name,
                        acc.requests,
                        acc.successful,
                        acc.tokens,
                    );
                }
            }
            None => {
                out.insert(item.date.clone(), item);
            }
        }
    }
    out
}

// this has 1 `legacy_bounds`（ to ）。** already delete**：
// requeststatistics not has and ， after not need need —— 、config
// and that 「 N = in N 」 1 and in by 。

// this has 1 `legacy_counts`（return and ， to 「empty
// 」 etc ）。** already delete**：T11 use 「」 after ，
// that not —— in before can can （ use request），
// is 「 already 」， old file not 
// （`logs` already up ）。 in + etc 
// （ `MARKER_REQUESTS`）， not need need 「empty not empty」 this 。

/// （`kv` ）。
///
/// ── Why this need need ， `logs` / `accounts` / `debug` not need need ──
/// that is 「 this is 」：`logs.id` is 、`accounts.id` is 
/// uuid、`debug_traffic.id` is request id —— is 「 already in 」 can by 
/// （`INSERT OR IGNORE`）。 `requests` / `request_daily` no has available 
/// ：
/// - `requests` is `row_id INTEGER PRIMARY KEY AUTOINCREMENT`（，
/// and none）， `id` is **request id、 old can can is empty**（
/// `record.rs` ）， not 1 ；
/// - `request_daily` `date` is ，** will UPSERT **
/// （ new request will 1 ）—— use 「date already in 」「」，
/// will 「 already request」「 old already 」。
/// this use 。 is ， as ** and in 1 **
/// （ `db::migrate::config` module header： that for by for is 
/// **** ； ⇒ and in ， ⇒ not in ，
/// 「 1 」 this in in up not in ）。
pub(crate) const MARKER_REQUESTS: &str = "requestsMigrated";
/// （ by [`MARKER_REQUESTS`]）
pub(crate) const MARKER_DAILY: &str = "dailyMigrated";

/// in not in （ etc ）。
pub(crate) fn marker_present(conn: &rusqlite::Connection, key: &str) -> rusqlite::Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM kv WHERE key = ?1 LIMIT 1",
            rusqlite::params![key],
            |row| row.get(0),
        )
        .ok();
    Ok(found.is_some())
}

/// （**must in **， [`MARKER_REQUESTS`] ）。
fn write_marker(conn: &rusqlite::Connection, key: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, 'true')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key],
    )?;
    Ok(())
}

/// use ：[`backfill`] （ in ， that and 
/// in `backfill::report_line` ； this 1 ，
/// as `backfill` is has ， not to ）。
pub(crate) fn backfill_report_line(changed: &[String]) -> String {
    backfill::report_line(changed)
}

/// use ： old `requests` （** 1 **）。
///
/// ── after Why not 1 ──────────────────────────────────
/// old in **** when to ， up file down 1 ——
/// old file、 is 。 that is 「 + 」 when ；
/// in requeststatistics****（ / / not by or ），
/// old file has —— and 「 not 」 to 1 ，
/// use can in requestempty。
///
/// use `unchecked_transaction` not is `transaction`： to is 
/// `&Connection`（`transaction()` need `&mut`）， use `Db::with` has 1 、
/// no has —— that 「 not 」 by use 
/// （ and `db::migrate` its 1 ）。
///
/// ── etc ： and ****（T11 after not can 「empty」）──
/// will （ use 「」 before can can already request），「empty
/// not empty」 will old file not （`logs` already up this ）。
/// use [`MARKER_REQUESTS`]，** in 1 ** and 1 ：
/// - ⇒ + in ， when `marker_present` in → ；
/// - ⇒ not in ， down 。
/// INSERT in this is （`requests` no has available 1 
/// `INSERT OR IGNORE`， by [`MARKER_REQUESTS`] ）。
pub(crate) fn import_legacy_requests(
    conn: &rusqlite::Connection,
    entries: &[RequestEntry],
) -> rusqlite::Result<usize> {
    let tx = conn.unchecked_transaction()?;
    for entry in entries {
        sql::insert_request(&tx, entry)?;
    }
    // and （ etc ）
    write_marker(&tx, MARKER_REQUESTS)?;
    tx.commit()?;
    Ok(entries.len())
}

/// use ： old `request_daily` ， 1 ****
/// （** 1 **，）。
///
/// return（ , ）。 use after 1 ——
/// that and use to can can not ， when need has 。
///
/// ── Why in this not is ────────────────────────────
/// `backfill` module header：`accountStats` is after ， old no has ，
/// （ ）， that will 「unknown」。
/// need need 「 + 」 when in ， this ** can can in 
/// old this 1 **： new （`fold_into_daily`
/// ）， not will 「」。 is ** 1 **，
/// all； not has use （ old `load()` that already ）。
/// 「 >= requests 」 in `backfill` ——
/// already no has ， in unknown。
///
/// ── etc ： and ****（ by [`MARKER_DAILY`]）──────
/// will （ use 「」 before request → that already 
/// UPSERT ），「empty」 will old not 。
/// use [`MARKER_DAILY`]， in ** 1 ** and 1 。
///
/// Note `upsert_daily` in this is ** need **（ not is 「 use 」）： old 
/// can can and already ， UPSERT is 「 use old file that 
/// 」—— this **** will already ， that is need ：
/// will use （ ） this ， 
/// by 1 （`rebuild_legacy_days` 「 ≥ 
/// requests」 ， not will ）。
pub(crate) fn import_legacy_daily(
    conn: &rusqlite::Connection,
    daily: &BTreeMap<String, DailyEntry>,
) -> rusqlite::Result<(usize, Vec<String>)> {
    let tx = conn.unchecked_transaction()?;
    for day in daily.values() {
        daily::upsert_daily(&tx, day)?;
    }
    let mut stored = daily::select_daily_map(&tx)?;
    let entries = match (sql::min_ts(&tx)?, sql::max_ts(&tx)?) {
        // in this when —— need 「 has 」，
        // will need need （ that is backfill ），
        // 1 ； 1 can 。
        // this 1 is 「 old 」（ ）， and when ：
        // excluding（ `select_between` and `RequestEntry::is_test`）
        (Some(min), Some(max)) => sql::select_between(&tx, min, max, true)?,
        _ => Vec::new(),
    };
    let changed = backfill::rebuild_legacy_days(&mut stored, &entries);
    for key in &changed {
        if let Some(day) = stored.get(key) {
            daily::upsert_daily(&tx, day)?;
        }
    }
    // and （ etc ）
    write_marker(&tx, MARKER_DAILY)?;
    tx.commit()?;
    Ok((daily.len(), changed))
}
