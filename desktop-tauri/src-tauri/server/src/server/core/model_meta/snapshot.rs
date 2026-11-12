//! models.dev and 。
//!
//! ── 1 ──────────────────────────────────────
//! **file** has 1 （ and new 1 ，
//! `scripts/sync-modelsdev.mjs` file）， use this [`parse`]：
//! · （[`EMBEDDED`]，`include_str!` ，`source = "builtin"`）
//! · 1 `kv` up 1 new （`source = "cache"`）
//! · new to （`source = "network"`， `refresh`）
//! is **cache > builtin**： is 「 1 no has when 」， need 
//! success new 1 ， 1 use that to down 1 success new （ and 
//! `providers::catalog_cache` 「 not valid」 1 to ： can to 1 old ，
//! not need when that old ）。
//!
//! ── 1 must and 1 ────────────────────────────
//! [`key_of`] is `scripts/sync-modelsdev.mjs` `keyOf` ： id 
//! ****（ etc ， models.dev id 1 
//! lab before ），、。 1 ， will 「 new after 
//! not in 」—— this not will ， will use ，
//! to for 。
//!
//! ── 1 、 after ──────────────────────────────────────
//! is 「」 ：path（ need by 1 ）
//! ， new when 。 and directory：** not 
//! request**（ new JSON 、， after in ）。
//!
//! ── ： not ────────────────────────────────
//! and `config::sql` / `providers::catalog_cache` 1 ：`Db::with` is 
//! 1 ， `logging::log` / `verbose` need 1 `logs` ——
//! `std::sync::Mutex` not can ， in will 。

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use rusqlite::{params, Connection};
use serde_json::Value;

use crate::server::db::Db;
use crate::server::logging;

/// ： （ `scripts/sync-modelsdev.mjs`）。
const EMBEDDED: &str = include_str!("modelsdev.json");

/// `kv` （， in `db::schema::RESERVED_KV_KEYS`）。
pub const KV_KEY: &str = "modelsDevCatalog";

/// 1 （** will use **， its in ）。
///
/// and is `Option`： up （models.json schema
/// `reasoning` / `tool_call` ）， this 、 can can 
/// ， 1 —— `None` is 「 this no 」，
/// use ** not **， not is 1 `false`（ that will empty「unsupported」）。
#[derive(Clone, Debug, Default)]
pub struct Record {
    /// canonical id（`zhipuai/glm-5.3`）—— and use ， not by 
    pub canonical: String,
    pub reasoning: Option<bool>,
    pub tool_call: Option<bool>,
    pub images: Option<bool>,
    pub video: Option<bool>,
    pub context: Option<u64>,
    pub output: Option<u64>,
    /// this ** **（ by to ，models.dev ）。
    /// empty = this no to （ / no ）—— use 。
    pub levels: Vec<String>,
}

/// 1 + 。
pub struct Snapshot {
    models: HashMap<String, Record>,
    /// this when （models.dev no has ， use when ）
    generated_at: i64,
    /// `builtin` / `cache` / `network`（ and new old ）
    source: &'static str,
}

impl Snapshot {
    /// 1 （`refresh` JSON after use ；`source` `network`）。
    pub fn new(models: HashMap<String, Record>, generated_at: i64, source: &'static str) -> Self {
        Self { models, generated_at, source }
    }

    /// （ new `count`）。
    pub fn len(&self) -> usize {
        self.models.len()
    }
}

/// （）。`None` in 「 no 」 when ； 1 not will 。
static SNAPSHOT: OnceLock<RwLock<Option<Arc<Snapshot>>>> = OnceLock::new();

/// （ and `config` / `providers::catalog_cache` 1 ）。
static DB: OnceLock<Option<Db>> = OnceLock::new();

/// 。`bootstrap` `Db` after use 1 ； use ignore。
///
/// `Option<Db>` and `catalog_cache::install` 1 ： not when use ，
/// that when （ can in ， is not to new ）。
pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn db() -> Option<Db> {
    DB.get().and_then(|slot| slot.as_ref()).cloned()
}

/// ：id + + （ `keyOf` ，module header）。
pub fn key_of(id: &str) -> String {
    id.rsplit('/')
        .next()
        .unwrap_or(id)
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// by id 1 （ not to to `None` = 「 this models.dev no 」）。
///
/// path： for use 1 ， in ， this is 1 
/// 1 。
pub fn lookup(model: &str) -> Option<Record> {
    let key = key_of(model.trim());
    if key.is_empty() {
        return None;
    }
    snapshot().models.get(&key).cloned()
}

/// current （ and new ；** not ** ）。
pub fn status() -> Value {
    let snapshot = snapshot();
    serde_json::json!({
        "source": snapshot.source,
        "updatedAt": snapshot.generated_at,
        "count": snapshot.models.len(),
    })
}

/// current and when （ use ； and [`status`] ， 1 ）。
pub fn origin() -> (&'static str, i64) {
    let snapshot = snapshot();
    (snapshot.source, snapshot.generated_at)
}

/// current， need when （**cache > builtin**， by module header）。
fn snapshot() -> Arc<Snapshot> {
    let lock = SNAPSHOT.get_or_init(|| RwLock::new(None));
    if let Some(loaded) = lock.read().ok().and_then(|slot| slot.clone()) {
        return loaded;
    }
    let loaded = Arc::new(load());
    let mut slot = match lock.write() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(loaded) = slot.clone() {
        return loaded;
    }
    *slot = Some(loaded.clone());
    loaded
}

/// ： new ， no has （ or ） use 。
fn load() -> Snapshot {
    if let Some(cached) = read_cache() {
        return cached;
    }
    // is ， not 「」； this when to 1 empty not is 
    // panic —— ， is all to 「」。
    match parse(EMBEDDED, "builtin") {
        Some(snapshot) => snapshot,
        None => {
            logging::log("[Models]", "内嵌的 models.dev 快照解析失败（模型元数据将不可用，请重新构建）");
            Snapshot { models: HashMap::new(), generated_at: 0, source: "builtin" }
        }
    }
}

/// use new （ new success after use ）。
pub fn replace(snapshot: Snapshot) {
    crate::server::core::providers::catalog::invalidate();
    let lock = SNAPSHOT.get_or_init(|| RwLock::new(None));
    match lock.write() {
        Ok(mut slot) => *slot = Some(Arc::new(snapshot)),
        Err(poisoned) => *poisoned.into_inner() = Some(Arc::new(snapshot)),
    }
    crate::server::core::providers::catalog::invalidate();
}

/// 1 （ use ，module header）。
pub fn parse(text: &str, source: &'static str) -> Option<Snapshot> {
    let root: Value = serde_json::from_str(text).ok()?;
    let object = root.get("models")?.as_object()?;
    let mut models = HashMap::with_capacity(object.len());
    for (key, value) in object {
        if let Some(record) = parse_record(value) {
            models.insert(key.clone(), record);
        }
    }
    if models.is_empty() {
        return None;
    }
    Some(Snapshot {
        models,
        generated_at: root.get("generatedAt").and_then(Value::as_i64).unwrap_or(0),
        source,
    })
}

/// file（ new use ； and [`parse`] ）。
///
/// ** after **： is by 1 （） ， 1 
/// up 「」； that JSON as can diff （HashMap 
/// will new not 1 ， for when is ）。
pub fn serialize(snapshot: &Snapshot) -> String {
    let mut models = serde_json::Map::new();
    let mut keys: Vec<&String> = snapshot.models.keys().collect();
    keys.sort();
    for key in keys {
        models.insert(key.clone(), record_value(&snapshot.models[key]));
    }
    Value::Object(serde_json::Map::from_iter([
        ("generatedAt".to_string(), Value::from(snapshot.generated_at)),
        ("source".to_string(), Value::String(snapshot.source.to_string())),
        ("models".to_string(), Value::Object(models)),
    ]))
    .to_string()
}

fn record_value(record: &Record) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("canonical".to_string(), Value::String(record.canonical.clone()));
    for (key, value) in [
        ("reasoning", record.reasoning),
        ("toolCall", record.tool_call),
        ("images", record.images),
        ("video", record.video),
    ] {
        if let Some(flag) = value {
            object.insert(key.to_string(), Value::Bool(flag));
        }
    }
    for (key, value) in [("context", record.context), ("output", record.output)] {
        if let Some(number) = value {
            object.insert(key.to_string(), Value::from(number));
        }
    }
    if !record.levels.is_empty() {
        object.insert(
            "levels".to_string(),
            Value::Array(record.levels.iter().cloned().map(Value::String).collect()),
        );
    }
    Value::Object(object)
}

fn parse_record(value: &Value) -> Option<Record> {
    let object = value.as_object()?;
    let canonical = object.get("canonical").and_then(Value::as_str).unwrap_or_default().to_string();
    if canonical.is_empty() {
        return None;
    }
    Some(Record {
        canonical,
        reasoning: object.get("reasoning").and_then(Value::as_bool),
        tool_call: object.get("toolCall").and_then(Value::as_bool),
        images: object.get("images").and_then(Value::as_bool),
        video: object.get("video").and_then(Value::as_bool),
        context: object.get("context").and_then(Value::as_u64),
        output: object.get("output").and_then(Value::as_u64),
        levels: object
            .get("levels")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|level| !level.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// new （unavailable / no / → `None`， by use ）。
fn read_cache() -> Option<Snapshot> {
    let db = db()?;
    // 1 （ is 1 ）， in —— not can ，
    // failure verbose need in 
    let text = db.with(read_text).flatten();
    let snapshot = text.as_deref().and_then(|text| parse(text, "cache"));
    if snapshot.is_none() && text.is_some() {
        logging::verbose("[Models]", "models.dev 缓存解析失败，改用内嵌快照");
    }
    snapshot
}

/// new （failure verbose： not after is 「 down use 
/// 」， not this 1 success new —— and `catalog_cache::save` 1 ）。
pub fn write_cache(text: &str) {
    let Some(db) = db() else { return };
    let written = db.with(|conn| write_text(conn, text)).unwrap_or(false);
    if !written {
        logging::verbose("[Models]", "models.dev 缓存写入失败：下次启动将回落到内嵌快照");
    }
}

fn read_text(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT value FROM kv WHERE key = ?1", params![KV_KEY], |row| row.get(0)).ok()
}

fn write_text(conn: &Connection, text: &str) -> bool {
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KV_KEY, text],
    )
    .is_ok()
}
