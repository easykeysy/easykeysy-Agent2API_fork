//! `{config_dir}/config.json` → `kv` ****。
//!
//! ── etc ：Why this 1 ** need need **（ and its not ）──────
//! its is 「 in not in 」：
//! - 1 （`accounts` / `logs` / `requests` / `request_daily` /
//! `debug_traffic`）「empty not empty」；
//! - `kv` 1 （`desktopSettings`）「 that in not in 」。
//! not ，Reason is **config is **：
//! - 「empty not empty」 not —— `kv` is （module header etc then already ）；
//! - 「 in not in 」 not —— no has 1 can config。 use 
//! `apiKeys` or is ： that can can no ；
//! - 「`kv` has no has 」 not —— 1 config will this ，
//! when can can no （ old file its no ）， will use old config
//! in file（`config::read_raw` path will 1 ，，
//! use 1 will no that in `.json` ）。
//! ** use ** 1 `configMigrated`（ in `kv` ， and in 
//! `db::schema::RESERVED_KV_KEYS`， not will config「 already config」
//! ）。
//!
//! ── Why this can 「 not use migratedXxx 」 that ──────
//! module header「 not use 」， by is ** and can can not 1 **
//! （ 1 ， is as ， that not up ）。
//! that by in ** not **， as and in ** 1 **
//! （`config::sql::import_conn`）：
//! - ⇒ config + in ；
//! - ⇒ not in ， down 。
//! 「、 1 」 this in in up not in 。：T6 for is 
//! **** ， not is 。 this is and 「」 that 
//! ， is 「 not can has not 」 that 1 。
//!
//! ── Why not delete（ and ）────────────────
//! `save_raw` is ****（ for file file）： new config no has 
//! need ，otherwise「empty 1 config」 in down not 。 is ****：
//! old file ， not delete already has 。 up delete will has 1 path
//! —— in `bootstrap` ， 1 will has in before config，delete will 
//! that 。 not delete then is 「 old file and 」， that is this 。
//! `config::sql::import_conn`。
//!
//! ── Why in ** 1 ** ──────────────────────────────
//! its need config：`import_logs` / `import_requests` / `import_debug` use 
//! `config::storage_dirs()` old file directory，`import_logs` use 
//! `config::retention_settings()` 。config after ， this 
//! `config` 「config already from old file」——`config::init` use 
//! file ** before ** （ `config::read_raw`），
//! after ** not **； 1 is 「 config，config
//! that 」 this in when 1 can ，「config already in 」 this 
//! for after and when 。
//!
//! ── failure ──────────────────────────────────────────────
//! filefailure / failure（ not is JSON for ）/ failure： 1 ❌ 
//! and return `None`，** not old file**（`backup_legacy_file` in success after ），
//! not 、 not panic。 down ， old file in can for 。
//! is `console_line` not is `logging::log`： in 
//! `logging::init_store` before （ `ServerState::bootstrap` ），
//! that when no ，`log()` that 1 will —— failure is 
//! use need need to （ and its 1 ）。

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde_json::Value;

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::config;
use crate::server::logging;

/// old configfile（`{config_dir}/config.json`）。
///
/// in file 1 not is `config::FILE_NAME`（ that already 
/// not in ）：file is ， and `debug.rs` `DEBUG_FILE_NAME` 1 。
const CONFIG_FILE_NAME: &str = "config.json";

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
pub(super) const LABEL: &str = "网关配置";

/// old file （ not in when `None`）。 and [`import_config`] 。
pub(super) fn legacy_file(dir: &Path) -> Option<PathBuf> {
    let path = dir.join(CONFIG_FILE_NAME);
    path.is_file().then_some(path)
}

/// old config `{config_dir}/config.json` → `kv` （ 1 ）。
pub(super) fn import_config(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = legacy_file(dir)?;
    // already → （ is ** in **， by module header）。check in old file
    // before ： already when file not 。
    match config::sql::marker_present_conn(conn, config::sql::KEY_CONFIG_MIGRATED) {
        Ok(false) => {}
        Ok(true) => return None,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 配置迁移失败：读取数据库失败（{error}）"),
            );
            return None;
        }
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 配置迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    // JSON ** for **：config is 1 for （ is config）。
    // / / JSON 「 not this 」， and file ——
    // not emptyconfig（ that will use old file ， no ）。
    let raw = match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map,
        Ok(_) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 配置迁移失败：{} 的顶层不是 JSON 对象", path.display()),
            );
            return None;
        }
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 配置迁移失败：{}（{error}）", path.display()),
            );
            return None;
        }
    };
    // `kv`， and in ** 1 **（module header）。
    let imported = match config::sql::import_conn(conn, &raw, config::sql::KEY_CONFIG_MIGRATED)
    {
        Ok(count) => count,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 配置迁移失败：{}（{error}）", path.display()),
            );
            return None;
        }
    };
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        // is ****： 1 config is 「 has config」，
        // 「 12 」「 1 」 can 
        // （ and `settings` by 1 for ）。
        imported,
        backup,
    })
}
