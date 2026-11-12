//! / to / by （ for server.mjs 871-911 ）。
//!
//! GET /api/usage （queryCreditsSummary）
//! GET /api/checkin/status to 
//! POST /api/checkin daily to （result.success ? 200 : 409）
//! POST /api/checkin/claim-and-report to + new 
//! GET /api/activity/banner banner
//! GET /api/activity/ambassador 
//! GET /api/checkin-keepalive （）
//! POST /api/checkin-keepalive （）
//!
//! ── response （ 1 ）──────────────────────────
//! ① successresponse is API `{success:true, data}`；
//! ② `POST /api/checkin` ****：success 200、 already 409，
//! and body `success` （ not is true）。 before 
//! 「 already to 」 1 warn not is error。
//! ③ / up failure when ，Node this in try → errorPayload
//! OpenAI body（`{error:{message,type,upstream_code?}}`），
//! ** not is ** API `{success:false,error}` 。 this use 
//!      `BillingError::to_gateway_error().into_response()`。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use crate::server::core::billing::{WorkbuddyActivity, BillingError};
use crate::server::core::providers::workbuddy::Region;
use crate::server::errors::GatewayError;
use crate::server::http::ok_json;
use crate::server::logging;
use crate::server::ServerState;

/// error → response。
///
/// OpenAI body（ by module header ③ ）， BillingError 
/// （504 when / 401 / up HTTP / 400 none to ）。
fn billing_error(error: BillingError) -> Response {
    logging::log("[Billing]", &format!("❌ {}", error.message));
    error.to_gateway_error().into_response()
}

/// GET /api/usage —— 
///
/// Node is `billing.queryCreditsSummary({ locale: opts.locale })`，
/// locale config.json locale（default zh-CN）。
pub async fn get_usage(State(state): State<ServerState>) -> Response {
    match state.billing().query_credits_summary_default().await {
        Ok(data) => ok_json(data),
        Err(error) => billing_error(error),
    }
}

/// GET /api/checkin/status —— to （none/none when as null）
pub async fn checkin_status(State(state): State<ServerState>) -> Response {
    match state.billing().get_checkin_status(None).await {
        Ok(data) => ok_json(data),
        Err(error) => billing_error(error),
    }
}

/// POST /api/checkin —— daily to 
///
/// ****（server.mjs 885-889 ）：
///   `sendJson(res, result.success ? 200 : 409, { success: result.success, data: result })`
/// 「 already 」 this use will return 409 + `success:false`，body up 
/// msg（「 already to 」）—— before by warn ， not is error。
pub async fn claim_checkin(State(state): State<ServerState>) -> Response {
    let result = match state.billing().claim_daily_checkin(None).await {
        Ok(result) => result,
        Err(error) => return billing_error(error),
    };
    let success = result
        .get("success")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    if !success {
        let detail = result.get("msg").and_then(serde_json::Value::as_str).unwrap_or("");
        logging::log("[Billing]", &format!("签到未领取（{detail}）"));
    }
    // and body success by ， not can use ok_json（ as 200/true）
    let status = if success {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::CONFLICT
    };
    (status, axum::Json(json!({ "success": success, "data": result }))).into_response()
}

/// POST /api/checkin/claim-and-report —— to + 
pub async fn claim_and_report(State(state): State<ServerState>) -> Response {
    match state.billing().checkin_and_report_default().await {
        Ok(data) => ok_json(data),
        Err(error) => billing_error(error),
    }
}

/// GET /api/activity/banner —— banner（ not to when as null）
pub async fn activity_banner(State(state): State<ServerState>) -> Response {
    let data = state.billing().get_activity_banner(None).await;
    ok_json(data)
}

/// GET /api/activity/ambassador —— （ not to when as null）
pub async fn activity_ambassador(State(state): State<ServerState>) -> Response {
    let data = state.billing().get_ambassador_status(None).await;
    ok_json(data)
}

/// GET /api/checkin-keepalive —— WorkBuddy （）
pub async fn get_keepalive() -> Response {
    ok_json(crate::server::core::billing::keepalive::state())
}

/// POST /api/checkin-activity —— WorkBuddy ****。
///
/// body：`{id, mode}`，`mode ∈ full | claim | keepalive`（ full）。
/// and to （/api/accounts/checkin） by ： that for 
/// （ when and ）， by —— by 
/// ， and to （`{id, name, claim, activity}`）， before toast use 。
pub async fn run_activity(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match serde_json::from_slice::<Value>(&body) {
        Ok(value) => value,
        Err(error) => {
            return GatewayError::new(format!("请求体不是合法 JSON：{error}")).into_response()
        }
    };
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if id.is_empty() {
        return GatewayError::with_status(400, "缺少账号 id").into_response();
    }
    let mode = WorkbuddyActivity::parse(payload.get("mode").and_then(Value::as_str));

    // must in and WorkBuddy —— will 
    // will （ and checkin_for by is 1 by ）
    let account = state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|account| account.get("id").and_then(Value::as_str) == Some(id.as_str()))
        })
        .cloned();
    let Some(account) = account else {
        return GatewayError::with_status(404, "账号不存在").into_response();
    };
    if account.get("provider").and_then(Value::as_str) != Some(Region::Intl.provider_id()) {
        return GatewayError::with_status(400, "该账号不是 WorkBuddy 国际版账号").into_response();
    }
    // ： 1 （ and / request，
    // `store_view::display_name_of`），（ old ） by old name
    let name = account
        .get("displayName")
        .filter(|value| value.as_str().is_some_and(|text| !text.is_empty()))
        .cloned()
        .or_else(|| account.get("name").cloned())
        .unwrap_or(Value::Null);
    let Some(entry) = state.store().get_session_by_id(&id) else {
        return GatewayError::with_status(400, "没有可用凭证").into_response();
    };
    let activity = state
        .billing()
        .workbuddy_daily_activity(&entry.session, mode)
        .await;
    let row = json!({
        "id": id,
        "name": name,
        "mode": mode.as_str(),
        "claim": activity.get("claim").cloned().unwrap_or(Value::Null),
        "activity": activity.get("activity").cloned().unwrap_or(Value::Null),
    });
    // to ： this path not run_checkin（ no has 1 can ），
    // 1 （source=activity）—— in checkin_records::record_activity
    // （keepalive ， that ）。failure in ，
    // not response。
    crate::server::core::checkin_records::record_activity(
        &row,
        account.get("provider").and_then(Value::as_str).unwrap_or(""),
        mode.as_str(),
    );
    // `checkinAt`： and / when （`run_checkin`）must ——
    // is `day_completed`（success / up / will 1 ），otherwise
    // 1 after to in 「」， and already already/ok 。
    // failure， not response（ and run_checkin ）。
    if crate::server::core::billing::checkin::day_completed(&row)
        && !state
            .store()
            .mark_checkin(&id, crate::server::logging::now_ms())
    {
        crate::server::logging::verbose(
            "[Accounts]",
            &format!("账号 {id} 的签到时间未能落盘（账号可能已被删除）"),
        );
    }
    ok_json(row)
}

/// POST /api/checkin-keepalive —— 。
///
/// body：`{ models: string[] }`（ before by ； /
/// / empty ， after 1 ）。empty = 
/// （ `billing::keepalive::set_models`）。return after 
/// （`{models, defaultModels}`）， before keepalive 。
pub async fn save_keepalive(body: Bytes) -> Response {
    let payload = match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(value) => value,
        Err(error) => {
            return GatewayError::new(format!("请求体不是合法 JSON：{error}")).into_response()
        }
    };
    // ：（） and （）——
    // by / / empty， before 1 。
    let raw: Vec<String> = match payload.get("models") {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Some(serde_json::Value::String(text)) => text
            .split([',', '，', '、', ' ', '\t'])
            .map(str::to_string)
            .collect(),
        _ => return GatewayError::new("models 必须是字符串数组或分隔符字符串").into_response(),
    };
    let api_key_id = match payload.get("apiKeyId") {
        None => None,
        Some(Value::Null) => Some(None),
        Some(Value::String(id)) if id.trim().len() <= 128 => {
            let id = id.trim();
            Some((!id.is_empty()).then(|| id.to_string()))
        }
        Some(_) => return GatewayError::bad_request("apiKeyId 必须是 Key ID 或 null").into_response(),
    };
    match crate::server::core::billing::keepalive::set_models(&raw, api_key_id) {
        Ok(_) => ok_json(crate::server::core::billing::keepalive::state()),
        Err(message) => GatewayError::new(message).into_response(),
    }
}
