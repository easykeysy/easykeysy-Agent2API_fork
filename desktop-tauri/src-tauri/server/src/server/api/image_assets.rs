//! down ；， can ， not or Key。
//!
//! ── limit can + use （Issue #420）───────────────
//! this is 「、limit」 1 ：128 MiB、1 when 、
//! 。（512M~1G VPS / NAS） need need can ，：
//! · limit as `AGENT2API_IMAGE_CACHE_MB`（default is 128）；
//! · not ****， as ** use ** —— limit
//! failure will 1 success 5xx， and not 。
//! has 「 this 1 need limit」 when old ： that not down 。
//!
//! and `core/upstream/image_compress.rs` 1 （ + 
//! use ，）， not 1 。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use axum::extract::Path;
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;

use crate::server::errors::GatewayError;
use crate::server::logging;

/// when （1 when ）：，`read` / `store` will 。
const TTL_MS: i64 = 60 * 60 * 1000;

/// limit default（MiB）。 before 128 not —— no 
/// （） as not 。
const DEFAULT_CACHE_MB: usize = 128;
/// limit down ： not down 1 ， etc can ，
/// not not 、default。
const MIN_CACHE_MB: usize = 8;
/// limit up ：4 GiB。 up can can is ，default。
const MAX_CACHE_MB: usize = 4096;
/// limit 。
const ENV_CACHE_MB: &str = "AGENT2API_IMAGE_CACHE_MB";

struct Asset {
    bytes: Bytes,
    mime: &'static str,
    expires_at: i64,
    /// 1 / （ new ），LRU use 。
    used: u64,
}

struct Cache {
    entries: HashMap<String, Asset>,
    /// current use ，。
    bytes: usize,
    /// ， to 「 use 」 when 。
    seq: u64,
}

fn cache() -> &'static Mutex<Cache> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(Cache {
            entries: HashMap::new(),
            bytes: 0,
            seq: 0,
        })
    })
}

impl Cache {
    /// expired and use （ 1 ）。
    fn retain_live(&mut self) {
        let now = logging::now_ms();
        self.entries.retain(|_, asset| asset.expires_at > now);
        self.bytes = self.entries.values().map(|asset| asset.bytes.len()).sum();
    }

    /// down 1 （「 use 」 when ）。
    fn tick(&mut self) -> u64 {
        self.seq = self.seq.wrapping_add(1);
        self.seq
    }

    /// use ， to use to limit。
    ///
    /// `len() > 1` this ： 1 when not ，otherwise 1 limit will 
    /// empty「 not 」—— that by `store` before down ， not this 
    /// in this empty。
    fn evict_to_fit(&mut self, cap: usize) {
        while self.bytes > cap && self.entries.len() > 1 {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, asset)| asset.used)
                .map(|(id, _)| id.clone())
            else {
                break;
            };
            if let Some(asset) = self.entries.remove(&oldest) {
                self.bytes = self.bytes.saturating_sub(asset.bytes.len());
            }
        }
    }
}

/// limit（）：`AGENT2API_IMAGE_CACHE_MB`（MiB，default 128， 8~4096）。
///
/// and `config::parse` 1 ：** / 1 default， not ** ——
/// 99999 to 4096 will as limit。empty（compose 
/// set ） no 。 use （ and `logging::log_file_path`
/// 1 ）， can ignore —— not in path up 。
fn max_cache_bytes() -> usize {
    let mb = std::env::var(ENV_CACHE_MB)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|mb| (MIN_CACHE_MB..=MAX_CACHE_MB).contains(mb))
        .unwrap_or(DEFAULT_CACHE_MB);
    mb * 1024 * 1024
}

pub(super) fn store(images: Vec<(Vec<u8>, &'static str)>) -> Result<Vec<String>, GatewayError> {
    let cap = max_cache_bytes();
    // can before failure failure：** this 1 **limit when ， old not down 。
    // must in before down —— otherwise after ， will 1 ，
    // use to down 404，。
    let total: usize = images.iter().map(|(bytes, _)| bytes.len()).sum();
    if total > cap {
        return Err(GatewayError::with_status(
            503,
            format!(
                "生成图片合计约 {:.1} MiB 超过下载缓存上限（{} MiB），请改用 response_format=b64_json",
                total as f64 / (1024.0 * 1024.0),
                cap / (1024 * 1024)
            ),
        ));
    }

    let expires_at = logging::now_ms() + TTL_MS;
    let mut tokens = Vec::with_capacity(images.len());
    let mut prepared = Vec::with_capacity(images.len());
    for (bytes, mime) in images {
        let mut random = [0u8; 32];
        getrandom::getrandom(&mut random)
            .map_err(|_| GatewayError::with_status(500, "无法生成安全的图片下载令牌"))?;
        let id: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        tokens.push(id.clone());
        prepared.push((id, bytes, mime));
    }

    let mut guard = cache()
        .lock()
        .map_err(|_| GatewayError::with_status(503, "图片下载缓存暂不可用"))?;
    guard.retain_live();
    for (id, bytes, mime) in prepared {
        let used = guard.tick();
        let len = bytes.len();
        guard.entries.insert(
            id,
            Asset {
                bytes: Bytes::from(bytes),
                mime,
                expires_at,
                used,
            },
        );
        // tick， has after ， when after that 。
        guard.bytes = guard.bytes.saturating_add(len);
    }
    guard.evict_to_fit(cap);
    Ok(tokens)
}

pub(super) fn read(id: &str) -> Option<(Bytes, &'static str)> {
    let mut guard = cache().lock().ok()?;
    guard.retain_live();
    let used = guard.tick();
    let asset = guard.entries.get_mut(id)?;
    asset.used = used;
    Some((asset.bytes.clone(), asset.mime))
}

pub async fn download(Path(id): Path<String>) -> Response {
    let Some((bytes, mime)) = read(&id) else {
        return GatewayError::with_status(404, "图片下载链接不存在或已过期").payload_response();
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    (StatusCode::OK, headers, bytes).into_response()
}
