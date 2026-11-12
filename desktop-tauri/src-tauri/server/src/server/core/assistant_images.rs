//! 助手图片附件独立保存，消息历史只存 ID；删除会话时由外键一起删除图片。
//!
//! fork：图片字节体（blob）挪出 SQLite，写到 `{cache_dir}/assistant-images/{id}`
//! （Linux 上 `cache_dir` 默认即 `/mnt/ramdisk/Agent2API/cache`；图片与 ramdisk 上
//! 的其它可丢弃数据同一取向）。
//! 库里的 `assistant_images` 表保留**台账**（id、session_id、mime、created_at）
//! 与 `assistant_sessions` 的 `ON DELETE CASCADE` 外键关系；只把 `bytes` 列搬进文件。
//! 台账在库、blob 在盘：ramdisk 重启清空后台账行会指着不存在的文件，读侧按「图片
//! 未保留」处理（前端裂图），会话本身完好——与 `logs` / `request_raw` 同一取舍。
//! 写侧（store / fork 复制）先落文件、后写台账：文件没落成的行不进库，不留
//! 悬空台账；读侧「文件在而台账无」的孤儿文件由 `prune_orphans` 按台账清掉。

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
    // 启动时对一次账：ramdisk 上可能留着上次台账清掉后的旧文件。
    // 台账读不出来时 `prune_orphans` 自己会放弃（宁可留孤儿，不可误删）。
    prune_orphans();
}

fn database() -> Result<&'static Db, GatewayError> {
    DB.get()
        .and_then(Option::as_ref)
        .ok_or_else(|| GatewayError::with_status(503, "助手图片存储不可用"))
}

/// 图片目录（fork：`{cache_dir}/assistant-images`，与 `logs` 同一条 ramdisk 规则）
fn dir() -> PathBuf {
    crate::paths::cache_dir().join(BLOB_DIR)
}

/// 单张图片的落盘路径（id 是 64 位十六进制，直接当文件名安全）
pub fn blob_path(id: &str) -> PathBuf {
    dir().join(id)
}

fn valid_id(id: &str) -> bool {
    id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// fork：图片用量 = 文件字节之和（文件被 ramdisk 清掉的行算 0，可丢弃数据不撑配额）
/// + 旧版留在 `bytes` 列里的存量（非 NULL 的历史行）。
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
    // fork：先校验配额（库锁内，避免并发两个 store 各算各的），再落文件、最后写台账。
    // 文件没落成 / 台账没写成都是「一张图都没进库」，不留悬空台账。
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

/// fork：图片字节落 `{cache_dir}/assistant-images/{id}`（目录按需创建）
fn write_blobs(entries: &[(String, Vec<u8>, String)]) -> Result<(), GatewayError> {
    let dir = dir();
    fs::create_dir_all(&dir).map_err(|_| GatewayError::new("助手图片保存失败（目录不可写）"))?;
    for (id, bytes, _) in entries {
        fs::write(&blob_path(id), bytes)
            .map_err(|_| GatewayError::new("助手图片保存失败（文件写入失败）"))?;
    }
    Ok(())
}

/// fork：从文件读；文件缺失（ramdisk 被清）时回落查库里的遗留行
/// （旧版写的 `bytes` 列非 NULL），都不在才认「图片未保留」。
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

/// fork：从台账读 mime（与 `read` 的回落读合用；库不可用给通用类型）
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

/// fork：从台账读 mime 的公共入口（`assistant_sessions::fork` 复制图片时用；
/// 带会话限定的版本只走私有 `mime_of`，读侧回落不收紧范围）
pub fn mime_of_public(id: &str) -> String {
    mime_of(id, None)
}

/// fork：按台账清掉孤儿文件（文件在而台账无的行——如台账被删后 ramdisk 还留着的旧文件）。
/// 台账读不出来（库不可用）不动文件：宁可留孤儿，不可误删。
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

/// fork：某会话的图片文件（会话删除/清空时级联清理；台账行由库外键连带删）
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

/// fork：清空全部图片文件（「清空全部会话」级联；台账行由库 `DELETE` 连带删）
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
