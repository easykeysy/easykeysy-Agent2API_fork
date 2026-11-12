//! —— （ for Node src/workbuddy-routing.mjs ）。
//!
//! is 「」：must 1 （Agent2API after use as 
//! ** provider **， by account-store in ， its module header），
//! not and —— and will 「」。****，
//! is （`store.list_accounts()` `accounts` ），
//! is UI up to that ：`id` / `name` / `priority` / `addedAt` /
//! `enabled` / `rateLimits`。
//!
//! ── Why use `serde_json::Value` not is ──────────────
//! Node use in 「 for 」 up ， is 
//! （/ not for not ， by JS ）。 this 1 and 1 
//! ， as 「」 in ——
//! that will and Node （e.g. as `priority: "abc"` ）。
//!
//! ── then （ Node ）──────────────────────────────
//! 1. = enabled in （`enabled !== false`） and for 「 this 」 ；
//! 2. that （ use ）；
//! 3. already （429 ） from in excluding，；
//! 4. unavailable when ， by use is is error。
//!
//! ── then 1 「 this 」** up **， not is request ─────────────
//! `rateLimits` is up to and that 。
//! （`modelRules.mappings`） will in before request up ， and 
//! must use —— [`cooldown_key`]， use request will has after that 。
//! `model` parameter 1 is **request**， by by 
//! （[`CooldownKeys`] this 1 ）。
//!
//! 「current」 not is ， is in 「 not 」 down that 
//! （account-store `get_current_entry` by ）。 up 「current」
//! and default use 1 ；only for when ， this request will when 
//! to down 1 —— that is by 1 ， not 「current」。

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::{json, Value};

use crate::server::core::account_store::priority::{by_priority_order, normalize_priority};
use crate::server::core::account_store::store_util::js_truthy;

/// 「request → up 」 ： 。
///
/// ── Whymust is not is request（ need 1 ）─────
/// `rateLimits` is ** up by that **。（`modelRules.mappings`）
/// will in before request up （`providers::catalog::wire_target_for_provider`），
/// up ： for `deepseek-v4.1-flash` ， `gpt-5.6-luna`
/// this for up no 。request will when ：
///
/// 1. ****： 1 `deepseek-v4.1-flash` and `gpt-5.6-luna`
/// not 。2026-09 ： 1 up `deepseek-v4.1-flash` /
/// `gpt-5.6-luna` / `gpt-6-astra` `resetAt` （
/// is 1 ， as after is to 1 for ），
/// 「3 in 」。
/// 2. ** in **（ ）： use request after ， in up ；
/// use request when is —— not to that ， is in this already 
/// ， 1 429。 to 「 not 1 」
/// ，「 no 、request is to 」。
/// 3. ****：success when by request（`provider_loop::cap_cleared`）， down 
/// that old 1 to when ， 1 not 。
///
/// ── Why by provider （ not is ）──────────────────
/// is ** by ** ： 1 `gpt-6-astra` in workbuddy to 
/// `deepseek-v4.1-flash`， in catpaw can can to target。 need for 
/// **** ， 1 has —— not ， 1 has 
/// `wire_target_for_provider`（ 1 `model_rules::current()` and 
/// 1 ）。 is provider id， etc （≤6）。
///
/// use `Mutex` not is `RefCell`： will **** path has （`select_target_account`
/// is async）， `RefCell` not is `Sync` —— will handler future 
/// `Send`，failure（ this await in ）。 has 
/// ， and and `wire_target_for_provider` not （ in ，
/// ）， not in 。
pub struct CooldownKeys<'a> {
    /// clientrequest（，）
    requested: &'a str,
    /// provider id → this to up 
    resolved: Mutex<HashMap<String, String>>,
}

impl<'a> CooldownKeys<'a> {
    pub fn new(requested: &'a str) -> Self {
        Self { requested, resolved: Mutex::new(HashMap::new()) }
    }

    /// request（ by ： not not ， ）
    pub fn request_model(&self) -> &'a str {
        self.requested
    }

    /// this to up —— is 。
    ///
    /// not （request as empty / this not 、 no has to ） when return
    /// request： and `wire_target_for_provider` ④ 1 ——
    /// that when is request， this is 。
    ///
    /// in （ panic ） when ** not **、 —— is ，
    /// not this as 1 failure。
    pub fn for_provider(&self, provider_id: &str) -> String {
        if self.requested.is_empty() || provider_id.is_empty() {
            return self.requested.to_string();
        }
        if let Ok(cache) = self.resolved.lock() {
            if let Some(cached) = cache.get(provider_id) {
                return cached.clone();
            }
        }
        // `None` ：`wire_target_for_provider` current not （
        // by target before ， that ）， when no in ——
        // 「 by 」 in none from 。
        //
        // ── （）───────────────────
        // modelRules not custom id（ by ），
        // `wire_target_for_provider` for will returnrequest —— 
        // has （ up `mappings`，
        // `custom_providers::wire_model_for`）。 this 1 will alias request 
        // in request up ： is module header that 「 in 、 already 
        // in 」 ， this must by 。
        let wire =
            if crate::server::core::custom_providers::is_custom_provider_id(provider_id) {
                crate::server::core::providers::custom::forward::cooldown_model(
                    provider_id,
                    self.requested,
                )
            } else {
                crate::server::core::providers::catalog::wire_target_for_provider(
                    self.requested,
                    provider_id,
                    None,
                )
                .model
            };
        if let Ok(mut cache) = self.resolved.lock() {
            cache.insert(provider_id.to_string(), wire.clone());
        }
        wire
    }

    /// ： by （ 1 ）。
    pub fn for_account(&self, account: &Value) -> String {
        if let Some(model) = super::routing_groups::candidate_model(account) { return model.to_string(); }
        self.for_provider(provider_of(account))
    }
}

/// for is 。
///
/// for Node `isRateLimited(account, model, now)`： `rateLimits[model].resetAt`
/// is **** when ； in expired = （， not ）。
///
/// this `keys` request** up **（ by [`CooldownKeys`]）。
pub fn is_rate_limited(account: &Value, keys: &CooldownKeys<'_>, now: i64) -> bool {
    rate_limit_reset_at(account, keys, now) > 0
}

/// when （return 0）。
///
/// Node ：`Number(limit.resetAt) || 0`， has now return，
/// otherwisereturn 0 —— this 0 in 「currentavailable」， not is 「」。
///
/// by [`CooldownKeys`] （， not is request）。
pub fn rate_limit_reset_at(account: &Value, keys: &CooldownKeys<'_>, now: i64) -> i64 {
    let key = keys.for_account(account);
    let Some(limit) = account
        .get("rateLimits")
        .and_then(|limits| limits.get(key.as_str()))
    else {
        return 0;
    };
    let reset_at = limit
        .get("resetAt")
        .and_then(js_number)
        .unwrap_or(0.0);
    if reset_at > now as f64 {
        reset_at as i64
    } else {
        0
    }
}

/// is available：enabled + this 。
/// `reason` ∈ `Some("disabled")` | `Some("rate-limited")` | `None`（available）。
pub fn account_usability(account: &Value, keys: &CooldownKeys<'_>, now: i64) -> AccountUsability {
    // Node: `account?.enabled === false` —— has false use ，
    // / "false"/0 as enabled（ and account-store enabled() ）
    if matches!(account.get("enabled"), Some(Value::Bool(false))) {
        return AccountUsability { usable: false, reason: Some("disabled") };
    }
    if is_rate_limited(account, keys, now) {
        return AccountUsability { usable: false, reason: Some("rate-limited") };
    }
    AccountUsability { usable: true, reason: None }
}

/// `account_usability` 
#[derive(Clone, Copy, Debug)]
pub struct AccountUsability {
    pub usable: bool,
    pub reason: Option<&'static str>,
}

/// up and limit（** when in ** down request）。
///
/// 「 1 down request = 1 」 in `upstream::connections`（「」
/// ）， is and limit need limit ， this up 
/// `maxConcurrent` and that for 。 no has this 、 or not is = 0 = ** not **
/// —— and （`store_crud::apply_patch` 0） and 
/// （`store_util::max_concurrent_public` 0） 1 。
pub fn max_concurrent_of(account: &Value) -> u64 {
    account
        .get("maxConcurrent")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

/// by request use 。
///
/// `accounts` as store.listAccounts().accounts ；
/// `exclude_ids` is request already id（429 use ）。
/// return for ， or None（ no has available）。
///
/// `keys` is request to （， [`CooldownKeys`]）。
///
/// `counts` is **current in request**（ id → ，`upstream::connections`
/// ）：`maxConcurrent > 0` and in already limit ，request to 
/// its 。 no has when can use **empty** = not and （
/// `pick_for_model` / `describe_route_decision` ）。
///
/// ── and limit is **limit**（ and rebind ）────────
/// and request can can in 1 in 1 、 after +1 —— 
/// not to for 。 as not ： that 1-2 request
/// （request），must is 「 1 、」
/// 。 this ：** 1-2 ， not 1 **。
///
/// as 1 （ 1 by ）； and 
/// ， by when ， old 。
pub fn pick_account_by_priority(
    accounts: &[Value],
    keys: &CooldownKeys<'_>,
    counts: &HashMap<String, usize>,
    exclude_ids: &[String],
    now: i64,
) -> Option<Value> {
    let mut candidates: Vec<Value> = accounts
        .iter()
        .filter(|account| {
            let Some(id) = account_id(account) else {
                return false;
            };
            if exclude_ids.iter().any(|excluded| excluded == id) {
                return false;
            }
            account_usability(account, keys, now).usable
                // and in usability after ：「 this not use 」，
                // 「 not 」—— use / Reason not ， this 1 。
                && !at_concurrency_limit(account, id, counts)
        })
        .cloned()
        .collect();
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(compare_by_priority);
    candidates.into_iter().next()
}

/// this in request is already and limit（`maxConcurrent == 0` = not ， false）。
///
/// by 0（empty）：（ `Connections::snapshot`），
/// 「」 and 「0 in 」 is 1 。
fn at_concurrency_limit(account: &Value, id: &str, counts: &HashMap<String, usize>) -> bool {
    let limit = max_concurrent_of(account);
    limit > 0 && counts.get(id).copied().unwrap_or(0) >= limit as usize
}

/// by ****： to 「 has this 」，
/// [`pick_account_by_priority`] （enabled + this + ）。
///
/// ── Why need has this （`pick_current` not use ）──────────────
/// 「current」 is ** not ** （enabled + ）， need 
/// 「 for this in 」 。 in 「 」 when to not ：
/// ★ that ，request not will 。 need is 「 down 1 request will use 」，
/// must by ， `/api/session` `routedAccountId`。
///
/// ── Why need `providers` not is to model ──────────────────
/// ， 1 can ** has **（
/// `providers::router::route_for_forward`）。 this ， 1 「、
/// for this 、 not this 」 will 。
/// `providers` as empty（unknown） when not —— can ， not to empty。
///
/// `hasCredentials` ： after need to will use ，none 
/// ， before by 1 when not will this ★。
///
/// by （[`CooldownKeys`]）—— ★ and will 
/// that ：request after ，★ not will to that already 。
///
/// `counts` need for （ and ，
/// [`pick_account_by_priority`]）：★ to when （`/api/session`
/// from `UpstreamService::connections()` ）， not to empty = and not ——
/// can ★ 1 ， not need in when 。
pub fn pick_for_model(
    accounts: &[Value],
    model: &str,
    providers: &[&str],
    counts: &HashMap<String, usize>,
    now: i64,
) -> Option<Value> {
    let candidates: Vec<Value> = accounts
        .iter()
        .filter(|account| {
            if !providers.is_empty() && !providers.iter().any(|known| *known == provider_of(account))
            {
                return false;
            }
            !matches!(account.get("hasCredentials"), Some(Value::Bool(false)))
        })
        .cloned()
        .collect();
    let keys = CooldownKeys::new(model);
    pick_account_by_priority(&candidates, &keys, counts, &[], now)
}

/// up provider id（ by default provider ， and store 
/// `StoredAccount::provider` ）。
pub fn provider_of(account: &Value) -> &str {
    account
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID)
}

/// （ and ）：
///   `{ picked, priority, candidateCount, blocked: [{ id, name, reason }], total }`
///
/// Node workbuddy-routing.mjs `describeRouteDecision` for etc ：
/// path use not to （Node use ，only smoke test ），
/// when can for 「Why B not is A」， —— and Node 1 for 。
#[allow(dead_code)]
pub fn describe_route_decision(
    accounts: &[Value],
    model: &str,
    exclude_ids: &[String],
    now: i64,
) -> Value {
    let keys = CooldownKeys::new(model);
    let mut blocked: Vec<Value> = Vec::new();
    let mut candidate_count = 0usize;
    for account in accounts {
        let Some(id) = account_id(account) else {
            continue;
        };
        if exclude_ids.iter().any(|excluded| excluded == id) {
            blocked.push(json!({
                "id": id,
                "name": account_name(account, id),
                "reason": "tried",
            }));
            continue;
        }
        let usability = account_usability(account, &keys, now);
        if usability.usable {
            candidate_count += 1;
        } else {
            blocked.push(json!({
                "id": id,
                "name": account_name(account, id),
                "reason": usability.reason.unwrap_or(""),
            }));
        }
    }
    // ， not to when in （ in UpstreamService ），
    // empty = and not —— this 「 by enabled/ this 」。
    let picked = pick_account_by_priority(accounts, &keys, &HashMap::new(), exclude_ids, now);
    let priority = picked
        .as_ref()
        .map(|account| normalize_priority_value_of(account))
        .unwrap_or(Value::Null);
    json!({
        "picked": picked.unwrap_or(Value::Null),
        "priority": priority,
        "candidateCount": candidate_count,
        "blocked": blocked,
        "total": accounts.len(),
    })
}

/// id（empty； for Node `account?.id` ）
pub fn account_id(account: &Value) -> Option<&str> {
    account
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
}

/// ：`displayName`（ 1 ， `store_view::display_name_of`）
/// → `name` → id（ `displayName` when by old ）
pub fn account_name(account: &Value, id: &str) -> String {
    account
        .get("displayName")
        .filter(|value| js_truthy(value))
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            account.get("name").filter(|value| js_truthy(value)).and_then(|value| match value {
                Value::String(text) => Some(text.to_string()),
                other => Some(other.to_string()),
            })
        })
        .unwrap_or_else(|| id.to_string())
}

/// from store `{ currentAccountId, accounts: [...] }` 。
/// not for （） when returnempty —— and Node 
/// `Array.isArray(accounts) ? accounts : []` 。
pub fn accounts_of(snapshot: &Value) -> Vec<Value> {
    snapshot
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// ：， by when （ use account-store then ，
/// 「」 and 「」 1 ）。
fn compare_by_priority(a: &Value, b: &Value) -> std::cmp::Ordering {
    let key = |account: &Value| {
        (
            normalize_priority(account.get("routingPriority").or_else(|| account.get("priority")), crate::server::core::account_store::priority::DEFAULT_PRIORITY),
            js_number(account.get("addedAt").unwrap_or(&Value::Null)).unwrap_or(0.0),
        )
    };
    let (a_priority, a_added) = key(a);
    let (b_priority, b_added) = key(b);
    by_priority_order((a_priority, a_added as i64), (b_priority, b_added as i64))
}

/// priority（ 1 after ）—— for Node in `priority`。
/// this use store 1 after （Rust 1 this ）。
fn normalize_priority_value_of(account: &Value) -> Value {
    Value::from(normalize_priority(
        account.get("priority"),
        crate::server::core::account_store::priority::DEFAULT_PRIORITY,
    ))
}

/// JS `Number(x)`： not （NaN） when return None， by use by `|| 0` 。
fn js_number(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                // JS: Number('') === 0（ and Number('abc') NaN not ）
                Some(0.0)
            } else {
                trimmed.parse::<f64>().ok().filter(|value| value.is_finite())
            }
        }
        // Boolean / null / for / ：Number(true)=1、Number(null)=0、
        // its NaN。 this in not will ， by JS not 
        Value::Bool(flag) => Some(if *flag { 1.0 } else { 0.0 }),
        Value::Null => Some(0.0),
        _ => None,
    }
}

// ─── by （plan_route）────────────────────────────────────
//
// （ / ） and 
// （ / / ） in this ： 1 ****， 1 **Reason 
// **。（rotate.rs） by from after ；
// （api::routing） to use 。 is 1 ，
// 「 」 and 「 」 not can can 。

use crate::server::config::{RouteStrategy, StickyMode};

/// 1 in （ use `account`， use its ）。
#[derive(Clone, Debug)]
pub struct PlanEntry {
    pub account: Value,
    pub id: String,
    pub name: String,
    pub provider: String,
    /// ：`usable` / `tried` / `disabled` / `cooldown` / `concurrency` /
    /// `limiter` / `no-credentials`
    pub state: &'static str,
    /// 1 「Why this / Why」（ and request）
    pub reason: String,
    /// when （ms；0 = ）
    pub reset_at: i64,
}

/// 1 ：available（）+ （Reason）。
pub struct RoutePlan {
    pub ordered: Vec<PlanEntry>,
    pub blocked: Vec<PlanEntry>,
    /// in is and （available and excluding）
    pub sticky_hit: bool,
}

/// need ****（ 1 ， not ）。
pub struct PlanMeta<'a> {
    pub strategy: RouteStrategy,
    /// → when （）
    pub sunk: &'a HashMap<String, i64>,
    /// in → when （failure）
    pub resting: &'a HashMap<String, i64>,
    /// 
    pub open: &'a std::collections::HashSet<String>,
    /// limit （ not / Token ， use ）
    pub limiter_blocked: &'a std::collections::HashSet<String>,
    /// ****（`None` = not —— ）。 by `base_order` ，
    /// use is after （ by `RouteState::rotate_take`）。
    pub cursor: Option<usize>,
    /// （`Provider`） down ** **（provider id → ），
    /// by `provider_order` by after 。
    /// its not this ；`None` = not （ / already ）
    pub provider_cursors: Option<&'a HashMap<String, usize>>,
    /// in id（`None` = none in ）
    pub sticky_account: Option<&'a str>,
}

/// when → 「HH:MM」（Reason use ， by ** when ** —— and request
/// statistics 1 ： use is 「 this 」）。 when 「」 before 。
fn clock_text(reset_at: i64, now: i64) -> String {
    use chrono::{TimeZone, Timelike};
    let Some(secs) = reset_at.checked_div(1000) else {
        return String::new();
    };
    let Some(time) = chrono::Local.timestamp_opt(secs, 0).single() else {
        return String::new();
    };
    let days_ahead = (reset_at - now).max(0) / (24 * 3600 * 1000);
    let prefix = match days_ahead {
        0 => String::new(),
        1 => "明天 ".to_string(),
        _ => format!("{days_ahead} 天后 "),
    };
    format!("{prefix}{:02}:{:02}", time.hour(), time.minute())
}

/// available by after （ not / / that ）。
fn base_order(candidates: &[Value], strategy: RouteStrategy, meta: &PlanMeta<'_>) -> Vec<Value> {
    let mut sorted: Vec<Value> = candidates.to_vec();
    sorted.sort_by(compare_by_priority);
    match strategy {
        RouteStrategy::Order => sorted,
        RouteStrategy::Rotate => {
            let len = sorted.len();
            if len <= 1 {
                return sorted;
            }
            // `cursor` is ， in this 1 —— use is ** after ** 
            // （`sorted` already excluding use / / and / none）， and when that 
            // none（ `RouteState::rotate_take`）。
            let start = meta.cursor.unwrap_or(0) % len;
            (0..len)
                .map(|offset| sorted[(start + offset) % len].clone())
                .collect()
        }
        RouteStrategy::Provider => provider_order(sorted, meta),
    }
}

/// ： by ****， by 「 」
/// （ before ）， by 。
///
/// ── Why not in up ─────────────────────────────
/// `sorted` already by ， is 1 ** **； in this up is `Rotate`
/// （ 1 will to ）。 need is 「 after 、 has 」，
/// must this by 、， by 。
///
/// ── Why「 」────────────────────────────
/// path down `sorted` has ， 1 that is this ****
/// —— by ， is 「 
/// 」， not 1 。
///
/// ⚠️ has 1 ， is has as ：`plan_route` will ** in ** to 
/// before （ that 「 in available to before 」 1 ）。 that when to 
/// 1 ，、 not by —— is 「 this will 
/// use 1 」， is for 。 in will ， will 
/// by 。
fn provider_order(sorted: Vec<Value>, meta: &PlanMeta<'_>) -> Vec<Value> {
    let mut groups: Vec<(String, Vec<Value>)> = Vec::new();
    for account in sorted {
        let provider = super::routing_groups::family_key(&account).to_string();
        match groups.iter_mut().find(|(name, _)| *name == provider) {
            Some((_, members)) => members.push(account),
            None => groups.push((provider, vec![account])),
        }
    }
    let mut arranged = Vec::new();
    for (provider, members) in groups {
        let len = members.len();
        if len <= 1 {
            arranged.extend(members);
            continue;
        }
        // no has when from 0 —— and `Rotate` for `cursor: None` 1 。
        // is （`RouteState::rotate_take_per_provider`）， in this 
        // 1 、 for ** this 1 after ** —— has when will ，
        // 。
        let start = meta
            .provider_cursors
            .and_then(|cursors| cursors.get(&provider))
            .copied()
            .unwrap_or(0)
            % len;
        for offset in 0..len {
            arranged.push(members[(start + offset) % len].clone());
        }
    }
    arranged
}

/// statistics 1 （`provider id → `， by ）。
///
/// use 「 has 」， as 1 （`rotate_take_per_provider`）。
/// not and —— in `provider_order` by ** after ** 
/// （ `RouteState::rotate_take` that ）。
pub fn provider_sizes(accounts: &[Value]) -> Vec<(String, usize)> {
    let mut sizes: Vec<(String, usize)> = Vec::new();
    for account in accounts {
        let provider = super::routing_groups::family_key(account).to_string();
        match sizes.iter_mut().find(|(name, _)| *name == provider) {
            Some((_, count)) => *count += 1,
            None => sizes.push((provider, 1)),
        }
    }
    sizes
}

/// 1 （ and use ** 1 **）。
///
/// `excluded` is already （429 / failure）， `blocked`
/// （state = `tried`）——「 this request not will 」 is when need 。
pub fn plan_route(
    accounts: &[Value],
    keys: &CooldownKeys<'_>,
    counts: &HashMap<String, usize>,
    excluded: &[String],
    now: i64,
    meta: &PlanMeta<'_>,
) -> RoutePlan {
    let mut ordered: Vec<Value> = Vec::new();
    let mut blocked: Vec<PlanEntry> = Vec::new();
    for account in accounts {
        let Some(id) = account_id(account) else { continue };
        let id = id.to_string();
        let name = account_name(account, &id);
        let provider = provider_of(account).to_string();
        let entry = |state: &'static str, reason: String| PlanEntry {
            account: account.clone(),
            id: id.clone(),
            name: name.clone(),
            provider: provider.clone(),
            state,
            reason,
            reset_at: rate_limit_reset_at(account, keys, now),
        };
        let candidate_id = super::routing_groups::attempt_key(account);
        if excluded.iter().any(|tried| tried == &candidate_id) {
            blocked.push(entry("tried", "本轮已试过，不再回试".to_string()));
            continue;
        }
        if matches!(account.get("enabled"), Some(Value::Bool(false))) {
            blocked.push(entry("disabled", "已禁用，不参与转发".to_string()));
            continue;
        }
        if meta.limiter_blocked.contains(&candidate_id) {
            blocked.push(entry(
                "limiter",
                "被限制器拦下（余额不足或 Token 周期用量到顶），自动恢复后回来".to_string(),
            ));
            continue;
        }
        let reset_at = rate_limit_reset_at(account, keys, now);
        if reset_at > 0 {
            blocked.push(entry(
                "cooldown",
                format!(
                    "对该模型限流冷却中，{} 恢复（恢复后自动回来）",
                    clock_text(reset_at, now)
                ),
            ));
            continue;
        }
        if at_concurrency_limit(account, &id, counts) {
            blocked.push(entry(
                "concurrency",
                format!(
                    "并发已满（{}/上限），等一个在途回落再参与",
                    counts.get(&id).copied().unwrap_or(0)
                ),
            ));
            continue;
        }
        if matches!(account.get("hasCredentials"), Some(Value::Bool(false))) {
            blocked.push(entry("no-credentials", "没有可用凭证，登录后才能接请求".to_string()));
            continue;
        }
        ordered.push(account.clone());
    }

    // （）→ ： no in before → no in before → no in before 
    // ****：（Sink） is 「 to 1 1 
    // 429」， is 「failure」—— in down ；
    // if for ，default down this is 。
    // after by （ down = ）。
    let mut arranged = base_order(&ordered, meta.strategy, meta);

    // ── in available to before （available，request will —— must）
    //
    // ⚠️ this 1 **must in `base_order` after **（2026-10-13 ）：`base_order` 
    // `sort_by(compare_by_priority)` ， if in before ， will 
    // that by —— in 「 in is 」 when 
    // ， in when request to （2026-10-13 in `.3` up to ：
    // 1 trace request in priority=107 and priority=100 up ）。
    // up is in before ， has ； to this 。
    //
    // in this has 1 ： down use is `sort_by_key`（****），
    // for ， not will ； in before 
    // in if / ， this —— in can to this 1 
    // available（unavailable in already `blocked`）， not 。
    let mut sticky_hit = false;
    if let Some(hit) = meta.sticky_account {
        if let Some(index) = arranged
            .iter()
            .position(|account| super::routing_groups::attempt_key(account) == hit)
        {
            let hit_account = arranged.remove(index);
            arranged.insert(0, hit_account);
            sticky_hit = true;
        }
    }

    let rank_of = |account: &Value, table: &HashMap<String, i64>| -> bool {
        account_id(account)
            .and_then(|id| table.get(id))
            .map(|until| *until > 0)
            .unwrap_or(false)
    };
    arranged.sort_by_key(|account| rank_of(account, meta.sunk));
    arranged.sort_by_key(|account| rank_of(account, meta.resting));
    // in （all 1 ）： has not when to 1 
    arranged.sort_by_key(|account| {
        account_id(account)
            .map(|id| meta.open.contains(id))
            .unwrap_or(false)
    });

    // 「 before is not is is 」—— `Provider` Reason
    // need use ： in 2 if is ** 1 down 1 **（ A = a1/a2、 B = b1），
    // before and no has 「 」， is 。 before that 1 
    // 「 etc or failure to 」， is 1 and 
    // （2026-10-14 review ）。
    let prefix_is_same_family: Vec<bool> = {
        let mut seen: Vec<String> = Vec::new();
        let mut flags = Vec::with_capacity(arranged.len());
        for account in &arranged {
            let provider = super::routing_groups::family_key(account).to_string();
            flags.push(seen.iter().all(|name| *name == provider));
            if !seen.iter().any(|name| *name == provider) {
                seen.push(provider);
            }
        }
        flags
    };

    let ordered_entries: Vec<PlanEntry> = arranged
        .into_iter()
        .enumerate()
        .map(|(index, account)| {
            let id = account_id(&account).unwrap_or("").to_string();
            let position = index + 1;
            let mut reason = if sticky_hit && Some(super::routing_groups::attempt_key(&account).as_str()) == meta.sticky_account {
                // 「 use 」 in ** ** in when （ after has 
                // ， if in / / ， will ）。
                // not this 1 will Reason and —— 「 use 」，
                // in 10 （2026-10-13 review ）。
                if position == 1 {
                    "会话粘住：这个会话上一轮是它答的，本轮继续用它".to_string()
                } else {
                    format!("会话粘住，但它当前排在第 {position} 位（正在让位于下面的状态）")
                }
            } else {
                match meta.strategy {
                    RouteStrategy::Order => {
                        if position == 1 {
                            "队列第 1 位：顺序优先下它接走所有请求，直到不能答".to_string()
                        } else {
                            format!("队列第 {position} 位：前面的都可用时轮不到它")
                        }
                    }
                    RouteStrategy::Rotate => {
                        if position == 1 {
                            "本轮轮到它：轮流均摊下每条请求换下一位".to_string()
                        } else {
                            format!("排在第 {position} 位：等前面的轮完或失败才到它")
                        }
                    }
                    RouteStrategy::Provider => {
                        if position == 1 {
                            "本轮轮到它：家之间按优先级、家内轮流，它是当前该上的那一位".to_string()
                        } else if prefix_is_same_family[index] {
                            // after 1 ： before no has ，「 etc 」 is 
                            format!("排在第 {position} 位：同一家内轮流，轮到它才上")
                        } else {
                            format!("排在第 {position} 位：等优先级更高的家轮完或失败才到它")
                        }
                    }
                }
            };
            if meta.sunk.contains_key(&id) {
                // `route_state::SINK_TTL_MS` ： in has 30 minutes，
                // not is 「 1 」（ that is up ， that when not ）。
                reason.push_str("；曾还有额度时被 429（垫底中，30 分钟后自动回到队列）");
            }
            if let Some(until) = meta.resting.get(&id) {
                reason.push_str(&format!("；转发失败休息至 {}", clock_text(*until, now)));
            }
            if meta.open.contains(id.as_str()) {
                reason.push_str("；熔断打开中（连续失败到阈值），到点放行试一次");
            }
            PlanEntry {
                name: account_name(&account, &id),
                provider: provider_of(&account).to_string(),
                account,
                id,
                state: "usable",
                reason,
                reset_at: 0,
            }
        })
        .collect();

    RoutePlan {
        ordered: ordered_entries,
        blocked,
        sticky_hit,
    }
}

/// TTL（ms）： and use （StickyMode → when ）。
pub fn sticky_ttl_ms(mode: StickyMode) -> i64 {
    mode.ttl_ms().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 1 meta：， its as empty。
    fn provider_meta<'a>(
        cursors: &'a HashMap<String, usize>,
        empty_numbers: &'a HashMap<String, i64>,
        empty_ids: &'a std::collections::HashSet<String>,
    ) -> PlanMeta<'a> {
        PlanMeta {
            strategy: RouteStrategy::Provider,
            sunk: empty_numbers,
            resting: empty_numbers,
            open: empty_ids,
            limiter_blocked: empty_ids,
            cursor: None,
            provider_cursors: Some(cursors),
            sticky_account: None,
        }
    }

    fn account(id: &str, provider: &str, priority: i64) -> Value {
        json!({ "id": id, "provider": provider, "priority": priority })
    }

    fn ordered_ids(accounts: &[Value], meta: &PlanMeta<'_>) -> Vec<String> {
        provider_order(accounts.to_vec(), meta)
            .iter()
            .map(|account| account_id(account).unwrap_or("").to_string())
            .collect()
    }

    /// ：， has 。
    #[test]
    fn provider_order_rotates_inside_a_family_but_keeps_families_in_priority_order() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        // A to 1 （a2）；B no has ， from 0 
        let cursors: HashMap<String, usize> = HashMap::from([("A".to_string(), 1usize)]);
        let meta = provider_meta(&cursors, &empty_numbers, &empty_ids);
        // already is 「 by 」 （plan_route ）
        let sorted = vec![
            account("a1", "A", 100),
            account("a2", "A", 110),
            account("b1", "B", 120),
        ];
        // A in before （ 100 ） and to a2；B in A after 
        assert_eq!(ordered_ids(&sorted, &meta), vec!["a2", "a1", "b1"]);
    }

    #[test]
    fn provider_order_without_a_cursor_starts_at_each_familys_top() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let cursors: HashMap<String, usize> = HashMap::new();
        let meta = provider_meta(&cursors, &empty_numbers, &empty_ids);
        let sorted = vec![
            account("a1", "A", 100),
            account("a2", "A", 110),
            account("b1", "B", 120),
        ];
        assert_eq!(ordered_ids(&sorted, &meta), vec!["a1", "a2", "b1"]);
    }

    /// by **current**： 、 when not 
    #[test]
    fn provider_cursor_wraps_instead_of_going_out_of_range() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let cursors: HashMap<String, usize> = HashMap::from([("A".to_string(), 7usize)]);
        let meta = provider_meta(&cursors, &empty_numbers, &empty_ids);
        let sorted = vec![account("a1", "A", 100), account("a2", "A", 110)];
        // 7 % 2 = 1 → from a2 
        assert_eq!(ordered_ids(&sorted, &meta), vec!["a2", "a1"]);
    }

    #[test]
    fn provider_sizes_counts_each_family() {
        let accounts = vec![
            account("a1", "A", 100),
            account("b1", "B", 120),
            account("a2", "A", 110),
        ];
        assert_eq!(
            provider_sizes(&accounts),
            vec![("A".to_string(), 2), ("B".to_string(), 1)]
        );
    }

    #[test]
    fn provider_strategy_round_trips_through_its_config_id() {
        assert_eq!(RouteStrategy::from_id("provider"), RouteStrategy::Provider);
        assert_eq!(RouteStrategy::Provider.as_str(), "provider");
    }

    /// `Provider` Reason：** down 1 ** not 「 etc 」。
    ///
    /// A = a1/a2、 B = b1 —— 2 is a2， before has a1， before 
    /// ** no has ** 。 before this 1 「 etc or failure to 
    /// 」， is 1 and （2026-10-14 review ）。Reason is 
    /// when 1 need ， not can is 。
    ///
    /// `plan_route`：Reason by ，`provider_order` not 。
    #[test]
    fn a_sibling_in_the_same_family_is_not_explained_by_a_higher_priority_family() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let cursors: HashMap<String, usize> = HashMap::new();
        let meta = provider_meta(&cursors, &empty_numbers, &empty_ids);
        let accounts = vec![
            account("a1", "A", 100),
            account("a2", "A", 110),
            account("b1", "B", 120),
        ];
        let plan = plan_route(
            &accounts,
            &CooldownKeys::new("test-model"),
            &counts,
            &[],
            1_700_000_000_000,
            &meta,
        );
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(order, vec!["a1", "a2", "b1"]);
        // 2 is a2
        assert!(plan.ordered[1].reason.contains("同一家内轮流"), "{}", plan.ordered[1].reason);
        assert!(!plan.ordered[1].reason.contains("优先级更高的家"), "{}", plan.ordered[1].reason);
        // 3 b1 before has A ， by 「 」
        assert!(plan.ordered[2].reason.contains("优先级更高的家"), "{}", plan.ordered[2].reason);
    }

    /// in must to 1 —— 。
    ///
    /// this is 2026-10-13 1 ** up has **： that 1 in 
    /// `base_order` before ， `base_order` `sort_by(compare_by_priority)`
    /// ， ⇒ in 「 in is 」 when 。
    /// ：`.3` up 1 trace request in priority=107 and 100 up 。
    ///
    /// `plan_route`（ not is `provider_order`）： and 
    /// ** after ** is this use need ， has from to 。
    #[test]
    fn sticky_pinning_survives_the_base_order_sort() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let keys = CooldownKeys::new("test-model");
        let accounts = vec![
            account("a1", "A", 100),
            account("a2", "A", 110),
            account("a3", "A", 120),
        ];
        // in a3 —— when must 1 
        let meta = PlanMeta {
            strategy: RouteStrategy::Order,
            sunk: &empty_numbers,
            resting: &empty_numbers,
            open: &empty_ids,
            limiter_blocked: &empty_ids,
            cursor: None,
            provider_cursors: None,
            sticky_account: Some("a3"),
        };
        let plan = plan_route(&accounts, &keys, &counts, &[], 1_700_000_000_000, &meta);
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(
            order,
            vec!["a3", "a1", "a2"],
            "粘性命中的 a3 必须排在队首（粘性的全部意义就是「继续用它」）"
        );
        assert!(plan.sticky_hit);
        // down by （ in that 1 ， not ）
        assert_eq!(plan.ordered[1].id, "a1");
        assert_eq!(plan.ordered[2].id, "a2");
    }

    /// down 1 ： in after ， ** its **。
    #[test]
    fn sticky_pinning_survives_under_the_provider_strategy() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let keys = CooldownKeys::new("test-model");
        let cursors: HashMap<String, usize> = HashMap::from([("A".to_string(), 1usize)]);
        let accounts = vec![
            account("a1", "A", 100),
            account("a2", "A", 110),
            account("b1", "B", 120),
        ];
        let meta = PlanMeta {
            strategy: RouteStrategy::Provider,
            sunk: &empty_numbers,
            resting: &empty_numbers,
            open: &empty_ids,
            limiter_blocked: &empty_ids,
            cursor: None,
            provider_cursors: Some(&cursors),
            sticky_account: Some("b1"),
        };
        let plan = plan_route(&accounts, &keys, &counts, &[], 1_700_000_000_000, &meta);
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        // b1 （ B ， after ）⇒ ， this is 
        assert_eq!(order, vec!["b1", "a2", "a1"]);
    }

    /// if in ， will —— that when Reason
    /// ** not can **「 use 」（2026-10-13 review ： in 
    /// before ， will 「 use 」 2 ）。
    #[test]
    fn a_sticky_account_that_is_sunk_says_so_instead_of_claiming_it_will_be_used() {
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let keys = CooldownKeys::new("test-model");
        let accounts = vec![account("a1", "A", 100), account("a2", "A", 110)];
        // a2 ， when in sink 
        let sunk: HashMap<String, i64> = HashMap::from([("a2".to_string(), 1_700_000_000_000i64)]);
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let meta = PlanMeta {
            strategy: RouteStrategy::Order,
            sunk: &sunk,
            resting: &empty_numbers,
            open: &empty_ids,
            limiter_blocked: &empty_ids,
            cursor: None,
            provider_cursors: None,
            sticky_account: Some("a2"),
        };
        let plan = plan_route(&accounts, &keys, &counts, &[], 1_700_000_000_000, &meta);
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        // a2 2 （ —— this is need ）
        assert_eq!(order, vec!["a1", "a2"]);
        assert!(plan.sticky_hit, "命中过就该记着（sticky_hit 表示「查到过」）");
        // Reasonmust： in ， not is 「 use 」
        let a2 = &plan.ordered[1];
        assert!(
            a2.reason.contains("排在第 2 位"),
            "垫底中的粘住账号不该声称「本轮继续用它」，实际原因句：{}",
            a2.reason
        );
        assert!(!a2.reason.contains("本轮继续用它"));
        // a1 Reason（ not is that ）
        assert!(plan.ordered[0].reason.contains("队列第 1 位"));
    }

    /// ** `plan_route`** ： by 、 by 。
    ///
    /// up `provider_order_*` that ； this 1 from `plan_route` ，
    /// is 「」—— to `base_order`、`provider_cursors` 
    /// to 。 ， `provider_cursors` `None` 。
    #[test]
    fn the_provider_strategy_keeps_families_in_priority_order_and_rotates_inside() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let keys = CooldownKeys::new("test-model");
        // B （120） A （100）⇒ B in A after 
        let accounts = vec![
            account("a1", "A", 100),
            account("b1", "B", 120),
            account("a2", "A", 110),
            account("b2", "B", 130),
        ];
        // is 1 ⇒ from 
        let cursors: HashMap<String, usize> =
            HashMap::from([("A".to_string(), 1usize), ("B".to_string(), 1usize)]);
        let meta = PlanMeta {
            strategy: RouteStrategy::Provider,
            sunk: &empty_numbers,
            resting: &empty_numbers,
            open: &empty_ids,
            limiter_blocked: &empty_ids,
            cursor: None,
            provider_cursors: Some(&cursors),
            sticky_account: None,
        };
        let plan = plan_route(&accounts, &keys, &counts, &[], 1_700_000_000_000, &meta);
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        // A in before （ from a2 ），B in after （ from b2 ）
        assert_eq!(order, vec!["a2", "a1", "b2", "b1"]);
        // Reason is that 1 （ not is / ）
        assert!(
            plan.ordered[0].reason.contains("家之间按优先级"),
            "队首原因句应当是家内轮询的口径，实际：{}",
            plan.ordered[0].reason
        );
    }

    /// down ** no has **（ / ） when ， from 。
    #[test]
    fn the_provider_strategy_without_cursors_shows_each_family_from_its_top() {
        let empty_numbers: HashMap<String, i64> = HashMap::new();
        let empty_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let counts: HashMap<String, usize> = HashMap::new();
        let keys = CooldownKeys::new("test-model");
        let accounts = vec![
            account("a1", "A", 100),
            account("b1", "B", 120),
            account("a2", "A", 110),
            account("b2", "B", 130),
        ];
        let meta = PlanMeta {
            strategy: RouteStrategy::Provider,
            sunk: &empty_numbers,
            resting: &empty_numbers,
            open: &empty_ids,
            limiter_blocked: &empty_ids,
            cursor: None,
            provider_cursors: None,
            sticky_account: None,
        };
        let plan = plan_route(&accounts, &keys, &counts, &[], 1_700_000_000_000, &meta);
        let order: Vec<&str> = plan.ordered.iter().map(|entry| entry.id.as_str()).collect();
        assert_eq!(order, vec!["a1", "a2", "b1", "b2"]);
    }
}


