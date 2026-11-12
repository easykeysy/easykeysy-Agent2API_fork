//! models.dev ：`GET /api/models/metadata`、`PUT /api/models/metadata`、
//! `POST /api/models/metadata/refresh`。
//!
//! ── Why 1 （ not is `/api/models/refresh`）────────
//! `POST /api/models/refresh` is 「 up directory」 new ： 1 for 
//! 1 provider（、 this ），「get」 by provider 
//! 。models.dev not is 1 provider（ no has 、 no has up directory， has 
//! JSON）， will that 1 no has 、 not 1 。
//!
//! is ** 1 **： and in set， new and directory
//! use 1 and （ `core::model_meta::refresh`）， is 。
//!
//! and `/api/captcha` ：GET 、PUT 、response is new 。 new that 
//! return 1 （ and directory new ）， 1 。
//!
//! ── Why GET when to 「」 and 「」───────────────────────
//! set that 1 need in 1 ：「 not 」、「 this is when 、
//! has 」、「 in 1 」。 1 ， will 
//! 「 already 、 is old 」 this in 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::json;

use crate::server::config;
use crate::server::core::model_meta;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// set「 new 」 use （minutes）： this ， not and when 
/// （ when that 1 `scheduled_tasks` 「directory new 」）。
const MANUAL_INTERVAL_MINUTES: i64 = 1440;

fn metadata_json() -> serde_json::Value {
    let mut payload = model_meta::status();
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "modelsDevEnabled".to_string(),
            serde_json::Value::Bool(config::models_dev_enabled()),
        );
    }
    payload
}

/// GET /api/models/metadata —— + current（ / when / ）
pub async fn get_metadata(State(_state): State<ServerState>) -> Response {
    ok_json(metadata_json())
}

/// PUT /api/models/metadata —— body `{modelsDevEnabled: true|false}`
///
/// ** new **（ `config::KEY_MODELS_DEV_ENABLED`）： already 
/// use （ that + that not ），。
pub async fn put_metadata(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let Some(value) = object.get(config::KEY_MODELS_DEV_ENABLED) else {
        return errors::management_error(400, "缺少 modelsDevEnabled 字段");
    };
    let Some(enabled) = value.as_bool() else {
        return errors::management_error(400, "modelsDevEnabled 必须是布尔值");
    };
    if !config::set_models_dev_enabled(enabled) {
        return errors::management_error(500, "写入配置失败，请重试");
    }
    logging::log(
        "[Models]",
        if enabled {
            "models.dev 模型元数据已开启：上游未声明的能力位与思考档位由它补齐"
        } else {
            "models.dev 模型元数据已关闭：不再联网刷新（现有数据继续使用）"
        },
    );
    ok_json(metadata_json())
}

/// POST /api/models/metadata/refresh —— 1 （ and directory new use and ）
///
/// return 2xx： by `status` （`refreshed` / `skipped` /
/// `failed`）， and `/api/models/refresh` 1 —— 「 this no 」 is 
/// （ to / already in set）， not is need need use error 。
pub async fn refresh_metadata(State(_state): State<ServerState>) -> Response {
    let row = model_meta::refresh(true, MANUAL_INTERVAL_MINUTES * 60_000).await;
    let status = row.get("status").and_then(serde_json::Value::as_str).unwrap_or("");
    if status == "failed" {
        logging::log(
            "[Models]",
            &format!(
                "models.dev 元数据刷新失败：{}",
                row.get("message").and_then(serde_json::Value::as_str).unwrap_or("未知原因")
            ),
        );
    }
    ok_json(json!({ "result": row, "status": metadata_json() }))
}
