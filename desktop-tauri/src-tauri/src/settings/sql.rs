//! set in `kv` ****（ `desktopSettings`）。
//!
//! ── Why 1 ， not is `closeToTray` / `autostart` / `proxyPort` ──
//! set is ****： before 1 PUT 、`AppSettings` is 1 
//! `Copy` 、`save_app_settings` is 。 will 「 1 」
//! 「 UPSERT + can can 」， this no has need 
//! —— and 「 1 」 in 。 `db::schema` module header 
//! （ that `desktopSettings` as 「 for 」 this 1 ）。
//!
//! ── Why this （ not is `Db`）─────────────────────
//! this is and its all store not ，：
//! ** this must can in `Db::open` before to set。**
//! `lib.rs` setup `settings::load()` `closeToTray`（ as ），
//! `gateway::proxy_port()` need —— server bind ，
//! `ServerState::bootstrap`（`Db::open` in that ）。 that when no has `Db` 
//! available，must in bind before has 。
//!
//! is this use ****： use to when 1 、/。 and ：
//! - `proxy_port()` ** is **（`gateway::ACTIVE_PORT` ，
//! `resolve_port` 1 1 ）→ 1 ；
//! - `lib.rs` `settings::load()` 1 ；
//! - its is use （`get_app_settings` / `save_app_settings` /
//! `change_port`）， 1 。
//! for SQLite is （ = 1 file）， is 
//! 「 in when available」 this can —— this is when 。
//!
//! ── Why not in this ──────────────────────────────────────
//! and schema `server::db::schema::migrate`（`Db::open` ），
//! that 1 。**file not in when will to 「 not in 」 error**， this not is 
//! is ：`settings::load` 「 not 」 1 by 「 no has this set」
//! （ old file / default）， in `Db::open` that 1 。
//!
//! [`write`] 1 `CREATE TABLE IF NOT EXISTS`： can can in `Db` in 
//! ** before ** use （ use in use 、`Db::open` no when 「set」），
//! that when no has this 。 this DDL and `schema::V1_SCHEMA` `kv` that 1 ，
//! and ** not schema ** —— `Db::open`。
//! （`load` not need need this ：failure by 「 no has this set」， up 1 。）
//!
//! ：** not PRAGMA `journal_mode`**。WAL is file （ in header
//! ），`Db::open` 1 after after ； in this 1 etc 
//! ， that not is 1 set this 。 `busy_timeout`（），
//! and up when SQLITE_BUSY（ when etc 5 seconds）。
//!
//! ── not 、 not ────────────────────────────────────
//! and its `sql.rs` 1 ：error 1 `Err` up ， by up in 。
//! this need Note —— `settings` in before can can use ，
//! that when `logging::log` that 1 will （）。

use std::path::Path;

use rusqlite::{Connection, OptionalExtension};

/// `kv` set （ `db::schema` module header， and in 
/// [`crate::server::db::schema::RESERVED_KV_KEYS`] —— config not will ）。
pub(crate) const KEY: &str = "desktopSettings";

/// 1 （ use ； by module header）。
fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    // ： in when etc 1 will not is SQLITE_BUSY
    conn.busy_timeout(std::time::Duration::from_millis(5_000))?;
    Ok(conn)
}

/// set（ not in 、 not in return `None`）。
///
/// by `Option<Option<String>>` ：`value` as `NOT NULL`，
/// 1 NULL is 1 can this ， that by 「 no has 」
/// （ old file / default， not is NULL 1 emptyset ）。
///
/// ** not in **（ new 、 no ） in this `None` not is error up ：
/// use for 「 not 」 and 「 no has this set」（），
/// up will use 1 。
pub(super) fn load(path: &Path) -> Option<String> {
    let conn = open(path).ok()?;
    let value: Option<Option<String>> = conn
        .query_row("SELECT value FROM kv WHERE key = ?1", rusqlite::params![KEY], |row| {
            row.get(0)
        })
        .optional()
        .ok()?;
    value.flatten()
}

/// set（UPSERT），** need when **。
///
/// use `ON CONFLICT DO UPDATE` not is `INSERT`： this can can already in （ use 
/// set） can can not in （ new use 1 ），path need can 。
/// ** this 1 is **， not need need （ 「 1 」 is this 1 ）。
///
/// ── Why `CREATE TABLE IF NOT EXISTS` ──────────────────
/// `schema::migrate`（`Db::open` ）， that need need `Db` ；
/// in `Db` in before can can use （ — module header）。 if that when no this ，
/// will 「no such table」failure。 1 etc 「set」 in when 
/// can success，** not schema **： by `Db::open` 
/// （`CREATE TABLE IF NOT EXISTS` etc ， after will up its and ；
/// file DDL and `schema::V1_SCHEMA` `kv` that 1 ）。
///
/// ── Whyfailure need use （ and `load` ）──────────────────
/// failure is use 「」 no up ，must —— return `Result` not is 
/// （`load` 「 not 」 then 1 by default， as that not use ）。
pub(super) fn write(path: &Path, text: &str) -> Result<(), String> {
    let conn = open(path).map_err(|error| format!("打开数据库失败: {error}"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS kv (
           key   TEXT PRIMARY KEY,
           value TEXT NOT NULL
         );",
    )
    .map_err(|error| format!("建表失败: {error}"))?;
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![KEY, text],
    )
    .map_err(|error| format!("写入设置失败: {error}"))?;
    Ok(())
}

