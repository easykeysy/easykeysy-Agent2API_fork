//! / ： and need 。
//!
//! `archive` 、Argon2id and ；`keys` and will 
//! new to ；`restore` 、 and failure。
//! old SQL can ， not ， not when 。
//!
//! use SQLite Backup 1 ， by SQL，
//! when use 。 SQL authorizer ATTACH、file after 
//! and unknown，all in has ， not can to 。
//! before use Backup API ； not will 。
//!
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use chrono::Local;
use rusqlite::backup::{Backup, StepResult};
use rusqlite::hooks::{AuthAction, AuthContext, Authorization};
use rusqlite::Connection;
use serde_json::{json, Value};

use super::Db;
use crate::server::core::crypto;
use crate::server::logging;

mod archive;
mod keys;
mod restore;
mod safety;
mod sql_import;
mod stream;
mod temp;

pub use archive::{BACKUP_EXTENSION, MAX_BACKUP_BYTES};
pub use temp::transfer_file;
use temp::TempRestoreDb;

/// file 1 。 this before file —— use its SQL
/// （、），「」。
pub const BACKUP_SQL_HEADER: &str = "-- Agent2API SQLite 导出";

/// file「 when 」 before （ in after ）。
/// in `BACKUP_SQL_HEADER` after ， is ****， not 「 before 」 。
const KEY_FINGERPRINT_LINE: &str = "-- account-key-fingerprint:";

/// when not in when （ for `crypto::key_fingerprint` return
/// `None` ）。 1 can not is empty：empty will 「 no this 1 」 and 「 has as empty」
/// in 1 ，none「 old no 」 and 「 no has 」。
const KEY_FINGERPRINT_ABSENT: &str = "(未生成)";

/// when file this —— is ， as 1 
/// up MB none need 。 will in before 1 。
const HEADER_SCAN_LINES: usize = 16;

/// file，used for to use （「 this file 1 」）。
/// and `crypto::SecretScope::key_file` 1 。
const ACCOUNT_KEY_FILE: &str = "account.key";

/// （ not ） ：、request/、 and request
/// 。（kv、accounts、account_usage_records）none
/// —— and config is 「」 need 。
const LOG_TABLES: &[&str] = &[
    "logs",
    "requests",
    "request_daily",
    "debug_traffic",
    "request_raw",
];

/// SQL after must in 。：
/// no has （ when by `schema::migrate` ）， this if file
/// or not is 。
const REQUIRED_TABLES: &[&str] = &["kv", "accounts"];

/// in （ to ）。 and no has 
/// （ after to to ， use to statistics）， not to 。
static RESTORE_MUTEX: Mutex<()> = Mutex::new(());

// ─── （ / ， before ）──────────────────────────
//
// ── Why need has this ──────────────────────────────────────────
// is ** 1 use **： MB SQL、 in up 、
// schema、、 use Backup API 。 in 30 seconds to minutes，
// in when return —— up has 1 「 in …」 by ， use not to 
// ， not need etc 。
//
// ── Why is 「 + 」 not is ────────────────
// and `/api/update/progress`、`/api/storage/progress` 1 ： 
// 1 ， before 500ms 1 。 by is not as 1 
// （ and need ）， and new after can up 1 ——
// by can ， not to （）。
//
// ── Why no has 「」────────────────────────────────────────
// and not can from in ；error by 1 。

/// 1 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestorePhase {
    /// empty（ no has in ）
    Idle,
    /// file + 
    Reading,
    Decrypting,
    Reencrypting,
    RollingBack,
    /// in up SQL
    Applying,
    /// schema / required / for 
    Validating,
    /// to current
    SafetyBackup,
    /// use Backup API 
    Replacing,
    /// （config / ）
    Reloading,
    /// 
    Done,
    /// failure
    Failed,
}

impl RestorePhase {
    /// → 。
    ///
    /// in after ： is 「 this 1 in 」 1 （ by this ），
    /// before 。 1 —— 「 in …」 is as 
    /// has 1 ， not 。
    fn label(self) -> &'static str {
        match self {
            Self::Idle => "",
            Self::Reading => "正在读取备份文件…",
            Self::Decrypting => "正在解密并校验备份包…",
            Self::Reencrypting => "正在迁移加密凭证到本机密钥…",
            Self::RollingBack => "正在还原替换前的数据…",
            Self::Applying => "正在把数据写入暂存库…",
            Self::Validating => "正在校验备份完整性…",
            Self::SafetyBackup => "正在备份当前数据（可回滚）…",
            Self::Replacing => "正在替换主数据库…",
            Self::Reloading => "正在重新加载配置…",
            Self::Done => "恢复完成",
            Self::Failed => "恢复失败",
        }
    }

    /// this （）。
    ///
    /// when （fileseconds、 SQL can can seconds、
    /// and ），** no has can **（`execute_batch` not ）。
    /// to is 「」 not is —— 1 、 not 
    /// 1 。 in `DONE` to 100。
    fn percent(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Reading => 5,
            Self::Decrypting => 8,
            Self::Reencrypting => 48,
            Self::RollingBack => 90,
            Self::Applying => 15,
            Self::Validating => 45,
            Self::SafetyBackup => 55,
            Self::Replacing => 65,
            Self::Reloading => 92,
            Self::Done => 100,
            Self::Failed => 100,
        }
    }
}

/// （ before ）。
#[derive(Clone, Debug)]
struct RestoreProgress {
    phase: RestorePhase,
    /// failure when Reason（`Failed` has ）
    error: Option<String>,
    /// when （`Done` has ； and `import_from_file` return 1 ）
    result: Option<Value>,
    /// when （ms）：「 already use when 」
    started_at: i64,
    /// when （ms； in as 0）
    finished_at: i64,
}

impl RestoreProgress {
    fn idle() -> Self {
        Self {
            phase: RestorePhase::Idle,
            error: None,
            result: None,
            started_at: 0,
            finished_at: 0,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "active": matches!(
                self.phase,
                RestorePhase::Reading
                    | RestorePhase::Decrypting
                    | RestorePhase::Reencrypting
                    | RestorePhase::RollingBack
                    | RestorePhase::Applying
                    | RestorePhase::Validating
                    | RestorePhase::SafetyBackup
                    | RestorePhase::Replacing
                    | RestorePhase::Reloading
            ),
            "phase": self.phase.label(),
            "percent": self.phase.percent(),
            "done": self.phase == RestorePhase::Done,
            "failed": self.phase == RestorePhase::Failed,
            "error": self.error,
            "result": self.result,
            "startedAt": self.started_at,
            "finishedAt": self.finished_at,
        })
    }
}

/// 。`None` = from no （ not ）。
static RESTORE_PROGRESS: Mutex<Option<RestoreProgress>> = Mutex::new(None);

/// 1 （ use ；failure when —— is up ， not this failure）。
fn set_phase(phase: RestorePhase) {
    if let Ok(mut guard) = RESTORE_PROGRESS.lock() {
        if let Some(progress) = guard.as_mut() {
            progress.phase = phase;
        }
    }
}

/// current（ before ）。 from no → `{"active": false, "phase": ""}`。
pub fn restore_progress() -> Value {
    match RESTORE_PROGRESS.lock() {
        Ok(guard) => guard
            .as_ref()
            .map(RestoreProgress::to_json)
            .unwrap_or_else(|| RestoreProgress::idle().to_json()),
        Err(_) => RestoreProgress::idle().to_json(),
    }
}

/// 1 ：。`finish` （success / failure）。
///
/// ── Why no has Drop ────────────────────────────────────
/// `import_from_file` `finish` is ** 1 **（`let result = …` ，
/// none Ok / Err will to ）， not in 「 no return」 path。 release is 
/// `panic=abort` —— to that 1 `Drop` not will ， is 。
/// will 「failure when will 」。
struct RestoreRun;

impl RestoreRun {
    fn begin() -> Self {
        if let Ok(mut guard) = RESTORE_PROGRESS.lock() {
            *guard = Some(RestoreProgress {
                phase: RestorePhase::Reading,
                error: None,
                result: None,
                started_at: logging::now_ms(),
                finished_at: 0,
            });
        }
        Self
    }

    fn finish(self, result: Result<Value, String>) {
        if let Ok(mut guard) = RESTORE_PROGRESS.lock() {
            if let Some(progress) = guard.as_mut() {
                progress.finished_at = logging::now_ms();
                match result {
                    Ok(value) => {
                        progress.phase = RestorePhase::Done;
                        progress.result = Some(value);
                    }
                    Err(message) => {
                        progress.phase = RestorePhase::Failed;
                        progress.error = Some(message);
                    }
                }
            }
        }
    }
}

/// VALUES limit： + （， as not ）。
/// 「 1 」 1 ，cc-switch 2 
/// 21 seconds → <100ms。
const INSERT_BATCH_MAX_ROWS: usize = 200;
const INSERT_BATCH_MAX_BYTES: usize = 1024 * 1024;

/// authorizer PRAGMA：dump this （ and 1 ）。
/// its 1 —— `temp_store_directory` can when file to to anydirectory，
/// `writable_schema` can schema check。
const IMPORT_ALLOWED_PRAGMAS: &[&str] = &["foreign_keys", "user_version"];

// ── ────────────────────────────────────────────────────────

/// to use path。
///
/// `include_logs = false` as （ and ）。
/// return `{ bytes, tables: {: } }` to before 。
pub fn export_to_file(
    db: &Db,
    target: &Path,
    include_logs: bool,
    password: &str,
) -> Result<Value, String> {
    archive::validate_password(password, true)?;
    let parent = target.parent().ok_or("备份路径缺少父目录")?;
    std::fs::create_dir_all(parent).map_err(|error| format!("创建备份目录失败: {error}"))?;
    let mut output = tempfile::Builder::new()
        .prefix(".agent2api-backup-")
        .tempfile_in(parent)
        .map_err(|error| format!("创建备份文件失败: {error}"))?;
    let result = archive::export(
        db,
        output.as_file_mut(),
        include_logs,
        password,
        &crate::paths::config_dir(),
    )?;
    output
        .as_file()
        .sync_all()
        .map_err(|error| format!("同步备份文件失败: {error}"))?;
    output
        .persist(target)
        .map_err(|error| format!("保存备份文件失败: {}", error.error))?;
    logging::log(
        "[Storage]",
        &format!("📤 加密整库备份完成: {}", target.display()),
    );
    Ok(result)
}

pub fn export_to_writer(
    db: &Db,
    writer: &mut impl Write,
    include_logs: bool,
    password: &str,
) -> Result<Value, String> {
    archive::export(
        db,
        writer,
        include_logs,
        password,
        &crate::paths::config_dir(),
    )
}

/// → 1 （ that 1 down ，module header）。
fn snapshot_to_memory(db: &Db) -> Result<Connection, String> {
    db.with(|conn| {
        let mut snapshot =
            Connection::open_in_memory().map_err(|e| format!("创建内存快照库失败: {e}"))?;
        {
            let backup =
                Backup::new(conn, &mut snapshot).map_err(|e| format!("初始化快照失败: {e}"))?;
            complete_backup(&backup, "创建内存快照")?;
        }
        Ok(snapshot)
    })
    .unwrap_or_else(|| Err("数据库连接不可用，导出未执行".to_string()))
}

/// SQLite Backup ：`step(-1)` must Done ，Busy/Locked/More is 
/// 「 no 」—— successreturn will down ，。
fn complete_backup(backup: &Backup<'_, '_>, context: &str) -> Result<(), String> {
    let result = backup.step(-1).map_err(|e| format!("{context}失败: {e}"))?;
    match result {
        StepResult::Done => Ok(()),
        other => Err(format!("{context}未完成: SQLite Backup 返回 {other:?}")),
    }
}

/// by VALUES SQL。
fn dump_sql_into(
    conn: &Connection,
    skip_tables: &[&str],
    writer: &mut impl Write,
) -> Result<(), String> {
    let mut output = String::new();
    let user_version: i64 = conn
        .query_row("PRAGMA user_version;", [], |row| row.get(0))
        .map_err(|e| format!("读取库版本失败: {e}"))?;

    output.push_str(&format!(
        "{BACKUP_SQL_HEADER}\n-- 生成时间: {}\n-- user_version: {user_version}\n",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
    ));
    // SQL old ； in in 。
    output.push_str(&account_key_fingerprint_line());
    output.push_str("PRAGMA foreign_keys=OFF;\n");
    output.push_str(&format!("PRAGMA user_version={user_version};\n"));
    output.push_str("BEGIN TRANSACTION;\n");

    // schema：// DDL （`type='table' DESC` in before ），
    // trigger to after —— before trigger will in INSERT when ，
    // dump ， INSERT remaining。
    let mut stmt = conn
        .prepare(
            "SELECT type, name, sql FROM sqlite_master
             WHERE sql NOT NULL AND type IN ('table','index','trigger','view')
             ORDER BY type='table' DESC, name",
        )
        .map_err(|e| format!("读取 schema 失败: {e}"))?;
    let mut tables = Vec::new();
    let mut triggers = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("读取 schema 失败: {e}"))?;
    for row in rows {
        let (obj_type, name, sql) = row.map_err(|e| format!("读取 schema 失败: {e}"))?;
        if name.starts_with("sqlite_") {
            continue;
        }
        if obj_type == "trigger" {
            triggers.push(sql);
            continue;
        }
        output.push_str(&sql);
        output.push_str(";\n");
        if obj_type == "table" {
            tables.push(name);
        }
    }

    writer
        .write_all(output.as_bytes())
        .map_err(|error| format!("写入备份 SQL 失败: {error}"))?;
    output.clear();
    for table in &tables {
        if skip_tables.contains(&table.as_str()) {
            continue;
        }
        dump_table_rows(conn, table, writer)?;
    }

    dump_sqlite_sequences(conn, skip_tables, &mut output)?;

    for sql in triggers {
        output.push_str(&sql);
        output.push_str(";\n");
    }

    output.push_str("COMMIT;\nPRAGMA foreign_keys=ON;\n");
    writer
        .write_all(output.as_bytes())
        .map_err(|error| format!("写入备份 SQL 失败: {error}"))
}

/// file that （）。
///
/// SQL ； 。
/// use `crypto::key_fingerprint`（），** not will ** as has this use ——
/// is ， not can in use up empty 1 `account.key`。
fn account_key_fingerprint_line() -> String {
    let fingerprint = crypto::key_fingerprint(crypto::SecretScope::Accounts)
        .unwrap_or_else(|| KEY_FINGERPRINT_ABSENT.to_string());
    format!("{KEY_FINGERPRINT_LINE} {fingerprint}\n")
}

/// 1 → VALUES INSERT 。
fn dump_table_rows(conn: &Connection, table: &str, writer: &mut impl Write) -> Result<(), String> {
    let quoted_table = quote_identifier(table);
    let columns = table_columns(conn, table)?;
    if columns.is_empty() {
        return Ok(());
    }
    let quoted_columns = columns
        .iter()
        .map(|c| quote_identifier(c))
        .collect::<Vec<_>>()
        .join(", ");
    let insert_prefix = format!("INSERT INTO {quoted_table} ({quoted_columns}) VALUES ");

    let mut stmt = conn
        .prepare(&format!("SELECT {quoted_columns} FROM {quoted_table}"))
        .map_err(|e| format!("读取表 {table} 失败: {e}"))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| format!("读取表 {table} 失败: {e}"))?;

    let mut pending_rows = 0usize;
    let mut batch = String::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| format!("读取表 {table} 失败: {e}"))?
    {
        let mut values = Vec::with_capacity(columns.len());
        for idx in 0..columns.len() {
            let value = row
                .get_ref(idx)
                .map_err(|e| format!("读取表 {table} 失败: {e}"))?;
            values.push(format_sql_value(value)?);
        }

        let row_sql = format!("({})", values.join(", "));
        let separator = usize::from(pending_rows > 0);
        if pending_rows > 0 && batch.len() + separator + row_sql.len() + 2 > INSERT_BATCH_MAX_BYTES
        {
            batch.push_str(";\n");
            writer
                .write_all(batch.as_bytes())
                .map_err(|error| format!("写入表 {table} 备份失败: {error}"))?;
            pending_rows = 0;
        }
        if pending_rows == 0 {
            batch.clear();
            batch.push_str(&insert_prefix);
        } else {
            batch.push(',');
        }
        batch.push_str(&row_sql);
        pending_rows += 1;

        if pending_rows >= INSERT_BATCH_MAX_ROWS {
            batch.push_str(";\n");
            writer
                .write_all(batch.as_bytes())
                .map_err(|error| format!("写入表 {table} 备份失败: {error}"))?;
            pending_rows = 0;
        }
    }
    if pending_rows > 0 {
        batch.push_str(";\n");
        writer
            .write_all(batch.as_bytes())
            .map_err(|error| format!("写入表 {table} 备份失败: {error}"))?;
    }
    Ok(())
}

/// （sqlite_sequence）： INSERT will to MAX(rowid)，
/// 「 」。 sequence not in （ no use AUTOINCREMENT）。
fn dump_sqlite_sequences(
    conn: &Connection,
    skip_tables: &[&str],
    output: &mut String,
) -> Result<(), String> {
    if !table_exists(conn, "sqlite_sequence")? {
        return Ok(());
    }
    let mut stmt = conn
        .prepare("SELECT name, seq FROM sqlite_sequence ORDER BY name")
        .map_err(|e| format!("读取自增序列失败: {e}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("读取自增序列失败: {e}"))?;
    let mut values = Vec::new();
    for row in rows {
        let (table, sequence) = row.map_err(|e| format!("读取自增序列失败: {e}"))?;
        if skip_tables.contains(&table.as_str()) {
            continue;
        }
        values.push(format!(
            "({}, {})",
            format_sql_value(rusqlite::types::ValueRef::Text(table.as_bytes()))?,
            sequence
        ));
    }
    output.push_str("DELETE FROM sqlite_sequence;\n");
    if !values.is_empty() {
        output.push_str("INSERT INTO sqlite_sequence (name, seq) VALUES ");
        output.push_str(&values.join(","));
        output.push_str(";\n");
    }
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        )
        .map_err(|e| format!("检查表 {table} 是否存在失败: {e}"))?;
    Ok(count > 0)
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let quoted = quote_identifier(table);
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({quoted})"))
        .map_err(|e| format!("读取表 {table} 结构失败: {e}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("读取表 {table} 结构失败: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("读取表 {table} 结构失败: {e}"))?;
    Ok(columns)
}

/// / SQL： + ，。
fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// SQL ： and SQLite then for （NaN NULL、 REAL 、
/// NUL use hex CAST —— SQL NUL ， not down ）。
fn format_sql_value(value: rusqlite::types::ValueRef<'_>) -> Result<String, String> {
    match value {
        rusqlite::types::ValueRef::Null => Ok("NULL".to_string()),
        rusqlite::types::ValueRef::Integer(i) => Ok(i.to_string()),
        rusqlite::types::ValueRef::Real(f) => Ok(format_sql_real(f)),
        rusqlite::types::ValueRef::Text(t) => match std::str::from_utf8(t) {
            Ok(text) if !text.contains('\0') => Ok(format!("'{}'", text.replace('\'', "''"))),
            _ => Ok(format!("CAST({} AS TEXT)", format_sql_blob(t))),
        },
        rusqlite::types::ValueRef::Blob(bytes) => Ok(format_sql_blob(bytes)),
    }
}

fn format_sql_real(value: f64) -> String {
    if value.is_nan() {
        return "NULL".to_string();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-9.0e999".to_string()
        } else {
            "9.0e999".to_string()
        };
    }
    if value == 0.0 && value.is_sign_negative() {
        return "-0.0".to_string();
    }
    let mut literal = value.to_string();
    if !literal.contains(['.', 'e', 'E']) {
        literal.push_str(".0");
    }
    literal
}

fn format_sql_blob(bytes: &[u8]) -> String {
    let mut s = String::from("X'");
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02X}");
    }
    s.push('\'');
    s
}

/// （statistics； ）。
fn table_row_counts(conn: &Connection, skip: &[&str]) -> Vec<Value> {
    let known = [
        "kv",
        "accounts",
        "logs",
        "requests",
        "request_daily",
        "debug_traffic",
        "request_raw",
        "account_usage_records",
        "account_limiter_events",
        "checkin_records",
        "assistant_sessions",
        "assistant_images",
    ];
    known
        .iter()
        .filter(|table| !skip.contains(table))
        .filter_map(|table| {
            let count: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {}", quote_identifier(table)),
                    [],
                    |row| row.get(0),
                )
                .ok()?;
            Some(json!({ "table": table, "rows": count }))
        })
        .collect()
}

// ── ────────────────────────────────────────────────────────

/// from file `-- account-key-fingerprint:` 。
///
/// `HEADER_SCAN_LINES` （ is ）， not as 1 。
/// old file no has this 1 → `None`（none for ， use by 「none」）。
fn parse_account_key_fingerprint(sql: &str) -> Option<String> {
    sql.lines()
        .take(HEADER_SCAN_LINES)
        .find_map(|line| line.trim().strip_prefix(KEY_FINGERPRINT_LINE))
        .map(|value| value.trim().to_string())
}

/// for and ，return** to use **（ for →
/// `None`）。
///
/// and ：
/// · 1 （：）→ not ；
/// · is 「」→ （ when that is 「 not in 」， if already has 
/// ， not ）；
/// · not / no has → （ this is ** 1 ** ）。
///
/// ** not **： use is need 「 ，」
/// （e.g.、 `account.key` ， or ）。
/// this ， by use return to / 。
fn account_key_warning(backup_marker: Option<String>) -> Option<String> {
    // old no has this 1 ： no has can ，（ not 「 no 」「 not 」）
    let backup_marker = backup_marker?;
    if backup_marker == KEY_FINGERPRINT_ABSENT {
        return Some(format!(
            "这份备份导出时账号加密密钥尚未生成，无法校验密钥是否一致：若账号数据无法解密（账号列表为空），\
             请把备份那台机器的 {ACCOUNT_KEY_FILE} 一并带过来后重新导入。"
        ));
    }
    let local = crypto::key_fingerprint(crypto::SecretScope::Accounts);
    if local.as_deref() == Some(backup_marker.as_str()) {
        return None;
    }
    Some(format!(
        "这份备份是用另一把加密密钥加密的，导入后账号数据将无法解密（账号列表可能为空）。\
         若要在别的机器恢复，请把备份那台机器的 {ACCOUNT_KEY_FILE} 一并带过来。"
    ))
}

/// from or old SQL ，failure when from 。
///
/// returnstatistics、 and ； old SQL not when 。
pub fn import_from_file(
    db: &Db,
    source: &Path,
    config_dir: &Path,
    password: &str,
) -> Result<Value, String> {
    let _guard = match RESTORE_MUTEX.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => {
            return Err("整库恢复正在进行，请等待当前恢复完成".into())
        }
    };
    let run = RestoreRun::begin();
    let result = restore::import(db, source, config_dir, password);
    run.finish(result.clone());
    result
}

/// to up ： 1 can **file** 。
/// by in prepare （module header「 」）。
fn import_authorizer(context: AuthContext<'_>) -> Authorization {
    let escapes_temp_db = match context.action {
        AuthAction::Attach { .. } | AuthAction::Detach { .. } => true,
        AuthAction::CreateVtable { .. } | AuthAction::DropVtable { .. } => true,
        AuthAction::Unknown { .. } => true,
        AuthAction::Pragma { pragma_name, .. } => !IMPORT_ALLOWED_PRAGMAS
            .iter()
            .any(|allowed| pragma_name.eq_ignore_ascii_case(allowed)),
        _ => false,
    };
    if escapes_temp_db {
        logging::log(
            "[Storage]",
            &format!("备份导入拒绝了越界语句: {:?}", context.action),
        );
        Authorization::Deny
    } else {
        Authorization::Allow
    }
}

fn validate_required_tables(conn: &Connection) -> Result<(), String> {
    let mut missing = Vec::new();
    for table in REQUIRED_TABLES {
        if !table_exists(conn, table)? {
            missing.push(*table);
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!(
        "备份缺少必需的数据表：{}（文件可能不完整）",
        missing.join(", ")
    ))
}
