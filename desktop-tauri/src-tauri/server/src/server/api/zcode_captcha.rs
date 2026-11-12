//! 。request and need ，
//! ； by use after ， not 。
//! and request use up config。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::providers::zcode;
use crate::server::errors::management_error;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// not need need ；config when by currentenabled 。
async fn call_action(state: &ServerState, account_id: &str, action: &str, payload: &Value) -> Result<Value, (i32, String)> {
    let adapter = if account_id.is_empty() {
        zcode::region::Region::ALL.into_iter()
            .find_map(|region| crate::server::core::providers::adapter::adapter_by_id(region.provider_id()))
    } else {
        state.store().account_provider_of(account_id)
            .and_then(|provider| crate::server::core::providers::adapter::adapter_by_id(&provider))
    };
    let Some(adapter) = adapter else {
        return Err((
            400,
            "ZCode 的提供商插件没有安装或未启用，无法访问验证码令牌池".to_string(),
        ));
    };
    adapter
        .admin_action(state.store(), account_id, action, payload)
        .await
        .map_err(|error| (error.status_code, error.message))
}

/// up parameter、 and request need 。
async fn stats_json(state: &ServerState) -> Value {
    let (start_plan_accounts, account_id) = start_plan_accounts(state);
    let mut body = match call_action(state, account_id.as_deref().unwrap_or(""), "captcha.stats", &Value::Null).await {
        Ok(body) => body,
        // not in when to 1 **** empty（ not is 500）：
        // 「unavailable」， not will quota。
        Err((_status, message)) => json!({ "ready": 0, "required": false, "error": message }),
    };
    let ready = body.get("ready").and_then(Value::as_u64).unwrap_or(0);
    let required = body.get("required").and_then(Value::as_bool).unwrap_or(false);
    let demand_active = body.get("demandActive").and_then(Value::as_bool).unwrap_or(false);
    let pool = config::captcha_pool_settings();
    if let Some(object) = body.as_object_mut() {
        object.insert("target".to_string(), Value::from(pool.target as u64));
        object.insert(
            "startPlanAccounts".to_string(),
            Value::from(start_plan_accounts as u64),
        );
        object.insert("demandActive".to_string(), Value::Bool(demand_active));
        object.insert(
            "needsTokens".to_string(),
            Value::Bool(required && demand_active && start_plan_accounts > 0 && ready == 0),
        );
        // old client this ， not 。
        object.insert("pollMs".to_string(), Value::from(pool.poll_ms as u64));
        object.insert(
            "maxPerRound".to_string(),
            Value::from(pool.max_per_round as u64),
        );
        // onlyreturn，SDK need config。
        object.insert(
            "captchaAccountId".to_string(),
            account_id.map(Value::String).unwrap_or(Value::Null),
        );
    }
    body
}

/// enabled and ； when anyenabled ZCode 。
fn start_plan_accounts(state: &ServerState) -> (usize, Option<String>) {
    let list = state.store().list_accounts();
    let Some(accounts) = list.get("accounts").and_then(Value::as_array) else {
        return (0, None);
    };
    let mut start_plan = 0usize;
    let mut start_plan_id: Option<String> = None;
    let mut fallback_id: Option<String> = None;
    for account in accounts {
        let provider = account.get("provider").and_then(Value::as_str).unwrap_or("");
        if zcode::region::Region::from_provider_id(provider).is_none() {
            continue;
        }
        let id = account.get("id").and_then(Value::as_str).unwrap_or("");
        if account.get("enabled").and_then(Value::as_bool) == Some(false) || id.is_empty() {
            continue;
        }
        if fallback_id.is_none() {
            fallback_id = Some(id.to_string());
        }
        if zcode::plan_of(account) == zcode::PLAN_START {
            start_plan += 1;
            if start_plan_id.is_none() {
                start_plan_id = Some(id.to_string());
            }
        }
    }
    (start_plan, start_plan_id.or(fallback_id))
}

/// GET /api/zcode/captcha —— （ / / / is this ）
pub async fn get_captcha(State(state): State<ServerState>) -> Response {
    ok_json(stats_json(&state).await)
}

/// POST /api/zcode/captcha —— body `{tokens: [{param, region}]}`；。
pub async fn push_captcha(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return management_error(400, "请求体必须是 JSON 对象");
    };
    // 1 1 （ up that ）
    let items: Vec<Value> = match object.get("tokens") {
        Some(Value::Array(items)) => items.clone(),
        Some(_) => return management_error(400, "tokens 必须是数组"),
        None => vec![payload.clone()],
    };
    // （param empty） in ： not ，
    // 「 has 1 need 」。 `accepted`， need not need 400。
    let accepted = match call_action(&state, "", "captcha.push", &json!({ "tokens": items })).await {
        Ok(outcome) => outcome
            .get("accepted")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        Err((status_code, message)) => return management_error(status_code, message),
    };
    if accepted == 0 {
        return management_error(400, "没有可入池的令牌（令牌为空或已提交）");
    }
    logging::console_line(
        "[ZCode]",
        &format!(
            "🔐 人机验证令牌入池 {} 个（目标 {}）",
            accepted,
            config::captcha_pool_settings().target,
        ),
    );
    let mut response = stats_json(&state).await;
    if let Some(object) = response.as_object_mut() {
        object.insert("accepted".to_string(), Value::from(accepted));
    }
    ok_json(response)
}
