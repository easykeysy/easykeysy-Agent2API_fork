//! Anthropic Messages ****：chat request → Anthropic up request、
//! up Anthropic SSE → chat SSE（ `anthropic` ，
//! and ZCode —— `upstream::translate` `AnthropicToChatStream`）。
//!
//! ── and `anthropic.rs` to ─────────────────────────────
//! `anthropic.rs` ** down **（`api::protocol` `/v1/messages`）；
//! file is to ， and ZCode ** up **：
//! - request：`anthropic_request_from_chat`（chat → Messages ，
//! for `chat_from_anthropic` ）；
//! - response：`ChatFromAnthropicStream`（ up Anthropic SSE → chat SSE，
//! after has `ForwardStream` / ）。
//!
//! ── （ and `anthropic.rs` module header as ）────
//! 1. chat system → `system` （empty）；
//! 2. chat `role:"tool"` → user `tool_result` ；
//! 3. chat `message.tool_calls` → assistant `tool_use` 
//! （`arguments` for ；Anthropic input must is for ）。
//!
//! ── thinking Anthropic then （ in this ）────────────
//! - `thinking.type:"enabled"` need `budget_tokens < max_tokens`：
//! when if max_tokens not ， to `budget + 1024`（）；
//! - thinking when Anthropic `temperature` / `top_p`（temperature
//! 1）： and its up 400， not ** not ** this optionalparameter —— and 
//! `model_rules` 「 not to up will request 」 1 to 。
//!
//! ── ──────────────────────────────────────────────────
//! `mod.rs`： unwrap/expect/panic；failure「 this 」。

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use super::{
    chat_frame, content_parts, content_text, is_truthy, json_text, native_tool, random_id,
    string_field, string_value, SseLineBuffer, FIELD_CACHE_CONTROL, FIELD_IS_ERROR,
};
use super::anthropic::{parse_json_object, tool_result_parts, DEFAULT_MAX_TOKENS, ToolResultParts};
use super::responses::ConvertError;
use crate::server::core::model_rules;

// ─── request：Chat → Anthropic ─────────────────────────────────

/// Chat Completions request → Anthropic Messages request。
///
/// ── for [`super::anthropic::chat_from_anthropic`] ────
/// - `model`（ by use to up ）
/// - `system` ← chat system / developer （empty）
/// - `messages` ← its （tool user tool_result ；
/// tool_calls assistant tool_use ； and 1 ——
/// Anthropic need user / assistant ）
/// - `max_tokens` ← `max_tokens` / `max_completion_tokens`，
/// [`DEFAULT_MAX_TOKENS`]（Anthropic ）
///   - `temperature` / `top_p` / `stop_sequences`（← chat `stop`）
/// - `tools`（chat function → `{name, description, input_schema}`）
///   - `tool_choice`（`required`→`any`、`{type:function,function:{name}}`
///     →`{type:"tool",name}`）
/// - `thinking` ← chat `reasoning_effort`（ etc budget，
/// [`thinking_budget`]； when can can max_tokens）
pub fn anthropic_request_from_chat(chat: &Value, model: &str) -> Result<Value, ConvertError> {
    let Some(messages_in) = chat.get("messages").and_then(Value::as_array) else {
        return Err("缺少 messages 数组".to_string());
    };
    let mut system: Vec<String> = Vec::new();
    // system up （ 1 system —— is 
    // 「 before to as 」， system after in ）
    let mut system_cache: Option<Value> = None;
    let mut messages: Vec<Value> = Vec::new();
    for message in messages_in {
        let role = {
            let raw = string_field(message, "role").to_lowercase();
            if raw == "developer" {
                "system".to_string()
            } else if raw.is_empty() {
                "user".to_string()
            } else {
                raw
            }
        };
        if role == "system" {
            let text = content_text(message.get("content").unwrap_or(&Value::Null));
            if !text.trim().is_empty() {
                system.push(text);
                if let Some(cache) =
                    message.get(FIELD_CACHE_CONTROL).filter(|value| value.is_object())
                {
                    system_cache = Some(cache.clone());
                }
            }
            continue;
        }
        let blocks = anthropic_blocks_of(message, &role);
        if blocks.is_empty() {
            continue;
        }
        let target_role = if role == "assistant" { "assistant" } else { "user" };
        append_merged(&mut messages, target_role, blocks);
    }

    let mut out = Map::new();
    out.insert("model".to_string(), Value::String(model.to_string()));
    out.insert("messages".to_string(), Value::Array(messages));
    out.insert(
        "stream".to_string(),
        Value::Bool(chat.get("stream").and_then(Value::as_bool).unwrap_or(false)),
    );
    if !system.is_empty() {
        match system_cache {
            Some(cache) => {
                // system use （Anthropic ），
                // cache_control in after 1 —— and when 1 
                let mut blocks: Vec<Value> = system
                    .iter()
                    .map(|text| json!({ "type": "text", "text": text }))
                    .collect();
                if let Some(last) = blocks.last_mut() {
                    if let Some(object) = last.as_object_mut() {
                        object.insert("cache_control".to_string(), cache);
                    }
                }
                out.insert("system".to_string(), Value::Array(blocks));
            }
            None => {
                out.insert("system".to_string(), Value::String(system.join("\n\n")));
            }
        }
    }

    // thinking ： max_tokens down ， temperature/top_p
    // can not can （module header then ）
    let thinking = chat
        .get("reasoning_effort")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .and_then(thinking_budget);
    let mut max_tokens = ["max_tokens", "max_completion_tokens"]
        .iter()
        .find_map(|key| chat.get(*key).and_then(Value::as_i64).filter(|value| *value > 0))
        .unwrap_or(DEFAULT_MAX_TOKENS);
    if let Some(budget) = thinking {
        // Anthropic need budget_tokens < max_tokens； not to budget up 
        if max_tokens <= budget {
            max_tokens = budget + 1024;
        }
    }
    out.insert("max_tokens".to_string(), Value::from(max_tokens));
    if thinking.is_some() {
        if let Some(budget) = thinking {
            out.insert(
                "thinking".to_string(),
                json!({ "type": "enabled", "budget_tokens": budget }),
            );
        }
    } else {
        for key in ["temperature", "top_p"] {
            if let Some(value) = chat.get(key).filter(|value| !value.is_null()) {
                out.insert(key.to_string(), value.clone());
            }
        }
    }
    // stop → stop_sequences（chat or ，Anthropic ）
    if let Some(stop) = chat.get("stop").filter(|value| is_truthy(value)) {
        let list = match stop {
            Value::Array(items) => items.clone(),
            other => vec![other.clone()],
        };
        if !list.is_empty() {
            out.insert("stop_sequences".to_string(), Value::Array(list));
        }
    }
    // ： Anthropic ；（server） has 
    // 「 is Anthropic」 （）， not —— and 
    // （ `native_tool` module header； or to up is #61 that ）
    let mut natives: Vec<Value> = Vec::new();
    if let Some(tools) = chat.get("tools").and_then(Value::as_array) {
        let converted: Vec<Value> = tools
            .iter()
            .filter_map(|tool| tool_to_anthropic(tool, &mut natives))
            .collect();
        if !converted.is_empty() {
            out.insert("tools".to_string(), Value::Array(converted));
        }
    }
    let mut choice_reason: Option<String> = None;
    if let Some(choice) = chat.get("tool_choice").filter(|value| is_truthy(value)) {
        // when `tool_choice` 1 and ： up will by 
        // 「 not in 」， 1 「unavailable」 400
        match native_tool::choice_conflict(choice, &natives) {
            Some(reason) => choice_reason = Some(reason),
            None => {
                out.insert("tool_choice".to_string(), tool_choice_to_anthropic(choice));
            }
        }
    }
    if !natives.is_empty() || choice_reason.is_some() {
        let dropped = native_tool::Downgrade { tools: natives, choice: choice_reason };
        crate::server::logging::log(
            "[Anthropic]",
            &dropped.describe(None, Some("目标上游按 anthropic 协议收，只认 anthropic 来源的原生声明")),
        );
    }
    Ok(Value::Object(out))
}

/// → Anthropic `tool_result.content`。
///
/// ── and to （ not is ）──────────────────────────
/// （`anthropic.rs`）**must** tool ：Chat not `tool` 
/// （OpenAI 400， `responses::PendingImages`）， that to 
/// user up 。 this not use —— Anthropic `tool_result.content`
/// （ `text` / `image` / `document` / `search_result`），
/// in is 。
///
/// no has when ： for up ， is in 。
fn tool_result_content(parts: ToolResultParts) -> Value {
    if parts.images.is_empty() {
        return Value::String(parts.text);
    }
    let mut blocks: Vec<Value> = Vec::new();
    if !parts.text.is_empty() {
        blocks.push(json!({ "type": "text", "text": parts.text }));
    }
    blocks.extend(parts.images);
    Value::Array(blocks)
}

/// 1 system chat → Anthropic 。
fn anthropic_blocks_of(message: &Value, role: &str) -> Vec<Value> {
    // tool → tool_result （ in user up ；Anthropic need 
    // tool_result and assistant tool_use for ，tool_use_id must to 
    // in id —— id client for ， 1 will 
    // up for to its ，、 up ）
    if role == "tool" {
        // is_error ： when 「failure」（
        // `anthropic::convert_message`）—— OpenAI no has this ，
        // that path strip ， has this can 
        let is_error = message.get(FIELD_IS_ERROR).and_then(Value::as_bool) == Some(true);
        let cache = message.get(FIELD_CACHE_CONTROL).filter(|value| value.is_object());
        let mut block = json!({
            "type": "tool_result",
            "tool_use_id": string_field(message, "tool_call_id"),
            "content": tool_result_content(tool_result_parts(
                message.get("content").unwrap_or(&Value::Null),
                image_to_anthropic,
            )),
        });
        if let Some(object) = block.as_object_mut() {
            if is_error {
                object.insert("is_error".to_string(), Value::Bool(true));
            }
            if let Some(cache) = cache {
                object.insert("cache_control".to_string(), cache.clone());
            }
        }
        return vec![block];
    }
    let mut blocks: Vec<Value> = Vec::new();
    if role == "assistant" {
        // → thinking （ not signature：Chat no has ，
        // `anthropic.rs` module header to ）
        let reasoning = {
            let from_field = string_field(message, "reasoning_content");
            if from_field.is_empty() {
                string_field(message, "reasoning")
            } else {
                from_field
            }
        };
        if !reasoning.is_empty() {
            blocks.push(json!({ "type": "thinking", "thinking": reasoning }));
        }
    }
    // ： / （ → image ， [`image_to_anthropic`]）
    match message.get("content").unwrap_or(&Value::Null) {
        Value::String(text) => {
            if !text.is_empty() {
                blocks.push(json!({ "type": "text", "text": text }));
            }
        }
        content => {
            for part in content_parts(content) {
                if let Some(text) = part.as_str() {
                    if !text.is_empty() {
                        blocks.push(json!({ "type": "text", "text": text }));
                    }
                    continue;
                }
                let kind = string_field(part, "type").to_lowercase();
                match kind.as_str() {
                    "text" | "input_text" | "output_text" => {
                        let text = string_field(part, "text");
                        if !text.is_empty() {
                            blocks.push(json!({ "type": "text", "text": text }));
                        }
                    }
                    "image_url" | "input_image" if role != "assistant" => {
                        if let Some(image) = image_to_anthropic(part) {
                            blocks.push(image);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    // use → tool_use （input must is for ；`arguments` not when 
    // 1 raw， and `anthropic.rs` `parse_json_object` ）
    if role == "assistant" {
        if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                let id = {
                    let raw = string_field(call, "id");
                    if raw.is_empty() {
                        random_id("toolu")
                    } else {
                        raw
                    }
                };
                blocks.push(json!({
                    "type": "tool_use",
                    "id": id,
                    "name": call.pointer("/function/name").map(string_value).unwrap_or_default(),
                    "input": parse_json_object(
                        call.pointer("/function/arguments").unwrap_or(&Value::Null)
                    ),
                }));
            }
        }
    }
    // → after 1 can 。Anthropic not 
    // cache_control in thinking up ，； no has can when 
    // （ to up 1 will 400）
    if let Some(cache) = message.get(FIELD_CACHE_CONTROL).filter(|value| value.is_object()) {
        for block in blocks.iter_mut().rev() {
            let kind = string_field(block, "type");
            let mountable = matches!(kind.as_str(), "text" | "tool_use" | "tool_result" | "image");
            if mountable {
                if let Some(object) = block.as_object_mut() {
                    object.insert("cache_control".to_string(), cache.clone());
                }
                break;
            }
        }
    }
    blocks
}

/// and 1 （Anthropic need user / assistant ，
/// will up ）。
fn append_merged(messages: &mut Vec<Value>, role: &str, blocks: Vec<Value>) {
    if let Some(last) = messages.last_mut() {
        let same_role = last.get("role").and_then(Value::as_str) == Some(role);
        if same_role {
            if let Some(existing) = last.get_mut("content").and_then(Value::as_array_mut) {
                existing.extend(blocks);
                return;
            }
        }
    }
    messages.push(json!({ "role": role, "content": blocks }));
}

/// Chat image_url → Anthropic image 。
///
/// need ：`data:` URI（ media_type and base64 ）→
/// `source:{type:"base64"}`；http(s) → `source:{type:"url"}`。
/// its （ for path、 data URI） —— 1 up not source
/// will request 400。
fn image_to_anthropic(part: &Value) -> Option<Value> {
    let source = part.get("image_url").unwrap_or(part);
    let url = match source {
        Value::String(text) => text.clone(),
        other => string_field(other, "url"),
    };
    let url = url.trim();
    if url.is_empty() {
        return None;
    }
    if let Some(rest) = url.strip_prefix("data:") {
        let (meta, data) = rest.split_once(',')?;
        // `data:<media_type>;base64,<data>`； no has `;base64` after 
        // （ base64 ）Anthropic not ，
        let media_type = meta.strip_suffix(";base64")?;
        if data.is_empty() {
            return None;
        }
        let media_type = if media_type.is_empty() {
            "image/png".to_string()
        } else {
            media_type.to_string()
        };
        return Some(json!({
            "type": "image",
            "source": { "type": "base64", "media_type": media_type, "data": data },
        }));
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        return Some(json!({
            "type": "image",
            "source": { "type": "url", "url": url },
        }));
    }
    None
}

/// Chat → Anthropic （`input_schema` ）。
///
/// （server）： is Anthropic （`natives`
/// not ，）； its （Responses 、 or chat ）
/// `natives` by use —— is anthropic， not 
/// （ `native_tool` module header）。
fn tool_to_anthropic(tool: &Value, natives: &mut Vec<Value>) -> Option<Value> {
    if native_tool::is_native(tool) {
        if native_tool::origin_of(tool) == Some(native_tool::ORIGIN_ANTHROPIC) {
            return Some(native_tool::restore(tool));
        }
        natives.push(tool.clone());
        return None;
    }
    // chat will has ； function for （ etc ）
    let function = tool.get("function").unwrap_or(tool);
    let name = string_field(function, "name");
    if name.is_empty() {
        return None;
    }
    let mut out = Map::new();
    out.insert("name".to_string(), Value::String(name));
    if let Some(description) = function.get("description").filter(|value| is_truthy(value)) {
        out.insert("description".to_string(), description.clone());
    }
    let schema = function.get("parameters").unwrap_or(&Value::Null);
    out.insert("input_schema".to_string(), normalize_input_schema(schema));
    // up （， is 
    // Anthropic 1 ； `anthropic::tool_to_chat`）
    if let Some(cache) = tool.get(FIELD_CACHE_CONTROL).filter(|value| value.is_object()) {
        out.insert("cache_control".to_string(), cache.clone());
    }
    Some(Value::Object(out))
}

/// `input_schema` 1 ：Anthropic need `type:"object"`、`properties`
/// in （ for `normalizeAnthropicInputSchema`）。
fn normalize_input_schema(schema: &Value) -> Value {
    let kind = string_field(schema, "type").to_lowercase();
    if !schema.is_object() || (!kind.is_empty() && kind != "object") {
        return json!({ "type": "object", "properties": {} });
    }
    let mut out = schema.as_object().cloned().unwrap_or_default();
    out.insert("type".to_string(), Value::String("object".to_string()));
    out.entry("properties".to_string()).or_insert_with(|| json!({}));
    Value::Object(out)
}

/// Chat tool_choice → Anthropic tool_choice（`chat_from_anthropic`
/// `tool_choice_to_chat` to ：`required`↔`any`、function↔tool）。
fn tool_choice_to_anthropic(choice: &Value) -> Value {
    if let Some(text) = choice.as_str() {
        return match text {
            "required" => json!({ "type": "any" }),
            "none" => json!({ "type": "none" }),
            // `auto` and its to auto（Anthropic this 
            // ； 1 not is 400）
            _ => json!({ "type": "auto" }),
        };
    }
    let name = {
        let nested = choice.pointer("/function/name").map(string_value).unwrap_or_default();
        if nested.is_empty() {
            string_field(choice, "name")
        } else {
            nested
        }
    };
    if name.is_empty() {
        return json!({ "type": "auto" });
    }
    json!({ "type": "tool", "name": name })
}

/// chat `reasoning_effort` → Anthropic `budget_tokens`。
///
/// 1 1 ， by [`model_rules::reasoning_rank`] ； etc 
/// （） no has can ，return None（ and `model_rules` 
/// 「 not and 」 1 ）：
///
/// ```text
///   rank 0..1（minimal / low）→ 1024
///   rank 2    （medium）      → 4096
///   rank 3    （high）        → 10240
///   rank 4..5（xhigh / max）  → 32768
/// ```
///
/// and `thinkingBudget` for （ low/medium/high/max 
/// in this up ）。
fn thinking_budget(effort: &str) -> Option<i64> {
    let rank = model_rules::reasoning_rank(effort)?;
    Some(match rank {
        0 | 1 => 1024,
        2 => 4096,
        3 => 10240,
        _ => 32768,
    })
}

// ─── response：Anthropic SSE → Chat SSE（）────────────────────

/// up Anthropic SSE → chat SSE （）。
///
/// （ for `AnthropicStream` to and 
/// `anthropicStreamToChat`， by chat ）：
/// - `message_start` → （role assistant）+ down message.id and 
/// `message.usage`（** by **， [`Self::absorb_usage`]）
/// - `content_block_start`（tool_use）→ `tool_calls` （id / name）
///   - `content_block_delta`：`text_delta` → `delta.content`；
///     `thinking_delta` → `delta.reasoning_content`；
/// `input_json_delta` → `delta.tool_calls[…]`；`signature_delta` 
/// - `message_delta` → stop_reason and usage（ is ** by **：
/// `input_tokens` in this 1 ， [`Self::absorb_usage`]）
/// - `message_stop` → + usage + `data: [DONE]`
/// - `error` → `data: {"error":{…}}` + `data: [DONE]`（ and ForwardStream
/// ， 502）
/// - `ping` and ignore
pub struct ChatFromAnthropicStream {
    buffer: SseLineBuffer,
    /// model（ to up ；ForwardStream by need ）
    model: String,
    id: String,
    created: i64,
    started: bool,
    finished: bool,
    /// tool_use （ by up content_block index ）→ is already parameter
    /// （ 1 no when ，`content_block_stop` need 1 empty for parameter）
    tools: BTreeMap<i64, bool>,
    input_tokens: i64,
    cache_read: i64,
    cache_creation: i64,
    output_tokens: i64,
    /// already chat finish_reason（`message_delta` to ）
    finish_reason: Option<String>,
}

impl ChatFromAnthropicStream {
    pub fn new(model: &str) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            model: model.to_string(),
            id: String::new(),
            created: crate::server::logging::now_ms() / 1000,
            started: false,
            finished: false,
            tools: BTreeMap::new(),
            input_tokens: 0,
            cache_read: 0,
            cache_creation: 0,
            output_tokens: 0,
            finish_reason: None,
        }
    }

    /// 1 up ， need down chat SSE 
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

    /// up （ no has `message_stop` when ）
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
        out.extend(self.complete());
        out
    }

    /// 1 up Anthropic → to chat 
    fn consume(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let kind = string_field(event, "type").to_lowercase();
        match kind.as_str() {
            "ping" => Vec::new(),
            "error" => self.fail(event),
            "message_start" => {
                if let Some(id) = event
                    .pointer("/message/id")
                    .and_then(Value::as_str)
                    .filter(|id| !id.is_empty())
                {
                    self.id = id.to_string();
                }
                if let Some(usage) = event.pointer("/message/usage").filter(|usage| usage.is_object()) {
                    self.absorb_usage(usage);
                }
                self.start()
            }
            "content_block_start" => {
                let block = event.get("content_block").unwrap_or(&Value::Null);
                if string_field(block, "type").to_lowercase() != "tool_use" {
                    return Vec::new();
                }
                let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                self.tools.insert(index, false);
                let mut out = self.start();
                let call_id = {
                    let raw = string_field(block, "id");
                    if raw.is_empty() {
                        random_id("toolu")
                    } else {
                        raw
                    }
                };
                out.push(self.delta_frame(json!({
                    "tool_calls": [{
                        "index": index,
                        "id": call_id,
                        "type": "function",
                        "function": { "name": string_field(block, "name"), "arguments": "" },
                    }],
                })));
                // `block.input` already is for when （ up not input_json_delta）
                // as parameter down 
                if let Some(map) = block.get("input").and_then(Value::as_object) {
                    if !map.is_empty() {
                        self.tools.insert(index, true);
                        out.push(self.arguments_frame(index, &json_text(&Value::Object(map.clone()))));
                    }
                }
                out
            }
            "content_block_delta" => {
                let delta = event.get("delta").unwrap_or(&Value::Null);
                let delta_kind = string_field(delta, "type").to_lowercase();
                match delta_kind.as_str() {
                    // 
                    "text_delta" => {
                        let text = string_field(delta, "text");
                        if text.is_empty() {
                            return Vec::new();
                        }
                        let mut out = self.start();
                        out.push(self.delta_frame(json!({ "content": text })));
                        out
                    }
                    // → reasoning_content
                    "thinking_delta" => {
                        let text = string_field(delta, "thinking");
                        if text.is_empty() {
                            return Vec::new();
                        }
                        let mut out = self.start();
                        out.push(self.delta_frame(json!({ "reasoning_content": text })));
                        out
                    }
                    // parameter（ by content_block index ）
                    "input_json_delta" => {
                        let partial = string_field(delta, "partial_json");
                        if partial.is_empty() {
                            return Vec::new();
                        }
                        let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                        // no to content_block_start （ up 
                        // when parameter has can ； by name/id）
                        self.tools.insert(index, true);
                        let mut out = self.start();
                        out.push(self.arguments_frame(index, &partial));
                        out
                    }
                    // ：Chat none（ `anthropic.rs` module header）
                    "signature_delta" => Vec::new(),
                    _ => Vec::new(),
                }
            }
            // ： 1 parameter no when 1 empty for 
            // （ for `AnthropicStream::close_block` to ）
            "content_block_stop" => {
                let index = event.get("index").and_then(Value::as_i64).unwrap_or(0);
                if self.tools.get(&index) == Some(&false) {
                    self.tools.insert(index, true);
                    return vec![self.arguments_frame(index, "{}")];
                }
                Vec::new()
            }
            // before after 1 ：stop_reason and usage（output_tokens）
            "message_delta" => {
                if let Some(reason) = event.pointer("/delta/stop_reason").map(string_value) {
                    let reason = reason.trim().to_lowercase();
                    if !reason.is_empty() {
                        self.finish_reason = Some(match reason.as_str() {
                            "max_tokens" => "length".to_string(),
                            "tool_use" => "tool_calls".to_string(),
                            // end_turn / stop_sequence / refusal etc ：
                            // Chat no has for ， to stop
                            _ => "stop".to_string(),
                        });
                    }
                }
                if let Some(usage) = event.get("usage").filter(|usage| usage.is_object()) {
                    self.absorb_usage(usage);
                }
                Vec::new()
            }
            "message_stop" => self.complete(),
            _ => Vec::new(),
        }
    }

    /// 1 usage，** by **update（ this 1 no ）。
    ///
    /// ── Whymust by not is ──────────────────────────
    /// up usage is **** up ， not is 。
    /// （2026-09-30，`zcode.z.ai` ，glm-5.3-flash ）：
    ///
    /// ```text
    ///   message_start  → {"usage":{"input_tokens":0,"output_tokens":0}}
    ///   message_delta  → {"usage":{"input_tokens":13,"output_tokens":64,
    ///                              "cache_read_input_tokens":0,…}}
    /// ```
    ///
    /// that `input_tokens` **0 **， in before that ；
    /// `output_tokens` has valid。：
    /// - use 「 after 1 」 ， will to that has ；
    /// - `output_tokens`（ before ） then and ** as 0** ——
    /// this is issue #56「 already use 70M、statistics 3M」 
    /// （， etc 1 ）。
    ///
    /// use 「 in not in 」 not is 「 is 」：0 is up （ in 
    /// is 0），「 no 」 will to 1 old 。
    fn absorb_usage(&mut self, usage: &Value) {
        let take = |key: &str| usage.get(key).and_then(Value::as_i64);
        if let Some(value) = take("input_tokens") {
            self.input_tokens = value;
        }
        if let Some(value) = take("cache_read_input_tokens") {
            self.cache_read = value;
        }
        if let Some(value) = take("cache_creation_input_tokens") {
            self.cache_creation = value;
        }
        if let Some(value) = take("output_tokens") {
            self.output_tokens = value;
        }
    }

    /// （role assistant）， 1 
    fn start(&mut self) -> Vec<bytes::Bytes> {
        if self.started {
            return Vec::new();
        }
        self.started = true;
        if self.id.is_empty() {
            self.id = random_id("chatcmpl");
        }
        vec![self.delta_frame(json!({ "role": "assistant", "content": "" }))]
    }

    /// ：finish_reason + usage + [DONE]（ 1 ）
    fn complete(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        if !self.started {
            // or none not can in EOF when 1 success empty。
            return self.fail_message("上游未返回有效 Anthropic 消息");
        }
        self.finished = true;
        let mut out = self.start();
        let finish_reason = self.finish_reason.clone().unwrap_or_else(|| {
            if self.tools.is_empty() {
                "stop".to_string()
            } else {
                "tool_calls".to_string()
            }
        });
        out.push(self.finish_frame(&finish_reason));
        // chat ：input_tokens （`usage_to_anthropic` to 
        // not etc —— that is 「」， this ）。 1 ：
        // requeststatistics 「」 by `prompt_tokens_details.cached_tokens` 
        // （ `upstream::usage::extract_usage`）， not this 1 in 
        // up 0 —— and 0 is 1 「 for 、 its no to 」。
        // `cache_creation` and this 1 ： in prompt_tokens ，
        // not and will 「input = prompt - cached」 for not up 。
        let cached = self.cache_read + self.cache_creation;
        let prompt_tokens = self.input_tokens + cached;
        out.push(chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [],
            "usage": {
                "prompt_tokens": prompt_tokens,
                "completion_tokens": self.output_tokens,
                "total_tokens": prompt_tokens + self.output_tokens,
                "prompt_tokens_details": { "cached_tokens": cached },
            },
        })));
        out.push(bytes::Bytes::from_static(b"data: [DONE]\n\n"));
        out
    }

    /// up error → chat error + [DONE]（ 1 ）
    fn fail(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let error = event.get("error").filter(|error| is_truthy(error));
        let message = match error {
            Some(error) => {
                let text = string_field(error, "message");
                if text.is_empty() { string_value(error) } else { text }
            }
            None => string_value(event),
        };
        let message = if message.trim().is_empty() {
            "上游流式返回错误".to_string()
        } else {
            message
        };
        self.fail_message(&message)
    }

    /// will to error as chat error。
    fn fail_message(&mut self, message: &str) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;
        vec![
            chat_frame(&json!({
                "error": { "message": message, "type": "upstream_error" },
            })),
            bytes::Bytes::from_static(b"data: [DONE]\n\n"),
        ]
    }

    /// 1 delta 
    fn delta_frame(&self, delta: Value) -> bytes::Bytes {
        chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": Value::Null }],
        }))
    }

    /// parameter（ index and function.arguments —— ，
    /// id / name in already to ）
    fn arguments_frame(&self, index: i64, arguments: &str) -> bytes::Bytes {
        self.delta_frame(json!({
            "tool_calls": [{ "index": index, "function": { "arguments": arguments } }],
        }))
    }

    /// （empty delta + finish_reason）
    fn finish_frame(&self, finish_reason: &str) -> bytes::Bytes {
        chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [{ "index": 0, "delta": {}, "finish_reason": finish_reason }],
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChatFromAnthropicStream, Value};

    fn output_text(frames: &[bytes::Bytes]) -> String {
        frames
            .iter()
            .map(|frame| String::from_utf8_lossy(frame).into_owned())
            .collect()
    }

    #[test]
    fn empty_upstream_stream_is_reported_as_an_error() {
        for input in [
            "",
            ": keepalive\n\n",
            "data: {\"type\":\"ping\"}\n\n",
            "data: {\"type\":\"unknown\"}\n\n",
            "data: {broken}\n\n",
            "data: {\"type\":",
            "<html>not an SSE response</html>",
            "data: [DONE]\n\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        ] {
            let mut stream = ChatFromAnthropicStream::new("glm-5.3");
            let mut frames = stream.push(input.as_bytes());
            frames.extend(stream.finish());
            assert_eq!(frames.len(), 2, "input: {input}");
            let text = std::str::from_utf8(&frames[0]).unwrap();
            let error: Value = serde_json::from_str(text.strip_prefix("data: ").unwrap()).unwrap();
            assert_eq!(error["error"]["type"], "upstream_error");
            assert_eq!(error["error"]["message"], "上游未返回有效 Anthropic 消息");
            assert!(error.get("choices").is_none());
            assert_eq!(frames[1].as_ref(), b"data: [DONE]\n\n");
            assert!(stream.finish().is_empty());
            assert!(stream.push(b"data: [DONE]\n\n").is_empty());
        }
    }

    #[test]
    fn valid_message_stream_still_completes_normally() {
        let mut stream = ChatFromAnthropicStream::new("glm-5.3");
        let output = stream
            .push(
                b"data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"usage\":{\"input_tokens\":3}}}\n\n",
            )
            .into_iter()
            .chain(stream.push(b"data: {\"type\":\"message_stop\"}\n\n"))
            .map(|frame| String::from_utf8_lossy(&frame).into_owned())
            .collect::<String>();

        assert!(!output.contains("\"error\""));
        assert!(output.contains("\"finish_reason\":\"stop\""));
        assert!(output.contains("data: [DONE]"));
    }

    #[test]
    fn content_without_message_start_survives_fragmentation_and_eof() {
        for (kind, field, chat_field) in [
            ("text_delta", "text", "content"),
            ("thinking_delta", "thinking", "reasoning_content"),
        ] {
            let input = format!(
                "data: {{\"type\":\"content_block_delta\",\"delta\":{{\"type\":\"{kind}\",\"{field}\":\"你好\"}}}}"
            );
            let mut stream = ChatFromAnthropicStream::new("glm-5.3");
            let mut frames = Vec::new();
            for chunk in input.as_bytes().chunks(1) {
                frames.extend(stream.push(chunk));
            }
            frames.extend(stream.finish());
            let output = output_text(&frames);
            assert!(output.contains(&format!("\"{chat_field}\":\"你好\"")));
            assert!(output.contains("\"finish_reason\":\"stop\""));
            assert!(!output.contains("\"error\""));
        }
    }

    #[test]
    fn tool_arguments_are_preserved_without_message_start() {
        let mut stream = ChatFromAnthropicStream::new("glm-5.3");
        let mut frames = stream.push(concat!(
            "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool_1\",\"name\":\"lookup\",\"input\":{}}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"key\\\":1}\"}}\n\n",
            "data: {\"type\":\"content_block_stop\",\"index\":1}\n\n"
        ).as_bytes());
        frames.extend(stream.finish());
        let output = output_text(&frames);
        assert!(output.contains("\"name\":\"lookup\""));
        assert!(output.contains("\"arguments\":\"{\\\"key\\\":1}\""));
        assert!(output.contains("\"finish_reason\":\"tool_calls\""));
        assert!(!output.contains("\"error\""));
    }

    #[test]
    fn upstream_error_is_preserved_and_terminated_once() {
        let mut stream = ChatFromAnthropicStream::new("glm-5.3");
        let frames = stream.push(
            b"data: {\"type\":\"error\",\"error\":{\"message\":\"upstream unavailable\"}}\n\n",
        );
        assert_eq!(frames.len(), 2);
        assert!(output_text(&frames).contains("upstream unavailable"));
        assert!(stream.finish().is_empty());
        assert!(stream.push(b"data: [DONE]\n\n").is_empty());
    }

    #[tokio::test]
    async fn empty_stream_becomes_502_when_aggregated() {
        use crate::server::core::upstream::{
            aggregate::aggregate_frame_stream, sse::FramePolicy, usage::RequestTelemetry,
        };
        use std::sync::Arc;

        let mut stream = ChatFromAnthropicStream::new("glm-5.3");
        let frames = futures::stream::iter(stream.finish().into_iter().map(Ok));
        let result =
            aggregate_frame_stream(Box::pin(frames), Arc::new(RequestTelemetry::new()), FramePolicy::default()).await;
        match result {
            Err(error) => {
                assert_eq!(error.status_code, 502);
                assert_eq!(error.message, "上游未返回有效 Anthropic 消息");
            }
            Ok(_) => panic!("empty upstream must not aggregate to a successful completion"),
        }
    }
}
