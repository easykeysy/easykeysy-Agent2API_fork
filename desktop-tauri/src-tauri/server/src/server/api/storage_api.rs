//! （set「」）。
//!
//! ```text
//! GET /api/storage 1 、 and 
//! ```
//!
//! ── is ****（ before is 「 1 directory」）────────
//! before this has by （`POST /api/storage/relocate` +
//! `GET /api/storage/progress` ）， can directory。
//! 1 `{config_dir}/agent2api.db` after that in up 
//! already not ：「 to 1 file」 will （ in ），
//! down 1 request when （ store `relocate`）。
//! by store `relocate` 1 delete，****。
//!
//! ── （ up need use that ）───────────────
//! by configdirectory，configdirectory by `AGENT2API_PROXY_HOME`
//! （ old `WORKBUDDY_PROXY_HOME` can ， `gateway::config_dir`）。
//! set、 not to 「」 by ， and this to use ——
//! 1 by no has by （ this is need path）。
//!
//! ── Why is `database: {...}` not is ────────────────
//! before return `{logs: {...}, requests: {...}, debug: {...}}`， before 
//! `storage[target]` 。 in ** no has directory and file**， can to 
//! has 「 1 + 」—— for will as 
//! （ before is this `bytes + dailyBytes` ，
//! that to 1 file， ）。
//! 1 `database` for after ，「 has 1 file、 has if 」 this 
//! in response up can 。
//!
//! `configDir` ： before use 「 in configdirectory down 」 this ，
//! is when 1 need 。

use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::http::ok_json;
use crate::server::ServerState;

/// GET /api/storage —— 1 、file + WAL use and 。
/// RequestStats::stats；`debug` as up 。
/// unavailable when `available` as false， not 0 。
pub async fn get_storage(State(state): State<ServerState>) -> Response {
    let config_dir = config::config_dir();
    // path is `Db::file()`（failure when use path： when 
    // 「 this 」 this and has no has none， and `AccountStore::file()`
    // 1 ）。
    let db_file = match state.db() {
        Some(db) => db.file().to_path_buf(),
        None => config_dir.join(crate::server::db::FILE_NAME),
    };
    let available = state.db().is_some();

    // ：（`list_accounts` will and ）。unavailable when return
    // empty， is 「0 」 this 。
    let accounts = state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    // ：`LogStore::stats()` `total`（unavailable when to 0）
    let logs = crate::server::logging::store_ref()
        .map(|store| store.stats().total)
        .unwrap_or(0);

    // request and ：`RequestStats::stats()` 1 to （ will ）
    let request_stats = state.request_stats();
    let stats = request_stats.stats();
    let requests = stats.get("total").and_then(Value::as_u64).unwrap_or(0);
    let daily_days = stats.get("dailyDays").and_then(Value::as_u64).unwrap_or(0);
    let downstream_raw = stats.get("downstreamRaw").and_then(Value::as_u64).unwrap_or(0);
    let upstream_raw = stats.get("upstreamRaw").and_then(Value::as_u64).unwrap_or(0);

    ok_json(json!({
        "configDir": config_dir.to_string_lossy(),
        "database": {
            "file": db_file.to_string_lossy(),
            "bytes": if available { request_stats.db_bytes() } else { 0 },
            // in available when can （unavailable when store 0， and 「 no has 
            // 」 in up none）—— this can 「unavailable」 not is 
            // 1 0
            "available": available,
            "accounts": accounts,
            "logs": logs,
            "requests": requests,
            "dailyDays": daily_days,
            "debug": upstream_raw,
            "downstreamRaw": downstream_raw,
            "upstreamRaw": upstream_raw,
        },
    }))
}
