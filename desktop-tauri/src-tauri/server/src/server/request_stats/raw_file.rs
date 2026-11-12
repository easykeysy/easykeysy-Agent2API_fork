//! 下游原始正文的落盘侧（fork）：SQLite 的 `request_raw` 表只留**台账**
//! （id / ts / size / truncated），正文写 `{cache_dir}/request-raw/{id 的十六进制}`。
//!
//! ── 为什么台账留库、正文走文件 ──────────────────────────────
//! 正文是这张库里单行最大的字段（实测平均 168 KB），且只在调试模式开启时产生。
//! 它与 `assistant_images` 同一取舍：台账在库，明细的删除 / 条数裁剪 / 计数
//! 仍走原来的 SQL，一条都不用改；正文渲染成文件后在 ramdisk 上，重启清空只
//! 影响「预览对话」读得到与否，请求日志与报表完好。
//!
//! ── 文件名 ──────────────────────────────────────────────────
//! 请求 id 是外部传入的任意字符串（不能直接当文件名：可能含 `/`、`..`、冒号）。
//! 这里按字节做十六进制编码，得到只含 `[0-9a-f]` 的名字；`sweep` 反向解码回 id
//! 去对台账。临时文件以 `.tmp` 结尾，`sweep` 一律当垃圾清掉。
//!
//! ── 原子性 ──────────────────────────────────────────────────
//! 先写 `{name}.tmp` 再 `rename` 覆盖目标：读侧永远看不到写了一半的 JSON。
//! 台账写入由调用方在「文件写成功之后」做；文件写失败就不写台账，不留悬空行。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `request-raw.jsonl` 时代的行形状，这里沿用为**单条**正文文件的 JSON 形状
/// （字段名与 DB `request_raw` 表对齐，读侧 / 导出不用两套命名）。
#[derive(Deserialize, Serialize)]
pub(super) struct RawRow {
    pub id: String,
    pub ts: i64,
    #[serde(rename = "requestBody", default)]
    pub request_body: String,
    #[serde(rename = "responseBody", default)]
    pub response_body: String,
    #[serde(default)]
    pub truncated: bool,
}

fn dir() -> PathBuf {
    crate::paths::cache_dir().join("request-raw")
}

fn file_name(id: &str) -> String {
    id.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn path(id: &str) -> PathBuf {
    dir().join(file_name(id))
}

/// 文件名 → 请求 id（不是本模块写的文件返回 None，`sweep` 据此清掉）
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

/// 写一条正文（覆盖同 id 的旧文件）。失败只返回错误，调用方据此放弃写台账。
pub(super) fn write(row: &RawRow) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let target = path(&row.id);
    let tmp = dir.join(format!("{}.tmp", file_name(&row.id)));
    let bytes = serde_json::to_vec(row).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &target)
}

/// 按 id 读正文；文件不在（ramdisk 被清）或内容损坏都按「未保留」处理。
pub(super) fn read(id: &str) -> Option<RawRow> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// 按台账清孤儿：不在 `keep` 里的正文文件、以及所有 `*.tmp` 残留。
///
/// 调用方必须在**成功读到台账 id 集合之后**才调；台账读不出来时不要调
/// （空集合会把全部正文误删）。
pub(super) fn sweep(keep: &std::collections::HashSet<String>) {
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

/// 删单条（台账写入被拒时回滚刚写的文件）。
pub(super) fn remove(id: &str) {
    let _ = fs::remove_file(path(id));
}
