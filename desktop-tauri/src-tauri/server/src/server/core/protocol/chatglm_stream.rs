//! up SSE（`parts` ）→ chat SSE （）。
//!
//! ── Why need need new 1 （ not is use 1 ）──────────────
//! is SSE（`data: {...}` ），** and OpenAI not **：
//!
//! ```text
//! ： to `parts:[{id, logic_id, status, content:[{type,text|think}]}]`
//! OpenAI ： to 「 this 1 delta」
//! {choices:[{delta:{content:"add "}, index:0}]}
//! ```
//!
//! ── （live ，2026-10）─────────────────────────
//! 1 ****， `status` ，** not can 1 by **
//! （ that is 「AI 」 ， `advance` ）：
//!
//! ```text
//! [1] top=init part=init text="1" ← ： is add
//! [2] top=init part=init text="\n" ← 
//! [3] top=init part=init text="2\n3" ← 
//! [4] top=init part=finish text="1\n2\n3" ← （Note part finish）
//! [5] top=finish part=finish text="1\n2\n3" ← + 
//! ```
//!
//! then is ：**`part.status == "init"` down and ；
//! finish （ or part） to is ， after after 。**
//! ：23 init = 157 ， and 157 。
//!
//! ── （`logic_id`） and ──────────────────────────────
//! 1 is **** ， 1 `logic_id`（： → →
//! → ）。： new if and old for ， will 
//! 「 for not up 」。 has 「」（`meta_data.engine.action == "loading"`，
//! is 「 in need ～」 this ）—— client will ，
//! 1 down not ，（ `is_loading_placeholder`）。
//!
//! 1 can can part： by ****（all `text` by 1 ），
//! as down chat has 1 。`think` `reasoning_content` ，
//! and 。
//!
//! ── ────────────────────────────────────────────────
//! `status == "finish"`（） as ： `finish_reason` + usage +
//! `data: [DONE]`。 up ** not `[DONE]`**（）， by this 。
//! 1 no to → by emptyresponse（ not emptysuccess）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Value};

use super::{chat_frame, content_parts, json_number_of, random_id, string_field, SseLineBuffer};

/// up `parts[*].content[*]` ， `type` / 
const KIND_TEXT: &str = "text";
const KIND_THINK: &str = "think";

/// up SSE → chat SSE （）。
pub struct ChatFromChatglmStream {
    buffer: SseLineBuffer,
    /// model（ to up ；ForwardStream by need ）
    model: String,
    id: String,
    created: i64,
    /// already down （role assistant）
    started: bool,
    finished: bool,
    /// current key（ up `logic_id`， when `part.id`）。
    ///
    /// up 1 is **** （：` → → → `，
    /// 1 `logic_id`）。 up 1 、 new 1 ，
    /// must，otherwise new will and old for 。
    segment_key: String,
    /// **current** already down （）
    segment_text: String,
    /// **current** already down （）
    segment_thinking: String,
    /// up after 1 to will id（，）
    conversation_id: String,
    /// up after 1 to error（ if has ）
    upstream_error: Option<String>,
    /// usage（ up not to token ， as empty —— `usage_payload`）
    usage: Option<Value>,
}

impl ChatFromChatglmStream {
    pub fn new(model: &str) -> Self {
        Self {
            buffer: SseLineBuffer::new(),
            model: model.to_string(),
            id: String::new(),
            created: crate::server::logging::now_ms() / 1000,
            started: false,
            finished: false,
            segment_key: String::new(),
            segment_text: String::new(),
            segment_thinking: String::new(),
            conversation_id: String::new(),
            upstream_error: None,
            usage: None,
        }
    }

    /// 1 up ， need down chat SSE 
    pub fn push(&mut self, chunk: &[u8]) -> Vec<bytes::Bytes> {
        let mut out = Vec::new();
        for payload in self.buffer.push(chunk) {
            // `None` = up `[DONE]`（ not ， down not ）
            let Some(line) = payload else {
                out.extend(self.finish());
                continue;
            };
            // is `HEARTBEAT`（** not **）， not is JSON
            if line.trim().eq_ignore_ascii_case("HEARTBEAT") {
                continue;
            }
            match serde_json::from_str::<Value>(&line) {
                Ok(value) => out.extend(self.consume(&value)),
                Err(error) => {
                    crate::server::logging::verbose(
                        "[ChatGLM]",
                        &format!("上游 SSE 载荷不是合法 JSON（已忽略）：{error}"),
                    );
                }
            }
            if self.finished {
                break;
            }
        }
        out
    }

    /// up ： by 「 has no has 」
    pub fn finish(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // （ up can can after 1 not ）
        let mut out = Vec::new();
        for payload in self.buffer.finish() {
            if let Some(line) = payload {
                if line.trim().eq_ignore_ascii_case("HEARTBEAT") {
                    continue;
                }
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    out.extend(self.consume(&value));
                }
            }
            if self.finished {
                return out;
            }
        }
        if self.finished {
            return out;
        }
        // no has EOF： up is 「 no 」。** 1 
        // `status:"finish"`**， to this can can is —— ，
        // not finish_reason（ that etc ）。
        out.extend(self.fail_message("上游流在收到完成信号之前结束（响应被截断）", 502));
        out
    }

    /// 1 → to chat 
    fn consume(&mut self, event: &Value) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        let Some(object) = event.as_object() else {
            return Vec::new();
        };
        // will id（ to ， after ； after 1 ）
        let conversation_id = string_field(event, "conversation_id");
        if !conversation_id.is_empty() {
            self.conversation_id = conversation_id;
        }
        // error： failure in 200 SSE （ -1001 not ），
        // is `error_code` + `error`（ `engine/stream` ）
        if let Some(code) = object.get("error_code").and_then(Value::as_i64) {
            if code != 0 {
                let message = string_field(event, "error");
                let message = if message.trim().is_empty() {
                    format!("上游返回错误码 {code}")
                } else {
                    message
                };
                self.upstream_error = Some(message.clone());
                return self.fail_with_code(&message, code);
            }
        }
        if object.get("error").is_some_and(|value| {
            !value.is_null() && !value.as_str().map(str::trim).unwrap_or("x").is_empty()
        }) {
            let message = string_field(event, "error");
            self.upstream_error = Some(message.clone());
            return self.fail_with_code(&message, 502);
        }

        // / ： + is 「」 is 「」（ `advance`）
        let (full_text, full_thinking) = collect_text(event);
        let is_snapshot = frame_is_snapshot(event);
        let (text_delta, thinking_delta) =
            self.advance(event, &full_text, &full_thinking, is_snapshot);

        // ： status == "finish"（ finish， its part）
        let frame_status = string_field(event, "status").to_lowercase();
        let is_finish = frame_status == "finish" || frame_status == "finished";

        let mut out = Vec::new();
        // （）
        if !text_delta.is_empty() || !thinking_delta.is_empty() {
            out.extend(self.start());
            let mut delta = serde_json::Map::new();
            if !thinking_delta.is_empty() {
                delta.insert(
                    "reasoning_content".to_string(),
                    Value::String(thinking_delta),
                );
            }
            if !text_delta.is_empty() {
                delta.insert("content".to_string(), Value::String(text_delta));
            }
            out.push(self.delta_frame(Value::Object(delta)));
        }
        if is_finish {
            out.extend(self.complete());
        }
        out
    }

    /// → need down `(, )`， and current 。
    ///
    /// ── （， for ）──────────────────────────
    /// 1. ****（`part.meta_data.engine.action == "loading"`）：
    /// up in before 1 （：「 for down 
    /// and ， in need ～」）。** not is 1 ** —— client
    /// is 。 1 down not ，。
    /// 2. ****（`part.status == "init"`）： is add 1 ，
    /// down and 。
    /// 3. ****（`part.status` / `status` as finish）： to is 
    /// ****，「 no down 」 that 1 。
    ///
    /// ── Why 3 not can before that " for not up "（）──────
    /// for 1 before ， to is 、 and before 1 for not up ，
    /// is ** down 1 ** —— use to is 「AI 」。
    /// for not up can can is up /， that down 1 
    /// （ is can ， and down none already ）。
    fn advance(
        &mut self,
        event: &Value,
        full_text: &str,
        full_thinking: &str,
        is_snapshot: bool,
    ) -> (String, String) {
        if is_loading_placeholder(event) {
            return (String::new(), String::new());
        }
        // （logic_id ）⇒ new 1 ，
        let key = segment_key_of(event);
        if !key.is_empty() && key != self.segment_key {
            self.segment_key = key;
            self.segment_text.clear();
            self.segment_thinking.clear();
        }
        let text = advance_segment(&mut self.segment_text, full_text, is_snapshot);
        let thinking = advance_segment(&mut self.segment_thinking, full_thinking, is_snapshot);
        (text, thinking)
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

    /// ：finish + usage + `[DONE]`
    fn complete(&mut self) -> Vec<bytes::Bytes> {
        if self.finished {
            return Vec::new();
        }
        // 1 no to ： this is 「success no has 」 ，
        // success will down by empty（ and AI SDK NDJSON that ）
        if !self.started {
            return self.fail_message("上游返回空响应（没有任何内容）", 429);
        }
        self.finished = true;
        let mut out = self.start();
        out.push(self.finish_frame("stop"));
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

    /// error → error + `[DONE]`（ 1 ）
    fn fail_with_code(&mut self, message: &str, code: i64) -> Vec<bytes::Bytes> {
        // -1001「remaining not 」 is failure： is **** not is 
        // ， to 429（ down / ， 502 has use ）
        let status = if code == -1001 { 429 } else { 502 };
        let kind = if status == 429 {
            "rate_limit_error"
        } else {
            "upstream_error"
        };
        self.fail_message_with_kind(message, kind, status, Some(code))
    }

    fn fail_message(&mut self, message: &str, status: i64) -> Vec<bytes::Bytes> {
        let kind = if status == 429 {
            "rate_limit_error"
        } else {
            "upstream_error"
        };
        self.fail_message_with_kind(message, kind, status, None)
    }

    /// 1 failure（error + `[DONE]`， 1 ）。
    ///
    /// errorpath** not ** usage ： `usage` and `choices` as empty is OpenAI
    /// ，client「 already 」—— in after error
    /// etc that client not to failure（ and AI SDK NDJSON that ）。
    fn fail_message_with_kind(
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
            "status": status,
        });
        if let Some(code) = code {
            if let Some(object) = error.as_object_mut() {
                object.insert("code".to_string(), Value::from(code));
            }
        }
        let mut frame = json!({ "error": error });
        if status == 429 {
            if let Some(object) = frame.as_object_mut() {
                object.insert("retry_after".to_string(), Value::from(30));
            }
        }
        let mut out = vec![chat_frame(&frame)];
        out.push(bytes::Bytes::from_static(b"data: [DONE]\n\n"));
        out
    }

    /// up （） not to token —— 0， not 
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

    /// 1 chat 
    fn delta_frame(&self, delta: Value) -> bytes::Bytes {
        chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": Value::Null }],
        }))
    }

    /// 
    fn finish_frame(&self, reason: &str) -> bytes::Bytes {
        chat_frame(&json!({
            "id": self.id,
            "object": "chat.completion.chunk",
            "created": self.created,
            "model": self.model,
            "choices": [{ "index": 0, "delta": {}, "finish_reason": reason }],
        }))
    }
}

/// 1 for **current already down ** ， and 。
///
/// - `is_snapshot == false`（）： is new ， down ；
/// - `is_snapshot == true`（）： already down as before ， after ；
/// for not up （ up /） when ** not ** —— is 「」（ `advance`）。
fn advance_segment(emitted: &mut String, frame: &str, is_snapshot: bool) -> String {
    if frame.is_empty() {
        return String::new();
    }
    if !is_snapshot {
        emitted.push_str(frame);
        return frame.to_string();
    }
    // `emitted` as empty when `strip_prefix` return， new 1 will down 
    match frame.strip_prefix(emitted.as_str()) {
        Some(rest) => {
            emitted.push_str(rest);
            rest.to_string()
        }
        None => String::new(),
    }
}

/// is not is 「」。
///
/// ：`part.status` will `status` finish（ from 1 to 5 ，
/// 6 `top=init / part=finish` and already is ）， need 。
fn frame_is_snapshot(event: &Value) -> bool {
    if is_finish_status(&string_field(event, "status")) {
        return true;
    }
    event
        .get("parts")
        .and_then(Value::as_array)
        .is_some_and(|parts| {
            !parts.is_empty()
                && parts
                    .iter()
                    .all(|part| is_finish_status(&string_field(part, "status")))
        })
}

fn is_finish_status(status: &str) -> bool {
    matches!(status.to_lowercase().as_str(), "finish" | "finished")
}

/// is not is 「 before when 」（：`engine.action == "loading"`）。
///
/// this has `logic_id`（ and not ）， is 1 。** not can down **：
/// client will ， down to is ， is 1 。
fn is_loading_placeholder(event: &Value) -> bool {
    event
        .get("parts")
        .and_then(Value::as_array)
        .is_some_and(|parts| {
            parts.iter().any(|part| {
                part.pointer("/meta_data/engine/action")
                    .and_then(Value::as_str)
                    == Some("loading")
            })
        })
}

/// key：`logic_id` ， `part.id`。
///
/// no has （ up ） when returnempty， use by 「 not 」 ——
/// can use old ， not need in to empty when （ that will new down 1 ）。
fn segment_key_of(event: &Value) -> String {
    let Some(parts) = event.get("parts").and_then(Value::as_array) else {
        return String::new();
    };
    for part in parts {
        let logic = string_field(part, "logic_id");
        if !logic.is_empty() {
            return logic;
        }
        let id = string_field(part, "id");
        if !id.is_empty() {
            return id;
        }
    }
    String::new()
}

/// 1 all part text / think by 。
///
/// return `(, )`。
fn collect_text(event: &Value) -> (String, String) {
    let mut text = String::new();
    let mut thinking = String::new();
    let Some(parts) = event.get("parts").and_then(Value::as_array) else {
        return (text, thinking);
    };
    for part in parts {
        let content = part.get("content").unwrap_or(&Value::Null);
        for item in content_parts(content) {
            let kind = string_field(item, "type").to_lowercase();
            if kind == KIND_TEXT {
                text.push_str(&string_field(item, "text"));
            } else if kind == KIND_THINK {
                thinking.push_str(&string_field(item, "think"));
            }
        }
    }
    (text, thinking)
}

/// ：usage 1 （ not use ， to will up token when ）
#[allow(dead_code)]
fn normalized_usage(value: &Value) -> Value {
    let prompt = json_number_of(value, &["prompt_tokens", "input_tokens"]);
    let completion = json_number_of(value, &["completion_tokens", "output_tokens"]);
    let total = json_number_of(value, &["total_tokens"]).max(prompt + completion);
    json!({
        "prompt_tokens": prompt,
        "completion_tokens": completion,
        "total_tokens": total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// down (, ) 
    fn collect(frames: &[bytes::Bytes]) -> (String, String, Option<String>) {
        let mut text = String::new();
        let mut thinking = String::new();
        let mut finish = None;
        for frame in frames {
            let raw = String::from_utf8_lossy(frame);
            let Some(payload) = raw.trim().strip_prefix("data: ") else {
                continue;
            };
            if payload == "[DONE]" {
                continue;
            }
            let Ok(value) = serde_json::from_str::<Value>(payload) else {
                continue;
            };
            let Some(choice) = value
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|list| list.first())
            else {
                continue;
            };
            if let Some(delta) = choice.get("delta") {
                if let Some(content) = delta.get("content").and_then(Value::as_str) {
                    text.push_str(content);
                }
                if let Some(reason) = delta.get("reasoning_content").and_then(Value::as_str) {
                    thinking.push_str(reason);
                }
            }
            if let Some(reason) = choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
            {
                finish = Some(reason.to_string());
            }
        }
        (text, thinking, finish)
    }

    fn sse(payload: &str) -> Vec<u8> {
        format!("data: {payload}\n\n").into_bytes()
    }

    /// **： not can down **（ use 「AI 」）。
    ///
    /// live ： before `part=init` is ****， to ****。
    /// old by before ， and before 1 for not up 「」，
    /// is down 1 —— is 。
    #[test]
    fn deltas_then_finish_snapshot_do_not_repeat_the_answer() {
        let mut machine = ChatFromChatglmStream::new("glm-5.3-flash");
        let mut frames = Vec::new();
        for text in ["1", "\n", "2\n3", "\n4\n", "5"] {
            frames.extend(machine.push(&sse(&format!(
                r#"{{"id":"m","conversation_id":"c","status":"init","parts":[{{"logic_id":"s","status":"init","content":[{{"type":"text","text":"{}"}}]}}]}}"#,
                text.replace('\n', "\\n")
            ))));
        }
        // 1 ：part finish， this 1 already is 
        frames.extend(machine.push(&sse(
            r#"{"id":"m","conversation_id":"c","status":"init","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"1\n2\n3\n4\n5"}]}]}"#,
        )));
        // ：， 1 
        frames.extend(machine.push(&sse(
            r#"{"id":"m","conversation_id":"c","status":"finish","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"1\n2\n3\n4\n5"}]}]}"#,
        )));
        let (text, _, finish) = collect(&frames);
        assert_eq!(text, "1\n2\n3\n4\n5", "完整回答只能出现一次");
        assert_eq!(finish.as_deref(), Some("stop"));
    }

    /// ： is ， not 1 （ old this ）
    #[test]
    fn whole_answer_in_first_delta_is_not_repeated() {
        let mut machine = ChatFromChatglmStream::new("m");
        let mut frames = machine.push(&sse(
            r#"{"id":"m","status":"init","parts":[{"logic_id":"s","status":"init","content":[{"type":"text","text":"AAA"}]}]}"#,
        ));
        frames.extend(machine.push(&sse(
            r#"{"id":"m","status":"init","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"AAA"}]}]}"#,
        )));
        frames.extend(machine.push(&sse(
            r#"{"id":"m","status":"finish","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"AAA"}]}]}"#,
        )));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "AAA");
    }

    /// when ， that 1 
    #[test]
    fn finish_snapshot_only_emits_the_missing_tail() {
        let mut machine = ChatFromChatglmStream::new("m");
        for text in ["前", "半"] {
            machine.push(&sse(&format!(
                r#"{{"status":"init","parts":[{{"logic_id":"s","status":"init","content":[{{"type":"text","text":"{text}"}}]}}]}}"#
            )));
        }
        let frames = machine.push(&sse(
            r#"{"status":"finish","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"前半加尾"}]}]}"#,
        ));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "加尾");
    }

    /// （`engine.action == "loading"`）， down 
    #[test]
    fn loading_placeholder_is_skipped() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"ph","status":"finish","meta_data":{"engine":{"type":"ppt","action":"loading"}},"content":[{"type":"text","text":"让我们先核对下本轮任务的目标与偏好，正在梳理您的需求～"}]}]}"#,
        ));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "", "占位文案不是回答，下发就收不回来了");
        // （ 1 logic_id） → ， down 
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"body","status":"init","content":[{"type":"text","text":"真正的回答"}]}]}"#,
        ));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "真正的回答");
    }

    /// （logic_id ）must，otherwise new will old 
    #[test]
    fn segment_change_resets_the_baseline() {
        let mut machine = ChatFromChatglmStream::new("m");
        machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"think","status":"init","content":[{"type":"think","think":"先想"}]}]}"#,
        ));
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"body","status":"finish","content":[{"type":"text","text":"正文全文"}]}]}"#,
        ));
        let (text, thinking, _) = collect(&frames);
        assert_eq!(text, "正文全文", "新段的第一个快照帧应整段下发");
        assert_eq!(thinking, "", "新段没有思考内容");
    }

    /// reasoning_content ， and 1 「 / 」 then 
    #[test]
    fn thinking_uses_reasoning_channel() {
        let mut machine = ChatFromChatglmStream::new("m");
        machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"t","status":"init","content":[{"type":"think","think":"想"}]}]}"#,
        ));
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"t","status":"finish","content":[{"type":"think","think":"想一会"}]}]}"#,
        ));
        let (text, thinking, _) = collect(&frames);
        assert_eq!(text, "");
        assert_eq!(thinking, "一会", "快照只补后缀");
    }

    /// and to when 
    #[test]
    fn text_and_thinking_are_separated() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"content":[{"type":"think","think":"推理"},{"type":"text","text":"答案"}]}]}"#,
        ));
        let (text, thinking, _) = collect(&frames);
        assert_eq!(text, "答案");
        assert_eq!(thinking, "推理");
    }

    /// HEARTBEAT （ not ）
    #[test]
    fn heartbeat_is_ignored() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(b"data: HEARTBEAT\n\n");
        assert!(frames.is_empty());
    }

    /// for not up （ up /） when ** not **： 
    #[test]
    fn mismatched_snapshot_is_not_replayed() {
        let mut machine = ChatFromChatglmStream::new("m");
        machine.push(&sse(
            r#"{"status":"init","parts":[{"logic_id":"s","status":"init","content":[{"type":"text","text":"ABC"}]}]}"#,
        ));
        let frames = machine.push(&sse(
            r#"{"status":"finish","parts":[{"logic_id":"s","status":"finish","content":[{"type":"text","text":"XYZ"}]}]}"#,
        ));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "", "对不上的快照不得整段重发");
    }

    /// part by after 
    #[test]
    fn multiple_parts_are_concatenated() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(&sse(
            r#"{"status":"init","parts":[{"content":[{"type":"text","text":"前半"}]},{"content":[{"type":"text","text":"后半"}]}]}"#,
        ));
        let (text, _, _) = collect(&frames);
        assert_eq!(text, "前半后半");
    }

    /// error（ not ）→ 429 error
    #[test]
    fn quota_error_becomes_rate_limit() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(&sse(
            r#"{"status":-1001,"message":"剩余积分不足","error_code":-1001,"error":"剩余积分不足，可购买积分充值包获得更多积分"}"#,
        ));
        let raw = String::from_utf8_lossy(&frames[0]).to_string();
        assert!(raw.contains("rate_limit_error"), "实际: {raw}");
        assert!(raw.contains("积分不足"));
        assert!(raw.contains("429"));
    }

    /// finish； no has EOF need （ not success）
    #[test]
    fn truncated_stream_reports_error() {
        let mut machine = ChatFromChatglmStream::new("m");
        machine.push(&sse(
            r#"{"status":"init","parts":[{"content":[{"type":"text","text":"半截"}]}]}"#,
        ));
        let frames = machine.finish();
        let raw = String::from_utf8_lossy(&frames[0]).to_string();
        assert!(raw.contains("截断") || raw.contains("upstream_error"), "实际: {raw}");
    }

    /// 1 no has finish → emptyresponse（ not emptysuccess）
    #[test]
    fn empty_stream_reports_error() {
        let mut machine = ChatFromChatglmStream::new("m");
        let frames = machine.push(&sse(
            r#"{"status":"finish","parts":[{"content":[]}]}"#,
        ));
        let raw = String::from_utf8_lossy(&frames[0]).to_string();
        assert!(raw.contains("空响应"), "实际: {raw}");
    }

    /// must [DONE]（ up not ）
    #[test]
    fn done_marker_is_appended() {
        let mut machine = ChatFromChatglmStream::new("m");
        machine.push(&sse(
            r#"{"status":"init","parts":[{"content":[{"type":"text","text":"x"}]}]}"#,
        ));
        let frames = machine.push(&sse(r#"{"status":"finish","parts":[]}"#));
        let joined = frames
            .iter()
            .map(|frame| String::from_utf8_lossy(frame).to_string())
            .collect::<String>();
        assert!(joined.contains("data: [DONE]"));
    }

    /// emptyresponse not will 「 has role 」
    #[test]
    fn role_only_start_is_not_content() {
        let mut machine = ChatFromChatglmStream::new("m");
        // to empty parts， finish
        let frames = machine.push(&sse(r#"{"status":"finish","parts":[]}"#));
        let raw = String::from_utf8_lossy(&frames[0]).to_string();
        assert!(raw.contains("error"));
    }

    /// ：、 after 、 for not up not 
    #[test]
    fn advance_segment_rules() {
        let mut emitted = String::new();
        // ： is new ， down and 
        assert_eq!(advance_segment(&mut emitted, "abc", false), "abc");
        assert_eq!(advance_segment(&mut emitted, "def", false), "def");
        // ： no down after 
        assert_eq!(advance_segment(&mut emitted, "abcdefgh", true), "gh");
        // and already down 1 → none（ need 1 ： not can down ）
        assert_eq!(advance_segment(&mut emitted, "abcdefgh", true), "");
        // for not up （ up / ）→ not ， not 
        assert_eq!(advance_segment(&mut emitted, "xyz", true), "");
        // empty → none
        assert_eq!(advance_segment(&mut emitted, "", true), "");
        assert_eq!(advance_segment(&mut emitted, "", false), "");
        // new 1 （ as empty）→ down 
        let mut fresh = String::new();
        assert_eq!(advance_segment(&mut fresh, "整段", true), "整段");
    }
}
