//! 、 and SQL。

use rusqlite::{params, params_from_iter, Connection};

use super::FilterPlan;

#[derive(Clone, Copy, Default)]
pub(in crate::server::request_stats) struct RawCounts {
    pub requests: usize,
    pub downstream: usize,
    pub upstream: usize,
}

fn raw_where(plan: &FilterPlan) -> String {
    if plan.where_sql.is_empty() {
        String::new()
    } else {
        format!(" WHERE id IN (SELECT id FROM requests{})", plan.where_sql)
    }
}

fn count_table(conn: &Connection, table: &str, plan: &FilterPlan) -> rusqlite::Result<usize> {
    let sql = format!("SELECT COUNT(*) FROM {table}{}", raw_where(plan));
    let count: i64 = conn.query_row(&sql, params_from_iter(plan.binds.iter()), |row| row.get(0))?;
    Ok(count.max(0) as usize)
}

pub(in crate::server::request_stats) fn count_raw_matching(
    conn: &Connection,
    plan: &FilterPlan,
) -> rusqlite::Result<RawCounts> {
    let where_sql = raw_where(plan);
    // use request ID；UNION 1 request as 。
    let sql = format!(
        "SELECT COUNT(*) FROM (SELECT id FROM request_raw{where_sql} \
         UNION SELECT id FROM debug_traffic{where_sql})"
    );
    let requests: i64 = conn.query_row(
        &sql,
        params_from_iter(plan.binds.iter().chain(plan.binds.iter())),
        |row| row.get(0),
    )?;
    Ok(RawCounts {
        requests: requests.max(0) as usize,
        downstream: count_table(conn, "request_raw", plan)?,
        upstream: count_table(conn, "debug_traffic", plan)?,
    })
}

/// use has ， and in delete before use ；none when 。
pub(in crate::server::request_stats) fn delete_raw_matching(
    conn: &Connection,
    plan: &FilterPlan,
) -> rusqlite::Result<RawCounts> {
    let mut counts = count_raw_matching(conn, plan)?;
    let where_sql = raw_where(plan);
    counts.downstream = conn.execute(
        &format!("DELETE FROM request_raw{where_sql}"),
        params_from_iter(plan.binds.iter()),
    )?;
    counts.upstream = conn.execute(
        &format!("DELETE FROM debug_traffic{where_sql}"),
        params_from_iter(plan.binds.iter()),
    )?;
    Ok(counts)
}

/// as has request ID ， after to down 。
///
/// fork： NULL —— in `{cache_dir}/request-raw/`， this ；
/// `size` by 。return（0 = already not in ， use file）。
pub(in crate::server::request_stats) fn upsert_raw(
    conn: &Connection,
    id: &str,
    ts: i64,
    size: i64,
) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO request_raw (id, ts, request_body, response_body, size, truncated) \
         SELECT ?1, ?2, NULL, NULL, ?3, 0 WHERE EXISTS (SELECT 1 FROM requests WHERE id = ?1) \
         ON CONFLICT(id) DO UPDATE SET ts = excluded.ts, request_body = NULL, \
         response_body = NULL, size = excluded.size, truncated = 0",
        params![id, ts, size],
    )
}

/// id（fork：file when use 「」）。
pub(in crate::server::request_stats) fn raw_ids(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut statement = conn.prepare("SELECT id FROM request_raw")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(|item| item.ok()).collect())
}

pub(in crate::server::request_stats) fn count_raw(conn: &Connection) -> rusqlite::Result<usize> {
    count_table(conn, "request_raw", &FilterPlan::of(&super::super::RequestQuery::default()))
}

pub(in crate::server::request_stats) fn count_upstream_raw(conn: &Connection) -> rusqlite::Result<usize> {
    count_table(conn, "debug_traffic", &FilterPlan::of(&super::super::RequestQuery::default()))
}

pub(in crate::server::request_stats) fn trim_raw_rows(
    conn: &Connection,
    overflow: usize,
) -> rusqlite::Result<usize> {
    if overflow == 0 {
        return Ok(0);
    }
    conn.execute(
        "DELETE FROM request_raw WHERE id IN \
         (SELECT id FROM request_raw ORDER BY ts ASC, rowid ASC LIMIT ?1)",
        params![overflow as i64],
    )
}

/// old in （fork after is NULL， by use by 「」）。
pub(in crate::server::request_stats) fn select_raw(
    conn: &Connection,
    id: &str,
) -> rusqlite::Result<Option<(String, Option<String>, Option<String>, bool)>> {
    let mut stmt = conn.prepare(
        "SELECT id, request_body, response_body, truncated FROM request_raw WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    match rows.next()? {
        Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))),
        None => Ok(None),
    }
}
