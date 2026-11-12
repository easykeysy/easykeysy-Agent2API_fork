//! LobsterAI **** and （ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! 、、directory、、 to to ****
//! （`agent2api-providers` `lobsterai/`）， not —— that is 
//! 1 。 by `login.callback` after `pluginSecret`，
//! `plugin_accounts::to_plugin_public_account`。
//!
//! in this has ****（ up 、 no that ） 
//! ： no has `pluginSecret`， is `accessToken` / `uuid` /
//! `keyfrom`， need need by 。 this not is up ，
//! is 1 ， in not will 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::Value;

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::{AccountStore, LOBSTERAI_PROVIDER_ID};

impl AccountStore {
    /// 1 LobsterAI （ JSON）。
    ///
    /// ****（`api::accounts::refresh_account` use 
    /// 「 this id 」）。
    pub fn lobsterai_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == LOBSTERAI_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, LOBSTERAI_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// LobsterAI ****（ HTTP response）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    ///
    /// `refreshToken` and ** need **： before is valid ， after is 
    /// —— 1 down etc 「 this can 」 。
    /// need need 「 can not can 」 by `canRefresh` ， not to 。
    pub fn to_lobsterai_public_account(&self, record: &StoredAccount) -> Value {
        let mut value = record.to_value();
        let can_refresh = record.get("refreshToken").is_some()
            && record.get("firstKeyfrom").is_some()
            && record.get("latestKeyfrom").is_some();
        if let Some(object) = value.as_object_mut() {
            object.remove("accessToken");
            object.remove("refreshToken");
            object.remove("uuid");
            object.remove("firstKeyfrom");
            object.remove("latestKeyfrom");
            object.insert("available".to_string(), Value::Bool(record.enabled()));
            object.insert("hasSession".to_string(), Value::Bool(true));
            object.insert("canRefresh".to_string(), Value::Bool(can_refresh));
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Map};

    /// must ** + **（ 1 canRefresh ）。
    #[test]
    fn public_shape_never_leaks_tokens_or_payload_fields() {
        let record = StoredAccount::from_map({
            let mut map = Map::new();
            map.insert("id".into(), Value::String("lobsterai-42".into()));
            map.insert("provider".into(), Value::String(LOBSTERAI_PROVIDER_ID.into()));
            map.insert("accessToken".into(), Value::String("at".into()));
            map.insert("refreshToken".into(), Value::String("rt".into()));
            map.insert("uuid".into(), Value::String("u-1".into()));
            map.insert("firstKeyfrom".into(), Value::String("fk".into()));
            map.insert("latestKeyfrom".into(), Value::String("lk".into()));
            map
        });
        let mut public = record.to_value();
        let can_refresh = public.get("refreshToken").is_some()
            && public.get("firstKeyfrom").is_some()
            && public.get("latestKeyfrom").is_some();
        if let Some(object) = public.as_object_mut() {
            for key in [
                "accessToken",
                "refreshToken",
                "uuid",
                "firstKeyfrom",
                "latestKeyfrom",
            ] {
                object.remove(key);
            }
            object.insert("canRefresh".into(), Value::Bool(can_refresh));
        }
        for key in [
            "accessToken",
            "refreshToken",
            "uuid",
            "firstKeyfrom",
            "latestKeyfrom",
        ] {
            assert!(public.get(key).is_none(), "{key} 不该出现在公开形态里");
        }
        assert_eq!(public.get("canRefresh"), Some(&Value::Bool(true)));
    }

    /// `json!` and `StoredAccount` not 。
    #[test]
    fn record_roundtrip_keeps_payload_fields() {
        let map = json!({
            "id": "lobsterai-42",
            "provider": LOBSTERAI_PROVIDER_ID,
            "firstKeyfrom": "fk",
            "latestKeyfrom": "lk",
        });
        let record = StoredAccount::from_map(map.as_object().cloned().unwrap_or_default());
        assert_eq!(record.get("firstKeyfrom"), Some(&Value::String("fk".into())));
        assert_eq!(record.get("latestKeyfrom"), Some(&Value::String("lk".into())));
    }
}
