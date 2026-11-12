//! up empty： none when （ for OmniProxy `stallGuard`）。
//!
//! ── Why need need （ not is when ）────────────────────────
//! `read_timeout` is ** after **（「 etc response」 and 「empty」 ，
//! `core::egress`）—— must not will as ，
//! is 「 etc response」「empty」 when ，empty when is this 。
//!
//! and set「responseempty when 」 for ：
//! no has new 。** to new when **，and when in 
//! when already —— 「 up request not 1 」 
//! （ etc response when to response as ， after has 1 ）。
//!
//! ── after ─────────────────────────────────────────────
//! 1 `io::Error` error and 。 by has ：
//! `ForwardStream` 「error + [DONE]」（client already to 200 and ），
//! 502。 [`IDLE_TIMEOUT_PREFIX`] ，`ForwardStream`
//! not 「 up in 」 before —— not is in ， is （ and 
//! `cancellation::MANUAL_TERMINATED` 1 ）。

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::Bytes;
use futures::stream::BoxStream;
use futures::{Future, Stream};

/// empty when error before （module header 「 after 」）
pub const IDLE_TIMEOUT_PREFIX: &str = "流式响应空闲超时";

/// to 1 up up empty（`idle` set「responseempty when 」）。
///
/// / is `BoxStream<Result<Bytes, io::Error>>`： use 
/// （`ForwardStream::new` and ） already in when reqwest error
/// `io::Error`（ ）， this when 。
pub fn idle_guard(
    inner: BoxStream<'static, Result<Bytes, std::io::Error>>,
    idle: Duration,
) -> BoxStream<'static, Result<Bytes, std::io::Error>> {
    Box::pin(IdleGuard {
        inner,
        idle,
        sleep: Box::pin(tokio::time::sleep(idle)),
        done: false,
    })
}

struct IdleGuard {
    inner: BoxStream<'static, Result<Bytes, std::io::Error>>,
    idle: Duration,
    /// to ； to 1 （module header）
    sleep: Pin<Box<tokio::time::Sleep>>,
    /// already （ when or up ）—— after poll 1 return None
    done: bool,
}

impl Stream for IdleGuard {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }
        match self.inner.as_mut().poll_next(cx) {
            // to ： when down （ after return， this 1 
            // 「 after empty」 down 1 ）
            Poll::Ready(Some(item)) => {
                let deadline = tokio::time::Instant::now() + self.idle;
                self.sleep.as_mut().reset(deadline);
                Poll::Ready(Some(item))
            }
            Poll::Ready(None) => {
                self.done = true;
                Poll::Ready(None)
            }
            Poll::Pending => match self.sleep.as_mut().poll(cx) {
                // up idle： and （ 1 ）
                Poll::Ready(()) => {
                    self.done = true;
                    Poll::Ready(Some(Err(std::io::Error::other(format!(
                        "{IDLE_TIMEOUT_PREFIX}({}秒)",
                        self.idle.as_secs()
                    )))))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}
