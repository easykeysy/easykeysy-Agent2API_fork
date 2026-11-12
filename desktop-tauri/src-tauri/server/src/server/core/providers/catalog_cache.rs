//! ： after not 。
//!
//! ── ────────────────────────────────────────────
//! before in （ 1 `OnceLock<RwLock<…>>`）： 1 
//! ， to 。 that new if failure
//! （ / noneavailable / up ），directory in up —— up 1 to 
//! new ： up add in and `/v1/models` 1 ，
//! up already down （client in failure）。
//!
//! 「 new success that 」 1 `kv` ， directory
//! in ** when ** —— is after is up 。
//!
//! ── not valid（ has ， not need ）───────────────────────
//! need success to 1 ， this 1 to down 1 new success。 by ：
//! in is 「 can to 1 old ， not need 」——
//! 1 valid etc to after ， and 。 when by new 
//! （ TTL + when + client `/v1/models` when new ），
//! 「」。`fetchedAt` down ， and 
//! 「 this is when to 」。
//!
//! ── Why in 1 ──────────────────────────────
//! `kv` must `db::schema::RESERVED_KV_KEYS`（configexcluding
//! 「 not 」 ， that 「 not can 」）。 1 1 need ，
//! for ； 1 for 1 ， not will provider 
//! 。 is 「」， in 1 （`Db::with`）， not will 
//! update —— new is （ TTL 5 minutes）。
//!
//! by when （directory when that 1 ， 
//! `restored_state` / `restored`）： when is 「 1 、
//! that 」—— need 1 。 this is has ：「 1 
//! to 」 need in after that ， 1 can can and not 1 。
//! in 1 path up ， is ms。
//!
//! ── ────────────────────────────────────────────────
//! ```json
//! { "workbuddy": { "models": [ … ], "fetchedAt": 1758000000000 } }
//! ```
//! `models` is ** **（ after ）， when 。
//! not in this ： ， 1 ，
//! is 「 after and new after not 1 」—— 1 。
//!
//! ── ： not ────────────────────────────────
//! and `config::sql` 1 ：`Db::with` is 1 ，
//! `logging::log` / `verbose` need 1 `logs` —— `std::sync::Mutex`
//! not can ， in will 。 1 in `with` 。

use std::sync::OnceLock;

use rusqlite::{params, Connection};
use serde_json::{json, Map, Value};

use crate::server::db::Db;
use crate::server::logging;

/// `kv` （， `db::schema::RESERVED_KV_KEYS`）。
pub const KV_KEY: &str = "modelCatalogCache";

// ─── （`kv` ， not is `kv` ）─────────
//
// in in this not is ： 1 after is 「 that not 
// 、 not 」， is 「 after 」—— and 「 no 」
// none， need 。
// is ** 1 directory 1 **： by directory（Qoder / AutoClaw / Accio）
// 1 ， and 1 。

/// WorkBuddy （`/v3/config`）
pub const SCOPE_WORKBUDDY: &str = "workbuddy";
/// WorkBuddy （`/v3/config`； 1 、 1 ，
/// `providers::workbuddy::region`）
pub const SCOPE_WORKBUDDY_INTL: &str = "workbuddyIntl";
/// CatPaw（`/api/agent/maas/model-types`）
pub const SCOPE_CATPAW: &str = "catpaw";
/// AutoClaw 
pub const SCOPE_AUTOCLAW_CN: &str = "autoclawCn";
/// AutoClaw 
pub const SCOPE_AUTOCLAW_INTL: &str = "autoclawIntl";
/// CodeArts（ as snap-access； and 1 ， 1 scope）
pub const SCOPE_CODEARTS: &str = "codearts";
/// Loomy（；`GET {}/api/v1/models`，OpenAI directory）
pub const SCOPE_LOOMY: &str = "loomy";
/// MonkeyCode （`GET {}/api/v1/users/models`）
pub const SCOPE_MONKEYCODE_CN: &str = "monkeycodeCn";
/// MonkeyCode （ 1 ，）
pub const SCOPE_MONKEYCODE_INTL: &str = "monkeycodeIntl";
/// Antigravity（`POST {base}/v1internal:fetchAvailableModels`； no has parameter
/// —— `sandbox`/`daily`/`prod` is not is ，，
/// `providers::antigravity::models` module header）
pub const SCOPE_ANTIGRAVITY: &str = "antigravity";

/// AtomCode（AtomGit / GitCode） directory
pub const SCOPE_ATOMCODE: &str = "atomcode";

/// LobsterAI（ has 「 has 」） directory
pub const SCOPE_LOBSTERAI: &str = "lobsterai";

/// 1 
pub struct CachedCatalog {
    /// after （ ）
    pub models: Vec<Value>,
    /// that success new when （ms）
    pub fetched_at: i64,
}

/// （ and `config` / `core::debug_traffic` 1 ）
static DB: OnceLock<Option<Db>> = OnceLock::new();

/// 。`bootstrap` `Db` after use 1 ； use ignore（`OnceLock`）。
///
/// `Option<Db>` and `config::init` / `logging::init_store` 1 ： not 
/// when use ， as 「 not not 」， its can not 。
pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn db() -> Option<Db> {
    DB.get().and_then(|slot| slot.as_ref()).cloned()
}

/// 1 。unavailable / not in / not is JSON / this 1 no has → `None`。
///
/// `None` is 「 no has available 」， use （ to 
/// ， and before 1 ）。** not return `Err`**： not to not is need need use 
/// ， has 「 has 」 and 「 no has 」。
pub fn load(scope: &str) -> Option<CachedCatalog> {
    let db = db()?;
    let all = db.with(read_all).flatten()?;
    let entry = all.get(scope)?;
    let models = entry.get("models").and_then(Value::as_array).cloned()?;
    // empty not ： is 「 this no to 」， etc directoryempty
    if models.is_empty() {
        return None;
    }
    let fetched_at = entry.get("fetchedAt").and_then(Value::as_i64).unwrap_or(0);
    Some(CachedCatalog { models, fetched_at })
}

/// 1 （， in 1 ）。
///
/// failure（unavailable / failure） 1 verbose 、 not use ——
/// not after is 「 down can can to 」， not this 1 success 
/// directory new 。
pub fn save(scope: &str, models: &[Value], fetched_at: i64) {
    if models.is_empty() {
        return;
    }
    let Some(db) = db() else { return };
    let written = db
        .with(|conn| {
            let mut all = read_all(conn).unwrap_or_default();
            all.insert(
                scope.to_string(),
                json!({ "models": models, "fetchedAt": fetched_at }),
            );
            write_all(conn, &all)
        })
        .unwrap_or(false);
    if !written {
        logging::verbose(
            "[Models]",
            &format!("模型清单缓存写入失败（{scope}）：下次启动可能回落到内置清单"),
        );
    }
}

pub fn load_legacy(provider: &str) -> Option<CachedCatalog> {
    super::catalog_legacy::load(provider)
}

pub fn save_migrated(scope: &str, cached: CachedCatalog) -> Option<CachedCatalog> {
    let Some(db) = db() else { return Some(cached) };
    let (catalog, written) = db.with(|conn| {
        let mut all = read_all(conn).unwrap_or_default();
        if let Some(entry) = all.get(scope) {
            if let Some(models) = entry.get("models").and_then(Value::as_array) {
                if models.iter().any(|model| model.get("id").and_then(Value::as_str)
                    .is_some_and(|id| !id.trim().is_empty()))
                {
                    return (CachedCatalog {
                        models: models.clone(),
                        fetched_at: entry.get("fetchedAt").and_then(Value::as_i64).unwrap_or(0),
                    }, true);
                }
            }
        }
        all.insert(scope.to_string(), json!({ "models": cached.models, "fetchedAt": cached.fetched_at }));
        let written = write_all(conn, &all);
        (cached, written)
    })?;
    if !written {
        logging::verbose("[Models]", &format!("旧模型清单迁移写入失败（{scope}）：本次仍沿用旧缓存"));
    }
    Some(catalog)
}

/// in empty and when ； not this directory already current。
pub fn cached_scopes() -> Vec<(String, i64)> {
    let Some(db) = db() else { return Vec::new() };
    let Some(all) = db.with(read_all).flatten() else { return Vec::new() };
    all.iter()
        .filter_map(|(scope, entry)| {
            let models = entry.get("models").and_then(Value::as_array)?;
            if models.is_empty() {
                return None;
            }
            let at = entry.get("fetchedAt").and_then(Value::as_i64).unwrap_or(0);
            Some((scope.clone(), at))
        })
        .collect()
}

/// when → 「 before 」 （ use ）。
///
/// use for when not is for when ： this need is 「 this old not old 」，
/// 「3 before 」「2026-09-22 14:03:11」； and not as 1 when 。
pub fn age_text(fetched_at: i64) -> String {
    if fetched_at <= 0 {
        return "未知".to_string();
    }
    let age = logging::now_ms() - fetched_at;
    if age < 60_000 {
        "刚刚".to_string()
    } else if age < 3_600_000 {
        format!("{} 分钟前", age / 60_000)
    } else if age < 86_400_000 {
        format!("{} 小时前", age / 3_600_000)
    } else {
        format!("{} 天前", age / 86_400_000)
    }
}

/// for （ not in / → `None`， by use by 「 no has 」）。
fn read_all(conn: &Connection) -> Option<Map<String, Value>> {
    let text: Option<String> = conn
        .query_row("SELECT value FROM kv WHERE key = ?1", params![KV_KEY], |row| {
            row.get(0)
        })
        .ok();
    serde_json::from_str::<Value>(&text?).ok()?.as_object().cloned()
}

/// （ UPSERT）。
fn write_all(conn: &Connection, all: &Map<String, Value>) -> bool {
    let Ok(text) = serde_json::to_string(&Value::Object(all.clone())) else {
        return false;
    };
    conn.execute(
        "INSERT INTO kv (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KV_KEY, text],
    )
    .is_ok()
}
