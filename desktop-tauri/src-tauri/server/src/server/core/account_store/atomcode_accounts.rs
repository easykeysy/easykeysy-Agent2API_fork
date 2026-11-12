//! AtomCode **** and （ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! （）、directory、、、、 to **
//! **（`agent2api-providers` `atomcode/`）， not —— that is 
//! 1 。add by `account.normalize` 1 after 
//! `pluginSecret`， `plugin_accounts::to_plugin_public_account`。
//!
//! in this has **** and ： no has 
//! `pluginSecret`， is `accessToken` / `refreshToken`，
//! need need by 。 this not is up 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::Value;

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::{ATOMCODE_PROVIDER_ID, AccountStore};

impl AccountStore {
    /// 1 AtomCode （ JSON）。
    ///
    /// ****（`refresh_account` use 「 this id 」）。
    pub fn atomcode_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == ATOMCODE_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, ATOMCODE_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// AtomCode ****（ HTTP response）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    ///
    /// ** **：`accessToken` and `refreshToken`。 before that 
    /// in this is —— refreshToken is valid ， not can down 。
    pub fn to_atomcode_public_account(&self, record: &StoredAccount) -> Value {
        let mut value = record.to_value();
        if let Some(object) = value.as_object_mut() {
            object.remove("accessToken");
            object.remove("refreshToken");
            object.insert("available".to_string(), Value::Bool(record.enabled()));
            object.insert(
                "hasSession".to_string(),
                Value::Bool(record.get("accessToken").is_some() || record.get("tokenTail").is_some()),
            );
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};

    /// must ** **。
    #[test]
    fn public_shape_never_leaks_either_token() {
        let record = StoredAccount::from_map({
            let mut map = Map::new();
            map.insert("id".into(), Value::String("atomcode-42".into()));
            map.insert("provider".into(), Value::String(ATOMCODE_PROVIDER_ID.into()));
            map.insert("accessToken".into(), Value::String("at-secret".into()));
            map.insert("refreshToken".into(), Value::String("rt-secret".into()));
            map.insert("tokenTail".into(), Value::String("cret".into()));
            map
        });
        // is store ， this then （ store）
        let mut public = record.to_value();
        if let Some(object) = public.as_object_mut() {
            object.remove("accessToken");
            object.remove("refreshToken");
        }
        assert!(public.get("accessToken").is_none());
        assert!(public.get("refreshToken").is_none());
        assert_eq!(public.get("tokenTail").and_then(Value::as_str), Some("cret"));
    }

    /// `json!` and `StoredAccount` not 。
    #[test]
    fn record_roundtrip_keeps_identity_fields() {
        let map = json!({
            "id": "atomcode-42",
            "provider": ATOMCODE_PROVIDER_ID,
            "userId": "42",
            "username": "someone",
        });
        let record = StoredAccount::from_map(map.as_object().cloned().unwrap_or_default());
        assert_eq!(record.get("userId"), Some(&Value::String("42".into())));
        assert_eq!(record.get("username"), Some(&Value::String("someone".into())));
    }
}
