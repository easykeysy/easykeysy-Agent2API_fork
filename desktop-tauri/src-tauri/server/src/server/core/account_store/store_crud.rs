//! （ from store.rs ，file）。
//!
//! Node workbuddy-account-store.mjs ：
//!   addAccount / removeAccount / promoteToFront / updateAccount(applyAccountPatch)
//!   / moveAccount / batchUpdate / batchRemove
//!
//! not in this ， before `super::mod` ：
//! 1. ** 1 **（all use 1 ）： to 1 409，
//! by use 1 empty； and in up 
//! （ after by 1 ：`sql::priority_holder` /
//! `sql::priorities_except`）， `renumber_consecutively`；
//! 2. **unknown**： is JSON for （`StoredAccount`）， need ；
//! 3. ** not request**：file is ， no has await。
//!
//! ── ： 1 （ ）────────────────
//! before path is 「load → that 1 → save 」。 in is ：
//! - ：`record_by_id` / `records_for_provider` by need （ not ）；
//! - and ：`priority_holder` / `priorities_except`
//! in up （ not JSON 1 ）；
//! - ：`sql::put` / `sql::update_in_place` / `sql::delete` 。
//!
//! ****（ need or ， ）：
//! - `promote_to_front`： after its must， 1 new ；
//! - `move_account`： can can is ** 1 ** （ 1 ），
//! must by 。
//! **** is （ need ），**** that 1 
//! （`move_account` UPDATE in 1 ；`promote_to_front` `save`，
//! `save` is by ， no 1 UPDATE not ）。
//!
//! add `provider` 1 default provider（`DEFAULT_PROVIDER_ID`）：
//! **workbuddy that 1 path**（ its in 
//! `raccoon_accounts.rs`， and catpaw / autoclaw / cline / qoder ）， and 
//! ** not payload provider** —— before 1 unknown provider 
//! no 。 in `api::accounts::add_account`：
//! by payload provider kind， to 。

use serde_json::{json, Map, Value};

use crate::server::core::account_store::priority::{
    grouped_queue_order, next_free_priority, normalize_priority, renumber_consecutively,
    QueueEntry, DEFAULT_PRIORITY,
};
use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::{mark_name_custom, StoredAccount};
use crate::server::core::account_store::store_view::Visibility;
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};
use crate::server::core::account_store::store_util::{
    js_string, js_truthy, number_or, object_or_empty, optional_text, pick_token, token_tail_of,
    truncate_chars, value_or, value_or_nullish,
};
use crate::server::core::account_store::MAX_TOKEN_LENGTH;
use crate::server::core::endpoints::resolve_edition;
use crate::server::core::limiter;
use crate::server::core::providers::DEFAULT_PROVIDER_ID;
use crate::server::core::proxies::describe_account_proxy;
use crate::server::logging;

/// and limit （0 = not ）。 use is 1 
/// （ui-islands `CONC_MAX`）； need for up ，otherwise will 「、 400」。
pub(crate) const MAX_CONCURRENT_LIMIT: u64 = 999;

/// 1 `maxConcurrent` ：must is ****（`as_u64` for / /
/// 1 return None）、 [`MAX_CONCURRENT_LIMIT`]。
///
/// （`apply_patch`） and （`batch_update`） use this 1 ： if 
/// 1 ，「set by 999 、 1 5000」。
pub(crate) fn normalize_max_concurrent(value: &Value) -> Result<u64, AccountStoreError> {
    match value.as_u64() {
        Some(number) if number <= MAX_CONCURRENT_LIMIT => Ok(number),
        // has to this ；， use for 「」limit
        Some(_) => Err(AccountStoreError::bad_request(
            "并发上限必须是不大于 999 的非负整数",
        )),
        None => Err(AccountStoreError::bad_request("并发上限必须是非负整数")),
    }
}

impl AccountStore {
    // ─── CRUD ────────────────────────────────────────────────

    /// /update（ for Node `addAccount`）。
    ///
    /// session （auth + account） or （accessToken/refreshToken，
    /// can uid/nickname/expiresAt/domain）。edition /prefixPath/platform
    /// default； endpoint/prefixPath/platform 。
    /// new 「 has + 1」 in ， not will current。
    ///
    /// ── `provider` is ****， not is optional（2026-10 WorkBuddy ）──────
    /// workbuddy （`workbuddy` / `workbuddy-intl`
    /// ）， by use to ： by use 、 by request 
    /// provider。**update has when use **（ will etc ，
    /// that not is this ）； new when use ， is `workbuddy`
    /// （：client not provider ）。
    ///
    /// `provider` workbuddy id（ is this 1 ：
    /// `uid` + accessToken/refreshToken + edition）。 path has 
    /// （`add_raccoon_account` / `add_autoclaw_account` / …）。
    pub fn add_account(
        &self,
        payload: &Value,
        name: Option<&str>,
        provider: Option<&str>,
    ) -> Result<Value, AccountStoreError> {
        let requested_provider = provider
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_PROVIDER_ID);
        let Some(payload_object) = payload.as_object() else {
            return Err(AccountStoreError::bad_request("账号内容必须是 JSON 对象"));
        };
        let auth = object_or_empty(payload_object.get("auth"));
        let account = object_or_empty(payload_object.get("account"));

        // （clientfile / wb2api ） `auth` for ； no has 
        // —— wb2api （`{"accessToken":…,"refreshToken":…}`）
        // no has `auth` ， before in this 「 accessToken」。
        let access_token = {
            let nested = pick_token(&auth, &["accessToken", "token", "access_token"]);
            if nested.is_empty() {
                pick_token(payload_object, &["accessToken", "token", "access_token"])
            } else {
                nested
            }
        };
        let refresh_token = {
            let nested = pick_token(&auth, &["refreshToken", "refresh_token"]);
            if nested.is_empty() {
                pick_token(payload_object, &["refreshToken", "refresh_token"])
            } else {
                nested
            }
        };
        if access_token.is_empty() {
            return Err(AccountStoreError::bad_request("缺少 accessToken"));
        }
        if access_token.chars().count() > MAX_TOKEN_LENGTH
            || refresh_token.chars().count() > MAX_TOKEN_LENGTH
        {
            return Err(AccountStoreError::bad_request("token 过长"));
        }
        let uid = {
            let candidate = value_or(
                account.get("uid").or_else(|| payload_object.get("uid")),
                Value::String(String::new()),
            );
            js_string(&candidate).trim().to_string()
        };
        if uid.is_empty() {
            return Err(AccountStoreError::bad_request("缺少 uid（无法标识账号）"));
        }

        let _guard = self.guard();
        let id = format!("user-{uid}");
        // this 1 （ not ）： has is 「update」 is 「 new 」
        let existing = self.record_by_id(&_guard, &id);
        // ── id （Agent2API W3-T4）────────────────────────────
        // id is `user-<>` （`add_raccoon_account` and 
        // old this ）， userId can can and workbuddy 
        // uid （ id empty）。 if this use has ， this workbuddy will 
        // will **** workbuddy —— and 1 。
        // to 1 ** provider** id when ， use that 。
        //
        // WorkBuddy （2026-10） after from 「must etc workbuddy」
        // 「must etc ** need that 1 **」： and is ， 1 uid
        // in is （`user-<uid>` in this down ——
        // WorkBuddy uid is up uuid， not will ）。
        if let Some(existing) = existing.as_ref() {
            let existing_provider = existing.provider();
            if existing_provider != requested_provider {
                return Err(AccountStoreError::bad_request(format!(
                    "账号 id「{id}」已被{}账号占用，无法用同一 uid 添加{}账号（请先处理那个账号）",
                    crate::server::core::providers::label_of(&existing_provider),
                    crate::server::core::providers::label_of(requested_provider)
                )));
            }
        }
        // provider：update has when ** use **（ etc 
        // ， that not is ）， new when use use to that 1 （ workbuddy，
        // ：client not provider ）。
        // ：provider by `StoredAccount::provider()` 
        // workbuddy， this not will to empty。
        // path in `raccoon_accounts::add_raccoon_account`（ and 
        // not ）， workbuddy ； id to （ has 
        // workbuddy 、 userId ） by that path 
        // 「 has unknown」： not will workbuddy raccoon。
        let provider = existing
            .as_ref()
            .map(|record| record.provider().to_string())
            .unwrap_or_else(|| requested_provider.to_string());

        // / in before ：， not down 
        let resolved_proxy = if payload_object.contains_key("proxy") {
            crate::server::core::proxies::normalize_account_proxy(
                payload_object.get("proxy").unwrap_or(&Value::Null),
            )
            .map_err(|error| AccountStoreError::new(error.message, error.status_code))?
            .unwrap_or(Value::Null)
        } else {
            existing
                .as_ref()
                .map(|record| value_or_nullish(record.get("proxy"), Value::Null))
                .unwrap_or(Value::Null)
        };
        // ── （edition） by ****（2026-10 ）──────────
        // to use （ 「」 down 、 will context），
        // has no to use 。 no has when by need that 1 ：
        // `workbuddy-intl` → ， its → （default）。
        // this 1 not can ： to 1 /prefixPath/platform，
        // etc ， 401， is 「 not can use 」。
        let edition = {
            let explicit = payload_object
                .get("edition")
                .filter(|value| !value.is_null())
                .map(js_string)
                .or_else(|| existing.as_ref().and_then(StoredAccount::edition));
            match explicit {
                Some(value) => resolve_edition(Some(value.as_str())),
                None => crate::server::core::providers::workbuddy::Region::from_provider_id(
                    requested_provider,
                )
                .map(|region| region.edition())
                .unwrap_or_else(|| resolve_edition(None)),
            }
        };
        // new default in ，empty has ； then 1 。
        // and （ 1 ，module header）—— in 
        // `priority` up ， not JSON：
        // `priorities_except(&id)`（ old `others` ），
        // `priority_holder`（ 1 SQL 「 + find」）。
        let existing_priority = existing.as_ref().map(StoredAccount::priority);
        let priority = match payload_object.get("priority") {
            Some(value) => normalize_priority(
                Some(value),
                existing_priority.unwrap_or(DEFAULT_PRIORITY),
            ),
            None => match existing_priority {
                Some(value) => value,
                None => {
                    let used = self.with_conn(&_guard, |conn| sql::priorities_except(conn, &id))?;
                    next_free_priority(&used)
                }
            },
        };
        if let Some(holder_name) = self.with_conn(&_guard, |conn| {
            sql::priority_holder(conn, priority, &id)
        })? {
            return Err(AccountStoreError::new(
                format!(
                    "优先级 {priority} 已被账号「{holder_name}」占用，请换一个\
                     （优先级全局唯一）"
                ),
                409,
            ));
        }

        let mut record = Map::new();
        record.insert("id".to_string(), Value::String(id.clone()));
        // provider is ** already **： when must up （ not 「unknown」
        // need —— already not can in when ）
        record.insert("provider".to_string(), Value::String(provider.clone()));
        // ：， its 、 has ， after by uid 
        let explicit_name = name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| truncate_chars(value, 100))
            .filter(|value| !value.is_empty());
        let explicit = explicit_name.is_some();
        let record_name = explicit_name
            .or_else(|| optional_text(account.get("nickname")))
            .or_else(|| optional_text(payload_object.get("nickname")))
            .or_else(|| existing.as_ref().and_then(|item| optional_text(item.get("name"))))
            .unwrap_or_else(|| format!("账号 {}", truncate_chars(&uid, 8)));
        record.insert("name".to_string(), Value::String(record_name.clone()));
        mark_name_custom(&mut record, explicit, existing.as_ref());
        record.insert(
            "uid".to_string(),
            Value::String(uid.clone()),
        );
        record.insert(
            "nickname".to_string(),
            Value::String(
                optional_text(account.get("nickname"))
                    .or_else(|| optional_text(payload_object.get("nickname")))
                    .or_else(|| existing.as_ref().map(StoredAccount::nickname))
                    .unwrap_or_default(),
            ),
        );
        record.insert(
            "type".to_string(),
            Value::String(
                optional_text(account.get("type"))
                    .or_else(|| optional_text(payload_object.get("type")))
                    .or_else(|| existing.as_ref().map(StoredAccount::account_type))
                    .unwrap_or_else(|| "personal".to_string()),
            ),
        );
        for (key, from_account, from_payload) in [
            ("enterpriseId", "enterpriseId", "enterpriseId"),
            ("enterpriseName", "enterpriseName", "enterpriseName"),
        ] {
            let value = optional_text(account.get(from_account))
                .or_else(|| optional_text(payload_object.get(from_payload)))
                .or_else(|| {
                    existing
                        .as_ref()
                        .and_then(|item| optional_text(item.get(key)))
                })
                .unwrap_or_default();
            record.insert(key.to_string(), Value::String(value));
        }
        record.insert("accessToken".to_string(), Value::String(access_token.clone()));
        record.insert(
            "refreshToken".to_string(),
            Value::String(if refresh_token.is_empty() {
                existing
                    .as_ref()
                    .map(StoredAccount::refresh_token)
                    .unwrap_or_default()
            } else {
                refresh_token
            }),
        );
        record.insert(
            "tokenTail".to_string(),
            Value::String(token_tail_of(&access_token)),
        );
        record.insert(
            "expiresAt".to_string(),
            number_or(
                auth.get("expiresAt").or_else(|| payload_object.get("expiresAt")),
                existing
                    .as_ref()
                    .and_then(StoredAccount::expires_at)
                    .map(Value::from)
                    .unwrap_or(Value::Null),
            ),
        );
        record.insert(
            "refreshExpiresAt".to_string(),
            number_or(
                auth
                    .get("refreshExpiresAt")
                    .or_else(|| payload_object.get("refreshExpiresAt")),
                existing
                    .as_ref()
                    .and_then(StoredAccount::refresh_expires_at)
                    .map(Value::from)
                    .unwrap_or(Value::Null),
            ),
        );
        record.insert(
            "domain".to_string(),
            Value::String(
                optional_text(auth.get("domain"))
                    .or_else(|| existing.as_ref().map(StoredAccount::domain))
                    .unwrap_or_default(),
            ),
        );
        record.insert("edition".to_string(), Value::String(edition.id.to_string()));
        // has this ：payload no to use （Node use `??`，
        // empty is valid、 has null/ default）
        let existing_prefix = existing.as_ref().and_then(|item| item.get("prefixPath")).cloned();
        let existing_endpoint = existing.as_ref().and_then(|item| item.get("endpoint")).cloned();
        let existing_platform = existing.as_ref().and_then(|item| item.get("platform")).cloned();
        record.insert(
            "prefixPath".to_string(),
            value_or_nullish(
                payload_object.get("prefixPath").or(existing_prefix.as_ref()),
                Value::String(String::new()),
            ),
        );
        record.insert(
            "endpoint".to_string(),
            value_or_nullish(
                payload_object.get("endpoint").or(existing_endpoint.as_ref()),
                Value::String(String::new()),
            ),
        );
        record.insert(
            "platform".to_string(),
            value_or_nullish(
                payload_object.get("platform").or(existing_platform.as_ref()),
                Value::String(String::new()),
            ),
        );
        record.insert("priority".to_string(), Value::from(priority));
        let enabled = match payload_object.get("enabled") {
            Some(value) => !matches!(value, Value::Bool(false)),
            None => existing.as_ref().map(StoredAccount::enabled).unwrap_or(true),
        };
        record.insert("enabled".to_string(), Value::Bool(enabled));
        record.insert("proxy".to_string(), resolved_proxy);
        let added_at = existing
            .as_ref()
            .map(StoredAccount::added_at)
            .filter(|value| *value != 0)
            .unwrap_or_else(logging::now_ms);
        record.insert("addedAt".to_string(), Value::from(added_at));
        record.insert("updatedAt".to_string(), Value::from(logging::now_ms()));

        let saved = StoredAccount::from_map(record);
        // ：`put` is DELETE + INSERT， is 「update has when in 
        // after 」 this old as （`retain` + `push`）（ `sql::put`）。
        self.with_conn(&_guard, |conn| sql::put(conn, &saved))?;
        logging::log(
            "[Accounts]",
            &format!(
                "✅ 账号已保存: {}（{}…，{}，优先级 {}）",
                record_name,
                truncate_chars(&uid, 8),
                edition.label,
                priority
            ),
        );
        Ok(self.public_account(&saved))
    }

    /// this is 、 not can delete。return `Some(Reason)` delete。
    ///
    /// ── in ** not **（）────────────────
    /// （ / CatPaw / AutoClaw） when in this 
    /// delete，** already **。 by for is 1 ： is 「
    /// 」 this ** 1 **， not token、
    /// when client file。 is 「 need use this clientcurrent 」
    /// —— use can can this 。 old 「 not can delete」 use ： to 「 use 」
    /// 1 ， use after this 、。
    ///
    /// delete**used for this **：client file from 、 from not ，
    /// is and can —— use ， 1 「」 can 。
    ///
    /// ── Why this return None ────────────────────────
    /// use （`remove_account` / `batch_remove`）「 can not can 」、 not 
    /// provider ， this is 1 ** **： will if 「 not 」 
    /// （e.g. not can ）， this 1 can ， use 1 not use 。
    /// in `raccoon_accounts.rs`（ that ）；file
    /// 「」 after ， this and provider none by this 。
    ///
    /// `_id`： not ，parameter will has unused （ use not ）。
    pub(crate) fn protected_from_removal(&self, _id: &str) -> Option<String> {
        None
    }

    /// delete（ not in → 404； → 400）。
    ///
    /// ── ** can **delete（W3-T4 ， after ）─────────────
    /// / CatPaw / AutoClaw is 「file」 ****
    /// （ §3.2 / §9 / §10）， as not can 。 that is 
    /// ： is ** this **， from not not client file，
    /// delete and can （ 1 「」 can ），
    /// 「 not can 」 use —— can use ， use after and 
    /// 。 in not （ `protected_from_removal` as 
    /// ，currentreturn None）。
    ///
    /// ── delete after must this CatPaw will （W5-T-d4）───────
    /// `conversationId` ** up up down **： no ， will not can 
    /// （ will to will or ）。 in when 
    /// （`notifySwitch` → `clearClientToolSessions`）， this by 。
    /// ── deletemust ─────────────────────────────────────────
    /// delete is ** 1 ** not deletepath：delete has 「🗑️ delete」、
    /// has 「[Auth] already current」， has not —— after 
    /// when and none from （2026-09-28 1 is this not ）。
    /// and delete for ， and up **id**： will 、 can can ，id is can for up 
    /// request and that 。
    ///
    /// provider and need in **delete before **： after can to None。
    pub fn remove_account(&self, id: &str) -> Result<(), AccountStoreError> {
        if let Some(reason) = self.protected_from_removal(id) {
            return Err(AccountStoreError::new(reason, 400));
        }
        let _guard = self.guard();
        // provider and need in **delete before **（ after can to None）：
        // before to `invalidate_catpaw_sessions`， after to down that delete
        let record = self.record_by_id(&_guard, id);
        let provider = record
            .as_ref()
            .map(|record| record.provider())
            .unwrap_or_default();
        // as empty when use id ： can ， not need 「 already delete: （user-…）」
        let name = record
            .as_ref()
            .map(|record| record.name())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| id.to_string());
        // delete，`false` no has this 1 → 404（ and old 
        // etc ： before not to this id）
        let removed = self.with_conn(&_guard, |conn| sql::delete(conn, id))?;
        if !removed {
            return Err(AccountStoreError::not_found("账号不存在"));
        }
        // after 1 （ and delete 1 ）
        logging::log(
            "[Accounts]",
            &format!("🗑️  账号已删除: {name}（{id}，{provider}）"),
        );
        // already ， in this ： is 1 ，
        // not （ not this ）（ `invalidate_catpaw_sessions` ）
        drop(_guard);
        self.invalidate_catpaw_sessions(id, &provider);
        Ok(())
    }

    /// to 1 ， its for not 。
    ///
    /// ， not enabled； use not and 。
    /// `currentAccountId` 「 enabled and has available 」 ，
    /// not 1 to 。
    ///
    /// ── Why this 1 must、 ──────────────────────────
    /// its 1 ， this 1 not ： 1 before down ， to 
    /// after ** its must**，otherwise will （`renumber_consecutively`
    /// to new ）。 by need 1 ——
    /// **** that ：`save_state` is by ，
    /// no 1 UPDATE not will （e.g. [0,1,2] when 
    /// 2→0/0→1/1→2 ， [0,5,9] when has 1 ）。
    pub fn promote_to_front(&self, id: &str) -> Result<Value, AccountStoreError> {
        let _guard = self.guard();
        let mut state = self.load(&_guard);
        if !state.accounts.iter().any(|item| item.id() == id) {
            return Err(AccountStoreError::not_found("账号不存在"));
        }

        // ── 「」 in down （2026-10 ）──────────
        // is ： in before when ， not up 1 。
        // ：** to before + to before ** —— this 
        // is 1 ， up 「 1 」 and 1 。
        // from `grouped_queue_order` （ not is by ）， is 
        // in ， this not not will 1 「 as 」。
        let target_provider = state
            .accounts
            .iter()
            .find(|item| item.id() == id)
            .map(StoredAccount::provider)
            .unwrap_or_default();
        let providers: Vec<String> = state.accounts.iter().map(StoredAccount::provider).collect();
        let entries: Vec<QueueEntry<'_>> = state
            .accounts
            .iter()
            .enumerate()
            .map(|(index, record)| QueueEntry {
                id: record.id(),
                provider: &providers[index],
                priority: record.priority(),
                added_at: record.added_at(),
            })
            .collect();
        let current_order = grouped_queue_order(&entries);
        let mut mine: Vec<String> = Vec::new();
        let mut others: Vec<String> = Vec::new();
        for item_id in current_order.iter() {
            let same_provider = state
                .accounts
                .iter()
                .find(|item| item.id() == item_id.as_str())
                .map(|item| item.provider() == target_provider)
                .unwrap_or(false);
            if same_provider {
                mine.push(item_id.clone());
            } else {
                others.push(item_id.clone());
            }
        }
        mine.retain(|item_id| item_id != id);
        mine.insert(0, id.to_string());
        let next_order: Vec<String> = mine.into_iter().chain(others).collect();
        let mut changes: Vec<String> = Vec::new();

        if next_order != current_order {
            // by new （ 1 + 1 ， up ）
            let mut ordered: Vec<StoredAccount> = next_order
                .iter()
                .filter_map(|item_id| {
                    state
                        .accounts
                        .iter()
                        .find(|item| item.id() == item_id.as_str())
                        .cloned()
                })
                .collect();
            let mut numbering: Vec<(String, String, i64)> = ordered
                .iter()
                .map(|item| (item.id().to_string(), item.name(), item.priority()))
                .collect();
            let assignments = renumber_consecutively(&mut numbering);
            // up ： （default etc when not ）
            for assignment in &assignments {
                if let Some(record) = ordered.iter_mut().find(|item| item.id() == assignment.id) {
                    record.set_priority(assignment.to);
                }
            }
            changes.push(format!(
                "优先级 → {}（本组置顶 + 本组排到最前）",
                ordered.first().map(StoredAccount::priority).unwrap_or(DEFAULT_PRIORITY)
            ));
            if let Some(record) = ordered.iter_mut().find(|item| item.id() == id) {
                record.set_updated_at(logging::now_ms());
            }
            state.accounts = ordered;
        }
        if changes.is_empty() {
            return Ok(json!({
                "id": id,
                "changed": false,
                "reason": "已在队列第一位",
                "currentAccountId": Self::pick_current(&state.accounts)
                    .map(|record| Value::String(record.id().to_string()))
                    .unwrap_or(Value::Null),
                "list": self.snapshot(&state, Visibility::UserFacing),
            }));
        }

        let display_name = state
            .accounts
            .iter()
            .find(|item| item.id() == id)
            .map(StoredAccount::name)
            .unwrap_or_default();
        // already by new 、 already （ up that 1 ）， to ：
        // `save_state` for data UPDATE， is 「 not no 」
        // this in this 。（）** not ** —— and 
        // by ， no has （ `sql.rs` module header「」 1 ）。
        self.save(&state, &_guard)?;
        let current_id = Self::pick_current(&state.accounts).map(|record| record.id().to_string());
        logging::log(
            "[Accounts]",
            &format!("⬆️  账号已置顶: {display_name}（{}）", changes.join("，")),
        );
        let mut result = json!({
            "id": id,
            "changed": true,
            "changes": changes,
            "currentAccountId": current_id,
        });
        if let Some(object) = result.as_object_mut() {
            object.insert("list".to_string(), self.snapshot(&state, Visibility::UserFacing));
        }
        Ok(result)
    }

    /// ： / / enabled / 。
    ///
    /// （patch ）， not 。
    /// return `(account, changes)`，changes as （ and before ）。
    ///
    /// ── return `account` use is ** before ** updatedAt（）──
    /// before is this ：`public_account` in `set_updated_at` ** before **，
    /// response `updatedAt` is old （ down 1 to new ）。 this 
    /// —— before can can 「 this 1 is not is 」 ，「」 will response
    /// and Node 。
    ///
    /// ── use when CatPaw will （W5-T-d4）──────────────────
    /// 「 use 」 is 「 not use 」。 conversationId is 
    /// up up down for ：， use new enabled after that 1 will 
    /// use 1 can can expired conversation（ up TTL / ）。
    /// use （`enabled` false） and delete、 new 1 need this 。
    /// enabled** not **（ that will use 1 ， is ）。
    pub fn update_account(
        &self,
        id: &str,
        patch: &Value,
    ) -> Result<(Value, Vec<String>), AccountStoreError> {
        let patch = patch
            .as_object()
            .cloned()
            .ok_or_else(|| AccountStoreError::bad_request("请求内容必须是 JSON 对象"))?;
        let _guard = self.guard();
        // that 1 ：apply_patch this 1 ， 1 
        // （ old as need find 1 ）
        let mut record = self
            .record_by_id(&_guard, id)
            .ok_or_else(|| AccountStoreError::not_found("账号不存在"))?;
        let provider = record.provider();
        let was_enabled = record.enabled();
        let changes = Self::apply_patch(&mut record, &patch, |next| {
            self.with_conn(&_guard, |conn| sql::priority_holder(conn, next, id))
        })?;
        let account = self.public_account(&record);
        if changes.is_empty() {
            return Ok((account, changes));
        }
        let now_disabled = !record.enabled();
        let name = record.name();
        record.set_updated_at(logging::now_ms());
        // update（ not ）： old in down up ， not ——
        // up is by ， not this as 1 「」
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
        logging::log(
            "[Accounts]",
            &format!("✏️  账号已更新: {name}（{}）", changes.join("，")),
        );
        let limiter_changed = patch.contains_key("limiterGroups") || patch.contains_key("limiters");
        if was_enabled && now_disabled || limiter_changed {
            drop(_guard);
            if was_enabled && now_disabled { self.invalidate_catpaw_sessions(id, &provider); }
            if limiter_changed { limiter::counter::refresh_account(&account); }
        }
        Ok((account, changes))
    }

    /// patch use to ****（： not 、 not ）。
    ///
    /// `update_account` and `batch_update` use this ， and 
    /// 1 。 by error， use is failure is failed。
    ///
    /// ── `find_holder` is （ ）────────────
    /// need 「 in **** has no has this 」，
    /// has 1 —— before is `AccountState`， is use 
    /// must 。 in that ： use to 「 by 
    /// 」 （ 1 SQL， 1 ），
    /// and 409 ** in 1 **。
    ///
    /// in ** need ** when use （ no not use ），「patch 
    /// 」 this down 1 not will 。
    ///
    /// `pub(crate)`： in `store_batch.rs`（ 1 `impl` 1 ），
    /// has for **** not can —— when can to this 。
    pub(crate) fn apply_patch(
        record: &mut StoredAccount,
        patch: &Map<String, Value>,
        find_holder: impl FnOnce(i64) -> Result<Option<String>, AccountStoreError>,
    ) -> Result<Vec<String>, AccountStoreError> {
        let mut changes = Vec::new();

        if let Some(value) = patch.get("name") {
            let next = match value {
                Value::String(text) => truncate_chars(text.trim(), 100),
                _ => String::new(),
            };
            let current = record.name();
            if next != current {
                if next.is_empty() {
                    return Err(AccountStoreError::bad_request("备注名不能为空"));
                }
                record.set("name", Value::String(next.clone()));
                // use ** to ** when （set will name ，
                // not can by 「」，otherwise from anyset after will 
                // ）。 and use in none， this 
                // 「default」 is 「 as 」（ displayNameOf）
                record.set("nameCustom", Value::Bool(true));
                changes.push(format!("备注名 → {next}"));
            }
        }

        if let Some(value) = patch.get("priority") {
            let current = record.priority();
            let next = normalize_priority(Some(value), current);
            if next != current {
                // （ 1 ，module header）。
                // excluding： not this old 
                if let Some(holder_name) = find_holder(next)? {
                    return Err(AccountStoreError::new(
                        format!(
                            "优先级 {next} 已被账号「{holder_name}」占用，请换一个\
                             （优先级全局唯一）"
                        ),
                        409,
                    ));
                }
                record.set_priority(next);
                changes.push(format!("优先级 → {next}"));
            }
        }

        if let Some(value) = patch.get("enabled") {
            let next = !matches!(value, Value::Bool(false));
            let current = record.enabled();
            if next != current {
                record.set_enabled(next);
                if next && !record.get("upstreamCooldownUntil").and_then(Value::as_i64)
                    .is_some_and(|until| until > logging::now_ms()) {
                    record.remove("upstreamBlockedReason");
                }
                changes.push(if next { "已启用".to_string() } else { "已禁用".to_string() });
            }
        }

        if let Some(value) = patch.get("allowRouteSkip") {
            let Some(next) = value.as_bool() else {
                return Err(AccountStoreError::bad_request("allowRouteSkip 必须是布尔值"));
            };
            let current = record.get("allowRouteSkip").and_then(Value::as_bool).unwrap_or(true);
            if next != current {
                record.set("allowRouteSkip", Value::Bool(next));
                changes.push(if next { "允许失败后换号".to_string() } else { "失败后不换号".to_string() });
            }
        }

        if let Some(value) = patch.get("proxy") {
            let next = crate::server::core::proxies::normalize_account_proxy(value)
                .map_err(|error| AccountStoreError::new(error.message, error.status_code))?
                .unwrap_or(Value::Null);
            // Node use JSON.stringify ：（）。
            // serde_json by ， and JS for 1 。
            let before = value_or_nullish(record.get("proxy"), Value::Null);
            if next != before {
                // ── 「」 1 （）────────────────
                // for OpenCode Zen has ： that not use API Key、** by IP
                // **（ IP use for this IP 1 429）， is 
                // ； by Key / ， will up to 1 
                // not IP， use 。 in this not is 1 ： 1 not 
                // ，「 this is 」 has （`is_opencode_zen_account`
                // when excluding not OpenCode Go）。
                //
                // in ** ** when （ not is ）：set
                // will proxy ， can can 
                // （ not this ， down ）—— that not this 
                // 1 and none then down 。
                // （store_batch apply_patch）， that Zen 
                // will failed， its 。
                //
                // **（account_transfer） not this **：file need can 
                // ， that 1 can can no has that （ will ，
                // 1 in ）； that down use will by 「 no has 
                // 」， use Reason can 。
                if crate::server::core::proxies::is_rotate_config(&next)
                    && !crate::server::core::custom_providers::is_opencode_zen_account(
                        &record.provider(),
                        record.get("baseUrl").and_then(Value::as_str),
                    )
                {
                    return Err(AccountStoreError::bad_request(
                        "轮询出网只对 OpenCode Zen 账号开放：该上游按 IP 限流，\
                         轮换出口可以绕开单个 IP 的 429；别的提供商请用「已保存的代理」",
                    ));
                }
                record.set_proxy(next.clone());
                let label = if next.is_null() {
                    "代理 → 无代理（直连）".to_string()
                } else {
                    let described = describe_account_proxy(Some(&next));
                    let label = described
                        .get("label")
                        .and_then(Value::as_str)
                        .unwrap_or("已设置");
                    format!("代理 → {label}")
                };
                changes.push(label);
            }
        }

        if let Some(value) = patch.get("maxConcurrent") {
            // and limit：0 = not ， 999（ and 1 ，
            // `normalize_max_concurrent`）。 when 
            // `routing::max_concurrent_of`（ = 0 = not ）。
            let next = normalize_max_concurrent(value)?;
            // none = not （ by 0 ）， this 
            let current = record.get("maxConcurrent").and_then(Value::as_u64).unwrap_or(0);
            if next != current {
                record.set("maxConcurrent", Value::from(next));
                changes.push(if next == 0 {
                    "并发上限 → 不限".to_string()
                } else {
                    format!("并发上限 → {next}")
                });
            }
        }

        if let Some(value) = patch.get("usageQuery") {
            // （enabled + interval seconds）： set
            // ， and name / proxy 1 。 1 after for 
            // （ = {enabled:false, interval:0}），（ / ）
            // not 「」。 and （usage_records），
            // otherwise「 29 seconds」。
            let next = Self::normalize_usage_query(value)?;
            let current = Self::normalize_usage_query_lenient(record.get("usageQuery"));
            if next != current {
                record.set("usageQuery", next.clone());
                changes.push(Self::describe_usage_query(&next));
            }
        }

        if let Some(value) = patch.get("lowBalance") {
            // 「 not 」（mode + threshold）： 1 。
            // off ， not 「 old 」 。
            // by provider （Cline not ， its 1，
            // usage_records::default_low_balance_mode）， ——
            // for not will empty 1 、 not will 。
            //
            // limit up after this is ****（ old before / API use in use ）：
            // new `limiters`； 1 request when ， down limiters
            // will by then lowBalance（limit is ， that ）。
            let provider = record.provider();
            let next = Self::normalize_low_balance(value, &provider)?;
            let current = Self::normalize_low_balance_lenient(record.get("lowBalance"), &provider);
            if next != current {
                record.set("lowBalance", next.clone());
                changes.push(Self::describe_low_balance(&next));
            }
        }

        if let Some(value) = patch.get("limiterGroups") {
            let groups = limiter::normalize_groups(value).map_err(AccountStoreError::bad_request)?;
            let next = limiter::types::json_of(&groups);
            let current = limiter::types::json_of(&limiter::effective_groups_in(record.fields()));
            if next != current {
                let legacy = limiter::legacy_rules(&groups);
                record.set("limiterGroups", next);
                record.set("limiters", limiter::rules_to_json(&legacy));
                record.set("lowBalance", Self::legacy_low_balance_of(&legacy));
                changes.push(limiter::describe_groups(&groups));
            }
        }

        if let Some(value) = patch.get("limiters").filter(|_| !patch.contains_key("limiterGroups")) {
            // 「limit」（ / Token then ，）：
            // ， and usageQuery 1 。 1 （），
            // = valid then （ up no has limiters when by old lowBalance /
            // provider ）—— for not will empty 1 。
            //
            // ── lowBalance（）───────────────────
            // then old {mode, threshold}， no has then off：
            // use old when limit not （Token then old not ，ignore can ，
            // then —— old not ）。
            let next_rules = limiter::normalize_rules(value, record.fields())
                .map_err(AccountStoreError::bad_request)?;
            let next = limiter::rules_to_json(&next_rules);
            let current = limiter::effective_rules_json_in(record.fields());
            if next != current || record.fields().contains_key("limiterGroups") {
                record.remove("limiterGroups");
                record.set("limiters", next);
                changes.push(limiter::describe_rules(&next_rules));
                record.set("lowBalance", Self::legacy_low_balance_of(&next_rules));
            }
        }

        Ok(changes)
    }

    /// then → old `lowBalance` （ ）： then 
    /// (action, threshold)， no has then = off（， and old off 1 ）。
    fn legacy_low_balance_of(rules: &[limiter::LimiterRule]) -> Value {
        let Some(rule) = rules
            .iter()
            .find(|rule| rule.kind == limiter::LimiterKind::Balance)
        else {
            return json!({ "mode": "off", "threshold": 0.0 });
        };
        let mode = match rule.action {
            limiter::LimiterAction::Skip => "skip",
            limiter::LimiterAction::Disable => "disable",
        };
        json!({ "mode": mode, "threshold": rule.threshold })
    }

    /// `usageQuery` 1 （：， store_view）。
    ///
    /// for update：`enabled` by ；`interval` must is seconds， when must in 
    /// `MIN~MAX_QUERY_INTERVAL_SECONDS` （ when any / ， 0）——
    /// use ， not will as old 400 。 for （null / ）
    /// 1 as 「」= 、1 minutes（ and `query_interval_of` 
    /// 1 ）；must is `{enabled:false}` for 。
    pub(crate) fn normalize_usage_query(value: &Value) -> Result<Value, AccountStoreError> {
        const MIN: i64 = crate::server::core::usage_records::MIN_QUERY_INTERVAL_SECONDS;
        const MAX: i64 = crate::server::core::usage_records::MAX_QUERY_INTERVAL_SECONDS;
        const DEFAULT: i64 = crate::server::core::usage_records::DEFAULT_QUERY_INTERVAL_SECONDS;
        let Some(fields) = value.as_object() else {
            return Ok(json!({ "enabled": true, "interval": DEFAULT }));
        };
        let enabled = fields.get("enabled").map(js_truthy).unwrap_or(false);
        let interval = match fields.get("interval") {
            None | Some(Value::Null) => 0,
            Some(raw) => raw.as_i64().ok_or_else(|| {
                AccountStoreError::bad_request("查询间隔必须是整数（单位：秒）")
            })?,
        };
        if interval != 0 && !(MIN..=MAX).contains(&interval) {
            return Err(AccountStoreError::bad_request(format!(
                "查询间隔必须是 {MIN}~{MAX} 秒"
            )));
        }
        if enabled && interval == 0 {
            return Err(AccountStoreError::bad_request(format!(
                "开启自动查询需要填写 {MIN}~{MAX} 秒的间隔"
            )));
        }
        Ok(json!({ "enabled": enabled, "interval": interval }))
    }

    /// `usageQuery` ： already 1 1 1 （ —— 
    /// by ， this not ）。** up no has this = 、1 minutes**：
    /// and 1 ， use for 1 set not will empty 1 
    /// 「 already 」 ， not will config。
    fn normalize_usage_query_lenient(value: Option<&Value>) -> Value {
        const DEFAULT: i64 = crate::server::core::usage_records::DEFAULT_QUERY_INTERVAL_SECONDS;
        Self::normalize_usage_query(value.unwrap_or(&Value::Null)).unwrap_or_else(|_| {
            json!({ "enabled": true, "interval": DEFAULT })
        })
    }

    /// `lowBalance` 1 ：mode 1 ； off need has and > 0；
    /// off 。 for （null / ） 1 as 「」= by provider
    /// （ and `balance_blocked` 1 ，
    /// `usage_records::default_low_balance_mode`）； for mode is not 
    /// ， by failure not is 。
    fn normalize_low_balance(value: &Value, provider: &str) -> Result<Value, AccountStoreError> {
        let Some(fields) = value.as_object() else {
            return Ok(Self::low_balance_default(provider));
        };
        let mode = fields.get("mode").and_then(Value::as_str).unwrap_or("");
        if !matches!(mode, "off" | "skip" | "disable") {
            return Err(AccountStoreError::bad_request(
                "余额不足的处理方式必须是「不处理 / 跳过 / 禁用」之一",
            ));
        }
        if mode == "off" {
            return Ok(json!({ "mode": "off", "threshold": 0.0 }));
        }
        let threshold = fields
            .get("threshold")
            .and_then(Value::as_f64)
            .ok_or_else(|| AccountStoreError::bad_request("余额阈值必须是大于 0 的数字"))?;
        if !threshold.is_finite() || threshold <= 0.0 {
            return Err(AccountStoreError::bad_request("余额阈值必须是大于 0 的数字"));
        }
        Ok(json!({ "mode": mode, "threshold": threshold }))
    }

    /// `lowBalance` （noneconfig / none when ）：
    /// `usage_records::default_low_balance`（ by provider ， and 
    /// `balance_blocked` ）。
    fn low_balance_default(provider: &str) -> Value {
        crate::server::core::usage_records::default_low_balance(provider)
    }

    /// `lowBalance` （， up ）。
    fn normalize_low_balance_lenient(value: Option<&Value>, provider: &str) -> Value {
        Self::normalize_low_balance(value.unwrap_or(&Value::Null), provider)
            .unwrap_or_else(|_| Self::low_balance_default(provider))
    }

    /// ：` 90 minutes` this can 。
    pub(crate) fn describe_usage_query(next: &Value) -> String {
        let enabled = next.get("enabled").and_then(Value::as_bool).unwrap_or(false);
        if !enabled {
            return "自动查询余额 → 关闭".to_string();
        }
        let seconds = next.get("interval").and_then(Value::as_i64).unwrap_or(0);
        let span = if seconds > 0 && seconds % 3600 == 0 {
            format!("{} 小时", seconds / 3600)
        } else if seconds > 0 && seconds % 60 == 0 {
            format!("{} 分钟", seconds / 60)
        } else {
            format!("{seconds} 秒")
        };
        format!("自动查询余额 → 开启（每 {span}）")
    }

    /// ： / use 。f64 Display not 
    /// （`100.0` → `100`）， and 1 。
    fn describe_low_balance(next: &Value) -> String {
        let mode = next.get("mode").and_then(Value::as_str).unwrap_or("off");
        let threshold = next.get("threshold").and_then(Value::as_f64).unwrap_or(0.0);
        match mode {
            "skip" => format!("余额不足处理 → 低于 {threshold} 时跳过"),
            "disable" => format!("余额不足处理 → 低于 {threshold} 时禁用"),
            _ => "余额不足处理 → 不处理".to_string(),
        }
    }

    /// up / down 1 （ and ）。
    ///
    /// 1 before down ， can use down ， not use 1 
    /// empty —— this is 1 down need 。
    /// already in / when return `{moved:false}`， not error。
    ///
    /// is **** ： in 1 ， up 1 / down 1 
    /// can can 1 ， after for —— this is 。
    pub fn move_account(&self, id: &str, direction: &str) -> Result<Value, AccountStoreError> {
        let _guard = self.guard();
        // need ****： for can can is 1 
        // （ 1 ）， this must 1 —— need ，
        // and `promote_to_front` 。**** that 。
        let mut state = self.load(&_guard);
        if !state.accounts.iter().any(|item| item.id() == id) {
            return Err(AccountStoreError::not_found("账号不存在"));
        }
        let mut ordered = state.accounts.clone();
        ordered.sort_by_key(StoredAccount::order_key);
        let index = ordered.iter().position(|item| item.id() == id).unwrap_or(0);
        let down = direction == "down";
        let target = if down {
            index as i64 + 1
        } else {
            index as i64 - 1
        };
        if target < 0 || target >= ordered.len() as i64 {
            // Node by will to this up list（`{ ...result, list }`），
            // this up —— before to response and success 1 
            return Ok(json!({
                "moved": false,
                "reason": if down { "已是最后一位" } else { "已是第一位" },
                "list": self.snapshot(&state, Visibility::UserFacing),
            }));
        }

        let target = target as usize;
        let mine = ordered[index].priority();
        let theirs = ordered[target].priority();
        let mine_id = ordered[index].id().to_string();
        let other_id = ordered[target].id().to_string();
        let mine_name = ordered[index].name();
        let theirs_name = ordered[target].name();
        // this ； —— Node this 
        // for （state.accounts not ）， not will as 1 
        // 「 up / down 」 。
        //
        // UPDATE in ** 1 **： is 「 when 」 ，
        // in failure will down in （ 1 not ）。
        let now = logging::now_ms();
        let mut mine_record = ordered[index].clone();
        let mut other_record = ordered[target].clone();
        mine_record.set_priority(theirs);
        mine_record.set_updated_at(now);
        other_record.set_priority(mine);
        other_record.set_updated_at(now);
        self.with_conn_mut(&_guard, |conn| {
            let tx = conn.transaction()?;
            sql::update_in_place(&tx, &mine_record)?;
            sql::update_in_place(&tx, &other_record)?;
            tx.commit()
        })?;
        // use 1 （ that new ）， not 1 ：
        // its this 1 no has ， and this 1 。
        for record in state.accounts.iter_mut() {
            if record.id() == mine_id {
                record.set_priority(theirs);
                record.set_updated_at(now);
            } else if record.id() == other_id {
                record.set_priority(mine);
                record.set_updated_at(now);
            }
        }
        logging::log(
            "[Accounts]",
            &format!("↕️  优先级交换: {mine_name}(P{mine}) ⇄ {theirs_name}(P{theirs})"),
        );
        Ok(json!({
            "moved": true,
            "id": id,
            "newPriority": theirs,
            "swappedWith": {
                "id": other_id,
                "name": theirs_name,
                "newPriority": mine,
            },
            "list": self.snapshot(&state, Visibility::UserFacing),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 「 to name」：set name（ from 
    /// ）， by 「」 will （ mark_name_custom）。
    #[test]
    fn renaming_marks_name_custom_but_same_value_or_other_fields_do_not() {
        let base = || {
            StoredAccount::from_map(
                json!({ "id": "a1", "provider": "workbuddy", "name": "种子名" })
                    .as_object()
                    .unwrap()
                    .clone(),
            )
        };
        let no_holder = |_| Ok(None);

        // ：
        let mut record = base();
        let patch = json!({ "name": "新备注" }).as_object().unwrap().clone();
        let changes = AccountStore::apply_patch(&mut record, &patch, no_holder).unwrap();
        assert_eq!(changes, vec!["备注名 → 新备注"]);
        assert_eq!(record.get("nameCustom"), Some(&Value::Bool(true)));

        // （）： not 
        let mut record = base();
        let patch = json!({ "name": "种子名" }).as_object().unwrap().clone();
        let changes = AccountStore::apply_patch(&mut record, &patch, no_holder).unwrap();
        assert!(changes.is_empty());
        assert_eq!(record.get("nameCustom"), None);

        // （）： not 
        let mut record = base();
        let patch = json!({ "priority": 5 }).as_object().unwrap().clone();
        AccountStore::apply_patch(&mut record, &patch, no_holder).unwrap();
        assert_eq!(record.get("nameCustom"), None);
    }
}
