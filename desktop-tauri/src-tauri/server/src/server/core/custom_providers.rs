//! （ use up ）： + 。
//!
//! ── and （Why 1 ）──────────────
//! （`providers::PROVIDERS`） is ****：id is 、
//! and is 。 is ****： use 
//! 、 and ，id by —— not `ProviderKind` 、 not 
//! `PROVIDERS` ， not and `kind_from_id`（ that unknownmust
//! 「 not is not 」）。
//!
//! use config kv ： `customProviders`， is JSON 。
//! and `model_rules` （`config::current().raw()` 、
//! `config::update_raw_field(KEY, value)` ）—— this 「 can 
//! config」 has 1 ， when not 。
//!
//! ── （ for ， need before after 1 ）──────────────────
//! ```json
//! {
//! "id": "custom-3f2a91b04c7e", // custom- + 12 （）
//! "name": " ", // 1~64 
//!   "protocol": "chat_completions", // chat_completions / responses / anthropic
//!   "baseUrl": "https://api.example.com/v1",
//!   "enabled": true,
//!   "createdAt": 1730000000000,
//! "models": [ // use （）
//!     { "id": "gpt-x", "enabled": true, "reasoning": "",
//! "capabilities": { "maxInputTokens": 200000 } } // optional： can 
//!   ],
//! "mappings": [ // for → up id （ up ）
//!     { "alias": "my-alias", "target": "gpt-x", "enabled": true, "reasoning": "" }
//!   ],
//! // per-provider （optional，「none」；create when ）：
//! "urlSuffix": "?beta=true", // to URL 
//! "headers": { "Anthropic-Beta": "…" }, // and to default up 
//! "anthropicToolType": "custom", // anthropic up when to type
//! "clientEmulation": "opencode" // by OpenCode CLI request
//! }
//! ```
//!
//! `models` / `mappings` Why in **** not is that 
//! `modelRules`： （directory + ）， then in 
//! (provider, model id) up ； is use ，
//! 「enabled」 is —— 1 modelRules etc 1 
//! （`set_models` for not up 「」）。
//! modelRules disabled/hidden ** not used for**（ not ）。
//!
//! `capabilities`（optional per-model can ） 1 ： that 
//! `modelRules.capabilities`（ is to ，「 up 」 is then ），
//! and ** is this **， in up 。
//! / 1 / then by `core::capability` 1 （ in 
//! `bindings::public_models`： target can ）。
//!
//! ── （ before ）───────────────────────────
//! 1. **id use not is when /**：、 and not can 
//! ；`custom-` before is 「 this is 」 （`is_custom_provider_id`），
//! before is 1 ；
//! 2. **baseUrl **：empty + 。 when path
//! （`{baseUrl}/chat/completions` this ） will ， will 
//! `//` —— up can ，（Anthropic 
//! URL ） will ， and its use in ， not when 1 ；
//! 3. **apiKey not **： is **** （ 1 can 、
//! 1 key）， in `apiKey` （
//! `account_store::custom_accounts`）。「 this is 」。
//!
//! ── （W ）────────────────────────────────
//! and API、、** + directory/ by + Chat
//! Completions **（ `providers::custom::forward`）。
//! `is_custom_provider_id` / `protocol_of` / `wire_model_for` is 
//! ；`responses` / `anthropic` —— config can 
//! ， when 400（ `providers::custom::forward` ）。

mod bindings;
mod presets;
/// **** and （file `customProviders` ）。
///
/// `pub(crate)`： use is `account_transfer::import_accounts` —— 
/// before （ has ， not empty）。 not this ：
/// `create` / `update` / `remove`， not 。
pub(crate) mod transfer;
pub use bindings::public_models;

use std::time::Duration;

use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::capability;
use crate::server::core::egress;
use crate::server::core::models::model_id;
use crate::server::errors::GatewayError;
use crate::server::logging;

/// kv ：config this 。
///
/// and `model_rules::KEY_MODEL_RULES` 1 —— is `config` 
/// ，** not can ** and `db::schema::RESERVED_KV_KEYS` （ after 
/// `config` module header）。
pub const KEY_CUSTOM_PROVIDERS: &str = "customProviders";

/// provider id before 。
///
/// 1 ：`is_custom_provider_id` need 「** and ** can in to 」。
/// `custom-` id（ ） not —— that is 
/// 「 no has 、 no has 」 id， by unknown（ and provider
/// 1 ： not id not 1 ）。
pub const ID_PREFIX: &str = "custom-";

/// ：OpenAI Chat Completions
pub const PROTOCOL_CHAT_COMPLETIONS: &str = "chat_completions";
/// ：OpenAI Responses
pub const PROTOCOL_RESPONSES: &str = "responses";
/// ：Anthropic Messages
pub const PROTOCOL_ANTHROPIC: &str = "anthropic";

/// （ use ；error and before down ）
pub const PROTOCOLS: &[&str] = &[PROTOCOL_CHAT_COMPLETIONS, PROTOCOL_RESPONSES, PROTOCOL_ANTHROPIC];

/// limit（；1~64 is ）
const MAX_NAME_CHARS: usize = 64;

/// id / limit（）。 up id 1 ，
/// 128 and `model_rules` limit —— 1 when not 。
const MAX_MODEL_ID_CHARS: usize = 128;
/// （ for ） limit（； and `model_rules::alias_valid` ）
const MAX_ALIAS_CHARS: usize = 128;
/// etc limit（； and `model_rules::normalize_reasoning` ）
const MAX_REASONING_CHARS: usize = 32;

/// ── per-provider （directory up ） limit ──
/// urlSuffix（， "?beta=true"）： 1 ，128 already 
const MAX_URL_SUFFIX_CHARS: usize = 128;
/// limit。 need need 1 （Anthropic-Beta / Referer ），
/// **client** need client ：Cline Pass is 
/// 9 （ and `providers/cline/headers.rs` DEFAULT_HEADERS 1 ， 1 
/// up by client will 403）。 16 —— down 「 + 」，
/// headers configfile 。
const MAX_HEADER_ENTRIES: usize = 16;
/// / limit： RFC 、（Anthropic-Beta 
/// beta ） can can up ，512 not 、 
const MAX_HEADER_NAME_CHARS: usize = 64;
const MAX_HEADER_VALUE_CHARS: usize = 512;
/// type 1 ：MiniMax Claude none type 
/// （`requireClaudeToolType`， translator/concerns/toolCall.js），
/// for to `type: "custom"`。empty = by default（none type）。
pub const TOOL_TYPE_CUSTOM: &str = "custom";

/// client 1 ：OpenCode CLI。 when will by client 
/// request（`Bearer public` 、`ses_…` will 、`bash` / `read` 
/// ），otherwise OpenCode Zen 1 403 `FreeTierError`（ and 
/// `providers::custom::emulation`）。empty = not （default）。
///
/// ** is opt-in and will request**，must by ****（
/// 「OpenCode Zen」 will ）， not can by baseUrl —— after is to up 
/// 。
pub const CLIENT_EMULATION_OPENCODE: &str = "opencode";

/// ── （`balanceScheme`）────────────────────────────
///
/// up **** has no has 、， not （ / 
/// / JWT / / API Key ）， this must，
/// not can by baseUrl 。 `providers::custom::balance`
/// （：OmniProxy `BalanceScheme`，， for ；
/// `zhipu_zai` is ）。
///
/// ── this ──────────────────────────────────────────────
/// ** API **（`ui/preset-providers.js` `balance.scheme`）： use from 
/// 「 API」 DeepSeek / NewAPI / new this when ， before 
/// 1 and ， after in this after 。 new not this ——
/// for 「 already is 1 」 up （ down baseUrl ）。
///
/// ── Why need by baseUrl ────────────────────────
/// this is new ：（ 1 before from ） no has ， 
/// baseUrl is that （DeepSeek / OpenCode Go / /
/// ）** can can **， is by 1 ， after can use 。
/// up （sub2api / NewAPI ） no has ，must 1 or 
/// —— this need use ， 1 not 。
pub const BALANCE_SCHEME_SUB2API: &str = "sub2api";
pub const BALANCE_SCHEME_NEW_API: &str = "new_api";
pub const BALANCE_SCHEME_DEEPSEEK: &str = "deepseek";
pub const BALANCE_SCHEME_OPENCODE: &str = "opencode";
pub const BALANCE_SCHEME_SENSENOVA: &str = "sensenova";
pub const BALANCE_SCHEME_ZHIPU: &str = "zhipu";
/// ****（z.ai）： and 1 、 1 ，
/// and not （API Key SIGN / ）， is not is 
/// `zhipu` 1 parameter —— 「、 use 」， to 
/// 「set」 not 1 （ need ， not use ）。
pub const BALANCE_SCHEME_ZHIPU_ZAI: &str = "zhipu_zai";

/// （ and 1 use ； not as ）
pub const BALANCE_SCHEMES: &[&str] = &[
    BALANCE_SCHEME_SUB2API,
    BALANCE_SCHEME_NEW_API,
    BALANCE_SCHEME_DEEPSEEK,
    BALANCE_SCHEME_OPENCODE,
    BALANCE_SCHEME_SENSENOVA,
    BALANCE_SCHEME_ZHIPU,
    BALANCE_SCHEME_ZHIPU_ZAI,
];

/// up total when （ms）。 is 1 ，
/// not and for （egress read_timeout is to SSE ）。
const FETCH_MODELS_TIMEOUT_MS: u64 = 15_000;

/// （： / 1 ）。
///
/// has 1 ：** not is empty id for **。 its （name / protocol /
/// baseUrl / enabled） when by default（ `item_of`）， not in this ——
/// 「 can when 」「」。
fn read_items() -> Vec<Value> {
    let raw = config::current();
    let items = raw
        .raw()
        .get(KEY_CUSTOM_PROVIDERS)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut items: Vec<Value> = items
        .into_iter()
        .filter_map(|item| {
            let object = item.as_object()?;
            let id = object.get("id").and_then(Value::as_str)?.trim();
            if id.is_empty() {
                return None;
            }
            Some(item_of(object))
        })
        .collect();
    // for is 「 by createdAt 」。 is this （create ），
    // config/ old can can ， when **total is ** 1 —— ，
    // 1 createdAt（ms） has for 。
    items.sort_by_key(|item| {
        item.get("createdAt").and_then(Value::as_i64).unwrap_or(0)
    });
    items
}

/// 1 → for ： default，** has **。
///
/// Why in this 「」 not is ： config can can 1 `enabled`，
/// will before to undefined， as in （、 ）。
/// after for ， and new path 1 。
///
/// `models` / `mappings` in this default（empty、）——
/// path（`carriers_of_model` / `wire_model_for`） ，
/// in this 1 ， not 。
fn item_of(object: &Map<String, Value>) -> Value {
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or(&id)
        .to_string();
    let protocol = object
        .get("protocol")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| valid_protocol(text))
        .unwrap_or(PROTOCOL_CHAT_COMPLETIONS)
        .to_string();
    let client_emulation = client_emulation_of(object.get("clientEmulation"));
    let base_url = migrate_base_url(
        object
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or(""),
        client_emulation == CLIENT_EMULATION_OPENCODE,
    );
    let mut models = model_entries_of(object.get("models"));
    let mut mappings = mapping_entries_of(object.get("mappings"));
    bindings::normalize(&mut models, &mut mappings);
    let mut item = json!({
        "id": id,
        "name": name,
        "protocol": protocol,
        "baseUrl": base_url,
        "enabled": object.get("enabled").and_then(Value::as_bool).unwrap_or(true),
        "createdAt": object.get("createdAt").and_then(Value::as_i64).unwrap_or(0),
        "models": models,
        "mappings": mappings,
        // per-provider ： 1 （ by ， and models 1 ）。
        // 「none」—— no has this ， by 。
        "urlSuffix": url_suffix_of(object.get("urlSuffix")),
        "headers": headers_of(object.get("headers")),
        "anthropicToolType": tool_type_of(object.get("anthropicToolType")),
        "clientEmulation": client_emulation,
        // ：** will by baseUrl **（ no has this ，
        // BALANCE_SCHEMES ）， this to not is is 1 
        "balanceScheme": balance_scheme_of_record(object),
    });
    presets::write_key(&mut item, presets::key_of_record(object));
    item
}

/// 1 ：。 use （ ），
/// / and baseUrl 1 is when by [`infer_balance_scheme`] 。
///
/// not by 「none」 not is ：path is 「 can 、 can use 」，
/// can （ and `client_emulation_of` 1 to ）—— 1 not this 
/// 。
fn balance_scheme_of_record(object: &Map<String, Value>) -> String {
    let stored = object
        .get("balanceScheme")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| BALANCE_SCHEMES.iter().any(|scheme| *scheme == *text));
    if let Some(scheme) = stored {
        return scheme.to_string();
    }
    infer_balance_scheme(
        object
            .get("baseUrl")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or(""),
    )
    .unwrap_or("")
    .to_string()
}

/// by baseUrl （** that **）。
///
/// is after ， not is etc ： this up path not 1 
/// （`/v1` has no has 、coding and use ）， will 1 。
/// up （sub2api / NewAPI） not in —— none from ， will 
/// up 。
fn infer_balance_scheme(base_url: &str) -> Option<&'static str> {
    let host = base_url
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("")
        .split('@')
        .next_back()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }
    let matches = |suffix: &str| host == suffix || host.ends_with(&format!(".{suffix}"));
    if matches("api.deepseek.com") {
        return Some(BALANCE_SCHEME_DEEPSEEK);
    }
    // OpenCode Go （） has `/v1/usage`；（Zen） no has ，
    // —— 「 path」， will failure and ，
    // not 。client and not （ `is_opencode_go_endpoint`）。
    if matches("opencode.ai") {
        return Some(BALANCE_SCHEME_OPENCODE);
    }
    // ： bigmodel.cn **** ，（z.ai） use 
    // API Key SIGN （ providers::custom::balance module header）。
    // 1 ， and not ， is 。
    if matches("bigmodel.cn") {
        return Some(BALANCE_SCHEME_ZHIPU);
    }
    if matches("z.ai") {
        return Some(BALANCE_SCHEME_ZHIPU_ZAI);
    }
    if matches("sensenova.cn") {
        return Some(BALANCE_SCHEME_SENSENOVA);
    }
    None
}

/// （`None` = this 1 no ）。
///
/// ── Why [`config::with_raw`] not is `get()` ─────────────
/// is （`account_store::store_view`，**** use ），
/// `get()` will `list()` config（）。 this need need 1 
/// 1 ， can （ is ， `with_raw` ）。
pub fn balance_scheme_of(provider_id: &str) -> Option<String> {
    let id = provider_id.trim();
    if id.is_empty() {
        return None;
    }
    let scheme = config::with_raw(|raw| {
        let items = raw.get(KEY_CUSTOM_PROVIDERS)?.as_array()?;
        let item = items
            .iter()
            .find(|item| item.get("id").and_then(Value::as_str) == Some(id))?
            .as_object()?;
        let scheme = balance_scheme_of_record(item);
        (!scheme.is_empty()).then_some(scheme)
    })?;
    scheme
}

/// 1 ：urlSuffix。 = empty（none） or `?` 、noneempty、
/// has （ `?beta=true`）； 1 by none —— 
/// not this as 1 after ， after available。
fn url_suffix_of(value: Option<&Value>) -> String {
    let text = value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let usable = text.starts_with('?')
        && text.len() >= 2
        && text.len() <= MAX_URL_SUFFIX_CHARS
        && !text.chars().any(char::is_whitespace);
    if usable { text.to_string() } else { String::new() }
}

/// 1 ：。「 is empty」 （limit
/// [`MAX_HEADER_ENTRIES`] ， / —— and model_entries_of 
/// 「 not 」 1 to ）； after 1 ， before 。
fn headers_of(value: Option<&Value>) -> Map<String, Value> {
    let mut out = Map::new();
    let Some(entries) = value.and_then(Value::as_array) else {
        return out;
    };
    for entry in entries.iter().take(MAX_HEADER_ENTRIES * 2) {
        let Some(object) = entry.as_object() else { continue };
        // [{name, value}] and for {"Name": "value"} ：
        // for is create/ ， to will 
        let pairs: Vec<(String, String)> = if let Some(name) =
            object.get("name").and_then(Value::as_str)
        {
            match object.get("value").and_then(Value::as_str) {
                Some(value) => vec![(name.to_string(), value.to_string())],
                None => vec![],
            }
        } else {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|text| (key.clone(), text.to_string()))
                })
                .collect()
        };
        for (key, value) in pairs {
            let key = truncate_chars(key.trim(), MAX_HEADER_NAME_CHARS);
            let value = truncate_chars(value.trim(), MAX_HEADER_VALUE_CHARS);
            if key.is_empty() || value.is_empty() || key.contains(':') {
                continue;
            }
            out.insert(key, Value::String(value));
            if out.len() >= MAX_HEADER_ENTRIES {
                return out;
            }
        }
    }
    out
}

/// 1 ： type 。 before has `custom` 1 （empty = default）。
fn tool_type_of(value: Option<&Value>) -> String {
    let text = value.and_then(Value::as_str).map(str::trim).unwrap_or("");
    if text.eq_ignore_ascii_case(TOOL_TYPE_CUSTOM) {
        TOOL_TYPE_CUSTOM.to_string()
    } else {
        String::new()
    }
}

/// 1 ：client。 before has `opencode` 1 （empty = not ）。
///
/// and `tool_type_of` 1 ： not by 「 not 」 —— will request，
/// can 1 （ as up 403， can ） not need 1 。
fn client_emulation_of(value: Option<&Value>) -> String {
    let text = value.and_then(Value::as_str).map(str::trim).unwrap_or("");
    if text.eq_ignore_ascii_case(CLIENT_EMULATION_OPENCODE) {
        CLIENT_EMULATION_OPENCODE.to_string()
    } else {
        String::new()
    }
}

/// ：`models` → 1 after （ / 1 ）。
///
/// and `read_items` 1 to ：** not is empty id for ** not need ——
/// 1 no has id for by none， will 1 empty。
/// （ ）** not 、**：path is 
/// 「 can 、 can use 」， 。
fn model_entries_of(value: Option<&Value>) -> Vec<Value> {
    let mut entries: Vec<Value> = Vec::new();
    for item in value.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        let Some(object) = item.as_object() else {
            continue;
        };
        let id = truncate_chars(
            object.get("id").and_then(Value::as_str).map(str::trim).unwrap_or(""),
            MAX_MODEL_ID_CHARS,
        );
        if id.is_empty() {
            continue;
        }
        if entries.iter().any(|model| model_id(model).eq_ignore_ascii_case(&id)) {
            continue;
        }
        let mut entry = json!({
            "id": id,
            "enabled": object.get("enabled").and_then(Value::as_bool).unwrap_or(true),
            "reasoning": truncate_chars(
                object.get("reasoning").and_then(Value::as_str).map(str::trim).unwrap_or(""),
                MAX_REASONING_CHARS,
            ),
        });
        // can （optional ， and 1 then `core::capability`：
        // 1 in `modelRules.capabilities`， use this ）。
        // **empty not **：（ no has this ） when 1 not ，
        // configfile not none —— and `reasoning` empty，
        // that is default， this is optional 。
        let capabilities = capability::normalize_object(object.get("capabilities"));
        if !capabilities.is_empty() {
            if let Some(map) = entry.as_object_mut() {
                map.insert("capabilities".to_string(), Value::Object(capabilities));
            }
        }
        entries.push(entry);
    }
    entries
}

/// ：`mappings` → 1 after （ [`model_entries_of`]）。
///
/// 1 ：**alias and target 1 not can ** —— has not to 
/// up （ etc no ）， has target no has for （ not will has 
/// request in ）。 in 1 available 。
fn mapping_entries_of(value: Option<&Value>) -> Vec<Value> {
    let mut entries: Vec<Value> = Vec::new();
    for item in value.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]) {
        let Some(object) = item.as_object() else {
            continue;
        };
        let alias = truncate_chars(
            object.get("alias").and_then(Value::as_str).map(str::trim).unwrap_or(""),
            MAX_ALIAS_CHARS,
        );
        let target = truncate_chars(
            object.get("target").and_then(Value::as_str).map(str::trim).unwrap_or(""),
            MAX_MODEL_ID_CHARS,
        );
        if alias.is_empty() || target.is_empty() {
            continue;
        }
        if entries.iter().any(|mapping| mapping.get("alias").and_then(Value::as_str)
            .is_some_and(|id| id.eq_ignore_ascii_case(&alias))
            && mapping.get("target").and_then(Value::as_str).is_some_and(|id| id.eq_ignore_ascii_case(&target)))
        { continue; }
        entries.push(json!({
            "alias": alias,
            "target": target,
            "enabled": object.get("enabled").and_then(Value::as_bool).unwrap_or(true),
            "reasoning": truncate_chars(
                object.get("reasoning").and_then(Value::as_str).map(str::trim).unwrap_or(""),
                MAX_REASONING_CHARS,
            ),
        }));
    }
    entries
}

/// by （limit ， not panic）
fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// （ 1 path）。
///
/// return is success；failure when ** use mustfailure**（ new no has ）——
/// and `model_rules::save` 1 ，failure by `config::update_raw_field` 
/// ， this to up ， not 1 。
fn write_items(items: &[Value]) -> bool {
    config::update_raw_field(KEY_CUSTOM_PROVIDERS, Value::Array(items.to_vec()))
}

/// current（ by createdAt ）。
pub fn list() -> Vec<Value> {
    read_items()
}

/// need （`{id, label, count}`）——「optional」 this 
/// ** that 1 **（ 1 is ， `providers::summary_json`）。
///
/// and that 1 ** 1 **：（`api::keys_api` 「available」）
/// 1 ， not in 1 ， that 
/// will 「 1 1 」 —— 1 when ， to is 1 
/// none （ not this 1 is ）。
///
/// `label` `name`： use is this 1 （ and `label_of` 1 ，
/// id not is 「unknown」—— `providers::label_of` ）。
///
/// Why `count` by use not is in this ： in `AccountStore` ， is 
/// config、 not （ and `providers::summary_json` 1 ）。 as 1 
/// ， `count` —— is 「 this in has 」。
pub fn summary_json<F>(count: F) -> Vec<Value>
where
    F: Fn(&str) -> usize,
{
    list()
        .into_iter()
        .filter_map(|item| {
            let id = item.get("id").and_then(Value::as_str)?;
            let label = item.get("name").and_then(Value::as_str).unwrap_or(id);
            Some(json!({ "id": id, "label": label, "count": count(id) }))
        })
        .collect()
}

/// by id 1 ； not in return None。
pub fn get(id: &str) -> Option<Value> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    read_items()
        .into_iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
}

/// this id is not is ** already **。
///
/// need ： [`ID_PREFIX`] 、 and can in to 。 before will 
/// 「 custom-xxx」 （、delete will ，
/// no has and ，failure）； and not 。
///
/// ：`providers::label_of`（）、`api::accounts::add_account`
/// 、 ， and down 1 path（ this ，
/// not ）。
pub fn is_custom_provider_id(id: &str) -> bool {
    let id = id.trim();
    id.starts_with(ID_PREFIX) && get(id).is_some()
}

/// ； not in return None（ use to ）。
pub fn label_of(id: &str) -> Option<String> {
    get(id).and_then(|item| {
        item.get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

/// ； not in return None。
///
/// down 1 （ 1 ）； API and use 。
pub fn protocol_of(id: &str) -> Option<String> {
    get(id).and_then(|item| {
        item.get("protocol")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

/// new 1 ：`{name, protocol, baseUrl}` → after 。
///
/// `apiKey` ** not in **： is （module header）， by 
/// use （`api::custom_providers::create`） in to provider id after 
/// 1 。
pub fn create(payload: &Value) -> Result<Value, String> {
    let object = payload
        .as_object()
        .ok_or_else(|| "请求体必须是 JSON 对象".to_string())?;
    let name = normalize_name(object.get("name"))?;
    let protocol = normalize_protocol(object.get("protocol"))?;
    let base_url = normalize_base_url(
        object
            .get("baseUrl")
            .and_then(Value::as_str)
            .unwrap_or(""),
    )?;
    let preset_key = presets::validate(object.get("presetKey"))?;
    let id = new_provider_id()?;
    let mut item = json!({
        "id": id,
        "name": name,
        "protocol": protocol,
        "baseUrl": base_url,
        "enabled": true,
        "createdAt": logging::now_ms(),
        "models": [],
        "mappings": [],
        // per-provider （directory up ）， validate_* 
        "urlSuffix": validate_url_suffix(object.get("urlSuffix"))?,
        "headers": validate_headers(object.get("headers"))?,
        "anthropicToolType": validate_tool_type(object.get("anthropicToolType"))?,
        "clientEmulation": validate_client_emulation(object.get("clientEmulation"))?,
        // （， BALANCE_SCHEMES ）
        "balanceScheme": validate_balance_scheme(object.get("balanceScheme"))?,
    });
    presets::write_key(&mut item, preset_key);
    let mut items = read_items();
    items.push(item.clone());
    if !write_items(&items) {
        return Err("保存失败：配置写入未成功（请检查磁盘空间与配置目录权限）".to_string());
    }
    Ok(item)
}

/// update 1 ： not ， protocol / baseUrl when 。
///
/// Whyunsupported `id` / `createdAt`： before is to （ will 
/// down ）， after is —— is 1 ， not is can 。
pub fn update(id: &str, payload: &Value) -> Result<Value, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("缺少提供商 id".to_string());
    }
    let object = payload
        .as_object()
        .ok_or_else(|| "请求体必须是 JSON 对象".to_string())?;
    let mut items = read_items();
    let Some(index) = items
        .iter()
        .position(|item| item.get("id").and_then(Value::as_str) == Some(id))
    else {
        return Err(format!("自定义提供商不存在: {id}"));
    };
    // ：** in request when **。 1 （ not ignore）——
    // 「success no 」 1 400 。
    let current = items[index].clone();
    let mut merged = current.as_object().cloned().unwrap_or_default();
    if let Some(value) = object.get("name") {
        let name = normalize_name(Some(value))?;
        merged.insert("name".to_string(), Value::String(name));
    }
    if let Some(value) = object.get("protocol") {
        let protocol = normalize_protocol(Some(value))?;
        merged.insert("protocol".to_string(), Value::String(protocol));
    }
    if let Some(value) = object.get("baseUrl") {
        let base_url = normalize_base_url(value.as_str().unwrap_or(""))?;
        merged.insert("baseUrl".to_string(), Value::String(base_url));
    }
    if let Some(value) = object.get("enabled") {
        let enabled = value
            .as_bool()
            .ok_or_else(|| "enabled 必须是布尔值".to_string())?;
        merged.insert("enabled".to_string(), Value::Bool(enabled));
    }
    // by patch ：，（ and up 1 to ）。
    // not this ， from create in / after 。
    if let Some(value) = object.get("urlSuffix") {
        merged.insert(
            "urlSuffix".to_string(),
            Value::String(validate_url_suffix(Some(value))?),
        );
    }
    if let Some(value) = object.get("headers") {
        merged.insert(
            "headers".to_string(),
            Value::Object(validate_headers(Some(value))?),
        );
    }
    if let Some(value) = object.get("anthropicToolType") {
        merged.insert(
            "anthropicToolType".to_string(),
            Value::String(validate_tool_type(Some(value))?),
        );
    }
    if let Some(value) = object.get("clientEmulation") {
        merged.insert(
            "clientEmulation".to_string(),
            Value::String(validate_client_emulation(Some(value))?),
        );
    }
    // ： and up 1 patch （）。 not ——
    // is 「 this 1 is 」 ， use / not this ；
    // need empty（empty = none）。
    if let Some(value) = object.get("balanceScheme") {
        merged.insert(
            "balanceScheme".to_string(),
            Value::String(validate_balance_scheme(Some(value))?),
        );
    }
    let updated = Value::Object(merged);
    items[index] = updated.clone();
    if !write_items(&items) {
        return Err("保存失败：配置写入未成功（请检查磁盘空间与配置目录权限）".to_string());
    }
    Ok(updated)
}

/// delete 1 ，** and delete this down **；return 。
///
/// ── Whydelete is must （ not is to use ）──────────────
/// is `provider` 。 after ， that will 
/// 「 to 1 not in 」：、 by not up 、
/// use 。 this and `model_rules::remove_custom` 
/// then is 1 —— delete is 「 this 1 not need 」， 1 。
///
/// ── ：， ───────────────────────────────
/// delete can can as unavailablefailure； that down **must**（otherwise
/// down 1 ，「 no 」）。（failure、
/// already ） will down 1 as 0 ， use 1 can —— to 
/// after not for etc ， not can 。
pub fn remove(id: &str, store: &AccountStore) -> Result<usize, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("缺少提供商 id".to_string());
    }
    if get(id).is_none() {
        return Err(format!("自定义提供商不存在: {id}"));
    }
    let accounts_removed = store
        .remove_custom_accounts(id)
        .map_err(|error| error.message)?;
    let mut items = read_items();
    items.retain(|item| item.get("id").and_then(Value::as_str) != Some(id));
    if !write_items(&items) {
        return Err("保存失败：配置写入未成功（请检查磁盘空间与配置目录权限）".to_string());
    }
    Ok(accounts_removed)
}

// ─── and （：directory / by / ）──────

/// this （ 1 after ；`item_of` already default， this is ）。
///
/// ：directory（`providers::custom` ）、`carriers_of_model` 
/// can 、 and 。**enabled not in this 1 ** —— 「」
/// and 「available 」 is （ for `catalog::manifest_for` and 
/// `advertised_manifest_for` ）， use by need 。
pub fn models_of(id: &str) -> Vec<Value> {
    get(id)
        .and_then(|item| item.get("models").and_then(Value::as_array).cloned())
        .unwrap_or_default()
}

/// this （ 1 after ； [`models_of`]）。
pub fn mappings_of(id: &str) -> Vec<Value> {
    get(id)
        .and_then(|item| item.get("mappings").and_then(Value::as_array).cloned())
        .unwrap_or_default()
}

/// **** this `models` and `mappings` and （UI ）。
///
/// ── Why is 「」 not is ────────────────────────
/// is 「 1 → 」：、
/// 、 1 reasoning。 API need 、 need before N 
/// request and in failure will down 。 and 1 1 for ，
/// （failure not ， `write_items` ）。
///
/// ── down ────────────────────────────────────────
/// is **** （`read_items` / `write_items`），must 
/// has 、 this —— 1 this for 
/// will name / protocol / baseUrl / enabled 。 in before ：
/// （empty id、、 not for ） 1 400， not —— 「success
/// no 」 1 error。
///
/// returnupdate after ****（response `{provider: ...}` to before ）。
pub fn set_models(id: &str, models: Value, mappings: Value) -> Result<Value, GatewayError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(GatewayError::bad_request("缺少提供商 id"));
    }
    let models_raw = models.as_array().ok_or_else(|| GatewayError::bad_request("models 必须是数组"))?;
    let mappings_raw = mappings.as_array().ok_or_else(|| GatewayError::bad_request("mappings 必须是数组"))?;
    let models = validate_model_entries(models_raw).map_err(GatewayError::bad_request)?;
    let mappings = validate_mapping_entries(mappings_raw).map_err(GatewayError::bad_request)?;
    config::try_update_raw_field(KEY_CUSTOM_PROVIDERS, |value| {
        let items = value.as_array_mut().ok_or_else(|| GatewayError::with_status(404, format!("自定义提供商不存在: {id}")))?;
        let stored = items.iter_mut().find(|item| item.get("id").and_then(Value::as_str) == Some(id))
            .ok_or_else(|| GatewayError::with_status(404, format!("自定义提供商不存在: {id}")))?;
        let previous = stored.as_object().map(item_of).unwrap_or(Value::Null);
        let groups = config::routing_profiles();
        if models.iter().filter_map(|item| item.get("id").and_then(Value::as_str))
            .chain(mappings.iter().filter_map(|item| item.get("alias").and_then(Value::as_str)))
            .any(|model| groups.group_for_model(model).is_some()) {
            return Err(GatewayError::with_status(409, "模型或映射的对外 ID 已被自定义路由组占用"));
        }
        bindings::validate_change(&previous, &models, &mappings)?;
        let map = stored.as_object_mut().ok_or_else(|| GatewayError::bad_request("自定义提供商记录损坏"))?;
        map.insert("models".to_string(), Value::Array(models.clone()));
        map.insert("mappings".to_string(), Value::Array(mappings.clone()));
        Ok(item_of(map))
    }, || GatewayError::new("保存失败：配置写入未成功，本次修改未生效；请检查磁盘空间与配置目录权限"))
}

/// 「 this by 」（`catalog::providers_for_model` ）。
///
/// （ when ）： this `enabled=true`， and in 1 **enabled**
/// id ， or 1 **enabled** and `alias==name` 。
/// `eq_ignore_ascii_case` —— and 1 （`model_rules` 
/// module header），client `GPT-X` and `gpt-x` must to 1 up 。
///
/// Note this ** not 、 not baseUrl**： is 「 can 」（ this 
/// ），「 can unavailable」 by directory（ need has enabled） and （
/// failure will ） —— and `manifest_for` 1 。
///
/// can ： `read_items()` （config is ，none IO）；
/// pathrequest use 1 ，none need 。
pub fn carriers_of_model(name: &str) -> Vec<String> {
    let name = name.trim();
    if name.is_empty() { return Vec::new(); }
    read_items().into_iter()
        .filter(|provider| bindings::resolve(provider, name).is_some())
        .filter_map(|provider| provider.get("id").and_then(Value::as_str).map(str::to_string))
        .collect()
}

/// to when use ** up ** and etc 。
///
/// ── （ and `catalog::wire_target_for_provider` for not ）──
/// ① in enabled and `alias == request` → = this `target`、
/// etc = this `reasoning`（ up no then empty）；
/// ② otherwise in enabled id → = request****（ is ，
/// 1 will 1 body）、 etc = this up `reasoning`；
/// ③ not in → = request、 etc = None（： down not will ，
/// is by [`carriers_of_model`] ）。
///
/// 1 `eq_ignore_ascii_case`（ by [`carriers_of_model`]）。
///
/// ── and etc （`providers::custom::forward`）────────
/// return `Some(level)` 「 this / up etc 」； need 
/// （ in use ， this not ）：clientrequest** no has ** to 
/// `reasoning_effort` / `reasoning`（ use default）， and etc not is 
/// `off`/`none`（ not to up 「」，
/// `model_rules::reasoning_is_off`）。 and etc **** —— 
/// use will 「A 、B etc 」 。
pub fn wire_model_for(provider_id: &str, requested_name: &str) -> (String, Option<String>) {
    let requested = requested_name.trim();
    get(provider_id).and_then(|provider| bindings::resolve(&provider, requested))
        .unwrap_or_else(|| (requested.to_string(), None))
}

/// **server** up （`POST /api/custom-providers/fetch-models`）。
///
/// Why by not is ： in origin up ，
/// will up CORS； and apiKey in server（ not ），
/// before key to —— by to 1 。
///
/// ── URL and （ baseUrl not ，）─────────────
/// - `chat_completions` / `responses`：baseUrl ** /v1**（
/// `https://api.example.com/v1`）→ `{baseUrl}/models`；
/// - `anthropic`：baseUrl is ** not /v1 **（
/// `https://api.anthropic.com`）→ `{baseUrl}/v1/models`。
/// ： before `Authorization: Bearer <key>`；anthropic is 
/// `x-api-key` + `anthropic-version: 2023-06-01`（ not is Bearer）。
///
/// key this ** 1 available** （`first_custom_credential`：enabled and 
/// 「apiKey empty ** or ** none need 」， by —— and 
/// 「」）； 1 no has when 「」。
/// up （`ResolvedProxy`）， no has then 。
///
/// ── as empty when ** not **（2026-09 ）──────────────────────
/// none need up （ Ollama、OpenCode Zen ） in `/models` up 
/// not 。 old in this none `Authorization: Bearer `（empty） or 
/// `x-api-key: `， up will 401 —— this path before not to （empty key
/// `first_custom_credential` ， that ）。
/// ** has not `clientEmulation` **：`/models` none need can 200，
/// 1 will this up 1 。
///
/// response：OpenAI and anthropic （`{data: [{id}]}`），
/// empty `id`。** not ** —— to before after 
/// `set_models`， and is 。
pub async fn fetch_upstream_models(
    provider_id: &str,
    store: &AccountStore,
) -> Result<Vec<String>, String> {
    let provider = get(provider_id)
        .ok_or_else(|| format!("自定义提供商不存在: {}", provider_id.trim()))?;
    let protocol = provider
        .get("protocol")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_CHAT_COMPLETIONS)
        .to_string();
    let base_url = provider
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if base_url.is_empty() {
        return Err("该提供商没有配置 baseUrl，无法拉取模型清单".to_string());
    }
    let url = if protocol == PROTOCOL_ANTHROPIC {
        format!("{base_url}/v1/models")
    } else {
        format!("{base_url}/models")
    };
    let credential = store
        .first_custom_credential(provider_id)
        .ok_or_else(|| {
            "请先添加账号：拉取模型清单需要一条启用且可用（填了 apiKey，或声明了无需鉴权）的账号"
                .to_string()
        })?;
    let client = egress::client_for(credential.proxy.as_ref());
    let mut builder = client
        .get(&url)
        .timeout(Duration::from_millis(FETCH_MODELS_TIMEOUT_MS));
    // as empty（none need ） when ** 1 not **：empty Bearer for up 
    // etc error， and 「 this not need 」 （ ）
    let api_key = credential.api_key.trim();
    if !api_key.is_empty() {
        builder = if protocol == PROTOCOL_ANTHROPIC {
            builder
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
        } else {
            builder.header("Authorization", format!("Bearer {api_key}"))
        };
    }
    let response = builder.send().await.map_err(|error| {
        format!("上游请求失败: {}", egress::describe_error_detail(&error))
    })?;
    let status = response.status().as_u16();
    let text = response.text().await.unwrap_or_default();
    if !response_ok(status) {
        return Err(format!("上游返回 {status}: {text}"));
    }
    let payload: Value = serde_json::from_str(&text)
        .map_err(|error| format!("上游响应不是合法 JSON: {error}"))?;
    let ids = payload
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("id")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|id| !id.is_empty())
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(ids)
}

/// up is success（2xx can ： will 200 success）
fn response_ok(status: u16) -> bool {
    (200..300).contains(&status)
}

/// ： ID、 and can must， ID（ignore）。
fn validate_model_entries(raw: &[Value]) -> Result<Vec<Value>, String> {
    let mut entries: Vec<Value> = Vec::new();
    for (position, item) in raw.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("models 第 {} 项必须是对象", position + 1));
        };
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if id.is_empty() {
            return Err(format!("models 第 {} 项缺少模型 id", position + 1));
        }
        if id.chars().count() > MAX_MODEL_ID_CHARS {
            return Err(format!(
                "模型 id 过长（最多 {MAX_MODEL_ID_CHARS} 个字符）"
            ));
        }
        let reasoning = validate_reasoning(object.get("reasoning"))?;
        // can （optional； validate_capabilities）—— is 
        // **must** ： before ， use can will 1 
        // 「」 。
        let capabilities = validate_capabilities(object.get("capabilities"), position + 1)?;
        let enabled = match object.get("enabled") {
            None => true,
            Some(Value::Bool(enabled)) => *enabled,
            Some(_) => return Err(format!("models 第 {} 项的 enabled 必须是布尔值", position + 1)),
        };
        let duplicated = entries.iter().any(|known| {
            known
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|text| text.eq_ignore_ascii_case(id))
        });
        if duplicated {
            return Err(format!("模型 ID「{id}」已存在，请勿重复登记"));
        }
        let mut entry = json!({
            "id": id,
            "enabled": enabled,
            "reasoning": reasoning,
        });
        if let Some(capabilities) = capabilities {
            if let Some(map) = entry.as_object_mut() {
                map.insert("capabilities".to_string(), Value::Object(capabilities));
            }
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// 1 (alias, target) not ； not target can ，enabled 1 。
fn validate_mapping_entries(raw: &[Value]) -> Result<Vec<Value>, String> {
    let mut entries: Vec<Value> = Vec::new();
    for (position, item) in raw.iter().enumerate() {
        let Some(object) = item.as_object() else {
            return Err(format!("mappings 第 {} 项必须是对象", position + 1));
        };
        let alias = object
            .get("alias")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        let target = object
            .get("target")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        if alias.is_empty() {
            return Err(format!("mappings 第 {} 项缺少映射名（alias）", position + 1));
        }
        if alias.chars().count() > MAX_ALIAS_CHARS {
            return Err(format!("映射名过长（最多 {MAX_ALIAS_CHARS} 个字符）"));
        }
        if target.is_empty() {
            return Err(format!("mappings 第 {} 项缺少目标上游模型（target）", position + 1));
        }
        if target.chars().count() > MAX_MODEL_ID_CHARS {
            return Err(format!(
                "目标上游模型过长（最多 {MAX_MODEL_ID_CHARS} 个字符）"
            ));
        }
        let reasoning = validate_reasoning(object.get("reasoning"))?;
        let enabled = match object.get("enabled") {
            None => true,
            Some(Value::Bool(enabled)) => *enabled,
            Some(_) => return Err(format!("mappings 第 {} 项的 enabled 必须是布尔值", position + 1)),
        };
        let duplicated = entries.iter().any(|known| {
            known.get("alias").and_then(Value::as_str)
                .is_some_and(|text| text.eq_ignore_ascii_case(alias))
                && known.get("target").and_then(Value::as_str)
                    .is_some_and(|text| text.eq_ignore_ascii_case(target))
        });
        if duplicated {
            return Err(format!("映射「{alias} → {target}」已存在，请勿重复添加"));
        }
        entries.push(json!({
            "alias": alias,
            "target": target,
            "enabled": enabled,
            "reasoning": reasoning,
        }));
    }
    Ok(entries)
}

/// can （`capabilities`） optional：must is for ， and need 
/// （ in `core::capability`， and that 1 ）。 when not is 
/// —— and `reasoning` 1 to ： before down ， not 1 will 
/// 1 。
///
/// return** 1 after **； / null / 1 after as emptyreturn `None`
/// （ use not ， `model_entries_of`）。
fn validate_capabilities(value: Option<&Value>, position: usize) -> Result<Option<Map<String, Value>>, String> {
    let Some(value) = value.filter(|item| !item.is_null()) else {
        return Ok(None);
    };
    let Some(object) = value.as_object() else {
        return Err(format!("models 第 {position} 项的 capabilities 必须是对象"));
    };
    for (key, item) in object {
        if !capability::KEYS.contains(&key.as_str()) {
            return Err(format!("models 第 {position} 项的能力字段不存在: {key}"));
        }
        if capability::normalize_value(key, item).is_none() {
            return Err(format!(
                "models 第 {position} 项的能力字段 {key} 取值非法（token 键要 1~1 亿的正整数，能力开关要布尔）"
            ));
        }
    }
    let normalized = capability::normalize_object(Some(value));
    Ok((!normalized.is_empty()).then_some(normalized))
}

/// reasoning ：/empty → empty（none）； to must is and ≤32 。
fn validate_reasoning(value: Option<&Value>) -> Result<String, String> {
    match value {
        None | Some(Value::Null) => Ok(String::new()),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.chars().count() > MAX_REASONING_CHARS {
                return Err(format!(
                    "思考等级过长（最多 {MAX_REASONING_CHARS} 个字符）"
                ));
            }
            Ok(text.to_string())
        }
        Some(_) => Err("reasoning 必须是字符串或 null".to_string()),
    }
}

/// new provider id：`custom-` + 12 。
///
/// 6 `getrandom`（ already in Cargo.toml；altcha / access etc 
/// use ）。** not use when or **：、 1 ms and 、 from 
/// config this down ， when can can ， after is config
/// for （`get` by id ）—— use then can ignore。
///
/// failure when （return Err）： can use 1 ， not 1 
/// id when —— that is 。
fn new_provider_id() -> Result<String, String> {
    let mut bytes = [0u8; 6];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| "无法生成安全的随机 id（系统随机源不可用），请重试".to_string())?;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!("{ID_PREFIX}{hex}"))
}

/// ：trim after 1~64 （ by ， not is —— in 1 64 limit）。
fn normalize_name(value: Option<&Value>) -> Result<String, String> {
    let name = value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if name.is_empty() {
        return Err("缺少提供商名称".to_string());
    }
    if name.chars().count() > MAX_NAME_CHARS {
        return Err(format!("提供商名称过长（最多 {MAX_NAME_CHARS} 个字符）"));
    }
    Ok(name.to_string())
}

/// ：must is [`PROTOCOLS`] 1 。
fn normalize_protocol(value: Option<&Value>) -> Result<String, String> {
    let protocol = value
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if !valid_protocol(protocol) {
        return Err(format!(
            "协议必须是以下之一: {}",
            PROTOCOLS.join(" / ")
        ));
    }
    Ok(protocol.to_string())
}

/// is （ not 、 not ；`item_of` use ）
fn valid_protocol(protocol: &str) -> bool {
    PROTOCOLS.iter().any(|known| *known == protocol)
}

/// baseUrl and ：must is http/https URL，empty and ****。
///
/// use `url::Url` not is before check： after `https://` this empty host、
/// empty、 or schema ， failure when will to when 
/// （400/ when / up ）， in this 。
/// `Url::parse` for 「 has no has host」 will ，success
/// host empty；`as_str()` for emptypath URL will 1 `/`（`https://a.com` →
/// `https://a.com/`）， 1 `trim_end_matches('/')` 1 —— use not 
/// after is 1 。 its （、、）。
///
/// `pub`： up `baseUrl` ****（`custom_accounts` ） 1 
/// —— and default ， 1 。
pub fn normalize_base_url(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("缺少 baseUrl".to_string());
    }
    let parsed = url::Url::parse(raw).map_err(|error| format!("baseUrl 不是合法的 URL: {error}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("baseUrl 必须以 http:// 或 https:// 开头".to_string());
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

/// OpenCode Zen ** old **：2026-10 1 403 `FreeTierError`
/// （， `providers::custom::emulation` module header）。
pub const LEGACY_OPENCODE_BASE_URL: &str = "https://opencode.ai/zen/v1";

/// OpenCode Zen **current**（ old already ， up ）。
pub const OPENCODE_BASE_URL: &str = "https://opencode.ai/inference/v1";

/// OpenCode **Go**（） 。 and need ** not **： need 「
/// client」（ will + for + ），Go need client** 1 
/// will **—— up `docs/go/#where-can-i-use-it` by need （「Send a
/// stable session ID in `x-opencode-session` for each conversation」），
/// `400 MissingSessionID`（： this check in before ， and in 、 not ）。
/// need not ， by ， `providers::custom::emulation`。
pub const OPENCODE_GO_BASE_URL: &str = "https://opencode.ai/zen/go/v1";

/// this baseUrl is in OpenCode Go （ignore and ASCII ）。
///
/// and [`migrate_base_url`] 1 ****： has that Go 
/// ， use in / 1 not （ that after is in 
/// ， not this ）。
pub fn is_opencode_go_endpoint(base_url: &str) -> bool {
    base_url
        .trim()
        .trim_end_matches('/')
        .eq_ignore_ascii_case(OPENCODE_GO_BASE_URL)
}

/// this （, ） is not is **OpenCode Zen ** —— 「」
/// 1 for 。
///
/// ：
/// 1. client（`clientEmulation == "opencode"`）。 this and 
/// `providers::custom::forward` 「 this need OpenCode that 」
/// is ** 1 **：VPN none Key、 by IP 、429 this IP ，
/// is this 1 。「OpenCode Zen」 is ；
/// 2. baseUrl not is Go 。OpenCode **Go** is （ by Key / 
/// ）， not ， for no has —— use 
/// in Go up when not this （`is_opencode_go_endpoint` is has 
/// ， use as ）。
///
/// `base_url_override` is **** up （None / empty = ，
/// baseUrl）： can can Go ， that 1 will 
/// 「 Zen、 Go」 this 。
pub fn is_opencode_zen_account(provider_id: &str, base_url_override: Option<&str>) -> bool {
    let Some(provider) = get(provider_id) else {
        return false;
    };
    let emulation = provider
        .get("clientEmulation")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !emulation.eq_ignore_ascii_case(CLIENT_EMULATION_OPENCODE) {
        return false;
    }
    let base_url = base_url_override
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .or_else(|| {
            provider
                .get("baseUrl")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    // baseUrl need ： can can old （/zen/v1），
    // is old —— this 1 will 「 old Zen 」 Zen
    let base_url = migrate_base_url(&base_url, true);
    !base_url.trim().is_empty() && !is_opencode_go_endpoint(&base_url)
}

/// ： ** etc old ** baseUrl current。
///
/// Why in not is 1 1 ： old is ，2026-10 
/// 1 403； has can can —— ，
/// and use in **** up （ that by `forward` in baseUrl after 1 
/// ）。 not 、 not can available， new 
/// use new 。
///
/// `is_opencode` is ****（ use `clientEmulation`）： for 
/// 「 by OpenCode client」 ， not ——
/// and `client_emulation` opt-in to 1 。
pub fn migrate_base_url(raw: &str, is_opencode: bool) -> String {
    let text = raw.trim();
    if is_opencode && text.trim_end_matches('/').eq_ignore_ascii_case(LEGACY_OPENCODE_BASE_URL) {
        return OPENCODE_BASE_URL.to_string();
    }
    text.to_string()
}

// ── per-provider （create / update use ）────────
//
// and （url_suffix_of / headers_of / tool_type_of） ： to 「 up 
// 」—— 1 ， by （ not this 1 after ）；
// to 「 use 」—— 。ignore etc 「success
// no 」， 1 400 （ and update has to 1 ）。

/// urlSuffix：empty = none；emptymust is `?` 、noneempty、 not 
/// [`MAX_URL_SUFFIX_CHARS`] （ `?beta=true`）。 when to 
/// URL （GLM / MiniMax Claude need `?beta=true`）。
fn validate_url_suffix(value: Option<&Value>) -> Result<String, String> {
    let Some(value) = value else { return Ok(String::new()) };
    let text = value.as_str().unwrap_or("").trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if !text.starts_with('?') {
        return Err("urlSuffix 必须是以 ? 开头的查询串（如 ?beta=true）".to_string());
    }
    if text.chars().any(char::is_whitespace) || text.chars().count() > MAX_URL_SUFFIX_CHARS {
        return Err(format!(
            "urlSuffix 不能包含空白且最多 {MAX_URL_SUFFIX_CHARS} 个字符"
        ));
    }
    Ok(text.to_string())
}

/// ：JSON for `{"": ""}`。 / is empty（
/// empty）， not ， not [`MAX_HEADER_ENTRIES`]。 when by and to 
/// default up —— ** default **（including Authorization / x-api-key： that is 
/// 「 1 」 use ， after default not ， by use ）。
fn validate_headers(value: Option<&Value>) -> Result<Map<String, Value>, String> {
    let Some(value) = value else { return Ok(Map::new()) };
    let object = value
        .as_object()
        .ok_or_else(|| "headers 必须是 JSON 对象（{\"头名\": \"值\"}）".to_string())?;
    if object.len() > MAX_HEADER_ENTRIES {
        return Err(format!("headers 最多 {MAX_HEADER_ENTRIES} 条"));
    }
    let mut out = Map::new();
    for (key, value) in object {
        let key = key.trim();
        let value_text = value.as_str().unwrap_or("").trim();
        if key.is_empty() || value_text.is_empty() {
            return Err("headers 的头名与值都必须是非空字符串".to_string());
        }
        if key.contains(':') || key.chars().any(char::is_whitespace) {
            return Err(format!("headers 的头名「{key}」不是合法的头名（不能含空白或冒号）"));
        }
        if key.chars().count() > MAX_HEADER_NAME_CHARS {
            return Err(format!("headers 的头名最长 {MAX_HEADER_NAME_CHARS} 个字符"));
        }
        if value_text.chars().count() > MAX_HEADER_VALUE_CHARS {
            return Err(format!("headers 的值最长 {MAX_HEADER_VALUE_CHARS} 个字符"));
        }
        out.insert(key.to_string(), Value::String(value_text.to_string()));
    }
    Ok(out)
}

/// type ：empty = default（none type）；`custom` = anthropic up when to 
/// `type: "custom"`。 its 1 —— this is request ， not is 
/// by ， etc as 。
fn validate_tool_type(value: Option<&Value>) -> Result<String, String> {
    let Some(value) = value else { return Ok(String::new()) };
    let text = value.as_str().unwrap_or("").trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if text.eq_ignore_ascii_case(TOOL_TYPE_CUSTOM) {
        return Ok(TOOL_TYPE_CUSTOM.to_string());
    }
    Err("anthropicToolType 只支持 \"custom\"（留空表示不补 type）".to_string())
}

/// client：empty = not ；`opencode` = by OpenCode CLI 
/// request（ `providers::custom::emulation`）。 its 1 —— and 
/// `validate_tool_type` 1 by ： is request ， etc 
/// as （ and this is **request**， after 1 ）。
fn validate_client_emulation(value: Option<&Value>) -> Result<String, String> {
    let Some(value) = value else { return Ok(String::new()) };
    let text = value.as_str().unwrap_or("").trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if text.eq_ignore_ascii_case(CLIENT_EMULATION_OPENCODE) {
        return Ok(CLIENT_EMULATION_OPENCODE.to_string());
    }
    Err(format!(
        "clientEmulation 只支持 \"{CLIENT_EMULATION_OPENCODE}\"（留空表示不伪装上游客户端）"
    ))
}

/// ：empty = this 1 not （ new ）；emptymust is 
/// 1 （ [`BALANCE_SCHEMES`]）。 its 1 —— and `validate_tool_type`
/// 1 by ：** up 、**， etc to 
/// up 。
///
/// this by （`ui/preset-providers.js` `balance.scheme`），
/// not to use （ is 「 this 1 is 」 ）。
fn validate_balance_scheme(value: Option<&Value>) -> Result<String, String> {
    let Some(value) = value else { return Ok(String::new()) };
    let text = value.as_str().unwrap_or("").trim();
    if text.is_empty() {
        return Ok(String::new());
    }
    if let Some(scheme) = BALANCE_SCHEMES.iter().find(|scheme| **scheme == text) {
        return Ok((*scheme).to_string());
    }
    Err(format!(
        "balanceScheme 只支持 {}（留空表示不查询余额）",
        BALANCE_SCHEMES.join(" / ")
    ))
}
