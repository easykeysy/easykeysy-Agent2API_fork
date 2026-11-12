//! Antigravity **** and （ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! 、Google OAuth new 、directory、`v1internal` and 
//! will to ****（`agent2api-providers` `antigravity/`），
//! not —— that is 1 。
//!
//! in this has ：
//! ```text
//! antigravity_account_record this is not is （ use ）
//! to_antigravity_public_account （，none）
//! ```
//! add by `account.normalize` 1 after `pluginSecret`，
//! `plugin_accounts::to_plugin_public_account` —— that path not file。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Map, Value};

use super::state::StoredAccount;
use super::store::AccountStore;
use super::ANTIGRAVITY_PROVIDER_ID;

impl AccountStore {
    /// 1 Antigravity （ JSON）。
    ///
    /// ****（`api::accounts::refresh_account` use 
    /// 「 this id 」）。`account_id` as empty when returnenabled —— this 
    /// already no has use ， is as and 1 。
    pub fn antigravity_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == ANTIGRAVITY_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, ANTIGRAVITY_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// Antigravity ****（ HTTP response）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    ///
    /// and 1 ：** not ** —— to `tokenTail`（refresh
    /// token 4 ） and `hasRefreshToken` / `hasAccessToken` in ；
    /// `expiresAt`、`projectId`、`email` is / need need ， can to 。
    pub fn to_antigravity_public_account(&self, record: &StoredAccount) -> Value {
        let refresh_token = record.refresh_token();
        let mut public = Map::new();
        for key in ["id", "provider", "name", "userId", "email", "projectId", "source", "tokenTail"] {
            public.insert(
                key.to_string(),
                record.get(key).cloned().unwrap_or(Value::Null),
            );
        }
        public.insert(
            "hasRefreshToken".to_string(),
            Value::Bool(!refresh_token.trim().is_empty()),
        );
        public.insert("refreshTokenLongLived".to_string(), Value::Bool(true));
        public.insert(
            "hasAccessToken".to_string(),
            Value::Bool(
                record
                    .get("accessToken")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty()),
            ),
        );
        public.insert(
            "expiresAt".to_string(),
            Value::from(record.expires_at().unwrap_or(0.0) as i64),
        );
        public.insert("desktop".to_string(), Value::Bool(false));
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
        // available = has （refresh token is this 1 ）
        public.insert(
            "available".to_string(),
            Value::Bool(!refresh_token.trim().is_empty()),
        );
        Value::Object(public)
    }
}
