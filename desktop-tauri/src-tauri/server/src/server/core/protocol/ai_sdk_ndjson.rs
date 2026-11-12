//! up **AI SDK v5 NDJSON** → chat SSE （）。
//!
//! ── this Why in ────────────────────────────────────
//! ** 1 **（Vercel AI SDK v5 UI message stream：
//! 1 1 JSON for 、none `data:` before 、 `text-delta` / `tool-call` /
//! `finish` etc ）， not 1 、 or request —— that in 。
//! in `chat` `UpstreamKind::Ndjson`， this 。
//!
//! ── ────────────────────────────────────────────────
//!   - `start` / `start-step` / `text-start` / `text-end` / `reasoning-start` /
//!     `reasoning-end` / `tool-input-start` / `tool-input-delta` / `tool-input-end` /
//! `tool-error` / `provider-metadata` → ，none use can ，ignore
//! - `text-delta` → `delta.content`（ `text`， `delta` ）
//!   - `reasoning-delta` → `delta.reasoning_content`
//! - `tool-call` → `tool_calls` （`id` / `name` / `arguments` 1 to ，
//! and Anthropic that 「 + parameter」 not ： this use is ****）
//! - `finish-step` → 「」+ finishReason/usage（** not **）
//! - `finish` → ****：finish + usage + `data: [DONE]`
//! - `error` → error（+ `[DONE]`）， by [`map_upstream_status`] down 
//! error.type（ 502；path to client）
//!
//! ── `finish` and [DONE] when ───────────────────────────
//! 1. ： 1 to —— `finish_reason` 、
//! usage 、`data: [DONE]`， after ；
//! 2. ** no has **（none `finish`、none `finish-step`） EOF：
//! by 「response 」（ can 502 ），** not ** finish_reason/[DONE]
//! —— that etc ；
//! 3. `finish-step`、 no has `finish`： by 「 up to 」
//! （ need is 「 1 no has 」）；
//! 4. `finish` `finishReason` 1 after has `upstream_error` 1 （ `error` /
//! `network|connection|upstream-error`）： is ** can failure** not is 
//! finish_reason，error not is （ will client
//! failure）。
//!
//! ── error `error.type` ────────────────────────────────────
//! use [`map_upstream_status`] （`rate_limit_error` /
//! `authentication_error` / …） not is 1 `upstream_error`：NDJSON 
//! `error` `statusCode` / `code`， this 1 in can 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Value};

use super::{chat_frame, is_truthy, json_text, random_id, string_field, string_value};

/// OpenAI `finish_reason` （ `OPENAI_FINISH_REASONS`）。
const OPENAI_FINISH_REASONS: [&str; 5] = [
    "stop",
    "length",
    "tool_calls",
    "content_filter",
    "function_call",
];

/// up error → down `(status, error.type)`。
///
/// 1 502 `upstream_error`。
/// 402（failure） 429、403（/） 401 is ** has ** down 
/// （ down by / ， not is server）。
fn map_upstream_status(status: Option<i64>) -> (i64, &'static str) {
    match status {
        Some(400) | Some(422) => (400, "invalid_request_error"),
        Some(401) | Some(403) => (401, "authentication_error"),
        // 402 failure → by （/ is use ， down this ）
        Some(402) => (429, "rate_limit_error"),
        Some(404) => (404, "not_found"),
        Some(429) => (429, "rate_limit_error"),
        Some(500) | Some(502) => (502, "upstream_error"),
        Some(503) => (503, "temporarily_unavailable"),
        _ => (502, "upstream_error"),
    }
}

/// up finishReason → （ `mapFinishReason`）。
///
/// `length` ** not `length` 1 **：`max_output_tokens` and 
/// `model_context_window_exceeded` is 「 」， stop etc 
/// 。 `error` and `network|connection|upstream-error` 1 and 
/// `upstream_error`（ can failure， not is finish_reason）。
fn map_finish_reason(reason: &str) -> String {
    let value = reason.trim().to_lowercase();
    if value.is_empty() {
        return "stop".to_string();
    }
    match value.as_str() {
        "tool-calls" | "tool_calls" | "tool_use" => "tool_calls".to_string(),
        "length" | "max_tokens" | "max_output_tokens" | "model_context_window_exceeded" => {
            "length".to_string()
        }
        "error" => "upstream_error".to_string(),
        other => {
            if is_network_failure_finish(other) {
                "upstream_error".to_string()
            } else {
                other.to_string()
            }
        }
    }
}

/// `/^(network|connection|upstream)[-_\s]?error$/` Rust （ not use regex ）
fn is_network_failure_finish(value: &str) -> bool {
    for prefix in ["network", "connection", "upstream"] {
        let Some(rest) = value.strip_prefix(prefix) else {
            continue;
        };
        let rest = rest.trim_start_matches(['-', '_', ' ']);
        if rest == "error" {
            return true;
        }
    }
    false
}

/// → down `finish_reason`（ `toOpenAIFinishReason`）。
///
/// `pause_turn`（Anthropic 「 after has 」） in Chat `length`；
/// not Reason `stop`（ up finish， is Reason not in ——
/// not can ）；`upstream_error` error， not from this 。
fn to_openai_finish_reason(reason: &str) -> String {
    if reason == "pause_turn" {
        return "length".to_string();
    }
    if OPENAI_FINISH_REASONS.contains(&reason) {
        return reason.to_string();
    }
    "stop".to_string()
}

/// NDJSON ： up by TCP to ， 1 JSON can can 。
///
/// and [`super::SseLineBuffer`] has 1 ：NDJSON ** no has `data:` before **，
/// is JSON； this not before ， by `\n` and in EOF when 
/// （ up after 1 no is in ， can ）。
#[derive(Default)]
struct NdjsonLineBuffer {
    tail: Vec<u8>,
}

impl NdjsonLineBuffer {
    fn new() -> Self {
        Self { tail: Vec::new() }
    }

    /// 1 ， its in **** （ already by empty / `[DONE]` / ）
    fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut out = Vec::new();
        self.tail.extend_from_slice(chunk);
        let mut start = 0usize;
        while let Some(offset) = self.tail[start..].iter().position(|byte| *byte == b'\n') {
            let end = start + offset;
            let line = String::from_utf8_lossy(&self.tail[start..end]).to_string();
            if let Some(line) = keep_line(&line) {
                out.push(line);
            }
            start = end + 1;
        }
        self.tail.drain(..start);
        out
    }

    /// ： 1 （ and [`Self::push`] ）
    fn finish(&mut self) -> Vec<String> {
        if self.tail.is_empty() {
            return Vec::new();
        }
        let line = String::from_utf8_lossy(&self.tail).to_string();
        self.tail.clear();
        keep_line(&line).into_iter().collect()
    }
}

/// 1 is need to （empty / `[DONE]` / `:` not is ）
fn keep_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed == "[DONE]" || trimmed.starts_with(':') {
        return None;
    }
    Some(trimmed.to_string())
}

/// up NDJSON → chat SSE （）。
pub struct ChatFromAiSdkNdjsonStream {
    buffer: NdjsonLineBuffer,
    /// model（ to up ；ForwardStream by need ）
    model: String,
    id: String,
    created: i64,
    /// already down （role assistant）—— when 「 up to 」
    started: bool,
    finished: bool,
    /// （ `finish` or `finish-step`）
    saw_finish: bool,
    /// `finish-step` Reason（ after ；`finish` when use ）
    step_finish_reason: Option<String>,
    /// use （`tool_calls[].index`， 1 ）
    tool_index: i64,
    /// usage（`finish-step` `usage` → `finish` `totalUsage` ）
    usage: Option<Value>,
}

impl ChatFromAiSdkNdjsonStream {
    pub fn new(model: &str) -> Self {
        Self {
            buffer: NdjsonLineBuffer::new(),
            model: model.to_string(),
            id: String::new(),
            created: crate::server::logging::now_ms() / 1000,
            started: false,
            finished: false,
            saw_finish: false,
            step_finish_reason: None,
            tool_index: 0,
            usage: None,
        }
    }

    /// 1 up ， need down chat SSE 
    pub fn push(&mut self, chunk: &[u8]) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        for line in self.buffer.push(chunk) {
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                out.extend(self.consume(&value));
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
        for line in self.buffer.finish() {
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                out.extend(self.consume(&value));
            }
            if self.finished {
                return out;
            }
        }
        if !self.saw_finish {
            // 1 no has ： up is 「 no 」， not can finish_reason
            out.extend(self.fail_message(
                "上游流在收到完成事件之前结束（响应被截断）",
                "upstream_error",
                502,
                None,
            ));
            return out;
        }
        let reason = self
            .step_finish_reason
            .clone()
            .unwrap_or_else(|| "stop".to_string());
        out.extend(self.complete(&reason));
        out
    }

    /// 1 NDJSON → to chat 
    fn consume(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let Some(object) = event.as_object() else {
            return Vec::new();
        };
        let kind = string_field(event, "type").to_lowercase();
        match kind.as_str() {
            // ：none use can （ and for ）
            "start" | "start-step" | "text-start" | "text-end" | "reasoning-start"
            | "reasoning-end" | "tool-input-start" | "tool-input-delta" | "tool-input-end"
            | "tool-error" | "provider-metadata" => Vec::new(),
            "text-delta" => {
                // ：`text` as 、 `delta`
                let text = {
                    let primary = string_field(event, "text");
                    if primary.is_empty() {
                        string_field(event, "delta")
                    } else {
                        primary
                    }
                };
                if text.is_empty() {
                    return Vec::new();
                }
                let mut out = self.start();
                out.push(self.delta_frame(json!({ "content": text })));
                out
            }
            "reasoning-delta" => {
                // ： `text`（ and 1 ， not `delta`）
                let text = string_field(event, "text");
                if text.is_empty() {
                    return Vec::new();
                }
                let mut out = self.start();
                out.push(self.delta_frame(json!({ "reasoning_content": text })));
                out
            }
            "tool-call" => {
                // use is ****（id / name / parameter 1 to ）
                let raw_id = string_field(event, "toolCallId");
                let id = if raw_id.is_empty() {
                    random_id("call")
                } else {
                    raw_id
                };
                let name = string_field(event, "toolName");
                let input = object.get("input").cloned().unwrap_or(Value::Null);
                let arguments = match &input {
                    Value::String(text) => text.clone(),
                    Value::Null => "{}".to_string(),
                    other => json_text(other),
                };
                let index = self.tool_index;
                self.tool_index += 1;
                let mut out = self.start();
                out.push(self.delta_frame(json!({
                    "tool_calls": [{
                        "index": index,
                        "id": id,
                        "type": "function",
                        "function": { "name": name, "arguments": arguments },
                    }],
                })));
                out
            }
            "finish-step" => {
                // ：「」， by finish or when EOF 
                self.saw_finish = true;
                if let Some(reason) = object
                    .get("finishReason")
                    .filter(|value| is_truthy(value))
                    .map(string_value)
                    .filter(|value| !value.trim().is_empty())
                {
                    self.step_finish_reason = Some(map_finish_reason(&reason));
                }
                if let Some(usage) = object.get("usage").filter(|usage| usage.is_object()) {
                    self.usage = Some(normalized_usage(usage));
                }
                Vec::new()
            }
            "finish" => {
                self.saw_finish = true;
                // ：finish-step finishReason ， its finish 
                let reason = match self.step_finish_reason.clone() {
                    Some(reason) => reason,
                    None => {
                        let raw = string_field(event, "finishReason");
                        if raw.trim().is_empty() {
                            "stop".to_string()
                        } else {
                            map_finish_reason(&raw)
                        }
                    }
                };
                if let Some(usage) = object
                    .get("totalUsage")
                    .or_else(|| object.get("usage"))
                    .filter(|usage| usage.is_object())
                {
                    self.usage = Some(normalized_usage(usage));
                }
                // `upstream_error` 1 （ `error` / `network|connection|upstream-error`）
                // by `complete` 1 can error —— not is finish_reason
                self.complete(&reason)
            }
            "error" => self.fail(event),
            other => {
                // unknown：ignore not —— if ， is 1 
                crate::server::logging::verbose(
                    "[AI-SDK-NDJSON]",
                    &format!("上游发了未知的 NDJSON 事件类型：{other}"),
                );
                Vec::new()
            }
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

    /// ：finish + usage + `[DONE]`（ 1 ）。
    ///
    /// `reason` is ** 1 after **（`map_finish_reason` ）：
    /// `upstream_error` in this can error（ not is finish_reason）；
    /// its `to_openai_finish_reason` OpenAI 。
    ///
    /// up 1 no to to this （ has finish / finish-step） when by emptyresponse
    /// ： that is 1 「success no has 」 ，success will down by empty。
    fn complete(&mut self, reason: &str) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // is （ and ）：**failure，** ——
        // provider failure when 「」 is ， by 502 to Reason
        if reason == "upstream_error" {
            return self.fail_message(
                "上游报告连接失败（provider reported an upstream connection failure）",
                "upstream_error",
                502,
                None,
            );
        }
        if !self.started {
            return self.fail_message(
                "上游返回空响应（没有任何内容事件）",
                "rate_limit_error",
                429,
                None,
            );
        }
        self.finished = true;
        let mut out = self.start();
        out.push(self.finish_frame(&to_openai_finish_reason(reason)));
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

    /// up error → error + `[DONE]`（ 1 ）
    ///
    /// （ `mapCcEventError`）：message `<NNN>` before 
    /// ， its `error.statusCode`， no has 502 —— not to will 
    /// 429/503 502， down not 、 after 。
    fn fail(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let error = event.get("error").filter(|value| is_truthy(value));
        // and ：`error.message` → `event.message` → 
        let message = match error {
            Some(error) => {
                let text = string_field(error, "message");
                if text.trim().is_empty() {
                    string_field(event, "message")
                } else {
                    text
                }
            }
            None => string_field(event, "message"),
        };
        let message = if message.trim().is_empty() {
            "上游流式返回错误".to_string()
        } else {
            message
        };
        // （ `mapCcEventError`）：message `<NNN>` before 
        // ， its `error.statusCode`， no has 502
        let reported = error.and_then(|error| {
            embedded_status(&message).or_else(|| error.get("statusCode").and_then(Value::as_i64))
        });
        // can ：`error.code` → `event.code`
        let code = error
            .and_then(|error| {
                error
                    .get("code")
                    .filter(|value| is_truthy(value))
                    .or_else(|| event.get("code").filter(|value| is_truthy(value)))
            })
            .map(string_value)
            .filter(|value| !value.is_empty());
        let (status, kind) = map_upstream_status(reported);
        crate::server::logging::verbose(
            "[AI-SDK-NDJSON]",
            &format!(
                "上游流内错误：{message}（上游状态 {}，映射为 {status} {kind}）",
                reported
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string())
            ),
        );
        self.fail_message(&message, kind, status, code.as_deref())
    }

    /// 「failure」 chat error + `[DONE]`（ 1 ）
    fn fail_message(
        &mut self,
        message: &str,
        kind: &str,
        status: i64,
        code: Option<&str>,
    ) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        self.finished = true;
        // ── Whyerror before ** not ** usage （ and successpath 1 ）──────
        // `usage` and `choices` as empty is OpenAI ****，
        // client「 already 」—— in after error etc that client
        // not to failure， 1 error 1 empty。errorpath
        // 「error + [DONE]」（ and `ForwardStream` 、Anthropic that 
        // `fail_message` ）。
        let mut error = json!({
            "message": message,
            "type": kind,
            // `status` is ****： is and ForwardStream 
            // `{error:{message,type}}` ， this 1 client and can 
            // up is 429 is 503（ down HTTP in path up already 200）
            "status": status,
        });
        if let Some(code) = code {
            if let Some(object) = error.as_object_mut() {
                object.insert("code".to_string(), Value::String(code.to_string()));
            }
        }
        let mut frame = json!({ "error": error });
        if status == 429 {
            // must retry_after，otherwise SDK not this etc （）
            if let Some(object) = frame.as_object_mut() {
                object.insert("retry_after".to_string(), Value::from(30));
            }
        }
        let mut out = vec![chat_frame(&frame)];
        out.push(bytes::Bytes::from_static(b"data: [DONE]\n\n"));
        out
    }

    /// chat usage（「outputTokens as 0 when input/cached」 then ）
    fn usage_payload(&self) -> Value {
        match self.usage.clone() {
            Some(usage) => usage,
            None => json!({
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0,
            }),
        }
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

/// message `<NNN>` before （ `parseEmbeddedErrorJSON(message)?.status`）
fn embedded_status(message: &str) -> Option<i64> {
    let rest = message.trim().strip_prefix('<')?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if digits.len() != 3 {
        return None;
    }
    digits.parse::<i64>().ok()
}

/// up usage → chat （ `normalizeUsage` + usage ）。
///
/// `inputTokens` is **total（）**，chat `prompt_tokens` ，
/// can use ； `prompt_tokens_details.cached_tokens`
/// （requeststatistics 「」 by ）。
/// `outputTokens === 0` when input/cached： up when 0 not this 
/// 1 （ then ）。
fn normalized_usage(usage: &Value) -> Value {
    let input = number_of(usage, &["inputTokens"]);
    let mut output = number_of(usage, &["outputTokens", "completionTokens"]);
    let mut cached = number_of(usage, &["cachedInputTokens", "cacheReadTokens"]);
    let mut input = input;
    if output == 0 {
        input = 0;
        cached = 0;
    }
    if output < 0 {
        output = 0;
        input = 0;
        cached = 0;
    }
    json!({
        "prompt_tokens": input,
        "completion_tokens": output,
        "total_tokens": input + output,
        "prompt_tokens_details": { "cached_tokens": cached },
    })
}

/// from for by 1 （ / to 0）
fn number_of(value: &Value, keys: &[&str]) -> i64 {
    for key in keys {
        if let Some(number) = value.get(*key).and_then(Value::as_i64) {
            return number;
        }
        if let Some(number) = value
            .get(*key)
            .and_then(Value::as_f64)
            .filter(|number| number.is_finite())
        {
            return number as i64;
        }
    }
    0
}
