//! ：、delete、 
//! （ `core::custom_providers` module header）。
//!
//! ── and path ────────────────────────────────────
//! not （JWT / token / apiKey / ），
//! has `add_*_account`。：** use 1 up 
//! API Key**，or「 this up not need need 」（`noAuth`， down ）—— 
//! not ， this no has new 、 no has 、 no has when ，
//! has 「 1 」。
//!
//! ── （ and for + has ）─────────────
//! ```json
//! {
//! "id": "custom-acct-8f2c1d4a6b90", // custom-acct- + 12 hex
//! "provider": "custom-3f2a91b04c7e", // to （）
//! "name": " A ",
//! "apiKey": "sk-…", // （ can empty：none need up ）
//! "noAuth": true, // optional：「 this up none need 」。
//! // and apiKey （ 1 ，
//! // `state::no_auth`）； not 
//! "baseUrl": "https://…", // optional； not in then use baseUrl
//! "tokenTail": "…abcd", // up （apiKey ）
//!   "source": "manual",
//! "priority": 5, // 1 （ and use ）
//!   "enabled": true,
//!   "addedAt": 1730000000000,
//!   "updatedAt": 1730000000000
//! }
//! ```
//!
//! ── Why id is + ─────────────────────────
//! `apiKey` empty when use 「 ID + Key」 SHA-256 before 12 ： 1 
//! and update， not can use Key。 has by ，
//! update when ID， or after use 。
//! no has key when （none need ， or no ） as 12 
//! （`getrandom`）： no has can ， and ** 1 and none need is 
//! **（ can not ， by ）， this is 
//! new 1 —— id 1 （ and / need ， by 
//! `custom_providers` id）。 then `identity`。
//! before `custom-acct-` and provider id `custom-` ** not **：
//! `is_custom_provider_id` by provider before ， id if will 
//! id（ id empty is ）。
//!
//! ── ────────────────────────────────────────────────
//! file is 「--」file，** no has request**（ not ）。
//! not unwrap/expect（release is panic=abort）。

mod identity;

use serde_json::{Map, Value};

use crate::server::core::account_store::priority::next_free_priority;
use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::{mark_name_custom, StoredAccount};
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};
use crate::server::core::account_store::store_util::{token_tail_of, truncate_chars};
use crate::server::core::custom_providers;
use crate::server::core::proxies::{resolve_account_proxy_rotating, ResolvedProxy};
use crate::server::logging;

/// id before 。** and provider id `custom-` not **（ by module header）。
pub const CUSTOM_ACCOUNT_ID_PREFIX: &str = "custom-acct-";

/// limit（ and 1 ）
const MAX_NAME_CHARS: usize = 100;

/// apiKey limit（： 1 when not ）。
/// up key 1 32~256 ，8192 and `MAX_TOKEN_LENGTH` 。
const MAX_API_KEY_LENGTH: usize = 8192;

/// `add_custom_account` return： + this is new is update。
///
/// etc （ `identity`） will 「 1 、 1 Key 」 and has 
/// 。 this to in 1 has 
/// （ has 🔄 / ✅ ）， as return to API ， before can 
/// 「 already update」 not is 「 already 」（「 already 」 will as will 
/// 1 ， ）。
pub struct CustomAccountUpsert {
    /// （ and 1 ， `public_account`）
    pub account: Value,
    /// `true` = new 1 ；`false` = and update has （id in key
    /// ）
    pub created: bool,
}

impl AccountStore {
    // ─── ： ────────────────────────────────────────

    /// /update 1 （`POST /api/accounts` custom and 
    /// `POST /api/custom-providers` 「 」 use ）。
    ///
    /// `provider_id` must** `custom-` **（ use already use 
    /// `custom_providers::is_custom_provider_id` in ； this 
    /// before —— not config， and config）。
    ///
    /// `payload` （ its ignore， and 「unknown not 」 1 ）：
    /// - `apiKey`：（ can empty —— up not need need key）；
    /// - `noAuth`：**「 this up none need 」**（， false）。 and 
    /// `apiKey` ****： as when apiKey empty，empty apiKey when this 
    /// not 。 this ， is as `has_credentials` and 「 not 
    /// 」 has 1 （ `state::no_auth`）。** is 「none」
    /// and 「 key」 1 **： after not this ， in up will 
    /// not configured，；
    /// - `baseUrl`：**optional**（ not this ， to 
    /// baseUrl）。 and 1 （`normalize_base_url`）；
    /// - `name` by `name` parameter（ use already from request， and 1 ）。
    ///
    /// etc ：`apiKey` empty when 1 、 Key = update has （ /
    /// baseUrl update，ID、 and enabled use ）。 no has key（none need /
    /// ） then is new —— 1 and none need is （
    /// can not ）。 to ** its provider** id（hash empty）
    /// when 409 not is 。 in 1 by return 
    /// [`CustomAccountUpsert::created`] ，API to before 。
    pub fn add_custom_account(
        &self,
        provider_id: &str,
        payload: &Value,
        name: Option<&str>,
    ) -> Result<CustomAccountUpsert, AccountStoreError> {
        let provider_id = provider_id.trim();
        if !provider_id.starts_with(custom_providers::ID_PREFIX) {
            return Err(AccountStoreError::bad_request(format!(
                "不是自定义提供商 id: {provider_id}"
            )));
        }
        let Some(object) = payload.as_object() else {
            return Err(AccountStoreError::bad_request("账号内容必须是 JSON 对象"));
        };
        // noAuth： not for （ and / etc 1 to ：「success
        // no 」 1 400 ）。 is 「 this not need need 」 ，
        // and apiKey 1 in down 。
        let no_auth = match object.get("noAuth") {
            None | Some(Value::Null) => false,
            Some(Value::Bool(flag)) => *flag,
            Some(_) => {
                return Err(AccountStoreError::bad_request("noAuth 必须是布尔值"));
            }
        };
        // apiKey：trim after （ when empty/ —— will 
        // Authorization up ，request in failure， use not Why）。
        // 「none need 」 no has key can ： not in this 。
        let api_key = if no_auth {
            String::new()
        } else {
            object
                .get("apiKey")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or("")
                .to_string()
        };
        if api_key.chars().count() > MAX_API_KEY_LENGTH {
            return Err(AccountStoreError::bad_request("apiKey 过长"));
        }
        // baseUrl ： to must（ and ）；** no to not ** ——
        // 「 no has 」 and 「 to empty」 in is ， up need can 
        let base_url_override = match object
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            Some(raw) => Some(
                custom_providers::normalize_base_url(raw)
                    .map_err(AccountStoreError::bad_request)?,
            ),
            None => None,
        };

        let _guard = self.guard();
        let existing = self.with_conn(&_guard, |conn| identity::existing(conn, provider_id, &api_key))?;
        let id = match existing.as_ref() {
            Some(record) => record.id().to_string(),
            None => {
                let id = identity::new_id(provider_id, &api_key)
                    .map_err(|error| AccountStoreError::new(error, 500))?;
                if self.with_conn(&_guard, |conn| sql::load_by_id(conn, &id))?.is_some() {
                    return Err(AccountStoreError::new(
                        format!("账号 id「{id}」已被占用，请重试或检查已有记录"),
                        409,
                    ));
                }
                id
            }
        };

        // ： → has → 「 + apiKey 」。
        // for has （ key）；none no has can ，
        // 「<>（none need ）」—— 1 「」 can 
        // 「 this is 」； no has when is that use 。 not ：
        // and none need default（ use ）。
        let explicit_name = name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| truncate_chars(value, MAX_NAME_CHARS));
        let explicit = explicit_name.is_some();
        let record_name = explicit_name
            .or_else(|| {
                existing
                    .as_ref()
                    .map(StoredAccount::name)
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| {
                let tail = token_tail_of(&api_key);
                if !tail.is_empty() {
                    format!("账号 {tail}")
                } else if no_auth {
                    let label = custom_providers::label_of(provider_id)
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "自定义账号".to_string());
                    format!("{label}（无需鉴权）")
                } else {
                    "自定义账号".to_string()
                }
            });

        let priority = existing
            .as_ref()
            .map(StoredAccount::priority)
            .unwrap_or_else(|| {
                // ： 1 （all use 1 ，
                // `account_store` module header）。failure（unavailable） to default，
                // and cline 1 。
                let used = self
                    .with_conn(&_guard, |conn| sql::priorities_all(conn))
                    .unwrap_or_default();
                next_free_priority(&used)
            });

        let now = logging::now_ms();
        // from has （ not 「unknown」： use 、
        // will add not can in this 1 ）—— new when is empty
        let mut fields: Map<String, Value> = existing
            .as_ref()
            .map(|item| item.fields().clone())
            .unwrap_or_default();
        fields.insert("id".to_string(), Value::String(id));
        fields.insert(
            "provider".to_string(),
            Value::String(provider_id.to_string()),
        );
        fields.insert("name".to_string(), Value::String(record_name.clone()));
        mark_name_custom(&mut fields, explicit, existing.as_ref());
        fields.insert("apiKey".to_string(), Value::String(api_key.clone()));
        // 「none need 」： when ， when **** —— and 
        // capabilities 1 to （「empty not 」）， not will empty
        // 1 `"noAuth": false`。
        if no_auth {
            fields.insert("noAuth".to_string(), Value::Bool(true));
        } else {
            fields.remove("noAuth");
        }
        match base_url_override {
            Some(base_url) => {
                fields.insert("baseUrl".to_string(), Value::String(base_url));
            }
            // empty baseUrl（ to default） need supports：
            // request** this ** as empty → ； no → 。
            // this is 「empty」 1 can （empty not is URL， not can ）。
            None => {
                if object.contains_key("baseUrl") {
                    fields.remove("baseUrl");
                }
            }
        }
        // config（`balanceQuery`，optional）： need that 
        // （NewAPI / / ）。
        // and apiKey 1 is ： is 「 this 」， is used for
        // not is （ `custom_balance_query_by_id` ）。
        if let Some(value) = object.get("balanceQuery") {
            if !value.is_null() {
                let Some(patch) = value.as_object() else {
                    return Err(AccountStoreError::bad_request("balanceQuery 必须是 JSON 对象"));
                };
                let mut query = fields
                    .get("balanceQuery")
                    .and_then(Value::as_object)
                    .cloned()
                    .unwrap_or_default();
                normalize_balance_query_patch(&mut query, patch)
                    .map_err(AccountStoreError::bad_request)?;
                if query.is_empty() {
                    fields.remove("balanceQuery");
                } else {
                    fields.insert("balanceQuery".to_string(), Value::Object(query));
                }
            }
        }
        fields.insert(
            "tokenTail".to_string(),
            Value::String(token_tail_of(&api_key)),
        );
        fields.insert(
            "source".to_string(),
            Value::String(
                existing
                    .as_ref()
                    .map(StoredAccount::source)
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "manual".to_string()),
            ),
        );
        fields.insert("priority".to_string(), Value::from(priority));
        fields.insert(
            "enabled".to_string(),
            Value::Bool(existing.as_ref().map(StoredAccount::enabled).unwrap_or(true)),
        );
        fields.insert(
            "addedAt".to_string(),
            Value::from(
                existing
                    .as_ref()
                    .map(StoredAccount::added_at)
                    .filter(|value| *value != 0)
                    .unwrap_or(now),
            ),
        );
        fields.insert("updatedAt".to_string(), Value::from(now));

        let saved = StoredAccount::from_map(fields);
        // ：`put` = DELETE + INSERT（update has when to ，
        // and old retain + push 1 ， `sql::put`）
        self.with_conn(&_guard, |conn| sql::put(conn, &saved))?;
        logging::log(
            "[Accounts]",
            &format!(
                "{} 自定义账号{}: {}（{}）",
                if existing.is_some() { "🔄" } else { "✅" },
                if existing.is_some() { "已更新" } else { "已添加" },
                record_name,
                custom_providers::label_of(provider_id)
                    .unwrap_or_else(|| provider_id.to_string()),
            ),
        );
        Ok(CustomAccountUpsert {
            account: self.public_account(&saved),
            created: existing.is_none(),
        })
    }

    /// delete down ****，returndelete。
    ///
    /// delete（`custom_providers::remove` in before use ）：
    /// not is 1 SQL —— in ， `sql::delete` is 
    /// has deletepath（「 not in return false」 ）， use can not add
    /// SQL、 not new （ not need need ： to 1 failure when 
    /// down in ， 1 can ）。
    pub fn remove_custom_accounts(&self, provider_id: &str) -> Result<usize, AccountStoreError> {
        let provider_id = provider_id.trim();
        let _guard = self.guard();
        let records = self.with_conn(&_guard, |conn| sql::load_by_provider(conn, provider_id))?;
        let mut removed = 0usize;
        for record in &records {
            let deleted = self.with_conn(&_guard, |conn| sql::delete(conn, record.id()))?;
            if deleted {
                removed += 1;
            }
        }
        Ok(removed)
    }

    /// this down ** etc to ** ，return 。
    ///
    /// is 「 Base URL 」 that （`api::custom_providers::
    /// update_custom_provider`）。 new when will when 
    /// （ as ）， up `baseUrl` is ****、 when ——
    /// is after request old ， up not to that not 
    /// （issue #455 ）。
    ///
    /// ** etc **（empty、 after ）： can can 
    /// to 1 ， that is has ， not can 1 and 。 is 「 to 
    /// 」， not is delete use config —— by that 。
    pub fn clear_custom_base_url_overrides(
        &self,
        provider_id: &str,
        matched: &str,
    ) -> Result<Vec<String>, AccountStoreError> {
        let provider_id = provider_id.trim();
        let target = matched.trim().trim_end_matches('/');
        if provider_id.is_empty() || target.is_empty() {
            return Ok(Vec::new());
        }
        let _guard = self.guard();
        let records = self.records_for_provider(&_guard, provider_id);
        let mut cleared: Vec<String> = Vec::new();
        for mut record in records {
            let Some(current) = record
                .fields()
                .get("baseUrl")
                .and_then(Value::as_str)
                .map(str::trim)
                .map(|text| text.trim_end_matches('/'))
            else {
                continue;
            };
            if current != target {
                continue;
            }
            let name = record.name();
            record.remove("baseUrl");
            record.set_updated_at(logging::now_ms());
            self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
            cleared.push(name);
        }
        Ok(cleared)
    }

    /// 1 **baseUrl **（`PATCH /api/accounts` 
    /// `baseUrl` ）。
    ///
    /// （ and up 「」 1 1 for ）：
    /// - `null` or empty = ****， use baseUrl；
    /// - empty = after （ and 1 `normalize_base_url`）。
    ///
    /// Why this ： in （`baseUrl`），
    /// before that 1 ， old not not 。 new 
    /// when that old is this （issue #455）。
    pub fn update_custom_base_url(
        &self,
        id: &str,
        value: &Value,
    ) -> Result<Vec<String>, AccountStoreError> {
        let id = id.trim();
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return Err(AccountStoreError::not_found("账号不存在"));
        };
        if !record.provider().starts_with(custom_providers::ID_PREFIX) {
            return Err(AccountStoreError::bad_request(
                "只有自定义提供商的账号有上游地址覆盖项",
            ));
        }
        let next = match value {
            Value::Null => None,
            Value::String(text) => {
                let text = text.trim();
                if text.is_empty() {
                    None
                } else {
                    Some(
                        custom_providers::normalize_base_url(text)
                            .map_err(AccountStoreError::bad_request)?,
                    )
                }
            }
            _ => {
                return Err(AccountStoreError::bad_request(
                    "baseUrl 必须是字符串或 null（null = 跟随提供商）",
                ))
            }
        };
        let before = record
            .fields()
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string);
        if before == next {
            return Ok(Vec::new());
        }
        let change = match &next {
            Some(url) => {
                record.set("baseUrl", Value::String(url.clone()));
                format!("上游地址覆盖 → 已设为 {url}")
            }
            None => {
                record.remove("baseUrl");
                "上游地址覆盖 → 已清除（跟随提供商）".to_string()
            }
        };
        record.set_updated_at(logging::now_ms());
        let name = record.name();
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
        logging::log(
            "[Accounts]",
            &format!("🌐 自定义账号上游地址已更新: {name}（{change}）"),
        );
        Ok(vec![change])
    }

    /// by id provider —— **only when **return Some。
    ///
    /// is `api::accounts::refresh_account` ： no has can new 
    /// （key is use ）， if not will workbuddy new and to 
    /// 1 「 not in 」。 use before not is `is_custom_provider_id`：
    /// this is 「 this is not is 」， and is in none
    /// （ after not this workbuddy new ）。
    pub fn custom_account_provider(&self, id: &str) -> Option<String> {
        let _guard = self.guard();
        let record = self.record_by_id(&_guard, id)?;
        let provider = record.provider();
        provider
            .starts_with(custom_providers::ID_PREFIX)
            .then_some(provider)
    }

    // ─── ：（ / use ）────────────────

    /// 1 ****（`custom_credential_by_id`）。
    ///
    /// ：`providers::custom::forward`（ for ， need apiKey and 
    /// baseUrl ）。**return apiKey ， not HTTP response** ——
    /// （`to_custom_public_account`） has tokenTail， that is to ；
    /// this is ， not can use 1 。
    ///
    /// in ****（ Clash ，file IO； and 
    /// `get_session_by_id` 1 ）， in 。
    pub fn custom_credential_by_id(&self, account_id: &str) -> Option<CustomCredential> {
        let record = self.read_record(account_id)?;
        Some(credential_of_record(&record))
    }

    /// this down 1 **available** （`custom_providers::
    /// fetch_upstream_models` ）。
    ///
    /// 「available」= enabled ** and **（apiKey empty or none need ）—— and 
    /// `has_credentials` 1 ： is 1 ， can not can success
    /// 「 this when has no has available」， not is 「 has no has key」
    /// （none up not key to ， `no_auth`）。
    ///
    /// 「 1 」= （`order_key` and 1 ）—— is 
    /// ， no has ， use can ； has 
    /// can in not in （ 1 up for key 1 ）。
    /// 1 no has when by use to （「」）， this None 。
    pub fn first_custom_credential(&self, provider_id: &str) -> Option<CustomCredential> {
        let _guard = self.guard();
        self.records_for_provider(&_guard, provider_id)
            .into_iter()
            .filter(|record| record.enabled() && (record.has_api_key() || record.no_auth()))
            .min_by_key(StoredAccount::order_key)
            .map(|record| credential_of_record(&record))
    }

    /// 1 ****：`apiKey`（ can ） and `noAuth`（none need 
    /// ）。return（ and ）， no has when returnempty。
    ///
    /// ── Why 1 （ not is and `update_account` apply_patch）──
    /// and CatPaw `balanceToken`、ZCode `zcodePlan` 1 ： this is **
    /// has ** 。 and use patch ，
    /// 「 1 apiKey」 will empty 1 none。
    /// in `api::accounts::patch_account` by id （ that already has 
    /// ）。
    ///
    /// ── not （ 1 ）────────────────────────
    /// 1. `noAuth` and `apiKey` ：none need → key empty；empty key
    /// → 。 is not ；
    /// 2. ** id not **：id is （、request、file by 
    /// ）， 1 key is 「 1 」 not is 「 1 」。
    /// 「 key and 」 in ****path up （ that id no ）；
    /// 3. `apiKey` as null / empty = ****（， use up 「」
    /// by —— empty is 「 not 」， this 「」）。
    ///
    /// key will new `tokenTail`（ must，otherwise up 
    /// is up 1 key ）。
    pub fn update_custom_credentials(
        &self,
        id: &str,
        patch: &Value,
    ) -> Result<Vec<String>, AccountStoreError> {
        let id = id.trim();
        let Some(object) = patch.as_object() else {
            return Err(AccountStoreError::bad_request("请求内容必须是 JSON 对象"));
        };
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return Err(AccountStoreError::not_found("账号不存在"));
        };
        if !record.provider().starts_with(custom_providers::ID_PREFIX) {
            return Err(AccountStoreError::bad_request(
                "只有自定义提供商的账号有 API Key / 无需鉴权这两项",
            ));
        }
        let mut changes: Vec<String> = Vec::new();
        // ── in this ：「 after is 」，
        // 1 —— in not will down 「、key in 」 in 
        let mut next_no_auth = record.no_auth();
        let mut next_api_key = record
            .fields()
            .get("apiKey")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if let Some(value) = object.get("noAuth") {
            // null = not （ and 「 not 」 1 to ）： need false，
            // 「 null」 1 
            if !value.is_null() {
                let flag = value
                    .as_bool()
                    .ok_or_else(|| AccountStoreError::bad_request("noAuth 必须是布尔值"))?;
                next_no_auth = flag;
                if flag {
                    // not 1：none need no has key can 
                    next_api_key = String::new();
                }
            }
        }
        if let Some(value) = object.get("apiKey") {
            let key = match value {
                Value::Null => String::new(),
                Value::String(text) => text.trim().to_string(),
                _ => {
                    return Err(AccountStoreError::bad_request(
                        "apiKey 必须是字符串（传 null 表示清除）",
                    ))
                }
            };
            if key.chars().count() > MAX_API_KEY_LENGTH {
                return Err(AccountStoreError::bad_request("apiKey 过长"));
            }
            next_api_key = key;
            if !next_api_key.is_empty() {
                // not 1 1 ： key not this 「none need 」
                next_no_auth = false;
            }
        }
        // ── （：、key、）──────────────────────
        let before_no_auth = record.no_auth();
        let before_key = record
            .fields()
            .get("apiKey")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if next_no_auth != before_no_auth {
            if next_no_auth {
                record.set("noAuth", Value::Bool(true));
                changes.push("无需鉴权 → 已开启（转发不发鉴权头）".to_string());
            } else {
                record.remove("noAuth");
                changes.push("无需鉴权 → 已关闭".to_string());
            }
        }
        if next_api_key != before_key {
            let tail = token_tail_of(&next_api_key);
            record.set("apiKey", Value::String(next_api_key.clone()));
            record.set("tokenTail", Value::String(tail.clone()));
            changes.push(if next_api_key.is_empty() {
                "API Key → 已清除".to_string()
            } else {
                format!("API Key → 已更新（尾号 {tail}）")
            });
        }
        if changes.is_empty() {
            return Ok(changes);
        }
        // not 、 not enabled：update（ and token 1 path）
        record.set_updated_at(logging::now_ms());
        let name = record.name();
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
        logging::log(
            "[Accounts]",
            &format!("🔑 自定义账号凭证已更新: {name}（{}）", changes.join("，")),
        );
        Ok(changes)
    }

    // ─── config（`balanceQuery`）──────────────────────────

    /// 1 **config**（ use ）。
    ///
    /// **return can can ， not HTTP response** —— has 
    /// `has*` and （ `store_view::to_custom_public_account`），
    /// and `custom_credential_by_id` is 1 。
    ///
    /// no has this （ this 1 not need need ， or use no ） to `None`。
    pub fn custom_balance_query_by_id(&self, account_id: &str) -> Option<CustomBalanceQuery> {
        let _guard = self.guard();
        let record = self.record_by_id(&_guard, account_id)?;
        Some(balance_query_of_record(&record))
    }

    /// 1 config。return（ and ），
    /// no has when returnempty。
    ///
    /// ── （ and `update_custom_credentials` 1 ）────────────
    /// · = not ；`null` or empty = ****（empty is 「 not 」，
    /// must by ， up 「」 by ）；
    /// · that （[`BALANCE_QUERY_FIELDS`]）， its 
    /// 400 —— ignore will use as ；
    /// · （`userId` / `username`），
    /// —— already is `enc1:` （ `sql.rs` module header）， not 
    /// （ that will 1 、「 1 」 1 ）。
    ///
    /// ── Why need JWT 1 ─────────────────────────
    /// （`sensenovaJwt`） is use ： 1 ， old 
    /// is old —— will 「」 this no （ use old 
    /// success）。 after down 1 。
    pub fn update_custom_balance_query(
        &self,
        id: &str,
        patch: &Value,
    ) -> Result<Vec<String>, AccountStoreError> {
        let id = id.trim();
        let Some(object) = patch.as_object() else {
            return Err(AccountStoreError::bad_request("请求内容必须是 JSON 对象"));
        };
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return Err(AccountStoreError::not_found("账号不存在"));
        };
        let provider = record.provider();
        if !provider.starts_with(custom_providers::ID_PREFIX) {
            return Err(AccountStoreError::bad_request(
                "只有自定义提供商的账号有余额查询配置（balanceQuery）",
            ));
        }
        // is 「 this 1 」 ： no has no has can 
        // （ in create when ， custom_providers）
        if custom_providers::balance_scheme_of(&provider).is_none() {
            return Err(AccountStoreError::bad_request(
                "该提供商没有登记余额查询方案，无法配置查询凭证",
            ));
        }
        let before = record
            .fields()
            .get("balanceQuery")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let mut after = before.clone();
        normalize_balance_query_patch(&mut after, object).map_err(AccountStoreError::bad_request)?;
        if after == before {
            return Ok(Vec::new());
        }
        let changes = describe_balance_query_changes(&before, &after);
        if after.is_empty() {
            record.remove("balanceQuery");
        } else {
            record.set("balanceQuery", Value::Object(after));
        }
        record.set_updated_at(logging::now_ms());
        let name = record.name();
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
        logging::log(
            "[Accounts]",
            &format!("🔎 自定义账号余额查询配置已更新: {name}（{}）", changes.join("，")),
        );
        Ok(changes)
    }

    /// JWT （path use ；failure down 1 ）。
    ///
    /// and update 1 path， not 、 not `updatedAt` 
    /// —— is 1 ****， not is use 。
    pub fn store_custom_balance_jwt(
        &self,
        id: &str,
        jwt: &str,
        expires_at_ms: i64,
    ) -> Result<(), AccountStoreError> {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id.trim()) else {
            return Err(AccountStoreError::not_found("账号不存在"));
        };
        let mut query = record
            .fields()
            .get("balanceQuery")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        query.insert(
            "sensenovaJwt".to_string(),
            Value::String(jwt.trim().to_string()),
        );
        query.insert(
            "sensenovaJwtExpiresAt".to_string(),
            Value::from(expires_at_ms),
        );
        record.set("balanceQuery", Value::Object(query));
        record.set_updated_at(logging::now_ms());
        let _ = self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))?;
        Ok(())
    }
}

/// **config**（`custom_balance_query_by_id` return）。
///
/// and 1 1 for （`userId` → `user_id`…）。`sensenova_jwt` is 
/// **** not is use ： 1 ， before use 、 after 
/// （ `providers::custom::balance` module header）。
pub struct CustomBalanceQuery {
    /// NewAPI：New-Api-User （ new can empty）
    pub user_id: String,
    /// NewAPI：（）
    pub access_token: String,
    /// ：（）
    pub username: String,
    /// ：（）
    pub password: String,
    /// ：bigmodel.cn （）
    pub token: String,
    /// ： （empty = no ）
    pub sensenova_jwt: String,
    /// ： to when （ms，0 = unknown）
    pub sensenova_jwt_expires_at: i64,
}

/// → （， not —— already 1 ）
fn balance_query_of_record(record: &StoredAccount) -> CustomBalanceQuery {
    let fields = record
        .fields()
        .get("balanceQuery")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let text_of = |key: &str| {
        fields
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string()
    };
    CustomBalanceQuery {
        user_id: text_of("userId"),
        access_token: text_of("accessToken"),
        username: text_of("username"),
        password: text_of("password"),
        token: text_of("token"),
        sensenova_jwt: text_of("sensenovaJwt"),
        sensenova_jwt_expires_at: fields
            .get("sensenovaJwtExpiresAt")
            .and_then(Value::as_i64)
            .unwrap_or(0),
    }
}

/// can config （ and ）。
///
/// （NewAPI use before 、 use in 、 use after 1 ），
/// 1 not is by ： not （）， by will 
/// 「 1 」 in 1 。
pub const BALANCE_QUERY_FIELDS: &[&str] = &["userId", "accessToken", "username", "password", "token"];

/// /（ and error use ； for use no has ）
fn balance_query_field_label(key: &str) -> &'static str {
    match key {
        "userId" => "用户 ID",
        "accessToken" => "访问令牌",
        "username" => "账号",
        "password" => "密码",
        "token" => "控制台令牌",
        _ => "查询凭证",
    }
}

/// when need 1 （ `update_custom_balance_query` ）
const BALANCE_QUERY_IDENTITY_FIELDS: &[&str] = &["accessToken", "username", "password", "token"];
/// limit（： 1 when not ； and apiKey ）
const MAX_BALANCE_QUERY_FIELD_CHARS: usize = 8192;

/// 1 to config up （`add_custom_account` and 
/// `update_custom_balance_query` ** use 1 **： not 、null / empty）。
fn normalize_balance_query_patch(
    query: &mut Map<String, Value>,
    patch: &Map<String, Value>,
) -> Result<(), String> {
    let mut identity_changed = false;
    for key in BALANCE_QUERY_FIELDS {
        let Some(value) = patch.get(*key) else { continue };
        let next = match value {
            Value::Null => String::new(),
            Value::String(text) => text.trim().to_string(),
            _ => {
                return Err(format!(
                    "balanceQuery.{} 必须是字符串（传 null 表示清除）",
                    balance_query_field_label(key)
                ))
            }
        };
        if next.chars().count() > MAX_BALANCE_QUERY_FIELD_CHARS {
            return Err(format!(
                "balanceQuery.{} 过长（最多 {MAX_BALANCE_QUERY_FIELD_CHARS} 个字符）",
                balance_query_field_label(key)
            ));
        }
        let before = query.get(*key).and_then(Value::as_str).unwrap_or("");
        if before == next {
            continue;
        }
        if BALANCE_QUERY_IDENTITY_FIELDS.contains(key) {
            identity_changed = true;
        }
        if next.is_empty() {
            query.remove(*key);
        } else {
            query.insert((*key).to_string(), Value::String(next));
        }
    }
    // → （ use ）
    if identity_changed {
        query.remove("sensenovaJwt");
        query.remove("sensenovaJwtExpiresAt");
    }
    Ok(())
}

/// → （** and ， not **）。
/// （sensenovaJwt） not ： not is use 。
fn describe_balance_query_changes(
    before: &Map<String, Value>,
    after: &Map<String, Value>,
) -> Vec<String> {
    let mut changes: Vec<String> = Vec::new();
    for key in BALANCE_QUERY_FIELDS {
        let old = before.get(*key).and_then(Value::as_str).unwrap_or("");
        let new = after.get(*key).and_then(Value::as_str).unwrap_or("");
        if old == new {
            continue;
        }
        let label = balance_query_field_label(key);
        changes.push(match (old.is_empty(), new.is_empty()) {
            (true, false) => format!("查询凭证 → {label} 已配置"),
            (false, true) => format!("查询凭证 → {label} 已清除"),
            _ => format!("查询凭证 → {label} 已更新"),
        });
    }
    changes
}

/// ****（`custom_credential_by_id` /
/// `first_custom_credential` return）。
///
/// and （`to_custom_public_account`） has ：
/// `apiKey`（， use ） and ** already **（`ResolvedProxy`，
/// can to egress；failure by —— that is to use 
/// 「failure」， this is can ）。
pub struct CustomCredential {
    /// id
    pub account_id: String,
    /// （ use ）
    pub account_name: String,
    /// up API Key（`add_custom_account` ；** can empty** —— none need 
    /// no has key， and not ）
    pub api_key: String,
    /// is 「 this up none need 」（ `state::no_auth`）
    pub no_auth: bool,
    /// up baseUrl ****（None = ， baseUrl）
    pub base_url_override: Option<String>,
    /// （None = not configured / failure → ）
    pub proxy: Option<ResolvedProxy>,
}

/// 1 → （ use 1 ， not will 1 ）
fn credential_of_record(record: &StoredAccount) -> CustomCredential {
    let fields = record.fields();
    // down 1 ： （、） is ****，
    // and 1 ； that 1 （） `describe_account_proxy`，
    // not （ `core::proxies` ）。
    let proxy = match resolve_account_proxy_rotating(Some(&record.proxy()), record.id()) {
        Some(resolution) => resolution.resolved().cloned(),
        // noneconfig → （ and 「failure」 in as up etc ，
        // in ： this no has request up down can notice， can ）
        None => None,
    };
    CustomCredential {
        account_id: record.id().to_string(),
        account_name: record.name(),
        api_key: fields
            .get("apiKey")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        no_auth: record.no_auth(),
        base_url_override: fields
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|text| !text.is_empty()),
        proxy,
    }
}
