//! / ：、 and 、， and 
//! 「 to 」 （「 when 」 already ）。
//!
//! ── use 1 ─────────────────────────────
//! - （「」→ `query_all(store, None)`）；
//! - （ 「」 by → `query_all(store, Some(id))`）；
//! - （`spawn_sweeper` ，「config and to 」
//! ）。
//! path****（`core::usage_records`）：
//! this down 1 （ to = up + ， no has ），
//! not / use to is new 。
//!
//! ── to （ no has ）────────────────────────
//! to = `.last_attempt_at + config <= now`， is 「 + config」 
//! ： not （ in ）、（
//! last_attempt）、delete（ ）。failure** not **：
//! failure（ up when ）， down 1 by config —— and 
//! OmniProxy 。
//!
//! ── not （ and OmniProxy ）───────────────────
//! - skip（）： not ， by `usage_records::balance_facts`
//! when （ `upstream::rotate`）， and ；
//! - disable（ use ）：success after by `enforce_low_balance_disable`
//! ， `enabled` false —— not ， need enabled；
//! ****（ use not and ， new use ）。
//! and 1 （`usage_records::extract_remaining`）。
//!
//! ── limit（ / Token then ） and Token ─────────
//! 「 not 」 already as **limit then **（`core::limiter`，
//! `limiters` ）： then use up and （
//! already valid then ），Token then 。Token in ：
//! ** new **（`limiter::refresh_token_facts`， for 
//! when ，）， **Token use **（`enforce_token_disable`，
//! disable in use 、 not ）；skip by 1 。

use std::time::Duration;

use serde_json::{json, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::core::limiter;
use crate::server::core::usage_records;

use crate::server::logging;

/// failure Reason：「 no has 」（Node ， name ） and 
/// 「requestfailure」（ name ）。 is to before ** can **（current
/// has 「not configured」 use ， `adapter::USAGE_NOT_CONFIGURED_CODE`）： before 
/// this 1 in not is failure， by can 。
enum UsageFailure {
    /// no has available —— Node that `return { id, usage:null, error }`
    NoCredentials,
    Request(String, Option<String>),
}

/// failure。
///
/// core not axum， this `message` / `status_code` ， by api 
/// response —— not is in core 1 `Response`。
#[derive(Clone, Debug)]
pub struct TargetError {
    pub message: String,
    pub status_code: u16,
}

/// ：**available**（`available: false` ），
/// 。
///
/// `provider`：`Some(id)` this ；`None` （**** use ——
/// not 、 by ）。 id when ** not **
/// （ and 「」 has 1 ）。
///
/// ── Why not by `enabled` ─────────────────────────────
/// use 「 not and 」， and 「 this 」none —— and path
/// （`?id=`） has 1 。 by enabled / when this 1 ， up that 
/// is 「」， use can 「」 by to ： when 
/// etc （： use when ， from no ）。
///
/// from `api::accounts` down （ `resolve_batch_targets`）： is ，
/// not HTTP ，「 when 」must and use ** 1 **
/// —— path 1 ，「 」 this will 。
///
/// return `(targets, skipped)`：`skipped` is 「**unavailable**
/// （`available: false`） 」—— that not ， will 
/// failure， not （ and 「 already failurerequest」 1 ）。
pub fn resolve_batch_targets(
    store: &AccountStore,
    provider: Option<&str>,
    id: Option<&str>,
) -> Result<(Vec<Value>, usize), TargetError> {
    let snapshot = store.list_accounts();
    let accounts: Vec<Value> = snapshot
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    // provider by default provider （ and store 1 ：
    // old no has this ），otherwise will in down 
    let in_scope = |account: &Value| match provider {
        None => true,
        Some(provider) => {
            account
                .get("provider")
                .and_then(Value::as_str)
                .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID)
                == provider
        }
    };
    let is_available = |account: &Value| {
        account.get("available").and_then(Value::as_bool).unwrap_or(true)
    };
    if let Some(id) = id.filter(|value| !value.is_empty()) {
        let found: Vec<Value> = accounts
            .iter()
            .filter(|account| account.get("id").and_then(Value::as_str) == Some(id))
            .cloned()
            .collect();
        if found.is_empty() {
            return Err(TargetError { message: "账号不存在".to_string(), status_code: 404 });
        }
        return Ok((found, 0));
    }
    // = **available**（ not enabled，）；
    // skipped is unavailable ，「Why」
    let (mut targets, mut skipped) = (Vec::new(), 0usize);
    for account in accounts.into_iter().filter(in_scope) {
        if is_available(&account) {
            targets.push(account);
        } else {
            skipped += 1;
        }
    }
    Ok((targets, skipped))
}

/// ：failure as `{error}` not is （ not 
/// ）， new in `query_usage_inner` 。
async fn query_usage_for(store: &AccountStore, account: &Value) -> Value {
    let id = account.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    let name = account.get("name").cloned().unwrap_or(Value::Null);
    let started = std::time::Instant::now();
    // and provider from for ，** not file**：
    // 20 and when that will 20 ， already in up 。
    // `hasCredentials` is store to 1 （ as has ）。
    //
    // （「 API」 that ） has ** 1 **（NewAPI 
    // / / ，
    // `providers::custom::balance`）： can ，「 has 」
    // —— not 1 no Key will to 「 no has available」，
    // this is 「 1 」（ to 400「not configured」）。
    let has_credentials = account.get("hasCredentials").and_then(Value::as_bool) != Some(false)
        || account
            .get("balanceQuery")
            .and_then(|query| query.get("scheme"))
            .is_some();
    let outcome = if has_credentials {
        let provider_id = account
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID);
        query_usage_inner(store, provider_id, &id).await
    } else {
        Err(UsageFailure::NoCredentials)
    };
    let row = match outcome {
        Ok(usage) => json!({ "id": id, "name": name, "usage": usage, "error": Value::Null }),
        // Node this `return` ** not name **（ has catch ）
        Err(UsageFailure::NoCredentials) => {
            json!({ "id": id, "usage": Value::Null, "error": "没有可用凭证" })
        }
        Err(UsageFailure::Request(message, code)) => {
            json!({ "id": id, "name": name, "usage": Value::Null, "error": message, "code": code })
        }
    };
    let provider = account.get("provider").and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID);
    let error = row.get("error").and_then(Value::as_str);
    let not_configured = row.get("code").and_then(Value::as_str)
        == Some(crate::server::core::providers::adapter::USAGE_NOT_CONFIGURED_CODE)
        || matches!(error, Some("没有可用凭证"));
    let level = if error.is_none() || not_configured { "debug" } else { "warn" };
    let summary = if error.is_none() { "余额查询完成" } else if not_configured { "余额查询未配置" } else { "余额查询失败" };
    logging::log_event(level, "account", summary, Some(json!({
        "provider": provider,
        "providerLabel": crate::server::core::providers::label_of(provider),
        "account": id,
        "accountName": name,
        "action": "usage.query",
        "summary": summary,
        "outcome": if error.is_none() { "success" } else { "failed" },
        "detail": error,
        "code": row.get("code"),
        "durationMs": started.elapsed().as_millis() as u64,
    })));
    row
}

/// 1 / （** by provider to **）。
///
/// ── 401： new after 1 ，** has can new has this 1 ** ────────
/// 「401 token server」——Node new ， its error
/// return； new not ， 1 `refresh_access_token`（force ）。
///
/// `supports_refresh()` is **CatPaw **： up no has new 
/// （`X-Passport-Token` can in new ， `catpaw/adapter.rs`）， for 
/// 401 new will failure， 1 「」error（ new failure 
/// Reason， use not this ）。
///
/// has new can if new failure，****（ 401 Reason + new failure 
/// up + 「 new 」 this ）， not new that Reason —— to 
/// `authorization_verify_error` this up ， use not this 。
///
/// workbuddy ： `query_usage` has （ not ）。
async fn query_usage_inner(
    store: &AccountStore,
    provider_id: &str,
    id: &str,
) -> Result<Value, UsageFailure> {
    // （「 API」 that ）**** that ：
    // no has `ProviderKind`（ not in ）， has up `balanceScheme`
    // （ `providers::custom::balance`）。 this must in down that 
    // 「unknown 」 before —— otherwise will up 。
    if crate::server::core::custom_providers::is_custom_provider_id(provider_id) {
        // this no has （ is or ）， no has 
        // 401 new that 1 —— has will 「 not for 」error
        return crate::server::core::providers::custom::balance::query_usage(store, id)
            .await
            .map_err(|error| UsageFailure::Request(error.message, error.code));
    }
    // no has id（ before after new 、 or file）：「unknown 
    // 」， not 1 （ and provider 1 ）。
    //
    // ── in this ────────────────────────────────────────
    // id not in ，** in when ** —— use `adapter_by_id`
    // when ， not is `kind_from_id`（ that will 
    // 「unknown 」， up has that ）。
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider_id) else {
        return Err(UsageFailure::Request(
            format!("未知的提供商 {provider_id}，无法查询余额"),
            None,
        ));
    };
    match adapter.query_usage(store, id).await {
        Ok(usage) => Ok(usage),
        Err(error) => {
            // unsupported new returnerror（CatPaw：failure， up ）
            if error.status_code != 401 || !adapter.supports_refresh() || store.needs_relogin(id) {
                return Err(UsageFailure::Request(error.message, error.code));
            }
            if let Err(refresh_error) = adapter.refresh_access_token(store, id).await {
                // new failure when **error not can **：401 （token /
                // already up ） in up is not ， new that 
                // up （ is `authorization_verify_error`） use not 、
                // not this 。 to ， and 「 new 」 this 
                // —— this can to new ， this has can ， is 1 。
                return Err(UsageFailure::Request(
                    format!(
                        "{}；自动续期也失败：{}（登录态可能已被上游作废，请重新登录或重新导入该账号的凭证）",
                        error.message, refresh_error.message
                    ),
                    error.code,
                ));
            }
            adapter
                .query_usage(store, id)
                .await
                .map_err(|retry| UsageFailure::Request(retry.message, retry.code))
        }
    }
}

/// this is not is ****（ up `isGuest`）。
///
/// ： by `account_view` to ， by 
/// `store_view::public_account` （ `chatglm_accounts::guest_of_record`）。
/// not ， this not JWT。
fn is_guest(account: &Value) -> bool {
    account.get("isGuest").and_then(Value::as_bool) == Some(true)
}

/// this provider is can （`ProviderAdapter::supports_usage`），
/// **and** this has can 。
///
/// Why need 「unsupported」 ：unsupported will to ** ** 1 
/// 501 —— use to 1 ， that not is 、 is can （currentsupports，
/// is can ）。 is can ， not is this request not success。
/// unknown provider id 1 and ； id use path will to that 。
///
/// （「 API」 that ）** not in **， can 
/// up （`balanceScheme`， `providers::custom::balance`）：
/// has = this 1 has （ no up when that to 「not configured」， not is unsupported）。
///
/// ── excluding in this （2026-10）─────────────────────
/// no has （ up that for 1 401）， 
/// ** as failure**， 1 is up 1 。 not is 「 this no to 」， is 
/// 「 this not in can 」—— and 「 this no has 」 is 1 ，
/// by 1 and excluding： not to 、 not 。
/// path（`?id=`）** not ** this （ `query_all`）， use that 1 when 
/// will to to in ， not is 1 not by 。
fn supports_usage(account: &Value) -> bool {
    if is_guest(account) {
        return false;
    }
    let provider_id = account
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID);
    supports_usage_provider(provider_id)
}

pub(crate) fn supports_usage_provider(provider_id: &str) -> bool {
    if crate::server::core::custom_providers::is_custom_provider_id(provider_id) {
        return crate::server::core::providers::custom::balance::supports_usage(provider_id);
    }
    // use `adapter_by_id` not is `kind_from_id`： id not in ，
    // `supports_usage`（ up that ）。
    crate::server::core::providers::adapter::adapter_by_id(provider_id)
        .map(|adapter| adapter.supports_usage())
        .unwrap_or(false)
}

/// and / total
/// （`{ results: [{id,name,usage,error,code?}], skipped }`）。
///
/// is **available**（`resolve_batch_targets(provider = None)`，
/// not enabled， that ）， by `provider` to 
/// `ProviderAdapter::query_usage`。 and this ， 
/// use 1 （`query_usage_for`）—— 
/// ****（module header）， not / use to 1 is 
/// new ， this down 1 。
///
/// ** and ** is ：Node use `Promise.all`，20 will before 20 
/// 。 this use `join_all` in 1 and （ future is etc ，
/// ）， and ** and targets 1 ** —— and `Promise.all` 。
/// ** when by **（15~20 seconds）； this not when —— that will 
/// 「 up 」 and 「」 in none。
///
/// ── `id`： when this 1 ──────────────────────────────
/// ** use 1 「」 by ** is 「 this in 」。
/// use not and ， and its can not can no has —— by enabled 
/// this ， will to 1 「return」， use not is 「 use 」
/// is 「 up 」。 id when by `resolve_batch_targets` 
/// （ that id， not and available）， not `supports_usage`：
/// can is to path 1 501 ，「 this unsupported」。
/// path in not enabled（ `resolve_batch_targets`），path 
/// 1 is 「 1 」 is 「available」。
///
/// unknown id by `resolve_batch_targets` 404「 not in 」。
pub async fn query_all(store: &AccountStore, id: Option<&str>) -> Result<Value, TargetError> {
    let single = id.filter(|value| !value.is_empty());
    // `provider = None`：（ to that by provider ）
    let (targets, skipped) = resolve_batch_targets(store, None, single)?;
    let queried: Vec<&Value> = targets
        .iter()
        .filter(|account| single.is_some() || supports_usage(account))
        .collect();
    let futures: Vec<_> = queried
        .iter()
        .map(|account| query_usage_for(store, account))
        .collect();
    let mut results = futures::future::join_all(futures).await;
    // + 「 not use 」： and path in 
    // this （module header）。 for and by 1 1 for 
    // （`queried` and `results` ），config in up ， not 。
    for (account, row) in queried.iter().zip(results.iter_mut()) {
        write_row_record(row);
        enforce_low_balance_disable(store, account, row);
        if let Some(id) = account.get("id").and_then(Value::as_str) {
            let record = usage_records::load(id);
            row["at"] = json!(record.last_attempt_at);
            row["lastSuccessAt"] = json!(record.last_success_at);
            if row.get("error").and_then(Value::as_str).is_some() {
                row["lastUsage"] = json!(record.usage);
            }
        }
    }
    Ok(json!({ "results": results, "skipped": skipped }))
}

// ─── and 「 not use 」 ────────────────────

/// 1 → （`core::usage_records`）。 not id not ，。
fn write_row_record(row: &Value) {
    let Some(id) = row
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
    else {
        return;
    };
    let outcome = usage_records::UsageOutcome {
        usage: row.get("usage").filter(|usage| !usage.is_null()).cloned(),
        error: row.get("error").and_then(Value::as_str).map(str::to_string),
        code: row.get("code").and_then(Value::as_str).map(str::to_string),
    };
    usage_records::write_result(id, &outcome);
}

/// 「 not use 」：**success** after （failure not —— 
/// is up success old ， use is old 1 ， etc down 1 success）。
///
/// in （ need ）：
/// 1. config `lowBalance.mode == "disable"`（`off` / `skip` not ）；
/// 2. and ****（ etc available， and ）；
/// 3. current is enabled（ etc ： already use not 、 not ）。
///
/// use `update_account` has path（ and use 1 ，「 already use 」 
/// ）， 1 Reason —— 「Why 」must can 。
/// ** not **： after need enabled；（ use not and 
/// ）， new ， use when 。
fn enforce_low_balance_disable(store: &AccountStore, account: &Value, row: &Value) {
    if account.get("limiterGroups").is_some() {
        limiter::enforce_disable(store, std::slice::from_ref(account));
        return;
    }
    if matches!(account.get("enabled"), Some(Value::Bool(false))) {
        return;
    }
    let Some(id) = account.get("id").and_then(Value::as_str) else {
        return;
    };
    let Some(threshold) = usage_records::low_balance_disable_threshold(account) else {
        return;
    };
    let Some(usage) = row.get("usage").filter(|usage| !usage.is_null()) else {
        return;
    };
    let (remaining, _unlimited) = usage_records::extract_remaining(usage);
    let Some(remaining) = remaining else { return };
    if remaining >= threshold {
        return;
    }
    let display = account
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .unwrap_or(id);
    match store.update_account(id, &json!({ "enabled": false })) {
        Ok(_) => logging::log_with_level(
            "[Usage]",
            &format!(
                "账号「{display}」余额 {remaining} 低于阈值 {threshold}，已自动禁用（余额回升后请手动启用，自动查询继续）"
            ),
            "warn",
        ),
        Err(error) => logging::verbose(
            "[Usage]",
            &format!("账号 {id} 余额不足自动禁用失败：{}", error.message),
        ),
    }
}

// ─── Token limit use ──────────────────────────────

/// 「Token use 」：sweeper （10 seconds） in new after 。
///
/// in （ need ）：
/// 1. enabled in （ etc ： already use not 、 not ）；
/// 2. enabled Token **disable** then （skip not —— in ）；
/// 3. current ≥ （ by up 1 `refresh_token_facts` new ，
/// request has ， old 10 seconds —— for ）。
///
/// use `update_account` has path（ and use 1 ，「 already use 」 
/// ）， 1 Reason 。** not **：
/// use ， need enabled —— and use 1 （ ）。
fn enforce_token_disable(store: &AccountStore, accounts: &[Value], now: i64) {
    let facts = limiter::token_facts();
    for account in accounts {
        if account.get("limiterGroups").is_some() { continue; }
        if matches!(account.get("enabled"), Some(Value::Bool(false))) {
            continue;
        }
        let Some(id) = account.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some((rule, used)) = limiter::token_disable_hit(account, &facts, now) else {
            continue;
        };
        let display = account
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .unwrap_or(id);
        match store.update_account(id, &json!({ "enabled": false })) {
            Ok(_) => logging::log_with_level(
                "[Usage]",
                &format!(
                    "账号「{display}」{}消耗 {} Token 达到上限 {}，已自动禁用（重置也不恢复，请手动启用）",
                    if rule.reset == limiter::LimiterReset::Daily {
                        "今日".to_string()
                    } else {
                        format!("{}内", limiter::describe_period(rule.period))
                    },
                    used,
                    rule.threshold,
                ),
                "warn",
            ),
            Err(error) => logging::verbose(
                "[Usage]",
                &format!("账号 {id} Token 限额自动禁用失败：{}", error.message),
            ),
        }
    }
}

// ─── ────────────────────────────────

/// ：10 seconds（ and `scheduled_tasks` tick 1 ）。
/// down is 30 seconds， this 「 to after 10 seconds」——
/// use not to ，empty is 10 seconds 1 （ to in and 
/// ）。
pub const SWEEP_TICK_MS: u64 = 10_000;

/// （`ServerState::bootstrap` 1 ，
/// has this 1 ）。： to → → and → → 。
pub fn spawn_sweeper(store: AccountStore) {
    crate::spawn_task(async move {
        loop {
            sweep_due(&store).await;
            tokio::time::sleep(Duration::from_millis(SWEEP_TICK_MS)).await;
        }
    });
}

/// 1 ： 1 「config and to 」 。
///
/// to （`usage_records::query_interval_of` + `last_attempt_at`）
/// is 「 + config」 ，module header —— this no has can 。
/// excluding `supports_usage == false` （ 1 501， and 
/// ）； no ** not excluding**： that is can to failure，
/// 「 no has available」， and 1 as 。
///
/// ── Token limit in this 1 （「 no has to 」 ）──
/// ① ** new **（`limiter::refresh_token_facts`）： by enabled in 
/// Token current 1 —— after old by 
/// ， new then by ；10 seconds 1 ， schema v9 。
/// ② **Token use **（`enforce_token_disable`）：disable in 
/// （current ≥ ） use —— and use 1 。
async fn sweep_due(store: &AccountStore) {
    usage_records::prune_orphans();
    let now = logging::now_ms();
    // （ not enabled / available）：Token need all then ，
    // including use —— use new enabled when already in ， not use etc down 1 new 
    let all_accounts: Vec<Value> = store
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    limiter::refresh(&all_accounts);
    limiter::enforce_disable(store, &all_accounts);
    let kinds = limiter::enabled_token_kinds(&all_accounts);
    if !kinds.is_empty() {
        limiter::refresh_token_facts(&kinds);
        enforce_token_disable(store, &all_accounts, now);
    }
    let due: Vec<(Value, i64)> = match resolve_batch_targets(store, None, None) {
        Ok((targets, _skipped)) => targets,
        Err(_) => return, // 目标集合解析失败（库不可用等）：这一轮什么都不做
    }
    .into_iter()
    .filter(supports_usage)
    .filter(|account| account.get("needsRelogin").and_then(Value::as_bool) != Some(true))
    .filter_map(|account| {
        let interval = usage_records::query_interval_of(&account)?;
        let id = account.get("id").and_then(Value::as_str)?;
        let due_at = usage_records::load(id).last_attempt_at + interval * 1000;
        (due_at <= now).then_some((account, interval))
    })
    .collect();
    if due.is_empty() {
        return;
    }
    // request（`usage_records` module header ： not 「 no 」）
    for (account, _) in &due {
        if let Some(id) = account.get("id").and_then(Value::as_str) {
            usage_records::mark_attempt(id);
        }
    }
    let futures: Vec<_> = due
        .iter()
        .map(|(account, _)| query_usage_for(store, account))
        .collect();
    let results = futures::future::join_all(futures).await;
    for ((account, _), row) in due.iter().zip(results.into_iter()) {
        write_row_record(&row);
        enforce_low_balance_disable(store, account, &row);
    }
}

// ─── （ before 20 seconds ）─────────────────────────────

/// 1 （** already **）。 from when to 
/// `{at: 0, results: [], skipped: 0}` —— 「 no has 」，
/// not is empty「 1 no has 」。
///
/// and response（ before `applyBalances` 1 ）， `at`
/// when ，`lastSuccessAt` success when 。failure `usage: null`，
/// success in `lastUsage`， not can in error。
///
/// ── Token limit（`tokenAt` / `tokenUsage`）──────────────
/// in to when ，`at` not no new ；Token 
/// request， when `tokenAt`（ as when ， before 
/// by need not need use ）， and `at` 。`tokenUsage` by to enabled in 
/// Token 1 `{period, windowStart, used}`（，
/// SQL； by 10 seconds new + request）。
///
/// （ and old kv `prune_stale_failures` ）：
/// - already delete not （ up no has that 1 ， will for not up ）；
/// - failure in 「 this not after 」 when —— new /
/// / token after ，「 when not to 」 that not 。success
/// not ：「available 5147 」 is 1 ， not will 。
///
/// ── ： （2026-10）──────────────────
/// no has （ `supports_usage` ）， down ** can can ** is 
/// failure。 will in up 1 ， that is 「 this not this has 」，
/// not is 「 this no to 」。 before that in this 1 and ， not etc 
/// 1 。
pub fn snapshot(store: &AccountStore) -> Value {
    let records = usage_records::load_all();
    let accounts = store.list_accounts();
    let list = accounts.get("accounts").and_then(Value::as_array);
    let mut rows = Vec::new();
    let mut at = 0i64;
    for (id, record) in &records {
        let account = list.and_then(|list| {
            list.iter().find(|account| {
                account.get("id").and_then(Value::as_str) == Some(id.as_str())
            })
        });
        let Some(account) = account else { continue };
        if is_guest(account) {
            continue;
        }
        at = at.max(record.last_attempt_at);
        let name = account.get("name").cloned().unwrap_or(Value::Null);
        let changed = account
            .get("updatedAt")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .max(account.get("addedAt").and_then(Value::as_i64).unwrap_or(0));
        let error = record.error.as_deref().filter(|_| record.last_attempt_at >= changed);
        if let Some(error) = error {
            rows.push(json!({
                "id": id,
                "name": name,
                "usage": Value::Null,
                "error": error,
                "code": record.code,
                "at": record.last_attempt_at,
                "lastUsage": record.usage,
                "lastSuccessAt": record.last_success_at,
            }));
        } else if let Some(usage) = record.usage.as_ref() {
            rows.push(json!({
                "id": id,
                "name": name,
                "usage": usage,
                "error": Value::Null,
                "at": record.last_attempt_at,
                "lastSuccessAt": record.last_success_at,
            }));
        }
    }
    // Token limit： by （ no has Token then not ，
    // before by 「 no then 」）。 before 20 seconds this ， when limit
    let now = logging::now_ms();
    let facts = limiter::token_facts();
    let mut token_usage = serde_json::Map::new();
    for account in list.into_iter().flatten() {
        let entries = limiter::token_usage_rows(account, &facts, now);
        if entries.is_empty() {
            continue;
        }
        if let Some(id) = account.get("id").and_then(Value::as_str) {
            token_usage.insert(id.to_string(), Value::Array(entries));
        }
    }
    if let Some(accounts) = list { limiter::counter::refresh(accounts); }
    let source_facts = limiter::sources::facts();
    let counts = limiter::counter::facts();
    let mut limiter_states = serde_json::Map::new();
    let mut quota_sources = serde_json::Map::new();
    let mut limiter_models = serde_json::Map::new();
    for account in list.into_iter().flatten() {
        let Some(id) = account.get("id").and_then(Value::as_str) else { continue };
        let provider = crate::server::core::routing::provider_of(account);
        quota_sources.insert(id.to_string(), limiter::sources::rows_for(account, &source_facts));
        limiter_states.insert(id.to_string(), limiter::types::json_of(&limiter::evaluate::states(account, &source_facts, &counts, now)));
        limiter_models.entry(provider.to_string()).or_insert_with(|| Value::Array(limiter::sources::account_models(provider)));
    }
    let visible_accounts: Vec<Value> = list.into_iter().flatten()
        .filter(|account| crate::server::core::account_store::store_view::account_visible(
            crate::server::core::routing::provider_of(account),
        )).cloned().collect();
    json!({
        "at": at,
        "results": rows,
        "skipped": 0,
        "tokenAt": now,
        "tokenUsage": token_usage,
        "limiterStates": limiter_states,
        "quotaSources": quota_sources,
        "limiterModels": limiter_models,
        "sorting": crate::server::core::account_order::sorting(
            &visible_accounts, now,
        ),
    })
}

