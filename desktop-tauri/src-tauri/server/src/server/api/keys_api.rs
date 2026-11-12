//! API Key （ Key ）：
//!
//! - `GET    /api/keys`          → `{keys: [{id,name,key,masked,enabled,createdAt,
//!                                  allowedProviders,allowedModels}], authRequired,
//!                                  providers, modelsByProvider}`
//! - `POST   /api/keys`          `{name?, key?, allowedProviders?, allowedModels?}` add；
//! key empty
//! - `PATCH  /api/keys/{id}`     `{name?, enabled?, allowedProviders?, allowedModels?}`
//! - `DELETE /api/keys/{id}`
//!
//! return new 。 key return： this is ， use need can when 
//! Key to client；used for。
//!
//! ── R9：available / available（ OmniProxy）──────────────────
//! ，**empty = unlimited**（ and 
//! `core::api_keys` module header）。response ，** not 
//! **：
//! · `providers`（ need + already ， as `{id,label,count}`）——
//! 「available」 。 this by after ： in ，
//! is 、 before need 1 `/api/custom-providers` and for when （
//! `list_json`）。 provider （ `account_store`
//! module header that ）， before not this 1 id ；
//! · `modelsByProvider`（ → for ）—— 「available」 。
//! by use current and （ 1 no is empty， `keys-panel.js`）。
//! this must by after to ： is + and ， before not to 
//! this ， `/api/session` `models` is **** after for ——
//! by will （
//! `catalog::models_by_provider` ）。
//!
//! in this 1 （`is_allowed_provider_id` id not 、must is 
//! ）， then 1 （ by `core::api_keys` module header that not ：
//! not can already has Key ）。 in not is core：core is 「 + 」，
//! 「」 in in 1 。

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::api_keys;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

fn list_json(state: &ServerState) -> Value {
    let entries = api_keys::list();
    let usable_key_count = entries.iter().filter(|entry| entry.is_usable()).count()
        + usize::from(crate::server::config::parse::env_api_key_value().is_some());
    let keys: Vec<Value> = entries.iter().map(api_keys::ApiKeyEntry::public_json).collect();
    // available ：** + already **（ 1 provider when 
    // this 1 ， before ）。 up count can 「 this has 」——
    // use in this 「」 when ， that has no has is need need 。
    //
    // must in this 1 and ： is （kv config，`custom-` before ），
    // no has ，「available」 that （`modelsByProvider`，
    // `catalog::models_by_provider`） in —— and not and this 
    // will 1 response （ can to ， not to ）。
    // before and not ： 1 `/api/custom-providers` 
    // when ， that not to （module header「 not 」）。
    //
    // （ 1 ， and `/api/session` providers need ）；
    // this as ， not ， by provider 1 can 。 use 
    // 1 ， and count is 1 。
    let accounts = crate::server::core::routing::accounts_of(&state.store().list_accounts());
    let count_accounts = |id: &str| {
        accounts
            .iter()
            .filter(|account| crate::server::core::routing::provider_of(account).eq_ignore_ascii_case(id))
            .count()
    };
    // provider id（）：`summary_json` use after 
    // 。 this 1 ， already Key `allowedProviders` that id will as 
    // not in `pickedFromOptions` （ 1 1 ）。
    let mut account_providers: Vec<String> = accounts
        .iter()
        .map(|account| crate::server::core::routing::provider_of(account).to_string())
        .collect();
    account_providers.sort();
    account_providers.dedup();
    let mut providers =
        crate::server::core::providers::summary_json(&count_accounts, &account_providers);
    providers.extend(crate::server::core::custom_providers::summary_json(&count_accounts));
    json!({
        "keys": keys,
        "authRequired": true,
        "usableKeyCount": usable_key_count,
        "providers": providers,
        // 「available」 ：** → for **， by current
        // and （module header that 1 and catalog::models_by_provider ）。
        // 1 to not is 「 by return and 」： need when ， and ，
        // 「response when use already 」 。
        "modelsByProvider": crate::server::core::providers::catalog::models_by_provider(&state.store()),
    })
}

/// 「available」 1 id not 。
///
/// ：（ `providers::is_known_provider_id`） and **current
/// in ** （`custom-` before and has ，
/// `custom_providers::is_custom_provider_id`）。
///
/// in 、 in ： not to 、 will 
/// 400 ， limit（`key_scope::allows_provider`） provider id
/// （、ignoreempty）—— ， use to is 「
/// not can Key」。
///
/// Why not 「 `custom-` before 1 」： before for not up is 
/// id（ no has and ，failure）， etc 1 Key available
/// 1 unavailable —— that is `is_custom_provider_id` need 。
fn is_allowed_provider_id(id: &str) -> bool {
    crate::server::core::providers::is_known_provider_id(id)
        || crate::server::core::custom_providers::is_custom_provider_id(id)
}

/// from request 1 （`None` = request no this = ** not **）。
///
/// and `model_rules` `reasoning` 1 （ that handler ）：
/// → `None`（ not ），empty → `Some(vec![])`（unlimited），
/// has → `Some(list)`。 1 by 「empty = empty」 will old before （ not this ）
/// limit —— that is 。
///
/// （unknown provider id / empty）： not 400， not is ——
/// use 1 up not in id can can is ， will as set。
fn parse_allowlist(
    object: &serde_json::Map<String, Value>,
    key: &str,
    check_provider: bool,
) -> Result<Option<Vec<String>>, Response> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    let items = match value {
        // null and empty（ is 「unlimited」）—— JSON client
        Value::Null => return Ok(Some(Vec::new())),
        Value::Array(items) => items,
        _ => return Err(errors::management_error(400, format!("{key} 必须是字符串数组"))),
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let Some(text) = item.as_str() else {
            return Err(errors::management_error(400, format!("{key} 的元素必须是字符串")));
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        if check_provider && !is_allowed_provider_id(text) {
            return Err(errors::management_error(400, format!("未知的提供商: {text}")));
        }
        if out.iter().any(|known: &String| known.eq_ignore_ascii_case(text)) {
            continue;
        }
        out.push(text.to_string());
    }
    Ok(Some(out))
}

fn body_object(body: &Bytes) -> Result<serde_json::Map<String, Value>, Response> {
    let payload = parse_body(body).map_err(|error| errors::management_error(400, error.message))?;
    payload
        .as_object()
        .cloned()
        .ok_or_else(|| errors::management_error(400, "请求体必须是 JSON 对象"))
}

/// already when ，「 after 1 enabled Key」（delete or disabled）。
///
/// 管理员一注册，`/api/*` need will or Key （`require_api_key` 
/// panel_auth ）。 in has will （`panel_session`： after 
/// `x-panel-token`），** has to Key ** —— need in no has will when 
/// 、 will after 、 and `AGENT2API_ALLOW_NO_KEY` not 。
/// 1 enabled Key not when ， this path will in ， Key need 
/// （）。 this ： new 、 old 。 before 
/// not 。
fn guard_last_active_key(entries: &[api_keys::ApiKeyEntry], target_id: &str, action: &str) -> Result<(), Response> {
    if !crate::server::access::panel_auth_enabled() {
        return Ok(());
    }
    let Some(target) = entries.iter().find(|entry| entry.id == target_id) else {
        return Ok(()); // not in id to has 404 path
    };
    if !target.enabled {
        return Ok(()); // is already disabled Key，enabled not will 
    }
    if entries.iter().filter(|entry| entry.enabled).count() > 1 {
        return Ok(());
    }
    Err(errors::management_error(
        400,
        format!(
            " already ， need 1 enabled Key（）：create new Key，{action} this 1 "
        ),
    ))
}

/// GET /api/keys
pub async fn list_keys(State(state): State<ServerState>) -> Response {
    ok_json(list_json(&state))
}

/// POST /api/keys
pub async fn create_key(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let name = object.get("name").and_then(Value::as_str).unwrap_or("");
    let key = object.get("key").and_then(Value::as_str);
    // new when = empty（= unlimited）； this and PATCH not ：
    // new no has old can ，「 no to 」 and 「 to empty」
    let allowed_providers = match parse_allowlist(&object, "allowedProviders", true) {
        Ok(value) => value.unwrap_or_default(),
        Err(response) => return response,
    };
    let allowed_models = match parse_allowlist(&object, "allowedModels", false) {
        Ok(value) => value.unwrap_or_default(),
        Err(response) => return response,
    };
    let expires_at = crate::server::core::key_governance::parse_expires_at(object.get("expiresAt"));
    let daily_request_limit = crate::server::core::key_governance::parse_limit(object.get("dailyRequestLimit"));
    let total_request_limit = crate::server::core::key_governance::parse_limit(object.get("totalRequestLimit"));
    let daily_token_limit = crate::server::core::key_governance::parse_limit(object.get("dailyTokenLimit"));
    let total_token_limit = crate::server::core::key_governance::parse_limit(object.get("totalTokenLimit"));
    let rpm_limit = crate::server::core::key_governance::parse_limit(object.get("rpmLimit"));
    match api_keys::add(
        name,
        key,
        allowed_providers,
        allowed_models,
        expires_at,
        daily_request_limit,
        total_request_limit,
        daily_token_limit,
        total_token_limit,
        rpm_limit,
    ) {
        Ok(entry) => {
            logging::log("[Config]", &format!("✅ add API Key「{}」，client need Authorization: Bearer <key>", entry.name));
            let mut payload = list_json(&state);
            if let Some(map) = payload.as_object_mut() {
                map.insert("created".to_string(), entry.public_json());
            }
            ok_json(payload)
        }
        Err(message) => errors::management_error(400, message),
    }
}

/// PATCH /api/keys/{id}
pub async fn update_key(State(state): State<ServerState>, Path(id): Path<String>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let name = object.get("name").and_then(Value::as_str);
    let enabled = object.get("enabled").and_then(Value::as_bool);
    // ： → None（ not ）。 old before name/enabled， is this 
    //（ after 「 」 not will limit， `api_keys::update` ）
    let allowed_providers = match parse_allowlist(&object, "allowedProviders", true) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let allowed_models = match parse_allowlist(&object, "allowedModels", false) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if let Some(false) = enabled {
        // disabled and delete 1 ： will 「enabled Key」
        if let Err(response) = guard_last_active_key(&api_keys::list(), &id, "disabled") {
            return response;
        }
    }
    let tri_i64 = |obj: &serde_json::Map<String, Value>, key: &str| -> Option<Option<i64>> {
        if !obj.contains_key(key) { return None; }
        Some(crate::server::core::key_governance::parse_expires_at(obj.get(key)))
    };
    let tri_u64 = |obj: &serde_json::Map<String, Value>, key: &str| -> Option<Option<u64>> {
        if !obj.contains_key(key) { return None; }
        Some(crate::server::core::key_governance::parse_limit(obj.get(key)))
    };
    let expires_at = tri_i64(&object, "expiresAt");
    let daily_request_limit = tri_u64(&object, "dailyRequestLimit");
    let total_request_limit = tri_u64(&object, "totalRequestLimit");
    let daily_token_limit = tri_u64(&object, "dailyTokenLimit");
    let total_token_limit = tri_u64(&object, "totalTokenLimit");
    let rpm_limit = tri_u64(&object, "rpmLimit");
    match api_keys::update(
        &id,
        name,
        enabled,
        allowed_providers,
        allowed_models,
        expires_at,
        daily_request_limit,
        total_request_limit,
        daily_token_limit,
        total_token_limit,
        rpm_limit,
    ) {
        Ok(entry) => {
            logging::log(
                "[Config]",
                &format!("API Key「{}」 already update（{}）", entry.name, if entry.enabled { "enabled" } else { "disabled" }),
            );
            ok_json(list_json(&state))
        }
        Err(message) => errors::management_error(404, message),
    }
}

/// DELETE /api/keys/{id}
pub async fn delete_key(State(state): State<ServerState>, Path(id): Path<String>) -> Response {
    if let Err(response) = guard_last_active_key(&api_keys::list(), &id, "delete") {
        return response;
    }
    match api_keys::remove(&id) {
        Ok(()) => {
            logging::log("[Config]", "API Key already delete； use need enabled Key");
            ok_json(list_json(&state))
        }
        Err(message) => errors::management_error(404, message),
    }
}
