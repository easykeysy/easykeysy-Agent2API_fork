//! models.dev 1 （** 1 **）。
//!
//! ── （ has this 1 then ）───────────────────────────
//! ** use > up > models.dev > **。 this 1 ：
//! ** no has **（ or null） when ， use models.dev up ； up already 
//! ** 1 not **。 use in use after （`model_rules` 
//! `apply_capability_overrides`）， in after 。
//!
//! Why up not is models.dev ： up that is ** this ** 
//! （ server down `supportThinking`、CatPaw down 
//! `support_thinking`）， models.dev is in ** that API** up 
//! can —— 1 not in can and not 。 use after before ， etc 
//! 1 。
//!
//! ── （`capSources`）─────────────────────────────────
//! up 1 「 this is 」（`upstream` = has 、
//! `modelsdev` = 、`user` = use ）。 「」
//! and 「 / 」 —— no has this ， up 
//! 1 can ， can 。
//!
//! ── 1 （ 1 ）─────────────────────────────
//! 、default and in 1 ， not 。
//! up empty or unsupported when ， not use models.dev 。
//!
//! 1 after ：** this > models.dev effort **。
//! not to ** not this ** —— before use （「」 and 「 1 
//! no has 」 for use is ， `ui-islands` `model-capability.ts`）。

use serde_json::{json, Map, Value};

use super::snapshot::{self, Record};

/// and models.dev for （ and `capability::KEYS` 1 ）。
///
/// this for 1 ： by `capability` ，models.dev by 
/// ， in this 。
const BOOL_BRIDGE: [(&str, fn(&Record) -> Option<bool>); 4] = [
    ("supportsToolCall", |record| record.tool_call),
    ("supportsImages", |record| record.images),
    ("supportsVideo", |record| record.video),
    ("supportsReasoning", |record| record.reasoning),
];

const TOKEN_BRIDGE: [(&str, fn(&Record) -> Option<u64>); 2] = [
    ("maxInputTokens", |record| record.context),
    ("maxOutputTokens", |record| record.output),
];

/// 「」 use （ by ： in before ）。
///
/// before 1 is （ZCode use ）， after is —— 1 
/// 1 ，** not **： that when by （ by module header）。
const LEVEL_KEYS: [&str; 3] = ["reasoningLevels", "reasoningEfforts", "efforts"];

/// models.dev this （； up already not ）。
///
/// use has ， is 「 1 1 」： 
/// `providers::catalog::manifest_for`、 
/// `custom_providers::bindings::public_models`。 in not can can 「
/// 、 down no 」 that （ and can 1 by ）。
pub fn apply(item: &mut Value) {
    let Some(id) = item.get("id").and_then(Value::as_str).map(str::to_string) else { return };
    let record = snapshot::lookup(&id);
    let mut sources: Map<String, Value> = Map::new();
    if let Some(record) = &record {
        if let Some(object) = item.as_object_mut() {
            for (key, pick) in BOOL_BRIDGE {
                match object.get(key) {
                    // up ： not ，
                    Some(value) if !value.is_null() => {
                        sources.insert(key.to_string(), json!("upstream"));
                    }
                    _ => {
                        if let Some(flag) = pick(record) {
                            object.insert(key.to_string(), Value::Bool(flag));
                            sources.insert(key.to_string(), json!("modelsdev"));
                        }
                    }
                }
            }
            for (key, pick) in TOKEN_BRIDGE {
                match object.get(key) {
                    Some(value) if !value.is_null() => {
                        sources.insert(key.to_string(), json!("upstream"));
                    }
                    _ => {
                        if let Some(number) = pick(record) {
                            object.insert(key.to_string(), Value::from(number));
                            sources.insert(key.to_string(), json!("modelsdev"));
                        }
                    }
                }
            }
        }
    } else if let Some(object) = item.as_object() {
        // no need up down ：「 this up 」，
        // not is 「 no 」。
        for key in BOOL_BRIDGE.iter().map(|(key, _)| *key).chain(TOKEN_BRIDGE.iter().map(|(key, _)| *key)) {
            if object.get(key).is_some_and(|value| !value.is_null()) {
                sources.insert(key.to_string(), json!("upstream"));
            }
        }
    }
    if !sources.is_empty() {
        if let Some(object) = item.as_object_mut() {
            object.insert("capSources".to_string(), Value::Object(sources));
        }
    }
    normalize_levels(item, record.as_ref());
}

/// 1 （，module header）。
fn normalize_levels(item: &mut Value, record: Option<&Record>) {
    let declared = item.pointer("/reasoning/supportedEfforts").and_then(levels_of)
        .or_else(|| LEVEL_KEYS.iter().find_map(|key| item.get(*key).and_then(levels_of)));
    let binding_supported = item.get("reasoningBindingSupported").and_then(Value::as_bool);
    let fixed = if binding_supported == Some(false) { None } else {
        item.get("reasoningFixedLevel").and_then(level_text).or_else(|| {
            if declared.as_ref().is_some_and(|levels| !levels.is_empty()) { return None; }
            item.pointer("/reasoning/effort").and_then(level_text)
        })
    };
    let explicit = declared.is_some() || fixed.is_some() || binding_supported == Some(false);
    let levels = if binding_supported == Some(false) { Vec::new() }
    else if let Some(fixed) = &fixed {
        vec![fixed.clone()]
    } else if explicit {
        declared.unwrap_or_default()
    } else {
        record.map(|record| record.levels.clone()).unwrap_or_default()
    };
    let default = item.get("reasoningDefaultLevel").and_then(level_text)
        .or_else(|| item.pointer("/reasoning/defaultEffort").and_then(level_text))
        .or_else(|| item.pointer("/reasoning/effort").and_then(level_text))
        .and_then(|value| {
            if levels.is_empty() && item.get("supportsReasoning").and_then(Value::as_bool) != Some(false) { Some(value) }
            else { levels.iter().find(|level| level.eq_ignore_ascii_case(&value)).cloned() }
        }).or_else(|| fixed.clone());
    let source = item.get("reasoningSource").and_then(Value::as_str)
        .filter(|source| matches!(*source, "upstream" | "modelsdev" | "undeclared"))
        .map(str::to_string)
        .unwrap_or_else(|| {
            (if explicit { "upstream" } else if !levels.is_empty() { "modelsdev" } else { "undeclared" })
                .to_string()
        });
    let Some(object) = item.as_object_mut() else { return };
    if explicit || !levels.is_empty() {
        object.insert("reasoningLevels".into(), json!(levels));
    }
    object.remove("reasoningDefaultLevel");
    object.remove("reasoningFixedLevel");
    if let Some(default) = default { object.insert("reasoningDefaultLevel".into(), json!(default)); }
    if let Some(fixed) = fixed { object.insert("reasoningFixedLevel".into(), json!(fixed)); }
    object.insert("reasoningSource".into(), json!(source));
}

fn level_text(value: &Value) -> Option<String> {
    value.as_str().map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

/// from 1 （ / empty / 1 ）。
fn levels_of(value: &Value) -> Option<Vec<String>> {
    let items = value.as_array()?;
    Some(
        items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|level| !level.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

/// use ， `user`（ in use ）。
///
/// is ** use ** ， not is up not is models.dev —— need this 
/// ，otherwise use will as that is 。
///
/// `sources` （`upstream` / `modelsdev` / `user`） is ** before after **：
/// before by 1 （`ui-islands` `model-capability.ts`）。
pub fn mark_user(item: &mut Value, keys: &[String]) {
    if keys.is_empty() {
        return;
    }
    let Some(object) = item.as_object_mut() else { return };
    let mut sources = object
        .get("capSources")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for key in keys {
        sources.insert(key.clone(), json!("user"));
    }
    object.insert("capSources".to_string(), Value::Object(sources));
}
