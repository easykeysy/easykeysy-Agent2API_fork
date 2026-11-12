//! Responses API（`POST /v1/responses`）↔ Chat Completions to 。
//!
//! ── to ────────────────────────────────────────────────
//! all is `X_from_chat` / `chat_from_X` ：
//! - `*_from_chat`： ** down ** Responses request/response Chat
//! - `chat_from_*`： ** up ** Chat response Responses
//! and （`responsesRequestToChat` ） not ： that 「A to B」
//! in to to ， to is this bug。
//!
//! ── supports（ has ， in this ）──────────────
//! Responses has 1 ** has **（`previous_response_id` / `conversation` /
//! `prompt` / `background`）—— server， is none 
//! （ up is ， no has Responses server）。
//! this ** 400** not is ignore：ignore will client as up down 
//! ， 1 in ，（「」） to 。
//! this as 「none」。
//!
//! `store` down `false`： up not ，client「」 is 。
//!
//! ── ────────────────────────────────────────
//! Chat is `reasoning_content`（），Responses is `output` 
//! `{type:"reasoning", summary:[{type:"summary_text", text}]}` 。
//! （Responses → Chat） when reasoning `reasoning_content`。

use serde_json::{json, Map, Value};

use super::{
    content_parts, content_text, event_frame, freeform, is_truthy, json_text, native_tool,
    random_id, string_field, string_value, tool_plan, SseLineBuffer, TOOL_IMAGE_PLACEHOLDER,
    FIELD_ENCRYPTED_CONTENT,
};
use crate::server::logging;

/// none when error（`Err` ）。
///
/// Why is 「」 not is error： use （`api::protocol`） 
/// 400 response， not need need HTTP —— 。
pub type ConvertError = String;

/// has ：none， this 1 （ by module header）
const STATEFUL_FIELDS: [&str; 4] =
    ["previous_response_id", "conversation", "prompt", "background"];

// ─── request：Responses → Chat ─────────────────────────────────

/// Responses request → Chat Completions request。
///
/// `model` by use already （ by in handler 1 ， `api::chat`），
/// this 。
pub fn chat_from_responses(body: &Value) -> Result<Value, ConvertError> {
    for field in STATEFUL_FIELDS {
        if body.get(field).map(is_truthy).unwrap_or(false) {
            return Err(format!(
                "字段 {field} 需要服务端保存对话状态，本网关是无状态转发，无法支持"
            ));
        }
    }
    let mut out = Map::new();
    out.insert("model".to_string(), body.get("model").cloned().unwrap_or(Value::Null));

    let messages = messages_from_input(body.get("instructions"), body.get("input"))?;
    out.insert("messages".to_string(), Value::Array(messages));
    out.insert(
        "stream".to_string(),
        Value::Bool(body.get("stream").and_then(Value::as_bool).unwrap_or(false)),
    );

    // max_output_tokens → max_completion_tokens（Responses ）
    if let Some(value) = body.get("max_output_tokens").filter(|value| is_truthy(value)) {
        out.insert("max_completion_tokens".to_string(), value.clone());
    }
    for key in ["temperature", "top_p", "service_tier", "parallel_tool_calls", "user", "metadata"] {
        if let Some(value) = body.get(key) {
            if !value.is_null() {
                out.insert(key.to_string(), value.clone());
            }
        }
    }
    // ： + （namespace 、custom ）。 need ——
    // `tools`， and `input[]` `type:"additional_tools"` （Codex 
    // Responses Lite path in input 、 `tools` as null）。
    // `tool_plan` module header： is 2026-09「Codex not 」 。
    let plan = tool_plan::plan_tools(body);
    if !plan.declarations.is_empty() {
        let mut converted: Vec<Value> = Vec::new();
        let mut downgraded: Vec<String> = Vec::new();
        let mut natives: Vec<Value> = Vec::new();
        for tool in &plan.declarations {
            if let Some(function) = tool_to_chat(tool) {
                converted.push(function);
                continue;
            }
            // no 。custom（freeform） is ** need ** —— is 
            // （Codex exec / apply_patch by ）；
            // its （web_search / tool_search / file_search etc Responses ）
            // is ** up server** can ：（ + ），
            // to by （ `native_tool` module header）。
            // in this —— issue #61 / #55 「unavailable」 by 。
            if freeform::is_custom_tool(tool) {
                if let Some(function) = freeform::downgrade_custom_tool(tool) {
                    downgraded.push(string_field(tool, "name"));
                    converted.push(function);
                    continue;
                }
            }
            natives.push(native_tool::carry(tool, native_tool::ORIGIN_RESPONSES));
        }
        if !natives.is_empty() {
            logging::verbose(
                "[Responses]",
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
        if !downgraded.is_empty() {
            logging::verbose(
                "[Responses]",
                &format!(
                    "自由文本（custom）工具已降级为 function：{}",
                    downgraded.join(", ")
                ),
            );
        }
        if !converted.is_empty() {
            logging::verbose(
                "[Responses]",
                &format!(
                    "工具声明 {} 条（含命名空间展开）→ 已转发 {} 条",
                    plan.declarations.len(),
                    converted.len()
                ),
            );
            out.insert("tools".to_string(), Value::Array(converted));
        }
    }
    if let Some(choice) = body.get("tool_choice").filter(|value| is_truthy(value)) {
        out.insert("tool_choice".to_string(), tool_choice_to_chat(choice));
    }
    // text.format → response_format
    if let Some(format) = body.pointer("/text/format").filter(|value| is_truthy(value)) {
        out.insert("response_format".to_string(), format_to_chat(format));
    }
    // reasoning.effort → reasoning_effort（ up this Chat ）
    if let Some(effort) = body.pointer("/reasoning/effort").filter(|value| has_effort(value)) {
        out.insert("reasoning_effort".to_string(), effort.clone());
    }
    Ok(Value::Object(out))
}

fn has_effort(value: &Value) -> bool {
    !string_value(value).trim().is_empty()
}

/// `instructions` + `input` → Chat `messages` 。
///
/// `input` has ， need ：
/// - （ use ）
/// - （`{type:"message", role, content}` or `{role, content}`）
/// - （`input_text` / `input_image` / …， as 1 user ）
/// - up 
fn messages_from_input(
    instructions: Option<&Value>,
    input: Option<&Value>,
) -> Result<Vec<Value>, ConvertError> {
    let mut messages: Vec<Value> = Vec::new();
    if let Some(text) = instructions.and_then(Value::as_str).filter(|text| !text.trim().is_empty()) {
        messages.push(json!({ "role": "system", "content": text }));
    }
    let Some(input) = input else {
        return Ok(messages);
    };
    if let Some(text) = input.as_str() {
        messages.push(json!({ "role": "user", "content": text }));
        return Ok(messages);
    }
    let Some(items) = input.as_array() else {
        // for （）：client to 1 message 
        if input.is_object() {
            let mut pending = PendingReasoning::default();
            let mut images = PendingImages::default();
            push_input_item(&mut messages, input, &mut pending, &mut images)?;
            images.flush(&mut messages);
            merge_successive_assistants(&mut messages);
        }
        return Ok(messages);
    };
    // ：reasoning need to ** that ** assistant up ，
    // in `input[]` is （ `PendingReasoning` ）；
    // need （ `PendingImages`）
    let mut pending = PendingReasoning::default();
    let mut images = PendingImages::default();
    for item in items {
        if let Some(text) = item.as_str() {
            // is client in ： need ， not will and 
            images.flush(&mut messages);
            messages.push(json!({ "role": "user", "content": text }));
            continue;
        }
        push_input_item(&mut messages, item, &mut pending, &mut images)?;
    }
    // ： after 1 after no has 「 down 1 」 down ， this 1 
    // （client request in up 、 etc ， is ）
    images.flush(&mut messages);
    merge_successive_assistants(&mut messages);
    Ok(messages)
}

/// ** assistant and 1 **。
///
/// ── Whymust and （ this is 400 Reason）────────────────────
/// DeepSeek assistant ，：
/// `does not support successive user or assistant messages … You should
/// interleave the user/assistant messages in the message sequence.`
///
/// Codex Responses when ** 1 can can assistant **： 
/// 「 1 」 and 「 use 」 
/// （`reasoning → message → custom_tool_call`）， after is 
/// assistant 。：43 request， 1 assistant that 
/// is 1 1 400； 1 **user** has 1~7 for success ——
/// can is 「 assistant」 this ， and reasoning none。
///
/// ── and then ────────────────────────────────────────────────
/// after 1 and before 1 ， by and ：
/// - `content`：****（ before 1 is 、 after 1 is ，
/// is ， not ）；
/// - `tool_calls`：****（ and / use ， up by ）；
/// - `reasoning_content`： to （ 1 this 1 ，
/// has when to as ， and [`PendingReasoning`] 1 ）。
///
/// and is DeepSeek —— is Codex 
/// （`content` and `tool_calls` in 1 assistant up ）。
fn merge_successive_assistants(messages: &mut Vec<Value>) {
    let mut out: Vec<Value> = Vec::with_capacity(messages.len());
    for message in messages.drain(..) {
        let can_merge = message.get("role").and_then(Value::as_str) == Some("assistant")
            && out
                .last()
                .and_then(|last| last.get("role"))
                .and_then(Value::as_str)
                == Some("assistant");
        if !can_merge {
            out.push(message);
            continue;
        }
        let Some(previous) = out.last_mut().and_then(Value::as_object_mut) else {
            out.push(message);
            continue;
        };
        let Some(incoming) = message.as_object() else {
            continue;
        };
        merge_assistant_fields(previous, incoming);
    }
    *messages = out;
}

/// `incoming` and `previous`（ [`merge_successive_assistants`]）
fn merge_assistant_fields(previous: &mut Map<String, Value>, incoming: &Map<String, Value>) {
    // content：。empty/Null as 「 no has this 」
    let merged_text = {
        let a = text_of(previous.get("content"));
        let b = text_of(incoming.get("content"));
        match (a.is_empty(), b.is_empty()) {
            (true, true) => None,
            (false, true) => Some(a),
            (true, false) => Some(b),
            (false, false) => Some(format!("{a}\n{b}")),
        }
    };
    match merged_text {
        Some(text) => {
            previous.insert("content".to_string(), Value::String(text));
        }
        None => {
            // no has ： Null（ tool_calls assistant is this ）
            if !previous.contains_key("content") {
                previous.insert("content".to_string(), Value::Null);
            }
        }
    }
    // tool_calls：
    let incoming_calls = incoming.get("tool_calls").and_then(Value::as_array);
    if let Some(calls) = incoming_calls.filter(|calls| !calls.is_empty()) {
        match previous.get_mut("tool_calls").and_then(Value::as_array_mut) {
            Some(existing) => existing.extend(calls.iter().cloned()),
            None => {
                previous.insert("tool_calls".to_string(), Value::Array(calls.clone()));
            }
        }
    }
    // reasoning_content： to （`entry` ： already has not ）
    if let Some(reasoning) = incoming.get("reasoning_content").cloned() {
        previous
            .entry("reasoning_content".to_string())
            .or_insert(reasoning);
    }
}

/// `content` （ use ； its to `content_text`）
fn text_of(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => text.clone(),
        Some(other) => content_text(other),
        None => String::new(),
    }
}

/// etc reasoning （ input ）。
///
/// ── Why need need ──────────────────────────────────────
/// Responses reasoning is **** function_call ，：
///
/// ```text
/// input: [ … , {type:"reasoning", summary:[…]}, {type:"function_call", …} , … ]
/// ```
///
/// Chat no has 「 」 this —— can as `reasoning_content`
/// in assistant up 。 is must this ** and ** 1 。
///
/// ── Whymust and （ not and is 400）────────────────────────
/// DeepSeek etc thinking need ：request `tools` when ， 
/// assistant `reasoning_content` need ，return
/// `400 the reasoning content from the previous turn must be passed back in
/// thinking mode`。 reasoning ， is 1 use 
/// reasoning —— Codex Responses when in （）。
///
/// ── 1 **** assistant need （ not is use that ）──────
/// 1 in `input[]` can can is ****，Codex ：
///
/// ```text
/// [1] reasoning ← this 1 （ has 1 ）
/// [2] message assistant ' 1 down 。' ← 
/// [3] custom_tool_call ← use 
/// ```
///
/// this ：**`[2]` and `[3]` need 1 reasoning**。 up 「 
/// assistant 」 1 for ， to `[3]` 、 `[2]` empty will 
/// `reasoning_content_missing`（：43 request 1 「 assistant」
/// that is 1 1 400）。
///
/// `take()` in **** assistant up will use ， and ** after not empty** ——
/// after assistant need use 1 。empty when is 「」，
/// by [`PendingReasoning::end_turn`] ， is ** down 1 user **。
///
/// ── Why「 to after 」 ────────────────────────────────────
/// Responses is reasoning in before 、 for message /
/// function_call in after （`response.output[]` ，client），
/// need need 「 → after assistant 」 this 1 to 。
///
/// to （assistant to 、reasoning after to ）** not **： can to 
/// 「 after 1 」assistant up ， that 1 can can is ** up 1 ** （e.g.
/// `… function_call → function_call_output → reasoning( down 1 ) → …`），
/// —— up will as this 1 。
///
/// 1 no has assistant can when ****（ when empty）： to user / tool
/// up will for 。
#[derive(Default)]
struct PendingReasoning {
    /// （ or that 1 ） reasoning 
    text: Option<String>,
    /// reasoning up `encrypted_content`（store=false 
    /// ，Codex / Grok CLI ）。 and ： by has 
    /// 「 1 assistant 」，** 1 **（9Router ——
    /// 1 in up ， up will ）。
    encrypted: Option<String>,
    /// this is already to up 。
    ///
    /// use 「 1 reasoning 」 and 「 down 1 1 reasoning 」：
    /// before this and already has ****， after this ****。
    /// is 「 in has no has assistant 」—— up 1 
    /// up 1 （`reasoning → tool_call → reasoning` this ， reasoning
    /// ， in 1 will up 1 to this 1 ， and 1 error）。
    attached: bool,
}

impl PendingReasoning {
    /// down 1 reasoning。
    ///
    /// up 1 ** already ** when （ new 1 ）；otherwise（ 1 
    /// reasoning ）。
    fn stash(&mut self, text: String) {
        if self.attached {
            self.text = Some(text);
            self.attached = false;
            return;
        }
        match self.text.as_mut() {
            Some(existing) => {
                existing.push('\n');
                existing.push_str(&text);
            }
            None => self.text = Some(text),
        }
    }

    /// down reasoning （ new reasoning to when old ；
    /// not `attached` —— by that 1 ）
    fn stash_encrypted(&mut self, value: &Value) {
        if let Some(encrypted) = value.as_str().filter(|text| !text.is_empty()) {
            self.encrypted = Some(encrypted.to_string());
        }
    }

    /// （****： 1 assistant ）
    fn take_encrypted(&mut self) -> Option<String> {
        self.encrypted.take()
    }

    /// need to assistant up reasoning（** not empty** —— 1 
    /// assistant need 1 ，）， when down 「 already 」
    fn attach(&mut self) -> Option<String> {
        let text = self.text.clone();
        if text.is_some() {
            self.attached = true;
        }
        text
    }

    /// （ to new user / system ）： reasoning
    fn end_turn(&mut self) {
        self.text = None;
        self.encrypted = None;
        self.attached = false;
    }
}

/// etc （ input ）。
///
/// ── Why not can in tool ───────────────────────────
/// Chat `tool` content **** part，OpenAI ：
/// `Invalid 'messages'. Image URLs are only allowed for messages with role
/// 'user', but this message with role 'tool' contains an image URL.`
/// 400。 can to 1 user up （ this is OpenAI
/// to ）， in tool etc 「」「requestfailure」。
///
/// ── Why need to after ─────────────────────────────
/// 1 is **** tool ， Chat need assistant 
/// `tool_calls` for tool ： in ， after 
/// tool_call_id （ up 11148 "tool calls and tool results
/// do not match"， `anthropic.rs` 1 ）。 and use 
/// （assistant 1 tool_calls， after tool ） in ，
/// to this 1 after 1 。
///
/// ── Why not [`PendingReasoning`] ──────────────────────────
/// this user is ， not is client to 。 if 
/// `end_turn()`， will ** already 、 need to down 1 assistant reasoning**
/// （：`reasoning → function_call`， flush in function_call
/// before ）， up old `reasoning_content_missing`。
#[derive(Default)]
struct PendingImages {
    /// already 1 Chat `image_url` 
    parts: Vec<Value>,
}

impl PendingImages {
    fn push(&mut self, part: Value) {
        self.parts.push(part);
    }

    /// down 1 user （ no has when not ）
    fn flush(&mut self, messages: &mut Vec<Value>) {
        if self.parts.is_empty() {
            return;
        }
        messages.push(json!({
            "role": "user",
            "content": Value::Array(std::mem::take(&mut self.parts)),
        }));
    }
}

/// input → to 1 Chat （ `messages`）
///
/// `pending` is reasoning （ [`PendingReasoning`]）：reasoning 
/// not ， to assistant 。
/// `images` is （ [`PendingImages`]）： not can in tool
/// ， need to after 1 user 。
fn push_input_item(
    messages: &mut Vec<Value>,
    item: &Value,
    pending: &mut PendingReasoning,
    images: &mut PendingImages,
) -> Result<(), ConvertError> {
    let kind = string_field(item, "type").to_lowercase();
    // ， is 「 this 1 to as 」 down 
    if kind != "function_call_output" && kind != "custom_tool_call_output" {
        images.flush(messages);
    }
    match kind.as_str() {
        // use and its ： assistant(tool_calls) + tool 。
        // up need tool must for assistant， this 1 。
        "function_call" => {
            let call_id = call_id_of(item);
            // empty ， in `name`、 in `namespace` ；
            // to up must is after ，otherwise to before after not 1 
            let name = tool_plan::flatten_call_name(item);
            let arguments = {
                let raw = json_text(item.get("arguments").unwrap_or(&Value::Null));
                if raw.is_empty() { "{}".to_string() } else { raw }
            };
            let mut call_message = tool_call_message(
                &call_id,
                &name,
                arguments,
                pending.attach(),
            );
            // 1 assistant（ take_encrypted ）
            if let Some(encrypted) = pending.take_encrypted() {
                if let Some(object) = call_message.as_object_mut() {
                    object.insert(FIELD_ENCRYPTED_CONTENT.to_string(), Value::String(encrypted));
                }
            }
            messages.push(call_message);
        }
        // ： tool ，（ `PendingImages`）。
        // ， is `output`， 1 
        "function_call_output" | "custom_tool_call_output" => {
            let output = tool_output_parts(item.get("output"));
            for image in output.images {
                images.push(json!({
                    "type": "image_url",
                    "image_url": { "url": image_url_of(&image) },
                }));
            }
            messages.push(json!({
                "role": "tool",
                "tool_call_id": call_id_of(item),
                // has when `text` is empty， by [`TOOL_IMAGE_PLACEHOLDER`]
                "content": if output.text.is_empty() {
                    TOOL_IMAGE_PLACEHOLDER.to_string()
                } else {
                    output.text
                },
            }));
        }
        // by （custom） use and ： and function_call ， is 
        // 「parameter」 in `input` not is `arguments`。must 1 for ，otherwise
        // not to up 1 —— use success， will 
        // empty。
        "custom_tool_call" => {
            let name = tool_plan::flatten_call_name(item);
            // need ：client is `input` ；
            // if （ already ），id 、 can can is 
            // `arguments`。 to use ， not 
            let raw = {
                let input = string_field(item, "input");
                if input.is_empty() {
                    json_text(item.get("arguments").unwrap_or(&Value::Null))
                } else {
                    input
                }
            };
            messages.push(tool_call_message(
                &call_id_of(item),
                &name,
                // by when `{"input": "…"}`， and 1 
                freeform::wrap_freeform_input(&raw),
                pending.attach(),
            ));
        }
        // reasoning ： not 1 for ，** not can ** —— need to 
        // assistant up （ [`PendingReasoning`]）。 this ，
        // by after message / function_call / custom_tool_call 
        "reasoning" => {
            if let Some(text) = reasoning_text_of(item) {
                pending.stash(text);
            }
            // （store=false ，Codex / Grok CLI），
            // by （responses_outbound） to to Responses up 
            // reasoning ； its strip （ up will ）
            pending.stash_encrypted(item.get("encrypted_content").unwrap_or(&Value::Null));
        }
        "input_text" | "text" => {
            messages.push(json!({ "role": "user", "content": string_field(item, "text") }));
            pending.end_turn();
        }
        "input_image" | "input_file" | "image_url" => {
            // （ not in message ）： 1 user 
            messages.push(json!({ "role": "user", "content": [item.clone()] }));
            pending.end_turn();
        }
        // ： not is for ， already in `tool_plan::plan_tools` 
        // request。 this —— to down will 1 
        // 「 will 1 」（ not this in for ）。
        // Note this must： use will ， from **all**
        // （ in has after when unreachable pattern）
        "additional_tools" => {}
        // （ type / as "message" ）
        _ => {
            let role = {
                let raw = string_field(item, "role").to_lowercase();
                if raw == "developer" { "system".to_string() } else if raw.is_empty() { "user".to_string() } else { raw }
            };
            let content = item.get("content").or_else(|| item.get("text"));
            let Some(content) = content else {
                // not in this 。 return， not not ——
                // 1 is new （ local_shell_call）， is 「
                // its 」，none from 。 can 1 ， need down 。
                let unknown = string_field(item, "type");
                if !unknown.is_empty() && unknown != "message" {
                    logging::log(
                        "[Responses]",
                        &format!("⚠️ 输入项类型 {unknown} 无法转换，已跳过（内容会缺一段）"),
                    );
                }
                return Ok(());
            };
            let content = content_to_chat(content, &role);
            // assistant empty need ： up for 「empty assistant」 not 1 ，
            // Responses reasoning-only will empty
            if role == "assistant"
                && content_text(&content).is_empty()
                && !content_is_structured(&content)
            {
                return Ok(());
            }
            let mut message = Map::new();
            message.insert("role".to_string(), Value::String(role.clone()));
            message.insert("content".to_string(), content.clone());
            // assistant reasoning has ，** 、 **：
            // - `summary`：client in 1 when use this ；
            // - （`PendingReasoning`）：Codex reasoning as **
            // **， when to assistant up 。
            //
            // this ** `summary`、 not `content`** —— message `content`
            // is ****（`{type:"output_text", text:…}`）， will 
            // `reasoning_content` 。 that is bug。
            //
            // assistant ** need ** reasoning（ not is to use 
            // that ）：Codex 1 can can 、 use ， is ** 
            // assistant **， up by for ， 1 
            // `reasoning_content_missing`（， [`PendingReasoning`]）。
            if role == "assistant" {
                let reasoning = summary_text_of(item).or_else(|| pending.attach());
                if let Some(reasoning) = reasoning {
                    message.insert("reasoning_content".to_string(), Value::String(reasoning));
                }
                // 1 assistant（ take_encrypted ）
                if let Some(encrypted) = pending.take_encrypted() {
                    message.insert(FIELD_ENCRYPTED_CONTENT.to_string(), Value::String(encrypted));
                }
            } else if role != "tool" {
                // assistant、 tool = up 1 ： reasoning，
                // to down 1 assistant up （ will 400）。
                // `tool` ** not ** —— 1 after has 
                // assistant （ that is Codex ），reasoning need use 。
                pending.end_turn();
            }
            messages.push(Value::Object(message));
        }
    }
    Ok(())
}

/// is is （ and ）——used for「empty」 when not can 
fn content_is_structured(content: &Value) -> bool {
    content_parts(content).iter().any(|part| {
        let kind = string_field(part, "type").to_lowercase();
        !matches!(kind.as_str(), "" | "text" | "input_text" | "output_text")
    })
}

/// use → Chat assistant(tool_calls) 。
///
/// `reasoning` is reasoning （ [`PendingReasoning`]）： up 
/// can thinking 「 tools when reasoning_content must」 need 。
/// as None when not this —— empty and not is ， up for empty not 1 。
fn tool_call_message(
    call_id: &str,
    name: &str,
    arguments: String,
    reasoning: Option<String>,
) -> Value {
    let mut message = Map::new();
    message.insert("role".to_string(), Value::String("assistant".to_string()));
    message.insert("content".to_string(), Value::Null);
    message.insert(
        "tool_calls".to_string(),
        json!([{
            "id": call_id,
            "type": "function",
            "function": { "name": name, "arguments": arguments },
        }]),
    );
    if let Some(reasoning) = reasoning {
        message.insert("reasoning_content".to_string(), Value::String(reasoning));
    }
    Value::Object(message)
}

/// ** 1 `reasoning` ** （`summary` ， to `content`）。
///
/// to `reasoning` use —— that `content` is 。
/// ** not can `message` **：message `content` is for ，
/// will `reasoning_content` （ [`summary_text_of`]）。
fn reasoning_text_of(item: &Value) -> Option<String> {
    summary_text_of(item).or_else(|| text_blocks_of(item.get("content")))
}

/// **`message` ** ： `summary`， not to `content`。
///
/// ── Why this is required （）────────────────────────
/// 「」 and 「」 not ：
/// - `message` ：`content: [{type:"output_text", text:""}]`，none `summary`；
/// - `reasoning` ：`summary: [{type:"summary_text", text:""}]`。
///
/// this use 1 「summary 、 to content」 ， is **
/// assistant **（message ） ，
/// `reasoning_content` and `content` 1 1 。 after not is 「 1 none 
/// 」：DeepSeek thinking by for reasoning， this 1 
/// ， that need need reasoning use not to 
/// （reasoning in before to ， ）， up 
/// `400 reasoning_content_missing`。
fn summary_text_of(item: &Value) -> Option<String> {
    text_blocks_of(item.get("summary"))
}

/// 1 `text` （empty / / empty to None）
fn text_blocks_of(blocks: Option<&Value>) -> Option<String> {
    let parts = blocks.and_then(Value::as_array)?;
    let text: String = parts
        .iter()
        .map(|part| string_field(part, "text"))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() { None } else { Some(text) }
}

/// （ [`tool_output_parts`]）
pub(super) struct ToolOutput {
    /// （ can can as empty —— has when by use ）
    pub(super) text: String,
    /// （， use by ；url not to already ）
    pub(super) images: Vec<Value>,
}

/// → + 。
///
/// ── Whymust（，issue #76）──────────────────────
/// Codex `view_image` in `function_call_output.output` ****：
/// `[{type:"input_image", image_url:"data:image/png;base64,…"}]`
/// （openai/codex `FunctionCallOutputContentItem`；OpenAI schema 
/// `output` as `String | Array[ResponseInputText | ResponseInputImage | …]`）。
/// use `json_text` JSON **** tool ， after ：
///
/// 1. up base64 —— token ≈ × 0.69， 1 400KB 
/// token， can 1M up down （ 325k → 973k）；
/// 2. is ** not to **， to 1 base64 —— will up down 1 
/// ，`view_image` 。
///
/// must（Chat [`PendingImages`]）， old 。
///
/// `pub(super)`： to （`responses_outbound`） chat tool 
/// `function_call_output` when use 1 ， 1 。
pub(super) fn tool_output_parts(output: Option<&Value>) -> ToolOutput {
    let Some(output) = output else {
        return text_only("(empty)".to_string());
    };
    match output {
        Value::String(text) => {
            if text.is_empty() { text_only("(empty)".to_string()) } else { text_only(text.clone()) }
        }
        Value::Null => text_only("(empty)".to_string()),
        Value::Array(parts) => {
            let images: Vec<Value> = parts
                .iter()
                .filter(|part| is_image_part(part) && !image_url_of(part).is_empty())
                .cloned()
                .collect();
            let text = content_text(output);
            if text.is_empty() && images.is_empty() {
                // 1 not （e.g. has ）： ，
                // empty
                return text_only(json_text(output));
            }
            report_unknown_parts(parts);
            ToolOutput { text, images }
        }
        // （ for / etc ）：Chat no has ， old 
        other => text_only(json_text(other)),
    }
}

fn text_only(text: String) -> ToolOutput {
    ToolOutput { text, images: Vec::new() }
}

/// is not is （Responses `input_image` and Chat `image_url` ）
fn is_image_part(part: &Value) -> bool {
    matches!(
        string_field(part, "type").to_lowercase().as_str(),
        "input_image" | "image_url"
    )
}

/// 1 「 has not 」。
///
/// and not 1 to （ `push_input_item` ）： can 1 ，
/// need 「 its 」 has can 。
fn report_unknown_parts(parts: &[Value]) {
    let unknown: Vec<String> = parts
        .iter()
        .filter(|part| {
            !part.is_string()
                && !matches!(
                    string_field(part, "type").to_lowercase().as_str(),
                    "text" | "input_text" | "output_text" | "input_image" | "image_url"
                )
        })
        .map(|part| {
            let kind = string_field(part, "type");
            if kind.is_empty() { "(缺 type)".to_string() } else { kind }
        })
        .collect();
    if !unknown.is_empty() {
        logging::log(
            "[Responses]",
            &format!("⚠️ 工具输出里的 {} 块跨协议表达不了，已丢弃", unknown.join(" / ")),
        );
    }
}

/// use id：`call_id` ， to `id`， no has then 1 
fn call_id_of(item: &Value) -> String {
    let call_id = string_field(item, "call_id");
    if !call_id.is_empty() {
        return call_id;
    }
    let id = string_field(item, "id");
    if !id.is_empty() {
        return id;
    }
    random_id("call")
}

/// Responses → Chat content（ or ）。
///
/// when ****：Chat content ， for up 
/// （ up for not ）， and 「client 1 」 etc 。
fn content_to_chat(content: &Value, role: &str) -> Value {
    if let Some(text) = content.as_str() {
        return Value::String(text.to_string());
    }
    let parts = content_parts(content);
    if parts.is_empty() {
        return Value::String(String::new());
    }
    let mut out: Vec<Value> = Vec::new();
    let mut only_text = true;
    for part in parts {
        if let Some(text) = part.as_str() {
            out.push(json!({ "type": "text", "text": text }));
            continue;
        }
        let kind = string_field(part, "type").to_lowercase();
        match kind.as_str() {
            "text" | "input_text" | "output_text" => {
                out.push(json!({ "type": "text", "text": string_field(part, "text") }));
            }
            "input_image" | "image_url" => {
                only_text = false;
                let url = image_url_of(part);
                out.push(json!({ "type": "image_url", "image_url": { "url": url } }));
            }
            // file//：Chat no has 1 ， will to 
            // 1 none JSON —— this ， by up not 
            _ => {
                only_text = false;
                out.push(part.clone());
            }
        }
    }
    // has 1 when （ and up by ）
    if only_text && out.len() == 1 {
        if let Some(text) = out[0].get("text").and_then(Value::as_str) {
            return Value::String(text.to_string());
        }
    }
    let _ = role;
    Value::Array(out)
}

/// url（`image_url` can is or `{url}` for ）
///
/// `pub(super)`： to （`responses_outbound`） chat image_url 
/// `input_image` when is 1 ， use 1 。
pub(super) fn image_url_of(part: &Value) -> String {
    let source = part.get("image_url").unwrap_or(part);
    match source {
        Value::String(url) => url.clone(),
        other => string_field(other, "url"),
    }
}

/// Responses → Chat （ → `function`）
fn tool_to_chat(tool: &Value) -> Option<Value> {
    // （`tools: ["web_search"]`）： not is function，
    if tool.is_string() {
        return None;
    }
    // already （client use ）：
    if tool_plan::is_nested_function(tool) {
        return Some(tool.clone());
    }
    let kind = string_field(tool, "type").to_lowercase();
    if kind != "function" && !kind.is_empty() {
        // function ： up Chat not ， not is up 
        // —— client need is 「 can 」， not is 「」。
        // custom（freeform） in this ， use will 
        // （ `chat_from_responses` ）， not will 。
        return None;
    }
    tool_plan::nested_from_flat(tool)
}

/// Responses tool_choice → Chat tool_choice
fn tool_choice_to_chat(choice: &Value) -> Value {
    if let Some(text) = choice.as_str() {
        return match text {
            // Responses "required" in Chat is "required"， 1 
            other => Value::String(other.to_string()),
        };
    }
    // `{type:"function", name}` → `{type:"function", function:{name}}`
    let name = {
        let flat = string_field(choice, "name");
        if flat.is_empty() { string_field(choice, "function.name") } else { flat }
    };
    // `function.name` is path，string_field not to ，
    let name = if name.is_empty() {
        choice.pointer("/function/name").map(string_value).unwrap_or_default()
    } else {
        name
    };
    if !name.is_empty() {
        return json!({ "type": "function", "function": { "name": name } });
    }
    choice.clone()
}

/// 1 use → Responses output item。
///
/// by `plan` ：
/// · is custom（ by ）→ `custom_tool_call`，parameter is 
/// `input` not is `arguments` JSON。 to ，Codex will by 
/// JSON parameter， not ；
/// · empty → `namespace` and `name` 。Codex
/// by by `{name, namespace}` ，
///     `unsupported call`。
///
/// `call_id` by use to （ up id， or ）： 
/// （ / / ） use this 1 ， 、 after 1 。
fn tool_call_item(name: &str, call_id: &str, arguments: &str, plan: &tool_plan::ToolPlan) -> Value {
    let item = if plan.is_custom(name) {
        json!({
            "type": "custom_tool_call",
            "id": random_id("ctc"),
            "status": "completed",
            "call_id": call_id,
            "name": name,
            "input": freeform::unwrap_freeform_input(arguments),
        })
    } else {
        json!({
            "type": "function_call",
            "id": random_id("fc"),
            "status": "completed",
            "call_id": call_id,
            "name": name,
            "arguments": arguments,
        })
    };
    tool_plan::restore_namespace(item, name, plan)
}

/// Responses `text.format` → Chat `response_format`
fn format_to_chat(format: &Value) -> Value {
    let kind = string_field(format, "type");
    if kind != "json_schema" {
        return format.clone();
    }
    // Responses is `{type, name, schema, strict}`，
    // Chat need `{type, json_schema:{name, schema, strict}}`
    let mut inner = Map::new();
    for key in ["name", "description", "schema", "strict"] {
        if let Some(value) = format.get(key).filter(|value| is_truthy(value)) {
            inner.insert(key.to_string(), value.clone());
        }
    }
    json!({ "type": "json_schema", "json_schema": Value::Object(inner) })
}

// ─── response：Chat → Responses（）─────────────────────────

/// Chat `chat.completion` → Responses `response` for 。
///
/// `request` is ** Responses request**： need 1 request
/// （instructions / temperature / tools / …），client
/// 「 parameter 」。
pub fn responses_from_chat(chat: &Value, model: &str, request: &Value) -> Value {
    let plan = tool_plan::plan_tools(request);
    let choice = chat.pointer("/choices/0");
    let message = choice.and_then(|choice| choice.get("message")).cloned().unwrap_or(Value::Null);
    let finish = choice
        .and_then(|choice| choice.get("finish_reason"))
        .map(string_value)
        .unwrap_or_default();

    let mut output: Vec<Value> = Vec::new();
    let reasoning = {
        let from_field = string_field(&message, "reasoning_content");
        if from_field.is_empty() { string_field(&message, "reasoning") } else { from_field }
    };
    if !reasoning.is_empty() {
        output.push(json!({
            "type": "reasoning",
            "id": random_id("rs"),
            "summary": [{ "type": "summary_text", "text": reasoning }],
        }));
    }
    let text = content_text(message.get("content").unwrap_or(&Value::Null));
    let tool_calls = message.get("tool_calls").and_then(Value::as_array);
    // no has use when to 1 message （ as empty）：
    // client `output_text` in ， will 「response」
    if !text.is_empty() || tool_calls.is_none() {
        output.push(json!({
            "type": "message",
            "id": random_id("msg"),
            "status": "completed",
            "role": "assistant",
            "content": [{ "type": "output_text", "text": text, "annotations": [], "logprobs": [] }],
        }));
    }
    if let Some(calls) = tool_calls {
        for call in calls {
            let name = call.pointer("/function/name").map(string_value).unwrap_or_default();
            let arguments = call
                .pointer("/function/arguments")
                .map(json_text)
                .unwrap_or_else(|| "{}".to_string());
            let call_id = {
                let raw = string_field(call, "id");
                if raw.is_empty() { random_id("call") } else { raw }
            };
            output.push(tool_call_item(&name, &call_id, &arguments, &plan));
        }
    }

    let incomplete = if finish == "length" {
        Some("max_output_tokens")
    } else if finish == "content_filter" {
        Some("content_filter")
    } else {
        None
    };
    let status = if incomplete.is_some() { "incomplete" } else { "completed" };
    let created = chat
        .get("created")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| logging::now_ms() / 1000);
    let id = {
        let raw = string_field(chat, "id");
        if raw.is_empty() { random_id("resp") } else { raw }
    };
    let mut body = response_envelope(
        &id,
        model,
        status,
        output,
        usage_to_responses(chat.get("usage")),
        request,
        created,
        incomplete,
    );
    if let Some(map) = body.as_object_mut() {
        // output_text is SDK （ all message ）
        map.insert("output_text".to_string(), Value::String(text));
    }
    body
}

/// Responses response （ `response.completed` use 1 ）
#[allow(clippy::too_many_arguments)]
pub fn response_envelope(
    id: &str,
    model: &str,
    status: &str,
    output: Vec<Value>,
    usage: Value,
    request: &Value,
    created: i64,
    incomplete_reason: Option<&str>,
) -> Value {
    // request（ and 1 ）
    let passthrough = |key: &str, fallback: Value| -> Value {
        request.get(key).filter(|value| !value.is_null()).cloned().unwrap_or(fallback)
    };
    let reasoning_effort = request
        .pointer("/reasoning/effort")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or(Value::Null);
    let reasoning_summary = request
        .pointer("/reasoning/summary")
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "id": id,
        "object": "response",
        "created_at": created,
        "status": status,
        "completed_at": if status == "completed" { Value::from(logging::now_ms() / 1000) } else { Value::Null },
        "background": false,
        "error": Value::Null,
        "incomplete_details": match incomplete_reason {
            Some(reason) => json!({ "reason": reason }),
            None => Value::Null,
        },
        "instructions": passthrough("instructions", Value::Null),
        "max_output_tokens": passthrough("max_output_tokens", Value::Null),
        "max_tool_calls": passthrough("max_tool_calls", Value::Null),
        "model": model,
        "output": output,
        "parallel_tool_calls": passthrough("parallel_tool_calls", Value::Bool(true)),
        "previous_response_id": Value::Null,
        "reasoning": { "effort": reasoning_effort, "summary": reasoning_summary },
        "store": false,
        "temperature": passthrough("temperature", Value::from(1)),
        "text": passthrough("text", json!({ "format": { "type": "text" } })),
        "tool_choice": passthrough("tool_choice", Value::String("auto".to_string())),
        "tools": request.get("tools").filter(|value| value.is_array()).cloned().unwrap_or_else(|| json!([])),
        "top_logprobs": passthrough("top_logprobs", Value::from(0)),
        "top_p": passthrough("top_p", Value::from(1)),
        "truncation": passthrough("truncation", Value::String("disabled".to_string())),
        "usage": usage,
        "user": passthrough("user", Value::Null),
        "metadata": passthrough("metadata", json!({})),
    })
}

/// Chat usage → Responses usage（ and not ）
/// `finish_reason` **failure** → `(code, can )`；`None` = not is failure。
///
/// before ZCode `network_error`（ 2026-09-30：
/// 、 up etc will ， HTTP is 200、 is empty ）。
///
/// this need ****： not by has （`length` →
/// `max_output_tokens`、`content_filter` → `content_filter`、 its → `completed`）。
/// 「 no 」 1 failure will up —— that 
/// is this ， not is 。
fn finish_failure(finish_reason: Option<&str>) -> Option<(&'static str, String)> {
    match finish_reason? {
        "network_error" => Some((
            "upstream_network_error",
            "上游报告 network_error：请求已到达上游但没有可用结果。\
             套餐池（Start Plan）通道上最常见的原因是思考预算吃光了输出额度（正文因此为空，\
             重试前请调高 max_tokens 或降低思考档位）；也可能是上游瞬时故障，\
             可直接重试。"
                .to_string(),
        )),
        _ => None,
    }
}

pub fn usage_to_responses(usage: Option<&Value>) -> Value {
    let Some(usage) = usage.filter(|value| value.is_object()) else {
        return Value::Null;
    };
    let number = |keys: &[&str]| -> i64 {
        for key in keys {
            if let Some(value) = usage.get(*key) {
                if let Some(parsed) = value.as_i64() {
                    return parsed;
                }
                if let Some(parsed) = value.as_f64().filter(|value| value.is_finite()) {
                    return parsed as i64;
                }
            }
        }
        0
    };
    let input = number(&["prompt_tokens", "input_tokens"]);
    let output = number(&["completion_tokens", "output_tokens"]);
    let cached = number(&[
        "prompt_tokens_details.cached_tokens",
        "prompt_cache_hit_tokens",
        "cache_read_input_tokens",
    ]);
    // need （ up not to ， this 1 ）
    let cached = {
        let nested = usage
            .pointer("/prompt_tokens_details/cached_tokens")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        if nested != 0 { nested } else { cached }
    };
    let reasoning = usage
        .pointer("/completion_tokens_details/reasoning_tokens")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let mut details = Map::new();
    if cached != 0 {
        details.insert("cached_tokens".to_string(), Value::from(cached));
    }
    let mut output_details = Map::new();
    if reasoning != 0 {
        output_details.insert("reasoning_tokens".to_string(), Value::from(reasoning));
    }
    let mut out = Map::new();
    out.insert("input_tokens".to_string(), Value::from(input));
    if !details.is_empty() {
        out.insert("input_tokens_details".to_string(), Value::Object(details));
    }
    out.insert("output_tokens".to_string(), Value::from(output));
    if !output_details.is_empty() {
        out.insert("output_tokens_details".to_string(), Value::Object(output_details));
    }
    out.insert(
        "total_tokens".to_string(),
        Value::from(number(&["total_tokens"]).max(input + output)),
    );
    Value::Object(out)
}

// ─── ：Chat SSE → Responses SSE ─────────────────────────

/// Chat SSE → Responses SSE （）。
///
/// ── Responses （client by this ）───────────
///   response.created
/// response.output_item.added（ output 1 ）
/// response.content_part.added（message has ）
/// response.output_text.delta / response.function_call_arguments.delta（）
/// response.output_text.done / …（）
///   response.output_item.done
/// response.completed（ response for ）
///
/// will SDK 「」， down `open_text` /
/// `close_text` / `open_tool` 「 1 when has 1 in 」。
pub struct ResponsesStream {
    buffer: SseLineBuffer,
    /// down to client response id（ up chunk id，）
    response_id: String,
    created: i64,
    model: String,
    request: Value,
    /// request as custom（freeform） ： in when need 
    /// `custom_tool_call` and `response.custom_tool_call_input.*`，
    /// not is `function_call` and `response.function_call_arguments.*`
    plan: tool_plan::ToolPlan,
    /// （Responses need sequence_number）
    sequence: i64,
    created_sent: bool,
    finished: bool,
    /// is already + output_index / item_id
    text_open: bool,
    text_index: i64,
    text_id: String,
    text: String,
    /// （reasoning）：， to or when need 
    reasoning_open: bool,
    reasoning_closed: bool,
    reasoning_index: i64,
    reasoning_id: String,
    reasoning: String,
    /// use ： by up to index （ 1 index 1 use ）
    tools: std::collections::BTreeMap<i64, ToolAccum>,
    /// down 1 available output_index
    next_index: i64,
    usage: Option<Value>,
    finish_reason: Option<String>,
    /// already output （ when by output_index response.output）
    output_items: Vec<(i64, Value)>,
}

#[derive(Default)]
struct ToolAccum {
    index: i64,
    item_id: String,
    call_id: String,
    name: String,
    arguments: String,
    announced: bool,
}

/// 1 （`close_tools` use ）。
///
/// 1 not is ： already has ， down is 。
struct ToolFinal {
    index: i64,
    item_id: String,
    call_id: String,
    name: String,
    announced: bool,
    item: Value,
}

impl ResponsesStream {
    pub fn new(model: &str, request: &Value) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            response_id: random_id("resp"),
            created: logging::now_ms() / 1000,
            model: model.to_string(),
            request: request.clone(),
            plan: tool_plan::plan_tools(request),
            sequence: 0,
            created_sent: false,
            finished: false,
            text_open: false,
            text_index: -1,
            text_id: String::new(),
            text: String::new(),
            reasoning_open: false,
            reasoning_closed: false,
            reasoning_index: -1,
            reasoning_id: String::new(),
            reasoning: String::new(),
            tools: std::collections::BTreeMap::new(),
            next_index: 0,
            usage: None,
            finish_reason: None,
            output_items: Vec::new(),
        }
    }

    /// 1 up ， need down SSE 
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

    /// up （ no has [DONE] when ）
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
        if self.finished {
            return out;
        }
        self.finished = true;
        out.extend(self.emit_created());
        // up failure in `finish_reason` when **failure**， not need 「successempty」。
        // ZCode in this down 
        // `finish_reason:"network_error"`， not is `length` not is 
        // `content_filter` —— by 「unknown = completed」 ，client to 
        // is 1 `status:"completed"`、`output` as empty response（issue #52「200 
        // as empty」 ）， when not up its 。
        if let Some((code, message)) = finish_failure(self.finish_reason.as_deref()) {
            let mut failed = response_envelope(
                &self.response_id,
                &self.model,
                "failed",
                Vec::new(),
                usage_to_responses(self.usage.as_ref()),
                &self.request,
                self.created,
                None,
            );
            if let Some(map) = failed.as_object_mut() {
                map.insert(
                    "error".to_string(),
                    json!({ "code": code, "message": message }),
                );
            }
            out.push(self.event(
                "error",
                json!({ "code": code, "message": message, "param": Value::Null }),
            ));
            out.push(self.event("response.failed", json!({ "response": failed })));
            return out;
        }
        out.extend(self.close_reasoning());
        // up no to （empty）： 1 empty message，otherwiseclient not to output
        if !self.text_open && self.tools.is_empty() {
            out.extend(self.open_text());
        }
        out.extend(self.close_text());
        out.extend(self.close_tools());
        let output: Vec<Value> = {
            let mut items = self.output_items.clone();
            items.sort_by_key(|(index, _)| *index);
            items.into_iter().map(|(_, item)| item).collect()
        };
        let incomplete = match self.finish_reason.as_deref() {
            Some("length") => Some("max_output_tokens"),
            Some("content_filter") => Some("content_filter"),
            _ => None,
        };
        let status = if incomplete.is_some() { "incomplete" } else { "completed" };
        let response = response_envelope(
            &self.response_id,
            &self.model,
            status,
            output,
            usage_to_responses(self.usage.as_ref()),
            &self.request,
            self.created,
            incomplete,
        );
        out.push(self.event("response.completed", json!({ "response": response })));
        out
    }

    /// 1 up Chat chunk → to Responses 
    fn consume(&mut self, chunk: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let mut out = Vec::new();
        // up error： Responses error + response.failed
        if let Some(error) = chunk.get("error").filter(|value| is_truthy(value)) {
            self.finished = true;
            out.extend(self.emit_created());
            let message = {
                let text = string_field(error, "message");
                if text.is_empty() { string_value(error) } else { text }
            };
            let code = error.get("code").filter(|value| is_truthy(value)).cloned().unwrap_or(Value::Null);
            out.push(self.event(
                "error",
                json!({ "code": code, "message": message, "param": Value::Null }),
            ));
            let mut failed = response_envelope(
                &self.response_id,
                &self.model,
                "failed",
                Vec::new(),
                usage_to_responses(self.usage.as_ref()),
                &self.request,
                self.created,
                None,
            );
            if let Some(map) = failed.as_object_mut() {
                map.insert(
                    "error".to_string(),
                    json!({ "code": code, "message": message }),
                );
            }
            out.push(self.event("response.failed", json!({ "response": failed })));
            return out;
        }
        if let Some(id) = chunk.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
            if !self.created_sent {
                self.response_id = id.to_string();
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
        out.extend(self.emit_created());
        let Some(delta) = choice.and_then(|choice| choice.get("delta")) else {
            return out;
        };
        // 
        let reasoning = {
            let from_field = string_field(delta, "reasoning_content");
            if from_field.is_empty() { string_field(delta, "reasoning") } else { from_field }
        };
        if !reasoning.is_empty() {
            out.extend(self.open_reasoning());
            self.reasoning.push_str(&reasoning);
            out.push(self.event(
                "response.reasoning_summary_text.delta",
                json!({
                    "output_index": self.reasoning_index,
                    "item_id": self.reasoning_id,
                    "summary_index": 0,
                    "delta": reasoning,
                }),
            ));
        }
        // ：must（ 1 when can has 1 in ）
        if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|text| !text.is_empty()) {
            out.extend(self.close_reasoning());
            out.extend(self.open_text());
            self.text.push_str(text);
            out.push(self.event(
                "response.output_text.delta",
                json!({
                    "output_index": self.text_index,
                    "item_id": self.text_id,
                    "content_index": 0,
                    "delta": text,
                }),
            ));
        }
        // use 
        if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in calls {
                out.extend(self.consume_tool(call));
            }
        }
        out
    }

    fn consume_tool(&mut self, call: &Value) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        let key = call.get("index").and_then(Value::as_i64).unwrap_or(0);
        // use and ： and 
        out.extend(self.close_reasoning());
        out.extend(self.close_text());

        // ── update and **** ──────────────────────────
        // need `self.sequence`，update need `self.tools`。
        // is not ，「 use `&mut self` 」 will 
        // use check —— update in 1 （ use ），
        // ，。
        let (index, item_id, call_id, name, announced_now, buffered) = {
            if !self.tools.contains_key(&key) {
                let next = self.next_index;
                self.next_index += 1;
                self.tools.insert(
                    key,
                    ToolAccum {
                        index: next,
                        item_id: random_id("fc"),
                        call_id: String::new(),
                        name: String::new(),
                        arguments: String::new(),
                        announced: false,
                    },
                );
            }
            let Some(tool) = self.tools.get_mut(&key) else {
                return out;
            };
            if let Some(id) = call.get("id").and_then(Value::as_str).filter(|id| !id.is_empty()) {
                tool.call_id = id.to_string();
            }
            if let Some(name) = call.pointer("/function/name").and_then(Value::as_str) {
                if !name.is_empty() {
                    tool.name = name.to_string();
                }
            }
            if let Some(arguments) = call.pointer("/function/arguments").and_then(Value::as_str) {
                tool.arguments.push_str(arguments);
            }
            // to can this 1 （ need name）
            let announced_now = !tool.announced && !tool.name.is_empty();
            if announced_now {
                if tool.call_id.is_empty() {
                    tool.call_id = random_id("call");
                }
                // custom id before and function not （ctc_ vs fc_）， this 
                // is 1 can when ： after item_id already in ，
                // for not up 
                if self.plan.is_custom(&tool.name) {
                    tool.item_id = random_id("ctc");
                }
                tool.announced = true;
            }
            (
                tool.index,
                tool.item_id.clone(),
                tool.call_id.clone(),
                tool.name.clone(),
                announced_now,
                tool.arguments.clone(),
            )
        };

        let is_custom = self.plan.is_custom(&name);
        if announced_now {
            // by 「parameter」 is input，item and function not 
            // —— custom_tool_call 
            let item = if is_custom {
                json!({
                    "type": "custom_tool_call",
                    "id": item_id,
                    "status": "in_progress",
                    "call_id": call_id,
                    "name": name,
                    "input": "",
                })
            } else {
                json!({
                    "type": "function_call",
                    "id": item_id,
                    "status": "in_progress",
                    "call_id": call_id,
                    "name": name,
                    "arguments": "",
                })
            };
            // empty need in this up `namespace`：client in ** when ** by 
            // `{name, namespace}` by ， etc to already 
            let item = tool_plan::restore_namespace(item, &name, &self.plan);
            out.push(response_event(
                &mut self.sequence,
                "response.output_item.added",
                json!({ "output_index": index, "item": item }),
            ));
            // before already down parameter need （ and parameter can can 1 to ）
            if !buffered.is_empty() {
                out.extend(self.tool_delta_event(index, &item_id, &call_id, &name, &buffered, is_custom));
            }
        }
        let arguments = call
            .pointer("/function/arguments")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !arguments.is_empty() && !announced_now {
            out.extend(self.tool_delta_event(index, &item_id, &call_id, &name, arguments, is_custom));
        }
        out
    }

    /// 1 （function and custom not ）。
    ///
    /// **custom 1 not **，returnempty。 by is for not up ： up by JSON 
    /// `{"input": "…"}`， `custom_tool_call_input.delta` is ** by 
    /// ** —— will client this JSON ， after 
    /// to is 1 `{"input":` before ， not 。 is 
    /// to ， in none can JSON （ can can in in ）。
    ///
    /// by `custom_tool_call_input.done` 1 （
    /// `close_tools`）， that 1 to is `arguments`， can 。
    /// clientmust `.done`， not 。
    fn tool_delta_event(
        &mut self,
        index: i64,
        item_id: &str,
        call_id: &str,
        name: &str,
        delta: &str,
        is_custom: bool,
    ) -> Vec<bytes::Bytes> {
        if is_custom {
            return Vec::new();
        }
        vec![self.event(
            "response.function_call_arguments.delta",
            json!({
                "output_index": index,
                "item_id": item_id,
                "call_id": call_id,
                "name": name,
                "delta": delta,
            }),
        )]
    }

    fn emit_created(&mut self) -> Vec<bytes::Bytes> {
        if self.created_sent {
            return Vec::new();
        }
        self.created_sent = true;
        let response = response_envelope(
            &self.response_id,
            &self.model,
            "in_progress",
            Vec::new(),
            Value::Null,
            &self.request,
            self.created,
            None,
        );
        vec![self.event("response.created", json!({ "response": response }))]
    }

    fn open_reasoning(&mut self) -> Vec<bytes::Bytes> {
        if self.reasoning_open {
            return Vec::new();
        }
        self.reasoning_open = true;
        self.reasoning_index = self.next_index;
        self.next_index += 1;
        self.reasoning_id = random_id("rs");
        vec![
            self.event(
                "response.output_item.added",
                json!({
                    "output_index": self.reasoning_index,
                    "item": {
                        "type": "reasoning",
                        "id": self.reasoning_id,
                        "status": "in_progress",
                        "summary": [],
                    },
                }),
            ),
            self.event(
                "response.reasoning_summary_part.added",
                json!({
                    "output_index": self.reasoning_index,
                    "item_id": self.reasoning_id,
                    "summary_index": 0,
                    "part": { "type": "summary_text", "text": "" },
                }),
            ),
        ]
    }

    fn close_reasoning(&mut self) -> Vec<bytes::Bytes> {
        if !self.reasoning_open || self.reasoning_closed {
            return Vec::new();
        }
        self.reasoning_closed = true;
        let item = json!({
            "type": "reasoning",
            "id": self.reasoning_id,
            "status": "completed",
            "summary": [{ "type": "summary_text", "text": self.reasoning }],
        });
        self.output_items.push((self.reasoning_index, item.clone()));
        vec![
            self.event(
                "response.reasoning_summary_text.done",
                json!({
                    "output_index": self.reasoning_index,
                    "item_id": self.reasoning_id,
                    "summary_index": 0,
                    "text": self.reasoning,
                }),
            ),
            self.event(
                "response.reasoning_summary_part.done",
                json!({
                    "output_index": self.reasoning_index,
                    "item_id": self.reasoning_id,
                    "summary_index": 0,
                    "part": { "type": "summary_text", "text": self.reasoning },
                }),
            ),
            self.event(
                "response.output_item.done",
                json!({ "output_index": self.reasoning_index, "item": item }),
            ),
        ]
    }

    fn open_text(&mut self) -> Vec<bytes::Bytes> {
        if self.text_open {
            return Vec::new();
        }
        self.text_open = true;
        self.text_index = self.next_index;
        self.next_index += 1;
        self.text_id = random_id("msg");
        vec![
            self.event(
                "response.output_item.added",
                json!({
                    "output_index": self.text_index,
                    "item": {
                        "type": "message",
                        "id": self.text_id,
                        "status": "in_progress",
                        "role": "assistant",
                        "content": [{ "type": "output_text", "text": "", "annotations": [] }],
                    },
                }),
            ),
            self.event(
                "response.content_part.added",
                json!({
                    "output_index": self.text_index,
                    "item_id": self.text_id,
                    "content_index": 0,
                    "part": { "type": "output_text", "text": "", "annotations": [] },
                }),
            ),
        ]
    }

    fn close_text(&mut self) -> Vec<bytes::Bytes> {
        if !self.text_open {
            return Vec::new();
        }
        self.text_open = false;
        let part = json!({
            "type": "output_text",
            "text": self.text,
            "annotations": [],
            "logprobs": [],
        });
        let item = json!({
            "type": "message",
            "id": self.text_id,
            "status": "completed",
            "role": "assistant",
            "content": [part.clone()],
        });
        self.output_items.push((self.text_index, item.clone()));
        vec![
            self.event(
                "response.output_text.done",
                json!({
                    "output_index": self.text_index,
                    "item_id": self.text_id,
                    "content_index": 0,
                    "text": self.text,
                }),
            ),
            self.event(
                "response.content_part.done",
                json!({
                    "output_index": self.text_index,
                    "item_id": self.text_id,
                    "content_index": 0,
                    "part": part,
                }),
            ),
            self.event(
                "response.output_item.done",
                json!({ "output_index": self.text_index, "item": item }),
            ),
        ]
    }

    fn close_tools(&mut self) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        // by output_index （ and 1 ）
        let mut keys: Vec<i64> = self.tools.keys().copied().collect();
        keys.sort_by_key(|key| self.tools.get(key).map(|tool| tool.index).unwrap_or(0));
        // 「 」， 1 —— and consume_tool
        // 1 by ： need `self.sequence`， not can and `self.tools` use 
        let mut finals: Vec<ToolFinal> = Vec::new();
        for key in keys {
            let Some(tool) = self.tools.get_mut(&key) else {
                continue;
            };
            let call_id = if tool.call_id.is_empty() { random_id("call") } else { tool.call_id.clone() };
            // to index、 no to ： not can （ need name），
            // not can —— 1 ，otherwiseclient 1 use 
            let name = if tool.name.is_empty() { "unknown".to_string() } else { tool.name.clone() };
            let arguments = if tool.arguments.is_empty() { "{}".to_string() } else { tool.arguments.clone() };
            // use 1 item：custom in when need custom_tool_call and 
            // input， to client by JSON ， not 
            let item = {
                let mut value = tool_call_item(&name, &call_id, &arguments, &self.plan);
                // item_id need use when that ：client by for up 
                if let Some(map) = value.as_object_mut() {
                    map.insert("id".to_string(), Value::String(tool.item_id.clone()));
                }
                value
            };
            self.output_items.push((tool.index, item.clone()));
            finals.push(ToolFinal {
                index: tool.index,
                item_id: tool.item_id.clone(),
                call_id,
                name,
                announced: tool.announced,
                item,
            });
        }
        // and use （ up ）： this already not `self.tools`
        for final_ in finals {
            let is_custom = final_
                .item
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "custom_tool_call");
            if final_.announced {
                if is_custom {
                    // by by `input` ， and not call_id / name
                    let input = string_field(&final_.item, "input");
                    out.push(self.event(
                        "response.custom_tool_call_input.done",
                        json!({
                            "output_index": final_.index,
                            "item_id": final_.item_id,
                            "input": input,
                        }),
                    ));
                } else {
                    let arguments = string_field(&final_.item, "arguments");
                    out.push(response_event(
                        &mut self.sequence,
                        "response.function_call_arguments.done",
                        json!({
                            "output_index": final_.index,
                            "item_id": final_.item_id,
                            "call_id": final_.call_id,
                            "name": final_.name,
                            "arguments": arguments,
                        }),
                    ));
                }
            }
            out.push(self.event(
                "response.output_item.done",
                json!({ "output_index": final_.index, "item": final_.item }),
            ));
        }
        out
    }
    /// 1 sequence_number 。
    ///
    /// by [`response_event`]： need `sequence`，
    /// use `self.tools` —— `&mut self` will use check，
    /// in by ， use `&mut self.sequence` can 。
    fn event(&mut self, event: &str, data: Value) -> bytes::Bytes {
        response_event(&mut self.sequence, event, data)
    }
}

/// 1 `sequence_number` Responses 。
///
/// Responses need **** ，client（ SDK）
/// will use and 。 by use has 。
pub fn response_event(sequence: &mut i64, event: &str, mut data: Value) -> bytes::Bytes {
    let current = *sequence;
    *sequence += 1;
    if let Some(map) = data.as_object_mut() {
        map.insert("type".to_string(), Value::String(event.to_string()));
        map.insert("sequence_number".to_string(), Value::from(current));
    }
    event_frame(event, &data)
}

/// ： Chat SSE 1 Responses response for 。
///
/// Responses request need up （ up is can ），
/// after JSON return —— and `/v1/chat/completions` path 1 。
pub struct ResponsesCollector {
    buffer: SseLineBuffer,
    text: String,
    reasoning: String,
    tools: std::collections::BTreeMap<i64, ToolAccum>,
    usage: Option<Value>,
    finish_reason: Option<String>,
    id: String,
    created: i64,
}

impl ResponsesCollector {
    pub fn new() -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            text: String::new(),
            reasoning: String::new(),
            tools: std::collections::BTreeMap::new(),
            usage: None,
            finish_reason: None,
            id: String::new(),
            created: logging::now_ms() / 1000,
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
        if let Some(created) = chunk.get("created").and_then(Value::as_i64) {
            self.created = created;
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

    /// 1 Responses response for 
    pub fn into_response(self, model: &str, request: &Value) -> Value {
        let plan = tool_plan::plan_tools(request);
        let mut output: Vec<Value> = Vec::new();
        if !self.reasoning.is_empty() {
            output.push(json!({
                "type": "reasoning",
                "id": random_id("rs"),
                "summary": [{ "type": "summary_text", "text": self.reasoning }],
            }));
        }
        if !self.text.is_empty() || self.tools.is_empty() {
            output.push(json!({
                "type": "message",
                "id": random_id("msg"),
                "status": "completed",
                "role": "assistant",
                "content": [{ "type": "output_text", "text": self.text, "annotations": [], "logprobs": [] }],
            }));
        }
        for (_, tool) in self.tools {
            let name = if tool.name.is_empty() { "unknown".to_string() } else { tool.name };
            let call_id = if tool.call_id.is_empty() { random_id("call") } else { tool.call_id };
            let arguments = if tool.arguments.is_empty() { "{}".to_string() } else { tool.arguments };
            // and use ：custom in when custom_tool_call
            // （item id before by tool_call_item by ， not in this ）
            output.push(tool_call_item(&name, &call_id, &arguments, &plan));
        }
        let incomplete = match self.finish_reason.as_deref() {
            Some("length") => Some("max_output_tokens"),
            Some("content_filter") => Some("content_filter"),
            _ => None,
        };
        let status = if incomplete.is_some() { "incomplete" } else { "completed" };
        let id = if self.id.is_empty() { random_id("resp") } else { self.id.clone() };
        let mut body = response_envelope(
            &id,
            model,
            status,
            output,
            usage_to_responses(self.usage.as_ref()),
            request,
            self.created,
            incomplete,
        );
        if let Some(map) = body.as_object_mut() {
            map.insert("output_text".to_string(), Value::String(self.text));
        }
        body
    }
}

impl Default for ResponsesCollector {
    fn default() -> Self {
        Self::new()
    }
}
