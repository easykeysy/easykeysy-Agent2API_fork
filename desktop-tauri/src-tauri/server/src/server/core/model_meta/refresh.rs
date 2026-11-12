//! models.dev new ： JSON → 1 → and 。
//!
//! ── 1 in ──────────────────────────
//! file by `scripts/sync-modelsdev.mjs` （ is ），
//! is that ****： 1 「 / 1 /
//! 1 」。must 1 —— is 「 new 1 after 
//! use 」， not 、 not ， is 。 this need that ，。
//!
//! ── Why need 1 （ already has ）─────────────────
//! is that 1 。models.dev is ， new and in 
//! ， up is when new —— use will 「 up already up 
//! new 、 no has 」。 and directory 1 ： when new + can 
//! before ，success，failure old （ not old ）。
//!
//! ── that （ not addconfig）───────────────────────
//! `update::client::fetch_public` —— and 「update」 1 （set update
//! > Clash > ）。 is 「 this 」，
//! and none； as models.dev 1 set will 1 no 、
//! when must for 。
//!
//! ── and failure ──────────────────────────────────────────
//! and directory new use `task_state` that ： `modelRefresh:modelsdev`、
//! 「directory new 」config 、failure by `task_state` 、
//! （`ManualBackoff::Bypass`： use by by is ）。

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::task_state::{self, Claim, ManualBackoff};
use crate::server::core::update::client;
use crate::server::logging;

use super::snapshot::{self, Record, Snapshot};

const API_URL: &str = "https://models.dev/api.json";
const MODELS_URL: &str = "https://models.dev/models.json";

/// requesttotal when （api.json has MB， to 1 minutes； when failure、 not ）。
const REQUEST_TIMEOUT_MS: u64 = 60_000;

/// new （ / and ； not ）。
const MIN_GAP_MS: i64 = 5_000;

/// （`task_state`）： and `modelRefresh:<provider>` 1 empty，
/// 「 new 」 when can 1 and （ `providers::catalog_refresh::reschedule`）。
pub(crate) const SCHEDULE_SCOPE: &str = "modelsdev";

/// new 1 and ** 1 **（ and directory new ，
/// `providers::adapter::refresh_implemented_forced` module header）。
///
/// `interval_ms` is after need （ use from 「directory new 」 config，
/// and directory new use 1 ）。
pub async fn refresh(manual: bool, interval_ms: i64) -> Value {
    let mut row = json!({
        // provider： not is 1 （ no has 、 no has up directory）， use 
        // and directory new 1 ， can use 1 （ `api::model_meta`）。
        "provider": SCHEDULE_SCOPE,
        "providerLabel": "models.dev 模型库",
        "refreshedAt": snapshot::origin().1,
    });
    if !config::models_dev_enabled() {
        row["status"] = json!("skipped");
        row["message"] = json!("已在设置里关闭 models.dev 模型元数据");
        return row;
    }
    let backoff = if manual { ManualBackoff::Bypass } else { ManualBackoff::Respect };
    let guard = match task_state::claim(
        &format!("modelRefresh:{SCHEDULE_SCOPE}"),
        interval_ms,
        manual,
        backoff,
        MIN_GAP_MS,
    ) {
        Ok(Claim::Acquired(guard)) => guard,
        Ok(Claim::Deferred(state)) => {
            row["status"] = json!("skipped");
            // （`ManualBackoff::Bypass`）， this when 「 in 」
            // will and 「 by its can 」 —— and directory new 1 ，
            // `providers::catalog_refresh::deferred_message` 。
            row["message"] = json!(if state.retry_at > logging::now_ms() || state.running() {
                if manual { state.waiting_message_bypassing() } else { state.waiting_message() }
            } else {
                "沿用缓存，尚未到刷新时间".to_string()
            });
            row["retryAt"] = json!(state.retry_at);
            return row;
        }
        Err(error) => {
            row["status"] = json!("failed");
            row["message"] = json!(error);
            return row;
        }
    };
    let outcome = fetch_and_reduce().await;
    match outcome {
        Ok((snapshot, count)) => {
            snapshot::write_cache(&snapshot::serialize(&snapshot));
            snapshot::replace(snapshot);
            let summary = format!("models.dev 元数据已更新（{count} 条）");
            logging::log("[Models]", &summary);
            row["status"] = json!("refreshed");
            row["count"] = json!(count);
            row["refreshedAt"] = json!(logging::now_ms());
            let _ = guard.finish(true, summary, None, 0, interval_ms);
        }
        Err(message) => {
            row["status"] = json!("failed");
            row["message"] = json!(message);
            let _ = guard.finish(false, message, None, 0, interval_ms);
        }
    }
    row
}

/// JSON and 。failure when return** to use Reason**（ not ： this 
/// 1 failure already is 1 ， 1 error no has ）。
async fn fetch_and_reduce() -> Result<(Snapshot, usize), String> {
    let (api, models) = tokio::try_join!(fetch_json(API_URL), fetch_json(MODELS_URL))?;
    let built = reduce(&api, &models).ok_or_else(|| "models.dev 返回的模型条目为空".to_string())?;
    let count = built.len();
    Ok((built, count))
}

async fn fetch_json(url: &str) -> Result<Value, String> {
    let response = client::fetch_public(
        url,
        &[("accept".to_string(), "application/json".to_string())],
        Some(REQUEST_TIMEOUT_MS),
    )
    .await?;
    if !response.status().is_success() {
        return Err(format!("{url} 返回 HTTP {}", response.status().as_u16()));
    }
    response.json::<Value>().await.map_err(|error| format!("{url} 响应不是合法 JSON：{error}"))
}

/// JSON → （ and `scripts/sync-modelsdev.mjs` ）。
fn reduce(api: &Value, models: &Value) -> Option<Snapshot> {
    let mut index: HashMap<String, Record> = HashMap::new();
    // ① canonical can and up down 
    for (canonical, entry) in models.as_object()? {
        if !is_chat_model(entry) {
            continue;
        }
        let key = snapshot::key_of(canonical);
        if key.is_empty() {
            continue;
        }
        let input = entry
            .pointer("/modalities/input")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let has = |name: &str| input.iter().any(|item| item.as_str() == Some(name));
        index.insert(
            key,
            Record {
                canonical: canonical.clone(),
                reasoning: entry.get("reasoning").and_then(Value::as_bool),
                tool_call: entry.get("tool_call").and_then(Value::as_bool),
                images: Some(has("image")),
                video: Some(has("video")),
                context: limit_of(entry.pointer("/limit/context")),
                output: limit_of(entry.pointer("/limit/output")),
                levels: Vec::new(),
            },
        );
    }
    // ② provider （lab ， its provider id ）
    let mut picks: HashMap<String, (Vec<String>, bool)> = HashMap::new();
    let mut providers: Vec<(&String, &Value)> = api.as_object()?.iter().collect();
    providers.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (provider_id, provider) in providers {
        let Some(entries) = provider.get("models").and_then(Value::as_object) else { continue };
        for entry in entries.values() {
            let canonical = entry
                .get("canonical_model_id")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .or_else(|| entry.get("id").and_then(Value::as_str))
                .unwrap_or_default();
            let key = snapshot::key_of(canonical);
            if key.is_empty() {
                continue;
            }
            let Some(levels) = effort_levels(entry) else { continue };
            let is_lab = canonical.split('/').next() == Some(provider_id.as_str());
            match picks.get(&key) {
                // that ； not is lab when by provider 1 
                Some((_, true)) => continue,
                Some((_, false)) if !is_lab => continue,
                _ => picks.insert(key, (levels, is_lab)),
            };
        }
    }
    for (key, (levels, _)) in picks {
        if let Some(record) = index.get_mut(&key) {
            record.levels = levels;
        }
    }
    if index.is_empty() {
        return None;
    }
    Some(Snapshot::new(index, logging::now_ms(), "network"))
}

/// limit， its （0 / / / ）→ `None`（=「 this no 」）。
fn limit_of(value: Option<&Value>) -> Option<u64> {
    let number = value?.as_f64()?;
    if !number.is_finite() || number < 1.0 {
        return None;
    }
    Some(number.trunc() as u64)
}

/// ** can ** for （embedding / reranking / not need ）。
fn is_chat_model(entry: &Value) -> bool {
    if let Some(kind) = entry.get("type").and_then(Value::as_str) {
        if kind != "chat" {
            return false;
        }
    }
    match entry.pointer("/modalities/output").and_then(Value::as_array) {
        Some(output) => output.iter().any(|item| item.as_str() == Some("text")),
        None => true,
    }
}

/// from 1 provider ****； has `effort` that 1 
/// （`toggle` / empty = no to → `None`， use ）。
fn effort_levels(entry: &Value) -> Option<Vec<String>> {
    let options = entry.get("reasoning_options").and_then(Value::as_array)?;
    let effort = options
        .iter()
        .find(|option| option.get("type").and_then(Value::as_str) == Some("effort"))?;
    let levels: Vec<String> = effort
        .get("values")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|level| !level.is_empty())
        .map(str::to_string)
        .collect();
    (!levels.is_empty()).then_some(levels)
}
