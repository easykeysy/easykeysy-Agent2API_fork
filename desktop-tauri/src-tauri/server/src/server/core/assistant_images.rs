//! 助手图片附件独立保存，消息历史只存 ID；删除会话时由外键一起删除图片。
//!
//! fork: image blobs move out of SQLite into `{cache_dir}/assistant-images/{id}`
//! (on Linux `cache_dir` defaults to `/mnt/ramdisk/Agent2API/cache`; images share the ramdisk's
//! disposable-data treatment with everything else there).
//! The `assistant_images` table keeps the **ledger** (id, session_id, mime, created_at)
//! and its `ON DELETE CASCADE` foreign key to `assistant_sessions`; only the `bytes` column moves to files.
//! Ledger in the DB, blob on disk: after a ramdisk wipe, ledger rows point at files that no longer exist; the read side treats that as image
//! not retained (broken image in the frontend); the sessions themselves are intact — same tradeoff as `logs` / `request_raw`.
//! Writers (store / fork copy) write the file first, then the ledger: a row whose file never landed stays out of the DB, leaving no
//! dangling ledger; read-side orphans ("file present, ledger absent") are cleaned up against the ledger by `prune_orphans`.

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
        // At startup, reconcile once: the ramdisk may still hold old files whose ledger rows were already cleared.
        // If the ledger can't be read, `prune_orphans` gives up on its own (better to keep orphans than to delete wrongly).
    prune_orphans();
}

fn database() -> Result<&'static Db, GatewayError> {
    DB.get()
        .and_then(Option::as_ref)
        .ok_or_else(|| GatewayError::with_status(503, "助手图片存储不可用"))
}

/// Image directory (fork: `{cache_dir}/assistant-images`, same ramdisk rule as `logs`)
fn dir() -> PathBuf {
    crate::paths::cache_dir().join(BLOB_DIR)
}

/// On-disk path for a single image (the id is 64-bit hex, safe as a filename as-is)
pub fn blob_path(id: &str) -> PathBuf {
    dir().join(id)
}

fn valid_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// fork: image usage = sum of file bytes (rows whose files the ramdisk wiped count 0 — disposable data doesn't eat quota)
/// + legacy stock left in the `bytes` column (historical rows that are non-NULL).
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
        // fork: validate quota first (inside the DB lock, so two concurrent stores don't tally separately), then write the file, finally the ledger.
        // A failed file write / failed ledger write both mean "not a single image entered the DB" — no dangling ledger is left.
    database()?.with_mut(|conn| -> Result<(), GatewayError> {
        let used: i64 = used_bytes(conn).map_err(|_| GatewayError::new("could not read assistant-image storage usage"))?;
        let added: i64 = entries.iter().map(|(_, bytes, _)| bytes.len() as i64).sum();
        if used.saturating_add(added) > MAX_STORAGE_BYTES {
            return Err(GatewayError::with_status(
                507,
                "assistant-image storage is full; the image was generated but not saved — delete unneeded old sessions",
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
            .map_err(|_| GatewayError::new("assistant-image save failed; the image was generated, do not auto-resend"))?;
        }
        tx.commit().map_err(|_| GatewayError::new("助手图片保存失败，图片已生成，请勿自动重发"))
    })
    .ok_or_else(|| GatewayError::with_status(503, "assistant-image database unavailable"))??;
    Ok(ids)
}

/// fork: image bytes land in `{cache_dir}/assistant-images/{id}` (directory created on demand)
fn write_blobs(entries: &[(String, Vec<u8>, String)]) -> Result<(), GatewayError> {
    let dir = dir();
    fs::create_dir_all(&dir).map_err(|_| GatewayError::new("assistant-image save failed (directory not writable)"))?;
    for (id, bytes, _) in entries {
        fs::write(&blob_path(id), bytes)
            .map_err(|_| GatewayError::new("assistant-image save failed (file write failed)"))?;
    }
    Ok(())
}

/// fork: read from the file; when it's missing (ramdisk wiped), fall back to legacy rows in the DB
/// (the legacy `bytes` column written by the old version, non-NULL); only when neither exists is it "image not retained".
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

/// fork: read mime from the ledger (shared with `read`'s fallback read; generic type when the DB is unavailable)
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

/// fork: public entry for reading mime from the ledger (used by `assistant_sessions::fork` when copying images;
/// the session-scoped variant stays private in `mime_of`; the read-side fallback does not narrow the scope)
pub fn mime_of_public(id: &str) -> String {
    mime_of(id, None)
}

/// fork: remove orphan files per the ledger (file present, ledger row gone — e.g. old files the ramdisk still holds after the ledger row was deleted).
/// When the ledger can't be read (DB unavailable), files are left untouched: better to keep orphans than to delete wrongly.
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

/// fork: one session's image files (cascade cleanup on session delete/clear; ledger rows are removed by the DB foreign key)
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

/// fork: clear all image files ("clear all sessions" cascade; ledger rows are removed by the DB `DELETE`)
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
