//! KukuAI ****（ this 1 ）。
//!
//! ── this fileWhy in ──────────────────────────────────────
//! already to （ / client / /
//! will / / directory / / to ）。
//!
//! ** ** has ： up `uid` /
//! `loginName` / `tokenTail` / `desktop` this （ no has `pluginSecret`），
//! need by this （`desktop` 「」，
//! `source` this is / file / ）。
//!
//! this 1 ****：、、 not up 。
//! new by （ in `pluginSecret` ， `to_plugin_public_account`）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Map, Value};

use crate::server::core::providers::{kind_id, ProviderKind};

use super::state::StoredAccount;
use super::store::AccountStore;

/// KukuAI provider id（`store_view` ）。
pub(crate) const KUKU_PROVIDER_ID: &str = kind_id(ProviderKind::Kuku);

impl AccountStore {
    /// （ before / will need use ；** not **）。
    pub fn to_kuku_public_account(&self, record: &StoredAccount) -> Value {
        let proxy = crate::server::core::proxies::describe_account_proxy(Some(&record.proxy()));
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        for key in ["uid", "loginName", "tokenTail"] {
            public.insert(
                key.to_string(),
                Value::String(record.get(key).and_then(Value::as_str).unwrap_or("").to_string()),
            );
        }
        public.insert("desktop".to_string(), Value::Bool(record.is_desktop()));
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), proxy);
        public.insert(
            "rateLimits".to_string(),
            record
                .get("rateLimits")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        );
        public.insert("available".to_string(), Value::Bool(true));
        Value::Object(public)
    }
}
