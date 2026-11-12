//! will ： before for request base64 。
//!
//! ── ──────────────────────────────────────────────
//! client / `data:image/...;base64,` ， 1 
//! MB base64，request、 up token （ by 
//! token）。 for 1 will client**** 1 。
//! in ** 1 will before ** to limit and JPEG，
//! 「 1 → 1 」（ [`Cache`]）， is 
//! not 。 and OmniProxy `gatewayImageCompress.ts` for ，
//! defaultparameter（ 2048 / 80 / 512KB）。
//!
//! ── Chat ，Why ──────────────────────────────────
//! down （chat / responses / anthropic） in `api::protocol` already 
//! ** 1 Chat Completions **（`chat_from_responses` /
//! `chat_from_anthropic`）， to is 
//! `{"type":"image_url","image_url":{"url":…}}`（ for ） or 
//! `{"type":"image_url","image_url":"…"}`（，client）。
//! `input_image` 1 and （： 1 if path not will ）。http(s)
//! use 、gif / svg 1 not 。
//!
//! ── Why not this ────────────────────────────────────
//! `send_body` in `ctx.images.is_some()`（ / ） when before return， 
//! use in after —— not （ need ： can use in for ）。
//!
//! ── and CatPaw `image_compress.rs` is ──────────
//! that is ** 1 **： in 、 to CatPaw up 200KB limit（
//! 60KB、 1568、parameter）。 is ** use optional**：parameter can 、
//! all、 use 。 in 「 before 」， if ， is 
//! （ that to already is 、 already 60KB ）， not 。
//!
//! ── （release is panic=abort）─────────────────────────
//! ** unwrap / expect / panic / **： not 、base64 not 、
//! not 、failure、 —— 1 return。 can （ up 
//! can ）， not 1 。，none、none IO。
//!
//! ── and OmniProxy （ etc or ）───────────
//! 1. ****：OmniProxy alpha WebP； 1 **
//! JPEG**（ and CatPaw that 1 ， and not need WebP ）。
//! in / none， is will 。
//! 2. ****：png / jpeg no has ；WebP can can is ， `ANIM` 
//! （ not ）， and OmniProxy 1 。

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};

use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::DecodePaddingMode;
use base64::Engine as _;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, RgbImage};
use serde_json::Value;

use crate::server::config::ImageCompressSettings;

/// data URL base64 /：， to Node 
/// `Buffer.from(x, 'base64')`（ and ）—— 1 will 
/// ， will this 。 to use 。
const DATA_URL_BASE64: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true),
);

/// statistics（ use ；statistics**** ）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageCompressStats {
    /// 
    pub compressed: usize,
    /// before data URI 
    pub bytes_before: usize,
    /// after data URI 
    pub bytes_after: usize,
}

impl ImageCompressStats {
    pub fn changed(&self) -> bool {
        self.compressed > 0
    }
}

/// request**all** base64 （ `messages[].content[]`）。
///
/// failure or none need when ， not request。returnstatistics 。
pub fn compress_images_in_place(
    body: &mut Value,
    settings: &ImageCompressSettings,
) -> ImageCompressStats {
    let targets = collect_targets(body);
    if targets.is_empty() {
        return ImageCompressStats::default();
    }
    let mut stats = ImageCompressStats::default();
    let mut writes: Vec<(usize, usize, String)> = Vec::with_capacity(targets.len());
    for (message_index, part_index, uri) in targets {
        let next = compress_with_cache(&uri, settings);
        let Some(next) = next else { continue };
        if next == uri {
            continue;
        }
        stats.compressed += 1;
        stats.bytes_before += uri.len();
        stats.bytes_after += next.len();
        writes.push((message_index, part_index, next));
    }
    for (message_index, part_index, next) in writes {
        let pointer = format!("/messages/{message_index}/content/{part_index}");
        if let Some(part) = body.pointer_mut(&pointer) {
            set_image_uri(part, next);
        }
    }
    stats
}

/// request has no has can （ base64 jpeg/png/webp）。
///
/// 1 ， use in 「 1 no has 」 when ** 1 request** ——
/// request no has ， this for `into_owned`。
pub fn has_compressible(body: &Value) -> bool {
    let Some(messages) = body.get("messages").and_then(Value::as_array) else {
        return false;
    };
    messages.iter().any(|message| {
        message
            .get("content")
            .and_then(Value::as_array)
            .is_some_and(|parts| {
                parts
                    .iter()
                    .any(|part| image_uri_of(part).is_some_and(|uri| uri.starts_with("data:image/")))
            })
    })
}

/// all「 base64 」 and data URI（ use 1 ）。
fn collect_targets(body: &Value) -> Vec<(usize, usize, String)> {
    let mut out = Vec::new();
    let Some(messages) = body.get("messages").and_then(Value::as_array) else {
        return out;
    };
    for (message_index, message) in messages.iter().enumerate() {
        let Some(parts) = message.get("content").and_then(Value::as_array) else {
            continue;
        };
        for (part_index, part) in parts.iter().enumerate() {
            let Some(uri) = image_uri_of(part) else { continue };
            if uri.starts_with("data:image/") {
                out.push((message_index, part_index, uri.to_string()));
            }
        }
    }
    out
}

/// 1 URL（** Chat **：`image_url` / `input_image`，
/// can is can is `{url}` for ）。return None。
fn image_uri_of(part: &Value) -> Option<&str> {
    let kind = part.get("type").and_then(Value::as_str).unwrap_or("");
    if !kind.eq_ignore_ascii_case("image_url") && !kind.eq_ignore_ascii_case("input_image") {
        return None;
    }
    match part.get("image_url")? {
        Value::String(text) => Some(text.as_str()),
        object @ Value::Object(_) => object.get("url").and_then(Value::as_str),
        _ => None,
    }
}

/// ：， for `url` 。
/// its （ `detail`）。
fn set_image_uri(part: &mut Value, next: String) {
    let Some(field) = part.get_mut("image_url") else {
        return;
    };
    match field {
        Value::String(slot) => *slot = next,
        Value::Object(map) => {
            map.insert("url".to_string(), Value::String(next));
        }
        _ => {}
    }
}

/// ：， in （）。
fn compress_with_cache(uri: &str, settings: &ImageCompressSettings) -> Option<String> {
    // （http use 、gif / svg etc ）， not 
    if !uri.starts_with("data:image/") {
        return None;
    }
    let key = cache_key(uri, settings);
    if let Some(hit) = Cache::lookup(&key) {
        return hit;
    }
    let next = compress_data_uri(uri, settings).filter(|next| next.as_str() != uri);
    // 「 not need need / 」（null）， 1 
    Cache::store(key, next.clone());
    next
}

/// data URI；`None` = （ not / / / after 
/// / failure）。
fn compress_data_uri(uri: &str, settings: &ImageCompressSettings) -> Option<String> {
    let base64_body = split_data_uri(uri.trim())?;
    // in ** before **、 by base64 after （ and OmniProxy
    // ）， as 1 1 。
    let estimate = decoded_len_estimate(base64_body);
    let threshold = settings.threshold_kb.max(0) as usize * 1024;
    if estimate < threshold {
        return None;
    }
    // ：（ base64 \r\n， when ignore）
    let compact: String = base64_body
        .chars()
        .filter(|ch| *ch != '\r' && *ch != '\n')
        .collect();
    let input = DATA_URL_BASE64.decode(compact.as_bytes()).ok()?;
    if input.is_empty() {
        return None;
    }
    // （WebP ANIM ） not ： will 
    if has_anim_chunk(&input) {
        return None;
    }
    let decoded = image::load_from_memory(&input).ok()?;
    let encoded = encode_jpeg(&decoded, settings)?;
    // （ `output.length >= input.length` ）
    if encoded.len() >= input.len() {
        return None;
    }
    Some(format!(
        "data:image/jpeg;base64,{}",
        DATA_URL_BASE64.encode(&encoded)
    ))
}

/// `data:image/<subtype>;base64,<body>` base64 。
///
/// jpeg / png / webp （ not ）， its （gif / svg / avif…）
/// 1 not —— and OmniProxy `DATA_URI_RE` 1 。
fn split_data_uri(uri: &str) -> Option<&str> {
    let rest = uri.strip_prefix("data:image/")
        .or_else(|| strip_prefix_ignore_ascii_case(uri, "data:image/"))?;
    let separator = rest.find(';')?;
    let subtype = &rest[..separator];
    let subtype_ok = ["jpeg", "jpg", "png", "webp"]
        .iter()
        .any(|known| subtype.eq_ignore_ascii_case(known));
    if !subtype_ok {
        return None;
    }
    let body = strip_prefix_ignore_ascii_case(&rest[separator..], ";base64,")?;
    if body.is_empty() {
        return None;
    }
    Some(body)
}

/// not before （return before after remaining）。
fn strip_prefix_ignore_ascii_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix).then(|| &text[prefix.len()..])
}

/// base64 after （，used for，）。
fn decoded_len_estimate(base64: &str) -> usize {
    let trimmed = base64.trim_end_matches(['\r', '\n']);
    let padding = if trimmed.ends_with("==") {
        2
    } else if trimmed.ends_with('=') {
        1
    } else {
        0
    };
    (trimmed.len() * 3 / 4).saturating_sub(padding)
}

/// is not is ： in WebP can can `ANIM` 。 to （ to —— 
/// will 「 1 」， not will ）。
fn has_anim_chunk(bytes: &[u8]) -> bool {
    if bytes.len() < 4 || &bytes[0..4] != b"RIFF" {
        return false;
    }
    bytes.windows(4).any(|window| window == b"ANIM")
}

/// → etc to limit（ not ）→ → JPEG 。
fn encode_jpeg(image: &DynamicImage, settings: &ImageCompressSettings) -> Option<Vec<u8>> {
    let max_edge = settings.max_edge.clamp(256, 8192) as u32;
    let quality = settings.quality.clamp(1, 100) as u8;
    let long_edge = image.width().max(image.height());
    let shrunk;
    // not need need when use ， 1 （ up is MB ）
    let source: &DynamicImage = if long_edge > max_edge {
        shrunk = image.resize(max_edge, max_edge, FilterType::Lanczos3);
        &shrunk
    } else {
        image
    };
    let rgb = flatten_to_rgb8(source)?;
    let (width, height) = rgb.dimensions();
    if width == 0 || height == 0 {
        // JPEG will use width-1 ，0 will to 
        return None;
    }
    let mut output = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(Cursor::new(&mut output), quality);
    encoder.encode_image(&rgb).ok()?;
    Some(output)
}

/// not alpha 8 RGB： has alpha by （over ），
/// no has RGB8。 etc sharp `.flatten({background:'#ffffff'})`。
fn flatten_to_rgb8(image: &DynamicImage) -> Option<RgbImage> {
    if !image.color().has_alpha() {
        return Some(image.to_rgb8());
    }
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(3);
    let mut data = Vec::new();
    if data.try_reserve_exact(pixels).is_err() {
        return None;
    }
    for pixel in rgba.pixels() {
        let channels = pixel.0;
        let alpha = u32::from(channels[3]);
        let opaque = 255 - alpha;
        for channel in &channels[..3] {
            let blended = (u32::from(*channel) * alpha + 255 * opaque + 127) / 255;
            data.push(blended as u8);
        }
    }
    RgbImage::from_raw(width, height, data)
}

// ─── （「 use 」）────────────────────────────────────

/// = data URI sha256 + parameter。
///
/// parameter and → set，none need （ and OmniProxy 1 ）。
/// use **** not is DefaultHasher： will A to B ，
/// that is 「 1 」 this ， 1 。
fn cache_key(uri: &str, settings: &ImageCompressSettings) -> String {
    format!(
        "{}|{}|{}|{}",
        sha256_hex(uri.as_bytes()),
        settings.max_edge,
        settings.quality,
        settings.threshold_kb
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// for client will ** 1 **， new + + JPEG
/// is （ 1 MB ms）。 this 「 +
/// parameter」 as ，「none need / failure」 `None` 
/// （ / not use ）。 and totallimit，
/// when ** use ** 1 。
struct Cache {
    entries: HashMap<String, CacheEntry>,
    bytes: usize,
    seq: u64,
}

struct CacheEntry {
    payload: Option<String>,
    bytes: usize,
    used: u64,
}

/// limit： and OmniProxy 1 （500）。** not ** —— use 
/// is totallimit，「」 this ， 500 already 
/// use ， 1 will 1 can 。
const CACHE_MAX_ENTRIES: usize = 500;
/// totallimit default：128 MiB。， KB，
/// will that 。
const DEFAULT_CACHE_MAX_MB: usize = 128;
const MIN_CACHE_MAX_MB: usize = 8;
const MAX_CACHE_MAX_MB: usize = 4096;
const ENV_CACHE_MAX_MB: &str = "AGENT2API_IMAGE_COMPRESS_CACHE_MB";

/// totallimit（Issue #420）：`AGENT2API_IMAGE_COMPRESS_CACHE_MB`
/// （MiB，default 128， 8~4096）。
///
/// and `api::image_assets::max_cache_bytes` （ / default、
/// not 、）—— is 1 ， need 1 ，
/// limit for 1 to 。
fn cache_max_bytes() -> usize {
    let mb = std::env::var(ENV_CACHE_MAX_MB)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|mb| (MIN_CACHE_MAX_MB..=MAX_CACHE_MAX_MB).contains(mb))
        .unwrap_or(DEFAULT_CACHE_MAX_MB);
    mb * 1024 * 1024
}

static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| {
        Mutex::new(Cache {
            entries: HashMap::new(),
            bytes: 0,
            seq: 0,
        })
    })
}

impl Cache {
    /// in return `Some( or None)`； in return `None`。
    ///
    /// Note「 in and as None（none need ）」 and 「 in 」 in return up 
    /// `Option<Option<String>>` ：`Some(None)` = in and none need ，
    /// `None` = in 。 use （[`compress_with_cache`]）「 has no has to 
    /// can use 」， by `None` ， in not 1 。
    fn lookup(key: &str) -> Option<Option<String>> {
        let mut guard = cache().lock().ok()?;
        let seq = guard.seq.wrapping_add(1);
        // ， entries —— otherwise `entry` can use and `guard.seq`
        // in 1 use will up use check
        guard.seq = seq;
        let entry = guard.entries.get_mut(key)?;
        entry.used = seq;
        Some(entry.payload.clone())
    }

    fn store(key: String, payload: Option<String>) {
        let Ok(mut guard) = cache().lock() else {
            return; // 锁中毒（此前有人 panic 持锁）：放弃缓存，不影响本次压缩
        };
        let bytes = key.len() + payload.as_ref().map(String::len).unwrap_or(0);
        if let Some(old) = guard.entries.remove(&key) {
            guard.bytes = guard.bytes.saturating_sub(old.bytes);
        }
        let seq = guard.seq.wrapping_add(1);
        guard.seq = seq;
        guard.entries.insert(
            key,
            CacheEntry {
                payload,
                bytes,
                used: seq,
            },
        );
        guard.bytes = guard.bytes.saturating_add(bytes);
        guard.evict();
    }

    fn evict(&mut self) {
        // limit 1 ， in （ 1 use ）。
        let max_bytes = cache_max_bytes();
        while self.entries.len() > CACHE_MAX_ENTRIES
            || (self.bytes > max_bytes && self.entries.len() > 1)
        {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(entry) = self.entries.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(entry.bytes);
            }
        }
    }
}
