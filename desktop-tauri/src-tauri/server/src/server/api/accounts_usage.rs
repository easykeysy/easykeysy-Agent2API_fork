//! `/api/accounts/usage` （****： already down to 
//! `core::usage_query`， that module header）。
//!
//! file： `?id=`、 core 、 `TargetError` 
//! response。、 and 、failure、401 new in core
//! ——「」 1 ， not axum。
//!
//! ── `?id=` and is ─────────────────────────────
//! not `id` = （ is 「**available**」， 「」）；
//! `id` = this 1 （ 1 「」 by this ）。
//! ** not enabled**： use 「 not and 」， and its can none ——
//! by enabled ， will that in 「」， use can 
//! （ that is 「 no 」 ）。 not （available:false）
//! in ，`skipped` is 。
//! in `core::usage_query::query_all` `id` parameter up （ that 1 has ）。
//!
//! ── 1 ： ─────────────────────────
//! `GET /api/accounts/usage/snapshot` （failure），
//! in ** not by ** down up （ to down ，
//! 1 can to new ）。

use axum::response::Response;

use crate::server::core::usage_query;
use crate::server::errors::management_error;
use crate::server::http::{ok_json, query_param};
use crate::server::ServerState;

/// GET /api/accounts/usage
///
/// and / total（`{ results: [{id,name,usage,error,code?}], skipped }`）。
/// use 「」 this ；`?id=` then that 1 （「」 by ）。
/// as `core::usage_query::query_all`。
pub async fn accounts_usage(state: &ServerState, query: &str) -> Response {
    // empty and （ before `''` when not id）， this filter empty down 
    let id = query_param(query, "id").filter(|value| !value.is_empty());
    match usage_query::query_all(state.store(), id.as_deref()).await {
        Ok(report) => ok_json(report),
        Err(error) => management_error(error.status_code as i32, error.message),
    }
}

/// GET /api/accounts/usage/snapshot
///
/// 1 
/// （`{ at, results: [{id,name,usage,error,code?,at}], skipped }`，`at` is ms
/// when 、0 no ； up `at` is ** this 1 ** when ）。
///
/// and `/api/accounts/usage` response（ 1 `at`）：
/// before can use 1 `applyBalances` ， not as 「」 and 「」
/// 1 。 `at` is by to need —— when 
/// not ，failure when must by 。
///
/// need up ： will 「 already delete」 、 and 「 old 」
/// failure（ `core::usage_query::snapshot` ）。
pub async fn accounts_usage_snapshot(state: &ServerState) -> Response {
    ok_json(usage_query::snapshot(state.store()))
}
