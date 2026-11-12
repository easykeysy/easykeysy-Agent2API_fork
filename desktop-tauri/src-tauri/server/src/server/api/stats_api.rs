//! API and （set「」）。
//!
//! ```text
//! GET /api/stats/summary （ / / in / ）
//! GET /api/stats/requests request（ + / / / when ）
//! GET /api/stats/requests/filters request down （ / ）
//! GET /api/stats/requests/raw request （ for ）
//! GET /api/stats/requests/clear-preview statistics（ will / / use / ）
//! DELETE /api/stats/requests （mode=raw / mode=all when ，default all）
//! POST /api/stats/requests/compact （checkpoint + VACUUM， after ）
//! GET /api/retention and / requestlimit
//! PUT /api/retention updateset and ****
//! ```
//!
//! ── Why before in 1 file ─────────────────────────────
//! `/api/retention` is ** 1 **（ in `logs_store`、
//! request and in `request_stats`）， its in requeststatistics；setmust
//! ， need `request_stats` —— 1 and ，
//! file will 「 → config → 」 this 。
//! file use `logs_api.rs` / `config_api.rs` `*_api.rs` 。
//!
//! ── parameter （ and logs_api 1 ）────────────────────
//! - `range` → **400**（ and 「 by 7 」 not ）： is use 
//! in up ， 1 will and for not up ，
//! 。 and `report::normalize_range`
//! 1 （ that is 1 ， this is by ）。
//! - `start` / `end` / `limit` / `offset` → **ignore**（）：
//! this is ， before can can empty（「 not when 」）； as 1 parameter not 
//! ， not this no 。
//! - `status` `ok` / `error`， its ignore —— 1 in 
//! （`report::normalize_status_filter`）， this ， not 1 。
//!
//! ── and has API ───────────────────────────────────────
//! is **add** and addoptionalparameter：`/api/logs` `start` / `end`
//! optionalparameter， not when as and before 1 （ `logs_api::query_logs`）。
//!
//! ── provider / account （Agent2API W4：T-e1）────────────
//! by response 1 **provider **， by 
//! （ by not ， by up ）：
//! - `/api/stats/requests`： `provider`（id， old / as empty）
//! and `providerLabel`（id → in ；unknown id id，
//! empty id empty， by use 「—」）， and in `phase` +
//! `phaseElapsedMs`（ and when ， `ui/request-phase.js`）。
//! - `/api/stats/summary`：add `providers` 
//! `[{id,label,requests,success,failures,totalTokens}]`， by requests ，
//! statistics and `overview` / `topModel` 1 ；empty id label as 「unknown」。
//!
//! **account ** and providers and （`accounts` ， and ），
//! is 「 in 」 that 1 —— 1 can ，
//! providers 。：`label` is ** when down **
//! not is （ can can already delete， not to ），「 → id」，
//! empty that 1 label as 「unknown」。
//!
//! ── by provider （add）────────────────────────────
//! `GET` / `DELETE` `provider` parameter（ provider id），
//! and `model` 1 （ `sql::FilterPlan`）—— 「
//! N 」 and 「empty that 」 is 1 。
//! `GET /api/stats/requests/filters`， by that 。
//!
//! up is **add**， has 、、 1 no 
//! （ before by 「providers / accounts in then 、 then 」）。

use std::collections::HashMap;

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use serde_json::{json, Value};

use super::date_range::{date_bounds, parse_day_param};
use crate::server::config::{
    self, RetentionPatch, RetentionSettings, KEY_DAILY_RETENTION_DAYS, KEY_LOG_RETENTION_COUNT,
    KEY_LOG_RETENTION_DAYS, KEY_REQUEST_RETENTION_COUNT, KEY_REQUEST_RETENTION_DAYS,
    RETENTION_MAX_COUNT, RETENTION_MAX_DAYS, RETENTION_MIN_COUNT, RETENTION_MIN_DAYS,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body, parse_query_ms};
use crate::server::logging;
use crate::server::request_stats::RequestQuery;
use crate::server::ServerState;

/// `/api/stats/summary` range （ and `request_stats::report` 
/// `normalize_range` 1 ； that is 1 ， this is by ）。
/// `custom` need `startDate` / `endDate`（ `stats_summary`）。
const RANGES: [&str; 7] = ["today", "yesterday", "7", "30", "month", "all", "custom"];
/// range ： and 「/ by 7 」 default
const DEFAULT_RANGE: &str = "7";

/// `/api/stats*` up path 404。
///
/// `logs_api::not_found` ： API 404 is 
/// （`{success:false,error:"Not found: <METHOD> <path>"}`），
/// and OpenAI （`{error:{message}}`） not 。
pub async fn not_found(request: axum::extract::Request) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    errors::management_error(404, format!("Not found: {method} {path}"))
}

/// （ and logs_api 1 ：HashMap to parameter
/// 「 has then use 、none then ignore」 ）
type Params = HashMap<String, String>;

/// 「valid」 parameter：empty and empty 1 ，return already trim。
///
/// before empty when will `?model=&start=`， this if to down 
/// will 「 etc empty」 this not in （ is empty ）。
/// trim： not will has before after empty， use from empty
/// etc 「」 （ is ）。
fn text_of(params: &Params, key: &str) -> Option<String> {
    params
        .get(key)
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// ：empty / / → 0（ `skip` 0 as ）
fn parse_offset(value: Option<&String>) -> usize {
    let Some(text) = value.map(|text| text.trim()).filter(|text| !text.is_empty()) else {
        return 0;
    };
    text.parse::<usize>().unwrap_or(0)
}

/// ：empty / / 0 → None（ use default 50）。
///
/// 0 「 no 」 not is 「 need 0 」（ and logs_api `0 || 200` 1 ）；
/// limit in （`query_requests` `clamp(1, MAX_LIMIT)`， 500），
/// this —— has 1 ， 1 will 。
fn parse_limit(value: Option<&String>) -> Option<usize> {
    let text = value.map(|text| text.trim()).filter(|text| !text.is_empty())?;
    text.parse::<usize>().ok().filter(|size| *size > 0)
}

/// GET /api/stats/summary?range=7&startDate=&endDate=
///
/// `range=custom` when must `startDate` / `endDate`（`YYYY-MM-DD`，，
/// ）： by use in up ， / / 1 
/// 400 —— and `range` 1 （ 1 will and for not up ，
/// ）。
pub async fn stats_summary(State(state): State<ServerState>, Query(params): Query<Params>) -> Response {
    // trim for ： `normalize_range` is trim after ，
    // if this for trim ，`?range=%207%20` will 400 can ——
    // not 1 when ， use to 400 will its 
    let requested = text_of(&params, "range").unwrap_or_else(|| DEFAULT_RANGE.to_string());
    // in by ： to 400 and （file ）
    if !RANGES.contains(&requested.as_str()) {
        return errors::management_error(
            400,
            format!("range 取值非法: {requested}（合法值: {}）", RANGES.join("、")),
        );
    }
    let custom = if requested == "custom" {
        let start = match parse_day_param(&params, "startDate") {
            Ok(value) => value,
            Err(message) => return errors::management_error(400, message),
        };
        let end = match parse_day_param(&params, "endDate") {
            Ok(value) => value,
            Err(message) => return errors::management_error(400, message),
        };
        if start > end {
            return errors::management_error(
                400,
                format!(
                    "开始日期不能晚于结束日期（startDate={}, endDate={}）",
                    start, end
                ),
            );
        }
        Some((start, end))
    } else {
        None
    };
    // （`{range, startDate, endDate, overview, providers,
    // heatmap, cacheRates, cacheTrend24h, dailyTrend}`）， by not ，。
    // `providers` is W4 add provider total（file ）
    //
    // ── Why in `spawn_blocking` ──────────────────────────
    // `usage_summary` is **** （SQLite + ）， and statistics
    // that 。 in async handler use will 1 tokio worker 
    // to —— is before （defaultseconds 1 、 can in when ），
    // this 「」 not this in async when up 。 and `backup_api` /
    // `upgrade_api` 1 。
    //
    // （ and 1 `Db` ） not this 1 —— that by 「
    // 24 when + in down SQL」 in **** up （
    // `RequestStats::usage_summary` ）； this is 「
    // runtime 」 this 1 。
    let api_key = text_of(&params, "apiKey");
    if api_key.as_deref().is_some_and(|value| crate::server::request_stats::ApiKeyIdentity::from_filter(value).is_none()) {
        return errors::management_error(400, "apiKey 取值非法（需要 key:<id>、env、none 或 unknown）");
    }
    let stats = state.request_stats();
    let result = tokio::task::spawn_blocking(move || stats.usage_summary(&requested, custom, api_key.as_deref())).await;
    match result {
        Ok(value) => ok_json(value),
        // （runtime in ） or panic： to 1 can 500 not is 
        // axum 1 none body error to before 
        Err(error) => errors::management_error(500, format!("报表计算任务失败: {error}")),
    }
}

/// GET /api/stats/requests?offset=&limit=&model=&provider=&status=&start=&end=
///
/// `provider`（id） and `providerLabel`（）， by `entry_json`
/// in after ； when 1 `phaseElapsedMs`（ in 「current already 
/// 」， not in as null）—— and `phase` is request 
/// 「 + when 」（ `ui/request-phase.js`）。
///
/// `provider` and `model` is ****（ down is ，
/// not —— will 「 A to B」none）。
///
/// response `running`： 1 down in in （status=0） ——
/// when will 1 this （ `RequestStats::record_started`），
/// after by 。status `running` （ in ）。
pub async fn stats_requests(State(state): State<ServerState>, Query(params): Query<Params>) -> Response {
    let filter = match checked_filter(&params) { Ok(value) => value, Err(response) => return response };
    ok_json(state.request_stats().query_requests(&filter))
}

/// from —— GET / DELETE / clear-preview by use 1 
/// ： 1 ， （「 N 」 and 「 
/// that 」「 N 」must is 1 ）。
fn filter_from_params(params: &Params) -> RequestQuery {
    RequestQuery {
        offset: parse_offset(params.get("offset")),
        limit: parse_limit(params.get("limit")),
        // ；empty（empty） no 
        model: text_of(&params, "model"),
        // provider id ；empty no （ RequestQuery::provider）
        provider: text_of(&params, "provider"),
        client: params.get("client").map(|value| value.trim().to_string()),
        api_key: text_of(&params, "apiKey"),
        production_only: params.get("productionOnly").is_some_and(|value| value == "1"),
        completed_only: params.get("completedOnly").is_some_and(|value| value == "1"),
        // ok / error / running， its in ignore（ not in this 1 ）
        status: text_of(&params, "status"),
        // [start, end)： when up 1 ts can down end
        start: parse_query_ms(params.get("start")),
        end: parse_query_ms(params.get("end")),
    }
}

fn checked_filter(params: &Params) -> Result<RequestQuery, Response> {
    let mut filter = filter_from_params(params);
    if filter.api_key.as_deref().is_some_and(|value| crate::server::request_stats::ApiKeyIdentity::from_filter(value).is_none()) {
        return Err(errors::management_error(400, "apiKey 取值非法（需要 key:<id>、env、none 或 unknown）"));
    }
    if let Some((start, end)) = date_bounds(params).map_err(|message| errors::management_error(400, message))? {
        filter.start = Some(start);
        filter.end = Some(end);
    }
    Ok(filter)
}

/// GET /api/stats/requests/filters —— down 。
///
/// `{models: […], providers: [{id, label}…]}`， by 
/// （ use before ）。** **， not is current config ——
/// request use / already delete provider need can to ，otherwise will 
/// 「 has this 1 、 down no has this 」 this not 1 。
///
/// not parameter： and when none（ by `select_filter_options`）。
pub async fn stats_request_filters(State(state): State<ServerState>) -> Response {
    ok_json(state.request_stats().filter_options())
}

/// DELETE /api/stats/requests?mode=raw|all&model=&provider=&status=&start=&end=&all=
///
/// `raw` in request up and down ， and ；
/// `all` when delete in ，none when 。 has when ， as 
/// by remaining will before already request 。
///
/// none when must `all=1`， and empty 。
/// `deleted` in raw as by ID request， in all as ；
/// `raw` as by ID request，`upstreamRaw` / `downstreamRaw` as 。
pub async fn clear_stats_requests(
    State(state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let mode = match params.get("mode").map(|value| value.trim()) {
        Some("raw") => "raw",
        // by all： and 「 not mode old use 」 as 
        Some("all") | None => "all",
        Some(other) => {
            return errors::management_error(
                400,
                format!("mode 取值非法: {other}（合法值: raw、all）"),
            )
        }
    };
    let filter = match checked_filter(&params) { Ok(value) => value, Err(response) => return response };
    let has_filters = filter.has_filters();
    if !has_filters {
        let explicit_all = params.get("all").map_or(false, |value| {
            let text = value.trim();
            text == "1" || text.eq_ignore_ascii_case("true")
        });
        if !explicit_all {
            return errors::management_error(
                400,
                "未指定筛选条件：要清空全部请求数据请显式带 all=1",
            );
        }
    }
    let stats = state.request_stats();
    let result = tokio::task::spawn_blocking(move || stats.clear_requests(&filter, mode == "raw")).await;
    let cleared = match result {
        Ok(Some(value)) => value,
        Ok(None) => return errors::management_error(500, "请求数据清理失败：数据库不可用或事务执行失败"),
        Err(error) => return errors::management_error(500, format!("请求数据清理任务失败: {error}")),
    };
    let deleted = cleared.deleted;
    match (mode, has_filters) {
        ("raw", _) => logging::log(
            "[Stats]",
            &format!(
                "已清空 {deleted} 个请求的原始报文（上游 {} 条、下游 {} 条；明细与按天聚合不受影响）",
                cleared.upstream_raw, cleared.downstream_raw,
            ),
        ),
        ("all", true) => logging::log(
            "[Stats]",
            &format!("已按筛选条件删除 {deleted} 条请求明细与两侧原始报文（按天聚合未重算）"),
        ),
        _ => logging::log("[Stats]", "请求统计已清空（明细 + 按天聚合 + 两侧原始报文）"),
    }
    let mut body = json!({
        "success": true,
        "mode": mode,
        "deleted": deleted,
        "raw": cleared.raw,
        "downstreamRaw": cleared.downstream_raw,
        "upstreamRaw": cleared.upstream_raw,
    });
    if mode == "all" && has_filters {
        body["note"] = Value::String(
            "已删除命中的请求明细与对应的上游、下游原始报文；按天聚合是全量数据，未按筛选重算，报表数字仍包含已删除的请求".to_string(),
        );
    }
    ok_json(body)
}

/// GET /api/stats/requests/raw?id=... —— request 。
///
/// （ for ） ：`{id, requestBody, responseBody, truncated}`，
/// return，`truncated` only old can can not 。
/// not in to 404 —— in **** 
/// request up （ when not ， `api::pipeline::capture_raw_body`），
/// can can already ； is ， not is error。
pub async fn stats_request_raw(
    State(state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let Some(id) = text_of(&params, "id") else {
        return errors::management_error(400, "缺少 id 参数");
    };
    match state.request_stats().raw_body(&id) {
        Some(value) => ok_json(value),
        None => errors::management_error(404, format!("未找到该请求的原始报文（id: {id}）")),
    }
}

/// GET /api/stats/requests/clear-preview —— statistics。
///
/// and DELETE use 1 。`all` is in ，`raw` is 
/// by ID request，`upstreamRaw` / `downstreamRaw` is ；
/// `dbBytes` is file + WAL use ， not by 。
pub async fn stats_clear_preview(
    State(state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let filter = match checked_filter(&params) { Ok(value) => value, Err(response) => return response };
    let stats = state.request_stats();
    match tokio::task::spawn_blocking(move || stats.clear_preview(&filter)).await {
        Ok(Some(value)) => ok_json(value),
        Ok(None) => errors::management_error(500, "清理预览读取失败：数据库不可用或查询失败"),
        Err(error) => errors::management_error(500, format!("清理预览任务失败: {error}")),
    }
}

/// POST /api/stats/requests/compact —— （checkpoint + VACUUM）。
///
/// after （ by `RequestStats::compact`），response「 is 」：
/// already to `{success, started:true}`， before clear-preview `vacuumRunning`
/// / `lastVacuum` ； to 409（ 1 when 1 ）；
/// unavailable， not 「 already 」。
pub async fn compact_stats_db(State(state): State<ServerState>) -> Response {
    match state.request_stats().compact() {
        crate::server::request_stats::CompactStart::Started => {
            ok_json(json!({ "success": true, "started": true }))
        }
        crate::server::request_stats::CompactStart::AlreadyRunning => {
            errors::management_error(409, "数据库压缩正在进行中，请稍后再试")
        }
        crate::server::request_stats::CompactStart::Unavailable => {
            errors::management_error(500, "数据库不可用，无法压缩")
        }
    }
}

/// POST /api/stats/requests/terminate?id=... —— 1 ** in **request。
///
/// OmniProxy `POST /api/request-logs/:id/terminate`，：
/// 1 —— this request （`core::upstream::cancellation`）。 
/// （ up 、 to clienterror、 408 + 「request already 」）
/// by path —— ** has that 1 path**
/// （ and 「 in to 1 not this 」 1 ： not ）。
///
/// not in in to 404（ already / up / id ）：
/// before 「 already or already new 」，「 already 」。
/// by etc （ is to ）： use return —— can can 
/// in in ， not this error。
pub async fn stats_request_terminate(
    State(_state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let Some(id) = text_of(&params, "id") else {
        return errors::management_error(400, "缺少 id 参数");
    };
    if crate::server::core::upstream::cancellation::cancel(&id) {
        ok_json(json!({ "success": true, "terminated": true }))
    } else {
        errors::management_error(
            404,
            "这条请求已不在进行中（可能已结束，或来自上次启动遗留的进行中行）",
        )
    }
}

/// GET /api/retention
pub async fn get_retention(State(_state): State<ServerState>) -> Response {
    ok_json(retention_json(config::retention_settings()))
}

/// PUT /api/retention —— supportsupdate and logRetentionCount / requestLogRetentionCount。
///
/// （ ）。 after ： config.json → 
/// → return** after ** （ before use response new ， not GET 1 ）。
///
/// need ： in 「 / 」 when ，
/// use 30 7 after if not ， old need etc to down 1 request or down ——
/// that when will 「set is 7 ， has 30 before 」， no 。
pub async fn put_retention(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };

    // ， 1 not 。
    let mut patch = RetentionPatch::default();
    let targets = [
        (
            KEY_LOG_RETENTION_DAYS,
            &mut patch.log_days,
            RETENTION_MIN_DAYS,
            RETENTION_MAX_DAYS,
            "天",
        ),
        (
            KEY_REQUEST_RETENTION_DAYS,
            &mut patch.request_days,
            RETENTION_MIN_DAYS,
            RETENTION_MAX_DAYS,
            "天",
        ),
        (
            KEY_DAILY_RETENTION_DAYS,
            &mut patch.daily_days,
            RETENTION_MIN_DAYS,
            RETENTION_MAX_DAYS,
            "天",
        ),
        (
            KEY_LOG_RETENTION_COUNT,
            &mut patch.log_count,
            RETENTION_MIN_COUNT,
            RETENTION_MAX_COUNT,
            "条",
        ),
        (
            KEY_REQUEST_RETENTION_COUNT,
            &mut patch.request_count,
            RETENTION_MIN_COUNT,
            RETENTION_MAX_COUNT,
            "条",
        ),
    ];
    for (key, slot, min, max, unit) in targets {
        let Some(value) = object.get(key).filter(|value| !value.is_null()) else {
            continue;
        };
        match parse_retention_value(key, value, min, max, unit) {
            Ok(value) => *slot = Some(value),
            Err(message) => return errors::management_error(400, message),
        }
    }

    // empty PUT returncurrent， not 、 not 、 not update。
    let updated = patch.log_days.is_some()
        || patch.log_count.is_some()
        || patch.request_days.is_some()
        || patch.request_count.is_some()
        || patch.daily_days.is_some();
    if updated {
        if !config::set_retention(patch) {
            // failure： already update（）， after will to old ——
            // must use ，otherwise「set」 will 
            logging::log("[Config]", "⚠️  数据保留设置写入失败，本次运行内仍生效");
        }
        // ── （ when not ， this 1 ）──
        // requeststatistics：prune() ****， to is 
        state.request_stats().prune();
        // ：LogStore （ `LogStore::with_db` ），
        // prune return ， 1 「」
        if let Some(store) = logging::store_ref() {
            let removed = store.prune();
            if removed > 0 {
                logging::console_line("[Logs]", &format!("按保留设置清理了 {removed} 条日志"));
            }
        }
    }

    let settings = config::retention_settings();
    if updated {
        logging::log(
            "[Config]",
            &format!(
                "数据保留设置已更新: 日志 {} 天 / {} 条，请求日志 {} 天 / {} 条，按天聚合 {} 天",
                settings.log_days,
                settings.log_count,
                settings.request_days,
                settings.request_count,
                settings.daily_days
            ),
        );
    }
    ok_json(retention_json(settings))
}

/// set response（GET and PUT use ， 1 ）。
///
/// use **** not is ：`json!` will 
/// `.into()` ， this to JSON is （`"logRetentionDays"` etc ）。
/// is in `config.rs` 1 ，（`days_field`）、
/// （`set_retention`） and response 1 —— ， 1 
/// not will ， will 1 。
fn retention_json(settings: RetentionSettings) -> Value {
    json!({
        KEY_LOG_RETENTION_DAYS: settings.log_days,
        KEY_LOG_RETENTION_COUNT: settings.log_count,
        KEY_REQUEST_RETENTION_DAYS: settings.request_days,
        KEY_REQUEST_RETENTION_COUNT: settings.request_count,
        KEY_DAILY_RETENTION_DAYS: settings.daily_days,
    })
}

fn parse_retention_value(
    key: &str,
    value: &Value,
    min: i64,
    max: i64,
    unit: &str,
) -> Result<i64, String> {
    config::parse::int_in_range(value, min, max)
        .ok_or_else(|| format!("{key} 必须是 {min}-{max} {unit}范围内的整数（收到: {value}）"))
}
