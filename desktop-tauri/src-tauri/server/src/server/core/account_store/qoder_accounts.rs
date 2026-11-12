//! Qoder provider（）+ userId ； and its 。
//!
//! ── （2026-10）──────────────────────────────────────────
//! is ** provider**（`qoder` in / `qoder-intl` ）， by 
//! provider id （`Region::from_provider_id`）， not from `mode` 。
//! `mode`（ has ， `edition` by ），
//! ** provider id** —— and AutoClaw / Accio / ZCode 1 。
//!
//! ── after （2026-10 ）───────────────────────────
//! 、PAT 、、directory、 this ** up ** already to 
//! ，file ：
//!
//! 1. ****（`qoder_account_record`）：「 this 
//! 」，otherwise will to workbuddy 、 to and none error。
//! 2. ** **（`to_qoder_public_account`）： up no has 
//! `pluginSecret`， not to ； not in this will WorkBuddy 
//! empty（uid empty、tokenTail empty）， can use 。
//! 3. ** and **： not can as ； 1 userId
//! in is ， new use request。
//!
//! new by （ in `pluginSecret` ），。

use std::sync::MutexGuard;

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use crate::server::core::providers::qoder::endpoints::Region;

use super::sql;
use super::state::StoredAccount;
use super::store::AccountStore;

fn explicit_region(fields: &Map<String, Value>) -> Option<Region> {
    let value = ["mode", "edition"].into_iter().find_map(|key| {
        fields.get(key).filter(|value| {
            !value.is_null() && !matches!(value, Value::String(text) if text.trim().is_empty())
        })
    })?;
    let text = value.as_str()?.trim();
    Region::from_provider_id(text).or_else(|| Region::parse(text).ok())
}

fn plugin_region(record: &StoredAccount) -> Option<Region> {
    // can can in after in ， when has 。
    record.get("pluginSecret").and_then(Value::as_object).and_then(explicit_region)
        .or_else(|| record.get("pluginPublic").and_then(Value::as_object).and_then(explicit_region))
}

pub(crate) fn migrated_region(record: &StoredAccount) -> Option<Region> {
    let region = Region::from_provider_id(&record.provider())?;
    if !record.has_plugin_secret() {
        return explicit_region(record.fields()).or(Some(region));
    }
    // provider is ； for has 。
    if region == Region::Global && plugin_region(record) == Some(Region::Cn) {
        Some(Region::Cn)
    } else {
        Some(region)
    }
}

pub(super) fn migrate_accounts(accounts: &mut [StoredAccount]) -> usize {
    let mut changed = 0;
    for record in accounts {
        let Some(region) = migrated_region(record) else { continue; };
        if record.provider() != region.provider_id() {
            record.set_provider(region.provider_id());
            changed += 1;
        }
    }
    changed
}

fn user_id_of(fields: &Map<String, Value>) -> Option<String> {
    ["userId", "user_id", "uid"].into_iter().find_map(|key| match fields.get(key) {
        Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim().to_string()),
        Some(Value::Number(number)) => Some(number.to_string()),
        _ => None,
    })
}

fn record_user_id(record: &StoredAccount) -> Option<String> {
    record.get("pluginSecret").and_then(Value::as_object).and_then(user_id_of)
        .or_else(|| user_id_of(record.fields()))
}

impl AccountStore {
    pub(super) fn qoder_account_for_upsert(
        &self,
        guard: &MutexGuard<'_, ()>,
        region: Region,
        identity: &str,
        secret: &Value,
    ) -> Result<(String, Option<StoredAccount>), String> {
        let mut records = self.with_conn(guard, |conn| {
            let ids: Vec<String> = sql::queue_rows(conn)?.into_iter()
                .filter(|row| Region::from_provider_id(&row.provider).is_some())
                .map(|row| row.id).collect();
            ids.into_iter().map(|id| sql::load_by_id(conn, &id))
                .collect::<rusqlite::Result<Vec<_>>>()
        }).map_err(|error| error.message)?;
        let user_id = secret.as_object().and_then(user_id_of);
        let existing = records.drain(..).flatten()
            .filter(|record| migrated_region(record) == Some(region))
            .filter(|record| match user_id.as_ref() {
                Some(user_id) => record_user_id(record).as_ref() == Some(user_id),
                None => record.id() == identity,
            })
            .min_by_key(|record| (record.added_at(), record.order_key(), record.id().to_string()));
        if let Some(record) = existing {
            return Ok((record.id().to_string(), Some(record)));
        }
        let id = format!("qoder-{}-{:x}", region.id(),
            Sha256::digest(user_id.as_deref().unwrap_or(identity).as_bytes()));
        if self.with_conn(guard, |conn| sql::load_by_id(conn, &id))
            .map_err(|error| error.message)?.is_some() {
            return Err("Qoder 账号 ID 已被其它账号占用，请先核对账号记录".to_string());
        }
        Ok((id, None))
    }

    /// 1 ：empty id by id （ for ），empty id this 
    /// **available**（enabled + has ， by ）—— and 
    /// `autoclaw_account_record` 1 （ after 「」must in ）。
    pub fn qoder_account_record(&self, region: Region, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        let provider = region.provider_id();
        if !account_id.is_empty() {
            // by id 1 after （ id will has 1 ，provider is ）
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == provider).then(|| record.to_value());
        }
        self.records_for_provider(&guard, provider)
            .into_iter()
            .filter(|record| record.enabled() && record.has_token())
            .min_by_key(|record| record.order_key())
            .map(|record| record.to_value())
    }

    pub fn to_qoder_public_account(&self, record: &StoredAccount) -> Value {
        // provider （ after ），payload `mode` is 
        // old —— `mode` when `from_payload` will default Global，
        // that default for in is ， not can 。
        let region = Region::from_provider_id(&record.provider())
            .or_else(|| Region::from_payload(&record.to_value()).ok());
        let available = record.has_token() && region.is_some();
        // can not can has no has refreshToken —— not this 
        // （`Credentials` to ）。 and 
        // `credentials.can_refresh()` etc ：empty。
        let can_refresh = !record.refresh_token().trim().is_empty();
        let mut public = Map::new();
        for key in ["id", "provider", "name", "userId", "email", "nickname", "mode", "source", "tokenTail", "expiresAt"] {
            public.insert(key.to_string(), record.get(key).cloned().unwrap_or(Value::Null));
        }
        public.insert("edition".to_string(), region.map(|value| Value::String(value.edition().to_string())).unwrap_or(Value::Null));
        public.insert("editionLabel".to_string(), region.map(|value| Value::String(value.label().to_string())).unwrap_or(Value::Null));
        public.insert("hasRefreshToken".to_string(), Value::Bool(can_refresh));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), crate::server::core::proxies::describe_account_proxy(Some(&record.proxy())));
        public.insert("rateLimits".to_string(), record.get("rateLimits").cloned().unwrap_or_else(|| json!({})));
        public.insert("desktop".to_string(), Value::Bool(false));
        public.insert("available".to_string(), Value::Bool(available));
        // `chatSupported` / `maxConcurrent` ** not in this **： is 1 
        // ， by `store_view::public_account` in （ before by 
        // `supports_chat()`， after by `store_util::max_concurrent_public_for`）。
        Value::Object(public)
    }
}
