//! CatPaw **** and （ ）。
//!
//! ── this fileWhy this ────────────────────────────────────
//! will 、 will 、directory、 to **
//! **（`agent2api-providers` `catpaw/`）， not —— that is 
//! 1 。add by `account.normalize` 1 after 
//! `pluginSecret`， `plugin_accounts::to_plugin_public_account`。
//!
//! in this has **** and ： no has 
//! `pluginSecret`， is `accessToken` / `uid` / `loginName`，
//! need need by 。 this not is up 。
//!
//! ── Why ──────────────────────────
//! in this when `~/.meituan-catpaw/auth.json`（ that in ）。
//! ，file not this in this ： not to when 
//! `available` false， in 1 path。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};
use crate::server::core::providers::{kind_id, ProviderKind};

/// （ and `balance::BALANCE_TOKEN_FIELD` ）。
///
/// 1 not is import： and no has ，
/// this is **** 1 。
const BALANCE_TOKEN_FIELD: &str = "balanceToken";

/// limit（ and `MAX_TOKEN_LENGTH` ）
const MAX_TOKEN_LENGTH: usize = 8192;

fn catpaw_id() -> &'static str {
    kind_id(ProviderKind::CatPaw)
}

impl AccountStore {
    /// 1 CatPaw （ JSON）。
    ///
    /// ****（`refresh_account` / `patch_account` use 
    /// 「 this id 」）。
    pub fn catpaw_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        let catpaw = catpaw_id();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == catpaw).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, catpaw)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// CatPaw ****（ HTTP response）。
    ///
    /// for ** no has ** `pluginSecret` （ in 
    /// `store_view::public_account`， that ）。
    ///
    /// ：`id` / `provider` / `name` / `uid` / `loginName` / `tokenTail` /
    /// `desktop` / `source` / `priority` / `enabled` / `addedAt` / `updatedAt` /
    /// `proxy` / `rateLimits` / `available` / `hasBalanceToken`。
    pub fn to_catpaw_public_account(&self, record: &StoredAccount) -> Value {
        let proxy = crate::server::core::proxies::describe_account_proxy(Some(&record.proxy()));
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        public.insert(
            "uid".to_string(),
            Value::String(record.get("uid").and_then(Value::as_str).unwrap_or("").to_string()),
        );
        public.insert(
            "loginName".to_string(),
            Value::String(
                record
                    .get("loginName")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
        );
        public.insert(
            "tokenTail".to_string(),
            Value::String(
                record
                    .get("tokenTail")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            ),
        );
        public.insert("desktop".to_string(), Value::Bool(record.is_desktop()));
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), proxy);
        // ： from （ and ）。
        // —— 1 will use in this up 1 。
        public.insert(
            "rateLimits".to_string(),
            record.get("rateLimits").cloned().unwrap_or_else(|| Value::Object(Map::new())),
        );
        // available： not token， no has when file path，
        // by 「 has no has token」（ false， etc ）。
        public.insert(
            "available".to_string(),
            Value::Bool(record.has_token() && !record.is_desktop()),
        );
        // **「 no 」**， not （ not before ：
        // is will / ， this 1 is will ）。
        public.insert(
            "hasBalanceToken".to_string(),
            Value::Bool(
                record
                    .get(BALANCE_TOKEN_FIELD)
                    .and_then(Value::as_str)
                    .map(|value| !value.trim().is_empty())
                    .unwrap_or(false),
            ),
        );
        Value::Object(public)
    }

    /// set / CatPaw ****（`balanceToken`）。
    ///
    /// ── Why 1 ， not is use `apply_patch` ─────────
    /// use patch is ** has ** （ / / enabled / ），
    /// `balanceToken` is CatPaw has 。 `apply_patch`
    /// will that use 1 provider ， 1 use is 1 。
    ///
    /// ── ────────────────────────────────────────────
    /// - ****（ has `pluginSecret`）： secret —— 
    /// when to is ，`balance.query` use ；
    /// - ****：（ and ）。
    ///
    /// `value` ：（empty after empty）= set；`null` / empty = ****。
    /// return（ and `update_account` changes ）。
    pub fn update_catpaw_balance_token(
        &self,
        id: &str,
        value: &Value,
    ) -> Result<Vec<String>, AccountStoreError> {
        let next: Option<String> = match value {
            Value::Null => None,
            other => {
                let text = match other {
                    Value::String(text) => text.clone(),
                    Value::Number(number) => number.to_string(),
                    Value::Bool(flag) => flag.to_string(),
                    _ => other.to_string(),
                };
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    // this will `Cookie` ： and CR/LF/ need 
                    let stripped = trimmed
                        .strip_prefix("Bearer ")
                        .or_else(|| trimmed.strip_prefix("bearer "))
                        .unwrap_or(trimmed)
                        .trim();
                    if stripped.chars().count() > MAX_TOKEN_LENGTH
                        || stripped.contains(['\r', '\n', ';'])
                    {
                        return Err(AccountStoreError::new(
                            "余额查询凭证格式无效（超长或含换行 / 分号）",
                            400,
                        ));
                    }
                    Some(stripped.to_string())
                }
            }
        };
        // ── ： `pluginSecret` ────────────────────────
        // is up ** has no has ** `pluginSecret`（ not is 「plugin_account can not can to 
        // secret」—— that for will 1 ）。 in this ：
        // `plugin_update_secret` need 1 。
        let plugin_secret = {
            let guard = self.guard();
            self.record_by_id(&guard, id)
                .filter(|record| record.has_plugin_secret())
                .map(|record| record.get("pluginSecret").cloned().unwrap_or(Value::Null))
        };
        if let Some(secret) = plugin_secret {
            let mut next_secret = secret.clone();
            if let Some(object) = next_secret.as_object_mut() {
                match &next {
                    Some(token) => {
                        object.insert(
                            BALANCE_TOKEN_FIELD.to_string(),
                            Value::String(token.clone()),
                        );
                    }
                    None => {
                        object.remove(BALANCE_TOKEN_FIELD);
                    }
                }
            }
            match self.plugin_update_secret(id, &secret, &next_secret) {
                Ok(true) => {
                    return Ok(vec![match next {
                        Some(_) => "余额查询凭证已更新".to_string(),
                        None => "余额查询凭证已清除".to_string(),
                    }])
                }
                Ok(false) => return Err(AccountStoreError::new("账号凭证已变化，请刷新后重试", 409)),
                Err(message) => return Err(AccountStoreError::new(message, 500)),
            }
        }
        // ── ： ────────────────────────────
        let _guard = self.guard();
        let mut record = self
            .record_by_id(&_guard, id)
            .filter(|record| record.provider() == catpaw_id())
            .ok_or_else(|| AccountStoreError::not_found("账号不存在或不属于 CatPaw"))?;
        let current = record
            .get(BALANCE_TOKEN_FIELD)
            .and_then(Value::as_str)
            .map(str::to_string);
        if current == next {
            return Ok(Vec::new());
        }
        let mut changes = Vec::new();
        match &next {
            Some(token) => {
                record.set(BALANCE_TOKEN_FIELD, Value::String(token.clone()));
                // 「 already update」，** not **（ and token 1 ）
                changes.push("余额查询凭证已更新".to_string());
            }
            None => {
                record.remove(BALANCE_TOKEN_FIELD);
                changes.push("余额查询凭证已清除".to_string());
            }
        }
        record.set_updated_at(crate::server::logging::now_ms());
        let name = record.name();
        self.with_conn(&_guard, |conn| {
            crate::server::core::account_store::sql::update_in_place(conn, &record)
        })?;
        crate::server::logging::log(
            "[Accounts]",
            &format!("✏️  CatPaw 账号已更新: {name}（{}）", changes.join("，")),
        );
        Ok(changes)
    }

    /// in CatPaw will —— ** after is empty**。
    ///
    /// ── Why this ──────────────────────────────────────
    /// will （`x-session-id → conversationId`） already 
    /// ， no has that ，none from 。 path will 
    /// `reconcile_identity`： 1 use when ， old will in ** down 1 request**
    /// and —— 「delete / use / new 」 need 
    /// 1 。
    ///
    /// not is use ： that use （ / / / /
    /// ） is 「， down need 」， this ——
    /// is down down in 、 and can 。 will if up 「 by will 」
    /// RPC， is this 。
    pub(crate) fn invalidate_catpaw_sessions(&self, _account_id: &str, _provider: &str) -> usize {
        0
    }
}
