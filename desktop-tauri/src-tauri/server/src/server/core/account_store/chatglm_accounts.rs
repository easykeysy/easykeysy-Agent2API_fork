//! （`chatglm.cn`） ****： + 。
//!
//! already to （`agent2api-providers`， not ）： /
//! / 、、directory、、、 to 、 in in that 。
//! this **must in ** ：
//!
//! ```text
//! 1. `guest_of_record` —— `isGuest` and to 1 
//! 。（ no has `pluginSecret`） no has 
//! this ，「 not in to 」 is 
//! （`billing::checkin`）。
//! 2. `migrate_nested_tokens` —— `auth`， when to 。
//! **** is （ and 
//! 1 ）， not 。
//! ```
//!
//! new by （ in `pluginSecret` ），`isGuest` and by 
//! `account_view` —— file for not and ， not this and ： not 
//! secret ， by that will to 「 1 false」。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic， not 。

use serde_json::Value;

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store_util::token_tail_of;
use crate::server::core::providers::{kind_id, ProviderKind};
use crate::server::logging;

fn chatglm_id() -> &'static str {
    kind_id(ProviderKind::Chatglm)
}

/// is not is ****（ `isGuest` and to 1 ）。
///
/// ── Why not use that 1 ────────────────────────────────────
/// `isGuest` in `pluginSecret` ， not to ；`public_account` will 
/// `account_view` and ， `isGuest` by to 。
/// ****（ no has `pluginSecret`）。
///
/// ── （）────────────────────────────────────
/// to is 「** is excluding**」，：
/// 1. `isGuest`（ when and ）；
/// 2. when JWT **** `is_guest: true`。
/// not 1 to `false`： in to ，「」 this 
/// will 。
pub fn guest_of_record(record: &StoredAccount) -> bool {
    if let Some(flag) = record.get("isGuest").and_then(Value::as_bool) {
        return flag;
    }
    // （ / ）： JWT 。 by ****
    // in ，`auth.accessToken` is will ，。
    let token = record
        .get("accessToken")
        .or_else(|| record.get("auth").and_then(|auth| auth.get("accessToken")))
        .and_then(Value::as_str)
        .unwrap_or("");
    guest_claim(token).unwrap_or(false)
}

/// JWT `is_guest` （ not ，； not return None）。
fn guest_claim(token: &str) -> Option<bool> {
    let payload = token.split('.').nth(1)?;
    let bytes = base64_url_decode(payload)?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value.get("is_guest").and_then(Value::as_bool)
}

/// base64url（none） —— JWT 。
fn base64_url_decode(text: &str) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::with_capacity(text.len());
    let mut accumulator: u32 = 0;
    let mut bits: u32 = 0;
    for byte in text.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' | b'+' => 62,
            b'_' | b'/' => 63,
            b'=' => continue,
            _ => return None,
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buffer.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    Some(buffer)
}

/// ： `auth.accessToken` to ****
/// （ etc ， can use ）。 `accounts`，return 。
///
/// ── is （ when down ）──────────────────────
/// `auth`， `has_credentials()`（ / /
/// `hasCredentials` use ） `accessToken`。 after is 
/// ****：success、directory to ， not to this 1 、
/// 「 no has available」、 no has available。
///
/// ── Why this ──────────────────────────────────────
/// `auth` `accessToken` / `refreshToken`， its 
/// not this none in has up ：「、 has 」 that 。
///
/// ── when `auth` ──────────────────────────────────────────
/// already in 、 and no has when this empty； has 
/// old when —— that down can can not 1 ， `auth` already 
/// no has ， is 1 「 is 」 。 when 
/// `auth` ： that file not ， etc 。
///
/// etc ： 1 after has ， 1 return 0、 not 。
pub(crate) fn migrate_nested_tokens(accounts: &mut [StoredAccount]) -> usize {
    let mut healed = 0usize;
    for record in accounts.iter_mut() {
        if record.provider() != chatglm_id() {
            continue;
        }
        // need ** has **：`record.get` is use ，
        // and after `record.set` / `record.remove`（ can use ） not can when 
        // （ and `clear_rate_limit` / `migrate_rate_limit_keys` 1 ）。
        let Some(nested) = record.get("auth").and_then(Value::as_object).cloned() else {
            continue;
        };
        // has no has （ this ）
        let nested_token = |key: &str| -> Option<String> {
            nested
                .get(key)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        };
        let nested_access = nested_token("accessToken");
        let nested_refresh = nested_token("refreshToken");
        // already has not ： already by new ， that old can 
        let top_access = record
            .get("accessToken")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        let top_refresh = record
            .get("refreshToken")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        let mut changed = false;
        if top_access.is_empty() {
            if let Some(token) = nested_access.as_ref() {
                record.set("accessToken", Value::String(token.clone()));
                changed = true;
            }
        }
        if top_refresh.is_empty() {
            if let Some(token) = nested_refresh.as_ref() {
                record.set("refreshToken", Value::String(token.clone()));
                changed = true;
            }
        }
        // empty：`auth` this no 
        let only_tokens = nested
            .keys()
            .all(|key| key == "accessToken" || key == "refreshToken");
        if only_tokens {
            record.remove("auth");
            changed = true;
        }
        if !changed {
            continue;
        }
        // new （ is from ； 1 ， is ）
        let tail_source: &str = if !top_access.is_empty() {
            &top_access
        } else {
            nested_access
                .as_deref()
                .or(nested_refresh.as_deref())
                .unwrap_or("")
        };
        if !tail_source.is_empty() {
            record.set("tokenTail", Value::String(token_tail_of(tail_source)));
        }
        record.set_updated_at(logging::now_ms());
        healed += 1;
    }
    healed
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// **must in **：`has_credentials()`（ / / 
    /// `hasCredentials` 1 ） `accessToken`。
    ///
    /// ── this is ──────────────────────────
    /// `auth`（ ）， after is 
    /// `has_credentials()` ⇒ ：success、directory
    /// to ， not to this 1 、「 no has available」、
    /// 「 no has available」。 and `state.rs::has_officeace_key` OfficeAce
    /// that —— before this 。
    #[test]
    fn tokens_must_sit_on_top_level_for_has_credentials() {
        let record = StoredAccount::from_map(
            json!({
                "id": "6ad26bd46976b1c1ab1c2cb2",
                "provider": "chatglm",
                "accessToken": "aaaaaaaaaaaaaaaaaaaa",
                "refreshToken": "bbbbbbbbbbbbbbbbbbbb",
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        assert!(
            record.has_credentials(),
            "顶层 accessToken 必须被 has_credentials 认到（否则整家被静默过滤）"
        );
        assert_eq!(record.access_token(), "aaaaaaaaaaaaaaaaaaaa");
    }

    /// `auth` ** not ** `has_credentials()` —— this is 
    /// that Reason（ up ， `migrate_nested_tokens`）。
    #[test]
    fn nested_auth_is_not_recognized_by_has_credentials() {
        let record = StoredAccount::from_map(
            json!({
                "id": "x",
                "provider": "chatglm",
                "auth": { "accessToken": "aaaaaaaaaaaaaaaaaaaa" },
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        assert!(
            !record.has_credentials(),
            "嵌套 auth 不被认（写入侧必须用顶层键）"
        );
    }

    /// ： and JWT 。
    #[test]
    fn guest_flag_reads_jwt_from_both_shapes() {
        // {"is_guest":true} base64url（none）
        let guest = "eyJpc19ndWVzdCI6dHJ1ZX0";
        let top = StoredAccount::from_map(
            json!({ "id": "g", "provider": "chatglm", "accessToken": format!("h.{guest}.s") })
                .as_object()
                .cloned()
                .unwrap_or_default(),
        );
        assert!(guest_of_record(&top), "顶层令牌里的访客声明应当认到");

        let nested = StoredAccount::from_map(
            json!({
                "id": "g2",
                "provider": "chatglm",
                "auth": { "accessToken": format!("h.{guest}.s") },
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        assert!(
            guest_of_record(&nested),
            "历史嵌套形态也要认（免得访客混进签到范围）"
        );

        // not 1 false： not 「excluding」
        let none = StoredAccount::from_map(
            json!({ "id": "n", "provider": "chatglm", "accessToken": "not-a-jwt" })
                .as_object()
                .cloned()
                .unwrap_or_default(),
        );
        assert!(!guest_of_record(&none), "读不出声明时不能当访客");
    }

    /// ** **： to after `has_credentials()` ，
    /// will new in and 。
    #[test]
    fn migration_promotes_nested_tokens_to_top_level() {
        let mut accounts = vec![StoredAccount::from_map(
            json!({
                "id": "6ad26bd46976b1c1ab1c2cb2",
                "provider": "chatglm",
                "auth": {
                    "accessToken": "nested-access-token",
                    "refreshToken": "nested-refresh-token",
                },
                "deviceId": "c9074ab1-208b-4255-8f71-09cc2ce32278",
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )];
        // before ： not （ this is 「」 that ）
        assert!(!accounts[0].has_credentials());
        let healed = migrate_nested_tokens(&mut accounts);
        assert_eq!(healed, 1, "应当修复 1 条");
        // after ： has 、、empty
        assert!(accounts[0].has_credentials());
        assert_eq!(accounts[0].access_token(), "nested-access-token");
        assert_eq!(accounts[0].refresh_token(), "nested-refresh-token");
        assert!(
            accounts[0].get("auth").is_none(),
            "只装令牌的空壳应当清掉（留着就有两份可能不一致的权威）"
        );
        // etc ： 1 none、 not 
        assert_eq!(migrate_nested_tokens(&mut accounts), 0);
    }

    /// ** not **： is provider， `auth` 
    /// （ that is will ， and this not is 1 ）。
    #[test]
    fn migration_ignores_other_providers() {
        let mut accounts = vec![StoredAccount::from_map(
            json!({
                "id": "user-abc",
                "provider": "workbuddy",
                "auth": { "accessToken": "someone-elses-token" },
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )];
        assert_eq!(migrate_nested_tokens(&mut accounts), 0);
        assert!(
            accounts[0].get("auth").is_some(),
            "别家的 auth 不该被本家迁移动到"
        );
    }

    /// already has when not （ already is new ， that old can ），
    /// empty will 。
    #[test]
    fn migration_keeps_existing_top_level_value() {
        let mut accounts = vec![StoredAccount::from_map(
            json!({
                "id": "x",
                "provider": "chatglm",
                "accessToken": "fresh-top-level-token",
                "auth": { "accessToken": "stale-nested-token" },
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )];
        assert_eq!(migrate_nested_tokens(&mut accounts), 1);
        assert_eq!(
            accounts[0].access_token(),
            "fresh-top-level-token",
            "顶层已有的新令牌不能被旧壳里的覆盖"
        );
        assert!(accounts[0].get("auth").is_none());
    }

    /// `auth` when ： that not ，
    /// etc 。
    #[test]
    fn migration_keeps_auth_shell_with_extra_keys() {
        let mut accounts = vec![StoredAccount::from_map(
            json!({
                "id": "x",
                "provider": "chatglm",
                "auth": {
                    "accessToken": "nested-access-token",
                    "somethingElse": "keep-me",
                },
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        )];
        assert_eq!(migrate_nested_tokens(&mut accounts), 1);
        assert_eq!(accounts[0].access_token(), "nested-access-token");
        assert!(
            accounts[0].get("auth").is_some(),
            "带其它键的 auth 不该被删（会丢数据）"
        );
    }
}
