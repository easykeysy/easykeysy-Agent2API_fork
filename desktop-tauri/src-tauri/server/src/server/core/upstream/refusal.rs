//! **** ：HTTP 200 can can is 1 。
//!
//! ── Why need need ────────────────────────────────────────────
//! has up （ OfficeAce / as AgentArts ） 「」 1 
//! ：`HTTP 200`、SSE ， is 1 （
//! 「 as 1 can ， no this …」），and 
//! `choices[].finish_reason` is `content_filter`。
//!
//! HTTP ， this 1 **success**： not will ，client to 
//! 「」 is 1 —— use as （：ZCode agentic request to 
//! ）。
//!
//! ── and in provider that ，「 etc 」 ──────
//! 「 `finish_reason` 」 is ** up **（ and 81004/11-128 ），
//! by `stream_refusal_finishes` ； 1 ： response
//! ** to can as **， to 、remaining、 and to `finish_reason`
//! 1 。** no has provider when 1 not ** —— use in 
//! `provider_loop` （response after 、Anthropic / chat down before ），
//! as empty when response， not and 。
//!
//! ── （ need ）────────────────────────────────────
//! 1. ** to `finish_reason`**： 1 to can ， not etc 。
//! 2. ** to limit**： can can ， not can none（otherwise etc ）。
//! has ， in limit ⇒ this 「」 for **** is in 。
//!
//! ── （ ， this by ）──────────
//! `finish_reason` in ****， 1 for etc 
//! 「 etc 」： not limit will ****， down from 「」
//! 「 to 」；limit by 2 before ， ≈ up this 
//! when 。 this not is ， is —— is 1 **** 200，
//! in before ， no has can 「」 and 「」。
//!
//! ── Whylimit 32 KiB（2026-10-10 use ： not ）─────────
//! up 50 token（≈300 ），32 KiB for has ， when to 
//! 「 before can can has 1 before 」 （ that no ）。limit can 
//! ， is this —— use not ， 32 KiB。
//!
//! ── not no （fail-open）────────────────────────────
//! error / empty when / no to `finish_reason` when return `None`， use 
//! by 「」 —— must is 「 1 」， not can is 
//! 「 1 」（ that will use can use in ，）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use std::time::Duration;

use axum::http::HeaderMap;
use bytes::Bytes;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde_json::Value;

use crate::server::config;
use crate::server::core::upstream::stall;

/// in 「 to `finish_reason`」 or 「 to limit」 when （module header）。
const HOLD_CAP_BYTES: usize = 32 * 1024;

/// 1 。
pub struct HeldHead {
    /// up HTTP （ before ， after `Response` already ）
    pub status: u16,
    /// up response（ up ； need ）
    pub headers: HeaderMap,
    /// to （****， 1 not ）
    pub buffered: Vec<u8>,
    /// remaining up （ after that ）
    pub rest: BoxStream<'static, Result<Bytes, std::io::Error>>,
    /// to `finish_reason`（`None` = no to / not ）
    pub finish_reason: Option<String>,
}

/// `finish_reason` is in 「」（ not ）。
///
/// `stream_refusal_finishes`；empty as `false`（default not ）。
pub fn is_refusal(finishes: &[&str], finish: &str) -> bool {
    finishes
        .iter()
        .any(|marker| marker.eq_ignore_ascii_case(finish))
}

/// is 1 「****」（，）。
///
/// `stream_refusal_markers`；empty as `None`（default not ）。
/// and [`is_refusal`] is 1 ： that `finish_reason`，
/// this —— has up in 、`finish_reason` not in 
/// （ trait up this ）。
///
/// to and 1 ： can （）， not can 
/// （ ）。****， not ——
/// by by 。
pub fn matched_refusal_marker<'a>(bytes: &[u8], markers: &'a [&'a str]) -> Option<&'a str> {
    markers
        .iter()
        .copied()
        .find(|marker| !marker.is_empty() && contains_bytes(bytes, marker.as_bytes()))
}

/// `haystack` is `needle`（empty needle ）。
///
/// and `api::pipeline` that ** not use **： that in `api` 、
/// as ； this in `upstream` 、 as 。 
/// 「/」 not ， to 1 will 。
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack.windows(needle.len()).any(|window| window == needle)
}

impl HeldHead {
    /// 「 to + remaining」 1 **** down 。
    ///
    /// in before （ 1 not ），remaining in after —— use to and 
    /// from up 。`capture` empty when to （
    /// that 1 ）： in `ForwardStream` before ， in this up ，otherwise
    /// will this 1 。
    pub fn into_stream(
        self,
        capture: Option<std::sync::Arc<crate::server::core::debug_traffic::TrafficCapture>>,
    ) -> BoxStream<'static, Result<Bytes, std::io::Error>> {
        let head = (!self.buffered.is_empty()).then(|| Bytes::from(self.buffered));
        if let (Some(bytes), Some(capture)) = (head.as_ref(), capture.as_ref()) {
            capture.push(bytes);
        }
        let rest = self.rest.map(move |item| {
            if let (Ok(bytes), Some(capture)) = (&item, &capture) {
                capture.push(bytes);
            }
            item
        });
        futures::stream::iter(head.into_iter().map(Ok)).chain(rest).boxed()
    }
}

/// 1 （ can can not ）SSE `data:` ，（** not **，
/// is ）。
///
/// ** for ** `data:` ， and need trim after `{` ——
/// no that not 1 （ 1 failure， will 「 no to 」
/// 「 no has 」）。`[DONE]` and （`: keepalive`）。
///
/// use is has ： path is ** 1 ** —— 「
/// `finish_reason`」（ this ） and 「 this 1 /」（`continuation` 
/// `TurnAccumulator`）。， will 1 1 、 1 ，
/// this will as 「 1 / 1 」， not 。
///
/// for `core::upstream` can （`pub(super)`） not is ： is this 1 
/// has ， not this by 。
pub(super) fn frame_payloads(text: &str) -> impl Iterator<Item = &str> {
    text.split("\n\n")
        .flat_map(|block| block.lines())
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .filter(|data| !data.is_empty() && *data != "[DONE]" && data.starts_with('{'))
}

/// from 1 （ can can not ）SSE ** 1 **empty `choices[].finish_reason`。
///
/// [`frame_payloads`]（ and 、`continuation` ）。
///
/// （ by by not ）； JSON 。
pub fn scan_finish_reason(text: &str) -> Option<String> {
    for data in frame_payloads(text) {
        let Ok(frame) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        let Some(choices) = frame.get("choices").and_then(Value::as_array) else {
            continue;
        };
        for choice in choices {
            if let Some(reason) = choice.get("finish_reason").and_then(Value::as_str) {
                if !reason.is_empty() {
                    return Some(reason.to_string());
                }
            }
        }
    }
    None
}

/// up to 「 can is 」 as （module header）。
///
/// is **** `reqwest::Response`（ use in after not need need ）：
/// and response in this ， up empty（ and `ForwardStream::new`
/// 1 ， is before to this —— up not can request）。
pub async fn hold_head(response: reqwest::Response) -> HeldHead {
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    // and `ForwardStream::new` / ：reqwest error io::Error
    let inner = response.bytes_stream().map(|item| {
        item.map_err(|error| {
            std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
        })
    });
    let mut source = stall::idle_guard(
        Box::pin(inner),
        Duration::from_millis(config::timeout_settings().stream_idle_ms()),
    );
    let mut buffered: Vec<u8> = Vec::new();
    let mut text = String::new();
    let mut finish_reason: Option<String> = None;
    loop {
        // already has no has （ 1 can can ）
        if let Some(reason) = scan_finish_reason(&text) {
            finish_reason = Some(reason);
            break;
        }
        if buffered.len() >= HOLD_CAP_BYTES {
            break;
        }
        match source.next().await {
            // / error / empty when ： not to ⇒ （fail-open，module header）
            None | Some(Err(_)) => break,
            Some(Ok(chunk)) => {
                buffered.extend_from_slice(&chunk);
                text.push_str(&String::from_utf8_lossy(&chunk));
            }
        }
    }
    HeldHead {
        status,
        headers,
        buffered,
        rest: source,
        finish_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scans_the_first_non_empty_finish_reason() {
        // finish_reason as null = not ； to stop
        let text = concat!(
            "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
        );
        assert_eq!(Some("stop".to_string()), scan_finish_reason(text));
        // content_filter （ in ）
        let text = "data: {\"choices\":[{\"delta\":{\"content\":\"x\"},\"finish_reason\":\"content_filter\"}]}\n\n";
        assert_eq!(Some("content_filter".to_string()), scan_finish_reason(text));
    }

    #[test]
    fn ignores_incomplete_trailing_frame_and_noise() {
        // no that not 1 （ JSON not can ）， this not to finish
        let text = "data: {\"choices\":[{\"delta\":{\"content\":\"x\"},\"finish_reason\":\"content_fil";
        assert_eq!(None, scan_finish_reason(text));
        // has null / empty / data / [DONE]： not 
        let text = concat!(
            ": keepalive\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"a\"},\"finish_reason\":\"\"}]}\n\n",
            "data: {\"choices\":[{\"delta\":{\"content\":\"b\"},\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n",
            "data: 不是一个 json\n\n",
        );
        assert_eq!(None, scan_finish_reason(text));
        // choice、 in when can to 
        let text = "data: {\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":null},{\"index\":1,\"delta\":{},\"finish_reason\":\"length\"}]}\n\n";
        assert_eq!(Some("length".to_string()), scan_finish_reason(text));
    }

    #[test]
    fn scan_tolerates_real_chunk_shape_with_usage_tail() {
        // usage （choices empty） not this 
        let text = concat!(
            "data: {\"id\":\"c1\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"好\"},\"finish_reason\":\"content_filter\"}],\"usage\":null}\n\n",
            "data: {\"id\":\"c1\",\"choices\":[],\"usage\":{\"prompt_tokens\":711006,\"completion_tokens\":47}}\n\n",
            "data: [DONE]\n\n",
        );
        assert_eq!(Some("content_filter".to_string()), scan_finish_reason(text));
        // usage not will 
        assert_eq!(
            None,
            scan_finish_reason("data: {\"choices\":[],\"usage\":{\"completion_tokens\":1}}\n\n")
        );
        let _ = json!({});
    }
}
