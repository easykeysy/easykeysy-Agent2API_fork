//! Accio ****（ this 1 ）。
//!
//! ── this fileWhy in ──────────────────────────────────────
//! already to （ / directory / / / ）。 not 
//! this ， not has and path。
//!
//! ** ** has ： up `userId` /
//! `accessToken` / `refreshToken` / `mode` this （ no has `pluginSecret`），
//! need by this （`edition` / `editionLabel`
//! ，`hasRefreshToken` 「 new Token」 by not ）。
//!
//! this 1 ****：、、 not up 。
//! new by （ in `pluginSecret` ， `to_plugin_public_account`）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Map, Value};

use crate::server::core::providers::accio::endpoints::Region;

use super::state::StoredAccount;
use super::store::AccountStore;

impl AccountStore {
    /// （ use ；`edition` / `editionLabel` ）。
    ///
    /// `available` is 「 has 、 and up provider 」——
    /// after used for provider （ in will ，
    /// this unavailable， use 1 failure ）。
    pub fn to_accio_public_account(&self, record: &StoredAccount) -> Value {
        let region = Region::from_provider_id(&record.provider());
        let available = record.has_token() && region.is_some();
        let can_refresh = record
            .get("refreshToken")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty());
        let mut public = Map::new();
        for key in [
            "id",
            "provider",
            "name",
            "userId",
            "email",
            "nickname",
            "source",
            "tokenTail",
            "expiresAt",
        ] {
            public.insert(key.to_string(), record.get(key).cloned().unwrap_or(Value::Null));
        }
        public.insert(
            "edition".to_string(),
            region
                .map(|value| Value::String(value.edition().to_string()))
                .unwrap_or(Value::Null),
        );
        public.insert(
            "editionLabel".to_string(),
            region
                .map(|value| Value::String(value.label().to_string()))
                .unwrap_or(Value::Null),
        );
        public.insert("hasRefreshToken".to_string(), Value::Bool(can_refresh));
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
        public.insert("available".to_string(), Value::Bool(available));
        Value::Object(public)
    }
}
