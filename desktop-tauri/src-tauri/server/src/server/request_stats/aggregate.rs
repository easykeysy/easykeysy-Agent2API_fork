//! requeststatistics ****： 、 and 。
//!
//! ── Why 1 ──────────────────────────────────────────
//! `request_daily` that 1 1 not 「 1 」：
//! （`record`）、（`rebuild_day`） and （`summarize`）
//! must 1 ， not will as path 1 for not up 
//! （「 and = total」）。 this by and `request_stats.rs` 
//! （ / / ） not ， after this not in 
//! 1 file， 1 can to 。 is `request_stats`
//! 1 （`pub(super)`）， for API and before 。
//!
//! ── 「 in not 」 ────────────────────────────────
//! status=0 （ when ， `RequestStats::record_started`） no has 
//! ， will empty 1 request token is 0。 this in 
//! [`fold_into_daily`] —— 1 ， / path 。

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Duration as ChronoDuration, NaiveDate};
use serde_json::{json, Value};

use super::clock::{date_key, day_of, local_midnight_ms};
use super::record::{DailyEntry, RequestEntry};
use super::report::{
    build_accounts, build_models, build_providers, build_top_model, cache_rates, cache_trend_24h,
    daily_trend, heatmap, model_stat_key, normalize_range, push_account_accum, push_model_accum,
    push_provider_accum, range_bounds, range_totals, streak, SummaryRow,
};
use super::{daily, sql};
use super::legacy;

/// usage （`kv` ）。
///
/// and `legacy::MARKER_*` 1 and ： in as already ， as `'true'`。
/// Why need need （ not is that from ），
/// `RequestStats::remap_model_dimension_once` 。
const MODEL_UPSTREAM_MARKER: &str = "modelUsageUpstreamRemapped";

/// [`RequestStats::remap_model_dimension_once`] ：
/// 「 → → → 」。
///
/// [`rebuild_day`]（ and / 1 ，
/// 1 ）；return （，）。
pub(super) fn remap_model_days(conn: &mut rusqlite::Connection) -> rusqlite::Result<Vec<String>> {
    if legacy::marker_present(conn, MODEL_UPSTREAM_MARKER)? {
        return Ok(Vec::new());
    }
    let tx = conn.unchecked_transaction()?;
    // to （）。DISTINCT ts ts ，
    // has 2 ， can ignore。
    let mut days: BTreeSet<NaiveDate> = BTreeSet::new();
    {
        let mut stmt = tx.prepare("SELECT DISTINCT ts FROM requests")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let ts: i64 = row.get(0)?;
            days.insert(day_of(ts));
        }
    }
    let mut changed = Vec::new();
    for day in days {
        let key = date_key(day);
        let start = local_midnight_ms(day);
        // up 1 ms： and rebuild_day 1 （ that ）
        let end = local_midnight_ms(day + ChronoDuration::days(1)).saturating_sub(1);
        let detail_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM requests WHERE ts >= ?1 AND ts <= ?2",
            rusqlite::params![start, end],
            |row| row.get(0),
        )?;
        // request。 not （ not in or failure） 0 ：
        // is by ， when 。
        let aggregate_requests: i64 = tx
            .query_row(
                "SELECT requests FROM request_daily WHERE date = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .unwrap_or(0);
        // backfill 1 ： ≥ request
        // （ no ）， —— will 
        // empty（ `backfill` module header 「 not not 」）
        if detail_count >= aggregate_requests {
            rebuild_day(&tx, day)?;
            changed.push(key);
        }
    }
    // and ： in （ / ） after no ， down 
    // —— etc ， is 1 
    tx.execute(
        "INSERT INTO kv (key, value) VALUES (?1, 'true')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![MODEL_UPSTREAM_MARKER],
    )?;
    tx.commit()?;
    Ok(changed)
}

/// 1 ： from **remaining**； 1 not that 1 。
///
/// `day` is ， by `[, )` —— and `date_key(day_of(ts))`
/// （ use `clock` when ），「 1 」
/// and 「 when by 」 not can can for not up 。
fn rebuild_day(conn: &rusqlite::Connection, day: NaiveDate) -> rusqlite::Result<()> {
    let key = date_key(day);
    let start = local_midnight_ms(day);
    // up 1 ms， etc `[start, next_start)`； use 1ms not is 
    // `next_start - 1` is as 「 when not in 」 this 
    // （`local_midnight_ms` has ， 1 to ）
    let end = local_midnight_ms(day + ChronoDuration::days(1)).saturating_sub(1);
    // and when （`fold_into_daily` use path）must：
    // not （ `RequestEntry::is_test`），otherwise「 1 」
    // will that and before for not up 
    let entries = sql::select_between(conn, start, end, true)?;
    if entries.is_empty() {
        daily::delete_daily(conn, &key)?;
        return Ok(());
    }
    let mut rebuilt = DailyEntry::new(key);
    for item in &entries {
        fold_into_daily(&mut rebuilt, item);
    }
    daily::upsert_daily(conn, &rebuilt)
}

/// ： to `report` ， `/api/stats/summary` response。
///
/// Why this from `usage_summary` ：unavailable when is 「 by empty 1 
/// 」（ this ）， after path**** ——
/// 1 JSON will and path ， is 
/// （ before will empty or 0， not will ）。
///
/// （ after [SummaryRow] this 1 ）：
/// - `daily`：`request_daily` （overview / / / from ）；
/// - `cache_totals`： in `(hit, input)` ， by SQL by 
/// （`sql::sum_cache_windows`）， not ；
/// - `entries`： 24 when ****（`sql::select_summary_rows`），
/// when 。
///
/// `custom` is `range == "custom"` when （， by by ）：
/// custom None（ not use —— has custom will ）。
pub(super) fn summarize(
    daily: &BTreeMap<String, DailyEntry>,
    cache_totals: &[(i64, i64)],
    entries: &[SummaryRow],
    range: &str,
    now: i64,
    custom: Option<(NaiveDate, NaiveDate)>,
) -> Value {
    let now_day = super::clock::today();
    let range = normalize_range(range);
    let (start_date, end_date) = range_bounds(range, daily, now_day, custom);

    // overview / dailyTrend / heatmap （； has request）
    let totals = range_totals(daily, &start_date, &end_date);
    let top_model = build_top_model(&totals.model_totals, totals.tokens);
    // providers and topModel ****： from this `range_totals` ，
    // is 「 by provider request and 」 etc overview.requests，
    // before and when not will for not up 
    let providers = build_providers(&totals.provider_totals);
    // accounts and providers / topModel ****（ up ）： 
    // request and etc overview.requests
    let accounts = build_accounts(&totals.account_totals);
    // models and topModel （`topModel` is this ）：
    // 「usage」， and etc overview.tokens
    let models = build_models(&totals.model_totals);
    let trend = daily_trend(daily, &start_date, &end_date);
    let map = heatmap(daily, now_day);
    let consecutive = streak(daily, now_day);

    // cacheRates SQL （ (hit, input)， `sql::sum_cache_windows`）；
    // cacheTrend24h （ has need need ）
    let rates = cache_rates(cache_totals);
    let cache_trend = cache_trend_24h(entries, now);

    json!({
        "range": range,
        "startDate": start_date,
        "endDate": end_date,
        "overview": {
            "requests": totals.requests,
            "successful": totals.successful,
            "tokens": totals.tokens,
            "activeDays": totals.active_days,
            "streak": consecutive,
            "topModel": top_model,
        },
        // by provider total（**add， not has **）。
        // before by 「 in then 、 then 」， old before to will ignore。
        // as （none when is empty not is null）： before not empty
        "providers": providers,
        // by total（**add， not has **， and providers ）。
        // is provider 1 （ 1 can ）， this is 
        // 「 in 」—— 1 will 1 。
        "accounts": accounts,
        // by total（**add， not has **， and up ）。
        // provider ： 1 can ， this is 
        // 「usage in up 」—— 「usage」
        "models": models,
        "heatmap": map,
        "cacheRates": rates,
        "cacheTrend24h": cache_trend,
        "dailyTrend": trend,
    })
}

/// 1 。
///
/// `record` and `rebuild_day` （ has ） use this 1 ：
/// **** by ，path
/// （ for not up ）。 （ / provider / ） and total and ，
/// and etc total， this is can for before 。
///
/// ** in （status=0） in this down **：。path 
/// not will has status=0（error is 4xx/5xx， `GatewayError` ），
/// this is `rebuild_day` / ** from ** when 
/// in —— no has ， will 1 「 no 」 requestemptyrequest
/// （token is 0）， and 。
pub(super) fn fold_into_daily(day: &mut DailyEntry, item: &RequestEntry) {
    if item.status == 0 || item.is_test {
        return;
    }
    super::source_report::fold_sources(day, item);
    let success = item.is_success();
    day.requests += 1;
    if success {
        day.successful += 1;
    }
    day.tokens += item.total_tokens;
    day.cache_hit_tokens += item.cache_read_tokens;
    day.cache_input_tokens += item.prompt_tokens;
    // use 「 up 」statistics（ `model_stat_key`）： is ，
    // request is up that 1 ， need by 
    push_model_accum(
        &mut day.model_tokens,
        model_stat_key(&item.model, &item.upstream_model),
        item.total_tokens,
        1,
    );
    // empty provider （ push_provider_accum ）
    push_provider_accum(
        &mut day.provider_stats,
        &item.provider,
        1,
        i64::from(success),
        item.total_tokens,
    );
    push_account_accum(
        &mut day.account_stats,
        &item.account_id,
        &item.account_name,
        1,
        i64::from(success),
        item.total_tokens,
    );
}
