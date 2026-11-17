//! Disk-offload side for downstream raw bodies (fork): SQLite's `request_raw` table keeps only the **ledger**
//! (id / ts / size / truncated); bodies are written to `{cache_dir}/request-raw/{hex of the id}`.
//!
//! ── Why the ledger stays in the DB and bodies go to files ──────────────────────────────
//! The body is the largest per-row field in this database (measured average 168 KB) and is only produced while debug mode is on.
//! Same trade-off as `assistant_images`: the ledger lives in the DB, while per-row deletion / count pruning / counting
//! still run the original SQL, unchanged; once bodies are rendered to files they sit on the ramdisk, so a restart wipe only
//! affects whether "conversation preview" can read them — request logs and reports stay intact.
//!
//! ── File names ──────────────────────────────────────────────────
//! The request id is an arbitrary externally supplied string (unusable as a file name directly: may contain `/`, `..`, colons).
//! Byte-wise hex-encoding here yields names made only of `[0-9a-f]`; `sweep` decodes them back into ids
//! to match the ledger. Temp files end in `.tmp` and are always treated as garbage by `sweep`.
//!
//! ── Atomicity ──────────────────────────────────────────────────
//! Write `{name}.tmp` first, then `rename` over the target: readers never see a half-written JSON.
//! The ledger write is the caller's job, done only after the file write succeeds; a failed file write means no ledger row, so no dangling rows.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The row shape from the `request-raw.jsonl` era, reused here as the JSON shape of a **single** body file
/// (field names aligned with the DB `request_raw` table, so readers / exporters never juggle two namings).
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

/// File name → request id (files not written by this module return None; `sweep` clears them on that basis)
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

/// Write one body (overwrites the old file for the same id). On failure returns the error only; the caller then skips the ledger write.
pub(super) fn write(row: &RawRow) -> std::io::Result<()> {
    let dir = dir();
    fs::create_dir_all(&dir)?;
    let target = path(&row.id);
    let tmp = dir.join(format!("{}.tmp", file_name(&row.id)));
    let bytes = serde_json::to_vec(row).map_err(std::io::Error::other)?;
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, &target)
}

/// Read a body by id; a missing file (ramdisk wiped) or corrupt content both count as "not retained".
pub(super) fn read(id: &str) -> Option<RawRow> {
    let bytes = fs::read(path(id)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Sweep orphans against the ledger: body files not in `keep`, plus every leftover `*.tmp`.
///
/// The caller must invoke this only **after successfully reading the ledger id set**; if the set cannot be read, do not call it
/// (an empty set would delete every body by mistake).
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

/// Delete a single row (rolls back the just-written file when the ledger write is rejected).
pub(super) fn remove(id: &str) {
    let _ = fs::remove_file(path(id));
}
