//! API（set「」）：
//!
//! - `GET /api/custom-providers` → `{providers: [...]}`（ by createdAt ）
//! - `POST /api/custom-providers` → new ，** when **create this 1 
//! - `POST /api/custom-providers/update`   → `{id, name?, protocol?, baseUrl?, enabled?}`
//! - `POST /api/custom-providers/remove` → `{id}`，delete down ，
//! and this id from Key available
//! - `POST /api/custom-providers/models` → `{providerId, models, mappings}`，
//! - `POST /api/custom-providers/fetch-models` → `{providerId}`，server up 
//!
//! 「HTTP ↔ core use 」 （core and 
//! `core::custom_providers` and `account_store::custom_accounts` module header）。
//! protected（ and /api/models/manage ：config、，
//! fetch-models will ** up **）。
//!
//! ── Why「 new 」 need 1 up 1 ────────────────────────
//! use 1 total is 「 need this up 」，（apiKey）
//! in 1 ； request will 「 no 」 as in 
//! （ not 、unavailable， up is 1 empty）。 1 ，failure when 
//! （ `create_custom_provider` ）， not 。
//!
//! ── （fetch → confirm → save）─────────────────
//! 「get」 up ** not **（core `fetch_upstream_models` is 
//! up ）， before and use ； use /models 
//! 。 no has server —— ， new ， not in 
//! 「 no 」 empty。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::api_keys;
use crate::server::core::custom_providers;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// request（empty body as `{}`）； for 1 400。
///
/// and `model_manage::body_object` 1 —— no has use is ： that is 
/// this has ， as 1 use 。
fn body_object(body: &Bytes) -> Result<serde_json::Map<String, Value>, Response> {
    let payload = parse_body(body).map_err(|error| errors::management_error(400, error.message))?;
    payload
        .as_object()
        .cloned()
        .ok_or_else(|| errors::management_error(400, "请求体必须是 JSON 对象"))
}

// ─── GET /api/custom-providers ──────────────────────────────

/// （ by createdAt ， in ）。
pub async fn get_custom_providers(State(_state): State<ServerState>) -> Response {
    ok_json(json!({ "providers": custom_providers::list() }))
}

// ─── POST /api/custom-providers ─────────────────────────────

/// new + this 1 。
///
/// body：`{name, protocol, baseUrl, apiKey?}`（`apiKey` ，
/// `name` when as default —— up and is ：
/// 1 not ， use can after in up ）。
///
/// ── ────────────────────────────────────────
/// （ `provider` need to id），
/// can can failure（unavailable / baseUrl ）。 when **must **
/// returnerror —— otherwise use when will in to 1 empty， and not in this 
/// request response，「 not in 」 。 not can can 
/// （ no ）， use 1 `remove` can （ to 0 ）。
pub async fn create_custom_provider(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let payload = Value::Object(object);
    let provider = match custom_providers::create(&payload) {
        Ok(provider) => provider,
        Err(message) => return errors::management_error(400, message),
    };
    let provider_id = provider
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let name = provider
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let store = state.store();
    // ** not baseUrl**： up `baseUrl` is ****，
    // when （ `custom_accounts` module header）。 this 1 
    // request 1 to up ， will 「 after 」 old 
    // —— new 、request old ， and that up not to 
    // not （issue #455 is this ）。 not this = 。
    let mut account_payload = payload.as_object().cloned().unwrap_or_default();
    account_payload.remove("baseUrl");
    let account_payload = Value::Object(account_payload);
    let upsert = match store.add_custom_account(&provider_id, &account_payload, name.as_deref()) {
        Ok(upsert) => upsert,
        Err(error) => {
            // （ down no has ，delete as 0）；
            // failure not —— that down will has create and config
            // can ， will before to error 1 。
            if let Err(rollback) = custom_providers::remove(&provider_id, store) {
                logging::log(
                    "[CustomProvider]",
                    &format!("❌ 回滚新建的提供商 {provider_id} 失败: {rollback}"),
                );
            }
            return crate::server::api::accounts::store_error(error);
        }
    };
    let label = provider
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&provider_id);
    logging::log(
        "[CustomProvider]",
        &format!("✅ 新建自定义提供商「{label}」（{provider_id}），并添加了首个账号"),
    );
    ok_json(json!({
        "provider": provider,
        "account": upsert.account,
        // down not can can already has ， this as true； up is as 
        // path response 1 （ before by 1 ）
        "created": upsert.created,
        "list": store.list_accounts(),
    }))
}

// ─── POST /api/custom-providers/update ──────────────────────

/// update（ not ； in core ， protocol / baseUrl new ）。
///
/// responseupdate after ， not —— not 
/// （ `baseUrl` is ， when **** to ，
/// not need need in when ）。
///
/// ** 1 is Base URL**： new when will when 
/// （ as ， `create_custom_provider` ）， is that old 
/// will 1 new —— `/v1/responses`、request
/// `/responses`（issue #455）。 this in when ， down ** etc old ** 
/// ， to 「」。 etc ： 
/// can can to 1 ， that is has 。
pub async fn update_custom_provider(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return errors::management_error(400, "缺少提供商 id");
    }
    // old used for：「 to 」（enabled core 
    // ， can to this 1 is ）
    let before = custom_providers::get(&id);
    let updated = match custom_providers::update(&id, &Value::Object(object)) {
        Ok(updated) => updated,
        // not in （404） and failure（400） in this —— core error
        // already is to use 1 ， by is 「 not in 」 can 
        Err(message) if before.is_none() => return errors::management_error(404, message),
        Err(message) => return errors::management_error(400, message),
    };
    // → that 「 old 」 （ ）。
    // failure not （ already ）， 1 ： 
    // old can from set「」。
    let before_base = before
        .as_ref()
        .and_then(|item| item.get("baseUrl"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    let after_base = updated
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_end_matches('/')
        .to_string();
    if !before_base.is_empty() && before_base != after_base {
        match state.store().clear_custom_base_url_overrides(&id, &before_base) {
            Ok(names) if !names.is_empty() => logging::log(
                "[CustomProvider]",
                &format!(
                    "🔗 已把 {} 个账号的旧地址覆盖项改回「跟随提供商」: {}",
                    names.len(),
                    names.join("、")
                ),
            ),
            Ok(_) => {}
            Err(error) => logging::log(
                "[CustomProvider]",
                &format!("⚠️ 清理账号旧地址覆盖项失败（可手动「改回跟随提供商」）: {}", error.message),
            ),
        }
    }
    let changes = describe_changes(before.as_ref(), &updated);
    let label = updated
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&id);
    logging::log(
        "[CustomProvider]",
        &format!("✏️  自定义提供商「{label}」已更新（{changes}）"),
    );
    ok_json(json!({ "provider": updated }))
}

/// **** （ to use ， not —— apiKey not in this ，
/// this can ）
fn describe_changes(before: Option<&Value>, after: &Value) -> String {
    const FIELDS: &[&str] = &["name", "protocol", "baseUrl", "enabled", "balanceScheme"];
    let mut changed: Vec<&str> = Vec::new();
    for field in FIELDS {
        let old = before.and_then(|item| item.get(*field));
        let new = after.get(*field);
        if old != new {
            changed.push(field);
        }
    }
    if changed.is_empty() {
        "无变化".to_string()
    } else {
        changed.join("、")
    }
}

// ─── POST /api/custom-providers/remove ──────────────────────

/// delete and **delete down **（ `custom_providers::remove`），
/// when this id from Key 「available」（ `api_keys::
/// strip_provider_from_allowlists`）。
///
/// response `{removed, accountsRemoved, keysUpdated, keysUnrestricted, list}`：
/// `removed` is （ in 1），`accountsRemoved` is 
/// —— 「 already delete N 」，`list` before ， not 1 。
/// after is ：`keysUpdated` is 「available to this 」、
/// Key ；`keysUnrestricted` is its in ** already noneavailable**、 is 
/// 「unlimited」 Key 。must after —— that is this Key from 「 not 」
/// 「 can 」， not can in （ core that ）。
pub async fn remove_custom_provider(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return errors::management_error(400, "缺少提供商 id");
    }
    // delete before 1 to （ after not to ）； use 1 
    // 404 —— not core error（ that is ，
    // ），「 in and 」 in this 1 can 
    if custom_providers::get(&id).is_none() {
        return errors::management_error(404, format!("自定义提供商不存在: {id}"));
    }
    let label = custom_providers::label_of(&id);
    let store = state.store();
    let accounts_removed = match custom_providers::remove(&id, store) {
        Ok(count) => count,
        Err(message) => return errors::management_error(400, message),
    };
    // in ** after **： （、） 1 failure，
    // down 「limit already 、 in 」 —— that is ， id 。
    // this 1 failure not response（ already ，），failure after is that Key 
    // 「 not 」， use 1 available can —— and before 1 。
    let stripped = api_keys::strip_provider_from_allowlists(&id);
    let unrestricted = stripped
        .iter()
        .filter(|entry| entry.allowed_providers.is_empty())
        .count();
    let mut note = format!("，连带清除 {accounts_removed} 个账号");
    if !stripped.is_empty() {
        // use `name || id` ：Key is optional （empty），empty will 1 
        // 「（）」；id 1 can to that Key
        let names: Vec<String> = stripped
            .iter()
            .map(|entry| {
                let name = entry.name.trim();
                if name.is_empty() { entry.id.clone() } else { format!("{}（{}）", name, entry.id) }
            })
            .collect();
        note.push_str(&format!(
            "；并从 {} 把 Key 的可用提供商里摘掉这家（{}）",
            stripped.len(),
            names.join("、"),
        ));
        if unrestricted > 0 {
            note.push_str(&format!(
                "，其中 {unrestricted} 把已无其它可用提供商，恢复为不限制",
            ));
        }
    }
    logging::log(
        "[CustomProvider]",
        &format!(
            "🗑️  自定义提供商「{}」（{id}）已删除{note}",
            label.as_deref().unwrap_or(&id),
        ),
    );
    ok_json(json!({
        "removed": 1,
        "accountsRemoved": accounts_removed,
        "keysUpdated": stripped.len(),
        "keysUnrestricted": unrestricted,
        "list": store.list_accounts(),
    }))
}

// ─── POST /api/custom-providers/models ──────────────────────

/// this and （`custom_providers::set_models` HTTP ）。
///
/// body：`{providerId, models, mappings}` —— is ****
/// （UI 「 → 」 ， core ）。
/// response `{provider: update after }`： before this ，
/// not 1 。
pub async fn set_custom_models(State(_state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let provider_id = object
        .get("providerId")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if provider_id.is_empty() {
        return errors::management_error(400, "缺少提供商 id");
    }
    let label = custom_providers::label_of(&provider_id);
    let updated = match custom_providers::set_models(
        &provider_id,
        object.get("models").cloned().unwrap_or(Value::Null),
        object.get("mappings").cloned().unwrap_or(Value::Null),
    ) {
        Ok(updated) => updated,
        Err(error) => return errors::management_error(error.status_code, error.message),
    };
    let model_count = updated
        .get("models")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let mapping_count = updated
        .get("mappings")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    logging::log(
        "[CustomProvider]",
        &format!(
            "✅ 自定义提供商「{}」已保存模型清单（{model_count} 个模型、{mapping_count} 条映射）",
            label.as_deref().unwrap_or(&provider_id),
        ),
    );
    ok_json(json!({ "provider": updated }))
}

// ─── POST /api/custom-providers/fetch-models ────────────────

/// server up （`custom_providers::fetch_upstream_models`）。
///
/// body：`{providerId}`。response `{models: ["id", ...]}` —— ** not **，
/// by before and use after /models （ by module header）。
/// will ** up **（ 1 GET）， protected and failure up response。
pub async fn fetch_custom_models(State(state): State<ServerState>, body: Bytes) -> Response {
    let object = match body_object(&body) {
        Ok(object) => object,
        Err(response) => return response,
    };
    let provider_id = object
        .get("providerId")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if provider_id.is_empty() {
        return errors::management_error(400, "缺少提供商 id");
    }
    if custom_providers::get(&provider_id).is_none() {
        return errors::management_error(404, format!("自定义提供商不存在: {provider_id}"));
    }
    let label = custom_providers::label_of(&provider_id);
    match custom_providers::fetch_upstream_models(&provider_id, state.store()).await {
        Ok(models) => {
            logging::log(
                "[CustomProvider]",
                &format!(
                    "✅ 拉取模型清单「{}」（{provider_id}）成功，共 {} 个模型（未保存，待确认）",
                    label.as_deref().unwrap_or(&provider_id),
                    models.len(),
                ),
            );
            ok_json(json!({ "models": models }))
        }
        Err(message) => errors::management_error(502, message),
    }
}
