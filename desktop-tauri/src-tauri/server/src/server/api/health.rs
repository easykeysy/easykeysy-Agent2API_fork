//! GET /health —— and before need 。
//!
//! shape for Node server.mjs handleHealth（562-584 ）：
//! status by 「 up is config」（configured ok，otherwise degraded）/
//!   upstreamBaseUrl / authSource / currentAccountId /
//! tokenExpiresAt / canRefresh `auth.get_config_summary()`；
//! login as `auth.get_status()` （ already /）；
//! authRequired use need need Key / defaultModel / models 。
//!
//! `authApiBase` / `chatApiUrl` already delete（2026-10 ）： is ** up **，
//! 「 up 」 1 to ， not has 。
//! not is to null —— （、 before ） not 。
//!
//! Note /health return** for **（ not is `{success,data}` ）—— Node this 
//! 用的是 sendJson 直接发对象，与 /api/* not 。
//!
//! `get_config_summary()` is （ is in ， not request），
//! health not need need etc 。
//!
//! `models` is directory（ 4 as ）：Node `modelCatalog.list().length`，
//! directory new when is ， new after is ——
//! this 「current directory」， not new （ new by and 
//! GET /v1/models ，/health ）。

use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config;
use crate::server::http::raw_json;
use crate::server::ServerState;

/// up not configured when 。
///
/// and `core::auth::UNCONFIGURED_REASON` 1 —— Node this has ：
/// `getConfigSummary()` 「（node server.mjs --login）」 and 
/// 「noneavailable」。 Rust no has that （ UI 
/// /api/session/login/*）， to 1 not in will use ， 1 after 。
pub const UNCONFIGURED_REASON: &str = crate::server::core::auth::UNCONFIGURED_REASON;

/// GET /health
pub async fn handle(State(state): State<ServerState>) -> Response {
    // headless （ / /v1 fail-closed） down 「」：
    // need will up 、、 to up 。
    // （Docker HEALTHCHECK / ） and status ， not ；
    // and not —— that down and fail-closed not will 。
    if crate::server::access::panel_gate() || crate::server::access::v1_fail_closed() {
        return raw_json(json!({ "status": "ok" }));
    }
    let snapshot = config::current();
    let summary = state.auth().get_config_summary();
    let status = state.auth().get_status();
    let healthy = summary
        .get("configured")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    raw_json(json!({
        "status": if healthy { "ok" } else { "degraded" },
        "transport": "upstream-api",
        "product": "WorkBuddy",
        "upstreamConfigured": healthy,
        "upstreamBaseUrl": summary.get("baseUrl").cloned().unwrap_or(Value::Null),
        "authSource": summary.get("authSource").cloned().unwrap_or(Value::Null),
        "currentAccountId": summary.get("currentAccountId").cloned().unwrap_or(Value::Null),
        "tokenExpiresAt": summary.get("tokenExpiresAt").cloned().unwrap_or(Value::Null),
        "canRefresh": summary.get("canRefresh").and_then(Value::as_bool).unwrap_or(false),
        "login": status,
        "unavailableReason": if healthy {
            Value::Null
        } else {
            summary
                .get("unavailableReason")
                .cloned()
                .unwrap_or_else(|| Value::String(UNCONFIGURED_REASON.to_string()))
        },
        "authRequired": true,
        "defaultModel": snapshot.default_model(),
        // directory（ for Node `models: modelCatalog.list().length`）。
        // is 「 1 up when 」 ： that when has workbuddy 1 
        // directory。 in directory by in when （ `core::providers::catalog`），
        // this **** that 1 —— and old for （ 1 when 1 ）。
        // available `/v1/models`（）。
        "models": crate::server::core::providers::catalog::manifest_for(
            crate::server::core::providers::ProviderKind::WorkBuddy,
        )
        .len(),
    }))
}
