//! OfficeAce ****（ this 1 ）。
//!
//! ── this fileWhy in ──────────────────────────────────────
//! already to （ / directory / / to / / ）。
//! not this ， not has and path —— that 
//! is 「 up 」 ， in this etc to 
//! 。
//!
//! ** ** has ： up `baseUrl` /
//! `modelAppKey` / `modelAppSecret` / `expiresAt` this （ no has 
//! `pluginSecret`）， need by this 
//! （`baseUrl` to use 、`expiresAt` to config `expiry` 、
//! `hasRefreshToken` 「 new Token」 by not ）。
//!
//! this 1 ****：、、 not up 。
//! new by （ in `pluginSecret` ， `to_plugin_public_account`），
//! not will to this 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::{AccountStore, OFFICEACE_PROVIDER_ID};

impl AccountStore {
    /// 1 OfficeAce （ JSON）。
    ///
    /// `account_id` empty → by id （ not return None）；
    /// as empty → **enabled in ** before 1 。
    ///
    /// ── Why need need ──────────────────────────────────────
    /// and 「 this 」 need can by id ； 
    /// （`provider` is ）， this for 。
    pub fn officeace_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == OFFICEACE_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, OFFICEACE_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// （ before / will need use ；** not **）。
    ///
    /// kuku that ： + + + when + ，
    /// will —— `baseUrl`（ when need use ）、
    /// `expiresAt`（ when to when ，config `expiry:'expiresAt'` ）、
    /// `hasRefreshToken`（「 new Token」 by ）。
    pub fn to_officeace_public_account(&self, record: &StoredAccount) -> Value {
        let proxy = crate::server::core::proxies::describe_account_proxy(Some(&record.proxy()));
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), proxy);
        public.insert(
            "baseUrl".to_string(),
            record.get("baseUrl").cloned().unwrap_or(Value::Null),
        );
        public.insert(
            "expiresAt".to_string(),
            record.get("expiresAt").cloned().unwrap_or(Value::Null),
        );
        public.insert(
            "hasRefreshToken".to_string(),
            Value::Bool(
                record
                    .get("refreshToken")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty()),
            ),
        );
        Value::Object(public)
    }
}
