//! ， ID；delete will when by 1 delete。
//!
//! fork：（blob） SQLite， to `{cache_dir}/assistant-images/{id}`
//! （Linux up `cache_dir` default `/mnt/ramdisk/Agent2API/cache`； and ramdisk up 
//! its can 1 to ）。
//! `assistant_images` ****（id、session_id、mime、created_at）
//! and `assistant_sessions` `ON DELETE CASCADE` ； `bytes` file。
//! in 、blob in ：ramdisk empty after will not in file， by 「
//! 」（ before ）， will —— and `logs` / `request_raw` 1 。
//! （store / fork ）file、 after ：file no not ， not 
//! empty；「file in none」 file by `prune_orphans` by 。

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use rusqlite::{params, Connection, OptionalExtension};

use crate::server::db::Db;
use crate::server::errors::GatewayError;
use crate::server::logging;

static DB: OnceLock<Option<Db>> = OnceLock::new();
const MAX_STORAGE_BYTES: i64 = 512 * 1024 * 1024;
const BLOB_DIR: &str = "assistant-images";

pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
    // when for 1 ：ramdisk up can can up after old file。
    // not when `prune_orphans` will （ can ， not can ）。
    prune_orphans();
}

fn database() -> Result<&'static Db, GatewayError> {
    DB.get()
        .and_then(Option::as_ref)
        .ok_or_else(|| GatewayError::with_status(503, "助手图片存储不可用"))
}

/// directory（fork：`{cache_dir}/assistant-images`， and `logs` 1 ramdisk then ）
fn dir() -> PathBuf {
    crate::paths::cache_dir().join(BLOB_DIR)
}

/// path（id is 64 ，file）
pub fn blob_path(id: &str) -> PathBuf {
    dir().join(id)
}

fn valid_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// fork：usage = file and （file ramdisk 0， can not quota）
/// + old in `bytes` （ NULL ）。
fn used_bytes(conn: &Connection) -> rusqlite::Result<i64> {
    let legacy: i64 = conn.query_row(
        "SELECT COALESCE(SUM(COALESCE(length(bytes), 0)), 0) FROM assistant_images",
        [],
        |row| row.get(0),
    )?;
    let mut statement = conn.prepare("SELECT id FROM assistant_images")?;
    let ids: Vec<String> = statement
        .query_map([], |row| row.get(0))?
        .filter_map(|item| item.ok())
        .collect();
    let files: i64 = ids
        .iter()
        .map(|id| fs::metadata(blob_path(id)).map(|meta| meta.len() as i64).unwrap_or(0))
        .sum();
    Ok(legacy.saturating_add(files))
}

pub fn available() -> Result<(), GatewayError> {
    let used = database()?
        .with(used_bytes)
        .ok_or_else(|| GatewayError::with_status(503, "助手图片数据库不可用"))?
        .map_err(|_| GatewayError::with_status(503, "助手图片存储无法读取"))?;
    if used >= MAX_STORAGE_BYTES {
        return Err(GatewayError::with_status(
            507,
            "助手图片存储已满，请删除不再需要的历史会话",
        ));
    }
    Ok(())
}

pub fn store(session: &str, images: Vec<(Vec<u8>, String)>) -> Result<Vec<String>, GatewayError> {
    let mut entries = Vec::with_capacity(images.len());
    for (bytes, mime) in images {
        let mut random = [0u8; 32];
        getrandom::getrandom(&mut random).map_err(|_| GatewayError::new("无法生成助手图片 ID"))?;
        let id: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        entries.push((id, bytes, mime));
    }
    let ids = entries.iter().map(|(id, _, _)| id.clone()).collect();
    // fork：quota（， and store ），file、 after 。
    // file no / no is 「 1 no 」， not empty。
    database()?.with_mut(|conn| -> Result<(), GatewayError> {
        let used: i64 = used_bytes(conn).map_err(|_| GatewayError::new("无法读取助手图片存储用量"))?;
        let added: i64 = entries.iter().map(|(_, bytes, _)| bytes.len() as i64).sum();
        if used.saturating_add(added) > MAX_STORAGE_BYTES {
            return Err(GatewayError::with_status(
                507,
                "助手图片存储已满，图片已生成但未保存，请删除不再需要的历史会话",
            ));
        }
        write_blobs(&entries)?;
        let tx = conn.transaction().map_err(|_| GatewayError::new("无法保存助手图片"))?;
        for (id, _, mime) in entries {
            tx.execute(
                "INSERT INTO assistant_images (id, session_id, mime, bytes, created_at)
                 VALUES (?1, ?2, ?3, NULL, ?4)",
                params![id, session, mime, logging::now_ms()],
            )
            .map_err(|_| GatewayError::new("助手图片保存失败，图片已生成，请勿自动重发"))?;
        }
        tx.commit().map_err(|_| GatewayError::new("助手图片保存失败，图片已生成，请勿自动重发"))
    })
    .ok_or_else(|| GatewayError::with_status(503, "助手图片数据库不可用"))??;
    Ok(ids)
}

/// fork： `{cache_dir}/assistant-images/{id}`（directory by need create）
fn write_blobs(entries: &[(String, Vec<u8>, String)]) -> Result<(), GatewayError> {
    let dir = dir();
    fs::create_dir_all(&dir).map_err(|_| GatewayError::new("助手图片保存失败（目录不可写）"))?;
    for (id, bytes, _) in entries {
        fs::write(&blob_path(id), bytes)
            .map_err(|_| GatewayError::new("助手图片保存失败（文件写入失败）"))?;
    }
    Ok(())
}

/// fork： from file；file（ramdisk ） when 
/// （ old `bytes` NULL）， not in 「」。
pub fn read(id: &str, session: Option<&str>) -> Option<(Vec<u8>, String)> {
    if !valid_id(id) {
        return None;
    }
    if let Ok(bytes) = fs::read(blob_path(id)) {
        let mime = mime_of(id, session);
        return Some((bytes, mime));
    }
    let db = database().ok()?;
    db.with(|conn| {
        conn.query_row(
            "SELECT bytes, mime FROM assistant_images WHERE id = ?1 AND (?2 IS NULL OR session_id = ?2)",
            params![id, session],
            |row| Ok((row.get::<_, Option<Vec<u8>>>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .ok()
        .flatten()
    })
    .flatten()
    .and_then(|(bytes, mime)| match bytes {
        Some(bytes) if !bytes.is_empty() => Some((bytes, mime)),
        _ => None,
    })
}

/// fork： from mime（ and `read` use ；unavailable to use ）
fn mime_of(id: &str, session: Option<&str>) -> String {
    let Some(db) = database().ok() else {
        return "image/png".to_string();
    };
    db.with(|conn| {
        conn.query_row(
            "SELECT mime FROM assistant_images WHERE id = ?1 AND (?2 IS NULL OR session_id = ?2)",
            params![id, session],
            |row| row.get(0),
        )
        .ok()
    })
    .flatten()
    .unwrap_or_else(|| "image/png".to_string())
}

/// fork： from mime （`assistant_sessions::fork` when use ；
/// will has `mime_of`， not ）
pub fn mime_of_public(id: &str) -> String {
    mime_of(id, None)
}

/// fork： by file（file in none —— after ramdisk old file）。
/// not （unavailable） not file： can ， not can 。
pub fn prune_orphans() {
    let Some(db) = database().ok() else {
        return;
    };
    let ids: Vec<String> = db
        .with(|conn| {
            let mut statement = match conn.prepare("SELECT id FROM assistant_images") {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            let rows = statement.query_map([], |row| row.get::<_, String>(0));
            match rows {
                Ok(rows) => rows.filter_map(|item| item.ok()).collect(),
                Err(_) => Vec::new(),
            }
        })
        .unwrap_or_default();
    let known: std::collections::HashSet<String> = std::collections::HashSet::from_iter(ids.iter().cloned());
    let Ok(entries) = fs::read_dir(dir()) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().is_empty() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !valid_id(&name) || !known.contains(&name) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// fork： will file（ will delete/empty when ； by ）
pub fn remove_for_session(session: &str) {
    let Some(db) = database().ok() else {
        return;
    };
    let ids: Vec<String> = db
        .with(|conn| {
            let mut statement = match conn.prepare("SELECT id FROM assistant_images WHERE session_id = ?1") {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            let rows = statement.query_map(params![session], |row| row.get::<_, String>(0));
            match rows {
                Ok(rows) => rows.filter_map(|item| item.ok()).collect(),
                Err(_) => Vec::new(),
            }
        })
        .unwrap_or_default();
    for id in ids {
        let _ = fs::remove_file(blob_path(&id));
    }
}

/// fork：emptyfile（「empty will 」； by `DELETE` ）
pub fn clear_all() {
    let Some(db) = database().ok() else {
        return;
    };
    let ids: Vec<String> = db
        .with(|conn| {
            let mut statement = match conn.prepare("SELECT id FROM assistant_images") {
                Ok(s) => s,
                Err(_) => return Vec::new(),
            };
            let rows = statement.query_map([], |row| row.get::<_, String>(0));
            match rows {
                Ok(rows) => rows.filter_map(|item| item.ok()).collect(),
                Err(_) => Vec::new(),
            }
        })
        .unwrap_or_default();
    for id in ids {
        let _ = fs::remove_file(blob_path(&id));
    }
}
