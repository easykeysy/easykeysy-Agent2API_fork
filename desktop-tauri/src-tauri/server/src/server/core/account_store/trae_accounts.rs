//! Trae（ AI IDE） ****：。
//!
//! already to （`agent2api-providers`， not ）：
//! （ + PKCE）、、（ will refreshToken）、directory、 will 、
//! and to in that 。 this 1 **** ： to （ no has 
//! `pluginSecret` that ） and 。
//!
//! ── Why need this 1 ────────────────────────────────────────
//! up Trae in is （`accessToken` / `refreshToken` /
//! `deviceId` … in ）。`public_account` must has ，otherwise will 
//! WorkBuddy ： 1 `uid` / `edition` and none empty，
//! 「 new Token」 by `hasRefreshToken` is **** in 
//! —— that not is ， is 。
//!
//! new not this （ has `pluginSecret`，`account_view` by to ）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store::AccountStore;

/// （ and `/api/accounts`）。
///
/// up ， not ： to is ****（ in when will 
/// ms， can can is seconds —— by ms， up ）。
/// not has this JWT and 1 ， that to 。
fn trae_public_account(record: &StoredAccount) -> Value {
    let access_token = record.access_token();
    let refresh_token = record.refresh_token();
    let mut public = Map::new();
    for key in [
        "id",
        "provider",
        "name",
        "uid",
        "nickname",
        "enterpriseId",
        "source",
        "tokenTail",
        "expiresAt",
        "variant",
    ] {
        public.insert(key.to_string(), record.get(key).cloned().unwrap_or(Value::Null));
    }
    public.insert(
        "expiresAtMs".to_string(),
        Value::from(record.expires_at().map(|value| value as i64).unwrap_or(0)),
    );
    public.insert(
        "hasRefreshToken".to_string(),
        Value::Bool(!refresh_token.trim().is_empty()),
    );
    public.insert("priority".to_string(), Value::from(record.priority()));
    public.insert("enabled".to_string(), Value::Bool(record.enabled()));
    public.insert("addedAt".to_string(), Value::from(record.added_at()));
    public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
    public.insert(
        "proxy".to_string(),
        crate::server::core::proxies::describe_account_proxy(Some(&record.proxy())),
    );
    public.insert(
        "rateLimits".to_string(),
        record.get("rateLimits").cloned().unwrap_or_else(|| json!({})),
    );
    public.insert("desktop".to_string(), Value::Bool(false));
    // has accessToken 「currentavailable」； has need 1 can use 
    public.insert(
        "available".to_string(),
        Value::Bool(!access_token.trim().is_empty()),
    );
    Value::Object(public)
}

impl AccountStore {
    /// （`store_view::public_account` use ）。
    pub(crate) fn to_trae_public_account(&self, record: &StoredAccount) -> Value {
        trae_public_account(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ** not **： need need and 「 can not can 」。
    #[test]
    fn the_public_form_never_carries_tokens() {
        let record = StoredAccount::from_map(
            json!({
                "id": "trae-solo-abc",
                "provider": "trae",
                "uid": "u-1",
                "variant": "solo",
                "accessToken": "A1",
                "refreshToken": "R1",
                "expiresAt": 1_900_000_000_000_i64,
                "tokenTail": "aaaa",
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        let public = trae_public_account(&record);
        assert_eq!("u-1", public.get("uid").and_then(Value::as_str).unwrap_or(""));
        assert_eq!("solo", public.get("variant").and_then(Value::as_str).unwrap_or(""));
        assert_eq!(
            1_900_000_000_000,
            public.get("expiresAtMs").and_then(Value::as_i64).unwrap_or(0)
        );
        assert!(public.get("hasRefreshToken").and_then(Value::as_bool).unwrap_or(false));
        assert!(public.get("available").and_then(Value::as_bool).unwrap_or(false));
        assert!(public.get("accessToken").is_none(), "公开形态不许带令牌");
        assert!(public.get("refreshToken").is_none(), "公开形态不许带续期串");
    }

    /// has ：`available` as （ need 1 can use ），
    /// `hasRefreshToken` as —— 「 new Token」 by this 。
    #[test]
    fn a_refresh_only_record_is_not_available_but_is_refreshable() {
        let record = StoredAccount::from_map(
            json!({
                "id": "trae-cn-abc",
                "provider": "trae",
                "refreshToken": "R1",
            })
            .as_object()
            .cloned()
            .unwrap_or_default(),
        );
        let public = trae_public_account(&record);
        assert!(!public.get("available").and_then(Value::as_bool).unwrap_or(true));
        assert!(public.get("hasRefreshToken").and_then(Value::as_bool).unwrap_or(false));
    }
}
