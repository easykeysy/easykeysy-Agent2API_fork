//! ： down **Responses**（`/v1/responses`） and **Anthropic Messages**
//! （`/v1/messages`） 1 **Chat Completions**， 。
//!
//! ── Why Chat as ， not is in 1 ────────
//! up ** in / up 1 as OpenAI Chat **：
//! has （ will up ） in `conversation` OpenAI chunk
//! down ；noneresponse （NDJSON、Gemini v1internal SSE etc ） by 
//! `upstream::translate` chat SSE —— in `chat` 
//! `UpstreamKind`，。 is ，「 up Chat」 this in up 
//! ** already **。
//!
//! is has can can ：
//! ① in 1 （ × 2 ）；
//! ② in 1 （2 ）， not 。
//! ② ： not down is （ Chat），
//! add 1 up when not use 1 。 is ② 。
//!
//! ── and （OmniProxy） ────────────────────────
//! OmniProxy **Responses as **（Chat ↔ Responses ↔ Anthropic），
//! as up when in 、 and need supports「 down Responses → up Anthropic」
//! this 。 in up ** Chat this 1 **（ up in before 
//! chat）， Chat：
//! 1 ， 1 in ， 1 「 in 」 。
//! then （///usage ） OmniProxy
//! `gatewayProtocol*.ts`， and by 。
//!
//! ── file ────────────────────────────────────────────────
//! mod.rs file：（// id/SSE /）
//! responses.rs Responses ↔ Chat（ down Responses ）
//! anthropic.rs Anthropic Messages ↔ Chat（ up ）
//! responses_outbound.rs chat → Responses up （）
//! anthropic_outbound.rs chat → Anthropic up （ up ）
//! ai_sdk_ndjson.rs up **AI SDK v5 NDJSON** → chat SSE
//! （； `UpstreamKind::Ndjson` when use ，
//! ， this filemodule header）
//! antigravity_stream.rs up Gemini v1internal SSE → chat SSE
//! （Antigravity already to ， this **response**
//! ：request and schema ）
//! history.rs Chat sanitize（client ）
//!
//! file and file to ：「 down X」 
//! （`api::protocol`），「 up X」 （
//! `providers::custom::forward` and up ）。file not is file： 
//! file already file（ 1000+ ）， to 
//! is 1 （request + SSE ）， after 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic， 1 Option ；
//! not 、 not file、 not axum（ + ，）。

pub mod anthropic;
pub mod anthropic_outbound;
pub mod antigravity_signature_cache;
pub mod antigravity_stream;
pub mod chatglm_stream;
pub mod ai_sdk_ndjson;
pub mod freeform;
pub mod history;
pub mod native_tool;
pub mod responses;
pub mod responses_outbound;
pub mod tool_plan;

use std::borrow::Cow;

use serde_json::Value;

/// JS `String(value)`：（`null` → empty， and `value == null ? ""` ）。
///
/// to need by 「client to can can is JSON 」（ request
/// `model` can can is 、`name` can can is for ）， JS is `stringValue`。
/// 1 this 1 ， `as_str().unwrap_or("")` 。
pub fn string_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// 1 for and （ → empty）
pub fn string_field(value: &Value, key: &str) -> String {
    value.get(key).map(string_value).unwrap_or_default()
}

/// has 、 no has when ， in Chat `tool` 。
///
/// not can content empty：tool content is ，empty in up will ；
/// this when 「 in that 」—— otherwise to returnempty。
///
/// use 1 （Responses `function_call_output`、Anthropic 
/// `tool_result`）： tool by and 1 ， not this 
/// —— when in 1 to this ， is 1 。
pub const TOOL_IMAGE_PLACEHOLDER: &str =
    "(the tool returned an image; it is attached in the following message)";

/// JS `jsonText(value)`：return， its JSON 。
///
/// use is 「 any JSON 1 」（parameter、）。
/// failure can can is （`Value` 1 can ， has ），
/// this to empty not is panic。
pub fn json_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// JS （`Boolean(x)`）—— `if (x)` is this 
pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}
/// 1 before id（`msg_<32>` ）。
///
/// ── Why not use when or ────────────────────────────────────
/// this id will in ** down **（`response.id` / `message.id` / `tool_use.id`），
/// client can can use and 。 1 seconds request if to id，
/// client will —— when not to 1 ， need 。
/// `getrandom`（ already is ，Qoder PKCE use ）。
///
/// failure（ not can can ） when when ：id ** 1 ** in that down ，
/// returnempty —— empty id will clientfailure。
pub fn random_id(prefix: &str) -> String {
    let mut bytes = [0u8; 16];
    if getrandom::getrandom(&mut bytes).is_err() {
        return format!("{prefix}_{:x}", crate::server::logging::now_ms());
    }
    let mut out = String::with_capacity(prefix.len() + 1 + 32);
    out.push_str(prefix);
    out.push('_');
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// from （ text ）。
///
/// for `contentText`：return； then 
/// `text` / `input_text` / `output_text` `text` 。
/// its （、）** not and ** —— is ，
/// will ， need need use 。
pub fn content_text(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => {
            let mut out = String::new();
            for part in parts {
                if let Some(text) = part.as_str() {
                    out.push_str(text);
                    continue;
                }
                let kind = part.get("type").map(string_value).unwrap_or_default();
                let kind = kind.to_lowercase();
                if matches!(kind.as_str(), "text" | "input_text" | "output_text") {
                    out.push_str(&string_field(part, "text"));
                }
            }
            out
        }
        _ => String::new(),
    }
}

/// （ when to empty）—— use need 
pub fn content_parts(content: &Value) -> &[Value] {
    match content.as_array() {
        Some(parts) => parts.as_slice(),
        None => &[],
    }
}
/// 1 `event:` SSE （Responses and Anthropic need ）。
///
/// Anthropic SSE ** `event:` + `data:` **（`data` `type`），
/// Responses 。 client by `event:` ，must。
pub fn event_frame(event: &str, value: &Value) -> bytes::Bytes {
    let text = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    bytes::Bytes::from(format!("event: {event}\ndata: {text}\n\n"))
}

/// 1 **chat** SSE （`data: <JSON>\n\n`， not `event:` ——
/// Chat SSE has data ）。
///
/// and `upstream::sse::sse_frame` ； not upstream 
/// （`core::protocol` is + ）， this 1 。
/// （`*_outbound`） chat this ， can 
/// is down has （`ForwardStream` / / ） 。
pub fn chat_frame(value: &Value) -> bytes::Bytes {
    let text = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    bytes::Bytes::from(format!("data: {text}\n\n"))
}

/// by from for 1 （ not to to 0）。
///
/// usage in not 1 （Chat is `prompt_tokens`、Responses is 
/// `input_tokens`）， usage when 1 this ， 
/// 。、 to has （ up token ）。
pub fn json_number_of(value: &Value, keys: &[&str]) -> i64 {
    for key in keys {
        let Some(field) = value.get(*key) else {
            continue;
        };
        if let Some(number) = field.as_i64() {
            return number;
        }
        if let Some(number) = field.as_f64().filter(|number| number.is_finite()) {
            return number as i64;
        }
    }
    0
}
/// 1 by SSE 1 `data:` 。
///
/// ── Why need need （ not can by `\n` ）────────────────────────
/// up is ** by TCP ** to ， 1 JSON can can 
/// （ in 1 in ）。must has 1 chunk ：
/// to after 1 `\n` as ， down to down 1 。
/// this and `core::upstream::sse` ReasoningCoalescer is 1 ，
/// that 「reasoning and + 」， this need need ****。
///
/// by **** not is String： in ， can can 1 
/// 3 ， by String will that 1 U+FFFD（）。
#[derive(Default)]
pub struct SseLineBuffer {
    tail: Vec<u8>,
}

impl SseLineBuffer {
    pub fn new() -> Self {
        Self { tail: Vec::new() }
    }

    /// 1 ， its in **** `data:` （0..n ）。
    ///
    /// return `None` `[DONE]`（ use ）。 `data:` 
    /// （`event:` / / empty） in this ： use ，
    /// from JSON `type` （ SSE in data type，
    /// this is `event.type || frame.event` ）。
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Option<String>> {
        let mut out: Vec<Option<String>> = Vec::new();
        self.tail.extend_from_slice(chunk);
        let mut start = 0usize;
        while let Some(offset) = self.tail[start..].iter().position(|byte| *byte == b'\n') {
            let end = start + offset;
            let line = String::from_utf8_lossy(&self.tail[start..end]).to_string();
            Self::take_line(&line, &mut out);
            start = end + 1;
        }
        self.tail.drain(..start);
        out
    }

    /// ： 1 。
    ///
    /// ── Why this ** need **（ and ReasoningCoalescer ）────────
    /// that tail will to down 1 can can ，，
    /// not 。 this not ： is ****， JSON will 
    /// `serde_json` and （ not will ），「 up after 1 
    /// no 」 this （`a` ） can —— this 1 ，
    /// after 1 will empty。
    pub fn finish(&mut self) -> Vec<Option<String>> {
        let mut out: Vec<Option<String>> = Vec::new();
        if self.tail.is_empty() {
            return out;
        }
        let line = String::from_utf8_lossy(&self.tail).to_string();
        self.tail.clear();
        Self::take_line(&line, &mut out);
        out
    }

    /// 1 ： `data:` ，//empty
    fn take_line(line: &str, out: &mut Vec<Option<String>>) {
        let line = line.strip_suffix('\r').unwrap_or(line);
        let Some(rest) = line.strip_prefix("data:") else {
            return; // event: / id: / 注释 / 空行都不带载荷
        };
        let data = rest.trim();
        if data.is_empty() {
            return;
        }
        if data == "[DONE]" {
            out.push(None);
            return;
        }
        out.push(Some(data.to_string()));
    }
}

// ─── （）────────────────────────────────
//
// ── ────────────────────────────────────────────
// Chat is ，** can **： down Anthropic 
// `cache_control`（）、`tool_result.is_error`（failure） and 
// down Responses reasoning `encrypted_content`（store=false 
// ） in Chat no has for ， when will 。
// 9Router in this is 「」（ cache_control、 is_error、
// and in Responses encrypted_content）， 1 。
//
// ── ────────────────────────────────────────────────────
// when this in ** Chat ** up （`_wb_` before ， down ），
// to when has ：
// · ****（`anthropic_outbound` / `responses_outbound`）：、
// to up for —— is new ， not ；
// · ****（、 chat ）： before use 
// [`strip_internal_fields`] 1 。
//
// Why is **must** not is ： OpenAI up will assistant
// up unknown and 400（9Router in Groq / Mistral up 
// `reasoning_content` this 1 ， stripContinuityFields ）。
// ，must 1 not to up to 。
//
// ：cache_control in Anthropic is **** ， Chat 
// content can can （`collapse_content`），none from ；
// this as ****（ this after 1 cache_control ）， when 
// this after 1 can up 。Claude Code in up ，
// this in ； ， is 。

/// cache_control （ for / for up ， = cache_control for ）
pub const FIELD_CACHE_CONTROL: &str = "_wb_cache_control";
/// tool_result is_error （role:"tool" up ，only in true when ）
pub const FIELD_IS_ERROR: &str = "_wb_is_error";
/// reasoning encrypted_content （assistant up ， = ）
pub const FIELD_ENCRYPTED_CONTENT: &str = "_wb_encrypted_content";

/// before ： by ** before ** not is —— will new when 
/// ， not will as to up 。
const INTERNAL_FIELD_PREFIX: &str = "_wb_";

/// request up （`_wb_` before ：、、 ）。
///
/// no has when return `Cow::Borrowed` （ —— this is request
/// ：chat not ）； has 。
pub fn strip_internal_fields(body: &Value) -> Cow<'_, Value> {
    let Some(object) = body.as_object() else {
        return Cow::Borrowed(body);
    };
    let top_dirty = object.keys().any(|key| key.starts_with(INTERNAL_FIELD_PREFIX));
    let messages_dirty = object
        .get("messages")
        .and_then(Value::as_array)
        .is_some_and(|messages| messages.iter().any(has_internal_fields));
    let tools_dirty = object
        .get("tools")
        .and_then(Value::as_array)
        .is_some_and(|tools| tools.iter().any(has_internal_fields));
    if !top_dirty && !messages_dirty && !tools_dirty {
        return Cow::Borrowed(body);
    }
    let mut next = object.clone();
    next.retain(|key, _| !key.starts_with(INTERNAL_FIELD_PREFIX));
    if let Some(messages) = next.get_mut("messages").and_then(Value::as_array_mut) {
        for message in messages.iter_mut() {
            strip_on_object(message);
        }
    }
    if let Some(tools) = next.get_mut("tools").and_then(Value::as_array_mut) {
        for tool in tools.iter_mut() {
            strip_on_object(tool);
        }
    }
    Cow::Owned(Value::Object(next))
}

/// 1 for up is 
fn has_internal_fields(value: &Value) -> bool {
    value
        .as_object()
        .is_some_and(|object| object.keys().any(|key| key.starts_with(INTERNAL_FIELD_PREFIX)))
}

/// 1 for up （ for in use ）
fn strip_on_object(value: &mut Value) {
    if let Some(object) = value.as_object_mut() {
        object.retain(|key, _| !key.starts_with(INTERNAL_FIELD_PREFIX));
    }
}
