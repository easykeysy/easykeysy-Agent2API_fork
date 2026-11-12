//! ZCode ： + + 。
//!
//! ── Why not JSON ────────────────────────────────
//! ** already in provider id **（`zcode` / `zcode-intl`），
//! `provider` —— can by `Region::from_provider_id`
//! ， not need need 1 region （ has ， when 
//! 「provider is 、 is 」 will to error ）。
//!
//! ── after （2026-10 ）───────────────────────────
//! CLI 、、、、、
//! and 、 to 。file
//! **** that ：
//!
//! 1. ** **（`to_zcode_public_account`）： up 
//! no has `pluginSecret`， not to ； not in this will 
//! WorkBuddy empty（userId empty、tokenTail empty）， can use 。
//! 2. ****（`update_zcode_plan`）：`zcodePlan` is ，
//! after is —— this and 「 up 」none。
//! 3. ****（`mark_zcode_claim`）：`claimAt` / `claimPlans` in 
//! up ， to in and is 。
//!
//! new by （ in `pluginSecret` ），`store_view::public_account`
//! 1 will ， to not this 。

use serde_json::{json, Map, Value};

use crate::server::core::providers::zcode;
use crate::server::core::providers::zcode::region::Region;
use crate::server::logging;

use super::sql;
use super::state::StoredAccount;
use super::store::{AccountStore, AccountStoreError};

impl AccountStore {
    /// use 1 ZCode （`account_id` as empty when that ）。
    ///
    /// empty id use ：`patch_account` / `refresh` 「 this is not is ZCode 」。
    /// empty id not ， 1 can 。
    pub fn zcode_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (Region::from_provider_id(&record.provider()).is_some())
                .then(|| record.to_value());
        }
        Region::ALL
            .into_iter()
            .filter_map(|region| {
                self.records_for_provider(&guard, region.provider_id())
                    .into_iter()
                    .filter(|record| record.enabled() && record.has_token())
                    .min_by_key(|record| record.order_key())
            })
            .min_by_key(|record| record.order_key())
            .map(|record| record.to_value())
    }

    /// ZCode （）。
    ///
    /// and its **、**， before not by provider ：
    /// `edition` 、`editionLabel` in ， is up 
    /// 「ZCode / 」 use has that 1 。
    ///
    /// ── this 1 ──────────────────────────────
    /// new has `pluginSecret`，`store_view::public_account` 1 
    /// （`account_view` by to ）。 to this is up 。
    pub fn to_zcode_public_account(&self, record: &StoredAccount) -> Value {
        let region = Region::from_provider_id(&record.provider());
        let available = record.has_token() && region.is_some();
        let jwt_present = !record.jwt().trim().is_empty();
        let mut public = Map::new();
        for key in ["id", "provider", "name", "userId", "source", "tokenTail"] {
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
        // no has ， false —— before not to this 「」
        // by ， not is 
        public.insert("hasRefreshToken".to_string(), Value::Bool(false));
        // 「 can not can 」 is **** can ： no has jwt when up 
        // by not can ， not is 「」
        public.insert("canClaim".to_string(), Value::Bool(jwt_present));
        // 1 when + to id（0 / empty = from ）。
        // and `checkinAt` 1 ： to ** when ** not is 「 no 」 ——
        // need by use when ， that in up already has 。
        public.insert("claimAt".to_string(), Value::from(record.claim_at()));
        public.insert(
            "claimPlanId".to_string(),
            Value::String(record.claim_plan_id()),
        );
        // （`{planId: ms}`）：****「 already 」——
        // 1 can can when can ，「 already 」 is up by 
        // （ `mark_zcode_claim` ），must to ， not can to 1 
        // 「」 by by 。
        public.insert(
            "claimPlans".to_string(),
            Value::Object(record.claim_plans()),
        );
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
        // use up （`coding-plan` / `start-plan`）。
        // / not when ** to default** not is null： up down need in current，
        // before default etc 1 
        public.insert(
            zcode::PLAN_FIELD.to_string(),
            Value::String(
                zcode::normalize_plan(&record.zcode_plan())
                    .unwrap_or(zcode::PLAN_CODING)
                    .to_string(),
            ),
        );
        // `chatSupported` / `maxConcurrent` not in this ： is 1 ，
        // by `store_view::public_account` （ and Qoder that ）
        Value::Object(public)
    }

    /// set this up （`coding-plan` / `start-plan`），return。
    ///
    /// ── Why not use `apply_patch` ────────────────────────
    /// and CatPaw `balanceToken` 1 ：`apply_patch` is ** use ** 
    /// ， 1 has ZCode ， etc can 1 
    /// 「 not 」 。 this use （`api::accounts::patch_account`），
    /// —— not 400， not is 1 default
    /// （ that will use as success）。
    ///
    /// empty = default（`coding-plan`）： up down to is this ，
    /// from old （ no has this ） up when is this —— `plan_of` 
    /// default，「default」 and 「 not 」 etc ， this down ，。
    ///
    /// and `mark_checkin` not ： this is ** use **，`updatedAt` new 。
    pub fn update_zcode_plan(
        &self,
        account_id: &str,
        patch: &Value,
    ) -> Result<Vec<String>, AccountStoreError> {
        let raw = patch
            .as_str()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(zcode::PLAN_CODING);
        let Some(plan) = zcode::normalize_plan(raw) else {
            return Err(AccountStoreError::bad_request(
                "未知的 ZCode 套餐通道（只支持 coding-plan / start-plan）",
            ));
        };
        let guard = self.guard();
        let Some(mut record) = self.record_by_id(&guard, account_id) else {
            return Err(AccountStoreError::not_found("账号不存在"));
        };
        // already is this （ and ） when not ：set when before in 「 」
        // this ， to this is —— 1 will 
        // `updatedAt` none
        let current = zcode::normalize_plan(&record.zcode_plan()).unwrap_or(zcode::PLAN_CODING);
        let mirrored = ["pluginSecret", "pluginPublic"].iter().all(|field| {
            record.get(field).and_then(Value::as_object)
                .map(|fields| fields.get(zcode::PLAN_FIELD).and_then(Value::as_str) == Some(plan))
                .unwrap_or(true)
        });
        if current == plan && record.get(zcode::PLAN_FIELD).and_then(Value::as_str) == Some(plan) && mirrored {
            return Ok(Vec::new());
        }
        record.set_zcode_plan(plan);
        record.set_updated_at(logging::now_ms());
        self.with_conn(&guard, |conn| sql::update_in_place(conn, &record))?;
        // （ not ， `remove_account` ）
        drop(guard);
        // ── （ not this after ）────────────────
        // is 「 this for in up to 」，** is by 
        // **： that ， not that 。 old 
        // will use 、 down 1 request （ up is 「 in 」），
        // 「 no use 」。 after down 1 request will use new 1 ：
        // will up ， that is 。
        let cleared = self.clear_all_rate_limits(account_id);
        let mut changes = vec![format!("套餐通道 → {}", zcode::plan_label(plan))];
        if cleared > 0 {
            changes.push(format!(
                "已清除 {cleared} 个模型的限额标记（换通道后旧记录不再适用）"
            ));
        }
        Ok(changes)
    }

    /// 1 （ id → when ），return is success。
    ///
    /// success and 「 up already 」 this ： after this 1 already 
    /// （ can can is 1 ）， this 「 already 」——
    /// otherwise use will 1 that by ， 1 「 this already 」。
    ///
    /// ── Why** 1 ** not is 1 「 」──────────
    /// 1 when can ** can can not 1 **（ and daily when in ，
    /// use can 1 ）， and up 「 already 」 is ** by **
    /// （ 1 1003， 1 can ）。 after 1 ，「 A after 
    /// B can not can 」none from ， can by 「」 by ——
    /// that is use to 。「 already 」， can 、
    /// no that 。
    ///
    /// by （ use ）， no has not old ；
    /// 1 ，none： `CLAIM_LEDGER_KEEP_DAYS` 
    /// ， and `CLAIM_LEDGER_MAX` （ new that ）。
    ///
    /// and `mark_checkin` 1 ：** not ** `updatedAt`（ that is 「 」 
    /// when ， will in 「update …」 up ； 1 not is use ）。
    pub fn mark_zcode_claim(&self, account_id: &str, plan_id: &str, at: i64) -> bool {
        if at <= 0 { return false; }
        let guard = self.guard();
        let Some(mut record) = self.record_by_id(&guard, account_id) else {
            return false;
        };
        if at > record.claim_at() {
            record.set_claim_at(at);
            record.set_claim_plan_id(plan_id.trim());
        }
        let plan_id = plan_id.trim();
        if !plan_id.is_empty() {
            let mut ledger = record.claim_plans();
            let current = ledger.get(plan_id).and_then(Value::as_i64).unwrap_or(0);
            ledger.insert(plan_id.to_string(), Value::from(at.max(current)));
            prune_claim_ledger(&mut ledger, logging::now_ms());
            record.set_claim_plans(ledger);
        }
        self.with_conn(&guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }
}

/// （ in 「」 this up use ）
const CLAIM_LEDGER_KEEP_DAYS: i64 = 7;

/// limit（ 1 1 1 ，7 ； this is ）
const CLAIM_LEDGER_MAX: usize = 60;

/// ： ， by when new if 。
///
/// need ：（ can can has new plan_id），
/// to `accounts.data` that 1 —— not this JSON will 1 。
fn prune_claim_ledger(ledger: &mut serde_json::Map<String, Value>, now: i64) {
    let cutoff = now - CLAIM_LEDGER_KEEP_DAYS * 24 * 3600 * 1000;
    ledger.retain(|_, value| value.as_i64().is_some_and(|at| at >= cutoff));
    if ledger.len() <= CLAIM_LEDGER_MAX {
        return;
    }
    // when ，（`Value::as_i64` not by 0 → after ）
    let mut entries: Vec<(String, i64)> = ledger
        .iter()
        .map(|(key, value)| (key.clone(), value.as_i64().unwrap_or(0)))
        .collect();
    entries.sort_by(|left, right| right.1.cmp(&left.1));
    let keep: Vec<String> = entries
        .into_iter()
        .take(CLAIM_LEDGER_MAX)
        .map(|(key, _)| key)
        .collect();
    ledger.retain(|key, _| keep.iter().any(|kept| kept == key));
}
