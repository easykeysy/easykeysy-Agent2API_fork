//! headless ： `ui/` directory 1 and 。
//!
//! ── Why not use tower-http ServeDir ───────────────────────
//! is 1 file， need need is 「file + content-type +
//! path」，； tower-http as this 1 ，
//! for 404 / as —— not （CORS not use is 1 by ）。
//!
//! ── ────────────────────────────────────────────────
//! · `ui/` directory file：path 1 ，`..` and not ；
//! · `/api`、`/v1`、`/auth`、`/health` before not —— fallback
//! 「 by 」， 1 ， will by after 
//! ；
//! · in index.html up ， its 。
//!
//! ── ────────────────────────────────────────────────────
//! file ETag（ + when + ），`Cache-Control` as `no-cache`：
//! can down ， use before must。file no when 304、 not ；
//! new `ui/` after mtime 、ETag ， to new —— not will 
//! 「 new after use old js」， not down MB。
//! by `compression` in 1 ， this `Vary: Accept-Encoding`。

use std::path::PathBuf;

use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use axum::response::Response as AxumResponse;

use crate::server::errors;

/// can ， use before need （module header 「」）。
const CACHE_CONTROL: &str = "no-cache";
/// 1 path client is gzip not ，must by this 。
const VARY: &str = "Accept-Encoding";

/// path is 「 before 」： not ，
/// path 404 （Node as ）。
fn is_api_path(path: &str) -> bool {
    path == "/health"
        || path.starts_with("/api/")
        || path.starts_with("/v1/")
        || path.starts_with("/auth/")
        || path == "/api"
        || path == "/v1"
        || path == "/auth"
}

/// ：`ui_dir` already when file，failure to 404 。
pub async fn serve(request: Request<Body>, ui_dir: PathBuf) -> AxumResponse {
    let path = request.uri().path().to_string();
    if is_api_path(&path) {
        return not_found(request).await;
    }
    let if_none_match = request
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let Some(file_path) = resolve_file(&ui_dir, &path) else {
        return not_found(request).await;
    };
    read_response(&file_path, if_none_match.as_deref()).await
}

/// URL path to `ui/` file（`None` = not in / ）。
///
/// 1 not is ：`..`、、 ，
/// 1 `..` before （） 1 。
fn resolve_file(ui_dir: &PathBuf, url_path: &str) -> Option<PathBuf> {
    // path，ignore（axum uri().path() already not query，）
    let url_path = url_path.split('?').next().unwrap_or("");
    let mut segments: Vec<&str> = Vec::new();
    for segment in url_path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => segments.push(other),
        }
    }
    let mut file_path = ui_dir.clone();
    if segments.is_empty() {
        file_path.push("index.html");
        return file_path.is_file().then_some(file_path);
    }
    for segment in &segments {
        // 1 not this path（Windows ）
        if segment.contains('/') || segment.contains('\\') {
            return None;
        }
        file_path.push(segment);
    }
    if file_path.is_file() {
        return Some(file_path);
    }
    // none path by .html（/login → login.html）：
    // is this ； request not （/app.js ）
    let no_extension = segments
        .last()
        .is_some_and(|segment| !segment.contains('.'));
    if no_extension {
        file_path.set_extension("html");
        return file_path.is_file().then_some(file_path);
    }
    None
}

/// file and response；failure（ etc ） by 404 （ not 404/403，
/// 1 is 「 not in 」）。
///
/// ETag in when 304， not file —— this is 「 not down 
/// MB 」 path。
async fn read_response(file_path: &PathBuf, if_none_match: Option<&str>) -> AxumResponse {
    let metadata = match tokio::fs::metadata(file_path).await {
        Ok(metadata) => metadata,
        Err(_) => return errors::not_found_response("GET", "static"),
    };
    let etag = etag_of(&metadata);
    if if_none_match.is_some_and(|value| etag_matches(value, &etag)) {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, &etag)
            .header(header::CACHE_CONTROL, CACHE_CONTROL)
            .header(header::VARY, VARY)
            .body(Body::empty())
            .unwrap_or_else(|_| errors::not_found_response("GET", "static"));
    }
    let bytes = match tokio::fs::read(file_path).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return errors::not_found_response("GET", "static");
        }
    };
    let is_index = file_path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "index.html");
    let body = if is_index {
        inject_web_shim(&bytes)
    } else {
        bytes
    };
    let mime = mime_of(file_path);
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, CACHE_CONTROL)
        .header(header::ETAG, &etag)
        .header(header::VARY, VARY)
        .header(header::CONTENT_LENGTH, body.len())
        .body(Body::from(body));
    response.unwrap_or_else(|_| errors::not_found_response("GET", "static"))
}

/// file： + when + 。
///
/// not file is ： 2 MB ，request 1 
/// file not ； `ui/` is ，mtime and 
/// new old 。 up is as `index.html` ——
/// that in ， after html file no ， can can 。
fn etag_of(metadata: &std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(
        "W/\"{}-{:x}-{:x}\"",
        env!("CARGO_PKG_VERSION"),
        metadata.len(),
        modified
    )
}

/// `If-None-Match` ：supports `*`、 and before （`W/`）。
///
/// by （ `W/`）： 1 file gzip and use 
/// this ETag， is 「、 can can not 」 。
fn etag_matches(header: &str, etag: &str) -> bool {
    let target = etag.strip_prefix("W/").unwrap_or(etag);
    header.split(',').any(|item| {
        let item = item.trim();
        item == "*" || item.strip_prefix("W/").unwrap_or(item).trim() == target
    })
}

/// index.html：`<head>` after 1 ，
/// `window.workbuddyDesktop` in all before 
/// （ and bridge.rs 「 before 」 1 when ）。
fn inject_web_shim(html: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(html) else {
        return html.to_vec();
    };
    let shim = format!("<script>{}</script>", crate::web_shim::shim_js());
    match text.replacen("<head>", &format!("<head>{shim}"), 1) {
        replaced if replaced != text => replaced.into_bytes(),
        // no has <head>（ ）： to 
        _ => format!("{text}<script>{}</script>", crate::web_shim::shim_js()).into_bytes(),
    }
}

/// → content-type（ use to ， its by ）
fn mime_of(file_path: &PathBuf) -> String {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        // PWA （`ui/manifest.webmanifest`）。 is `application/manifest+json`，
        // for `application/json` ；** to down `_` that 
        // `application/octet-stream` not is JSON ** —— failure，
        // 「 to 」 / / not 
        // （iOS that `apple-touch-icon` + `apple-mobile-web-app-*`， not ）。
        "webmanifest" => "application/manifest+json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
    .to_string()
}

async fn not_found(request: Request<Body>) -> AxumResponse {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    errors::not_found_response(&method, &path)
}
