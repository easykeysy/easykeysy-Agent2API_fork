//! in ****（ OmniProxy `res.on('close')` ）。
//!
//! ── ──────────────────────────────────────────────
//! `record_started` status=0 after ， has ：handler to 
//! （`pipeline::record_entry` / `error_response`）， or path by 
//! `RecordingStream` in when settle。client in **response before **
//! when ，axum/hyper will handler future（：
//! handler， axum discussions #1094 / #2811）—— not to ，
//! in 「 in 」（ can etc 1 when ， 1 in request）。
//! this is use to 「 1 not in 」。
//!
//! 「handler 」**path**：Drop ， by 
//! 「client in response before 」 1 408 
//! （`RequestStats::finalize_interrupted`， UPDATE not INSERT， no has in 
//! when not ）。
//!
//! ── （pathmust 1 ）───────────────────────
//! - [`DisconnectGuard::complete`]： already （ / failure）→
//! and ****（request already not in ）；
//! - [`DisconnectGuard::handoff`]：path —— response already to axum，
//! `RecordingStream`（ has Drop ）， not ；**
//! ** —— no ， this request can 「request」
//! （ by `RecordingStream::settle` ）；
//! - not （Drop）：handler → 408 + 。
//!
//! ── Why use Drop not is ─────────────────────────
//! axum not 「client」 （ that is Node `res.on('close')` has ），
//! handler is 1 can ， Drop is in Rust 。
//! is （）， in Drop no has async 。
//!
//! ── Whyfile ────────────────────────────────────────
//! `pipeline.rs` already 1000 （ / / response / ），
//! and that ：「id + RequestStats + 」，
//! （chat / protocol ×2）create 1 ， and handler 。

use std::sync::Arc;

use crate::server::core::upstream::cancellation;
use crate::server::request_stats::RequestStats;

/// client in **response before ** when error need （）。
///
/// and `pipeline::STREAM_ABORTED` ： that is 「 already down 、 in 
/// 」， this is 「response no 、handler 」—— in in 
/// not ， can 1 in 1 。
pub const DISCONNECTED: &str = "客户端在响应完成前断开连接";

/// 「 in 」 （ and module header）。
pub struct DisconnectGuard {
    stats: Arc<RequestStats>,
    /// request id（ and in 、 1 ）
    id: String,
    /// is 「 no response」 （armed）
    armed: bool,
}

impl DisconnectGuard {
    /// in `record_started` after create（ in already in ）。
    pub fn new(stats: Arc<RequestStats>, id: String) -> Self {
        Self { stats, id, armed: true }
    }

    /// already —— and （request not in ）。
    pub fn complete(&mut self) {
        self.armed = false;
        cancellation::unregister(&self.id);
    }

    /// to response —— **** ： no ，
    /// this request can 「request」（ request is need need 
    /// that ）。 `pipeline::RecordingStream::settle`（ when ）。
    pub fn handoff(&mut self) {
        self.armed = false;
    }
}

impl Drop for DisconnectGuard {
    fn drop(&mut self) {
        // armed to this = handler in response before （client / ）：
        // 1 and ， this 1 in 「 in 」。
        // handoff after （armed = false） this ** not ** —— 
        // handler ， and `RecordingStream`。
        if self.armed {
            let stats = self.stats.clone();
            let id = self.id.clone();
            crate::server::core::blocking::in_place(move || stats.finalize_interrupted(&id, DISCONNECTED));
            cancellation::unregister(&self.id);
        }
    }
}
