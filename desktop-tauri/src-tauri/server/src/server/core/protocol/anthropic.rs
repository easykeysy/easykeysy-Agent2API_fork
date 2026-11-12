//! Anthropic Messages API（`POST /v1/messages`）↔ Chat Completions to 。
//!
//! ── and Responses ──────────────────────────
//! Anthropic and Chat Responses ， has must：
//!
//! 1. **system is **（ not is messages 1 ）—— Chat need 
//! 1 `role:"system"` in before 。
//! 2. ** in user **（`{type:"tool_result"}` ）， Chat 
//! is `role:"tool"` 。 when need 。
//! 3. ** use is content **（`{type:"tool_use"}`），Chat is 
//! `message.tool_calls` 。 and Anthropic ** need ** tool_use and tool_result
//! for ， need for （ `normalize_tool_pairing`）。
//!
//! ── （thinking） ────────────────────────────────
//! Anthropic is `thinking` + `signature`（，used for）；
//! Chat is `reasoning_content` 。Chat no has this ，：
//! - （Anthropic → Chat）：thinking `reasoning_content`，
//! signature ****（Chat none， and down up not ）；
//! - （Chat → Anthropic）：`reasoning_content` thinking ，
//! not signature（Anthropic in need need ）。
//!
//! `thinking.type:"enabled"` when Anthropic need `budget_tokens < max_tokens`，
//! will max_tokens to budget up （）。
//!
//! ── ──────────────────────────────────────────────────
//! `mod.rs`： unwrap/expect/panic。

use serde_json::{json, Map, Value};

use super::{
    content_parts, content_text, event_frame, is_truthy, json_text, native_tool, random_id,
    string_field, string_value, tool_plan, SseLineBuffer, TOOL_IMAGE_PLACEHOLDER,
    FIELD_CACHE_CONTROL, FIELD_IS_ERROR,
};
use super::responses::ConvertError;
use crate::server::logging;

/// Anthropic `max_tokens` 。
///
/// Anthropic ** need ** this ；client when to 1 not is 
/// （ will 「 1 」 request failure， 8192 for for use ）。
///
/// `pub(super)`： to （`anthropic_outbound`） 1 —— 
/// 1 。
pub(super) const DEFAULT_MAX_TOKENS: i64 = 8192;

// ─── request：Anthropic → Chat ─────────────────────────────────

/// Anthropic Messages request → Chat Completions request。
pub fn chat_from_anthropic(body: &Value) -> Result<Value, ConvertError> {
    let mut out = Map::new();
    out.insert("model".to_string(), body.get("model").cloned().unwrap_or(Value::Null));

    let mut messages: Vec<Value> = Vec::new();
    // ① system → before 1 system 
    if let Some(system) = body.get("system") {
        let text = system_text(system);
        if !text.trim().is_empty() {
            let mut entry = Map::new();
            entry.insert("role".to_string(), Value::String("system".to_string()));
            entry.insert("content".to_string(), Value::String(text));
            // system （Claude Code in system after 1 up ）
            // ， when system 
            if let Some(cache) = system
                .as_array()
                .map(|parts| last_cache_control(parts))
                .unwrap_or(None)
            {
                entry.insert(FIELD_CACHE_CONTROL.to_string(), cache);
            }
            messages.push(Value::Object(entry));
        }
    }
    // ② 
    let Some(input) = body.get("messages").and_then(Value::as_array) else {
        return Err("缺少 messages 数组".to_string());
    };
    for message in input {
        convert_message(&mut messages, message)?;
    }

    out.insert("messages".to_string(), Value::Array(messages));
    out.insert(
        "stream".to_string(),
        Value::Bool(body.get("stream").and_then(Value::as_bool).unwrap_or(false)),
    );
    // max_tokens is Anthropic ；Chat use max_tokens（ up 
    // max_completion_tokens， this to that ）
    let max_tokens = body
        .get("max_tokens")
        .and_then(Value::as_i64)
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_TOKENS);
    out.insert("max_tokens".to_string(), Value::from(max_tokens));

    for key in ["temperature", "top_p", "metadata"] {
        if let Some(value) = body.get(key).filter(|value| !value.is_null()) {
            out.insert(key.to_string(), value.clone());
        }
    }
    if let Some(stops) = body.get("stop_sequences").filter(|value| is_truthy(value)) {
        out.insert("stop".to_string(), stops.clone());
    }
    // ：Anthropic `{name, description, input_schema}` → Chat ；
    // server（`{"type":"web_search_20250305", …}`， up ）****——
    // `type` is （DeepSeek Anthropic web_search_*
    // ， 400）， in this 1 **empty function**
    // （`name` in 、`type` no ）， is 「 none use 」
    // （issue #61）。 to by ， `native_tool` module header。
    if let Some(tools) = body.get("tools").and_then(Value::as_array) {
        let mut converted: Vec<Value> = Vec::new();
        let mut natives: Vec<Value> = Vec::new();
        for tool in tools {
            if native_tool::is_native_anthropic(tool) {
                natives.push(native_tool::carry(tool, native_tool::ORIGIN_ANTHROPIC));
                continue;
            }
            if let Some(function) = tool_to_chat(tool) {
                converted.push(function);
            }
        }
        if !natives.is_empty() {
            logging::verbose(
                "[Anthropic]",
                &format!(
                    "原生（服务端执行）工具声明 {} 条随行：{}",
                    natives.len(),
                    natives
                        .iter()
                        .map(tool_plan::tool_kind_label)
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
            converted.extend(natives);
        }
        if !converted.is_empty() {
            out.insert("tools".to_string(), Value::Array(converted));
        }
    }
    if let Some(choice) = body.get("tool_choice").filter(|value| is_truthy(value)) {
        out.insert("tool_choice".to_string(), tool_choice_to_chat(choice));
    }
    // ：Anthropic thinking/output_config → Chat reasoning_effort。
    // Note budget and max_tokens （module header）， this ，
    // max_tokens in `effort_to_chat` 1 and 。
    if let Some(effort) = anthropic_effort(body) {
        out.insert("reasoning_effort".to_string(), Value::String(effort));
    }
    Ok(Value::Object(out))
}

/// system （ can is ， can is `{type:"text"}` ）
fn system_text(system: &Value) -> String {
    if let Some(text) = system.as_str() {
        return text.to_string();
    }
    let parts = content_parts(system);
    parts
        .iter()
        .map(|part| string_field(part, "text"))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// 1 up cache_control（ no has / not is for → None）
fn block_cache_control(part: &Value) -> Option<Value> {
    part.get("cache_control").filter(|value| value.is_object()).cloned()
}

/// ** after 1 ** cache_control （Anthropic is in 
/// need before up ， 1 / 1 system after 1 ）
fn last_cache_control(parts: &[Value]) -> Option<Value> {
    parts.iter().rev().find_map(block_cache_control)
}

/// 1 Anthropic → 1 to Chat （ `messages`）。
///
/// can can ：Anthropic tool_result in user content ，
/// Chat need tool `role:"tool"` 。 1 user if has 
/// has tool_result， need 「tool 」+「user 」—— ** not can **：
/// down need use assistant（ by 「」 1 ）。
fn convert_message(messages: &mut Vec<Value>, message: &Value) -> Result<(), ConvertError> {
    let role = string_field(message, "role").to_lowercase();
    if role != "user" && role != "assistant" {
        // Anthropic has user / assistant ； its user 
        // （ will 1 role request failure， its can ）
    }
    let content = message.get("content").unwrap_or(&Value::Null);
    // ： 1 
    if let Some(text) = content.as_str() {
        let role = if role == "assistant" { "assistant" } else { "user" };
        messages.push(json!({ "role": role, "content": text }));
        return Ok(());
    }
    let parts = content_parts(content);
    if parts.is_empty() {
        return Ok(());
    }

    // ：（ need tool ）、 use （ tool_calls）、
    // its （//，）
    let mut tool_results: Vec<Value> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();
    let mut reasoning = String::new();
    let mut normal: Vec<Value> = Vec::new();
    // cache_control → （ mod.rs「」 ）。
    // by Anthropic then ** not can ** cache_control， this not 。
    let mut normal_cache: Option<Value> = None;
    let mut tool_use_cache: Option<Value> = None;
    let mut tool_result_caches: Vec<Option<Value>> = Vec::new();
    for part in parts {
        if let Some(text) = part.as_str() {
            normal.push(json!({ "type": "text", "text": text }));
            continue;
        }
        let cache = block_cache_control(part);
        let kind = string_field(part, "type").to_lowercase();
        match kind.as_str() {
            "tool_result" => {
                tool_results.push(part.clone());
                tool_result_caches.push(cache.clone());
            }
            "tool_use" => {
                let id = string_field(part, "id");
                let arguments = {
                    let raw = json_text(part.get("input").unwrap_or(&Value::Null));
                    if raw.is_empty() { "{}".to_string() } else { raw }
                };
                tool_calls.push(json!({
                    "id": if id.is_empty() { random_id("call") } else { id },
                    "type": "function",
                    "function": {
                        "name": string_field(part, "name"),
                        "arguments": arguments,
                    },
                }));
                if cache.is_some() {
                    tool_use_cache = cache.clone();
                }
            }
            "thinking" | "redacted_thinking" => {
                let text = string_field(part, "thinking");
                if !text.is_empty() {
                    if !reasoning.is_empty() {
                        reasoning.push('\n');
                    }
                    reasoning.push_str(&text);
                }
            }
            "text" => normal.push(json!({ "type": "text", "text": string_field(part, "text") })),
            "image" => {
                if let Some(converted) = image_to_chat(part) {
                    normal.push(converted);
                }
            }
            _ => normal.push(part.clone()),
        }
        // ： after 1 cache_control as （）
        if cache.is_some() && kind != "tool_result" && kind != "tool_use" {
            normal_cache = cache.clone();
        }
    }

    let chat_role = if role == "assistant" { "assistant" } else { "user" };

    // ： tool ， and must in ** before **。
    //
    // Anthropic 1 user tool_result （Claude Desktop
    // skill ）， down need ：Chat tool 
    // must use assistant（ in for ， up 11148
    // "tool calls and tool results do not match"），Anthropic tool_result must
    // in user before （otherwise "Did not find 1 tool_result block(s) at the
    // beginning of this message"）。Anthropic is tool_result
    // in before 、 in after —— will use assistant and its ，
    // up for after request 400， will 。
    //
    // ：Chat tool not （OpenAI 400， by 
    // `responses::PendingImages`），、 in **** after 
    // 1 user 。 not need need Anthropic —— Anthropic need 1 
    // assistant tool_result in that user 。
    let mut tool_result_images: Vec<Value> = Vec::new();
    for (index, result) in tool_results.iter().enumerate() {
        let tool_use_id = string_field(result, "tool_use_id");
        let output = result.get("content").unwrap_or(&Value::Null);
        // is_error is 「 this failure」 ： after will failure
        // 。 in up ， by anthropic 
        // ；OpenAI no has this ， strip 。
        let is_error = result.get("is_error").and_then(Value::as_bool) == Some(true);
        let cache = tool_result_caches.get(index).cloned().flatten();
        let parts = tool_result_parts(output, image_to_chat);
        tool_result_images.extend(parts.images);
        let mut entry = Map::new();
        entry.insert("role".to_string(), Value::String("tool".to_string()));
        entry.insert(
            "tool_call_id".to_string(),
            Value::String(if tool_use_id.is_empty() { random_id("call") } else { tool_use_id }),
        );
        entry.insert(
            "content".to_string(),
            // has when `text` is empty， by [`TOOL_IMAGE_PLACEHOLDER`]
            Value::String(if parts.text.is_empty() {
                TOOL_IMAGE_PLACEHOLDER.to_string()
            } else {
                parts.text
            }),
        );
        if is_error {
            entry.insert(FIELD_IS_ERROR.to_string(), Value::Bool(true));
        }
        if let Some(cache) = cache {
            entry.insert(FIELD_CACHE_CONTROL.to_string(), cache);
        }
        messages.push(Value::Object(entry));
    }
    // after 、 before ：， is this 。
    // （client after old 1 ， user not and ——
    // up this ， and will client ）
    if !tool_result_images.is_empty() {
        messages.push(json!({ "role": "user", "content": Value::Array(tool_result_images) }));
    }

    // use ：must in assistant up （Anthropic tool_use in assistant ）
    if !tool_calls.is_empty() {
        let mut entry = Map::new();
        entry.insert("role".to_string(), Value::String("assistant".to_string()));
        let text = content_text(&Value::Array(normal.clone()));
        entry.insert(
            "content".to_string(),
            if text.is_empty() { Value::Null } else { Value::String(text) },
        );
        if !reasoning.is_empty() {
            entry.insert("reasoning_content".to_string(), Value::String(reasoning.clone()));
        }
        entry.insert("tool_calls".to_string(), Value::Array(tool_calls));
        // tool_use up （Claude Code ），otherwise use 
        // 1 —— is 1 Anthropic 
        if let Some(cache) = tool_use_cache.or(normal_cache) {
            entry.insert(FIELD_CACHE_CONTROL.to_string(), cache);
        }
        messages.push(Value::Object(entry));
    } else if !normal.is_empty() || !reasoning.is_empty() {
        let mut entry = Map::new();
        entry.insert("role".to_string(), Value::String(chat_role.to_string()));
        entry.insert(
            "content".to_string(),
            collapse_content(Value::Array(normal.clone())),
        );
        // reasoning_content in assistant up has 
        if chat_role == "assistant" && !reasoning.is_empty() {
            entry.insert("reasoning_content".to_string(), Value::String(reasoning));
        }
        if let Some(cache) = normal_cache {
            entry.insert(FIELD_CACHE_CONTROL.to_string(), cache);
        }
        messages.push(Value::Object(entry));
    }
    Ok(())
}

/// （ [`tool_result_parts`]）
pub(super) struct ToolResultParts {
    /// （ can can as empty —— has when by use ）
    pub(super) text: String,
    /// ，** already by use to **
    pub(super) images: Vec<Value>,
}

/// → + 。
///
/// `convert` is this to 「 → 」 （ to 
/// [`image_to_chat`]， to `image_to_anthropic`）： not in this 
/// ， use to `images` 1 down ， not 1 。
///
/// ── Whymust（ and responses.rs issue #76 1 ）──────
/// Anthropic `tool_result.content` ****，
/// `text` / `image` / `document` / `search_result` ， is use 
/// （Claude Code 1 、MCP this ）。 1 
/// [`content_text`] ，** **； has when then `json_text`
/// base64 —— before is not to up down （ token
/// ， as token not ）， after and issue #76 。
///
/// after ： Chat tool ， by use user 
/// （Chat tool not ， by `responses::PendingImages`）。
///
/// `pub(super)`： to （`anthropic_outbound`） chat tool 
/// `tool_result` when use 1 ， 1 。
pub(super) fn tool_result_parts(
    content: &Value,
    convert: fn(&Value) -> Option<Value>,
) -> ToolResultParts {
    let text_only = |text: String| ToolResultParts { text, images: Vec::new() };
    match content {
        Value::String(text) => {
            if text.is_empty() { text_only("(empty)".to_string()) } else { text_only(text.clone()) }
        }
        Value::Null => text_only("(empty)".to_string()),
        Value::Array(parts) => {
            let images: Vec<Value> = parts
                .iter()
                .filter(|part| is_image_block(part))
                .filter_map(|part| convert(part))
                .collect();
            let text = content_text(content);
            if text.is_empty() && images.is_empty() {
                // 1 not down （e.g. Anthropic Files API `file` 、
                // has `search_result` / `document` ）： 
                // ，empty
                let raw = json_text(content);
                return text_only(if raw.is_empty() { "(empty)".to_string() } else { raw });
            }
            ToolResultParts { text, images }
        }
        other => {
            let raw = json_text(other);
            text_only(if raw.is_empty() { "(empty)".to_string() } else { raw })
        }
    }
}

/// is not is （Anthropic `image` and Chat `image_url` / `input_image`
/// —— to is its in 1 ， not to `convert` ）
fn is_image_block(part: &Value) -> bool {
    matches!(
        string_field(part, "type").to_lowercase().as_str(),
        "image" | "image_url" | "input_image"
    )
}

/// Anthropic image → Chat image_url 。
///
/// Anthropic base64 （`{type:"base64", media_type, data}`） need data URI，
/// as Chat URL 。
fn image_to_chat(part: &Value) -> Option<Value> {
    let source = part.get("source")?;
    let kind = string_field(source, "type").to_lowercase();
    match kind.as_str() {
        "base64" => {
            let media_type = string_field(source, "media_type");
            let data = string_field(source, "data");
            if data.is_empty() {
                return None;
            }
            let media = if media_type.is_empty() { "image/png".to_string() } else { media_type };
            Some(json!({
                "type": "image_url",
                "image_url": { "url": format!("data:{media};base64,{data}") },
            }))
        }
        "url" => {
            let url = string_field(source, "url");
            if url.is_empty() {
                None
            } else {
                Some(json!({ "type": "image_url", "image_url": { "url": url } }))
            }
        }
        _ => None,
    }
}

/// → Chat content（ has when ）
fn collapse_content(content: Value) -> Value {
    let parts = match content.as_array() {
        Some(parts) => parts.clone(),
        None => return content,
    };
    if parts.is_empty() {
        return Value::String(String::new());
    }
    let only_text = parts.iter().all(|part| {
        let kind = string_field(part, "type").to_lowercase();
        kind.is_empty() || kind == "text"
    });
    if only_text {
        let text: String = parts
            .iter()
            .map(|part| string_field(part, "text"))
            .collect();
        return Value::String(text);
    }
    Value::Array(parts)
}

/// Anthropic → Chat 
fn tool_to_chat(tool: &Value) -> Option<Value> {
    let name = string_field(tool, "name");
    if name.is_empty() {
        return None;
    }
    let parameters = tool
        .get("input_schema")
        .filter(|value| is_truthy(value))
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
    let mut function = Map::new();
    function.insert("name".to_string(), Value::String(name));
    if let Some(description) = tool.get("description").filter(|value| is_truthy(value)) {
        function.insert("description".to_string(), description.clone());
    }
    function.insert("parameters".to_string(), parameters);
    let mut out = Map::new();
    out.insert("type".to_string(), Value::String("function".to_string()));
    out.insert("function".to_string(), Value::Object(function));
    // up （， is Anthropic 
    // 1 ）， when to anthropic for up 
    if let Some(cache) = block_cache_control(tool) {
        out.insert(FIELD_CACHE_CONTROL.to_string(), cache);
    }
    Some(Value::Object(out))
}

/// Anthropic tool_choice → Chat tool_choice
///
/// for ：`auto`→`auto`、`any`→`required`、`none`→`none`、
/// `{type:"tool", name}`→`{type:"function", function:{name}}`。
fn tool_choice_to_chat(choice: &Value) -> Value {
    if let Some(text) = choice.as_str() {
        return Value::String(text.to_string());
    }
    let kind = string_field(choice, "type").to_lowercase();
    match kind.as_str() {
        "any" => Value::String("required".to_string()),
        "auto" => Value::String("auto".to_string()),
        "none" => Value::String("none".to_string()),
        "tool" => {
            let name = string_field(choice, "name");
            if name.is_empty() {
                Value::String("auto".to_string())
            } else {
                json!({ "type": "function", "function": { "name": name } })
            }
        }
        _ => Value::String("auto".to_string()),
    }
}

/// Anthropic → Chat `reasoning_effort`
///
/// （）：`thinking.type` ，`output_config.effort`
/// 。`disabled`/`none` 1 not （ up use default）。
fn anthropic_effort(body: &Value) -> Option<String> {
    let thinking_type = {
        let raw = string_value(
            body.get("thinking").and_then(|value| value.get("type")).unwrap_or(&Value::Null),
        );
        raw.trim().to_lowercase()
    };
    if matches!(thinking_type.as_str(), "none" | "off" | "disabled") {
        return None;
    }
    let explicit = string_value(
        body.get("output_config").and_then(|value| value.get("effort")).unwrap_or(&Value::Null),
    );
    let explicit = explicit.trim().to_lowercase();
    if !explicit.is_empty() {
        return Some(explicit);
    }
    // thinking.enabled no to ： by budget 1 
    if thinking_type == "enabled" || thinking_type == "adaptive" {
        let budget = body
            .get("thinking")
            .and_then(|value| value.get("budget_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        return Some(effort_from_budget(budget).to_string());
    }
    None
}

/// budget → 
fn effort_from_budget(budget: i64) -> &'static str {
    if budget <= 0 {
        return "medium";
    }
    if budget <= 1024 {
        "low"
    } else if budget <= 4096 {
        "medium"
    } else if budget <= 10240 {
        "high"
    } else if budget <= 32768 {
        "xhigh"
    } else {
        "max"
    }
}

// ─── response：Chat → Anthropic（）────────────────────────

/// Chat `chat.completion` → Anthropic `message` for 。
pub fn anthropic_from_chat(chat: &Value, model: &str) -> Value {
    let choice = chat.pointer("/choices/0");
    let message = choice.and_then(|choice| choice.get("message")).cloned().unwrap_or(Value::Null);
    let finish = choice
        .and_then(|choice| choice.get("finish_reason"))
        .map(string_value)
        .unwrap_or_default();

    let mut content: Vec<Value> = Vec::new();
    // in before （ and Anthropic 1 ）
    let reasoning = {
        let from_field = string_field(&message, "reasoning_content");
        if from_field.is_empty() { string_field(&message, "reasoning") } else { from_field }
    };
    if !reasoning.is_empty() {
        content.push(json!({ "type": "thinking", "thinking": reasoning }));
    }
    let text = content_text(message.get("content").unwrap_or(&Value::Null));
    if !text.is_empty() {
        content.push(json!({ "type": "text", "text": text }));
    }
    let mut has_tool = false;
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            has_tool = true;
            let id = string_field(call, "id");
            content.push(json!({
                "type": "tool_use",
                "id": if id.is_empty() { random_id("toolu") } else { id },
                "name": call.pointer("/function/name").map(string_value).unwrap_or_default(),
                "input": parse_json_object(
                    call.pointer("/function/arguments").unwrap_or(&Value::Null)
                ),
            }));
        }
    }
    // Anthropic need content empty
    if content.is_empty() {
        content.push(json!({ "type": "text", "text": "" }));
    }
    let stop_reason = if finish == "length" {
        "max_tokens"
    } else if finish == "tool_calls" || has_tool {
        "tool_use"
    } else if finish == "stop" || finish.is_empty() {
        "end_turn"
    } else {
        // its finish_reason（content_filter etc ）：Anthropic no has for ，
        // to end_turn（ 1 not in ）
        "end_turn"
    };
    let id = {
        let raw = string_field(chat, "id");
        if raw.is_empty() { random_id("msg") } else { raw }
    };
    json!({
        "id": id,
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": content,
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": usage_to_anthropic(chat.get("usage")),
    })
}

/// parameter（ or for ） for 
///
/// `pub(super)`： to （`anthropic_outbound`） tool_use input need 
/// 「 is for 」 this 1 1 （ up for for input will 400）。
pub(super) fn parse_json_object(value: &Value) -> Value {
    if value.is_object() {
        return value.clone();
    }
    let text = json_text(value);
    if text.is_empty() {
        return json!({});
    }
    match serde_json::from_str::<Value>(&text) {
        Ok(parsed) if parsed.is_object() => parsed,
        // not （ up to JSON parameter）： 1 ， input is for 
        _ => json!({ "raw": text }),
    }
}

/// Chat usage → Anthropic usage
pub fn usage_to_anthropic(usage: Option<&Value>) -> Value {
    let Some(usage) = usage.filter(|value| value.is_object()) else {
        return json!({ "input_tokens": 0, "output_tokens": 0 });
    };
    let number = |key: &str| -> i64 {
        usage.get(key).and_then(Value::as_i64).unwrap_or(0)
    };
    let cached = usage
        .pointer("/prompt_tokens_details/cached_tokens")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let input = number("prompt_tokens");
    json!({
        // Anthropic input_tokens ** not ** in ， this 
        "input_tokens": (input - cached).max(0),
        "output_tokens": number("completion_tokens"),
        "cache_read_input_tokens": cached,
    })
}

/// token ： ÷ 3.5（ in ， and Anthropic 
/// 「 3.5 1 token」）。empty to 1。
///
/// and `/v1/messages/count_tokens`（`api::chat::count_tokens`）** use this 1 **：
/// client will ， for not up new 。 need 「 not 」 use 
/// `None` / empty， not is this return 0。
pub fn estimate_tokens_for_text(text: &str) -> i64 {
    let characters = text.chars().count();
    ((characters as f64) / 3.5).ceil().max(1.0) as i64
}

/// by request token（ for JSON ， [`json_text`]）。
///
/// Anthropic and after chat can ： is for 、 1 ，
/// 1 `model` not use 。
pub fn estimate_tokens_for_body(body: &Value) -> i64 {
    estimate_tokens_for_text(&json_text(body))
}

// ─── ：Chat SSE → Anthropic SSE ─────────────────────────

/// Chat SSE → Anthropic SSE （）。
///
/// ── Anthropic （client by this ）───────────
///   message_start
/// content_block_start / content_block_delta / content_block_stop（ 1 ）
/// message_delta（ stop_reason and usage）
///   message_stop
///
/// Anthropic is **** ： 1 stop after can start down 1 。
/// down `block_open`， when old 。
pub struct AnthropicStream {
    buffer: SseLineBuffer,
    message_id: String,
    model: String,
    started: bool,
    finished: bool,
    /// current ： + + /parameter
    block_open: bool,
    block_index: i64,
    block_type: String,
    block_text: String,
    block_tool_id: String,
    block_tool_name: String,
    block_arguments: String,
    saw_arguments: bool,
    has_tool: bool,
    stop_reason: Option<String>,
    usage: Option<Value>,
    /// down request token （`estimate_tokens_for_body`）。
    ///
    /// in `message_start` use up ： up usage need etc that to ，client
    /// （Claude Code / Anthropic SDK） in content block that 1 
    /// `message_start` usage ， etc not to `message_delta`。 0
    /// will and （issue #437： 2.6M / 1M 50 ）。
    input_estimate: Option<i64>,
}

impl AnthropicStream {
    /// `input_estimate`：`message_start` use ，`None` use no has 
    /// request can （ 0， and before 1 ）。 usage when that as 。
    pub fn new(model: &str, input_estimate: Option<i64>) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            message_id: random_id("msg"),
            model: model.to_string(),
            started: false,
            finished: false,
            block_open: false,
            block_index: -1,
            block_type: String::new(),
            block_text: String::new(),
            block_tool_id: String::new(),
            block_tool_name: String::new(),
            block_arguments: String::new(),
            saw_arguments: false,
            has_tool: false,
            stop_reason: None,
            usage: None,
            input_estimate,
        }
    }

    pub fn push(&mut self, chunk: &[u8]) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        for payload in self.buffer.push(chunk) {
            match payload {
                None => out.extend(self.finish()),
                Some(data) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&data) {
                        out.extend(self.consume(&value));
                    }
                }
            }
        }
        out
    }

    pub fn finish(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let mut out = Vec::new();
        for payload in self.buffer.finish() {
            if let Some(data) = payload {
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    out.extend(self.consume(&value));
                }
            }
        }
        if self.finished {
            return out;
        }
        self.finished = true;
        out.extend(self.emit_start());
        out.extend(self.close_block());
        let stop_reason = self
            .stop_reason
            .clone()
            .unwrap_or_else(|| if self.has_tool { "tool_use".to_string() } else { "end_turn".to_string() });
        out.push(self.event(
            "message_delta",
            json!({
                "delta": { "stop_reason": stop_reason, "stop_sequence": Value::Null },
                // Chat usage can can to ；message_delta and usage。
                "usage": usage_to_anthropic(self.usage.as_ref()),
            }),
        ));
        out.push(self.event("message_stop", json!({})));
        out
    }

    fn consume(&mut self, chunk: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let mut out = Vec::new();
        if let Some(error) = chunk.get("error").filter(|value| is_truthy(value)) {
            self.finished = true;
            out.extend(self.emit_start());
            let message = {
                let text = string_field(error, "message");
                if text.is_empty() { string_value(error) } else { text }
            };
            let kind = {
                let text = string_field(error, "type");
                if text.is_empty() { "api_error".to_string() } else { text }
            };
            out.push(self.event("error", json!({ "error": { "type": kind, "message": message } })));
            return out;
        }
        if let Some(id) = chunk.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
            if !self.started {
                self.message_id = id.to_string();
            }
        }
        if let Some(usage) = chunk.get("usage").filter(|value| value.is_object()) {
            self.usage = Some(usage.clone());
        }
        let choice = chunk.pointer("/choices/0");
        if let Some(finish) = choice
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
        {
            self.stop_reason = Some(if finish == "length" {
                "max_tokens".to_string()
            } else if finish == "tool_calls" {
                "tool_use".to_string()
            } else {
                "end_turn".to_string()
            });
        }
        out.extend(self.emit_start());
        let Some(delta) = choice.and_then(|choice| choice.get("delta")) else {
            return out;
        };
        // → thinking 
        let reasoning = {
            let from_field = string_field(delta, "reasoning_content");
            if from_field.is_empty() { string_field(delta, "reasoning") } else { from_field }
        };
        if !reasoning.is_empty() {
            out.extend(self.ensure_block("thinking"));
            self.block_text.push_str(&reasoning);
            out.push(self.event(
                "content_block_delta",
                json!({
                    "index": self.block_index,
                    "delta": { "type": "thinking_delta", "thinking": reasoning },
                }),
            ));
        }
        // → text 
        if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|text| !text.is_empty()) {
            out.extend(self.ensure_block("text"));
            self.block_text.push_str(text);
            out.push(self.event(
                "content_block_delta",
                json!({
                    "index": self.block_index,
                    "delta": { "type": "text_delta", "text": text },
                }),
            ));
        }
        // use → tool_use 
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                out.extend(self.consume_tool(call));
            }
        }
        out
    }

    fn consume_tool(&mut self, call: &Value) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        let name = call.pointer("/function/name").and_then(Value::as_str).unwrap_or("");
        let id = call.get("id").and_then(Value::as_str).unwrap_or("");
        // new use （ id/name and and current not ）： old 、 new 
        let is_new = !id.is_empty() && id != self.block_tool_id;
        if is_new || (self.block_type != "tool_use" && !name.is_empty()) {
            out.extend(self.close_block());
            self.has_tool = true;
            self.block_open = true;
            self.block_index += 1;
            self.block_type = "tool_use".to_string();
            self.block_tool_id = if id.is_empty() { random_id("toolu") } else { id.to_string() };
            self.block_tool_name = name.to_string();
            self.block_arguments.clear();
            self.saw_arguments = false;
            out.push(self.event(
                "content_block_start",
                json!({
                    "index": self.block_index,
                    "content_block": {
                        "type": "tool_use",
                        "id": self.block_tool_id,
                        "name": self.block_tool_name,
                        "input": {},
                    },
                }),
            ));
        } else if self.block_type != "tool_use" {
            // has parameter、 no to id/name： 1 （）
            out.extend(self.close_block());
            self.has_tool = true;
            self.block_open = true;
            self.block_index += 1;
            self.block_type = "tool_use".to_string();
            self.block_tool_id = random_id("toolu");
            self.block_tool_name = String::new();
            self.block_arguments.clear();
            self.saw_arguments = false;
            out.push(self.event(
                "content_block_start",
                json!({
                    "index": self.block_index,
                    "content_block": {
                        "type": "tool_use",
                        "id": self.block_tool_id,
                        "name": "",
                        "input": {},
                    },
                }),
            ));
        }
        // to （parameter、 after name）： 1 
        if !name.is_empty() && self.block_tool_name.is_empty() {
            self.block_tool_name = name.to_string();
        }
        let arguments = call.pointer("/function/arguments").and_then(Value::as_str).unwrap_or("");
        if !arguments.is_empty() {
            self.saw_arguments = true;
            self.block_arguments.push_str(arguments);
            out.push(self.event(
                "content_block_delta",
                json!({
                    "index": self.block_index,
                    "delta": { "type": "input_json_delta", "partial_json": arguments },
                }),
            ));
        }
        out
    }

    /// current is （ not then old ）
    fn ensure_block(&mut self, kind: &str) -> Vec<bytes::Bytes> {
        if self.block_open && self.block_type == kind {
            return Vec::new();
        }
        let mut out = self.close_block();
        self.block_open = true;
        self.block_index += 1;
        self.block_type = kind.to_string();
        self.block_text.clear();
        let content_block = if kind == "thinking" {
            json!({ "type": "thinking", "thinking": "" })
        } else {
            json!({ "type": "text", "text": "" })
        };
        out.push(self.event(
            "content_block_start",
            json!({ "index": self.block_index, "content_block": content_block }),
        ));
        out
    }

    fn close_block(&mut self) -> Vec<bytes::Bytes> {
        if !self.block_open {
            return Vec::new();
        }
        self.block_open = false;
        // before if 1 parameter no to ， 1 empty for ：
        // Anthropic client in input_json_delta when not input
        let mut out = Vec::new();
        if self.block_type == "tool_use" && !self.saw_arguments {
            out.push(self.event(
                "content_block_delta",
                json!({
                    "index": self.block_index,
                    "delta": { "type": "input_json_delta", "partial_json": "{}" },
                }),
            ));
        }
        out.push(self.event("content_block_stop", json!({ "index": self.block_index })));
        out
    }

    fn emit_start(&mut self) -> Vec<bytes::Bytes> {
        if self.started {
            return Vec::new();
        }
        self.started = true;
        let mut usage = usage_to_anthropic(self.usage.as_ref());
        usage["output_tokens"] = json!(0);
        // up usage not （ need etc that ），client 
        // `message_start` usage —— 0 will 
        // and （issue #437）。 use use to up ； to when 
        // by `message_delta` 。 up this before usage ，
        // not 。 not in ， not in this —— 
        // `cache_read_input_tokens` will requeststatistics in 。
        if self.usage.is_none() {
            if let Some(estimate) = self.input_estimate {
                usage["input_tokens"] = json!(estimate);
            }
        }
        vec![self.event(
            "message_start",
            json!({
                "message": {
                    "id": self.message_id,
                    "type": "message",
                    "role": "assistant",
                    "model": self.model,
                    "content": [],
                    "stop_reason": Value::Null,
                    "stop_sequence": Value::Null,
                    "usage": usage,
                },
            }),
        )]
    }

    fn event(&self, event: &str, mut data: Value) -> bytes::Bytes {
        if let Some(map) = data.as_object_mut() {
            map.insert("type".to_string(), Value::String(event.to_string()));
        }
        event_frame(event, &data)
    }
}

/// ： Chat SSE 1 Anthropic message for 。
pub struct AnthropicCollector {
    buffer: SseLineBuffer,
    text: String,
    reasoning: String,
    tools: std::collections::BTreeMap<i64, ToolAccum>,
    usage: Option<Value>,
    finish_reason: Option<String>,
    id: String,
}

#[derive(Default)]
struct ToolAccum {
    call_id: String,
    name: String,
    arguments: String,
}

impl AnthropicCollector {
    pub fn new() -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            text: String::new(),
            reasoning: String::new(),
            tools: std::collections::BTreeMap::new(),
            usage: None,
            finish_reason: None,
            id: String::new(),
        }
    }

    pub fn push(&mut self, chunk: &[u8]) {
        for payload in self.buffer.push(chunk) {
            if let Some(data) = payload {
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    self.consume(&value);
                }
            }
        }
    }

    pub fn finish(&mut self) {
        for payload in self.buffer.finish() {
            if let Some(data) = payload {
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    self.consume(&value);
                }
            }
        }
    }

    fn consume(&mut self, chunk: &Value) {
        if let Some(id) = chunk.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
            if self.id.is_empty() {
                self.id = id.to_string();
            }
        }
        if let Some(usage) = chunk.get("usage").filter(|value| value.is_object()) {
            self.usage = Some(usage.clone());
        }
        let choice = chunk.pointer("/choices/0");
        if let Some(finish) = choice
            .and_then(|choice| choice.get("finish_reason"))
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
        {
            self.finish_reason = Some(finish.to_string());
        }
        let Some(delta) = choice.and_then(|choice| choice.get("delta")) else {
            return;
        };
        let reasoning = {
            let from_field = string_field(delta, "reasoning_content");
            if from_field.is_empty() { string_field(delta, "reasoning") } else { from_field }
        };
        self.reasoning.push_str(&reasoning);
        if let Some(text) = delta.get("content").and_then(Value::as_str) {
            self.text.push_str(text);
        }
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let key = call.get("index").and_then(Value::as_i64).unwrap_or(0);
                let entry = self.tools.entry(key).or_default();
                if let Some(id) = call.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
                    entry.call_id = id.to_string();
                }
                if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                    if !name.is_empty() {
                        entry.name = name.to_string();
                    }
                }
                if let Some(arguments) = call.pointer("/function/arguments").and_then(Value::as_str) {
                    entry.arguments.push_str(arguments);
                }
            }
        }
    }

    pub fn into_response(self, model: &str) -> Value {
        let mut content: Vec<Value> = Vec::new();
        if !self.reasoning.is_empty() {
            content.push(json!({ "type": "thinking", "thinking": self.reasoning }));
        }
        if !self.text.is_empty() {
            content.push(json!({ "type": "text", "text": self.text }));
        }
        let mut has_tool = false;
        for (_, tool) in self.tools {
            has_tool = true;
            let arguments = if tool.arguments.is_empty() { "{}".to_string() } else { tool.arguments };
            content.push(json!({
                "type": "tool_use",
                "id": if tool.call_id.is_empty() { random_id("toolu") } else { tool.call_id },
                "name": if tool.name.is_empty() { "unknown".to_string() } else { tool.name },
                "input": parse_json_object(&Value::String(arguments)),
            }));
        }
        if content.is_empty() {
            content.push(json!({ "type": "text", "text": "" }));
        }
        let stop_reason = match self.finish_reason.as_deref() {
            Some("length") => "max_tokens",
            Some("tool_calls") => "tool_use",
            _ if has_tool => "tool_use",
            _ => "end_turn",
        };
        let id = if self.id.is_empty() { random_id("msg") } else { self.id.clone() };
        json!({
            "id": id,
            "type": "message",
            "role": "assistant",
            "model": model,
            "content": content,
            "stop_reason": stop_reason,
            "stop_sequence": Value::Null,
            "usage": usage_to_anthropic(self.usage.as_ref()),
        })
    }
}

impl Default for AnthropicCollector {
    fn default() -> Self {
        Self::new()
    }
}
/// error Anthropic error（）
pub fn error_kind_for_status(status: u16) -> &'static str {
    match status {
        400 => "invalid_request_error",
        401 => "authentication_error",
        403 => "permission_error",
        404 => "not_found_error",
        413 => "request_too_large",
        429 => "rate_limit_error",
        500..=599 => "api_error",
        _ => "api_error",
    }
}

/// error Responses error
pub fn responses_error_code(status: u16) -> &'static str {
    match status {
        400 => "invalid_request_error",
        401 => "authentication_error",
        403 => "permission_error",
        404 => "not_found_error",
        429 => "rate_limit_error",
        500..=599 => "server_error",
        _ => "api_error",
    }
}
