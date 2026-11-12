//! **** （fork）：SQLite `debug_traffic` 
//! （id / ts / url / provider / status / size），request、request、response、response
//! `{cache_dir}/debug-traffic/{id }`。
//!
//! ── Why、file ──────────────────────────────
//! this is `.db` will ： 500 、 is 1 up request +
//! 1 SSE response， MB 。 when SQL ：
//! `request_stats` 「」UNION、 by request、 and by 。
//! **** in 、 ， this SQL 1 not use （ and 
//! `request_raw` / `assistant_images` 1 ）。
//!
//! ── file ──────────────────────────────────────────────────
//! request id （ can can `/`、`..`、）， not can file； by 
//! to `[0-9a-f]` ，`sweep` to id for 。
//!
//! ── ──────────────────────────────────────────────────
//! `{name}.tmp` `rename` ： not to 1 JSON。filefailure
//! when use to 「 old 」， can 1 `.db` not this 。

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TrafficEntry;

/// file JSON 。 / in （ is and statistics need use 
/// ）， this that 。
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

/// file → id（ not is file to `None`，`sweep` ）
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

/// from 1 need that （request / request / response / response）。
fn payload_of(entry: &TrafficEntry) -> Payload {
    Payload {
        request_headers: entry.request_headers.clone(),
        request_body: entry.request_body.clone(),
        response_headers: entry.response_headers.clone(),
        response_body: entry.response_body.clone(),
    }
}

/// 1 （ id ）。failurereturnerror， by use need not need 。
pub(super) fn write(entry: &TrafficEntry) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let name = file_name(&entry.id);
    let tmp = dir.join(format!("{name}.tmp"));
    let bytes = serde_json::to_vec(&payload_of(entry)).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dir.join(name))
}

/// by id ；file not in （ramdisk ） or by 「」。
pub(super) fn read(id: &str) -> Option<Payload> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// and （ that when is empty ）。
pub(super) fn hydrate(entry: &mut TrafficEntry) {
    let Some(payload) = read(&entry.id) else {
        return;
    };
    entry.request_headers = payload.request_headers;
    entry.request_body = payload.request_body;
    entry.response_headers = payload.response_headers;
    entry.response_body = payload.response_body;
}

/// by ： not in `keep` file、 and all `*.tmp` 。
///
/// use must**success to id after **； not when not need 
/// （empty will ）。
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
