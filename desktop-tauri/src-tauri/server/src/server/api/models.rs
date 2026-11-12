//! POST /api/models/refresh —— new （「 new 」 by ）。
//!
//! ── Why 1 not is api/chat.rs ────────────────
//! `chat.rs` already `/v1/models`（** for ** OpenAI ） and 
//! `/v1/chat/completions`。 this is ** API**： protected 、 need API Key、
//! will up 、return `{success, data}` —— and chat.rs for 、
//! 、error not 。 in 1 will 「 this to not for 」 need 
//! ；file + （ and `auto_checkin.rs` / `billing.rs` 1 ）
//! in by up can 。
//!
//! ── Whyreturn 2xx（ and credential_maintenance 1 ）────────
//! 「， 1 failure」 is ****：、 token 、
//! up when 5xx。 this already in `results` （status = failed + message），
//! use （ before ） can to 「、Why」。 if HTTP return 2xx， use 
//! errorresponse、 from error that —— and 「success」 will 1 
//! 「failure」， use to is 1 no ， not to its to 。
//! 「 this new no has 」（ = 2xx）——
//! and `/api/accounts/refresh-expiring` 1 ； that is 「 failure」。
//!
//! this 2xx is 「request no 」： body is optional 
//! （`{accounts: {providerId: accountId}, providers: [providerId, ...]}`，
//! down ），empty and JSON to 「 + default」—— use 
//! （ when 、bridge old ） not parameter， not can as this optional 400。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::providers::adapter::refresh_implemented_forced;
use crate::server::core::providers::catalog;
use crate::server::http::ok_json;
use crate::server::logging;
use crate::server::ServerState;

/// **total**：`{results, refreshed, skipped, failed}`。
///
/// and `credential_maintenance::refresh_expiring_report` 1 （total + ），
/// can 1 to ， can Why。 is `(success, , failure)`，
/// and use 1 statistics（「success 2、responsesuccess 3」 this for not up ）。
/// is ** before after **， and `refresh_implemented_forced`。
fn summary_of(results: &[Value]) -> (Value, (usize, usize, usize)) {
    let count = |status: &str| {
        results
            .iter()
            .filter(|item| item.get("status").and_then(Value::as_str) == Some(status))
            .count()
    };
    let (refreshed, skipped, failed) = (count("refreshed"), count("skipped"), count("failed"));
    (
        json!({
            "results": results,
            "refreshed": refreshed,
            "skipped": skipped,
            "failed": failed,
        }),
        (refreshed, skipped, failed),
    )
}

/// optional `accounts` （`{providerId: accountId}`， down ）。
///
/// empty / JSON / `accounts` / not is for —— 1 returnempty（= by 
/// default）。failure** not 400**： this is add optional， use not 
/// is use （module header）。
fn parse_accounts(body: &Bytes) -> serde_json::Map<String, Value> {
    serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|value| value.get("accounts").cloned())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

/// optional `providers` （`["workbuddy", ...]`， down ）。
///
/// ****（empty / JSON / use ）→ `None` = already ；
/// in → by empty（trim），** not for or empty = empty**
/// —— empty is valid 「 not 」：「 up not to not need 」 this 
/// can not （ is this need as ）。
fn parse_providers(body: &Bytes) -> Option<Vec<String>> {
    let value = serde_json::from_slice::<Value>(body).ok()?;
    let items = value.get("providers")?;
    Some(
        items
            .as_array()
            .map(|array| {
                array
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::trim)
                    .filter(|id| !id.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
    )
}

/// POST /api/models/refresh
///
/// new 「supportsdirectory」 （）， and ** new after **
/// 1 return。
///
/// ── Whyresponse， not is before 1 /api/session ──────
/// use by down this by ** 1 ** is 「 up to new 」。 ：
/// 1. in that 1 is 「 new success + is old 」， no ；
/// 2. need 1 `store` and `list_models`（ after for workbuddy
/// is + ）， in request already ；
/// 3. 1 1 can can failure will ， this failure will 「 new success」
/// 「 new no update」—— 1 。
/// this return `models`（ = `/api/session` `models` ，
/// 1 `catalog::session_models`）， before ，。
/// use 1 not is 1 ： new when 1 。
/// ── optional `accounts` （「」 down ）──────────────────
/// body（JSON for ， can ）：`{"accounts": {"<providerId>": "<accountId>"}}` ——
/// 「 use directory」。 provider / empty = this by default
/// （available， `AccountStore::current_entry_for_provider`）；
/// not to that when ， to failureReason（ not to ）。
/// `accountId` is **** use ， that 1 。
///
/// ── optional `providers` （「get」 ）────
/// `{"providers": ["<providerId>", ...]}` —— ， 
/// ** not 、 not **。 by before by 「 has ∪ has enabled
/// 」（ models-fetch-modal.js scopeProviders）： use in up 
/// not to （ no has enabled、 as empty） will to 「」 。
/// = already —— when and old use this （
/// `parse_providers`）。
///
/// Why is optional ： this has use （ when 、bridge old ） not 
/// parameter，「 」「」 is add ， not can use 400。
pub async fn refresh_models(State(state): State<ServerState>, body: Bytes) -> Response {
    let store = state.store();
    let accounts = parse_accounts(&body);
    let providers = parse_providers(&body);
    let results = refresh_implemented_forced(store, &accounts, providers.as_deref()).await;
    // total 1 ：response that and （ `summary_of`）
    let (mut payload, (refreshed, skipped, failed)) = summary_of(&results);
    // 1 total： use in can to 「 new 、」
    // （ in ， this total）。
    // ：success is ，「failure 0 」 error
    logging::log_with_level(
        "[Models]",
        &format!("手动刷新模型清单：成功 {refreshed}，跳过 {skipped}，失败 {failed}"),
        if failed > 0 { "error" } else { "info" },
    );
    // in ** new after **： when already is new （ up 「Why」）
    let models = catalog::session_models(store);
    if let Some(object) = payload.as_object_mut() {
        object.insert("models".to_string(), Value::Array(models));
    }
    ok_json(payload)
}
