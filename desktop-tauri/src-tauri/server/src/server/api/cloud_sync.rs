//! （set「 → 」）。
//!
//! ```text
//! GET /api/cloud-sync set + （ / Secret hasXxx）
//! PUT /api/cloud-sync set（empty = use ）
//! POST /api/cloud-sync/test （directory）
//! GET /api/cloud-sync/remote need （ use ）
//! POST /api/cloud-sync/upload up 
//! POST /api/cloud-sync/download down and use 
//! POST /api/cloud-sync/import-key { password } and 
//! ```
//!
//! ── Why protected ────────────────────────────────────
//! up will and Key to ； down will ；
//! will 。 is `/api/config` 。`GET` 
//! `hasXxx`， when path and ， and `/api/storage` 。
//!
//! ── Why down / up need ──────────────────────────
//! to use request when is 60 seconds（`gateway::REQUEST_TIMEOUT_MS`， to 
//! API request）。 need MB，60 seconds not —— `bridge.rs` to this 
//! to 300 seconds（ `commands::cloud_sync_upload`）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::cloud_sync::{self, manifest, settings};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// GET /api/cloud-sync
pub async fn get_settings(State(_state): State<ServerState>) -> Response {
    ok_json(cloud_sync::settings_json())
}

/// PUT /api/cloud-sync
///
/// request is ****（ and cc-switch `webdavSyncSaveSettings` ）。
/// empty = use that ；empty = new 。
pub async fn put_settings(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let input = match parse_save_input(&payload) {
        Ok(input) => input,
        Err(message) => return errors::management_error(400, message),
    };
    match cloud_sync::save_settings(input) {
        Ok(data) => ok_json(data),
        Err(message) => errors::management_error(400, message),
    }
}

/// POST /api/cloud-sync/test
pub async fn post_test(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let input = match parse_save_input(&payload) {
        Ok(input) => input,
        Err(message) => return errors::management_error(400, message),
    };
    match cloud_sync::test_connection(input).await {
        Ok(data) => ok_json(data),
        Err(message) => errors::management_error(502, message),
    }
}

/// GET /api/cloud-sync/remote
pub async fn get_remote(State(_state): State<ServerState>) -> Response {
    match cloud_sync::remote_info().await {
        Ok(data) => ok_json(data),
        // config not （ / use no ） is 400， / error is 502 ——
        // by ： before use need set， after need need or 
        Err(message) => {
            if is_config_error(&message) {
                errors::management_error(400, message)
            } else {
                errors::management_error(502, message)
            }
        }
    }
}

/// POST /api/cloud-sync/upload
pub async fn post_upload(State(state): State<ServerState>) -> Response {
    let Some(db) = state.db().cloned() else {
        return errors::management_error(500, "数据库不可用，无法上传同步数据");
    };
    match cloud_sync::upload(&db).await {
        Ok(data) => ok_json(data),
        Err(message) => {
            cloud_sync::record_failure(&message, "manual");
            if is_config_error(&message) {
                errors::management_error(400, message)
            } else {
                errors::management_error(502, message)
            }
        }
    }
}

/// POST /api/cloud-sync/download
pub async fn post_download(State(state): State<ServerState>) -> Response {
    let Some(db) = state.db().cloned() else {
        return errors::management_error(500, "数据库不可用，无法下载同步数据");
    };
    let config_dir = state.config_dir.clone();
    match cloud_sync::download(&db, &config_dir).await {
        Ok(data) => ok_json(data),
        Err(message) => {
            // 「 need need 」 not is failure： is down in ， not this 
            // 「 up failure」 （otherwise use after will to 1 ）
            if message == cloud_sync::KEY_REQUIRED {
                return errors::management_error(409, message);
            }
            cloud_sync::record_failure(&message, "manual");
            if is_config_error(&message) {
                errors::management_error(400, message)
            } else {
                errors::management_error(502, message)
            }
        }
    }
}

/// POST /api/cloud-sync/import-key
pub async fn post_import_key(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(password) = payload.get("password").and_then(Value::as_str) else {
        return errors::management_error(400, "缺少 password");
    };
    match cloud_sync::import_key(password).await {
        Ok(data) => ok_json(data),
        Err(message) => errors::management_error(400, message),
    }
}

/// this error is not is 「config」（ use need set） not is 「」（ can ）。
///
/// by not ， for is `Result<_, String>`（ and its API
/// 1 ）， as 1 error will `settings` / `transport` 。
/// config， can 502（none）。
fn is_config_error(message: &str) -> bool {
    // GET to ；invalid Location not is configerror。
    if message.starts_with("WebDAV GET") {
        return false;
    }
    message.contains("不能为空")
        || message.contains("未启用")
        || message.contains("请先设置同步密码")
        || message.contains("未设置同步密码")
        || message.contains("至少需要")
        || message.contains("只支持 http/https")
        || message.contains("地址无效")
        || message.contains("不能包含 ..")
}

/// request → `SaveInput`（ by empty， and before 「」 1 ）
fn parse_save_input(payload: &Value) -> Result<settings::SaveInput, String> {
    let Some(object) = payload.as_object() else {
        return Err("请求体必须是 JSON 对象".to_string());
    };
    let text = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    let flag = |key: &str| {
        object
            .get(key)
            .and_then(Value::as_bool)
            .unwrap_or(false)
    };
    Ok(settings::SaveInput {
        provider: {
            let provider = text("provider");
            if provider.is_empty() {
                settings::PROVIDER_WEBDAV.to_string()
            } else {
                provider
            }
        },
        webdav_base_url: text("webdavBaseUrl"),
        webdav_username: text("webdavUsername"),
        webdav_password: text("webdavPassword"),
        webdav_user_agent: text("webdavUserAgent"),
        s3_region: text("s3Region"),
        s3_bucket: text("s3Bucket"),
        s3_access_key_id: text("s3AccessKeyId"),
        s3_secret_access_key: text("s3SecretAccessKey"),
        s3_endpoint: text("s3Endpoint"),
        s3_user_agent: text("s3UserAgent"),
        sync_password: text("syncPassword"),
        enabled: flag("enabled"),
        auto_sync: flag("autoSync"),
        remote_root: text("remoteRoot"),
        profile: text("profile"),
    })
}

/// use ： 1 「 in to 」
#[allow(dead_code)]
pub fn describe() -> Value {
    let settings = settings::load();
    json!({
        "provider": settings.provider,
        "remotePath": manifest::remote_dir_display(&settings.remote_root, &settings.profile),
        "enabled": settings.enabled,
        "autoSync": settings.auto_sync,
        "lastSyncAt": settings.status.last_sync_at,
    })
}
