//! down （fork）：SQLite `request_raw` ****
//! （id / ts / size / truncated）， `{cache_dir}/request-raw/{id }`。
//!
//! ── Why、file ──────────────────────────────
//! is this （ 168 KB）， and in when 。
//! and `assistant_images` 1 ： in ， delete / / 
//! SQL， 1 not use ；file after in ramdisk up ，empty
//! 「 for 」 to and ，request and 。
//!
//! ── file ──────────────────────────────────────────────────
//! request id is any（ not can file： can can `/`、`..`、）。
//! this by ， to `[0-9a-f]` ；`sweep` to id
//! for 。 when file `.tmp` ，`sweep` 1 。
//!
//! ── ──────────────────────────────────────────────────
//! `{name}.tmp` `rename` ： not to 1 JSON。
//! by use in 「filesuccess after 」；filefailure not ， not empty。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `request-raw.jsonl` when ， this use as ****file JSON 
/// （ and DB `request_raw` for ， / not use ）。
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

/// file → request id（ not is filereturn None，`sweep` ）
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

/// 1 （ id old file）。failurereturnerror， use 。
pub(super) fn write(row: &RawRow) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let target = path(&row.id);
    let tmp = dir.join(format!("{}.tmp", file_name(&row.id)));
    let bytes = serde_json::to_vec(row).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &target)
}

/// by id ；file not in （ramdisk ） or by 「」。
pub(super) fn read(id: &str) -> Option<RawRow> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// by ： not in `keep` file、 and all `*.tmp` 。
///
/// use must in **success to id after **； not when not need 
/// （empty will ）。
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

/// （ when file）。
pub(super) fn remove(id: &str) {
    let _ = fs::remove_file(path(id));
}
