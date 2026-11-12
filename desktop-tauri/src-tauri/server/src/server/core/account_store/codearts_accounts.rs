//! CodeArts ****（ already to ）。
//!
//! ── and config ──────────────────────────────────────
//! `CODEARTS_PROVIDER_ID` ； old ，limit already 
//! Token ， not 、 and enabled。
//!
//! ── Whymust ──────────────────────────────────────
//! in （`access_key_id` / `security_token` /
//! `oauth_context`…）， no has `pluginSecret` —— `store_view::public_account`
//! not to 「」 that 1 。 not ， this will 
//! `public_account` use （uid empty、tokenTail empty、`hasRefreshToken` false），
//! up 。
//!
//! ── already ──────────────────────────────────────────
//! （`Credential::from_payload` / `to_record_value`）、
//! （`domain_id + user_id`）、、、 —— in 。
//! new by `plugin_upsert_account` （ in `pluginSecret`），
//! up that 「」 will ，file for this is empty。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Value};

use crate::server::core::limiter::{self, LimiterKind, LimiterRule};
use crate::server::core::limiter::types::Metric;
use crate::server::core::providers::{kind_id, ProviderKind};

use super::state::StoredAccount;

/// provider id（`store_view` and for need use ，
/// and its 1 ：`<X>_PROVIDER_ID`）。
pub const CODEARTS_PROVIDER_ID: &str = kind_id(ProviderKind::CodeArts);

/// Token then not ； already empty limitunlimited。
pub(crate) fn migrate_limiters(accounts: &mut [StoredAccount]) -> usize {
    let mut changed = 0;
    for record in accounts {
        if record.provider() != CODEARTS_PROVIDER_ID { continue; }
        let mut groups = limiter::effective_groups_in(record.fields());
        let mut removed = limiter::effective_rules_in(record.fields()).iter().any(|rule| rule.kind == LimiterKind::Token);
        for group in &mut groups {
            let before = group.conditions.len();
            group.conditions.retain(|condition| !matches!(condition.metric, Metric::Tokens | Metric::Requests));
            removed |= group.conditions.len() != before;
        }
        if !removed { continue; }
        groups.retain(|group| !group.conditions.is_empty());
        let configured_empty = if let Some(groups) = record.get("limiterGroups").and_then(Value::as_array) {
            groups.is_empty()
        } else {
            record.get("limiters").and_then(Value::as_array).is_some_and(Vec::is_empty)
        };
        if groups.is_empty() && !configured_empty {
            let rule = LimiterRule { kind: LimiterKind::Balance, action: limiter::LimiterAction::Skip,
                threshold: crate::server::core::usage_records::DEFAULT_LOW_BALANCE_THRESHOLD,
                period: 0, reset: limiter::LimiterReset::Fixed, enabled: true };
            groups = limiter::groups::groups_from_rules(&[rule]);
        }
        for group in &mut groups {
            if let Some(condition) = group.conditions.first_mut() {
                condition.join = limiter::types::Join::Or;
            }
        }
        let rules = limiter::legacy_rules(&groups);
        record.set("limiterGroups", limiter::types::json_of(&groups));
        record.set("limiters", limiter::rules_to_json(&rules));
        let balance = rules.iter().find(|rule| rule.kind == LimiterKind::Balance);
        record.set("lowBalance", balance.map(|rule| json!({
            "mode": if rule.action == limiter::LimiterAction::Disable { "disable" } else { "skip" },
            "threshold": rule.threshold,
        })).unwrap_or_else(|| json!({ "mode": "off", "threshold": 0 })));
        changed += 1;
    }
    changed
}

impl super::AccountStore {
    /// use （****）。
    ///
    /// ** not **（AK/SK/STS/refresh token/DPoP 1 not 
    /// in this ）。
    ///
    /// not —— not this （ that in ）。
    /// this use ：`hasRefreshToken` `refreshToken` in not in ，
    /// `available` has no has token。 1 ， not 。
    pub(crate) fn to_codearts_public_account(&self, record: &StoredAccount) -> Value {
        let available = record.has_token();
        let can_refresh = !record.refresh_token().is_empty();
        let mut public = serde_json::Map::new();
        for key in ["id", "provider", "name", "userId", "source", "tokenTail", "expiresAt"] {
            public.insert(key.to_string(), record.get(key).cloned().unwrap_or(Value::Null));
        }
        public.insert("edition".to_string(), Value::Null);
        public.insert("editionLabel".to_string(), Value::Null);
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
        // `maxConcurrent` not in this ：** no has 「 not 」 this **， up 0
        // （ from set） need up default and 3 not is 0，otherwise to 0 etc 
        // not 、 not will in when this （ `store_util::
        // max_concurrent_public_for`， that by provider up this ）。
        // this by `store_view::public_account` 1 。
        //
        // daily （ `codeartsWelfare`；
        // new is `pluginLedger.codeartsWelfare`， that ）：
        // need 「 already / is already to 」。** has id、 etc 
        // and **， no has ， not need need that 。
        public.insert(
            "welfare".to_string(),
            record.get("codeartsWelfare").cloned().unwrap_or(Value::Null),
        );
        // can by to （ and by this ）， and is not is 
        for (key, source) in [
            ("domainId", "domain_id"),
            ("userName", "user_name"),
            ("loginType", "login_type"),
        ] {
            if let Some(value) = record.get(source).filter(|value| !value.is_null()) {
                public.insert(key.to_string(), value.clone());
            }
        }
        Value::Object(public)
    }
}
