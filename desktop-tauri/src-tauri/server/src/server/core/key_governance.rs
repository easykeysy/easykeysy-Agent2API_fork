//! 网关 Key 的配额 / 过期 / 速率限制与用量统计（面向重度免费额度采集与 API 消费者）。
//!
//! ── 为什么单独一个模块 ──────────────────────────────────────
//! `api_keys` 负责 Key 的存取与白名单判定；本模块负责「这把 Key 还能不能再用、
//! 还剩多少」这类**运行期治理**事实：
//!   - 配额/过期在鉴权通过后、请求进入上游前判定（拒绝要早，避免白耗上游）；
//!   - 用量与 RPM 窗口是高频写入的运行态，不适合塞进配置快照的读改写链路。
//!
//! ── 存储取舍（与 fork 的“重内容离 DB”策略一致）──────────────
//! 用量计数落在 `paths::cache_dir()/key-governance.json`（小文件、原子写）：
//!   - 它是派生/计费窗口状态，不是账号或设置真相源；
//!   - 在 Linux ramdisk 外置模式下 cache_dir 位于易失侧目录，重启会清零——
//!     这是可接受的降级（保护免费池的上限/速率在运行期仍生效，跨重启额度
//!     统计从零开始），避免把高频计数写进主库。
//! RPM 使用进程内滑动窗口（不持久化，重启即清），语义上本来就是瞬时限流。
//!
//! ── 约束 ────────────────────────────────────────────────────
//! 持本模块状态锁期间不做日志/IO 之外的重活；文件写失败只记日志、不放大为
//! 请求失败（治理判定以内存态为准，落盘是尽力而为）。

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::server::core::api_keys::ApiKeyEntry;
use crate::server::logging;
use crate::paths;

/// 一天的毫秒数（UTC 日界，用于日配额窗口）
const DAY_MS: i64 = 24 * 60 * 60 * 1000;
/// RPM 滑动窗口
const RPM_WINDOW_MS: i64 = 60 * 1000;
/// 落盘节流：至少间隔 2 秒才把内存态刷到磁盘（高频请求下避免每次都写文件）
const FLUSH_INTERVAL_MS: i64 = 2_000;

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn utc_day_key(ms: i64) -> i64 {
    if ms <= 0 { 0 } else { ms / DAY_MS }
}

/// 某把 Key 的持久化用量计数
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct KeyUsage {
    /// 当前 UTC 日键（ms / DAY_MS）
    day: i64,
    daily_requests: u64,
    daily_tokens: u64,
    total_requests: u64,
    total_tokens: u64,
}

/// 文件整体形态
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct UsageFile {
    keys: HashMap<String, KeyUsage>,
}

#[derive(Default)]
struct RpmState {
    /// 每个 key id 的最近请求时间戳（毫秒，升序窗口尾部追加）
    hits: HashMap<String, VecDeque<i64>>,
}

struct State {
    usage: UsageFile,
    rpm: RpmState,
    dirty: bool,
    last_flush_ms: i64,
    path: PathBuf,
}

impl State {
    fn load() -> Self {
        let path = paths::cache_dir().join("key-governance.json");
        let usage = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<UsageFile>(&text).ok())
            .unwrap_or_default();
        Self { usage, rpm: RpmState::default(), dirty: false, last_flush_ms: 0, path }
    }

    fn flush_if_due(&mut self, now: i64) {
        if !self.dirty {
            return;
        }
        if now.saturating_sub(self.last_flush_ms) < FLUSH_INTERVAL_MS {
            return;
        }
        self.flush_now();
    }

    fn flush_now(&mut self) {
        if !self.dirty {
            return;
        }
        let Some(parent) = self.path.parent() else { return; };
        if let Err(error) = fs::create_dir_all(parent) {
            logging::log("[Keys]", &format!("⚠️  创建 key-governance 目录失败: {error}"));
            return;
        }
        let tmp = self.path.with_extension("json.tmp");
        let payload = match serde_json::to_vec(&self.usage) {
            Ok(bytes) => bytes,
            Err(error) => {
                logging::log("[Keys]", &format!("⚠️  序列化 key-governance 失败: {error}"));
                return;
            }
        };
        let result = (|| {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(&payload)?;
            file.sync_all()?;
            fs::rename(&tmp, &self.path)?;
            Ok::<(), std::io::Error>(())
        })();
        match result {
            Ok(()) => {
                self.dirty = false;
                self.last_flush_ms = now_ms();
            }
            Err(error) => {
                logging::log("[Keys]", &format!("⚠️  写入 key-governance 失败: {error}"));
                let _ = fs::remove_file(&tmp);
            }
        }
    }

    fn usage_mut(&mut self, key_id: &str, now: i64) -> &mut KeyUsage {
        let today = utc_day_key(now);
        let entry = self.usage.keys.entry(key_id.to_string()).or_default();
        if entry.day != today {
            entry.day = today;
            entry.daily_requests = 0;
            entry.daily_tokens = 0;
        }
        entry
    }
}

fn state() -> &'static Mutex<State> {
    static STATE: OnceLock<Mutex<State>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(State::load()))
}

/// 治理拒绝原因（供调用方映射为 HTTP 错误）
#[derive(Clone, Debug)]
pub enum DenyReason {
    Expired { expires_at: i64 },
    DailyRequestLimit { limit: u64, used: u64, reset_at_ms: i64 },
    TotalRequestLimit { limit: u64, used: u64 },
    DailyTokenLimit { limit: u64, used: u64, reset_at_ms: i64 },
    TotalTokenLimit { limit: u64, used: u64 },
    RpmLimit { limit: u64, retry_after_secs: u64 },
}

/// 对一次模型请求做治理检查并（通过时）记入一次请求与 RPM 命中。
/// `count_request` 为 false 时仅做只读/探针类检查不计入请求配额（例如 `/v1/models`）。
pub fn admit_request(entry: &ApiKeyEntry, count_request: bool) -> Result<(), DenyReason> {
    let now = now_ms();
    // 过期（配置项，优先于计数）
    if let Some(exp) = entry.expires_at {
        if exp > 0 && now >= exp {
            return Err(DenyReason::Expired { expires_at: exp });
        }
    }

    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());

    // RPM（进程内滑动窗口）
    if let Some(limit) = positive_u64(entry.rpm_limit) {
        let deque = guard.rpm.hits.entry(entry.id.clone()).or_default();
        let cutoff = now.saturating_sub(RPM_WINDOW_MS);
        while matches!(deque.front(), Some(&ts) if ts <= cutoff) {
            deque.pop_front();
        }
        if deque.len() as u64 >= limit {
            let oldest = deque.front().copied().unwrap_or(now);
            let retry_after_ms = (oldest + RPM_WINDOW_MS).saturating_sub(now).max(1);
            let retry_after_secs = ((retry_after_ms + 999) / 1000) as u64;
            return Err(DenyReason::RpmLimit { limit, retry_after_secs });
        }
        if count_request {
            deque.push_back(now);
        }
    }

    // 配额检查（基于当前用量）
    let (daily_req, daily_tok, total_req, total_tok, next_reset) = {
        let u = guard.usage_mut(&entry.id, now);
        let reset = (u.day + 1) * DAY_MS;
        (u.daily_requests, u.daily_tokens, u.total_requests, u.total_tokens, reset)
    };
    if count_request {
        if let Some(limit) = positive_u64(entry.daily_request_limit) {
            if daily_req >= limit {
                return Err(DenyReason::DailyRequestLimit { limit, used: daily_req, reset_at_ms: next_reset });
            }
        }
        if let Some(limit) = positive_u64(entry.total_request_limit) {
            if total_req >= limit {
                return Err(DenyReason::TotalRequestLimit { limit, used: total_req });
            }
        }
        if let Some(limit) = positive_u64(entry.daily_token_limit) {
            if daily_tok >= limit {
                return Err(DenyReason::DailyTokenLimit { limit, used: daily_tok, reset_at_ms: next_reset });
            }
        }
        if let Some(limit) = positive_u64(entry.total_token_limit) {
            if total_tok >= limit {
                return Err(DenyReason::TotalTokenLimit { limit, used: total_tok });
            }
        }
        // 通过：记入一次请求
        let u = guard.usage_mut(&entry.id, now);
        u.daily_requests = u.daily_requests.saturating_add(1);
        u.total_requests = u.total_requests.saturating_add(1);
        guard.dirty = true;
        guard.flush_if_due(now);
    }
    Ok(())
}

/// 记录一次请求消耗的 token（请求完成后尽力回填；未知/0 则忽略）。
pub fn add_tokens(key_id: &str, tokens: u64) {
    if tokens == 0 || key_id.is_empty() {
        return;
    }
    let now = now_ms();
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    let u = guard.usage_mut(key_id, now);
    u.daily_tokens = u.daily_tokens.saturating_add(tokens);
    u.total_tokens = u.total_tokens.saturating_add(tokens);
    guard.dirty = true;
    guard.flush_if_due(now);
}

/// 查询某把 Key 当前用量快照（用于管理端/自助端展示）
pub fn usage_snapshot(key_id: &str) -> Value {
    let now = now_ms();
    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());
    let today = utc_day_key(now);
    let (daily_req, daily_tok, total_req, total_tok, reset_at) = {
        let entry = guard.usage.keys.entry(key_id.to_string()).or_default();
        if entry.day != today {
            entry.day = today;
            entry.daily_requests = 0;
            entry.daily_tokens = 0;
        }
        (
            entry.daily_requests,
            entry.daily_tokens,
            entry.total_requests,
            entry.total_tokens,
            (entry.day + 1) * DAY_MS,
        )
    };
    json!({
        "dailyRequests": daily_req,
        "dailyTokens": daily_tok,
        "totalRequests": total_req,
        "totalTokens": total_tok,
        "dailyResetAt": reset_at,
    })
}

/// 面向单把 Key 的完整治理视图（配置 + 用量 + 剩余），供 API 返回。
pub fn status_json(entry: &ApiKeyEntry) -> Value {
    let usage = usage_snapshot(&entry.id);
    let daily_requests = usage.get("dailyRequests").and_then(Value::as_u64).unwrap_or(0);
    let daily_tokens = usage.get("dailyTokens").and_then(Value::as_u64).unwrap_or(0);
    let total_requests = usage.get("totalRequests").and_then(Value::as_u64).unwrap_or(0);
    let total_tokens = usage.get("totalTokens").and_then(Value::as_u64).unwrap_or(0);

    let remaining = |limit: Option<u64>, used: u64| -> Value {
        match positive_u64(limit) {
            Some(lim) => json!(lim.saturating_sub(used)),
            None => Value::Null,
        }
    };

    json!({
        "limits": {
            "expiresAt": entry.expires_at,
            "dailyRequestLimit": entry.daily_request_limit,
            "totalRequestLimit": entry.total_request_limit,
            "dailyTokenLimit": entry.daily_token_limit,
            "totalTokenLimit": entry.total_token_limit,
            "rpmLimit": entry.rpm_limit,
        },
        "usage": usage,
        "remaining": {
            "dailyRequests": remaining(entry.daily_request_limit, daily_requests),
            "totalRequests": remaining(entry.total_request_limit, total_requests),
            "dailyTokens": remaining(entry.daily_token_limit, daily_tokens),
            "totalTokens": remaining(entry.total_token_limit, total_tokens),
        }
    })
}

/// 将治理拒绝映射为 (status, code, message, 可选 retry_after_secs)。
pub fn deny_to_http(reason: &DenyReason) -> (u16, &'static str, String, Option<u64>) {
    match reason {
        DenyReason::Expired { expires_at } => (
            403,
            "key_expired",
            format!("API Key 已过期（expiresAt={expires_at}）"),
            None,
        ),
        DenyReason::DailyRequestLimit { limit, used, reset_at_ms } => (
            429,
            "key_quota_exceeded",
            format!("已达到该 Key 的每日请求上限（{used}/{limit}），将在 {reset_at_ms} (ms) 重置"),
            Some(((reset_at_ms.saturating_sub(now_ms()) + 999) / 1000) as u64),
        ),
        DenyReason::TotalRequestLimit { limit, used } => (
            429,
            "key_quota_exceeded",
            format!("已达到该 Key 的总请求上限（{used}/{limit}）"),
            None,
        ),
        DenyReason::DailyTokenLimit { limit, used, reset_at_ms } => (
            429,
            "key_quota_exceeded",
            format!("已达到该 Key 的每日 Token 上限（{used}/{limit}），将在 {reset_at_ms} (ms) 重置"),
            Some(((reset_at_ms.saturating_sub(now_ms()) + 999) / 1000) as u64),
        ),
        DenyReason::TotalTokenLimit { limit, used } => (
            429,
            "key_quota_exceeded",
            format!("已达到该 Key 的总 Token 上限（{used}/{limit}）"),
            None,
        ),
        DenyReason::RpmLimit { limit, retry_after_secs } => (
            429,
            "key_rate_limited",
            format!("超出该 Key 的速率限制（{limit} RPM）"),
            Some(*retry_after_secs),
        ),
    }
}

fn positive_u64(v: Option<u64>) -> Option<u64> {
    v.filter(|n| *n > 0)
}

/// 解析请求中的可选限制字段（JSON number 或缺失/null → None = 不限制）。
pub fn parse_limit(value: Option<&Value>) -> Option<u64> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_u64().filter(|v| *v > 0),
        Some(Value::String(s)) => s.trim().parse::<u64>().ok().filter(|v| *v > 0),
        _ => None,
    }
}

/// 解析 expiresAt（毫秒时间戳，>0 有效；缺失/null/0 → None）
pub fn parse_expires_at(value: Option<&Value>) -> Option<i64> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_i64().filter(|v| *v > 0),
        Some(Value::String(s)) => s.trim().parse::<i64>().ok().filter(|v| *v > 0),
        _ => None,
    }
}
