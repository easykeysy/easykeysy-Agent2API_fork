//! Loomy ：、 session、。
//!
//! ── （ for clientresponse ）────────────────────
//! ```text
//! ：{ id, provider:"loomy", name, userId, phone, accessToken(=session),
//!                tokenTail, addedAt, updatedAt, source, priority, enabled }
//! clientresponse：{ session, userid, phone }（data.data， providers/loomy/login.rs）
//! ```
//! `accessToken` is ** session**（ when is `token`）——
//! and 1 （`has_token()` / `access_token()` 1 this ），
//! `session` / `token`。
//!
//! ── Why not `expiresAt` ──────────────────────────────────
//! up response has `{session, userid, phone}`，** no has to when **。 by 「
//! when + 14 」 1 `expiresAt` valid， use 
//! 「expired」， up （ to after can to 、 can ）。 not is 
//! ， in not 、 not ： up `020002` / `100002`， that when by to 
//! `needsRelogin`（ `upstream::provider_loop`）。
//! up can can `expiresAt`：file not update，
//! ** not **（ [`AccountStore::to_loomy_public_account`]）。
//!
//! ── and path ────────────────────────────────────
//! use this 1 ：（`api::session::login_loomy_sms_verify`）、
//! session（`api::accounts::add_account` loomy ）。** no has **
//! —— Loomy not in can file（ `credentials.rs` module header）。
//!
//! file is 「--」file，** no has request**（ not ）。
//! not unwrap/expect（release is panic=abort）。

use serde_json::{Map, Value};

use crate::server::core::account_store::priority::next_free_priority;
use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::{mark_name_custom, StoredAccount};
use crate::server::core::account_store::{AccountStore, AccountStoreError, LOOMY_PROVIDER_ID};
use crate::server::logging;

/// limit（ and 1 ）
const MAX_NAME_LENGTH: usize = 100;

/// （userId / phone）limit
const MAX_IDENTITY_LENGTH: usize = 256;

/// session limit（ this 1 is ）
const MAX_SESSION_LENGTH: usize = 8192;

/// to `max` ****（ not is ； can can has in ）
fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    value.chars().take(max).collect()
}

/// session 4 （ ，）
fn session_tail(session: &str) -> String {
    let chars: Vec<char> = session.chars().collect();
    let start = chars.len().saturating_sub(4);
    chars[start..].iter().collect()
}

/// from payload 1 empty（ and ）
fn pick(payload: &Value, keys: &[&str]) -> String {
    for key in keys {
        let value = payload
            .get(*key)
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if !value.is_empty() {
            return value.to_string();
        }
    }
    String::new()
}

impl AccountStore {
    /// 1 Loomy （ JSON）。
    ///
    /// `account_id` empty → by id （ not return None）；
    /// as empty → **enabled in ** before 1 （ and `autoclaw_account_record`
    /// 1 ：default use ）。
    pub fn loomy_account_record(&self, account_id: &str) -> Option<Value> {
        let guard = self.guard();
        if !account_id.is_empty() {
            let record = self.record_by_id(&guard, account_id)?;
            return (record.provider() == LOOMY_PROVIDER_ID).then(|| record.to_value());
        }
        let mut candidates: Vec<StoredAccount> = self
            .records_for_provider(&guard, LOOMY_PROVIDER_ID)
            .into_iter()
            .filter(StoredAccount::enabled)
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next().map(|item| item.to_value())
    }

    /// /update 1 Loomy （ and session use ）。
    ///
    /// payload（ 1 ）：
    /// - `session` / `accessToken` / `token`： session（）；
    /// - `userId` / `userid`： userid；
    /// - `phone`：（optional， use ）。
    ///
    /// payload `expiresAt` ** ignore**： up not to to when ， not is 
    /// （module header）。
    ///
    /// ：session empty and not ；userId and phone has 1 （otherwise not 
    /// can ，`loomy-` before id will in 1 ）。
    ///
    /// id（ already has id ） when ： provider update（ and enabled
    /// 、 use use ）， to ** provider** when ——
    /// not （ and 1 ）。
    pub fn add_loomy_account(
        &self,
        payload: &Value,
        name: Option<&str>,
    ) -> Result<Value, AccountStoreError> {
        if !payload.is_object() {
            return Err(AccountStoreError::new("上传内容必须是 JSON 对象", 400));
        }
        let session = pick(payload, &["session", "accessToken", "token"]);
        if session.is_empty() {
            return Err(AccountStoreError::new(
                "缺少 session（accessToken / session / token）",
                400,
            ));
        }
        if session.chars().count() > MAX_SESSION_LENGTH {
            return Err(AccountStoreError::new("session 过长", 400));
        }
        let user_id = truncate_chars(&pick(payload, &["userId", "userid", "user_id"]), MAX_IDENTITY_LENGTH);
        let phone = truncate_chars(&pick(payload, &["phone"]), MAX_IDENTITY_LENGTH);
        if user_id.is_empty() && phone.is_empty() {
            return Err(AccountStoreError::new(
                "缺少 userId 或手机号：至少给一个才能生成可辨认的账号身份",
                400,
            ));
        }
        // ： userid（）； no has 
        let identity = if user_id.is_empty() { phone.clone() } else { user_id.clone() };
        let id = format!("loomy-{identity}");

        let guard = self.guard();
        let existing = self.record_by_id(&guard, &id);
        if let Some(existing) = existing.as_ref() {
            let existing_provider = existing.provider();
            if existing_provider != LOOMY_PROVIDER_ID {
                return Err(AccountStoreError::new(
                    format!(
                        "账号 id「{id}」已被{existing_provider}账号占用，无法添加同一身份的\
                         Loomy 账号（请先处理那个账号）"
                    ),
                    400,
                ));
            }
        }
        // ： → has → → userId
        let explicit_name = name
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| truncate_chars(value, MAX_NAME_LENGTH));
        let record_name = explicit_name
            .or_else(|| {
                existing
                    .as_ref()
                    .map(StoredAccount::name)
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| {
                if phone.len() >= 11 {
                    format!("Loomy {}", crate::server::core::providers::loomy::mask_phone(&phone))
                } else if !user_id.is_empty() {
                    format!("Loomy {user_id}")
                } else {
                    "Loomy 账号".to_string()
                }
            });
        let priority = match existing.as_ref() {
            Some(record) => record.priority(),
            None => {
                // 1 （ use 1 ）， already use 
                let used = self.with_conn(&guard, |conn| sql::priorities_all(conn))?;
                next_free_priority(&used)
            }
        };
        let now = logging::now_ms();

        let mut record = Map::new();
        record.insert("id".to_string(), Value::String(id.clone()));
        record.insert("provider".to_string(), Value::String(LOOMY_PROVIDER_ID.to_string()));
        record.insert("name".to_string(), Value::String(record_name.clone()));
        // ： use or has already when ，
        // 「 etc default」 is 「 as 」。 and its 1 
        // （ `state::mark_name_custom`）—— path（ not 
        // `store_crud::upsert_account` 1 ），must in this use ，
        // will when in will 。
        mark_name_custom(&mut record, name.is_some_and(|value| !value.trim().is_empty()), existing.as_ref());
        if !user_id.is_empty() {
            record.insert("userId".to_string(), Value::String(user_id));
        }
        if !phone.is_empty() {
            record.insert("phone".to_string(), Value::String(phone));
        }
        record.insert("accessToken".to_string(), Value::String(session.clone()));
        record.insert("tokenTail".to_string(), Value::String(session_tail(&session)));
        // not `expiresAt`： up not to to when （module header）。 new when 
        // up —— will old 。
        record.remove("expiresAt");
        record.insert("source".to_string(), Value::String("manual".to_string()));
        record.insert("priority".to_string(), Value::from(priority));
        record.insert(
            "enabled".to_string(),
            Value::Bool(
                existing
                    .as_ref()
                    .map(StoredAccount::enabled)
                    .unwrap_or(true),
            ),
        );
        record.insert(
            "addedAt".to_string(),
            Value::from(
                existing
                    .as_ref()
                    .map(StoredAccount::added_at)
                    .filter(|value| *value != 0)
                    .unwrap_or(now),
            ),
        );
        record.insert("updatedAt".to_string(), Value::from(now));
        // unknown（ use not can as 1 「update」）
        let mut merged = record;
        if let Some(existing) = existing.as_ref() {
            for (key, value) in existing.fields() {
                merged.entry(key.clone()).or_insert_with(|| value.clone());
            }
        }
        let saved = StoredAccount::from_map(merged);
        // ：`put` = DELETE + INSERT（ and 1 ）
        self.with_conn(&guard, |conn| sql::put(conn, &saved))?;
        logging::log(
            "[Accounts]",
            &format!("✅ Loomy 账号已保存: {record_name}（优先级 {priority}）"),
        );
        Ok(self.public_account(&saved))
    }

    /// Loomy ****（ HTTP response）： session ， 4 。
    ///
    /// when `expiresAt`： up that is （ when + 14 ），
    /// not is up to ， down will available 「expired」。
    /// by `needsRelogin` （ in to `020002` / `100002` when ）。
    pub fn to_loomy_public_account(&self, record: &StoredAccount) -> Value {
        let mut value = record.to_value();
        if let Some(object) = value.as_object_mut() {
            object.remove("accessToken");
            object.remove("expiresAt");
            object.insert("available".to_string(), Value::Bool(record.enabled()));
            object.insert("hasSession".to_string(), Value::Bool(true));
        }
        value
    }
}
