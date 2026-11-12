//! Images response： not Chat ， OpenAI data or Gemini inlineData。

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use serde_json::{json, Value};

use crate::server::core::protocol::SseLineBuffer;
use crate::server::core::providers::images::ImageResponseKind;
use crate::server::errors::GatewayError;

use super::usage::RequestTelemetry;

pub(super) struct ImageFailure {
    pub status: u16,
    pub body: Value,
}

impl ImageFailure {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            status: 502,
            body: json!({ "message": message.into() }),
        }
    }
}

pub(super) enum ImageResponse {
    Live(reqwest::Response),
    Complete(Value),
}

impl ImageResponse {
    pub fn status(&self) -> reqwest::StatusCode {
        match self {
            Self::Live(response) => response.status(),
            Self::Complete(_) => reqwest::StatusCode::OK,
        }
    }
}

pub(super) async fn collect(
    response: reqwest::Response,
    kind: ImageResponseKind,
    transport: &super::request::TransportRequest,
    telemetry: &Arc<RequestTelemetry>,
) -> Result<Value, ImageFailure> {
    let budget = Duration::from_millis(crate::server::config::timeout_settings().body_ms());
    let reading = collect_body(response, kind, transport, telemetry);
    let timed = async {
        tokio::time::timeout(budget, reading)
            .await
            .unwrap_or_else(|_| {
                Err(ImageFailure::invalid(format!(
                    "生图响应超时（{}秒），上游可能已消耗额度",
                    budget.as_secs()
                )))
            })
    };
    if let Some(token) = telemetry.cancel_token() {
        tokio::select! {
            result = timed => result,
            _ = token.cancelled() => Err(ImageFailure {
                status: super::cancellation::MANUAL_TERMINATED_STATUS as u16,
                body: json!({ "message": super::cancellation::MANUAL_TERMINATED }),
            }),
        }
    } else {
        timed.await
    }
}

async fn collect_body(
    response: reqwest::Response,
    kind: ImageResponseKind,
    transport: &super::request::TransportRequest,
    telemetry: &Arc<RequestTelemetry>,
) -> Result<Value, ImageFailure> {
    const MAX_RESPONSE_BYTES: usize = 96 * 1024 * 1024;
    // ： in for 、 and to is ， need 1 ——
    // `doubao_body`（ that has Whymust in ）。
    if matches!(kind, ImageResponseKind::Doubao) {
        return doubao_body(response, transport, telemetry).await;
    }
    let status = response.status().as_u16();
    let sse = matches!(kind, ImageResponseKind::Gemini)
        && response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("text/event-stream"));
    let capture = telemetry.capture();
    if let Some(capture) = capture.as_deref() {
        capture.attach_response(status, response.headers());
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    let mut lines = SseLineBuffer::new();
    let mut total = 0usize;
    let mut images = Vec::new();
    let mut usage = None;
    let mut finished = false;
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|_| ImageFailure::invalid("上游生图响应传输中断，上游可能已消耗额度"))?;
        total = total.saturating_add(chunk.len());
        if total > MAX_RESPONSE_BYTES {
            return Err(ImageFailure::invalid("上游生图响应超过 96 MiB 上限"));
        }
        if !chunk.is_empty() {
            telemetry.note_first_frame();
        }
        if let Some(capture) = capture.as_deref() {
            capture.push(&chunk);
        }
        if sse {
            for line in lines.push(&chunk) {
                let Some(line) = line else {
                    finished = true;
                    continue;
                };
                let frame: Value = serde_json::from_str(&line)
                    .map_err(|_| ImageFailure::invalid("上游生图 SSE 含不完整的 JSON 帧"))?;
                gemini_frame(&frame, &mut images, &mut usage, &mut finished)?;
            }
        } else {
            bytes.extend_from_slice(&chunk);
        }
    }
    if sse {
        for line in lines.finish() {
            let Some(line) = line else {
                finished = true;
                continue;
            };
            let frame: Value = serde_json::from_str(&line)
                .map_err(|_| ImageFailure::invalid("上游生图 SSE 被截断"))?;
            gemini_frame(&frame, &mut images, &mut usage, &mut finished)?;
        }
        if !finished {
            return Err(ImageFailure::invalid(
                "上游生图流没有正常收尾，未将可能截断的图片当作成功",
            ));
        }
    } else {
        let payload: Value = serde_json::from_slice(&bytes)
            .map_err(|_| ImageFailure::invalid("上游生图响应不是有效 JSON"))?;
        if matches!(kind, ImageResponseKind::ModelscopeAsync) {
            return super::image_tasks::collect(&payload, transport, telemetry).await;
        }
        if matches!(kind, ImageResponseKind::Workbuddy) {
            return workbuddy_body(&payload, status, telemetry);
        }
        if matches!(kind, ImageResponseKind::Openai) {
            if payload.get("error").is_some_and(|error| !error.is_null()) {
                return Err(failure_from(&payload, status));
            }
            let data = payload.get("data").and_then(Value::as_array);
            if data.is_some_and(|data| !data.is_empty()) {
                if let Some(usage) = payload.get("usage") {
                    telemetry.report_usage(usage);
                }
                // has / ** in response**（ not in usage ）：
                // Loomy `points_consumed` is 。 by credit 1 ，
                // not is token。
                if let Some(credit) = payload
                    .as_object()
                    .and_then(|object| super::usage::credit_from_object(object))
                {
                    telemetry.report_usage(&json!({ "credit": credit }));
                }
                return Ok(payload);
            }
            return Err(failure_from(&payload, status));
        }
        gemini_frame(&payload, &mut images, &mut usage, &mut finished)?;
    }
    if images.is_empty() {
        return Err(ImageFailure::invalid(
            "上游没有返回生成图片（可能被内容策略拦截），未将文字回复当作生图成功",
        ));
    }
    let mut result = json!({ "created": crate::server::logging::now_ms() / 1000, "data": images });
    if let Some(usage) = usage {
        telemetry.report_usage(&usage);
        result["usage"] = usage;
    }
    Ok(result)
}

/// response： from for ，**none**。
///
/// ── Why in ────────────────────────────────────────
/// up no has ： is ** in for agent **， as 
/// `gen_image_block`（`block_type = 10010`） in 1 SSE 。 this 1 is 
/// `ImageResponseKind::Doubao` has （ OpenAI / Gemini in this 
/// not use ）， and `ModelscopeAsync`（ after ） is 1 ： 
/// `ImagePlan` " 1 request"， this need by 。
///
/// ── none from ────────────────────────────────────────
/// （2026-11-03） has ：`10010` to is ** **
/// （`~tplv-…-downsize_watermark…`，134 KB、 up 「AI 」），
/// `10084`（`rich_media_layout_block`） to **none**
/// （`~tplv-…-image_raw.png`）—— and 
/// `/creativity/resource/get_without_watermark` `cgen_wm0` ****
/// （4624288 、 1 md5）， and 1 to 。default 1 request；
/// [`doubao_without_watermark`] in no to when 1 （ old /）。
///
/// ── failure not ────────────────────────────────────────
/// not to ： has total， 1 。
async fn doubao_body(
    response: reqwest::Response,
    transport: &super::request::TransportRequest,
    telemetry: &Arc<RequestTelemetry>,
) -> Result<Value, ImageFailure> {
    /// this 1 agent will not ， to limit（ KB）。
    const MAX_STREAM_BYTES: usize = 8 * 1024 * 1024;
    let status = response.status().as_u16();
    let capture = telemetry.capture();
    if let Some(capture) = capture.as_deref() {
        capture.attach_response(status, response.headers());
    }
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk =
            chunk.map_err(|_| ImageFailure::invalid("上游生图响应传输中断，上游可能已消耗额度"))?;
        if bytes.len().saturating_add(chunk.len()) > MAX_STREAM_BYTES {
            return Err(ImageFailure::invalid("上游生图响应超过 8 MiB 上限"));
        }
        if let Some(capture) = capture.as_deref() {
            capture.push(&chunk);
        }
        bytes.extend_from_slice(&chunk);
    }
    let text = String::from_utf8_lossy(&bytes);
    let images = doubao_images(&text);
    if images.is_empty() {
        // up can can is （ / parameter）： error，
        // " no has "。 —— `data:` ， and 
        // is 1 JSON for （ has failure not SSE， that is ）。
        let failure = doubao_stream_failure(&text).or_else(|| {
            serde_json::from_str::<Value>(text.trim())
                .ok()
                .and_then(|value| doubao_value_failure(&value))
        });
        if let Some(failure) = failure {
            return Err(failure);
        }
        return Err(ImageFailure::invalid(
            "豆包没有返回图片（可能被内容策略拦截），未把文字回复当作生图成功",
        ));
    }
    // already none（`10084` `image_ori_raw`）； has 
    // —— that is 1 request， can （ `DoubaoImage::raw`）。
    let missing: Vec<DoubaoImage> = images
        .iter()
        .filter(|image| image.raw.is_none())
        .cloned()
        .collect();
    let clean = if missing.is_empty() {
        Vec::new()
    } else {
        doubao_without_watermark(transport, &missing).await
    };
    let data = images
        .iter()
        .map(|image| {
            let url = image
                .raw
                .clone()
                .or_else(|| {
                    clean
                        .iter()
                        .find(|(key, _)| key == &image.key)
                        .map(|(_, url)| url.clone())
                })
                .unwrap_or_else(|| image.preview.clone());
            json!({ "url": url })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "created": crate::server::logging::now_ms() / 1000,
        "data": data,
    }))
}

/// to 1 （ will 1 ， by for and ）。
#[derive(Clone)]
struct DoubaoImage {
    /// for （`tos-cn-i-…/rc_gen_image/x.jpeg`）， that is 。
    key: String,
    /// none（`10084` `image_ori_raw` / `~tplv-…-image_raw.png`）。
    ///
    /// （2026-11-03） 1 ： this `image_raw` and 
    /// `/creativity/resource/get_without_watermark` return `cgen_wm0` **
    /// **（4624288 、 1 md5）。 down 1 request，** not **
    /// ；`doubao_without_watermark` " no to " 。
    raw: Option<String>,
    /// （`10010` `uri` / `10084` `image_ori`）， after 。
    preview: String,
}

/// by for 1 and （ to 、 after to ，）。
fn doubao_merge(
    found: &mut Vec<DoubaoImage>,
    key: String,
    raw: Option<String>,
    preview: Option<String>,
) {
    if let Some(known) = found.iter_mut().find(|known| known.key == key) {
        if known.raw.is_none() {
            known.raw = raw;
        }
        if known.preview.is_empty() {
            known.preview = preview.unwrap_or_default();
        }
        return;
    }
    found.push(DoubaoImage {
        key,
        raw,
        preview: preview.unwrap_or_default(),
    });
}

/// from SSE （ by for 、 after ）。
///
/// ── need ────────────────────────────────────────────
/// `10010`（`gen_image_block`，） to is ** **；
/// `10084`（`rich_media_layout_block`，） to is **none**
/// （`image_ori_raw`） and for 。 before can to that ， after then in 
/// when no has —— ， by for and （ `DoubaoImage`）。
fn doubao_images(text: &str) -> Vec<DoubaoImage> {
    let mut found: Vec<DoubaoImage> = Vec::new();
    for data in doubao_data_frames(text) {
        let Ok(value) = serde_json::from_str::<Value>(&data) else {
            continue;
        };
        let mut blocks: Vec<&Value> = Vec::new();
        for patch in value
            .get("patch_op")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if patch.get("patch_object").and_then(Value::as_u64) != Some(1) {
                continue;
            }
            blocks.extend(
                patch
                    .pointer("/patch_value/content_block")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten(),
            );
        }
        for holder in [
            &value["content"]["content_block"],
            &value["message"]["content_block"],
        ] {
            blocks.extend(holder.as_array().into_iter().flatten());
        }
        for block in blocks {
            match block.get("block_type").and_then(Value::as_u64) {
                Some(10010) => {
                    for image in block
                        .pointer("/content/gen_image_block/images")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        let Some(url) = ["uri", "thumb_url"].iter().find_map(|key| {
                            image[*key]
                                .as_str()
                                .filter(|value| !value.trim().is_empty())
                        }) else {
                            continue;
                        };
                        let Some(key) = doubao_object_key(url) else {
                            continue;
                        };
                        doubao_merge(&mut found, key, None, Some(url.to_string()));
                    }
                }
                Some(10084) => {
                    for media in block
                        .pointer("/content/rich_media_layout_block/media")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        let image = &media["creation"]["image"];
                        let key = image["key"]
                            .as_str()
                            .filter(|value| !value.trim().is_empty())
                            .map(str::to_string)
                            .or_else(|| {
                                ["image_ori", "image_preview", "image_thumb"]
                                    .iter()
                                    .find_map(|field| {
                                        doubao_object_key(image[*field]["url"].as_str()?)
                                    })
                            });
                        let Some(key) = key else { continue };
                        let raw = image["image_ori_raw"]["url"]
                            .as_str()
                            .filter(|value| !value.trim().is_empty())
                            .map(str::to_string);
                        let preview = ["image_ori", "image_preview", "image_thumb"]
                            .iter()
                            .find_map(|field| {
                                image[*field]["url"]
                                    .as_str()
                                    .filter(|value| !value.trim().is_empty())
                            })
                            .map(str::to_string);
                        doubao_merge(&mut found, key, raw, preview);
                    }
                }
                _ => {}
            }
        }
    }
    found
}

/// → for 。
///
/// `https://host/tos-cn-i-…/rc_gen_image/x.jpeg~tplv-…?…`， for is 
/// `~` before 、 and host that 1 （`tos-cn-i-…/rc_gen_image/x.jpeg`）——
/// that is 。
fn doubao_object_key(url: &str) -> Option<String> {
    let without_query = url.split('?').next().unwrap_or(url);
    let path = without_query.split('~').next().unwrap_or(without_query);
    let key = path
        .strip_prefix("https://")
        .or_else(|| path.strip_prefix("http://"))
        .map(|rest| rest.split_once('/').map(|(_, path)| path).unwrap_or(""))
        .unwrap_or(path);
    (!key.is_empty()).then(|| key.to_string())
}

/// from SSE `data:` （ not `event:` —— this ）。
fn doubao_data_frames(text: &str) -> Vec<String> {
    let mut frames = Vec::new();
    for block in text.split("\n\n") {
        let mut data: Vec<&str> = Vec::new();
        for line in block.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                data.push(rest.strip_prefix(' ').unwrap_or(rest));
            }
        }
        if !data.is_empty() {
            frames.push(data.join("\n"));
        }
    }
    frames
}

/// no has when ， is up ：error， use to Reason。
fn doubao_stream_failure(text: &str) -> Option<ImageFailure> {
    doubao_data_frames(text)
        .iter()
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .find_map(|value| doubao_value_failure(&value))
}

/// 1 failure → 1 failure： code/message ， that `710022004`
/// to 1 can （ and 1 ）。
///
/// error （`error_code` / `error_msg` / `error_message` /
/// `detail`， `{"error_code":710022004,"error_msg":"rate limited","extra":…}`）：
/// not use `message` / `msg` —— has `message` this ** for **，
/// will 1 no up 。
fn doubao_value_failure(value: &Value) -> Option<ImageFailure> {
    let code = value.get("error_code").and_then(Value::as_u64);
    let message = ["error_msg", "error_message", "detail"]
        .iter()
        .find_map(|key| value[*key].as_str().filter(|text| !text.trim().is_empty()));
    if code.is_none() && message.is_none() {
        return None;
    }
    let message = message.unwrap_or("上游返回了无法解析的错误");
    if code == Some(710022004) {
        return Some(ImageFailure {
            status: 429,
            body: json!({
                "message": "豆包工作生图触发了上游人机验证：请在该账号的豆包网页端发一条消息完成验证后重试；也可以等待约 30 分钟或更换出口网络再试",
                "error_code": 710022004,
            }),
        });
    }
    Some(ImageFailure {
        status: 502,
        body: json!({ "message": format!("豆包工作生图失败：{message}"), "error_code": code }),
    })
}

/// for none（ for → ）。
///
/// failure 1 returnempty（ use ）： this 1 is " up "， not this as 
/// already 。
async fn doubao_without_watermark(
    transport: &super::request::TransportRequest,
    images: &[DoubaoImage],
) -> Vec<(String, String)> {
    let Ok(base) = url::Url::parse(&transport.url) else {
        return Vec::new();
    };
    let Ok(mut target) = base.join("/creativity/resource/get_without_watermark") else {
        return Vec::new();
    };
    // need ： that client parameter（device_id / pc_version / fp …） is 
    // this and ， will client。
    target.set_query(base.query());
    let keys: Vec<&str> = images.iter().map(|image| image.key.as_str()).collect();
    let client = crate::server::core::egress::client_without_redirects(transport.proxy.as_ref());
    let mut request = client
        .post(target)
        .timeout(Duration::from_secs(60))
        .json(&json!({ "uri": keys }));
    for (name, value) in &transport.headers {
        // `content-length` / `content-type` to reqwest by `.json()` ：
        // will （request is SSE that ）。
        if name.eq_ignore_ascii_case("content-length") || name.eq_ignore_ascii_case("content-type")
        {
            continue;
        }
        request = request.header(name, value);
    }
    let response = match request
        .header("Accept", "application/json, text/plain, */*")
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            crate::server::logging::verbose(
                "[Upstream]",
                &format!("nonefailure， use ：{error}"),
            );
            return Vec::new();
        }
    };
    let status = response.status().as_u16();
    let Ok(value) = response.json::<Value>().await else {
        crate::server::logging::verbose(
            "[Upstream]",
            "none response not is JSON， use ",
        );
        return Vec::new();
    };
    if !(200..300).contains(&status) || value.get("code").and_then(Value::as_i64).unwrap_or(0) != 0
    {
        crate::server::logging::verbose(
            "[Upstream]",
            &format!("none （HTTP {status}）， use "),
        );
        return Vec::new();
    }
    let data = value.get("data").unwrap_or(&Value::Null);
    if data.get("without_watermark").and_then(Value::as_bool) != Some(true) {
        crate::server::logging::verbose("[Upstream]", " this no has none， use ");
        return Vec::new();
    }
    let mut found = Vec::new();
    for key in ["download_image", "preview_image"] {
        for (uri, entry) in data
            .get(key)
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let Some(url) = entry
                .get("url")
                .and_then(Value::as_str)
                .filter(|url| !url.is_empty())
            else {
                continue;
            };
            found.push((uri.clone(), url.to_string()));
        }
        if !found.is_empty() {
            break;
        }
    }
    found
}

fn gemini_frame(
    frame: &Value,
    images: &mut Vec<Value>,
    usage: &mut Option<Value>,
    finished: &mut bool,
) -> Result<(), ImageFailure> {
    let response = frame.get("response").unwrap_or(frame);
    if response.get("error").is_some_and(|error| !error.is_null()) {
        return Err(failure_from(response, 200));
    }
    if frame.get("error").is_some_and(|error| !error.is_null()) {
        return Err(failure_from(frame, 200));
    }
    if let Some(metadata) = response
        .get("usageMetadata")
        .or_else(|| response.get("usage_metadata"))
    {
        let input = metadata
            .get("promptTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let output = metadata
            .get("candidatesTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let thought = metadata
            .get("thoughtsTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        *usage = Some(json!({
            "input_tokens": input,
            "output_tokens": output.saturating_add(thought),
            "total_tokens": metadata.get("totalTokenCount").and_then(Value::as_u64)
                .unwrap_or(input.saturating_add(output).saturating_add(thought)),
        }));
    }
    let Some(candidates) = response.get("candidates").and_then(Value::as_array) else {
        if response.pointer("/promptFeedback/blockReason").is_some() {
            return Err(ImageFailure {
                status: 400,
                body: json!({ "code": "content_policy_violation", "message": " up " }),
            });
        }
        return Ok(());
    };
    for candidate in candidates {
        if candidate.get("finishReason").and_then(Value::as_str) == Some("STOP") {
            *finished = true;
        }
        if let Some(parts) = candidate
            .pointer("/content/parts")
            .and_then(Value::as_array)
        {
            for part in parts {
                if part.get("thought").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                let Some(inline) = part.get("inlineData").or_else(|| part.get("inline_data"))
                else {
                    continue;
                };
                let Some(data) = inline
                    .get("data")
                    .and_then(Value::as_str)
                    .filter(|data| !data.is_empty())
                else {
                    continue;
                };
                let mime = inline
                    .get("mimeType")
                    .or_else(|| inline.get("mime_type"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !mime.starts_with("image/") {
                    continue;
                }
                if !images
                    .iter()
                    .any(|image| image.get("b64_json").and_then(Value::as_str) == Some(data))
                {
                    images.push(json!({ "b64_json": data, "mime_type": mime }));
                }
            }
        }
        if candidate
            .get("finishReason")
            .and_then(Value::as_str)
            .is_some_and(|reason| {
                matches!(reason, "SAFETY" | "IMAGE_SAFETY" | "PROHIBITED_CONTENT")
            })
        {
            return Err(ImageFailure {
                status: 400,
                body: json!({ "code": "content_policy_violation", "message": " up " }),
            });
        }
    }
    Ok(())
}

/// WorkBuddy response 1 `{code, msg, request_id, data:{data:[…], usage:{credit}}}`
/// ：`code != 0` is failure（HTTP can can is 200）， in `data.data[]`
/// （ and OpenAI ：`b64_json` or `url`），`data.usage.credit` is 
/// —— not is token， by `credit` up ， not is 
/// token usage。
fn workbuddy_body(
    payload: &Value,
    http_status: u16,
    telemetry: &Arc<RequestTelemetry>,
) -> Result<Value, ImageFailure> {
    let code = payload.get("code").and_then(Value::as_i64).unwrap_or(0);
    if code != 0 {
        return Err(failure_from(payload, http_status));
    }
    let envelope = payload.get("data").filter(|data| data.is_object());
    let data = envelope
        .and_then(|data| data.get("data"))
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
        .ok_or_else(|| failure_from(payload, http_status))?;
    if let Some(credit) = envelope.and_then(|data| data.pointer("/usage/credit")) {
        telemetry.report_usage(&json!({ "credit": credit.clone() }));
    }
    Ok(json!({
        "created": crate::server::logging::now_ms() / 1000,
        "data": data,
    }))
}

fn failure_from(payload: &Value, http_status: u16) -> ImageFailure {
    let error = payload.get("error").unwrap_or(payload);
    let code = error.get("code").cloned().unwrap_or(Value::Null);
    let message = error
        .get("message")
        .or_else(|| error.get("msg"))
        .and_then(Value::as_str)
        .unwrap_or(" up no has returnvalid ");
    let upstream_status = error.get("status").and_then(Value::as_str).unwrap_or("");
    let status = if http_status >= 400 {
        http_status
    } else {
        match upstream_status {
            "UNAUTHENTICATED" => 401,
            "PERMISSION_DENIED" => 403,
            "RESOURCE_EXHAUSTED" => 429,
            "INVALID_ARGUMENT" => 400,
            _ => code
                .as_u64()
                .filter(|code| (400..600).contains(code))
                .map(|code| code as u16)
                .unwrap_or(502),
        }
    };
    ImageFailure {
        status,
        body: json!({ "code": code, "message": format!("{upstream_status} {message}").trim() }),
    }
}

pub(super) fn output_error(message: impl Into<String>) -> GatewayError {
    GatewayError::with_status(502, message).with_code("image_generation_error")
}
