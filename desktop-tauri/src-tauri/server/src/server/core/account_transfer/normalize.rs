//! 1 （merge 「update」 and 「add」 use ）。
//!
//! ── ─────────────────────────────────────────────────
//! 1. **unknown to **：update when as 、file 
//! its up ；add when file 。 down 
//! and `rateLimits`（ when ） 1 not and ， not 。
//! 2. ** not can **：id / priority / addedAt / updatedAt /
//! desktop by use ， this not 。
//! 3. ** WorkBuddy**：edition / prefixPath / endpoint /
//! platform is WorkBuddy already ， in WorkBuddy by edition 1 ；
//! its 1 not （file ）—— not 
//! this ， will when to 。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store_util::{token_tail_of, truncate_text};
use crate::server::core::account_store::MAX_TOKEN_LENGTH;
use crate::server::core::endpoints::resolve_edition;

/// not （；module header）。
const LOCAL_OWNED_KEYS: [&str; 6] = [
    "id",
    "priority",
    "addedAt",
    "updatedAt",
    "rateLimits",
    "desktop",
];

/// limit（ and path 1 ）
const MAX_NAME_LENGTH: usize = 100;

/// → （JS `String(x)` ：/ by ，null/ to empty）
fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        _ => String::new(),
    }
}

/// `Number(x) > 0` （ already is JSON when ， `state::json_number`）
fn positive_number_of(value: Option<&Value>) -> Option<Value> {
    match value {
        Some(Value::Number(number)) => {
            let parsed = number.as_f64().unwrap_or(0.0);
            (parsed.is_finite() && parsed > 0.0).then(|| value.cloned().unwrap_or(Value::Null))
        }
        Some(Value::String(text)) => match text.trim().parse::<f64>() {
            Ok(parsed) if parsed.is_finite() && parsed > 0.0 => {
                Some(crate::server::core::account_store::state::json_number(parsed))
            }
            _ => None,
        },
        _ => None,
    }
}

/// this is and （ and when not and ）
fn importable_key(key: &str) -> bool {
    !key.starts_with('_') && key != "format" && key != "customProviders"
        && !LOCAL_OWNED_KEYS.iter().any(|owned| *owned == key)
}

/// 1 1 。
///
/// `provider` already by use by ；`before` as in （add None）。
/// return ** not ** id / priority / addedAt / updatedAt / rateLimits ——
/// that by use by 。 when return Err（ this failure， not ）。
pub(super) fn normalize_imported(
    item: &Map<String, Value>,
    provider: &str,
    before: Option<&StoredAccount>,
) -> Result<Map<String, Value>, String> {
    let mut record = Map::new();

    // update： as （、 when and ），
    // unknown
    if let Some(before) = before {
        for (key, value) in before.fields() {
            if key.starts_with('_') || key == "rateLimits" || !importable_key(key) {
                continue;
            }
            record.insert(key.clone(), value.clone());
        }
    }
    // file its up （null / empty as ， not ）
    for (key, value) in item {
        if !importable_key(key) || value.is_null() {
            continue;
        }
        if matches!(value, Value::String(text) if text.trim().is_empty()) {
            continue;
        }
        record.insert(key.clone(), value.clone());
    }

    // provider （ after WorkBuddy need by ； not ）
    record.insert("provider".to_string(), Value::String(provider.to_string()));

    // ── ：empty（ as empty when old ， empty）──
    let access_token = {
        let incoming = text_of(item.get("accessToken"));
        if incoming.is_empty() {
            before.map(StoredAccount::access_token).unwrap_or_default()
        } else {
            incoming
        }
    };
    let refresh_token = {
        let incoming = text_of(item.get("refreshToken"));
        if incoming.is_empty() {
            before.map(StoredAccount::refresh_token).unwrap_or_default()
        } else {
            incoming
        }
    };
    if !access_token.is_empty() {
        record.insert("accessToken".to_string(), Value::String(access_token.clone()));
    }
    if !refresh_token.is_empty() {
        record.insert("refreshToken".to_string(), Value::String(refresh_token));
    }
    let token_tail = {
        let incoming = text_of(item.get("tokenTail"));
        if !incoming.is_empty() {
            incoming
        } else if access_token.is_empty() {
            String::new()
        } else {
            token_tail_of(&access_token)
        }
    };
    if !token_tail.is_empty() {
        record.insert("tokenTail".to_string(), Value::String(token_tail));
    }

    // ── ：empty， and path 1 ──
    let name = {
        let incoming = truncate_text(&text_of(item.get("name")), MAX_NAME_LENGTH);
        if incoming.is_empty() {
            before.map(StoredAccount::name).unwrap_or_default()
        } else {
            incoming
        }
    };
    if !name.is_empty() {
        record.insert("name".to_string(), Value::String(name));
    }

    // ── enabled：（ as enabled， and path）──
    if let Some(value) = item.get("enabled") {
        record.insert(
            "enabled".to_string(),
            Value::Bool(!matches!(value, Value::Bool(false))),
        );
    } else if before.is_none() {
        record.insert("enabled".to_string(), Value::Bool(true));
    }

    // ── ：； this failure， not ──
    if item.contains_key("proxy") {
        let normalized = crate::server::core::proxies::normalize_account_proxy(
            item.get("proxy").unwrap_or(&Value::Null),
        )
        .map_err(|error| error.message)?
        .unwrap_or(Value::Null);
        record.insert("proxy".to_string(), normalized);
    } else if before.is_none() {
        record.insert("proxy".to_string(), Value::Null);
    }

    // is 「workbuddy 」 not is id： and use 1 schema
    // （uid + edition/endpoint/prefixPath/platform）， id will 
    // else that **** —— after default， 401。
    if crate::server::core::providers::workbuddy::is_workbuddy_family(&provider) {
        normalize_workbuddy_known_fields(&mut record, item, before, provider);
    } else {
        // WorkBuddy：edition / prefixPath / endpoint / platform is 
        // ， not its schema， 1 not 。
        //
        // has 「 is not is WorkBuddy 」 1 ，** not ** —— add 1 
        // provider when not need need 1 （ old not in this ， in 
        // `identity.rs` ， that already use ）。
        strip_foreign_endpoint_keys(&mut record, &provider);
    }

    // ：apiKey / baseUrl / tokenTail is schema，
    // use not （、 URL）， 1 
    if provider.starts_with(crate::server::core::custom_providers::ID_PREFIX) {
        normalize_custom_known_fields(&mut record, item, before)?;
    }

    // provider already in up ： this not 
    Ok(record)
}

/// ** ** from （add and updatepath use ）。
///
/// 1 old file（ ） will this 
/// ； not ， will when to 。
///
/// provider is WorkBuddy ，** not **：add 1 provider
/// when not need need 1 —— this is old 。
fn strip_foreign_endpoint_keys(record: &mut Map<String, Value>, provider: &str) {
    if crate::server::core::providers::workbuddy::is_workbuddy_family(provider) {
        return;
    }
    for key in ["edition", "prefixPath", "endpoint", "platform"] {
        record.remove(key);
    }
}

/// ：apiKey（）、noAuth（none need ）、
/// baseUrl（）、tokenTail（）。
///
/// is `custom_accounts::add_custom_account` ， use 1 
/// then ：apiKey trim after （ this failure）、 as empty when old 
/// （ and accessToken / refreshToken and 1 ）；baseUrl ——
/// empty（default），empty `normalize_base_url` ；
/// tokenTail ，otherwise by apiKey new （must and ，
/// use old new key will for not up ）。
///
/// ── noAuth and apiKey in this need ─────────────────────
/// and （`add_custom_account` / `update_custom_credentials`） 1 not ：
/// 「none need 」 not this when 1 key。** noAuth apiKey** ——
/// will file 1 old key 1 none「 has 」，
/// will is 1 。
fn normalize_custom_known_fields(
    record: &mut Map<String, Value>,
    item: &Map<String, Value>,
    before: Option<&StoredAccount>,
) -> Result<(), String> {
    // file to by file（true / false is ）， no to then use 
    // （record already from ， normalize_imported ）
    let no_auth = match item.get("noAuth") {
        Some(Value::Bool(flag)) => *flag,
        _ => matches!(record.get("noAuth"), Some(Value::Bool(true))),
    };
    if no_auth {
        record.insert("noAuth".to_string(), Value::Bool(true));
    } else {
        record.remove("noAuth");
    }
    let api_key = if no_auth {
        String::new()
    } else {
        let incoming = text_of(item.get("apiKey"));
        if incoming.is_empty() {
            before
                .and_then(|record| record.get("apiKey"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        } else {
            incoming
        }
    };
    if api_key.chars().count() > MAX_TOKEN_LENGTH {
        return Err("apiKey 过长".to_string());
    }
    record.insert("apiKey".to_string(), Value::String(api_key.clone()));

    let token_tail = if no_auth {
        // none need no has can （ and `add_custom_account` 1 ）
        String::new()
    } else {
        let incoming = text_of(item.get("tokenTail"));
        if !incoming.is_empty() {
            incoming
        } else if api_key.is_empty() {
            String::new()
        } else {
            token_tail_of(&api_key)
        }
    };
    if token_tail.is_empty() {
        record.remove("tokenTail");
    } else {
        record.insert("tokenTail".to_string(), Value::String(token_tail));
    }

    if item.contains_key("baseUrl") {
        let raw = item
            .get("baseUrl")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if raw.is_empty() {
            // as empty = ， baseUrl（ and path：
            // 「 no has 」 and 「empty」must can ，empty not is URL not ）
            record.remove("baseUrl");
        } else {
            let base_url = crate::server::core::custom_providers::normalize_base_url(&raw)?;
            record.insert("baseUrl".to_string(), Value::String(base_url));
        }
    }
    Ok(())
}

/// WorkBuddy already ：/ by edition 1 ， use old 。
///
/// WorkBuddy ** not will ** to this （module header 3 ）。
///
/// ── `apiBase` is `endpoint` （issue #381）──────────────────
/// when use up has file `auth.domain`（`www.workbuddy.ai`），
/// will `domain` / `apiHost` / `apiBase` this 。 its in has `apiBase`
/// （ ）， from no has —— is 「success、
/// none」 `endpoint` is empty， to ：
/// to ， 401 → → 1 
/// → after 「expired or 」 this to 。
///
/// this `endpoint`（）：
/// `apiBase`（available、）> `endpoint`（）
/// > `apiHost` > `domain`（file `auth.domain` ）。
///
/// `domain` in after is ** has **： in `X-Domain` ， not is 。
/// use to when ， by 「 that 」 ——
/// issue #381 6 in 5 failure， 1 is 「success、」。
///
/// no when `https://`： `normalize_endpoint` to no has scheme 
/// will `www.workbuddy.ai/v2/...`，reqwest as for failure。
fn endpoint_alias_of(item: &Map<String, Value>) -> String {
    const ALIASES: [&str; 4] = ["apiBase", "endpoint", "apiHost", "domain"];
    let raw = ALIASES
        .iter()
        .map(|key| text_of(item.get(*key)))
        .find(|text| !text.is_empty())
        .unwrap_or_default();
    if raw.is_empty() {
        return raw;
    }
    let trimmed = raw.trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    }
}

fn normalize_workbuddy_known_fields(
    record: &mut Map<String, Value>,
    item: &Map<String, Value>,
    before: Option<&StoredAccount>,
    provider: &str,
) {
    let edition = crate::server::core::providers::workbuddy::Region::from_provider_id(provider)
        .map(|region| region.edition()).unwrap_or_else(|| resolve_edition(None));
    let explicit = |key: &str| item.get(key).filter(|value| !value.is_null());
    record.insert("edition".to_string(), Value::String(edition.id.to_string()));
    record.insert(
        "prefixPath".to_string(),
        Value::String(
            explicit("prefixPath")
                .map(|value| text_of(Some(value)))
                .or_else(|| before.and_then(StoredAccount::prefix_path))
                .unwrap_or_default(),
        ),
    );
    // ： `endpoint` / `apiBase` / `apiHost` / `domain` > old > empty。
    //
    // ── emptyWhy is （issue #381）──────────────────────────
    // `Context::make` in endpoint as empty when by **edition** to this 
    // （`workbuddy_shared::endpoints`）。「 」
    // is down that `edition` —— by provider id （`workbuddy-intl` → intl），
    // not use for 。 before not is 「endpoint empty」， is when 
    // `edition` `cn`（`resolve_edition(None)` ）， is 
    // to 。
    //
    // not ： that in （`EDITION_INTL.endpoint`），
    // use to or empty。
    let endpoint = {
        let incoming = endpoint_alias_of(item);
        if !incoming.is_empty() {
            incoming
        } else {
            before.and_then(StoredAccount::endpoint).unwrap_or_default()
        }
    };
    record.insert("endpoint".to_string(), Value::String(endpoint));
    record.insert(
        "platform".to_string(),
        Value::String(
            explicit("platform")
                .map(|value| text_of(Some(value)))
                .or_else(|| before.and_then(StoredAccount::platform))
                .unwrap_or_default(),
        ),
    );

    let before_text = |getter: fn(&StoredAccount) -> String| -> String {
        before.map(getter).unwrap_or_default()
    };
    for (key, fallback) in [
        ("nickname", before_text(StoredAccount::nickname)),
        ("enterpriseId", before_text(StoredAccount::enterprise_id)),
        ("enterpriseName", before_text(StoredAccount::enterprise_name)),
        ("domain", before_text(StoredAccount::domain)),
    ] {
        let incoming = text_of(item.get(key));
        record.insert(
            key.to_string(),
            Value::String(if incoming.is_empty() { fallback } else { incoming }),
        );
    }
    let account_type = {
        let incoming = text_of(item.get("type"));
        if incoming.is_empty() {
            let fallback = before_text(StoredAccount::account_type);
            if fallback.is_empty() { "personal".to_string() } else { fallback }
        } else {
            incoming
        }
    };
    record.insert("type".to_string(), Value::String(account_type));
    for (key, getter) in [
        ("expiresAt", StoredAccount::expires_at as fn(&StoredAccount) -> Option<f64>),
        ("refreshExpiresAt", StoredAccount::refresh_expires_at),
    ] {
        let fallback = before
            .and_then(getter)
            .map(crate::server::core::account_store::state::json_number)
            .unwrap_or(Value::Null);
        let value = positive_number_of(item.get(key)).unwrap_or(fallback);
        record.insert(key.to_string(), value);
    }
}
