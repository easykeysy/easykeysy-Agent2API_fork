//! **response**（： 1 up JSON → 1 1 ）。
//!
//! and `balance.rs` ： that 「、 、 to 」， this 
//! 「 to JSON 」。 this （ 1 in not /
//! not down ） in in 1 ， not in request。
//!
//! ：OmniProxy `server/src/utils/providerBalance.ts` 
//! `parseSub2api` / `parseNewApiAccount` / `parseNewApiDisplay` / `parseDeepSeek`
//! / `parseOpenCode` / `parseSensenova` / `parseZhipu`， for 。
//!
//! ── 1 （`ProviderAdapter::query_usage` ）────────
//! ```jsonc
//! {
//! "available": 62.5, // （； not to null）
//! "unit": "%", // 
//! "wallets": [ // （）
//! { "type": "rolling", "displayName": "5 when ", "balance": 62.5, "unit": "%" }
//!   ],
//! "windows": [ // （optional， has OpenCode）： 1 1 ，
//! { "key": "rolling", "label": "5 when ", // 1 
//!       "remainingPercent": 62.5, "usedPercent": 37.5,
//! "view": " 62.5%", "resetsAt": 1789… }
//!   ],
//! "subscription": null, // this no has 
//! "raw": { } // up response（ use ；default not ）
//! }
//! ```
//! 「 not 、null not is 0」 is （ `accounts-credit-detail`
//! ）： not to to `null` not is 0 —— 「—」 not is 
//! 「 0」。file `wallets` in ** to ** when 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap / expect / panic。

use chrono::TimeZone;
use serde_json::{json, Map, Value};

/// for （ / to None）
pub(super) fn map_of(value: &Value) -> Option<&Map<String, Value>> {
    value.as_object()
}

/// ： and 「」（ up ）
pub(super) fn number_of(value: Option<&Value>) -> Option<f64> {
    match value? {
        Value::Number(number) => number.as_f64().filter(|value| value.is_finite()),
        Value::String(text) => text
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite()),
        _ => None,
    }
}

/// for （`number_of(map.get(key))` ）
pub(super) fn number_at(map: &Map<String, Value>, key: &str) -> Option<f64> {
    number_of(map.get(key))
}

/// seconds Unix when （ms）→ ms； / to None
pub(super) fn unix_to_ms(value: Option<&Value>) -> Option<i64> {
    let number = number_of(value)?;
    if number <= 0.0 {
        return None;
    }
    Some(if number < 1e12 {
        (number * 1000.0).round() as i64
    } else {
        number.round() as i64
    })
}

/// （ up error can can is 1 HTML； to use ）
pub(super) fn shorten(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(max.saturating_sub(3)).collect();
    format!("{head}...")
}

/// 1 （`balance` as None when not —— module header 「 not 」）
pub(super) fn wallet(kind: &str, display_name: &str, balance: Option<f64>, unit: &str) -> Option<Value> {
    let balance = balance?;
    Some(json!({
        "type": kind,
        "displayName": display_name,
        "balance": balance,
        "unit": unit,
    }))
}

/// 1 （ to ）
pub(super) fn document(
    available: Option<f64>,
    unit: &str,
    wallets: Vec<Value>,
    raw: &Value,
) -> Value {
    json!({
        "available": available,
        "unit": unit,
        "wallets": wallets,
        "subscription": Value::Null,
        "raw": raw,
    })
}

/// Sub2API `/v1/usage`：`{ quota: {limit, used}, remaining|balance, unit }`。
///
/// can （ not sub2api to not 1 ）， need to 1 。
pub(super) fn parse_sub2api(payload: &Value) -> Option<Value> {
    let data = map_of(payload)?;
    let quota = data.get("quota").and_then(map_of);
    let total = quota.and_then(|map| number_at(map, "limit"));
    let used = quota.and_then(|map| number_at(map, "used"));
    let remaining = number_at(data, "remaining")
        .or_else(|| number_at(data, "balance"))
        .or_else(|| quota.and_then(|map| number_at(map, "remaining")))
        .or_else(|| match (total, used) {
            (Some(total), Some(used)) => Some(total - used),
            _ => None,
        });
    if remaining.is_none() && total.is_none() && used.is_none() {
        return None;
    }
    let unit = data
        .get("unit")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or("USD")
        .to_string();
    let wallets = vec![
        wallet("remaining", "剩余额度", remaining, &unit),
        wallet("used", "已用额度", used, &unit),
        wallet("total", "总额度", total, &unit),
    ]
    .into_iter()
    .flatten()
    .collect();
    Some(document(remaining, &unit, wallets, payload))
}

/// DeepSeek `/user/balance`：`{ balance_infos: [{currency, total_balance, …}] }`。
///
/// ** 1 **（： up by return）。
pub(super) fn parse_deepseek(payload: &Value) -> Option<Value> {
    let data = map_of(payload)?;
    let infos = data.get("balance_infos")?.as_array()?;
    let mut wallets: Vec<Value> = Vec::new();
    let mut primary: Option<(f64, String)> = None;
    for item in infos {
        let Some(info) = map_of(item) else { continue };
        let Some(amount) = number_at(info, "total_balance") else { continue };
        let currency = info
            .get("currency")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or("CNY")
            .to_string();
        if primary.is_none() {
            primary = Some((amount, currency.clone()));
        }
        if let Some(row) = wallet("total_balance", &currency, Some(amount), &currency) {
            wallets.push(row);
        }
        if let Some(row) = wallet("granted_balance", &format!("{currency} · 赠送余额"), number_at(info, "granted_balance"), &currency) {
            wallets.push(row);
        }
        if let Some(row) = wallet("topped_up_balance", &format!("{currency} · 充值余额"), number_at(info, "topped_up_balance"), &currency) {
            wallets.push(row);
        }
    }
    let (available, unit) = match primary {
        Some((amount, currency)) => (Some(amount), currency),
        None => (None, "CNY".to_string()),
    };
    if available.is_none() {
        return None;
    }
    Some(document(available, &unit, wallets, payload))
}

/// usage （ in OpenCode not ， ）
struct UsageWindow {
    remaining_percent: Option<f64>,
    used_percent: Option<f64>,
    resets_at_ms: Option<i64>,
}

fn usage_window(window: &Map<String, Value>) -> UsageWindow {
    let percent = number_at(window, "percent")
        .or_else(|| number_at(window, "usage_percent"))
        .or_else(|| number_at(window, "percent_used"));
    let used = number_at(window, "used")
        .or_else(|| number_at(window, "used_quota"))
        .or_else(|| number_at(window, "usage"))
        .or_else(|| number_at(window, "amount_used"));
    let limit = number_at(window, "limit")
        .or_else(|| number_at(window, "total"))
        .or_else(|| number_at(window, "quota"))
        .or_else(|| number_at(window, "allowance"))
        .or_else(|| number_at(window, "total_quota"));
    let remaining = number_at(window, "remaining")
        .or_else(|| number_at(window, "remaining_quota"))
        .or_else(|| number_at(window, "left"));
    // ； has used/limit（ or remaining/limit） when by 
    let used_percent = percent.or_else(|| match (used, limit) {
        (Some(used), Some(limit)) if limit > 0.0 => Some(used / limit * 100.0),
        _ => match (remaining, limit) {
            (Some(remaining), Some(limit)) if limit > 0.0 => {
                Some(((limit - remaining) / limit * 100.0).max(0.0))
            }
            _ => None,
        },
    });
    let resets_at_ms = [
        "resetsAt",
        "resets_at",
        "reset_at",
        "resetAt",
        "expires_at",
    ]
    .iter()
    .find_map(|key| window.get(*key))
    .and_then(|value| match value {
        // can can is RFC 3339， can can is （seconds / ms），
        Value::String(text) => {
            let trimmed = text.trim();
            match trimmed.parse::<f64>() {
                Ok(number) => Some(number),
                Err(_) => rfc3339_to_ms(trimmed).map(|ms| ms as f64 / 1000.0),
            }
        }
        other => number_of(Some(other)),
    })
    .and_then(|number| unix_to_ms(Some(&json!(number))));
    UsageWindow { remaining_percent: used_percent.map(|used| (100.0 - used).clamp(0.0, 100.0)), used_percent, resets_at_ms }
}

/// ISO 8601（RFC 3339）→ ms when ； not to None。
///
/// up to is （`2026-10-15T12:34:56.789Z` / `+08:00`），
/// to chrono —— 1 「 + when 」 will 1 。
fn rfc3339_to_ms(text: &str) -> Option<i64> {
    let parsed = chrono::DateTime::parse_from_rfc3339(text.trim()).ok()?;
    Some(parsed.timestamp_millis())
}

/// OpenCode `/v1/usage`：`{ usage: { rolling: {…}, weekly: {…}, monthly: {…} } }`。
///
/// is **（5 when ） remaining**（：`100 - already use %`）；
/// / （ already use 100%） when 0 —— that is 「 no 」，
/// need by （ OmniProxy `providerBalanceRule` `opencodeRemainingPercent`）。
///
/// ── （`windows[]`）───────────────────────────
/// this 1 not is 「 1 」 is ** remaining**： 1 
/// not 「5 when 、 already use 」 this ， is need 
/// 1 （ / no ）。 to 1 `windows[]`：
/// 1 1 ，、remaining、 already use 、 when and ，
/// ** 1 **。 this is can （ has `windows` ） not is provider ——
/// and `wallets` / `credits` 1 （ before `usageWindows`）。
///
/// use 1 ： is ** before **（「 62.5%」）， when in 
/// after （），「 · 」 and `zcode::monitor` `wallet_view` 1 。
pub(super) fn parse_opencode(payload: &Value) -> Option<Value> {
    let data = map_of(payload)?;
    let usage = data.get("usage").and_then(map_of)?;
    let mut reads: Vec<(&str, &str, &str, UsageWindow)> = Vec::new();
    for (key, label, code) in [
        ("rolling", "5 小时", "5h"),
        ("weekly", "每周", "weekly"),
        ("monthly", "每月", "monthly"),
    ] {
        let Some(window) = usage.get(key).and_then(map_of) else { continue };
        reads.push((key, label, code, usage_window(window)));
    }
    if reads.is_empty() {
        return None;
    }
    let exhausted = reads
        .iter()
        .any(|(key, _, _, window)| (*key == "weekly" || *key == "monthly") && window.used_percent.map(|used| used >= 100.0) == Some(true));
    let rolling = reads.iter().find(|(key, _, _, _)| *key == "rolling").map(|(_, _, _, window)| window);
    let primary = rolling.or_else(|| reads.first().map(|(_, _, _, window)| window));
    let available = if exhausted {
        Some(0.0)
    } else {
        primary.and_then(|window| window.remaining_percent)
    };
    let mut wallets: Vec<Value> = Vec::new();
    let mut windows: Vec<Value> = Vec::new();
    for (key, label, code, window) in &reads {
        // in 、 not to （ up ）： this 1 ， not ——
        // is 「—」， and 「 0%」 is （module header 「null not is 0」）
        let mut row = Map::new();
        row.insert("key".to_string(), Value::String((*key).to_string()));
        row.insert("label".to_string(), Value::String((*label).to_string()));
        // `window` is ** to before **（ and `antigravity::balance` 1 
        // ）：`label` is in 、， up by before by ——
        // after to in ， will 1 
        row.insert("window".to_string(), Value::String((*code).to_string()));
        if let Some(remaining) = window.remaining_percent {
            row.insert("remainingPercent".to_string(), json!(remaining));
            row.insert("view".to_string(), Value::String(format!("剩 {}%", percent_text(remaining))));
        }
        if let Some(used) = window.used_percent {
            row.insert("usedPercent".to_string(), json!(used));
        }
        if let Some(resets) = window.resets_at_ms {
            row.insert("resetsAt".to_string(), Value::from(resets));
        }
        windows.push(Value::Object(row));
        // in ** to ** when （「 not 」）， up when ：
        // this 1 is 「 to 」，remaining and when is 1 
        let Some(remaining) = window.remaining_percent else { continue };
        let Some(mut row) = wallet(key, label, Some(remaining), "%") else { continue };
        let mut parts = vec![format!("剩 {}%", percent_text(remaining))];
        if let Some(used) = window.used_percent {
            parts.push(format!("已用 {}%", percent_text(used)));
        }
        if let Some(text) = reset_text(window.resets_at_ms) {
            parts.push(format!("{text} 重置"));
        }
        if let Some(object) = row.as_object_mut() {
            object.insert("balanceView".to_string(), Value::String(parts.join(" · ")));
            if let Some(resets) = window.resets_at_ms {
                object.insert("resetsAt".to_string(), Value::from(resets));
            }
        }
        wallets.push(row);
    }
    let mut document = document(available, "%", wallets, payload);
    if let Some(object) = document.as_object_mut() {
        object.insert("windows".to_string(), Value::Array(windows));
    }
    Some(document)
}

/// （1 、 `.0`； and `zcode::monitor::percent_text`
/// 1 —— 1 ， as 1 for has crate can ）
fn percent_text(value: f64) -> String {
    format!("{value:.1}").trim_end_matches(".0").to_string()
}

/// when → when `MM-DD HH:MM`（ and `zcode::monitor::reset_text` ：
/// up to is for when ， by when ； when to None ——
/// can not this 1 ， not 1 when ）
fn reset_text(ms: Option<i64>) -> Option<String> {
    let ms = ms.filter(|value| *value > 0)?;
    chrono::Local
        .timestamp_millis_opt(ms)
        .single()
        .map(|at| at.format("%m-%d %H:%M").to_string())
}

/// NewAPI `/api/status` quota（`data.quota_per_unit` etc ）。
pub(super) struct QuotaDisplay {
    pub display_type: String,
    pub quota_per_unit: f64,
    pub usd_exchange_rate: f64,
    pub custom_currency_symbol: String,
    pub custom_currency_exchange_rate: f64,
}

/// `/api/status` （ not / when to None —— and 
/// ：** can quota， not use default 1 **）。
pub(super) fn new_api_display(payload: &Value) -> Option<QuotaDisplay> {
    let data = payload.get("data").and_then(map_of)?;
    let quota_per_unit = number_at(data, "quota_per_unit").filter(|value| *value > 0.0)?;
    let mut display_type = data
        .get("quota_display_type")
        .and_then(Value::as_str)
        .map(|text| text.trim().to_ascii_uppercase())
        .unwrap_or_default();
    if !["USD", "CNY", "TOKENS", "CUSTOM"].contains(&display_type.as_str()) {
        display_type = if data.get("display_in_currency").and_then(Value::as_bool) == Some(false) {
            "TOKENS".to_string()
        } else {
            "USD".to_string()
        };
    }
    let usd_exchange_rate = number_at(data, "usd_exchange_rate").unwrap_or(1.0);
    let custom_currency_exchange_rate = number_at(data, "custom_currency_exchange_rate").unwrap_or(1.0);
    if display_type == "CNY" && !(usd_exchange_rate > 0.0) {
        return None;
    }
    if display_type == "CUSTOM" && !(custom_currency_exchange_rate > 0.0) {
        return None;
    }
    Some(QuotaDisplay {
        display_type,
        quota_per_unit,
        usd_exchange_rate,
        custom_currency_symbol: data
            .get("custom_currency_symbol")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string(),
        custom_currency_exchange_rate,
    })
}

/// quota → （ `displayedAmountRemaining` 1 ）
pub(super) fn new_api_display_amount(quota: f64, display: Option<&QuotaDisplay>) -> (f64, String) {
    let Some(display) = display else { return (quota, "quota".to_string()) };
    if display.quota_per_unit <= 0.0 {
        return (quota, "quota".to_string());
    }
    let amount_usd = quota / display.quota_per_unit;
    match display.display_type.as_str() {
        "CNY" if display.usd_exchange_rate > 0.0 => (amount_usd * display.usd_exchange_rate, "CNY".to_string()),
        "CUSTOM" if display.custom_currency_exchange_rate > 0.0 => (
            amount_usd * display.custom_currency_exchange_rate,
            if display.custom_currency_symbol.is_empty() {
                "CUSTOM".to_string()
            } else {
                display.custom_currency_symbol.clone()
            },
        ),
        "TOKENS" => (quota, "quota".to_string()),
        // USD and not （ after already new_api_display 1 USD/TOKENS）
        _ => (amount_usd, "USD".to_string()),
    }
}

/// NewAPI `/api/user/self`：`data.quota`（remainingquota）/ `data.used_quota`（ already use ）。
pub(super) fn parse_new_api_account(payload: &Value) -> Option<(f64, Option<f64>)> {
    let data = payload.get("data").and_then(map_of)?;
    let remaining = number_at(data, "quota")?;
    Some((remaining, number_at(data, "used_quota")))
}

/// NewAPI ： by （`/api/status` no to use quota）。
pub(super) fn new_api_document(
    remaining_raw: f64,
    used_raw: Option<f64>,
    display: Option<&QuotaDisplay>,
    account_payload: &Value,
    status_payload: Option<&Value>,
) -> Value {
    let (remaining, unit) = new_api_display_amount(remaining_raw, display);
    let used = used_raw.map(|used| new_api_display_amount(used, display).0);
    let wallets = vec![
        wallet("remaining", "剩余配额", Some(remaining), &unit),
        wallet("used", "已用配额", used, &unit),
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut document = document(Some(remaining), &unit, wallets, account_payload);
    if let Some(object) = document.as_object_mut() {
        // to 、quota is ： when this has use 
        object.insert("quota".to_string(), json!(remaining_raw));
        if let Some(used_raw) = used_raw {
            object.insert("usedQuota".to_string(), json!(used_raw));
        }
        if let Some(status) = status_payload {
            object.insert("status".to_string(), status.clone());
        }
    }
    document
}

/// `query-customer-account-report`（ 1 、 1 ）：
/// `{code, msg, data: {balance, rechargeAmount, giveAmount, totalSpendAmount, …}}`。
///
/// （ / ）， by use 。
/// `code != 200` is failure：failure 1 （401 ；1000 not ；
/// 1001 no Authorization —— after is ， 1 ）「」
/// 1 （ need can use ， not ：、
/// API Key ）， its up up 。
pub(super) fn parse_zhipu(
    payload: &Value,
    unit: &str,
    invalid_credential: &str,
) -> Result<Value, String> {
    let body = map_of(payload).ok_or_else(|| "智谱余额响应不是 JSON 对象".to_string())?;
    let code = number_at(body, "code");
    if code != Some(200.0) {
        let message = body
            .get("msg")
            .or_else(|| body.get("message"))
            .and_then(Value::as_str)
            .map(|text| shorten(text, 120))
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| "未知错误".to_string());
        return Err(matches!(code, Some(401.0) | Some(1000.0) | Some(1001.0))
            .then(|| format!("{invalid_credential}：{message}"))
            .unwrap_or_else(|| {
                format!(
                    "智谱余额接口返回错误（code={}）：{message}",
                    code.map(|value| value.to_string()).unwrap_or_else(|| "?".to_string())
                )
            }));
    }
    let data = body.get("data").and_then(map_of);
    let balance = data.and_then(|map| number_at(map, "balance"))
        .or_else(|| data.and_then(|map| number_at(map, "availableBalance")));
    let Some(balance) = balance else {
        return Err("智谱余额响应里没有余额字段".to_string());
    };
    let wallets = data
        .map(|map| {
            vec![
                wallet("rechargeAmount", "充值余额", number_at(map, "rechargeAmount"), unit),
                wallet("giveAmount", "赠送余额", number_at(map, "giveAmount"), unit),
                wallet("frozenBalance", "冻结金额", number_at(map, "frozenBalance"), unit),
            ]
            .into_iter()
            .flatten()
            .collect()
        })
        .unwrap_or_default();
    let mut document = document(Some(balance), unit, wallets, payload);
    if let Some(object) = document.as_object_mut() {
        if let Some(spent) = data.and_then(|map| number_at(map, "totalSpendAmount")) {
            object.insert("totalSpendAmount".to_string(), json!(spent));
        }
    }
    Ok(document)
}

/// new `tokenplan/pool-usage`：
/// `{ plan, pools: [{ id, name, pool_type, model_ids, window_5h, window_7d, grant_balance }] }`。
///
/// is ** use （`pool_type = default`）5 when remaining**（ when 
/// 7 ）—— and OmniProxy 1 。
/// by × （ ）， 1 。
pub(super) fn parse_sensenova(payload: &Value) -> Option<Value> {
    let data = map_of(payload)?;
    let pools_raw = data.get("pools")?.as_array()?;
    struct Pool {
        name: String,
        pool_type: String,
        window_5h: Option<Value>,
        window_7d: Option<Value>,
        grant_balance: Option<f64>,
    }
    let mut pools: Vec<Pool> = Vec::new();
    for item in pools_raw {
        let Some(map) = map_of(item) else { continue };
        let window_5h = read_sensenova_window(map.get("window_5h"));
        let window_7d = read_sensenova_window(map.get("window_7d"));
        if window_5h.is_none() && window_7d.is_none() {
            continue;
        }
        pools.push(Pool {
            name: map
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or("积分池")
                .to_string(),
            pool_type: map
                .get("pool_type")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            window_5h,
            window_7d,
            grant_balance: number_at(map, "grant_balance"),
        });
    }
    if pools.is_empty() {
        return None;
    }
    let general = pools
        .iter()
        .find(|pool| pool.pool_type == "default")
        .or_else(|| pools.first())?;
    let available = window_remaining_percent(general.window_5h.as_ref())
        .or_else(|| window_remaining_percent(general.window_7d.as_ref()));
    let mut wallets: Vec<Value> = Vec::new();
    for pool in &pools {
        for (key, label, window) in [
            ("5h", "5 小时", pool.window_5h.as_ref()),
            ("7d", "7 天", pool.window_7d.as_ref()),
        ] {
            let Some(window) = window else { continue };
            if let Some(row) = wallet(
                key,
                &format!("{} · {}", pool.name, label),
                window_remaining_percent(Some(window)),
                "%",
            ) {
                wallets.push(row);
            }
        }
        if let Some(row) = wallet("grant", &format!("{} · 活动返赠", pool.name), pool.grant_balance, "积分") {
            wallets.push(row);
        }
    }
    Some(document(available, "%", wallets, payload))
}

/// → （`{limit, used, remaining, reset_at}`）
fn read_sensenova_window(value: Option<&Value>) -> Option<Value> {
    let map = value.and_then(map_of)?;
    let limit = number_at(map, "limit");
    let used = number_at(map, "used");
    let remaining = number_at(map, "remaining");
    if limit.is_none() && used.is_none() && remaining.is_none() {
        return None;
    }
    let mut out = Map::new();
    if let Some(limit) = limit {
        out.insert("limit".to_string(), json!(limit));
    }
    if let Some(used) = used {
        out.insert("used".to_string(), json!(used));
    }
    if let Some(remaining) = remaining {
        out.insert("remaining".to_string(), json!(remaining));
    }
    if let Some(resets) = unix_to_ms(map.get("reset_at")) {
        out.insert("resetsAt".to_string(), Value::from(resets));
    }
    Some(Value::Object(out))
}

/// remaining（`remaining / limit`， in 0~100； to None）
fn window_remaining_percent(window: Option<&Value>) -> Option<f64> {
    let window = window.and_then(map_of)?;
    let limit = number_at(window, "limit").filter(|value| *value > 0.0)?;
    let remaining = number_at(window, "remaining")?;
    Some((remaining / limit * 100.0).clamp(0.0, 100.0))
}
