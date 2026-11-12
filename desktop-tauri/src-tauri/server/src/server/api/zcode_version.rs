//! ZCode client：`GET/PUT /api/zcode/version`。
//!
//! ── this Why in （ this is use can 1 ）──────────────
//! ZCode （ when ， 2026-10 「ZCode 1024 Dev Day」）
//! by up by **client** down ：`GET billing/preview?app_version=…` in 
//! to when return `data.plans`，otherwise 1 **empty**
//! （`code: 0`， not is error —— this is 「 no has 」 ）。
//!
//! 2026-10-23 （， 1 ）：
//!
//! | app_version | `data.plans` |
//! |---|---|
//! | 3.13.0 / 3.14.0 / 3.14.3 / 3.15.0 / 3.15.1 / 3.16.0 | `[]` |
//! | 3.16.1 / 3.16.2 / 3.17.0 | `zcode-v3-start-plan-1024` |
//!
//! up ，** after when is empty**。 that default
//! is （`zcode::claim::DEFAULT_APP_VERSION`）， need 、
//! 。 this is that ： use to 「 in not to 」 when ，
//! in this 1 update can ， not etc 。
//!
//! ── config in ──────────────────────────────────────────────
//! `plugin.zcode.appVersion`（ and `settings_get("zcode", "appVersion")`
//! is 1 ， `plugin_host::plugin_settings_key`）。
//!
//! ── Why not use `settings.set` ────────────────────────
//! will set（`settings_set`）， this 1 is ** use config**：
//! 、。 will 「config」 ——
//! and no when this 1 need can can （ use config not this 1 ）。
//!
//! ── ──────────────────────────────────────────────────
//! `ZCODE_APP_VERSION` > set > default。
//! before is ： by ****， not this 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// config（ `config.json` `plugin.zcode.appVersion`）。
///
/// and `claim::SETTING_APP_VERSION` + `APP_VERSION_SETTING_OWNER`
/// path**must 1 ** —— is 「 up 、
/// not to 」， that will default。
const KEY: &str = "plugin.zcode.appVersion";

/// limit（`3.16.2` this not will ）。
const MAX_LEN: usize = 32;

/// GET /api/zcode/version —— current and 。
///
/// response（ before and ）：
/// ```json
/// { "version": "3.16.2", "envOverride": null }
/// ```
/// `version` is **currentset **（set when as empty，「 use default」）；
/// `envOverride` empty when —— 「 up 
/// not 」， not is use for 1 no 。
pub async fn get_version(State(_state): State<ServerState>) -> Response {
    ok_json(json!({
        "version": stored_version().unwrap_or_default(),
        "envOverride": env_override(),
    }))
}

/// PUT /api/zcode/version —— body `{"version": "3.16.2"}`（empty = empty）。
///
/// ： and 。** not 「 this in 」** —— up 
/// for is （ in server，client not to ）， in 1 then 
/// will in up then when 。 and must： will URL 
/// （`?app_version=…`）， `&` or empty will URL。
pub async fn put_version(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let Some(raw) = object.get("version") else {
        return errors::management_error(400, "缺少 version");
    };
    let Some(text) = raw.as_str() else {
        return errors::management_error(400, "version 必须是字符串");
    };
    let trimmed = text.trim();
    if let Err(message) = validated(trimmed) {
        return errors::management_error(400, message);
    }
    let next = if trimmed.is_empty() {
        Value::Null
    } else {
        Value::String(trimmed.to_string())
    };
    if !config::update_raw_field(KEY, next) {
        crate::server::logging::log("[Config]", "⚠️  ZCode 客户端版本号写入失败，本次运行内仍生效");
    }
    crate::server::logging::log(
        "[Config]",
        if trimmed.is_empty() {
            "ZCode 客户端版本号已清空（回落插件内置默认值）".to_string()
        } else {
            format!("ZCode 客户端版本号设为 {trimmed}（下一次探测/领取生效）")
        }
        .as_str(),
    );
    ok_json(json!({
        "version": trimmed,
        "envOverride": env_override(),
    }))
}

/// （empty is ：empty）。
///
/// `[0-9A-Za-z._-]`： `3.16.2`、`3.16.2-beta.1` this ，
/// when excluding will （`&`、`#`、empty、 in ）。
fn validated(text: &str) -> Result<(), String> {
    if text.len() > MAX_LEN {
        return Err(format!("版本号最多 {MAX_LEN} 个字符"));
    }
    if text.is_empty() {
        return Ok(());
    }
    let legal = text
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));
    if !legal {
        return Err("版本号只能包含字母、数字与 . _ -（它会拼进请求 URL）".to_string());
    }
    // must is ： up as `3.x.y`， `v3.16.2` 
    // will in when up invalid（ and that failure is empty，）。
    if !text.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return Err("版本号必须以数字开头（形如 3.16.2）".to_string());
    }
    Ok(())
}

/// config （ / / empty → `None`）。
fn stored_version() -> Option<String> {
    config::with_raw(|raw| {
        raw.get(KEY)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    })
    .flatten()
}
/// `ZCODE_APP_VERSION` （emptyreturn）。
///
/// set（module header）： this ， is as can 
/// 「 」。
fn env_override() -> Option<String> {
    std::env::var("ZCODE_APP_VERSION")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_legal_and_clears_the_override() {
        assert!(validated("").is_ok());
    }

    #[test]
    fn a_plain_semver_is_accepted() {
        for text in ["3.16.2", "3.16.2-beta.1", "3.17.0", "4.0"] {
            assert!(validated(text).is_ok(), "{text} 应当通过");
        }
    }

    #[test]
    fn characters_that_break_a_query_string_are_rejected() {
        // `&` will 、empty need 、 in up not —— in this 1 
        for text in ["3.16.2&platform=x", "3.16 2", "3.16.2版", "3.16.2\n"] {
            assert!(validated(text).is_err(), "{text:?} 不该通过");
        }
    }

    #[test]
    fn a_version_must_start_with_a_digit() {
        // `v3.16.2` will up invalid， is empty not is 
        assert!(validated("v3.16.2").is_err());
        assert!(validated("3.16.2").is_ok());
    }

    #[test]
    fn an_over_long_version_is_rejected() {
        let long = "3.".to_string() + &"1".repeat(MAX_LEN);
        assert!(validated(&long).is_err());
    }
}
