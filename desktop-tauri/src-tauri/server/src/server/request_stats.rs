//! requeststatistics：`requests` （）+ `request_daily` （ by ）+
//! `request_raw` （ for ；fork after in `{cache_dir}/request-raw/`，
//! id / ts / size / truncated， `raw_file` module header）。
//!
//! ── （ is as 1 、file not ）─────
//! ```text
//! server/
//! request_stats.rs ：（ in → in → ）/
//! / （file）
//!   request_stats/
//! sql.rs SQL： + use FilterPlan（ SQL 1 
//! 1 ； 1 is daily.rs and ）
//! daily.rs SQL：`request_daily` and --
//! record.rs and JSON （RequestEntry / DailyEntry / Retention…）
//! report.rs ：（、、 in 、topModel、）
//! aggregate.rs ：（fold_into_daily）/ / 
//! maintenance.rs ： / / （VACUUM）
//! clock.rs when （chrono::Local in 1 use ）
//! backfill.rs old （** by use **， that ）
//! legacy.rs old file ： old JSONL、、+
//! （ by use ， 1 not will to ）
//! ```
//!
//! ── ──────────────────────────────────────
//! （`usage_summary` / `query_requests` / `prune` / `stats` / and ）
//! already by `api::stats_api` by up ， this up `#[allow(dead_code)]`
//! already ：add if no use will warning， and when by 。
//! allow has `file()` and `import_legacy_*`
//! （ by in ）。
//!
//! ── Why is （ 「 file」）──────────────────────
//! this ** 1 **， is from file：
//! - `requests`：request， 1 1 。（ in 10 minutes / 1 when 
//! 、 24 when ）must， by to not this 。
//! - `request_daily`： by ， 1 1 。 365 、`all` can can ，
//! （`SUMMARY_WINDOW_MS`， after 24 when ——
//! in already in SQL ， not need ）， this 
//! ****： after in ， not will empty。
//! `request_raw` （ not ， `db/schema.rs` V3_SCHEMA）；
//! fork after `{cache_dir}/request-raw/`， / / 。
//! file ：** and 1 **（ is 
//! ， in will down 「、 in 」 ） in by ** 1 **
//! ； ****（ / empty when ）
//! `maintenance.rs` module header 。
//!
//! ── ： ──────────────────────────────────────
//! before is 「 as + `COMPACT_STEP` file」， is 
//! 「 50 or up 60 seconds file」（）。 after 
//! this ：
//! - 1 `INSERT`， 1 UPSERT，****（`record` 
//! 1 ）， no has 「file」 for ， no has file；
//! - is `dirty` / `appends_since_compact` / `COMPACT_STEP` / `insert_sorted` /
//! that for and when not in ；
//! - path `flush()` not is 「 」—— new this 。
//!
//! ── and ────────────────────────────────────────────────
//! all `&self`， by `Db` that Mutex ： in 
//! ** 1 ** `Db::with` / `with_mut` use （ use ），
//! and not can can 。 that `inner` is ， use and 
//! [`RequestStats::guard`]。
//! in when to and T2/T3 1 ：`Db` in return `None`（ can can in in ，
//! not can ），`inner` that then `into_inner()` （ is empty）。

mod backfill;
mod clock;
// old file （ to `db::migrate` use ， not and ）。
// `pub(crate)` not is has ： in `db::migrate` ， and `request_stats`
// not is ， has not to 。
pub(crate) mod legacy;
// （ / / ） and （ / /
// ）： by module header。
mod aggregate;
mod daily;
mod maintenance;
mod record;
// （） and 「request」 。
//
// this ** not allow**： already by `api::stats_api` by ，
// is 「add 1 no use 」 will warning —— that is 。
// is `pub(super)`： has `request_stats.rs` can to ，
// for in this 1 。
mod report;
mod routing;
mod trace;
mod flow;
pub use flow::FlowQuery;
mod raw_file;
mod retention;
mod source;
mod source_report;
mod source_backfill;
mod lifecycle;
mod sql;

pub use source::{ApiKeyIdentity, RequestSource};

pub fn request_date_bounds(start: NaiveDate, end: NaiveDate) -> Option<(i64, i64)> {
    let next = end.succ_opt()?;
    Some((clock::local_midnight_ms(start), clock::local_midnight_ms(next)))
}

use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use chrono::NaiveDate;
use serde_json::{json, Value};

use crate::server::db::Db;
use crate::server::logging;

use clock::{date_key, now_ms, today};
use retention::RetentionBounds;
pub use retention::Retention;
// to ， this ：record / usage_summary 、
// remap 、（ as `pub(super)`， aggregate.rs module header）
use aggregate::{fold_into_daily, remap_model_days, summarize};
use report::{entry_json, provider_label, CACHE_WINDOWS};

// CompactStart in re-export：api::stats_api compact by by response。
// VacuumStatus in （ / maintenance ） can 。
use maintenance::VacuumStatus;
pub use maintenance::CompactStart;
pub use record::{
    AttemptDetail, NewRequestEntry, RequestEntry, RequestQuery, RetryEvent,
    RunningProgress, SensitiveHit, DEFAULT_LIMIT, MAX_LIMIT, MAX_RAW_ROWS,
};

/// ****： from when by down ， not 。
///
/// after ** 24 when ** 1 need need （ in already by 
/// `sql::sum_cache_windows` in SQL ）， from 7 to 1 ——
/// this is 「 」 this 1 （ up 26,763 → 5,975 ）。
///
/// and none ： in `last7d` by 7 （ in SQL
/// `WHERE ts >= ?` down ），`request_daily` not 
/// （ and `all` ）。
const SUMMARY_WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

/// down limit（ and provider id 1 ）。
///
/// 200 is 「 use and not 」 ： use in ， this 
/// by ， that 1 no will use down 。
/// is ****， not can —— before will current in in 
/// （ `ui/requests-panel.js` `fillFilterSelect`）。
const MAX_FILTER_OPTIONS: usize = 200;

/// 「 in 」 old ：status=0 and when 1 when as 。
///
/// Why is 1 when ： in by （`record` UPDATE）；request
/// from to minutes（ up when + ），1 when in 
/// status=0 —— （， will 
/// 1 ） and response。 no has （token 0、none），
/// in will 「 1 in request」， to 。 when in 
/// `record_started` up （ has new request when 1 ）， not need need when 。
const RUNNING_STALE_MS: i64 = 60 * 60 * 1000;

/// path up / 。
///
/// **request**（ + 1 ） need `COUNT(*)` 
/// is limit， 1 `request_raw` —— request，
/// up when this will when 。 128 or 
/// 60 seconds 1 ：limit 127 ， down 1 ； when by when ，
/// not will 「 no has new request not 」。
const MAINTENANCE_EVERY: u64 = 128;
const MAINTENANCE_INTERVAL_MS: i64 = 60_000;

/// after in by `ts` ： when is 。
/// when not is when ， can request in when in 。
fn stale_cutoff() -> i64 {
    std::cmp::max(now_ms() - RUNNING_STALE_MS, crate::server::logging::process_started_at())
}

/// requeststatistics。all `&self`， 1 `Mutex` use 。
pub struct RequestStats {
    /// SQLite 。`None` = failure（ when already ）。
    ///
    /// ── Why not 「file」─────────────────────────────────
    /// and `LogStore` / `AccountStore` 1 （ that ）， in requeststatistics up 
    /// 1 ： this in ** request path** up （`api::chat` ），
    /// will 1 、 in when 1 ——
    /// that is 1 「 no 」。statistics can ：
    /// ** 1 not **（ 1 ）。unavailable when by 
    /// empty（ ）， not file。
    db: Option<Db>,
    /// filepath（`stats()` `file` / `dailyFile` and use ）。
    ///
    /// from 「file / file」「 this file」—— and 
    /// `LogStore::file()` / `AccountStore::file()` 1 。 not when to 
    /// path（`{config_dir}/agent2api.db`）： this will in and set 
    /// （`api::storage_api` ）， to 1 has to empty has use 。
    ///
    /// Why is `RwLock` not is ： and `LogStore::file_path` 1 ——
    /// not path（`relocate` already delete）， use `RwLock`
    /// is as 「」 this and before ，。
    file_path: RwLock<PathBuf>,
    /// 。** when use **， is set down 1 /
    /// `prune` use new ， not need need （ this is parameter Reason）。
    /// use `Arc` is as `RequestStats` `ServerState`， need can has 。
    /// when use ，****（ `config::retention_settings()`），
    /// not need —— is request need 1 path。
    get_retention: Arc<dyn Fn() -> Retention + Send + Sync>,
    /// 「 1 use 」 not 1 use 。
    ///
    /// ── in to （）─────────────────────────────
    /// 1 ** not **： is `Db` that Mutex + SQLite
    /// （ in 1 ， is 1 `Db::with`）。
    /// use ； 1 by and 。
    /// and 、 store and 1 。
    /// ****： this not can `logging::log` 1 ——
    /// need 1 ，`std::sync::Mutex` not can ， will 。
    inner: Mutex<()>,
    /// （VACUUM） 。`Arc` is as in ** 1 **
    /// ， need can update this （`RequestStats` not has ）。
    ///
    /// Why not `static`：`RequestStats` is 
    /// （`ServerState` that 1 `Arc`）， in up 「 has 」
    /// ；OmniProxy use let is as router is file，
    /// etc ， this and 1 。
    vacuum: Arc<VacuumStatus>,
    /// up （ + 1 ）。
    maintenance_writes: AtomicU64,
    /// up ms when ； when use 。
    last_maintenance_ms: AtomicI64,
}

impl RequestStats {
    /// （unavailable when as empty， `db` ）。
    ///
    /// ── Why from `new(directory, ...)` `Db` ──────────
    /// statistics not has 「 directory」： in 1 ，path this by `Db`
    /// 1 has （`Db::file()`）。 and `AccountStore::with_db` /
    /// `LogStore::with_db` 1 ， store 1 ；
    /// `Option<Db>` 1 （ not when can ， is ）。
    ///
    /// ── Why not has `load()` ──────────────────────────────────
    /// before when need file、 by 1 、 for old 1 
    /// （`backfill`）。 in is ， no has can ：
    /// - 「 when 1 」 by `record` / `prune` （ up ）；
    /// - 「 old 」 can can in ** from old file up that **
    /// （ new ）， that 
    /// （`import_legacy_daily`）， not need need ；
    /// - that `[Stats] already requeststatistics N ` ——
    /// for 「」 this already not 。
    ///
    /// when use ，****（ `config::retention_settings()`），
    /// not need —— is request need 1 path。
    pub fn with_db(
        db: Option<Db>,
        get_retention: impl Fn() -> Retention + Send + Sync + 'static,
    ) -> Self {
        let file_path = match db.as_ref() {
            Some(db) => db.file().to_path_buf(),
            None => crate::server::config::config_dir().join(crate::server::db::FILE_NAME),
        };
        Self {
            db,
            file_path: RwLock::new(file_path),
            get_retention: Arc::new(get_retention),
            inner: Mutex::new(()),
            vacuum: Arc::new(VacuumStatus::new()),
            maintenance_writes: AtomicU64::new(0),
            last_maintenance_ms: AtomicI64::new(0),
        }
    }

    /// ； in （ panic） when use —— and `LogStore::guard` /
    /// `AccountStore::guard` 1 to ： this is 「 use 」 not is ，
    /// no has will panic in （ 1 by ）。
    fn guard(&self) -> MutexGuard<'_, ()> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// this 1 need not need / 。
    ///
    /// and when ， not ；return「 is 
    /// DELETE/COUNT」， and down 1 no has use 。
    fn should_maintain(&self) -> bool {
        let writes = self.maintenance_writes.fetch_add(1, Ordering::Relaxed) + 1;
        let now = now_ms();
        let last = self.last_maintenance_ms.load(Ordering::Relaxed);
        if writes == 1
            || writes % MAINTENANCE_EVERY == 0
            || now.saturating_sub(last) >= MAINTENANCE_INTERVAL_MS
        {
            self.last_maintenance_ms.store(now, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// filepath（`stats()` `file` / `dailyFile` and use ）。
    ///
    /// Why is `RwLock` not is ： and `LogStore::file_path` 1 ——
    /// not path（`relocate` and `*_file()` already 
    /// delete）， use `RwLock` is as 「」 this and before ，
    /// 。
    fn file(&self) -> PathBuf {
        match self.file_path.read() {
            Ok(guard) => guard.clone(),
            // in ：path is ， use 
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// in already before down 1 。
    ///
    /// `None` = unavailable（ / in ） or SQL failure， by use 
    /// （ Why is that ）。
    ///
    /// failure****： not can `logging::log` —— need 1 ，
    /// `std::sync::Mutex` not can ， will 。
    fn with_conn<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Option<T> {
        let db = self.db.as_ref()?;
        match db.with(action) {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                eprintln!("[Stats] 统计数据库操作失败: {error}");
                None
            }
            None => {
                eprintln!("[Stats] 统计数据库不可用（连接已标记中毒）");
                None
            }
        }
    }

    /// and [`RequestStats::with_conn`] ， to `&mut Connection` ——
    /// ** need need use **：、request and 。
    fn with_conn_mut<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Option<T> {
        let db = self.db.as_ref()?;
        match db.with_mut(action) {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                eprintln!("[Stats] 统计数据库操作失败: {error}");
                None
            }
            None => {
                eprintln!("[Stats] 统计数据库不可用（连接已标记中毒）");
                None
            }
        }
    }

    /// current（ use when and 1 ）
    fn retention(&self) -> Retention {
        (self.get_retention)().normalized()
    }

    /// （ use ms down ， use down ）。
    ///
    /// from 「」 before ， and ** use **（），
    /// is set， down 1 /， not need need 。
    fn retention_bounds(&self) -> RetentionBounds {
        RetentionBounds::of(self.retention())
    }

    /// 1 request： 1 + update（** 1 **）。
    ///
    /// ── and old 「」 etc ────────────────────────────
    /// old is 「`insert_sorted` + `guard.daily.entry(...).or_insert(...)`
    /// + by need 」； in is 「`INSERT` 1 + 、`fold_into_daily`
    /// 、 UPSERT」。 1 for ：
    /// - ****：`fold_into_daily` 1 no ，total and by 
    /// 1 （ this is 「 and = total」 ， this ）；
    /// - ****：`ORDER BY ts, row_id` and old 「 by ts 」，
    /// ts by after （`row_id` ）—— and `insert_sorted` 
    /// 「ms to in before 」 1 ；
    /// - ** can **： old in 1 「 + 」， and 
    /// need to 、 need not to ； this for ，
    /// before not to this ， not will to 「、 no 」 in 
    /// （ that is old when will down ）。
    ///
    /// ── 「 UPDATE after INSERT」（： in ）───────
    /// before `record_started` already 1 status=0 ； is 
    /// ****： by `id + status=0` UPDATE （row_id and 
    /// 「 in 」 in up not ），UPDATE 0 INSERT ——
    /// before failure etc no has started path has as 。Why not use 1 +
    /// UPSERT： id can can 、 not can 1 ，
    /// `sql::update_running_request`。path will to down ——
    /// 「 in not 、 1 1 」， and before etc 。
    ///
    /// ── unavailable when not 、 not ──────────────────────────────
    /// in requestpath up （`api::chat` `record_entry`）， failure** not can **
    /// already success request： 1 statistics is 1 。
    pub fn record(&self, entry: NewRequestEntry) {
        let record = entry.normalize();
        let bounds = self.retention_bounds();
        let maintain = self.should_maintain();
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            if !record.id.is_empty() && sql::terminal_record_exists(&tx, &record.id)? {
                return tx.commit();
            }
            // in ； no has in （ 0 ） new 
            let updated = sql::update_running_request(&tx, &record)?;
            if updated == 0 {
                sql::insert_request(&tx, &record)?;
            }
            // must when and limit。 by ，
            // `MAINTENANCE_EVERY` —— request 1 COUNT 。
            if maintain {
                sql::delete_expired_requests(&tx, bounds.requests_ms)?;
                retention::trim_requests(&tx, bounds.request_count)?;
            }
            // ：**-- that 1 **（Why not is UPDATE： `sql.rs` module header）
            //
            // ** not **（ `RequestEntry::is_test`）：
            // （request need can to 、 need can for ）， is 「」，
            // total and will 。 has this 1 
            // and `rebuild_day` that ，must —— otherwise「 1 」 will 
            // that and before for not up 。
            if !record.is_test {
                lifecycle::add_to_daily(&tx, &record)?;
            }
            daily::delete_daily_before(&tx, &bounds.daily_key)?;
            // request_raw **limit**（ `record::MAX_RAW_ROWS`）：
            // is this （ 168 KB）， not 
            // when will 1 。 is 1 `COUNT(*)`（ up 
            // ms）， when to as —— that `ORDER BY` DELETE
            // not 。
            //
            // Why in `record` not is `store_raw`： is 
            // 1 （ not path）， need need 1 。 this path
            // request， 1 new 1 。 use is 「
            // not new 」 when will （ need has request in ）—— this is need 
            // ： old not will as not new empty。
            //
            // and up 1 （`maintain`）： no has 「 by 」
            // ， 127 not will 。
            // **ignorefailure**： is ， this not this 
            // （ is this request and 1 no ）。failure 
            // in （ by `with_conn_mut` that ）。
            if maintain {
                if let Ok(raw_rows) = sql::count_raw(&tx) {
                    if raw_rows > MAX_RAW_ROWS {
                        let _ = sql::trim_raw_rows(&tx, raw_rows - MAX_RAW_ROWS);
                    }
                }
            }
            tx.commit()?;
            // fork： in file， after （ 1 ， 1 directory）。
            if maintain {
                if let Ok(ids) = sql::raw_ids(conn) {
                    raw_file::sweep(&std::collections::HashSet::from_iter(ids));
                }
            }
            Ok(())
        });
    }

    /// 1 ** in ** request（status=0； before 、success after use ）。
    ///
    /// ── Why need has this 1 ────────────────────────────────────────
    /// before 1 need etc to ：request in ** after 1 ** when 
    /// （`RecordingStream` settle）， is 1 minutes request in this 
    /// minutes for request not can 。 in 「 this request」 before to 
    /// that 1 ： can to （status=0， before as in ）， when by 
    /// `record` UPDATE —— 1 request 1 。
    ///
    /// ── （started → update → stale ）────────────────
    /// 1. `record_started`： status=0 （ has id / ts / ）；
    /// 2. `record`：UPDATE this （UPDATE 0 then no has 
    /// started —— before failure path， INSERT， as before ）；
    /// 3. if / ， in 1 —— `delete_stale_running`
    /// in `record_started` when 「 [`RUNNING_STALE_MS`] in 
    /// in 」 （ this ）。
    ///
    /// in ** not **（`fold_into_daily` by status=0 ）： no has 
    /// ， will empty 1 request、 token is 0 ——
    /// etc when will ， 1 not 、 1 not 。
    ///
    /// ── etc and ──────────────────────────────────────────────
    /// id already has in when （ use not ）；unavailable when 
    /// —— this 1 failure is 「 1 to this request」， and statistics 
    /// 「 not request」 1 to 。
    /// ── （`is_test`，schema v7）───────────────────────────
    /// and id / ts 1 when ： is ****（`api::model_test`） ，
    /// in 1 down ， is 「 in 」 that 1 1 can 。
    ///
    /// `client`（client，schema v12）：request after is ，
    /// ， in 「client」 not etc 。
    pub fn record_started(
        &self,
        id: &str,
        ts: i64,
        model: &str,
        client_model: &str,
        client_reasoning: &str,
        source: &RequestSource,
        is_test: bool,
    ) {
        if id.is_empty() {
            return;
        }
        let bounds = self.retention_bounds();
        let maintain = self.should_maintain();
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            sql::insert_started_request(
                &tx,
                id,
                ts,
                model,
                client_model,
                client_reasoning,
                source,
                is_test,
            )?;
            // old and ： when and when 1 ，
            // in 「 this has no has 」， not will down 
            lifecycle::finish_stale(&tx, stale_cutoff(), now_ms())?;
            if maintain {
                sql::delete_expired_requests(&tx, bounds.requests_ms)?;
                retention::trim_requests(&tx, bounds.request_count)?;
            }
            tx.commit()
        });
    }

    /// up 1 in （ when 1 ）。
    ///
    /// `record_started` that need etc 1 request； when 1 ，
    /// 1 not will to 「 up 1 request」。 and 1 
    /// （ `stale_cutoff()` and `finish_stale_running`）， not then 。
    pub fn sweep_stale_running(&self) {
        let bounds = self.retention_bounds();
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            lifecycle::finish_stale(&tx, stale_cutoff(), now_ms())?;
            daily::delete_daily_before(&tx, &bounds.daily_key)?;
            tx.commit()
        });
    }

    /// 1 「 in 」 ** in **（ when 1 ）。
    ///
    /// ── Why need need ────────────────────────────────────────────
    /// `record_started` that has id / ts / ， and in 
    /// ** after ** —— is that 1 「 / / up 」
    /// is empty （ before 「—」）， this its in request before already 。
    /// before `requests` up has path：（`record` UPDATE） and 
    /// 。 is ：** in **， in from 「 has 」
    /// 「 in 、 is 、 already 」。
    ///
    /// ── 、 1 ──────────────────────────────────────
    /// `api::pipeline::live_row_sink` ， by `RequestTelemetry` in **
    /// ** when use （ 1 up 1 ， `usage::LiveStamp`）——
    /// 1 request is 「」 not is 「 up 」（path
    /// 1 will `note_first_frame`， in ）。 in api 、telemetry
    /// has 1 ：core not （ to `core/mod.rs` ）。
    ///
    /// ── etc and ──────────────────────────────────────────────
    /// `id + status = 0`（ and 1 ）： after empty， in to 1 
    /// not will ； 1 request is 1 。unavailable
    /// when —— and `record_started` 1 to ： 1 in new not ，
    /// 「 in has 」 has 。
    pub fn update_running(&self, id: &str, progress: &RunningProgress) {
        if id.is_empty() {
            return;
        }
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| sql::update_running_progress(conn, id, progress));
    }

    /// only when down ， and 。
    /// by old ，failure not request。
    pub fn store_raw(&self, id: &str, ts: i64, request_body: Option<&str>, response_body: Option<&str>) {
        let id = id.trim();
        if id.is_empty() {
            return;
        }
        let request = request_body.unwrap_or("");
        let response = response_body.unwrap_or("");
        // empty not 1 ： is 「 has ⇔ has 」
        if request.is_empty() && response.is_empty() {
            return;
        }
        // size by （ / use ； not ）。
        let size = (request.len() + response.len()) as i64;
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| {
            // fork： `{cache_dir}/request-raw/{id}`， 
            // （ bytes NULL）。 in guard ：otherwise and `sweep`
            // （record / prune that ） will no new file。
            let row = raw_file::RawRow {
                id: id.to_string(),
                ts,
                request_body: request.to_string(),
                response_body: response.to_string(),
                truncated: false,
            };
            if raw_file::write(&row).is_err() {
                return Ok(());
            }
            let tx = conn.transaction()?;
            let written = sql::upsert_raw(&tx, id, ts, size)?;
            // already not in （ after to ）： ，。
            if written == 0 {
                raw_file::remove(id);
            }
            tx.commit()
        });
    }

    /// by id down ； by 。
    pub fn raw_body(&self, id: &str) -> Option<Value> {
        let id = id.trim();
        if id.is_empty() {
            return None;
        }
        // fork：file；file not in （ramdisk ） old in 。
        if let Some(row) = raw_file::read(id) {
            return Some(json!({
                "id": row.id,
                "requestBody": row.request_body,
                "responseBody": row.response_body,
                "truncated": row.truncated,
            }));
        }
        let loaded = {
            let guard = self.guard();
            self.with_conn(&guard, |conn| sql::select_raw(conn, id))
        }?;
        let Some((raw_id, request_body, response_body, truncated)) = loaded else {
            return None;
        };
        let request_body = request_body.unwrap_or_default();
        let response_body = response_body.unwrap_or_default();
        // in 、 is NULL（file already fork ）： by 「」。
        if request_body.is_empty() && response_body.is_empty() {
            return None;
        }
        Some(json!({
            "id": raw_id,
            "requestBody": request_body,
            "responseBody": response_body,
            "truncated": truncated,
        }))
    }

    /// 。， is `report` 。
    ///
    /// `range` 1 by `"7"` ， and in 1 after ：
    /// is ， as 1 parameter， not to default
    /// （ before not use as this error）。 by （`api::stats_api::stats_summary`）
    /// for to 400 —— that is use in up ， will 
    /// and for not up ； 1 ， this 。
    ///
    /// ── ────────────────────────────────────────────────
    /// （ 1 365 ）； in ** not down **（ by SQL by 
    /// ， `sql::sum_cache_windows`）；** 24 when ** 
    /// （`sql::select_summary_rows`， when ）。 in 
    /// will ， not 。
    /// `now` in this ** 1 ** and when used for up and ：
    /// ， to can can in 「 not in 」（none）
    /// or 「 in no 」（） this up ， before is 、 after is 。
    ///
    /// ：unavailable when ** by empty 1 **（ not is 1 empty JSON）——
    /// before to and 「 this when no has 」 1 ， to is 
    /// 「none」 empty， not is undefined 
    /// （`ui/report.js` `overviewHtml` / `paintRank` by `Number(x) || 0`
    /// and `Array.isArray` ，empty is 「 no has request」）。
    ///
    /// `custom` in `range == "custom"` when has （， by already 
    /// ）； its None， by `range_bounds` 。
    pub fn usage_summary(
        &self,
        range: &str,
        custom: Option<(NaiveDate, NaiveDate)>,
        api_key: Option<&str>,
    ) -> Value {
        let now = clock::now_ms();
        let windows: Vec<i64> = CACHE_WINDOWS.iter().map(|(_, ms)| *ms).collect();
        let loaded = {
            let guard = self.guard();
            self.with_conn(&guard, |conn| {
                let daily = daily::select_daily_map(conn)?;
                // in in SQL （ 1 ）
                let cache_totals = sql::sum_cache_windows(conn, now, &windows)?;
                // excluding（ `select_summary_rows` ）：
                // and is 「」
                let entries = sql::select_summary_rows(conn, now - SUMMARY_WINDOW_MS, now)?;
                Ok((daily, cache_totals, entries))
            })
        };
        let (daily, cache_totals, entries) = loaded.unwrap_or_default();
        let mut summary = summarize(&daily, &cache_totals, &entries, range, now, custom);
        let start = summary["startDate"].as_str().unwrap_or("");
        let end = summary["endDate"].as_str().unwrap_or("");
        let mut sources = source_report::source_summary(&daily, start, end, api_key);
        let keys = crate::server::core::api_keys::list();
        if let Some(rows) = sources["apiKeys"].as_array_mut() {
            for row in rows {
                if row["source"] != "key" { continue; }
                let entry = keys.iter().find(|key| Some(key.id.as_str()) == row["id"].as_str());
                row["state"] = json!(match entry { Some(key) if key.enabled => "enabled", Some(_) => "disabled", None => "deleted" });
                if let Some(key) = entry { row["label"] = json!(key.name); }
            }
        }
        if let (Some(target), Some(sources)) = (summary.as_object_mut(), sources.as_object()) {
            target.extend(sources.clone());
        }
        summary
    }

    /// ：（ new in before ）。
    ///
    /// by SQL `ORDER BY ts DESC, row_id DESC` —— and old 「 by ts
    /// ， == to 」 1 ， not will 「request
    /// 」 （ ts by `row_id` ）。
    ///
    /// and ** use 1 ** `sql::FilterPlan`（ that ）：
    /// 「 up N 」 and 「empty that 」 is 1 。
    ///
    /// response `running` is ** 1 down ** in （status=0） ：
    /// use 「 has no 」， 1 COUNT ， not 。
    ///
    /// ：unavailable when to `{entries: [], total: 0, matched: 0, running: 0}` ——
    /// before `ui/requests-panel.js` for this is `Number(result?.total) || 0`
    /// and `Array.isArray(result?.entries)`，empty will 「nonerequest」（`emptyText`），
    /// not will not will has 。**Note not will 0 not **：
    /// **failure**（HTTP error） is 1 （`render('requestfailure…')`
    /// and 「—」）， this return is 「success as empty」—— in before is has 。
    pub fn query_requests(&self, filter: &RequestQuery) -> Value {
        // limit in [1, MAX_LIMIT]： before 0 or not this 
        let limit = filter.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let plan = sql::FilterPlan::of(filter);
        let loaded = {
            let guard = self.guard();
            self.with_conn(&guard, |conn| {
                // use 1 （ `sql::counts` ）。
                let (total, matched, running) = sql::counts(conn, &plan)?;
                let entries = sql::select_page_desc(conn, &plan, filter.offset, limit)?;
                Ok((total, matched, running, entries))
            })
        };
        let (total, matched, running, entries) = loaded.unwrap_or((0, 0, 0, Vec::new()));
        // `entry_json` ：`providerLabel`（id → label ；
        // and total providers 1 ， 1 ） and 
        // `phaseElapsedMs`（ in 「current already 」， that 1 ——
        // by that ）。total is ** Value**， not is 1 ：
        // by record.rs serde ， this not 
        let rows: Vec<Value> = entries.iter().map(entry_json).collect();
        json!({
            "entries": rows,
            "total": total,
            "matched": matched,
            "running": running,
        })
    }

    /// down ： and provider（ by ）。
    ///
    /// `{models: ["…"], providers: [{id, label}]}` —— provider label is 
    /// as before not id → （ and `providerLabel`
    /// 1 ， `report::provider_label`）。
    ///
    /// ── Why is not is response ──────────────────────
    /// and 「 this 1 」none： in when 1 ， after / 
    /// not this （ 1 GROUP BY to request up ， is ）。，
    /// response 1 will ， 1 use not up 。
    ///
    /// ：unavailable when to empty（`{models: [], providers: []}`）——
    /// before 「」，unavailable not ；
    /// and `query_requests` empty 1 to （ 1 can failure）。
    pub fn filter_options(&self) -> Value {
        let loaded = {
            let guard = self.guard();
            self.with_conn(&guard, |conn| {
                let (models, providers) = sql::select_filter_options(conn, MAX_FILTER_OPTIONS)?;
                let sources = sql::source_filter_options(conn, MAX_FILTER_OPTIONS)?;
                Ok((models, providers, sources))
            })
        };
        let (models, providers, mut sources) = loaded.unwrap_or((Vec::new(), Vec::new(), json!({})));
        if let Some(clients) = sources["clients"].as_array_mut() {
            for client in clients {
                client["label"] = json!(crate::server::core::client::display_name(
                    client["value"].as_str().unwrap_or(""), client["label"].as_str().unwrap_or(""),
                ));
            }
        }
        let providers: Vec<Value> = providers
            .into_iter()
            .map(|id| json!({ "id": id, "label": provider_label(&id) }))
            .collect();
        json!({ "models": models, "providers": providers, "clients": sources["clients"], "apiKeys": sources["apiKeys"] })
    }

    /// （ and use ）
    ///
    /// `file` / `dailyFile` to **file**（ not has file and file，
    /// by current 1 （ + by ）。
    ///
    /// use is `stats_api::put_retention`：set after ，
    /// otherwise old need etc down 1 request， will 「set is 7 ，
    /// has 30 before 」。`record` will 
    /// （ this ）， is 「set up 」 that 1 down 。
    ///
    /// ：unavailable when not —— `prune` is 「」，
    /// no has return can failure（ use `stats_api` returnresponse），
    /// failure already by `with_conn_mut` 1 。
    pub fn prune(&self) {
        let bounds = self.retention_bounds();
        let guard = self.guard();
        let _ = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            sql::delete_expired_requests(&tx, bounds.requests_ms)?;
            retention::trim_requests(&tx, bounds.request_count)?;
            daily::delete_daily_before(&tx, &bounds.daily_key)?;
            tx.commit()?;
            // fork：file by 1 （ / for file）。
            if let Ok(ids) = sql::raw_ids(conn) {
                raw_file::sweep(&std::collections::HashSet::from_iter(ids));
            }
            Ok(())
        });
        // （ / ） after ： down 1 from new 1 。
        self.maintenance_writes.store(0, Ordering::Relaxed);
        self.last_maintenance_ms.store(now_ms(), Ordering::Relaxed);
    }

    /// `file_path` ）。`firstTs` / `lastTs` `ts` / ，
    /// and old 「 ts」 etc 。
    ///
    /// ：unavailable when to 0 and `firstTs` / `lastTs` as `null`。
    /// `api::storage_api` is `total` and `dailyDays`（set 
    /// 「request N / N 」）；unavailable when that is 0，
    /// `storage_api` will when to `available: false`，「—」
    /// not is 1 0（0 and 「 no has 」 in up none）。
    pub fn stats(&self) -> Value {
        let loaded = {
            let guard = self.guard();
            self.with_conn(&guard, |conn| {
                let total = sql::count_all(conn)?;
                let days = daily::count_daily(conn)?;
                let first = sql::min_ts(conn)?;
                let last = sql::max_ts(conn)?;
                let downstream_raw = sql::count_raw(conn)?;
                let upstream_raw = sql::count_upstream_raw(conn)?;
                Ok((total, days, first, last, downstream_raw, upstream_raw))
            })
        };
        let (total, daily_days, first_ts, last_ts, downstream_raw, upstream_raw) =
            loaded.unwrap_or((0, 0, None, None, 0, 0));
        let file = self.file().to_string_lossy().to_string();
        json!({
            "total": total,
            "dailyDays": daily_days,
            "downstreamRaw": downstream_raw,
            "upstreamRaw": upstream_raw,
            "file": file,
            // and file ： and 1 。 this is as not 
            // has （`file` already file），** not need 
            // ** —— before before this ， to 1 file，
            // is 。set in `database.bytes` 1 。
            "dailyFile": file,
            "firstTs": first_ts,
            "lastTs": last_ts,
            "today": date_key(today()),
        })
    }

    /// when ： WAL and file and `-wal` / `-shm` 。
    ///
    /// ── Why not is 「 」────────────────────────
    /// old has 1 （ 50 or up 60 seconds 
    /// file），pathmust `flush()` 1 ，otherwise use to request in down after 
    /// will 1 （`server/mod.rs` is this ）。 after **
    /// **， no has 「 no 」 ， that by 。
    ///
    /// ── that Why need 1 checkpoint ────────────────────────────
    /// Note** not is ** as ：WAL is （`synchronous=NORMAL`
    /// down if already ， not will ）， already down when 
    /// by WAL 。 has in ：
    /// 1. will after file is —— use 、、「 use 」 when 
    /// has 1 file is （ not checkpoint in `-wal` ，
    /// after not when WAL； in statistics and WAL total use ）；
    /// 2. `-wal` / `-shm` file， not 1 「 up has 
    /// WAL」 。
    /// is when 1 check（ WAL ）—— path up 1 
    /// IO， use not to （ in MB ）。
    /// `TRUNCATE` will in check after WAL 0 ； if in has 
    /// （ has 1 ， not will ），SQLite will WAL not —— this 
    /// ignorereturn is ：failure「 this no 」，。
    pub fn flush(&self) {
        let guard = self.guard();
        let _ = self.with_conn(&guard, |conn| {
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
            Ok(())
        });
    }

    /// 1 ： 「 by request（）」 ，
    /// use by new （ up ， `model_stat_key`）。
    ///
    /// ── Why need need ──────────────────────────────────────────
    /// before ， is request —— client
    /// when ，usage in down （ 1 up in ）。
    /// 1 has `upstreamModel`（schema v2 has ）， to 
    /// can ， not is 。
    ///
    /// ── Why is kv + 1 ──────────────────────────
    /// and `backfill::rebuild_legacy_days`（） not ： that 
    /// 「 accountStats」 can from （）， etc ；
    /// this （ is ） and for （ is ） in ** 1 **，
    /// no has can ， etc must（`kv` ， and `legacy::MARKER_*`
    /// 1 ）。
    ///
    /// ── not not ──────────────────────────────────────────
    /// use `backfill` ： can can use ，
    /// 「 ≥ requests」，；
    /// not —— use will empty，
    /// 。 already ** not **： that no has 。
    ///
    /// ── use when （ and `account_bootstrap` 1 ）──────────────
    /// when 1 （`server/mod.rs`）； old file no when （`upgrade_pending`）
    /// —— will etc ， old not to ；
    /// after by `api::upgrade_api` 1 。 not will 
    /// （ + ， up ）。
    pub fn remap_model_dimension_once(&self) {
        // not can in `inner` （ will 1 ，`Mutex` not can ，
        // `guard` ）， ， after 
        let outcome = {
            let guard = self.guard();
            self.with_conn_mut(&guard, remap_model_days)
        };
        match outcome {
            Some(changed) if !changed.is_empty() => logging::log(
                "[Stats]",
                &format!(
                    "🔁 已按上游真名重算 {} 天的模型用量（此前按请求名 / 映射别名累计）：{}",
                    changed.len(),
                    changed.join("、")
                ),
            ),
            Some(_) => {}
            // Err（ already by with_conn_mut ） and unavailable in this 1 ：
            // no ， down ， not will 
            None => {}
        }
    }
}
