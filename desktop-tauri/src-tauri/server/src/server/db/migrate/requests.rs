//! `{request_stats_dir}/requests.jsonl` + `request-daily.jsonl` → `requests` /
//! `request_daily` 。
//!
//! old file** **（ not is 1 file）， by has ：
//! 1. **failure**： MB、 has KB，file failure
//! （ when 、 use ） not this that 1 1 ；
//! 2. ** etc **： has 「empty not empty」， 1 
//! and ， etc already to in 1 ；
//! 3. ** has **： need in after 1 （ `requests` ），
//! must in after —— when this in 
//! 1 can ， 1 in 。
//!
//! ── old file in ****directory when （ and `logs` 1 ）────────
//! `config::storage_dirs().request_stats_dir` is use in set can can ，
//! configdirectory is after —— `requestStatsDir` is after ， 1 in config
//! directory。 ， use set that （ and `logs` 1 ）。
//!
//! ── ────────────────────────────────────────────────
//! 、、 serde `default`（`attempts` by 1、
//! token by 0、`id` empty…）。 this ** not in this **， in 
//! `request_stats` （`parse_requests_jsonl` / `parse_daily_jsonl`）：
//! 「 1 JSON 」 is 1 （`record.rs` serde is 
//! ）， and **must** use 1 —— 1 ，
//! after is 「 1 old and to not 」。
//! this 「file in 、 when 、」。
//!
//! ── etc ────────────────────────────────────────────────────
//! empty（module header then ），check in ** before ** ——
//! empty when old file not ， use can in directory to 。
//!
//! ── after not 1 ────────────────────────────────────────
//! old in **** when to （ by and 、 by 
//! and limit）， up file down 1 —— old file is ，
//! that when after need by 1 1 。 in requeststatistics****： no has 、
//! no has limit， old file has ， not will 。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::logging;
use crate::server::request_stats::{self, RequestEntry};

/// request old file（`{request_stats_dir}/requests.jsonl`）。
///
/// in file 1 not is `request_stats` ： that 
/// already ** not in **（statistics）， is ——
/// is ， not will 。
const REQUESTS_FILE_NAME: &str = "requests.jsonl";
/// by old file（`{request_stats_dir}/request-daily.jsonl`）
const DAILY_FILE_NAME: &str = "request-daily.jsonl";

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
///
/// old file for ** **（ by module header）， has ，
/// and for 。
pub(super) const LABEL: &str = "请求明细";
pub(super) const DAILY_LABEL: &str = "请求聚合";

/// old file：`storage_dirs()` to directory（ that is use set ），
/// configdirectory after 。
///
/// Why need need after ：`requestStatsDir` is ** after ** can config， 
/// 1 in configdirectory；config directory will that use 。
/// `storage_dirs()` in // in when 「configdirectory」 ，
/// not to directory not will empty（configdirectory）。
///
/// when is ****（ `locate` and use ）：
/// 「 has no has 」 and is 1 path，
/// not will 「 has 、 not to file」。
fn locate(dir: &Path, name: &str) -> Option<PathBuf> {
    let configured = crate::server::config::storage_dirs().request_stats_dir;
    let candidates: [PathBuf; 2] = [configured.join(name), dir.join(name)];
    candidates.into_iter().find(|item| item.is_file())
}

/// old file （）
pub(super) fn requests_file(dir: &Path) -> Option<PathBuf> {
    locate(dir, REQUESTS_FILE_NAME)
}

/// old file （）
pub(super) fn daily_file(dir: &Path) -> Option<PathBuf> {
    locate(dir, DAILY_FILE_NAME)
}

/// old file and 。
///
/// failure and failure 1 and return `None` —— and `logs` 1 
/// （module header 「 not 」 then ）。「」 not is failure：
/// MB file has 1 is can ， use not 。
fn read_requests(path: &Path) -> Option<Vec<RequestEntry>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 请求明细迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    Some(request_stats::legacy::parse_requests_jsonl(&text))
}

/// old `{request_stats_dir}/requests.jsonl` → `requests` 。
///
/// ── `id` can ， not need need ────────────────────────────────
/// and that 1 ** **：`logs.id` is ， that need for file 
/// id （ `logs` ）。`requests.id` ** not is ** ——
/// is `row_id`（），`id` is request （ and ），
/// old can can is empty、 and ** not 1 **（、 if path will id
/// ， `db/schema.rs` DDL ）。 this `id`，
/// ；`row_id` by ， after is 。
/// ── etc ：Why** not can **「empty」（T11 after 1 ）──
/// module header then is 「empty」， that in ** in **
/// when is for 。 T11 ** use in 「」**， is in use 
/// after —— use can can in that before already use request（ already has ），
/// when 「empty」 will 「 already 」， that `requests.jsonl` 
/// ** not **（ old file not ， 1 「 has 
/// can 」）。`logs` already up this ， is 1 。
///
/// ** + **（`request_stats::legacy::MARKER_REQUESTS`，
/// that ）： no has available 1 `INSERT OR IGNORE`
/// （ is `row_id`， `id` is request id、 old can can is empty）。
pub(super) fn import_requests(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = requests_file(dir)?;
    match request_stats::legacy::marker_present(conn, request_stats::legacy::MARKER_REQUESTS) {
        Ok(true) => return None,
        Ok(false) => {}
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 请求明细迁移失败：读取迁移标记失败（{error}）"),
            );
            return None;
        }
    }
    let entries = read_requests(&path)?;
    if let Err(error) = request_stats::legacy::import_legacy_requests(conn, &entries) {
        logging::console_line(
            "[Storage]",
            &format!("❌ 请求明细迁移失败：写入数据库失败（{error}）"),
        );
        return None;
    }
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        // is ** **（file has ）： in not 
        // ， this and 1 ； will ， is 
        // 「file 」， not will then 。
        imported: entries.len(),
        backup,
    })
}

/// old `{request_stats_dir}/request-daily.jsonl` → `request_daily` ，
/// and in after 1 ****（ use ）。
///
/// ── Why in this （ 6 ）──────────────────────────
/// 「 `accountStats`」 can can ** old file**： in 1 by 
/// `fold_into_daily` ， as （ is empty），
/// not will 「 1 」 this 。 is is ** 1 
/// **， for 「 from old file that 」 has —— all，
/// not need need （ old `load()` that already T4 ）。
///
/// in ： need 、 1 「 has no has 」，
/// this in after 1 is 「 no has 」。** can not can **：
/// use not 「unknown」 ， not is ，
/// is use when （ `request_stats::legacy::import_legacy_daily` ）。
/// 「 not not 」（ >= requests ）。
///
/// need need already ，must in `import_requests` after （）。
pub(super) fn import_daily(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = daily_file(dir)?;
    // etc and （ + ， `import_requests` etc ）
    match request_stats::legacy::marker_present(conn, request_stats::legacy::MARKER_DAILY) {
        Ok(true) => return None,
        Ok(false) => {}
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 请求聚合迁移失败：读取迁移标记失败（{error}）"),
            );
            return None;
        }
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 请求聚合迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    let daily = request_stats::legacy::parse_daily_jsonl(&text);
    let (imported, backfilled) = match request_stats::legacy::import_legacy_daily(conn, &daily) {
        Ok(result) => result,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 请求聚合迁移失败：写入数据库失败（{error}）"),
            );
            return None;
        }
    };
    // ： this and use to can can not （ use is 
    // ， `backfill` module header ）， when need has 。
    // —— in `logging::init_store` before （module header）。
    if !backfilled.is_empty() {
        logging::console_line("[Storage]", &request_stats::legacy::backfill_report_line(&backfilled));
    }
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: DAILY_LABEL,
        source: path,
        // （ by ： this file has ）
        imported,
        backup,
    })
}
