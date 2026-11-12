//! for ****：request、 up error、 id 。
//!
//! ── fileWhy（Agent2API W2b-T3）────────────
//! before this has 、URL 、system and 11128/6004 ——
//! that is **workbuddy **（X-IDE-* 、`/v2/chat/completions`、
//! must is system、11128 ）， already 
//! `core::providers::workbuddy`。 after provider in ，
//! file for 「 up is 1 」 1 none： `TransportRequest` this 
//! **none** （URL + + already body + ）。
//!
//! ：
//! 1. `send_chat_request`： by reqwest Client 1 request，
//! not body（SSE need need `bytes_stream()`）；
//! 2. `read_upstream_error`： up errorresponse 1 `{code, message}`
//! —— 「 1 HTTP error」 is ， and 1 none；
//! 3. `new_request_id`： 1 for id（ when use ）。
//!
//! ── ────────────────────────────────────────────────────
//! `core::egress::client_for`： 1 use 1 ，
//! client up connect/read when 1 and use （ egress ）。

use std::time::Duration;

use serde_json::{json, Value};

use crate::server::core::egress;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::errors::GatewayError;

/// for request total when （ etc when ）。
///
/// and Node ：Node proxyFetch not total when （bodyTimeout: 0），
/// client and up 。Rust reqwest client use 
/// `read_timeout(600s)` limit「」 （ core::egress ），
/// this not total when —— that will SSE 。
pub const NO_TOTAL_TIMEOUT: Option<u64> = None;

/// ** etc up response** limit（ms）。
///
/// and OmniProxy `headers_timeout`（default 300 seconds）： after 、
/// response to before etc must has limit —— `read_timeout` is 「
/// 」， before etc ，minutes etc × （ ×
/// ） will 1 request to minutes，client etc not and 。
///
/// in is **config**（set「request when → etc response when 」，default 300 seconds，
/// 1–3600）： when from 1 ，set down 1 request。
///
/// Whydefault 300 seconds is ：request up ** `stream: true`**（
/// `UpstreamService::forward` ），response in SSE when to ，
/// and 「」none； this path（）。
/// when by failure（502 + has ）， and failure 1 。
fn headers_timeout() -> Duration {
    Duration::from_millis(crate::server::config::timeout_settings().headers_ms())
}

/// 「 in when 」 （seconds）： when seconds use ，
/// and set「request when → in when 」 1 config。
fn connect_timeout_seconds() -> u64 {
    crate::server::config::timeout_settings().connect_ms() / 1000
}

/// 1 up request （none；provider in already ）
#[derive(Clone)]
pub struct TransportRequest {
    /// up URL（ by to ）
    pub url: String,
    /// request（ by to ； not ）
    pub headers: Vec<(String, String)>,
    /// already request（ by from `plan.body` ）
    pub payload: String,
    /// （； and provider none， by after up ）
    pub proxy: Option<ResolvedProxy>,
    /// when is ****（ by can ，
    /// default false = ）。 as true `ProviderAdapter::system_proxy_when_unset`。
    pub system_proxy_when_unset: bool,
}

/// 1 after up error：`{code, message}`
///
/// request `message` response before 500 ，request。
/// `classify_error` is JSON （`to_value`）——
/// is 「 up error」 and 「 this error 1 」：
/// before is file（）， after is （provider ）。
pub struct UpstreamErrorDetail {
    pub code: Option<i64>,
    pub message: String,
    pub payload: Option<Value>,
}

impl UpstreamErrorDetail {
    /// ：`{code, message}`（ and up JSON 1 ，
    /// is can use `error_body.get("code")` ， not need need in ）
    pub fn to_value(&self) -> Value {
        let mut body = self.payload.clone().filter(Value::is_object).unwrap_or_else(|| json!({}));
        if let Some(object) = body.as_object_mut() {
            if let Some(code) = self.code { object.insert("code".into(), Value::from(code)); }
            object.insert("message".into(), Value::String(self.message.clone()));
        }
        body
    }
}

/// up errorresponse → `{code, message}`（ for Node readUpstreamError）。
///
/// `capture`： （None = ）。**must in this ** —— 
/// use `response.text()` response ， use not to ； not in this 
/// ，errorresponse not （failure is need need that 1 ）。
pub async fn read_upstream_error(
    response: reqwest::Response,
    capture: Option<&crate::server::core::debug_traffic::TrafficCapture>,
) -> UpstreamErrorDetail {
    if let Some(capture) = capture {
        capture.attach_response(response.status().as_u16(), response.headers());
    }
    let text = response.text().await.unwrap_or_default();
    if let Some(capture) = capture {
        capture.push(text.as_bytes());
    }
    let mut code = None;
    let mut parsed = None;
    let mut message = if capture.is_some() {
        text.clone()
    } else {
        text.chars().take(500).collect()
    };
    if let Ok(payload) = serde_json::from_str::<Value>(&text) {
        code = payload.get("code").and_then(Value::as_i64);
        // Node: `payload?.message || payload?.msg || payload?.error?.message || message`
        let candidate = payload
            .get("message")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .or_else(|| {
                payload
                    .get("msg")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
            })
            .or_else(|| {
                payload
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
            });
        if let Some(candidate) = candidate {
            message = candidate.to_string();
        }
        parsed = Some(payload);
    }
    UpstreamErrorDetail { code, message, payload: parsed }
}

/// 1 up request（ not body，response to and error）。
///
/// and `core::auth_http::send_raw` ： that response，used for；
/// this `reqwest::Response` to use ，SSE need need `bytes_stream()`。
pub async fn send_chat_request(
    plan: &TransportRequest,
) -> Result<reqwest::Response, UpstreamRequestError> {
    // client 1 ： → （ can ，
    // `TransportRequest::system_proxy_when_unset`）→ 
    let client = match plan.proxy.as_ref() {
        Some(proxy) => egress::client_for(Some(proxy)),
        None if plan.system_proxy_when_unset => egress::client_for_system_proxy(),
        None => egress::client_for(None),
    };
    send_chat_with_client(plan, &client).await
}

pub(super) async fn send_chat_with_client(
    plan: &TransportRequest,
    client: &reqwest::Client,
) -> Result<reqwest::Response, UpstreamRequestError> {
    let multipart = plan.headers.iter().any(|(key, value)| {
        key.eq_ignore_ascii_case("content-type") && value.eq_ignore_ascii_case("multipart/form-data")
    });
    let mut builder = client.post(&plan.url);
    if multipart {
        let form = super::multipart::request_form(&plan.payload).map_err(|message| {
            UpstreamRequestError {
                message,
                reason: "上传请求构造失败".to_string(),
                safe_to_retry: false,
                network_failure: false,
            }
        })?;
        builder = builder.multipart(form);
    } else {
        builder = builder.body(plan.payload.clone());
    }
    for (key, value) in &plan.headers {
        if multipart && key.eq_ignore_ascii_case("content-type") {
            continue;
        }
        builder = builder.header(key, value);
    }
    if let Some(timeout) = NO_TOTAL_TIMEOUT {
        builder = builder.timeout(Duration::from_millis(timeout));
    }
    // （failure use ）： Node fetchViaProxy
    let via = match &plan.proxy {
        Some(proxy) if !proxy.label.is_empty() => format!("经代理 {}", proxy.label),
        Some(proxy) => format!("经代理 {}", proxy.host),
        // （Antigravity） in when is ，
        // not can 「」—— when need can 1 from （
        // `ProviderAdapter::system_proxy_when_unset`）
        None if plan.system_proxy_when_unset => "跟随系统代理".to_string(),
        None => "直连".to_string(),
    };
    // etc response has limit（ headers_timeout ）： when after future ，
    // up （ and client when is 1 ）
    let headers_budget = headers_timeout();
    match tokio::time::timeout(headers_budget, builder.send()).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(error)) => {
            // `send()` when can can （ etc response by when ，
            // ≤ client read_timeout， egress ）： when error
            // for use no has （ is 「 no up 」）， to set 
            // and seconds； its （ECONNREFUSED、failure、DNS…）
            // and ， is not is 
            let reason = if error.is_timeout() {
                format!("连接中超时({}秒)", connect_timeout_seconds())
            } else {
                "上游连接失败".to_string()
            };
            let message = if error.is_timeout() {
                format!("连接中超时({}秒，出口 {via})", connect_timeout_seconds())
            } else {
                format!(
                    "上游请求失败（{via}）: {}",
                    egress::describe_error_detail(&error)
                )
            };
            let safe_to_retry = error.is_connect() || error.is_builder();
            let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
            let mut certificate_error = false;
            while let Some(error) = cause {
                let message = error.to_string().to_ascii_lowercase();
                certificate_error |= message.contains("certificate") || message.contains("invalid peer")
                    || message.contains("unknownissuer") || message.contains("certnotvalid");
                cause = error.source();
            }
            let network_failure = !certificate_error && !error.is_builder() && (error.is_connect() || error.is_timeout());
            Err(UpstreamRequestError { message, reason, safe_to_retry, network_failure })
        }
        Err(_elapsed) => {
            let seconds = headers_budget.as_secs();
            Err(UpstreamRequestError {
                reason: format!("等待响应超时({seconds}秒)"),
                message: format!("等待响应超时({seconds}秒，出口 {via})"),
                safe_to_retry: false,
                network_failure: true,
            })
        }
    }
}

/// failure（ 1 502， and Node fetchViaProxy 1 ）
#[derive(Clone, Debug)]
pub struct UpstreamRequestError {
    pub message: String,
    /// Reason（「 this Why」 ）：
    /// 「 in when (Nseconds)」/「 etc response when (Nseconds)」/「 up failure」。
    /// `message` is failure （）， this is 。
    pub reason: String,
    /// has request（ or failure） etc 。
    pub safe_to_retry: bool,
    pub network_failure: bool,
}

impl UpstreamRequestError {
    pub fn to_gateway_error(&self) -> GatewayError {
        GatewayError::with_status(502, self.message.clone())
    }
}

/// 1 for id（ for Node `randomUUID()`）。
///
/// use RFC 4122 v4 ： up will X-Request-ID server，
/// UUID and client as for （ down by UUID ）。
/// `RandomState`（ by OS ）+ + seconds when ，
/// id in 1 —— not is ， not use rand 。
pub fn new_request_id() -> String {
    use std::hash::{BuildHasher, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut bytes = [0u8; 16];
    let mix = |label: &[u8], salt: u64| -> u64 {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write(label);
        hasher.write_u64(salt);
        hasher.finish()
    };
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let pid = std::process::id() as u64;
    let first = mix(b"wb-request-id-a", nanos ^ pid);
    let second = mix(b"wb-request-id-b", counter.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    bytes[..8].copy_from_slice(&first.to_be_bytes());
    bytes[8..].copy_from_slice(&second.to_be_bytes());
    // 4 + 10xx（RFC 4122）
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

// ─── UUIDv7 （WorkBuddy for will // id）──────
//
// 2026-10-16 for client 5.8.4 （
// workbuddy2api-hub #381， and for ）：client request / id is 
// 32 none hex、 will id is UUID， is UUIDv7 
// （48 ms when ， when has ）。 up not —— for 
// X-Conversation-ID is v4、request is v7， —— 
// ， not 。 up not this id for （ 1 id 、
// system+user， not up 1 ），**request**，
// not by will ； will （400 / code 11-128，
// `conversation id missing`）。

/// 16 UUIDv7（RFC 9562）：ms when + 7 + 10。
///
/// and [`new_request_id`] （`RandomState` + ），
/// not as new 。
fn uuid7_bytes() -> [u8; 16] {
    use std::hash::{BuildHasher, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mix = |label: &[u8], salt: u64| -> u64 {
        let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
        hasher.write(label);
        hasher.write_u64(salt);
        hasher.finish()
    };
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
        & ((1 << 48) - 1);
    let rand_a = mix(b"wb-uuid7-a", counter) & 0x0fff;
    let rand_b = mix(b"wb-uuid7-b", counter.wrapping_mul(0x9E37_79B9_7F4A_7C15)) & ((1 << 62) - 1);
    let int = ((millis as u128) << 80)
        | (0x7u128 << 76)
        | ((rand_a as u128) << 64)
        | (0b10u128 << 62)
        | (rand_b as u128);
    int.to_be_bytes()
}

/// request / id（`X-Request-ID` / `X-Conversation-Message-ID` /
/// `X-Conversation-Request-ID` / `X-Root-Request-ID`）：32 none hex。
pub fn new_message_id() -> String {
    let bytes = uuid7_bytes();
    let mut id = String::with_capacity(32);
    for byte in bytes {
        id.push_str(&format!("{byte:02x}"));
    }
    id
}

/// will id（`X-Conversation-ID`）： UUID 。
pub fn new_conversation_id() -> String {
    let bytes = uuid7_bytes();
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

/// span id（`traceparent` / b3 / `X-B3-SpanId`）：16 hex。
pub fn new_span_id() -> String {
    let bytes = uuid7_bytes();
    let mut id = String::with_capacity(16);
    for byte in &bytes[..8] {
        id.push_str(&format!("{byte:02x}"));
    }
    id
}
