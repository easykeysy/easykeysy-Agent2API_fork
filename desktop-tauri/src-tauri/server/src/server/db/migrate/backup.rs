//! old file ： already old file****。
//!
//! this 1 ：all need in success after use ，「、
//! 、failure」 must 1 for —— in 1 
//! （ down ）。

use std::path::{Path, PathBuf};

use crate::server::logging;

/// （） 1 already old file，return after path。
///
/// is `{file}.migrated`（`accounts.json` → `accounts.json.migrated`）；
/// if this already use ， use `{file}.migrated-{epochseconds}`。
///
/// ── Why is not is delete ──────────────────────────────────
/// is ** not can ** ： 1 and old file for not up ， no has can 
/// 。 old file in （ is ） use when can 
/// or for ； is configdirectory `.migrated` file。delete is not can 
/// ， no has when —— this not can 。
///
/// ── Why not is ────────────────────────────────────
/// is 1 file （、 not ）； need as 
/// old file 1 （`requests.jsonl` MB）， and after 
/// 「file」 is 1 not can 。 1 to ，「 1 old file
/// not will 」—— when in path up already not in 。
///
/// ── when after （Why not ）─────────────
/// `-{epochseconds}` in before 1 when will use to 。 need up ， need need in 1 seconds
/// for ** 1 ** old file， 1 already —— 
/// `is_file()` not ， not to this 。 not old file not ，
/// not not can can 。 not 「」 ： that is 
/// not to （ for to is by not is ）。
///
/// return `None` ：**file not in **（path —— already or no has ），
/// and **failure**（、 not 、 use ）。 after is ， will 1 
/// ； is ： in ， already ， and down will 
/// as 「empty」， not will 。
///
/// use `OsString` not is `to_str()`：Windows up path not is UTF-8，
/// for this `to_str()` return `None`， will （ is ）。
/// and none， by / up can 。
///
/// use ：all（ / / request / request / /
/// ）。must**** 1 use for （、、failure
/// is ， in 1 has ）。
pub(super) fn backup_legacy_file(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    let name = path.file_name()?.to_os_string();
    let mut target_name = name.clone();
    target_name.push(".migrated");
    let mut target = path.with_file_name(target_name);
    if target.exists() {
        // epoch seconds：「 not is 1 」， not need need ms
        // （ 1 seconds in up 1 already not can can ）
        let stamp = logging::now_ms() / 1000;
        let mut stamped = name;
        stamped.push(format!(".migrated-{stamp}"));
        target = path.with_file_name(stamped);
    }
    match std::fs::rename(path, &target) {
        Ok(()) => Some(target),
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("⚠️  旧文件改名失败（数据保留在原处）: {} → {} ({error})", path.display(), target.display()),
            );
            None
        }
    }
}
