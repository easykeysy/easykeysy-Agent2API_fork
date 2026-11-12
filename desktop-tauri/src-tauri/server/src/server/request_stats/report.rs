//! —— for ****， not 、 not 。
//!
//! Why 1 ： is （、、
//! 、 and 、 when ）， in in 1 can 1 「 1 / 1 when 」
//! is 1 ； after when this 1 。
//!
//! ── ──────────────────────────────────────────
//! - `overview` / `dailyTrend` / `heatmap` / `streak` ****：
//! this need （ 365 、`all` can can 1 up ），
//! 30 ， will 。
//! - `cacheRates` **SQL **（`sql::sum_cache_windows` 1 
//! ， and ）： 7 ，「 by and 」
//! no has —— ， is 1 。
//! - `cacheTrend24h` ****： need to ， by to not this 
//! ； 24 when （`sql::select_summary_rows`）。

use std::collections::{BTreeMap, HashMap};

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate};
use serde_json::{json, Value};

use super::clock::{date_key, hour_floor, hour_key, ms_to_local, now_ms};
use super::record::{AccountAccum, DailyEntry, ModelAccum, ProviderAccum, RequestEntry};

/// unknown provider （`id` as empty that 1 ： old 、
/// and 「 1 no failure」 request）。 before will use 。
pub(super) const UNKNOWN_PROVIDER_LABEL: &str = "未知";

/// unknown （ no has that 1 ）。
///
/// and provider that is ** ** not is use 1 ： 「unknown」
/// not （ before is no down ， after is no ）， will if ，
/// use will 1 etc 。
pub(super) const UNKNOWN_ACCOUNT_LABEL: &str = "未知账号";

/// return （ and range ）
pub(super) const HEATMAP_DAYS: i64 = 365;

/// need ****（`sql::SUMMARY_COLUMNS` ）。
///
/// Why need has this （ not use `RequestEntry`）： for has 
/// 「 by + by 」， use to is this 6 。 up 
/// `RequestEntry` need by 25 （`route` / `attempt_details`
/// in ）， this 1 use not up —— 1 ， that 1 no 
/// 「」（ `sql::select_summary_rows` and `db::schema` v15 ）。
#[derive(Clone, Debug)]
pub(super) struct SummaryRow {
    pub ts: i64,
    pub model: String,
    /// up （empty = 1 no ），statistics 
    /// [`model_stat_key`]
    pub upstream_model: String,
    pub total_tokens: i64,
    pub cache_read_tokens: i64,
    pub prompt_tokens: i64,
}

/// in ：**JSON + ms**。
///
/// in 1 is must ：`cache_rates` by this SQL that 
/// （`sql::sum_cache_windows` return and 1 1 for ）， is 
/// before `cacheRates.last10m` etc —— （ 1 、 1 
/// ms） will 「 」， and from up 。
pub(super) const CACHE_WINDOWS: [(&str, i64); 4] = [
    ("last10m", 10 * 60_000),
    ("last1h", 60 * 60_000),
    ("last24h", 24 * 60 * 60_000),
    ("last7d", 7 * 24 * 60 * 60_000),
];

/// / limit（ ）
const MAX_TREND_DAYS: i64 = 4000;

/// when 
const RANGE_TODAY: &str = "today";
/// 「」： 0 24 （ and `today` use 1 ）
const RANGE_YESTERDAY: &str = "yesterday";
const RANGE_7: &str = "7";
const RANGE_30: &str = "30";
const RANGE_MONTH: &str = "month";
const RANGE_ALL: &str = "all";
/// ： by `startDate` / `endDate` to （ by already ），
/// this 1 and that for up 。
pub(super) const RANGE_CUSTOM: &str = "custom";

/// range 1 ： by `"7"`（ `usage_summary` ）
pub(super) fn normalize_range(range: &str) -> &'static str {
    match range.trim() {
        RANGE_TODAY => RANGE_TODAY,
        RANGE_YESTERDAY => RANGE_YESTERDAY,
        RANGE_30 => RANGE_30,
        RANGE_MONTH => RANGE_MONTH,
        RANGE_ALL => RANGE_ALL,
        RANGE_CUSTOM => RANGE_CUSTOM,
        _ => RANGE_7,
    }
}

/// （ `[startDate, endDate]`）。
///
/// - `today` is ；`yesterday` is 1 （ 0 ~ 24 ，
/// and `today` use 1 ）
/// - `7` / `30` from before （， days-1）
/// - `month` as 1 to （ when ）
/// - `all` 1 ；** no has ** when to 「 before 29 」，
/// before to 1 not is empty or null
/// - `custom` use use to （ by already `YYYY-MM-DD`）；
/// `custom` no （ not this ， by will ） when by `7` ——
/// can to 1 can ， not need 500
///
/// 1 （ / when this ）：
/// - `start > end` when ： to will before when ；
/// - `end` to ： no has ， 1 「」 
/// will and （ not will has ）。
pub(super) fn range_bounds(
    range: &str,
    daily: &BTreeMap<String, DailyEntry>,
    today: NaiveDate,
    custom: Option<(NaiveDate, NaiveDate)>,
) -> (String, String) {
    if range == RANGE_CUSTOM {
        if let Some((custom_start, custom_end)) = custom {
            let (custom_start, custom_end) = if custom_start > custom_end {
                (custom_end, custom_start)
            } else {
                (custom_start, custom_end)
            };
            let end = custom_end.min(today);
            let start = custom_start.min(end);
            return (date_key(start), date_key(end));
        }
    }
    // 「」 is 1 end not is ： = 0 to 24 。
    // `end = today` ， 。
    if range == RANGE_YESTERDAY {
        let yesterday = today - ChronoDuration::days(1);
        return (date_key(yesterday), date_key(yesterday));
    }
    let end = today;
    let start = match range {
        RANGE_TODAY => today,
        RANGE_30 => today - ChronoDuration::days(29),
        RANGE_MONTH => today.with_day(1).unwrap_or(today),
        RANGE_ALL => daily
            .keys()
            .next()
            .and_then(|key| NaiveDate::parse_from_str(key, "%Y-%m-%d").ok())
            .unwrap_or_else(|| today - ChronoDuration::days(29)),
        // RANGE_7 and to this 
        _ => today - ChronoDuration::days(6),
    };
    // ： has 「」 （ when before ），start not can end，
    // otherwise is to ， before will when 
    let start = if start > end { end } else { start };
    (date_key(start), date_key(end))
}

/// total and by / by provider / by （overview ）
pub(super) struct RangeTotals {
    pub requests: i64,
    pub successful: i64,
    pub tokens: i64,
    pub active_days: i64,
    pub model_totals: Vec<ModelAccum>,
    pub provider_totals: Vec<ProviderAccum>,
    pub account_totals: Vec<AccountAccum>,
}

/// by 。`BTreeMap::range` by ——
/// when ， this is `BTreeMap` 。
pub(super) fn range_totals(
    daily: &BTreeMap<String, DailyEntry>,
    start: &str,
    end: &str,
) -> RangeTotals {
    let mut totals = RangeTotals {
        requests: 0,
        successful: 0,
        tokens: 0,
        active_days: 0,
        model_totals: Vec::new(),
        provider_totals: Vec::new(),
        account_totals: Vec::new(),
    };
    // [start, end]： when ， by 。
    // this String is has —— use is 「 use 1 down 」，
    // not `range` 1 need 。
    for (_, day) in daily.range(start.to_string()..=end.to_string()) {
        totals.requests += day.requests;
        totals.successful += day.successful;
        totals.tokens += day.tokens;
        // activeDays **** has request （ not 「」）
        if day.requests > 0 {
            totals.active_days += 1;
        }
        for acc in &day.model_tokens {
            push_model_accum(&mut totals.model_totals, &acc.model, acc.tokens, acc.requests);
        }
        for acc in &day.provider_stats {
            push_provider_accum(
                &mut totals.provider_totals,
                &acc.provider,
                acc.requests,
                acc.successful,
                acc.tokens,
            );
        }
        for acc in &day.account_stats {
            push_account_accum(
                &mut totals.account_totals,
                &acc.account_id,
                &acc.account_name,
                acc.requests,
                acc.successful,
                acc.tokens,
            );
        }
    }

    // ── old ──────────────────────────────────────
    // before no has `providerStats` （ after is empty）， that request
    // no has provider 。 this 「 」empty id ：
    // ① provider and etc total —— can for ， before 
    // `providers` and `overview.requests` and when not will for not up ；
    // ② old 「unknown」 not is —— need not to old 
    // ****（ not workbuddy），「unknown」 is ， not is 。
    // and use `&mut`： not can use when not can when this 。
    let covered_requests: i64 = totals.provider_totals.iter().map(|item| item.requests).sum();
    let covered_successful: i64 = totals.provider_totals.iter().map(|item| item.successful).sum();
    let covered_tokens: i64 = totals.provider_totals.iter().map(|item| item.tokens).sum();
    // `max(0)` file 「total」； when this will 
    // 1 empty， `build_providers` will （ that `requests > 0
    // || tokens > 0`）， not will emptyempty 「unknown」
    push_provider_accum(
        &mut totals.provider_totals,
        "",
        (totals.requests - covered_requests).max(0),
        (totals.successful - covered_successful).max(0),
        (totals.tokens - covered_tokens).max(0),
    );

    // ── ────────────────────────────────────────────
    // `accountStats` is add ，**all** already has no has ， this 1 
    // in this provider that 1 in ： after 1 ，
    // request will 「unknown」。 this is 「 that no 」，
    // not is —— its has ， need ， can by 。
    let covered_requests: i64 = totals.account_totals.iter().map(|item| item.requests).sum();
    let covered_successful: i64 = totals.account_totals.iter().map(|item| item.successful).sum();
    let covered_tokens: i64 = totals.account_totals.iter().map(|item| item.tokens).sum();
    push_account_accum(
        &mut totals.account_totals,
        "",
        "",
        (totals.requests - covered_requests).max(0),
        (totals.successful - covered_successful).max(0),
        (totals.tokens - covered_tokens).max(0),
    );
    totals
}

/// 1 request（ or 1 ） and by 。
/// can ： is to ， HashMap 。
pub(super) fn push_model_accum(list: &mut Vec<ModelAccum>, model: &str, tokens: i64, requests: i64) {
    match list.iter_mut().find(|item| item.model == model) {
        Some(item) => {
            item.tokens += tokens;
            item.requests += requests;
        }
        None => list.push(ModelAccum {
            model: model.to_string(),
            requests,
            tokens,
        }),
    }
}

/// by use **statistics**： up ，request。
///
/// ── Why not can use `model` ────────────────────────────────
/// after （ `pipeline::resolve_model` ），request
/// client， down to by （`payload::send_body` →
/// `catalog::wire_target_for_provider`）。 is `model`（request） in 
/// client when is **** —— if by ， 1 up 
/// will 「 + 」。 is ，request is up 
/// that 1 ：`upstream_model`（telemetry in down ） is 
/// 「usage in up 」 。
///
/// ── ──────────────────────────────────────────────────
/// before failure request 1 no ，`upstream_model` as empty —— request is 
/// when （token already ，request）。 no has 
/// `upstreamModel` old ： this not is 「 to old 」，empty
/// is has 。
///
/// has （ and = total before in this ）：
/// （`aggregate::fold_into_daily`）、 24 when （file `cache_trend_24h`）
/// and （`aggregate::rebuild_day` before ），**must 1 **。
///
/// ── Why &str not is &RequestEntry ───────────────────
/// 1 in need ： `RequestEntry`（ by ） and 
/// [`SummaryRow`]（ 24 when ， 6 ， `sql::select_summary_rows`）。
/// if `&RequestEntry`， that 1 can 1 `if upstream empty
/// ` —— that is this need 「 1 」。 not is ，
/// use can use 。
pub(super) fn model_stat_key<'a>(model: &'a str, upstream_model: &'a str) -> &'a str {
    if upstream_model.is_empty() {
        model
    } else {
        upstream_model
    }
}

/// → JSON （ by and 24 when use ）。
///
/// `[{model, label, tokens, requests}]`， by tokens （ and requests，
/// and ， 1 use 1 ）。
///
/// and `build_models` ：
/// - is `tokens > 0`（ this to use ， 1 token as 0
/// no has can ；request is ， `fold_into_daily` ）；
/// - emptyreturn（label as 「unknown」）： usage not this from 
/// ，otherwise and and total for not up 。
pub(super) fn model_breakdown_json(list: &[ModelAccum]) -> Vec<Value> {
    let mut rows: Vec<&ModelAccum> = list.iter().filter(|item| item.tokens > 0).collect();
    rows.sort_by(|left, right| {
        right
            .tokens
            .cmp(&left.tokens)
            .then(right.requests.cmp(&left.requests))
            .then(left.model.cmp(&right.model))
    });
    rows.into_iter()
        .map(|item| {
            json!({
                "model": item.model,
                "label": if item.model.is_empty() {
                    UNKNOWN_MODEL_LABEL.to_string()
                } else {
                    item.model.clone()
                },
                "tokens": item.tokens,
                "requests": item.requests,
            })
        })
        .collect()
}

/// 1 （ 1 or 1 ）request and by provider 。
///
/// and `push_model_accum` 1 ：provider is ， HashMap
/// （ not ）。`provider` as empty when ****， not ——
/// old and 「 before failure」 request in this 1 ， will 
/// 「 provider and 」 for not up total。
pub(super) fn push_provider_accum(
    list: &mut Vec<ProviderAccum>,
    provider: &str,
    requests: i64,
    successful: i64,
    tokens: i64,
) {
    match list.iter_mut().find(|item| item.provider == provider) {
        Some(item) => {
            item.requests += requests;
            item.successful += successful;
            item.tokens += tokens;
        }
        None => list.push(ProviderAccum {
            provider: provider.to_string(),
            requests,
            successful,
            tokens,
        }),
    }
}

/// 1 （ 1 or 1 ）request and by 。
///
/// ── account_id ─────────────────────────────────────
/// by `account_id` ，** not and etc **： can in ，
/// by will 1 in before after ， 
/// 「 1 」。 in this 1 ：** when down **。
/// after 1 if new ，update（ after new ，
/// this is ）， not is old 。
///
/// ── empty id ─────────────────────────────────────────────
/// and `push_provider_accum` 1 ：`account_id` as empty when ****， not ——
/// default request、 and old in this 1 ，
/// will 「 and 」 for not up total。
pub(super) fn push_account_accum(
    list: &mut Vec<AccountAccum>,
    account_id: &str,
    account_name: &str,
    requests: i64,
    successful: i64,
    tokens: i64,
) {
    match list.iter_mut().find(|item| item.account_id == account_id) {
        Some(item) => {
            item.requests += requests;
            item.successful += successful;
            item.tokens += tokens;
            // 「 new 」：empty， not empty already has 
            // （request not to 1 to ， has that 1 ）
            if !account_name.is_empty() {
                item.account_name = account_name.to_string();
            }
        }
        None => list.push(AccountAccum {
            account_id: account_id.to_string(),
            account_name: account_name.to_string(),
            requests,
            successful,
            tokens,
        }),
    }
}

/// provider id → label。
///
/// `providers::label_of` —— 「id → to 」 1 ，
/// ** not **（ is provider 1 ，，
/// this not will ， will and for not up ）。 1 
/// ： provider `meta().label`；`custom-` before 
/// use —— that is （config ， not ），
/// will `custom-…`。
///
/// / already delete id by ： 1 id empty，
/// can 「 this is 」。emptyreturn（empty label）， by use 
/// （current is 「unknown」/「—」）—— this 1 「id for 」，
/// not 。
pub(super) fn provider_label(id: &str) -> String {
    crate::server::core::providers::label_of(id)
}

/// by provider total → `/api/stats/summary` `providers` 。
///
/// ：`requests` （ need ）； and when by `totalTokens` 、
/// and by id —— after as （ 1 use 1 ），
/// before 「 before N 」 when not will to not 。
///
/// empty id that 1 label use 「unknown」 not is empty： this is ** to use **，
/// need has can ；empty label in is not empty。
/// （ that empty provider empty label， use 「—」 ——
/// not is as use not ， not is 。）
pub(super) fn build_providers(totals: &[ProviderAccum]) -> Vec<Value> {
    // not return： will in file， 1 「0 request」 
    // no has （path not will this ）
    let mut rows: Vec<&ProviderAccum> = totals
        .iter()
        .filter(|item| item.requests > 0 || item.tokens > 0)
        .collect();
    rows.sort_by(|left, right| {
        right
            .requests
            .cmp(&left.requests)
            .then(right.tokens.cmp(&left.tokens))
            .then(left.provider.cmp(&right.provider))
    });
    rows.into_iter()
        .map(|item| {
            json!({
                "id": item.provider,
                "label": if item.provider.is_empty() {
                    UNKNOWN_PROVIDER_LABEL.to_string()
                } else {
                    provider_label(&item.provider)
                },
                "requests": item.requests,
                // failures by not is 1 ： record::ProviderAccum 。
                // max(0) file 
                "success": item.successful,
                "failures": (item.requests - item.successful).max(0),
                "totalTokens": item.tokens,
            })
        })
        .collect()
}

/// unknown （ as empty that 1 ）。
///
/// and provider / that is ** **： 「unknown」 not 
/// （ no down / no / no down ）， will if ，
/// use will 1 etc 。
pub(super) const UNKNOWN_MODEL_LABEL: &str = "未知模型";

/// by total → `/api/stats/summary` `models` 。
///
/// and `build_providers` / `build_accounts` （requests → tokens 
/// → ）。 that in （`model_totals` 1 is 
/// `range_totals` 1 ，`topModel` is from ）， before for 
/// 1 —— this to ， 「usage」 use 。
///
/// ── and providers ─────────────────────────────────
/// 1. **empty return**（label as 「unknown」）：`build_top_model` 
/// emptyexcluding in （empty）， this is need 1 
/// 「usage in 」 ， that 1 usage not this from up ——
/// will and for not up `overview.tokens`。
/// 2. no has `label` and `id` ： is is ， 1 。
///
/// （ and 1 ）： can can ，
/// is 1 as 0 。
pub(super) fn build_models(totals: &[ModelAccum]) -> Vec<Value> {
    let mut rows: Vec<&ModelAccum> = totals
        .iter()
        .filter(|item| item.requests > 0 || item.tokens > 0)
        .collect();
    rows.sort_by(|left, right| {
        right
            .requests
            .cmp(&left.requests)
            .then(right.tokens.cmp(&left.tokens))
            .then(left.model.cmp(&right.model))
    });
    rows.into_iter()
        .map(|item| {
            json!({
                "model": item.model,
                "label": if item.model.is_empty() {
                    UNKNOWN_MODEL_LABEL.to_string()
                } else {
                    item.model.clone()
                },
                "requests": item.requests,
                "totalTokens": item.tokens,
            })
        })
        .collect()
}

/// by total → `/api/stats/summary` `accounts` 。
///
/// and `build_providers` （requests → tokens → ）：
/// and when ， 1 not will 「 1 by request、 1 by 」。
///
/// ── and providers ─────────────────────────────────
/// 1. is `id` + `label` ：provider `label` by ****，
/// is ** when down **（ can can already delete， not to ）。
/// is 「 → id」，empty that 1 use 「unknown」。
/// 2. no has `providerLabel` that 「id → label」 ： not is ，
/// is ， to 。
pub(super) fn build_accounts(totals: &[AccountAccum]) -> Vec<Value> {
    // and build_providers 1 ： not return
    let mut rows: Vec<&AccountAccum> = totals
        .iter()
        .filter(|item| item.requests > 0 || item.tokens > 0)
        .collect();
    rows.sort_by(|left, right| {
        right
            .requests
            .cmp(&left.requests)
            .then(right.tokens.cmp(&left.tokens))
            .then(left.account_id.cmp(&right.account_id))
    });
    rows.into_iter()
        .map(|item| {
            // 「 → id」；empty is unknown（default）
            let label = if !item.account_name.is_empty() {
                item.account_name.clone()
            } else if !item.account_id.is_empty() {
                item.account_id.clone()
            } else {
                UNKNOWN_ACCOUNT_LABEL.to_string()
            };
            json!({
                "id": item.account_id,
                "label": label,
                "requests": item.requests,
                "success": item.successful,
                "failures": (item.requests - item.successful).max(0),
                "totalTokens": item.tokens,
            })
        })
        .collect()
}

/// → response JSON： in up ****
/// `providerLabel` and `phaseElapsedMs`。
///
/// ── providerLabel ───────────────────────────────────────────
/// Why not in `RequestEntry` up 1 ： `providerLabel` is to before 
/// （§6「 provider use label」）， by id 、 will 
/// ， `RequestEntry` is ****—— file will 
/// 「 after label， is old 」。
///
/// ── phaseElapsedMs（add， and up 1 ）────────────────
/// 「 in current already （ms）」， for ** in **（ has `phase` that ）
/// has ； not in to `null`， before not 。
///
/// Why not is when ： when ，（`phase_started_at`）
/// is 、（ in ） in that 1 。 1 from down 
/// that 1 —— is 「 in 」。
/// and server when ： when not will seconds
/// （ and OmniProxy `phase_elapsed_ms` use `julianday('now')` 1 to ）。
///
/// `saturating_sub` + `max(0)`： can can to 1 when ， that 
/// when will 「-5seconds」，。
///
/// failure up not can can （ is i64 / String / Option<String>），
/// not unwrap（panic=abort down will use ）： 1 empty for ，
/// before that 1 empty，。
pub(super) fn entry_json(entry: &RequestEntry) -> Value {
    let Ok(mut value) = serde_json::to_value(entry) else {
        return Value::Object(serde_json::Map::new());
    };
    if let Some(object) = value.as_object_mut() {
        object.insert("clientLabel".into(), json!(crate::server::core::client::display_name(
            &entry.source.client, entry.source.recorded_client_name(),
        )));
        object.insert(
            "providerLabel".to_string(),
            Value::String(provider_label(&entry.provider)),
        );
        object.insert(
            "phaseElapsedMs".to_string(),
            match entry.phase_started_at {
                Some(started) => json!((now_ms() - started).max(0)),
                None => Value::Null,
            },
        );
    }
    value
}

/// topModel： by `tokens` ， and `requests`， and 
/// （ is as ， not ）。
/// none or all no when return `null`。
pub(super) fn build_top_model(totals: &[ModelAccum], range_tokens: i64) -> Value {
    let best = totals
        .iter()
        // empty not and ：otherwise will 「empty」，empty，
        // not to 
        .filter(|item| !item.model.is_empty() && (item.tokens > 0 || item.requests > 0))
        .max_by(|left, right| {
            left.tokens
                .cmp(&right.tokens)
                .then(left.requests.cmp(&right.requests))
                // ： before ， and when 
                .then(right.model.cmp(&left.model))
        });
    let Some(best) = best else {
        return Value::Null;
    };
    // percentage = this tokens / total tokens；total as 0 when to 0.0，
    // not NaN/Infinity（serde_json has will panic or null）
    let percentage = if range_tokens > 0 {
        best.tokens as f64 / range_tokens as f64
    } else {
        0.0
    };
    json!({
        "model": best.model,
        "tokens": best.tokens,
        "requests": best.requests,
        "percentage": percentage,
    })
}

/// （，）。
///
/// before by ， that must（ 0） not is not return ——
/// 1 will in up ， is 。
///
/// `models`（`[{model, label, tokens, requests}]`， by tokens ）：
/// need to 「 this 1 totalusage is 、 in up 」，
/// by in （`model_tokens` ）—— in 
/// 24 when ， this not to not will （ [`model_breakdown_json`]）。
pub(super) fn daily_trend(daily: &BTreeMap<String, DailyEntry>, start: &str, end: &str) -> Vec<Value> {
    let (Some(start_date), Some(end_date)) = (parse_key(start), parse_key(end)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut cursor = start_date;
    // limit：file to not will response
    while cursor <= end_date && out.len() < MAX_TREND_DAYS as usize {
        let key = date_key(cursor);
        let day = daily.get(&key);
        let (tokens, requests) = day.map(|item| (item.tokens, item.requests)).unwrap_or((0, 0));
        let models = day.map(|item| super::source_report::all_model_rows(&item.model_tokens)).unwrap_or_default();
        let (clients, api_keys) = day.map(super::source_report::source_breakdown).unwrap_or_default();
        out.push(json!({
            "date": key,
            "tokens": tokens,
            "requests": requests,
            "models": models,
            "clients": clients,
            "apiKeys": api_keys,
        }));
        cursor += ChronoDuration::days(1);
    }
    out
}

/// ：** 365 **（）， and range ，none 。
/// return， before by 「 1 = if 」。
pub(super) fn heatmap(daily: &BTreeMap<String, DailyEntry>, today: NaiveDate) -> Vec<Value> {
    let start = today - ChronoDuration::days(HEATMAP_DAYS - 1);
    let mut out = Vec::with_capacity(HEATMAP_DAYS as usize);
    let mut cursor = start;
    while cursor <= today {
        let key = date_key(cursor);
        let (requests, tokens) = daily
            .get(&key)
            .map(|day| (day.requests, day.tokens))
            .unwrap_or((0, 0));
        out.push(json!({ "date": key, "requests": requests, "tokens": tokens }));
        cursor += ChronoDuration::days(1);
    }
    out
}

/// has request ： from **** before 。
///
/// 「 no has request not 」—— when is empty ，
/// if ， need 1 ， and use 。
/// no when from 。
pub(super) fn streak(daily: &BTreeMap<String, DailyEntry>, today: NaiveDate) -> i64 {
    let has = |day: NaiveDate| -> bool {
        daily
            .get(&date_key(day))
            .is_some_and(|item| item.requests > 0)
    };
    let mut cursor = today;
    if !has(cursor) {
        cursor -= ChronoDuration::days(1);
    }
    let mut count = 0i64;
    // limit：「」 not will 
    while count < MAX_TREND_DAYS && has(cursor) {
        count += 1;
        cursor -= ChronoDuration::days(1);
    }
    count
}

// ─── in （，must from ）────────────────────

/// in 
pub(super) fn cache_rates(totals: &[(i64, i64)]) -> Value {
    let mut out = serde_json::Map::new();
    for (index, (key, _window)) in CACHE_WINDOWS.iter().enumerate() {
        // by 「 this no has 」（ not this ：SQL has 1 
        // ， `sql::sum_cache_windows`）—— and safe_rate 1 
        let (hit, input) = totals.get(index).copied().unwrap_or((0, 0));
        out.insert(
            key.to_string(),
            json!({ "hitTokens": hit, "inputTokens": input, "rate": safe_rate(hit, input) }),
        );
    }
    Value::Object(out)
}

/// 24 **** in and usage（none ）。
///
/// by when ： when `DateTime<Local>` `%H`，
/// is 「14 」 is use when up 14 ， not is UTC 14 。
///
/// when to `totalTokens`： before that 1 need ****
/// （ in + total Token），must —— request will 
/// in 「 this 1 when not 」 up ， its in 1 1 。
///
/// `models`（`[{model, label, tokens, requests}]`， by tokens ）：
/// need to 「 this 1 when totalusage is 、 in up 」。
/// statistics [`model_stat_key`]（ and 1 ）， is 1 when 
/// 「 and 」 etc this when `totalTokens`（）。
///
/// is ****（[`SummaryRow`]， 6 ） not is ： is after 
/// 1 need ， need need in （
/// `sql::select_summary_rows`）。（`ts` 23 before ） old ——
/// use to can can is `request_stats::SUMMARY_WINDOW_MS` ， in up 
/// not （ when and 「now before N when 」minutes）。
pub(super) fn cache_trend_24h(entries: &[SummaryRow], now: i64) -> Vec<Value> {
    /// ：total + by （ is to ，
    /// can —— and `push_model_accum` 1 ）
    #[derive(Default)]
    struct HourBucket {
        hit: i64,
        input: i64,
        total: i64,
        models: Vec<ModelAccum>,
    }

    let start = hour_floor(ms_to_local(now)) - ChronoDuration::hours(23);
    let start_ms = start.timestamp_millis();

    // by ， by 24 ——
    // 1 is 24×N， this need 1 
    let mut buckets: HashMap<String, HourBucket> = HashMap::new();
    for item in entries {
        if item.ts < start_ms || item.ts > now {
            continue;
        }
        let slot = buckets.entry(hour_key(ms_to_local(item.ts))).or_default();
        slot.hit += item.cache_read_tokens;
        slot.input += item.prompt_tokens;
        slot.total += item.total_tokens;
        push_model_accum(
            &mut slot.models,
            model_stat_key(&item.model, &item.upstream_model),
            item.total_tokens,
            1,
        );
    }

    let mut out = Vec::with_capacity(24);
    for step in 0..24 {
        let key = hour_key(start + ChronoDuration::hours(step));
        let slot = buckets.remove(&key).unwrap_or_default();
        out.push(json!({
            "hour": key,
            "hitTokens": slot.hit,
            "inputTokens": slot.input,
            "totalTokens": slot.total,
            "rate": safe_rate(slot.hit, slot.input),
            "models": model_breakdown_json(&slot.models),
        }));
    }
    out
}

/// in ：`hit / input`， as 0（ or none in ） when return 0.0。
///
/// **must**：0/0 in down is NaN，`serde_json` NaN when 
/// need panic need `null`， before to after empty。
/// this （file can can ）——「 in 」 as no has ，
/// by 0 。
///
/// Note** not to 1.0**： up has cacheRead and promptTokens
/// （promptTokens not ）， when in will 100%。
/// 1 down ， that is —— need ，
/// and to before 。
pub(super) fn safe_rate(hit: i64, input: i64) -> f64 {
    if input <= 0 || hit <= 0 {
        return 0.0;
    }
    let rate = hit as f64 / input as f64;
    if rate.is_finite() {
        rate
    } else {
        0.0
    }
}

/// status 1 ****： for 1 SQL （
/// `sql::FilterPlan::of` ）。
///
/// Why from 「`Option<bool>`」： has ok / error ，bool use ；
/// 「 in 」（running，status=0） after is —— bool need 
/// use bool， need running error 。
/// 「 new 」 in 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StatusFilter {
    /// success（2xx and noneerror need ）
    Ok,
    /// failure（success ；** not ** in ， `sql::FilterPlan::of`）
    Error,
    /// in （status = 0， `request_stats::record_started`）
    Running,
}

/// status 1 ：`"ok"` → success，`"error"` → failure，`"running"` → in ，
/// its （ None and ） not 。 not 、 before after empty。
pub(super) fn normalize_status_filter(value: Option<&str>) -> Option<StatusFilter> {
    match value.map(str::trim).map(str::to_lowercase).as_deref() {
        Some("ok") => Some(StatusFilter::Ok),
        Some("error") => Some(StatusFilter::Error),
        Some("running") => Some(StatusFilter::Running),
        _ => None,
    }
}

/// `YYYY-MM-DD` → NaiveDate（file return None， use ）
fn parse_key(text: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(text.trim(), "%Y-%m-%d").ok()
}
