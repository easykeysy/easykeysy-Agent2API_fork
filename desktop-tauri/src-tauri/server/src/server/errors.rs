//! error and OpenAI error payload。
//!
//! for Node server.mjs 313-332 `errorPayload()`：
//! up /error 1 `statusCode`（optional `upstreamCode`），
//! by to client `{ error: { message, type, ... } }`。
//!
//! then （， not 「」）：
//! - 429 or up 6004 → rate_limit_exceeded（ and reset_at when ）
//!   - 401                          → authentication_error
//!   - statusCode < 500             → invalid_request_error
//! - its （ 5xx and status） → proxy_error
//!
//! default 500：Node use `Number.isInteger(statusCode) ? statusCode : 500`，
//! this use `Option<i32>` 1 。
//!
//! Note: release profile is `panic = "abort"`，handler panic
//! will use ，error 1 Result + ，
//! not use unwrap/expect。

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::TimeZone;
use serde_json::{json, Map, Value};

/// up （Node QUOTA_LIMIT_CODE）
pub const QUOTA_LIMIT_CODE: i64 = 6004;

/// 1 error。
///
/// `status_code` for Node error for up `statusCode`，
/// `upstream_code` for `upstreamCode`（ up will in response，
/// client ），
/// `code` for Node body `code` （ before has not in use ：
/// `code: 'model_not_found'`）。
#[derive(Clone, Debug)]
pub struct GatewayError {
    pub message: String,
    pub status_code: i32,
    pub upstream_code: Option<i64>,
    pub code: Option<String>,
    /// `Retry-After`（seconds）；`None` = not this 。
    ///
    /// has ** when ** error this has （
    /// [`GatewayError::with_retry_after_seconds`]）： to not when not to ，
    /// client by —— 1 etc when not to 。
    pub retry_after_secs: Option<u64>,
    /// onlyadd scoped runtime ； not ， not up code。
    pub(crate) conversation_class: Option<crate::server::core::providers::adapter::UpstreamErrorClass>,
}

impl GatewayError {
    /// use ：default 500（ for Node 「 500」）
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: 500,
            upstream_code: None,
            code: None,
            retry_after_secs: None,
            conversation_class: None,
        }
    }

    /// 
    pub fn with_status(status: i32, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: status,
            upstream_code: None,
            code: None,
            retry_after_secs: None,
            conversation_class: None,
        }
    }

    /// 400 parametererror（ after use ）
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::with_status(400, message)
    }

    /// up error（Node body `code` ）
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// up up （ 6004 ）
    pub fn upstream_code(mut self, code: i64) -> Self {
        self.upstream_code = Some(code);
        self
    }

    /// up （Option ；None when not ）——
    /// up is Option， use this can 1 。
    pub fn with_optional_code(mut self, code: Option<i64>) -> Self {
        if let Some(code) = code {
            self.upstream_code = Some(code);
        }
        self
    }

    /// is as error（HTTP 429 or up code 6004）
    pub fn is_quota_limit(&self) -> bool {
        if let Some(class) = &self.conversation_class {
            if matches!(self.status_code, 499 | 504 | 556) { return false; }
            return matches!(class,
                crate::server::core::providers::adapter::UpstreamErrorClass::QuotaLimited { status, .. }
                    if !matches!(*status, 499 | 504 | 556));
        }
        self.status_code == 429 || self.upstream_code == Some(QUOTA_LIMIT_CODE)
    }

    /// up `Retry-After`（seconds）。
    ///
    /// use in ** already when ** error up （ up to 、
    /// ）。⚠️ not —— client will this ， 1 when not to 。
    /// 0 seconds will responseignore（ `response`）。
    pub fn with_retry_after_seconds(mut self, seconds: u64) -> Self {
        self.retry_after_secs = Some(seconds);
        self
    }

    /// 1 after ： 100..600 1 500（ HTTP response）
    fn safe_status(&self) -> i32 {
        if (100..600).contains(&self.status_code) {
            self.status_code
        } else {
            500
        }
    }

    /// error， Node errorPayload 
    fn error_type(&self) -> &'static str {
        if self.is_quota_limit() {
            "rate_limit_exceeded"
        } else if self.status_code == 401 {
            "authentication_error"
        } else if self.status_code < 500 {
            "invalid_request_error"
        } else {
            "proxy_error"
        }
    }

    /// OpenAI error body（ for Node errorPayload().body）。
    ///
    /// and Node 1 ：message → type → upstream_code（optional）→
    /// `code`（optional， body use ）→ reset_at（optional）。
    /// `reset_at` / `reset_at_text` in success when when （Node ）。
    pub fn payload(&self) -> Value {
        let mut error = Map::new();
        error.insert("message".to_string(), Value::String(self.message.clone()));
        error.insert("type".to_string(), Value::String(self.error_type().to_string()));
        if let Some(code) = &self.code {
            error.insert("code".to_string(), Value::String(code.clone()));
        }
        if let Some(code) = self.upstream_code {
            error.insert("upstream_code".to_string(), Value::from(code));
        }
        if self.is_quota_limit() {
            let reset_at = match &self.conversation_class {
                Some(crate::server::core::providers::adapter::UpstreamErrorClass::QuotaLimited { reset_at, .. }) => {
                    let now = crate::server::logging::now_ms();
                    reset_at.filter(|reset| valid_future_reset(*reset, now))
                        .or_else(|| self.retry_after_secs.and_then(|seconds| reset_from_retry(seconds, now)))
                        .unwrap_or(0)
                }
                Some(_) => 0,
                // old no has ， has and not 。
                None => parse_quota_reset_at(&self.message),
            };
            if reset_at > 0 {
                error.insert("reset_at".to_string(), Value::from(reset_at));
                error.insert(
                    "reset_at_text".to_string(),
                    Value::String(format_reset_at_text(reset_at)),
                );
            }
        }
        json!({ "error": Value::Object(error) })
    }

    /// HTTP response（ + OpenAI body）。
    ///
    /// and `IntoResponse` etc ， is use ——
    /// `Err(error) => error.payload_response()` 
    /// `IntoResponse::into_response(error)` can 。
    pub fn payload_response(self) -> Response {
        self.response()
    }

    /// 1 response： + body（+ has when when `Retry-After`）。
    ///
    /// `IntoResponse` and `payload_response` this ， 1 
    /// （ before 1 `(status, Json(payload))`）。
    fn response(self) -> Response {
        let status = self.http_status();
        // 0 seconds not `Retry-After` ： up 「 up 」 when client，
        // not is to 1 `Retry-After: 0` 1 。
        let retry_after = self.retry_after_secs.filter(|seconds| *seconds > 0);
        let mut response = (status, Json(self.payload())).into_response();
        if let Some(seconds) = retry_after {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }

    /// HTTP 
    pub fn http_status(&self) -> StatusCode {
        StatusCode::from_u16(self.safe_status() as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

fn valid_future_reset(reset: i64, now: i64) -> bool {
    reset > now && reset > 0 && reset <= (1_i64 << 53) - 1
        && chrono::DateTime::from_timestamp_millis(reset).is_some()
}

fn reset_from_retry(seconds: u64, now: i64) -> Option<i64> {
    let reset = i64::try_from(seconds).ok()?.checked_mul(1000)?.checked_add(now)?;
    valid_future_reset(reset, now).then_some(reset)
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for GatewayError {}

/// handler can `Result<T, GatewayError>` return，error
/// + OpenAI body（none need handler ）。
impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        self.response()
    }
}

/// from `std::io::Error` / `serde_json::Error` this error 500。
impl From<std::io::Error> for GatewayError {
    fn from(error: std::io::Error) -> Self {
        Self::new(format!("文件操作失败: {error}"))
    }
}

impl From<serde_json::Error> for GatewayError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(format!("JSON 解析失败: {error}"))
    }
}

/// 401 response。
///
/// and type Node unauthorized()：
/// `{error:{message:"Unauthorized: invalid or missing API key", type:"invalid_api_key"}}`
///
/// Note type is `invalid_api_key` ** not is ** errorPayload() to 401 use 
/// `authentication_error` —— as Node unauthorized() is body，
/// no errorPayload()。 not need use 。
pub fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": {
                "message": "Unauthorized: invalid or missing API key",
                "type": "invalid_api_key",
            }
        })),
    )
        .into_response()
}

/// 401：（web_shim 「」 not is Key ）。
///
/// `type` use `panel_login_required` and Key failure（`invalid_api_key`）
/// —— is 401， for not 。
///
/// need need `headers` is as 「refresh not 」—— down 。
pub fn panel_login_required_response(headers: &axum::http::HeaderMap) -> Response {
    panel_login_required_response_with_refresh_state(
        crate::server::access::has_live_refresh(headers),
    )
}

/// up ，「refresh 」 by use 。
///
/// `http::require_api_key` as to 401 already 1 `has_live_refresh`；
/// this 1 is 1 request for refresh 。must** 1 
/// request** —— will 「 not cookie」， this use 1 up 
/// that 。
pub fn panel_login_required_response_with_refresh_state(live_refresh: bool) -> Response {
    let mut response = (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": {
                "message": "请先登录管理面板",
                "type": "panel_login_required",
            }
        })),
    )
        .into_response();
    // in ** will no ** when that will cookie —— `access::has_live_refresh`。
    //
    // refresh when **must**：access has 2 when （`ACCESS_TTL`），
    // will cookie when 30 refresh（ 1 ， `access::ACCESS_COOKIE`）。
    // 401 none etc 30 2 when 1 —— cookie
    // need and Path when in ， after client that `/api/panel/refresh` no has 
    // refresh （ cookie down localStorage no has ， not ）。
    //
    // already no when ， not is "" ： that 、
    // server new 1 invalid（ 1 ： 1 for not up 51 ，
    // when ； after dev up 1 from ， after is 0 ）。
    //
    // when 1 `Set-Cookie`： and in will 1 （
    // `access::session_valid` ）， etc 1 to not 。
    if !live_refresh {
        if let Ok(value) = axum::http::HeaderValue::from_str(&crate::server::access::clear_cookie())
        {
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, value);
        }
    }
    response
}

/// 503：headless 公网形态未配置任何 API Key 时，`/v1/*` fail-closed response。
///
/// and 401 not ： this not is 「Key 」， is 「 no has Key can 」——
/// use create 1 （， not need need Key）。
pub fn v1_fail_closed_response() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "error": {
                "message": "not configured API Key：， in 「 Key」create 1 ",
                "type": "api_key_not_configured",
            }
        })),
    )
        .into_response()
}

/// 404 response。
///
/// Note this ** not type ** —— Node 404 is 
/// `{ error: { message: "Not found: <METHOD> <path>" } }`，
/// no has errorPayload()， not will has type/upstream_code。
/// 1 not is use GatewayError， is as not 「」 up type。
pub fn not_found_response(method: &str, path: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": { "message": format!("Not found: {method} {path}") } })),
    )
        .into_response()
}

/// API error `{ success: false, error: "..." }`。
/// and OpenAI payload not —— （/api/*） this ，
/// for Node route `sendJson(res, status, { success:false, error })`。
pub fn management_error(status: i32, message: impl Into<String>) -> Response {
    let status = StatusCode::from_u16(status as u16).unwrap_or(StatusCode::BAD_REQUEST);
    (status, Json(json!({ "success": false, "error": message.into() }))).into_response()
}

/// up when ：`YYYY-MM-DD HH:MM:SS UTC+8` → ms when 。
///
/// for Node parseQuotaResetAt（workbuddy-upstream-client.mjs 82-87 ）：
/// ```js
/// /(\d{4})-(\d{2})-(\d{2})\s+(\d{2}):(\d{2}):(\d{2})\s*UTC\+8/
/// ```
/// not when return 0（Node return 0， use need not need reset_at ），
/// not is —— this path in up error， not can new error。
///
/// ── Why「 by + 」（ 4 and ）────
/// up 1 is 「 + `&text[start..start+19]` 」， in ** in error** up 
/// will panic（release is panic=abort， will use ）：
/// up 429 msg is 「 usage already limit， will in 2026-09-11 19:43:46
/// UTC+8 …」， 1 in in （「」 2、3 ），
/// in up ，`is_char_boundary` check in ** after ** already not and 。
/// this path in 429 response up 100% 。
///
/// in by `&[u8]` ： has is `dddd-dd-dd` 
/// （ is ASCII and `-`）， ASCII not can can is 
/// ， after in up 。 Node then 
/// `\s+`（ and when 1 empty） and `\s*UTC+8` after ——
/// up 1 「 1 empty + 19 」 and not UTC+8。
pub fn parse_quota_reset_at(text: &str) -> i64 {
    parse_quota_reset_at_at(text, crate::server::logging::now_ms())
}

/// [`parse_quota_reset_at`] ** can 「 in 」**。
///
/// has 「 / HH:MM」 that 1 need need is （ `scan_same_day_window`），
/// now —— up is not now that ， use this 。
pub fn parse_quota_reset_at_at(text: &str, now_ms: i64) -> i64 {
    let bytes = text.as_bytes();
    // empty：`\s` in Node then empty//（ ASCII empty can ）
    let space = |index: usize| -> bool {
        bytes
            .get(index)
            .map(|byte| byte.is_ascii_whitespace())
            .unwrap_or(false)
    };
    /// `width` （ not or return None）
    fn number(bytes: &[u8], start: usize, width: usize) -> Option<u32> {
        let mut value = 0u32;
        for offset in 0..width {
            let byte = bytes.get(start + offset)?;
            if !byte.is_ascii_digit() {
                return None;
            }
            value = value * 10 + (byte - b'0') as u32;
        }
        Some(value)
    }

    let mut index = 0usize;
    while index < bytes.len() {
        // ① dddd-dd-dd
        let date = (
            number(bytes, index, 4),
            bytes.get(index + 4),
            number(bytes, index + 5, 2),
            bytes.get(index + 7),
            number(bytes, index + 8, 2),
        );
        if let (Some(year), Some(b'-'), Some(month), Some(b'-'), Some(day)) = date {
            // ② \s+ 
            let mut cursor = index + 10;
            let space_start = cursor;
            while space(cursor) {
                cursor += 1;
            }
            if cursor > space_start {
                // ③ dd:dd:dd
                let time = (
                    number(bytes, cursor, 2),
                    bytes.get(cursor + 2),
                    number(bytes, cursor + 3, 2),
                    bytes.get(cursor + 5),
                    number(bytes, cursor + 6, 2),
                );
                if let (Some(hour), Some(b':'), Some(minute), Some(b':'), Some(second)) = time {
                    // ④ \s*UTC+8 —— must in ，otherwise not is when （ and Node then 1 ）
                    let mut suffix = cursor + 8;
                    while space(suffix) {
                        suffix += 1;
                    }
                    if bytes[suffix..].starts_with(b"UTC+8") {
                        if let Some(millis) = to_utc_plus_8_millis(year, month, day, hour, minute, second) {
                            return millis;
                        }
                    }
                }
            }
        }
        index += 1;
    }
    // ② 「 / HH:MM」—— 1 ： no has 、 no has seconds、 no has 
    // `UTC+8` after ， up not 。AtomCode quota is this ：
    // 「…current in 18:18 and ， after 。」
    // （2026-10-18 in dev up ）。⚠️ this 1 before ， this is 0，
    // is to 「10 minutes」 not is 。
    scan_same_day_window(text, now_ms).unwrap_or(0)
}

/// 「 / HH:MM」→ when **** that when ms when 。
///
/// （ when ， as no to and seconds）：
/// · `` / `` after **16 ** `H[H]:MM`；
/// · （`25:00` ） and 「minutes after 」 not ；
/// · ** already ** ⇒ `None`（ this 。return 1 when 
/// etc use 1 ， not ）。
fn scan_same_day_window(text: &str, now_ms: i64) -> Option<i64> {
    let date = crate::server::core::beijing::date_time_of_ms(now_ms)?.date_naive();
    for keyword in ["", ""] {
        let Some(at) = text.find(keyword) else {
            continue;
        };
        // by ****：`&rest[..16]` in in will panic
        // （release profile is panic=abort， this error will ）
        let rest: String = text[at + keyword.len()..].chars().take(16).collect();
        let Some((hour, minute)) = first_clock(&rest) else {
            continue;
        };
        let Some(naive) = date.and_hms_opt(hour, minute, 0) else {
            continue;
        };
        let instant = crate::server::core::beijing::offset()
            .from_local_datetime(&naive)
            .single()?;
        let millis = instant.timestamp_millis();
        if millis > now_ms {
            return Some(millis);
        }
    }
    None
}

/// 1 `H[H]:MM`（ when 1–2 、minutesmust 2 ， and minutes after not ）。
fn first_clock(text: &str) -> Option<(u32, u32)> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        let mut cursor = index;
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            cursor += 1;
        }
        let hour_digits = cursor - index;
        // is ASCII ⇒ by in up 
        let hour = text[index..cursor].parse::<u32>().ok();
        let minute = (bytes.get(cursor) == Some(&b':')
            && bytes.get(cursor + 1).is_some_and(u8::is_ascii_digit)
            && bytes.get(cursor + 2).is_some_and(u8::is_ascii_digit))
        .then(|| u32::from(bytes[cursor + 1] - b'0') * 10 + u32::from(bytes[cursor + 2] - b'0'));
        let trailing_digit = bytes.get(cursor + 3).is_some_and(u8::is_ascii_digit);
        if let (Some(hour), Some(minute)) = (hour, minute) {
            if (1..=2).contains(&hour_digits) && !trailing_digit && hour < 24 && minute < 60 {
                return Some((hour, minute));
            }
        }
        index = cursor;
    }
    None
}

/// `YYYY-MM-DD HH:MM:SS`（UTC+8）→ ms when ； when None。
///
/// not regex： then 「failure 0」 。
fn to_utc_plus_8_millis(
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<i64> {
    // UTC+8 ： up is this ， not when 
    let offset = chrono::FixedOffset::east_opt(8 * 3600)?;
    let naive = chrono::NaiveDate::from_ymd_opt(year as i32, month, day)?
        .and_hms_opt(hour, minute, second)?;
    Some(offset.from_local_datetime(&naive).single()?.timestamp_millis())
}

/// when （ for Node `toLocaleString('zh-CN', { hour12:false })`）。
/// 1 by UTC+8 ： up when is UTC+8 ，
/// use when will use for not up up to 。
pub(crate) fn format_reset_at_text(reset_at: i64) -> String {
    // +8 1 can success； 1 failure to empty（ this is ，
    // not can as 1 failure request —— release is panic=abort）
    let Some(offset) = chrono::FixedOffset::east_opt(8 * 3600) else {
        return String::new();
    };
    match chrono::DateTime::from_timestamp_millis(reset_at) {
        Some(utc) => utc
            .with_timezone(&offset)
            .format("%Y/%m/%d %H:%M:%S")
            .to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod panel_401_cookie_tests {
    //! 401 cookie ** in will no when **：
    //!
    //! - no that cookie is 「」 —— that cookie 
    //! ，server new 1 invalid（： 1 for not up 
    //! 51 ）。
    //! - ， is 1 to ， and ：access has 2 when 、 will 
    //! cookie when 30 refresh， 1 etc 2 when 
    //! 1 （client to 401 down 1 is this cookie new ）。
    //!
    //! to has 1 use 。 1 ： and in will 1 
    //! （ `access::session_valid` ）， etc 1 to not 。

    use super::*;
    use crate::server::access;
    use axum::http::{HeaderMap, HeaderValue};

    /// `Cookie` request。
    fn with_cookie(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_str(value).unwrap(),
        );
        headers
    }

    fn set_cookies(response: &Response) -> Vec<String> {
        response
            .headers()
            .get_all(axum::http::header::SET_COOKIE)
            .iter()
            .map(|value| value.to_str().unwrap_or_default().to_string())
            .collect()
    }

    #[test]
    fn the_panel_401_clears_the_stale_access_cookie() {
        // no has refresh （ / ）： this 
        let response = panel_login_required_response(&HeaderMap::new());
        let cookies = set_cookies(&response);
        assert_eq!(cookies.len(), 1, "401 this 1 （ will in ）");
        assert!(cookies[0].starts_with(&format!("{}=;", access::ACCESS_COOKIE)));
        assert!(cookies[0].contains("Path=/"), " need in access Path up ");
        assert!(cookies[0].contains("Max-Age=0"));
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "error not can as 1 "
        );
    }

    /// ** for （ this is ）**：access refresh when ，401 not 
    /// cookie —— 1 not 。
    ///
    /// after is can ： cookie need and Path when in ， this cookie
    /// when refresh， after client `/api/panel/refresh` no has 
    /// refresh （ cookie down localStorage is empty ， not ），
    /// is 2 when （`ACCESS_TTL`） 1 。
    #[test]
    fn the_panel_401_leaves_a_live_refresh_alone() {
        let session = access::IssuedSession::new_session();
        let headers = with_cookie(&format!(
            "{}=not-a-live-access.{}",
            access::ACCESS_COOKIE, session.refresh_token
        ));
        // before ： this refresh （otherwise down that in empty）
        assert!(
            access::has_live_refresh(&headers),
            " refresh as "
        );
        let response = panel_login_required_response(&headers);
        assert!(
            set_cookies(&response).is_empty(),
            "refresh when 401 not cookie —— "
        );
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod quota_reset_text_tests {
    //! up quota「 when 」 ****。
    //!
    //! 1 is when （`… 2026-09-11 19:43:46 UTC+8 `，Node then that 1 ），
    //! 1 has when （`… current in 18:18 and …`，
    //! AtomCode ）。 in before will 0， is 
    //! to 「10 minutes」 not is 。

    use super::*;

    /// when ms when （ use ；seconds 0）。
    fn beijing(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
        crate::server::core::beijing::offset()
            .with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .expect(" when ")
            .timestamp_millis()
    }

    #[test]
    fn the_same_day_window_form_is_readable() {
        // 2026-10-18 in dev up to 
        let text = "「glm5.3-flash」 in current CodingPlan usage already limit。\
                    current in 18:18 and ， after 。";
        let now = beijing(2026, 10, 18, 12, 0);
        assert_eq!(
            beijing(2026, 10, 18, 18, 18),
            parse_quota_reset_at_at(text, now),
            "「 18:18」 when 18:18"
        );
        // 「」 is 1 1 
        assert_eq!(
            beijing(2026, 10, 18, 18, 18),
            parse_quota_reset_at_at(" in 18:18 and ", now)
        );
    }

    #[test]
    fn a_same_day_window_already_in_the_past_is_not_invented() {
        // is old （）：return 0 use use ，
        // not is 1 when —— that etc no 。
        let now = beijing(2026, 10, 18, 12, 0);
        assert_eq!(0, parse_quota_reset_at_at(" in 09:05 and ", now));
    }

    #[test]
    fn the_full_utc_plus_8_stamp_still_parses() {
        // ：add not can that 1 
        let text = " usage already limit， will in 2026-09-11 19:43:46 UTC+8 ";
        assert_eq!(
            beijing(2026, 9, 11, 19, 43) + 46_000,
            parse_quota_reset_at(text)
        );
        assert_eq!(0, parse_quota_reset_at(" no has when "));
    }

    #[test]
    fn malformed_clocks_are_not_read() {
        let now = beijing(2026, 10, 18, 12, 0);
        for text in [
            " 25:00 ", // when 
            " 18:6 ", // minutes has 1 
            " 1818 ", // no has 
            " 18:180 ", // minutes after 
        ] {
            assert_eq!(0, parse_quota_reset_at_at(text, now), " not this ：{text}");
        }
    }
}
