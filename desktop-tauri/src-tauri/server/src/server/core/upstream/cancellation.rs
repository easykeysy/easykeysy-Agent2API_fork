//! ： in request and （ OmniProxy 
//! `utils/requestTermination.ts`）。
//!
//! ── ──────────────────────────────────────────────
//! request can to 1 request「 in 」， before no has down ：
//! up when can etc when （ or client ）。
//! to in request 1 **** and in （ = id），
//! 「request」 by id to and ； in 1 etc select
//! ， after etc 、 by 「」（error + 408 ）。
//!
//! ── Why not use tokio_util::sync::CancellationToken ──────────
//! that crate as 1 not （ ）。 this 
//! need has ——「 after all etc 」 and 「 etc 」——
//! `AtomicBool + Notify` can ， and and `upstream::mod` `InFlight`
//! （） use is 1 ， not as this 1 1 。
//!
//! ── （、）───────────────────────────────
//! - `register`： for in `record_started` after （ that when has id）；
//! - `unregister`：** when ** by after 1 path —— path is 
//! `api::pipeline::DisconnectGuard`（handler when ），path is 
//! `RecordingStream::settle`（ when ）。 is 「 has 」 etc 
//! 「 this request can 」， to 404 / 。
//! - after empty（）， by 
//! （`RequestStats::sweep_stale_running`）， not 。
//!
//! ── ────────────────────────────────────────────
//! and OmniProxy `registerActiveRequest` 1 ： 
//! request（ has 1 ；， 1 request in ，
//! this not to 「 not in in 」—— 1 「 already 」）。

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::stream::BoxStream;
use futures::{Future, Stream};
use tokio::sync::Notify;

use crate::server::errors::GatewayError;

/// 1 （ error / client error / use ）。
pub const MANUAL_TERMINATED: &str = "请求已被手动终止";

/// in 。
///
/// and 「 in 」 1 （client、 408）： for is 
/// 「 this 1 no has 」， use 1 not will 「 use 」 up 
/// （ that need error ， and `finish_stale_running` 1 ）。
pub const MANUAL_TERMINATED_STATUS: i64 = 408;

/// error（ ；client and to is ）。
pub fn cancelled_error() -> GatewayError {
    GatewayError::with_status(MANUAL_TERMINATED_STATUS as i32, MANUAL_TERMINATED)
}

/// 1 in request 。
///
/// is ** to and etc ** ： after allcurrent and will `cancelled()` return，
/// `cancel()` not has （ etc ， not ）。
pub struct CancelToken {
    /// already （`cancelled()` ，「 etc 」）
    flag: AtomicBool,
    /// （`notify_waiters` ** when already ** etc ，
    /// `cancelled()` must「、」， not ）
    notify: Notify,
}

impl CancelToken {
    fn new() -> Self {
        Self { flag: AtomicBool::new(false), notify: Notify::new() }
    }

    /// and all etc （ etc ； and `InFlight::finish` 1 ）
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    /// is already （， check）
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }

    /// to （ already then return）。
    ///
    /// 「、」 and `upstream::mod` `wait_for_in_flight`
    /// ：`notify_waiters` when already etc ， will 
    /// 「 to await 」 。
    pub async fn cancelled(&self) {
        let mut notified = std::pin::pin!(self.notify.notified());
        // enable() return true = in before already to （ this not can can ，
        // as notify in cancel when and ； by has ）
        if notified.as_mut().enable() {
            return;
        }
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

/// 1 up and as 「 can 」 。
///
/// as None when return（ use ）。
///
/// ── Why not use `futures::stream::select`（）─────────────
/// `select` is 「****」。 in to when 
/// 1 ， up when in pending —— is `poll_next`
/// not to `Ready(None)`，path when ：
/// - （`ForwardStream`）：client already all（ `data: [DONE]`），
/// response not 、 not ， `chunked` not ——
/// client 1 etc ，request in 「response in 」， to when ；
/// - （`aggregate_frame_stream_inner`）：`while let Some(..) = next()`
/// not ， 1 empty to total when （default 300 seconds） 502。
/// not is 「 up has 」， is 。
///
/// this ， has ：
/// - ** up None **， not etc （ this is select that ）；
/// - up in pending when —— and **** up ，
/// this after down 1 poll to error， not etc down 1 to 
/// （ up when is this ，module header）。
///
/// and `upstream::stall::idle_guard` 1 （ `done` + ），
/// is 1 。
pub fn cancellable(
    inner: BoxStream<'static, Result<Bytes, std::io::Error>>,
    cancel: Option<Arc<CancelToken>>,
) -> BoxStream<'static, Result<Bytes, std::io::Error>> {
    let Some(token) = cancel else {
        return inner;
    };
    Box::pin(CancellableStream {
        inner,
        // etc in this future ：（`cancelled()` 
        // 「、」 ， ）
        wait: Box::pin(async move { token.cancelled().await }),
        done: false,
    })
}

struct CancellableStream {
    inner: BoxStream<'static, Result<Bytes, std::io::Error>>,
    wait: Pin<Box<dyn Future<Output = ()> + Send>>,
    /// already （ up or already ）—— after poll 1 return None
    done: bool,
}

impl Stream for CancellableStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        // ： after to error， not etc up down 1 
        if self.wait.as_mut().poll(cx).is_ready() {
            self.done = true;
            return Poll::Ready(Some(Err(std::io::Error::other(MANUAL_TERMINATED))));
        }
        match self.inner.as_mut().poll_next(cx) {
            // up ：（** not **， `cancellable` ）
            Poll::Ready(None) => {
                self.done = true;
                Poll::Ready(None)
            }
            other => other,
        }
    }
}

/// in request： id → 。
fn registry() -> &'static Mutex<HashMap<String, Arc<CancelToken>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Arc<CancelToken>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// ； in not （ and / 1 ： is ，
/// no has will panic in ）
fn lock_registry() -> MutexGuard<'static, HashMap<String, Arc<CancelToken>>> {
    match registry().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// 1 in request，return 。
///
/// `id` as emptyreturn None（ that request no has in ，none from —— and 
/// `record_started` for empty id ）。 id will old ：
/// id is request new UUID，path not will ； up （） when new 1 
/// as ， old has （ up 1 ） to is Arc， not 。
pub fn register(id: &str) -> Option<Arc<CancelToken>> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    let token = Arc::new(CancelToken::new());
    lock_registry().insert(id.to_string(), token.clone());
    Some(token)
}

/// 1 in request（ etc ： not in when not ）。
pub fn unregister(id: &str) {
    let id = id.trim();
    if id.is_empty() {
        return;
    }
    lock_registry().remove(id);
}

/// 1 。
///
/// return `false` = this request not in in —— already 、 up ，
/// or id 。 use （） to 404 not is 「 already 」。
pub fn cancel(id: &str) -> bool {
    let id = id.trim();
    if id.is_empty() {
        return false;
    }
    // after ****： will etc ， not this in 
    // （ and `flush_live` for `live` 1 ）
    let token = lock_registry().get(id).cloned();
    match token {
        Some(token) => {
            token.cancel();
            true
        }
        None => false,
    }
}
