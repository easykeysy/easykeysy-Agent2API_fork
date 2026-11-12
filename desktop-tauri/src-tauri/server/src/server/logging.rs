//! ： + （「」）。
//!
//! for Node server.mjs 174-240 emit/log/logEvent：
//! - `[HH:mm:ss.SSS] TAG text`， when use **UTC**
//! （ for Node `new Date().toISOString().slice(11, 23)`， not is when ）
//! - when to ， by /（inferLevel）
//! - `verbose` （debug） has --verbose ，
//!
//! Why use `eprintln!` not is `println!`： is GUI ，Windows down 
//! no has when `println!` can can panic（release is panic=abort，
//! will use ）。stderr failure will ignore， is 。
//!
//! ： in when 1 ， after all use （Node is 
//! 1 logStore ）。 use `OnceLock` not is `Mutex<Option<...>>`，
//! pathnone； by LogStore 。

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{TimeZone, Utc};
use serde_json::Value;

use crate::server::db::Db;
use crate::server::logs_store::{category_for_tag, LogEntry, LogStore, NewEntry};

/// 1 。 when by `init_store` 。
static STORE: OnceLock<LogStore> = OnceLock::new();

/// is need need debug up （ for Node `opts.verbose`）
static VERBOSE: OnceLock<bool> = OnceLock::new();

/// file：`Some(...)` = already enabled（），`None` = not enabled（ before ）。
///
/// ── Why need need ────────────────────────────────────────────
/// Windows ** no has **：`eprintln!` to is not in stderr，
/// 。「 not 」 etc no ，
/// in SCM before use [`init_file_sink`]， that 1 to file。
///
/// use `OnceLock<Option<Mutex<File>>>` not is `OnceLock<Mutex<File>>`：
/// failure（directory not can 、path） when need can —— 「 has 
/// stderr」， not is panic。
static FILE_SINK: OnceLock<Option<Mutex<File>>> = OnceLock::new();

/// filelimit： `.1`（ 1 ）。
/// that by ，file is 1 ， need need limit。
const LOG_FILE_MAX_BYTES: u64 = 8 * 1024 * 1024;

/// file。default `{logs_dir}/server.log`，available
/// `AGENT2API_LOG_FILE` （ for path by CWD ）。
///
/// use 1 （OnceLock ）。failure not panic：
/// 「 has stderr」—— stderr in down is empty ，failure when will stderr
/// 1 （ before when ）。
///
/// Why by ****：（SCM / systemd Restart=） after 
/// up 1 not can —— 「Why」 when that is need 。
pub fn init_file_sink() {
    let path = log_file_path();
    let sink = open_log_file(&path).map(Mutex::new);
    if sink.is_none() {
        eprintln!(
            "⚠️  日志文件打不开（{}）：服务形态下将没有运行日志",
            path.display()
        );
    }
    let _ = FILE_SINK.set(sink);
}

/// 。 use 1 （OnceLock ），return is success。
///
/// in `ServerState::bootstrap` use —— after 、 its before ，
/// after its can 。
///
/// ── parameterWhy from `directory` `Db` ─────────────────────
/// in in 1 `logs` （ from `logs.jsonl` ），
/// not has 「 directory」。 and T2 `AccountStore::with_db(db)` 1 ：
/// path by `Db` 1 has ， is 1 use 。
/// `Option<Db>` is as `ServerState::bootstrap` is `Option<Db>`
/// （ not when need can ）：`None` when ，、
/// returnempty —— `LogStore` 。
///
/// ****（ when `config::retention_settings()`）：
/// is set， down 1 / prune ， not need need 
/// （ and `RequestStats` `get_retention` 1 ）。
/// is config **** not is —— is for path。
pub fn init_store(db: Option<Db>, verbose: bool) -> bool {
    let _ = VERBOSE.set(verbose);
    if STORE.get().is_some() {
        return false;
    }
    // fork：Linux + when `{logs_dir}/events.jsonl`（
    // `agent2api.db`）； its （Windows / / ） use after ， as and 
    // before 1 。 is directory is `/mnt/ramdisk` ，
    // `paths::side_dirs_external`。
    let store = if crate::paths::side_dirs_external() {
        crate::paths::ensure_side_dirs();
        LogStore::with_events_file(db, crate::paths::logs_dir().join("events.jsonl"), || {
            crate::server::config::retention_settings().log_days
        })
    } else {
        LogStore::with_db(db, || {
            crate::server::config::retention_settings().log_days
        })
    };
    store.prune();
    let _ = STORE.set(store);
    true
}

/// ； when return None（ when has ）
fn store() -> Option<&'static LogStore> {
    STORE.get()
}

/// is verbose（default， and Node 1 ）
pub fn is_verbose() -> bool {
    *VERBOSE.get().unwrap_or(&false)
        || crate::server::config::with_raw(|raw| raw.get("debugMode").and_then(Value::as_bool) == Some(true)).unwrap_or(false)
}

/// currentms Unix when （ for Node `Date.now()`）。
/// `ts` use ； when used for and ， not can use when 。
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

/// when （ms Unix when ； use when ）。
///
/// request use 「 in request」 and 「 up 1 」：
/// when in not can can to （ `request_stats` 
/// `sweep_stale_running`）。
pub fn process_started_at() -> i64 {
    static STARTED: OnceLock<i64> = OnceLock::new();
    *STARTED.get_or_init(now_ms)
}

/// when before ：`[HH:mm:ss.SSS]`，**UTC**（ for Node toISOString）。
fn timestamp_prefix() -> String {
    match Utc.timestamp_millis_opt(now_ms()).single() {
        Some(now) => now.format("[%H:%M:%S%.3f]").to_string(),
        None => "[--:--:--.---]".to_string(),
    }
}

/// → ； to server（ for Node `TAG_CATEGORY[tag] || 'server'`）
fn category_of(tag: &str) -> &'static str {
    category_for_tag(tag).unwrap_or("server")
}

/// from and ， Node inferLevel then ， not regex ：
/// - ❌ / 「failure」/ ❌ → error
/// - or ⚠️ → warn
/// - its info
fn infer_level(tag: &str, text: &str) -> &'static str {
    // Node /\bfailure\b/ in in etc 「failure」（ in is ）
    if text.starts_with('❌') || text.contains("失败") || tag.starts_with('❌') {
        return "error";
    }
    if text.starts_with('⚠') || tag.starts_with('⚠') {
        return "warn";
    }
    "info"
}

/// filepath：`AGENT2API_LOG_FILE` ，default `{logs_dir}/server.log`
fn log_file_path() -> PathBuf {
    if let Ok(value) = std::env::var("AGENT2API_LOG_FILE") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    crate::paths::logs_dir().join("server.log")
}

/// filepath for （set「」 use ）。
///
/// and [`log_file_path`] 1 ： when use need is 「 to in 」，
/// that can can `AGENT2API_LOG_FILE` —— path will 
/// to 1 emptyfile up 。
pub fn log_file_path_public() -> PathBuf {
    log_file_path()
}

/// （ and by need ）file。
///
/// use `.1` after old that 1 ， not —— need use is 
/// 「 this Why not 」， 1 ； need 。
fn open_log_file(path: &Path) -> Option<File> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir).ok()?;
        }
    }
    let too_big = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) > LOG_FILE_MAX_BYTES;
    if too_big {
        // before can ：Windows up rename to already in file will failure
        let rotated = path.with_extension("log.1");
        let _ = std::fs::remove_file(&rotated);
        let _ = std::fs::rename(path, &rotated);
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .ok()
}

/// file 1 。enabled when is empty；failure（ not can ，
/// that will to `console_line`，）。
fn write_file_line(line: &str) {
    let Some(Some(sink)) = FILE_SINK.get() else {
        return;
    };
    if let Ok(mut file) = sink.lock() {
        use std::io::Write;
        let _ = writeln!(file, "{line}");
    }
}

/// （ not ）。 when use ，
/// otherwise「failure」 this will 1 ，。
///
/// file when when 1 to file（ no has ）。
pub fn console_line(tag: &str, text: &str) {
    let line = format!("{} {tag} {text}", timestamp_prefix());
    eprintln!("{line}");
    write_file_line(&line);
}

/// （ for Node log()）。
///
/// before will and Node ： parameter use empty after trim，
/// empty（）。 by infer_level 。
pub fn log(tag: &str, text: &str) {
    console_line(tag, text);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    let Some(store) = store() else {
        return;
    };
    // and Node 1 ：message as 「 + 」， before can to 
    let message = format!("{tag} {trimmed}");
    crate::server::core::blocking::in_place(|| store.append(NewEntry {
        level: infer_level(tag, trimmed),
        category: category_of(tag),
        message: &message,
        data: None,
        ts: None,
    }));
}

/// ， its as and `log()` 1 。
///
/// total（「success X， Y，failure Z」） must by **** not is 
/// ：failure as 0 has 「failure」，`infer_level`
/// will error（statistics error， etc 1 when ）。
/// use has failure， by `"info"` / `"error"` can 。
pub fn log_with_level(tag: &str, text: &str, level: &str) {
    console_line(tag, text);
    if matches!(level, "debug" | "verbose" | "trace") && !is_verbose() {
        return;
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    let Some(store) = store() else {
        return;
    };
    let message = format!("{tag} {trimmed}");
    crate::server::core::blocking::in_place(|| store.append(NewEntry {
        level,
        category: category_of(tag),
        message: &message,
        data: None,
        ts: None,
    }));
}

/// ： has when （ for Node verbose()）。
/// ， when use can to 。
pub fn verbose(tag: &str, text: &str) {
    console_line(tag, text);
    if !is_verbose() {
        return;
    }
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }
    let Some(store) = store() else {
        return;
    };
    let message = format!("{tag} {trimmed}");
    crate::server::core::blocking::in_place(|| store.append(NewEntry {
        level: "debug",
        category: category_of(tag),
        message: &message,
        data: None,
        ts: None,
    }));
}

/// up （ for Node logEvent）。
///
/// 429 this need need this ， can by category/data
/// and （logs-panel.js `data.from` / `data.to` / `data.resetAtText`）。
///
/// `message` as empty when return None and not 。 not —— this 
/// use already use `log()` 1 can 。
pub fn log_event(
    level: &str,
    category: &str,
    message: &str,
    data: Option<Value>,
) -> Option<LogEntry> {
    if matches!(level, "debug" | "verbose" | "trace") && !is_verbose() {
        return None;
    }
    let store = store()?;
    crate::server::core::blocking::in_place(|| store.append(NewEntry {
        level,
        category,
        message,
        data: data.as_ref(),
        ts: None,
    }))
}

/// by （/statistics/empty）。 when return None，
/// by return 503「enabled」， and Node as 1 。
pub fn store_ref() -> Option<&'static LogStore> {
    store()
}

#[cfg(test)]
mod tests {
    use super::infer_level;

    /// 1 **success** total（`failed == 0`）， 。
    /// 「failure」 —— as 「failure {failed} 」 is none 。
    const IMPORT_SUMMARY_ALL_OK: &str =
        "📥 账号导入完成: 共 2 条，新增 2 个，更新 0 个，跳过 0 条，失败 0 条\
         （自定义提供商定义：新增 0 家、更新 0 家）";

    /// this is 「** will **」 this ， use this 。
    ///
    /// need not is `infer_level` —— by is for ， is 
    /// **total**。total must by use by to （`log_with_level`），
    /// this 。`core/account_transfer.rs` total use `log()`，
    /// is success 1 1 error，statistics error
    /// （`ui/app.js` `updateLogsBadge`： has error）——
    /// etc 。
    #[test]
    fn infer_level_misreads_successful_import_summary_as_error() {
        assert_eq!(infer_level("[Accounts]", IMPORT_SUMMARY_ALL_OK), "error");
    }

    /// failuremust is error —— `log_with_level` after by use by `failed` to ，
    /// this 「 by to 」 。
    #[test]
    fn infer_level_keeps_real_failures_as_error() {
        assert_eq!(infer_level("[Accounts]", "❌ 账号导入失败"), "error");
        assert_eq!(
            infer_level("[Checkin]", "定时签到完成: 0/2 个账号成功领取，失败 1 个"),
            "error"
        );
    }

    /// success not error，otherwise etc 。
    #[test]
    fn infer_level_treats_plain_success_as_info() {
        assert_eq!(infer_level("[Checkin]", "⏰ 定时签到开始（到点触发）"), "info");
        assert_eq!(
            infer_level("[Checkin]", "定时签到完成: 2/2 个账号成功领取"),
            "info"
        );
    }
}

