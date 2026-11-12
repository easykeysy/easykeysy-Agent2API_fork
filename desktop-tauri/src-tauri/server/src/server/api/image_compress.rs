//! will set：`GET/PUT /api/image-compress`。
//!
//! ── this set ──────────────────────────────────────────
//! before for request** base64 ** to limit and JPEG，
//! request、 by up token ； 1 in 
//! when use （ [`crate::server::core::upstream::image_compress`]）。
//!
//! - `imageCompressEnabled`：total（default****， `config::KEY_IMAGE_COMPRESS_ENABLED`）；
//! - `imageCompressMaxEdge`：limit（default 2048， 256~8192）；
//! - `imageCompressQuality`：JPEG （default 80， 40~95）；
//! - `imageCompressThresholdKb`： KB（default 512， 0~10240）。
//!
//! ── use in for ────────────────────────────────────────
//! in `core::upstream::payload::send_body`： / 、
//! in this return， not 。 this set** not will **。
//!
//! ── and /api/auto-continue 1 ────────────────────────
//! 1 + if has 、GET / PUT （）、response is 
//! new 、 protected（ is to up ）。 after for down 1 request
//! （request）， not 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Map, Value};

use crate::server::config::parse::int_in_range;
use crate::server::config::{
    self, ImageCompressPatch, ImageCompressSettings, IMAGE_COMPRESS_MAX_MAX_EDGE,
    IMAGE_COMPRESS_MAX_QUALITY, IMAGE_COMPRESS_MAX_THRESHOLD_KB, IMAGE_COMPRESS_MIN_MAX_EDGE,
    IMAGE_COMPRESS_MIN_QUALITY, IMAGE_COMPRESS_MIN_THRESHOLD_KB, KEY_IMAGE_COMPRESS_ENABLED,
    KEY_IMAGE_COMPRESS_MAX_EDGE, KEY_IMAGE_COMPRESS_QUALITY, KEY_IMAGE_COMPRESS_THRESHOLD_KB,
};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/image-compress —— current and parameter
pub async fn get_image_compress(State(_state): State<ServerState>) -> Response {
    ok_json(state_json(config::image_compress_settings()))
}

/// PUT /api/image-compress —— body `{imageCompressEnabled?, imageCompressMaxEdge?, …}`
///
/// （ not ），（ and `put_auto_continue` /
/// `put_retry` 1 ），「、parameter no 」 set。config after 
/// down 1 request。
pub async fn put_image_compress(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let patch = match patch_from(object) {
        Ok(patch) => patch,
        Err(message) => return errors::management_error(400, message),
    };
    if patch == ImageCompressPatch::default() {
        // 1 no to ： not 、 not ， current to use 
        return ok_json(state_json(config::image_compress_settings()));
    }
    if !config::set_image_compress(patch) {
        logging::log("[Config]", "⚠️  图片压缩设置写入失败，本次运行内仍生效");
    }
    logging::log("[Config]", &change_note(&patch));
    ok_json(state_json(config::image_compress_settings()))
}

/// request → update（`Err` is 400 ）。
///
/// （ and `api::auto_continue::patch_from` 1 ）：
/// need can ， handler need need `ServerState` not 。
///
/// `null` 1 not （ PUT no has 「 this 1 」 ，「 not 」）。
fn patch_from(object: &Map<String, Value>) -> Result<ImageCompressPatch, String> {
    let mut patch = ImageCompressPatch::default();
    if let Some(value) = object.get(KEY_IMAGE_COMPRESS_ENABLED) {
        let Some(enabled) = value.as_bool() else {
            return Err(format!("{KEY_IMAGE_COMPRESS_ENABLED} 必须是 true 或 false"));
        };
        patch.enabled = Some(enabled);
    }
    if let Some(value) = object.get(KEY_IMAGE_COMPRESS_MAX_EDGE) {
        let Some(number) = int_in_range(value, IMAGE_COMPRESS_MIN_MAX_EDGE, IMAGE_COMPRESS_MAX_MAX_EDGE)
        else {
            return Err(format!(
                "{KEY_IMAGE_COMPRESS_MAX_EDGE} 必须是 {IMAGE_COMPRESS_MIN_MAX_EDGE}~{IMAGE_COMPRESS_MAX_MAX_EDGE} 的整数"
            ));
        };
        patch.max_edge = Some(number);
    }
    if let Some(value) = object.get(KEY_IMAGE_COMPRESS_QUALITY) {
        let Some(number) = int_in_range(value, IMAGE_COMPRESS_MIN_QUALITY, IMAGE_COMPRESS_MAX_QUALITY)
        else {
            return Err(format!(
                "{KEY_IMAGE_COMPRESS_QUALITY} 必须是 {IMAGE_COMPRESS_MIN_QUALITY}~{IMAGE_COMPRESS_MAX_QUALITY} 的整数"
            ));
        };
        patch.quality = Some(number);
    }
    if let Some(value) = object.get(KEY_IMAGE_COMPRESS_THRESHOLD_KB) {
        let Some(number) =
            int_in_range(value, IMAGE_COMPRESS_MIN_THRESHOLD_KB, IMAGE_COMPRESS_MAX_THRESHOLD_KB)
        else {
            return Err(format!(
                "{KEY_IMAGE_COMPRESS_THRESHOLD_KB} 必须是 {IMAGE_COMPRESS_MIN_THRESHOLD_KB}~{IMAGE_COMPRESS_MAX_THRESHOLD_KB} 的整数"
            ));
        };
        patch.threshold_kb = Some(number);
    }
    Ok(patch)
}

/// this 1 （）。** that **， after 
/// 「 from 512 1024 」。
fn change_note(patch: &ImageCompressPatch) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(enabled) = patch.enabled {
        parts.push(if enabled {
            "会话图片压缩已开启（转发前压缩内嵌 base64 大图）".to_string()
        } else {
            "会话图片压缩已关闭：图片原样转发（与接入前逐字节一致）".to_string()
        });
    }
    if let Some(max_edge) = patch.max_edge {
        parts.push(format!("长边上限设为 {max_edge}px"));
    }
    if let Some(quality) = patch.quality {
        parts.push(format!("重编码质量设为 {quality}"));
    }
    if let Some(threshold_kb) = patch.threshold_kb {
        parts.push(format!("触发阈值设为 {threshold_kb}KB"));
    }
    if parts.is_empty() {
        "会话图片压缩设置未变".to_string()
    } else {
        parts.join(" / ")
    }
}

/// response（GET and PUT use ； for before is ）
fn state_json(settings: ImageCompressSettings) -> Value {
    json!({
        KEY_IMAGE_COMPRESS_ENABLED: settings.enabled,
        KEY_IMAGE_COMPRESS_MAX_EDGE: settings.max_edge,
        KEY_IMAGE_COMPRESS_QUALITY: settings.quality,
        KEY_IMAGE_COMPRESS_THRESHOLD_KB: settings.threshold_kb,
    })
}
