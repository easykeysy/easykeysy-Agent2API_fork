//! Qoder UMID （） and ：
//!
//! - `GET /api/qoder-umid` → `risk.status`（available / source /
//!   installSupported / installing）
//! - `POST /api/qoder-umid/install` → `risk.install`（ down ；only
//! Linux；Windows / macOS client / qodercli，none need down ）
//!
//! Why is 1 not is by ： is **** （ 1 
//! ）， and 1 none； `/api/accounts/{id}/…` will 
//! 「 in not in 」 。
//!
//! ── already to ─────────────────────────────────
//! from npm 、 integrity、ELF by 、 to 
//! path —— is 「 up as this is client」 ， by 
//! need not in 。 this 「 1 1 RPC」。
//!
//! ： to in in Qoder ，Linux and 
//! when to 「 1 」 by （ when minutes —— need down ~10MB npm and 
//! ， before by down after GET installing can ）。

use axum::extract::State;
use axum::response::Response;
use serde_json::Value;

use crate::server::errors::management_error;
use crate::server::http::ok_json;
use crate::server::ServerState;

/// ****（`qoder-intl`） that 1 。
const PROVIDER: &str = "qoder-intl";

/// 1 `admin.action`。
async fn call_action(state: &ServerState, action: &str) -> Result<Value, (i32, String)> {
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(PROVIDER) else {
        return Err((400, "Qoder 的提供商插件没有安装，无法查询设备组件".to_string()));
    };
    adapter
        .admin_action(state.store(), "", action, &Value::Null)
        .await
        .map_err(|error| (error.status_code, error.message))
}

/// GET /api/qoder-umid
pub async fn get_status(State(state): State<ServerState>) -> Response {
    match call_action(&state, "risk.status").await {
        Ok(status) => ok_json(status),
        Err((status_code, message)) => management_error(status_code, message),
    }
}

/// POST /api/qoder-umid/install
pub async fn install(State(state): State<ServerState>) -> Response {
    match call_action(&state, "risk.install").await {
        Ok(manifest) => {
            crate::server::logging::log("[Qoder]", "UMID 组件安装完成（来自签到中心）");
            ok_json(manifest)
        }
        Err((status_code, message)) => management_error(status_code, message),
    }
}
