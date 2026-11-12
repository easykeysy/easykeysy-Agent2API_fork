//! `{config_dir}/desktop-settings.json` → `kv` `desktopSettings` 。
//!
//! ── etc ： in ─────────────────────────────────────────
//! `kv` is ****（ `priorityScope`、 `logsNextId`、
//! config `configMigrated` in ）， is 
//! 「** that in not in **」 not is 「 as empty」。module header
//! etc then （ that 1 for `kv` ）。
//!
//! ** not need need ** `configMigrated` that ：set is ** **
//! （set 1 ），「 that in not in 」 is ——
//! has config that need need （ `config.rs` module header）。
//! Note `desktopSettings` in `db::schema::RESERVED_KV_KEYS` ，
//! `config::save_raw` not will 「 already config」。
//!
//! ── Why this 1 is set** 1 ** （ no has and path）─────
//! `settings::load()` can can in `Db::open` ** before ** use （ ——
//! `settings` module header when ）， that 1 「 not to old 
//! file， to 」 。** that path already **， as in 
//! down in before （ `lib.rs` setup 1 
//! `closeToTray`， `Db::open` in `backend::ensure_ready` ，）， is 
//! 「 already in 」 —— after is old file not as `.migrated`、
//! that 「✅ set old already 」 not ， down （ and 
//! `backup_legacy_file` use ） this 。
//! `settings` module header「Why no has 」。
//!
//! ── Why in file ──────────────────────────
//! crate before in `settings` （ old file / / ）。
//! is 1 ， in headless not and —— need use 
//! file， and not 。 is 「 this JSON can not 
//! can by set」 （[`LegacyAppSettings`]） and `AppSettings` 
//! ****： to `AppSettings` when ，must this —— file
//! has to for ， this is crate 。
//!
//! is in ， 1 not can ：
//! - ****（ old file `kv`）；
//! - ****（ + ）—— this has to 。
//!
//! ── ────────────────────────────────────────────────────
//! success after old file `desktop-settings.json.migrated`（ and its 1 ）。
//! after path（ not to old file）， after will that 。
//!
//! ── failure ──────────────────────────────────────────────
//! failure / failure / failure： 1 ❌ and return `None`，
//! ** not old file**（ that file is after —— 1 ，
//! not ， use set ）。 not 、 not panic。
//! `console_line` not is `logging::log`： in before 
//! （ and its 1 ， `db::migrate` module header）。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::logging;
use crate::paths;

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
pub(super) const LABEL: &str = "桌面设置";

/// old file （ not in when `None`）。
///
/// path `paths::legacy_desktop_settings_file()`（ from `paths::config_dir()` ）
/// not is `dir.join(...)`：`dir` is use configdirectory， down 
/// ，「set in 」 this this has 1 （`paths` ），
/// 1 path 1 。 not 「configdirectory no has 」， use 
/// paths path —— and [`import_settings`] use this 1 。
pub(super) fn legacy_file(_dir: &Path) -> Option<PathBuf> {
    let path = paths::legacy_desktop_settings_file();
    path.is_file().then_some(path)
}

/// set `{config_dir}/desktop-settings.json` → `kv` `desktopSettings` 。
pub(super) fn import_settings(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    // and 「」 use 1 （ [`legacy_file`]）：`dir` ，
    // not 「configdirectory no has 」，path by settings to 。
    let path = legacy_file(dir)?;
    // already → （ is ** in **，module header）。check in old file before 。
    match key_present(conn) {
        Ok(false) => {}
        Ok(true) => return None,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 桌面设置迁移失败：读取数据库失败（{error}）"),
            );
            return None;
        }
    }
    let Some(text) = read_legacy_text(&path) else {
        logging::console_line(
            "[Storage]",
            &format!("❌ 桌面设置迁移失败：无法读取 {}", path.display()),
        );
        return None;
    };
    // not not （****： 1 not will 
    // —— `settings::load` in not old file， use will to 
    // setdefault， old file 。 by `legacy_parses` ）。
    if !legacy_parses(&text) {
        logging::console_line(
            "[Storage]",
            &format!("❌ 桌面设置迁移失败：{} 不是合法的设置 JSON（文件保留原处）", path.display()),
        );
        return None;
    }
    // 1 set is 1 （UPSERT）， is —— need 
    // 「 1 」 in this （ UPSERT is ）。
    if let Err(error) = write_migrated(conn, &text) {
        logging::console_line(
            "[Storage]",
            &format!("❌ 桌面设置迁移失败：写入数据库失败（{error}）"),
        );
        return None;
    }
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        // set is （ 1 ）， this as 1 ——
        // and `config` by in ： that 1 is 「 1 has N 」，
        // this 1 is 1 。
        imported: 1,
        backup,
    })
}

/// `kv` set （ and `settings::sql::KEY` 1 ；
/// `db::schema` module header， and in [`crate::server::db::schema::RESERVED_KV_KEYS`]
/// —— config not will ）。
const KEY: &str = "desktopSettings";

/// `kv` has no has `desktopSettings` （ etc ）。
///
/// `SELECT 1 ... LIMIT 1` not is ： in 。
fn key_present(conn: &Connection) -> rusqlite::Result<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM kv WHERE key = ?1 LIMIT 1",
            rusqlite::params![KEY],
            |row| row.get(0),
        )
        .ok();
    Ok(found.is_some())
}

/// use ： old file （** not **， to ）。
///
/// ： use unknown 1 and （ and config「
/// unknown」 not 1 ）。
fn read_legacy_text(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// use ： this old set can not can by set。
///
/// Why need 1 ： 1 not ， will ** **
/// —— has （ not ）， `settings::load` not will 
/// （ in return，failuredefault）， is use to is 「setdefault」，
/// old file 。 1 then can file in 、 not 
/// （ + old file， and its to JSON when 1 ）。
///
/// [`LegacyAppSettings`] is `AppSettings` ****（module header），
/// must this 。
fn legacy_parses(text: &str) -> bool {
    serde_json::from_str::<LegacyAppSettings>(text).is_ok()
}

/// `AppSettings` ：used for [`legacy_parses`] 。
///
/// and serde （`camelCase` + `default`）must and `AppSettings`
/// 1 —— 「 not 」， not （ `_` before ）。
/// ** `AppSettings` when this **（ to ）。
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct LegacyAppSettings {
    _close_to_tray: bool,
    _autostart: bool,
    _proxy_port: u16,
    _lan_access: bool,
    _lan_panel: bool,
    _lightweight_mode: bool,
    /// will （ is ）。 old file not will has ，`default` 。
    _panel_session: LegacyPanelSession,
}

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct LegacyPanelSession {
    _refresh: String,
    _access: String,
}

impl Default for LegacyAppSettings {
    fn default() -> Self {
        Self {
            _close_to_tray: true,
            _autostart: false,
            _proxy_port: 0,
            _lan_access: false,
            _lan_panel: false,
            _lightweight_mode: false,
            _panel_session: LegacyPanelSession::default(),
        }
    }
}

/// use ： 1 old set（** use to **）。
///
/// Why not 「」 ： must and its 1 
/// in that ，「 not when old file need not need 」 this need and in 
/// 1 up down 。UPSERT and `settings::sql::write_conn` 1 ；
/// not —— in `Db::open` ，schema already 。
fn write_migrated(conn: &Connection, text: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![KEY, text],
    )?;
    Ok(())
}
