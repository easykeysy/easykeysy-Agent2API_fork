//! `{log_dir}/logs.jsonl` → `logs` 。
//!
//! （directory、、 after 、id ） in down 
//! ； then [`super`] module header。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::logs_store::{self, LogEntry};
use crate::server::logging;

/// old file（`{log_dir}/logs.jsonl`）。
///
/// `ACCOUNTS_FILE_NAME`：`logs_store` after not has this （），
/// is —— is ， by has 。
const LOGS_FILE_NAME: &str = "logs.jsonl";

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
pub(super) const LABEL: &str = "事件日志";

/// old file （ not in when `None`）—— and [`import_logs`] use this 1 。
///
/// ── old file in **** log_dir when （ need need ）──
/// to `dir: &Path`（configdirectory），directory is use in set can 
/// （ in config.json `logDir`， by `config::storage_dirs()` ）。
/// ：
/// 1. `run_legacy_migrations` `storage_dirs()` —— need 、
/// use （`ServerState::bootstrap`）， and ** 1 use not up 
/// parameter**（、requeststatistics、 this old file in directory，
/// will 1 need ）；
/// 2. ** `config::storage_dirs()`** —— not 。
/// 2。 before in this ：
/// - `config::init()` already in `Db::open` ** before **（`bootstrap` ），
/// `storage_dirs()` to already use `logDir`；
/// - this in // in when 1 configdirectory（ ），
/// not to directory not will empty —— 1 default as 。
/// ：`db/migrate` 1 for `config` ， is 。
/// this is can —— 「 old file in 」 is **** ， not is ；
/// `requests` / `debug` after by 1 directory（ that 
/// `locate` / `legacy_file`）， and 1 。
///
/// ── ────────────────────────────────────────────────
/// `storage_dirs()` to directory（ that is use set ），configdirectory after ——
/// can can in that ， use after set。Why need need after ：`logDir`
/// is ** after ** can config， 1 in configdirectory； `log_dir`
/// will that use 。
///
/// this ** when **： when file， and 「 has no has 
/// 」。 use ， not will 「 has 、 not to file」。
pub(super) fn legacy_file(dir: &Path) -> Option<PathBuf> {
    let log_dir = crate::server::config::storage_dirs().log_dir;
    let candidates: [PathBuf; 2] = [log_dir.join(LOGS_FILE_NAME), dir.join(LOGS_FILE_NAME)];
    candidates.into_iter().find(|item| item.is_file())
}

/// old file `{log_dir}/logs.jsonl` → `logs` 。
///
/// ── etc ：Why** not can **「empty」（ 1 ）────────
/// module header then is 「empty」， for ** not **：
/// `logs` not is 「 has 」 —— ** 1 in **。 use after 
/// ms has （`[Server] … in `、`[Config] API `
/// etc ）， in by use in 「」（T11 ）， that when 
/// ** already has **。 is 「empty」 will 「 already 」：
/// old file no → `has_pending` 1 as true → 「 has 
/// can 」，， use to is 1 ** not **
/// （ up ：7 success、「」）。
///
/// ** by etc **： old file、 INSERT， use 
/// `INSERT OR IGNORE` —— already in id not 、 not ， new 。
/// this for ：
/// - ****： has （id from 1 ）， old file id is 
/// （ 4297..4856）， not ，；
/// - ****： old file id already in → IGNORE， 1 not ，
/// and 「」 etc ，** not will **（ this is not to ）。
///
/// Why not use （`kv.logsMigrated` ）： and can can not 1 
/// （ 1 in ， that when must will ），
/// 「 already in 」 is ， not will —— and module header 
/// etc then 1 to 。
///
/// ── and ** 1 ** ─────────────────────────────
/// 、；`message` by empty、`level`/`category` 
/// `logs_store` 1 、`data` `normalize_data` 、`id` when 
/// by ****（ old `load()` is `parsed.len() as u64 + 1`）。
/// this from `logs_store` has as `pub(crate)` and use ——
/// 1 ， after is 「 1 and 
/// to not level/category」。
///
/// ── after Why need 1 ────────────────────────────────────
/// old file can can already 、 or `MAX_ENTRIES` （ old is 
/// when `load()` to ，file down 1 —— ** up 
/// is **）。 will after 。
/// and ： by when （`logs_store::LogStore::retention_cutoff_ms`
/// ， this use 1 ， down ）， by 。
///
/// ── failure and error ──────────────────────────────────────────
/// and `import_accounts` ：allfailure 1 and return `None`
/// （module header 「 not 」 then ）。 in ** 1 ** —— in not will down ，
/// down 「」 can 。
pub(super) fn import_logs(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = legacy_file(dir)?;

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 日志迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    let entries = parse_jsonl(&text);
    let (imported, skipped) = match write_logs(conn, &entries) {
        Ok(result) => result,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 日志迁移失败：写入数据库失败（{error}）"),
            );
            return None;
        }
    };
    let backup = backup_legacy_file(&path);
    // ：「」 when is （ IGNORE），
    // 「 0 、 560 」 is 「 this no 」 ——
    // when `imported` has use （`imported` is file ，
    // 1 ）。
    if skipped > 0 {
        logging::console_line(
            "[Storage]",
            &format!(
                "事件日志迁移：{imported} 条新导入、{skipped} 条已存在（重复导入自动跳过）"
            ),
        );
    }
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        // `imported` is ** **（ old file has ），
        // not is after —— when when / will 1 and 
        // （ old file this is ， `write_logs` ）。
        // and `import_accounts` 1 （ that is file ）：
        // need 「 from old file」， is ** then ** in 、
        // not is ， this will use as 。
        imported: entries.len(),
        backup,
    })
}

/// old JSONL，return 1 after （** and old `LogStore::load` 1 **）。
///
/// 「 not 」 ：empty；JSON failure or not is for ；
/// when by old default（`message` empty、`level` info、`category`
/// server、`ts` 0、`id` by ）。
///
/// **Note this （ has ） not id **： old `load()` `Vec` id ，
/// this ；「 id to up will 」 this by [`write_logs`] in 
/// when （ that Whymust in that 、 and is ）。
/// to this will 「」 and 「」 —— is need 
/// file。
fn parse_jsonl(text: &str) -> Vec<LogEntry> {
    let mut parsed: Vec<LogEntry> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        let Some(object) = value.as_object() else {
            continue;
        };
        let message = object
            .get("message")
            .map(|item| item.as_str().unwrap_or_default().to_string())
            .unwrap_or_default();
        let level = object.get("level").and_then(serde_json::Value::as_str).unwrap_or("info");
        let category = object
            .get("category")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("server");
        parsed.push(LogEntry {
            // id when by **** —— old `load()` is 
            // `parsed.len() as u64 + 1`（Note is 「 already down 」 not is ：
            // after will before ， and old 1 ）
            id: object
                .get("id")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(parsed.len() as u64 + 1),
            ts: object.get("ts").and_then(serde_json::Value::as_i64).unwrap_or(0),
            level: logs_store::normalize_level(level),
            category: logs_store::normalize_category(category),
            message: logs_store::clamp_message(&message),
            data: logs_store::normalize_data(object.get("data")),
        });
    }
    parsed
}

/// `logs` （** 1 **， after ）。
///
/// ： → by when → by → `kv` to 
/// `MAX(id) + 1`。 after 1 is **must **： `append` this id，
/// if in 1， （id can can to ） will new ——
/// that new （`INSERT` failure → → `append` return None）。
///
/// ── id Why need 「 when 」 not is file ────────
/// old `load()` `Vec`，id is up 1 —— 、、
/// as 0 not can 。 this `id` is ****， after not is 「 that 1 
/// has 」 is **failure**：`INSERT` UNIQUE → → use 
/// 1 not （ to this path）。 old file id is ** can ** ：
/// `parse_jsonl` id is 「 already down + 1」， is from 1 
/// 1 file（、 or from 1 ） will 。
/// this ， not to **currentvalid id up **（`assign_log_id`）：
/// not （ old in ， this in ）， not and id 。
///
/// ── 「」 when not （`INSERT OR IGNORE`）────────
/// `import_logs` etc ：** not can **「empty」（ 1 in 
/// this ， will 、 old file not 、「 has can 
/// 」）。 as by etc —— use `INSERT OR IGNORE`， already in id not 。
///
/// by 1 **must** ：`used` need ** already has id**。
/// not after ： when `assign_log_id` to is 「empty」，
/// is old 「id use 」 → if this id already in ，
/// `OR IGNORE` will ignore（ this no ）；**file id ** that 
/// will to `max_valid + 1` —— that can can is empty ，
/// is ** **（ 1 ，）。
/// after 「 already has id」 and ， when to IGNORE ，
/// 1 not 。
///
/// return `(, already in )`： after use 
/// 「 this its no 」（ ），。
fn write_logs(conn: &Connection, entries: &[LogEntry]) -> rusqlite::Result<(usize, usize)> {
    let tx = conn.unchecked_transaction()?;
    // already use id + 。 from 「valid id + 1」， 
    // id 1 file 1 id —— and old 「 after id 」
    // to 1 （ before id already ， to not will new as already ）。
    let mut used: HashSet<i64> = HashSet::new();
    // already has id（ by 「」 1 ）
    {
        let mut stmt = tx.prepare("SELECT id FROM logs")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            used.insert(row.get::<_, i64>(0)?);
        }
    }
    let max_valid = entries
        .iter()
        .map(|item| item.id)
        .filter(|id| *id > 0 && *id <= i64::MAX as u64)
        .max()
        .unwrap_or(0);
    let mut next_free = max_valid.saturating_add(1).min(i64::MAX as u64);
    // already has id 「」， not can in up 
    //（otherwise id ， that will IGNORE —— 1 ）
    while used.contains(&(next_free as i64)) && next_free < i64::MAX as u64 {
        next_free += 1;
    }
    let mut imported = 0usize;
    let mut skipped = 0usize;
    for entry in entries {
        let id = assign_log_id(entry.id, &mut used, &mut next_free)?;
        // `data` and 1 （JSON / NULL）。
        // `OR IGNORE`：id already in （ ）， not 、 not 
        // —— will that （ can can ），
        // ，ignore is 1 。
        let changed = tx.execute(
            "INSERT OR IGNORE INTO logs (id, ts, level, category, message, data) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id,
                entry.ts,
                entry.level,
                entry.category,
                entry.message,
                entry.data.as_ref().map(|item| item.to_string()),
            ],
        )?;
        if changed > 0 {
            imported += 1;
        } else {
            skipped += 1;
        }
    }
    // after to （ old file/ is ， `import_logs` ）。
    // when use that ：`logs_store` cutoff ，
    // this no has `LogStore` —— `config::retention_settings()`
    // by ** 1 ** 1 。must： this if （`clamp`），
    // config will empty（ that is `RETENTION_*_DAYS` need ）。
    let settings = crate::server::config::retention_settings();
    let days = settings
        .log_days
        .clamp(
            crate::server::config::RETENTION_MIN_DAYS,
            crate::server::config::RETENTION_MAX_DAYS,
        );
    let day = chrono::Local::now().date_naive() - chrono::Duration::days(days - 1);
    let cutoff = day
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| chrono::TimeZone::from_local_datetime(&chrono::Local, &naive).earliest())
        .map(|value| chrono::DateTime::timestamp_millis(&value))
        .unwrap_or(0);
    tx.execute("DELETE FROM logs WHERE ts < ?1", params![cutoff])?;
    logs_store::sql_trim_capacity(&tx)?;
    // to MAX(id) + 1（ by ）。`raise_next_id` in when ，
    // this to 「 down 1 available id」 not is 「current id」。
    let max: Option<i64> = tx.query_row("SELECT MAX(id) FROM logs", [], |row| row.get(0))?;
    let next = max.unwrap_or(0).max(0) as u64 + 1;
    logs_store::sql_raise_next_id(&tx, next)?;
    tx.commit()?;
    Ok((imported, skipped))
}

/// to 1 1 **available** id： and use use ，otherwise to empty。
///
/// 「」 （ for `decode_row` can `u64` ）：
/// `id > 0`（ old id from 1 ，0 is 「 no 」 ）、
/// `id <= i64::MAX`（SQLite INTEGER is has 64 ， not ）、
/// and not in `used` （ will failure， `write_logs` ）。
///
/// ── Why（`next_free` use after ）─────────────
/// path is in `next_free`（= filevalid id + 1） up ** after **，
/// is id file id ，「 after id 」
/// this before to 。 can can 1 to `i64::MAX`（file has 1 
/// i64::MAX 、 or 1 id）： that when if ， use 1 
/// 1 not —— its empty。
/// from 1 ** **： when 「 id 」 this （ is to ，
/// not is ）， is 「 not 」—— this need 。
///
/// has （ id empty 1 empty no has ， not can can in file）
/// ： can not ， not need 1 id 。
/// use `ToSqlConversionFailure` not is `InvalidQuery`： after in rusqlite 
/// is 「 this SQL not is SELECT」，「 not available id」 will 
/// SQL up （error will failure）。
fn assign_log_id(
    wanted: u64,
    used: &mut HashSet<i64>,
    next_free: &mut u64,
) -> rusqlite::Result<i64> {
    let usable = wanted > 0 && wanted <= i64::MAX as u64 && !used.contains(&(wanted as i64));
    let id = if usable {
        wanted as i64
    } else {
        match find_free_id(used, *next_free) {
            Some(candidate) => {
                *next_free = candidate as u64 + 1;
                candidate
            }
            // use → from 1 （ up ）
            None => match find_free_id(used, 1) {
                Some(candidate) => {
                    *next_free = candidate as u64 + 1;
                    candidate
                }
                None => {
                    return Err(rusqlite::Error::ToSqlConversionFailure(
                        "旧日志文件里的 id 已占满可用范围，无法为冲突条目改派 id".into(),
                    ))
                }
            },
        }
    };
    used.insert(id);
    Ok(id)
}

/// from `start` 1 not in `used` id（`None` = from this to 
/// `i64::MAX` ）。 in up —— `used` is 
/// file （limit），empty is 。
fn find_free_id(used: &HashSet<i64>, start: u64) -> Option<i64> {
    let mut candidate = start.max(1);
    while candidate <= i64::MAX as u64 {
        let value = candidate as i64;
        if !used.contains(&value) {
            return Some(value);
        }
        candidate += 1;
    }
    None
}

// this has 1 `count_logs`（「empty」 etc ）。** already delete**：
// that for `logs` not —— 1 in （），
// is will 「 already 」、 old file not 、
// 「 has can 」（ up ）。 in `INSERT OR IGNORE`
// by etc ， not need need 「empty not empty」 this —— `import_logs` etc 。
