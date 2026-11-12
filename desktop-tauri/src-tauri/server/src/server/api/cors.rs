//! ：`GET/PUT /api/cors`。
//!
//! ── this ──────────────────────────────────────────
//! 打开后网关面（`/v1/*`） by by CORS：request 204，
//! `Access-Control-Allow-*` response down （ `*`）。 when （default）
//! not CORS —— not to ：request will to API Key
//! in up 401 （ by not `Authorization` ）， can to 
//! 1 「none API」。
//!
//! ── Why need this ， not is ─────────────────────────
//! is ** up 、** that 1 ： `*` no API Key when ，
//! can up 。default + ， use 
//! 。 by （`/api/*`） not ， has none CORS
//! （ and `/api/*` ， not need need ， 1 ）。
//!
//! ── ────────────────────────────────────────────────
//! `config` `corsEnabled` （ 1 `kv` config）。CORS in request
//! ， down 1 request， not 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/cors —— current
pub async fn get_cors(State(_state): State<ServerState>) -> Response {
    ok_json(cors_json())
}

/// PUT /api/cors —— body `{corsEnabled: true|false}`
///
/// config， down 1 request（CORS in request， not ）。
/// in 1 ： is 「 can 」 ，
/// use this in to 。
pub async fn put_cors(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "requestmust is JSON for ");
    };
    let key = config::KEY_CORS_ENABLED;
    let Some(value) = object.get(key) else {
        return errors::management_error(400, format!(" {key}"));
    };
    let Some(enabled) = value.as_bool() else {
        return errors::management_error(400, format!("{key} must is true or false"));
    };
    if !config::set_cors_enabled(enabled) {
        logging::log("[Config]", "⚠️ setfailure，");
    }
    logging::log(
        "[Config]",
        if enabled {
            " already ：/v1/* use （Access-Control-Allow-Origin: *）"
        } else {
            " already ：/v1/* not request，client will down "
        },
    );
    ok_json(cors_json())
}

/// （GET and PUT use ， before use response new ）
fn cors_json() -> Value {
    json!({
        config::KEY_CORS_ENABLED: config::current().cors_enabled(),
    })
}
