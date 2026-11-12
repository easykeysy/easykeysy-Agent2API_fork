//! 「none」（Agent2API W2a-T2 from `models/mod.rs` ）。
//!
//! ── Why 1 file ──────────────────────────────────────
//! this is 「 1 → /v1/models 」「 1 id → 」
//! this ** and 1 provider none** 。 after has need ：
//!
//! - `models/mod.rs`：workbuddy （`ModelCatalog::list_response`）；
//! - `providers/catalog.rs`：（ data and 1 response）。
//!
//! in 1 （ not is in 1 ） is need ： or then need 
//! ， will ， in 「 1 / 1 」 this up 
//! not will ， will use 「 1 before after not 1 」。
//!
//! file Reason：`models/mod.rs` file 800 
//! （ this and 「directory / new 」 is ， this ）。

use serde_json::{json, Map, Value};

/// id （JS `String(m.id)`； to empty）
pub fn model_id(model: &Value) -> String {
    model.get("id").map(value_text).unwrap_or_default()
}

/// JS `String(x)`（id/name/credits this ）：
/// 、 by 、null/ to empty、 its use JSON 。
pub fn value_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        other => other.to_string(),
    }
}

/// JS （`Boolean(x)`）：null/false/0/"" as ， its as 
pub fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// → /v1/models （ and Node listResponse）。
///
/// `provider_id` used for `owned_by`：workbuddy `"workbuddy"` has ；
/// ** provider id**（ after that 1 ）。
/// `owned_by` is 1 add （ as "workbuddy"），
/// for 1 workbuddy use not 。
pub fn list_item(model: &Value, provider_id: &str) -> Value {
    // Node for ： as undefined（） when 
    // JSON will ** this **， this in in when ——
    // e.g. maxOutputTokens ，Node no has 
    // `max_output_tokens`， not is `null`
    let mut item = Map::new();
    if let Some(id) = model.get("id").filter(|value| !value.is_null()) {
        item.insert("id".to_string(), id.clone());
    }
    item.insert("object".to_string(), Value::String("model".to_string()));
    item.insert("created".to_string(), Value::from(0));
    item.insert(
        "owned_by".to_string(),
        Value::String(provider_id.to_string()),
    );
    if let Some(name) = model.get("name").filter(|value| !value.is_null()) {
        item.insert("name".to_string(), name.clone());
    }
    // `credits: m.credits || ''`：（ null/empty/0） 1 to empty，
    // （ etc ， and Node 1 ）
    item.insert(
        "credits".to_string(),
        match model.get("credits") {
            Some(value) if js_truthy(value) => value.clone(),
            _ => Value::String(String::new()),
        },
    );
    if let Some(value) = model.get("maxOutputTokens").filter(|value| !value.is_null()) {
        item.insert("max_output_tokens".to_string(), value.clone());
    }
    if let Some(value) = model.get("maxInputTokens").filter(|value| !value.is_null()) {
        item.insert("max_input_tokens".to_string(), value.clone());
    }
    // ： down `input_modalities` must and ****
    // —— 1 （ use ）、 1 ， not will 
    let images = model.get("supportsImages").map(js_truthy).unwrap_or(false)
        || crate::server::core::providers::images::provider_model_supports_editing(provider_id, model);
    let video = model.get("supportsVideo").map(js_truthy).unwrap_or(false);
    item.insert(
        "supports_tool_call".to_string(),
        Value::Bool(model.get("supportsToolCall").map(js_truthy).unwrap_or(false)),
    );
    item.insert("supports_images".to_string(), Value::Bool(images));
    item.insert("supports_video".to_string(), Value::Bool(video));
    item.insert(
        "supports_reasoning".to_string(),
        Value::Bool(model.get("supportsReasoning").map(js_truthy).unwrap_or(false)),
    );
    // `input_modalities`：OpenAI up ** use **
    // （ZenMux / OpenRouter / AIHubMix this in `/v1/models` to 
    // ；CLIProxyAPI as Codex client 1 ）。 and up 
    // `supports_images` / `supports_video` is ** 1 ** —— 1 
    // client before ， 1 after ， to ， and must。
    //
    // `text` in ； and by and kind 。
    // as when 
    // for that 1 。 use use ， not is has ：
    // up no has （audio / file） not ， can not 。
    let mut modalities = vec![Value::String("text".to_string())];
    if images {
        modalities.push(Value::String("image".to_string()));
    }
    if video {
        modalities.push(Value::String("video".to_string()));
    }
    item.insert("input_modalities".to_string(), Value::Array(modalities));
    let image_generation = crate::server::core::providers::images::model_is_image(model);
    item.insert("supports_image_generation".to_string(), Value::Bool(
        crate::server::core::providers::images::provider_model_supports_generation(provider_id, model),
    ));
    item.insert("supports_image_editing".to_string(), Value::Bool(
        crate::server::core::providers::images::provider_model_supports_editing(provider_id, model),
    ));
    if let Some(output) = model.get("output_modalities").filter(|value| value.is_array()) {
        item.insert("output_modalities".to_string(), output.clone());
    } else if image_generation {
        item.insert("output_modalities".to_string(), serde_json::json!(["image"]));
    }
    item.insert(
        "is_default".to_string(),
        Value::Bool(!image_generation && model.get("isDefault").map(js_truthy).unwrap_or(false)),
    );
    item.insert(
        "kind".to_string(),
        match model.get("kind") {
            _ if image_generation => Value::String("image".to_string()),
            Some(value) if js_truthy(value) => value.clone(),
            _ => Value::String("chat".to_string()),
        },
    );
    for (source, target) in [
        ("reasoningLevels", "reasoning_efforts"),
        ("reasoningDefaultLevel", "reasoning_default_effort"),
        ("reasoningFixedLevel", "reasoning_fixed_effort"),
        ("reasoningBindingSupported", "reasoning_binding_supported"),
        ("reasoningSource", "reasoning_source"),
    ] {
        if let Some(value) = model.get(source).filter(|value| !value.is_null()) {
            item.insert(target.to_string(), value.clone());
        }
    }
    if let Some(status) = model.get("upstreamStatus").filter(|value| value.is_string()) {
        item.insert("upstream_status".to_string(), status.clone());
    }
    Value::Object(item)
}

/// OpenAI response ：`{object:"list", data, meta:{source,lastRefreshedAt,count}}`。
///
/// `count` is data （path = directory，path = and after ）。
/// data：`json!` will Vec ， after borrow is error。
pub fn list_response_from(data: Vec<Value>, source: &str, last_refreshed_at: i64) -> Value {
    let count = data.len();
    json!({
        "object": "list",
        "data": data,
        "meta": {
            "source": source,
            "lastRefreshedAt": last_refreshed_at,
            "count": count,
        },
    })
}

/// id 1 ： + ASCII ，`deepseek-v4.1-flash` → `deepseekv41flash`
fn normalize_model_id(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

/// Levenshtein （ id ，O(nm) ）
fn edit_distance(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur: Vec<usize> = vec![i];
        for j in 1..=b.len() {
            let substitution = prev[j - 1] + if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur.push((prev[j] + 1).min(cur[j - 1] + 1).min(substitution));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// before （ already 1 ， by ）
fn common_prefix_length(a: &str, b: &str) -> usize {
    a.chars()
        .zip(b.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

/// 「 is not is need 」 ： from 1 id and `id` 。
///
/// and then Node suggest：
/// ① 1 after before ≥ 4；
/// ② and （ as before or ≤ max(3, floor( × 0.4))）；
/// ③ by 、 before ， before limit 。
///
/// （ not is `ModelCatalog::suggest` ） is to directory use ：
/// Agent2API after 400 need **all provider** ，
/// is if ， not this as use this 1 when ModelCatalog。
/// `ModelCatalog::suggest` and this ， 1 。
pub fn suggest_from(
    candidates: impl IntoIterator<Item = String>,
    id: &str,
    limit: usize,
) -> Vec<String> {
    let target = normalize_model_id(id);
    if target.is_empty() {
        return Vec::new();
    }
    let threshold = 3usize.max((target.chars().count() as f64 * 0.4).floor() as usize);
    let mut scored: Vec<(String, usize, usize)> = candidates
        .into_iter()
        .filter_map(|model_id| {
            let candidate = normalize_model_id(&model_id);
            let dist = edit_distance(&target, &candidate);
            let prefix = common_prefix_length(&target, &candidate);
            let starts_with = candidate.starts_with(&target) || target.starts_with(&candidate);
            if prefix < 4 || !(starts_with || dist <= threshold) {
                return None;
            }
            Some((model_id, dist, prefix))
        })
        .collect();
    // ： before when （， and to 
    // provider 1 —— is 「 1 」 use ）
    scored.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.cmp(&a.2)));
    scored.truncate(limit);
    scored.into_iter().map(|(id, _, _)| id).collect()
}
