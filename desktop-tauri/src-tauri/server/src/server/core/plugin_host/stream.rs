//! 。
//!
//! ── this ────────────────────────────────────────
//! `ForwardOutcome::Stream` need 1 ** ** `Stream`， in 1 
//! 。 after ， this to is 「 1 none + 1 
//! 」， that new 1 `Stream` —— up （`chat.rs` 
//! `RecordingStream`、、） to and 1 。
//!
//! ── in this ──────────────────────────────────────────
//! is [`proto::DEFAULT_WINDOW_BYTES`]， 1 chunk 
//! `forget()` etc （ SDK `StreamSink`）。 has 
//! `credit{n}` 。** 1 chunk etc **：
//! client → up not poll → not → in `send` up 。
//! this is 「 」，。
//!
//! ── Why need error ──────────────────────────────────
//! client when already to HTTP 200 and ， 1 can 「 this is failure 」
//! is in 1 error（ `ForwardStream` is 1 ）。
//! can can already 1 —— before ： already has `[DONE]`
//! not ，client to 。

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::Stream;
use serde_json::{json, Value};

use agent2api_provider_protocol as proto;

use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::errors::GatewayError;
use crate::server::logging;

pub(crate) type FailureSlot = Arc<Mutex<Option<GatewayError>>>;
pub(crate) type PrivateFailureSlot = Arc<Mutex<Option<Value>>>;

use super::errors::strict;
use super::{Host, StreamEvent};

/// Chat SSE usage； has ， not down 。
#[derive(Default)]
struct UsageSniffer {
    strict_provider: Option<&'static str>,
    line: Vec<u8>,
    data: Vec<u8>,
    line_bytes: usize,
    event_bytes: usize,
    skipping_event: bool,
    skip_lf: bool,
    first_line_seen: bool,
    done: bool,
    failure: Option<Value>,
}

impl UsageSniffer {
    const MAX_EVENT_BYTES: usize = 1024 * 1024;

    fn feed(&mut self, bytes: &[u8], telemetry: &RequestTelemetry) {
        for &byte in bytes {
            if self.skip_lf {
                self.skip_lf = false;
                if byte == b'\n' { continue; }
            }
            if byte == b'\r' || byte == b'\n' {
                self.consume_line(telemetry);
                self.skip_lf = byte == b'\r';
            } else {
                self.line_bytes = self.line_bytes.saturating_add(1);
                self.event_bytes = self.event_bytes.saturating_add(1);
                if self.event_bytes > Self::MAX_EVENT_BYTES {
                    self.line.clear();
                    self.data.clear();
                    // in down 1 empty， not can new 。
                    self.skipping_event = true;
                }
                if !self.skipping_event { self.line.push(byte); }
            }
        }
    }

    fn finish(&mut self, telemetry: &RequestTelemetry) {
        if self.line_bytes > 0 { self.consume_line(telemetry); }
        if !self.skipping_event { self.consume_data(telemetry, false); }
        self.line.clear();
        self.data.clear();
        self.line_bytes = 0;
        self.event_bytes = 0;
        self.skipping_event = false;
        self.skip_lf = false;
    }

    fn consume_line(&mut self, telemetry: &RequestTelemetry) {
        if self.line_bytes > 0 {
            self.event_bytes = self.event_bytes.saturating_add(1);
            if self.event_bytes > Self::MAX_EVENT_BYTES {
                self.line.clear();
                self.data.clear();
                self.skipping_event = true;
            }
        }
        if self.line_bytes == 0 {
            if !self.skipping_event { self.consume_data(telemetry, true); }
            self.data.clear();
            self.event_bytes = 0;
            self.skipping_event = false;
        } else if !self.skipping_event {
            let line = if !self.first_line_seen {
                self.line.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&self.line)
            } else { &self.line };
            let payload = line.strip_prefix(b"data:").or_else(|| (line == b"data").then_some(&[][..]));
            if let Some(payload) = payload {
                let payload = payload.strip_prefix(b" ").unwrap_or(payload);
                self.data.extend_from_slice(payload);
                self.data.push(b'\n');
            }
        }
        self.first_line_seen = true;
        self.line.clear();
        self.line_bytes = 0;
    }

    fn consume_data(&mut self, telemetry: &RequestTelemetry, complete: bool) {
        let Ok(text) = std::str::from_utf8(&self.data) else { return };
        if text.trim() == "[DONE]" {
            self.done |= complete;
            return;
        }
        if ![b"usage".as_slice(), b"error".as_slice()]
            .iter().any(|field| self.data.windows(field.len()).any(|window| window == *field))
        {
            return;
        }
        let Ok(chunk) = serde_json::from_slice::<Value>(&self.data) else { return };
        if let Some(usage) = chunk.get("usage").filter(|value| value.is_object()) {
            telemetry.report_usage(usage);
        }
        if let Some(provider) = self.strict_provider {
            if let Some((failure, private)) = strict::stream_failure(provider, &chunk) {
                let prior_interruption = self.failure.as_ref().is_some_and(|body| {
                    matches!(body.get("statusCode").and_then(Value::as_u64), Some(499 | 504))
                });
                if self.failure.is_none() || strict::stopped(&failure) && !prior_interruption {
                    self.failure = Some(private);
                }
                telemetry.note_error(&failure.message);
            }
            return;
        }
        if let Some(error) = chunk.get("error").filter(|value| crate::server::core::protocol::is_truthy(value)) {
            if self.failure.is_none() { self.failure = Some(error.clone()); }
            let message = error.get("message").and_then(Value::as_str)
                .or_else(|| error.as_str()).filter(|text| !text.trim().is_empty())
                .map(str::to_string).unwrap_or_else(|| "上游流式返回错误".to_string());
            telemetry.note_error(&message);
        }
    }
}

/// 1 in 。
pub(crate) struct RpcStream {
    /// （Drop when need abort / ）。
    host: Arc<Host>,
    lease: Option<Arc<super::activity::UsageLease>>,
    /// use RPC id。
    id: i64,
    /// chunk / 。
    events: tokio::sync::mpsc::UnboundedReceiver<StreamEvent>,
    /// （module header）。 = ，。
    credit: tokio::sync::mpsc::UnboundedSender<usize>,
    /// down （error / `[DONE]`）。
    pending: VecDeque<Bytes>,
    /// already ， down 1 poll return `None`。
    ended: bool,
    /// usage / 。
    telemetry: Arc<RequestTelemetry>,
    usage: UsageSniffer,
    failure: Option<FailureSlot>,
    private_failure: Option<PrivateFailureSlot>,
}

impl RpcStream {
    /// 1 ， and 1 ****。
    ///
    /// ── Why need 1 ────────────────────────────────
    /// `poll_next` is ， need stdin（）。 in `poll_next`
    /// `tokio::spawn` 1 will chunk 1 （ up is 
    /// seconds ）， is 1 1 ： in `recv` up ， 1 1 。
    ///
    /// `pending` is 「 already from 、 no down 」 （
    /// [`Self::with_pending`] ）。
    pub(super) fn with_pending(
        provider: &'static str,
        host: Arc<Host>,
        id: i64,
        events: tokio::sync::mpsc::UnboundedReceiver<StreamEvent>,
        pending: Option<Vec<u8>>,
        telemetry: Arc<RequestTelemetry>,
        failure: Option<FailureSlot>,
        private_failure: Option<PrivateFailureSlot>,
        lease: Arc<super::activity::UsageLease>,
    ) -> Self {
        let (credit, mut credit_rx) = tokio::sync::mpsc::unbounded_channel::<usize>();
        let pump_host = host.clone();
        tokio::spawn(async move {
            while let Some(n) = credit_rx.recv().await {
                pump_host.credit(id, n).await;
            }
        });
        let mut queue = VecDeque::new();
        let mut usage = UsageSniffer {
            strict_provider: strict::applies(provider).then_some(provider),
            ..UsageSniffer::default()
        };
        if let Some(bytes) = pending {
            usage.feed(&bytes, &telemetry);
            queue.push_back(Bytes::from(bytes));
        }
        Self {
            host,
            lease: Some(lease),
            id,
            events,
            credit,
            pending: queue,
            ended: false,
            telemetry,
            usage,
            failure,
            private_failure,
        }
    }

    fn capture_failure(&self, error: GatewayError) {
        if let Some(slot) = &self.failure {
            let mut slot = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            // already / when /failure not can 。
            if slot.is_none() || self.usage.strict_provider.is_some() && strict::stopped(&error)
                && !slot.as_ref().is_some_and(|prior| matches!(prior.status_code, 499 | 504)) {
                *slot = Some(error);
            }
        }
    }

    fn capture_frame_failure(&self) {
        let Some(error) = &self.usage.failure else { return };
        if let Some(provider) = self.usage.strict_provider {
            if let Some((failure, private)) = strict::stream_failure(provider, &json!({ "error": error })) {
                if let Some(slot) = &self.private_failure {
                    let mut slot = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    if slot.is_none() || strict::stopped(&failure) { *slot = Some(private); }
                }
                self.capture_failure(failure);
            }
            return;
        }
        let status = error.get("status").or_else(|| error.get("statusCode")).and_then(Value::as_u64)
            .and_then(|status| i32::try_from(status).ok()).filter(|status| (400..600).contains(status)).unwrap_or(502);
        let mut failure = GatewayError::with_status(status,
            error.get("message").and_then(Value::as_str).unwrap_or("上游流式返回错误"));
        failure.code = error.get("code").and_then(|value| value.as_str().map(str::to_string)
            .or_else(|| value.as_i64().map(|code| code.to_string())))
            .or_else(|| error.get("type").and_then(Value::as_str).map(str::to_string));
        failure.upstream_code = error.get("upstreamCode").or_else(|| error.get("upstream_code"))
            .and_then(Value::as_i64).or_else(|| error.get("code").and_then(Value::as_i64));
        self.capture_failure(failure);
    }

    /// up 「error + [DONE]」。
    fn finish_with_error(&mut self, message: &str) {
        self.usage.finish(&self.telemetry);
        self.capture_frame_failure();
        if let Some(provider) = self.usage.strict_provider {
            let failure = strict::fatal(provider, 502, "runtime_stream_interrupted");
            self.finish_strict_error(failure);
            return;
        }
        self.capture_failure(GatewayError::with_status(502, message).with_code("runtime_stream_interrupted"));
        self.ended = true;
        logging::console_line("[Model]", &format!("❌ {message}"));
        self.telemetry.note_error(message);
        if self.usage.done {
            // already ： 1 will client to 
            return;
        }
        self.pending.push_back(crate::server::core::upstream::sse::sse_frame(&json!({
            "error": {
                "message": message,
                "type": "proxy_error",
            }
        })));
        if self.failure.is_none() {
            self.pending.push_back(Bytes::from_static(b"data: [DONE]\n\n"));
        }
    }
    fn finish_strict_error(&mut self, failure: GatewayError) {
        let failure = self.failure.as_ref().and_then(|slot| {
            let guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.as_ref().filter(|prior| matches!(prior.status_code, 499 | 504)).cloned()
        }).unwrap_or(failure);
        if strict::stopped(&failure) {
            if let Some(slot) = &self.private_failure {
                slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take();
            }
        }
        self.capture_failure(failure.clone());
        self.ended = true;
        logging::console_line("[Model]", &failure.message);
        self.telemetry.note_error(&failure.message);
        if !self.usage.done {
            self.pending.push_back(crate::server::core::upstream::sse::sse_frame(&failure.payload()));
        }
    }
}

impl Stream for RpcStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        loop {
            if let Some(frame) = this.pending.pop_front() {
                return Poll::Ready(Some(Ok(frame)));
            }
            if this.ended {
                return Poll::Ready(None);
            }
            match this.events.poll_recv(context) {
                Poll::Pending => return Poll::Pending,
                // no has `Done`： （ no / ）。
                // not error ，client will 。
                Poll::Ready(None) => this.finish_with_error("插件流在收尾信号之前中断"),
                // head（）： 1 ， after 。
                // not is as not —— 1 not this 。
                Poll::Ready(Some(StreamEvent::Head(_))) => {}
                Poll::Ready(Some(StreamEvent::Chunk(bytes))) => {
                    // etc ： + （ and 1 ，
                    // 1 will or ）
                    let _ = this.credit.send(bytes.len() + proto::FRAME_OVERHEAD_BYTES);
                    this.usage.feed(&bytes, &this.telemetry);
                    this.capture_frame_failure();
                    this.telemetry.note_first_frame();
                    return Poll::Ready(Some(Ok(Bytes::from(bytes))));
                }
                Poll::Ready(Some(StreamEvent::Failed(message))) => {
                    if let Some(provider) = this.usage.strict_provider {
                        this.usage.finish(&this.telemetry);
                        this.capture_frame_failure();
                        this.finish_strict_error(strict::fatal(provider, 502, "runtime_stream_interrupted"));
                    } else { this.finish_with_error(&message); }
                }
                Poll::Ready(Some(StreamEvent::Done(Ok(_)))) => {
                    this.usage.finish(&this.telemetry);
                    this.capture_frame_failure();
                    // 。 this `[DONE]`； in this up 
                    // —— 1 will client 1 etc （ not ）。
                    if !this.usage.done {
                        if this.failure.is_some() {
                            this.finish_with_error("辅助程序流缺少真实 [DONE]，未补成功终态");
                            continue;
                        }
                        this.pending.push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                    }
                    this.ended = true;
                }
                Poll::Ready(Some(StreamEvent::Done(Err(error)))) => {
                    if let Some(provider) = this.usage.strict_provider {
                        this.usage.finish(&this.telemetry);
                        this.capture_frame_failure();
                        if let Some((_, private)) = strict::private_control(&error) {
                            if let Some(slot) = &this.private_failure {
                                let mut slot = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                                if slot.is_none() { *slot = Some(private); }
                            }
                        }
                        let failure = super::rpc::rpc_error(provider, error, proto::methods::gateway::CONVERSATION);
                        this.finish_strict_error(failure);
                    } else {
                        this.capture_failure(super::rpc::rpc_error("", error.clone(), proto::methods::gateway::CONVERSATION));
                        this.finish_with_error(&error.message);
                    }
                }
            }
        }
    }
}

impl Drop for RpcStream {
    fn drop(&mut self) {
        let unfinished = self.host.close_stream(self.id);
        if unfinished {
            let host = self.host.clone();
            let id = self.id;
            let lease = self.lease.take();
            crate::spawn_task(async move {
                let _lease = lease;
                host.abort(id).await;
            });
        }
    }
}
