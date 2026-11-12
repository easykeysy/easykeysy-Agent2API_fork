//! by ****： / will / failure / / 。
//!
//! for 「 by 」 set in ** when ** need that 。 is 
//! ****（ not 、 not ）： this is 「 down this 1 will request
//! 」， is （ and magpie Sink / resting ）；
//! need is `rateLimits`（， already has ）。
//!
//! ── and ────────────────────────────────────────
//! `routing.rs` ****（ + + Reason，）；
//! is that need / need ****。 is ：
//! - when ： from 1 （ Map），；
//! - request has when ： mark_* update。
//! 「」 and 「」 after ，queue （ not ） and path（ + ）
//! to 1 —— is 1 。
//!
//! ── Why is 「 + use 」 ────────────────
//! not will （ that is client ）， not to magpie that session id。
//! 「 1 for 」 in down request **** is ： not 、 use 
//! not （ after is in ）。 to will ， DefaultHasher
//! can （ not 、 not need need ，empty is ）。
//!
//! ── and （ when not and ）─────────────────────
//! - （backoff）：**failure** after （15 minutes，failure，
//! limit 1 when ）。 is 「 this failure 1 will 」，、；
//! - （breaker）：**failure to **， and need success if 。
//! is 「 this in not for ， 1 」。
//! as 、（），：，
//! （ is —— that when by 「 1 」，
//! and and 1 ： can 1 ， not request）。

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use axum::http::HeaderMap;

/// failure when （15 minutes） and limit（1 when ）。
pub(super) const BACKOFF_BASE_MS: i64 = 15 * 60 * 1000;
const BACKOFF_MAX_MS: i64 = 60 * 60 * 1000;

/// when （30 minutes）。
///
/// ── Why need need （ up no has ，2026-10-13 ）─────────────────
/// up `sink` is 「 when 」** from not no **（`snapshot` 
/// ）， 1 path is `sink_clear` —— in ** this success 1 **
/// after use 。 is ：429 → → to → not to → no will 
/// success → not 。： 11 in 1 429 after 
/// ，「 by 」 is to after ，
/// not to （ `projects/agent2api/deploy-to-03` and that ）。
///
/// ── Why 30 minutes ────────────────────────────────────────
/// is 「 1 will 、」（`routing.rs` Reason is 
/// 「 1 」）， not is 。30 minutes and ：
/// - ： up to when （WorkBuddy 10 minutes）；
/// - ：15 minutes、 1 when ；
/// - ： 30 minutes（ not —— is 「 」，
/// not is 「 this has 」， not this ）。
/// to to ， and 「 to 」 1 。
const SINK_TTL_MS: i64 = 30 * 60 * 1000;

/// 1 ： to `until`；`strikes` is failure（）。
#[derive(Clone, Copy, Debug)]
struct BackoffEntry {
    until: i64,
    strikes: u32,
}

/// 1 。
#[derive(Clone, Copy, Debug, Default)]
struct BreakerEntry {
    /// currentfailure（success）
    failures: u32,
    /// to when （ms；0 = ）
    open_until: i64,
    /// after （） already success
    half_open_successes: u32,
}

/// by 。 in `UpstreamService` up （ and `Connections` ）。
#[derive(Default)]
pub(super) struct RouteState {
    /// ：（request）→ down 1 。
    ///
    /// Why by ： not available not ， use 1 will 
    /// 「A 2 」error B from 。empty（client no ）
    /// use empty， 1 。
    cursors: Mutex<HashMap<String, usize>>,
    sticky: super::route_sticky::StickySessions,
    /// failure： id → 
    backoff: Mutex<HashMap<String, BackoffEntry>>,
    /// ： id → when （ has when 429 ； [`SINK_TTL_MS`] ）
    sink: Mutex<HashMap<String, i64>>,
    /// ： id → 
    breaker: Mutex<HashMap<String, BreakerEntry>>,
}

/// / / （ ）。
pub(super) struct RouteSnapshot {
    /// will in id（ no has = None）
    pub sticky_hit: Option<String>,
    /// （ = when ，only use ）
    pub sunk: HashMap<String, i64>,
    /// in （ = when ）
    pub resting: HashMap<String, i64>,
    /// 
    pub open: std::collections::HashSet<String>,
}

impl RouteState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// **limit**（）。
    ///
    /// client can this ， will as 1 （ in when 
    /// by TTL ）， not this by client —— will is UUID or 
    /// ，256 ；「 no has 」 to ， is 。
    const EXPLICIT_ID_MAX_CHARS: usize = 256;

    /// will ：****， no has to 「 + use 」 。
    ///
    /// ── from （）──────────────────────────
    /// 1. `AH-Thread-Id` —— AxonHub ** will **（ 1 thread = 1 
    /// for ）。 `AH-Trace-Id` ** not can ** will ： is 
    /// 「 use + allrequest」（AxonHub is Thread →
    /// Trace → Request）， use will 1 new 1 、。
    /// 2. OpenCode ：`X-Opencode-Session` → `X-Session-Id` → `X-Session-Affinity`
    /// 3. body `prompt_cache_key` —— ** when 1 can to that **
    /// （AxonHub when Content-Type / Accept， will id body）
    /// 4. Claude Code `metadata.user_id`（v2 JSON and legacy ）
    /// 5. Codex： `Session_id` / `Session-Id`， or `X-Codex-Turn-Metadata` `session_id`
    ///
    /// ── Why， not is ──────────────────
    /// by 「 + use 」 （ messages request
    /// can ）， that is **** —— 1 client（、、
    /// ） 1 will 。client to will id when that is ， use 
    /// not will 1 。 is 「」 and 「」 ：
    /// no client， as and up 。
    ///
    /// empty = no has 、 no has available will （ no has messages / no has use 
    /// ）， use by 「 not 」。
    pub(super) fn conversation_key(headers: &HeaderMap, body: &serde_json::Value) -> String {
        // in this 1 up ， 1 （ will 
        // ）。「 no has 」， to 。
        if let Some(explicit) = Self::explicit_conversation_id(headers, body)
            .filter(|value| value.chars().count() <= Self::EXPLICIT_ID_MAX_CHARS)
        {
            // before ： is client to any， and down 16 
            // empty， client id 。
            return format!("id:{explicit}");
        }
        Self::content_conversation_key(body)
    }

    /// （ up ，）： + use （ 4KB）。
    fn content_conversation_key(body: &serde_json::Value) -> String {
        let model = body.get("model").and_then(serde_json::Value::as_str).unwrap_or("");
        let first_user = body
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .and_then(|messages| {
                messages.iter().find(|message| {
                    message.get("role").and_then(serde_json::Value::as_str) == Some("user")
                })
            })
            .map(|message| message.get("content").cloned().unwrap_or(serde_json::Value::Null));
        let Some(content) = first_user else {
            return String::new();
        };
        let material = match &content {
            serde_json::Value::String(text) => {
                let end = text
                    .char_indices()
                    .take_while(|(index, _)| *index < 4096)
                    .last()
                    .map(|(index, ch)| index + ch.len_utf8())
                    .unwrap_or(0);
                text[..end].to_string()
            }
            other => other.to_string(),
        };
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        model.hash(&mut hasher);
        0x1fu8.hash(&mut hasher);
        material.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// will （， [`Self::conversation_key`]）。
    ///
    /// **none**： 1 in not up ， by path will in 
    /// client when 1 will （ up is none ）。
    fn explicit_conversation_id(headers: &HeaderMap, body: &serde_json::Value) -> Option<String> {
        // ① AxonHub ** will **。⚠️ `AH-Trace-Id`： that is 
        // 「 use + allrequest」（AxonHub is Thread → Trace
        // → Request）， 1 client will to 1 new ， will will 
        // —— down that （）。 up is 
        // Trace，2026-10-14 as Thread。
        if let Some(value) = Self::header_text(headers, "ah-thread-id") {
            return Some(value);
        }
        // ② OpenCode （ by new old ）
        for name in ["x-opencode-session", "x-session-id", "x-session-affinity"] {
            if let Some(value) = Self::header_text(headers, name) {
                return Some(value);
            }
        }
        // ③ OpenAI Chat prompt_cache_key：** this 1 will **。
        //
        // ：client → AxonHub（Anthropic ）→ Chat →
        // Chat 。AxonHub when Content-Type / Accept 
        // （`llm/transformer/openai/outbound.go`）， will id **body 
        // `prompt_cache_key`**（`outbound_convert.go` will id 
        // `req.PromptCacheKey`）。 up empty when ， up 。
        //
        // 、empty use ：`prompt_cache_key` is 「empty」
        // （ 1 will use ）， will and this use 1 ； no has 
        // ，empty is use to ， not need need 。
        //
        // ⚠️ not etc need use by 「 1 will 1 」 use ： **
        // ** client（all will use 1 ） will client in 1 up 
        // —— after this will ， etc 。 has 1 
        // （ [`Self::EXPLICIT_ID_MAX_CHARS`]）： is this 1 no has 
        // ， will 。
        if let Some(value) = body
            .get("prompt_cache_key")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Some(value.to_string());
        }
        // ④ Claude Code：`metadata.user_id`（ can ，
        // [`Self::parse_claude_code_user_id`]）。 not no has —— can no has 
        // ， not need 1 not will id（ client this 
        // when ， id will not request to 1 ）。
        if let Some(value) = body
            .get("metadata")
            .and_then(|metadata| metadata.get("user_id"))
            .and_then(serde_json::Value::as_str)
            .and_then(Self::parse_claude_code_user_id)
        {
            return Some(value);
        }
        // ⑤ Codex： will ， or turn metadata session_id
        for name in ["session_id", "session-id"] {
            if let Some(value) = Self::header_text(headers, name) {
                return Some(value);
            }
        }
        if let Some(metadata) = headers
            .get("x-codex-turn-metadata")
            .and_then(|value| value.to_str().ok())
        {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(metadata) {
                if let Some(session) = payload
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    return Some(session.to_string());
                }
            }
        }
        None
    }

    /// ： / ASCII 1 「 no has 」（ not URL etc ）。
    ///
    /// `HeaderMap::get` not （HTTP not ）， use 
    /// can 。** ASCII （ in 、emoji） will `to_str` failure、 this 
    /// 「 no has 」**—— this is HTTP limit（ is ISO-8859-1 ）， not is 
    /// this ：client will is UUID or 。 use curl
    /// when need —— `-H "AH-Thread-Id: in "` request not available
    /// ，「 not 」。
    fn header_text(headers: &HeaderMap, name: &str) -> Option<String> {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    /// from Claude Code `metadata.user_id` `session_id`（ AxonHub
    /// `claudecode.ParseUserID`）。
    ///
    /// ：
    /// - **v2（>=2.1.78）**：JSON 
    ///     `{"device_id":"…","account_uuid":"…","session_id":"…"}`；
    /// - **legacy**：`user_<64>_account__session_<uuid>`。
    fn parse_claude_code_user_id(raw: &str) -> Option<String> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        // v2：JSON 
        if raw.starts_with('{') {
            let payload: serde_json::Value = serde_json::from_str(raw).ok()?;
            let session = payload.get("session_id").and_then(serde_json::Value::as_str)?;
            let session = session.trim();
            return if session.is_empty() { None } else { Some(session.to_string()) };
        }
        // legacy：user_<64hex>_account__session_<uuid>
        let rest = raw.strip_prefix("user_")?;
        let (device, tail) = rest.split_once("_account__session_")?;
        let is_hex64 = device.len() == 64 && device.chars().all(|c| c.is_ascii_hexdigit());
        if is_hex64 && Self::is_uuid_shape(tail) {
            return Some(tail.to_string());
        }
        None
    }

    /// `8-4-4-4-12` UUID （ not —— then no ）
    fn is_uuid_shape(text: &str) -> bool {
        let parts: Vec<&str> = text.split('-').collect();
        let lengths = [8, 4, 4, 4, 12];
        parts.len() == lengths.len()
            && parts
                .iter()
                .zip(lengths)
                .all(|(part, len)| part.len() == len && part.chars().all(|c| c.is_ascii_hexdigit()))
    }

    pub(super) fn sticky_generation(&self) -> u64 {
        self.sticky.generation()
    }

    pub(super) fn sticky_bind(
        &self, key: &str, model: &str, account_id: &str, source: &str,
        now: i64, ttl_ms: i64, generation: u64,
    ) {
        self.sticky.bind(key, model, account_id, source, now, ttl_ms, generation);
    }

    pub(super) fn sticky_list(&self, model: Option<&str>, now: i64) -> Vec<serde_json::Value> {
        let (profiles, base) = crate::server::config::routing_snapshot();
        self.sticky.list(model, now, |name| profiles.settings_for(name, base).sticky.ttl_ms().unwrap_or(0))
    }

    pub(super) fn sticky_clear(&self, id: Option<&str>, model: Option<&str>) -> usize {
        self.sticky.clear(id, model)
    }

    pub(super) fn conversation_source(headers: &HeaderMap, body: &serde_json::Value) -> &'static str {
        if Self::explicit_conversation_id(headers, body)
            .is_none_or(|value| value.chars().count() > Self::EXPLICIT_ID_MAX_CHARS)
        {
            return "content-hash";
        }
        for (name, label) in [
            ("ah-thread-id", "AH-Thread-Id"),
            ("x-opencode-session", "X-Opencode-Session"),
            ("x-session-id", "X-Session-Id"),
            ("x-session-affinity", "X-Session-Affinity"),
        ] {
            if Self::header_text(headers, name).is_some() { return label; }
        }
        if body.get("prompt_cache_key").and_then(serde_json::Value::as_str)
            .is_some_and(|key| !key.trim().is_empty()) { return "prompt_cache_key"; }
        if body.pointer("/metadata/user_id").and_then(serde_json::Value::as_str)
            .and_then(Self::parse_claude_code_user_id).is_some() { return "Claude Code"; }
        if Self::explicit_conversation_id(headers, body).is_some() { return "Codex"; }
        "content-hash"
    }

    /// and before 1 ，return**** —— to use 。
    ///
    /// ── Why this not （2026-10-14 ）────────────────────────
    /// can to （`plan_route` before that ） and use when use 
    /// （ use / / and / none after ）** not is 1 **。 old 
    /// by before 、 ， etc in ：
    /// 3 has 1 when ， `0,1,2,0,1,2…` by 2 
    /// `0,1,0,0,1,0…` —— ** 1 in **。
    ///
    /// after ， 1 、 in 1 up ， is ；
    /// down and old （`0,1,…,len-1,0,…`）。
    pub(super) fn rotate_take(&self, model_key: &str) -> usize {
        let Ok(mut table) = self.cursors.lock() else {
            return 0;
        };
        let cursor = table.entry(model_key.to_string()).or_insert(0);
        let counter = *cursor;
        // not ： use not ， debug in limit panic。
        *cursor = counter.wrapping_add(1);
        counter
    }

    /// 「 」（ use ）， to use 。
    ///
    /// and [`Self::rotate_take`] use 1 ， 1 provider ——
    /// 「 1 down ， 1 」。 not （ after by ），
    /// **must 1 **： use 1 will A to error
    /// B from ， that is 「」 need 。
    ///
    /// `sizes` is （provider id → ）， this use ——
    /// not in this 1 （ by [`Self::rotate_take`]）， use by ** after **
    /// ；`len == 0` （ no has ， not will has ）。
    pub(super) fn rotate_take_per_provider(
        &self,
        model_key: &str,
        sizes: &[(String, usize)],
    ) -> HashMap<String, usize> {
        let Ok(mut table) = self.cursors.lock() else {
            return HashMap::new();
        };
        let mut counters = HashMap::new();
        for (provider, len) in sizes {
            if *len == 0 {
                continue;
            }
            // use `\u{1}` not is can ： is client to ，
            // can can ， 1 not will in not will 
            // not 1 。
            let key = format!("{model_key}\u{1}{provider}");
            let cursor = table.entry(key).or_insert(0);
            let counter = *cursor;
            *cursor = counter.wrapping_add(1);
            counters.insert(provider.clone(), counter);
        }
        counters
    }

    /// 1 failure（failure，limit 1 when ），return when 。
    pub(super) fn backoff_mark(&self, account_id: &str, now: i64) -> i64 {
        let Ok(mut table) = self.backoff.lock() else {
            return 0;
        };
        let entry = table.entry(account_id.to_string()).or_insert(BackoffEntry {
            until: 0,
            strikes: 0,
        });
        entry.strikes = entry.strikes.saturating_add(1);
        let factor = 1i64 << entry.strikes.saturating_sub(1).min(3); // 1x / 2x / 4x / 8x
        let wait = (BACKOFF_BASE_MS * factor).min(BACKOFF_MAX_MS);
        entry.until = now + wait;
        entry.until
    }

    /// success 1 → （failure ）。
    pub(super) fn backoff_clear(&self, account_id: &str) {
        if let Ok(mut table) = self.backoff.lock() {
            table.remove(account_id);
        }
    }

    /// 1 （ has when 429 ）。
    pub(super) fn sink_mark(&self, account_id: &str, now: i64) {
        if let Ok(mut table) = self.sink.lock() {
            table.insert(account_id.to_string(), now);
        }
    }

    /// success 1 → from （ can ）。
    pub(super) fn sink_clear(&self, account_id: &str) {
        if let Ok(mut table) = self.sink.lock() {
            table.remove(account_id);
        }
    }

    /// 1 failure；return is ****（used for）。
    pub(super) fn breaker_failure(
        &self,
        account_id: &str,
        threshold: i64,
        timeout_seconds: i64,
        now: i64,
    ) -> bool {
        let Ok(mut table) = self.breaker.lock() else {
            return false;
        };
        let entry = table.entry(account_id.to_string()).or_default();
        let was_open = entry.open_until > now;
        entry.failures = entry.failures.saturating_add(1);
        entry.half_open_successes = 0;
        if !was_open && entry.failures as i64 >= threshold.max(1) {
            entry.open_until = now + timeout_seconds.max(0) * 1000;
            return true;
        }
        // failure： new 
        if entry.open_until > 0 && entry.open_until <= now {
            entry.open_until = now + timeout_seconds.max(0) * 1000;
            return true;
        }
        false
    }

    /// success 1 → （success to ）。
    pub(super) fn breaker_success(&self, account_id: &str, success_threshold: i64) {
        let Ok(mut table) = self.breaker.lock() else {
            return;
        };
        let Some(entry) = table.get_mut(account_id) else {
            return;
        };
        entry.failures = 0;
        if entry.open_until > 0 {
            entry.half_open_successes = entry.half_open_successes.saturating_add(1);
            if entry.half_open_successes as i64 >= success_threshold.max(1) {
                table.remove(account_id);
            }
        }
    }

    /// ： in + / / （ 1 ）。
    pub(super) fn snapshot(&self, conv_key: &str, now: i64, ttl_ms: i64) -> RouteSnapshot {
        self.snapshot_for_model(conv_key, "", now, ttl_ms)
    }

    pub(super) fn snapshot_for_model(&self, conv_key: &str, model: &str, now: i64, ttl_ms: i64) -> RouteSnapshot {
        let sticky_hit = self.sticky.lookup(conv_key, model, now, ttl_ms);
        // by [`SINK_TTL_MS`] —— and down resting / open 。
        // up is （ no has when ）， is not ；
        // 1 path is 「 this success 1 」， in —— 。
        let sunk = self
            .sink
            .lock()
            .map(|table| {
                table
                    .iter()
                    .filter(|(_, at)| now - **at < SINK_TTL_MS)
                    .map(|(id, at)| (id.clone(), *at))
                    .collect()
            })
            .unwrap_or_default();
        let resting = self
            .backoff
            .lock()
            .map(|table| {
                table
                    .iter()
                    .filter(|(_, entry)| entry.until > now)
                    .map(|(id, entry)| (id.clone(), entry.until))
                    .collect()
            })
            .unwrap_or_default();
        let open = self
            .breaker
            .lock()
            .map(|table| {
                table
                    .iter()
                    .filter(|(_, entry)| entry.open_until > now)
                    .map(|(id, _)| id.clone())
                    .collect()
            })
            .unwrap_or_default();
        RouteSnapshot {
            sticky_hit,
            sunk,
            resting,
            open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};
    use serde_json::json;

    fn headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(
                HeaderName::from_bytes(name.as_bytes()).expect("合法头名"),
                HeaderValue::from_str(value).expect("合法头值"),
            );
        }
        map
    }

    fn chat_body() -> serde_json::Value {
        json!({ "model": "m", "messages": [{ "role": "user", "content": "hello" }] })
    }

    /// ： 1 body， 1 ，must in up 。
    #[test]
    fn an_explicit_id_beats_the_content_hash() {
        let body = chat_body();
        let keyed = RouteState::conversation_key(&headers(&[("ah-thread-id", "thread-abc")]), &body);
        let hashed = RouteState::conversation_key(&HeaderMap::new(), &body);
        assert_eq!(keyed, "id:thread-abc");
        assert_ne!(keyed, hashed);
        // no when is up that （16 ）
        assert_eq!(hashed.len(), 16);
        assert!(hashed.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// `AH-Trace-Id` is 「 use 」，** not is ** will —— not can will 。
    ///
    /// AxonHub is Thread（ will ）→ Trace（ use 1 ）→ Request。
    /// Trace will ，client 1 new 1 ⇒ ，
    /// （ use not 、）。 up is Trace，
    /// 2026-10-14 as Thread and Trace from 。
    #[test]
    fn an_axonhub_trace_id_is_not_a_session_key() {
        let body = chat_body();
        // has Trace when ， not is id:at-abc
        let key = RouteState::conversation_key(&headers(&[("ah-trace-id", "at-abc")]), &body);
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
        // will Thread then 
        assert_eq!(
            RouteState::conversation_key(&headers(&[("ah-thread-id", "thread-abc")]), &body),
            "id:thread-abc"
        );
    }

    /// （）。
    #[test]
    fn every_source_of_an_explicit_id_is_recognised() {
        let body = chat_body();
        let cases: Vec<(HeaderMap, serde_json::Value, &str)> = vec![
            (headers(&[("ah-thread-id", "thread-abc")]), body.clone(), "id:thread-abc"),
            (headers(&[("x-opencode-session", "sess-opencode")]), body.clone(), "id:sess-opencode"),
            (headers(&[("x-session-id", "sess-x")]), body.clone(), "id:sess-x"),
            (headers(&[("x-session-affinity", "sess-affinity")]), body.clone(), "id:sess-affinity"),
            (headers(&[("session-id", "sess-codex")]), body.clone(), "id:sess-codex"),
            (
                headers(&[("x-codex-turn-metadata", "{\"session_id\":\"sess-turn\"}")]),
                body.clone(),
                "id:sess-turn",
            ),
            (HeaderMap::new(), json!({ "prompt_cache_key": "sess-cache" }), "id:sess-cache"),
            (
                HeaderMap::new(),
                json!({ "metadata": { "user_id": "{\"session_id\":\"sess-claude\"}" } }),
                "id:sess-claude",
            ),
        ];
        for (headers, body, expected) in &cases {
            assert_eq!(RouteState::conversation_key(headers, body), *expected);
        }
    }

    /// not for `metadata.user_id` not this will id —— can no has 。
    #[test]
    fn a_malformed_claude_user_id_falls_back_to_the_content_hash() {
        let body = json!({
            "model": "m",
            "messages": [{ "role": "user", "content": "hello" }],
            "metadata": { "user_id": "随便一个串" },
        });
        let key = RouteState::conversation_key(&HeaderMap::new(), &body);
        assert!(!key.starts_with("id:"));
        assert_eq!(key.len(), 16);
    }

    /// is ****， by use —— not 1 when not will 。
    ///
    /// is 2026-10-14 1 ： old by 「 use 」、
    /// ， use is ** after ** 。 3 、 its in 1 
    /// （ and has 2 ） when ， in `0,1,2` 、 by 2
    /// `0,1,0,0,1,0` —— 3、4 in 1 。
    #[test]
    fn rotation_counters_are_not_wrapped_inside_the_state() {
        let state = RouteState::new();
        // 3 、 can 2 ：， by can 2 
        let positions: Vec<usize> = (0..6)
            .map(|_| state.rotate_take("m") % 2)
            .collect();
        // 0..5 = 0,1,0,1,0,1 —— （ old will is 0,1,0,0,1,0）
        assert_eq!(positions, vec![0, 1, 0, 1, 0, 1]);
    }

    /// ： 1 ， not ； not then 1 。
    #[test]
    fn family_counters_advance_per_family_and_per_model() {
        let state = RouteState::new();
        let sizes = vec![("A".to_string(), 3usize), ("B".to_string(), 2usize)];
        let first = state.rotate_take_per_provider("m", &sizes);
        let second = state.rotate_take_per_provider("m", &sizes);
        // return is （ not ）， and 
        assert_eq!(first.get("A"), Some(&0));
        assert_eq!(first.get("B"), Some(&0));
        assert_eq!(second.get("A"), Some(&1));
        assert_eq!(second.get("B"), Some(&1));
        // → 1 
        assert_eq!(state.rotate_take_per_provider("m2", &sizes).get("A"), Some(&0));
    }

    /// must** to **。
    ///
    /// up `snapshot` sink （ no has when ），
    /// 1 path `sink_clear` in 「 this success 1 」 after use ——
    /// can in ， not to ， is 。： 11 
    /// in 1 429 after 。
    #[test]
    fn sunk_accounts_come_back_when_their_ttl_expires() {
        let state = RouteState::new();
        let now = 1_700_000_000_000i64;
        state.sink_mark("a1", now);

        // up ： in 
        let fresh = state.snapshot("", now, 0);
        assert!(fresh.sunk.contains_key("a1"), "刚垫底的账号应当在快照里");

        // 1 to TTL： in 
        let almost = state.snapshot("", now + SINK_TTL_MS - 1, 0);
        assert!(almost.sunk.contains_key("a1"), "未到 TTL 前不该提前解除");

        // to ：（ this is ）
        let expired = state.snapshot("", now + SINK_TTL_MS, 0);
        assert!(
            !expired.sunk.contains_key("a1"),
            "超过 TTL 后应当自动回到队列 —— 上游原实现在这里恒为「仍在」"
        );
    }

    /// `sink_clear` need can （success after not this etc TTL）。
    #[test]
    fn a_successful_forward_clears_the_sink_immediately() {
        let state = RouteState::new();
        let now = 1_700_000_000_000i64;
        state.sink_mark("a1", now);
        state.sink_clear("a1");
        let snap = state.snapshot("", now, 0);
        assert!(!snap.sunk.contains_key("a1"), "成功后应当立刻解除垫底");
    }
}
