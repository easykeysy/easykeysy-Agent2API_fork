//! and use SQLite ， by ， not configlimit。
//! can by need 、、， and in before 。
//! not delete； id in when 。

mod presentation;
mod sql;

pub(crate) use presentation::prepare as prepare_entry;
pub use presentation::{category_for_tag, LogGroup, LogView};

use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Duration as ChronoDuration, Local, TimeZone};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::server::config;
use crate::server::db::Db;

pub const MAX_ENTRIES: usize = 500;
pub const MAX_EVENT_ENTRIES: usize = 5_000;
pub const MAX_WARN_ENTRIES: usize = 1_000;
pub const MAX_ERROR_ENTRIES: usize = 1_000;
pub const MAX_ALERT_ENTRIES: usize = MAX_WARN_ENTRIES + MAX_ERROR_ENTRIES;
pub const MAX_DEBUG_ENTRIES: usize = 1_000;
const LEVEL_CAPACITY_WEIGHT: usize = MAX_EVENT_ENTRIES + MAX_ALERT_ENTRIES + MAX_DEBUG_ENTRIES;

pub(crate) fn retention_capacity() -> usize {
    config::retention_settings().log_count
        .clamp(config::RETENTION_MIN_COUNT, config::RETENTION_MAX_COUNT) as usize
}

pub(crate) fn level_capacity(total: usize, level: &str) -> usize {
    let weight = match level {
        "debug" => MAX_DEBUG_ENTRIES,
        "warn" => MAX_WARN_ENTRIES,
        "error" => MAX_ERROR_ENTRIES,
        _ => MAX_EVENT_ENTRIES,
    };
    ((total as u64 * weight as u64 / LEVEL_CAPACITY_WEIGHT as u64) as usize).max(1)
}
pub const DEBUG_RETENTION_DAYS: i64 = 3;
pub const LEVELS: [&str; 4] = ["debug", "info", "warn", "error"];
pub const CATEGORIES: [(&str, &str); 11] = [
    ("server", "服务"),
    ("auth", "登录"),
    ("account", "账号"),
    ("model", "模型"),
    ("upstream", "上游"),
    ("desensitize", "脱敏"),
    ("config", "配置"),
    ("checkin", "自动签到"),
    ("maintenance", "凭证自动维护"),
    ("update", "软件版本检查"),
    ("plugin", "插件"),
];

const MAX_MESSAGE_LENGTH: usize = 1000;
const MAX_DATA_KEYS: usize = 24;

/// 1 / 。
///
/// 1 need `COUNT(*)` + DELETE is ；
/// empty when will ， this no has 。 by 
/// path「 1 INSERT」， 63 （ down 1 ）。
const MAINTENANCE_EVERY: u64 = 64;

/// ： will /statistics （、、empty） +1。
///
/// 「 1 」 （ 1 seconds、statistics and 
/// 1 1 ）；must new 。 use not is 
/// `LogStore` ， is as store that path
/// （`sql_trim_capacity` in ， can bump）。
static CACHE_EPOCH: AtomicU64 = AtomicU64::new(0);
/// up already （， store none）。
static APPENDS_SINCE_TRIM: AtomicU64 = AtomicU64::new(0);

fn bump_cache_epoch() {
    CACHE_EPOCH.fetch_add(1, Ordering::Relaxed);
}

fn cache_epoch() -> u64 {
    CACHE_EPOCH.load(Ordering::Relaxed)
}

fn should_trim_now() -> bool {
    let count = APPENDS_SINCE_TRIM.fetch_add(1, Ordering::Relaxed) + 1;
    if count >= MAINTENANCE_EVERY {
        APPENDS_SINCE_TRIM.store(0, Ordering::Relaxed);
        true
    } else {
        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: u64,
    pub ts: i64,
    pub level: String,
    pub category: String,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub entries: Vec<LogEntry>,
    pub groups: Vec<LogGroup>,
    pub total: usize,
    pub matched: usize,
    pub grouped: usize,
    pub offset: usize,
    pub limit: usize,
    pub by_level: Map<String, Value>,
    pub last_id: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total: usize,
    pub max: usize,
    pub by_level: Map<String, Value>,
    pub by_category: Map<String, Value>,
    pub last_id: u64,
    pub file: String,
}

#[derive(Clone, Debug, Default)]
pub struct Query {
    pub limit: Option<usize>,
    pub offset: usize,
    pub aggregate: bool,
    pub view: LogView,
    /// ，add not will in 。
    pub until_id: Option<u64>,
    pub level: Option<String>,
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub since_id: Option<u64>,
    pub start: Option<i64>,
    /// when as [start, end)，ms。
    pub end: Option<i64>,
}

pub(crate) fn normalize_level(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    if matches!(lower.as_str(), "verbose" | "trace") {
        return "debug".to_string();
    }
    if LEVELS.contains(&lower.as_str()) {
        lower
    } else {
        "info".to_string()
    }
}

pub(crate) fn level_rank(level: &str) -> usize {
    LEVELS.iter().position(|item| *item == level).unwrap_or(1)
}

pub(crate) fn normalize_category(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    if CATEGORIES.iter().any(|(key, _)| *key == lower) {
        lower
    } else {
        "server".to_string()
    }
}

pub(crate) fn normalize_data(value: Option<&Value>) -> Option<Value> {
    let Value::Object(map) = value? else {
        return None;
    };
    let mut out = Map::new();
    for (key, item) in map.iter().take(MAX_DATA_KEYS) {
        let trimmed = match item {
            Value::String(text) if text.chars().count() > MAX_MESSAGE_LENGTH => {
                Value::String(format!(
                    "{}…",
                    text.chars().take(MAX_MESSAGE_LENGTH).collect::<String>()
                ))
            }
            other => other.clone(),
        };
        out.insert(key.clone(), trimmed);
    }
    (!out.is_empty()).then_some(Value::Object(out))
}

pub(crate) fn squeeze_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn clamp_message(text: &str) -> String {
    text.chars().take(MAX_MESSAGE_LENGTH).collect()
}

pub(crate) fn sql_raise_next_id(conn: &rusqlite::Connection, value: u64) -> rusqlite::Result<()> {
    sql::raise_next_id(conn, value)
}

pub(crate) fn sql_trim_capacity(conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
    let removed = sql::trim_to_capacity(conn, retention_capacity())?;
    // path will 、 not LogStore； use in after ，none
    // has no has need can can already in / statistics（ is new ）。
    bump_cache_epoch();
    Ok(removed)
}

pub struct LogStore {
    db: Option<Db>,
    file_path: PathBuf,
    get_retention_days: Arc<dyn Fn() -> i64 + Send + Sync>,
    inner: Mutex<()>,
    /// 。 will 「 + 」，
    /// 1 will ； this 。
    query_cache: Mutex<VecDeque<CachedQuery>>,
    /// statistics（ / ）。 and ； no has new when 
    /// use ，seconds。
    stats_cache: Mutex<Option<(u64, Stats)>>,
    /// fork： to file when （`{logs_dir}/events.jsonl`）。
    /// `None` = after （ Linux / no has / ）。
    events_file: Option<PathBuf>,
    /// file ：file +1， need not need 。
    file_gen: AtomicU64,
    /// file down id （ = and file id ）。
    next_id: AtomicU64,
    /// file （ SQLite `logs` ）。 / statistics / 
    /// use `sql` ， not 。
    mirror: Mutex<Option<Mirror>>,
}

/// file 。`gen` is when ， and `LogStore::file_gen` not 
/// file already （ / empty）， down before need 。
struct Mirror {
    conn: Connection,
    gen: u64,
}

struct CachedQuery {
    epoch: u64,
    key: String,
    result: QueryResult,
}

impl LogStore {
    pub fn with_db(
        db: Option<Db>,
        get_retention_days: impl Fn() -> i64 + Send + Sync + 'static,
    ) -> Self {
        let file_path = db
            .as_ref()
            .map(|db| db.file().to_path_buf())
            .unwrap_or_else(|| config::config_dir().join(crate::server::db::FILE_NAME));
        Self {
            db,
            file_path,
            get_retention_days: Arc::new(get_retention_days),
            inner: Mutex::new(()),
            query_cache: Mutex::new(VecDeque::new()),
            stats_cache: Mutex::new(None),
            events_file: None,
            file_gen: AtomicU64::new(0),
            next_id: AtomicU64::new(1),
            mirror: Mutex::new(None),
        }
    }

    /// fork：file after （Linux + `/mnt/ramdisk` when by 
    /// `logging::init_store` use ； no has when [`with_db`] after ）。
    ///
    /// ── Why not in ────────────────────────────────────
    /// is 「、 can 、」 1 ， and / config not is 
    /// 1 。`events.jsonl` in `{logs_dir}`（default）， ——
    /// this is 「 `agent2api.db`」 this 1 。
    ///
    /// ── Why no has 1 ──────────────────────────────
    /// path file SQLite `logs` （ no use ）， after 
    /// `sql` ：`Query` 、`presentation` 、`Stats` and 
    /// after use 1 ， not will 。
    ///
    /// ── ────────────────────────────────────────────
    /// already has （ before ） when ，file from ；
    /// not in （ can `.db` when not ， not ）。
    pub fn with_events_file(
        db: Option<Db>,
        events_file: PathBuf,
        get_retention_days: impl Fn() -> i64 + Send + Sync + 'static,
    ) -> Self {
        let migrated_max = migrate_rows_to_file(&db, &events_file);
        let stored_next = db
            .as_ref()
            .and_then(|db| db.with(sql::peek_next_id))
            .and_then(Result::ok)
            .unwrap_or(1);
        let next = stored_next.max(migrated_max.saturating_add(1)).max(1);
        Self {
            db,
            file_path: events_file.clone(),
            get_retention_days: Arc::new(get_retention_days),
            inner: Mutex::new(()),
            query_cache: Mutex::new(VecDeque::new()),
            stats_cache: Mutex::new(None),
            events_file: Some(events_file),
            file_gen: AtomicU64::new(0),
            next_id: AtomicU64::new(next),
            mirror: Mutex::new(None),
        }
    }

    fn guard(&self) -> MutexGuard<'_, ()> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn file(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn with_conn<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Option<T> {
        match self.db.as_ref()?.with(action) {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                eprintln!("[Logs] 日志数据库操作失败: {error}");
                None
            }
            None => {
                eprintln!("[Logs] 日志数据库不可用（连接已标记中毒）");
                None
            }
        }
    }

    fn with_conn_mut<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Option<T> {
        match self.db.as_ref()?.with_mut(action) {
            Some(Ok(value)) => Some(value),
            Some(Err(error)) => {
                eprintln!("[Logs] 日志数据库操作失败: {error}");
                None
            }
            None => {
                eprintln!("[Logs] 日志数据库不可用（连接已标记中毒）");
                None
            }
        }
    }

    /// fork：path 1 —— after [`with_conn`]，file after 。
    fn with_source<T>(
        &self,
        guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&Connection) -> rusqlite::Result<T>,
    ) -> Option<T> {
        let Some(path) = self.events_file.as_ref() else {
            return self.with_conn(guard, action);
        };
        let mut slot = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(mirror) = self.ensure_mirror(&mut slot, path) else {
            eprintln!("[Logs] 事件日志读取失败（{}）", path.display());
            return None;
        };
        match action(&mirror.conn) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("[Logs] 事件日志查询失败: {error}");
                None
            }
        }
    }

    fn ensure_mirror<'a>(&self, slot: &'a mut Option<Mirror>, path: &Path) -> Option<&'a Mirror> {
        let gen = self.file_gen.load(Ordering::Relaxed);
        let stale = slot.as_ref().map_or(true, |mirror| mirror.gen != gen);
        if stale {
            match build_mirror(path) {
                Some(conn) => *slot = Some(Mirror { conn, gen }),
                None => return None,
            }
        }
        slot.as_ref()
    }

    /// file after ： 1 。failure（failure not can ）。
    fn file_append(&self, record: &LogEntry) -> bool {
        if !append_entries(self.events_file.as_deref().unwrap_or(Path::new("")), &[record.clone()]) {
            return false;
        }
        self.file_gen.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// file after ： （）。
    fn file_dump(&self, conn: &Connection, path: &Path) -> bool {
        let Ok(entries) = sql::select_all_asc(conn) else {
            return false;
        };
        let mut text = String::new();
        for entry in &entries {
            if let Ok(line) = serde_json::to_string(entry) {
                text.push_str(&line);
                text.push('\n');
            }
        }
        let tmp = path.with_extension("jsonl.tmp");
        if std::fs::write(&tmp, text).is_err() {
            eprintln!("[Logs] 事件日志重写失败（{}）", tmp.display());
            return false;
        }
        if std::fs::rename(&tmp, path).is_err() {
            let _ = std::fs::remove_file(&tmp);
            return false;
        }
        true
    }

    /// file after ： by / （ `MAINTENANCE_EVERY` 1 ）。
    fn file_trim(
        &self,
        _guard: &MutexGuard<'_, ()>,
        level: &str,
        keep_id: u64,
        cutoff: i64,
        debug_cutoff: i64,
    ) {
        let Some(path) = self.events_file.clone() else {
            return;
        };
        let mut slot = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(mirror) = self.ensure_mirror(&mut slot, &path) else {
            return;
        };
        let conn = &mirror.conn;
        let capacity = retention_capacity();
        let _ = sql::delete_expired(conn, cutoff, Some(keep_id));
        let _ = sql::delete_expired_debug(conn, debug_cutoff, Some(keep_id));
        let _ = sql::trim_level_capacity(conn, level, capacity);
        let _ = sql::trim_total_capacity(conn, capacity);
        self.file_dump(conn, &path);
    }

    /// file after ：（ / ）。
    fn file_prune(&self, _guard: &MutexGuard<'_, ()>) -> usize {
        let Some(path) = self.events_file.clone() else {
            return 0;
        };
        let cutoff = self.retention_cutoff_ms();
        let debug_cutoff = self.debug_cutoff_ms();
        let capacity = retention_capacity();
        let mut slot = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(mirror) = self.ensure_mirror(&mut slot, &path) else {
            return 0;
        };
        let conn = &mirror.conn;
        let before = sql::count_all(conn).unwrap_or(0);
        let _ = sql::delete_expired(conn, cutoff, None);
        let _ = sql::delete_expired_debug(conn, debug_cutoff, None);
        let _ = sql::trim_to_capacity(conn, capacity);
        let after = sql::count_all(conn).unwrap_or(before);
        if after != before {
            self.file_dump(conn, &path);
        }
        before.saturating_sub(after)
    }

    /// file after ： by delete（ id，、）。
    fn file_clear_where(&self, _guard: &MutexGuard<'_, ()>, plan: &sql::FilterPlan) -> usize {
        let Some(path) = self.events_file.clone() else {
            return 0;
        };
        let mut slot = self
            .mirror
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(mirror) = self.ensure_mirror(&mut slot, &path) else {
            return 0;
        };
        let conn = &mirror.conn;
        let ids: Vec<u64> = match sql::select_matching_desc(conn, plan) {
            Ok(entries) => entries.iter().map(|entry| entry.id).collect(),
            Err(error) => {
                eprintln!("[Logs] 日志筛选失败: {error}");
                return 0;
            }
        };
        let removed = sql::delete_by_ids(conn, &ids).unwrap_or(0);
        if removed > 0 {
            self.file_dump(conn, &path);
        }
        removed
    }

    /// file after ：（ id 1 ， and after 1 ）。
    fn file_clear(&self, _guard: &MutexGuard<'_, ()>) -> bool {
        let Some(path) = self.events_file.as_ref() else {
            return false;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(path, b"").is_err() {
            eprintln!("[Logs] 事件日志清空失败（{}）", path.display());
            return false;
        }
        self.file_gen.fetch_add(1, Ordering::Relaxed);
        self.next_id.store(1, Ordering::Relaxed);
        if let Some(db) = self.db.as_ref() {
            let _ = db.with_mut(|conn| sql::reset_next_id(conn));
        }
        true
    }

    pub(crate) fn retention_cutoff_ms(&self) -> i64 {
        let days = (self.get_retention_days)()
            .clamp(config::RETENTION_MIN_DAYS, config::RETENTION_MAX_DAYS);
        cutoff_ms(days)
    }

    fn debug_cutoff_ms(&self) -> i64 {
        self.retention_cutoff_ms()
            .max(cutoff_ms(DEBUG_RETENTION_DAYS))
    }

    pub fn prune(&self) -> usize {
        let guard = self.guard();
        let cutoff = self.retention_cutoff_ms();
        let debug_cutoff = self.debug_cutoff_ms();
        let removed = if self.events_file.is_some() {
            self.file_prune(&guard)
        } else {
            self.with_conn_mut(&guard, |conn| {
                let tx = conn.transaction()?;
                let removed = sql::delete_expired(&tx, cutoff, None)?
                    + sql::delete_expired_debug(&tx, debug_cutoff, None)?
                    + sql::trim_to_capacity(&tx, retention_capacity())?;
                tx.commit()?;
                Ok(removed)
            })
            .unwrap_or(0)
        };
        // （ / / empty） after ： down 1 
        // not from 64 check 1 。 this no has ，limit can can 
        // already （`Stats.max` is config）， need 。
        APPENDS_SINCE_TRIM.store(0, Ordering::Relaxed);
        bump_cache_epoch();
        removed
    }

    pub fn append(&self, entry: NewEntry<'_>) -> Option<LogEntry> {
        let message = clamp_message(&squeeze_whitespace(entry.message));
        if message.is_empty() {
            return None;
        }
        let guard = self.guard();
        let cutoff = self.retention_cutoff_ms();
        let debug_cutoff = self.debug_cutoff_ms();
        let ts = entry.ts.unwrap_or_else(crate::server::logging::now_ms);
        let level = normalize_level(entry.level);
        let category = normalize_category(entry.category);
        let data = normalize_data(entry.data);
        // / by ， `MAINTENANCE_EVERY` 。
        let trim_now = should_trim_now();
        // fork：file after —— `events.jsonl`， not （ 
        // already in `with_events_file` ）。id by ： when 
        // and file id，ramdisk empty after not 。
        if self.events_file.is_some() {
            let record = presentation::prepare(LogEntry {
                id: self.next_id.fetch_add(1, Ordering::Relaxed),
                ts,
                level,
                category,
                message,
                data,
            });
            if !self.file_append(&record) {
                return None;
            }
            if trim_now {
                let cutoff = self.retention_cutoff_ms();
                let debug_cutoff = self.debug_cutoff_ms();
                self.file_trim(&guard, &record.level, record.id, cutoff, debug_cutoff);
            }
            bump_cache_epoch();
            return Some(record);
        }
        let result = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            let id = sql::allocate_next_id(&tx)?;
            let record = presentation::prepare(LogEntry {
                id,
                ts,
                level,
                category,
                message,
                data,
            });
            sql::insert(&tx, &record)?;
            if trim_now {
                sql::delete_expired(&tx, cutoff, Some(id))?;
                sql::delete_expired_debug(&tx, debug_cutoff, Some(id))?;
                let capacity = retention_capacity();
                sql::trim_level_capacity(&tx, &record.level, capacity)?;
                sql::trim_total_capacity(&tx, capacity)?;
            }
            tx.commit()?;
            Ok(record)
        });
        if result.is_some() {
            bump_cache_epoch();
        }
        result
    }

    pub fn query(&self, query: &Query) -> QueryResult {
        let guard = self.guard();
        let key = format!("{query:?}");
        if let Some(cached) = self.cached_query(&guard, &key) {
            return cached;
        }
        let plan = sql::FilterPlan::of(query);
        let limit = query.limit.unwrap_or(200).clamp(1, MAX_ENTRIES);
        let fallback = QueryResult {
            entries: Vec::new(),
            groups: Vec::new(),
            total: 0,
            matched: 0,
            grouped: 0,
            offset: query.offset,
            limit,
            by_level: empty_level_map(),
            last_id: 0,
        };
        let computed = self.with_source(&guard, |conn| -> rusqlite::Result<QueryResult> {
            let total = sql::count_all(conn)?;
            let last_id = sql::max_id(conn)?;
            let filtered = sql::select_matching_desc(conn, &plan)?;
            let matched = filtered.len();
            let mut counts: std::collections::HashMap<String, i64> =
                std::collections::HashMap::new();
            for entry in &filtered {
                *counts.entry(entry.level.clone()).or_insert(0) += 1;
            }
            let by_level = level_map(LEVELS.iter().map(|level| {
                (
                    (*level).to_string(),
                    counts.get(*level).copied().unwrap_or(0),
                )
            }));
            let (grouped, groups) =
                presentation::groups_paged(filtered, query.aggregate, query.offset, limit);
            let entries = groups.iter().map(|group| group.entry.clone()).collect();
            Ok(QueryResult {
                entries,
                groups,
                total,
                matched,
                grouped,
                offset: query.offset,
                limit,
                by_level,
                last_id,
            })
        });
        let Some(result) = computed else {
            return fallback;
        };
        self.store_query_cache(&guard, key, &result);
        result
    }

    fn cached_query(&self, _guard: &MutexGuard<'_, ()>, key: &str) -> Option<QueryResult> {
        let epoch = cache_epoch();
        let cache = self
            .query_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache
            .iter()
            .find(|entry| entry.epoch == epoch && entry.key == key)
            .map(|entry| entry.result.clone())
    }

    fn store_query_cache(&self, _guard: &MutexGuard<'_, ()>, key: String, result: &QueryResult) {
        let epoch = cache_epoch();
        let mut cache = self
            .query_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // old （epoch not ） 1 ： can can for already / empty 。
        cache.retain(|entry| entry.epoch == epoch);
        if let Some(index) = cache.iter().position(|entry| entry.key == key) {
            cache.remove(index);
        }
        cache.push_front(CachedQuery {
            epoch,
            key,
            result: result.clone(),
        });
        cache.truncate(2);
    }

    pub fn clear_where(&self, query: &Query) -> (usize, Stats) {
        let removed = {
            let guard = self.guard();
            let plan = sql::FilterPlan::of(query);
            if self.events_file.is_some() {
                self.file_clear_where(&guard, &plan)
            } else {
                self.with_conn_mut(&guard, |conn| {
                    let tx = conn.transaction()?;
                    // / in SQL 「」（ old when will 
                    // ），delete need Rust matches 1 。
                    let ids: Vec<u64> = sql::select_matching_desc(&tx, &plan)?
                        .iter()
                        .map(|entry| entry.id)
                        .collect();
                    let removed = sql::delete_by_ids(&tx, &ids)?;
                    tx.commit()?;
                    Ok(removed)
                })
                .unwrap_or(0)
            }
        };
        if removed > 0 {
            bump_cache_epoch();
        }
        (removed, self.stats())
    }

    pub fn stats(&self) -> Stats {
        let guard = self.guard();
        let epoch = cache_epoch();
        {
            let cache = self
                .stats_cache
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some((cached_epoch, stats)) = cache.as_ref() {
                if *cached_epoch == epoch {
                    return stats.clone();
                }
            }
        }
        let loaded = self.with_source(&guard, sql::stats_snapshot);
        let (total, last_id, buckets) = match loaded {
            Some(value) => value,
            _ => {
                return Stats {
                    total: 0,
                    max: retention_capacity(),
                    by_level: empty_level_map(),
                    by_category: empty_category_map(),
                    last_id: 0,
                    file: self.file_path.display().to_string(),
                }
            }
        };
        let mut by_level = empty_level_map();
        let mut by_category = empty_category_map();
        for (level, category, count) in buckets {
            let level_slot = by_level.entry(level).or_insert(Value::from(0));
            let level_total = level_slot.as_i64().unwrap_or(0) + count;
            *level_slot = Value::from(level_total);
            let category_slot = by_category.entry(category).or_insert(Value::from(0));
            let category_total = category_slot.as_i64().unwrap_or(0) + count;
            *category_slot = Value::from(category_total);
        }
        let stats = Stats {
            total,
            max: retention_capacity(),
            by_level,
            by_category,
            last_id,
            file: self.file_path.display().to_string(),
        };
        *self
            .stats_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((epoch, stats.clone()));
        stats
    }

    pub fn clear(&self) -> Stats {
        {
            let guard = self.guard();
            let changed = if self.events_file.is_some() {
                self.file_clear(&guard)
            } else {
                self.with_conn_mut(&guard, |conn| {
                    let tx = conn.transaction()?;
                    sql::delete_all(&tx)?;
                    sql::reset_next_id(&tx)?;
                    tx.commit()
                })
                .is_some()
            };
            if changed {
                APPENDS_SINCE_TRIM.store(0, Ordering::Relaxed);
                bump_cache_epoch();
            }
        }
        self.stats()
    }

    pub fn to_jsonl(&self) -> String {
        let guard = self.guard();
        let entries = self
            .with_source(&guard, sql::select_all_asc)
            .unwrap_or_default();
        render_jsonl(&entries)
    }

    pub fn to_jsonl_where(&self, query: &Query) -> String {
        let guard = self.guard();
        let plan = sql::FilterPlan::of(query);
        let mut entries = self
            .with_source(&guard, |conn| sql::select_matching_desc(conn, &plan))
            .unwrap_or_default();
        entries.reverse();
        render_jsonl(&entries)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let guard = self.guard();
        self.with_source(&guard, sql::count_all).unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn empty_level_map() -> Map<String, Value> {
    LEVELS
        .iter()
        .map(|level| (level.to_string(), Value::from(0)))
        .collect()
}

fn empty_category_map() -> Map<String, Value> {
    CATEGORIES
        .iter()
        .map(|(key, _)| (key.to_string(), Value::from(0)))
        .collect()
}

fn level_map(pairs: impl IntoIterator<Item = (String, i64)>) -> Map<String, Value> {
    let mut map = empty_level_map();
    for (level, count) in pairs {
        map.insert(level, Value::from(count.max(0)));
    }
    map
}

fn cutoff_ms(days: i64) -> i64 {
    let day = Local::now().date_naive() - ChronoDuration::days(days - 1);
    day.and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).earliest())
        .map(|value| value.timestamp_millis())
        .unwrap_or(0)
}

fn render_jsonl(entries: &[LogEntry]) -> String {
    let mut text = String::new();
    for entry in entries {
        if let Ok(line) = serde_json::to_string(entry) {
            text.push_str(&line);
            text.push('\n');
        }
    }
    text
}

pub struct NewEntry<'a> {
    pub level: &'a str,
    pub category: &'a str,
    pub message: &'a str,
    pub data: Option<&'a Value>,
    pub ts: Option<i64>,
}

/// file → （`logs` DDL， and `db::schema` 1 ）。
///
/// not ：，limit is （），
/// 。empty / —— path not as 1 。
fn build_mirror(path: &Path) -> Option<Connection> {
    let conn = Connection::open_in_memory().ok()?;
    conn.execute_batch(
        "CREATE TABLE logs (id INTEGER PRIMARY KEY, ts INTEGER NOT NULL, level TEXT NOT NULL, \
         category TEXT NOT NULL, message TEXT NOT NULL, data TEXT);",
    )
    .ok()?;
    for entry in read_entries(path) {
        let _ = sql::insert(&conn, &entry);
    }
    Some(conn)
}

/// file （failure ）。
fn read_entries(path: &Path) -> Vec<LogEntry> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            serde_json::from_str::<LogEntry>(line).ok()
        })
        .collect()
}

fn max_file_id(path: &Path) -> u64 {
    read_entries(path)
        .iter()
        .map(|entry| entry.id)
        .max()
        .unwrap_or(0)
}

/// 1 ；directory not in when 。failurereturn false（ use not ）。
fn append_entries(path: &Path, entries: &[LogEntry]) -> bool {
    if path.as_os_str().is_empty() {
        return false;
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        eprintln!("[Logs] 事件日志文件打不开（{}）", path.display());
        return false;
    };
    for entry in entries {
        let Ok(line) = serde_json::to_string(entry) else {
            continue;
        };
        if writeln!(file, "{line}").is_err() {
            eprintln!("[Logs] 事件日志追加失败");
            return false;
        }
    }
    true
}

/// ： file， from 。
///
/// return after file id（used for to ）。
/// file not when ** not ** —— can `.db` when not ， not 。
fn migrate_rows_to_file(db: &Option<Db>, path: &Path) -> u64 {
    let Some(db) = db else {
        return max_file_id(path);
    };
    let rows = db
        .with(sql::select_all_asc)
        .and_then(Result::ok)
        .unwrap_or_default();
    if rows.is_empty() {
        return max_file_id(path);
    }
    let existing: std::collections::HashSet<u64> = read_entries(path)
        .iter()
        .map(|entry| entry.id)
        .collect();
    let fresh: Vec<LogEntry> = rows
        .into_iter()
        .filter(|entry| !existing.contains(&entry.id))
        .collect();
    let max = fresh
        .iter()
        .map(|entry| entry.id)
        .max()
        .unwrap_or(0)
        .max(max_file_id(path));
    if !fresh.is_empty() && !append_entries(path, &fresh) {
        return max_file_id(path);
    }
    let cleared = db.with_mut(|conn| -> rusqlite::Result<usize> {
        let removed = sql::delete_all(conn)?;
        sql::raise_next_id(conn, max.saturating_add(1))?;
        Ok(removed)
    });
    if let Some(Err(error)) = cleared {
        eprintln!("[Logs] 旧日志行清理失败: {error}");
    }
    max
}
