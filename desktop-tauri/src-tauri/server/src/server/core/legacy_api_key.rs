//! old `apiKey`（1.x Key ） ****。
//!
//! ── Why 1 file（ not is `core::api_keys`）────────────────
//! this has ， （ not 、 etc 、failure not 
//! ） and `api_keys.rs` （ ） is 1 for ， need for 
//! ； `core::api_keys` already file（），
//! 1 「」 will —— and its ， not 「 old 」
//! 1 can grep to 。（、、 not ）
//! `core::api_keys` `decode_key` / `from_value`： that has 。
//!
//! ── is 1 ──────────────────────────────────────────
//! `apiKeys` `key` from is `enc1:` ， before 
//! by `api_keys::encrypt_legacy_keys` 。** has 1 old Key**：
//! `apiKey`（ use from Key when down ， `api_keys` module header）。
//! before 1 is ， in 1 `kv` —— 「file」 ，
//! need ， is 。
//!
//! ── Why is ****， not is `apiKeys` ─────────────
//! is 1 ****：`apiKeys` 1 ，`entries_from` and that 
//! ，「 1 `apiKeys` and `apiKey`」 this has will none
//! before —— use no ，config。 after not is 「」：
//! 「`apiKey` in ⇒ use no Key 」 this 1 ，config
//! when will to error to 。、 and not ， old ，
//! up not （ 1 not ， `config::build` ）。
//!
//! ── etc ：， not ─────────────────────────
//! has ， and in up （ `pending_plaintext`）： is empty、
//! and not is `enc1:` 。 is can ， no has —— 1 
//! in 1 ，** 1 not **（config not ）。 not kv 
//! （ and `encrypt_legacy_keys` / `encrypt_legacy_accounts` 1 ）： will 1 
//! 「 this no has 」 ， can can and not 1 （ in 、 is ——
//! up 1 successfailure when is this ）， no has this not 1 。
//!
//! **「 already is 」including not **（file / ）： that not can 
//! `seal` 1 —— will 「」， use after file， 
//! is in that ， Key not （ 1 、 not can ）。
//! for `api_keys::decode_key`：、 not empty、 not 。
//!
//! ── failure not ─────────────────────────────────────────
//! unavailable（`config_dir` not can etc ） or unavailable when 1 and return 0：
//! ， this Key available， down —— and 
//! `encrypt_legacy_keys` 1 、 1 by （path up not this as 1 failure
//! not ）。

use serde_json::{Map, Value};

use crate::server::config;
use crate::server::core::api_keys::KEY_API_KEY;
use crate::server::core::crypto::{self, SecretScope};
use crate::server::logging;

/// config `apiKey` **** `enc1:` （ etc ， can use ）。
///
/// return：`1` = already and ，`0` = not need need or failure（failure already in 
/// ， use not ）。 use is `account_bootstrap::run`（
/// `api_keys::encrypt_legacy_keys`）—— must etc **config**：
/// `config::current().raw()`，path `config::init` 、path use 
/// `config::reload()`（ `api::upgrade_api`），。
///
/// and `encrypt_legacy_accounts` / `encrypt_legacy_keys` has 1 ： after 
/// is 「 / 」， is ** 1 **，
/// `config::update_raw_field` this has 「 」（ ）。
pub fn encrypt_legacy_api_key() -> usize {
    // and ** 1 **（ and `encrypt_legacy_keys` 1 ）：
    // will in 「 has config」 when to in ——
    // this 、 when already 。 is ， not as 1 raw
    // 。
    let raw = config::current().raw().clone();
    let Some(pending) = pending_plaintext(&raw) else {
        // no / empty / already is ： this path， 1 not 
        return 0;
    };
    let envelope = match crypto::seal(pending.as_bytes(), SecretScope::Accounts) {
        Ok(envelope) => envelope,
        Err(error) => {
            logging::log(
                "[Keys]",
                &format!(
                    "⚠️  旧字段 apiKey 加密未完成（下次启动会自动重试，本次 Key 照常可用）: {error}"
                ),
            );
            return 0;
        }
    };
    // `config::update_raw_field`（config 「 and 」 has ）：
    // 1 —— 、 raw 、
    // `RuntimeConfig::api_key`（ that ）。 1 will down 
    // ： already is is （ down 1 ）、 or raw already 
    // `api_key` is old （）。「 config 」
    // has this 1 ， will config not will this 。
    if !config::update_raw_field(KEY_API_KEY, Value::String(envelope)) {
        // failure when `update_raw_field` already 1 `[Config]` failure；
        // this 1 `[Keys]` ： use need is 「 this Key in 、
        // this 」， that error「failure」。
        logging::log(
            "[Keys]",
            "⚠️  旧字段 apiKey 已加密但未落盘（库不可用；下次启动会自动重试）",
        );
        return 0;
    }
    logging::log(
        "[Keys]",
        "🔒 已加密 1 把网关 Key（旧字段 apiKey；字段名与形态保持不变）",
    );
    1
}

/// need need （`None` = not need need ， 1 not ）。
///
/// in up （module header etc ）：
/// - **empty**： / empty / has empty not 1 Key —— and 
/// `api_keys::entries_from` 1 （ that in before empty）。
/// empty not can ： after empty， will 1 Key， is 
/// empty —— in 「empty token request」， etc （ this 
/// in `ApiKeyEntry::from_value` 「 not empty」 that 1 ）。
/// - ** not is `enc1:` **： 1 is `crypto::is_envelope`（ not in this 
/// `starts_with`， will not will 1 ）。 already is （ not 
/// that ） not —— this is etc 。
fn pending_plaintext(raw: &Map<String, Value>) -> Option<String> {
    let text = raw.get(KEY_API_KEY)?.as_str()?.trim();
    if text.is_empty() || crypto::is_envelope(text) {
        return None;
    }
    Some(text.to_string())
}
