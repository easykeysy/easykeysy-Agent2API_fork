//! ：**、、**。
//!
//! ```text
//! build(db) → snapshot.json（， ）
//! apply(db, …) snapshot.json → （，）
//! ```
//!
//! ── Why not is 「」 use ─────────────────────────────
//! `db::backup` is ****（、request、），current
//! this has 200MB and is 。 need is 「 use 」
//! that —— 、config、 to and ； is ** **，
//! up no has （ after request will ）。
//!
//! for 「」 not ： is use path、；
//! is file to ，（`panelTokens` /
//! `panelAccessSessions` / `panelAdmin`）、set and after 1 not 。
//!
//! ── Why is ** **（）───────────
//! `accounts.data` and `kv.apiKeys` in already is `enc1:` （ is 
//! `account.key`）。 if in when ， etc 1 
//! need up to file —— and 「」 。 ：
//! · file not （ has + file）；
//! · use when not need need 「」， 1 and 1 failure；
//! · and 1 （ that is SQL ）。
//!
//! ── kv （ 1 ）─────────────────────────
//! [`is_synced_kv_key`]。excluding：
//! 1. **path**：`logDir` / `requestStatsDir` / `debugDir` / `kernelDir`
//! is for path， to not in directory；
//! 2. ** 1 **：`githubToken` use `update-token.key`， that 
//! file not in （ updatecheck）—— not ；
//! 3. ****：、set、 after 、、
//! 。 need and this ， need can from 。
//!
//! ── use Why is 「」 not is 「 and 」────────────────────────
//! and 1 to ： is 「 this 1 」。
//! and （ that ） is 1 （、），
//! use will 「 down ，Why in 」none from 。

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::server::db::schema::{self, SCHEMA_VERSION};
use crate::server::db::Db;
use crate::server::logging;

use super::manifest::{ArtifactMeta, SyncManifest};

/// file （ and manifest format ： 1 is 「」，
/// 1 is 「」， will can can its in 1 ）
pub(crate) const SNAPSHOT_FORMAT: &str = "agent2api-cloud-sync-snapshot";
pub(crate) const SNAPSHOT_VERSION: u32 = 1;

/// （： its ，failure when 1 to 1 ）
pub(crate) const SYNCED_TABLES: &[&str] = &["accounts", "account_usage_records", "checkin_records", "account_limiter_events"];

/// config** not ** （ by module headerexcluding）。
const EXCLUDED_CONFIG_KEYS: &[&str] = &[
    // for path
    "logDir",
    "requestStatsDir",
    "debugDir",
    "kernelDir",
    // 1 （update-token.key）， not 
    "githubToken",
    // down ：
    "kernelProxy",
    crate::server::config::KEY_INSTALLED_UPDATE_VERSION,
];

/// **** （ its 1 not ）。
///
/// this is 「 use 」 not is 「 this 」：
/// · `agentProfiles`：「Agent」 clientconfig（ need to configfile that ）
/// · `proxyPool` / `proxySettings` / `subscriptions`：「」 、
/// and 
/// · `sidebarLayout`： / / 。 is use 1 1 
/// ， this （ and / not —— that is 「 this 
/// 」， is 「 this use this 」）
const SYNCED_RESERVED_KEYS: &[&str] = &[
    "agentProfiles",
    "proxyPool",
    "proxySettings",
    "subscriptions",
    "sidebarLayout",
];

/// this kv need not need 。** 1 ** —— and use ，
/// 「 when excluding、 when 」 this 。
pub(crate) fn is_synced_kv_key(key: &str) -> bool {
    if SYNCED_RESERVED_KEYS.contains(&key) {
        return true;
    }
    if schema::is_reserved(key) {
        // its ：（ / set / / /
        // / cloudSync ）
        return false;
    }
    !EXCLUDED_CONFIG_KEYS.contains(&key)
}

/// file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    pub format: String,
    pub version: u32,
    pub schema_version: i64,
    /// when （ms）
    pub exported_at: i64,
    /// → （ is 「 → 」）
    pub tables: BTreeMap<String, Vec<Map<String, Value>>>,
    /// kv → ** JSON **（ value ）
    pub kv: BTreeMap<String, String>,
}

impl Snapshot {
    /// ：used for「 no not up 」 and manifest snapshotId。
    ///
    /// ** after **： by `BTreeMap` and 
    /// `serde_json` ， not 。
    pub(crate) fn content_hash(&self) -> String {
        match serde_json::to_vec(self) {
            Ok(bytes) => super::sha256_hex(&bytes),
            Err(_) => String::new(),
        }
    }

    /// need （「 this has 」）
    pub(crate) fn row_counts(&self) -> Value {
        let mut counts = Map::new();
        for (table, rows) in &self.tables {
            counts.insert(table.clone(), json!(rows.len()));
        }
        counts.insert("kv".to_string(), json!(self.kv.len()));
        Value::Object(counts)
    }
}

/// 。
///
/// ：、usage、 to and kv no has 1 need 
/// （ not is 1 ），** 1 ** and config
/// when —— 1 is 。
pub(crate) fn build(db: &Db) -> Result<Snapshot, String> {
    let exported_at = logging::now_ms();
    db.with(|conn| build_conn(conn, exported_at))
        .ok_or_else(|| "数据库连接不可用，无法生成同步快照".to_string())?
}

fn build_conn(conn: &Connection, exported_at: i64) -> Result<Snapshot, String> {
    let mut tables = BTreeMap::new();
    for table in SYNCED_TABLES {
        tables.insert((*table).to_string(), read_table(conn, table)?);
    }

    let mut kv = BTreeMap::new();
    let mut stmt = conn
        .prepare("SELECT key, value FROM kv ORDER BY key")
        .map_err(|error| format!("读取配置失败: {error}"))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| format!("读取配置失败: {error}"))?;
    for row in rows {
        let (key, value) = row.map_err(|error| format!("读取配置失败: {error}"))?;
        if is_synced_kv_key(&key) {
            kv.insert(key, value);
        }
    }

    Ok(Snapshot {
        format: SNAPSHOT_FORMAT.to_string(),
        version: SNAPSHOT_VERSION,
        schema_version: SCHEMA_VERSION,
        exported_at,
        tables,
        kv,
    })
}

/// 1 （ → JSON ）。
///
/// `PRAGMA table_info`， and 1 —— will in 
/// down when new （ this after is 「 1 」）。
fn read_table(conn: &Connection, table: &str) -> Result<Vec<Map<String, Value>>, String> {
    let columns = table_columns(conn, table)?;
    if columns.is_empty() {
        return Ok(Vec::new());
    }
    let quoted = columns
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {quoted} FROM {}",
            quote_identifier(table)
        ))
        .map_err(|error| format!("读取 {table} 失败: {error}"))?;
    let mut rows = stmt
        .query([])
        .map_err(|error| format!("读取 {table} 失败: {error}"))?;
    let mut out = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| format!("读取 {table} 失败: {error}"))?
    {
        let mut entry = Map::new();
        for (index, column) in columns.iter().enumerate() {
            let value = row
                .get_ref(index)
                .map_err(|error| format!("读取 {table}.{column} 失败: {error}"))?;
            entry.insert(column.clone(), sql_to_json(value));
        }
        out.push(entry);
    }
    Ok(out)
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({})", quote_identifier(table)))
        .map_err(|error| format!("读取 {table} 结构失败: {error}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| format!("读取 {table} 结构失败: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("读取 {table} 结构失败: {error}"))?;
    Ok(columns)
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn sql_to_json(value: rusqlite::types::ValueRef<'_>) -> Value {
    use rusqlite::types::ValueRef;
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(number) => json!(number),
        ValueRef::Real(number) => json!(number),
        ValueRef::Text(bytes) => match std::str::from_utf8(bytes) {
            Ok(text) => Value::String(text.to_string()),
            // UTF-8 in not in （ by ），
            // by base64 to null —— not this 
            Err(_) => Value::Null,
        },
        ValueRef::Blob(bytes) => json!(base64_encode(bytes)),
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;
    STANDARD.encode(bytes)
}

// ─── use ─────────────────────────────────────────────────────

/// use 1 to （）。
///
/// return `{ tables: {: }, kv: n }` to 。
///
/// ── ──────────────────────────────────────────────────
/// can can and current not 1 （ old / update ）。 this by 
/// **current **： ignore（ not ）， 
/// default。`schema_version` in `manifest::validate_compat`
/// already ，「 」 can can is down 。
///
/// ── kv ─────────────────────────────────────
/// if `DELETE FROM kv` ， 、set、 will 1 ——
/// use down 1 。 kv `is_synced_kv_key` in 。
pub(crate) fn apply(db: &Db, snapshot: &Snapshot) -> Result<Value, String> {
    if snapshot.format != SNAPSHOT_FORMAT {
        return Err(format!(
            "远端快照格式不兼容（format = {}）",
            snapshot.format
        ));
    }
    if snapshot.version != SNAPSHOT_VERSION {
        return Err(format!(
            "远端快照协议版本不兼容（v{}，本机 v{SNAPSHOT_VERSION}）",
            snapshot.version
        ));
    }
    if snapshot.schema_version > SCHEMA_VERSION {
        return Err(format!(
            "远端快照来自更新版本的应用（库版本 {} > 当前 {SCHEMA_VERSION}）",
            snapshot.schema_version
        ));
    }

    let applied = db.with_mut(|conn| {
        db.invalidate_accounts();
        let applied = apply_conn(conn, snapshot);
        db.invalidate_accounts();
        applied
    })
        .ok_or_else(|| "数据库连接不可用，无法应用同步快照".to_string())?;
    db.invalidate_accounts();
    applied
}

fn apply_conn(conn: &mut Connection, snapshot: &Snapshot) -> Result<Value, String> {
    let tx = conn
        .transaction()
        .map_err(|error| format!("开启同步应用事务失败: {error}"))?;

    let mut applied = Map::new();
    for table in SYNCED_TABLES {
        let Some(rows) = snapshot.tables.get(*table) else {
            // no has this = that 1 no this ，（ not empty）
            continue;
        };
        let current_columns = table_columns(&tx, table)?;
        if current_columns.is_empty() {
            continue;
        }
        tx.execute(&format!("DELETE FROM {}", quote_identifier(table)), [])
            .map_err(|error| format!("清空 {table} 失败: {error}"))?;
        let mut inserted = 0usize;
        for row in rows {
            insert_row(&tx, table, &current_columns, row)?;
            inserted += 1;
        }
        applied.insert((*table).to_string(), json!(inserted));
    }

    // kv： ， 
    let mut stale = Vec::new();
    {
        let mut stmt = tx
            .prepare("SELECT key FROM kv")
            .map_err(|error| format!("读取配置键清单失败: {error}"))?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|error| format!("读取配置键清单失败: {error}"))?;
        for row in rows {
            let key = row.map_err(|error| format!("读取配置键清单失败: {error}"))?;
            if is_synced_kv_key(&key) {
                stale.push(key);
            }
        }
    }
    for key in &stale {
        tx.execute("DELETE FROM kv WHERE key = ?1", rusqlite::params![key])
            .map_err(|error| format!("删除配置项 {key} 失败: {error}"))?;
    }
    for (key, value) in &snapshot.kv {
        // if （ old / ）， 1 ignore：
        // use and must is 1 ，otherwise down will directory。
        if !is_synced_kv_key(key) {
            continue;
        }
        tx.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )
        .map_err(|error| format!("写入配置项 {key} 失败: {error}"))?;
    }
    applied.insert("kv".to_string(), json!(snapshot.kv.len()));

    tx.commit()
        .map_err(|error| format!("提交同步应用事务失败: {error}"))?;
    Ok(Value::Object(applied))
}

fn insert_row(
    conn: &Connection,
    table: &str,
    columns: &[String],
    row: &Map<String, Value>,
) -> Result<(), String> {
    // has ； by current （ and 1 1 for ）
    let used: Vec<&String> = columns.iter().filter(|column| row.contains_key(*column)).collect();
    if used.is_empty() {
        return Ok(());
    }
    let names = used
        .iter()
        .map(|column| quote_identifier(column))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = (1..=used.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let values: Vec<rusqlite::types::Value> = used
        .iter()
        .map(|column| json_to_sql(row.get(*column).unwrap_or(&Value::Null)))
        .collect();
    conn.execute(
        &format!(
            "INSERT INTO {} ({names}) VALUES ({placeholders})",
            quote_identifier(table)
        ),
        rusqlite::params_from_iter(values),
    )
    .map(|_| ())
    .map_err(|error| format!("写入 {table} 失败: {error}"))
}

fn json_to_sql(value: &Value) -> rusqlite::types::Value {
    use rusqlite::types::Value as SqlValue;
    match value {
        Value::Null => SqlValue::Null,
        Value::Bool(flag) => SqlValue::Integer(i64::from(*flag)),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                SqlValue::Integer(integer)
            } else if let Some(real) = number.as_f64() {
                SqlValue::Real(real)
            } else {
                SqlValue::Null
            }
        }
        Value::String(text) => SqlValue::Text(text.clone()),
        // / for not is this ；， not is 
        other => SqlValue::Text(other.to_string()),
    }
}

// ─── ─────────────────────────────────────────────────

/// use before current to `{config_dir}/backups/` down ，returnfile。
///
/// and `safety_backup` 1 ： is ， use after when need has 
/// 1 can 。 this is ** 1 JSON **（ not is SQL）—— will 
/// ， this and ； use SQL will as can 
/// 。
pub(crate) fn safety_backup(snapshot: &Snapshot, config_dir: &std::path::Path) -> Result<String, String> {
    let dir = config_dir.join("backups");
    std::fs::create_dir_all(&dir).map_err(|error| format!("创建备份目录失败: {error}"))?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let name = format!("cloud-sync-{stamp}.json");
    let target = dir.join(&name);
    let bytes = serde_json::to_vec_pretty(snapshot)
        .map_err(|error| format!("序列化安全备份失败: {error}"))?;
    // when file： in failure not will down JSON available 
    let tmp = target.with_extension("json.tmp");
    std::fs::write(&tmp, &bytes).map_err(|error| format!("写入安全备份失败: {error}"))?;
    std::fs::rename(&tmp, &target).map_err(|error| {
        let _ = std::fs::remove_file(&tmp);
        format!("保存安全备份失败: {error}")
    })?;
    Ok(name)
}

// ─── manifest ───────────────────────────────────────────

/// by and （optional ）file manifest。
pub(crate) fn build_manifest(
    snapshot_bytes: &[u8],
    key_bytes: Option<&[u8]>,
    key_fingerprint: Option<String>,
) -> SyncManifest {
    let mut artifacts = BTreeMap::new();
    artifacts.insert(
        super::manifest::REMOTE_SNAPSHOT.to_string(),
        ArtifactMeta {
            sha256: super::sha256_hex(snapshot_bytes),
            size: snapshot_bytes.len() as u64,
        },
    );
    if let Some(bytes) = key_bytes {
        artifacts.insert(
            super::manifest::REMOTE_KEY_FILE.to_string(),
            ArtifactMeta {
                sha256: super::sha256_hex(bytes),
                size: bytes.len() as u64,
            },
        );
    }
    SyncManifest {
        format: super::manifest::PROTOCOL_FORMAT.to_string(),
        version: super::manifest::PROTOCOL_VERSION,
        app_version: crate::server::core::update::CURRENT_VERSION.to_string(),
        schema_version: SCHEMA_VERSION,
        device_name: super::manifest::detect_device_name(),
        created_at: chrono::Local::now().to_rfc3339(),
        snapshot_id: super::manifest::snapshot_id_of(&artifacts),
        key_fingerprint,
        artifacts,
    }
}

/// 1 kv （ / use ）
#[allow(dead_code)]
pub(crate) fn kv_value(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM kv WHERE key = ?1", [key], |row| row.get(0))
        .optional()
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synced_key_filter_excludes_machine_local_state() {
        // config：
        assert!(is_synced_kv_key("apiKeys"));
        assert!(is_synced_kv_key("customProviders"));
        assert!(is_synced_kv_key("promptMode"));
        // for path： not 
        assert!(!is_synced_kv_key("logDir"));
        assert!(!is_synced_kv_key("kernelDir"));
        // 1 ： not 
        assert!(!is_synced_kv_key("githubToken"));
        // ： ， its not 
        assert!(is_synced_kv_key("agentProfiles"));
        assert!(is_synced_kv_key("proxyPool"));
        assert!(!is_synced_kv_key("desktopSettings"));
        assert!(!is_synced_kv_key("panelTokens"));
        assert!(!is_synced_kv_key("cloudSync"));
        assert!(!is_synced_kv_key("backgroundTaskState"));
    }

    #[test]
    fn apply_rejects_newer_schema() {
        let snapshot = Snapshot {
            format: SNAPSHOT_FORMAT.to_string(),
            version: SNAPSHOT_VERSION,
            schema_version: SCHEMA_VERSION + 1,
            exported_at: 0,
            tables: BTreeMap::new(),
            kv: BTreeMap::new(),
        };
        let (db, _guard) = crate::server::db::test_temp::TempDb::open("cloud-sync-newer-schema");
        assert!(apply(&db, &snapshot).is_err());
    }
}
