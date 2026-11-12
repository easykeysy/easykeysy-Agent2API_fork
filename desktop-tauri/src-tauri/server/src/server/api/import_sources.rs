//! 「 from its 」 API（current：cc-switch）。
//!
//! - `GET /api/import/cc-switch` → cc-switch （`core::import_ccswitch`）
//!
//! and custom-providers 1 ：** 1 use ** config
//! file and in response API Key （ need need ）， in protected after 。
//! response `available: false` is （ no cc-switch / file not can ），
//! not is error —— before by empty， this not return 4xx/5xx。

use axum::extract::State;
use axum::response::Response;

use crate::server::core::import_ccswitch;
use crate::server::http::ok_json;
use crate::server::ServerState;

pub async fn scan_cc_switch(State(_state): State<ServerState>) -> Response {
    ok_json(import_ccswitch::scan())
}
