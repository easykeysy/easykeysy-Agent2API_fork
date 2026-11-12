//! up **Antigravity / Gemini v1internal SSE** → chat SSE（）。
//!
//! from `antigravity_outbound.rs` （file）： that filerequest
//! ，fileresponse to 。
//! [`super::anthropic_outbound::ChatFromAnthropicStream`]：
//! `new(model)` / `push(&[u8]) -> Vec<Bytes>` / `finish() -> Vec<Bytes>`；
//! （`upstream::translate::AntigravityToChatStream`）empty、
//! and ， in `upstream::provider_loop`。
//!
//! ── （ §4.2， 1 ）──────────────────
//! up is v1internal ： Gemini response in `response` down ，
//! ****（ is this ：Manager
//! `response.get("response").unwrap_or(response)`、9router `chunk.response || chunk`）。
//!
//! ── parts （ for 9router `gemini-to-openai.js` and Manager
//! `mappers/openai/streaming.rs`， as ）────────────────
//!   - `.text` + `thought != true` → `delta.content`
//! - `.text` + `thought == true` → `delta.reasoning_content`（）
//! - `functionCall{name,args,id}` → ** 1 to ** `tool_calls` 
//! （ use is ， not is Anthropic that 「 + parameter」）
//! - `inlineData` / `inline_data`（response）→ chat not down 
//! （ no has images ），** and **（ already limit）
//! - 1 part（none text、none functionCall）→ to its after 
//! functionCall（9router `pendingThoughtSignature` ）
//!
//! ── `finishReason` / usage / when ────────────────────────
//! - 1 [`map_finish_reason`]； use and 1 is `stop` when 
//! `tool_calls`（9router/Manager ：otherwiseclient 「 need 」
//! for ）；
//! - `usageMetadata` → chat usage（9router `USAGE_EXTRACTORS.gemini` ：
//!     `completion = candidatesTokenCount + thoughtsTokenCount`，
//!     `cachedContentTokenCount` → `prompt_tokens_details.cached_tokens`）；
//! after 1 （）；
//! - = `finishReason` ** or ** `data: [DONE]`；EOF before 1 no to 
//! → by （error + `[DONE]`，** not ** finish）；error before not usage
//! （ usage and choices as empty is ， will client before as 
//! already —— and `ai_sdk_ndjson` ）。
//!
//! ── thoughtSignature and （issue #291）───────────────
//! up functionCall ** need **： → 400「missing a
//! thought_signature」（issue #237 is `extra_content` that ，
//! client path）， → 400「Corrupted thought signature」——
//! not default。 tool_call ， Google OpenAI
//! `extra_content.google.thought_signature`；** when 
//! server**（[`super::antigravity_signature_cache`]， that module header）
//! —— OpenAI client down 1 will `extra_content` ，request
//! （`antigravity_outbound` ） by ，client 
//! （`call_signature` has path）。 will from request [`Self::new`] 
//! （ ）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Map, Value};

use super::{chat_frame, is_truthy, json_text, random_id, string_field, string_value, SseLineBuffer};

/// up `finishReason` → chat `finish_reason`（9router
/// `toOpenAIFinish(reason, "gemini")` 1 + Manager 
/// `MALFORMED_FUNCTION_CALL`→stop）。 Reason `stop`（）。
fn map_finish_reason(raw: &str) -> String {
    match raw.trim().to_ascii_uppercase().as_str() {
        "MAX_TOKENS" => "length".to_string(),
        "SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" => "content_filter".to_string(),
        _ => "stop".to_string(),
    }
}

/// `usageMetadata` → chat usage（9router `USAGE_EXTRACTORS.gemini` ：
/// `candidatesTokenCount` when use `total - prompt - thoughts` ）。
fn usage_metadata(usage: &Value) -> Value {
    let number = |key: &str| usage.get(key).and_then(Value::as_i64).unwrap_or(0);
    let prompt = number("promptTokenCount");
    let thoughts = number("thoughtsTokenCount");
    let mut candidates = number("candidatesTokenCount");
    let total = number("totalTokenCount");
    if candidates == 0 && total > 0 {
        candidates = (total - prompt - thoughts).max(0);
    }
    let cached = number("cachedContentTokenCount");
    json!({
        "prompt_tokens": prompt,
        "completion_tokens": candidates + thoughts,
        "total_tokens": if total > 0 { total } else { prompt + candidates + thoughts },
        "prompt_tokens_details": { "cached_tokens": cached },
    })
}

/// up error → down `error.type`（ and `ai_sdk_ndjson` ）
fn error_type(status: u16) -> &'static str {
    match status {
        400 | 422 => "invalid_request_error",
        401 | 403 => "authentication_error",
        404 => "not_found",
        429 => "rate_limit_error",
        500 | 502 => "upstream_error",
        503 => "temporarily_unavailable",
        _ => "upstream_error",
    }
}

/// up Gemini SSE → chat SSE （）。
pub struct ChatFromAntigravityStream {
    buffer: SseLineBuffer,
    /// model（ to up ；ForwardStream by need ）
    model: String,
    id: String,
    created: i64,
    /// already down （role assistant）
    started: bool,
    finished: bool,
    /// （`finishReason` or `data: [DONE]`）
    saw_finish: bool,
    /// already 1 chat Reason
    finish_reason: Option<String>,
    /// use （`tool_calls[].index`， 1 ）
    tool_index: i64,
    /// down use （finish_reason `tool_calls` use ）
    tools_emitted: bool,
    /// 1 part to its after functionCall（9router ）
    pending_signature: Option<String>,
    /// response will `sessionId`（ ，module header；
    /// when from request， not to as empty）
    session: String,
    /// usage（ after 1 ， and ）
    usage: Option<Value>,
}

impl ChatFromAntigravityStream {
    /// `session` is request `sessionId`（ will ， 
    /// ； not to to empty —— ，）
    pub fn new(model: &str, session: &str) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            model: model.to_string(),
            id: String::new(),
            created: crate::server::logging::now_ms() / 1000,
            started: false,
            finished: false,
            saw_finish: false,
            finish_reason: None,
            tool_index: 0,
            tools_emitted: false,
            pending_signature: None,
            session: session.to_string(),
            usage: None,
        }
    }

    /// 1 up ， need down chat SSE 
    pub fn push(&mut self, chunk: &[u8]) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        for payload in self.buffer.push(chunk) {
            match payload {
                // `data: [DONE]`：（， not etc to EOF）
                None => out.extend(self.done()),
                Some(data) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&data) {
                        out.extend(self.consume(&value));
                    }
                }
            }
            if self.finished {
                break;
            }
        }
        out
    }

    /// up ：， by 「 has no has 」
    pub fn finish(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let mut out = Vec::new();
        for payload in self.buffer.finish() {
            match payload {
                None => out.extend(self.done()),
                Some(data) => {
                    if let Ok(value) = serde_json::from_str::<Value>(&data) {
                        out.extend(self.consume(&value));
                    }
                }
            }
            if self.finished {
                return out;
            }
        }
        if !self.saw_finish {
            // 1 no has ： up is 「 no 」， not can finish_reason
            return self.fail_message(
                "上游流在收到结束原因之前结束（响应被截断）",
                "upstream_error",
                502,
                None,
            );
        }
        let reason = self
            .finish_reason
            .clone()
            .unwrap_or_else(|| "stop".to_string());
        out.extend(self.complete(&reason));
        out
    }

    /// `data: [DONE]` to ：（ no has finishReason when by stop ）
    fn done(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.saw_finish = true;
        let reason = self
            .finish_reason
            .clone()
            .unwrap_or_else(|| "stop".to_string());
        self.complete(&reason)
    }

    /// 1 up JSON → to chat 
    fn consume(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // ： in `response` down ，（ §4.2）
        let inner = event
            .get("response")
            .filter(|value| value.is_object())
            .unwrap_or(event);
        if let Some(error) = inner.get("error").filter(|value| is_truthy(value)) {
            return self.fail(error);
        }
        if self.id.is_empty() {
            if let Some(id) = inner
                .get("responseId")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
            {
                self.id = id.to_string();
            }
        }
        if let Some(usage) = inner.get("usageMetadata").filter(|value| value.is_object()) {
            self.usage = Some(usage_metadata(usage));
        }
        let Some(candidate) = inner.pointer("/candidates/0") else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if let Some(parts) = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
        {
            for part in parts {
                out.extend(self.consume_part(part));
            }
        }
        if let Some(raw) = candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|reason| !reason.is_empty())
        {
            self.saw_finish = true;
            self.finish_reason = Some(map_finish_reason(raw));
        }
        out
    }

    /// 1 part → to chat 
    fn consume_part(&mut self, part: &Value) -> Vec<bytes::Bytes> {
        let Some(object) = part.as_object() else {
            return Vec::new();
        };
        let signature = object
            .get("thoughtSignature")
            .or_else(|| object.get("thought_signature"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|signature| !signature.is_empty())
            .map(str::to_string);
        let text = object.get("text").and_then(Value::as_str).unwrap_or("");
        let is_thought = object
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        // （9router `pendingThoughtSignature` ： part
        // can can is its after functionCall ）， down 1 functionCall 
        if signature.is_some() {
            self.pending_signature = signature;
        }
        if let Some(call) = object
            .get("functionCall")
            .and_then(Value::as_object)
            .filter(|call| !call.is_empty())
        {
            let signature = self.pending_signature.take();
            return self.tool_call_frames(&Value::Object(call.clone()), signature);
        }
        if text.is_empty() {
            if object
                .get("inlineData")
                .or_else(|| object.get("inline_data"))
                .is_some()
            {
                // chat not down response（ already limit）
                crate::server::logging::verbose(
                    "[Antigravity]",
                    "上游返回了 inlineData 图片，本仓 chat 形态无法承载（缺 images 通道），已跳过",
                );
            }
            return Vec::new();
        }
        let mut out = self.start();
        if is_thought {
            out.push(self.delta_frame(json!({ "reasoning_content": text })));
        } else {
            out.push(self.delta_frame(json!({ "content": text })));
        }
        out
    }

    /// `functionCall` → 1 to `tool_calls` （ use is ）
    fn tool_call_frames(&mut self, call: &Value, signature: Option<String>) -> Vec<bytes::Bytes> {
        let name = string_field(call, "name");
        let raw_id = string_field(call, "id").trim().to_string();
        let id = if raw_id.is_empty() {
            random_id("call")
        } else {
            raw_id
        };
        let arguments = match call.get("args") {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Null) | None => "{}".to_string(),
            Some(other) => json_text(other),
        };
        if let Some(signature) = signature.as_deref() {
            // server（issue #291，module header）：client down 1 not will 
            // `extra_content`，request by this 
            super::antigravity_signature_cache::remember_call(&self.session, &id, signature);
            super::antigravity_signature_cache::remember_for_session(&self.session, &name, signature);
        }
        let index = self.tool_index;
        self.tool_index += 1;
        self.tools_emitted = true;
        let mut tool = Map::new();
        tool.insert("index".to_string(), Value::from(index));
        tool.insert("id".to_string(), Value::String(id));
        tool.insert("type".to_string(), Value::String("function".to_string()));
        tool.insert(
            "function".to_string(),
            json!({ "name": name, "arguments": arguments }),
        );
        if let Some(signature) = signature {
            // tool_call （Google OpenAI ，
            // module header）；request 1 and 
            tool.insert(
                "extra_content".to_string(),
                json!({ "google": { "thought_signature": signature } }),
            );
        }
        let mut out = self.start();
        out.push(self.delta_frame(json!({ "tool_calls": [Value::Object(tool)] })));
        out
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

    /// ：finish + usage + `[DONE]`（ 1 ）。
    ///
    /// has finishReason（， SAFETY ） when ； 1 
    /// no has 「empty」 by emptyresponse（ and `ai_sdk_ndjson` 1 ：
    /// not can empty 1 success）。
    fn complete(&mut self, reason: &str) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        if !self.started && self.finish_reason.is_none() {
            return self.fail_message(
                "上游返回空响应（没有任何内容帧）",
                "rate_limit_error",
                429,
                None,
            );
        }
        self.finished = true;
        // use when stop tool_calls（9router/Manager ）
        let finish = match reason {
            "stop" if self.tools_emitted => "tool_calls",
            other => other,
        };
        let mut out = self.start();
        out.push(self.finish_frame(finish));
        out.push(chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [],
            "usage": self.usage_payload(),
        })));
        out.push(bytes::Bytes::from_static(b"data: [DONE]\n\n"));
        out
    }

    /// up error for → error + `[DONE]`（ 1 ）
    fn fail(&mut self, error: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let message = {
            let text = string_field(error, "message");
            if text.trim().is_empty() {
                string_value(error)
            } else {
                text
            }
        };
        let message = if message.trim().is_empty() {
            "上游流式返回错误".to_string()
        } else {
            message
        };
        let code = error
            .get("code")
            .and_then(Value::as_i64)
            .or_else(|| {
                let text = string_field(error, "code");
                text.trim().parse::<i64>().ok()
            });
        // Google in `error.status` （ §3.2 ），
        // code / when by 
        let status = code
            .and_then(|code| u16::try_from(code).ok())
            .filter(|status| (100..=599).contains(status))
            .unwrap_or_else(|| {
                match string_field(error, "status").to_ascii_uppercase().as_str() {
                    "RESOURCE_EXHAUSTED" => 429,
                    "PERMISSION_DENIED" | "UNAUTHENTICATED" => 401,
                    "NOT_FOUND" => 404,
                    "UNAVAILABLE" => 503,
                    "DEADLINE_EXCEEDED" | "INTERNAL" => 502,
                    _ => 502,
                }
            });
        crate::server::logging::verbose(
            "[Antigravity]",
            &format!(
                "上游流内错误：{message}（映射为 {status} {}）",
                error_type(status)
            ),
        );
        self.fail_message(&message, error_type(status), i64::from(status), code)
    }

    /// 「failure」 chat error + `[DONE]`（ 1 ）。
    ///
    /// and successpath 1 is ** not usage **（ by module header）。
    fn fail_message(
        &mut self,
        message: &str,
        kind: &str,
        status: i64,
        code: Option<i64>,
    ) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;
        let mut error = json!({
            "message": message,
            "type": kind,
            // `status` is ****： is and ForwardStream 
            // `{error:{message,type}}` ， this 1 client and can 
            // up is 429 is 503
            "status": status,
        });
        if let (Some(code), Some(object)) = (code, error.as_object_mut()) {
            object.insert("code".to_string(), Value::from(code));
        }
        let mut frame = json!({ "error": error });
        if status == 429 {
            // must retry_after，otherwise SDK not this etc （）
            if let Some(object) = frame.as_object_mut() {
                object.insert("retry_after".to_string(), Value::from(30));
            }
        }
        vec![
            chat_frame(&frame),
            bytes::Bytes::from_static(b"data: [DONE]\n\n"),
        ]
    }

    /// chat usage（ no has usageMetadata when to ， and ）
    fn usage_payload(&self) -> Value {
        self.usage.clone().unwrap_or_else(|| {
            json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0,
            })
        })
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
