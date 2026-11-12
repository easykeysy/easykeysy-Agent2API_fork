//! up SSE （ for Node aggregateSseCompletion，600-670 ）。
//!
//! up **supports**（`stream:false` return code=11101），client need when ，
//! `stream:true` request up ， SSE 
//! OpenAI `chat.completion` return。
//!
//! then for Node：
//! - by `data:`，`[DONE]` and empty
//! - `delta.content` / `delta.reasoning_content` / `delta.reasoning` 
//! （ after 1 is Cline ， `consume_line` ）
//! - `delta.tool_calls` by `index` and （id/type 、function.name and 
//! function.arguments ）
//! - `usage` after 1 （ up in chunk down ）
//! - `finish_reason` after 1 empty
//! - `error` → 502（ up error in ）
//! - id/created when to `wb-agg-<ms>` / currentseconds
//!
//! ── usage （requeststatistics）────────────────────────────────
//! `consume_chunk` `usage` `usage::RequestTelemetry`。
//! ** not response**： down `usage` is `self.usage`（file 
//! has ）， 1 ， not 。

use std::collections::BTreeMap;
use std::sync::Arc;

use futures::StreamExt;
use serde_json::{json, Map, Value};

use crate::server::errors::GatewayError;

use super::cancellation;
use super::sse::{is_newline_keepalive, FramePolicy, ModelRewrite};
use super::usage::RequestTelemetry;

/// limit： SSE （failure will ，
/// not can 1 ）
const MAX_LINE_BYTES: usize = 8 * 1024 * 1024;

/// after response + use chunk 
pub struct AggregatedCompletion {
    pub body: Value,
    pub chunk_count: usize,
}

/// up response（SSE ） 1 chat.completion。
///
/// `on_chunk` parameter already ：Node `controller.signal.aborted` check use is 
/// 「client when 」，Rust etc is caller in select! 
/// future（ forward.rs）—— none need in check。
///
/// `telemetry` is usage （`Arc` use ： need in await use ，
/// is ）。 、 not and `acc` ——
/// ** not return body**（usage by aggregate 
/// `self.usage` ， and this none，path not ）。
///
/// `policy` and （ `sse_model_rewrite()` and 
/// `sse_strip_newline_chunks()`）：**response `model` need **
/// clientrequest —— client to model not this as 「 need not need 」
/// （ `forwardChatCompletions` is 
/// `payload.model = requestedModel`）。： up that 
/// 「 has 」 in will ， if 1 
/// to client（ `sse.rs` module header ）。
pub async fn aggregate_sse_completion(
    response: reqwest::Response,
    telemetry: Arc<RequestTelemetry>,
    policy: FramePolicy,
) -> Result<AggregatedCompletion, GatewayError> {
    // and `ForwardStream::new` ：reqwest error in this 
    // io::Error（`describe_error_detail` reqwest::Error）
    let capture = telemetry.capture();
    let stream = response.bytes_stream().map(move |item| {
        let item = item.map_err(|error| {
            std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
        });
        if let (Ok(bytes), Some(capture)) = (&item, &capture) {
            capture.push(bytes);
        }
        item
    });
    aggregate_frame_stream(Box::pin(stream), telemetry, policy).await
}

/// 1 ** chat SSE** （ not ）。
///
/// （responses / anthropic up ） in before 
/// `providers::custom` `ProtocolTranslateStream` —— and 
/// [`aggregate_sse_completion`] use 1 then ， is from 
/// reqwest::Response already 。telemetry / policy 
/// and that 1 （ ）。
/// by in before ， not chat 。
pub async fn aggregate_frame_stream(
    stream: futures::stream::BoxStream<'static, Result<bytes::Bytes, std::io::Error>>,
    telemetry: Arc<RequestTelemetry>,
    policy: FramePolicy,
) -> Result<AggregatedCompletion, GatewayError> {
    // responsetotal when （set「request when 」）：** 1 when 、 not **
    // —— and empty when is （ that is 「」， this 
    // response total， for OmniProxy readBodyWithStallGuard）。
    // when in ：client when no to response， not will ，
    // errorreturn（502 + ）。
    let budget = std::time::Duration::from_millis(
        crate::server::config::timeout_settings().body_ms(),
    );
    match tokio::time::timeout(budget, aggregate_frame_stream_inner(stream, telemetry, policy))
        .await
    {
        Ok(result) => result,
        Err(_elapsed) => Err(GatewayError::with_status(
            502,
            format!("非流式响应超时({}秒)", budget.as_secs()),
        )),
    }
}

/// [`aggregate_frame_stream`] （total when by up ， that ）。
async fn aggregate_frame_stream_inner(
    stream: futures::stream::BoxStream<'static, Result<bytes::Bytes, std::io::Error>>,
    telemetry: Arc<RequestTelemetry>,
    policy: FramePolicy,
) -> Result<AggregatedCompletion, GatewayError> {
    // ── （ and `ForwardStream::from_translated` 1 ）──
    // is `while let Some(item) = stream.next().await` ： no has 
    // when ， need etc down 1 up （ up when can can etc ）。 
    // etc ， after down 1 await to Err， by down map_err 
    // 408 error（client when no to response， will 
    // 「request already 」）。 in not is `aggregate_sse_completion`：
    // this （ then use ）， need 。
    //
    // use `cancellation::cancellable` ** not is ** `stream::select`：
    // after is 「」， in up when not ， is 
    // down this `while let` not —— already to up EOF， need empty to 
    // total when （default 300 seconds） 502（ `cancellable` ）。
    let mut stream = cancellation::cancellable(stream, telemetry.cancel_token());
    let mut buffer = Vec::<u8>::new();
    let mut acc = CompletionAccumulator {
        native_tools: super::native_tools::Repair::new(&policy.native_tools),
        rewrite: policy.rewrite,
        strip_newline_chunks: policy.strip_newline_chunks,
        ..Default::default()
    };
    // ：path not RecordingStream（client need is JSON，
    // no has down can ）， 1 ** up ** chunk in this —— is 
    // 「 up 」 when 。request use when need etc has ，
    // up 「 up is is 」 this （ and path： as ）。
    let mut first_chunk_seen = false;
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|error| {
            // to is （ up ）： 408 
            // error， not 「 up in 」 before —— not is up 
            let text = error.to_string();
            if text == cancellation::MANUAL_TERMINATED {
                return cancellation::cancelled_error();
            }
            // empty when is not is in ：（seconds），
            // use （ and `ForwardStream` error 1 ）
            if text.starts_with(super::stall::IDLE_TIMEOUT_PREFIX) {
                return GatewayError::with_status(502, text);
            }
            // error already in when io::Error（reqwest in 
            // `aggregate_sse_completion`、 in `ProtocolTranslateStream`）
            GatewayError::with_status(502, format!("上游流式传输中断: {error}"))
        })?;
        if !first_chunk_seen {
            first_chunk_seen = true;
            telemetry.note_first_frame();
        }
        // by （HTTP or ）， chat 。
        buffer.extend_from_slice(&chunk);
        // （ to after 1 '\n' before ）
        while let Some(index) = buffer.iter().position(|byte| *byte == b'\n') {
            let line = String::from_utf8_lossy(&buffer[..index]).trim().to_string();
            buffer.drain(..=index);
            if line.is_empty() {
                continue;
            }
            acc.consume_line(&line, &telemetry)?;
        }
        if buffer.len() > MAX_LINE_BYTES {
            return Err(GatewayError::with_status(502, "上游返回的单行数据过大，已中断"));
        }
    }
    // （ up no ）
    let tail = String::from_utf8_lossy(&buffer).trim().to_string();
    if !tail.is_empty() {
        acc.consume_line(&tail, &telemetry)?;
    }

    if let Some(frame) = acc.native_tools.flush(&json!({ "id": acc.id, "model": acc.model, "created": acc.created })) {
        acc.consume_chunk(&frame, &telemetry)?;
    }
    Ok(acc.into_completion())
}

/// （Node in to this ， `consume_line` return Result）
#[derive(Default)]
struct CompletionAccumulator {
    id: String,
    model: String,
    created: i64,
    role: String,
    content: String,
    reasoning: String,
    finish: String,
    usage: Option<Value>,
    chunk_count: usize,
    tool_calls: BTreeMap<i64, Value>,
    /// model parameter（None = not ， use up to model）
    rewrite: Option<ModelRewrite>,
    /// 「 has 」 content （WorkBuddy / AutoClaw ，
    /// and by `sse::is_newline_keepalive`）。false = and before 1 。
    strip_newline_chunks: bool,
    native_tools: super::native_tools::Repair,
}

impl CompletionAccumulator {
    /// 1 SSE（`data: {...}` / `data: [DONE]` / its ）
    fn consume_line(&mut self, line: &str, telemetry: &RequestTelemetry) -> Result<(), GatewayError> {
        let Some(data) = line.strip_prefix("data:") else {
            return Ok(());
        };
        let data = data.trim();
        if data == "[DONE]" {
            if let Some(frame) = self.native_tools.flush(&json!({ "id": self.id, "model": self.model, "created": self.created })) {
                self.consume_chunk(&frame, telemetry)?;
            }
            return Ok(());
        }
        if data.is_empty() { return Ok(()); }
        // Node: `try { handleChunk(JSON.parse(line)) } catch { /* JSON ignore */ }`
        // —— JSON ignore， handleChunk WorkBuddyUpstreamError need 
        let Ok(chunk) = serde_json::from_str::<Value>(data) else {
            if let Some(frame) = self.native_tools.flush(&json!({ "id": self.id, "model": self.model, "created": self.created })) {
                self.consume_chunk(&frame, telemetry)?;
            }
            return Ok(());
        };
        for chunk in self.native_tools.transform(&chunk) {
            self.consume_chunk(&chunk, telemetry)?;
        }
        Ok(())
    }

    fn consume_chunk(&mut self, chunk: &Value, telemetry: &RequestTelemetry) -> Result<(), GatewayError> {
        let Some(object) = chunk.as_object() else {
            return Ok(());
        };
        // ── usage （requeststatistics）──────────────────────────────
        // in error** before **： up error in when can can up already 
        // usage（ already ）， 1 not down 502。
        // Why this failure is 「none use 」 ： `chunk` 1 ，
        // not `self.usage`（ that is response usage that ），
        // return and not when 1 。
        if let Some(usage) = object.get("usage") {
            telemetry.report_usage(usage);
        }
        if let Some(error) = object.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
                .unwrap_or("上游流式返回错误");
            return Err(GatewayError::with_status(502, message.to_string()));
        }
        self.chunk_count += 1;
        if let Some(id) = object.get("id").and_then(Value::as_str) {
            if !id.is_empty() {
                self.id = id.to_string();
            }
        }
        if let Some(model) = object.get("model").and_then(Value::as_str) {
            self.model = model.to_string();
        }
        // Node: `created = chunk.created || created` —— （0 not already has ）
        if let Some(created) = object.get("created") {
            if js_number_truthy(created) {
                self.created = created.as_i64().or_else(|| created.as_f64().map(|value| value as i64)).unwrap_or(0);
            }
        }
        if let Some(usage) = object.get("usage") {
            if usage.is_object() {
                self.usage = Some(usage.clone());
            }
        }

        let Some(choice) = object
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
        else {
            return Ok(());
        };
        let delta = choice.get("delta").cloned().unwrap_or(json!({}));
        if let Some(role) = delta.get("role").and_then(Value::as_str) {
            if !role.is_empty() {
                self.role = role.to_string();
            }
        }
        if let Some(content) = delta.get("content").and_then(Value::as_str) {
            // ：****， not before return —— 1 
            // id / model / created / role / usage （ is this 
            // ： and ， this 1 ）。path 1 ，
            // 「 1 request need not need 」 not will 。
            let keepalive = self.strip_newline_chunks && is_newline_keepalive(chunk);
            if !keepalive {
                self.content.push_str(content);
            }
        }
        // in up ， up and not 1 ：`reasoning_content`（
        // OpenAI ） and `reasoning`（Cline ， `reasoning_details`）
        // 。 before will 「 is 」 response 1 
        // content empty、reasoning empty —— down empty
        // 「empty response detected」。 cline-proxy `sawContent`
        // `reasoning` ， by and this 1 （ its proxy.go ）。
        // 、 not ： and cline-proxy 。
        for key in ["reasoning_content", "reasoning"] {
            if let Some(reasoning) = delta.get(key).and_then(Value::as_str) {
                self.reasoning.push_str(reasoning);
            }
        }
        if let Some(finish) = choice.get("finish_reason").and_then(Value::as_str) {
            if !finish.is_empty() {
                self.finish = finish.to_string();
            }
        }
        if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
            for call in tool_calls {
                let index = call
                    .get("index")
                    .and_then(Value::as_i64)
                    // Node: `Number.isInteger(tc.index) ? tc.index : 0`
                    .unwrap_or(0);
                let entry = self
                    .tool_calls
                    .entry(index)
                    .or_insert_with(|| {
                        json!({
                            "id": "",
                            "type": "function",
                            "function": { "name": "", "arguments": "" },
                        })
                    });
                merge_tool_call(entry, call);
            }
        }
        Ok(())
    }

    /// body（ for Node handleChunk after return for ）
    fn into_completion(self) -> AggregatedCompletion {
        // ── ： up 、 as empty ──────────────────────────
        // this is **** 1 （ in down will 
        // in up ， up 200， `providers::zcode::reasoning` module header）。
        // not in this —— 1 error will 「 up to 」
        // can ； this ： this when is 
        // 「 up its 」 this 1 。
        if self.content.trim().is_empty() && !self.reasoning.trim().is_empty() {
            crate::server::logging::verbose(
                "[Model]",
                &format!(
                    "⚠️ 上游只回了思考、正文为空（model={} finish={} 思考 {} 字）：\
                     多为输出额度被思考吃光，可调高 max_tokens 或降低思考档位",
                    self.model,
                    if self.finish.is_empty() { "-" } else { &self.finish },
                    self.reasoning.chars().count(),
                ),
            );
        }
        let mut message = Map::new();
        message.insert(
            "role".to_string(),
            Value::String(if self.role.is_empty() { "assistant".to_string() } else { self.role }),
        );
        message.insert("content".to_string(), Value::String(self.content));
        if !self.reasoning.is_empty() {
            message.insert("reasoning_content".to_string(), Value::String(self.reasoning));
        }
        if !self.tool_calls.is_empty() {
            message.insert(
                "tool_calls".to_string(),
                Value::Array(self.tool_calls.into_values().collect()),
            );
        }
        let mut body = Map::new();
        body.insert(
            "id".to_string(),
            Value::String(if self.id.is_empty() {
                format!("wb-agg-{}", crate::server::logging::now_ms())
            } else {
                self.id
            }),
        );
        body.insert("object".to_string(), Value::String("chat.completion".to_string()));
        body.insert(
            "created".to_string(),
            Value::from(if self.created != 0 {
                self.created
            } else {
                crate::server::logging::now_ms() / 1000
            }),
        );
        // model （ `aggregate_sse_completion` ）：config when 
        // use clientrequest ，otherwise use up to （ can can as empty —— and before 1 ）
        let model = match &self.rewrite {
            Some(rewrite) => rewrite.requested.clone(),
            None => self.model,
        };
        body.insert("model".to_string(), Value::String(model));
        body.insert(
            "choices".to_string(),
            json!([{
                "index": 0,
                "message": Value::Object(message),
                "finish_reason": if self.finish.is_empty() { "stop".to_string() } else { self.finish },
            }]),
        );
        // Node return for `usage` is ** in ** ： up no down when is null
        // （JSON.stringify will null， not is this ）——
        // OpenAI SDK for `usage: null` and not 1 ，。
        body.insert(
            "usage".to_string(),
            self.usage.unwrap_or(Value::Null),
        );
        AggregatedCompletion { body: Value::Object(body), chunk_count: self.chunk_count }
    }
}

/// and 1 tool_call to already up （id/type 、name/arguments ）。
///
/// `extra_content` need ：antigravity Gemini in 
/// `extra_content.google.thought_signature`（ `antigravity_stream.rs` 
/// tool_call_frames），client down 1 、 `call_signature` 
/// `functionCall.thoughtSignature` —— ，（）path down 1 
/// will up 400「Function call is missing a thought_signature」
/// （issue #237）。 in 1 ， not 。
fn merge_tool_call(entry: &mut Value, incoming: &Value) {
    let Some(target) = entry.as_object_mut() else {
        return;
    };
    if let Some(id) = incoming.get("id").and_then(Value::as_str) {
        if !id.is_empty() {
            target.insert("id".to_string(), Value::String(id.to_string()));
        }
    }
    if let Some(kind) = incoming.get("type").and_then(Value::as_str) {
        if !kind.is_empty() {
            target.insert("type".to_string(), Value::String(kind.to_string()));
        }
    }
    if let Some(extra) = incoming.get("extra_content") {
        if !extra.is_null() {
            target.insert("extra_content".to_string(), extra.clone());
        }
    }
    let Some(function) = incoming.get("function") else {
        return;
    };
    let Some(target_function) = target
        .get_mut("function")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    if let Some(name) = function.get("name").and_then(Value::as_str) {
        let current = target_function
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        target_function.insert("name".to_string(), Value::String(format!("{current}{name}")));
    }
    if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
        let current = target_function
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        target_function.insert(
            "arguments".to_string(),
            Value::String(format!("{current}{arguments}")),
        );
    }
}

/// （Node `chunk.created || created`）
fn js_number_truthy(value: &Value) -> bool {
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

    /// 1 SSE（ JSON / data: ignore， and path 1 ）
    fn feed(acc: &mut CompletionAccumulator, line: &str) {
        let telemetry = RequestTelemetry::new();
        acc.consume_line(line, &telemetry)
            .unwrap_or_else(|error| panic!("consume_line 失败：{error}"));
    }

    /// up in `delta.reasoning`（Cline ，
    /// `reasoning_details`） when ，must down ： `reasoning_content`
    /// will 「 is 」 response content empty、reasoning empty ，
    /// down empty（ AxonHub）「empty response detected」。
    /// cline-proxy `sawContent` `reasoning` 。
    #[test]
    fn reasoning_delta_is_kept() {
        let mut acc = CompletionAccumulator::default();
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"role":"assistant"},"finish_reason":null}]}"#,
        );
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"reasoning":"我们先分析"},"finish_reason":null}]}"#,
        );
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"reasoning":"题目。"},"finish_reason":"length"}]}"#,
        );
        feed(&mut acc, "data: [DONE]");

        let body = acc.into_completion().body;
        let message = &body["choices"][0]["message"];
        assert_eq!(message["reasoning_content"], "我们先分析题目。");
        // content empty —— must has 1 empty can ，
        // this is 「 has no has 」 
        assert_eq!(message["content"], "");
    }

    /// `reasoning_content` and `reasoning` 、 not （ and cline-proxy
    /// ）： up in not when not will 。
    #[test]
    fn both_reasoning_fields_accumulate() {
        let mut acc = CompletionAccumulator::default();
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"reasoning_content":"甲"},"finish_reason":null}]}"#,
        );
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"reasoning":"乙"},"finish_reason":null}]}"#,
        );
        let body = acc.into_completion().body;
        assert_eq!(body["choices"][0]["message"]["reasoning_content"], "甲乙");
    }

    /// not （reasoning as empty when not this ）
    #[test]
    fn content_only_stays_as_before() {
        let mut acc = CompletionAccumulator::default();
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"content":"你好"},"finish_reason":"stop"}]}"#,
        );
        let body = acc.into_completion().body;
        let message = &body["choices"][0]["message"];
        assert_eq!(message["content"], "你好");
        assert!(message.get("reasoning_content").is_none());
    }

    /// `extra_content` must tool_call （issue #237）：antigravity 
    /// in `extra_content.google.thought_signature`，client down 1 、
    /// `functionCall.thoughtSignature` —— ，
    /// client down 1 will up 400「missing a thought_signature」。
    #[test]
    fn tool_call_extra_content_survives_aggregation() {
        let mut acc = CompletionAccumulator::default();
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","type":"function","function":{"name":"pwsh","arguments":""},"extra_content":{"google":{"thought_signature":"sig_abc"}}}]}}]}"#,
        );
        feed(
            &mut acc,
            r#"data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"command\":\"ls\"}"}}]},"finish_reason":"tool_calls"}]}"#,
        );
        feed(&mut acc, "data: [DONE]");

        let body = acc.into_completion().body;
        let call = &body["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["id"], "call_1");
        assert_eq!(call["function"]["name"], "pwsh");
        assert_eq!(call["function"]["arguments"], r#"{"command":"ls"}"#);
        assert_eq!(
            call["extra_content"]["google"]["thought_signature"], "sig_abc",
            "签名丢了，下一轮 functionCall 就缺 thoughtSignature"
        );
    }

    /// 「A / \n / ' B'」 1 （`strip` = not ）
    fn fold(strip: bool) -> String {
        let mut acc = CompletionAccumulator {
            strip_newline_chunks: strip,
            ..Default::default()
        };
        let telemetry = RequestTelemetry::new();
        for delta in [json!({"content": "A"}), json!({"content": "\n"}), json!({"content": " B"})] {
            let chunk = json!({
                "id": "c1",
                "model": "m",
                "created": 1,
                "choices": [{"index": 0, "delta": delta, "finish_reason": null}],
                "usage": Value::Null,
            });
            acc.consume_chunk(&chunk, &telemetry)
                .expect("测试里的帧不该被判成上游错误");
        }
        let body = acc.into_completion().body;
        body.pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn non_stream_folding_drops_the_keepalive_newlines_too() {
        // pathmust 1 ： 1 request need not need ， not this not 1 
        assert_eq!(fold(false), "A\n B", "关掉时照旧逐字拼接");
        assert_eq!(fold(true), "A B", "开启时非流式客户端也拿不到那一记换行");
    }
}
