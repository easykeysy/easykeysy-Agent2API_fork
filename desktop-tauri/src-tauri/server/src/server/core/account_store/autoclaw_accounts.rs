//! AutoClaw ：** **（ this 1 ）。
//!
//! ── Why this no has 「 / / new 」 ─────────────────
//! AutoClaw already to ：
//!
//! ```text
//! / enc: / auth.json plugins/autoclaw/credentials.rs + crypto.rs
//! （ / file / ） plugins/autoclaw account.normalize
//! token + - plugins/autoclaw/refresh.rs + store::save
//! / OAuth plugins/autoclaw/login.rs + web_login.rs
//! ```
//!
//! new `pluginSecret`， by `account_view` to 
//! （`store_view::public_account` 1 ）。**file** ——
//! that is before ， no has `pluginSecret`， not to that 。
//!
//! ── Why not to when ──────────────────────
//! （`autoclaw-desktop` / `autoclaw-intl-desktop`） up not token，
//! when `%APPDATA%/AutoClaw/auth.json` and safeStorage —— that 
//! （DPAPI + AES-GCM） is ** up client**， already 。
//! in up token / expiration time will is empty 、`available` false， etc use 
//! up after new 1 can （ will 1 `pluginSecret` new ）。
//!
//! ── id Why in this ────────────────────────────
//! is ****： in ， / when 「 id」
//! （`account_transfer::identity::RESERVED_DESKTOP_IDS`）must，
//! otherwise 1 「 id not desktop 」 will id， after 
//! not when 。
//!
//! ── ────────────────────────────────────────────────
//! file is ，** no has request**； not unwrap/expect。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::{is_autoclaw_family, AccountStore};
use crate::server::core::providers::autoclaw::Region;

/// id（）。
pub const DESKTOP_ACCOUNT_ID: &str = "autoclaw-desktop";

/// id（ and **must not **： use 1 auth.json，
/// is ； id will 1 ）。
pub const INTL_DESKTOP_ACCOUNT_ID: &str = "autoclaw-intl-desktop";

/// by id。
pub fn desktop_account_id(region: Region) -> &'static str {
    match region {
        Region::Cn => DESKTOP_ACCOUNT_ID,
        Region::Intl => INTL_DESKTOP_ACCOUNT_ID,
    }
}

impl AccountStore {
    /// AutoClaw （ for `toPublicAccount`）。
    ///
    /// ：`id` / `provider` / `name` / `userId` / `email` / `deviceId` /
    /// `tokenTail` / `tokenExpiresAt` / `hasRefreshToken` / `desktop` / `source` /
    /// `priority` / `enabled` / `addedAt` / `updatedAt` / `proxy` / `rateLimits` /
    /// `available`。
    ///
    /// ── ， not when ─────────────────────────────
    /// after not auth.json 、 not （ that is ）。
    /// `tokenTail` / `tokenExpiresAt` can is 
    /// （ as empty），`available` by 「 has no has available token」。
    ///
    /// ── `email` Why not ───────────────────────────────
    /// ； not to （ etc ）** no has **
    /// that 1 ， not is 1 empty。
    pub fn to_autoclaw_public_account(&self, record: &StoredAccount) -> Value {
        debug_assert!(
            is_autoclaw_family(&record.provider()),
            "to_autoclaw_public_account 只接受 AutoClaw 系记录"
        );
        let proxy = crate::server::core::proxies::describe_account_proxy(Some(&record.proxy()));
        let token_tail = record
            .get("tokenTail")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let expires_at = record.expires_at().or_else(|| record.token_expires_at());
        let email = record
            .get("email")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let device_id = record
            .get("deviceId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        // by not token：available can by 「 has no has 」，
        // when by （module header）。
        let available = record.has_token() || !record.access_token().is_empty();
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        public.insert("userId".to_string(), Value::String(record.user_id()));
        if !email.is_empty() {
            public.insert("email".to_string(), Value::String(email));
        }
        public.insert("deviceId".to_string(), Value::String(device_id));
        public.insert("tokenTail".to_string(), Value::String(token_tail));
        public.insert(
            "tokenExpiresAt".to_string(),
            expires_at
                .map(crate::server::core::account_store::state::json_number)
                .unwrap_or(Value::Null),
        );
        public.insert(
            "hasRefreshToken".to_string(),
            Value::Bool(!record.refresh_token().is_empty()),
        );
        public.insert("desktop".to_string(), Value::Bool(record.is_desktop()));
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), proxy);
        public.insert(
            "rateLimits".to_string(),
            record
                .get("rateLimits")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        );
        public.insert("available".to_string(), Value::Bool(available));
        Value::Object(public)
    }
}
