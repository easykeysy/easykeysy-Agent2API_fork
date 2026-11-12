//! `{debug_dir}/debug-traffic.jsonl` → `debug_traffic` 。
//!
//! ── old file in ****directory when ───────────────────────────
//! `config::storage_dirs().debug_dir` is use in set can can ，configdirectory
//! is after —— `debugDir` is after ， 1 in configdirectory。 ，
//! use set that （ and `logs` / `requests` 1 ）。
//!
//! ── ────────────────────────────────────────────────
//! 、empty、、** not panic**。 in 
//! `core::debug_traffic::parse_legacy_jsonl` not is file：「 1 JSON 」
//! is （`TrafficEntry` serde is ）， and must use 1 
//! —— 1 ， after is 「 1 old and 
//! to not 」。
//!
//! ── can empty （ need need ）─────────────────
//! `TrafficEntry` `status` / `response_headers` / `response_body` is `Option`，
//! 「 this 1 no to 」：request not when no has response。must ** can empty**
//! （NULL），** not can use empty** —— before this 
//! （`ui/requests-panel.js` `renderDetail`：`data.status == null` `-`，
//! `data.responseBody` as empty then not that 1 ）。 by 
//! `debug_traffic::sql::encode` 1 ， and use ， not will 。
//!
//! ── etc ────────────────────────────────────────────────────
//! empty（ then ），check in ** before ** —— empty when old file not 。
//!
//! ── after Why need 1 ────────────────────────────────────
//! old in **** when to （、total）， up file
//! down 1 —— old file is 。 and ：
//! `debug_traffic::import_legacy` 1 `sql::enforce_limits`
//! （、 1 ）， not can can 。
//!
//! ── `id` will ─────────────────────────────────────────
//! `id` is this （`schema.rs` ： and `requests.id` 、 is 
//! ， 1 request has 1 ）， old file is **** —— id in 
//! up can （ before `get` after that ， not ）。
//! this use ** INSERT**：、failurereturn `None`。
//! Why not UPSERT（ `record` that ）：「 after 」 will 
//! 1 ， and use not is 1 —— can not 、 old file
//! （`backup_legacy_file` in success after ，failure when file not ）， down or 
//! for when can to 。 this in not will （ id 1 
//! request ，path by `done` ）。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::core::debug_traffic;
use crate::server::logging;

/// old file（`{debug_dir}/debug-traffic.jsonl`）。
///
/// in file 1 not is `debug_traffic` ： that 
/// already ** not in **（）， is ——
/// is ， not will 。
const DEBUG_FILE_NAME: &str = "debug-traffic.jsonl";

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
pub(super) const LABEL: &str = "调试报文";

/// old file （ not in when `None`）—— and [`import_debug`] use this 1 。
///
/// ── directory（ and `logs` / `requests` 1 ）─────────
/// `config::storage_dirs().debug_dir` is use in set can can ，configdirectory
/// is after —— `debugDir` is after ， 1 in configdirectory。 ，
/// use set that 。
/// Why need need after ：config directory will that use 。
/// `storage_dirs()` in // in when 「configdirectory」 。
pub(super) fn legacy_file(dir: &Path) -> Option<PathBuf> {
    let configured = crate::server::config::storage_dirs().debug_dir;
    let candidates: [PathBuf; 2] = [configured.join(DEBUG_FILE_NAME), dir.join(DEBUG_FILE_NAME)];
    candidates.into_iter().find(|item| item.is_file())
}

/// old `{debug_dir}/debug-traffic.jsonl` → `debug_traffic` 。
///
/// ── ────────────────────────────────────────────────
/// 、empty、、** not panic**。 in 
/// `core::debug_traffic::parse_legacy_jsonl` not is file：「 1 JSON 」
/// is （`TrafficEntry` serde is ）， and must use 1 
/// —— 1 ， after is 「 1 old and 
/// to not 」。
///
/// ── can empty （ need need ）─────────────────
/// `TrafficEntry` `status` / `response_headers` / `response_body` is `Option`，
/// 「 this 1 no to 」：request not when no has response。must ** can empty**
/// （NULL），** not can use empty** —— before this 
/// （`ui/requests-panel.js` `renderDetail`：`data.status == null` `-`，
/// `data.responseBody` as empty then not that 1 ）。 by 
/// `debug_traffic::sql::encode` 1 ， and use ， not will 。
///
/// ── etc ────────────────────────────────────────────────────
/// empty（ then ），check in ** before ** —— empty when old file not 。
///
/// ── after Why need 1 ────────────────────────────────────
/// old in **** when to （、total）， up file
/// down 1 —— old file is 。 and ：
/// `debug_traffic::import_legacy` 1 `sql::enforce_limits`
/// （、 1 ）， not can can 。
///
/// ── `id` will ─────────────────────────────────────────
/// `id` is this （`schema.rs` ： and `requests.id` 、 is 
/// ， 1 request has 1 ）， old file is **** —— id in 
/// up can （ before `get` after that ， not ）。
/// this use ** INSERT**：、failurereturn `None`。
/// Why not UPSERT（ `record` that ）：「 after 」 will 
/// 1 ， and use not is 1 —— can not 、 old file
/// （`backup_legacy_file` in success after ，failure when file not ）， down or 
/// for when can to 。 this in not will （ id 1 
/// request ，path by `done` ）。
/// ── etc ：Why** not can **「empty」（T11 after 1 ）──
/// module header then is 「empty」， that in ** in **
/// when is for 。 T11 ** use in 「」**， is in use 
/// after —— use can can in that before request（ already has ），
/// when 「empty」 will 「 already 」， that `debug-traffic.jsonl`
/// ** not **（ old file not ， 1 「 has 
/// can 」）。`logs` already up this ， is 1 。
///
/// ** by etc **：`debug_traffic.id` is （ and `requests.id` 、
/// is ）， use `INSERT OR IGNORE` —— already in id not ， new 。
/// for ： when and id not （ before old file、
/// after is new request），； when IGNORE， not 。
pub(super) fn import_debug(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = legacy_file(dir)?;
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 调试报文迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    let entries = debug_traffic::parse_legacy_jsonl(&text);
    let (imported, skipped) = match debug_traffic::import_legacy(conn, &entries) {
        Ok(result) => result,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 调试报文迁移失败：写入数据库失败（{error}）"),
            );
            return None;
        }
    };
    if skipped > 0 {
        logging::console_line(
            "[Storage]",
            &format!("调试报文迁移：{imported} 条新导入、{skipped} 条已存在（重复导入自动跳过）"),
        );
    }
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        // is ** **（file has ）， not is after 
        // —— when will （ old file this is ），
        // is ** then ** in 、 not is ， this will use 
        // as （ and `logs` / `requests` 1 ）。
        imported: entries.len(),
        backup,
    })
}
