//! response（gzip）。
//!
//! ── Why need need this 1 ────────────────────────────────────────
//! is 1 not （`islands/ui.js` 2.3 MB、`ui.css` 130 KB、
//! 700 KB）， up `/api/session`（ when up KB）。
//! this is ，gzip can to 1 down ； 1 will 
//! `Accept-Encoding: gzip`。 not ，（ up / ） is 
//! down need seconds —— 2026-11 「 1 」 is this path。
//!
//! ── Why in ──────────────────────────────────────
//! `/v1/*` for response is SSE 。 for will 、 down ，
//! not 。`panel_router` not `/v1/*`， this in in up 
//! response；（`gateway_router`）。
//!
//! ── ──────────────────────────────────────────────
//! response：
//! · client gzip（ `*`；`gzip;q=0` as not ）；
//! · 200（304 / 302 / 4xx no has can ，）；
//! · no has `Content-Encoding`（）；
//! · `Content-Type` is （JSON / JS / CSS / HTML / SVG / XML …）；
//! · already and in 1 KB – 8 MB —— not ， 
//! （、 down ） need not is ， need will ；
//! · response no has `Cache-Control: no-transform`。
//!
//! use `Body::size_hint().exact()` ， not is `Content-Length` ：
//! axum `Json` / `Body::from(Vec)` in in when no has `Content-Length`
//! （ that is hyper response when ）， will allresponse。
//!
//! ── and ────────────────────────────────────────
//! file ETag， when 304、 not ， not will to this 
//! ； has 「 1 down 」 or 「file 」 will 。 not 
//! not will in new when CPU。

use axum::body::{to_bytes, Body, HttpBody};
use axum::http::{header, HeaderMap, HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

/// this not ：gzip and will 。
const MIN_SIZE: usize = 1024;
/// limit。 response（ / down ） need already ， need not is 。
const MAX_SIZE: usize = 8 * 1024 * 1024;

/// gzip in 。
pub(super) async fn gzip(request: Request<Body>, next: Next) -> Response {
    // HEAD response、 no has ； will Content-Length and not 。
    if request.method() == axum::http::Method::HEAD {
        return next.run(request).await;
    }
    let accepts_gzip = accepts_gzip(request.headers());

    let mut response = next.run(request).await;

    if !compressible(&response) {
        return response;
    }

    // this not （e.g.）， need Accept-Encoding ——
    // otherwise in can can that to gzip client，。
    append_vary(&mut response, "Accept-Encoding");

    if !accepts_gzip {
        return response;
    }

    let size = response
        .body()
        .size_hint()
        .exact()
        .or_else(|| content_length(&response).map(|value| value as u64));
    let Some(size) = size else {
        // unknown = （SSE / down ）， 1 not 。
        return response;
    };
    if !((MIN_SIZE as u64)..=(MAX_SIZE as u64)).contains(&size) {
        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = match to_bytes(body, MAX_SIZE).await {
        Ok(bytes) => bytes,
        // up not to （ already and in up limit）。 when can empty 500，
        // not can already body response。
        Err(_) => return internal_error(),
    };

    // is CPU ，2 MB in NAS up need ms； to ，
    // tokio IO 。
    let original = bytes.clone();
    let compressed = tokio::task::spawn_blocking(move || gzip_bytes(&bytes)).await;
    let compressed = match compressed {
        Ok(Ok(value)) => value,
        // failure not is failure： ， can use 。
        _ => {
            parts.headers.remove(header::CONTENT_LENGTH);
            parts
                .headers
                .insert(header::CONTENT_LENGTH, length_value(original.len()));
            return Response::from_parts(parts, Body::from(original));
        }
    };

    parts
        .headers
        .insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
    parts.headers.remove(header::CONTENT_LENGTH);
    parts
        .headers
        .insert(header::CONTENT_LENGTH, length_value(compressed.len()));
    Response::from_parts(parts, Body::from(compressed))
}

/// response is can to gzip （ not 「client is 」 this 1 ）。
fn compressible(response: &Response) -> bool {
    if response.status() != StatusCode::OK {
        return false;
    }
    if response.headers().contains_key(header::CONTENT_ENCODING) {
        return false;
    }
    let no_transform = response
        .headers()
        .get(header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("no-transform"));
    if no_transform {
        return false;
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    is_text_mime(content_type)
}

/// Content-Type （ will that ）。
fn is_text_mime(content_type: &str) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    // SSE 1 ： is text/*， need need down （module header）。
    if mime == "text/event-stream" {
        return false;
    }
    mime.starts_with("text/")
        || matches!(
            mime.as_str(),
            "application/json"
                | "application/manifest+json"
                | "application/javascript"
                | "application/x-javascript"
                | "application/xml"
                | "application/xhtml+xml"
                | "image/svg+xml"
        )
}

/// `Accept-Encoding`，client not gzip。
///
/// `gzip` / `x-gzip` `*`；`q=0` 。 this not 
/// `identity` or its —— gzip 1 。
fn accepts_gzip(headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let mut gzip_q: Option<f32> = None;
    let mut wildcard_q: Option<f32> = None;
    for item in value.split(',') {
        let mut parts = item.trim().split(';');
        let coding = parts.next().unwrap_or("").trim().to_ascii_lowercase();
        if coding.is_empty() {
            continue;
        }
        let mut q = 1.0_f32;
        for parameter in parts {
            let parameter = parameter.trim();
            if let Some(raw) = parameter.strip_prefix("q=") {
                q = raw.trim().parse().unwrap_or(0.0);
            }
        }
        match coding.as_str() {
            "gzip" | "x-gzip" => gzip_q = Some(q),
            "*" => wildcard_q = Some(q),
            _ => {}
        }
    }
    gzip_q.or(wildcard_q).unwrap_or(0.0) > 0.0
}

/// in `Vary` 1 （ already has then not ）。
fn append_vary(response: &mut Response, value: &'static str) {
    let existing = response
        .headers()
        .get(header::VARY)
        .and_then(|header| header.to_str().ok())
        .unwrap_or("");
    if existing
        .split(',')
        .any(|item| item.trim().eq_ignore_ascii_case(value))
    {
        return;
    }
    let merged = if existing.trim().is_empty() {
        value.to_string()
    } else {
        format!("{existing}, {value}")
    };
    if let Ok(header) = HeaderValue::from_str(&merged) {
        response.headers_mut().insert(header::VARY, header);
    }
}

/// from already has `Content-Length` （`size_hint` not to when ）。
fn content_length(response: &Response) -> Option<usize> {
    response
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse().ok())
}

fn length_value(length: usize) -> HeaderValue {
    HeaderValue::from_str(&length.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
}

fn gzip_bytes(bytes: &[u8]) -> std::io::Result<Vec<u8>> {
    // 6 is gzip default： and CPU in 。（9） ，
    // will NAS / up etc 1 。
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(bytes)?;
    encoder.finish()
}

fn internal_error() -> Response {
    // this 1 ， not this error； to this and not 。
    // use 500 ， 「 already body」response。
    let body = Body::from("response compression failed");
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(body)
        .unwrap_or_else(|_| Response::new(Body::empty()))
}
