//! API（）：
//!
//! - `GET /api/models/manage` → `{models, mappings, reasoningLevels}`（ use and ）
//! - `POST /api/models/state` `{id, enabled?}` （`hidden` already ， 400）
//! - `POST /api/models/mappings` `{alias, target, reasoning?, enabled?}` add / etc / 
//! - `POST /api/models/mappings/remove` `{alias, target, provider?}` delete
//! - `POST /api/models/custom` `{provider, id}` 1 up directory no has 
//! - `POST /api/models/custom/remove` `{provider, id}` this 
//! - `POST /api/models/capabilities` `{provider, id, capabilities}` for down can 
//!
//! return new `{models, mappings, reasoningLevels}`， before 、 not 1 。
//!
//! `reasoning` is 「 OmniProxy etc 」（
//! `model_rules::REASONING_LEVELS`， manage response 1 and to before ， before not 
//! 1 ）。** already **： etc in that ， by that 
//! up （ then and 「 not 」
//! `model_rules::reasoning` module header）。 and none —— is 
//! `ModelRules` 1 ， this 1 not use 。
//!
//! `enabled`（） and `reasoning` use 1 、 1 ：
//! request** no ** this = not ； bool = / 。 when 
//! (alias, target, provider, enabled) ， not `reasoning` ——
//! ， not （ `model_rules::add_mapping`）。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::Value;

use crate::server::core::capability;
use crate::server::core::model_rules;
use crate::server::core::providers::catalog;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

fn body_object(body: &Bytes) -> Result<serde_json::Map<String, Value>, Response> {
    let payload = parse_body(body).map_err(|error| errors::management_error(400, error.message))?;
    payload
        .as_object()
        .cloned()
        .ok_or_else(|| errors::management_error(400, "请求体必须是 JSON 对象"))
}

fn text_field(object: &serde_json::Map<String, Value>, key: &str) -> String {
    object
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

/// up id is in current（ignore）
/// GET /api/models/manage
pub async fn get_manage(State(state): State<ServerState>) -> Response {
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/state
pub async fn set_state(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let id = text_field(&object, "id");
    if id.is_empty() {
        return errors::management_error(400, "缺少模型 id");
    }
    // `hidden` already 「delete/」（ ）：
    // old before / this when ， 400 not is ignore ——
    // ignore will use as 「delete」success， its 。
    if object.contains_key("hidden") {
        return errors::management_error(400, "hidden 已移除，请使用 enabled 开关");
    }
    let enabled = match object.get("enabled") {
        None => return errors::management_error(400, "缺少 enabled 字段"),
        Some(Value::Bool(value)) => Some(*value),
        Some(_) => return errors::management_error(400, "enabled 必须是布尔值"),
    };
    // ： new before total is （ is 「 × id」）；
    // old —— has old before （ before ） will this 
    let provider = text_field(&object, "provider");
    let provider_opt = if provider.is_empty() { None } else { Some(provider.as_str()) };
    if let Some(provider) = provider_opt {
        if !crate::server::core::providers::is_known_provider_id(provider) {
            return errors::management_error(400, format!("未知的提供商: {provider}"));
        }
    }
    if let Err(error) = model_rules::set_state(provider_opt, &id, enabled) {
        return errors::management_error(error.status_code, error.message);
    }
    // up ： in when in when ， id not is 
    let subject = match provider_opt {
        Some(name) => format!("[{name}] {id}"),
        None => id.clone(),
    };
    let what = if enabled == Some(true) { "已启用" } else { "已禁用" };
    logging::log("[Models]", &format!("模型 {subject} {what}"));
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/mappings
///
/// 1 for ID enabled 1 ；。
/// `create=true` add，`create=false` ， old use 。
///
/// ── `reasoning` （ and `provider` not ，）──────
/// before in ** already has etc ** when is this （ = in 1 ），
/// 「request has no has this 」must can ：
/// - **** → `None`， not already has etc （ old before and use as ）；
/// - to empty/null → `Some(None)`，「 not 」；
/// - to → `Some(Some(x))`， x（，
/// by `model_rules::normalize_reasoning` ）。
/// 1 by 「empty = empty」 will old before （ not this ） can can in 
/// —— that is 。
pub async fn add_mapping(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let alias = text_field(&object, "alias");
    let target = text_field(&object, "target");
    let provider = text_field(&object, "provider");
    // ** not in this **： this when 「 new 1 for 」 and 
    // 「 / etc 」， after is 、 can can is up 
    // ASCII id（ `` / ``）。 in `model_rules::add_mapping`
    // by 「 this is not is new 」， new when use is `alias_name_valid`
    // （ not ， that ）。
    if target.is_empty() {
        return errors::management_error(400, "缺少目标上游模型");
    }
    let provider_opt = if provider.is_empty() { None } else { Some(provider.as_str()) };
    if let Some(kind) = provider_opt {
        // provider must is ：， will 
        // 1 by not to 
        if !crate::server::core::providers::is_known_provider_id(kind) {
            return errors::management_error(400, format!("未知的提供商: {kind}"));
        }
    }
    // etc ； in this （ before ， not 1 
    // will `normalize_reasoning` ）
    let reasoning = match object.get("reasoning") {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.is_empty() {
                Some(None)
            } else if model_rules::normalize_reasoning(text).is_none() {
                return errors::management_error(400, "思考等级过长（最多 32 个字符）");
            } else {
                Some(Some(text))
            }
        }
        Some(_) => return errors::management_error(400, "reasoning 必须是字符串或 null"),
    };
    // / `model_rules::add_mapping`： = not 
    // （ new default）； bool = / 。 bool in this ， not 1 
    // will 。
    let enabled = match object.get("enabled") {
        None => None,
        Some(Value::Bool(value)) => Some(*value),
        Some(_) => return errors::management_error(400, "enabled 必须是布尔值"),
    };
    let create = match object.get("create") {
        None => None,
        Some(Value::Bool(value)) => Some(*value),
        Some(_) => return errors::management_error(400, "create 必须是布尔值"),
    };
    if let Err(error) = model_rules::add_mapping(&alias, &target, provider_opt, reasoning, enabled, create) {
        return errors::management_error(error.status_code, error.message);
    }
    let subject = match provider_opt {
        Some(name) => format!("{alias} → {target}（{name}）"),
        None => format!("{alias} → {target}"),
    };
    // etc and 1 and （ etc / is this ，
    // not not ）
    let reasoning_text = match reasoning.flatten() {
        Some(level) => format!("，思考等级 {level}"),
        None => String::new(),
    };
    let enabled_text = match enabled {
        Some(true) => "，开关 开",
        Some(false) => "，开关 关",
        None => "",
    };
    logging::log("[Models]", &format!("保存映射 {subject}{reasoning_text}{enabled_text}"));
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/mappings/remove
///
/// delete 1 。 after by alias will has ， by 
/// `{alias, target, provider?}` （ and add 1 ）。
///
/// 1 delete old when ， its and etc not 。
pub async fn remove_mapping(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let alias = text_field(&object, "alias");
    let target = text_field(&object, "target");
    let provider = text_field(&object, "provider");
    if alias.is_empty() || target.is_empty() {
        return errors::management_error(400, "缺少映射名或目标上游模型");
    }
    let provider_opt = if provider.is_empty() { None } else { Some(provider.as_str()) };
    // （default）** not can ** —— in ， need disabled 。
    // 1 is ** this already not in **（ up down 、 that or no has available
    // ）： this when no has ，「」none from ，delete is 1 —— 
    // 「 」 is to this 。 and `dangling`
    // that 1 （`catalog::host_row_exists`），「 as 、
    // delete 」 。
    if alias.eq_ignore_ascii_case(&target)
        && catalog::host_row_exists(state.store(), provider_opt, &target)
    {
        return errors::management_error(400, "原始 ID 的默认绑定不能删除，请关闭它的开关");
    }
    let removed = match model_rules::remove_mapping(&alias, &target, provider_opt) {
        Ok((_, removed)) => removed,
        Err(error) => return errors::management_error(error.status_code, error.message),
    };
    if !removed {
        return errors::management_error(404, format!("映射不存在: {alias} → {target}"));
    }
    logging::log("[Models]", &format!("删除映射 {alias} → {target}"));
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/custom
///
/// 1 up ：`{provider, id}`。
///
/// ── ────────────────────────────────────────────
/// up directory no 、 can by （ in new 、 by down no 
/// directory ）。 before not 、 not —— use no has 。
///
/// ── Why this ────────────────────────────────────────
/// 1. **provider must already **： this 1 。
/// （ or 1 not in ） after is 「success、 no has 、 not 」，
/// and `add_mapping` unknown provider 1 by 。
/// 2. **id use `alias_valid`**： that （ and `- _ . / :`，≤128）
/// is up id （ Cline `pool/model` and 
/// Qoder ）。 1 will 。
/// 3. ****：CatPaw ID has （ up by 、
/// not and can ， `catpaw::models`）， this 
/// will to 1 「 can not 」 。 this this CatPaw 
/// directory， down has use 。
pub async fn add_custom(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let provider = text_field(&object, "provider");
    let id = text_field(&object, "id");
    if provider.is_empty() {
        return errors::management_error(400, "缺少提供商");
    }
    // already （ `providers::is_known_provider_id` ）：
    // will already in then 1 to 「unknown 」。
    if !crate::server::core::providers::is_known_provider_id(&provider) {
        return errors::management_error(400, format!("未知的提供商: {provider}"));
    }
    if id.is_empty() {
        return errors::management_error(400, "缺少上游模型 ID");
    }
    if !model_rules::alias_valid(&id) {
        return errors::management_error(
            400,
            "模型 ID 只能包含字母、数字与 - _ . / :，且不超过 128 个字符",
        );
    }
    if id.chars().all(|ch| ch.is_ascii_digit()) {
        return errors::management_error(
            400,
            "纯数字 ID 只对 CatPaw 有意义，且需要该家目录里的档位信息；请改用它自己的模型清单",
        );
    }
    if let Err(error) = model_rules::add_custom(&provider, &id) {
        return errors::management_error(error.status_code, error.message);
    }
    logging::log("[Models]", &format!("登记自定义模型 [{provider}] {id}"));
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/custom/remove
///
/// 1 ：`{provider, id}`。
///
/// ── Why is 「」 not is 「」────────────────────────────
/// in by `modelRules.custom` ， no has 「 up new will 
/// 」 this 。 in （ is not request）， after 
/// will —— use is 「 this not need 」。 is that ，
/// and after `/v1/models`、、 by when 。
/// for then （ `model_rules::remove_custom`）。
pub async fn remove_custom(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let provider = text_field(&object, "provider");
    let id = text_field(&object, "id");
    if provider.is_empty() || id.is_empty() {
        return errors::management_error(400, "缺少提供商或上游模型 ID");
    }
    let removed = match model_rules::remove_custom(&provider, &id) {
        Ok((_, removed)) => removed,
        Err(error) => return errors::management_error(error.status_code, error.message),
    };
    if !removed {
        return errors::management_error(
            404,
            format!("自定义模型不存在: [{provider}] {id}"),
        );
    }
    logging::log("[Models]", &format!("移除自定义模型 [{provider}] {id}"));
    ok_json(catalog::manage_view(state.store()))
}

/// POST /api/models/capabilities
///
/// ** for down ** can ：`{provider, id, capabilities: {...}}`。
/// down （client） by this request（ not 、 by ），
/// up directory to can can is —— this is that 。
/// （`/v1/models` and Anthropic 、）， not by / / as 。
///
/// ── Why 1 ──────────────────────────────────────
/// is `(provider, id)` （ is ， is ；
/// is ）。 1 has will that 。
///
/// ── `capabilities` （ and `add_mapping` reasoning 1 ）──
/// · **** → not this 1 ；
/// · to `null` → this 1 （ to ）；
/// · to → （token need 1~1 ， can need ）。
/// when 1 to （「」 not ）；
/// to 1 pathsupports， to after 。
///
/// ── ────────────────────────────────────────────
/// can in **** `models[].capabilities` 
/// （ and that / etc 1 ）， is 
/// `POST /api/custom-providers/models`。 this for `custom-` id 400 not is 
/// modelRules —— after will to 1 ** not ** （ 
/// not modelRules）， is 1 。
pub async fn set_capabilities(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let provider = text_field(&object, "provider");
    let id = text_field(&object, "id");
    if provider.is_empty() || id.is_empty() {
        return errors::management_error(400, "缺少提供商或模型 id");
    }
    if !crate::server::core::providers::is_known_provider_id(&provider) {
        return errors::management_error(
            400,
            format!("未知的提供商: {provider}（自定义提供商的能力走整表保存接口）"),
        );
    }
    let patch = match object.get("capabilities") {
        None => return errors::management_error(400, "缺少 capabilities 字段"),
        Some(Value::Object(map)) => map.clone(),
        Some(_) => return errors::management_error(400, "capabilities 必须是对象"),
    };
    // in this （ and add_mapping reasoning 1 to ： not 1 
    // will 1 ）；`null` is 「」， not 。
    for (key, value) in &patch {
        if !capability::KEYS.contains(&key.as_str()) {
            return errors::management_error(400, format!("未知的能力字段: {key}"));
        }
        if !value.is_null() && capability::normalize_value(key, value).is_none() {
            return errors::management_error(
                400,
                format!("能力字段 {key} 取值非法（token 键要 1~1 亿的正整数，能力开关要布尔）"),
            );
        }
    }
    if let Err(error) = model_rules::set_capabilities(&provider, &id, &patch) {
        return errors::management_error(error.status_code, error.message);
    }
    // 1 「= / =」，otherwise when to 「 can 」
    // this 1 ， not 1 、
    let summary = capability::KEYS
        .iter()
        .filter_map(|key| patch.get(*key).map(|value| (key, value)))
        .map(|(key, value)| match value {
            Value::Null => format!("{key}=恢复"),
            other => format!("{key}={other}"),
        })
        .collect::<Vec<_>>()
        .join(", ");
    logging::log("[Models]", &format!("保存模型能力 [{provider}] {id}: {summary}"));
    ok_json(catalog::manage_view(state.store()))
}
