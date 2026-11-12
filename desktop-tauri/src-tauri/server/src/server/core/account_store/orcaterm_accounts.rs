//! OrcaTerm ****（ ）。
//!
//! ── this fileWhy 1 ──────────────────────────────
//! 、`GetUserInfo` 、、 to ****
//! （`agent2api-providers` `orcaterm/`）， not —— that is 
//! 1 。add by `account.normalize` 1 after 
//! `pluginSecret`， `plugin_accounts::to_plugin_public_account`。
//!
//! in this has ****（ up 、 no that ） 
//! ： no has `pluginSecret`， is `accessToken` /
//! `tokenExpireAt`， need need by 。 this not is up ，
//! is 1 ， in not will 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store::AccountStore;
use crate::server::core::providers::{kind_id, ProviderKind};

/// OrcaTerm provider id（ need use ；** from **）。
pub(crate) const ORCATERM_PROVIDER_ID: &str = kind_id(ProviderKind::OrcaTerm);

impl AccountStore {
    // ─── ： ────────────────────────────────────────

    /// （ before / will need use ；** not **）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    pub fn to_orcaterm_public_account(&self, record: &StoredAccount) -> Value {
        let proxy = crate::server::core::proxies::describe_account_proxy(Some(&record.proxy()));
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        public.insert(
            "uid".to_string(),
            Value::String(record.get("uid").and_then(Value::as_str).unwrap_or("").to_string()),
        );
        public.insert(
            "loginName".to_string(),
            Value::String(record.get("loginName").and_then(Value::as_str).unwrap_or("").to_string()),
        );
        public.insert(
            "tokenTail".to_string(),
            Value::String(record.get("tokenTail").and_then(Value::as_str).unwrap_or("").to_string()),
        );
        // tokenExpireAt （ms；0 = unknown）： before 「 to when 」，
        // 「 is in 」 can 
        public.insert(
            "tokenExpireAt".to_string(),
            Value::from(record.get("tokenExpireAt").and_then(Value::as_i64).unwrap_or(0)),
        );
        public.insert("desktop".to_string(), Value::Bool(record.is_desktop()));
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), proxy);
        public.insert(
            "rateLimits".to_string(),
            record.get("rateLimits").cloned().unwrap_or_else(|| Value::Object(Map::new())),
        );
        public.insert("available".to_string(), Value::Bool(true));
        Value::Object(public)
    }
}
