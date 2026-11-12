//! can ： 、 1 ， and directory。
//!
//! ── this is ──────────────────────────────────────────
//! is `list_item`（`/v1/models` ） to ** down client** that ：
//!
//! maxInputTokens up down （client by ）
//! maxOutputTokens limit
//! supportsToolCall supports use 
//! supportsImages supports
//! supportsVideo supports
//! supportsReasoning supports
//!
//! directory（ / up directory）。** can is **
//! —— up directory、 not up up when ， down will by can request
//! （ to unsupported 、 by ）。 before 1 is ；
//! in use in （）， this is that ****。
//!
//! ── Why use （ and directory）， not is down ────
//! 「directory」（`apply_overrides`）， is `list_item`
//! 1 not use can ； down （`max_input_tokens` etc ） by 
//! `list_item` 1 。 if use 1 ， 1 ， that 
//! is 「 1 、 1 」 。
//!
//! ── use ──────────────────────────────────────
//! `modelRules.capabilities`（ `model_rules`），
//! `models[].capabilities` 。 、
//! this ， 1 in not will 。

use serde_json::{Map, Value};

/// can 。 and can （ before by 1 
/// ，`overridden_keys` by ）。
pub const KEYS: [&str; 6] = [
    "maxInputTokens",
    "maxOutputTokens",
    "supportsToolCall",
    "supportsImages",
    "supportsVideo",
    "supportsReasoning",
];

/// token limit（1 ）： is ， up down 
/// will down by error —— that is this need 1 ， not this by 
/// 1 new 。 down is 1（0 / for down no has ）。
pub const MAX_TOKEN_VALUE: f64 = 100_000_000.0;

/// this is not is token （`maxInputTokens` / `maxOutputTokens`）。
pub fn is_token_key(key: &str) -> bool {
    key == "maxInputTokens" || key == "maxOutputTokens"
}

/// this is not is can 。
pub fn is_bool_key(key: &str) -> bool {
    key == "supportsToolCall"
        || key == "supportsImages"
        || key == "supportsVideo"
        || key == "supportsReasoning"
}

/// 1 ：token （， `200000.0`）， bool；
/// its / 1 `None`（ use by 「」）。
pub fn normalize_value(key: &str, value: &Value) -> Option<Value> {
    if is_token_key(key) {
        let number = value.as_f64()?;
        if !number.is_finite() || number < 1.0 || number > MAX_TOKEN_VALUE {
            return None;
        }
        return Some(Value::from(number.trunc() as u64));
    }
    if is_bool_key(key) {
        return value.as_bool().map(Value::Bool);
    }
    None
}

/// from any JSON for can ，return****（ no to not ；
/// / ；null —— not in 「 null」 this 1 ，
/// ）。
pub fn normalize_object(value: Option<&Value>) -> Map<String, Value> {
    let mut result = Map::new();
    let Some(object) = value.and_then(Value::as_object) else {
        return result;
    };
    for key in KEYS {
        if let Some(found) = object.get(key).and_then(|item| normalize_value(key, item)) {
            result.insert(key.to_string(), found);
        }
    }
    result
}

/// directory（）。 not will null（ `normalize_object`），
/// this 「 has then 」。
pub fn apply_overrides(item: &mut Value, overrides: &Map<String, Value>) {
    if overrides.is_empty() {
        return;
    }
    let Some(object) = item.as_object_mut() else {
        return;
    };
    for (key, value) in overrides {
        object.insert(key.clone(), value.clone());
    }
}

/// can ： ，** to null**。
///
/// and `list_item` not ： that `false`（ down need ），
/// need 「unsupported」 and 「 not 」—— otherwise use when will 「」
/// 「unsupported」。 is 1 ， by to 。
pub fn effective(item: &Value) -> Value {
    let mut result = Map::new();
    for key in KEYS {
        let value = item.get(key).filter(|value| !value.is_null()).cloned();
        result.insert(key.to_string(), value.unwrap_or(Value::Null));
    }
    Value::Object(result)
}

/// （ by [`KEYS`] ）。
pub fn overridden_keys(overrides: &Map<String, Value>) -> Vec<String> {
    KEYS.iter()
        .filter(|key| overrides.contains_key(**key))
        .map(|key| (*key).to_string())
        .collect()
}
