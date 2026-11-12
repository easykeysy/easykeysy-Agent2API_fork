//! up ：** 1 request** and error。
//!
//! ── this 1 in ──────────────────────────────────────
//! `send_raw`： 1 request、 response `{status, ok, payload}`， by use 
//! `core::egress`（ not = ）。
//!
//! in （Raccoon / AutoClaw / CodeArts / ） use 
//! （、directory、、）。WorkBuddy that `unwrap`、
//! 、`Context` to 。
//!
//! `WorkBuddyAuthError` in this ： is 「failure」 ，
//! use （、、 by ） by `status_code` / `upstream_code`。

use std::time::Duration;

use serde_json::Value;

use crate::server::core::egress;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::errors::GatewayError;

/// token new ： before 5 minutes as need need new 
pub const REFRESH_WINDOW_MS: f64 = 5.0 * 60.0 * 1000.0;

/// defaultrequest when （ when not can API ）
pub const REQUEST_TIMEOUT_MS: u64 = 30_000;

/// error（ for Node WorkBuddyAuthError）。
///
/// `status_code` as None when by by by 502 —— and Node 
/// `Number.isInteger(error.statusCode) ? error.statusCode : 502` 1 。
#[derive(Clone, Debug)]
pub struct WorkBuddyAuthError {
    pub message: String,
    pub status_code: Option<i32>,
    pub upstream_code: Option<i64>,
}

impl WorkBuddyAuthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), status_code: None, upstream_code: None }
    }

    pub fn with_status(status: i32, message: impl Into<String>) -> Self {
        Self { message: message.into(), status_code: Some(status), upstream_code: None }
    }

    pub fn with_code(message: impl Into<String>, status: Option<i32>, code: Option<i64>) -> Self {
        Self { message: message.into(), status_code: status, upstream_code: code }
    }

    /// 1 after HTTP （none → 502， Node ）
    pub fn http_status(&self) -> i32 {
        self.status_code.unwrap_or(502)
    }

    /// 1 error（ by by API return）
    pub fn to_gateway_error(&self) -> GatewayError {
        let mut error = GatewayError::with_status(self.http_status(), self.message.clone());
        if let Some(code) = self.upstream_code {
            error = error.upstream_code(code);
        }
        error
    }
}

impl std::fmt::Display for WorkBuddyAuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

/// 1 up request ： + is 2xx + JSON（failure as None）。
#[derive(Clone, Debug, Default)]
pub struct ApiResponse {
    pub status: u16,
    pub ok: bool,
    pub payload: Option<Value>,
}

/// 1 up request，return and payload。
///
/// ── ──────────────────────────────────────────────────
/// `proxy` as `None` when （「 no 」 and 「failure」
/// ）；otherwise to `core::egress::client_for` this Client（）。
///
/// ── when ──────────────────────────────────────────────────
/// `timeout_ms` is **requesttotal when **， and client up `connect_timeout` /
/// `read_timeout` is （ to ）。 None not total when 。
pub async fn send_raw(
    method: &str,
    url: &str,
    body: Option<&Value>,
    headers: &[(String, String)],
    proxy: Option<&ResolvedProxy>,
    timeout_ms: Option<u64>,
) -> Result<ApiResponse, reqwest::Error> {
    let client = egress::client_for(proxy);
    let mut builder = match method {
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };
    builder = builder.header("Accept", "application/json");
    for (key, value) in headers {
        builder = builder.header(key, value);
    }
    if let Some(payload) = body {
        builder = builder
            .header("Content-Type", "application/json")
            .body(payload.to_string());
    }
    if let Some(timeout) = timeout_ms {
        builder = builder.timeout(Duration::from_millis(timeout));
    }
    let response = builder.send().await?;
    let status = response.status().as_u16();
    let ok = response.status().is_success();
    let text = response.text().await?;
    let payload = if text.trim().is_empty() {
        None
    } else {
        serde_json::from_str::<Value>(&text).ok()
    };
    Ok(ApiResponse { status, ok, payload })
}
