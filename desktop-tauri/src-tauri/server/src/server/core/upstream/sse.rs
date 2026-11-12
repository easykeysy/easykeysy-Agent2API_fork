//! SSE reasoning and （ for Node workbuddy-upstream-client.mjs 
//! createReasoningCoalescingStream，114-176 ）。
//!
//! ── Why need need ────────────────────────────────────────────
//! up （reasoning_content） 1-2 1 ，client by 
//! SSE ， will 1 。 when reasoning 
//! to ≥ REASONING_COALESCE_CHARS（ or to reasoning /） down ，
//! **， not **。
//!
//! ── （ for Node Transform）────────────────────────
//! - by `\n` ，** chunk in tail **（ not will by for ，
//! no has tail will 1 JSON ，failure）
//! - empty； `data:` ** + `\n\n`** （Node is this ，
//! will ， this is has as ， 1 ）
//! - `data: [DONE]` → reasoning ， `data: [DONE]\n\n`
//! - `data:` not is JSON → （ not up ）
//! - JSON：（id/model/created，** 1 ** valid）
//! - reasoning （ has reasoning_content and no has content/tool_calls/function_call）
//! → ， 1 
//! - its → reasoning 1 ， this 
//! - when （flush）remaining reasoning
//!
//! is **** not is `Stream` ： need need 
//! 「request → response → 」 （ forward.rs），
//! 「 1 chunk 、 to chunk 」 this can 。
//!
//! ── usage （requeststatistics）────────────────────────────────
//! is SSE ** 1 **， usage in this 
//! （`handle_line` that 1 ）： 1 JSON `usage` 、
//! `usage::RequestTelemetry`， not and —— and not when 
//! 1 （ this ）。
//!
//! ── model （Agent2API W3-T4）────────────────────────────
//! up will response chunk `model` ，client is 
//! request when to （ `raccoon-sse-pipe.mjs` `rewriteSseLine`）。
//! 「 need not need 」 by ****（`ProviderAdapter::sse_model_rewrite`），
//! is 1 down ， in this 。
//! **default**：`rewrite` as None when ， and before 1 
//! （workbuddy not is need ）。
//!
//! ── 「 has 」 content ──────────────────────
//! WorkBuddy（copilot.tencent.com） and AutoClaw up will in 
//! 1 `delta.content` —— that is （ 1 
//! `: heartbeat` in ）， not is 。
//! （ `request_raw` is ** down to client **， 58 
//! WorkBuddy 200 request）：20 this （34.5%），request
//! ** in 13.4%**（ 1.2%–30.8%），request 0 —— is **** ，
//! 1 request need 1 no has ， need 1 。（ 1 
//! ）：`'\n'` 249 / `'\n\n\n'` 47 / `' \n'` 7 / `'\n \n'` 5 / `'\n\n'` 1，
//! its in empty by 「 can not 」 not 。
//! ⚠️ and when **none**：request in 12.1 seconds、 14.3 seconds，
//! >20% that 4 in has 9.8 seconds（≤20% is 13.3 seconds）。 this 
//! 「 when 」 to ， when ； can can 
//! up and server flush 。
//! client by markdown when 1 is 1 `<br>`， is 1 
//! 「Let me run the unit tests」 1 1 —— is 「」。
//!
//! and in ： this is SSE 1 ，path
//! to 1 。**default**（`strip_newline_chunks = false`），
//! when and before 1 ； and not by ****
//! （`ProviderAdapter::sse_strip_newline_chunks`）。「 can not 」，
//! [`is_newline_keepalive`]。

use std::sync::Arc;

use bytes::Bytes;
use serde_json::Value;

use crate::server::logging;

use super::usage::RequestTelemetry;

/// （reasoning_content） and （ Node REASONING_COALESCE_CHARS）
pub const REASONING_COALESCE_CHARS: usize = 60;

/// 1 ：`Bytes` already is `data: ...\n\n` 
pub type Frame = Bytes;

/// model parameter（module header； has `sse_model_rewrite()` has ）
#[derive(Clone, Debug)]
pub struct ModelRewrite {
    /// clientrequest （）
    pub requested: String,
}

/// down ： by ****， use （ and `aggregate`）
/// ， not in that provider 。
///
/// 1 not is 1 parameter： in up ****
/// （ 1 `adapter` + 1 request ）， parameter will 
/// `ForwardStream` / 1 `bool`， use up not 
/// that `true` is 1 。
#[derive(Clone, Debug, Default)]
pub struct FramePolicy {
    /// model （None = up `model`）
    pub rewrite: Option<ModelRewrite>,
    /// 「 has 」 content （module header and [`is_newline_keepalive`]）
    pub strip_newline_chunks: bool,
    pub native_tools: Vec<String>,
}

impl FramePolicy {
    /// model 、 its by default（ path use ： that 
    /// no has ， not ）
    pub fn with_rewrite(rewrite: Option<ModelRewrite>) -> Self {
        Self { rewrite, strip_newline_chunks: false, native_tools: Vec::new() }
    }

    /// by ** can × **（ up 1 ，
    /// `provider_loop::frame_policy_of`）。
    ///
    /// **** is has ： can in before 、 in after ⇒ this can ****
    /// （， 1 can ）， not can ****
    /// （， not will 1 no up empty）。 ，
    /// 「 1 parameter」 can to any 1 down —— that is ：
    /// WorkBuddy / AutoClaw （module header），
    /// parameter is can 。
    ///
    /// default（`config::KEY_STRIP_NEWLINE_KEEPALIVE`）： will ·
    /// that —— 1 ， not by use ，
    /// in / up ， not is 。
    pub fn assembled(rewrite: Option<ModelRewrite>, declared: bool, switch_on: bool) -> Self {
        Self { rewrite, strip_newline_chunks: declared && switch_on, native_tools: Vec::new() }
    }
}

/// this 1 is not is 「 has 」 （ is ** can not ** to ）。
///
/// in need when down 1 ：
/// - `choices[0].delta.content` is empty、 and ** by ** ；
/// - 1 delta no has valid：`reasoning_content` as empty/、
/// `tool_calls` empty、`function_call` —— 1 need has ，
/// not is ， not can ；
/// - no has /statistics/error：`finish_reason` 、`usage` or null、
/// no has `error` —— this client can from this 1 to ， 1 in 
/// [`carries_terminal_signal`]（ 1 ， not 1 ）。
///
/// `delta.role` in ** not **：AutoClaw 1 `role:"assistant"`，
/// 「 has 」 etc 1 not 。
pub fn is_newline_keepalive(chunk: &Value) -> bool {
    // / statistics / error ** has 1 **（ `carries_terminal_signal`），
    // in this 1 ： 1 1 ， is 2026-10 that 「 」
    // （ when is this ）。
    if carries_terminal_signal(chunk) {
        return false;
    }
    let Some(choice) = chunk
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
    else {
        return false;
    };
    let Some(delta) = choice.get("delta").and_then(Value::as_object) else {
        return false;
    };
    let newline_only = delta
        .get("content")
        .and_then(Value::as_str)
        .map(|text| !text.is_empty() && text.chars().all(|ch| ch == '\n' || ch == '\r'))
        .unwrap_or(false);
    if !newline_only {
        return false;
    }
    // content has not 
    let carries_reasoning = delta
        .get("reasoning_content")
        .and_then(Value::as_str)
        .map(|text| !text.is_empty())
        .unwrap_or(false);
    let carries_tool_calls = delta
        .get("tool_calls")
        .and_then(Value::as_array)
        .map(|items| !items.is_empty())
        .unwrap_or(false);
    let carries_function_call = delta.get("function_call").map(is_truthy).unwrap_or(false);
    !(carries_reasoning || carries_tool_calls || carries_function_call)
}

/// this 1 is **client can from to **： `error`、 `usage` has 、
/// or `choices[0].finish_reason` as 。
///
/// use is 「 by 」 path（、reasoning ）must：
/// 1 can has 、 is 1 /statistics/error， by will 
/// 1 ， this client** can ** from this 1 to （`finish_reason` ，
/// as 「 this 1 」 client will not ；`error` ， ）。
///
/// ** this is that 1 ** —— this ， not need in use 1 ：
/// 2026-10 that （OfficeAce 1 empty `reasoning_content`， reasoning
/// ，client etc not to stop reason） is 1 、
/// has 1 。
pub fn carries_terminal_signal(chunk: &Value) -> bool {
    if chunk.get("error").is_some() {
        return true;
    }
    if chunk.get("usage").is_some_and(|usage| !usage.is_null()) {
        return true;
    }
    chunk
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first().cloned())
        .filter(|choice| choice.get("finish_reason").is_some_and(is_truthy))
        .is_some()
}

/// （ for Node `meta = { id, model, created }`）
#[derive(Default, Clone, Debug)]
struct FrameMeta {
    id: Option<Value>,
    model: Option<Value>,
    created: Option<Value>,
}

/// reasoning and 
pub struct ReasoningCoalescer {
    acc: String,
    meta: FrameMeta,
    /// chunk （ not will by for ）
    ///
    /// is **** not is String： in ， TCP 
    /// can can 1 3 。Node is `tail += chunk.toString('utf8')`，
    /// in in when that 1 will U+FFFD —— up 
    /// 。 this by 、 in up ， as up ；
    /// for 「」 and Node 1 。
    tail: Vec<u8>,
    /// usage （optional）： `report_usage` and `usage.rs` 。
    /// in this is as ** is SSE 1 ** —— up 
    /// `data:` will `handle_line`， 1 `usage` not need need 1 
    /// ， not will to path in 。
    telemetry: Option<Arc<RequestTelemetry>>,
    /// model parameter（optional；module header）。None = up model 
    rewrite: Option<ModelRewrite>,
    /// （module header and [`is_newline_keepalive`]）。
    /// false = and before 1 。
    strip_newline_chunks: bool,
    native_tools: super::native_tools::Repair,
}

impl Default for ReasoningCoalescer {
    fn default() -> Self {
        Self::new()
    }
}

impl ReasoningCoalescer {
    pub fn new() -> Self {
        Self {
            acc: String::new(),
            meta: FrameMeta::default(),
            tail: Vec::new(),
            telemetry: None,
            rewrite: None,
            strip_newline_chunks: false,
            native_tools: Default::default(),
        }
    }

    /// usage and （ use ；`new()` to nonestatistics need use ）
    pub fn with_telemetry(telemetry: Arc<RequestTelemetry>) -> Self {
        Self { telemetry: Some(telemetry), ..Self::new() }
    }

    /// up （model + ；module header）。
    /// use when as and before 1 。
    pub fn with_policy(mut self, policy: FramePolicy) -> Self {
        self.native_tools = super::native_tools::Repair::new(&policy.native_tools);
        self.rewrite = policy.rewrite;
        self.strip_newline_chunks = policy.strip_newline_chunks;
        self
    }

    /// 1 up ， need down to client （0..n ）
    pub fn push(&mut self, chunk: &[u8]) -> Vec<Frame> {
        let mut out: Vec<Frame> = Vec::new();
        self.tail.extend_from_slice(chunk);
        // ： to after 1 '\n' as ， down （） in tail 
        let mut start = 0usize;
        while let Some(offset) = self.tail[start..].iter().position(|byte| *byte == b'\n') {
            let end = start + offset;
            let line = String::from_utf8_lossy(&self.tail[start..end]).to_string();
            self.handle_line(&line, &mut out);
            start = end + 1;
        }
        self.tail.drain(..start);
        out
    }

    /// ： reasoning。
    ///
    /// ── Why not tail（ and Node 1 ， and ）────────────────
    /// `tail` 「 not `\n` 」。：
    /// a. up after 1 no has → that its is 1 ；
    /// b. up in in → that is JSON。
    /// Node flush acc、** not tail**， is (b) 、(a) 。
    /// this 1 as ： will 1 `data:` ，
    /// client； can 1 （ and up `\n\n` ，
    /// this path not will to ）， not need 1 clientfailure。
    pub fn finish(&mut self) -> Vec<Frame> {
        let mut out: Vec<Frame> = Vec::new();
        if !self.acc.is_empty() {
            out.push(self.coalesced_frame());
            self.acc.clear();
        }
        if let Some(frame) = self.native_tools.flush(&serde_json::json!({
            "id": self.meta.id, "model": self.meta.model, "created": self.meta.created,
            "object": "chat.completion.chunk",
        })) { out.push(sse_frame(&frame)); }
        out
    }

    /// 1 （ already `\r`）
    fn handle_line(&mut self, raw: &str, out: &mut Vec<Frame>) {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if line.trim().is_empty() {
            return;
        }
        if !line.starts_with("data:") {
            out.push(plain_frame(line));
            return;
        }
        let data = line[5..].trim();
        if data == "[DONE]" {
            if !self.acc.is_empty() {
                out.push(self.coalesced_frame());
                self.acc.clear();
            }
            if let Some(frame) = self.native_tools.flush(&serde_json::json!({
                "id": self.meta.id, "model": self.meta.model, "created": self.meta.created,
                "object": "chat.completion.chunk",
            })) { out.push(sse_frame(&frame)); }
            out.push(Bytes::from_static(b"data: [DONE]\n\n"));
            return;
        }
        let Ok(chunk) = serde_json::from_str::<Value>(data) else {
            if let Some(frame) = self.native_tools.flush(&serde_json::json!({
                "id": self.meta.id, "model": self.meta.model, "created": self.meta.created,
                "object": "chat.completion.chunk",
            })) { out.push(sse_frame(&frame)); }
            // failure ： up not can 
            out.push(plain_frame(line));
            return;
        };
        if chunk.is_object() {
            // ── usage （requeststatistics）──────────────────────────
            // in 「 already this is JSON for 」 after 、「 no 」 before 。
            // Why** not will **： this `chunk` 1 and 
            // 1 ， not `chunk` not and down `out` ——
            // none in and ， and not this when 1 。
            // failure（usage / for ）：statistics 1 can ，
            // this no has error， no has 「 」 can can 。
            if let Some(telemetry) = &self.telemetry {
                if let Some(usage) = chunk.get("usage") {
                    telemetry.report_usage(usage);
                }
            }
            // up 1 valid（Node: `if (chunkObj.id) meta.id = ...`）
            if let Some(id) = chunk.get("id").filter(|value| is_truthy(value)) {
                self.meta.id = Some(id.clone());
            }
            if let Some(model) = chunk.get("model").filter(|value| is_truthy(value)) {
                self.meta.model = Some(model.clone());
            }
            if let Some(created) = chunk.get("created").filter(|value| is_truthy(value)) {
                self.meta.created = Some(created.clone());
            }
        }
        // ── （module header and `is_newline_keepalive`）──────
        // in usage and ** after **： this 1 usage / id / model /
        // created statistics and and ， is 「 this 1 down 」 this 。
        // this ** not acc** —— 1 not this 1 before ，
        // otherwise「」 will reasoning （ that is 1 ）。
        if self.strip_newline_chunks && is_newline_keepalive(&chunk) {
            return;
        }
        let delta = chunk
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("delta"));
        let reasoning = delta
            .and_then(|delta| delta.get("reasoning_content"))
            .and_then(Value::as_str)
            .map(str::to_string);
        // Node: `typeof content === 'string' && content.length > 0 || tool_calls?.length || function_call`
        let other_delta = delta
            .map(|delta| {
                let content = delta
                    .get("content")
                    .and_then(Value::as_str)
                    .map(|text| !text.is_empty())
                    .unwrap_or(false);
                let tool_calls = delta
                    .get("tool_calls")
                    .and_then(Value::as_array)
                    .map(|items| !items.is_empty())
                    .unwrap_or(false);
                // `delta.function_call` in （Node is ）
                let function_call = delta.get("function_call").map(is_truthy).unwrap_or(false);
                content || tool_calls || function_call
            })
            .unwrap_or(false);

        // ── reasoning ：「」────────────────────────
        // use `reasoning.is_some()`（ in ）， OfficeAce that 
        // `delta:{content:"", reasoning_content:""} + finish_reason + usage`
        // **** will this —— empty not 、`acc` as empty，
        // is this 1 、 `[DONE]` not 。
        // client etc not to stop reason（Qoder incomplete_stream
        // and 「 already tool_call parameter」）： `carries_terminal_signal`。
        if reasoning.is_some() && !other_delta && !carries_terminal_signal(&chunk) {
            if let Some(text) = reasoning {
                self.acc.push_str(&text);
            }
            if self.acc.chars().count() >= REASONING_COALESCE_CHARS {
                out.push(self.coalesced_frame());
                self.acc.clear();
            }
            return;
        }
        // reasoning ：，（ can model，module header）
        if !self.acc.is_empty() {
            out.push(self.coalesced_frame());
            self.acc.clear();
        }
        if !self.native_tools.enabled() {
            out.push(self.rewritten_frame(line, &chunk));
            return;
        }
        for chunk in self.native_tools.transform(&chunk) {
            let serialized = serde_json::to_string(&chunk).ok().map(|text| format!("data: {text}"));
            out.push(self.rewritten_frame(serialized.as_deref().unwrap_or(line), &chunk));
        }
    }

    /// 1 ， need when `model` clientrequest 。
    ///
    /// ── Why model、 not ─────────────────────────────
    /// is **client can ** （client by to response），
    /// ； its 1 —— up /
    /// （empty tool_calls ） is bug， not this by delete，
    /// that will 「 to 」 and 「 up 」none from 。
    /// not configured（workbuddy） when return** **， not 。
    ///
    /// use `serde_json::to_string`（、noneempty）， and 
    /// `JSON.stringify(parsed)` ；failure when （ can 1 ，
    /// not need down 1 ）。
    fn rewritten_frame(&self, line: &str, chunk: &Value) -> Frame {
        let Some(rewrite) = &self.rewrite else {
            return plain_frame(line);
        };
        let Some(object) = chunk.as_object() else {
            return plain_frame(line);
        };
        // up no model when ** not **： in `'model' in parsed` when 
        // （ 1 client will no has ， to 「 up to 」）
        if !object.contains_key("model") {
            return plain_frame(line);
        }
        let current = object.get("model").and_then(Value::as_str).unwrap_or("");
        if current == rewrite.requested {
            return plain_frame(line);
        }
        let mut next = chunk.clone();
        if let Some(map) = next.as_object_mut() {
            map.insert(
                "model".to_string(),
                Value::String(rewrite.requested.clone()),
            );
        }
        match serde_json::to_string(&next) {
            Ok(text) => Bytes::from(format!("data: {text}\n\n")),
            Err(_) => plain_frame(line),
        }
    }

    /// and ：`{id, object:'chat.completion.chunk', created, model, choices:[...]}`
    ///
    /// id/created Node（`wb-coalesce-<ms>` / currentseconds）。
    ///
    /// ── Why JSON not is `json!` + ──────────────
    /// serde_json `preserve_order`，`Value::Object` is by 
    /// BTreeMap， is `choices, created, id, model, object`；
    /// Node `JSON.stringify` for ****。
    /// is JSON、 etc ， this is ** **（ not 
    /// —— that is up ， not ）， can to and Node
    /// 1 。 to ，「 → 」，
    /// client/ for when not will 。
    /// use `serde_json::to_string` ， is JSON。
    fn coalesced_frame(&self) -> Frame {
        let id = match &self.meta.id {
            Some(value) if is_truthy(value) => value.clone(),
            _ => Value::String(format!("wb-coalesce-{}", logging::now_ms())),
        };
        let created = match &self.meta.created {
            Some(value) if is_truthy(value) => value.clone(),
            _ => Value::from(logging::now_ms() / 1000),
        };
        let model = match &self.meta.model {
            // config when ， and use clientrequest （ takeFrame is 
            // use requestedModel this 1 ）；not configured when use up 1 
            _ if self.rewrite.is_some() => Value::String(
                self.rewrite
                    .as_ref()
                    .map(|rewrite| rewrite.requested.clone())
                    .unwrap_or_default(),
            ),
            Some(value) if is_truthy(value) => value.clone(),
            _ => Value::String(String::new()),
        };
        // JSON （ will and ，//null ）
        let encode = |value: &Value| serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
        let text = format!(
            r#"{{"id":{},"object":"chat.completion.chunk","created":{},"model":{},"choices":[{{"index":0,"delta":{{"reasoning_content":{}}},"finish_reason":null}}]}}"#,
            encode(&id),
            encode(&created),
            encode(&model),
            encode(&Value::String(self.acc.clone())),
        );
        Bytes::from(format!("data: {text}\n\n"))
    }
}

/// 1 SSE：`data: <JSON>\n\n`（Node sseFrame）
pub fn sse_frame(value: &Value) -> Frame {
    let text = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
    Bytes::from(format!("data: {text}\n\n"))
}

/// `data:` ： + `\n\n`（Node: `line + '\n\n'`）
fn plain_frame(line: &str) -> Frame {
    Bytes::from(format!("{line}\n\n"))
}

/// （Node this 1 `if (chunkObj.id)` is ）
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn raw(value: Value) -> String {
        format!("data: {}\n\n", serde_json::to_string(&value).expect("测试里的帧必须是合法 JSON"))
    }

    /// up （WorkBuddy） 1 ： usage:null and finish_reason ，
    /// and for —— this ， not can to 1 。
    fn frame(delta: Value) -> String {
        raw(json!({
            "id": "cmb-1",
            "object": "chat.completion.chunk",
            "created": 1_700_000_000,
            "model": "deepseek-v4.1-flash",
            "choices": [{"index": 0, "delta": delta, "finish_reason": null}],
            "usage": Value::Null,
        }))
    }

    fn coalescer(strip: bool) -> ReasoningCoalescer {
        ReasoningCoalescer::new().with_policy(FramePolicy {
            rewrite: None,
            strip_newline_chunks: strip,
            ..Default::default()
        })
    }

    fn feed(coalescer: &mut ReasoningCoalescer, text: &str) -> String {
        coalescer
            .push(text.as_bytes())
            .iter()
            .map(|frame| String::from_utf8_lossy(frame).to_string())
            .collect()
    }

    #[test]
    fn a_newline_only_chunk_passes_through_when_the_policy_is_off() {
        let keepalive = frame(json!({"content": "\n"}));
        let mut off = coalescer(false);
        let out = feed(&mut off, &keepalive);
        // when is ：
        assert_eq!(out, keepalive, "关着时帧的字节必须一模一样");
        assert!(out.contains("\"content\":\"\\n\""), "关着时不能动这一帧: {out}");
    }

    #[test]
    fn a_newline_only_chunk_is_dropped_when_the_policy_is_on() {
        let mut on = coalescer(true);
        assert!(feed(&mut on, &frame(json!({"content": "\n"}))).is_empty());
        assert!(feed(&mut on, &frame(json!({"content": "\n\n\n"}))).is_empty());
        assert!(feed(&mut on, &frame(json!({"content": "\r\n"}))).is_empty());
    }

    #[test]
    fn autoclaw_shape_is_dropped_too_role_included() {
        // 1 `role:"assistant"`， and no has `finish_reason` ——
        // role if 「 has valid」， this 1 1 not （ ）。
        let dropped = raw(json!({
            "id": "20260927001447d076",
            "created": 1_790_439_287,
            "object": "chat.completion.chunk",
            "model": "glm-5.3-flash",
            "choices": [{"index": 0, "delta": {"role": "assistant", "content": "\n"}}],
        }));
        let mut on = coalescer(true);
        assert!(feed(&mut on, &dropped).is_empty());
        let mut off = coalescer(false);
        assert!(!feed(&mut off, &dropped).is_empty());
    }

    #[test]
    fn a_dropped_keepalive_does_not_flush_the_reasoning_accumulator() {
        let reasoning = frame(json!({"reasoning_content": "abc"}));
        let keepalive = frame(json!({"content": "\n"}));
        let mut on = coalescer(true);
        let mut out = feed(&mut on, &reasoning);
        out.push_str(&feed(&mut on, &keepalive));
        assert!(out.is_empty(), "纯噪声帧既不该下发，也不该把攒了一半的思考结掉: {out}");
        // when is 「 reasoning 」， old will —— this for 
        // this 1 path， not is 1 
        let mut off = coalescer(false);
        let mut out_off = feed(&mut off, &reasoning);
        out_off.push_str(&feed(&mut off, &keepalive));
        assert!(out_off.contains("reasoning_content"), "关着时应先冲刷思考帧: {out_off}");
    }

    #[test]
    fn terminal_usage_and_error_frames_are_never_dropped() {
        let cases = vec![
            // ：client etc not to 
            json!({"choices": [{"index": 0, "delta": {"content": "\n"}, "finish_reason": "stop"}]}),
            // usage ：statistics 1 
            json!({"choices": [{"index": 0, "delta": {"content": "\n"}}], "usage": {"total_tokens": 3}}),
            // error
            json!({"error": {"message": "\n"}, "choices": [{"index": 0, "delta": {"content": "\n"}}]}),
        ];
        for case in cases {
            let mut on = coalescer(true);
            let text = raw(case.clone());
            let out = feed(&mut on, &text);
            assert!(!out.is_empty(), "这类帧绝不能被当成保活噪声丢掉: {case}");
            assert_eq!(out, text, "保留时原样透传: {case}");
        }
    }

    #[test]
    fn text_bearing_frames_are_untouched_by_the_strip() {
        let mut on = coalescer(true);
        for delta in [
            json!({"content": " real"}),
            json!({"content": ".\n\n"}),
            json!({"content": "\n \n"}),
            json!({"content": "端"}),
        ] {
            let text = frame(delta.clone());
            assert_eq!(feed(&mut on, &text), text, "带正文的帧要逐字透传: {delta}");
        }
    }

    /// （NAS `request_raw` down ，WorkBuddy 
    /// `deepseek-v4.1-flash`，request id `185d8dbf…`） ****：
    /// up in 。 after 1 `":\n"` is 「 + 」，
    /// 1 will —— this 。
    const CAPTURED: [&str; 7] = [
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":"A","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":"\n","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":" real","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":"\n","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":" finding","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":"\n","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
        r#"data: {"id":"cmb-22097200b9bf11f1a8cfb69a5c373198","model":"deepseek-v4.1-flash","object":"chat.completion.chunk","created":1790436599,"choices":[{"index":0,"delta":{"content":":\n","reasoning_content":"","function_call":null,"refusal":"","tool_calls":[],"extra_fields":null},"logprobs":null,"finish_reason":""}],"usage":null}"#,
    ];

    #[test]
    fn a_replayed_production_stream_keeps_only_the_text_bearing_frames() {
        let stream: String = CAPTURED.iter().map(|line| format!("{line}\n\n")).collect();
        let text_bearing: Vec<String> = vec![0, 2, 4, 6]
            .into_iter()
            .map(|index| format!("{}\n\n", CAPTURED[index]))
            .collect();

        // ： ， and not （ is ， not ）
        let mut on = coalescer(true);
        let dropped = feed(&mut on, &stream);
        assert_eq!(dropped, text_bearing.concat(), "开启时应当只留下带正文的帧");

        // ： —— and can before 1 
        let mut off = coalescer(false);
        assert_eq!(feed(&mut off, &stream), stream, "关掉时要逐字节透传整段流");

        // client （ for ）
        let content_of = |text: &str| -> String {
            text.split("\n")
                .filter_map(|line| line.strip_prefix("data: "))
                .filter_map(|data| serde_json::from_str::<Value>(data).ok())
                .filter_map(|chunk| {
                    chunk
                        .get("choices")?
                        .get(0)?
                        .get("delta")?
                        .get("content")?
                        .as_str()
                        .map(str::to_string)
                })
                .collect()
        };
        assert_eq!(content_of(&dropped), "A real finding:\n");
        assert_eq!(content_of(&stream), "A\n real\n finding\n:\n");
    }

    #[test]
    fn the_predicate_requires_every_signal_to_be_empty() {
        assert!(is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n"}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n "}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": ""}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": 7}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": []})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n"}, "finish_reason": "length"}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n", "reasoning_content": "x"}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n", "tool_calls": [{"index": 0}]}}]})));
        assert!(!is_newline_keepalive(&json!({"choices": [{"delta": {"content": "\n", "function_call": {"name": "x"}}}]})));
        assert!(!is_newline_keepalive(&Value::Null));
        assert!(!is_newline_keepalive(&json!("")));
    }

    /// then ：** can × **，。
    ///
    /// for 1 will to ——
    /// - `declared=false, switch=true` ⇒ not ： can not can ，
    /// otherwise" 1 parameter" will to 1 no up down ；
    /// - `declared=true, switch=false` ⇒ not ：default（ is when ），
    /// down that to for ；
    /// - ⇒ 。
    #[test]
    fn the_strip_needs_both_the_capability_bit_and_the_switch() {
        let rewrite = Some(ModelRewrite { requested: "m".to_string() });
        assert!(
            !FramePolicy::assembled(rewrite.clone(), false, true).strip_newline_chunks,
            "开关不能替一家没声明能力位的提供商开始丢帧"
        );
        assert!(
            !FramePolicy::assembled(rewrite.clone(), true, false).strip_newline_chunks,
            "开关关着时，声明了能力位的那两家也不动字节"
        );
        assert!(FramePolicy::assembled(rewrite.clone(), true, true).strip_newline_chunks);
        // that 1 ，model not （ is ）
        let policy = FramePolicy::assembled(rewrite, true, false);
        assert_eq!(policy.rewrite.map(|value| value.requested).as_deref(), Some("m"));
    }

    /// to 1 ：** can 、** when ， that need 。
    ///
    /// this is "default" ， not is up that ： is 
    /// （`ReasoningCoalescer::push`）， 1 will to （e.g. to 
    /// reasoning before ）， will ， this will 。
    #[test]
    fn a_capable_provider_with_the_switch_off_still_streams_byte_identical_frames() {
        let stream: String = CAPTURED.iter().map(|line| format!("{line}\n\n")).collect();
        let declared_but_switched_off =
            FramePolicy::assembled(None, true, false).strip_newline_chunks;
        let mut coalescer = ReasoningCoalescer::new().with_policy(FramePolicy {
            rewrite: None,
            strip_newline_chunks: declared_but_switched_off,
            ..Default::default()
        });
        let out: String = coalescer
            .push(stream.as_bytes())
            .iter()
            .map(|frame| String::from_utf8_lossy(frame).to_string())
            .collect();
        assert_eq!(out, stream, "开关关着时七帧要一字不动地出去");
    }

    /// OfficeAce use ****（NAS dev `debug_traffic`，
    /// request `74eeb698…`， up 17 ）：`delta` when has `content:""` and 
    /// `reasoning_content:""`， and statistics in ** this 1 ** up 。
    fn officeace_terminal_frame() -> String {
        raw(json!({
            "id": "as-92kpqm9s7j",
            "object": "chat.completion.chunk",
            "created": 1_791_906_909_u64,
            "model": "deepseek-v4.1-flash",
            "choices": [{
                "index": 0,
                "delta": {"content": "", "role": "assistant", "reasoning_content": ""},
                "finish_reason": "tool_calls",
                "flag": 0,
            }],
            "usage": {
                "prompt_tokens": 293, "completion_tokens": 48, "total_tokens": 341,
                "completion_tokens_details": {"reasoning_tokens": 11},
            },
            "service_tier": "default",
        }))
    }

    /// client not to stop reason will not （Qoder 
    /// `incomplete_stream` and already tool_call parameter**** ⇒ will ）。
    /// old this 1 reasoning ""：
    /// `reasoning_content` need ** in **，empty not ，
    /// `[DONE]` not —— this by up ， old 。
    #[test]
    fn a_terminal_frame_with_an_empty_reasoning_delta_still_reaches_the_client() {
        let terminal = officeace_terminal_frame();
        let mut coalescer = ReasoningCoalescer::new();
        let mut out = feed(&mut coalescer, &terminal);
        out.push_str(&feed(&mut coalescer, "data: [DONE]\n\n"));
        assert!(
            out.contains("\"finish_reason\":\"tool_calls\""),
            "收尾帧不许被累积分支吃掉: {out}"
        );
        assert!(out.contains("\"usage\":{"), "usage 与终止信息同帧，一起丢了就没处补: {out}");
        assert!(out.contains("data: [DONE]"), "[DONE] 照常透传");

        // for ： this 1 is **** ， no has " 1 "
        assert!(out.contains(terminal.trim_end()), "保留时原样透传: {out}");
    }

    /// for ： old （"/statistics" that 1 ，
    /// no has ""）。
    #[test]
    fn a_pure_reasoning_delta_is_still_absorbed() {
        let mut coalescer = ReasoningCoalescer::new();
        assert!(
            feed(&mut coalescer, &frame(json!({"reasoning_content": "边想"}))).is_empty(),
            "没有终止信息的思考帧仍应攒着"
        );
    }

    /// `carries_terminal_signal` 。
    ///
    /// is that ** 1 **（ and path）， can 
    /// ： up that to use ** when ** `usage` and `finish_reason`，
    /// usage that 1 —— `finish_reason` 1 and `error` 1 before none。
    #[test]
    fn the_terminal_signal_predicate_answers_each_clause_on_its_own() {
        // finish_reason 1 （ no has usage）： in 
        assert!(carries_terminal_signal(&json!({
            "choices": [{"delta": {"content": ""}, "finish_reason": "stop"}],
        })));
        // 1 not ：null / empty（ is is_truthy， not is 「 in not in 」）
        assert!(!carries_terminal_signal(&json!({
            "choices": [{"delta": {"content": ""}, "finish_reason": Value::Null}],
        })));
        assert!(!carries_terminal_signal(&json!({
            "choices": [{"delta": {"content": ""}, "finish_reason": ""}],
        })));
        // usage 1 ： null in ，null and not 
        assert!(carries_terminal_signal(&json!({"usage": {"prompt_tokens": 1}})));
        assert!(!carries_terminal_signal(&json!({"usage": Value::Null})));
        assert!(!carries_terminal_signal(&json!({})));
        // error that 1 ：error 1 not by 
        assert!(carries_terminal_signal(&json!({"error": {"message": "boom"}})));
        // for not this in 
        assert!(!carries_terminal_signal(&json!([1, 2])));
        assert!(!carries_terminal_signal(&json!("data")));
    }

    /// error** 1 empty** when ， not reasoning 。
    ///
    /// this is and 1 after that 1 ：path 1 `error`，
    /// before no has —— 1 `error`、`choices[0].delta` has empty
    /// `reasoning_content` will ""， from 。
    #[test]
    fn an_error_frame_with_an_empty_reasoning_delta_is_not_absorbed() {
        let mut coalescer = ReasoningCoalescer::new();
        let error_frame = raw(json!({
            "error": {"message": "upstream exploded", "type": "server_error"},
            "choices": [{
                "index": 0,
                "delta": {"content": "", "reasoning_content": ""},
                "finish_reason": Value::Null,
            }],
        }));
        let out = feed(&mut coalescer, &error_frame);
        assert!(out.contains("upstream exploded"), "错误帧必须下发: {out}");
    }

    /// 1 in before 、 in after ： not can （client will ），
    /// and after `acc` must already empty、 not 1 empty reasoning。
    #[test]
    fn the_accumulated_reasoning_is_flushed_before_the_terminal_frame() {
        let terminal = officeace_terminal_frame();
        let mut coalescer = ReasoningCoalescer::new();
        let mut out = feed(&mut coalescer, &frame(json!({"reasoning_content": "先想一句"})));
        assert!(out.is_empty(), "还没到阈值时不该下发");
        out.push_str(&feed(&mut coalescer, &terminal));
        let reasoning_at = out
            .find("先想一句")
            .expect("攒着的思考应在终止帧之前被冲刷出去");
        let terminal_at = out
            .find("\"finish_reason\":\"tool_calls\"")
            .expect("终止帧要下发");
        assert!(reasoning_at < terminal_at, "顺序：思考在前、终止帧在后: {out}");
        assert!(
            feed(&mut coalescer, "data: [DONE]\n\n").trim() == "data: [DONE]",
            "acc 已清空，[DONE] 处不该再补一帧空的思考"
        );
    }
}
