//! Cline ：`GET/PUT /api/cline/headers`。
//!
//! ── this ──────────────────────────────────────────
//! Cline up request ****。 this has ** use **
//! （`config::KEY_CLINE_UPSTREAM_HEADERS`，config）；default and 「default ∪
//! 」 and in **** —— that that ， is as need 
//! CLI not up 403，， by need 
//! not in 。
//!
//! ── ────────────────────────────────────────────────
//! GET return `{overrides, defaults, effective}`： 1 is use （
//! ， ）， and need 
//! （`admin.action` `headers.preview`， down ）—— use `defaults` 
//! 「default」 and 「 no 」，`effective` is will that 。
//! PUT `{overrides: {...}}` ****：empty by /add，
//! empty = this not ； from = default。
//! （ [`FIXED_HEADERS`]） and / 1 400 ， not 。
//!
//! ── not in when ────────────────────────────────────────
//! `defaults` / `effective` returnempty，`overrides` return（ use config not ）。
//! 「unavailable」 not is 1 default —— no has that 。

use std::collections::BTreeMap;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderName, HeaderValue};
use axum::response::Response;
use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::providers::adapter;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// limit： ，64 is to 
const MAX_ENTRIES: usize = 64;
/// limit（ and `http::HeaderName` up ）
const MAX_KEY_LEN: usize = 128;
/// limit
const MAX_VALUE_LEN: usize = 4096;

/// ****：required（`Content-Type` / `Accept`） or request
/// （`Authorization` token、`X-Task-ID` and body `session_id` ），
/// 1 400。
///
/// and default **** 1 1 for —— that is （ this by 
/// ）， this is config（ use not this can ）。must in config： 1 
/// `Authorization` will and ， `X-Task-ID` will and 
/// body `session_id` not 。
const FIXED_HEADERS: [&str; 4] = ["Authorization", "Content-Type", "Accept", "X-Task-ID"];

/// GET /api/cline/headers —— current + default + and after 
pub async fn get_cline_headers(State(state): State<ServerState>) -> Response {
    ok_json(headers_json(&state).await)
}

/// PUT /api/cline/headers —— body `{overrides: {"User-Agent": "...", ...}}`
///
/// （ not is merge： by 「 after 」，delete
/// is from ）。 [`validated_overrides`]：、、、
/// and ； 400， not 。
pub async fn put_cline_headers(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let Some(raw_overrides) = object.get("overrides") else {
        return errors::management_error(400, "缺少 overrides");
    };
    let Some(entries) = raw_overrides.as_object() else {
        return errors::management_error(400, "overrides 必须是对象（头名 → 头值）");
    };
    let overrides = match validated_overrides(entries) {
        Ok(overrides) => overrides,
        Err(message) => return errors::management_error(400, message),
    };
    if !config::set_cline_upstream_headers(overrides) {
        logging::log("[Config]", "⚠️  Cline 伪装头写入失败，本次运行内仍生效");
    }
    logging::log("[Config]", "Cline 伪装头覆盖表已更新（下一个 Cline 请求生效）");
    ok_json(headers_json(&state).await)
}

/// and 1 （PUT ， JSON for ）。
///
/// 1 `Err`（400 ）， not 。**must in this **：
/// not is HTTP token（empty、、 ASCII） or when ，
/// `core::upstream::request::send_chat_request` `builder.header(...)` will 
/// not to `HeaderName`/`HeaderValue`，reqwest this requestfailure
/// —— is ** 1 ** Cline request not ，client to is 「 up requestfailure
/// （…）」， and Reason（config has ）none， and config already 、
/// in 。 use `HeaderName`/`HeaderValue` `try_from`： and reqwest
/// request when use is ** 1 then **， not need need 1 。
fn validated_overrides(entries: &Map<String, Value>) -> Result<BTreeMap<String, String>, String> {
    if entries.len() > MAX_ENTRIES {
        return Err(format!("overrides 最多 {} 项", MAX_ENTRIES));
    }
    let mut overrides = BTreeMap::new();
    for (key, value) in entries {
        let key = key.trim();
        if key.is_empty() || key.len() > MAX_KEY_LEN {
            return Err(format!("头名非法或过长: {key:?}"));
        }
        if HeaderName::try_from(key).is_err() {
            return Err(format!(
                "头名含非法字符（只能是字母、数字与 !#$%&'*+-.^_`|~）: {key:?}"
            ));
        }
        if FIXED_HEADERS.iter().any(|name| name.eq_ignore_ascii_case(key)) {
            return Err(format!("{key} 是网关固定头，不能覆盖"));
        }
        let Some(text) = value.as_str() else {
            return Err(format!("头 {key} 的值必须是字符串"));
        };
        if text.len() > MAX_VALUE_LEN {
            return Err(format!("头 {key} 的值过长"));
        }
        if HeaderValue::try_from(text).is_err() {
            return Err(format!("头 {key} 的值含非法字符（不能有换行等控制字符）"));
        }
        overrides.insert(key.to_string(), text.to_string());
    }
    Ok(overrides)
}

/// response（GET and PUT use ， before use response new ）。
///
/// `overrides` is config，** not **；`defaults` / `effective` 
/// need （`admin.action` `headers.preview`）。 not in when that to empty ——
/// no has default this ， 1 will is when 。
///
/// `defaults` must to ： need 「 use default」 and 「 no 」
/// （ before is 1 、 after not this in ）， no has default not to this 。
async fn headers_json(state: &ServerState) -> Value {
    let overrides: BTreeMap<String, String> =
        config::cline_upstream_overrides().into_iter().collect();
    let (defaults, effective) = plugin_header_tables(state).await;
    json!({
        "overrides": overrides,
        "defaults": defaults,
        "effective": effective,
    })
}

/// need `{defaults, effective}`（ 1 ： use 1 ）。
///
/// failure / not in when returnempty： this is ****， not this as no 
/// —— `overrides` need can can （ that is config）。
async fn plugin_header_tables(state: &ServerState) -> (Map<String, Value>, Map<String, Value>) {
    let provider = crate::server::core::providers::kind_id(
        crate::server::core::providers::ProviderKind::ClineFree,
    );
    let Some(adapter) = adapter::adapter_by_id(provider) else {
        return (Map::new(), Map::new());
    };
    match adapter
        .admin_action(state.store(), "", "headers.preview", &Value::Null)
        .await
    {
        Ok(value) => (
            value
                .get("defaults")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default(),
            value
                .get("effective")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default(),
        ),
        Err(error) => {
            logging::verbose(
                "[Config]",
                &format!("读取 Cline 伪装头默认值失败（界面显示为空）：{}", error.message),
            );
            (Map::new(), Map::new())
        }
    }
}

#[cfg(test)]
mod tests {
    //! （ for 「 will to after 1 
    //! request」 path： 、requestfailure）。
    use super::*;

    fn entries(pairs: &[(&str, &str)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), Value::String((*value).to_string())))
            .collect()
    }

    #[test]
    fn a_fixed_header_cannot_be_overridden() {
        for (key, value) in [
            ("Authorization", "Bearer stolen"),
            ("x-task-id", "fixed"),
            ("Content-Type", "text/plain"),
            ("Accept", "*/*"),
        ] {
            let result = validated_overrides(&entries(&[(key, value)]));
            assert!(result.is_err(), "{key} not this ");
            assert!(result.unwrap_err().contains(""));
        }
    }

    #[test]
    fn a_header_name_with_illegal_characters_is_rejected() {
        // empty、、 ASCII not is HTTP token： down will Cline request
        // in reqwest failure（ not is this ， is http crate then ）
        for key in ["X My Header", "Accept:", "X-"] {
            let result = validated_overrides(&entries(&[(key, "v")]));
            assert!(result.is_err(), "{key} not this ");
            assert!(result.unwrap_err().contains(""));
        }
    }

    #[test]
    fn a_header_value_with_a_control_character_is_rejected() {
        let result = validated_overrides(&entries(&[("X-Trace", "abc\ndef")]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(""));
    }

    #[test]
    fn a_non_string_value_is_rejected() {
        let raw = json!({ "X-Trace": 1 });
        let result = validated_overrides(raw.as_object().unwrap_or(&Map::new()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must is "));
    }

    #[test]
    fn a_valid_table_is_normalized_and_kept() {
        let overrides = validated_overrides(&entries(&[
            (" X-Title ", "My Cline"),
            ("X-PLATFORM", ""),
            ("X-Custom-Trace", "abc"),
        ]))
        .expect("");
        // empty；empty（= not this ） and 
        assert_eq!(overrides.get("X-Title").map(String::as_str), Some("My Cline"));
        assert_eq!(overrides.get("X-PLATFORM").map(String::as_str), Some(""));
        assert_eq!(
            overrides.get("X-Custom-Trace").map(String::as_str),
            Some("abc")
        );
    }
}
