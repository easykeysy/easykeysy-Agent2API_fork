//! `debug_traffic` ** SQL ** —— 1 SQL 。
//!
//! ── Why 1 ──────────────────────────────────────────
//! before is 「 `VecDeque` + / file」， is 
//! 「 in 」。 after that ： is 1 UPSERT、
//! is 1 DELETE、 is 1 —— file this ， up 
//! （`debug_traffic.rs`）、、 and API 。
//!
//! ── `size` ────────────────────────────────────────────
//! after ， not SQLite and 。
//! by ， not by this delete。
//!
//! ── ：`ts DESC, rowid DESC` ────────────────────────────
//! 「 new 」 by **ts** （`idx_debug_traffic_ts` as in ）， ts when use `rowid`
//! 。 this and before 「 by （`VecDeque` push ）」 in 
//! **request** up will not ：`entry.ts` is **** when ， in response****
//! when —— 1 SSE（、） in old in 1 ， in this will 
//! by ts to 1 。 ts by ： is 「 this request」 when ，
//! is and use when ； by will 「 new that request」
//! 。/ has schema， not 。
//!
//! ── and ： not ────────────────────────────────────────
//! all `&Connection`， by `Db` that Mutex （ up in 
//! ** 1 ** `Db::with` / `with_mut` use ， use ）。
//! ****： this not can `logging::log` / `logging::verbose` ——
//! need 1 ，`std::sync::Mutex` not can ， will （all
//! not ，error 1 `Err` up ）。

use rusqlite::types::Value as SqlValue;
use rusqlite::{params, Connection};

use super::{TrafficEntry, MAX_ENTRIES};

/// （all SELECT by this ，[`decode_row`] ；
/// not `SELECT *` is as 「 when need up 」 this in diff can ）
const COLUMNS: &str = "id, ts, url, provider, request_headers, request_body, status, \
     response_headers, response_body, truncated, size";

/// 1 after （ `size` ，module header）
pub(super) fn payload_bytes(entry: &TrafficEntry) -> i64 {
    let headers = entry.request_headers.to_string().len();
    let body = entry.request_body.to_string().len();
    let response_headers = entry.response_headers.as_ref().map(|item| item.to_string().len()).unwrap_or(0);
    let response_body = entry.response_body.as_deref().map(str::len).unwrap_or(0);
    let identity = entry.id.len() + entry.url.len() + entry.provider.len();
    (headers + body + response_headers + response_body + identity) as i64
}

/// → `TrafficEntry`。
///
/// can empty（`status` / `response_headers` / `response_body`） `Option`
/// —— ** not NULL empty**： before this 「 this 1 no to 」 and 
/// 「 to as empty」（`ui/requests-panel.js` `renderDetail`： `null`
/// `-`，response as empty then not that 1 ）。（ [`encode`]）。
/// `request_headers` / `request_body` is `NOT NULL`， as down NULL，
/// this use `Option` default —— not this as 1 failure。
fn decode_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrafficEntry> {
    let headers: Option<String> = row.get(4)?;
    let body: Option<String> = row.get(5)?;
    let status: Option<i64> = row.get(6)?;
    let response_headers: Option<String> = row.get(7)?;
    let truncated: i64 = row.get(9)?;
    Ok(TrafficEntry {
        id: row.get(0)?,
        ts: row.get(1)?,
        url: row.get(2)?,
        provider: row.get(3)?,
        request_headers: headers
            .as_deref()
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or(serde_json::Value::Null),
        request_body: body
            .as_deref()
            .and_then(|text| serde_json::from_str(text).ok())
            .unwrap_or(serde_json::Value::Null),
        // （） by 「 no to 」：`u16` not down no has 
        status: status.and_then(|value| u16::try_from(value).ok()),
        response_headers: response_headers
            .as_deref()
            .and_then(|text| serde_json::from_str(text).ok()),
        response_body: row.get(8)?,
        truncated: truncated != 0,
    })
}

/// JSON → （failure to `'null'`／`'{}'`， and DDL default）
fn encode_json(value: &serde_json::Value, fallback: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| fallback.to_string())
}

/// use （`insert_entry` and `upsert_entry` use 1 ，path 
/// 1 —— and not this has ）
fn encode(entry: &TrafficEntry) -> [SqlValue; 11] {
    [
        SqlValue::Text(entry.id.clone()),
        SqlValue::Integer(entry.ts),
        SqlValue::Text(entry.url.clone()),
        SqlValue::Text(entry.provider.clone()),
        SqlValue::Text(encode_json(&entry.request_headers, "{}")),
        SqlValue::Text(encode_json(&entry.request_body, "null")),
        // can empty：None → NULL（** not empty**， `decode_row` ）
        match entry.status {
            Some(value) => SqlValue::Integer(i64::from(value)),
            None => SqlValue::Null,
        },
        match entry.response_headers.as_ref() {
            Some(value) => SqlValue::Text(encode_json(value, "null")),
            None => SqlValue::Null,
        },
        match entry.response_body.as_ref() {
            Some(value) => SqlValue::Text(value.clone()),
            None => SqlValue::Null,
        },
        SqlValue::Integer(i64::from(entry.truncated)),
        SqlValue::Integer(payload_bytes(entry)),
    ]
}

/// 1 （** INSERT**：）。
///
/// ⚠️ ** already not use **（ use [`insert_entry_ignore`]）：T11 
/// use 「」 after ，「」， INSERT will in when 
/// —— use to 「failure」，Reason is 「」。
/// is as 「 id = 」 this ** can can need need ** ；
/// currentnone use （`#[allow(dead_code)]` and `upsert_entry` ：
/// is this ， will 「」 this in ）。
#[allow(dead_code)]
pub(super) fn insert_entry(conn: &Connection, entry: &TrafficEntry) -> rusqlite::Result<()> {
    let binds = encode(entry);
    conn.execute(
        "INSERT INTO debug_traffic (id, ts, url, provider, request_headers, request_body, \
         status, response_headers, response_body, truncated, size) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params_from_iter(binds.iter()),
    )?;
    Ok(())
}

/// 1 ，** already in when **（ use ）。
///
/// return `true` = ，`false` = already in —— use statistics
/// 「、」（ when is `false`）。
///
/// and [`upsert_entry`] is ** when not that **： old file is ，
/// that can can update（path、 in after ）—— not this 
/// update 。
pub(super) fn insert_entry_ignore(
    conn: &Connection,
    entry: &TrafficEntry,
) -> rusqlite::Result<bool> {
    let binds = encode(entry);
    let changed = conn.execute(
        "INSERT OR IGNORE INTO debug_traffic (id, ts, url, provider, request_headers, \
         request_body, status, response_headers, response_body, truncated, size) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params_from_iter(binds.iter()),
    )?;
    Ok(changed > 0)
}

/// 1 （ id ）。
///
/// Why not is INSERT：`id` is ， 1 id is can 
/// （path 、 in 、 1 request path 1 ）。
/// before is 1 JSONL（ id ，`get` after that ）， this 1 ——
/// not （`get` to after that ）， not 。
pub(super) fn upsert_entry(conn: &Connection, entry: &TrafficEntry) -> rusqlite::Result<()> {
    let binds = encode(entry);
    conn.execute(
        "INSERT INTO debug_traffic (id, ts, url, provider, request_headers, request_body, \
         status, response_headers, response_body, truncated, size) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
         ON CONFLICT(id) DO UPDATE SET ts = excluded.ts, url = excluded.url, \
         provider = excluded.provider, request_headers = excluded.request_headers, \
         request_body = excluded.request_body, status = excluded.status, \
         response_headers = excluded.response_headers, response_body = excluded.response_body, \
         truncated = excluded.truncated, size = excluded.size",
        rusqlite::params_from_iter(binds.iter()),
    )?;
    Ok(())
}

/// ： and already 。
pub(super) fn count_and_bytes(conn: &Connection) -> rusqlite::Result<(usize, i64)> {
    let (count, bytes): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), COALESCE(SUM(size), 0) FROM debug_traffic",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok((count.max(0) as usize, bytes.max(0)))
}

/// by id 1 （ id when ts ， rowid —— and before 
/// 「 from before 1 」 etc ）
pub(super) fn select_one(conn: &Connection, id: &str) -> rusqlite::Result<Option<TrafficEntry>> {
    let sql = format!("SELECT {COLUMNS} FROM debug_traffic WHERE id = ?1 ORDER BY ts DESC, rowid DESC LIMIT 1");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some(decode_row(row)?)),
        None => Ok(None),
    }
}

/// use （fork： already file， this ）。
///
/// `request_headers` / `request_body` is `NOT NULL` ， use DDL default
/// （`'{}'` / `''`）；`response_headers` / `response_body` NULL —— 
/// use NULL 「 this 1 no to 」， in file when this 。
fn encode_ledger(entry: &TrafficEntry) -> [SqlValue; 11] {
    [
        SqlValue::Text(entry.id.clone()),
        SqlValue::Integer(entry.ts),
        SqlValue::Text(entry.url.clone()),
        SqlValue::Text(entry.provider.clone()),
        SqlValue::Text("{}".to_string()),
        SqlValue::Text(String::new()),
        match entry.status {
            Some(value) => SqlValue::Integer(i64::from(value)),
            None => SqlValue::Null,
        },
        SqlValue::Null,
        SqlValue::Null,
        SqlValue::Integer(i64::from(entry.truncated)),
        SqlValue::Integer(payload_bytes(entry)),
    ]
}

/// fork：（ id ），empty —— in 
/// `{cache_dir}/debug-traffic/{id}`（ `payload` ）。
pub(super) fn upsert_ledger(conn: &Connection, entry: &TrafficEntry) -> rusqlite::Result<()> {
    let binds = encode_ledger(entry);
    conn.execute(
        "INSERT INTO debug_traffic (id, ts, url, provider, request_headers, request_body, \
         status, response_headers, response_body, truncated, size) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
         ON CONFLICT(id) DO UPDATE SET ts = excluded.ts, url = excluded.url, \
         provider = excluded.provider, request_headers = excluded.request_headers, \
         request_body = excluded.request_body, status = excluded.status, \
         response_headers = excluded.response_headers, response_body = excluded.response_body, \
         truncated = excluded.truncated, size = excluded.size",
        rusqlite::params_from_iter(binds.iter()),
    )?;
    Ok(())
}

/// id（file when use 「」）。
pub(super) fn ids(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM debug_traffic")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|row| row.ok()).collect())
}

/// in （ before ）。
///
/// `request_headers` DDL default is `'{}'`（ 2），「 2 」 is 
/// has ； its use NULL / empty。
pub(super) fn payload_rows(conn: &Connection) -> rusqlite::Result<Vec<TrafficEntry>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM debug_traffic WHERE length(request_headers) > 2 \
         OR length(request_body) > 0 OR response_headers IS NOT NULL \
         OR response_body IS NOT NULL"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        entries.push(decode_row(row)?);
    }
    Ok(entries)
}

/// already empty（ in file，）。
pub(super) fn blank_payloads(conn: &Connection, ids: &[String]) -> rusqlite::Result<usize> {
    let mut changed = 0;
    for batch in ids.chunks(200) {
        let placeholders = vec!["?"; batch.len()].join(", ");
        changed += conn.execute(
            &format!(
                "UPDATE debug_traffic SET request_headers = '{{}}', request_body = '', \
                 response_headers = NULL, response_body = NULL WHERE id IN ({placeholders})"
            ),
            rusqlite::params_from_iter(batch.iter()),
        )?;
    }
    Ok(changed)
}

/// by old ， not delete；return 
/// （fork： use need not need file）。
pub(super) fn enforce_limits(conn: &Connection) -> rusqlite::Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM debug_traffic", [], |row| row.get(0))?;
    if count > MAX_ENTRIES as i64 {
        return conn.execute(
            "DELETE FROM debug_traffic WHERE id NOT IN \
             (SELECT id FROM debug_traffic ORDER BY ts DESC, rowid DESC LIMIT ?1)",
            params![MAX_ENTRIES as i64],
        );
    }
    Ok(0)
}
