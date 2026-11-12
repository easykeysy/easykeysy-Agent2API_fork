//! GET/POST/PUT /api/config —— config（API Key、）。
//!
//! for Node server.mjs 913-954 ：
//!
//! GET return：
//! `{apiKey: or null, apiKeySet, locale, defaultModel,
//!     sanitizeBlacklistFingerprints}`
//! use Node 920 ：` before 6... after 4`。
//!
//! POST / PUT `{apiKey?, locale?}`（ as 1 ，PUT is ，
//! `http::router` ）；
//! - apiKey 、 or < 8 → 400 `{"success":false,"error":"API Key need need 8 "}`
//! - apiKey as null → delete key
//! - apiKey unavailable（file / configdirectory not etc ）→ 500， and 
//! ** 1 not **（ up is old Key，available； `config::set_api_key`）
//! - locale empty → update
//! - after return `{success:true,data:{apiKeySet,locale,defaultModel}}`
//!
//! `providerRoute`（provider by ） already 「 1 」 down ：
//! use 1 by ，config not has this 1 （file 、ignore）。
//!
//! 「 in 」：Node is opts for ； this is 
//! `server::config` RwLock ， in request —— not use 。
//!
//! `desensitize` for （`{enabled, termCount, roles}` need ）
//! 1 delete； in this is ****（`sanitizeBlacklistFingerprints`），
//! and `GET /api/sanitize` 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// API Key （Node 8）
const MIN_API_KEY_LENGTH: usize = 8;
/// not when ， Node 
const API_KEY_TOO_SHORT: &str = "API Key 至少需要 8 个字符";

/// GET /api/config
pub async fn get_config(State(_state): State<ServerState>) -> Response {
    let snapshot = config::current();
    ok_json(json!({
        "apiKey": snapshot.masked_api_key(),
        "apiKeySet": snapshot.api_key_set(),
        "locale": snapshot.locale(),
        "defaultModel": snapshot.default_model(),
        // （ and GET /api/sanitize 1 、 1 ）
        config::KEY_SANITIZE_FINGERPRINTS: snapshot.sanitize_fingerprints(),
    }))
}

/// POST /api/config（`PUT /api/config` 1 ，module header）
///
/// and Node 1 ： apiKey（ 400，locale not ），
/// locale。
pub async fn post_config(State(_state): State<ServerState>, body: Bytes) -> Response {
    // empty body as {}（Node `|| '{}'`）； JSON to 400 not is 500
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // ── apiKey ──────────────────────────────────────────────
    if let Some(value) = object.get("apiKey") {
        match value {
            // null → delete key（ for Node delete config.apiKey + opts.apiKey = null）
            // delete not ， not will `Err`（ `config::set_api_key` ）；
            // failure use has （ already 、 already ，requestsuccess）
            Value::Null => {
                let _ = config::set_api_key(None);
                logging::log("[Config]", "API Key 已清除；模型调用仍需携带启用的 Key");
            }
            Value::String(text) => {
                if text.chars().count() < MIN_API_KEY_LENGTH {
                    return errors::management_error(400, API_KEY_TOO_SHORT);
                }
                // 、 `enc1:` （ `config::set_api_key`）。failure when 1 
                // no 、 up is old —— must， not can success（ will as 
                // Key already update）。 and not 400 not ：request no ， is 
                // unavailable， is 500（ and panel 「failure」 1 ）；
                // Reason 1 and up ， use can 
                if let Err(reason) = config::set_api_key(Some(text.clone())) {
                    logging::log("[Config]", &format!("❌ API Key 保存失败: {reason}"));
                    return errors::management_error(500, format!("API Key 保存失败: {reason}"));
                }
                // not key （ will and ）， already update
                logging::log("[Config]", "✅ API Key 已更新，客户端需带 Authorization: Bearer <key>");
            }
            // null and → 1 400 （ Node ）
            _ => return errors::management_error(400, API_KEY_TOO_SHORT),
        }
    }

    // ── locale ──────────────────────────────────────────────
    if let Some(Value::String(text)) = object.get("locale") {
        if !text.is_empty() {
            config::set_locale(text);
            logging::log("[Config]", &format!("计费语言已更新为 {text}"));
        }
    }

    let snapshot = config::current();
    ok_json(json!({
        "apiKeySet": snapshot.api_key_set(),
        "locale": snapshot.locale(),
        "defaultModel": snapshot.default_model(),
    }))
}

