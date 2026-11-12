//! to by （ to in 「 to 」 ）。
//!
//! ```text
//! GET /api/checkin-records （?limit=&offset=&outcome=&kind=&provider=&keyword=&start=&end=）
//! DELETE /api/checkin-records delete（ = in ；must all=1）
//! ```
//!
//! ── response ────────────────────────────────────────────────
//! GET → `{records: [...], total}`：total is **** total（ use ），
//! records by when 。DELETE → `{success:true, removed}`（removed = delete）。
//!
//! ── DELETE （ logs then ）────────────────────────
//! 1 parameter when in （ and GET 1 —— up 
//! ）；emptymust**** `all=1`， not emptyrequest to 400。 
//! by `api::logs_api::clear_logs`：emptyparameter 
//! ， use 1 parameter this 1 400。
//!
//! ── parameter ────────────────────────────────────────────────
//! and / 1 （`api::logs_api` ）： is 
//! ，（empty / ）**ignore not is **；limit in 
//! 1..1000、 200。start / end is ms when （`http::parse_query_ms`）。

use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::response::Response;
use serde_json::json;

use crate::server::core::checkin_records;
use crate::server::http::{ok_json, parse_query_ms};
use crate::server::ServerState;

type Params = HashMap<String, String>;

/// usize（empty / → None）
fn parse_usize(value: Option<&String>) -> Option<usize> {
    let text = value?.trim();
    if text.is_empty() {
        return None;
    }
    text.parse::<usize>().ok()
}

/// → （ outcome / kind / provider / keyword / start / end ）
fn query_of(params: &Params) -> checkin_records::Query {
    checkin_records::Query {
        outcome: params.get("outcome").cloned(),
        kind: params.get("kind").cloned(),
        provider: params.get("provider").cloned(),
        keyword: params.get("keyword").cloned(),
        start: parse_query_ms(params.get("start")),
        end: parse_query_ms(params.get("end")),
    }
}

/// GET /api/checkin-records —— 1 + total
pub async fn query_records(
    State(_state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let limit = parse_usize(params.get("limit")).unwrap_or(200);
    let offset = parse_usize(params.get("offset")).unwrap_or(0);
    ok_json(checkin_records::query(&query_of(&params), limit, offset))
}

/// DELETE /api/checkin-records —— by delete / （module header）
pub async fn clear_records(
    State(_state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let query = query_of(&params);
    if !query.has_filters() {
        let explicit = params
            .get("all")
            .map(|value| value.trim() == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if !explicit {
            return crate::server::errors::management_error(
                400,
                "未指定筛选条件：要清空全部签到记录请显式带 all=1",
            );
        }
        let removed = checkin_records::clear_all().unwrap_or(0);
        crate::server::logging::log("[Checkin]", &format!("签到记录已全部清空（{removed} 条）"));
        return ok_json(json!({ "success": true, "removed": removed }));
    }
    match checkin_records::clear_where(&query) {
        Some(removed) => ok_json(json!({ "success": true, "removed": removed })),
        // unavailable： no not ，（ as empty is ，
        // delete will success）
        None => crate::server::errors::management_error(500, "签到记录删除失败（库不可用）"),
    }
}
