//! API HTTP client。
//!
//! ：
//! - etc after ， and request
//! - use will ， Key
//! - 1 `{ success, data }` ， data to before 
//! - when ： to 、 this ， to 60 seconds
//!
//! 127.0.0.1 up HTTP， not need need TLS。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use serde_json::Value;

/// default（ and server.mjs default 1 ）
pub const DEFAULT_PORT: u16 = 3065;

/// （debug ） default。
///
/// ── Why need 1 ──────────────────────────────────
/// and 3065（ + ），`tauri dev` 
/// if use 3065，bind failure —— before can 、 or 。
/// default by after ，`npm run tauri:dev` can and and 。
///
/// this is and `tauri.dev.conf.json`（identifier ） ：
/// and not ， not will 、
/// （ lib.rs ）。
///
/// release （ 1 ） as not ， use [`DEFAULT_PORT`]。
pub const DEV_PORT: u16 = 3066;
pub const REQUEST_TIMEOUT_MS: u64 = 60_000;

/// use 。
///
/// in when （server bind after not ），「 use set」 this 
/// in —— new after need 。 this ** before ** 
/// （、 API client URL） to is current in ，
/// not will as set already up new to 1 no 。
static ACTIVE_PORT: AtomicU16 = AtomicU16::new(0);

/// ： > configfile > default。
///
/// is as （1.x supports use and ），
/// and 、 not ，。
///
/// **configfile **（`desktop-settings.json` `proxyPort`） is 2.0.4 add ：
/// when ， for use （ need ），
/// this can —— up 「」 is this 。
///
/// ** after 1 default** by ：debug use [`DEV_PORT`]、release use 
/// [`DEFAULT_PORT`]（ by [`DEV_PORT`]）。 before not ， when 
/// to /， old use can 。
pub fn proxy_port() -> u16 {
    let cached = ACTIVE_PORT.load(Ordering::Relaxed);
    if cached != 0 {
        return cached;
    }
    resolve_port()
}

/// by and `ACTIVE_PORT`（ in use when ）。
fn resolve_port() -> u16 {
    let port = env_port("AGENT2API_PROXY_PORT")
        .or_else(|| env_port("WORKBUDDY_PROXY_PORT")) // 旧名兼容读（1.x 起沿用）
        .or_else(|| configured_port())
        .unwrap_or(default_port());
    ACTIVE_PORT.store(port, Ordering::Relaxed);
    port
}

/// no has 、set no when use default。
///
/// `debug_assertions` not is profile ： and lib.rs 「 not 
/// / new 」 is 1 ， for 「」 1 
/// （`tauri build --debug` ，）。
fn default_port() -> u16 {
    if cfg!(debug_assertions) {
        DEV_PORT
    } else {
        DEFAULT_PORT
    }
}

/// setfile ；set / 1 set（ to down 1 ）。
///
/// use `settings::load()` not is file：setfile path and 
/// （、、 not default） this has 1 。
fn configured_port() -> Option<u16> {
    let port = crate::settings::load().proxy_port;
    if port > 0 {
        Some(port)
    } else {
        None
    }
}

/// is （ when not this 「」——
/// set not will ， will use 1 ）
pub fn port_from_env() -> Option<u16> {
    env_port("AGENT2API_PROXY_PORT").or_else(|| env_port("WORKBUDDY_PROXY_PORT"))
}

/// ：set / / 0 1 set（ to down 1 ）
fn env_port(name: &str) -> Option<u16> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port > 0)
}

/// API response：`{ success, data, error }`。
/// failure when ， up to use 。
fn unwrap_envelope(text: &str, status: u16) -> Result<Value, String> {
    let payload: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        // JSON： error（e.g. to return HTML error）
        Err(_) => {
            if (200..300).contains(&status) {
                return Err(if text.trim().is_empty() {
                    "本地代理返回了空响应".to_string()
                } else {
                    text.trim().to_string()
                });
            }
            return Err(if text.trim().is_empty() {
                format!("本地代理返回 HTTP {status}，响应内容为空")
            } else {
                text.trim().to_string()
            });
        }
    };

    let ok = (200..300).contains(&status);
    let success = payload.get("success").and_then(Value::as_bool);
    if !ok || success == Some(false) {
        let detail = payload
            .get("error")
            .or_else(|| payload.get("message"))
            .map(describe_error)
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(detail);
    }
    Ok(match payload.get("data") {
        Some(data) => data.clone(),
        None => payload,
    })
}

/// error can can is ， can can is `{ message }` / `{ error: { message } }`
fn describe_error(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    if let Some(text) = value.get("message").and_then(Value::as_str) {
        return text.to_string();
    }
    if let Some(text) = value.get("msg").and_then(Value::as_str) {
        return text.to_string();
    }
    value.to_string()
}

/// configdirectory： and after use `~/.agent2api`（available）。
///
/// already to crate（`agent2api_server::paths::config_dir`，
/// 1 ）， as —— use not crate ，
/// 「 key」 and 「server」 to 1 directory。
///
/// ：`AGENT2API_PROXY_HOME` ， old `WORKBUDDY_PROXY_HOME` 
/// （1.x / can can old ）。
pub fn config_dir() -> PathBuf {
    agent2api_server::paths::config_dir()
}

/// total when Client。
///
/// request，must and 。
///
/// ── Why need ──────────────────────────────────
/// use 60 seconds is to 「 API request」 ； up / down need MB，
/// up 60 seconds not 。 is ** this 1 ** when ， not is ——
/// its requestfailure（ ）。
pub(crate) fn local_client(timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .no_proxy()
        .timeout(timeout)
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

fn request_builder_with_timeout(
    method: &str,
    path: &str,
    body: Option<&Value>,
    timeout_ms: u64,
) -> Result<reqwest::RequestBuilder, String> {
    let url = format!("http://127.0.0.1:{}{path}", proxy_port());
    let base = local_client(Duration::from_millis(timeout_ms))?;
    let mut builder = match method.to_uppercase().as_str() {
        "GET" => base.get(&url),
        "POST" => base.post(&url),
        "PUT" => base.put(&url),
        "PATCH" => base.patch(&url),
        "DELETE" => base.delete(&url),
        other => return Err(format!("不支持的请求方法: {other}")),
    };
    builder = builder.header("Accept", "application/json");
    builder = crate::panel_session::attach(builder)?;
    // and 1 ：POST/PUT/PATCH noneparameter 1 empty for ，
    // after by by 「 has body」， will 
    if let Some(payload) = body {
        builder = builder
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(payload).map_err(|e| format!("请求体序列化失败: {e}"))?);
    } else if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
        builder = builder
            .header("Content-Type", "application/json")
            .body("{}");
    }
    Ok(builder)
}

/// 1 request，return after data。
///
/// failure will and 1 ； Key error and error not 。
pub async fn call(method: &str, path: &str, body: Option<&Value>) -> Result<Value, String> {
    call_with_timeout(method, path, body, REQUEST_TIMEOUT_MS).await
}

pub async fn call_with_api_key(method: &str, path: &str, body: Option<&Value>, api_key: &str) -> Result<Value, String> {
    if path != "/api/models/test" {
        return Err("显式 API Key 仅用于模型测试".to_string());
    }
    crate::startup::wait().await?;
    let send = || async {
        let mut request = request_builder_with_timeout(method, path, body, REQUEST_TIMEOUT_MS)?.build()
            .map_err(|error| format!("构造模型测试请求失败: {error}"))?;
        let key = reqwest::header::HeaderValue::from_str(api_key)
            .map_err(|_| "API Key 包含无效的请求头字符".to_string())?;
        request.headers_mut().insert("x-api-key", key);
        let credentials = request.headers().clone();
        let response = local_client(Duration::from_millis(REQUEST_TIMEOUT_MS))?.execute(request).await
            .map_err(|error| describe_transport_error(&error))?;
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|error| format!("读取响应失败: {error}"))?;
        let payload = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
        Ok::<_, String>(RawResponse { status, text, payload, credentials })
    };
    let first = send().await?;
    if recover_management_session(&first, path)? {
        let retried = send().await?;
        return unwrap_envelope(&retried.text, retried.status);
    }
    unwrap_envelope(&first.text, first.status)
}

/// and [`call`] ， use total when 。
///
/// up / down （、 MB，60 seconds not ）。 when by use 
/// to ： that is 1 ， not in this —— will if need need request，
/// ， in 1 up can 。
pub async fn call_with_timeout(
    method: &str,
    path: &str,
    body: Option<&Value>,
    timeout_ms: u64,
) -> Result<Value, String> {
    crate::startup::wait().await?;
    let first = send_once(method, path, body, timeout_ms).await?;
    if recover_management_session(&first, path)? {
        let retried = send_once(method, path, body, timeout_ms).await?;
        return unwrap_envelope(&retried.text, retried.status);
    }
    unwrap_envelope(&first.text, first.status)
}

/// 1 request （ and ， and use ）。
struct RawResponse {
    status: u16,
    text: String,
    payload: Value,
    credentials: reqwest::header::HeaderMap,
}

fn recover_management_session(response: &RawResponse, path: &str) -> Result<bool, String> {
    if response.status != 401 || !path.starts_with("/api/") {
        return Ok(false);
    }
    let kind = response.payload.pointer("/error/type").and_then(Value::as_str);
    let management_failure = kind == Some("panel_login_required")
        || (kind == Some("invalid_api_key") && !response.credentials.contains_key("x-api-key"));
    if !management_failure {
        return Ok(false);
    }
    crate::panel_session::recover_rejected(&response.credentials)
}

async fn send_once(
    method: &str,
    path: &str,
    body: Option<&Value>,
    timeout_ms: u64,
) -> Result<RawResponse, String> {
    let request = request_builder_with_timeout(method, path, body, timeout_ms)?.build()
        .map_err(|error| format!("构造管理请求失败: {error}"))?;
    let credentials = request.headers().clone();
    let response = local_client(Duration::from_millis(timeout_ms))?
        .execute(request)
        .await
        .map_err(|error| describe_transport_error(&error))?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .await
        .map_err(|error| format!("读取响应失败: {error}"))?;
    let payload = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    Ok(RawResponse {
        status,
        text,
        payload,
        credentials,
    })
}

/// （ use ；call will by JSON ， not ）。
/// will when and 1 。
pub async fn call_text(method: &str, path: &str) -> Result<String, String> {
    crate::startup::wait().await?;
    let first = send_once(method, path, None, REQUEST_TIMEOUT_MS).await?;
    if recover_management_session(&first, path)? {
        let retried = send_once(method, path, None, REQUEST_TIMEOUT_MS).await?;
        return checked_text(retried);
    }
    checked_text(first)
}

fn checked_text(response: RawResponse) -> Result<String, String> {
    if !(200..300).contains(&response.status) {
        return match unwrap_envelope(&response.text, response.status) {
            Err(message) => Err(message),
            Ok(_) => Err(format!("HTTP {}", response.status)),
        };
    }
    Ok(response.text)
}

fn describe_transport_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        // 「 when 」 this will as no 。 when request already 
        // 、 after in （）， is not etc —— need this 1 ，
        // otherwise use will 1 ， 1 。
        "等待本地代理响应超时（网关可能仍在处理，请稍后查看结果）".to_string()
    } else {
        format!("无法连接本地代理（127.0.0.1:{}）: {error}", proxy_port())
    }
}
