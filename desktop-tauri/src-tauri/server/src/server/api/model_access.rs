use axum::http::HeaderMap;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::api_keys;
use crate::server::core::key_governance;
use crate::server::core::key_scope::KeyScope;
use crate::server::errors::GatewayError;
use crate::server::request_stats::{ApiKeyIdentity, RequestSource};

pub const MODEL_TEST: &str = "model_test";
pub const ASSISTANT: &str = "assistant";
pub const ASSISTANT_IMAGES: &str = "assistant_images";
pub const ASSISTANT_TITLE: &str = "assistant_title";
pub const ENV_KEY_ID: &str = "env";

pub fn is_model_endpoint(path: &str) -> bool {
    matches!(path, "/api/models/test" | "/api/model-access/models" | "/api/assistant/chat" | "/api/assistant/images/generations" | "/api/assistant/images/edits")
        || path.strip_prefix("/api/assistant/sessions/").is_some_and(|tail| tail.ends_with("/title"))
        // 工具端点自行校验网关 Key；账号型搜索在运行时校验提供商权限，
        // 独立搜索 API 和网页抓取不使用模型白名单。
        || path.starts_with("/api/web-search")
        || path.starts_with("/skill/")
        || path.starts_with("/mcp/")
}

/// 认证结果：包含本次请求的 Key 身份/范围，以及（如有）匹配到的可治理 Key 记录。
#[derive(Clone, Debug)]
pub struct ModelAccess {
    pub scope: Option<KeyScope>,
    pub source: RequestSource,
    /// 匹配到的网关 Key 记录（环境变量 Key 为 `None`）
    pub key_entry: Option<api_keys::ApiKeyEntry>,
}

fn presented_key(headers: &HeaderMap) -> Option<String> {
    // 显式选择的 x-api-key 不得被 Authorization 中的管理凭证兜底替换。
    let presented = if let Some(value) = headers.get("x-api-key") {
        value.to_str().ok()
    } else {
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .map(crate::server::http::strip_bearer_prefix)
    };
    presented
        .filter(|key| !key.is_empty())
        .map(|key| key.to_string())
}

/// 认证并（对可治理 Key）执行配额/过期/RPM 检查。
/// `count_request=true` 时将本次通过的请求计入配额与 RPM 窗口；
/// 自助查询等不应消耗配额的场景传 `false`。
pub fn authenticate(headers: &HeaderMap, client: &str, count_request: bool) -> Result<ModelAccess, GatewayError> {
    let key = presented_key(headers).ok_or_else(|| {
        GatewayError::with_status(401, "模型调用必须选择并携带 API Key，请先在「网关 Key」页创建并启用一把 Key")
            .with_code("model_api_key_required")
    })?;
    let snapshot = config::current();
    if config::parse::env_api_key_value().as_deref() == Some(key.as_str()) {
        // 环境变量注入的 Key：不受 per-key 治理配置约束（其生命周期由环境管理）
        return Ok(ModelAccess {
            scope: None,
            source: RequestSource {
                client: client.to_string(),
                api_key: ApiKeyIdentity { source: "env".to_string(), ..Default::default() },
                ..Default::default()
            },
            key_entry: None,
        });
    }
    let entry = api_keys::entries_from(snapshot.raw())
        .into_iter()
        .find(|entry| entry.is_usable() && entry.key == key)
        .ok_or_else(|| {
            GatewayError::with_status(401, "所选 API Key 已失效、停用或不可用，请重新选择一把启用的 Key")
                .with_code("invalid_api_key")
        })?;

    // 治理检查：过期 / 配额 / RPM（在放行前执行，失败不计入请求）
    if let Err(reason) = key_governance::admit_request(&entry, count_request) {
        let (status, code, message, retry_after) = key_governance::deny_to_http(&reason);
        let mut err = GatewayError::with_status(status as i32, message).with_code(code);
        if let Some(secs) = retry_after {
            err = err.with_retry_after_seconds(secs);
        }
        return Err(err);
    }

    Ok(ModelAccess {
        scope: Some(KeyScope::from_entry(&entry)),
        source: RequestSource {
            client: client.to_string(),
            api_key: ApiKeyIdentity {
                id: entry.id.clone(),
                name: entry.name.clone(),
                source: "key".to_string(),
            },
            ..Default::default()
        },
        key_entry: Some(entry),
    })
}

/// GET /v1/models —— 按 Key 白名单过滤的模型列表（或无 Key 时公开视图，见 chat.rs）。
pub async fn models(
    axum::extract::State(state): axum::extract::State<crate::server::ServerState>,
    headers: HeaderMap,
) -> axum::response::Response {
    match authenticate(&headers, "", true) {
        Ok(access) => crate::server::http::ok_json(
            crate::server::core::providers::catalog::models_response(state.store(), access.scope.as_ref()),
        ),
        Err(error) => error.payload_response(),
    }
}

/// 消费者自助查询：返回当前 Key 的身份、scope、治理限制、用量与剩余。
/// 不计入请求配额/RPM（只读状态查询）。
pub async fn key_info(headers: HeaderMap) -> axum::response::Response {
    match authenticate(&headers, "key-info", false) {
        Ok(access) => {
            if let Some(entry) = access.key_entry {
                crate::server::http::ok_json(json!({
                    "id": entry.id,
                    "name": entry.name,
                    "masked": api_keys::mask(&entry.key),
                    "enabled": entry.enabled,
                    "createdAt": entry.created_at,
                    "allowedProviders": entry.allowed_providers,
                    "allowedModels": entry.allowed_models,
                    "governance": key_governance::status_json(&entry),
                }))
            } else {
                // 环境变量 Key：无 per-key 治理记录
                crate::server::http::ok_json(json!({
                    "id": ENV_KEY_ID,
                    "name": "环境变量 Key",
                    "source": "env",
                    "governance": { "limits": null, "usage": null, "remaining": null, "note": "env key is not subject to per-key governance" }
                }))
            }
        }
        Err(error) => error.payload_response(),
    }
}

pub async fn list_keys() -> axum::response::Response {
    crate::server::http::ok_json(json!({ "keys": key_choices() }))
}

pub fn key_choices() -> Vec<Value> {
    let mut choices: Vec<Value> = api_keys::list()
        .into_iter()
        .filter(api_keys::ApiKeyEntry::is_usable)
        .map(|entry| {
            let mut value = entry.public_json();
            value["source"] = json!("key");
            value
        })
        .collect();
    if let Some(key) = config::parse::env_api_key_value() {
        choices.push(json!({
            "id": ENV_KEY_ID,
            "name": "环境变量 Key",
            "masked": api_keys::mask(&key),
            "key": key,
            "source": "env",
            "enabled": true,
            "allowedProviders": [],
            "allowedModels": [],
            "governance": null,
        }));
    }
    choices
}
