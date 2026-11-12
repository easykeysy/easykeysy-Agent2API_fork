//! JS （ use ）。
//!
//! Node JS for ， JS 
//! （`x || y`、`x ?? y`、`Number(x)`、`String(x)`、`JSON.stringify` …）。
//! this 1 is ** in Rust that **， as and Node 1 ：
//! can （ not for ）， not need or —— use 
//! （ `data` ， or file） is supports use 。
//!
//! from store.rs （file）。 use ：store.rs / store_view.rs
//! （ and ）、store_crud.rs（）、account_transfer.rs
//! （ 1 ）。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::json_number;

/// Node `jwt.slice(-4)`： 4 （ not 4 then ）。
/// 1 use （tokenTail when by accessToken ）。
pub(crate) fn token_tail_of(token: &str) -> String {
    token_tail(token)
}

/// Node `text.slice(0, max)`： by （ 1 use ）
pub(crate) fn truncate_text(text: &str, max: usize) -> String {
    truncate_chars(text, max)
}

/// Node `jwt.slice(-4)`： 4 （ not 4 then ）
pub(super) fn token_tail(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    let start = chars.len().saturating_sub(4);
    chars[start..].iter().collect()
}

/// Node `text.slice(0, max)`： by 
pub(super) fn truncate_chars(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// JS （used for `x || default` and `Boolean(x)` ）：
/// null / false / 0 / "" as ， its （empty、empty for ） as 。
///
/// for （core::routing etc ） can ： need 1 
/// JS ， 1 not use 1 。
pub(crate) fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JS `String(x)`（ for use ）：、/ by 、
/// its （/ for ） use JSON —— this in not will ，
/// not can panic or 。
pub(super) fn js_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        other => other.to_string(),
    }
}

/// JS `x || fallback`：（null/false/0/""） fallback
pub(super) fn value_or(value: Option<&Value>, fallback: Value) -> Value {
    match value {
        Some(inner) if js_truthy(inner) => inner.clone(),
        _ => fallback,
    }
}

/// JS `x ?? fallback`： has null / fallback（empty is valid）
pub(super) fn value_or_nullish(value: Option<&Value>, fallback: Value) -> Value {
    match value {
        Some(Value::Null) | None => fallback,
        Some(inner) => inner.clone(),
    }
}

/// set → empty（ for `existing?.nickname` this 「 can can is any」 ）。
/// return None 「set」， use down 1 。
pub(super) fn optional_text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        Some(other) if js_truthy(other) => Some(js_string(other)),
        _ => None,
    }
}

/// for Node `Number(x) || fallback`（NaN / 0 / fallback）。
///
/// already is JSON when **return**（， `json_number` ）；
/// then by JS `Number()` after or 。
pub(super) fn number_or(value: Option<&Value>, fallback: Value) -> Value {
    match value {
        Some(Value::Number(number)) => {
            let parsed = number.as_f64().unwrap_or(0.0);
            if parsed.is_finite() && parsed != 0.0 {
                value.cloned().unwrap_or(fallback)
            } else {
                fallback
            }
        }
        Some(Value::String(text)) => match text.trim().parse::<f64>() {
            Ok(number) if number.is_finite() && number != 0.0 => json_number(number),
            _ => fallback,
        },
        _ => fallback,
    }
}

/// `Bearer ` before （ not ， 1 empty）， not when return
/// —— and Node `String.replace(/^Bearer\s+/i, '')` 1 。
pub(super) fn strip_bearer_prefix(value: &str) -> String {
    const PREFIX: &str = "bearer";
    if value.len() < PREFIX.len() {
        return value.to_string();
    }
    let (head, rest) = value.split_at(PREFIX.len());
    if !head.eq_ignore_ascii_case(PREFIX) {
        return value.to_string();
    }
    if rest.trim_start().len() == rest.len() {
        return value.to_string();
    }
    rest.trim_start().to_string()
}

/// for Node `pickToken`： by 1 empty， Bearer before 
pub(super) fn pick_token(payload: &Map<String, Value>, keys: &[&str]) -> String {
    for key in keys {
        if let Some(Value::String(text)) = payload.get(*key) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return strip_bearer_prefix(trimmed);
            }
        }
    }
    String::new()
}

/// `maxConcurrent`（ and limit）： no has this = set
/// = ** not **， 1 0。
///
/// （ `routing::max_concurrent_of` and before ） by 「0 = not 」
/// ， and 0 in up etc ； 1 has ，
/// before not 「 not in 」。
///
/// ** this 1 **， and ** not this **： 1 by 
/// `store_view::public_account` in by provider after 1 （ and 
/// `hasCredentials` / `checkinAt` ）。 from before is 
/// `insert`， is 「」 that （monkeycode / antigravity / officeace）
/// —— in 0 = not ，** and limit**。
pub(crate) fn max_concurrent_public(value: Option<&Value>) -> u64 {
    value.and_then(Value::as_u64).unwrap_or(0)
}

/// `maxConcurrent`： in [`max_concurrent_public`] up up 
/// 「default and 」 。
///
/// has CodeArts 1 has default：** no has 「 not 」 this **， up 0（ from 
/// set） by up 「 by default and 」（ `SessionGate::limit_for` is 
/// 1 ）， need 3 not is 0 —— otherwise to 0 etc not ，
/// `routing::at_concurrency_limit` not will in when this ，request、
/// 4 up **409**， 409 is 、 not ，client to 1 
/// 「 has empty」 。
///
/// ── this Why in ──────────────────────────────
/// is **parameter**（「 this can not can 」）， not is up ：
/// in by 1 ， in this by 1 ，must 1 。
///
/// ── 0 default， not （ `max(config, 3)` ）──────────
/// ** 1 / 2** 1 3： is 1、 by 1 ， and 
/// to 3。 as 「 1、 after is 3」； as in 1 
/// in 、 409 ， has path 409 not —— 1 3 
/// error。 need by limit [`MAX_SESSION_LIMIT`] ： 0–999
/// （all use 1 ）， to 64， if 999， will as has 
/// 935 empty。
///
/// 0 is 0（ not ）—— in this 1 ， 
/// not 1 。
/// CodeArts default and （ up for to 3 will ； 1 ）。
const CODEARTS_DEFAULT_SESSION_LIMIT: u32 = 3;
/// CodeArts and limit（ to this ； 0–999， will 
/// ）。
const CODEARTS_MAX_SESSION_LIMIT: u32 = 64;

pub(crate) fn max_concurrent_public_for(provider: &str, value: Option<&Value>) -> u64 {
    let configured = max_concurrent_public(value);
    if provider != super::codearts_accounts::CODEARTS_PROVIDER_ID {
        return configured;
    }
    if configured == 0 {
        u64::from(CODEARTS_DEFAULT_SESSION_LIMIT)
    } else {
        configured.min(u64::from(CODEARTS_MAX_SESSION_LIMIT))
    }
}

/// for ； for （） 1 empty —— for `payload.auth` / `payload.account`
/// this ， and Node undefined etc 。
pub(super) fn object_or_empty(value: Option<&Value>) -> Map<String, Value> {
    match value {
        Some(Value::Object(map)) => map.clone(),
        _ => Map::new(),
    }
}
