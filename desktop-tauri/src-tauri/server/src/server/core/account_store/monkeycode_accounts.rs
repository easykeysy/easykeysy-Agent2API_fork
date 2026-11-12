//! MonkeyCode **** and （ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! session 、`imageId` 、directory、 will to **
//! **（`agent2api-providers` `monkeycode/`）， not —— that is 
//! 1 。add by `account.normalize` 1 after 
//! `pluginSecret`， `plugin_accounts::to_plugin_public_account`。
//!
//! in this has **** and ： no has 
//! `pluginSecret`， is `accessToken` / `imageId` / `userId`，
//! need need by （）。 this not is up 。
//!
//! ── Why not JSON ────────────────────────────────
//! ** already in provider id **（`monkeycode` / `monkeycode-intl`），
//! `provider` —— can by `Region::from_provider_id` 。
//! will in when to error 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Map, Value};

use crate::server::core::providers::monkeycode::Region;

use super::state::StoredAccount;
use super::store::AccountStore;

impl AccountStore {
    /// 1 MonkeyCode （ JSON）。
    ///
    /// ****（`api::accounts::refresh_account` use 
    /// 「 this id 」）。 is provider， not can to 。
    pub fn monkeycode_account_record(&self, region: Region, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        let provider_id = region.provider_id();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == provider_id).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, provider_id)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// MonkeyCode ****（ HTTP response）： session ，
    /// 4 ； up `imageId` and `edition`（）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    pub fn to_monkeycode_public_account(&self, record: &StoredAccount) -> Value {
        let region = Region::from_provider_id(&record.provider());
        let available = record.has_token() && region.is_some();
        let mut public = Map::new();
        for key in ["id", "provider", "name", "userId", "source", "tokenTail", "imageId"] {
            public.insert(
                key.to_string(),
                record.get(key).cloned().unwrap_or(Value::Null),
            );
        }
        public.insert(
            "edition".to_string(),
            region
                .map(|value| Value::String(value.provider_id().to_string()))
                .unwrap_or(Value::Null),
        );
        public.insert(
            "editionLabel".to_string(),
            region
                .map(|value| Value::String(value.label().to_string()))
                .unwrap_or(Value::Null),
        );
        // has no has image_id： no has when up 「create」failure， before 
        public.insert(
            "hasImageId".to_string(),
            Value::Bool(
                record
                    .get("imageId")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty()),
            ),
        );
        // no has （30 limit）， false
        public.insert("hasRefreshToken".to_string(), Value::Bool(false));
        public.insert("desktop".to_string(), Value::Bool(false));
        public.insert("expiresAt".to_string(), Value::from(record.expires_at().unwrap_or(0.0) as i64));
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
        public.insert("available".to_string(), Value::Bool(available));
        Value::Object(public)
    }
}
