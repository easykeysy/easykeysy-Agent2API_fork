//! ALTCHA （proof-of-work， https://altcha.org； use 
//! OmniProxy： / before must 1 ，server + ）。
//!
//! ── ────────────────────────────────────────────────
//! `POST /api/panel/login` and `/api/panel/setup` is ， can 
//! none：、。failure（access::ATTEMPTS） is 「 1 
//! failure」， IP 。ALTCHA is 1 ：** requestmust
//! 1 **—— to 、 after 1 ；
//! 1 need 1 ， from request
//! 。 for none： in when 。
//!
//! ── （SHA-256 ）─────────────────────────────────────
//! challenge（JSON）：
//! `algorithm = "SHA-256"`、`salt`（ hex）、
//! `challenge = sha256(salt + number)` hex（number not down ，client）、
//! `maxnumber`（limit）、
//! `signature = HMAC-SHA256(secret, challenge)` —— /。
//! client payload（base64 JSON）：`{algorithm, challenge, number,
//! salt, signature}`。
//! = 1 + `sha256(salt+number) == challenge` + ** in **
//! （ this valid， ）+ + ** use after **
//! （ 1 payload not can ，）。
//!
//! ── and ──────────────────────────────────────────────
//! secret use when （32 hex）（`kv` `panelAltchaSecret`，
//! `RESERVED_KV_KEYS`——**config not can **）， after ； HMAC
//! use ， from not 。（challenge → when ） in ：empty，
//! already down use challenge 1 by 10 minutes ——
//! to to has seconds。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::server::access;
use crate::server::config;

const KV_SECRET_KEY: &str = "panelAltchaSecret";
/// challenge valid： to limit（OmniProxy 10 minutes）
const CHALLENGE_TTL_MS: i64 = 10 * 60 * 1000;
/// limit： 5–10 SHA-256 in ms to 1 seconds，
/// for 「request need 1 」 already is ；
const MAXNUMBER_MIN: u32 = 50_000;
const MAXNUMBER_MAX: u32 = 100_000;
/// limit： this （ ~）
const STORE_CAP: usize = 4096;

/// challenge → when （ms）。 is challenge hex —— 
/// 「 」，client not in ，。
static STORE: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, i64>> {
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn hmac_hex(secret: &str, message: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .expect("HMAC 接受任意长度的密钥");
    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

/// 1 `[min, max]` u32（getrandom ）
fn random_range(min: u32, max: u32) -> u32 {
    let span = max - min + 1;
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).expect("系统随机源不可用");
    min + (u32::from_le_bytes(buf) % span)
}

/// HMAC ： no has and （ etc ：unavailable when as 
/// —— no has ，none；headless has ）。
fn secret() -> String {
    let existing = access::kv_get(KV_SECRET_KEY);
    if let Some(value) = existing {
        return value;
    }
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("系统随机源不可用");
    let generated: String = buf.iter().map(|b| format!("{b:02x}")).collect();
    access::kv_put(KV_SECRET_KEY, &generated);
    generated
}

/// is enabled（set，default）。`local_panel` 1 
/// （ true、headless false， `config::captcha_enabled`）。
pub fn enabled(local_panel: bool) -> bool {
    config::current().captcha_enabled(local_panel)
}

/// 1 challenge（ `GET /api/panel/captcha` use ）。
/// when return `None`（ 400， before ）。
pub fn challenge(local_panel: bool) -> Option<Value> {
    if !enabled(local_panel) {
        return None;
    }
    let salt = access::random_hex(12);
    let maxnumber = random_range(MAXNUMBER_MIN, MAXNUMBER_MAX);
    let number = random_range(0, maxnumber);
    let challenge_hex = sha256_hex(&format!("{salt}{number}"));
    let signature = hmac_hex(&secret(), &challenge_hex);

    {
        let mut table = match store().lock() {
            Ok(table) => table,
            Err(poisoned) => poisoned.into_inner(),
        };
        let now = now_ms();
        // + when （ is ）
        table.retain(|_, expires| *expires > now);
        if table.len() >= STORE_CAP {
            table.retain(|_, _| false);
        }
        table.insert(challenge_hex.clone(), now + CHALLENGE_TTL_MS);
    }

    Some(json!({
        "algorithm": "SHA-256",
        "challenge": challenge_hex,
        "maxnumber": maxnumber,
        "salt": salt,
        "signature": signature,
    }))
}

/// / request ALTCHA payload。
/// failurereturn to use Reason（ 400）。
pub fn verify(token: &str) -> Result<(), String> {
    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        token.trim(),
    )
    .or_else(|_| {
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, token.trim())
    })
    .map_err(|_| "机器人校验无效，请刷新页面重试".to_string())?;
    let parsed: Value = serde_json::from_slice(&decoded)
        .map_err(|_| "机器人校验无效，请刷新页面重试".to_string())?;

    let get = |field: &str| parsed.get(field).and_then(Value::as_str).unwrap_or("");
    let algorithm = get("algorithm");
    let challenge_field = get("challenge");
    let salt = get("salt");
    let signature = get("signature");
    let number = parsed.get("number").and_then(Value::as_u64);
    if algorithm != "SHA-256"
        || challenge_field.is_empty()
        || salt.is_empty()
        || number.is_none()
    {
        return Err("机器人校验无效，请刷新页面重试".to_string());
    }
    let number = number.unwrap();

    // （ challenge），
    if !constant_time_eq(
        signature.as_bytes(),
        hmac_hex(&secret(), challenge_field).as_bytes(),
    ) {
        return Err("机器人校验无效，请刷新页面重试".to_string());
    }
    if !constant_time_eq(
        challenge_field.as_bytes(),
        sha256_hex(&format!("{salt}{number}")).as_bytes(),
    ) {
        return Err("机器人校验无效，请刷新页面重试".to_string());
    }

    // ： in （）+ + use after （）
    let mut table = match store().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    match table.remove(challenge_field) {
        Some(expires) if expires > now_ms() => Ok(()),
        _ => Err("机器人校验已过期，请刷新页面重试".to_string()),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
