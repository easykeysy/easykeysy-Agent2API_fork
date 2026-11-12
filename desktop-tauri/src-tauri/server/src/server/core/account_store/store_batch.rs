//! （Agent2API W3-T4 from `store_crud.rs` ，file）。
//!
//! batch_update （enabled / / and / / limit； apply_patch）
//! batch_remove delete（ not in id failed， not in ）
//!
//! ── and ──────────────────────────────────────
//! `store_crud` has （`AccountStore::apply_patch`、
//! `to_raccoon_public_account` etc ）—— 1 `impl AccountStore` ，
//! file can ， not can （Rust has for can ）。
//!
//! ── delete（W3-T4）────────────────────────────────────────
//! `batch_remove` will 「 not can delete」 （ when ），
//! `failed` and Reason —— and `remove_account` 1 。
//! **Note**：must in before （`protected_from_removal`
//! ，`std::sync::Mutex` not can ， will ）。

use serde_json::{json, Map, Value};

use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store_view::Visibility;
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};
use crate::server::core::account_store::store_crud::normalize_max_concurrent;
use crate::server::core::account_store::store_util::truncate_chars;
use crate::server::core::proxies::describe_account_proxy;
use crate::server::core::{limiter, usage_query};
use crate::server::logging;

#[derive(Clone, Copy)]
enum BatchLimiterMode {
    Append,
    Replace,
    Enable,
    Disable,
    Clear,
}

impl BatchLimiterMode {
    fn parse(value: &Value) -> Result<Self, AccountStoreError> {
        match value.as_str() {
            Some("append") => Ok(Self::Append),
            Some("replace") => Ok(Self::Replace),
            Some("enable") => Ok(Self::Enable),
            Some("disable") => Ok(Self::Disable),
            Some("clear") => Ok(Self::Clear),
            _ => Err(AccountStoreError::bad_request(
                "限制器操作必须是追加、替换、启用、停用或清空",
            )),
        }
    }

    fn needs_rules(self) -> bool {
        matches!(self, Self::Append | Self::Replace)
    }

    fn apply(
        self,
        fields: &Map<String, Value>,
        incoming: &[limiter::LimiterRule],
    ) -> Value {
        let mut current = limiter::effective_rules_in(fields);
        match self {
            Self::Append => {
                // 1 not can not then 。
                for rule in incoming {
                    if !current.contains(rule) {
                        current.push(*rule);
                    }
                }
            }
            Self::Replace => current = incoming.to_vec(),
            Self::Enable | Self::Disable => {
                let enabled = matches!(self, Self::Enable);
                for rule in &mut current {
                    rule.enabled = enabled;
                }
            }
            Self::Clear => current.clear(),
        }
        limiter::rules_to_json(&current)
    }

    fn summary(self) -> &'static str {
        match self {
            Self::Append => "限制器 → 追加规则",
            Self::Replace => "限制器 → 替换规则",
            Self::Enable => "限制器 → 全部启用",
            Self::Disable => "限制器 → 全部停用",
            Self::Clear => "限制器 → 清空规则",
        }
    }
}

impl AccountStore {
    /// ；none need config。
    ///
    /// up is 「 for in 1 update_account」， 1 。
    /// failure not in ： ok / failed（Reason）。
    pub fn batch_update(
        &self,
        ids: &[String],
        patch: &Value,
    ) -> Result<Value, AccountStoreError> {
        if ids.is_empty() {
            return Err(AccountStoreError::bad_request("缺少要操作的账号 id"));
        }
        let Some(patch_object) = patch.as_object() else {
            return Err(AccountStoreError::bad_request("批量修改内容必须是 JSON 对象"));
        };
        // 、 and ， not 。
        let mut allowed = Map::new();
        let mut enabled_value = false;
        if let Some(value) = patch_object.get("enabled") {
            enabled_value = !matches!(value, Value::Bool(false));
            allowed.insert("enabled".to_string(), Value::Bool(enabled_value));
        }
        if let Some(value) = patch_object.get("proxy") {
            let normalized = crate::server::core::proxies::normalize_account_proxy(value)
                .map_err(|error| AccountStoreError::new(error.message, error.status_code))?
                .unwrap_or(Value::Null);
            allowed.insert("proxy".to_string(), normalized);
        }
        if let Some(value) = patch_object.get("maxConcurrent") {
            // ** 1 **（ and 1 ， `normalize_max_concurrent`）：
            // will 1 400 N ， this is 「parameter」，
            // not is 「 not down 」—— after this failed。
            let next = normalize_max_concurrent(value)?;
            allowed.insert("maxConcurrent".to_string(), Value::from(next));
        }
        if let Some(value) = patch_object.get("usageQuery") {
            if !value.is_object() {
                return Err(AccountStoreError::bad_request("批量查询设置必须是 JSON 对象"));
            }
            allowed.insert("usageQuery".to_string(), Self::normalize_usage_query(value)?);
        }
        let limiter_mode = patch_object
            .get("limiterMode")
            .map(BatchLimiterMode::parse)
            .transpose()?;
        let incoming_groups = patch_object.get("limiterGroups")
            .map(limiter::normalize_groups).transpose().map_err(AccountStoreError::bad_request)?;
        if let Some(groups) = &incoming_groups {
            if groups.iter().any(|group| group.conditions.iter().any(|condition| {
                !matches!(condition.source_id.as_str(), "" | "account" | "local")
            })) {
                return Err(AccountStoreError::bad_request("套餐实例不能跨账号复制，请使用通用条件"));
            }
            if groups.is_empty() && limiter_mode.is_some_and(BatchLimiterMode::needs_rules) {
                return Err(AccountStoreError::bad_request("请至少添加一个条件组"));
            }
        }
        let incoming_rules = if incoming_groups.is_none() && limiter_mode.is_some_and(BatchLimiterMode::needs_rules) {
            let value = patch_object
                .get("limiters")
                .filter(|value| value.is_array())
                .ok_or_else(|| AccountStoreError::bad_request("追加或替换限制器时必须提供规则数组"))?;
            let rules = limiter::normalize_rules(value, &Map::new())
                .map_err(AccountStoreError::bad_request)?;
            if rules.is_empty() {
                return Err(AccountStoreError::bad_request("请至少添加一条限制规则；清空请使用清空操作"));
            }
            rules
        } else {
            Vec::new()
        };
        if allowed.is_empty() && limiter_mode.is_none() {
            return Err(AccountStoreError::bad_request(
                "批量修改只支持启用状态、代理、并发上限、查询设置与限制器",
            ));
        }

        let group_needs_usage = incoming_groups.as_ref().is_some_and(|groups| groups.iter().any(|group|
            group.conditions.iter().any(|condition| !matches!(condition.metric, limiter::types::Metric::Tokens | limiter::types::Metric::Requests))));
        let needs_usage = allowed.contains_key("usageQuery")
            || (limiter_mode.is_some_and(BatchLimiterMode::needs_rules)
                && (group_needs_usage || incoming_rules.iter().any(|rule| rule.kind == limiter::LimiterKind::Balance)));
        let _guard = self.guard();
        // by need ** in that **（ apply_patch、
        // ok/failed）， this is 「 by need that 」 not is ：ids 1 
        // 1 。 in 3 when 3 ， not is 20 JSON
        // 1 。
        let mut ok: Vec<Value> = Vec::new();
        let mut failed: Vec<Value> = Vec::new();
        let mut skipped: Vec<Value> = Vec::new();
        // need （ in ， after 1 ）： has ， is 
        // 「 no has 」 not will 
        // （ old in that will file）。
        let mut dirty: Vec<StoredAccount> = Vec::new();
        // use CatPaw (id, provider)： 「 use 」 and ——
        // will must（ `store_crud::update_account` ）。
        // in 、 after （ not ）。
        let mut disabled_catpaw: Vec<(String, String)> = Vec::new();
        for id in ids {
            let Some(mut record) = self.record_by_id(&_guard, id) else {
                failed.push(json!({ "id": id, "error": "账号不存在" }));
                continue;
            };
            let name = record.name();
            let provider = record.provider();
            // and 「 this no has 」 1 （ `usage_query::supports_usage`）：
            // not and ， 1 set will to 1 not config ——
            // up will 「set」 and 。
            //
            // ****： `isGuest` in `pluginPublic` ，
            // `record.get("isGuest")` not to （ `plugin_accounts` ）。
            let guest = self
                .public_account(&record)
                .get("isGuest")
                .and_then(Value::as_bool)
                == Some(true);
            let has_usage = !needs_usage || (!guest && usage_query::supports_usage_provider(&provider));
            let mut account_patch = allowed.clone();
            if !has_usage {
                account_patch.remove("usageQuery");
            }
            if let Some(mode) = limiter_mode {
                if mode.needs_rules()
                    && !has_usage
                    && (group_needs_usage || incoming_rules.iter().any(|rule| rule.kind == limiter::LimiterKind::Balance))
                {
                    skipped.push(json!({
                        "id": id, "name": name,
                        "reason": "该提供商没有余额查询，不能应用余额限制规则",
                    }));
                    continue;
                }
                if incoming_groups.is_some() || record.fields().contains_key("limiterGroups") && !mode.needs_rules() {
                    let mut groups = limiter::effective_groups_in(record.fields());
                    match mode {
                        BatchLimiterMode::Append => for group in incoming_groups.iter().flatten() {
                            if !groups.contains(group) { groups.push(group.clone()); }
                        },
                        BatchLimiterMode::Replace => groups = incoming_groups.clone().unwrap_or_default(),
                        BatchLimiterMode::Enable | BatchLimiterMode::Disable => for group in &mut groups {
                            group.enabled = matches!(mode, BatchLimiterMode::Enable);
                        },
                        BatchLimiterMode::Clear => groups.clear(),
                    }
                    account_patch.insert("limiterGroups".to_string(), limiter::types::json_of(&groups));
                } else {
                    account_patch.insert("limiters".to_string(), mode.apply(record.fields(), &incoming_rules));
                }
            }
            if account_patch.is_empty() {
                skipped.push(json!({
                    "id": id, "name": name, "reason": "该提供商没有余额查询",
                }));
                continue;
            }
            let was_enabled = record.enabled();
            // ：** not ** priority（ up allowed
            // ）， this not will use —— to 1 success ，
            // `apply_patch` 1 （ need can " this patch can can "）。
            match Self::apply_patch(&mut record, &account_patch, |_| Ok(None)) {
                Ok(changes) => {
                    if !changes.is_empty() {
                        record.set_updated_at(logging::now_ms());
                        if was_enabled && !record.enabled() {
                            disabled_catpaw.push((id.clone(), provider));
                        }
                        dirty.push(record);
                    }
                    ok.push(json!({ "id": id, "name": name, "changes": changes }));
                }
                Err(error) => {
                    failed.push(json!({ "id": id, "name": name, "error": error.message }));
                }
            }
        }
        if !dirty.is_empty() {
            // 1 ： need 、 need not 。 / enabled when 
            // in failure will down 「 before 、 after no 」 in ， use to 
            // is 「 1 ， 1 」—— this can can 。
            self.with_conn_mut(&_guard, |conn| {
                let tx = conn.transaction()?;
                for record in &dirty {
                    sql::update_in_place(&tx, record)?;
                }
                tx.commit()
            })?;
        }
        // after （， `disabled_catpaw` ）
        drop(_guard);
        if limiter_mode.is_some() { limiter::counter::invalidate(); }
        for (id, provider) in disabled_catpaw {
            self.invalidate_catpaw_sessions(&id, &provider);
        }
        // need in after to （ before to is new ）。 is /enabled，
        // not and ， new 1 to is 。
        // this ** new 1 ** not is use up that ： that already in before 
        // （ not ， `disabled_catpaw` ）， in that when 
        // can can already —— + is ** down ** ，
        // not is 「 before that 1 」 。 1 in can 
        // （ is 、 ）。
        let snapshot = {
            let guard = self.guard();
            self.snapshot(&self.load(&guard), Visibility::UserFacing)
        };
        // need ： and Node 1 （enabled and proxy ）
        let mut summary: Vec<String> = Vec::new();
        if patch_object.contains_key("enabled") {
            summary.push(format!(
                "启用状态 → {}",
                if enabled_value { "启用" } else { "禁用" }
            ));
        }
        if let Some(proxy) = allowed.get("proxy") {
            // none when 「none（）」，otherwise use describe label
            let label = if proxy.is_null() {
                "无代理（直连）".to_string()
            } else {
                describe_account_proxy(Some(proxy))
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or("已设置")
                    .to_string()
            };
            summary.push(format!("代理 → {label}"));
        }
        if let Some(value) = allowed.get("maxConcurrent").and_then(Value::as_u64) {
            summary.push(if value == 0 {
                "并发上限 → 不限".to_string()
            } else {
                format!("并发上限 → {value}")
            });
        }
        if let Some(value) = allowed.get("usageQuery") {
            summary.push(Self::describe_usage_query(value));
        }
        if let Some(mode) = limiter_mode {
            summary.push(mode.summary().to_string());
        }
        let changed_count = ok
            .iter()
            .filter(|item| {
                item.get("changes")
                    .and_then(Value::as_array)
                    .map(|changes| !changes.is_empty())
                    .unwrap_or(false)
            })
            .count();
        logging::log(
            "[Accounts]",
            &format!(
                "🔀 批量修改 {} 个账号（{}）: 成功 {changed_count} 个{}{}",
                ids.len(),
                summary.join("，"),
                if failed.is_empty() {
                    String::new()
                } else {
                    format!("，失败 {} 个", failed.len())
                },
                if skipped.is_empty() {
                    String::new()
                } else {
                    format!("，跳过 {} 个", skipped.len())
                }
            ),
        );
        Ok(json!({ "ok": ok, "failed": failed, "skipped": skipped, "list": snapshot }))
    }

    /// delete。 not in id failed not is —— use delete
    /// （ or already new ） not will to its failure。current by ，
    /// after down 1 available，none need 。
    ///
    /// ── delete after CatPaw will （W5-T-d4）──────────────────
    /// and `remove_account` 1 ：conversationId up up down ，
    /// no not can 。provider in **delete before ** 1 
    /// （ after can to None）， in 、 after 
    /// （ not ）。
    pub fn batch_remove(&self, ids: &[String]) -> Result<Value, AccountStoreError> {
        if ids.is_empty() {
            return Err(AccountStoreError::bad_request("缺少要删除的账号 id"));
        }
        // must in ** before **（`protected_from_removal`
        // will ， std::sync::Mutex not can —— will ）
        let protected: Vec<(String, String)> = ids
            .iter()
            .filter_map(|id| {
                self.protected_from_removal(id)
                    .map(|reason| (id.clone(), reason))
            })
            .collect();
        let _guard = self.guard();
        let mut removed: Vec<Value> = Vec::new();
        let mut failed: Vec<Value> = Vec::new();
        // ( id, provider)： need use （ after not to ）
        let mut removed_owners: Vec<(String, String)> = Vec::new();
        // id：（ / not in ）， after 1 。
        // 「 in 」 use （ not ）， in 。
        let mut targets: Vec<String> = Vec::new();
        for id in ids {
            // （ when ） and 「 not in 」 1 failed，
            // not in —— and remove_account 1 
            if let Some((_, reason)) = protected.iter().find(|(target, _)| target == id) {
                failed.push(json!({ "id": id, "error": reason }));
                continue;
            }
            match self.record_by_id(&_guard, id) {
                Some(record) => {
                    removed.push(json!({ "id": id, "name": record.name() }));
                    removed_owners.push((id.clone(), record.provider()));
                    targets.push(id.clone());
                }
                None => failed.push(json!({ "id": id, "error": "账号不存在" })),
            }
        }
        if targets.is_empty() {
            // no has 1 can ： not ， to current
            let state = self.load(&_guard);
            return Ok(json!({ "removed": removed, "failed": failed, "list": self.snapshot(&state, Visibility::UserFacing) }));
        }
        // 1 ：delete need 、 need 1 not 
        self.with_conn_mut(&_guard, |conn| {
            let tx = conn.transaction()?;
            for id in &targets {
                sql::delete(&tx, id)?;
            }
            tx.commit()
        })?;
        let state = self.load(&_guard);
        let snapshot = self.snapshot(&state, Visibility::UserFacing);
        let names: String = removed
            .iter()
            .filter_map(|item| item.get("name").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("、");
        logging::log(
            "[Accounts]",
            &format!(
                "🗑️  批量删除 {} 个账号: {}{}",
                removed.len(),
                truncate_chars(&names, 200),
                if failed.is_empty() {
                    String::new()
                } else {
                    format!("（{} 个不存在，已跳过）", failed.len())
                }
            ),
        );
        // after this CatPaw will （，）
        drop(_guard);
        for (id, provider) in removed_owners {
            self.invalidate_catpaw_sessions(&id, &provider);
        }
        Ok(json!({ "removed": removed, "failed": failed, "list": snapshot }))
    }
}
