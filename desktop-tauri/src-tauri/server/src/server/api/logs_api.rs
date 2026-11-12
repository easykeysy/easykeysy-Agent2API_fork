//! by （ for src/workbuddy-log-routes.mjs ）。
//!
//! GET /api/logs （?limit=&level=&category=&keyword=&sinceId=&start=&end=）
//! GET /api/logs/stats / （、 down ）
//! GET /api/logs/download JSONL 
//! DELETE /api/logs empty
//!
//! response `levels` / `categories` / `max` by 1 ——
//! before logs-panel.js use down and limit， response need 。
//!
//! 与 Node 版的差异：Node 版对未匹配的 /api/logs/* pathreturn
//! `{success:false, error:"Not found: ..."}`（404）， unknownpath to 
//! 404 （OpenAI body）。 is 404， and before up path，
//! 1 is unknownpath error —— unknownpath not ， 1 。
//!
//! parameter is file ： is ，
//! and Node `Number(...)` / `Math.min(Math.max(...))` need 1 1 for up 
//! （NaN 、limit in 1..MAX_ENTRIES）。
//!
//! `start` / `end`（ms when ） is **add optional**parameter，Node no has ：
//! when need need 。 then and API 1 —— （empty / ）
//! **ignore not is **（ before can can empty「 not when 」），
//! not when and before 1 。

use std::collections::HashMap;

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Number, Value};

use super::date_range::date_bounds;
use crate::server::http::{ok_json, parse_query_ms, raw_json};
use crate::server::logging;
use crate::server::logs_store::{self, LogView, Query as LogQuery};
use crate::server::ServerState;

/// `/api/logs*` up path 404：Node this by 404
/// is API （`{success:false,error:"Not found: <METHOD> <path>"}`），
/// and OpenAI （`{error:{message}}`） not —— is 404，
/// before and by body ， not can use 。
pub async fn not_found(request: axum::extract::Request) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    crate::server::errors::management_error(404, format!("Not found: {method} {path}"))
}

/// `all` is need 「empty」（`all=1` / `all=true`）
fn explicit_all(params: &Params) -> bool {
    params.get("all").map_or(false, |value| {
        let text = value.trim();
        text == "1" || text.eq_ignore_ascii_case("true")
    })
}

/// ；enabled when return and Node 1 503
fn store_or_error() -> Result<&'static logs_store::LogStore, Response> {
    logging::store_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            raw_json(json!({ "success": false, "error": "enabled" })),
        )
            .into_response()
    })
}

/// can can has this （axum Query use HashMap ，
/// to parameter Node 「 has then use 、none then ignore」 ）
type Params = HashMap<String, String>;

/// by Node Number() ：empty/ → None
fn parse_number(value: Option<&String>) -> Option<f64> {
    let text = value?.trim();
    if text.is_empty() {
        return None;
    }
    text.parse::<f64>().ok().filter(|item| item.is_finite())
}

/// limit ， for Node 
/// `Math.min(Math.max(Number(limit) || 200, 1), MAX_ENTRIES)`：
/// - / / 0 → 200（JS `0 || 200` 200，0 not is valid limit）
/// - → 1；limit → MAX_ENTRIES
fn effective_limit(raw: Option<&String>) -> usize {
    let parsed = parse_number(raw).unwrap_or(0.0);
    let base = if parsed == 0.0 { 200.0 } else { parsed };
    base.max(1.0).min(logs_store::MAX_ENTRIES as f64) as usize
}

fn log_query(params: &Params) -> Result<LogQuery, Response> {
    let mut query = LogQuery {
        limit: Some(effective_limit(params.get("limit"))),
        offset: parse_number(params.get("offset")).unwrap_or(0.0).max(0.0) as usize,
        aggregate: params.get("aggregate").map_or(false, |value| value == "1" || value == "true"),
        view: params.get("view").map_or(LogView::All, |value| LogView::parse(value)),
        until_id: parse_number(params.get("untilId")).map(|value| value.max(0.0) as u64),
        level: params.get("level").cloned(),
        category: params.get("category").cloned(),
        keyword: params.get("keyword").cloned(),
        since_id: parse_number(params.get("sinceId")).map(|value| value.max(0.0) as u64),
        start: parse_query_ms(params.get("start")),
        end: parse_query_ms(params.get("end")),
    };
    if let Some((start, end)) = date_bounds(params)
        .map_err(|message| crate::server::errors::management_error(400, message))? {
        query.start = Some(start);
        query.end = Some(end);
    }
    Ok(query)
}

/// GET /api/logs
pub async fn query_logs(State(_state): State<ServerState>, Query(params): Query<Params>) -> Response {
    let store = match store_or_error() {
        Ok(store) => store,
        Err(response) => return response,
    };

    let query = match log_query(&params) { Ok(query) => query, Err(response) => return response };

    let result = store.query(&query);
    let mut data = match serde_json::to_value(&result) {
        Ok(value) => value,
        Err(error) => {
            logging::log("[Logs]", &format!("❌ failure: {error}"));
            return raw_json(json!({ "success": false, "error": "failure" }));
        }
    };
    if let Some(object) = data.as_object_mut() {
        attach_dictionary(object);
    }
    ok_json(data)
}

/// GET /api/logs/stats
pub async fn stats_logs(State(_state): State<ServerState>) -> Response {
    let store = match store_or_error() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let mut data = match serde_json::to_value(store.stats()) {
        Ok(value) => value,
        Err(error) => {
            logging::log("[Logs]", &format!("❌ statisticsfailure: {error}"));
            return raw_json(json!({ "success": false, "error": "statisticsfailure" }));
        }
    };
    if let Some(object) = data.as_object_mut() {
        attach_dictionary(object);
    }
    ok_json(data)
}

/// DELETE /api/logs?level=&category=&keyword=&start=&end=&all=
///
/// parameter when ** in **（「empty」—— and GET 1 ，
/// up ）；emptymust**** `all=1`
/// （ not emptyrequest to 400）—— by down `explicit_all` 。
/// use `clear()` （id new from 1 ， already has for ）。
/// responsedelete after statistics， has delete when `removed`（delete）。
pub async fn clear_logs(State(_state): State<ServerState>, Query(params): Query<Params>) -> Response {
    let store = match store_or_error() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let query = match log_query(&params) { Ok(query) => query, Err(response) => return response };
    let has_filters = query.level.as_deref().map_or(false, |level| logs_store::LEVELS.contains(&level.trim().to_lowercase().as_str()))
        || query.category.as_deref().map_or(false, |category| logs_store::CATEGORIES.iter().any(|(key, _)| *key == category.trim().to_lowercase()))
        || query.keyword.as_deref().map_or(false, |keyword| !keyword.trim().is_empty())
        || query.start.is_some()
        || query.end.is_some()
        || query.until_id.is_some()
        || query.since_id.is_some()
        || query.view != LogView::All;

    if !has_filters {
        // ── ：noneparameter when must all=1 ──────────────
        // Why 1 parameter：emptyparameter（ ：
        // before URLSearchParams for 、empty） is 
        // ** no **，「」 is —— use 1 parameter
        // this 1 400，。
        if !explicit_all(&params) {
            return crate::server::errors::management_error(
                400,
                "： need empty all=1",
            );
        }
        let stats = store.clear();
        logging::log("[Logs]", " already empty");
        return match serde_json::to_value(stats) {
            Ok(value) => ok_json(value),
            Err(error) => {
                logging::log("[Logs]", &format!("❌ empty after statisticsfailure: {error}"));
                raw_json(json!({ "success": false, "error": "emptyfailure" }))
            }
        };
    }

    let (removed, stats) = store.clear_where(&query);
    logging::log(
        "[Logs]",
        &format!(" already by empty {removed} "),
    );
    match serde_json::to_value(stats) {
        Ok(mut value) => {
            if let Some(object) = value.as_object_mut() {
                attach_dictionary(object);
                object.insert("removed".to_string(), json!(removed));
            }
            ok_json(value)
        }
        Err(error) => {
            logging::log("[Logs]", &format!("❌ empty after statisticsfailure: {error}"));
            raw_json(json!({ "success": false, "error": "emptyfailure" }))
        }
    }
}

/// GET /api/logs/download —— JSONL 
pub async fn download_logs(State(_state): State<ServerState>, Query(params): Query<Params>) -> Response {
    let store = match store_or_error() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let query = match log_query(&params) { Ok(query) => query, Err(response) => return response };
    let body = if params.is_empty() { store.to_jsonl() } else { store.to_jsonl_where(&query) };
    let filename = format!("workbuddy-logs-{}.jsonl", file_stamp());

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-ndjson; charset=utf-8")
        .body(Body::from(body))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());

    // file/ use HeaderValue （ ASCII not panic）
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

/// to response up levels / categories / max 
/// （ for Node `{ ...data, levels: LEVELS, categories: CATEGORIES, max: MAX_ENTRIES }`）
fn attach_dictionary(object: &mut serde_json::Map<String, Value>) {
    object.insert(
        "levels".to_string(),
        Value::Array(logs_store::LEVELS.iter().map(|item| Value::String(item.to_string())).collect()),
    );
    let mut categories = serde_json::Map::new();
    for (key, label) in logs_store::CATEGORIES {
        categories.insert(key.to_string(), Value::String(label.to_string()));
    }
    object.insert("categories".to_string(), Value::Object(categories));
    let capacity = logs_store::retention_capacity();
    let warn = logs_store::level_capacity(capacity, "warn");
    let error = logs_store::level_capacity(capacity, "error");
    object.insert("max".to_string(), Value::Number(Number::from(capacity as u64)));
    object.insert("fetchMax".to_string(), json!(logs_store::MAX_ENTRIES));
    object.insert("retention".to_string(), json!({
        "events": logs_store::level_capacity(capacity, "info"),
        "alerts": warn + error,
        "warn": warn,
        "error": error,
        "debug": logs_store::level_capacity(capacity, "debug"),
        "debugDays": logs_store::DEBUG_RETENTION_DAYS,
    }));
    object.insert("debugEnabled".to_string(), json!(logging::is_verbose()));
}

/// file when ： for Node 
/// `new Date().toISOString().replace(/[:.]/g, '-')`（ : and . -），
/// `2026-09-17T13-45-02-123Z` —— in Windows file，must。
fn file_stamp() -> String {
    match chrono::DateTime::from_timestamp_millis(logging::now_ms()) {
        Some(now) => now.format("%Y-%m-%dT%H-%M-%S-%3fZ").to_string(),
        None => "unknown".to_string(),
    }
}
