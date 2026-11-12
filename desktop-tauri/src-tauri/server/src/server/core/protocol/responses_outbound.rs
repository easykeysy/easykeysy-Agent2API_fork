//! Responses ****：chat request → Responses up request、
//! up Responses SSE → chat SSE（ `responses` ）。
//!
//! ── and `responses.rs` to ─────────────────────────────
//! `responses.rs` is ** down **（`api::protocol` `/v1/responses`）：
//! client Responses request chat、 chat response/SSE 
//! Responses。file is ** to **， ** up **：
//! 1 is chat， this up Responses， is 
//! - request：`responses_request_from_chat`（chat → Responses ，
//! for `chat_from_responses` ）；
//! - response：`ChatFromResponsesStream`（ up Responses SSE → chat SSE，
//! after has `ForwardStream` / —— that chat ）。
//!
//! ── up `stream:true` request ────────────────────────────
//! and chat 1 （ up is can ； down by 
//! ）。response need need ； JSON path not will has up 
//! 。 OmniProxy `responsesStreamToChat`，
//! by chat （`responses.rs` ）。
//!
//! ── ──────────────────────────────────────────────────
//! `mod.rs`： unwrap/expect/panic；failure「 this 」。

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use super::{
    chat_frame, content_parts, content_text, is_truthy, json_number_of, json_text, native_tool,
    random_id, string_field, string_value, SseLineBuffer, FIELD_ENCRYPTED_CONTENT,
};
use super::responses::{image_url_of, tool_output_parts, ConvertError, ToolOutput};
use super::tool_plan;

// ─── request：Chat → Responses ─────────────────────────────────

/// Chat Completions request → Responses request。
///
/// ── for [`super::responses::chat_from_responses`] ────
/// that Responses request chat， in ； this 
/// （ 1 is 「clientparameter」）：
/// - `model`（ by use to up ， `providers::custom::forward`）
/// - `messages` → `instructions`（system ）+ `input`（message /
/// function_call / function_call_output ）
///   - `stream` / `max_output_tokens`（← `max_completion_tokens` / `max_tokens`）
///   - `temperature` / `top_p` / `service_tier` / `parallel_tool_calls` /
///     `user` / `metadata`
/// - `tools`（chat function → Responses function）
///   - `tool_choice`、`response_format` → `text.format`
///   - `reasoning_effort` → `reasoning:{effort, summary:"auto"}`
/// - `store: false`（none； for up not ）
///
/// unsupported chat （`n` / `logprobs` etc ）****：Responses no has 
/// for ， up not will 1 400。
pub fn responses_request_from_chat(chat: &Value, model: &str) -> Result<Value, ConvertError> {
    let Some(messages) = chat.get("messages").and_then(Value::as_array) else {
        return Err("缺少 messages 数组".to_string());
    };
    let mut instructions: Vec<String> = Vec::new();
    let mut items: Vec<Value> = Vec::new();
    for message in messages {
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
        match role.as_str() {
            // system / developer → instructions（ use empty，
            // and anthropic system_text 1 ）
            "system" => {
                let text = content_text(message.get("content").unwrap_or(&Value::Null));
                if !text.trim().is_empty() {
                    instructions.push(text);
                }
            }
            // tool → function_call_output （call_id is for ，
            // up in not this use ）
            "tool" => {
                items.push(json!({
                    "type": "function_call_output",
                    "call_id": string_field(message, "tool_call_id"),
                    "output": tool_output_to_responses(tool_output_parts(message.get("content"))),
                }));
            }
            "assistant" => push_assistant_items(&mut items, message),
            // user and 「 not 」 user（ and chat_from_responses 
            // 1 to ： can ）
            _ => {
                let content =
                    chat_content_to_responses(message.get("content").unwrap_or(&Value::Null), "user");
                items.push(json!({ "type": "message", "role": "user", "content": content }));
            }
        }
    }

    let mut out = Map::new();
    out.insert("model".to_string(), Value::String(model.to_string()));
    out.insert("input".to_string(), Value::Array(items));
    out.insert(
        "stream".to_string(),
        Value::Bool(chat.get("stream").and_then(Value::as_bool).unwrap_or(false)),
    );
    if !instructions.is_empty() {
        out.insert(
            "instructions".to_string(),
            Value::String(instructions.join("\n\n")),
        );
    }
    // max_output_tokens ← max_completion_tokens（Responses ），
    // to max_tokens； as no to （0 / up will up ）
    let max_output = ["max_completion_tokens", "max_tokens"]
        .iter()
        .find_map(|key| chat.get(*key).and_then(Value::as_i64).filter(|value| *value > 0));
    if let Some(value) = max_output {
        out.insert("max_output_tokens".to_string(), Value::from(value));
    }
    for key in ["temperature", "top_p", "service_tier", "parallel_tool_calls", "user", "metadata"] {
        if let Some(value) = chat.get(key).filter(|value| !value.is_null()) {
            out.insert(key.to_string(), value.clone());
        }
    }
    // ： Responses ；（server） has 
    // 「 is Responses」 （， web_search / tool_search etc 
    // 1 ）， not —— and （ `native_tool` module header）
    let mut natives: Vec<Value> = Vec::new();
    if let Some(tools) = chat.get("tools").and_then(Value::as_array) {
        let converted: Vec<Value> = tools
            .iter()
            .filter_map(|tool| tool_to_responses(tool, &mut natives))
            .collect();
        if !converted.is_empty() {
            out.insert("tools".to_string(), Value::Array(converted));
        }
    }
    let mut choice_reason: Option<String> = None;
    if let Some(choice) = chat.get("tool_choice").filter(|value| is_truthy(value)) {
        // when `tool_choice` 1 and （ by `anthropic_outbound`）
        match native_tool::choice_conflict(choice, &natives) {
            Some(reason) => choice_reason = Some(reason),
            None => {
                out.insert("tool_choice".to_string(), tool_choice_to_responses(choice));
            }
        }
    }
    if !natives.is_empty() || choice_reason.is_some() {
        let dropped = native_tool::Downgrade { tools: natives, choice: choice_reason };
        crate::server::logging::log(
            "[Responses]",
            &dropped.describe(None, Some("目标上游按 responses 协议收，只认 responses 来源的原生声明")),
        );
    }
    // response_format → text.format（json_schema → ）
    if let Some(format) = chat.get("response_format").filter(|value| is_truthy(value)) {
        out.insert("text".to_string(), json!({ "format": format_to_responses(format) }));
    }
    // reasoning_effort → reasoning.effort（ up 「」 1 
    // from this ；summary:"auto" and —— up need ）
    if let Some(effort) = chat
        .get("reasoning_effort")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        out.insert(
            "reasoning".to_string(),
            json!({ "effort": effort, "summary": "auto" }),
        );
    }
    // store:false and module header 1 by （none， not up ）
    out.insert("store".to_string(), Value::Bool(false));
    Ok(Value::Object(out))
}

/// 1 assistant → to input （：reasoning → message →
/// function_call）。
///
/// not is ： `chat_from_responses` by 「reasoning in before 、
/// assistant in after 」（`PendingReasoning`）， to must 1 
/// ， can up 。
fn push_assistant_items(items: &mut Vec<Value>, message: &Value) {
    let reasoning = {
        let from_field = string_field(message, "reasoning_content");
        if from_field.is_empty() {
            string_field(message, "reasoning")
        } else {
            from_field
        }
    };
    // （store=false ） reasoning ： and summary 
    // can and （OpenAI is ）， has when summary
    // to empty。 no has this ，Codex / Grok CLI down 1 not up ——
    // up down 1 request。
    let encrypted = message
        .get(FIELD_ENCRYPTED_CONTENT)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty());
    if !reasoning.is_empty() || encrypted.is_some() {
        let mut item = json!({
            "type": "reasoning",
            "id": random_id("rs"),
            "summary": if reasoning.is_empty() {
                Vec::<Value>::new()
            } else {
                vec![json!({ "type": "summary_text", "text": reasoning })]
            },
        });
        if let Some(encrypted) = encrypted {
            if let Some(object) = item.as_object_mut() {
                object.insert(
                    "encrypted_content".to_string(),
                    Value::String(encrypted.to_string()),
                );
            }
        }
        items.push(item);
    }
    // ：Responses not need need empty assistant message（ use 
    // function_call ）；empty can up emptyerror
    let text = content_text(message.get("content").unwrap_or(&Value::Null));
    if !text.is_empty() {
        items.push(json!({
            "type": "message",
            "id": random_id("msg"),
            "role": "assistant",
            "content": [{ "type": "output_text", "text": text }],
        }));
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            let call_id = {
                let raw = string_field(call, "id");
                if raw.is_empty() {
                    random_id("call")
                } else {
                    raw
                }
            };
            let name = call
                .pointer("/function/name")
                .map(string_value)
                .unwrap_or_default();
            let arguments = {
                let raw = json_text(call.pointer("/function/arguments").unwrap_or(&Value::Null));
                if raw.is_empty() {
                    "{}".to_string()
                } else {
                    raw
                }
            };
            items.push(json!({
                "type": "function_call",
                "id": random_id("fc"),
                "call_id": call_id,
                "name": name,
                "arguments": arguments,
            }));
        }
    }
}

/// → Responses `function_call_output.output`。
///
/// ── and to （ not is ）──────────────────────────
/// （`responses.rs`）**must** tool ：Chat not `tool` 
/// （OpenAI 400， `responses::PendingImages`）。 this not use ——
/// Responses schema `output` is 
/// `String | Array[ResponseInputText | ResponseInputImage | …]`， in 
/// is （DeepSeek Responses `input_image` by 
/// ）。 this is 「 JSON 」， not 。
///
/// no has when ： for up ， is in 。
fn tool_output_to_responses(output: ToolOutput) -> Value {
    if output.images.is_empty() {
        return Value::String(output.text);
    }
    let mut parts: Vec<Value> = Vec::new();
    if !output.text.is_empty() {
        parts.push(json!({ "type": "input_text", "text": output.text }));
    }
    for image in &output.images {
        let url = image_url_of(image);
        if !url.is_empty() {
            parts.push(json!({ "type": "input_image", "image_url": url }));
        }
    }
    if parts.is_empty() {
        // not to url（ file_id ）：，empty up 
        return Value::String(output.text);
    }
    Value::Array(parts)
}

/// Chat content（ or ）→ Responses input content。
///
/// for `chat_from_responses` `content_to_chat` ：
/// - （ / ） this has （user available，
/// assistant use `[{type:"output_text"}]`）；
/// - → `input_image`（url `image_url` or `{url}` ）；
/// - its （/file etc ）**** —— in chat no has ，
/// 1 up not will request 400。
fn chat_content_to_responses(content: &Value, role: &str) -> Value {
    let text_kind = if role == "assistant" { "output_text" } else { "input_text" };
    if let Some(text) = content.as_str() {
        if role == "assistant" {
            return if text.is_empty() {
                Value::String(String::new())
            } else {
                json!([{ "type": "output_text", "text": text }])
            };
        }
        return Value::String(text.to_string());
    }
    let parts = content_parts(content);
    if parts.is_empty() {
        return Value::String(String::new());
    }
    let mut out: Vec<Value> = Vec::new();
    for part in parts {
        if let Some(text) = part.as_str() {
            out.push(json!({ "type": text_kind, "text": text }));
            continue;
        }
        let kind = string_field(part, "type").to_lowercase();
        match kind.as_str() {
            "text" | "input_text" | "output_text" => {
                out.push(json!({ "type": text_kind, "text": string_field(part, "text") }));
            }
            "image_url" | "input_image" if role == "user" => {
                let url = image_url_of(part);
                if !url.is_empty() {
                    out.push(json!({ "type": "input_image", "image_url": url }));
                }
            }
            _ => {}
        }
    }
    if out.is_empty() {
        Value::String(String::new())
    } else {
        Value::Array(out)
    }
}

/// Chat → Responses （）。
///
/// `parameters` to empty for schema（`nested_from_flat` 1 ）；
/// `strict` （）。
///
/// （server）： is Responses （`natives` not ，
/// ）； its （Anthropic 、 or chat ） `natives`
/// by use —— is responses， no has for 。
fn tool_to_responses(tool: &Value, natives: &mut Vec<Value>) -> Option<Value> {
    if native_tool::is_native(tool) {
        if native_tool::origin_of(tool) == Some(native_tool::ORIGIN_RESPONSES) {
            return Some(native_tool::restore(tool));
        }
        natives.push(tool.clone());
        return None;
    }
    if !tool_plan::is_nested_function(tool) {
        // （ name/parameters）： no has can ，
        // not is 1 up 
        return None;
    }
    let function = tool.get("function")?;
    let name = string_field(function, "name");
    if name.is_empty() {
        return None;
    }
    let mut out = Map::new();
    out.insert("type".to_string(), Value::String("function".to_string()));
    out.insert("name".to_string(), Value::String(name));
    if let Some(description) = function.get("description").filter(|value| is_truthy(value)) {
        out.insert("description".to_string(), description.clone());
    }
    let parameters = function
        .get("parameters")
        .filter(|value| is_truthy(value))
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
    out.insert("parameters".to_string(), parameters);
    if let Some(strict) = function.get("strict") {
        out.insert("strict".to_string(), strict.clone());
    }
    Some(Value::Object(out))
}

/// Chat tool_choice → Responses tool_choice
/// （`{type:"function", function:{name}}` → `{type:"function", name}`；
/// ，）。
fn tool_choice_to_responses(choice: &Value) -> Value {
    if choice.is_string() {
        return choice.clone();
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
        return choice.clone();
    }
    json!({ "type": "function", "name": name })
}

/// Chat `response_format` → Responses `text.format`
/// （`{type:"json_schema", json_schema:{…}}` → `{type:"json_schema", …}`）。
fn format_to_responses(format: &Value) -> Value {
    let kind = string_field(format, "type");
    if kind != "json_schema" {
        return format.clone();
    }
    let mut out = Map::new();
    out.insert("type".to_string(), Value::String("json_schema".to_string()));
    for key in ["name", "description", "schema", "strict"] {
        if let Some(value) = format
            .pointer(&format!("/json_schema/{key}"))
            .filter(|value| !value.is_null())
        {
            out.insert(key.to_string(), value.clone());
        }
    }
    Value::Object(out)
}

// ─── response：Responses SSE → Chat SSE（）────────────────────

/// up Responses SSE → chat SSE （）。
///
/// must down has （`ForwardStream` reasoning and 
/// and usage 、 `choices[0].delta` 、 down 
/// to this ），：
/// - chat chunk is `data: {"id","object":"chat.completion.chunk",
///     "created","model","choices":[{"index":0,"delta":{…},"finish_reason":…}]}`
///   - `response.output_text.delta` → `delta.content`；
///     `response.reasoning_summary_text.delta` → `delta.reasoning_content`；
///     `response.function_call_arguments.delta` → `delta.tool_calls[…]`
/// - `response.completed` → （`finish_reason`）+ usage 
/// （`choices: []`，usage chat ）+ `data: [DONE]`
///   - `error` / `response.failed` → `data: {"error":{…}}` + `data: [DONE]`
/// （ and ForwardStream ： 502）
///
/// usage will when to （ForwardStream + usage
/// ）—— etc ， after 1 as ， not 。
pub struct ChatFromResponsesStream {
    buffer: SseLineBuffer,
    /// model（ to up ；ForwardStream will by need 
    /// clientrequest， not ）
    model: String,
    id: String,
    created: i64,
    started: bool,
    finished: bool,
    /// already ：item_id / call_id → （chat tool_calls index +
    /// is already parameter）
    tools: BTreeMap<String, ToolSlot>,
    next_tool_index: i64,
    /// up usage（Responses ； when chat ）
    usage: Option<Value>,
}

/// 1 chat （key is up to item_id / call_id）
#[derive(Default)]
struct ToolSlot {
    index: i64,
    has_arguments: bool,
}

impl ChatFromResponsesStream {
    pub fn new(model: &str) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            model: model.to_string(),
            id: String::new(),
            created: crate::server::logging::now_ms() / 1000,
            started: false,
            finished: false,
            tools: BTreeMap::new(),
            next_tool_index: 0,
            usage: None,
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

    /// up （ no has when ）
    pub fn finish(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // after 1 
        let mut out = Vec::new();
        for payload in self.buffer.finish() {
            if let Some(data) = payload {
                if let Ok(value) = serde_json::from_str::<Value>(&data) {
                    out.extend(self.consume(&value));
                }
            }
        }
        out.extend(self.complete(None));
        out
    }

    /// 1 up Responses → to chat 
    fn consume(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // usage not 1 ： in response.completed response.usage
        // ，。（ 1 as ）
        let usage = event
            .get("usage")
            .filter(|usage| usage.is_object())
            .or_else(|| event.pointer("/response/usage").filter(|usage| usage.is_object()));
        if let Some(usage) = usage {
            self.usage = Some(usage.clone());
        }
        let kind = string_field(event, "type").to_lowercase();
        match kind.as_str() {
            "response.created" => {
                if let Some(id) = event
                    .pointer("/response/id")
                    .and_then(Value::as_str)
                    .filter(|id| !id.is_empty())
                {
                    self.id = id.to_string();
                }
                self.start()
            }
            // （refusal is ，chat no has use ，
            // and —— will client to 1 empty）
            "response.output_text.delta" | "response.refusal.delta" => {
                let Some(text) = event
                    .get("delta")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                else {
                    return Vec::new();
                };
                let mut out = self.start();
                out.push(self.delta_frame(json!({ "content": text })));
                out
            }
            // → reasoning_content（chat 1 ）
            "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => {
                let Some(text) = event
                    .get("delta")
                    .and_then(Value::as_str)
                    .filter(|text| !text.is_empty())
                else {
                    return Vec::new();
                };
                let mut out = self.start();
                out.push(self.delta_frame(json!({ "reasoning_content": text })));
                out
            }
            // ： to id / name can in chat tool_calls 
            "response.output_item.added" => {
                let item = event.get("item").unwrap_or(&Value::Null);
                let item_kind = string_field(item, "type").to_lowercase();
                if !matches!(item_kind.as_str(), "function_call" | "custom_tool_call") {
                    return Vec::new();
                }
                let mut out = self.start();
                let key = tool_key(event, item);
                out.extend(self.ensure_tool(&key, item));
                out
            }
            "response.function_call_arguments.delta" => {
                let delta = string_field(event, "delta");
                if delta.is_empty() {
                    return Vec::new();
                }
                let mut out = self.start();
                let key = tool_key(event, event);
                out.extend(self.ensure_tool(&key, event));
                out.extend(self.push_arguments(&key, &delta));
                out
            }
            // by （custom） ： is ， in none can 
            // JSON （ `ResponsesStream::tool_delta_event` ），
            // by chat function.arguments； in 1 no 
            // `.done` （wrap_freeform_input）
            "response.custom_tool_call_input.delta" => {
                let delta = {
                    let raw = event.get("delta").unwrap_or(event.get("input").unwrap_or(&Value::Null));
                    json_text(raw)
                };
                if delta.is_empty() {
                    return Vec::new();
                }
                let mut out = self.start();
                let key = tool_key(event, event);
                out.extend(self.ensure_tool(&key, event));
                out.extend(self.push_arguments(&key, &delta));
                out
            }
            // parameter： 1 no when （ .done）parameter
            "response.function_call_arguments.done" => {
                let key = tool_key(event, event);
                if self.has_arguments(&key) {
                    return Vec::new();
                }
                let arguments = {
                    let raw = json_text(event.get("arguments").unwrap_or(&Value::Null));
                    if raw.is_empty() { "{}".to_string() } else { raw }
                };
                let mut out = self.start();
                out.extend(self.ensure_tool(&key, event));
                out.extend(self.push_arguments(&key, &arguments));
                out
            }
            "response.custom_tool_call_input.done" => {
                let key = tool_key(event, event);
                if self.has_arguments(&key) {
                    return Vec::new();
                }
                let input = json_text(event.get("input").unwrap_or(&Value::Null));
                let arguments = super::freeform::wrap_freeform_input(&input);
                let mut out = self.start();
                out.extend(self.ensure_tool(&key, event));
                out.extend(self.push_arguments(&key, &arguments));
                out
            }
            // ： and up .done （ can can to ）
            "response.output_item.done" => {
                let item = event.get("item").unwrap_or(&Value::Null);
                let item_kind = string_field(item, "type").to_lowercase();
                let arguments = match item_kind.as_str() {
                    "function_call" => {
                        let raw = json_text(item.get("arguments").unwrap_or(&Value::Null));
                        if raw.is_empty() { "{}".to_string() } else { raw }
                    }
                    "custom_tool_call" => {
                        super::freeform::wrap_freeform_input(&json_text(item.get("input").unwrap_or(&Value::Null)))
                    }
                    _ => return Vec::new(),
                };
                let key = tool_key(event, item);
                if self.has_arguments(&key) {
                    return Vec::new();
                }
                let mut out = self.start();
                out.extend(self.ensure_tool(&key, item));
                out.extend(self.push_arguments(&key, &arguments));
                out
            }
            // up error（ failure）： chat error + [DONE]
            "error" | "response.failed" => self.fail(event),
            // ：（usage already in ）
            "response.completed" | "response.done" => self.complete(None),
            "response.incomplete" => {
                let reason = event
                    .pointer("/response/incomplete_details/reason")
                    .map(string_value)
                    .unwrap_or_default();
                let reason = if reason.is_empty() { "length".to_string() } else { reason };
                self.complete(Some(&reason))
            }
            _ => Vec::new(),
        }
    }

    /// （role assistant）： and bridge —— emptyresponse or has 
    /// can assistant （ down by delta when not will to 「none」 ）
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

    /// 1 （ if ）：chat tool_calls id / name / emptyparameter
    fn ensure_tool(&mut self, key: &str, item: &Value) -> Vec<bytes::Bytes> {
        if self.tools.contains_key(key) {
            return Vec::new();
        }
        let index = self.next_tool_index;
        self.next_tool_index += 1;
        self.tools.insert(key.to_string(), ToolSlot { index, has_arguments: false });
        let call_id = {
            let raw = string_field(item, "call_id");
            if raw.is_empty() {
                let id = string_field(item, "id");
                if id.is_empty() { random_id("call") } else { id }
            } else {
                raw
            }
        };
        let name = string_field(item, "name");
        vec![self.delta_frame(json!({
            "tool_calls": [{
                "index": index,
                "id": call_id,
                "type": "function",
                "function": { "name": name, "arguments": "" },
            }],
        }))]
    }

    /// 1 parameter（ not in when —— no no has index can ）
    fn push_arguments(&mut self, key: &str, delta: &str) -> Vec<bytes::Bytes> {
        let Some(slot) = self.tools.get_mut(key) else {
            return Vec::new();
        };
        slot.has_arguments = true;
        let index = slot.index;
        vec![self.delta_frame(json!({
            "tool_calls": [{ "index": index, "function": { "arguments": delta } }],
        }))]
    }

    fn has_arguments(&self, key: &str) -> bool {
        self.tools.get(key).map(|slot| slot.has_arguments).unwrap_or(false)
    }

    /// ：finish_reason + usage + [DONE]（ 1 ）
    fn complete(&mut self, incomplete_reason: Option<&str>) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;
        let mut out = self.start();
        let finish_reason = match incomplete_reason {
            Some(reason) => reason.to_string(),
            // Responses no has chat finish_reason， by ：
            // has use → tool_calls；otherwise stop（length/content_filter
            // by response.incomplete reason to ）
            None => {
                if self.tools.is_empty() {
                    "stop".to_string()
                } else {
                    "tool_calls".to_string()
                }
            }
        };
        out.push(self.finish_frame(&finish_reason));
        if let Some(usage) = self.usage.take() {
            // usage （choices empty）：ForwardStream and 
            // from this 1 （ for include_usage ）
            out.push(chat_frame(&json!({
                "id": self.id,
                "object": "chat.completion.chunk",
                "created": self.created,
                "model": self.model,
                "choices": [],
                "usage": chat_usage_from_responses(&usage),
            })));
        }
        out.push(bytes::Bytes::from_static(b"data: [DONE]\n\n"));
        out
    }

    /// up error → chat error + [DONE]（ 1 ）
    fn fail(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;
        let error = event
            .get("error")
            .filter(|error| is_truthy(error))
            .or_else(|| event.pointer("/response/error").filter(|error| is_truthy(error)));
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
        vec![
            chat_frame(&json!({
                "error": { "message": message, "type": "upstream_error" },
            })),
            bytes::Bytes::from_static(b"data: [DONE]\n\n"),
        ]
    }

    /// 1 delta （`choices[0].delta = delta`，finish_reason as null）
    fn delta_frame(&self, delta: Value) -> bytes::Bytes {
        chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": Value::Null }],
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

/// / ：call_id ， to id / item_id / output_index。
///
/// 1 use （added / delta / done）must to 1 ，
/// up this in 1 （ and by this ）。
fn tool_key(event: &Value, item: &Value) -> String {
    for text in [
        string_field(item, "call_id"),
        string_field(item, "id"),
        string_field(event, "item_id"),
    ] {
        if !text.is_empty() {
            return text;
        }
    }
    event.get("output_index").map(string_value).unwrap_or_default()
}

/// Responses usage → chat usage（ and by chat ）。
///
/// and `responses.rs::usage_to_responses` as ， for ：
/// `input_tokens`→`prompt_tokens`、`output_tokens`→`completion_tokens`、
/// `input_tokens_details.cached_tokens`→`prompt_tokens_details.cached_tokens`、
/// `output_tokens_details.reasoning_tokens`→`completion_tokens_details.reasoning_tokens`。
fn chat_usage_from_responses(usage: &Value) -> Value {
    let input = json_number_of(usage, &["input_tokens", "prompt_tokens"]);
    let output = json_number_of(usage, &["output_tokens", "completion_tokens"]);
    let cached = usage
        .pointer("/input_tokens_details/cached_tokens")
        .and_then(Value::as_i64)
        .filter(|value| *value != 0)
        .unwrap_or_else(|| json_number_of(usage, &["cache_read_input_tokens"]));
    let reasoning = usage
        .pointer("/output_tokens_details/reasoning_tokens")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let mut out = Map::new();
    out.insert("prompt_tokens".to_string(), Value::from(input));
    if cached != 0 {
        out.insert(
            "prompt_tokens_details".to_string(),
            json!({ "cached_tokens": cached }),
        );
    }
    out.insert("completion_tokens".to_string(), Value::from(output));
    if reasoning != 0 {
        out.insert(
            "completion_tokens_details".to_string(),
            json!({ "reasoning_tokens": reasoning }),
        );
    }
    out.insert(
        "total_tokens".to_string(),
        Value::from(json_number_of(usage, &["total_tokens"]).max(input + output)),
    );
    Value::Object(out)
}
