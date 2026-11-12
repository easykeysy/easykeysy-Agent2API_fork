//! WorkBuddy **** （ / / / ）。
//!
//! ```text
//! GET /api/accounts/{id}/growth （）： + can +
//! + 
//! POST /api/accounts/{id}/growth/task ** ** { code, mode: full|claim }
//! POST /api/accounts/{id}/growth/school + optional { only?, lottery? }
//! POST /api/accounts/{id}/growth/travel 1 
//! ```
//!
//! ── this ────────────────────────────────────────────
//! in ****（`workbuddy::admin`）： / / 
//! is WorkBuddy has ， and 「client up 」 is 1 
//! —— by need ， this not in 。 this down 
//! HTTP （、、）， and return JSON by and 
//! `admin.action` 。
//!
//! ── Why is 「」 not is 「 1 」─────────────────
//! （14+ ）must（ ≥1s），
//! total when can 1 minutes —— API has 60 secondsrequest when 
//! （`gateway::REQUEST_TIMEOUT_MS`）， 1 request will in and not 。
//! this ****： 「 1 」 in use 
//! （request is seconds）， can 、 not total when ——
//! and to 「 1 request」 not ， that is as to 。
//!
//! ── WorkBuddy ─────────────────────────────────
//! in （`supports_growth` provider id）： no has CN
//! in ，request will to 400 and can Reason —— and to path「、
//! not 」 1 。
//!
//! ── use ────────────────────────────────────────
//! and （default dry-run、`--yes` ） not ： this by use 
//! in up ，。** and **：
//! （`GET`）、 by is （`POST`）， not to 「 1 」 will 。

use serde_json::Value;

use axum::body::Bytes;
use axum::response::Response;

use crate::server::errors::management_error;
use crate::server::http::ok_json;
use crate::server::ServerState;

/// request（ / JSON 1 to empty for ）。
fn parse_body(body: &Bytes) -> Value {
    serde_json::from_slice(body).unwrap_or(Value::Null)
}

/// （ not to 404）。
fn account_of(state: &ServerState, account_id: &str) -> Result<Value, Response> {
    let accounts = state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    accounts
        .into_iter()
        .find(|account| account.get("id").and_then(Value::as_str) == Some(account_id))
        .ok_or_else(|| management_error(404, "账号不存在"))
}

/// 1 （ + use ）。
///
/// in ：`admin.action` will to ** this provider **，
/// id ， not to （500/401）， use to is 1 
/// none error。 this by ， RPC。
async fn call_action(
    state: &ServerState,
    account_id: &str,
    action: &str,
    payload: &Value,
) -> Result<Value, Response> {
    let account = account_of(state, account_id)?;
    let provider = account
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if !crate::server::core::plugin_host::serves_provider(&provider) {
        return Err(management_error(
            400,
            "该账号的提供商没有安装插件，无法执行成长任务",
        ));
    }
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
        return Err(management_error(400, "该账号的提供商未注册"));
    };
    let _ = account;
    adapter
        .admin_action(state.store(), account_id, action, payload)
        .await
        .map_err(|error| management_error(error.status_code, error.message))
}

/// `GET /api/accounts/{id}/growth` —— （， not request）。
///
/// return：（ and supports）、 can /、
/// 、（）。： 1 failure
/// in this to `error` ， not its （ use need to 「」）。
pub async fn status(state: &ServerState, account_id: &str) -> Response {
    match call_action(state, account_id, "growth.status", &Value::Null).await {
        Ok(data) => ok_json(data),
        Err(response) => response,
    }
}

/// `POST /api/accounts/{id}/growth/task` —— ** **。
///
/// `mode=full`（）：accept（ if ）→ up → → claim；
/// `mode=claim`：（ this already / already when claim， etc ；）。
///
/// return（code / outcome / message / credit / energy）。 
/// 「 1 」 in use （module header 60 seconds when ）。
pub async fn run_task(state: &ServerState, account_id: &str, body: &Bytes) -> Response {
    let payload = parse_body(body);
    match call_action(state, account_id, "growth.task", &payload).await {
        Ok(data) => ok_json(data),
        Err(response) => response,
    }
}

/// `POST /api/accounts/{id}/growth/school` —— + optional。
///
/// `only` ；`lottery=true` when after empty（limit 20 ）。
pub async fn school_run(state: &ServerState, account_id: &str, body: &Bytes) -> Response {
    let payload = parse_body(body);
    match call_action(state, account_id, "growth.school", &payload).await {
        Ok(data) => ok_json(data),
        Err(response) => response,
    }
}

/// `POST /api/accounts/{id}/growth/travel` —— 1 。
///
/// 1 1 （none → ；idle → ；arrived → ；traveling → ）。
pub async fn travel_run(state: &ServerState, account_id: &str) -> Response {
    match call_action(state, account_id, "growth.travel", &Value::Null).await {
        Ok(data) => ok_json(data),
        Err(response) => response,
    }
}

/// `api::checkin_center` 「 this has no has 」 use ：
/// has WorkBuddy has 。
pub fn supports_growth(account: &Value) -> bool {
    account.get("provider").and_then(Value::as_str) == Some("workbuddy")
        && !account
            .get("enterpriseId")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty())
}
