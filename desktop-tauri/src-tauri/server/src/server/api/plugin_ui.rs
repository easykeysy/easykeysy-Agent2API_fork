//! UI ：`GET /plugins/{pack}/ui/{*rest}`。
//!
//! ── Why by not is client ──────────────────────
//! is ，** has in **（`{config_dir}/plugins/
//! <id>/<version>/ui/`）。 is ， that file not to ——
//! must by 。
//!
//! ── Why this by ** not protected** ────────────────────────
//! is （HTML + JS + CSS）， need can iframe ； iframe 
//! request not up 。 to is use `postMessage` 
//! （见客户端的 `plugin-frame` 桥接）。真正的数据请求仍走 `/api/*`， that 
//! protected —— not ， is 。
//!
//! this is and 「 `ui/` directory」 1 ：`ui/index.html` not 。
//!
//! ── ────────────────────────────────────────────────────
//! path and check in `plugin_host::resolve_ui_file`：** already enabled** 
//! `ui/` directory，`..` and for path 1 。 this file + content-type。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, StatusCode};
use axum::response::Response;

use crate::server::core::plugin_host;
use crate::server::errors;

/// GET /plugins/{pack}/ui/{*rest}
pub async fn serve(Path((pack, rest)): Path<(String, String)>) -> Response {
    let Some(path) = plugin_host::resolve_ui_file(&pack, &rest) else {
        return errors::not_found_response("GET", "plugin-ui");
    };
    let bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,
        Err(_) => return errors::not_found_response("GET", "plugin-ui"),
    };
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_of(&path))
        // and 1 ： after to new ，
        // not will 「 use old js」 in 
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(bytes));
    response.unwrap_or_else(|_| errors::not_found_response("GET", "plugin-ui"))
}

/// → content-type。 is 1 before ， use 。
fn mime_of(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "webp" => "image/webp",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "map" => "application/json",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}
