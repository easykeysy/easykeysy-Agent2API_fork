//! config in `kv` **** —— 1 SQL 。
//!
//! ── ： 1 1 ────────────────────────────────
//! before config is `{config_dir}/config.json` 1 JSON for ； in 
//! **** is `kv` 1 （ and configfile 1 not ）， is this that 
//! JSON 。 is ：
//! - = 1 ， `is_reserved` that （ that is ，
//! `db::schema` ）；
//! - = 1 「 UPSERT + new config already no has 」。
//! Whyconfig not ** 1 ** `kv` （e.g. `config: {...}`）：config 
//! when is HTTP API ， is （`core::auto_checkin`
//! `autoCheckin`、`core::model_rules` `modelRules`）—— 1 1 
//! after ， this not ； 1 then need 「 → 1 
//! → 」， 1 will config（ that is `update()` in 
//! 、 after can by UPSERT ）。
//!
//! ── ────────────────────────────────────────────────
//! `kv.value` is that JSON（ `30`、 for `{...}`、
//! `"zh-CN"`）， and `config.json` 1 1 for —— when 
//! configfile can for （ `db::schema` module header）。
//!
//! ── Why 1 `&Connection`、 1 `&Db` ─────────────
//! 1 SQL has use ， not ：
//! - ****（`load` / `save_raw`） is `Db`， need `Db::with` ；
//! - ****（`db::migrate::config`） to is `&Connection`（
//! ， and ）。
//! `*_conn` after that is ****（， in use /），
//! not after is （、 error up need ）。 1 
//! SQL， is 「 config」 and 「 to config」 not can can as 
//! （ and `request_stats` 1 ）。
//!
//! ── 「delete」Why in `write_conn` ──────────────────
//! this is **** has （ for file 「file」），
//! is **** not is ： use [`import_conn`]， UPSERT。
//! `db::migrate::config` module header（Why not can delete）。
//!
//! ── and ： not 、 not ──────────────────────────────
//! all `&Connection`， by `Db` that Mutex 。****：
//! this not can `logging::log` / `verbose` —— need 1 ，
//! `std::sync::Mutex` not can ， will （all not ，
//! error 1 `Err` up ， by up in ）。

use rusqlite::Connection;
use serde_json::{Map, Value};

use crate::server::db::schema;
use crate::server::db::Db;

/// config （`db::migrate::config` etc ）。
///
/// and `settings` that 「 in not in 」 not —— this **must** use ：
/// config is ****（）， no 「config no has 」——
/// 「empty」 can can is ， can can is ， will 
/// use old config in file。
/// T6 「 not use `migratedXxx: true` 」， by is 「 and can can not 1 
/// （ 1 ）」—— that for in this ** not **： and in 
/// ** 1 **（ [`import_conn`]）， not in 「 no 」 
/// in 。 `db::migrate::config` module header。
pub(crate) const KEY_CONFIG_MIGRATED: &str = "configMigrated";

/// config（config that ）。
///
/// `is_reserved` ：`kv` is ****（ `priorityScope`、 
/// `logsNextId`、set `desktopSettings`、 ， and **** 
/// `desensitize`（ that can already 、） in ）。 not excluding ， will 「config unknown」
/// `raw` —— after not is GET /api/config ： down 1 
/// will this **config**， delete is 「」，
/// is config can can or （`db::schema` module header 
/// 「 not can 」 is this 1 ）。
pub(crate) fn read_conn(conn: &Connection) -> rusqlite::Result<Map<String, Value>> {
    let mut statement = conn.prepare("SELECT key, value FROM kv")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let mut raw = Map::new();
    for row in rows {
        let (key, value) = row?;
        if schema::is_reserved(&key) {
            continue;
        }
        // not is JSON（、 use ）→ this 1 
        // not is config： is 「 this 1 to default」，
        // will use to allconfig 1 default。
        let Some(value) = value else { continue };
        if let Ok(parsed) = serde_json::from_str::<Value>(&value) {
            raw.insert(key, parsed);
        }
    }
    Ok(raw)
}

/// ：** UPSERT + new config already no has **， 1 。
///
/// ── Whymust（ and file ）────────────────
/// file down this is 1 「file」，`raw` no has not in file ——
/// is 「 1 config」（`set_api_key(None)` will `raw.remove`） can 。
/// after if UPSERT， that will ** in **：
/// down 1 ， use will that （ and not ）。
/// delete this 1 is ** etc required** ， not is 。
///
/// ── delete ：「」─────────────────────────────
/// `schema::is_reserved` as not and delete —— not config。 use 「 has 
/// 、 in Rust 、」 not is 1 `DELETE ... WHERE key NOT IN (…)`：
/// after excluding need SQL， 1 is SQL；
/// `RESERVED_KV_KEYS` ， when not SQL。config in ，
/// not will 「 already 」 this （otherwise down will 1 old file）。
///
/// ── （ 1 ）───────────────────────────
/// 「 UPSERT + delete」must has ，otherwise in failure will down ** new old ** config：
/// already new that + no to old + this no 。 use is 「 after 
/// 1 set、 1 no 」， and none from is 。file down 
/// not in this （ 1 `fs::write` ）， this is ** new need **。
///
/// use `unchecked_transaction` not is `transaction`： need when 
/// 「`Db::with` to `&Connection`」 and 「 to `&Connection`」，
/// `transaction()` need `&mut Connection`（ by and `db::migrate` module header
/// ）。 use not use 1 。
pub(crate) fn write_conn(conn: &Connection, raw: &Map<String, Value>) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| format!("开启配置写入事务失败: {error}"))?;
    for (key, value) in raw {
        upsert(&tx, key, value)?;
    }
    // ： − new config = need 
    for key in stale_keys(&tx, raw)? {
        tx.execute("DELETE FROM kv WHERE key = ?1", rusqlite::params![key])
            .map_err(|error| format!("删除配置项 {key} 失败: {error}"))?;
    }
    tx.commit()
        .map_err(|error| format!("提交配置写入失败: {error}"))?;
    Ok(())
}

/// ：** UPSERT（ not delete）**， and in 1 。
///
/// ── Why not delete ──────────────────────────────────────
/// is 「 old file 」， not is 「 use old file config」。
/// up delete will has 1 （） path： in `bootstrap` ，
/// config use in after —— 1 will has in before 1 config，
/// 「 old file no has 」 will that 。 not delete then is 「 old file
/// and 」， that is this 。
///
/// ── Why and must 1 ──────────────────────────
/// is 「 this old file already 」。 if ， in failure will 
/// down 「 in 、 1 」 ： as ， that ** not will **
/// up （ use is 「 1 set its to default」）。 1 
/// after ， need in 、 need not in 。
///
/// return config（ not ）， use 「 N 」。
pub(crate) fn import_conn(
    conn: &Connection,
    raw: &Map<String, Value>,
    marker: &str,
) -> Result<usize, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| format!("开启配置迁移事务失败: {error}"))?;
    for (key, value) in raw {
        upsert(&tx, key, value)?;
    }
    tx.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![marker, "true"],
    )
    .map_err(|error| format!("写入配置迁移标记失败: {error}"))?;
    tx.commit()
        .map_err(|error| format!("提交配置迁移失败: {error}"))?;
    Ok(raw.len())
}

/// UPSERT（`write_conn` and `import_conn` use ）。
///
/// and when ** not is **： will use set not to ；
/// can in to Reason。 this in current down not can （`db::schema`
/// module header has for ）， is as 1 can is （ ）。
fn upsert(conn: &Connection, key: &str, value: &Value) -> Result<(), String> {
    if schema::is_reserved(key) {
        return Err(format!("配置键与保留键同名，拒绝写入: {key}"));
    }
    let text = serde_json::to_string(value)
        .map_err(|error| format!("配置项 {key} 序列化失败: {error}"))?;
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, text],
    )
    .map_err(|error| format!("写入配置项 {key} 失败: {error}"))?;
    Ok(())
}

/// this config：、 and not in new config。
fn stale_keys(conn: &Connection, raw: &Map<String, Value>) -> Result<Vec<String>, String> {
    let mut statement = conn
        .prepare("SELECT key FROM kv")
        .map_err(|error| format!("读取配置键清单失败: {error}"))?;
    let rows = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| format!("读取配置键清单失败: {error}"))?;
    let mut stale: Vec<String> = Vec::new();
    for row in rows {
        let key = row.map_err(|error| format!("读取配置键清单失败: {error}"))?;
        if !schema::is_reserved(&key) && !raw.contains_key(&key) {
            stale.push(key);
        }
    }
    Ok(stale)
}

/// in not in （ etc ）。
///
/// `SELECT 1 ... LIMIT 1` not is config： in ，
/// `modelRules` this can can has KB， as 「 no has 」 is 。
pub(crate) fn marker_present_conn(conn: &Connection, marker: &str) -> rusqlite::Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM kv WHERE key = ?1 LIMIT 1",
            rusqlite::params![marker],
            |row| row.get(0),
        )
        .ok();
    Ok(found.is_some())
}

// ─── （ + error）────────────────────

/// 1 「config + in not in 」。
///
/// 1 not is use ：`read_raw` need use 「 in not in 」 need not need 
/// old file， will in in 1 —— if in that ，
/// 1 to raw is before 、 to is after ， is ** not 
/// old file、 before that not raw**。 when up not can can （ and config
/// in `bootstrap` path up ）， 1 no has this ，
/// and 1 。
pub(super) fn read_snapshot(db: &Db) -> Option<(Map<String, Value>, bool)> {
    db.with(|conn| {
        let raw = read_conn(conn).ok()?;
        let migrated = marker_present_conn(conn, KEY_CONFIG_MIGRATED).unwrap_or(false);
        Some((raw, migrated))
    })
    .flatten()
}

/// 1 config；unavailable by failure up ， by up as 。
///
/// ** not **：`Db::with` is 1 that ，
/// `logging::log` need 1 `logs` —— `std::sync::Mutex` not can ，
/// in etc 。 this `Result` ， by use 
/// in （`config::save_raw` is this ）。
pub(super) fn write_all(db: &Db, raw: &Map<String, Value>) -> Result<(), String> {
    match db.with(|conn| write_conn(conn, raw)) {
        None => Err("数据库不可用".to_string()),
        Some(result) => result,
    }
}
