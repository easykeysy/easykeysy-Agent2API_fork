//! API Key （config.json `apiKeys` ）。
//!
//! ：`apiKeys: [{id, name, key, enabled, createdAt, allowedProviders, allowedModels}]`。
//! use must in 1 enabled、 can and empty Key； no has available Key when use 。
//!
//! ── and old `apiKey` ────────────────────────────────
//! 1.x has 1 Key（`apiKey` ）。：`apiKeys` `apiKey` in when ，
//! 1 id as `legacy` / ； 1 `apiKeys`
//! and `apiKey`， from has 1 。 `WORKBUDDY_PROXY_API_KEY` 
//! 1 enabled Key（ not ）。
//! this from is `enc1:` （
//! `core::legacy_api_key`，、 and not ），must
//! `decode_key` —— and `key` 1 。
//!
//! ── Key **available / available**（R9， OmniProxy）──
//! ， and OmniProxy `allowed_models` / `allowed_provider_ids`
//! ** for **（ that 1 `server/dist/utils/apiKeyRestrictions.js`）：
//! - `allowedProviders: string[]`： by to **provider id**
//!     （`workbuddy` / `raccoon` / `catpaw` / `autoclaw` / `qoder` /
//! `cline-free` / `cline-pass`）。OmniProxy is **** provider id
//! （ up ）， this provider is 
//! （ `providers::PROVIDERS` ：「 is 」），
//! this use —— not need as 「 for 」 id 。
//! - `allowedModels: string[]`：request ** for **（ down request 
//! that ， alias）。
//!
//! **empty / = unlimited**： this and OmniProxy 「`NULL` = unlimited」 etc 
//! （ this `apiKeys` is JSON ， can empty， not need need 1 null
//! ）。 ** when when by **（ need in ， and need 
//! in ）， and OmniProxy 1 。
//!
//! ── 1 not ： not can already has Key ────────────────────
//! old no has this ，`from_value` empty `Vec`（= unlimited）；
//! **path not as **（ that etc use Key ，
//! allclient 401）。 this 「 not to use empty」，
//! not —— `allowedProviders` unknown provider id 
//! （ can can new 、 old to ； in API ）。
//! when **total is ** this （empty is `[]`）：config，
//! use config when this in 。
//!
//! ── `key` is （ ）───────────────────────────
//! `key`（kv `apiKeys` that JSON `key` ） not is 
//! `sk-a2a-…` ， is `core::crypto` `enc1:` （AES-256-GCM）。
//! is [`SecretScope::Accounts`] —— and ** use ** `account.key` /
//! `AGENT2API_ACCOUNT_KEY`，** not add**： Key and 「」，
//! no has need ， 1 1 need / file、 1 
//! 「 no 」 。 and its （id / name / enabled /
//! createdAt / ）** JSON**：「 is 」 
//! that 1 —— its need to （config、）。
//!
//! ， this file when not need ：
//! 1. ** and for is **。`ApiKeyEntry::key` in is ，
//! `public_json()` return（ need can to client）。
//! in `to_value()` this ** 1 ** —— and `encode_data`
//! 1 to （ `core::account_store::sql` module header 1 not ）。
//! 2. **failure not **（fail closed）。`to_value` return `Result`，
//! failure → failure and up （ `save`）： is 「 
//! when not to 」，failure in this （file
//! not etc ）—— etc 「 this 」， and use from up not 。
//! 3. ** not `key` not 、 not 、 not **（ down that not ，
//! this is add 1 ）。file / when `crypto::open` failure， when 
//! `from_value` ：、**** in `key`
//! （ not empty、 not ）、 1 。

use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::crypto::{self, SecretScope};
use crate::server::logging;

/// ：`apiKeys` （ ）
pub const KEY_API_KEYS: &str = "apiKeys";
/// old ： Key 。 in （`entries_from` ），
/// **** —— `config::set_api_key`（、 `enc1:` ）
/// and （`core::legacy_api_key`）； path in 
/// config （`set_api_key(None)`、`replace_api_keys` to `apiKeys` when ）。
/// `RuntimeConfig::masked_api_key`（， in 
/// that 1 ）。
///
/// and `KEY_API_KEYS` and ， is as this has 1 ： up this 
/// （ `config::update_raw_field` ） use is ，
/// 1 1 ；only `config::build` that 1 is 
/// old 。
pub const KEY_API_KEY: &str = "apiKey";
const LEGACY_ID: &str = "legacy";
const LEGACY_NAME: &str = "默认 Key";
/// and old 1 
pub const MIN_KEY_LENGTH: usize = 8;

/// 1 Key 
#[derive(Clone, Debug)]
pub struct ApiKeyEntry {
    pub id: String,
    pub name: String,
    pub key: String,
    pub enabled: bool,
    pub created_at: i64,
    /// available（provider id ）；**empty = unlimited**（module header）
    pub allowed_providers: Vec<String>,
    /// available（** for **， alias）；**empty = unlimited**
    pub allowed_models: Vec<String>,
    /// Expiration time (ms timestamp, UTC); `None` = never expires
    pub expires_at: Option<i64>,
    /// Daily request limit (UTC day window); `None` = unlimited
    pub daily_request_limit: Option<u64>,
    /// Total request limit; `None` = unlimited
    pub total_request_limit: Option<u64>,
    /// Daily token limit (best-effort, UTC day window); `None` = unlimited
    pub daily_token_limit: Option<u64>,
    /// Total token limit (best-effort); `None` = unlimited
    pub total_token_limit: Option<u64>,
    /// Requests-per-minute limit (RPM, sliding window); `None` = unlimited
    pub rpm_limit: Option<u64>,
    /// this `key` ** not **（file / ），。
    ///
    /// as true when `key` is ** **（ not is ）：
    /// - not can and client for （ not etc → this Key 
    /// is when unavailable，** not will **）；
    /// - not can 1 （ will → use file，
    /// Key not ， `to_value`）。
    ///
    /// has ： this `key` ， this is 「 this 
    /// 」（ 1 is `to_value`）。
    key_unreadable: bool,
}

/// from JSON 1 「」（ / not for / empty → empty）。
///
/// ── Why 1 、 not ────────────────────────
/// this is **limit**： not when is 「unlimited」（），
/// not is 「」 or 「 this Key」。 1 old = use Key empty、
/// allclient 401； 「limit」= allrequest 403。 not can 。
/// this 「 can 」： to empty、、
/// empty、（ for ignore， `allows_model`）。
///
/// ： up is use that 。
fn string_list(value: Option<&Value>) -> Vec<String> {
    let Some(Value::Array(items)) = value else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else { continue };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        if out.iter().any(|known| known.eq_ignore_ascii_case(text)) {
            continue;
        }
        out.push(text.to_string());
    }
    out
}

/// → （**empty `[]`**，module header）
fn list_value(list: &[String]) -> Value {
    Value::Array(list.iter().map(|item| Value::String(item.clone())).collect())
}

/// 「 has 1 Key not 」 ** 1 **。
///
/// ── Whymust 1 ────────────────────────────────────
/// `from_value` in **path** up （ request `active_keys_from` 1 ，
/// by `entry_for_key_from` ）， `logging::log` is need ——
/// will in ， has use request。
/// this has 1 （file/）， to 1 ， use 
/// `AtomicBool::swap` ： and down is to that not need ， need is ** has 1 **。
fn warn_unreadable_once(id: &str, reason: &str) {
    static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    logging::log(
        "[Keys]",
        &format!(
            "⚠️  网关 Key「{id}」解密失败（{reason}）：记录与原始密文都已保留，这把 Key 暂时不可用（不会误放行）。恢复密钥文件后重启即可，或直接在管理页重存这把 Key"
        ),
    );
}

/// `key` → ，「 this 」。
///
/// 、：
/// - ****（ no has `enc1:` before ）：return，`false` —— 
/// ，（ `encrypt_legacy_keys`） will ，；
/// - ** and **：return，`false`；
/// - ** not **（file / /）：**return** + `true`，
/// and （ 1 ） 1 —— Why not can 、 not can empty、 not can ，
/// `from_value`。
///
/// 「 is not is 」 use `crypto::is_envelope`（ 1 in `core::crypto`），
/// not in this `starts_with("enc1:")` —— will （`enc2:`） not will 1 。
fn decode_key(raw: &str, id: &str) -> (String, bool) {
    if !crypto::is_envelope(raw) {
        return (raw.to_string(), false);
    }
    match crypto::open(raw, SecretScope::Accounts) {
        // `open` already UTF-8（ `core::crypto` module header「」），
        // this is String； by 「 not 」
        // （）， not 1 Key use 
        Ok(bytes) => match std::str::from_utf8(&bytes) {
            Ok(text) => (text.to_string(), false),
            Err(_) => {
                warn_unreadable_once(id, "解密结果不是合法 UTF-8 文本");
                (raw.to_string(), true)
            }
        },
        Err(error) => {
            warn_unreadable_once(id, &error);
            (raw.to_string(), true)
        }
    }
}

impl ApiKeyEntry {
    pub fn is_usable(&self) -> bool {
        self.enabled && !self.key_unreadable && !self.key.is_empty()
    }

    /// from 1 JSON （** and can **）。
    ///
    /// ── failureWhymust（file 1 ）────────
    /// file（`account.key`） / when `crypto::open` failure。 if 
    /// 「 / not for 」 that return None， this will `entries_from` 
    /// `filter_map` —— use Key from configempty： up not to 、
    /// not ， this ** not is use **（ is module header that not need 
    /// ， is Reason from 「」「」）。：
    /// - ， `key` in ** ** up and `key_unreadable`；
    /// - ** not empty**：empty will in for in 「empty token request」
    /// （`Bearer ` / empty `x-api-key`）， etc —— ；
    /// - ** not will **（client not can can ）， is 
    /// 「 this Key in when unavailable」， this is can 、 use 
    /// （file）；
    /// - when （`to_value`）， up 1 not ：
    /// use file after ， this ** old can Key**。
    ///
    /// ： no has `enc1:` before `key` is ， use （ `decode_key`）。
    fn from_value(value: &Value) -> Option<Self> {
        let raw = value.get("key")?.as_str()?.trim();
        if raw.is_empty() {
            return None;
        }
        let id = value
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .map(str::to_string)?;
        let (key, key_unreadable) = decode_key(raw, &id);
        let parse_u64 = |v: Option<&Value>| -> Option<u64> {
            match v {
                Some(Value::Number(n)) => n.as_u64().filter(|x| *x > 0),
                Some(Value::String(t)) => t.trim().parse::<u64>().ok().filter(|x| *x > 0),
                _ => None,
            }
        };
        let parse_i64 = |v: Option<&Value>| -> Option<i64> {
            match v {
                Some(Value::Number(n)) => n.as_i64().filter(|x| *x > 0),
                Some(Value::String(t)) => t.trim().parse::<i64>().ok().filter(|x| *x > 0),
                _ => None,
            }
        };
        Some(Self {
            id,
            name: value
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            key,
            enabled: !matches!(value.get("enabled"), Some(Value::Bool(false))),
            created_at: value.get("createdAt").and_then(Value::as_i64).unwrap_or(0),
            allowed_providers: string_list(value.get("allowedProviders")),
            allowed_models: string_list(value.get("allowedModels")),
            expires_at: parse_i64(value.get("expiresAt")),
            daily_request_limit: parse_u64(value.get("dailyRequestLimit")),
            total_request_limit: parse_u64(value.get("totalRequestLimit")),
            daily_token_limit: parse_u64(value.get("dailyTokenLimit")),
            total_token_limit: parse_u64(value.get("totalTokenLimit")),
            rpm_limit: parse_u64(value.get("rpmLimit")),
            key_unreadable,
        })
    }

    /// ：`key` `enc1:` ， its JSON（module header）。
    ///
    /// ── Whyreturn `Result` not is （fail closed）────────
    /// and `encode_data` 1 （ `core::account_store::sql` module header）：
    /// is 「file when not to 」，failure in this 
    /// （`config_dir` not can 、file not etc ）。 when 
    /// etc 「 this 」， use from up not 。failure
    /// ****failure and up ， use （`save` → API） error。
    ///
    /// ── （`key_unreadable`）Why ────────────────
    /// that `key` is not ****。 if 1 ， 
    /// 「 」： use after file， is in that ，
    /// Key not —— 1 、 not can 。 up 
    /// not ， after down 1 can 。
    fn to_value(&self) -> Result<Value, String> {
        let key = if self.key_unreadable {
            self.key.clone()
        } else {
            crypto::seal(self.key.as_bytes(), SecretScope::Accounts)?
        };
        Ok(json!({
            "id": self.id,
            "name": self.name,
            "key": key,
            "enabled": self.enabled,
            "createdAt": self.created_at,
            "allowedProviders": list_value(&self.allowed_providers),
            "allowedModels": list_value(&self.allowed_models),
            "expiresAt": self.expires_at,
            "dailyRequestLimit": self.daily_request_limit,
            "totalRequestLimit": self.total_request_limit,
            "dailyTokenLimit": self.daily_token_limit,
            "totalTokenLimit": self.total_token_limit,
            "rpmLimit": self.rpm_limit,
        }))
    }

    /// （ key： need can to client）。
    ///
    /// 1 is ： that when `key` is not （ `from_value`），
    /// this —— up is 、 up to ， use in when 
    /// to not is 1 「 no use 」 Key。
    pub fn public_json(&self) -> Value {
        let mut out = json!({
            "id": self.id,
            "name": self.name,
            "key": self.key,
            "masked": mask(&self.key),
            "enabled": self.enabled,
            "createdAt": self.created_at,
            "allowedProviders": list_value(&self.allowed_providers),
            "allowedModels": list_value(&self.allowed_models),
            "expiresAt": self.expires_at,
            "dailyRequestLimit": self.daily_request_limit,
            "totalRequestLimit": self.total_request_limit,
            "dailyTokenLimit": self.daily_token_limit,
            "totalTokenLimit": self.total_token_limit,
            "rpmLimit": self.rpm_limit,
        });
        if let Some(map) = out.as_object_mut() {
            map.insert(
                "governance".to_string(),
                crate::server::core::key_governance::status_json(self),
            );
        }
        out
    }
}

// ── in `core::key_scope`， this ** not ** 1 ──────────
// 「」： 、。（ignore、empty =
// unlimited、 ） in `key_scope::KeyScope` up ， that is 1 
// —— in this 1 for `allows_model` / `allows_provider`，
// dead_code ： 1 then ， 1 1 
// will 「 limit」 and 「 when limit」 not 1 ，
// that in up not （ is 200， is / ）。

/// ： before 6 after 4（ use old ）
pub fn mask(key: &str) -> String {
    let head: String = key.chars().take(6).collect();
    let total = key.chars().count();
    let tail: String = key.chars().skip(total.saturating_sub(4)).collect();
    format!("{head}...{tail}")
}

/// from config（ old ）
pub fn entries_from(raw: &Map<String, Value>) -> Vec<ApiKeyEntry> {
    if let Some(Value::Array(items)) = raw.get(KEY_API_KEYS) {
        return items.iter().filter_map(ApiKeyEntry::from_value).collect();
    }
    legacy_entry(raw).into_iter().collect()
}

/// old `apiKey` → 1 `legacy` （`None` = no / empty）。
///
/// ── Why need `decode_key` ──────────────────────────────────
/// `apiKey` from and `key` 1 is `enc1:` （ 
/// `core::legacy_api_key`），must and `from_value` and 
/// 1 ： use 、、 not by 。「 is not is 」 
/// 1 is `crypto::is_envelope`（ in `decode_key` ）， not in this 1 。
///
/// failure when **、 not empty、 not ** by and `from_value` 
/// （ that ）， this 「 old has 1 」 this ：
/// 1 enabled Key no has → in （）， 1 when use not 
/// 。
///
/// empty in ** before **（ and `from_value` 1 ）： is empty、
/// as empty this （ can can ） ——
/// and up 1 （ can 1 unavailable ， not empty）。
fn legacy_entry(raw: &Map<String, Value>) -> Option<ApiKeyEntry> {
    let stored = raw.get(KEY_API_KEY)?.as_str()?.trim();
    if stored.is_empty() {
        return None;
    }
    let (key, key_unreadable) = decode_key(stored, LEGACY_ID);
    Some(ApiKeyEntry {
        id: LEGACY_ID.to_string(),
        name: LEGACY_NAME.to_string(),
        key,
        enabled: true,
        created_at: 0,
        // old Key from no has —— empty = unlimited，
        // and before as （ 1 ） 1 
        allowed_providers: Vec::new(),
        allowed_models: Vec::new(),
        expires_at: None,
        daily_request_limit: None,
        total_request_limit: None,
        daily_token_limit: None,
        total_token_limit: None,
        rpm_limit: None,
        key_unreadable,
    })
}

/// up **** old → （ use ）。
///
/// use has `config::masked_api_key`： is （ up is 
/// is ， can can is `enc1:` ）， before must —— otherwise up will 1 
/// `enc1:b...xxxx`， use as Key 。
/// and `legacy_entry` use `decode_key`， is has 1 ：、
/// 、** not then return**（ ： up is 、
/// ， and `public_json` has to 1 ）。
///
/// Why not in （`config::build`） this ： that in `config::init` ，
/// no ，「file」 that 1 will in up （ down 
/// use not to ）—— `RuntimeConfig::masked_api_key` 。
pub fn stored_plaintext(stored: &str) -> String {
    decode_key(stored.trim(), LEGACY_ID).0
}

/// current
pub fn list() -> Vec<ApiKeyEntry> {
    entries_from(config::current().raw())
}

/// currentenabled Key config（ and Environment variable key）， not is can use Key 。
/// not can need ， as not configured Key。
/// use by `is_usable` this 。
pub fn active_keys_from(raw: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = entries_from(raw)
        .into_iter()
        .filter(|entry| entry.enabled)
        .map(|entry| entry.key)
        .collect();
    if let Some(env) = crate::server::config::parse::env_api_key_value() {
        if !keys.contains(&env) {
            keys.push(env);
        }
    }
    keys
}

/// by Key currentavailable ；return None not already 。
/// Environment variable key by ，invalid、disabled or not can not will 。
pub fn entry_for_key_from(raw: &Map<String, Value>, key: &str) -> Option<ApiKeyEntry> {
    let env = crate::server::config::parse::env_api_key_value();
    if env.as_deref() == Some(key) {
        return None;
    }
    entries_from(raw)
        .into_iter()
        .find(|entry| entry.is_usable() && entry.key == key)
}

/// 1 Key：`sk-a2a-` + 32 。
///
/// not crate（ Cargo.toml for getrandom ）， this use 
/// sha256(seconds when + id + + ) before 16 ： 
/// ， is 「 to 」 not is ， this 。
pub fn generate() -> String {
    use sha2::{Digest, Sha256};
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let probe = 0u8;
    let address = &probe as *const u8 as usize;
    let mut hasher = Sha256::new();
    hasher.update(nanos.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(counter.to_le_bytes());
    hasher.update(address.to_le_bytes());
    let digest = hasher.finalize();
    let hex: String = digest.iter().take(16).map(|b| format!("{b:02x}")).collect();
    format!("sk-a2a-{hex}")
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// config（ when old `apiKey`）。
///
/// ── Why `?` up （fail closed）─────────────────────────
/// 1 `to_value` failure → ** 1 not **：`replace_api_keys`
/// not will to ， and ， use 「failure」 to use 。
/// not will 「success Key is 」 this not 。
///
/// ── failure（unavailable）Why not up ──────────────────────────
/// `config::replace_api_keys` return false when already 
/// （ `config::save_raw`）， and config is 「failure after in not in ，
/// not this in request or config」（ `config::update` ）——
/// this use ， not 1 API 。failure and failure in ：
/// before will down **error** （）， after is no to up 。
fn save(entries: &[ApiKeyEntry]) -> Result<(), String> {
    let mut list: Vec<Value> = Vec::with_capacity(entries.len());
    for entry in entries {
        list.push(entry.to_value()?);
    }
    let _ = config::replace_api_keys(Value::Array(list));
    Ok(())
}

/// `apiKeys` ** is ** `key` （0 = this config not need need ）。
///
/// ── Why need in JSON up ， not is in up ─────────────
/// `from_value` `key` ** is **（ is / ），
/// from not need need —— has no `decode_key` up 
/// 「 is not is 」。 to 1 （ not is 「 1 」）。
///
/// and `from_value` 1 （`key` and `id` is empty）：
/// will `filter_map` not will ， not this 。
/// old `apiKey`（ Key ）** not ** —— not in `apiKeys` ，
/// （ old ， `core::legacy_api_key`）。
fn plaintext_key_count(raw: &Map<String, Value>) -> usize {
    let Some(Value::Array(items)) = raw.get(KEY_API_KEYS) else {
        return 0;
    };
    items
        .iter()
        .filter(|item| {
            let key = item.get("key").and_then(Value::as_str).map(str::trim);
            let id = item.get("id").and_then(Value::as_str).map(str::trim);
            matches!(
                (key, id),
                (Some(key), Some(id))
                    if !key.is_empty() && !id.is_empty() && !crypto::is_envelope(key)
            )
        })
        .count()
}

/// ** is ** Key （ 1 ， etc ， can use ）。
///
/// ── is ────────────────────────────────────────────
/// `key` is `enc1:` ，** before already config ** is 
/// （`from_value` ， that Key available —— that is 、
/// not is ： need has ，file）。 in when 
/// ， and `encrypt_legacy_rows` （、 etc 、failure for ）。
///
/// ── etc ： ──────────────────────────────────────
/// is 「`apiKeys` has no has not `enc1:` `key`」，** not use kv **：
/// is can ， no has ， 1 in 1 、
/// config not 。 1 「 this no has 」 。
///
/// ── Why `save`（） not is that ────────────────
/// `apiKeys` is ** 1 ** kv 1 JSON ， no has 「 by update」 ——
/// ， its is new seal 1 （nonce 、 not 
/// ）， up no has （ and `config::replace_api_keys` has 1 ）。
///
/// ── failure not ──────────────────────────────────────────
/// unavailable / unavailable when `save` will failure： this 1 and return 0 ——
/// ， Key available， down 。
///
/// ── old `apiKey` Why not in this ──────────────────────────
/// not in `apiKeys` ， not will in 「」 。
/// `apiKeys` is ** not can ** ： that will before 「 1 
/// `apiKeys` and `apiKey`」 this has （ use no config，）。
/// is 1 ：****（ and not ， ），
/// `core::legacy_api_key::encrypt_legacy_api_key` —— and and ， by 
/// `account_bootstrap::run` use 。
pub fn encrypt_legacy_keys() -> usize {
    // 1 、 use （ + ）：config is ，
    // not as 1 raw （ and path 1 ， this is for ）
    let raw = config::current().raw().clone();
    let pending = plaintext_key_count(&raw);
    if pending == 0 {
        // new / already this path： 1 not 
        return 0;
    }
    let entries = entries_from(&raw);
    if let Err(error) = save(&entries) {
        logging::log(
            "[Keys]",
            &format!(
                "⚠️  网关 Key 加密未完成（下次启动会自动重试，本次 Key 照常可用）: {error}"
            ),
        );
        return 0;
    }
    logging::log("[Keys]", &format!("🔒 已加密 {pending} 把网关 Key"));
    pending
}

/// add：`key` as empty then 。return new ；unlimited，key 。
///
/// 1 （empty = unlimited）。** not in this 1 provider id**
/// is in ： that is API（） —— is 「 + 」，
/// in in can 「」 has 1 ，
/// （module header that not ）。
pub fn add(
    name: &str,
    key: Option<&str>,
    allowed_providers: Vec<String>,
    allowed_models: Vec<String>,
    expires_at: Option<i64>,
    daily_request_limit: Option<u64>,
    total_request_limit: Option<u64>,
    daily_token_limit: Option<u64>,
    total_token_limit: Option<u64>,
    rpm_limit: Option<u64>,
) -> Result<ApiKeyEntry, String> {
    let mut entries = list();
    let key = match key.map(str::trim).filter(|k| !k.is_empty()) {
        Some(given) => {
            if given.chars().count() < MIN_KEY_LENGTH {
                return Err(format!("API Key 至少需要 {MIN_KEY_LENGTH} 个字符"));
            }
            given.to_string()
        }
        None => generate(),
    };
    if entries.iter().any(|entry| entry.key == key) {
        return Err("这把 Key 已经存在".to_string());
    }
    let created_at = now_millis();
    let entry = ApiKeyEntry {
        id: format!("k{created_at:x}{}", entries.len()),
        name: name.trim().to_string(),
        key,
        enabled: true,
        created_at,
        allowed_providers,
        allowed_models,
        expires_at,
        daily_request_limit,
        total_request_limit,
        daily_token_limit,
        total_token_limit,
        rpm_limit,
        // Key is can ， and none
        key_unreadable: false,
    };
    entries.push(entry.clone());
    // failure（fail closed）→ this Key 1 no down （ no 、 no ），
    // error to use ， use to 「failure」 not is 1 in 
    save(&entries)?;
    Ok(entry)
}

/// / / （`None` = this not ， and 1 ）。
///
/// ── Why use `Option<Vec<String>>` not is `Vec<String>` ──────
/// `PATCH /api/keys/{id}` is update： old before `{name}` / `{enabled}`，
/// use `Vec` 「 not 」 can 「empty」， that is 「limit」 ——
/// is 「 in 」 will already has limit。`Option` this ：
/// `None` = not ，`Some(vec![])` = unlimited，`Some(list)` = list。
pub fn update(
    id: &str,
    name: Option<&str>,
    enabled: Option<bool>,
    allowed_providers: Option<Vec<String>>,
    allowed_models: Option<Vec<String>>,
    expires_at: Option<Option<i64>>,
    daily_request_limit: Option<Option<u64>>,
    total_request_limit: Option<Option<u64>>,
    daily_token_limit: Option<Option<u64>>,
    total_token_limit: Option<Option<u64>>,
    rpm_limit: Option<Option<u64>>,
) -> Result<ApiKeyEntry, String> {
    let mut entries = list();
    let Some(entry) = entries.iter_mut().find(|entry| entry.id == id) else {
        return Err("Key 不存在".to_string());
    };
    if let Some(name) = name {
        entry.name = name.trim().to_string();
    }
    if let Some(enabled) = enabled {
        entry.enabled = enabled;
    }
    if let Some(list) = allowed_providers {
        entry.allowed_providers = list;
    }
    if let Some(list) = allowed_models {
        entry.allowed_models = list;
    }
    if let Some(v) = expires_at { entry.expires_at = v; }
    if let Some(v) = daily_request_limit { entry.daily_request_limit = v; }
    if let Some(v) = total_request_limit { entry.total_request_limit = v; }
    if let Some(v) = daily_token_limit { entry.daily_token_limit = v; }
    if let Some(v) = total_token_limit { entry.total_token_limit = v; }
    if let Some(v) = rpm_limit { entry.rpm_limit = v; }
    let updated = entry.clone();
    save(&entries)?;
    Ok(updated)
}

/// delete
pub fn remove(id: &str) -> Result<(), String> {
    let mut entries = list();
    let before = entries.len();
    entries.retain(|entry| entry.id != id);
    if entries.len() == before {
        return Err("Key 不存在".to_string());
    }
    save(&entries)?;
    Ok(())
}

/// provider id from **all** Key 「available」（）。
///
/// use in 「delete 1 」 after （`api::custom_providers::remove_custom_provider`）：
/// no ， that id 1 ** in not ** limit —— limit this 1 
/// Key will 「 not 」（ request `key_scope` ）， Key up 
/// is 1 not `custom-xxxx`。 this to use can 。
///
/// ── Why in this 、 not in `custom_providers::remove` ──────────
/// `custom_providers` is config 1 ， `AccountStore` already is as 
/// （ 1 ， that ）； apiKeys this ， is 
/// 1 。 in API ——
/// that is 「 new + 」 。
///
/// ── return and 「 can can unlimited」 this ───────────────────────
/// return** **（ after ； no this Key not in its in ）， use by 
/// `allowed_providers.is_empty()` can 「 its in already as unlimited」。 this 1 must
/// use and ：**empty = unlimited**（module header）， 1 limit this 1 
/// Key is empty —— from 「 not 」「 can 」。 this is this 
/// ， not is can ；， 1 id this Key 
/// 404（ and up not Reason），、 by use in and up 。
///
/// for ignore（ and `key_scope` 、 1 ）：config
/// `CUSTOM-xxx` in when in this 1 （ will 1 ）， when not will 
/// 1 。** to this ** —— its 1 not （ 1 delete not this 
/// Key）。
pub fn strip_provider_from_allowlists(provider_id: &str) -> Vec<ApiKeyEntry> {
    let id = provider_id.trim();
    if id.is_empty() {
        return Vec::new();
    }
    let mut entries = list();
    let mut touched: Vec<ApiKeyEntry> = Vec::new();
    for entry in entries.iter_mut() {
        let before = entry.allowed_providers.len();
        entry
            .allowed_providers
            .retain(|known| !known.eq_ignore_ascii_case(id));
        if entry.allowed_providers.len() == before {
            continue;
        }
        touched.push(entry.clone());
    }
    if !touched.is_empty() {
        if let Err(error) = save(&entries) {
            // this pathreturn `Vec`（ not to use Result），failure can in this 。
            // 、** not return**： use （delete） not this as 1 
            // no —— that is 1 ，
            // down delete/ when will 1 （ etc ）。
            logging::log(
                "[Keys]",
                &format!("⚠️  网关 Key 白名单清理未落盘（改动仅本次运行生效）: {error}"),
            );
        }
    }
    touched
}

/// WorkBuddy （2026-10） ： to `workbuddy` Key up 
/// `workbuddy-intl`。
///
/// ── Whymust（ not after is 400）──────────────────────
/// before `workbuddy` this 1 id and ，「
/// WorkBuddy」 Key is ** can use **。 after is 1 id，
/// not that Key for will 「 this Key not use this 」——
/// use no ， can 1 ， to Key limitconfig，
/// and Reason（）none，。
///
/// ── Why is 「」 not is 「 workbuddy 」────────────
/// `workbuddy` not （ is that 1 id），****
/// —— before after available， no has no has 。
///
/// etc ： already when not 。** to ** ：
/// no （unlimited， or ） 1 not —— 1 Key limit is 
/// use （ and `strip_provider_from_allowlists` 1 ）。
/// return 。
pub fn add_intl_to_workbuddy_allowlists() -> Vec<ApiKeyEntry> {
    use crate::server::core::providers::{kind_id, ProviderKind};
    let cn = kind_id(ProviderKind::WorkBuddy);
    let intl = kind_id(ProviderKind::WorkBuddyIntl);
    let mut entries = list();
    let mut touched: Vec<ApiKeyEntry> = Vec::new();
    for entry in entries.iter_mut() {
        if !entry
            .allowed_providers
            .iter()
            .any(|known| known.eq_ignore_ascii_case(cn))
        {
            continue;
        }
        if entry
            .allowed_providers
            .iter()
            .any(|known| known.eq_ignore_ascii_case(intl))
        {
            continue;
        }
        entry.allowed_providers.push(intl.to_string());
        touched.push(entry.clone());
    }
    if !touched.is_empty() {
        if let Err(error) = save(&entries) {
            // and `strip_provider_from_allowlists` 1 ：return `Vec`、 not 
            // use （`account_bootstrap` ）；failure「 after in 
            // not in 」， this etc ， down will 1 。
            logging::log(
                "[Keys]",
                &format!("⚠️  网关 Key 白名单补齐未落盘（改动仅本次运行生效）: {error}"),
            );
        }
    }
    touched
}
