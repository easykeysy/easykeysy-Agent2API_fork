//! CodeArts daily 。
//!
//! ── Why is 「、」 ─────────────────────
//! is ** **（ 1 down will to up 1 will ）， up must
//! has 1 use 「 has can 、 already 」， by 1 
//! request。 this and ZCode when 1 （`api::zcode_claim`），
//! and 「 not can need has 」 has 1 。
//!
//! ── this ────────────────────────────────────────────
//! in ****（`codearts::welfare`， `welfare.preview` /
//! `welfare.claim`）：、 etc 、6 /10 minutes
//! ， is 「 as 」 —— by need ，
//! this not in 。 this 、`auto` and 。
//!
//! ── ： already and daily to （2026-10， by use need ）─────────────
//! in cron up （ when `5-55/10 * * * *`， 6 ）。
//! ** not request**（ is 1 ）， by is 「none
//! will is ， not is after 」； use need this 1 
//! daily to after ， this ：CodeArts daily in by to 
//! （`core::billing::checkin::checkin_for` codearts ， and when ）。
//! file this **** —— → → that in 
//! （`ui/codearts-welfare.js` ）， is to in not 。
//!
//! path （ ≤6 、 ≥10 minutes） in `welfare` ，
//! `welfare.claim` `auto` parameter： when and true（），
//! use in up false（ not ）—— is none ， not is 。

use axum::body::Bytes;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::errors::management_error;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// （ not CodeArts 1 404）。
fn codearts_account_of(state: &ServerState, account_id: &str) -> Result<Value, Response> {
    state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|account| {
                account.get("id").and_then(Value::as_str) == Some(account_id)
                    && account.get("provider").and_then(Value::as_str) == Some("codearts")
            })
        })
        .cloned()
        .ok_or_else(|| management_error(404, "未找到 CodeArts 账号"))
}

/// 1 。
async fn call_action(
    state: &ServerState,
    account_id: &str,
    action: &str,
    payload: &Value,
) -> Result<Value, Response> {
    codearts_account_of(state, account_id)?;
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id("codearts") else {
        return Err(management_error(
            400,
            "CodeArts 的提供商插件没有安装，无法执行福利动作",
        ));
    };
    adapter
        .admin_action(state.store(), account_id, action, payload)
        .await
        .map_err(|error| management_error(error.status_code, error.message))
}

/// `POST /api/accounts/{id}/codearts-welfare/preview` —— ** is **，
/// 1 request not ； use POST is as 「 + 」 by after in 
/// POST that up （ZCode `zcode-claim/preview` 1 ）。
/// up that 1 has 1 `GET /v1/ops/delivery`。
pub async fn preview(state: &ServerState, account_id: &str) -> Response {
    match call_action(state, account_id, "welfare.preview", &Value::Null).await {
        Ok(document) => ok_json(document),
        Err(response) => response,
    }
}

/// `POST /api/accounts/{id}/codearts-welfare` —— 。
///
/// body optional `{"auto": true}`：default by ****（ not ， as 
/// is none ）。 will when `auto:true` will that 6 / 10 minutes
/// 。
pub async fn claim(state: &ServerState, account_id: &str, body: &Bytes) -> Response {
    let Ok(parsed) = parse_body(body) else {
        return management_error(400, "请求体不是合法 JSON");
    };
    let auto = auto_requested(&parsed);
    match call_action(
        state,
        account_id,
        "welfare.claim",
        &json!({ "auto": auto }),
    )
    .await
    {
        Ok(mut outcome) => {
            // by `ledger_put` （ `pluginLedger.codeartsWelfare`），
            // this to 「」。
            //
            // ── Why not is ──────────────
            // after ： by `ctx.ledger_*` ， is 
            // `pluginLedger.<kind>`（ `account_store::plugin_accounts`）。
            // `codeartsWelfare` for ****valid， new is null。
            if let Some(object) = outcome.as_object_mut() {
                object.insert(
                    "welfare".to_string(),
                    state
                        .store()
                        .plugin_ledger(account_id, "codeartsWelfare"),
                );
            }
            ok_json(outcome)
        }
        Err(response) => response,
    }
}

/// this not 「none 」。
///
/// must is **false（）**： in by is 「 in no when 
/// will 」， default etc default 。 will when 
/// `{"auto": true}` will 6 / 10 minutes。
/// （`"true"` / `1`） by —— will use ****。
fn auto_requested(body: &Value) -> bool {
    body.get("auto").and_then(Value::as_bool).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    //! 1 ：** default is 「」**。 this not will has ，
    //! after is 6 /10 minutes（ or ）。
    use super::auto_requested;
    use serde_json::json;

    #[test]
    fn only_an_explicit_boolean_auto_enables_the_rate_limited_path() {
        assert!(!auto_requested(&json!({})), "面板默认调用必须是手动");
        assert!(!auto_requested(&json!({"auto": null})));
        assert!(!auto_requested(&json!({"auto": false})));
        assert!(auto_requested(&json!({"auto": true})));
        // not ： will use 
        assert!(!auto_requested(&json!({"auto": "true"})));
        assert!(!auto_requested(&json!({"auto": 1})));
    }
}
