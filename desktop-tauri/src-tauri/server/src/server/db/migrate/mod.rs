//! old file → ** 1 **（ + ）。
//!
//! ── fileWhy in ─────────────────────────────
//! need old file has ：`config.json`、`accounts.json`、`logs.jsonl`、
//! `requests.jsonl`、`request-daily.jsonl`、`debug-traffic.jsonl`、
//! `desktop-settings.json`。 by ** not ** 
//! ，「 already 」「failure」「 old file not 」 this must
//! 1 1 —— 1 ， will ， and 
//! 「 will not will 」 this need will not 。
//! 、 etc then and down ， after 「file → 」。
//!
//! （ `desensitize.json` in this ： old file。
//! that can already then delete， for and 
//! 1 and —— old file in not ， `desensitize` is 
//! ， `db::schema` 。）
//!
//! ── etc then （ after must）──────────────────────
//! ** in as empty when **： 1 is 「 
//! in has no has 」——** 1 ** `SELECT COUNT(*) FROM `，
//! ** `kv` **「 that in not in 」（`kv` is ，empty and no has 
//! none， `settings.rs` module header）。 0 / already in return `None`。
//! is ：
//! 1. not will —— 1 in that ；
//! 2. old file no （failure、 use in after file），
//! is ， not will 1 ；
//! 3. old file after file not to ， etc ——
//! 「empty」 this check is ， not is 1 。
//! Why use 「 has no has 」 not is in `kv` 1 `migratedXxx: true` ：
//! and can can not 1 （ 1 in —— that when must，
//! will ； no ，）。
//! 「 has no has 」 is ， not will 。Note `kv` that 
//! （ `kv.desktopSettings` ）** is ** this then ：
//! need in that 、 need not in ， not in 「 1 」 in 。
//!
//! ── failureWhy not ────────────────────────────────────
//! is ** old 1 **， failure not this not ： before 
//! can that old file（ not has store ），
//! 「 this failure → down 」 is path。 is 1 
//! is 「 as 」：failure 1 、return `None`， after 。
//!
//! [`crate::server::logging::console_line`] ** not is ** `logging::log`：
//! in `logging::init_store` ** before **（must，
//! `ServerState::bootstrap` ）， when no ，`log()`
//! that 1 will —— is use need need in to 。
//! `console_line` ， not 。
//!
//! ── file ────────────────────────────────────────────────
//! ```text
//! server/db/
//!   migrate/
//! mod.rs ： + （file）
//! backup.rs ： old file（all use ）
//! accounts.rs accounts.json → accounts 
//! logs.rs logs.jsonl → logs 
//! requests.rs requests.jsonl + request-daily.jsonl → requests / request_daily 
//! debug.rs debug-traffic.jsonl → debug_traffic 
//! config.rs config.json → kv （** 1 **）
//! settings.rs desktop-settings.json → kv `desktopSettings` 
//! ```
//! 1 file：****（ not file、 not /、 not ），
//! in 1 file will 「 this 1 need 」 need need can ；
//! 800 ， file。
//! （、`LegacyOutcome`、） in `mod.rs` and `backup.rs`：
//! that is **all need ** ， 1 must when 。
//!
//! ── etc （ when in 1 ）─────────────
//! - ** 1 **（ / / request / request / ）：
//! 「empty not empty」；
//! - ** `kv` 1 **（set）：「 that in not in 」
//! —— not can empty not empty，`kv` is （ `settings.rs` module header）；
//! - ** `kv` **（config）： use **** —— config no has can 
//! config（ `config.rs` module header that 1 ， when 
//! 「T6 Why for 」 this ： for is ）。
//!
//! ── （ to after ）─────────────────────────
//! is 1 [`LegacyMigration`]， by ： can 、** old file**、
//! ：
//! ```text
//! fn import_accounts(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
//! let path = legacy_file(dir)?; // （ and down ）
//! if count(conn, "accounts") > 0 { return None; } // empty → already 
//! let outcome = match (conn, &path) { // 1 
//!         Ok(n) => LegacyOutcome { label: LABEL, source: path.clone(), imported: n, backup: backup_legacy_file(&path) },
//! Err(error) => { console_line("[Storage]", &format!("❌ failure: {error}")); return None; }
//!     };
//!     Some(outcome)
//! }
//! ```
//! Note：
//! - ** 1 **： N must（ in after need empty、 need ），
//! otherwise「empty」 this etc will up 。 this to is `&Connection`，
//! use `unchecked_transaction()`（`transaction()` need `&mut`）——
//! use is `Db::with`， has 1 、 no has ， that 
//! 「 not 」 by use 。
//! - **failure need **：`Option` not 「 no has old file」 and 「failure」 
//! （ is `None`），failuremust by use `console_line` ，
//! not （ [`run_legacy_migrations`]）。
//! - ** not panic**：release is `panic = abort`， no has unwind path can 
//! panic（`catch_unwind` in abort down not will return）， 1 
//! will use 。 old file 1 use `Option`/`Result` ，
//! not need 「 old 1 this 」 before 。
//!
//! ── Why and in 1 ────────────────
//! 「 has no has old file」 need by （） in ** before **，
//! is 「 1 old file in not in 」。 if 1 ，
//! （add 1 ，「 no has 」；
//! or old file， and for not up ）。 1 after ，
//! 1 1 ，[`pending_items`] and [`run_legacy_migrations`] 1 。
//! ** by **（file and directory in that ），
//! 。

mod accounts;
mod backup;
mod config;
mod debug;
mod logs;
mod requests;
mod settings;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

/// old 。** this is after all 1 return**。
///
/// ：`label` is to （ 
/// `✅ {label} old already （{imported} ）`）；`source` is old filepath；
/// `imported` is ；`backup` is old file after path
/// （failure when as `None` —— in ， already ）。
///
/// `source` and `backup` before ** no has **（ use `label` / `imported`，
/// `ServerState::bootstrap`）， `#[allow(dead_code)]`： is 
/// 「」 this 1 ， after 「」 need 「 from file、 in 
/// 」， will that 1 this （ that when in use ）。
/// by 「 has and by 」 （ `server/mod.rs` 
/// dead_code 1 ）。
pub struct LegacyOutcome {
    /// （ in ，）
    pub label: &'static str,
    /// old filepath
    #[allow(dead_code)]
    pub source: PathBuf,
    /// 
    pub imported: usize,
    /// old file after path（`None` = no success or no has old file）
    #[allow(dead_code)]
    pub backup: Option<PathBuf>,
}

/// 1 ： can + old file + 。
///
/// in 1 not is and （ by module header「Why and 
/// in 1 」）。`label` and `LegacyOutcome` 
/// is ** 1 **（ `LABEL`）， and 
/// not will 。
pub struct LegacyMigration {
    /// （ in ；「」 and use ）
    pub label: &'static str,
    /// old file in not in （`Some` = ）。 not to when return `None`。
    pub locate: fn(&Path) -> Option<PathBuf>,
    /// （ etc 、failure，module header）
    pub import: fn(&Connection, &Path) -> Option<LegacyOutcome>,
}

/// ：****，current（config、set、、、
/// request、request、、）。
///
/// use `const` not is （`OnceLock<Vec<...>>` / can ）：
/// 1. ， in 1 can —— ** has **
/// （「」 need 「requeststatistics」， after account_id has can ），
/// can will this in when up ；
/// 2. no has when ：`Db::open` after when can ， not ；
/// 3. 1 is 1 ， no has need 。
///
/// when Note: ** 1 can 1 **（ no has ）， and is `fn`
/// （ not can is —— need ， that as `&dyn Fn` and 
/// ，「 + directory」 parameter， no has need ）。
///
/// ── （ has 1 ****，）───────────────
/// `import_daily` must in `import_requests` ** after **： old 
/// after need 1 ****（ use that ，
/// `backfill` module header）， is `requests` ——
/// ， will as 「 as empty」 1 not up ， that in 
/// 「unknown」（「 not not 」 will ， not ）。
/// its no has ： /、 。
/// ：config → set → → → request → request
/// → → 。
///
/// `import_config` / `import_settings` before is ** can **
/// need ：`import_logs` / `import_requests` / `import_debug` need 
/// `config::storage_dirs()`（ old file directory） and 
/// `config::retention_settings()`（），config after that 
/// 「config already from old file」—— this 1 by `config::init` **file**
/// （ `config::read_raw` when ）， in not 。
/// before 「 config、config 」 this in up 1 can 。
///
/// in after `import_debug` and request**none**（ `debug_traffic`，
/// not not ）—— and id ， that is **** 
/// （ use in by down by ）， after not 。
/// in is by 「 from to 」 ： can MB， after not will 
/// before file in after 。
///
/// `import_settings` none（ `kv` ， and 、、statistics
/// not ）—— ** and 1 not **（「 that in not in 」 not is 
/// 「empty not empty」， as `kv` is ）；`import_config` is ****
/// （， as config is ）—— 
/// `settings.rs` / `config.rs` module header。
pub const LEGACY_MIGRATIONS: &[LegacyMigration] = &[
    LegacyMigration { label: config::LABEL, locate: config::legacy_file, import: config::import_config },
    LegacyMigration { label: settings::LABEL, locate: settings::legacy_file, import: settings::import_settings },
    LegacyMigration { label: accounts::LABEL, locate: accounts::legacy_file, import: accounts::import_accounts },
    LegacyMigration { label: logs::LABEL, locate: logs::legacy_file, import: logs::import_logs },
    LegacyMigration { label: requests::LABEL, locate: requests::requests_file, import: requests::import_requests },
    LegacyMigration { label: requests::DAILY_LABEL, locate: requests::daily_file, import: requests::import_daily },
    LegacyMigration { label: debug::LABEL, locate: debug::legacy_file, import: debug::import_debug },
];

/// in old file for （**「」 use **）。
///
/// is 「 old file in not in 」：success after as `*.migrated`，
/// 「file in 」 and 「 this no 」 is 1 ， not need need 1 
/// —— and can can not 1 ，file in not in is （ and module header
/// 「 etc then 」 1 ）。
///
/// return **used for**： [`run_legacy_migrations`]，
/// 1 will 1 etc （empty not empty / in not in ）， this 
/// 「 has 」 no is （ already has ）， not is bug。
pub fn pending_items(dir: &Path) -> Vec<&'static str> {
    LEGACY_MIGRATIONS
        .iter()
        .filter(|migration| (migration.locate)(dir).is_some())
        .map(|migration| migration.label)
        .collect()
}

/// has no has old file（[`pending_items`] ）。
///
/// to 1 not is use `!pending_items(dir).is_empty()`：
/// path（`ServerState::bootstrap`） need need this ， need 1 
/// —— 1 `Vec` is has （ need 1 `is_file()`）。
pub fn has_pending(dir: &Path) -> bool {
    LEGACY_MIGRATIONS
        .iter()
        .any(|migration| (migration.locate)(dir).is_some())
}

/// 1 ，return**success** （failure/ not in return）。
///
/// 「 not 」 this in up ： no has `?`、 no has `break`，
/// 1 return「 need not need 」， not will after 1 —— before that 
/// failure、 no old file， after 1 use 。
///
/// Why not as failure ：`Option` 「 no has old file」 and 「failure」
/// 1 `None`（ this is —— not this need 1 error
/// ，module header ），**none from ** 1 `None` is 
/// is ， 1 「failure」 will failure。failure
/// by that 1 （） use `console_line` 。
pub fn run_legacy_migrations(conn: &Connection, dir: &Path) -> Vec<LegacyOutcome> {
    let mut outcomes = Vec::new();
    for migration in LEGACY_MIGRATIONS {
        if let Some(outcome) = (migration.import)(conn, dir) {
            outcomes.push(outcome);
        }
    }
    outcomes
}
