//! ：** up ** use then （provider use ）。
//!
//! workbuddy2api `contentBlockedRule`（`internal/upstream/client.go`）：
//! HTTP 400 + down 1 = 。
//!
//! ```text
//!   blocked by security policy
//!   unapproved channel
//!   illegal api invocation
//! ```
//!
//! ── Why this is 「」 not is ──────────────────────────
//! up by ****（ not is ），client system 
//! in 。 when ：、 no 、session no —— 
//! is 1 （ 1 body will ）。
//! [`UpstreamErrorClass::ContentBlocked`] in ** not **（none、none、
//! not error）， is 「 to in after 1 」。
//!
//! ── Why in providers not is ───────────────────────
//! 「」 is ** up **（ and 11-128、6004 ）， by 
//! ；， not 。 then use ，
//! in providers down ， by in after use 。
//!
//! ── and workbuddy 11-128 is ─────────────────────────
//! `11-128` is this ****（ `Illegal API invocation from an
//! unapproved channel`）。（ / ） to 1 ，
//! and ：「 up no 」，「 up no 」。
//!
//! ── and （ and 1 ）───────────────────────
//! ** not **（Go `matchLower`： `strings.ToLower(body)` 
//! ）； need `>= 400`（ this 1 in `status >= 400`
//! ，401 / 429 already in before ）。

use serde_json::Value;

use super::adapter::UpstreamErrorClass;

/// （； before up ）
const PATTERNS: &[&str] = &[
    "blocked by security policy",
    "unapproved channel",
    "illegal api invocation",
];

/// client can ****： in ` up return {status}: { up }` after 
/// （ for `hint.go` `ErrContentBlocked` that 
/// `request content was rejected by content policy; adjust the prompt and retry`）。
///
/// and 1 use ： 1 up ， use not 「 this is is 」，
/// not down 1 this 。 this to ** can ** ：
/// - system → to 「」（ `core::prompt`）；
/// - for → after 。
///
/// ── up not 「 already 」──────────────────────────
/// 1 message will in down to client：`custom` ** not **
/// （system already by ， `core::prompt` `degradable`），
/// request no has 「 1 」 this 。「 already in 1 」
/// in this down is —— use can ， not 
/// （ in request ， that is request）。
pub const CONTENT_BLOCK_HINT: &str = "；内容被上游内容策略拒绝（这类拦截按逐字匹配，不是账号问题）：\
若为客户端 system 模板误报，可在设置页「通用 → 系统提示词」切到「替换」模式；\
若为对话内容本身触发审核，请调整后重试";

/// is in 。
pub fn matched(status: u16, error_body: &Value) -> bool {
    if status < 400 {
        return false;
    }
    let Some(message) = error_body.get("message").and_then(Value::as_str) else {
        return false;
    };
    let lowered = message.to_lowercase();
    PATTERNS.iter().any(|pattern| lowered.contains(pattern))
}

/// ： in → [`UpstreamErrorClass::ContentBlocked`]
/// （ and to up [`CONTENT_BLOCK_HINT`]），otherwise → [`UpstreamErrorClass::Fatal`]。
///
/// `message` by use to （ error not ）， and 。
pub fn classify_or_fatal(
    status: u16,
    error_body: &Value,
    message: String,
    upstream_code: Option<i64>,
) -> UpstreamErrorClass {
    if matched(status, error_body) {
        return UpstreamErrorClass::ContentBlocked {
            status,
            message: format!("{message}{CONTENT_BLOCK_HINT}"),
            upstream_code,
        };
    }
    UpstreamErrorClass::Fatal {
        status,
        message,
        upstream_code,
    }
}
