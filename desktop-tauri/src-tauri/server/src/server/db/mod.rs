//! ** 1 **：file SQLite `{config_dir}/agent2api.db`。
//!
//! ── this 1 need ────────────────────────────────────
//! before all is 「file + 」：config.json、accounts.json、
//! logs.jsonl、requests.jsonl、request-daily.jsonl、debug-traffic.jsonl、
//! desktop-settings.json。 store 「 / /
//! file / after from file」， 1 （e.g.、unknown
//! 、 and ） ， has 。 to SQLite after ：
//! 、、 by ， store not need need has 。
//!
//! ── （ ）─────────────────────────────
//! 、schema and old file；** store by after 
//! **， will 「 1 in 、 1 in old file」 
//! and ， that is in in ， not is bug。
//! already ：****（`core::account_store` → `accounts` ； old 
//! `accounts.json` by `migrate::import_accounts` 1 ）、
//! ****（`logs_store` → `logs` ； old `logs.jsonl`
//! by `migrate::import_logs` 1 ）、
//!           **requeststatistics**（`request_stats` → `requests` / `request_daily`
//! ； old `requests.jsonl` / `request-daily.jsonl` by 
//! `migrate::import_requests` / `migrate::import_daily` 1 ）、
//! ****（`core::debug_traffic` → `debug_traffic` ；
//! old `debug-traffic.jsonl` by `migrate::import_debug` 1 ）、
//! **config**（`config` → `kv` ，** 1 **； old 
//! `config.json` by `migrate::import_config` 1 ）、
//! **set**（ `settings` → `kv` `desktopSettings` ；
//! old `desktop-settings.json` by `migrate::import_settings` 1 ）
//! ：none —— old file（ is after 1 ）。
//!
//! ── `kv` （ ）───────────────
//! config （ use ） and 「 its 」 （`desktopSettings` /
//! `priorityScope` / `logsNextId` / `configMigrated`， and **** 
//! `desensitize` —— that can already 、） use this 1 
//! ，**config need 「delete already not in 」**：must not ，
//! otherwise 1 config will 。 this 1 is 
//! [`schema::RESERVED_KV_KEYS`]， `schema` module header 
//! 「 not can 」 1 （add when **must** for ）。
//!
//! ```text
//! server/db/
//! mod.rs and （file）：Db::open / with / with_mut / file
//! schema.rs DDL and （PRAGMA user_version ）
//! migrate/ old file → 1 （ + 1 file）
//! ```
//!
//! ── Why not is ────────────────────────────────
//! is file， SQLite （ 1 when 1 
//! ）， can and 「」—— in in API up ，
//! no to need need and 。： 1 、
//! SQLITE_BUSY、 and 「 to 」 this in down in bug。
//! + 1 Mutex and in 「 + 」 ，
//! as for 。 need and ， by SQLite up WAL + ， not is 
//! 。
//!
//! ── Why is Mutex not is RwLock ────────────────────────────
//! `rusqlite::Connection` `&self`（`prepare` / `execute` /
//! `query_row`）， up `RwLock<Connection>` can and 。：
//! 1. ** is **， not ， and has 「 when 」
//! this 1 ；
//! 2. need need `&mut self`（ is ）， 1 has ，RwLock 
//! 「」， 1 「 will 」 need ；
//! 3. and in `std` not ，。
//! ：`Mutex` 、 and 「SQLite 」 this 1 ， not use `RwLock`。
//! （ for `request_stats` / `logs_store` `file_path` use RwLock： that is **
//! ** （ as old in ）， and this 「 can can 
//! 」 not ， not is 1 。）
//!
//! ── ： not request ──────────────────────────
//! this is has 、 1 need （ `core::account_store` module header）：
//! this has 1 ， 1 request， etc **all** API and 
//! in that 1 seconds up 。 （`with` / `with_mut`） is as 
//! this ：「/」 ， need need in 
//! 、request、 after 1 。
//!
//! ── failureWhy not is error ──────────────────────────────
//! `Db::open` return `Result<Db, String>`， use （`ServerState::bootstrap`）
//! failure `Option<Db> = None`：release is panic=abort， /
//! not / file not this use 。 can to 
//! can error and 。 store need by 「 can can not in 」。

pub mod backup;
pub mod migrate;
mod revisions;
pub mod schema;

pub(crate) use revisions::AccountChanges;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// file。configdirectory（`~/.agent2api`）， and config.json / and ——
/// use in set to 「configdirectory」 is this 1 ， and need need 。
/// path by `config::config_dir()` ， not home directory。
/// （ store —— 、、requeststatistics、、 ——
/// after not has file， in 。）
pub const FILE_NAME: &str = "agent2api.db";

/// SQLite 。`Clone` ** 1 **（ `Arc`）。
///
/// and `AccountStore` `Arc<Inner>` is 1 ：`ServerState` is `Clone` 
/// （axum handler ）， can 1 「 + 」，
/// handler to can use ， not 。 `Db` not will 。
///
/// `Send + Sync + 'static` by `Arc<Mutex<Connection>>` ：
/// `rusqlite::Connection` is `Send` ** not is ** `Sync`（ has SQLite 
/// ），`Mutex<T>: Sync` need `T: Send`， —— this is must 1 
/// `Mutex` not can `Connection` `Arc` Reason（ that will not ）。
#[derive(Clone)]
pub struct Db {
    /// + filepath。path and 1 `Arc` ，`file()` not 
    /// 1 —— path from after not （ not ：directory is 「 new + 」，
    /// not is this ）。
    inner: Arc<DbInner>,
}

struct DbInner {
    /// 。 in when by 「unavailable」，** not use ** `into_inner()` ——
    /// and `core::egress` / `logs_store` to not ， by `with` 。
    conn: Mutex<Connection>,
    /// filepath（ when ， after not ）
    file: PathBuf,
    accounts: Arc<revisions::AccountRevisions>,
}

impl Db {
    /// （ need when create） and ：directory → → PRAGMA →
    /// schema 。
    ///
    /// ** old file not in this **（`migrate::run_legacy_migrations` by use in 
    /// success after ）： need 、 need to use 「」，
    /// return has 1 `Db` —— can that ，
    /// or as 1 return（ is 「」 this ）。
    /// after its ， and use can by 。
    /// （ in that use is `POST /api/upgrade/run`， not is ——
    /// 「 has no has old file」， `migrate::has_pending`。）
    ///
    /// return `String` error not is error： and `AccountStoreError` 
    /// 1 —— this 1 is ， use need need 「 1 can to 
    /// 」， not need need by error。
    ///
    /// directorymust： when `{config_dir}` can can not in （ new 、
    /// use ）， SQLite `open` create**file**， not will 
    /// directory —— directory when will 「nonefile」failure。
    pub fn open(path: &Path) -> Result<Db, String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("创建数据库目录失败: {error}"))?;
            }
        }
        let conn = Connection::open(path).map_err(|error| format!("打开数据库失败: {error}"))?;
        apply_pragmas(&conn)?;
        schema::migrate(&conn).map_err(|error| format!("建表/升级 schema 失败: {error}"))?;
        let accounts = Arc::new(revisions::AccountRevisions::default());
        install_cloud_sync_hook(&conn, accounts.clone());
        Ok(Db {
            inner: Arc::new(DbInner {
                conn: Mutex::new(conn),
                file: path.to_path_buf(),
                accounts,
            }),
        })
    }

    /// （）。** in （ panic） when return `None`**，
    /// use （returndefault / this 1 / old filepath）。
    ///
    /// ── Why not its `poisoned.into_inner()` ──────────
    /// that （`request_stats` / `core::debug_traffic`） is 
    /// （ before is + ， in 1 「 use 」），panic 
    /// 1 ， use is —— is 
    /// 「statistics not not statistics need 」。
    /// this not ： panic can can is ** 1 SQLite ** or 1 
    /// to 1 。`into_inner()` after none from in in ，
    /// will 「」 to up —— that 「 this 1 can when unavailable」
    /// 。 in as unavailable：`None` use as 
    /// （e.g. this 1 not ）， limit in store up 。
    /// Note this and 「failure」 is 1 to ：** not this **，
    /// not this to error。
    ///
    /// ── and store that to not ──────────────────────────
    /// `AccountStore::guard` / `LogStore::guard` **** `into_inner()` ——
    /// is 「 1 」 this ， no has need need 1 
    /// （ in ）。 not is as ** for not **： this is 
    /// （ can can in in ）， that is empty。
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> T) -> Option<T> {
        match self.inner.conn.lock() {
            Ok(conn) => Some(f(&conn)),
            Err(_) => None,
        }
    }

    /// ， to is `&mut Connection` —— ** need need use this **。
    ///
    /// `Connection::transaction()` is `&mut self`（ is ，
    /// not in 1 up ， need use ）。
    /// `with` can to `&Connection`， not to 。 can ：
    /// - `unchecked_transaction()`（ `&self`）： can ， 「
    /// not use 」 this then check， use is `SQLITE_MISUSE`；
    /// - `with_mut`： 「 need 1 」 this use 。
    /// after ： use 1 can is ， `with` / `with_mut` 
    /// is 1 「 need not need 」， not API is unsafe 。
    ///
    /// and `with` use 1 ，** not will ** and ： 1 `with_mut` 
    /// ， its `with` use will in up 。 this is need 。
    ///
    /// use ：`AccountStore::with_conn_mut`（ and ——
    /// /、`migrate_startup`、 old 、 and ）。
    pub fn with_mut<T>(&self, f: impl FnOnce(&mut Connection) -> T) -> Option<T> {
        match self.inner.conn.lock() {
            Ok(mut conn) => Some(f(&mut conn)),
            Err(_) => None,
        }
    }

    pub(crate) fn account_revision(&self) -> u64 {
        self.inner.accounts.current()
    }

    pub(crate) fn account_changes(&self, since: Option<u64>) -> AccountChanges {
        self.inner.accounts.since(since)
    }

    pub(crate) fn invalidate_accounts(&self) {
        self.inner.accounts.invalidate();
    }

    /// filepath（ / ）。
    ///
    /// return `&Path` not is `PathBuf`：path in when （** not ** ——
    /// is 「 new + 」， not is this ）， use is or 
    /// ， not need need all。
    /// ** not **：path not in ， not this and in 
    /// （ that 1 is —— that when can can has in ）。
    pub fn file(&self) -> &Path {
        &self.inner.file
    }
}

/// file `PRAGMA user_version`（ not to return `None`）。
///
/// ── Why need need 1 「 `Db`」 ──────────────────────
/// 1 use is `ServerState::bootstrap` `Db::open` **failure after ** that 1 ： that when 
/// no has available `Db` （ as schema not failure）， need need 「 
/// is 」 can this not this failure（**** = use to 
/// old ， `schema::migrate` and `SCHEMA_VERSION` v14 ）。
/// this 1 ****「 → 1 PRAGMA → 」。
///
/// failure（file not in 、、、PRAGMA not ） 1 return `None`：
/// ** not to ≠ not **， will 「」 this this 
/// failure。 use in to `Some()` and when 。
pub fn stored_schema_version(path: &Path) -> Option<i64> {
    let conn = Connection::open(path).ok()?;
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .ok()
}

/// after mustset PRAGMA。
///
/// use `execute_batch` 1 not is `pragma_update`：`journal_mode` this 
/// PRAGMA ** will return 1 **（ new / old ）， `pragma_update` will to 
/// 「return no 」 return；`execute_batch` need ，
/// not ， this 1 。
///
/// use ：
/// - `journal_mode=WAL`： not （ down will ）。
/// Note WAL is **file set**， 1 after 1 valid； this 
/// 1 is as etc ， and can from 。
/// is `-wal` / `-shm` file， when need use SQLite 
/// in ， need 1 （ after 「/」 need need Note）。
/// - `synchronous=NORMAL`：WAL down —— not fsync
/// WAL file， in check when fsync。** not will **（WAL is 
/// ）， if already ； is 1 。
/// is /statistics/， after can ， not use FULL。
/// - `foreign_keys=ON`：SQLite default****（ as old ），
/// must。 in no has ， —— after when 
/// not will as 「 this PRAGMA」。
/// - `busy_timeout=5000`： when etc 5 seconds， not is 
/// `SQLITE_BUSY`。 this 1 in **** when use ： can can when 
/// （ and and —— and identifier ，directory
/// use 1 ）， that not to 1 。set
/// that 5 seconds， 1 。set， not file，
/// need 。
fn apply_pragmas(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;",
    )
    .map_err(|error| format!("设置数据库 PRAGMA 失败: {error}"))
}

/// to （`core::cloud_sync::auto_sync`）。
///
/// ── ────────────────────────────────────────
/// `accounts` / `account_usage_records` / `checkin_records`。** not `kv`**：
/// `kv` `logsNextId` this seconds in ， `update_hook` 
/// in use （ already has ）， in 「 is 」
/// —— that will 1 1 。kv in 
/// （ `auto_sync` module header）。
///
/// ── failure not ──────────────────────────────────────
/// this 1 optional can 。 not up （ old SQLite etc ） when is 
/// 「 not 」， up available —— not this not 。
/// when ； not ， not get or 。
fn install_cloud_sync_hook(conn: &Connection, accounts: Arc<revisions::AccountRevisions>) {
    let result = conn.update_hook(Some(move |_action, _db: &str, table: &str, rowid: i64| {
        if table == "accounts" {
            accounts.changed(rowid);
        }
        crate::server::core::cloud_sync::auto_sync::on_table_change(table);
    }));
    if let Err(error) = result {
        // when no （`Db::open` `logging::init_store`）， can 
        crate::server::logging::console_line(
            "[CloudSync]",
            &format!("⚠️ 自动同步的数据库钩子注册失败（自动同步将不触发）: {error}"),
        );
    }
}

/// use ： will **file** when 。
///
/// Why not is " before `remove_file`"： that " not use old ", not 
/// " not "。file id, `cargo test` is 1 new pid —— Trae 1 
/// this in `$TMPDIR` down 401 file、 1 MB。must in 
/// has `Drop` up , use need ** in use **(`let (store, _db) = …`),
/// not is that `Db`。
///
/// in panic will drop( is unwind profile,`panic=abort`
/// used for), not need need cleanup use 。
#[cfg(test)]
pub(crate) mod test_temp {
    use std::path::{Path, PathBuf};

    use super::Db;

    pub struct TempDb {
        path: PathBuf,
    }

    impl TempDb {
        /// 1 when 。 return is file ,**must by use 
        /// has to use **( etc to " 1 " as )。
        pub fn open(label: &str) -> (Db, TempDb) {
            let safe: String = label
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
                .collect();
            let path = std::env::temp_dir().join(format!("agent2api-test-{safe}-{}.db", std::process::id()));
            let _ = std::fs::remove_file(&path);
            let db = Db::open(&path)
                .unwrap_or_else(|error| panic!("临时库打不开（{}）：{error}", path.display()));
            (db, TempDb { path })
        }

        #[allow(dead_code)]
        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            // WAL / SHM need :file will in 。
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", self.path.display(), suffix));
            }
        }
    }
}
