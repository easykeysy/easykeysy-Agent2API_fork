//! requeststatistics ：、 and 。
//!
//! up and down use request ID。 by in ID ，none
//! when empty；delete before must， and in 1 in 。
//! not can use logging::log， will 1 。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::server::logging;

use super::clock::now_ms;
use super::record::RequestQuery;
use super::{daily, sql, RequestStats};

#[derive(Clone, Copy, Default)]
pub struct ClearResult {
    pub deleted: usize,
    pub raw: usize,
    pub downstream_raw: usize,
    pub upstream_raw: usize,
}

impl RequestQuery {
    pub fn has_filters(&self) -> bool {
        !sql::FilterPlan::of(self).is_empty()
    }
}

impl RequestStats {
    /// ， or when request；nonedelete when 。
    pub fn clear_requests(&self, filter: &RequestQuery, raw_only: bool) -> Option<ClearResult> {
        let plan = sql::FilterPlan::of(filter);
        self.with_conn_mut(&self.guard(), |conn| {
            let tx = conn.transaction()?;
            let raw = sql::delete_raw_matching(&tx, &plan)?;
            let deleted = if raw_only {
                raw.requests
            } else if plan.is_empty() {
                let removed = sql::delete_all_requests(&tx)?;
                daily::delete_all_daily(&tx)?;
                removed
            } else {
                sql::delete_matching(&tx, &plan)?
            };
            tx.commit()?;
            // fork：file not in ， by 1 that 。
            if let Ok(ids) = sql::raw_ids(conn) {
                super::raw_file::sweep(&std::collections::HashSet::from_iter(ids));
            }
            Ok(ClearResult {
                deleted,
                raw: raw.requests,
                downstream_raw: raw.downstream,
                upstream_raw: raw.upstream,
            })
        })
    }

    pub fn clear_preview(&self, filter: &RequestQuery) -> Option<Value> {
        let plan = sql::FilterPlan::of(filter);
        let (all, raw) = self.with_conn(&self.guard(), |conn| {
            let all = sql::count_matching(conn, &plan)?;
            let raw = sql::count_raw_matching(conn, &plan)?;
            Ok((all, raw))
        })?;
        Some(json!({
            "all": all,
            "raw": raw.requests,
            "downstreamRaw": raw.downstream,
            "upstreamRaw": raw.upstream,
            "dbBytes": self.db_bytes(),
            "vacuumRunning": self.vacuum.is_running(),
            "lastVacuum": self.vacuum.last_finished(),
        }))
    }

    /// file use （ + WAL，）。
    ///
    /// WAL is already 1 （checkpoint before is 「 new 」），
    /// will use —— 「 can 」 file need 。
    /// not to （file not in / none） by 0：set 0 500 。
    pub fn db_bytes(&self) -> u64 {
        let file = self.file();
        file_size(&file) + file_size(&PathBuf::from(format!("{}-wal", file.to_string_lossy())))
    }

    /// ：`checkpoint(TRUNCATE)` → `VACUUM` → `checkpoint(TRUNCATE)`
    /// —— checkpoint 1 not can （ by down ）， in that VACUUM 
    /// fileempty。** after **，return。
    ///
    /// ── Why VACUUM after need checkpoint 1 （issue #297）────
    /// WAL down `VACUUM` is ** 1 **， checkpoint in 
    /// **** —— not 。 is after will `-wal`，
    /// VACUUM that checkpoint and ** not WAL file**：
    /// ，`-wal` and etc 1 already empty。
    ///
    /// in before 1 （ in #297 before ） will this issue：
    /// 「 126MB + WAL 0MB」 1 「110MB + 111MB」，total use 
    /// from 126MB to 221MB（1.76 ）—— and #297 624MB → 1.09GB（1.747 ）
    /// 1 。 up checkpoint after 1 「110MB + 0MB」， down 。
    ///
    /// is 「 before `-wal` already 」（= 、 or up ）：
    /// that when no has can WAL ， use to 「 up that 」。
    /// if WAL is （ 1 no ）， ， is 
    /// down —— this is 1 bug 1 use 「」 Reason。
    ///
    /// ── Whymust that ──────────────────────────────────
    /// is 「 rusqlite + `Mutex`」：VACUUM not in 
    /// ， and must and its —— 1 is `Db::with`（）。
    /// its （、、 API） will ：file
    /// MB，VACUUM seconds， this can —— this is 
    /// 「 after + use 」 not is path by 。
    ///
    /// ── Why is std::thread not is tokio::spawn ────────────────
    /// `db.with` is **** use （ not 1 etc ）， will 1 
    /// tokio worker ； use in when ，。
    ///
    /// ── and and ──────────────────────────────────────────────
    /// in ** before **（`VacuumStatus::try_start`）， by 
    /// use to 409； when none（failure is 1 ）。
    pub fn compact(&self) -> CompactStart {
        if !self.vacuum.try_start() {
            return CompactStart::AlreadyRunning;
        }
        let Some(db) = self.db.clone() else {
            // unavailable：（otherwise vacuumRunning true）， up 
            self.vacuum.mark_finished();
            return CompactStart::Unavailable;
        };
        let vacuum = Arc::clone(&self.vacuum);
        std::thread::spawn(move || {
            let outcome = db.with(|conn| -> rusqlite::Result<()> {
                db.invalidate_accounts();
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
                conn.execute_batch("VACUUM;")?;
                // checkpoint： VACUUM WAL that and and 。
                // ， empty from to `-wal`，total use not （#297）。
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
                db.invalidate_accounts();
                Ok(())
            });
            db.invalidate_accounts();
            let ok = matches!(outcome, Some(Ok(())));
            // ： need 1 （）， if ，
            // will 1 will ； after and request and not 
            vacuum.mark_finished();
            if ok {
                logging::log("[Stats]", "数据库压缩完成（checkpoint + VACUUM + checkpoint）");
            } else {
                logging::log("[Stats]", "⚠️ 数据库压缩未完成：数据库不可用或压缩失败");
            }
        });
        CompactStart::Started
    }
}

/// [`RequestStats::compact`] （ to has response）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactStart {
    /// already in after 
    Started,
    /// already has in （ use to 409）
    AlreadyRunning,
    /// unavailable，none from （ use not is 「 already 」）
    Unavailable,
}

/// [`RequestStats::compact`] 。
///
/// Why in `RequestStats` up not is `static`：`RequestStats` is 
/// （`ServerState` that 1 `Arc`）， in up 「 has 
/// 」；OmniProxy use let is as router is file
/// ， etc ， this and 1 。
pub(super) struct VacuumStatus {
    /// is has in （`swap` 「 + 」， and ）
    running: AtomicBool,
    /// 1 **** when （ms Unix；None = no ）。
    /// none —— 「」includingfailure（failure is 1 ， before ）。
    last_finished_at: Mutex<Option<i64>>,
}

impl VacuumStatus {
    pub(super) fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            last_finished_at: Mutex::new(None),
        }
    }

    /// use 「 in 」：return false already has in 。
    ///
    /// use `swap` not is 「 load store」： and in 1 ，
    /// and request has 1 can to `false` old —— not need need 。
    fn try_start(&self) -> bool {
        !self.running.swap(true, Ordering::AcqRel)
    }

    fn mark_finished(&self) {
        if let Ok(mut slot) = self.last_finished_at.lock() {
            *slot = Some(now_ms());
        }
        self.running.store(false, Ordering::Release);
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    fn last_finished(&self) -> Option<i64> {
        self.last_finished_at.lock().ok().and_then(|slot| *slot)
    }
}

/// file（ not to by 0；`db_bytes` ）
fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}
