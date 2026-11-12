//! 「limit」： can limit then （ / Token ） and 
//! Token 。
//!
//! ── this ─────────────────────────────────────
//! old 「 not 」 has 1 config（`lowBalance {mode, threshold}`），
//! not 「 this when use 50 Token」。limit then 
//! （ up `limiters` ）： then = （ / Token）+ +
//! （ / use ）+ enabled，Token then **** —— 
//! （ for when ，30 minutes ~ 24 when ） or （ when 0 
//! ， has 0 ）。
//!
//! ── and old `lowBalance` （ + ）──────────
//! up ** no has ** `limiters` when （ old ），valid then by 
//! `lowBalance` / provider **** 1 then —— as and before 
//! 1 （ [`effective_rules`]）。（`store_crud::apply_patch`）
//! `limiters` when then `lowBalance`： use old 
//! when limit not （Token then old not ，ignore can ）。
//!
//! ── Token ────────────────────────────────────
//! 「current」 from `requests` by 
//! （`SUM(total_tokens) WHERE ts >= `，schema v9 ）。
//! ****（`TokenFact`， = × seconds）， ：
//! - ** new **：`usage_query` （10 seconds） currentenabled in 
//! Token 1 ， —— （ for when ） after old 
//! ；
//! - ****：request（`api::pipeline::record_entry`） 
//! current —— not etc down 1 new ，path up no has IO。
//!
//! （[`token_skip_blocked`]） and 1 ：、 IO、
//! by 「」—— （ no has this 、 or already ）
//! by 0 ， not 。
//!
//! ── ：（） not ──────────────────────
//! and `usage_records` 1 ：`Db::with*` is 1 ，
//! will 。 1 in 。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::{json, Value};

use crate::server::core::account_store::store_util::js_truthy;
use crate::server::core::providers::{kind_id, ProviderKind};
use crate::server::core::usage_records::{default_low_balance_mode, DEFAULT_LOW_BALANCE_THRESHOLD};
use crate::server::db::Db;
use crate::server::logging;
use agent2api_provider_protocol as proto;

/// Cline provider id（ and `usage_records::CLINE_FREE_PROVIDER` ）。
const CLINE_FREE_PROVIDER: &str = kind_id(ProviderKind::ClineFree);
/// Token then up down （seconds）：30 minutes ~ 24 when 。 and 
/// （`store_crud::apply_patch`） use this 1 for —— 1 「 will 」。
/// down and down （30 seconds） not ： by when for ， will 
/// 「 」，30 minutes is available and in （ ）。
pub const MIN_TOKEN_PERIOD_SECONDS: i64 = 1_800;
pub const MAX_TOKEN_PERIOD_SECONDS: i64 = 86_400;

static DB: OnceLock<Option<Db>> = OnceLock::new();

/// （`ServerState::bootstrap` 1 ， and `usage_records::install` and ）。
pub fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn database() -> Option<&'static Db> {
    DB.get().and_then(Option::as_ref)
}

// ─── then and ────────────────────────────────────────

/// then 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimiterKind {
    /// ：（ and old 「 not 」 1 ，）。
    Balance,
    /// Token： to （ etc ）。
    Token,
}

/// Token then 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimiterReset {
    /// ： for when （ / N minutes 1 ）， = `period`。
    Fixed,
    /// ： when 0 （ has 0 ，
    /// 「 N when 」 not this ）。 not use `period`。
    Daily,
}

/// ： and old 1 —— （ / ），
/// use not （ need enabled）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimiterAction {
    Skip,
    Disable,
}

/// 1 limit then （；`period` only「」 Token then valid， its 0）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimiterRule {
    pub kind: LimiterKind,
    pub action: LimiterAction,
    /// ： = down ；Token = limit。 > 0。
    pub threshold: f64,
    /// （seconds）；only「」 Token then valid。
    pub period: i64,
    /// ； then Fixed（，none）。
    pub reset: LimiterReset,
    pub enabled: bool,
}

impl LimiterRule {
    /// 1 then for （， before not 「」）。
    /// daily not period： no has ， has 「 0 」 this 1 。
    pub fn to_json(self) -> Value {
        let kind = match self.kind {
            LimiterKind::Balance => "balance",
            LimiterKind::Token => "token",
        };
        let action = match self.action {
            LimiterAction::Skip => "skip",
            LimiterAction::Disable => "disable",
        };
        let reset = match self.reset {
            LimiterReset::Fixed => "fixed",
            LimiterReset::Daily => "daily",
        };
        let mut object = json!({
            "type": kind,
            "action": action,
            "threshold": self.threshold,
            "reset": reset,
            "enabled": self.enabled,
        });
        if self.kind == LimiterKind::Token && self.reset == LimiterReset::Fixed {
            object["period"] = Value::from(self.period);
        }
        object
    }
}

/// from `limiters` then （****：， not 1 
/// its then ）。 / return `None` —— use 
/// 「 old lowBalance / provider 」 。
fn parse_rules(value: Option<&Value>) -> Option<Vec<LimiterRule>> {
    let entries = value?.as_array()?;
    Some(
        entries
            .iter()
            .filter_map(|entry| {
                let fields = entry.as_object()?;
                let kind = match fields.get("type").and_then(Value::as_str) {
                    Some("balance") => LimiterKind::Balance,
                    Some("token") => LimiterKind::Token,
                    _ => return None,
                };
                let action = match fields.get("action").and_then(Value::as_str) {
                    Some("skip") => LimiterAction::Skip,
                    Some("disable") => LimiterAction::Disable,
                    _ => return None,
                };
                // reset = fixed（ old no has this ， as and before 1 ）
                let reset = match fields.get("reset").and_then(Value::as_str) {
                    Some("daily") => LimiterReset::Daily,
                    _ => LimiterReset::Fixed,
                };
                let threshold = fields.get("threshold").and_then(Value::as_f64)?;
                if !threshold.is_finite() || threshold <= 0.0 {
                    return None;
                }
                // period for 「」 Token then need ；（）
                // ， not —— 「 90 24 when 」 will use 
                // ， can not 。daily then no has ，period 0。
                let period = if kind == LimiterKind::Token && reset == LimiterReset::Fixed {
                    let period = fields.get("period").and_then(Value::as_i64)?;
                    if !(MIN_TOKEN_PERIOD_SECONDS..=MAX_TOKEN_PERIOD_SECONDS).contains(&period) {
                        return None;
                    }
                    period
                } else {
                    0
                };
                let enabled = fields.get("enabled").map(js_truthy).unwrap_or(true);
                Some(LimiterRule { kind, action, threshold, period, reset, enabled })
            })
            .collect(),
    )
}

/// up ** no has ** `limiters` when ： by `lowBalance`（config） or 
/// provider 1 then —— and limit up before as 1 。
/// `limiters`（empty）：empty = use then = unlimited。
/// provider **limit then **（ up no has `limiters` no has `lowBalance`
/// config when use ）。
///
/// ── ：， ──────────────────────
/// then is 「 this 1 」 ， provider ，
/// in when （`AccountFeatures::default_limiters`）。 this ：
/// （「 as empty」 this 1 ）， no down ——
/// is no old old ， after as by 。
///
/// when ：Cline none then ， its provider by total < 1 。
pub fn default_rules(provider: &str) -> Vec<LimiterRule> {
    if let Some(declared) = declared_rules(provider) {
        if provider != kind_id(ProviderKind::CodeArts) || declared.iter().all(|rule| rule.kind == LimiterKind::Balance) {
            return declared;
        }
    }
    if provider == CLINE_FREE_PROVIDER {
        return Vec::new();
    }
    vec![LimiterRule {
        kind: LimiterKind::Balance,
        action: LimiterAction::Skip,
        threshold: DEFAULT_LOW_BALANCE_THRESHOLD,
        period: 0,
        reset: LimiterReset::Fixed,
        enabled: true,
    }]
}

/// then （`None` = this 1 no ， use ）。
///
/// `DefaultLimiter` and this `LimiterRule` is 1 ：
/// before is （、）， after is （ `reset`
/// this ）。 in this 1 ， not need 1 。
fn declared_rules(provider: &str) -> Option<Vec<LimiterRule>> {
    let features = crate::server::core::plugin_host::account_features_of(provider)?;
    let declared = features.default_limiters?;
    Some(
        declared
            .into_iter()
            .filter(|rule| rule.threshold.is_finite() && rule.threshold > 0.0)
            .map(|rule| {
                let (kind, reset) = match rule.kind {
                    proto::DefaultLimiterKind::Balance => {
                        (LimiterKind::Balance, LimiterReset::Fixed)
                    }
                    proto::DefaultLimiterKind::Token => {
                        // `period_seconds == 0` = ；otherwise is 
                        let reset = if rule.period_seconds == 0 {
                            LimiterReset::Daily
                        } else {
                            LimiterReset::Fixed
                        };
                        (LimiterKind::Token, reset)
                    }
                };
                let action = match rule.action {
                    proto::DefaultLimiterAction::Skip => LimiterAction::Skip,
                    proto::DefaultLimiterAction::Disable => LimiterAction::Disable,
                };
                LimiterRule {
                    kind,
                    action,
                    threshold: rule.threshold,
                    period: i64::from(rule.period_seconds),
                    reset,
                    enabled: true,
                }
            })
            .collect(),
    )
}

/// up ** no has ** `limiters` when ： has `lowBalance`（ use / old 
/// ）； no has when by provider to then （[`default_rules`]）。
fn derive_from_legacy(fields: &serde_json::Map<String, Value>) -> Vec<LimiterRule> {
    let provider = fields
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID);
    let Some(config) = fields.get("lowBalance") else {
        return default_rules(provider);
    };
    let mode = config
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or(default_low_balance_mode(provider));
    let threshold = config
        .get("threshold")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(DEFAULT_LOW_BALANCE_THRESHOLD);
    let (kind, action) = match mode {
        "skip" => (LimiterKind::Balance, LimiterAction::Skip),
        "disable" => (LimiterKind::Balance, LimiterAction::Disable),
        _ => return Vec::new(),
    };
    vec![LimiterRule { kind, action, threshold, period: 0, reset: LimiterReset::Fixed, enabled: true }]
}

/// this **validlimit then **： `limiters` ， when by old config。
/// （`rotate`）、 use （`usage_query`）、
/// （`store_view`） and （`store_crud`） use this 1 ——
/// 「 1 then 」 has this 1 。（ fields ）。
pub fn effective_rules_in(fields: &serde_json::Map<String, Value>) -> Vec<LimiterRule> {
    match parse_rules(fields.get("limiters")) {
        Some(rules) => rules,
        None => derive_from_legacy(fields),
    }
}

/// [`effective_rules_in`] JSON （ / is for ）。
pub fn effective_rules(account: &Value) -> Vec<LimiterRule> {
    match account.as_object() {
        Some(fields) => effective_rules_in(fields),
        // （ for ） no has can config： not to then （ not ），
        // and 「 not not 」 total 1 
        None => Vec::new(),
    }
}

/// then → JSON （ / use ）。
pub fn rules_to_json(rules: &[LimiterRule]) -> Value {
    Value::Array(rules.iter().map(|rule| rule.to_json()).collect())
}

/// valid then → JSON （`store_view` / use ）。
pub fn effective_rules_json_in(fields: &serde_json::Map<String, Value>) -> Value {
    rules_to_json(&effective_rules_in(fields))
}

/// 1 （）：must is then for ，
/// otherwise（ not —— use in up 1 ， need ）。
/// by 「」（return then ）， and `lowBalance` for 1 。
pub fn normalize_rules(
    value: &Value,
    fields: &serde_json::Map<String, Value>,
) -> Result<Vec<LimiterRule>, String> {
    let Some(entries) = value.as_array() else {
        return Ok(effective_rules_in(fields));
    };
    let mut rules = Vec::new();
    for entry in entries {
        let Some(fields) = entry.as_object() else {
            return Err("限制规则的每一项都必须是对象".to_string());
        };
        let kind = match fields.get("type").and_then(Value::as_str) {
            Some("balance") => LimiterKind::Balance,
            Some("token") => LimiterKind::Token,
            _ => return Err("限制规则的类型必须是「余额 / Token」之一".to_string()),
        };
        let action = match fields.get("action").and_then(Value::as_str) {
            Some("skip") => LimiterAction::Skip,
            Some("disable") => LimiterAction::Disable,
            _ => return Err("限制规则的触发动作必须是「跳过 / 禁用」之一".to_string()),
        };
        let threshold = fields
            .get("threshold")
            .and_then(Value::as_f64)
            .ok_or_else(|| "限制阈值必须是大于 0 的数字".to_string())?;
        if !threshold.is_finite() || threshold <= 0.0 {
            return Err("限制阈值必须是大于 0 的数字".to_string());
        }
        // reset = fixed（ before old not this can ）；， not 
        let reset = match fields.get("reset").and_then(Value::as_str) {
            None | Some("fixed") => LimiterReset::Fixed,
            Some("daily") => LimiterReset::Daily,
            Some(_) => return Err("Token 重置方式必须是「固定周期 / 自然日」之一".to_string()),
        };
        let period = if kind == LimiterKind::Token && reset == LimiterReset::Fixed {
            let period = fields.get("period").and_then(Value::as_i64).ok_or_else(|| {
                "Token 规则必须带重置周期（整数秒）".to_string()
            })?;
            if !(MIN_TOKEN_PERIOD_SECONDS..=MAX_TOKEN_PERIOD_SECONDS).contains(&period) {
                return Err(format!(
                    "Token 重置周期必须是 {} ~ {} 秒（30 分钟 ~ 24 小时）",
                    MIN_TOKEN_PERIOD_SECONDS, MAX_TOKEN_PERIOD_SECONDS
                ));
            }
            period
        } else {
            0
        };
        let enabled = fields.get("enabled").map(js_truthy).unwrap_or(true);
        rules.push(LimiterRule { kind, action, threshold, period, reset, enabled });
    }
    Ok(rules)
}

// ─── （ / use use ）───────────────────────

/// ： 1 enabled skip then 「」。
/// `facts` is `usage_records::balance_facts()` （ not = ，
/// that not ）。 old by `lowBalance.mode` config —— then 
/// old to 1 。
pub fn balance_skip_blocked(account: &Value, facts: &HashMap<String, crate::server::core::usage_records::BalanceFact>) -> bool {
    let Some(id) = account.get("id").and_then(Value::as_str) else { return false };
    let fact = facts.get(id);
    effective_rules(account).iter().any(|rule| {
        rule.kind == LimiterKind::Balance
            && rule.action == LimiterAction::Skip
            && rule.enabled
            && fact.is_some_and(|fact| matches!(fact.remaining, Some(remaining) if remaining < rule.threshold))
    })
}

/// use ： 1 enabled disable then 「 < 」 this ，
/// etc its in **** （`remaining < max` ⇔ 1 in ）。
/// `None` = no has this then （ not enabled use not ）。
pub fn balance_disable_threshold(account: &Value) -> Option<f64> {
    effective_rules(account)
        .iter()
        .filter(|rule| {
            rule.kind == LimiterKind::Balance && rule.action == LimiterAction::Disable && rule.enabled
        })
        .map(|rule| rule.threshold)
        .fold(None, |max: Option<f64>, threshold| {
            Some(match max {
                Some(current) if current >= threshold => current,
                _ => threshold,
            })
        })
}

/// current when in （ms）： for when （ / N minutes 1 ）。
/// 「」 can （ can to when ）。
pub fn window_start(now_ms: i64, period_seconds: i64) -> i64 {
    let period_ms = period_seconds * 1000;
    now_ms - now_ms.rem_euclid(period_ms)
}

/// when ** 0 ** ms（ ）。
///
/// when and `request_stats::clock` / to in 1 （`chrono::Local`）： use 
/// 「 0 」 is when 0 。DST when `single()`
/// failure when for （ 1 when ， panic —— release none unwrap）。
fn local_midnight_ms(now_ms: i64) -> i64 {
    use chrono::{Local, TimeZone};
    let Some(local) = Local.timestamp_millis_opt(now_ms).single() else {
        return window_start(now_ms, 3600);
    };
    let midnight = local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|naive| Local.from_local_datetime(&naive).single());
    match midnight {
        Some(midnight) => midnight.timestamp_millis(),
        // DST 0 not in （）： to this when in when 
        None => window_start(now_ms, 3600),
    }
}

/// then → / ****： = seconds（ 1800，
/// not will 0）， = 0。 then 1 and 1 。
fn rule_kind(rule: &LimiterRule) -> i64 {
    match rule.reset {
        LimiterReset::Daily => 0,
        LimiterReset::Fixed => rule.period,
    }
}

/// → current（ms）：0 = 0 ， its by for 。
fn kind_window_start(kind: i64, now_ms: i64) -> i64 {
    if kind == 0 {
        local_midnight_ms(now_ms)
    } else {
        window_start(now_ms, kind)
    }
}

/// Token ： 1 enabled Token skip then in **current** to 。
/// （ no has / already ） by 0 —— is for 「 this this 
/// already use 」 ， not not （ and 1 ）。
pub fn token_skip_blocked(
    account: &Value,
    facts: &HashMap<(String, i64), TokenFact>,
    now_ms: i64,
) -> bool {
    let Some(id) = account.get("id").and_then(Value::as_str) else { return false };
    effective_rules(account).iter().any(|rule| {
        if rule.kind != LimiterKind::Token || rule.action != LimiterAction::Skip || !rule.enabled {
            return false;
        }
        let start = kind_window_start(rule_kind(&rule), now_ms);
        let used = facts
            .get(&(id.to_string(), rule_kind(&rule)))
            .filter(|fact| fact.window_start == start)
            .map(|fact| fact.used)
            .unwrap_or(0);
        used as f64 >= rule.threshold
    })
}

/// Token use （ use use ）：return in 「current ≥ 」
/// then and （ need 「 use / limit」）。 1 
/// in when return that —— 「 that 」 use 。
pub fn token_disable_hit(
    account: &Value,
    facts: &HashMap<(String, i64), TokenFact>,
    now_ms: i64,
) -> Option<(LimiterRule, i64)> {
    let id = account.get("id").and_then(Value::as_str)?;
    let hits: Vec<(LimiterRule, i64)> = effective_rules(account)
        .into_iter()
        .filter(|rule| rule.kind == LimiterKind::Token && rule.action == LimiterAction::Disable && rule.enabled)
        .filter_map(|rule| {
            let start = kind_window_start(rule_kind(&rule), now_ms);
            let used = facts
                .get(&(id.to_string(), rule_kind(&rule)))
                .filter(|fact| fact.window_start == start)
                .map(|fact| fact.used)
                .unwrap_or(0);
            (used as f64 >= rule.threshold).then_some((rule, used))
        })
        .collect();
    hits.into_iter()
        .min_by(|(a, _), (b, _)| a.threshold.partial_cmp(&b.threshold).unwrap_or(std::cmp::Ordering::Equal))
}

/// seconds → can / （` 90 minutes` this ）： when / minutes，
/// its by seconds。（store_crud） and use （usage_query） use this 1 ，
/// and before `formatIntervalSeconds` 1 。
pub fn describe_period(seconds: i64) -> String {
    if seconds > 0 && seconds % 3600 == 0 {
        format!("{} 小时", seconds / 3600)
    } else if seconds > 0 && seconds % 60 == 0 {
        format!("{} 分钟", seconds / 60)
    } else {
        format!("{seconds} 秒")
    }
}

/// 1 then → can 1 （ use ）。f64 Display not 
/// （`100.0` → `100`）， and 1 。
pub fn describe_rule(rule: &LimiterRule) -> String {
    let action = match rule.action {
        LimiterAction::Skip => "跳过",
        LimiterAction::Disable => "禁用",
    };
    match rule.kind {
        LimiterKind::Balance => format!("余额 < {} 时{action}", rule.threshold),
        // not ：「daily」 is 
        LimiterKind::Token if rule.reset == LimiterReset::Daily => {
            format!("每日消耗 ≥ {} Token 时{action}", rule.threshold)
        }
        LimiterKind::Token => format!(
            "每{}消耗 ≥ {} Token 时{action}",
            describe_period(rule.period),
            rule.threshold
        ),
    }
}

/// then → 1 。
pub fn describe_rules(rules: &[LimiterRule]) -> String {
    if rules.is_empty() {
        return "限制器 → 无规则（不限制）".to_string();
    }
    let (enabled, disabled): (Vec<&LimiterRule>, Vec<&LimiterRule>) =
        rules.iter().partition(|rule| rule.enabled);
    let mut parts: Vec<String> = enabled.iter().map(|rule| describe_rule(rule)).collect();
    if !disabled.is_empty() {
        parts.push(format!("{} 条已停用", disabled.len()));
    }
    format!("限制器 → {}", parts.join("；"))
}

// ─── Token ──────────────────────────────

/// 1 (, ) 。
#[derive(Clone, Copy, Debug)]
pub struct TokenFact {
    /// this （ms）。 and current for not up = old ，
    /// by 0 （ already 、 new no new to ）。
    pub window_start: i64,
    /// （`requests.total_tokens` and ）。
    pub used: i64,
}

fn facts_slot() -> &'static Mutex<HashMap<(String, i64), TokenFact>> {
    static FACTS: OnceLock<Mutex<HashMap<(String, i64), TokenFact>>> = OnceLock::new();
    FACTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Token （ / ； use 1 Map）。
pub fn token_facts() -> HashMap<(String, i64), TokenFact> {
    facts_slot()
        .lock()
        .map(|facts| facts.clone())
        .unwrap_or_default()
}

/// enabled in Token then ****（ new by —— 
/// / then 1 ； is 0）。
pub fn enabled_token_kinds(accounts: &[Value]) -> Vec<i64> {
    let mut kinds: Vec<i64> = accounts
        .iter()
        .flat_map(effective_rules)
        .filter(|rule| rule.kind == LimiterKind::Token && rule.enabled)
        .map(|rule| rule_kind(&rule))
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}

/// new ： to in **current** 1 ，。
///
/// Why not is and ： after old must（ by old 
/// will 「」「 already use 」），「」
/// 。 10 seconds 1 ， by distinct 1 SQL，
/// schema v9 `(account_id, ts)` —— down is ms。
pub fn refresh_token_facts(kinds: &[i64]) {
    let Some(db) = database() else { return };
    let now = logging::now_ms();
    let mut fresh: HashMap<(String, i64), TokenFact> = HashMap::new();
    for &kind in kinds {
        let start = kind_window_start(kind, now);
        // return Vec（unavailable / failure to empty —— up by 「 no 」，
        // and unavailable 1 ）； not 。
        let rows = db.with(|conn| {
            let Ok(mut statement) = conn.prepare(
                "SELECT account_id, COALESCE(SUM(total_tokens), 0) FROM requests \
                 WHERE account_id != '' AND ts >= ?1 GROUP BY account_id",
            ) else {
                return Vec::new();
            };
            let Ok(rows) = statement.query_map([start], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            }) else {
                return Vec::new();
            };
            rows.filter_map(|item| item.ok()).collect::<Vec<_>>()
        });
        for (account_id, used) in rows.unwrap_or_default() {
            fresh.insert((account_id, kind), TokenFact { window_start: start, used });
        }
    }
    if let Ok(mut slot) = facts_slot().lock() {
        *slot = fresh;
    }
}

/// ：request current。
///
/// ** already has ** (, ) update —— new then need etc down 1 new 
/// （≤ 10 seconds），requestpath not as 。 in request when 
/// （ / 0 ）， is new 1 
/// （ from ）。
pub fn note_request_tokens(account_id: &str, tokens: i64, now_ms: i64) {
    if account_id.is_empty() || tokens <= 0 {
        return;
    }
    let Ok(mut facts) = facts_slot().lock() else { return };
    let kinds: Vec<i64> = facts
        .keys()
        .filter(|(id, _)| id == account_id)
        .map(|(_, kind)| *kind)
        .collect();
    for kind in kinds {
        let start = kind_window_start(kind, now_ms);
        if let Some(fact) = facts.get_mut(&(account_id.to_string(), kind)) {
            if fact.window_start == start {
                fact.used += tokens;
            } else {
                fact.window_start = start;
                fact.used = tokens;
            }
        }
    }
}

/// Token → JSON （enabled in Token then 
/// 1 ，`kind` = seconds or 0（）； / old by 0 
/// `windowStart` is **current**， before when and 「 already use 」
/// is 1 ）。
pub fn token_usage_rows(account: &Value, facts: &HashMap<(String, i64), TokenFact>, now_ms: i64) -> Vec<Value> {
    let id = match account.get("id").and_then(Value::as_str) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => return Vec::new(),
    };
    let mut kinds: Vec<i64> = effective_rules(account)
        .into_iter()
        .filter(|rule| rule.kind == LimiterKind::Token && rule.enabled)
        .map(|rule| rule_kind(&rule))
        .collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
        .into_iter()
        .map(|kind| {
            let start = kind_window_start(kind, now_ms);
            let used = facts
                .get(&(id.clone(), kind))
                .filter(|fact| fact.window_start == start)
                .map(|fact| fact.used)
                .unwrap_or(0);
            json!({ "kind": kind, "windowStart": start, "used": used })
        })
        .collect()
}
