//! WorkBuddy **config**。
//!
//! ── Why is 1 config ──────────────────────────────────
//! ， is up ： down 、、add， can can 
//! that —— failure 1 ， 1 。 to 
//! use ： to in 「 to 」， config.json 
//! `checkinKeepalive.models`。
//!
//! ── in （2026-10 ）─────────────────────────
//! 「 up has 」 is up ， to 。 this 
//! is **empty**： use no when not down `models`， by use ——
//! not has that will when 。 use use use （ that is config）。
//!
//! ── ─────────────────────────────────────────────────
//! ： for `{"models": […]}`（ is this 1 ） and 
//! `[…]`（ old / config）。 / up / is empty ——
//! and `auto_checkin::normalize_providers` 1 ： old config.json no has 
//! this ，must is default as 。 1 trim、empty、（）：
//! use in is 「/empty 1 」，api ，
//! this after 1 。

use serde_json::{Value, json};

use crate::server::config;
use crate::server::logging;

/// config.json 
const CONFIG_KEY: &str = "checkinKeepalive";

/// ：**empty** —— by use that （module header）。
///
/// this is as `models()` 「」 has ：
/// no has 「」 ，empty is this 。
pub fn default_models() -> Vec<String> {
    Vec::new()
}

/// 1 1 ： trim、empty、（）。
fn normalize(list: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::with_capacity(list.len());
    for item in list {
        let model = item.trim();
        if model.is_empty() || seen.iter().any(|existing| existing == model) {
            continue;
        }
        seen.push(model.to_string());
    }
    seen
}

/// current （config / when ）。
///
/// ── （ #183）────────────────────────────
/// （[`set_models`]） is ** for ** `{"models": […]}`， this by 
/// **** —— is for 、 by ，`Value::Array` not in ，
/// is ： as 「success、success， and after 
/// is 」， up is for 。 in for `models` ，（ old /
/// config）， 1 。
pub fn models() -> Vec<String> {
    // ：`config::current()` return when must to use 
    let snapshot = config::current();
    let Some(raw) = snapshot.raw().get(CONFIG_KEY) else {
        return default_models();
    };
    let items = match raw {
        Value::Array(items) => items.clone(),
        Value::Object(_) => raw
            .get("models")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => return default_models(),
    };
    let picked = normalize(
        &items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>(),
    );
    if picked.is_empty() {
        default_models()
    } else {
        picked
    }
}

/// 。**empty = **： use empty is 
/// 「 not this 」， to 
/// （ as empty etc failure， that not this is 1 config can ）。
/// return 1 after （failurereturn Err， use Reason to ）。
///
/// ── API Key ： 1 available Key ────────────────────────
/// before not Key is `None`， `authorized_models()` for `None` 
/// 「 API Key」—— use must in to in 1 can ，
/// up that in after 。 in to [`first_key_id`]：
/// has available Key use ， use after 。 `null` 「 not 」，
/// that is use empty down ，must。
pub fn set_models(list: &[String], api_key_id: Option<Option<String>>) -> Result<Vec<String>, String> {
    let selected = api_key_id.unwrap_or_else(|| binding_id().or_else(first_key_id));
    if let Some(id) = selected.as_deref() {
        access_for_id(id).map_err(|error| error.message)?;
    }
    let picked = normalize(list);
    let value: Vec<String> = if picked.is_empty() { default_models() } else { picked };
    config::update_raw_field(
        CONFIG_KEY,
        json!({ "models": value, "apiKeyId": selected }),
    );
    logging::log(
        "[Checkin]",
        &format!("WorkBuddy 国际版保活模型链已改为 {}", value.join(" → ")),
    );
    Ok(value)
}

/// 1 **available** Key（ use ）。
///
/// and 「 Key」 1 （`api_keys::list()` = create），
/// disabled / failure 。Environment variable key in after ： no has 、 no has 
/// ， use Key when use that 。
fn first_key_id() -> Option<String> {
    use crate::server::core::api_keys;
    if let Some(entry) = api_keys::list().into_iter().find(api_keys::ApiKeyEntry::is_usable) {
        return Some(entry.id);
    }
    crate::server::config::parse::env_api_key_value().map(|_| "env".to_string())
}

pub fn binding_id() -> Option<String> {
    config::current().raw().get(CONFIG_KEY).and_then(|value| value.get("apiKeyId"))
        .and_then(Value::as_str).map(str::trim).filter(|id| !id.is_empty()).map(str::to_string)
}

/// Key： that ； no is 1 available Key。
///
/// （`state` / `authorized_models`） 1 this ，（`set_models`） in use 
/// no when to —— is 「 」 and 「 use 」 1 。
pub fn effective_key_id() -> Option<String> {
    binding_id().or_else(first_key_id)
}

fn access_for_id(id: &str) -> Result<(crate::server::core::key_scope::KeyScope, crate::server::request_stats::ApiKeyIdentity), crate::server::errors::GatewayError> {
    use crate::server::core::{api_keys, key_scope::KeyScope};
    use crate::server::errors::GatewayError;
    use crate::server::request_stats::ApiKeyIdentity;
    if id == "env" && config::parse::env_api_key_value().is_some() {
        return Ok((KeyScope::default(), ApiKeyIdentity { source: "env".to_string(), ..Default::default() }));
    }
    let entry = api_keys::list().into_iter().find(|entry| entry.id == id && entry.is_usable())
        .ok_or_else(|| GatewayError::with_status(401, "保活 API Key 已停用、删除或不可用，请重新选择"))?;
    Ok((KeyScope::from_entry(&entry), ApiKeyIdentity { id: entry.id, name: entry.name, source: "key".to_string() }))
}

pub fn authorized_models() -> Result<(Vec<String>, crate::server::request_stats::ApiKeyIdentity), crate::server::errors::GatewayError> {
    use crate::server::errors::GatewayError;
    let id = effective_key_id().ok_or_else(|| GatewayError::with_status(401, "日活保活会调用模型，请先在「网关 Key」页创建一把 Key（或设置环境变量 Key）"))?;
    let (scope, identity) = access_for_id(&id)?;
    if !scope.allows_provider("workbuddy-intl") {
        return Err(GatewayError::with_status(403, "保活 API Key 不允许 WorkBuddy 国际版"));
    }
    let models = models();
    if models.is_empty() && scope.restricts_models() {
        return Err(GatewayError::with_status(403, "受限 API Key 的保活模型链不能为空，请明确填写获准使用的模型"));
    }
    if models.iter().any(|model| !scope.allows_model(model)) {
        return Err(GatewayError::with_status(403, "保活模型链包含所选 API Key 未授权的模型"));
    }
    Ok((models, identity))
}

static REQUEST_STATS: std::sync::OnceLock<std::sync::Arc<crate::server::request_stats::RequestStats>> = std::sync::OnceLock::new();

pub fn install_stats(stats: std::sync::Arc<crate::server::request_stats::RequestStats>) {
    let _ = REQUEST_STATS.set(stats);
}

pub fn record_activity(
    account_id: &str,
    started_at: i64,
    payload: &Value,
    result: &Result<Value, crate::server::errors::GatewayError>,
    identity: crate::server::request_stats::ApiKeyIdentity,
) {
    use crate::server::request_stats::{NewRequestEntry, RequestSource};
    let Some(stats) = REQUEST_STATS.get() else { return };
    let body = result.as_ref().ok();
    let model = body.and_then(|value| value.pointer("/activity/pokeModel")).and_then(Value::as_str)
        .or_else(|| body.and_then(|value| value.pointer("/activity/webConversation/model")).and_then(Value::as_str)).unwrap_or("");
    let success = result.as_ref().ok().is_some_and(|value| {
        value.pointer("/activity/pokeSucceeded").and_then(Value::as_bool) == Some(true)
            || value.pointer("/activity/webConversation/succeeded").and_then(Value::as_bool) == Some(true)
    });
    let mut entry = NewRequestEntry::new(model, if success { 200 } else { 502 });
    entry.ts = Some(started_at);
    entry.id = format!("keepalive-{account_id}-{started_at}");
    entry.account_id = account_id.to_string();
    entry.provider = Some("workbuddy-intl".to_string());
    entry.client_model = payload.get("models").and_then(Value::as_array)
        .map(|models| models.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("、")).unwrap_or_default();
    entry.upstream_model = model.to_string();
    entry.duration_ms = logging::now_ms() - started_at;
    entry.is_test = true;
    entry.source = RequestSource { client: "checkin_keepalive".to_string(), api_key: identity, ..Default::default() };
    if !success {
        entry.status = result.as_ref().err().map(|error| i64::from(error.http_status().as_u16())).unwrap_or(502);
        entry.error = Some(result.as_ref().err().map(|error| error.message.clone()).unwrap_or_else(|| "日活模型保活未成功，详情见签到记录".to_string()));
    }
    stats.record(entry);
}

/// for （ to in `keepalive` and response use this 1 ）：
/// `models` is current，`defaultModels` to "empty this "。
pub fn state() -> Value {
    json!({
        "models": models(),
        "defaultModels": default_models(),
        // Key（ > 1 available Key）： that is use 
        "apiKeyId": effective_key_id(),
        // is is in （ can 「 use is default Key」）
        "apiKeyDefaulted": binding_id().is_none() && effective_key_id().is_some(),
        "authError": authorized_models().err().map(|error| error.message),
    })
}
