//! Disk-offload side for debug-traffic **payloads** (fork): SQLite's `debug_traffic` table keeps only a ledger
//! (id / ts / url / provider / status / size); request headers, request body, response headers and response body
//! are written to `{cache_dir}/debug-traffic/{hex of the id}`.
//!
//! ── Why the ledger stays in the DB and payloads go to files ──────────────────────────────
//! This table is the main reason the `.db` grows: it retains 500 entries, each one a full upstream request plus
//! a full SSE response — several hundred MB is normal. Three SQL paths depend on it at the same time:
//! the `request_stats` "raw message count" UNION, per-request filtered cleanup, and count-based pruning.
//! Keep the **rows** in the DB and move only the payloads out, and none of that SQL needs to change (same
//! trade-off as `request_raw` / `assistant_images`).
//!
//! ── File names ──────────────────────────────────────────────────
//! Request ids come from the outside (may contain `/`, `..`, colons) and cannot serve as file names directly; byte-wise
//! hex-encoding yields names made only of `[0-9a-f]`, and `sweep` decodes them back into ids to match the ledger.
//!
//! ── Atomicity ──────────────────────────────────────────────────
//! Write `{name}.tmp` first, then `rename` over the target: readers never see a half-written JSON. If the file write
//! fails, the caller falls back to "payload goes into the DB as before" — a slightly bigger `.db` beats a lost message.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::TrafficEntry;

/// JSON shape of a payload file. Status code / truncation marker stay in the ledger (small fields the list and
/// stats views need); only the four texts that actually take up space live here.
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

/// File name → message id (files not written by this module yield `None`; `sweep` clears them on that basis)
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

/// Extract from one message the half that gets offloaded (request headers / request body / response headers / response body).
fn payload_of(entry: &TrafficEntry) -> Payload {
    Payload {
        request_headers: entry.request_headers.clone(),
        request_body: entry.request_body.clone(),
        response_headers: entry.response_headers.clone(),
        response_body: entry.response_body.clone(),
    }
}

/// Write one payload (overwrites the file for the same id). On failure returns the error only; the caller decides whether to fall back.
pub(super) fn write(entry: &TrafficEntry) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let name = file_name(&entry.id);
    let tmp = dir.join(format!("{name}.tmp"));
    let bytes = serde_json::to_vec(&payload_of(entry)).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, dir.join(name))
}

/// Read a payload by id; a missing file (ramdisk wiped) or corrupt content both count as "not retained".
pub(super) fn read(id: &str) -> Option<Payload> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Merge the payload back into the ledger row (the four ledger columns are empty at that point).
pub(super) fn hydrate(entry: &mut TrafficEntry) {
    let Some(payload) = read(&entry.id) else {
        return;
    };
    entry.request_headers = payload.request_headers;
    entry.request_body = payload.request_body;
    entry.response_headers = payload.response_headers;
    entry.response_body = payload.response_body;
}

/// Sweep orphans against the ledger: payload files not in `keep`, plus every leftover `*.tmp`.
///
/// The caller must invoke this only **after successfully reading the ledger id set**; if the set cannot be read, do not call it
/// (an empty set would delete every payload by mistake).
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
