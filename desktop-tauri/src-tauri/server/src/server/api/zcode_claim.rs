//! ZCode when （ + ）。
//!
//! ── Whyfile（ not in `api::accounts` ）────────────────
//! `api::accounts` already is CRUD + （ 550 ），
//! this 1 ****（biz code 、404 is 、
//! by before ）， will that file when 「」 and 「」
//! 。 and `api::accounts_usage`（） 1 。
//!
//! ── ────────────────────────────────────────────────
//! ```text
//! POST /api/accounts/{id}/zcode-claim/preview can （** not need **）
//! POST /api/accounts/{id}/zcode-claim/captcha-config config（ need not need ）
//! POST /api/accounts/{id}/zcode-claim （** need **parameter）
//! ```
//! not is 「 1 」 by ： is etc ， can 、
//! can in up new ； is has use ， and must etc before 
//! can 。 use when and failure not 。
//!
//! ── this ────────────────────────────────────────────
//! in ****（`zcode::claim` / `zcode::activation`，
//! `claim.preview` / `claim.captcha-config` / `claim.claim`）：biz code 
//! 、、request， is 「 ZCode 」 
//! —— by need ， this not in 。 this 
//! 、`claimedAt` （ that is ） and 。
//!
//! ── 「 1 」 and 「 already 」 ──────────────────────
//! 2026-09-28 that （ZCode Trust Build） is ** 1 new **
//! （`plan_id` ）， this by 。file 
//! （`claimAt` / `claimPlanId`， `mark_zcode_claim`）：
//! - **success** need ；
//! - **`already_claimed` need ** —— up this 1 already 
//! （ can can is 1 ）， this 「 already 」，
//! otherwise use will 1 that by 、 1 「 this already 」。
//! response up after ， before not as new that by 1 。
//!
//! ── when and ──────────────────────────────────────
//! by ； when by by need ，
//! `zcode_claim_verification` ， 1 and to 。
//! path not use use ， no has or when need need 。
//!
//! ── ： up ** before ** ─────────────────────────
//! by up （`app_launch` / `app_daily_active`）， not 
//! client、 that 1 not will 。 will （ 
//! `claim.preview` / `claim.claim` ）：
//! · `claim`： before （failure not ）；
//! · `preview`：** as empty when ** 1 （empty when not ， 
//! in this 1 ）。
//! response `activated` / `activationError` is this 。
//!
//! ── need not need ：** need **（ and 「」）────────
//! is ， by IP ； by from ， not and 。

use axum::body::Bytes;
use axum::response::Response;
use serde_json::Value;

use crate::server::errors::management_error;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// （ not is ZCode 1 ）。
fn zcode_account_of(
    state: &ServerState,
    account_id: &str,
) -> Result<Value, (i32, String)> {
    let record = state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|account| account.get("id").and_then(Value::as_str) == Some(account_id))
        })
        .cloned()
        .ok_or_else(|| (404, "找不到该 ZCode 账号".to_string()))?;
    let provider = record
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !crate::server::core::providers::zcode::region::Region::from_provider_id(provider).is_some()
    {
        return Err((400, "该账号不是 ZCode 账号".to_string()));
    }
    Ok(record)
}

/// 1 。
async fn call_action(
    state: &ServerState,
    account_id: &str,
    action: &str,
    payload: &Value,
) -> Result<Value, (i32, String)> {
    let provider = state
        .store()
        .account_provider_of(account_id)
        .ok_or_else(|| (404, "找不到该 ZCode 账号".to_string()))?;
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
        return Err((
            400,
            format!(
                "{}的提供商插件没有安装，无法执行套餐动作",
                crate::server::core::providers::label_of(&provider)
            ),
        ));
    };
    adapter
        .admin_action(state.store(), account_id, action, payload)
        .await
        .map_err(|error| (error.status_code, error.message))
}

/// `POST /api/accounts/{id}/zcode-claim/preview`：current can 。
///
/// successresponse （ before and 「」 by ）：
/// ```json
/// { "plans": [ { "planId", "name", "description", "priority",
///                "startsAt", "endsAt", "entitlements": [...] } ],
///   "deployed": true, "activated": true, "activationError": null }
/// ```
/// `deployed: false` up （404）—— ** this not is error**，
/// is before 。 before 「current no has can 」 not is 。
///
/// ── `activated` / `activationError` ────────────────────
/// · `null` —— no （ to ： not ；：none from ；
/// no has `userId`： not ）。** no **；
/// · `true` / `false` —— / failure（Reason in ）。
pub async fn preview(state: &ServerState, account_id: &str) -> Response {
    if let Err((status, message)) = zcode_account_of(state, account_id) {
        return management_error(status, message);
    }
    match call_action(state, account_id, "claim.preview", &Value::Null).await {
        Ok(body) => {
            record_daily_checkin(state, account_id, &body);
            ok_json(body)
        },
        Err((status, message)) => management_error(status, message),
    }
}

/// `POST /api/accounts/{id}/zcode-claim/captcha-config`：config。
///
/// before SDK（`prefix` / `region` / `sceneId` 1 not can ）。
/// and AutoClaw OAuth `/api/session/login/oauth/captcha-config` is 1 
/// 1 ， is not 。
///
/// ── not ──────────────────────────────────────────────────
/// is （ 1 1 ）， not in this TTL 。
///
/// ── this 1 ** not need has jwt** ──────────────────────────────
/// config is 「 up need not need 」 ， and none。 before 
/// 「 jwt」 will use as has ， Reason is no 。
pub async fn captcha_config(state: &ServerState, account_id: &str) -> Response {
    if let Err((status, message)) = zcode_account_of(state, account_id) {
        return management_error(status, message);
    }
    match call_action(state, account_id, "claim.captcha-config", &Value::Null).await {
        Ok(body) => ok_json(body),
        Err((status, message)) => management_error(status, message),
    }
}

/// `POST /api/accounts/{id}/zcode-claim`： 1 。
///
/// request：
/// ```json
/// { "planId": "...", "captchaVerifyParam": "...", "captchaRegion": "..." }
/// ```
/// `planId` or as empty when **** that can （）。
/// before can 「、 use 」， can 「 after use 1 」。
///
/// ── failurereturn HTTP 200（ not is HTTP error）─────────────
/// 「 already 」「 use 」「 no 」 is ** **， before need 
/// `failure` to not 。 has 「request no 」（ not in 、 jwt、
/// ） use 2xx。
pub async fn claim_plan(state: &ServerState, account_id: &str, body: &Bytes) -> Response {
    if let Err((status, message)) = zcode_account_of(state, account_id) {
        return management_error(status, message);
    }
    let payload = parse_body(body).ok().unwrap_or(Value::Null);
    let outcome = match call_action(state, account_id, "claim.claim", &payload).await {
        Ok(outcome) => outcome,
        Err((status, message)) => return management_error(status, message),
    };
    record_daily_checkin(state, account_id, &outcome);
    if let Some(object) = outcome.as_object() {
        let ok = object.get("ok").and_then(Value::as_bool).unwrap_or(false);
        let failure = object.get("failure").and_then(Value::as_str).unwrap_or("");
        let plan_id = object.get("planId").and_then(Value::as_str).unwrap_or("");
        // ── 1 ────────────────────────────────────────
        // success 、 and 「 up already 」， need ： after is 「 this 1 to 」。
        // its failure not —— no 、 not already to 。
        let claimed_at = if ok || failure == "already_claimed" {
            record_claim(state, account_id, plan_id, object.get("claimedAt").and_then(Value::as_i64))
        } else {
            0
        };
        let mut result = outcome;
        if let Some(object) = result.as_object_mut() {
            object.insert("claimedAt".to_string(), Value::from(claimed_at));
        }
        return ok_json(result);
    }
    ok_json(outcome)
}

fn record_daily_checkin(state: &ServerState, account_id: &str, body: &Value) {
    let Some(snapshot) = body.get("dailyCheckin").filter(|value| value.is_object()) else { return; };
    if let Err(message) = state.store().plugin_set_ledger(account_id, "dailyCheckin", snapshot) {
        crate::server::logging::verbose("[Claim]", &message);
    }
}

/// 1 ，return when （ms；0 = no ， by 「」）。
///
/// failure、 not ： in up already to ， as 1 failure 
/// success failure is （ and `billing::checkin::run_checkin` 1 ）。
fn record_claim(state: &ServerState, account_id: &str, plan_id: &str, claimed_at: Option<i64>) -> i64 {
    let now = crate::server::logging::now_ms();
    let at = claimed_at.filter(|at| *at > 0 && *at <= now).unwrap_or(now);
    if state.store().mark_zcode_claim(account_id, plan_id, at) {
        at
    } else {
        crate::server::logging::verbose(
            "[Claim]",
            &format!("账号 {account_id} 的领取时间未能落盘（账号可能已被删除）"),
        );
        0
    }
}
