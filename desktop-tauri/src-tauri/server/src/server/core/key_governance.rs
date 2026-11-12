//! Per-gateway-key quotas, expiry, rate limits, and usage tracking (for heavy API consumers and free-tier collectors).
//!
//! ── Why a separate module ───────────────────────────────────
//! `api_keys` handles key storage and allowlist checks; this module owns runtime governance:
//! whether a key can still be used and how much budget remains.
//!   - Quota/expiry are enforced after auth and before upstream forwarding (fail early to avoid wasting upstream).
//!   - Usage counters and RPM windows are hot runtime state, not config snapshot read-modify-write data.
//!
//! ── Storage tradeoff (aligned with "heavy content out of DB") ─
//! Usage counters live in `paths::cache_dir()/key-governance.json` (small file, atomic writes):
//!   - Derived quota-window state, not the source of truth for accounts or settings.
//!   - In Linux ramdisk-external mode cache_dir is volatile; counters reset on reboot —
//!     an acceptable degradation (limits/rate protection still apply at runtime,
//!     cross-reboot accounting restarts from zero) to avoid high-frequency DB writes.
//! RPM uses an in-process sliding window (not persisted) — instantaneous rate limiting by nature.
//!
//! ── Constraints ─────────────────────────────────────────────
//! Keep work under the state lock minimal (no heavy I/O except flush). Flush failures are logged
//! but do not fail requests (in-memory state is authoritative; persistence is best-effort).

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

/// Milliseconds per UTC day (daily quota window).
const DAY_MS: i64 = 24 * 60 * 60 * 1000;
/// RPM sliding window.
const RPM_WINDOW_MS: i64 = 60 * 1000;
/// Flush throttle: at most once every 2s under load.
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

/// Persisted usage counters for one key.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct KeyUsage {
    /// Current UTC day bucket (ms / DAY_MS).
    day: i64,
    daily_requests: u64,
    daily_tokens: u64,
    total_requests: u64,
    total_tokens: u64,
}

/// On-disk file shape.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct UsageFile {
    keys: HashMap<String, KeyUsage>,
}

#[derive(Default)]
struct RpmState {
    /// Recent request timestamps per key id (ms, ascending).
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
            logging::log("[Keys]", &format!("⚠️  Failed to create key-governance directory: {error}"));
            return;
        }
        let tmp = self.path.with_extension("json.tmp");
        let payload = match serde_json::to_vec(&self.usage) {
            Ok(bytes) => bytes,
            Err(error) => {
                logging::log("[Keys]", &format!("⚠️  Failed to serialize key-governance state: {error}"));
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
                logging::log("[Keys]", &format!("⚠️  Failed to write key-governance state: {error}"));
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

/// Governance denial reason (mapped to HTTP errors by callers).
#[derive(Clone, Debug)]
pub enum DenyReason {
    Expired { expires_at: i64 },
    DailyRequestLimit { limit: u64, used: u64, reset_at_ms: i64 },
    TotalRequestLimit { limit: u64, used: u64 },
    DailyTokenLimit { limit: u64, used: u64, reset_at_ms: i64 },
    TotalTokenLimit { limit: u64, used: u64 },
    RpmLimit { limit: u64, retry_after_secs: u64 },
}

/// Check governance for a request and, on success, record one request/RPM hit.
/// When `count_request` is false, perform checks without consuming request quota (e.g. read-only probes).
pub fn admit_request(entry: &ApiKeyEntry, count_request: bool) -> Result<(), DenyReason> {
    let now = now_ms();
    // Expiry (config check, before counters).
    if let Some(exp) = entry.expires_at {
        if exp > 0 && now >= exp {
            return Err(DenyReason::Expired { expires_at: exp });
        }
    }

    let mut guard = state().lock().unwrap_or_else(|e| e.into_inner());

    // RPM (in-process sliding window).
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

    // Quota checks (against current usage).
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
        // Admitted: record one request.
        let u = guard.usage_mut(&entry.id, now);
        u.daily_requests = u.daily_requests.saturating_add(1);
        u.total_requests = u.total_requests.saturating_add(1);
        guard.dirty = true;
        guard.flush_if_due(now);
    }
    Ok(())
}

/// Record tokens consumed by a request (best-effort post-response; 0/unknown ignored).
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

/// Current usage snapshot for a key (admin and self-service views).
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

/// Full governance view for a key (limits + usage + remaining) for API responses.
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

/// Map a governance denial to (status, code, message, optional retry_after_secs).
pub fn deny_to_http(reason: &DenyReason) -> (u16, &'static str, String, Option<u64>) {
    match reason {
        DenyReason::Expired { expires_at } => (
            403,
            "key_expired",
            format!("API key expired (expiresAt={expires_at})"),
            None,
        ),
        DenyReason::DailyRequestLimit { limit, used, reset_at_ms } => (
            429,
            "key_quota_exceeded",
            format!("Daily request quota exceeded for this key ({used}/{limit}); resets at {reset_at_ms} ms"),
            Some(((reset_at_ms.saturating_sub(now_ms()) + 999) / 1000) as u64),
        ),
        DenyReason::TotalRequestLimit { limit, used } => (
            429,
            "key_quota_exceeded",
            format!("Total request quota exceeded for this key ({used}/{limit})"),
            None,
        ),
        DenyReason::DailyTokenLimit { limit, used, reset_at_ms } => (
            429,
            "key_quota_exceeded",
            format!("Daily token quota exceeded for this key ({used}/{limit}); resets at {reset_at_ms} ms"),
            Some(((reset_at_ms.saturating_sub(now_ms()) + 999) / 1000) as u64),
        ),
        DenyReason::TotalTokenLimit { limit, used } => (
            429,
            "key_quota_exceeded",
            format!("Total token quota exceeded for this key ({used}/{limit})"),
            None,
        ),
        DenyReason::RpmLimit { limit, retry_after_secs } => (
            429,
            "key_rate_limited",
            format!("Rate limit exceeded for this key ({limit} RPM)"),
            Some(*retry_after_secs),
        ),
    }
}

fn positive_u64(v: Option<u64>) -> Option<u64> {
    v.filter(|n| *n > 0)
}

/// Parse an optional numeric limit field (number or missing/null => None = unlimited).
pub fn parse_limit(value: Option<&Value>) -> Option<u64> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_u64().filter(|v| *v > 0),
        Some(Value::String(s)) => s.trim().parse::<u64>().ok().filter(|v| *v > 0),
        _ => None,
    }
}

/// Parse expiresAt (millisecond timestamp, >0 valid; missing/null/0 => None).
pub fn parse_expires_at(value: Option<&Value>) -> Option<i64> {
    match value {
        None | Some(Value::Null) => None,
        Some(Value::Number(n)) => n.as_i64().filter(|v| *v > 0),
        Some(Value::String(s)) => s.trim().parse::<i64>().ok().filter(|v| *v > 0),
        _ => None,
    }
}
