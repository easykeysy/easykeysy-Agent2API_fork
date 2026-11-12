//! ** up **：`debug_traffic` 。
//!
//! ── （ has up ， not down ）───────────────────────────
//! （config.json `debugMode`） after ， in ** will before **
//! 1 envelope：
//! - request： up URL、** after **request、 request（ already by 
//! model ）；
//! - response： up 、** after **response、response（ as up SSE
//! ， as before ）。
//! down （client request + down response） by `request_raw` 
//! （`api::pipeline::capture_raw_body` / `capture_raw_json`）， and ** 1 
//! **： when 、 when not （ for default no has 
//! can —— for OmniProxy ）。 has 1 and none 
//! （`write_debug_files` → `{config_dir}/debug/last-request.json`）， that is 
//! 「 1 request」 ， not 。
//!
//! ── Why 1 ， not requests ──────────────────
//! 1 SSE response KB，request will that 、 1 
//! 。 after request and none； by id to this 。
//! use ** 1 id **（`record::RequestEntry::id`）。 this in 
//! 「file → 」 1 ， is 「 1 file」「 1 」。
//!
//! ── （** need **， not can ）─────────────────────────────
//! request `authorization` / `cookie` / `x-api-key` etc 1 
//! `[redacted]`（ [`redact_headers`]）。 this not is optional： is ，
//! not etc token in up 。response `set-cookie` 。
//!
//! ── ──────────────────────────────────────────────
//! request and response， not by ；only [`MAX_ENTRIES`] when old 。
//! by to ； as in not will to 。
//!
//! ── `Db` is （ 1 ）────────────
//! and `AccountStore` / `LogStore` / `RequestStats` not ， no has 「」
//! ： use `static DB: OnceLock<Option<Db>>` 1 ， its 。
//! Reason is —— [`TrafficCapture`] response to request drop，
//! that 1 [`record`]，** that not to `ServerState`**（response
//! handler ）。 can is can ， and `logging::store_ref`
//! 1 。
//! before has ， ：
//! - `STORE: OnceLock<Mutex<TrafficStore>>` **delete**：（`VecDeque` +
//! `bytes`） not need need —— is ， is 1 ；
//! - `DIRECTORY: RwLock<Option<PathBuf>>` **delete**： not has directory，
//! not has 「directory」 this （ 1 ， `file()` ）。
//!
//! ── and ────────────────────────────────────────────────
//! before is 「 1 `Mutex` + 」； in by `Db` that Mutex
//! （ in 1 `Db::with` / `with_mut` ）。**allfailure not 
//! request**：failure 1 。
//!
//! ── ： `Db` not can `logging::log` / `logging::verbose` ──
//! that will ** 1 **（`logging::store` → `LogStore::append`），
//! `std::sync::Mutex` not can —— in `Db::with` will 。
//! errorpath** 1 use `eprintln!`**（ before `write_all` / `append_line`
//! use is ， this in need ： is ， will ）。

use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::server::db::Db;
use crate::server::logging;

mod payload;
mod sql;

/// 。 after ** to limit**（ that ）。
///
/// 500： and limit，「 this request」；
/// is ** when ** use ， not is —— need ， use this use ，
/// not is this none。
///
/// ── Why not 「 old 1 」───────────────────────────────
/// old 「 1 」 is as down file 
/// （ 1 1 ，）。 after this by ****：
/// is 1 DELETE， and none（ old is 「」
/// need ）。
/// to limit is can ： use to `count()` to 500 as ，
/// not will 「 500 to 250」 that 1 。
/// is after `record` will 1 DELETE， that delete
/// ** 1 **（`record` 1 ， has 1 ）——
/// and in when （ [`record`] after ）。
pub const MAX_ENTRIES: usize = 500;

/// request： in `[redacted]`（ not ）。
///
/// in OmniProxy up ** use to **：
/// 会在头里带 token / cookie / 签名（见 `providers/*/adapter.rs` 
/// `build_chat_request`）。 can （， is ）。
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "cookie",
    "cookie2",
    "set-cookie",
    "x-api-key",
    "api-key",
    "x-auth-token",
    "x-access-token",
    "x-refresh-token",
    "x-ide-token",
    "x-ide-auth",
    "x-workbuddy-token",
    "x-raccoon-token",
    "x-cline-token",
    "x-qoder-token",
    "x-catpaw-cookie",
    "x-autoclaw-token",
];

/// （ and OmniProxy ， for ）
pub const REDACTED: &str = "[redacted]";

/// （ and `logging::store_ref` 1 ：`None` = unavailable）。
///
/// `OnceLock<Option<Db>>` not is `OnceLock<Db>`：`Db::open` can can failure，
/// `init` need `Option<Db>`（ use is ）—— in `Option` 
/// 「」 and 「unavailable」 use 1 path（ is `None`），
/// need 1 。
static DB: OnceLock<Option<Db>> = OnceLock::new();

/// （ or unavailable when as `None`）
fn db() -> Option<&'static Db> {
    DB.get().and_then(|slot| slot.as_ref())
}

/// （ when use 1 ；`OnceLock` etc —— use has 1 ）。
///
/// ── in need need （ before 1 ）───────────────
/// before ：directory、empty、 old file and 1 （ need 
/// file）。 after after not need need ：
/// - ** no has 「」**： is ， is 1 （ before must file
/// can `get` / `count`， that is file when ）；
/// - ** no has 「」**： by [`record`] ， when 
/// is up when （ that when already ）； has 
/// ， down 1 `record` will 。
/// 「 `Db` up 」 this 1 —— ** need and must use **： no 
/// no enabled，[`record`] will 、[`get`] 1 `None`。
/// nonereturn、 not ： not not （module header to ）。
///
/// none is need （`ServerState::bootstrap` ）： is 
/// **request** （set down 1 request）， no will after 1 
/// requestnone can 。
pub fn init(db: Option<Db>) {
    // use 1 （`set` failure already ）。 and before 1 ：
    // that when is directory + new ， in is ， no has 。
    let _ = DB.set(db);
    // fork：file after ， need （file has ）：
    // 1. file、 that empty —— otherwise old `.db`
    // not down ；
    // 2. （ramdisk empty / after down file）。
    super::blocking::in_place(move_legacy_payloads);
}

/// in to file， after empty that 。
///
/// is 「、 after 」：filefailure in （ can 1 `.db`
/// not ）；success after 1 ， down 「 no has 」 。
fn move_legacy_payloads() {
    let Some(db) = db() else {
        return;
    };
    let rows = db
        .with(sql::payload_rows)
        .and_then(Result::ok)
        .unwrap_or_default();
    let mut moved = Vec::new();
    for entry in rows {
        if payload::write(&entry).is_ok() {
            moved.push(entry.id);
        }
    }
    if !moved.is_empty() {
        if let Some(Err(error)) = db.with(|conn| sql::blank_payloads(conn, &moved)) {
            eprintln!("[Debug] old failure: {error}");
        }
    }
    // in can when （ not file， `payload::sweep`）。
    if let Some(Ok(ids)) = db.with(sql::ids) {
        payload::sweep(&ids.into_iter().collect());
    }
}

/// fork：**** in directory（`{cache_dir}/debug-traffic/`； use ）。
///
/// after `debug_traffic` is ， in directory； use 
/// `api::storage_api` is `Db::file()`， no has use —— is as 
/// 「 in 」 this 。
///
/// old （** is file**；set use ） git 。
///
/// from 「filepath」「 file」—— and `AccountStore::file()`
/// / `LogStore::file()` / `RequestStats::file()` 1 。
///
/// ── unavailable when Why to `None`（ not is path）────────────────
/// use `api::storage_api` in is `Db::file()`（unavailable when use path，
/// by that ），** no has use **。 is as and store 
/// 「file()」 1 （ this ）， and is when 「 in 」
/// this —— by 「 has and by 」 
/// ， not is （ after need use `sqlite3` in ）。
///
/// to `None` not is path ： use （ if has ） can 「 no has 」，
/// to 1 「 0 、 can can not in 」 path。
#[allow(dead_code)]
pub fn file() -> Option<PathBuf> {
    db()?;
    Some(payload::dir())
}

/// （set use ；unavailable when 0）
pub fn count() -> usize {
    match db() {
        Some(db) => db
            .with(sql::count_and_bytes)
            .and_then(Result::ok)
            .map(|(count, _)| count)
            .unwrap_or(0),
        None => 0,
    }
}

/// 1 ； id ，failure， not 。
pub fn record(entry: TrafficEntry) {
    super::blocking::in_place(|| record_sync(entry));
}

fn record_sync(entry: TrafficEntry) {
    let Some(db) = db() else {
        // unavailable：（ and before 「STORE 」 1 ）。
        // this not —— request will to this 1 ，。
        return;
    };
    // fork：file，。filefailure to 「 old 」——
    // can 1 `.db`， not this as when not can 。
    // failure：** not can ** use logging::log / verbose —— need 1 ，
    // that （module header ）。
    let written = match payload::write(&entry) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("[Debug] filefailure（）: {error}");
            false
        }
    };
    let stored = db.with_mut(|conn| {
        if written {
            sql::upsert_ledger(conn, &entry)
        } else {
            sql::upsert_entry(conn, &entry)
        }
    });
    match stored {
        Some(Ok(())) => {}
        Some(Err(error)) => {
            eprintln!("[Debug] failure: {error}");
            return;
        }
        // in （`Db` to ： not ）
        None => return,
    }
    // after old ； 1 。
    match db.with(|conn| sql::enforce_limits(conn)) {
        Some(Ok(removed)) if removed > 0 => {
            if let Some(Ok(ids)) = db.with(sql::ids) {
                payload::sweep(&ids.into_iter().collect());
            }
        }
        Some(Ok(_)) | None => {}
        Some(Err(error)) => eprintln!("[Debug] failure: {error}"),
    }
}

/// by id 1 （request 「」 use ；unavailable when `None`）
///
/// id when ** after that 1 **（`ORDER BY ts DESC, rowid DESC LIMIT 1`），
/// and before 「 from before 1 」 1 。
pub fn get(id: &str) -> Option<TrafficEntry> {
    let mut entry = db()?
        .with(|conn| sql::select_one(conn, id))
        .and_then(Result::ok)
        .flatten()?;
    // fork： is empty ， in file；（ no ）
    // in has ，`hydrate` not to file when return。
    payload::hydrate(&mut entry);
    Some(entry)
}

/// 1 。
///
/// use `Option` 「 this 1 no to 」：request not when no has response，
/// when not 。`id` is —— and request `id` 。
/// this `Option` in is ** can empty**（`status` / `response_headers` /
/// `response_body`），** not use empty NULL**： before this 
/// （`ui/requests-panel.js` `renderDetail`：`data.status == null` `-`，
/// `data.responseBody ? String(...)` as empty then not that 1 ）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrafficEntry {
    /// ：request id（ 1 request in id）
    pub id: String,
    /// when （ms Unix when ）
    pub ts: i64,
    /// up URL（path；）
    #[serde(default)]
    pub url: String,
    /// provider id
    #[serde(default)]
    pub provider: String,
    /// to up request（** already **）
    #[serde(default)]
    pub request_headers: Value,
    /// to up request（ already by model ）
    #[serde(default)]
    pub request_body: Value,
    /// up response（ / to response when ）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    /// up response（** already **）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_headers: Option<Value>,
    /// up response： as SSE ， as before 
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
    /// ； new not 。
    #[serde(default)]
    pub truncated: bool,
}

/// ： [`REDACTED`]， its 。
///
/// `Vec<(String, String)>`（ to ） and `HeaderMap`（reqwest response）
/// ， 1 JSON for —— by ** after to **（ and HTTP 1 ，
/// and this ）。
pub fn redact_headers<'a>(headers: impl IntoIterator<Item = (&'a str, &'a str)>) -> Value {
    let mut object = serde_json::Map::new();
    for (name, value) in headers {
        let lower = name.to_ascii_lowercase();
        let shown = if SENSITIVE_HEADERS.contains(&lower.as_str()) {
            REDACTED
        } else {
            value
        };
        object.insert(lower, Value::String(shown.to_string()));
    }
    Value::Object(object)
}

/// request「」 response： + request 。
///
/// by `api::debug_api` （ to that ）。
pub fn detail_payload(entry: &TrafficEntry) -> Value {
    json!({
        "id": entry.id,
        "ts": entry.ts,
        "url": entry.url,
        "provider": entry.provider,
        "requestHeaders": entry.request_headers,
        "requestBody": entry.request_body,
        "status": entry.status,
        "responseHeaders": entry.response_headers,
        "responseBody": entry.response_body,
        "truncated": entry.truncated,
    })
}

// ─── （ use ）────────────────────────────────────────

/// 1 。
///
/// ── ────────────────────────────────────────────────
/// request in ** will before **create（ when URL / / already ），response in to 
/// response when up and ，response， after in ** or drop when **
/// （[`Drop`]）。 use Drop not is 「 when use 」 by and 
/// `RecordingStream` ：client、 up 、request path not will 
/// to 「」， use will that need need failure。
///
/// ── Why `Arc` ────────────────────────────────────────
/// path need `ForwardStream` （ can can in handler return after 
/// axum ），path。path has 1 
/// `Arc<TrafficCapture>`， after drop 。
pub struct TrafficCapture {
    state: std::sync::Mutex<CaptureState>,
}

struct CaptureState {
    entry: TrafficEntry,
    /// response（ when to entry.response_body）
    body: Vec<u8>,
    /// is up request（`reset_request` ）。
    ///
    /// = request in to before failure（ no has available、 not in …），
    /// that ** not **： id and when no has ， will 
    /// 「」 1 empty。
    sent: bool,
    /// already （ 1 ）
    done: bool,
}

impl TrafficCapture {
    /// request： when create， id（URL / / etc when by 
    /// [`Self::reset_request`] up —— that when ）。
    pub fn begin(id: &str) -> Self {
        Self {
            state: std::sync::Mutex::new(CaptureState {
                entry: blank_entry(id, "", "", &[], &Value::Null),
                body: Vec::new(),
                sent: false,
                done: false,
            }),
        }
    }

    /// as 1 new （ 1 request / 401 new ）。
    ///
    /// ** after 1 as **（ and OmniProxy 「 as after 1 」）：
    /// when up 1 response， use to is that ，
    /// not is 「 1 failure + success」 in 1 body。
    pub fn reset_request(
        &self,
        url: &str,
        provider: &str,
        headers: &[(String, String)],
        body: &Value,
    ) {
        let mut guard = self.lock();
        guard.entry = blank_entry(&guard.entry.id.clone(), url, provider, headers, body);
        guard.body.clear();
        // to this request already 、 will —— this has can 
        guard.sent = true;
        // done not ：request 1 ， not new 
    }

    /// response to when up and response（**must in consume response before **）。
    ///
    /// empty already response： down up 1 body not this 
    /// （ [`Self::reset_request`] 「 after 1 as 」）。
    pub fn attach_response(&self, status: u16, headers: &reqwest::header::HeaderMap) {
        let mut guard = self.lock();
        guard.entry.status = Some(status);
        guard.body.clear();
        let pairs = headers
            .iter()
            .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("[binary]")));
        guard.entry.response_headers = Some(redact_headers(pairs));
    }

    pub fn push(&self, chunk: &[u8]) {
        self.lock().body.extend_from_slice(chunk);
    }

    #[cfg(test)]
    pub(crate) fn captured_body(&self) -> String {
        String::from_utf8_lossy(&self.lock().body).into_owned()
    }

    /// （ etc ： use not ）。
    ///
    /// no request（`sent` as ） when ： that id no has 
    /// （ `CaptureState::sent` ）。
    pub fn finish(&self) {
        let mut guard = self.lock();
        if guard.done {
            return;
        }
        guard.done = true;
        if !guard.sent {
            return;
        }
        let body = std::mem::take(&mut guard.body);
        if !body.is_empty() {
            // can can ， after 。
            guard.entry.response_body = Some(match String::from_utf8(body) {
                Ok(text) => text,
                Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned(),
            });
        }
        let entry = std::mem::replace(&mut guard.entry, blank_entry("", "", "", &[], &Value::Null));
        drop(guard);
        record(entry);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, CaptureState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for TrafficCapture {
    fn drop(&mut self) {
        self.finish();
    }
}

/// 1 empty （`begin` / `reset_request` use ）
fn blank_entry(
    id: &str,
    url: &str,
    provider: &str,
    headers: &[(String, String)],
    body: &Value,
) -> TrafficEntry {
    let pairs = headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()));
    TrafficEntry {
        id: id.to_string(),
        ts: logging::now_ms(),
        url: url.to_string(),
        provider: provider.to_string(),
        request_headers: redact_headers(pairs),
        request_body: body.clone(),
        status: None,
        response_headers: None,
        response_body: None,
        truncated: false,
    }
}

/// is （** 1 **）： up （） and 
/// down （`api::pipeline::capture_raw_body` / `capture_raw_json`）；
/// request 「 for 」 and 「」。
///
/// use config（`config::current()` is RwLock + ）：
/// down 1 request， not 。path up 1 is can 
/// —— and 「set」 1 to （ `config::RetrySettings`）。
pub fn enabled() -> bool {
    crate::server::config::current().debug_mode()
}

/// use ： 1 old file `debug_traffic` （** 1 **， after ）。
///
/// and `logs` / `requests` ：， after by 。
///
/// use `unchecked_transaction` not is `transaction`： to is 
/// `&Connection`（`transaction()` need `&mut`）， use `Db::with` has 1 、
/// no has —— that 「 not 」 by use 。
///
/// ── `INSERT OR IGNORE` not is INSERT（T11 after must）──────
/// `id` is ， in this （ use in 「」 before can can already 
/// request）。「」 when old file id already in ， INSERT will 
/// —— is use to 「failure」Reason is 「」。
/// use `OR IGNORE`： already in 、 new ， as none empty。
/// return `( new , already in )` use 。
///
/// Why this ** not ** use `upsert_entry`（ id ， that use ）：
/// will that ， can can not （ can can ——
/// path will 、 in will 1 ）。 old file is ****，
/// not this update that 。
pub(crate) fn import_legacy(
    conn: &rusqlite::Connection,
    entries: &[TrafficEntry],
) -> rusqlite::Result<(usize, usize)> {
    let tx = conn.unchecked_transaction()?;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    for entry in entries {
        if sql::insert_entry_ignore(&tx, entry)? {
            imported += 1;
        } else {
            skipped += 1;
        }
    }
    sql::enforce_limits(&tx)?;
    tx.commit()?;
    Ok((imported, skipped))
}

// this has 1 `legacy_count`（「empty」 etc ）。** already delete**：
// T11 use 「」 after ， that not —— use can can in that before 
// request（ already has ）， is 「 already 」，
// old file not （`logs` already up ）。
// in `INSERT OR IGNORE` by etc ， not need need 「empty not empty」 this ——
// `db::migrate::debug::import_debug` etc 。

/// use ： old file （** and before `init` 1 **）。
///
/// 、empty、（ not 、 not panic）—— not 
/// not 。 in not is ， by and T4 `parse_*` ：
/// 「 1 JSON 」 is （`TrafficEntry` serde is ），
/// and must use 1 。
pub(crate) fn parse_legacy_jsonl(text: &str) -> Vec<TrafficEntry> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<TrafficEntry>(line) {
            out.push(entry);
        }
    }
    out
}
