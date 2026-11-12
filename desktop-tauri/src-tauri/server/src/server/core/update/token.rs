//! 「updateset」 GitHub ：、 and 。
//!
//! ── already to `core::crypto` ───────────────────────────
//! ** not **：、`enc1:` 、file //
//! to `crate::server::core::crypto`（ need use 1 ， by 
//! module header）。 this ：
//! 1. `SecretScope::UpdateToken`（file `update-token.key`）；
//! 2. ： then 、 and `GITHUB_TOKEN` 、 not when 
//! ， and `status_json` to use that 。
//!
//! ── Why before need ─────────────────────────────────────
//! can GitHub API （ 60 / when → 5000 / when ）， is ** has 
//! **： after can 、 CI。config（`agent2api.db`） will 
//! / / ， in etc 「 1 file」。
//! this is AES-256-GCM （ and then `core::crypto`）：
//! · ：32 ，**** `{config_dir}/update-token.key`（
//! when ；Unix up 0600）。 when no has file， not 。
//! · ：`enc1:<base64(nonce(12) || +tag)>`， as config `githubToken`
//! （ config KEY_GITHUB_TOKEN）。`enc1` is before ，
//! will （e.g.） when can old 。
//!
//! ── ， not ─────────────────────────────
//! 1. `effective_token`：`github_headers` Authorization when use ；
//! 2. `stored_token`： to 「 this has no has 」 use （ not 
//! none and already ， will ）；
//! 3. and path： after ， not ；
//! 4. API **「 has no has 、」**（`status_json`）， not ——
//! 「 already 」 is can to （ = ）。
//!
//! ── ： > ──────────────────────────
//! `GITHUB_TOKEN` / old `WORKBUDDY_GITHUB_TOKEN`（ is 1 ，
//! Docker ） as 。 and config has 1 （「config >
//! 」）： in up is use ** after 1 **，。
//!
//! ── failure ────────────────────────────────────────
//! file / / no when ：** not 「 no 」**——config has 
//! 1 ， by will use as in 。`effective_token` 
//! and 1 verbose；`status_json` Reason， up new 
//! 1 （ new + new ，）。「 1 can 」 this by 
//! `core::crypto` by （ not not ）， not 。

use crate::server::config;
use crate::server::core::crypto::{self, SecretScope};
use crate::server::logging;

/// use （GitHub use ；file is `update-token.key`）
const SCOPE: SecretScope = SecretScope::UpdateToken;

/// limit：GitHub 255 ，
const MAX_TOKEN_LEN: usize = 255;

// ─── and ──────────────────────────────────────────

/// 1 `enc1:` → UTF-8 。
///
/// is 「 + String」 ： 、、
/// AES-GCM and UTF-8 in `core::crypto::open`。`open` already is 
/// UTF-8（ its module header）， this not will —— use `from_utf8_lossy`
/// is as `String`， not errorpath（ and old etc error by 
/// `open` to ）。
fn decode_envelope(envelope: &str) -> Result<String, String> {
    let bytes = crypto::open(envelope, SCOPE)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// （Docker / config， 、empty no ）
fn env_token() -> Option<String> {
    ["GITHUB_TOKEN", "WORKBUDDY_GITHUB_TOKEN"]
        .iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

/// that 1 （ after ）。 / not → None。
///
/// and `effective_token` ： this ** that **（ not ），
/// 「 this has no has 」 use （api::update need not need 
/// check —— 1 not this 1 ）。
pub fn stored_token() -> Option<String> {
    let envelope = config::github_token_envelope()?;
    decode_envelope(&envelope).ok()
}

/// ： （）> 。
///
/// `github_headers` checkupdate：failure and in this 
/// 1 ， use to `Option<String>` 。
pub fn effective_token() -> Option<String> {
    if let Some(envelope) = config::github_token_envelope() {
        match decode_envelope(&envelope) {
            Ok(token) => return Some(token),
            Err(reason) => logging::verbose(
                "[Update]",
                &format!("已保存的 GitHub 令牌解密失败（{reason}），本次跳过使用"),
            ),
        }
    }
    env_token()
}

/// `set_token` 。
///
/// `saved`： is success（false when already ， and config setter 1 ）。
/// `changed`： is ** /** —— use （api::update） has to 
/// true 「checkupdate」 failure，「 1 」 、
/// 。
#[derive(Clone, Copy, Debug)]
pub struct TokenUpdate {
    pub saved: bool,
    pub changed: bool,
}

/// / （`None` = ）。
///
/// ：empty、 not 、 in 「 down 」——
/// GitHub in this ， need HTTP ，empty or 
/// is request，must in 。
pub fn set_token(token: Option<&str>) -> Result<TokenUpdate, String> {
    let previous = stored_token();
    let Some(raw) = token else {
        // ： in （ not ） 1 
        let changed = previous.is_some() || config::github_token_envelope().is_some();
        return Ok(TokenUpdate {
            saved: config::set_github_token_envelope(None),
            changed,
        });
    };
    let token = raw.trim();
    if token.is_empty() {
        return Err("令牌不能为空（要清除请传 null）".to_string());
    }
    if token.len() > MAX_TOKEN_LEN {
        return Err(format!("令牌过长（最多 {MAX_TOKEN_LEN} 字符）"));
    }
    if !token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
        return Err("令牌只能包含字母、数字、下划线与连字符".to_string());
    }
    // and already （ not when that 1 「」， will as ）
    let changed = previous.as_deref() != Some(token);
    let envelope = crypto::seal(token.as_bytes(), SCOPE)?;
    Ok(TokenUpdate {
        saved: config::set_github_token_envelope(Some(envelope)),
        changed,
    })
}

/// → API JSON （** not **，module header「」）。
///
/// `origin`：`stored`（ ）/ `env`（）/ null（ no ）。
/// already not when `error` Reason —— 「 new 1 」，
/// not is 「config」「 no config」。
pub fn status_json() -> serde_json::Value {
    if let Some(envelope) = config::github_token_envelope() {
        return match decode_envelope(&envelope) {
            Ok(_) => serde_json::json!({ "filled": true, "origin": "stored", "error": null }),
            Err(reason) => {
                let env = env_token().is_some();
                let origin = if env { Some("env") } else { None };
                serde_json::json!({
                    "filled": env,
                    "origin": origin,
                    "error": reason,
                })
            }
        };
    }
    if env_token().is_some() {
        serde_json::json!({ "filled": true, "origin": "env", "error": null })
    } else {
        serde_json::json!({ "filled": false, "origin": null, "error": null })
    }
}
