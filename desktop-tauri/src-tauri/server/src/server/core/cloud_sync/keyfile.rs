//! use 「」file（`account.key`）， can up 。
//!
//! ```text
//! seal_key(key_bytes, password) → account.key.enc（JSON ）
//! open_key(envelope, password)   → key_bytes
//! ```
//!
//! ── Why need need this 1 （ not is account.key）───────────────
//! `account.key` is and Key for 。 up ， etc 
//! 「 and in 1 」—— to that / directory can 。
//! up is ** use after **file； in use 
//! （ is use 1 ， `settings` module header）。
//!
//! ── Why is PBKDF2-HMAC-SHA256 not is sha256() ────────
//! 。 etc 「 1 」 to 
//! 「 1 sha256」；PBKDF2 use to ms，
//! salt， for use 。
//! 210000（OWASP 2023 for PBKDF2-HMAC-SHA256 ）。
//! this （`iterations` ）， will when ** old can ** ——
//! this is 「parameter」 and 「 to after 」 can ， not need in after not 。
//!
//! ── not add ──────────────────────────────────────────────
//! `hmac` / `sha2` / `aes-gcm` / `getrandom` / `base64` already in 
//! `server/Cargo.toml` （ as ALTCHA、update、、PKCE and 
//! ）， this is PBKDF2 30 。
//!
//! ── ────────────────────────────────────────────────
//! ```json
//! {
//!   "format": "agent2api-cloud-sync-key",
//!   "version": 1,
//!   "kdf": "pbkdf2-hmac-sha256",
//!   "iterations": 210000,
//!   "salt": "<base64 32B>",
//!   "nonce": "<base64 12B>",
//!   "ciphertext": "<base64>",
//! "keyFingerprint": "<sha256()[:16]>"
//! }
//! ```
//! `keyFingerprint` is **** （ and `crypto::key_fingerprint` ），
//! use in after 「 is manifest that 」——
//! and manifest not when （ up in ） can ， not is etc to use 
//! after not 。

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::server::core::crypto;

pub(crate) const ENVELOPE_FORMAT: &str = "agent2api-cloud-sync-key";
pub(crate) const ENVELOPE_VERSION: u32 = 1;
/// default（； after old by ）
pub(crate) const DEFAULT_ITERATIONS: u32 = 210_000;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
/// （AES-256）
pub(crate) const KEY_LEN: usize = 32;

/// （ / up JSON ）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyEnvelope {
    pub format: String,
    pub version: u32,
    pub kdf: String,
    pub iterations: u32,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
    /// （module header）
    pub key_fingerprint: String,
}

/// PBKDF2-HMAC-SHA256 32 。
///
/// not is `pbkdf2` crate： has this 1 use ，30 1 
/// （ and 「 not add」 1 ， `server/Cargo.toml`）。
fn derive_key(password: &str, salt: &[u8], iterations: u32) -> [u8; KEY_LEN] {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;

    // PBKDF2 1 （dkLen = 32 = 1 SHA-256 ， need 1 ）
    let mut mac = HmacSha256::new_from_slice(password.as_bytes())
        .expect("HMAC 接受任意长度的密钥");
    mac.update(salt);
    mac.update(&1u32.to_be_bytes());
    let mut block = mac.finalize().into_bytes();
    let mut result = [0u8; KEY_LEN];
    result.copy_from_slice(&block);

    for _ in 1..iterations {
        let mut mac = HmacSha256::new_from_slice(password.as_bytes())
            .expect("HMAC 接受任意长度的密钥");
        mac.update(&block);
        block = mac.finalize().into_bytes();
        for (slot, byte) in result.iter_mut().zip(block.iter()) {
            *slot ^= *byte;
        }
    }
    result
}

fn random_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("系统随机源不可用: {error}"))?;
    Ok(bytes)
}

/// use 。
///
/// `key_bytes` must is 32 （`account.key` ）； its by error up ——
/// / will up that and not is 1 ， after not 。
pub(crate) fn seal_key(key_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if key_bytes.len() != KEY_LEN {
        return Err(format!(
            "账号密钥长度异常（应为 {KEY_LEN} 字节，实际 {} 字节）",
            key_bytes.len()
        ));
    }
    if password.is_empty() {
        return Err("同步密码不能为空".to_string());
    }
    use aes_gcm::aead::{consts::U12, Aead as _};
    use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};

    let salt: [u8; SALT_LEN] = random_bytes()?;
    let nonce_bytes: [u8; NONCE_LEN] = random_bytes()?;
    let derived = derive_key(password, &salt, DEFAULT_ITERATIONS);
    let cipher = Aes256Gcm::new_from_slice(&derived)
        .map_err(|_| "派生密钥长度不受 AES-256 支持".to_string())?;
    let nonce = Nonce::<U12>::from(nonce_bytes);
    let ciphertext = cipher
        .encrypt(&nonce, key_bytes)
        .map_err(|_| "加密账号密钥失败".to_string())?;

    let envelope = KeyEnvelope {
        format: ENVELOPE_FORMAT.to_string(),
        version: ENVELOPE_VERSION,
        kdf: "pbkdf2-hmac-sha256".to_string(),
        iterations: DEFAULT_ITERATIONS,
        salt: BASE64.encode(salt),
        nonce: BASE64.encode(nonce_bytes),
        ciphertext: BASE64.encode(ciphertext),
        key_fingerprint: crypto::fingerprint_of(key_bytes),
    };
    serde_json::to_vec_pretty(&envelope).map_err(|error| format!("序列化密钥信封失败: {error}"))
}

/// use 。
///
/// error → GCM failure → 「 not 」，** not **return。
pub(crate) fn open_key(raw: &[u8], password: &str) -> Result<[u8; KEY_LEN], String> {
    use aes_gcm::aead::{consts::U12, Aead as _};
    use aes_gcm::{Aes256Gcm, KeyInit as _, Nonce};

    let envelope: KeyEnvelope = serde_json::from_slice(raw)
        .map_err(|_| "远端密钥文件的格式不认识（不是本功能生成的 account.key.enc）".to_string())?;
    if envelope.format != ENVELOPE_FORMAT {
        return Err(format!("远端密钥文件格式不兼容（format = {}）", envelope.format));
    }
    if envelope.version != ENVELOPE_VERSION {
        return Err(format!(
            "远端密钥文件版本不兼容（v{}，本机支持 v{ENVELOPE_VERSION}）",
            envelope.version
        ));
    }
    if envelope.kdf != "pbkdf2-hmac-sha256" {
        return Err(format!("不支持的密钥派生算法: {}", envelope.kdf));
    }
    if envelope.iterations == 0 || envelope.iterations > 10_000_000 {
        return Err("远端密钥文件的迭代次数异常".to_string());
    }
    let salt = BASE64
        .decode(&envelope.salt)
        .map_err(|_| "远端密钥文件的 salt 不是合法 base64".to_string())?;
    let nonce_bytes = BASE64
        .decode(&envelope.nonce)
        .map_err(|_| "远端密钥文件的 nonce 不是合法 base64".to_string())?;
    if nonce_bytes.len() != NONCE_LEN {
        return Err("远端密钥文件的 nonce 长度异常".to_string());
    }
    let ciphertext = BASE64
        .decode(&envelope.ciphertext)
        .map_err(|_| "远端密钥文件的密文不是合法 base64".to_string())?;

    let derived = derive_key(password, &salt, envelope.iterations);
    let cipher = Aes256Gcm::new_from_slice(&derived)
        .map_err(|_| "派生密钥长度不受 AES-256 支持".to_string())?;
    let nonce = Nonce::<U12>::try_from(nonce_bytes.as_slice())
        .map_err(|_| "远端密钥文件的 nonce 长度异常".to_string())?;
    let plaintext = cipher.decrypt(&nonce, ciphertext.as_slice()).map_err(|_| {
        "同步密码不正确（无法解开远端保存的账号密钥）".to_string()
    })?;
    let key: [u8; KEY_LEN] = plaintext
        .as_slice()
        .try_into()
        .map_err(|_| "解密出的账号密钥长度异常".to_string())?;

    // ： for and manifest not when （ up in ），
    // this can ， not is etc use after not 。
    let actual = crypto::fingerprint_of(&key);
    if actual != envelope.key_fingerprint {
        return Err(
            "远端密钥文件自校验失败（指纹不一致），文件可能已损坏，请重新上传".to_string(),
        );
    }
    Ok(key)
}

/// to ： `account.key`， old file。
///
/// ── Why need old file ────────────────────────────────────
/// down can can is 「 use old 」， can can is 「」。 old 1 
/// and no has ， has not 。
/// `account.key.bak-< when >` not is delete： when use can 。
///
/// ── Whymust crypto ──────────────────────────────
/// `crypto::reset_cache` ： not ，remaining when use is old 。
pub(crate) fn install_key(key_bytes: &[u8]) -> Result<Option<String>, String> {
    if key_bytes.len() != KEY_LEN {
        return Err("账号密钥长度异常".to_string());
    }
    let path = crypto::key_file_path(crypto::SecretScope::Accounts);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| format!("创建配置目录失败: {error}"))?;
    }
    let backup_name = if path.exists() {
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let backup = path.with_file_name(format!("account.key.bak-{stamp}"));
        std::fs::rename(&path, &backup).map_err(|error| {
            format!(
                "备份原账号密钥失败（{}）：{error}",
                path.display()
            )
        })?;
        Some(
            backup
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
        )
    } else {
        None
    };
    std::fs::write(&path, key_bytes)
        .map_err(|error| format!("写入账号密钥失败（{}）：{error}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    crypto::reset_cache(crypto::SecretScope::Accounts);
    Ok(backup_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_and_open_round_trip() {
        let key = [7u8; KEY_LEN];
        let sealed = seal_key(&key, "correct horse battery").expect("seal");
        let opened = open_key(&sealed, "correct horse battery").expect("open");
        assert_eq!(opened, key);
    }

    #[test]
    fn open_rejects_wrong_password() {
        let key = [9u8; KEY_LEN];
        let sealed = seal_key(&key, "right-password").expect("seal");
        let error = open_key(&sealed, "wrong-password").expect_err("should fail");
        assert!(error.contains("同步密码不正确"), "unexpected: {error}");
    }

    #[test]
    fn envelope_records_iterations_for_forward_compat() {
        let sealed = seal_key(&[1u8; KEY_LEN], "password123").expect("seal");
        let envelope: KeyEnvelope = serde_json::from_slice(&sealed).expect("parse");
        assert_eq!(envelope.iterations, DEFAULT_ITERATIONS);
        assert_eq!(envelope.kdf, "pbkdf2-hmac-sha256");
        assert_eq!(envelope.key_fingerprint.len(), 16);
    }

    #[test]
    fn seal_rejects_wrong_key_length() {
        assert!(seal_key(&[0u8; 16], "password123").is_err());
    }
}
