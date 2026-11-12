//! up **response** ： up not is chat SSE when ， chat SSE down 。
//!
//! ── Why in path up need need ────────────────────────────
//! `ForwardStream` and ** chat SSE**（ is SSE 1 
//! ：reasoning and 、usage 、model in that ）。 
//! path（`providers::custom::forward` `ProtocolTranslateStream`） already 
//! this ， that in `providers::custom` down 、and need slot /
//! connections parameter —— （ZCode ，
//! `providers::zcode::plan`） by ， this to 1 ** not ** 
//! ：、 to `ForwardStream::from_translated` or ， by 
//! `provider_loop` that by has then 。
//!
//! ── file up response（ 1 ， not use ）──────────
//!   1. **Anthropic Messages SSE**（`AnthropicToChatStream` ←
//! `protocol::anthropic_outbound::ChatFromAnthropicStream`）： has 
//! ZCode this ；
//!   2. **AI SDK v5 NDJSON**（`AiSdkNdjsonToChatStream` ←
//!      `protocol::ai_sdk_ndjson::ChatFromAiSdkNdjsonStream`）：
//! `api.commandcode.ai` `/alpha/generate` return
//! `application/x-ndjson`（ 1 1 JSON 、**HTTP 200**、error in ）。
//! not and ：（ has none `data:` before ） and not ，
//! and in ， no has up 。
//! 3. **Antigravity Gemini v1internal SSE**（`AntigravityToChatStream` ←
//!      `protocol::antigravity_stream::ChatFromAntigravityStream`）：Google Cloud
//! Code Assist `:streamGenerateContent?alt=sse`， is SSE is 
//! `{"response":{…gemini…}}` （），path（`candidates`
//! / `usageMetadata` / `thought` / `thoughtSignature`） is Gemini ——
//! and up no has can use ， 1 。
//! （ 1 error、 1 empty、 1 ），
//! has that and before —— down struct is 。
//!
//! ── and that （ is ）────────────────────
//! 1. `model` ** up **（ and 1 ）： `model` by down 
//! by `sse_model_rewrite()` need not need request；
//! 2. `model` not provider parameter： ，
//! ；
//! 3. in this ** up **（ before ）， and chat path「 up 
//! 」 1 —— after chat is in 。
//!
//! ── ────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use futures::Stream;

use crate::server::core::protocol::anthropic_outbound::ChatFromAnthropicStream;
use crate::server::core::protocol::antigravity_stream::ChatFromAntigravityStream;
use crate::server::core::protocol::chatglm_stream::ChatFromChatglmStream;
use crate::server::core::protocol::ai_sdk_ndjson::ChatFromAiSdkNdjsonStream;

/// up Anthropic SSE → chat SSE （`reqwest::Response` ）。
///
/// ：`response → empty → → `。error in when 
/// `io::Error`（ and `ForwardStream::new` 1 ）：empty is 
/// this ， down to is 1 —— is 「 up 」 and 「
/// empty when 」 in path by `ForwardStream` error、 in path 502，
/// and chat path error。
pub struct AnthropicToChatStream {
    /// up （ already error、 already empty）
    inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
    /// （ up → chat ）
    machine: ChatFromAnthropicStream,
    /// already down （ 1 up chunk can can ）
    pending: VecDeque<Bytes>,
    /// up already （ not poll up ， pending and None）
    upstream_done: bool,
    /// （None = ）
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

impl AnthropicToChatStream {
    /// `model` is ** up **（ down `model` ，module header 2 ）。
    pub fn new(
        response: reqwest::Response,
        model: &str,
        telemetry: &Arc<super::usage::RequestTelemetry>,
    ) -> Self {
        use futures::StreamExt;
        // reqwest error（ io::Error after can ）；
        // empty use set「request when 」 「responseempty when 」 that 1 ——
        // and chat path、path，set 1 
        let described = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        let guarded = super::stall::idle_guard(
            Box::pin(described),
            Duration::from_millis(crate::server::config::timeout_settings().stream_idle_ms()),
        );
        Self {
            inner: guarded,
            machine: ChatFromAnthropicStream::new(model),
            pending: VecDeque::new(),
            upstream_done: false,
            capture: telemetry.capture(),
        }
    }
}

impl Stream for AnthropicToChatStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            // down ， up —— will 1 up chunk
            // （ use mustparameter）
            if let Some(frame) = self.pending.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // up EOF：（finish_reason / usage / [DONE]）
                    self.upstream_done = true;
                    for frame in self.machine.finish() {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.machine.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    self.upstream_done = true;
                    // already in when （ inner ）： up ，
                    // by down / error or 502
                    return Poll::Ready(Some(Err(error)));
                }
            }
        }
    }
}

/// up **Command Code NDJSON** → chat SSE 
/// （`reqwest::Response` ）。
///
/// and [`AnthropicToChatStream`] ： is 
/// `response → empty → → `，error in when 
/// `io::Error`，`pending` 「 1 up chunk not 」
/// （ mustparameter —— is 1 `tool-call` ，
/// and usage must in after ），EOF when `machine.finish()`
/// 「 / 」 。
///
/// ── 1 ： up HTTP 200 ───────────────────────────
/// not is （failure、、 not in `{"type":"error"}`），
/// ** not ** by `response.status()` error —— old 200 and response
/// 1 ， by this error（ 502、
/// path error to client）。
pub struct AiSdkNdjsonToChatStream {
    /// up （ already error、 already empty）
    inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
    /// （ up NDJSON → chat ）
    machine: ChatFromAiSdkNdjsonStream,
    /// already down （ 1 up chunk can can ）
    pending: VecDeque<Bytes>,
    /// up already （ not poll up ， pending and None）
    upstream_done: bool,
    /// （None = ）
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

impl AiSdkNdjsonToChatStream {
    /// `model` is ** up **（ down `model` ，module header）。
    pub fn new(
        response: reqwest::Response,
        model: &str,
        telemetry: &Arc<super::usage::RequestTelemetry>,
    ) -> Self {
        use futures::StreamExt;
        // reqwest error（ io::Error after can ）；
        // empty use set「request when 」 「responseempty when 」 that 1 ——
        // and chat path、Anthropic that ，set 1 
        let described = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        let guarded = super::stall::idle_guard(
            Box::pin(described),
            Duration::from_millis(crate::server::config::timeout_settings().stream_idle_ms()),
        );
        Self {
            inner: guarded,
            machine: ChatFromAiSdkNdjsonStream::new(model),
            pending: VecDeque::new(),
            upstream_done: false,
            capture: telemetry.capture(),
        }
    }
}

impl Stream for AiSdkNdjsonToChatStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            // down ， up —— will 1 up chunk
            // （must / error）
            if let Some(frame) = self.pending.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // up EOF：（ finish / usage / [DONE]，
                    // or 「 no has 」 error —— in that ）
                    self.upstream_done = true;
                    for frame in self.machine.finish() {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.machine.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    self.upstream_done = true;
                    // already in when （ inner ）： up ，
                    // by down / error or 502
                    return Poll::Ready(Some(Err(error)));
                }
            }
        }
    }
}

/// up ** SSE（ + ）** → chat SSE 
/// （`reqwest::Response` ）。
///
/// and ： is 
/// `response → empty → → `，error in when 
/// `io::Error`，`pending` 「 1 up chunk not 」，
/// EOF when `machine.finish()` 「 / 」 。
///
/// ── and ： 1 「」 and 「」 ──
/// `part.status == "init"` is **add 1 **， finish to is 
/// ****（ `protocol::chatglm_stream` module header），
/// OpenAI need is 1 delta。 that 
/// （`protocol::chatglm_stream`） by ： down ，
/// no down after 。 1 by will ** 1 **
/// （ use 「AI 」）。
///
/// ── up is ──────────────────────────────────
/// return `text/event-stream`；**failure will HTTP 200 +
/// `application/json` return**（ `-1001 remaining not `）， that 1 by 
/// `ChatglmAdapter::is_error_response` in down and has error，
/// not this （ that ）。
pub struct ChatglmToChatStream {
    /// up （ already error、 already empty）
    inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
    /// （ → chat ）
    machine: ChatFromChatglmStream,
    /// already down （ 1 up chunk can can ）
    pending: VecDeque<Bytes>,
    /// up already （ not poll up ， pending and None）
    upstream_done: bool,
    /// （None = ）
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

impl ChatglmToChatStream {
    /// `model` is ** up **（ down `model` ，module header）。
    pub fn new(
        response: reqwest::Response,
        model: &str,
        telemetry: &Arc<super::usage::RequestTelemetry>,
    ) -> Self {
        use futures::StreamExt;
        // reqwest error（ io::Error after can ）；
        // empty use set「request when 」 「responseempty when 」 that 1 ——
        // and ，set 1 
        let described = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        let guarded = super::stall::idle_guard(
            Box::pin(described),
            Duration::from_millis(crate::server::config::timeout_settings().stream_idle_ms()),
        );
        Self {
            inner: guarded,
            machine: ChatFromChatglmStream::new(model),
            pending: VecDeque::new(),
            upstream_done: false,
            capture: telemetry.capture(),
        }
    }
}

impl Stream for ChatglmToChatStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            // down ， up —— will 1 up chunk
            // （must / error）
            if let Some(frame) = self.pending.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // up EOF：（ finish / usage / [DONE]，
                    // or 「 no has 」 error —— in that ）
                    self.upstream_done = true;
                    for frame in self.machine.finish() {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.machine.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    self.upstream_done = true;
                    // already in when （ inner ）： up ，
                    // by down / error or 502
                    return Poll::Ready(Some(Err(error)));
                }
            }
        }
    }
}

/// up **Antigravity Gemini v1internal SSE** → chat SSE 
/// （`reqwest::Response` ）。
///
/// and [`AnthropicToChatStream`] ： is 
/// `response → empty → → `，error in when 
/// `io::Error`，`pending` 「 1 up chunk not 」
/// （ and usage must in after ），EOF when `machine.finish()`
/// 「 / 」 。
///
/// ── and ： up is ─────────────────
/// error（`data: {"error":{…}}`） in ， HTTP 2xx is 
/// error（401/429/5xx）—— old by ， **200 
/// ** chat（errorerror）。 this and Command Code「HTTP 200、
/// in 」 that not 。
pub struct AntigravityToChatStream {
    /// up （ already error、 already empty）
    inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
    /// （Gemini → chat ）
    machine: ChatFromAntigravityStream,
    /// already down （ 1 up chunk can can ）
    pending: VecDeque<Bytes>,
    /// up already （ not poll up ， pending and None）
    upstream_done: bool,
    /// （None = ）
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

impl AntigravityToChatStream {
    /// `model` is ** up **（ down `model` ，module header）。
    /// `session` is request `sessionId`（ will ， from ；
    /// ， not to to empty —— 
    /// `protocol::antigravity_signature_cache` module header）。
    pub fn new(
        response: reqwest::Response,
        model: &str,
        session: &str,
        telemetry: &Arc<super::usage::RequestTelemetry>,
    ) -> Self {
        use futures::StreamExt;
        // reqwest error（ io::Error after can ）；
        // empty use set「request when 」 「responseempty when 」 that 1 ——
        // and ，set 1 
        let described = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        let guarded = super::stall::idle_guard(
            Box::pin(described),
            Duration::from_millis(crate::server::config::timeout_settings().stream_idle_ms()),
        );
        Self {
            inner: guarded,
            machine: ChatFromAntigravityStream::new(model, session),
            pending: VecDeque::new(),
            upstream_done: false,
            capture: telemetry.capture(),
        }
    }
}

impl Stream for AntigravityToChatStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            // down ， up —— will 1 up chunk
            // （must / error）
            if let Some(frame) = self.pending.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // up EOF：（ finish / usage / [DONE]，
                    // or 「 no has 」 error —— in that ）
                    self.upstream_done = true;
                    for frame in self.machine.finish() {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.machine.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                Poll::Ready(Some(Err(error))) => {
                    self.upstream_done = true;
                    // already in when （ inner ）： up ，
                    // by down / error or 502
                    return Poll::Ready(Some(Err(error)));
                }
            }
        }
    }
}
