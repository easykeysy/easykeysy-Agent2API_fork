//! directory ** use new **： when 、client `/v1/models` new 、
//! 「get」 new from this 。
//!
//! ── Why need need （）──────────────────────────────
//! before this has 「 up 」 ， not ：
//! 1. when 1 （）；
//! 2. client**** `/v1/models` 1 after new （ has 
//! TTL ，）；
//! 3. 「directory new 」 when by 1 。
//! after is 「 1 in minutes 」，、 when its 。
//! in use `core::task_state` （ `modelRefresh:<scope>`）：
//! 、 in 、failure， not 。
//!
//! ── path： before 、failure， not and ──────────────
//! `manual = true` ****（ use by down by is 「 in 1 」，
//! TTL will 「 no 」 and none）， and **failure**
//! （`ManualBackoff::Bypass`）： this 1 is 「 up 1 Why no success」，
//! use by by is as that Reason（ new 、）—— 
//! old ， up will up 1 error， is by 。
//! in and request： is and and ， and not is 1 。
//!
//! ── Cline 1 ───────────────────────────────────
//! `cline-free` / `cline-pass` is provider、， is ** 1 **
//! （`/ai/cline/recommended-models` 1 return， `cline::models` module header）。
//! use 1 （`modelRefresh:cline`）， and 1 will 
//! use —— otherwise 1 new will for 1 。
//!
//! ── ： and use must in 1 ────────────────────
//! `task_state::claim` 「 to + + 」， 
//! （ and use 1 ） or 1 use not will when request。

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::task_state::{self, Claim};
use crate::server::logging;

use super::adapter::{adapter_by_id, adapter_for, implemented_kinds};
use super::{catalog, executable_provider_ids, is_executable_provider, kind_id, meta, ProviderKind};

/// this 1 when to use that （ use path use ）。
///
/// ── Whypath not 「 in 」──────────────────────────────
/// `manual = true` is [`task_state::ManualBackoff::Bypass`]：** no has **
/// 。 when if by remaining when ， use to 「 in ，5 when after 」，
/// by its can 1 —— ， is 。
/// this 1 can can is in or 1 seconds ，。
///
/// ── path old by ───────────────────────────────────
/// when / new ， that when 「 need etc 」 is use 。
fn deferred_message(state: &task_state::TaskState, manual: bool) -> String {
    if state.retry_at > logging::now_ms() || state.running() {
        if manual { state.waiting_message_bypassing() } else { state.waiting_message() }
    } else {
        "沿用缓存，尚未到刷新时间".to_string()
    }
}

/// new 1 （module header）。return** 1 ** ，
/// `adapter::refresh_implemented_forced` （ before after ）。
///
/// `providers` is need （`None` = already ）： 
/// not not 。
pub async fn refresh(
    store: &AccountStore,
    accounts: &serde_json::Map<String, Value>,
    providers: Option<&[String]>,
    manual: bool,
) -> Vec<Value> {
    let mut results = Vec::new();
    let mut cline_result: Option<Value> = None;
    // old its ； and runtime ID in down 1 1 。
    for kind in implemented_kinds() {
        let provider = kind_id(kind);
        // already has then to down 1 ， 1 provider not can 。
        if is_executable_provider(provider) {
            continue;
        }
        if providers.is_some_and(|allowed| !allowed.iter().any(|id| id == provider)) {
            continue;
        }
        let adapter = adapter_for(kind);
        let requested = accounts.get(provider).and_then(Value::as_str).unwrap_or("").trim();
        let mut item = json!({
            "provider": provider,
            "providerLabel": meta(kind).label,
            "refreshedAt": catalog::refresh_meta(kind).1,
        });
        if adapter.refresh_uses_account() {
            let used = if requested.is_empty() {
                store.current_entry_for_provider(provider).map(|entry| entry.id)
            } else {
                Some(requested.to_string())
            };
            if let Some(id) = used { item["accountId"] = json!(id); }
        }
        if !adapter.supports_model_refresh() {
            item["status"] = json!("skipped");
            item["fixed"] = json!(true);
            item["message"] = json!("该提供商使用固定模型清单");
            results.push(item);
            continue;
        }
        let cline = matches!(kind, ProviderKind::ClineFree | ProviderKind::ClinePass);
        if cline {
            if let Some(previous) = &cline_result {
                // use 1 （ 1 return）： use 
                // 1 that 1 ， not up 。
                if let (Value::Object(target), Value::Object(source)) = (&mut item, previous) {
                    for field in ["status", "count", "message", "refreshedAt"] {
                        if let Some(value) = source.get(field) {
                            target.insert(field.to_string(), value.clone());
                        }
                    }
                }
                results.push(item);
                continue;
            }
        }
        let scope = if cline { "cline" } else { provider };
        let key = format!("modelRefresh:{scope}");
        let interval = config::scheduled_settings().model_refresh.interval * 60_000;
        // failure（ ManualBackoff）： use 「get」 is need in 1 。
        // path old （`backoff` in `manual = false` when not and ）。
        let backoff = if manual {
            task_state::ManualBackoff::Bypass
        } else {
            task_state::ManualBackoff::Respect
        };
        let guard = match task_state::claim(&key, interval, manual, backoff, 1_000) {
            Ok(Claim::Acquired(guard)) => guard,
            Ok(Claim::Deferred(state)) => {
                item["status"] = json!("skipped");
                item["message"] = json!(deferred_message(&state, manual));
                item["retryAt"] = json!(state.retry_at);
                results.push(item);
                continue;
            }
            Err(error) => {
                item["status"] = json!("failed");
                item["message"] = json!(error);
                results.push(item);
                continue;
            }
        };
        let outcome = adapter.refresh_models(store, requested, manual).await;
        if outcome.refreshed {
            item["status"] = json!("refreshed");
            item["count"] = json!(outcome.count);
        } else if let Some(message) = outcome.message {
            item["status"] = json!("failed");
            item["message"] = json!(message);
        } else {
            item["status"] = json!("skipped");
            item["message"] = json!("沿用缓存或当前没有可用的目录来源");
        }
        item["refreshedAt"] = json!(catalog::refresh_meta(kind).1);
        let success = item["status"] != "failed";
        let summary = item.get("message").and_then(Value::as_str).unwrap_or("模型清单已刷新").to_string();
        // in this ** 1 **（ not is use before that ）： this 1 can can seconds 
        // request， use can can ； use old will 「 already 」
        // by old （ and `scheduled_tasks::finish` 1 ）。
        let interval = config::scheduled_settings().model_refresh.interval * 60_000;
        if let Err(error) = guard.finish(success, summary, None, 0, interval) {
            item["status"] = json!("failed");
            item["message"] = json!(error);
        }
        if cline { cline_result = Some(item.clone()); }
        results.push(item);
    }
    // and runtime ID and ， not to old 。
    for id in executable_provider_ids() {
        if providers.is_some_and(|allowed| !allowed.iter().any(|allowed_id| allowed_id == &id)) {
            continue;
        }
        let Some(adapter) = adapter_by_id(&id) else {
            continue;
        };
        let label = adapter.label();
        let requested = accounts.get(&id).and_then(Value::as_str).unwrap_or("").trim();
        let mut item = json!({
            "provider": id,
            "providerLabel": label,
            "refreshedAt": catalog::refresh_meta_for_provider(&id).1,
        });
        if !adapter.supports_model_refresh() {
            item["status"] = json!("skipped");
            item["fixed"] = json!(true);
            item["message"] = json!("该提供商使用固定模型清单");
            results.push(item);
            continue;
        }
        // need need when from this provider enabled；error not 。
        // not secret， not empty to builtin / 。
        let account_id = if adapter.refresh_uses_account() {
            let available = store.accounts_for_provider(&id);
            let selected = available.iter().filter_map(|account| account.get("id").and_then(Value::as_str))
                .find(|account_id| requested.is_empty() || *account_id == requested);
            let Some(selected) = selected else {
                item["status"] = json!("skipped");
                item["message"] = json!(if requested.is_empty() {
                    "当前没有可用的目录账号"
                } else {
                    "指定账号不属于该提供商、未启用或没有可用凭证"
                });
                results.push(item);
                continue;
            };
            item["accountId"] = json!(selected);
            selected.to_string()
        } else {
            requested.to_string()
        };
        let plugin = crate::server::core::plugin_host::serves_provider(&id);
        // ； use new 。
        let guard = if plugin {
            None
        } else {
            let interval = config::scheduled_settings().model_refresh.interval * 60_000;
            let backoff = if manual { task_state::ManualBackoff::Bypass } else { task_state::ManualBackoff::Respect };
            match task_state::claim(&format!("modelRefresh:{id}"), interval, manual, backoff, 1_000) {
                Ok(Claim::Acquired(guard)) => Some(guard),
                Ok(Claim::Deferred(state)) => {
                    item["status"] = json!("skipped");
                    item["message"] = json!(deferred_message(&state, manual));
                    item["retryAt"] = json!(state.retry_at);
                    results.push(item);
                    continue;
                }
                Err(error) => {
                    item["status"] = json!("failed");
                    item["message"] = json!(error);
                    results.push(item);
                    continue;
                }
            }
        };
        let outcome = adapter.refresh_models(store, &account_id, manual).await;
        item["status"] = json!(if outcome.refreshed {
            "refreshed"
        } else if outcome.message.is_some() {
            "failed"
        } else {
            "skipped"
        });
        item["count"] = json!(outcome.count);
        item["refreshedAt"] = json!(catalog::refresh_meta_for_provider(&id).1);
        if let Some(message) = outcome.message {
            item["message"] = json!(message);
        }
        if let Some(guard) = guard {
            let success = item["status"] != "failed";
            let summary = item.get("message").and_then(Value::as_str).unwrap_or("模型清单已刷新").to_string();
            let interval = config::scheduled_settings().model_refresh.interval * 60_000;
            if let Err(error) = guard.finish(success, summary, None, 0, interval) {
                item["status"] = json!("failed");
                item["message"] = json!(error);
            }
        }
        results.push(item);
    }
    results
}

/// by new new （ use 「directory new 」 when use ）。
///
/// need ： in `modelRefresh:<scope>` up ， that 
/// 「 when when 1 」—— will in old up 。
pub fn reschedule(interval_ms: i64) -> Result<(), String> {
    for kind in implemented_kinds() {
        if !adapter_for(kind).supports_model_refresh() { continue; }
        let scope = if matches!(kind, ProviderKind::ClineFree | ProviderKind::ClinePass) {
            "cline"
        } else { kind_id(kind) };
        task_state::reschedule(&format!("modelRefresh:{scope}"), interval_ms)?;
    }
    for id in executable_provider_ids() {
        // has ； this use 。
        if crate::server::core::plugin_host::serves_provider(&id) { continue; }
        if !adapter_by_id(&id).is_some_and(|adapter| adapter.supports_model_refresh()) { continue; }
        task_state::reschedule(&format!("modelRefresh:{id}"), interval_ms)?;
    }
    // models.dev 1 、 1 （ `scheduled_tasks` 
    // TASK_MODEL_REFRESH ）： and 1 empty， when if not 
    // 1 and ， will by old to down 1 to new as 。
    task_state::reschedule(
        &format!("modelRefresh:{}", crate::server::core::model_meta::SCHEDULE_SCOPE),
        interval_ms,
    )?;
    Ok(())
}
