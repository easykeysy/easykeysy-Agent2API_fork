//! 调试报文**载荷**的落盘侧（fork）：SQLite 的 `debug_traffic` 只留台账
//! （id / ts / url / provider / status / size），请求头、请求体、响应头、响应体
//! 写 `{cache_dir}/debug-traffic/{id 的十六进制}`。
//!
//! ── 为什么台账留库、载荷走文件 ──────────────────────────────
//! 这张表是 `.db` 会长大的主因：留存 500 条、每条是一整个上游请求 +
//! 一整段 SSE 响应，几百 MB 很正常。而它同时被三处 SQL 依赖：
//! `request_stats` 的「原始报文条数」UNION、按请求筛选清理、以及按条数裁剪。
//! 把**行**留在库里、只把载荷搬走，这些 SQL 一条都不用改（与
//! `request_raw` / `assistant_images` 同一取舍）。
//!
//! ── 文件名 ──────────────────────────────────────────────────
//! 请求 id 来自外部（可能含 `/`、`..`、冒号），不能直接当文件名；按字节做
//! 十六进制编码得到只含 `[0-9a-f]` 的名字，`sweep` 再反向解码回 id 对台账。
//!
//! ── 原子性 ──────────────────────────────────────────────────
//! 先写 `{name}.tmp` 再 `rename` 覆盖：读侧看不到写了一半的 JSON。写文件失败
//! 时调用方回落到「载荷照旧进库」，宁可多占一点 `.db` 也不丢这条报文。

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TrafficEntry;

/// 载荷文件的 JSON 形状。状态码 / 截断标记留在台账里（它们是列表和统计要用的
/// 小字段），这里只放真正占体积的那四份文本。
#[derive(Deserialize, Serialize)]
pub(super) struct Payload {
    #[serde(rename = "requestHeaders", default)]
    pub request_headers: Value,
    #[serde(rename = "requestBody", default)]
    pub request_body: Value,
    #[serde(rename = "responseHeaders", default, skip_serializing_if = "Option::is_none")]
    pub response_headers: Option<Value>,
    #[serde(rename = "responseBody", default, skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
}

pub(super) fn dir() -> PathBuf {
    crate::paths::cache_dir().join("debug-traffic")
}

fn file_name(id: &str) -> String {
    id.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn path(id: &str) -> PathBuf {
    dir().join(file_name(id))
}

/// 文件名 → 报文 id（不是本模块写的文件给 `None`，`sweep` 据此清掉）
fn id_from_name(name: &str) -> Option<String> {
    if name.is_empty() || name.len() % 2 != 0 {
        return None;
    }
    let bytes = name.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        decoded.push((high * 16 + low) as u8);
    }
    String::from_utf8(decoded).ok()
}

/// 从一条报文里取出要落盘的那半（请求头 / 请求体 / 响应头 / 响应体）。
fn payload_of(entry: &TrafficEntry) -> Payload {
    Payload {
        request_headers: entry.request_headers.clone(),
        request_body: entry.request_body.clone(),
        response_headers: entry.response_headers.clone(),
        response_body: entry.response_body.clone(),
    }
}

/// 写一条载荷（同 id 覆盖）。失败只返回错误，由调用方决定要不要回落。
pub(super) fn write(entry: &TrafficEntry) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let name = file_name(&entry.id);
    let tmp = dir.join(format!("{name}.tmp"));
    let bytes = serde_json::to_vec(&payload_of(entry)).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dir.join(name))
}

/// 按 id 读载荷；文件不在（ramdisk 被清）或内容损坏都按「未保留」处理。
pub(super) fn read(id: &str) -> Option<Payload> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// 把载荷并回台账行（台账里的那四列此时是空的）。
pub(super) fn hydrate(entry: &mut TrafficEntry) {
    let Some(payload) = read(&entry.id) else {
        return;
    };
    entry.request_headers = payload.request_headers;
    entry.request_body = payload.request_body;
    entry.response_headers = payload.response_headers;
    entry.response_body = payload.response_body;
}

/// 按台账清孤儿：不在 `keep` 里的载荷文件、以及所有 `*.tmp` 残留。
///
/// 调用方必须**成功读到台账 id 集合之后**才调；读不出来时不要调
/// （空集合会把全部载荷误删）。
pub(super) fn sweep(keep: &HashSet<String>) {
    let Ok(entries) = fs::read_dir(dir()) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".tmp") {
            let _ = fs::remove_file(entry.path());
            continue;
        }
        match id_from_name(&name) {
            Some(id) if keep.contains(&id) => {}
            _ => {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}
