//! ：`GET/PUT /api/sidebar-layout`。
//!
//! ── Why need has this 1 for ────────────────────────────────────
//! from before localStorage，（`tauri://localhost`） and 
//! （`http://127.0.0.1:<port>`） is ** origin**，localStorage
//! —— in WebUI ，client old 。 + 1 for after 
//! 1 ， this is 「set in 1 」 1 。
//!
//! ── ────────────────────────────────────────────────────
//! ```text
//! GET /api/sidebar-layout → { layout: < for > | null }
//! PUT /api/sidebar-layout → { layout: < for > | null }， 1 
//! ```
//! · `layout: null` = no / use default → before `defaultLayout()`；
//! · its 1 400（must is for or null）。
//!
//! ── Why after 「 is not is for 」────────────────────────
//! by before `ui/sidebar-layout.js` `MENUS` ，**
//! is has **。 after if 1 ， before or after 
//! this will —— this has 1 ， and this in has that 1 。
//! `core::sidebar_layout` module header。
//!
//! ── not to when Why not ────────────────────────────────────
//! 「 no has 」 is ** **， not is ：return `layout: null`
//! before default。 if this 404 / 500， before need as 「 not in 」 and 
//! 「 no 」 —— 1 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::sidebar_layout;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// GET /api/sidebar-layout
pub async fn get_layout(State(_state): State<ServerState>) -> Response {
    ok_json(json!({ "layout": sidebar_layout::load() }))
}

/// PUT /api/sidebar-layout —— body `{layout: {...} | null}`
///
/// `layout` and `null` （「default」）： before 
/// can can （ / null）， not will as 。
pub async fn put_layout(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    // = default（ up ）
    let layout: Option<&Value> = match object.get("layout") {
        None | Some(Value::Null) => None,
        Some(value @ Value::Object(_)) => Some(value),
        Some(_) => {
            return errors::management_error(400, "layout 必须是 JSON 对象或 null");
        }
    };
    if let Err(message) = sidebar_layout::save(layout) {
        return errors::management_error(400, message);
    }
    ok_json(json!({ "layout": sidebar_layout::load() }))
}
