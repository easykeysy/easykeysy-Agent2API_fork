//! Command Code ：** **（ this 1 ）。
//!
//! ── Why this no has 「」 ────────────────────────────────
//! already to ：API Key 1 / check / best-effort
//! / directory / NDJSON in 。 this **** 
//! —— that （ no has `pluginSecret`） after need can in 。
//!
//! new by 、 `pluginSecret`，`public_account` 1 will to 
//! `account_view`， not will to this 。
//!
//! file already has 、** no has request**。 not unwrap/expect（release is 
//! panic=abort）。

use serde_json::Value;

use super::state::StoredAccount;
use super::store::AccountStore;
use super::COMMANDCODE_PROVIDER_ID;

impl AccountStore {
    /// 1 Command Code （ JSON）。
    ///
    /// `account_id` empty → by id （ not return None）；
    /// as empty → **enabled in ** before 1 （ and `loomy_account_record`
    /// 1 ：default use ）。
    pub fn commandcode_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == COMMANDCODE_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, COMMANDCODE_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// Command Code ****（ HTTP response）： key ，。
    pub fn to_commandcode_public_account(&self, record: &StoredAccount) -> Value {
        let mut value = record.to_value();
        if let Some(object) = value.as_object_mut() {
            object.remove("accessToken");
            object.insert("available".to_string(), Value::Bool(record.enabled()));
            // has no has key（ etc has_credentials， this to is **** 
            // ， by 「 already config / need 」）
            object.insert(
                "hasKey".to_string(),
                Value::Bool(
                    record
                        .get("accessToken")
                        .and_then(Value::as_str)
                        .is_some_and(|value| !value.trim().is_empty()),
                ),
            );
        }
        value
    }
}
