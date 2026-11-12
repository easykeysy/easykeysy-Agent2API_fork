//! ， and Clash Verge 。
//! by ；， not 。

use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;

use crate::server::core::proxies::ResolvedProxy;
// 「validenabled」（enabled ∧ use ） use ， not 1 
// —— `enabled` will 「 in 「」 in 」：
// use by down not ， and in failure。
use crate::server::core::proxy_pool::is_enabled;

/// when **config id**： and `core::proxies` 1 limit
/// （ 64 will 1 ， in this ， 1 that use ）。
pub const MAX_IDS: usize = 64;

/// limit（ when empty）。
///
/// is id， in —— this limit「 」 this 
/// （ 1 new id）。empty is ： is 1 ， from 。
const MAX_CURSORS: usize = 512;

/// id → down 1 from （，module header）
static CURSORS: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

fn lock_cursors() -> std::sync::MutexGuard<'static, Option<HashMap<String, u64>>> {
    match CURSORS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// ：return from ， and to down 1 。
///
/// `len` must 0（ use already ）； when to new 。
fn take_cursor(key: &str, len: usize) -> usize {
    let mut guard = lock_cursors();
    let cursors = guard.get_or_insert_with(HashMap::new);
    if cursors.len() >= MAX_CURSORS && !cursors.contains_key(key) {
        cursors.clear();
    }
    let entry = cursors.entry(key.to_string()).or_insert(0);
    let start = (*entry as usize) % len;
    *entry = entry.wrapping_add(1);
    start
}

// ─── config ───────────────────────────────────────────────

/// `all` ： / null ****。
///
/// and （ 1 ），「」 can can ； that when by 
/// 「」 by 「 1 not 」 —— before is this config default，
/// after will 。
fn is_all(config: &Value) -> bool {
    match config.get("all") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(flag)) => *flag,
        // （ / ） by 「 not is false as 」 JS ： false
        Some(_) => true,
    }
}

/// config id（`all: false` that 1 ；empty、、、 to [`MAX_IDS`]）
fn configured_ids(config: &Value) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let Some(items) = config.get("proxyIds").and_then(Value::as_array) else {
        return ids;
    };
    for item in items {
        let Some(text) = item.as_str() else { continue };
        let id = text.trim();
        if id.is_empty() || ids.iter().any(|existing| existing == id) {
            continue;
        }
        ids.push(id.to_string());
        if ids.len() >= MAX_IDS {
            break;
        }
    }
    ids
}

fn text_of(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

/// id（ by config，module header）。
///
/// `all: true` → **enabled**（ use not and ： in is 「 use 」
/// ， is request， no has 「 this 」 —— in will 
/// failure， 1 ）。 （ new in before ）。
pub fn candidate_ids(config: &Value) -> Vec<String> {
    if !is_all(config) {
        return configured_ids(config);
    }
    crate::server::core::proxy_pool::raw_items()
        .iter()
        .filter(|item| is_enabled(item))
        .filter_map(|item| {
            let id = text_of(item, "id");
            if id.is_empty() { None } else { Some(id) }
        })
        .collect()
}

/// ****available（ not ）：used for need 
/// 「available N 」。 use in that 1 （`proxy_pool::resolve_reference`）。
fn entry_ready(item: &Value) -> Result<(), String> {
    if item.get("source").and_then(Value::as_str) == Some("clash") {
        return match crate::server::core::proxy_pool::resolve_item(item) {
            Some(crate::server::core::proxies::ProxyResolution::Resolved(_)) => Ok(()),
            Some(crate::server::core::proxies::ProxyResolution::Failed(reason)) => Err(reason),
            None => Err("条目地址不完整".to_string()),
        };
    }
    let host = text_of(item, "host");
    let port = item.get("port").and_then(Value::as_u64).unwrap_or(0);
    if host.is_empty() || !(1..=65535).contains(&port) {
        return Err("条目地址不完整".to_string());
    }
    Ok(())
}

// ─── use ───────────────────────────────────────────────────

/// from `start` 1 1 can ； 1 not when to ** 1 
/// Reason**（ has ： in ， use 1 this to that ）+ 1 total。
///
/// `ids` by use （ need 1 ， 1 request this 1 ）。
fn pick_at(ids: &[String], start: usize, all: bool) -> Result<ResolvedProxy, String> {
    if ids.is_empty() {
        return Err(if all {
            "轮询没有可用出口：「网络代理」页还没有启用的代理".to_string()
        } else {
            "轮询没有可用出口：账号设置里选中的代理都不在了（可能已被删除）".to_string()
        });
    }
    let mut first_error: Option<String> = None;
    for offset in 0..ids.len() {
        let id = &ids[(start + offset) % ids.len()];
        match crate::server::core::proxy_pool::resolve_reference(id) {
            Ok(proxy) => return Ok(proxy),
            Err(reason) => {
                if first_error.is_none() {
                    first_error = Some(reason);
                }
            }
        }
    }
    Err(format!(
        "轮询的出口都不可用：{}",
        first_error.unwrap_or_else(|| "没有可用的代理".to_string())
    ))
}

/// down 1 （ by `key` before ；`key` = id）。
///
/// `key` as empty when as 「 1 available」： no has can not up ，
/// when （ 1 use to 1 ， and ）。
pub fn next(config: &Value, key: &str) -> Result<ResolvedProxy, String> {
    let ids = candidate_ids(config);
    // empty when not （`take_cursor` need len 0）—— by pick_at empty
    let start = if ids.is_empty() || key.is_empty() {
        0
    } else {
        take_cursor(key, ids.len())
    };
    pick_at(&ids, start, is_all(config))
}

/// not before ： to **** and **** use （、
/// this 「 1 use not 1 」 ）。
///
/// 1 available —— is 「 this config in can not can use 、」，
/// not is 「 this this 」。
pub fn first_usable(config: &Value) -> Result<ResolvedProxy, String> {
    let ids = candidate_ids(config);
    pick_at(&ids, 0, is_all(config))
}

// ─── need （ / use ， not ）───────────────

/// need ： and its in available （ [`entry_ready`]）
pub struct Summary {
    /// is 「」
    pub all: bool,
    /// （all=false when ** in ** ）
    pub total: usize,
    /// its in available （， [`entry_ready`]）
    pub usable: usize,
    /// 1 unavailable when Reason（available when as None）
    pub error: Option<String>,
}

/// 1 need （ 1 ， not request）。
pub fn summary(config: &Value) -> Summary {
    let all = is_all(config);
    let items = crate::server::core::proxy_pool::raw_items();
    let mut total = 0usize;
    let mut usable = 0usize;
    let mut consider = |item: &Value| {
        total += 1;
        if is_enabled(item) && entry_ready(item).is_ok() {
            usable += 1;
        }
    };
    if all {
        for item in items.iter().filter(|item| is_enabled(item)) {
            consider(item);
        }
    } else {
        for id in configured_ids(config) {
            if let Some(item) = items
                .iter()
                .find(|item| text_of(item, "id") == id)
            {
                consider(item);
            }
        }
    }
    let error = if total == 0 {
        Some(if all {
            "「网络代理」页还没有启用的代理".to_string()
        } else {
            "轮询里选中的代理都不在了（可能已被删除）".to_string()
        })
    } else if usable == 0 {
        Some("轮询里的代理都不可用（已禁用或未就绪）".to_string())
    } else {
        None
    };
    Summary { all, total, usable, error }
}
