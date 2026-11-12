//! Cline（ ） ****：。
//!
//! already to （`agent2api-providers`， not ）：
//! 、、、directory、、、 in that 。 this ：
//!
//! ```text
//! 1. `to_cline_public_account` —— （ no has `pluginSecret`） 
//! 。 up in is ， not will WorkBuddy 
//! （ 1 none empty）。
//! 2. `cline_account_provider` / `cline_is_desktop_account` —— **
//! **： new by need this 、 is not is when 
//! （ after new ， by `api::accounts::refresh_account`）。
//! ```
//!
//! new not this （ has `pluginSecret`，`account_view` by to ）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store::AccountStore;
use crate::server::core::account_store::is_cline_family;

impl AccountStore {
    /// this （ not is Cline return None）。
    ///
    /// （ new ）：`serves_provider` that use need 
    /// provider id， can can 。
    pub fn cline_account_provider(&self, account_id: &str) -> Option<String> {
        let guard = self.guard();
        let record = self.record_by_id(&guard, account_id)?;
        let provider = record.provider();
        is_cline_family(&provider).then_some(provider)
    }

    /// this is not is ** when **（`desktop: true` ）。
    ///
    /// this in Cline client `providers.json` 、 not token，
    /// new will client that will （ in ， after ）—— 
    /// new 。
    pub fn cline_is_desktop_account(&self, account_id: &str) -> bool {
        let guard = self.guard();
        self.record_by_id(&guard, account_id)
            .map(|record| record.is_desktop() && is_cline_family(&record.provider()))
            .unwrap_or(false)
    }

    /// Cline （** not token**）。
    ///
    /// ── Why this 1 in ────────────────────────────────────────
    /// and 1 by ：（ no has `pluginSecret`） need need has 。
    ///
    /// ── ──────────────────────────────────────────────────
    /// ** no has `pool` **（ before has ）： already by `provider` —— before by 
    /// provider id ， this ， 1 `pool` will 
    /// 。
    ///
    /// `rateLimits` ** not **： not is 「 to 」， is ** 
    /// ** —— up 429 after in ， in up 
    /// `rateLimits[< up >].resetAt` 「 this for this can not can use 」。
    /// = no ， already in 、 1 
    /// 429 （2026-10-03 up to is this ）。
    pub(crate) fn to_cline_public_account(&self, record: &StoredAccount) -> Value {
        cline_public_account(record)
    }
}

/// （ not —— ）。
fn cline_public_account(record: &StoredAccount) -> Value {
    let value = record.to_value();
        let mut out = Map::new();
        out.insert("id".to_string(), Value::String(record.id().to_string()));
        // provider ** **（ not is ）： use this 1 
        // ， will in 1 
        out.insert("provider".to_string(), Value::String(record.provider()));
        for key in ["account", "displayName", "tokenTail"] {
            if let Some(field) = value.get(key).filter(|field| !field.is_null()) {
                out.insert(key.to_string(), field.clone());
            }
        }
        // ： （ as empty / and account / and when ）
        // ； use 。 and 1 ——
        // before after up not can as this 1 。
        let account = value
            .get("account")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let stored_display = value
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let stored_name = record.name();
        let stored_name_trimmed = stored_name.trim();
        let auto_name = stored_name_trimmed.is_empty()
            || stored_name_trimmed == account
            || (!stored_display.is_empty() && stored_name_trimmed == stored_display.as_str());
        let name = if auto_name && !stored_display.is_empty() {
            stored_display
        } else {
            stored_name
        };
        out.insert("name".to_string(), Value::String(name));
        out.insert(
            "hasRefreshToken".to_string(),
            Value::Bool(
                value
                    .get("refreshToken")
                    .and_then(Value::as_str)
                    .map(|text| !text.trim().is_empty())
                    .unwrap_or_else(|| {
                        // not refreshToken（ in clientfile），
                        // use when down 
                        value
                            .get("hasRefreshToken")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                    }),
            ),
        );
        out.insert("priority".to_string(), Value::from(record.priority()));
        out.insert("enabled".to_string(), Value::Bool(record.enabled()));
        out.insert("desktop".to_string(), Value::Bool(record.is_desktop()));
        out.insert("source".to_string(), Value::String(record.source()));
        out.insert("addedAt".to_string(), Value::from(record.added_at()));
        out.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        out.insert(
            "rateLimits".to_string(),
            value
                .get("rateLimits")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        );
        out.insert(
            "available".to_string(),
            // use `has_credentials()` not is `has_token()`： by not 
            // token（ when clientfile）， has_token will unavailable
            Value::Bool(record.enabled() && record.has_credentials()),
        );
        if let Some(expires) = value.get("expiresAt").filter(|value| !value.is_null()) {
            out.insert("expiresAt".to_string(), expires.clone());
        }
        out.insert(
            "proxy".to_string(),
            crate::server::core::proxies::describe_account_proxy(Some(&record.proxy())),
        );
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(fields: Value) -> StoredAccount {
        StoredAccount::from_map(fields.as_object().cloned().unwrap_or_default())
    }

    /// ** not **： need need and 「 can not can 」。
    #[test]
    fn the_public_form_never_carries_tokens() {
        let public = cline_public_account(&record(json!({
            "id": "cline-pass-usr-1",
            "provider": "cline-pass",
            "account": "usr-1",
            "accessToken": "workos:secret",
            "refreshToken": "r-1",
            "tokenTail": "cret",
        })));
        assert!(public.get("accessToken").is_none(), "公开形态不许带令牌");
        assert!(public.get("refreshToken").is_none(), "公开形态不许带续期串");
        assert_eq!(
            "cline-pass",
            public.get("provider").and_then(Value::as_str).unwrap_or("")
        );
        assert!(public.get("pool").is_none(), "池由 provider 表达，不重复下发");
    }

    /// ； use must。
    #[test]
    fn the_display_name_follows_only_auto_generated_names() {
        let auto = cline_public_account(&record(json!({
            "id": "x",
            "provider": "cline-free",
            "name": "usr-1",
            "account": "usr-1",
            "displayName": "someone@example.com",
        })));
        assert_eq!(
            "someone@example.com",
            auto.get("name").and_then(Value::as_str).unwrap_or(""),
            "自动生成的账号名要换成展示名"
        );

        let custom = cline_public_account(&record(json!({
            "id": "y",
            "provider": "cline-free",
            "name": "我的号",
            "account": "usr-1",
            "displayName": "someone@example.com",
        })));
        assert_eq!(
            "我的号",
            custom.get("name").and_then(Value::as_str).unwrap_or(""),
            "用户改过的备注名不能被覆盖"
        );
    }

    /// ：`desktop` as 、`hasRefreshToken` when 、
    /// `available` `has_credentials()`（ not token available）。
    #[test]
    fn desktop_accounts_are_available_without_a_stored_token() {
        let public = cline_public_account(&record(json!({
            "id": "cline-free-desktop",
            "provider": "cline-free",
            "desktop": true,
            "hasRefreshToken": true,
        })));
        assert!(public.get("desktop").and_then(Value::as_bool).unwrap_or(false));
        assert!(
            public
                .get("hasRefreshToken")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        assert!(public.get("available").and_then(Value::as_bool).unwrap_or(false));
    }
}
