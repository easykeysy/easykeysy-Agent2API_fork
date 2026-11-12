//! requeststatistics ** SQL ** —— 1 SQL 。
//!
//! ── Why 1 ──────────────────────────────────────────
//! before is 1 by ts `Vec`（`insert_sorted` ）， is 
//! 1 `BTreeMap<String, DailyEntry>`， file has （
//! 「 as + `COMPACT_STEP` file」，「 50 or 
//! up 60 seconds」）。 that 「 + 」 
//! after ： is 1 `INSERT`、 is 1 UPSERT， by 
//! `report.rs` —— file this ， up （`request_stats.rs`）
//! 1 、 and API 。
//!
//! ── `row_id` and `id` （`db/schema.rs` is ）──
//! - `row_id`：`INTEGER PRIMARY KEY AUTOINCREMENT`，****， use 。
//! AUTOINCREMENT ** not use **（ after new not will to 1 ）——
//! 1 use will / 。
//! - `id`：request （ and ），** can 、 can is empty**，
//! not is not and （ ts `row_id`）。
//! old 「 by ts 、ms in before 」 for `ORDER BY ts, row_id`；
//! 「 new in before 」 for `ORDER BY ts DESC, row_id DESC`（ ts when row_id is after 
//! that ， and 「」 1 ）。
//!
//! ── request_raw and 「 in 」（add ）────────────
//! - `request_raw`： down （ for ）， and requests id 。
//! （`upsert_raw`） by up in after ；**deletemust in 
//! requests 1 delete after / before **（delete ），
//! otherwise no in ， as not 。
//! - status=0 ： before by `insert_started_request` 「 in 」
//! ， when by `update_running_request` （ UPDATE after 
//! INSERT by `update_running_request`）。 this ** not **
//! （`fold_into_daily` by status=0 ）， and will `delete_stale_running`
//! by when （ down ）。
//!
//! ── Why can down to SQL ──────────────────────────
//! this no has 「 can in Rust 」 ：`model` is 、`status` is 
//! 、`start`/`end` is ， can WHERE。`query_requests` and 
//! and use 1 
//! [`FilterPlan`] ——「 up N 」 and 「empty that 」 is 
//! 1 （ that keyword not can down is as SQLite LIKE 
//! ASCII ； no has ， no has that ）。
//!
//! ── -- ────────────────────────────────────────
//! `request_daily` 1 1 ， `*Stats` is JSON 。must
//! **、 in Rust 、**：total and is 1 
//! ， use `UPDATE ... SET requests = requests + 1` total、，
//! will 「 and = total」 this for before （ `fold_into_daily`）。
//! 1 has 1 、 ，-- in ** 1 **， can 
//! （ this is / UPDATE not ： has 1 ）。
//! this 1 （ and ） in `daily.rs` —— file after 
//! ，file `requests` and use [`FilterPlan`]。
//!
//! ── and ： not ────────────────────────────────────────
//! all `&Connection`， by `Db` that Mutex （ up 
//! in ** 1 ** `Db::with` / `with_mut` use ， use ）。
//! ****： this not can `logging::log` —— need 1 ，
//! `std::sync::Mutex` not can ， will （`request_stats.rs` module header this ）。

use rusqlite::types::Value as SqlValue;
use rusqlite::{params, params_from_iter, Connection};
use serde_json::Value;

use super::record::{AttemptDetail, RequestEntry, RequestQuery, RunningProgress, SensitiveHit};
use super::report::{normalize_status_filter, SummaryRow};

mod raw;
pub(super) use raw::{
    count_raw, count_raw_matching, count_upstream_raw, delete_raw_matching, raw_ids, select_raw,
    trim_raw_rows, upsert_raw,
};

/// `requests` （all SELECT by this ，`decode_request` ；
/// not `SELECT *` is as 「 when need up 」 this in diff can ）
///
/// ** new 1 in **：`decode_request` by ， has not will 
/// before （ in 1 will all after 1 ， that in file
/// not ， will ）。 after in after ：
/// - `attempt_details` / `sensitive_hits`（schema v2 JSON ）；
/// - `credit`（schema v23 ， can empty REAL）。
pub(super) const REQUEST_COLUMNS: &str = "id, ts, model, account_id, account_name, status, duration_ms, \
     first_response_ms, attempts, error, prompt_tokens, completion_tokens, total_tokens, \
     cache_read_tokens, provider, client_model, upstream_model, attempt_details, sensitive_hits, \
     client_reasoning, upstream_reasoning, phase, phase_started_at, is_test, route, client, \
     api_key_id, api_key_name, api_key_source, credit, client_evidence";

/// —— ** and `db::schema` `idx_requests_summary` 1 **
/// （ is `ts, is_test` before + this after 5 ）， 1 need for 1 。
///
/// ： 24 when when （file 
/// [`select_summary_rows`]） and in （ already by [`sum_cache_windows`] in SQL 
/// ， not ）。 this 6 ， `requests` 1 that 
/// （`route` in 1.5 KB、`attempt_details` / `sensitive_hits` is 
/// JSON …） 1 use not up —— by [`REQUEST_COLUMNS`] ， etc 
/// 「 24 when statistics」「 24 when in MB none」，
/// this is need need （ 1 ： 273 ms、
/// 98 ms，Rust ）。
///
/// `is_test` in WHERE （excluding）， not ， not in this 
/// —— in in in `ts` after 、 this 5 before 。
const SUMMARY_COLUMNS: &str =
    "ts, model, upstream_model, total_tokens, cache_read_tokens, prompt_tokens";

// `request_daily`（ by ） that 1 、 and -- in 
// `daily.rs` —— file after 。

// ─── ─────────────────────────────────────────────────

/// → `RequestEntry`。
///
/// and 1 1 for ，** not 1 **（ not new 、 not `is_success`）：
/// is `NewRequestEntry::normalize` 1 ， 1 ，
/// will 「 has 」 this 。 can empty （`error` /
/// `first_response_ms`） `Option` ——「 no has error」 and 「emptyerror」 in 
/// already 1 `None`， not 。
///
/// JSON [`decode_json_list`]：empty not is not 
/// （ by that ）。
pub(super) fn decode_request(row: &rusqlite::Row<'_>) -> rusqlite::Result<RequestEntry> {
    let attempt_details: String = row.get(17)?;
    let sensitive_hits: String = row.get(18)?;
    Ok(RequestEntry {
        id: row.get(0)?,
        ts: row.get(1)?,
        model: row.get(2)?,
        account_id: row.get(3)?,
        account_name: row.get(4)?,
        status: row.get(5)?,
        duration_ms: row.get(6)?,
        first_response_ms: row.get(7)?,
        attempts: row.get(8)?,
        error: row.get(9)?,
        prompt_tokens: row.get(10)?,
        completion_tokens: row.get(11)?,
        total_tokens: row.get(12)?,
        cache_read_tokens: row.get(13)?,
        provider: row.get(14)?,
        client_model: row.get(15)?,
        upstream_model: row.get(16)?,
        attempt_details: decode_json_list::<AttemptDetail>(&attempt_details),
        sensitive_hits: decode_json_list::<SensitiveHit>(&sensitive_hits),
        // is schema v5 （ and REQUEST_COLUMNS 1 ）；
        // old DEFAULT '' ， before by empty
        client_reasoning: row.get(19)?,
        upstream_reasoning: row.get(20)?,
        // is schema v6 （ in and ）； not in / old 
        // is '' and NULL —— and 「 not in 」 in 1 
        phase: row.get(21)?,
        phase_started_at: row.get(22)?,
        // is schema v7 （）； old DEFAULT 0 → false（）。
        // SQLite no has ， by INTEGER —— not for bool 
        // （ 2 「 is 」）
        is_test: row.get::<_, i64>(23)? != 0,
        // is schema v10 （）； old / no to is 
        // NULL —— Null， not that 1 。rusqlite serde_json
        // feature，Value not can FromSql： TEXT ， Null
        // （ and decode_json_list 「」 1 to ）
        route: row
            .get::<_, Option<String>>(24)?
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or(Value::Null),
        // is schema v12 （requestclient）； old DEFAULT '' 
        // —— empty = no has （ before ）， and none UA request 1 
        source: super::RequestSource {
            client: row.get(25)?,
            api_key: super::ApiKeyIdentity {
                id: row.get(26)?,
                name: row.get(27)?,
                source: row.get(28)?,
            },
            evidence: serde_json::from_str::<crate::server::core::client::ClientEvidence>(&row.get::<_, String>(30)?).ok()
                .filter(|evidence| !evidence.method.is_empty()),
            behavior_client: String::new(),
        },
        // is schema v23 （）； old / no up is NULL ——
        // None， and 「 up 0」（ before before 「—」）。
        credit: row.get(29)?,
    })
}

/// JSON → `Vec<T>`（ ）。failure or not is when 
/// empty。
///
/// and `decode_accum`（ that ， already to `daily.rs`） 1 to ： this is 
/// ****， 1 not this not —— empty is /
/// in ，、、usage、error in 。， if in this ， 1 
/// JSON can request not （`select_page_desc` will as 1 `?` failure）。
fn decode_json_list<T: serde::de::DeserializeOwned>(text: &str) -> Vec<T> {
    serde_json::from_str(text).unwrap_or_default()
}

/// JSON ← `Vec<T>`。failure to `'[]'`（ and DDL default 1 ，
/// is 「 this 1 no has 」 in has 1 ）。
fn encode_json_list<T: serde::Serialize>(items: &[T]) -> String {
    serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())
}

/// `Value` → JSON （`Null` SQL NULL —— 「 no has 」 and old ，
/// and `encode_json_list` `'[]'` not ： that empty is has ， this NULL is ）。
fn encode_optional_json(value: &serde_json::Value) -> SqlValue {
    if value.is_null() {
        SqlValue::Null
    } else {
        SqlValue::Text(value.to_string())
    }
}

// ─── → SQL ─────────────────────────────────────────

/// 1 ****：WHERE + 。
///
/// 、 and use 1 。
/// old is use 1 `matches_filter` ， in use down ： SQL 
/// WHERE is 1 ， not can can 。
/// `offset` / `limit` is parameter、 not is ， not in this 。
pub(super) struct FilterPlan {
    /// `WHERE ...` ；none when as empty（ in FROM after ）
    where_sql: String,
    /// and `where_sql` `?` 1 1 for 
    binds: Vec<SqlValue>,
}

impl FilterPlan {
    /// by old `matches_filter` （none，SQL AND ）。
    pub(super) fn of(filter: &RequestQuery) -> Self {
        let mut fragments: Vec<String> = Vec::new();
        let mut binds: Vec<SqlValue> = Vec::new();

        // ① ；empty（empty） no 。
        // ** **：`model`（request） and `upstream_model`
        // （ to up ）。 by up （
        // `fold_into_daily` `model_stat_key`）， `model` is 
        // request —— 1 when ，request in 
        // （ in upstream_model ） will 0 。 this can to ，
        // and ； and down （`select_model_options`） 1 。
        if let Some(want) = filter.model.as_deref().filter(|text| !text.is_empty()) {
            fragments.push("(model = ? OR upstream_model = ?)".to_string());
            binds.push(SqlValue::Text(want.to_string()));
            binds.push(SqlValue::Text(want.to_string()));
        }

        // ①′ provider id （ 1 ：empty no ）。
        //
        // by `provider` not is 「 1 」： that 1 is **
        // request that 1 **（ after is after down that ）， and 
        // is 1 —— and to 1 。
        // empty id （ 1 no failure） not will empty id in ， this is has ：
        // not 1 ， use status=error （ RequestQuery::provider）。
        if let Some(want) = filter.provider.as_deref().filter(|text| !text.is_empty()) {
            fragments.push("provider = ?".to_string());
            binds.push(SqlValue::Text(want.to_string()));
        }

        if let Some(want) = filter.client.as_deref() {
            fragments.push("client = ?".to_string());
            binds.push(SqlValue::Text(want.to_string()));
        }
        if let Some(identity) = filter.api_key.as_deref().and_then(super::ApiKeyIdentity::from_filter) {
            fragments.push("api_key_source = ? AND api_key_id = ?".to_string());
            binds.push(SqlValue::Text(identity.source));
            binds.push(SqlValue::Text(identity.id));
        }
        if filter.production_only {
            fragments.push("is_test = 0".to_string());
        }
        if filter.completed_only {
            fragments.push("status <> 0".to_string());
        }

        // ② success / failure / in ： and `RequestEntry::is_success` ** etc ** 
        // （success and failure）， up in 。
        //
        // success is 「2xx ** and ** no has error need 」（ 1 ，
        // record.rs that ：request 200 is response ， after 
        // up can error ）。：
        //   success   → `status >= 200 AND status < 300 AND error IS NULL`
        // failure → up （** not is ** `status NOT BETWEEN`， that will 
        // 「2xx error need 」 this 1 —— must is failure）
        // in → `status = 0`（ when ， no ；
        //            `insert_started_request`）
        //
        // failure**excluding** status=0： will 「 no 」 in 
        // failure —— that is 1 （ use to 1 failure， its is 
        // in ）。status=0 in old from use （schema ）， this excluding
        // new 。
        //
        // `error IS NULL` and `is_none()` etc for **path**：
        // - `normalize`， empty need `None`
        // （`error.filter(|text| !text.is_empty())`）→ is NULL；
        // - from old file ****（ not `normalize`）， old file
        // `"error": ""` will empty。 when SQL is 「 has error」（`IS NULL`
        // as ）、Rust `is_success()` has error（`Some("")` not is `None`）
        // —— 1 。 this not need need `error <> ''` 。
        if let Some(want) = normalize_status_filter(filter.status.as_deref()) {
            let success = "status >= 200 AND status < 300 AND error IS NULL";
            let fragment = match want {
                super::report::StatusFilter::Ok => format!("({success})"),
                super::report::StatusFilter::Error => format!("(NOT ({success}) AND status <> 0)"),
                super::report::StatusFilter::Running => "status = 0".to_string(),
            };
            fragments.push(fragment);
        }

        // ③ / ④ when ： [start, end)。`end` is ， and 1 
        //（ up 1 after 1 ts can down 1 end， not will to 1 ）。
        if let Some(from) = filter.start {
            fragments.push("ts >= ?".to_string());
            binds.push(SqlValue::Integer(from));
        }
        if let Some(to) = filter.end {
            fragments.push("ts < ?".to_string());
            binds.push(SqlValue::Integer(to));
        }

        let where_sql = if fragments.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", fragments.join(" AND "))
        };
        Self { where_sql, binds }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.where_sql.is_empty()
    }
}

// ─── ： and ───────────────────────────────────────

/// total（ `total` use ）
pub(super) fn count_all(conn: &Connection) -> rusqlite::Result<usize> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM requests", [], |row| row.get(0))?;
    Ok(count.max(0) as usize)
}

/// in （`QueryResult.matched`）—— and `limit` none，
/// before total（ `ui/requests-panel.js` `renderPager`）
pub(super) fn count_matching(
    conn: &Connection,
    plan: &FilterPlan,
) -> rusqlite::Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM requests{}", plan.where_sql);
    let count: i64 = conn.query_row(&sql, params_from_iter(plan.binds.iter()), |row| row.get(0))?;
    Ok(count.max(0) as usize)
}

/// 1 need （total / in / its in in ）—— ** 1 **。
///
/// `COUNT(*)`： in WHERE up 1 ，seconds
/// 1 ， up when etc seconds。 this 1 
/// `SELECT` `COUNT(*)` and `SUM(CASE ...)`， from to 1 。
///
/// `CASE` need 1 parameter（SQLite not use ），
/// by 「 in 」 and 「 in and status=0」 1 。
pub(super) fn counts(
    conn: &Connection,
    plan: &FilterPlan,
) -> rusqlite::Result<(usize, usize, usize)> {
    let filter = plan
        .where_sql
        .strip_prefix(" WHERE ")
        .unwrap_or("1");
    let sql = format!(
        "SELECT COUNT(*), \
         COALESCE(SUM(CASE WHEN ({filter}) THEN 1 ELSE 0 END), 0), \
         COALESCE(SUM(CASE WHEN ({filter}) AND status = 0 THEN 1 ELSE 0 END), 0) \
         FROM requests"
    );
    let mut binds = Vec::with_capacity(plan.binds.len() * 2);
    binds.extend(plan.binds.iter().cloned());
    binds.extend(plan.binds.iter().cloned());
    let (total, matched, running): (i64, i64, i64) = conn.query_row(
        &sql,
        params_from_iter(binds.iter()),
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    Ok((
        total.max(0) as usize,
        matched.max(0) as usize,
        running.max(0) as usize,
    ))
}

/// down ： and provider id（ by ）。
///
/// ── Why not by current when ──────────────────────────────
/// is 「 this 」， and up when is ：
/// ， 1 need 1 ， down will in use need 
/// when （ in can can empty）。， is to 「」
/// after can can 1 before —— in to empty，
/// Reason 1 can （ when that 1 「」）。
///
/// ── limit ──────────────────────────────────────────
/// down is to ， already can ； and this by ，
/// that 1 。 is ****， not is can ：
/// before will current in in （ ui/requests-panel.js 
/// `fillFilterSelect`），「 → no has 」 not will 。
pub(super) fn select_filter_options(
    conn: &Connection,
    max: usize,
) -> rusqlite::Result<(Vec<String>, Vec<String>)> {
    Ok((
        select_model_options(conn, max)?,
        select_distinct_ordered(conn, "provider", max)?,
    ))
}

/// ：`model` and `upstream_model` empty and 。
///
/// and [`FilterPlan`] 1 （ this can to ， by that 
/// ）： if `model` ， by up after ， use 
/// when down no has that —— is 「 has this 1 、 down no has 
/// this 」 not 1 （ `stats_request_filters` ）。request 
/// ， in and 1 ， by 。
///
/// not use [`select_distinct_ordered`]： that by ， this is 
/// `UNION ALL` after ， will parameter SQL ， can 。
fn select_model_options(conn: &Connection, max: usize) -> rusqlite::Result<Vec<String>> {
    let sql = "SELECT name FROM (
         SELECT model AS name FROM requests WHERE model <> ''
         UNION ALL
         SELECT upstream_model AS name FROM requests WHERE upstream_model <> ''
       ) GROUP BY name ORDER BY COUNT(*) DESC, name ASC LIMIT ?1";
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params![max as i64])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(row.get(0)?);
    }
    Ok(out)
}

/// 1 empty， by 、 by （ `max` ）。
///
/// when **must** has ： by `COUNT(*)` when by 
/// SQLite ， use can can to not —— down will none by 
/// （ use need that 1 can can ）。
///
/// by use to （ not use ）， SQL is ；
/// limitparameter， and its 1 。
fn select_distinct_ordered(
    conn: &Connection,
    column: &str,
    max: usize,
) -> rusqlite::Result<Vec<String>> {
    let sql = format!(
        "SELECT {column} FROM requests WHERE {column} <> '' \
         GROUP BY {column} ORDER BY COUNT(*) DESC, {column} ASC LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![max as i64])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(row.get(0)?);
    }
    Ok(out)
}

pub(super) fn source_filter_options(conn: &Connection, max: usize) -> rusqlite::Result<Value> {
    let mut client_stmt = conn.prepare(
        "SELECT grouped.client, CASE WHEN json_valid(r.client_evidence) \
         THEN COALESCE(json_extract(r.client_evidence, '$.clientName'), '') ELSE '' END \
         FROM (SELECT client, MAX(row_id) AS latest, COUNT(*) AS n FROM requests \
         GROUP BY client ORDER BY n DESC, client LIMIT ?1) grouped \
         JOIN requests r ON r.row_id = grouped.latest ORDER BY grouped.n DESC, grouped.client",
    )?;
    let clients = client_stmt.query_map(params![max as i64], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut stmt = conn.prepare(
        "SELECT api_key_id, api_key_source, \
         (SELECT r.api_key_name FROM requests r WHERE r.api_key_id = requests.api_key_id \
          AND r.api_key_source = requests.api_key_source AND r.api_key_name <> '' ORDER BY r.ts DESC, r.row_id DESC LIMIT 1) \
         FROM requests GROUP BY api_key_source, api_key_id \
         ORDER BY COUNT(*) DESC, api_key_source, api_key_id LIMIT ?1",
    )?;
    let keys = stmt.query_map(params![max as i64], |row| {
        let identity = super::ApiKeyIdentity {
            id: row.get(0)?, source: row.get(1)?, name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        };
        Ok(serde_json::json!({ "value": identity.filter_value(), "label": super::source_report::key_label(&identity) }))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    let mut clients: Vec<Value> = clients.into_iter().filter(|(client, _)| !client.is_empty()).map(|(client, label)| serde_json::json!({
        "value": client, "label": if label.is_empty() { super::source_report::client_label(&client).to_string() } else { label },
    })).collect();
    clients.push(serde_json::json!({ "value": "", "label": "未记录客户端" }));
    Ok(serde_json::json!({ "clients": clients, "apiKeys": keys }))
}

/// / when （empty as `None`）—— `stats()` `firstTs` / `lastTs`。
///
/// old is （ by ts ） ts， and `MIN` / `MAX` etc 。
pub(super) fn min_ts(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT MIN(ts) FROM requests", [], |row| row.get(0))
}

pub(super) fn max_ts(conn: &Connection) -> rusqlite::Result<Option<i64>> {
    conn.query_row("SELECT MAX(ts) FROM requests", [], |row| row.get(0))
}

/// ，** new in before **（`ORDER BY ts DESC, row_id DESC`）。
///
/// old is 「 after `skip(offset).take(limit)`」： after ts 
/// is after in before ， is `row_id DESC` 。`offset` from 0 （ by 
/// ）。
pub(super) fn select_page_desc(
    conn: &Connection,
    plan: &FilterPlan,
    offset: usize,
    limit: usize,
) -> rusqlite::Result<Vec<RequestEntry>> {
    let sql = format!(
        "SELECT {REQUEST_COLUMNS} FROM requests{} ORDER BY ts DESC, row_id DESC LIMIT ? OFFSET ?",
        plan.where_sql
    );
    let mut binds = plan.binds.clone();
    binds.push(SqlValue::Integer(limit as i64));
    binds.push(SqlValue::Integer(offset as i64));
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(binds.iter()))?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(decode_request(row)?);
    }
    Ok(out)
}

/// `[from_ms, to_ms]` ****， by （ts 、 ts by row_id）。
///
/// in 1 use ：`aggregate::rebuild_day` when that 1 
/// （ already —— in [`sum_cache_windows`] SQL 、
/// [`select_summary_rows`] 6 ， ； is this 
/// that 25 「 1 use not to 」 ）。
/// is as and `record` 1 —— 
/// after ， no need 。
///
/// `exclude_tests` by use by 「 this need not need 」 to （`rebuild_day` 
/// true， `RequestEntry::is_test`）： is ， will 
/// 。request that 1 is `select_page_desc`， not this 
/// —— need 。
pub(super) fn select_between(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
    exclude_tests: bool,
) -> rusqlite::Result<Vec<RequestEntry>> {
    // in this not is SQL： SQL will （ 1 、
    // 1 ）， has 1 
    let filter = if exclude_tests { " AND is_test = 0" } else { "" };
    let sql = format!(
        "SELECT {REQUEST_COLUMNS} FROM requests WHERE ts >= ?1 AND ts <= ?2{filter} \
         ORDER BY ts ASC, row_id ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![from_ms, to_ms])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(decode_request(row)?);
    }
    Ok(out)
}

/// ****（[`SUMMARY_COLUMNS`] that 6 ）， to 24 when use 。
///
/// and [`select_between`] ： that return `RequestEntry`（ by and 
/// need need need ）， —— is 
/// `idx_requests_summary` that ， is this is ****， 1 
/// （`route` etc ） not will （ and for 
/// [`SUMMARY_COLUMNS`]）。
///
/// excluding（`is_test = 0`）： is 「」，
/// is （ and `select_between` `exclude_tests` 1 ）。
/// use not that —— is this 1 use 。
///
/// not `ORDER BY`： use by ， and not 
/// （`report::cache_trend_24h` by 24 ， by 
/// `model_breakdown_json` ）。 SQLite can by to 。
pub(super) fn select_summary_rows(
    conn: &Connection,
    from_ms: i64,
    to_ms: i64,
) -> rusqlite::Result<Vec<SummaryRow>> {
    let sql = format!(
        "SELECT {SUMMARY_COLUMNS} FROM requests \
         WHERE ts >= ?1 AND ts <= ?2 AND is_test = 0"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params![from_ms, to_ms])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        out.push(SummaryRow {
            ts: row.get(0)?,
            model: row.get(1)?,
            upstream_model: row.get(2)?,
            total_tokens: row.get(3)?,
            cache_read_tokens: row.get(4)?,
            prompt_tokens: row.get(5)?,
        });
    }
    Ok(out)
}

/// in `(hitTokens, inputTokens)` ，** 1 **。
///
/// ── Why down to SQL ────────────────────────────────────────
/// before is 「 7 Rust， for 1 」——
/// that （ `route` etc none） this 。defaultseconds
/// 1 （`config::DEFAULT_REPORT_AUTO_REFRESH_SECONDS`）， this is **** 。
/// 1 can when `CASE WHEN` 。
///
/// ── down must is **** that （ 1 ）──────────
/// 「 down 」，， then ****：`WHERE`
/// already limit in 10 minutes， after `CASE WHEN ts >= (now-24h)` 
/// for this ** as **， is 1 （ etc 10 minutes）。
/// this not will 、 not will ， will up in 1 1 ——
/// before when its 。 is 「
/// ， in `CASE WHEN` 」。
///
/// up this ：`WHERE ts >= ?1`（） `idx_requests_summary`
/// is ****（ in in ， not ）， 1 up 
/// 3 、7 68 ms → 3.4 ms。
///
/// `windows_ms` by use to （`report::CACHE_WINDOWS`， and JSON in 
/// that 1 ），return and 1 1 for —— 「 by and 」， not 
/// （`safe_rate` and in `report` ）。
///
/// to `SUM(CASE WHEN ... END)` not is `SUM(...) FILTER`：SQLite to 
/// 3.30 supports `FILTER` ， `CASE WHEN` in all up 。`COALESCE`
/// 「 1 no has 」 when NULL —— return 0 and 「 no has 」 in is 1 
/// （ in 0%）， not use 。
pub(super) fn sum_cache_windows(
    conn: &Connection,
    now_ms: i64,
    windows_ms: &[i64],
) -> rusqlite::Result<Vec<(i64, i64)>> {
    let Some(longest) = windows_ms.iter().copied().max() else {
        return Ok(Vec::new());
    };
    // parameter：?1 = down （****， up that ）、?2..?n = down 、
    // after 1 = now（ up ， and `ts <= now` 1 ）
    let mut sql = String::from("SELECT ");
    for index in 0..windows_ms.len() {
        if index > 0 {
            sql.push_str(", ");
        }
        let slot = index + 2;
        sql.push_str(&format!(
            "COALESCE(SUM(CASE WHEN ts >= ?{slot} THEN cache_read_tokens END), 0), \
             COALESCE(SUM(CASE WHEN ts >= ?{slot} THEN prompt_tokens END), 0)"
        ));
    }
    let last = windows_ms.len() + 2;
    sql.push_str(&format!(
        " FROM requests WHERE ts >= ?1 AND ts <= ?{last} AND is_test = 0"
    ));
    let mut binds: Vec<i64> = Vec::with_capacity(windows_ms.len() + 2);
    binds.push(now_ms - longest);
    binds.extend(windows_ms.iter().map(|window| now_ms - window));
    binds.push(now_ms);
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(binds.iter()))?;
    // none GROUP BY return 1 ；empty（ not this ） when by 「
    // no has 」 —— and `safe_rate` 1 ， panic 
    let Some(row) = rows.next()? else {
        return Ok(vec![(0, 0); windows_ms.len()]);
    };
    let mut out = Vec::with_capacity(windows_ms.len());
    for index in 0..windows_ms.len() {
        out.push((row.get::<_, i64>(index * 2)?, row.get::<_, i64>(index * 2 + 1)?));
    }
    Ok(out)
}

/// 1 ** in ** （status=0， before use ； `RequestStats::record_started`）。
///
/// etc ： id already has in when （ use not —— 
/// `update_running_request` by 「id + status=0」， 1 will 
/// to error up ）。「 already has 」 not ： id is 
/// （、）， new 1 this has in 。
///
/// ， its DDL DEFAULT： and 「path not 」 to 1 ——
/// in 「 this request、」， 1 etc when 。
///
/// ── ： in this ────────────────────────────────
/// `phase='connecting'` + `phase_started_at=ts`（request when ） is ** in ** 
/// has ， not is 「 will has 」：request 1 in 「 in 」 this 1 
/// （ / / in ）， is this 1 `ts`。 in this not is etc 
/// 1 in ， is as 「、 no to up 」 that 1 
/// （ms， 1 can can to ） has can —— ， will 1 down 
/// empty「 in 」。 and OmniProxy `insertPendingLog`（
/// `phase='connecting'`）。
///
/// return is new （ use before not need need ， and can can ）。
pub(super) fn insert_started_request(
    conn: &Connection,
    id: &str,
    ts: i64,
    model: &str,
    client_model: &str,
    client_reasoning: &str,
    source: &super::RequestSource,
    is_test: bool,
) -> rusqlite::Result<bool> {
    let existing: i64 = conn.query_row(
        "SELECT COUNT(*) FROM requests WHERE id = ?1 AND status = 0",
        params![id],
        |row| row.get(0),
    )?;
    if existing > 0 {
        return Ok(false);
    }
    // client_reasoning ： down etc in request when （ and client_model
    // 1 when 、 1 ）， in can 「request ( etc )」
    // is_test ： is ， etc will 「 in 」
    // that 1 and 1 
    // client（client） 1 by ： in request that 1 ，
    // in up —— etc will 「client」 in empty 1 
    conn.execute(
        "INSERT INTO requests (id, ts, model, client_model, client_reasoning, status, \
         phase, phase_started_at, is_test, client, api_key_id, api_key_name, api_key_source, client_evidence) \
         VALUES (?1, ?2, ?3, ?4, ?5, 0, 'connecting', ?2, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![id, ts, model, client_model, client_reasoning, is_test as i64,
            source.client, source.api_key.id, source.api_key.name, source.api_key.source, source.evidence_json()],
    )?;
    Ok(true)
}

/// ** old in **：status=0 and when cutoff no has will to 
/// UPDATE（ / ）， 1 failure。
///
/// Why is 「」 not is delete： up in → is use will 1 
/// 。 1 etc requestempty， use will as ； 408 then 
/// 「 this no 」。OmniProxy `applyInterruptedLogs`（requestLogLifecycle.ts）
/// is 1 。`duration_ms` and `ts` 1 and ， 0 when 。
pub(super) fn finish_stale_running(
    conn: &Connection,
    cutoff: i64,
    now: i64,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE requests SET status = 408, error = '网关重启或流中断，请求未能完成', \
         duration_ms = MAX(?2 - ts, 0), attempts = 1, \
         attempt_details = '[]', sensitive_hits = '[]', phase = '', phase_started_at = NULL \
         WHERE status = 0 AND ts < ?1",
        params![cutoff, now],
    )
}

/// by id 1 ** in **（`RequestStats::finalize_interrupted` 1 ）。
///
/// and [`finish_stale_running`] ： that by 「 when 」，
/// is ； this by id ， by （`api::pipeline::DisconnectGuard`）
/// in handler axum （client） when use 。 408 —— for 
/// is 「 this 1 no 」，Reason in error 。
///
/// ── ，** not ** ────────────────────────
/// in can can already and in ， that is （ for 
/// `finish_stale_running`： no has in ， is ）。
/// `attempts` `MAX(attempts, 1)`： in already ， not to 0
/// （ is 「、 ≥1」）。
///
/// ****（ and 1 ）： in has ， this 1 already 
/// 408 ，「 up to 」 will empty 1 
/// not before when 。
///
/// and UPDATE 1 （`id + status = 0`）： already empty，
/// not —— and 「 in to 1 」 1 。
pub(super) fn finish_running_request(
    conn: &Connection,
    id: &str,
    error: &str,
    now: i64,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE requests SET status = 408, error = ?2, duration_ms = MAX(?3 - ts, 0), \
         attempts = MAX(attempts, 1), phase = '', phase_started_at = NULL \
         WHERE id = ?1 AND status = 0",
        params![id, error, now],
    )
}

/// 1 ** in ** ** in **（ when 1 ；
/// `RequestStats::update_running`）。
///
/// ── and ** 1 **（`id + status = 0`）────────────
/// after this 1 empty —— that is need ： by 1 ， in 
/// to 1 not this （ 1 already 「 in 」 ）。
/// 1 request is 1 ， not need need etc 。
///
/// ── Why not this ────────────────────────────────────────
/// - `status` / `error`： before not this is 「 in 」（ has error not is ，
/// `ui/requests-panel.js` isRunning）—— in failure can can is before 
/// 1 failure， not this before failure；
/// - `ts` / `model` / `client_model`：request when ， in no has new ；
/// - `duration_ms` / token ： has has （usage in in not ）。
///
/// ** in this 1 and not **： is need （ `phase` 
/// ）， is and status/error 1 「 in 」 not is 。
pub(super) fn update_running_progress(
    conn: &Connection,
    id: &str,
    progress: &RunningProgress,
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE requests SET provider = ?2, account_id = ?3, account_name = ?4, \
         upstream_model = ?5, upstream_reasoning = ?6, attempts = ?7, first_response_ms = ?8, \
         attempt_details = ?9, sensitive_hits = ?10, phase = ?11, phase_started_at = ?12, \
         route = COALESCE(?13, route) WHERE id = ?1 AND status = 0",
        params![
            id,
            progress.provider,
            progress.account_id,
            progress.account_name,
            progress.upstream_model,
            progress.upstream_reasoning,
            progress.attempts,
            progress.first_response_ms,
            encode_json_list(&progress.attempt_details),
            encode_json_list(&progress.sensitive_hits),
            progress.phase,
            progress.phase_started_at,
            progress.route.as_ref().map(Value::to_string),
        ],
    )
}

/// 1 in ： by `id + status=0` ，****。
///
/// ── Why is 「 UPDATE after INSERT」 not is 1 + UPSERT ──────
/// `requests.id` up ** not can ** 1 ： id can can has 
/// （ / etc path ， schema id ），
/// will 1 、。 no has 1 no has UPSERT 
/// —— `ON CONFLICT(id) DO UPDATE` in no has for 1 when will error。
/// ： UPDATE in （ has ）， 0 INSERT
/// （ before failure etc no has in path）。 in 1 ，
/// not in 「UPDATE 、INSERT before 」 to in 。
///
/// update** not row_id**： in already not ，
/// （ in 「 in 」 in up ， after not this to ）。`id` as empty 
/// return 0（ not UPDATE ）：empty id started not in ，failurepath
/// no has id can —— to INSERT。
pub(super) fn update_running_request(
    conn: &Connection,
    entry: &RequestEntry,
) -> rusqlite::Result<usize> {
    if entry.id.is_empty() {
        return Ok(0);
    }
    conn.execute(
        "UPDATE requests SET ts = ?2, model = ?3, account_id = ?4, account_name = ?5, status = ?6, \
         duration_ms = ?7, first_response_ms = ?8, attempts = ?9, error = ?10, prompt_tokens = ?11, \
         completion_tokens = ?12, total_tokens = ?13, cache_read_tokens = ?14, provider = ?15, \
         client_model = ?16, upstream_model = ?17, attempt_details = ?18, sensitive_hits = ?19, \
         client_reasoning = ?20, upstream_reasoning = ?21, phase = '', phase_started_at = NULL, \
         is_test = ?22, route = ?23, client = ?24, \
         api_key_id = ?25, api_key_name = ?26, api_key_source = ?27, credit = ?28, client_evidence = ?29 \
         WHERE id = ?1 AND status = 0",
        params![
            entry.id,
            entry.ts,
            entry.model,
            entry.account_id,
            entry.account_name,
            entry.status,
            entry.duration_ms,
            entry.first_response_ms,
            entry.attempts,
            entry.error,
            entry.prompt_tokens,
            entry.completion_tokens,
            entry.total_tokens,
            entry.cache_read_tokens,
            entry.provider,
            entry.client_model,
            entry.upstream_model,
            encode_json_list(&entry.attempt_details),
            encode_json_list(&entry.sensitive_hits),
            entry.client_reasoning,
            entry.upstream_reasoning,
            // when ** 1 **（ not is in when ）： this UPDATE is 
            // none in when path， 1 path 1 
            entry.is_test as i64,
            encode_optional_json(&entry.route),
            // client is_test： already 1 ， 1 
            // 「 has in 」 and 「」path 
            entry.source.client,
            entry.source.api_key.id,
            entry.source.api_key.name,
            entry.source.api_key.source,
            entry.credit,
            entry.source.evidence_json(),
        ],
    )
}

pub(super) fn terminal_record_exists(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM requests WHERE id = ?1 AND status <> 0) \
         AND NOT EXISTS(SELECT 1 FROM requests WHERE id = ?1 AND status = 0)",
        params![id], |row| row.get(0),
    )
}

/// 1 （`row_id` by ， use not not this to ）
///
/// JSON by [`encode_json_list`] ；empty `'[]'`， and DDL 
/// default（ is 「 no has 」 in has 1 ， not 「NULL is empty」）。
///
/// ** not in this **： as empty（ `RequestEntry::phase`），
/// DDL default（`''` / `NULL`） is that —— DEFAULT 1 not 
/// （ will when not will this ）。
pub(super) fn insert_request(conn: &Connection, entry: &RequestEntry) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO requests (id, ts, model, account_id, account_name, status, duration_ms, \
         first_response_ms, attempts, error, prompt_tokens, completion_tokens, total_tokens, \
         cache_read_tokens, provider, client_model, upstream_model, attempt_details, \
         sensitive_hits, client_reasoning, upstream_reasoning, is_test, route, client, \
         api_key_id, api_key_name, api_key_source, credit, client_evidence) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, \
         ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29)",
        params![
            entry.id,
            entry.ts,
            entry.model,
            entry.account_id,
            entry.account_name,
            entry.status,
            entry.duration_ms,
            entry.first_response_ms,
            entry.attempts,
            entry.error,
            entry.prompt_tokens,
            entry.completion_tokens,
            entry.total_tokens,
            entry.cache_read_tokens,
            entry.provider,
            entry.client_model,
            entry.upstream_model,
            encode_json_list(&entry.attempt_details),
            encode_json_list(&entry.sensitive_hits),
            entry.client_reasoning,
            entry.upstream_reasoning,
            entry.is_test as i64,
            encode_optional_json(&entry.route),
            entry.source.client,
            entry.source.api_key.id,
            entry.source.api_key.name,
            entry.source.api_key.source,
            entry.credit,
            entry.source.evidence_json(),
        ],
    )?;
    Ok(())
}

// ─── ：delete and ───────────────────────────────────────

/// delete in ，returndelete。
///
/// use must in 1 in （`delete_raw_matching`）。
pub(super) fn delete_matching(
    conn: &Connection,
    plan: &FilterPlan,
) -> rusqlite::Result<usize> {
    let sql = format!("DELETE FROM requests{}", plan.where_sql);
    conn.execute(&sql, params_from_iter(plan.binds.iter()))
}

/// empty。 use must in 1 in 。
pub(super) fn delete_all_requests(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM requests", [])
}

/// `ts < cutoff` （：`record` and `prune` ）。
///
/// **** this for request_raw （ by `delete_raw_matching`）：
/// as 0～，`ts` can ignore —— path up 
/// 1 DELETE， can ignore（default by 99999 ，
/// delete is use that when ）。
pub(super) fn delete_expired_requests(conn: &Connection, cutoff: i64) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM request_raw WHERE id IN \
         (SELECT id FROM requests WHERE ts < ?1 AND id <> '')",
        params![cutoff],
    )?;
    conn.execute("DELETE FROM requests WHERE ts < ?1", params![cutoff])
}

// ─── ：-- ─────────────────────────────────────────
//
// `request_daily` already to `daily.rs`（、 and 
// select / upsert / delete / trim）， by this file module header。
