//! use for ：AES-256-GCM + `enc1:` ， by `SecretScope` 。
//!
//! ── Why use ──────────────────────────────────────
//! this （32 ****、`enc1:<base64(nonce||+tag)>` 、
//! GCM Postfix tag 、failure when use 「 1 can 」） in 
//! `core::update::token` ， GitHub 。（`accounts`） need by 
//! by ：config will / / ， in etc 
//! 「 1 file」。 and its that （ will 、
//! 「 1 1 no 」 ）， not ** and none** that 1 to this ：
//! · ：、、file //、error；
//! · ：、 not when need not need 、 for API 
//! （ this and ， `update/token.rs` module header）。
//!
//! ── （`SecretScope`）──────────────────────────────
//! `UpdateToken` → file `update-token.key`
//! `Accounts` → file `account.key`
//! ** has 、 has **， not use ： use 1 to 
//! 1 1 （ == ）， when no 1 。 is 
//! need —— will after before 1 ， to is 
//! 「failure」 this 。
//! ：`core::key_scope.rs` `KeyScope` is 「request in API Key
//! /limit」， and none， this `SecretScope`。
//!
//! ── file in `config_dir`， and directory（ need ）──
//! file and `agent2api.db` in `paths::config_dir()` down 。（`token.rs` old 
//! is `config::config_dir()`， that is `paths::config_dir` 1 ， to 1 
//! directory； this 1 `paths::config_dir()`， 1 。） is ** **
//! this 1 （ / / ）， not to or `config_dir` 。
//! this is 「 」 limit： /
//! KMS need need 、 need ， not is 。 this 
//! 「file not to 」。
//!
//! ── `AGENT2API_ACCOUNT_KEY` Whymust in ──────────────
//! file** not ** already —— this is ： can 
//! etc 。 down use 1 can ， Docker no has 「 use 」 this 
//! ： when if `config_dir` no ，file 1 ，
//! not 。 1 `AGENT2API_ACCOUNT_KEY`
//! （ = 32 **64 **， compose/.env， not ）：
//! and 1 in ，。
//! set / not is 64 / → **file**（ is 1 
//! ， not this unavailable）， 1 verbose，「 env no use up 」
//! in no 。
//! `UpdateToken` ** not ** this env：GitHub has `GITHUB_TOKEN` 
//! （ `update/token.rs` `env_token`）， that is 「」 ， and 「」
//! not ； 1 will 「 to from 」none from 。
//!
//! ── ：`open` UTF-8 ──────────────────────────
//! old `token.rs::decrypt` return `String`， UTF-8 to is 「 not is 
//! UTF-8 」， `status_json` will this to use —— for as 。
//! 1 ：`open` return `Vec<u8>`（ for this use ，
//! use ）， in return before 1 UTF-8，all has use errorpath not 。
//! is （、 JSON）， this not will 。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;

use crate::server::logging;

/// before （ when old ，e.g. will use ）
const ENVELOPE_PREFIX: &str = "enc1:";

/// this is not is （`enc1:` ）。
///
/// use 「 is is 」——e.g. `data` before 
/// is JSON、 after is （ `core::account_store::sql`）。Why need this 
/// not is use `starts_with("enc1:")`： that before is ** **，
/// to has 「」 ， will （ `enc2:`）
/// 1 。 1 （）， before 1 （ up ）。
pub fn is_envelope(text: &str) -> bool {
    text.starts_with(ENVELOPE_PREFIX)
}

/// GCM nonce （96 bit，）
const NONCE_LEN: usize = 12;

/// GCM （128 bit）。used for down —— tag in ，
/// to aes-gcm Postfix ， not （ `open` ）
const TAG_LEN: usize = 16;

/// （AES-256）
const KEY_LEN: usize = 32;

/// （ = 32 64 ，module header）
const ACCOUNT_KEY_ENV: &str = "AGENT2API_ACCOUNT_KEY";

/// ： use 、 file、。
///
/// 1 = 1 + to `key_file` / `recovery_hint` 1 ，
/// its （、、） use 。
#[derive(Clone, Copy)]
pub enum SecretScope {
    /// GitHub （`core::update::token`）。file is ， not 
    UpdateToken,
    /// （`accounts`）。：supports `AGENT2API_ACCOUNT_KEY` 
    Accounts,
    /// 「」（`core::cloud_sync`）。
    ///
    /// ── Whymust is ──────────────────────────────────
    /// need use ** that `account.key`**， 
    /// in 。 if ， down and `account.key`
    /// after ， that not —— use will to 「
    /// no ，」。 1 `cloud-sync.key` this 、 not and 
    /// ， this not in 。
    CloudSync,
}

impl SecretScope {
    /// file（ in `paths::config_dir()` down ， and directory）。
    ///
    /// `update-token.key` **must**： etc file， already use 
    /// up GitHub will not ， use not will to 「 1 」。
    pub(crate) fn key_file(self) -> &'static str {
        match self {
            SecretScope::UpdateToken => "update-token.key",
            SecretScope::Accounts => "account.key",
            SecretScope::CloudSync => "cloud-sync.key",
        }
    }

    /// failure when use 「」 after （ before ， `open`）。
    ///
    /// by not is 1 ： this will `status_json` to up ，
    /// to will use 1 not in 。
    fn recovery_hint(self) -> &'static str {
        match self {
            SecretScope::UpdateToken => "重存一次令牌即可自愈",
            SecretScope::Accounts => "重新保存一次账号即可自愈",
            SecretScope::CloudSync => "在「设置 → 数据 → 云同步」里重新输入同步密码即可自愈",
        }
    }
}

// ─── （ / ， by in ）───────────────

/// 。
///
/// Why 1 `OnceLock`：must** by **（ after module header），
/// need is 「 1 」 。 1 `HashMap`
/// ： is already ， no has 。
fn cache_for(scope: SecretScope) -> &'static std::sync::Mutex<Option<[u8; KEY_LEN]>> {
    static UPDATE_TOKEN: std::sync::OnceLock<std::sync::Mutex<Option<[u8; KEY_LEN]>>> =
        std::sync::OnceLock::new();
    static ACCOUNTS: std::sync::OnceLock<std::sync::Mutex<Option<[u8; KEY_LEN]>>> =
        std::sync::OnceLock::new();
    static CLOUD_SYNC: std::sync::OnceLock<std::sync::Mutex<Option<[u8; KEY_LEN]>>> =
        std::sync::OnceLock::new();
    match scope {
        SecretScope::UpdateToken => UPDATE_TOKEN.get_or_init(|| std::sync::Mutex::new(None)),
        SecretScope::Accounts => ACCOUNTS.get_or_init(|| std::sync::Mutex::new(None)),
        SecretScope::CloudSync => CLOUD_SYNC.get_or_init(|| std::sync::Mutex::new(None)),
    }
}

/// （ no has 1 new ）。allfailure in error up 。
///
/// Why use `Mutex<Option<…>>` not is `OnceLock` ： need 
/// ** not in **，failure（directory not can etc ） not can failure ——
/// down can ； when and need （ not 、
/// file， that not ）。
pub(crate) fn secret_key(scope: SecretScope) -> Result<[u8; KEY_LEN], String> {
    let cell = cache_for(scope);
    let Ok(mut guard) = cell.lock() else {
        return Err("密钥锁中毒".to_string());
    };
    if let Some(key) = *guard {
        return Ok(key);
    }
    let key = resolve_key(scope)?;
    *guard = Some(key);
    Ok(key)
}

/// this current this use ：（ has has ）file。
///
/// and file** is 「 1 」 **， not is ：Docker 
/// env is that file 。 this env、 not to file；env 1 
/// valid not （ not not need need file）——empty 1 file is 
/// 1 need 。
fn resolve_key(scope: SecretScope) -> Result<[u8; KEY_LEN], String> {
    if let Some(key) = env_key(scope) {
        return Ok(key);
    }
    load_or_create_key(scope)
}

/// （ has ，module header）。set / 1 None。
fn env_key(scope: SecretScope) -> Option<[u8; KEY_LEN]> {
    // `UpdateToken` no has env ： has GITHUB_TOKEN（module header）
    let SecretScope::Accounts = scope else {
        return None;
    };
    let Some(raw) = std::env::var(ACCOUNT_KEY_ENV).ok() else {
        return None;
    };
    match decode_hex_key(&raw) {
        Some(key) => Some(key),
        None => {
            // file，must：env is 「 can 」 1 
            // ， no ， etc this not in 
            logging::verbose(
                "[Crypto]",
                &format!(
                    "{ACCOUNT_KEY_ENV} 不是 64 位十六进制（实际 {} 个字符），本次回落密钥文件",
                    raw.trim().len()
                ),
            );
            None
        }
    }
}

/// 64 → 32 （ can ）。return None。
///
/// not is `hex` crate： has this 1 use to ， as 20 1 not 
/// （「 not add」 ， `Cargo.toml` ）。
fn decode_hex_key(value: &str) -> Option<[u8; KEY_LEN]> {
    let text = value.trim();
    if text.len() != KEY_LEN * 2 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut key = [0u8; KEY_LEN];
    for (index, slot) in key.iter_mut().enumerate() {
        let high = hex_nibble(bytes[index * 2])?;
        let low = hex_nibble(bytes[index * 2 + 1])?;
        *slot = (high << 4) | low;
    }
    Some(key)
}

/// → 4 bit
fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// this file； not in then and （Unix up to 0600）。
fn load_or_create_key(scope: SecretScope) -> Result<[u8; KEY_LEN], String> {
    let path = crate::paths::config_dir().join(scope.key_file());
    match std::fs::read(&path) {
        Ok(bytes) => match <[u8; KEY_LEN]>::try_from(&bytes[..]) {
            Ok(key) => Ok(key),
            Err(_) => Err(format!(
                "密钥文件 {} 长度异常（期望 {KEY_LEN} 字节，实际 {} 字节）",
                path.display(),
                bytes.len()
            )),
        },
        // not in new ； its error（ etc ） up —— will 
        // already not 
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => create_key_file(&path),
        Err(error) => Err(format!("读取密钥文件 {} 失败: {error}", path.display())),
    }
}

/// 32 and 
fn create_key_file(path: &std::path::Path) -> Result<[u8; KEY_LEN], String> {
    let mut key = [0u8; KEY_LEN];
    getrandom::getrandom(&mut key).map_err(|error| format!("系统随机源不可用: {error}"))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| format!("创建配置目录失败: {error}"))?;
    }
    std::fs::write(path, key).map_err(|error| format!("写入密钥文件 {} 失败: {error}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(key)
}

// ─── （， not ）───────────────────────────

/// this ** down ** ； not in / not can / → `None`。
///
/// and `secret_key`（ or ） ：** no has **， can use in 
/// 「 is in use is 」 （ when ）。 not can use 
/// `secret_key`： that path in file not in when will **** 1 new —— 1 
/// in use up empty `account.key`， etc 
/// （ down when use is this 「 when 」，）。
///
/// must and `resolve_key` for （ → → file），otherwise
/// will 1 ** not is ** down in use ， for 。
fn existing_key(scope: SecretScope) -> Option<[u8; KEY_LEN]> {
    // 1) already ： is seal/open in use ，
    if let Ok(guard) = cache_for(scope).lock() {
        if let Some(key) = *guard {
            return Some(key);
        }
    }
    // 2) （ has ，module header）； and resolve_key 1 
    if let Some(key) = env_key(scope) {
        return Some(key);
    }
    // 3) file：；「 not in 」 this 1 ** not **（ and load_or_create_key ）
    let path = crate::paths::config_dir().join(scope.key_file());
    let bytes = std::fs::read(&path).ok()?;
    <[u8; KEY_LEN]>::try_from(&bytes[..]).ok()
}

/// this has ****（`sha256()` before 16 ）； not in 
/// or not can → `None`。
///
/// Why is not is ： need ** file**， is SQL、
/// will to / 。 if ， etc and 1 （file
/// from ）。 to 「 is not is 1 」， not ； before 
/// 16 （64 bit）「」， to 1 。
///
/// use is `db::backup`： when file， when and for ， not 1 
/// 「 this is use 1 」， not is use for empty。
pub fn key_fingerprint(scope: SecretScope) -> Option<String> {
    let key = existing_key(scope)?;
    Some(short_fingerprint(&key))
}

/// `sha256(bytes)` before 16 —— ， 1 。
fn short_fingerprint(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

/// ：any → before 16 。
///
/// need use to ** that `account.key.enc` **（ and 
/// [`key_fingerprint`] for is 1 ）， can 「 that is not is 
/// in this 」。 not this use will 1 sha256 —— 
/// 1 ， for will to error 「 1 / not 1 」。
pub fn fingerprint_of(bytes: &[u8]) -> String {
    short_fingerprint(bytes)
}

// ─── （ use ）───────────────────────────────────

/// ， down 1 `seal` / `open` new from （ or ）。
///
/// ── 1 use is down ────────────────────────────────────
/// down will ** `account.key`**（ that to ）。 `secret_key` in 
/// ， not down when use is ** old ** —— new 
/// use old 、 down 1 use new ， not 。 this is 
/// 1 ， in crypto this （ not is use has ）。
///
/// `Accounts` not will ， is 1 file，** not **： use 
/// this that ，「 = file」 。
pub fn reset_cache(scope: SecretScope) {
    if let Ok(mut guard) = cache_for(scope).lock() {
        *guard = None;
    }
    if matches!(scope, SecretScope::Accounts) {
        crate::server::core::providers::catalog::invalidate();
    }
}

/// this file **path**（ not 、 not ）。 / 。
pub fn key_file_path(scope: SecretScope) -> std::path::PathBuf {
    crate::paths::config_dir().join(scope.key_file())
}

/// this 32 （ not new ， [`existing_key`]）。
///
/// up when use `account.key` up ； no has 
/// （ from ） when return `None`， that not need need file。
pub fn existing_key_bytes(scope: SecretScope) -> Option<[u8; KEY_LEN]> {
    existing_key(scope)
}

pub(crate) fn backup_key(scope: SecretScope) -> Result<Option<[u8; KEY_LEN]>, String> {
    let guard = cache_for(scope).lock().map_err(|_| "密钥锁不可用".to_string())?;
    if let Some(key) = *guard {
        return Ok(Some(key));
    }
    drop(guard);
    if let Some(key) = env_key(scope) {
        return Ok(Some(key));
    }
    let path = key_file_path(scope);
    match std::fs::read(&path) {
        Ok(bytes) => <[u8; KEY_LEN]>::try_from(bytes.as_slice())
            .map(Some)
            .map_err(|_| format!("密钥文件 {} 长度异常", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("读取密钥文件 {} 失败: {error}", path.display())),
    }
}

// ─── / （ for ）─────────────────────────────────────

/// → `enc1:<base64(nonce||+tag)>`。
///
/// nonce （GCM nonce use is ： 1 down use 1 nonce
/// will ），； not use —— after no has ，
/// nonce is this 。
pub fn seal(plaintext: &[u8], scope: SecretScope) -> Result<String, String> {
    let key = secret_key(scope)?;
    seal_with_key(plaintext, &key)
}

pub(crate) fn seal_with_key(plaintext: &[u8], key: &[u8; KEY_LEN]) -> Result<String, String> {
    use aes_gcm::aead::{consts::U12, Aead as _};
    use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};

    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| "密钥长度不受 AES-256 支持".to_string())?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    getrandom::getrandom(&mut nonce_bytes).map_err(|error| format!("系统随机源不可用: {error}"))?;
    let nonce =
        Nonce::<U12>::try_from(nonce_bytes.as_slice()).map_err(|_| "生成 nonce 失败".to_string())?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| "AES-GCM 加密失败".to_string())?;
    let mut blob = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    blob.extend_from_slice(&nonce_bytes);
    blob.extend_from_slice(&ciphertext);
    Ok(format!("{ENVELOPE_PREFIX}{}", BASE64.encode(&blob)))
}

/// 1 `enc1:` → （ already is UTF-8，module header）。
///
/// failureReason is 「 can 」 ： not for / base64 / not / not 。
/// not that will **file** —— use need this file，
/// this will `status_json` to up 。
pub fn open(envelope: &str, scope: SecretScope) -> Result<Vec<u8>, String> {
    decrypt_envelope(envelope, || secret_key(scope), &format!(
        "AES-GCM 解密失败（密钥文件 {} 是否被删除或更换？{}）",
        scope.key_file(), scope.recovery_hint(),
    ))
}

pub(crate) fn open_with_key(envelope: &str, key: &[u8; KEY_LEN]) -> Result<Vec<u8>, String> {
    decrypt_envelope(envelope, || Ok(*key), "AES-GCM 解密失败（密文与备份密钥不匹配）")
}

fn decrypt_envelope(
    envelope: &str,
    resolve: impl FnOnce() -> Result<[u8; KEY_LEN], String>,
    failure: &str,
) -> Result<Vec<u8>, String> {
    use aes_gcm::aead::{consts::U12, Aead as _};
    use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};

    let Some(encoded) = envelope.trim().strip_prefix(ENVELOPE_PREFIX) else {
        return Err("密文格式不受支持（不是 enc1 信封）".to_string());
    };
    let blob = BASE64
        .decode(encoded)
        .map_err(|_| "密文不是合法 base64".to_string())?;
    // down ：nonce + GCM tag（16）。aes-gcm tag in ，
    // this to decrypt（Postfix ， and autoclaw/crypto.rs 1 ）
    if blob.len() <= NONCE_LEN + TAG_LEN {
        return Err("密文长度异常".to_string());
    }
    // in after ： not this 「file」 this use 
    let key = resolve()?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|_| "密钥长度不受 AES-256 支持".to_string())?;
    let nonce = Nonce::<U12>::try_from(&blob[..NONCE_LEN])
        .map_err(|_| "密文 nonce 长度异常".to_string())?;
    let plaintext = cipher.decrypt(&nonce, &blob[NONCE_LEN..])
        .map_err(|_| failure.to_string())?;
    // UTF-8 ： old `token.rs::decrypt` this will ，
    // for as ， not can as return（module header）
    std::str::from_utf8(&plaintext).map_err(|_| "解密结果不是合法 UTF-8 文本".to_string())?;
    Ok(plaintext)
}
