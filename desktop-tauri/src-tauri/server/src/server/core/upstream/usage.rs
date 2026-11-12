//! usage （requeststatistics use ）： from up chunk token usage and 。
//!
//! ── Why「」──────────────────────────────────────────
//! use in ** already 、 up need down ** （`sse.rs` 
//! 、`aggregate.rs` ）： 1 JSON、 1 ，
//! not and 、 not need down 。nonesuccess、failure is 
//! ，client to and before 1 —— this is file 。
//!
//! ── Why need ───────────────────────────────
//! up is OpenAI ， not / not for usage and not 1 ：
//! prompt_tokens（OpenAI ） / input_tokens（）/ promptTokens
//! / inputTokens（， `protocol::commandcode_outbound` 1 ）
//!   completion_tokens / output_tokens / completionTokens / outputTokens
//! in ：prompt_tokens_details.cached_tokens（OpenAI ）
//! / cache_read_tokens / cache_read_input_tokens（Anthropic ）
//! / cacheReadTokens / cachedInputTokens
//! / input_tokens_details.cached_tokens（Responses new ）
//! ：credit（WorkBuddy `usage.credit`， can ）
//! / credits / credit_used / creditUsed / credit_cost etc 
//! to ： can 1 ， not need as for not up 0。
//!
//! ── Why 1 need 「 1 」 not is 「 1 in 」 ──
//! up will ** when ** to and ， and that is 。 1 ：
//! `{"prompt_tokens_details":{"cached_tokens":0}, "cache_read_input_tokens":4800}`
//! —— by 「 in and 」， in is that 0 ，usage 0，
//! in 0（ 0 is 「 this no 」 ，）。
//! by 「 1 」，0 and as 「 this 1 no 」， to down 1 。
//! for prompt/completion/total 1 ：`total_tokens: 0` is ，
//! this to prompt + completion， not is total 0。
//!
//! ── Why this ** not ** cache_write / cache_creation ────────────
//! `usage.cache_read` 1 is request `cache_read_tokens` 
//! （`db/schema.rs:354`）， is ** in ** 
//! （`request_stats::report::safe_rate` = hit / input，`aggregate.rs:216` 
//! `cache_hit_tokens`）。「」 and 「 in 」 is not ，
//! 1 will in （`protocol::anthropic_outbound.rs:801` that 
//! `cache_read + cache_creation` and is to Anthropic `cached_tokens`
//! use ， is 「prompt total」， and this in not ）。
//! need ， to —— that not is 「 for 」 ，file。
//!
//! ── Why is 「 up 、 by 」────────────────
//! usage in after 1 chunk （ has will in in ），
//! is 「 after 1 up 」。 to is up 
//! none： not need need 「 this 1 is not is after 1 」， not can can as 。
//!
//! ── Why this not return Result ─────────────────────────────────
//! up is ： 1 （ not for 、 in ） not can 。
//! all、 not panic —— statistics 1 is can ， use request
//! not can （release is panic=abort， 1 panic will use ）。

use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{Map, Value};

mod attempt;
pub use attempt::{
    AttemptDetail, RetryEvent, MAX_ATTEMPT_DETAILS, MAX_ATTEMPT_ERROR_CHARS,
    MAX_ATTEMPT_ACCOUNT_CHARS, MAX_ATTEMPT_RETRY_REASON_CHARS, MAX_ATTEMPT_NOTICE_CHARS,
};

use crate::server::logging;

/// 1 up usage（ token + optional ， for ）。
///
/// token is ；`credit` is ****（ up by request to ，
/// e.g. WorkBuddy `usage.credit`）， can as ， use `Option<f64>`
/// —— `None` = this 1 no up ， and 「 up 0」 is 。
///
/// has this is as has this （module header「Why not cache_write」 that ）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UsageTokens {
    pub prompt: i64,
    pub completion: i64,
    pub total: i64,
    pub cache_read: i64,
    pub credit: Option<f64>,
}

/// 「 token 」 （）。** is **： in when by 
/// 1 ****（ [`first_positive`]）， not is and 、 not is 。
const PROMPT_KEYS: &[&str] = &["prompt_tokens", "input_tokens", "promptTokens", "inputTokens"];
/// 「 token 」 。
const COMPLETION_KEYS: &[&str] = &[
    "completion_tokens",
    "output_tokens",
    "completionTokens",
    "outputTokens",
];
/// 「total」 ；（ or as 0） when by prompt + completion 。
const TOTAL_KEYS: &[&str] = &["total_tokens", "totalTokens"];
/// 「 in token 」 ****。
///
/// `cache_read_input_tokens` is Anthropic （ / in ，
/// `protocol::anthropic.rs` and `protocol::responses_outbound.rs`）； is 
/// Command Code 1 up （`protocol::commandcode_outbound.rs` 1 before 
/// is this 1 ）；`cached_*` is to 。
///
/// ** is **（ and [`PROMPT_KEYS`] 1 ）：`cached_tokens` in before ，
/// as is OpenAI ； by not is 「」 is 「」， as when 
/// by 1 。 need is ** this 1 1 as 0 when **： 1 ****
/// not is 1 「 in 」，otherwise that 0 `cached_tokens` will has 
/// `cache_read_input_tokens` （0 in before not is ， is 「 this no 」 
/// ， 1 ）。
const CACHE_READ_KEYS: &[&str] = &[
    "cached_tokens",
    "cached_input_tokens",
    "cache_read_tokens",
    "cache_read_input_tokens",
    "cacheReadTokens",
    "cacheReadInputTokens",
    "cachedInputTokens",
];
/// 「 in token 」 ****（ [`CACHE_READ_KEYS`]）。
///
/// `prompt_tokens_details` is OpenAI （ `cached_tokens`）；
/// `input_tokens_details` is Responses prompt input after 。
/// ： up when to 、 its in 1 is already （module header）。
const CACHE_READ_CONTAINERS: &[&str] = &["prompt_tokens_details", "input_tokens_details"];
/// 「」 （）。
///
/// WorkBuddy `usage.credit` is already 1 ； its is 
/// down / 。`points_*` 1 Loomy `points_consumed`
/// （ is ）， 1 and ，「 up points」 not need need 。
/// `cost` Cline `usage.cost`： is **** not is ，
/// by 「 is 」 and use 1 （ in after ， when in when 
/// credit）。 and token 1 ：** 1 **，
/// 0 in after ； 1 no has when return None。
const CREDIT_KEYS: &[&str] = &[
    "credit",
    "credits",
    "credit_used",
    "creditUsed",
    "credits_used",
    "creditsUsed",
    "credit_cost",
    "creditCost",
    "credit_consumed",
    "creditConsumed",
    "points_consumed",
    "pointsConsumed",
    "points_used",
    "pointsUsed",
    "cost",
];

/// from usage for token （ and 「 1 」module header）。
///
/// return None has 1 ：`usage` not is for （null / / / ）。
/// for ** or as ** by 「 this no 」， to down 1 ；
/// empty by 0 。`total` is prompt + completion。
pub fn extract_usage(usage: &Value) -> Option<UsageTokens> {
    let object = usage.as_object()?;
    let prompt = first_positive(object, PROMPT_KEYS).unwrap_or(0);
    let completion = first_positive(object, COMPLETION_KEYS).unwrap_or(0);
    let total = first_positive(object, TOTAL_KEYS).unwrap_or(prompt + completion);
    // （），；。
    let cache_read = CACHE_READ_CONTAINERS
        .iter()
        .filter_map(|container| object.get(*container).and_then(Value::as_object))
        .find_map(|details| first_positive(details, CACHE_READ_KEYS))
        .or_else(|| first_positive(object, CACHE_READ_KEYS))
        .unwrap_or(0);
    let credit = first_amount(object, CREDIT_KEYS);
    Some(UsageTokens { prompt, completion, total, cache_read, credit })
}

/// from 1 JSON for by 。
///
/// to 「 in response、 not in `usage` 」 use ：Loomy 
/// `points_consumed` is this （`{created, data:[…], points_consumed}`）。
/// and [`extract_usage`] use 1 [`CREDIT_KEYS`] ， 1 when 。
pub fn credit_from_object(object: &Map<String, Value>) -> Option<f64> {
    first_amount(object, CREDIT_KEYS)
}

/// by ** 1 **（module header）。
fn first_positive(object: &Map<String, Value>, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| positive_field(object.get(*key)))
}

/// ，****。
///
/// up need ：
/// - （`as_i64`）；
/// - `1.0` this （`as_i64` for failure， 1 f64 ；
/// up not to / has ）；
/// - ****（`"34"`， usage ；
/// f64 `"12.0"` ）。
///
/// 0 and 1 return None（= 「 this 1 no 」， to down 1 ）：
/// - 0 is up to ， by 「 has 」 use will in after that up 
/// （module header that `cached_tokens` 0 is this 1 ）；
/// - is or up 。 this will ****
/// （`request_stats::aggregate` `cache_hit_tokens += …`）， 1 can 
/// ， `report::safe_rate` 、 not and 。
/// 0 can ， that will this 「 has 」 after ，
/// by None 。
///
/// （bool / null / / not ） None， to 。
fn positive_field(value: Option<&Value>) -> Option<i64> {
    let raw = match value? {
        Value::Number(number) => number
            .as_i64()
            .or_else(|| number.as_f64().map(|number| number as i64)),
        Value::String(text) => text
            .trim()
            .parse::<f64>()
            .ok()
            .map(|number| number as i64),
        _ => None,
    };
    raw.filter(|number| *number > 0)
}

/// by ** 1 **； is 0 when return 0（「 up 0」）。
///
/// and [`first_positive`] has ， is as ：
/// - ** not **（`0.06` must， not can 0）；
/// - 0 not is ：all to 0 when ， this request「」，
/// and 「 1 no has 」（None = this 1 not this ）must。
///
/// （ / up ）： not can 「 has 」，
/// not can 「」。
fn first_amount(object: &Map<String, Value>, keys: &[&str]) -> Option<f64> {
    let mut zero = None;
    for key in keys {
        let Some(value) = amount_field(object.get(*key)) else { continue };
        if value > 0.0 {
            return Some(value);
        }
        zero = zero.or(Some(0.0));
    }
    zero
}

/// has ： and 「」（ and [`positive_field`] 1 ，
/// not i64 ）。
fn amount_field(value: Option<&Value>) -> Option<f64> {
    let raw = match value? {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    };
    raw.filter(|number| number.is_finite() && *number >= 0.0)
}

/// in requestcurrent ****（request 1 ）。
///
/// ── ──────────────────────────────────────────
/// - `Connecting` in ：request， / / / request
/// in this （ up request no ）；
/// - `Waiting` etc response： up request** already **， etc 1 
/// （ when in this 1 ， and OmniProxy 「 etc response（）」）；
/// - `Streaming` response in ： already to 、 in down ；
/// - `Retrying` in ：failure， etc / before （ down 1 to 
///     `Waiting`）。
///
/// ── and OmniProxy for ─────────────────────────────────
/// before and that `LogPhase`（`connecting` / `waiting` / `streaming` /
/// `retrying`） for ， （`queued`） is （ up 
/// etc ，OmniProxy no has for ）：`as_str()` return is ，
/// before by 
/// （ in / etc response / response in / in —— 1 ）。 when 1 ：
/// ** when ** 1 （[`TelemetrySnapshot::phase_started_at`]），
/// before that is 「 in current」 not is total when 。
///
/// ── Why and 「」 is ──────────────────────────
/// `Streaming` and `first_response_at` in 1 use （`note_first_frame`），
/// not etc ： is ** 1 **（ after need ， 「」），
/// is **current**（empty， in has ）。 use 1 will 
/// 「requestresponse in 」 this has will 。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogPhase {
    /// request already ， no up request（ ）
    #[default]
    Connecting,
    /// up request already ， etc 
    Waiting,
    /// already to ， in down 
    Streaming,
    /// failure， / in 
    Retrying,
    /// ** up in **： up no ， is request（），
    /// by up when etc （ `qoder::queue_backoff`）。
    ///
    /// and `Retrying` by ： ** and not ** —— is 「 this 1 
    /// failure 1 」， is 「 this 1 no ， in etc up 」。 use to 「 in 」
    /// 「 not is 、 not is has ， is 」，「 in 」 will as 
    /// up 。 etc by `note_queued` ， after 
    /// `Waiting`（`note_attempt_started`）。
    Queued,
}

impl LogPhase {
    /// and API （ and OmniProxy `LogPhase` ）
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Connecting => "connecting",
            Self::Waiting => "waiting",
            Self::Streaming => "streaming",
            Self::Retrying => "retrying",
            Self::Queued => "queued",
        }
    }
}

/// need （ 1 ， when ）
#[derive(Clone, Debug, Default)]
pub struct TelemetrySnapshot {
    /// ****（0 = no to failure）。
    ///
    /// ： 1 「」= 1 = to 1 1 up request
    /// （ `forward` ）。 1 11128 
    /// （`request_with_waf_retry` 10s/25s ）** not ** —— is when 
    /// not is ，「 success」「total up 」 use 。
    ///
    /// is 「、 ≥1」， 0 in 「 1 no 」 when ，
    /// by `.max(1)` 1 。
    pub attempts: i64,
    /// request id（ before 1 ， not ）。
    ///
    /// use ：request and （`core::debug_traffic`） use 1 
    /// id —— 「」。empty = this no has （
    /// before old ）， before not 。
    pub id: String,
    /// request （ use default、 or none when as empty）
    pub account_id: String,
    /// ， and 1 ： → will → id
    /// （ id as empty will when ， this can can empty）
    pub account_name: String,
    /// request **provider id**（Agent2API W2b-T3 add；
    /// `None` = no to failure，e.g.request / not in ）。
    ///
    /// Why id not is `ProviderKind`： this need to 
    /// （`api::chat` → `request_stats`），request `provider` ，
    /// in and before —— is id （architecture §3.6）。
    /// 1 「kind → id」 and 。
    pub provider: Option<String>,
    /// to up （ + by after ****；empty =
    /// 1 no ，e.g. before failure）。
    ///
    /// and `provider` 1 「 after 1 as 」：429 （ can can ） after ，
    /// request 「 up 」 this that 1 。 in 
    /// （`upstream::payload::send_body`， when 1 ）。
    /// empty and `account_id` 1 ： in 、empty no has 。
    pub upstream_model: String,
    /// up request ** etc **（empty = no has etc ）。
    ///
    /// and `upstream_model` when （`payload::send_body`，）、
    /// 1 「 after 1 as 」：429 after down that 1 etc 。
    /// empty ：client no and no 、 not etc （ 
    /// `outbound_reasoning` return None）、「」 down 。
    /// request use to up `( etc )` after 。
    pub upstream_reasoning: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cache_read_tokens: i64,
    /// up by request up ****（WorkBuddy `usage.credit`；`None` = this 1 
    /// not this ， as NULL， before 「—」 not is 0）。
    ///
    /// and token 1 「 after 1 up 」： 1 request can can has ，
    /// after that up `usage` （ and prompt / completion ）。
    pub credit: Option<f64>,
    pub quota_channel: String,
    /// up to ** for when **（ms Unix when ；None = no has to ，
    /// e.g. before failure / request down ）。
    ///
    /// Why for when not is for when ：（response） and （request）
    /// in ，「request when 」 has —— for 
    /// not need need ， in 1 （`record_entry`）， and durationMs
    /// use 1 `started_at`， not can can 。
    ///
    /// for OmniProxy request `ttfb_ms`（time to first byte）：
    /// 「 up 」， 「 etc up 」 and 「」
    /// when —— has durationMs when ， 1 30 seconds request not is up 
    /// is 。
    pub first_response_at: Option<i64>,
    /// current （ [`LogPhase`]； 1 is 「 in 」）。
    ///
    /// in ** in ** has ： not （ `phase` 
    /// 1 empty， `request_stats::sql` ）， has in 
    /// （`phaseElapsedMs` down 1 ）。
    pub phase: LogPhase,
    /// **current** for when （ms Unix when ；None = in 
    /// 「 in 」 up ， 1 no ）。
    ///
    /// and `first_response_at` 1 for ： not 「request when 
    /// 」， need is 「 in this 」——
    /// in that 1 can （ in `requests.phase_started_at`，
    /// API when `phaseElapsedMs`）。
    ///
    /// ── Why is None not is when now ────────────────────
    /// 「 in 」 is **request when **（`record_started` `ts`），
    /// that is `started_at`， this not to 。None 
    /// 「 no 」， in use `started_at` （
    /// `api::pipeline::live_row_sink`）—— is 1 when ，
    /// not will 「 3 seconds、 2.9 seconds」 this for not up 。
    pub phase_started_at: Option<i64>,
    /// in / Reason（success as None）
    pub error: Option<String>,
    /// **request in then **（ then + ；empty = 1 no in ）。
    ///
    /// ── from ──────────────────────────────────────────────
    /// `core::sanitize` `sanitize_body` in when in 
    /// （ by then in ）。 and ** then **
    /// —— `sensitive_hits`、 `{word, count}`、 before 「」 and 
    /// ， is `word` from 「 in 」
    /// 「 in then 」（ `11128` / `cc_*=` / ）。
    ///
    /// in `payload::send_body`（ 1 will 、 to that 1 ），
    /// by [`Self::note_sensitive_hits`] and 。
    ///
    /// ── Why is and （ and attempts for ）───────────────
    /// `payload::send_body` ： when in 1 ，
    /// and is 「 this request in 」 。
    pub sensitive_hits: Vec<SensitiveHit>,
    /// ** 1 up **（ by ， and 1 1 for ）。
    ///
    /// ── Why need need （Agent2API request）───────────────────
    /// has `attempts`（） and `provider`（） ，
    /// is request that 1 can 1 「」 —— use to 「」，
    /// not to **、 1 failure in 、 after is down **。 this in 
    /// 1 is （provider id、HTTP 、error need ），
    /// is before no has 。
    ///
    /// ── （ and `attempts` / `provider` not ）──────────────
    /// `attempts` and `provider` is 「 after 1 as 」 ****；
    /// ， is **** 。 not ：「 up 」
    /// （ need is this ），「 this 1 is 」。
    ///
    /// ── Why「 already 」 not is 「 to 」───────────────────
    /// in `note_attempt` （ this in ** already 、 will before **
    /// use ）， and `attempts` 。 in 
    /// （ use / / unsupported this ）** not this ** —— no has 
    /// up ， will 「path」 1 from no request 。
    ///
    /// limit [`MAX_ATTEMPT_DETAILS`]： this is 1 request ，
    /// when JSON ，must has up 。
    pub attempts_detail: Vec<AttemptDetail>,
    pub route: Option<Value>,
    route_revision: u64,
}

/// 1 in and its （`TelemetrySnapshot::sensitive_hits` ）。
///
/// and before `sensitiveHits` 1 （`word` / `count`），
/// and OmniProxy `SensitiveTermHit` —— that `sensitive_masked_terms`
/// is 1 `[{word, count}]`。 not is ： 1 
/// （ in × ）， 1 no 。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SensitiveHit {
    /// in （ ， 1 —— use is ）
    pub word: String,
    /// request in 
    pub count: i64,
}

/// by （ use `…` ）。
///
/// not `request_stats::truncate_chars`： in `api::pipeline` and 
/// `account_store::store_util` ， in `core::upstream`， 1 
/// has will to （core not api）。 by **** not is ：
/// in need by will 。
fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out: String = text.chars().take(limit).collect();
    out.push('…');
    out
}

/// in ： current to 「 in 」（
/// [`RequestTelemetry::set_live_sink`]）。
///
/// is `Arc<dyn Fn>` not is ： in `core::upstream` down ，
/// ** not ** `request_stats`（ to `core/mod.rs` ）。 in api 
/// （`api::pipeline::live_row_sink`）， this in when use 。
pub type LiveSink = Arc<dyn Fn(&TelemetrySnapshot) + Send + Sync>;

/// in ****（ [`RequestTelemetry::flush_live`]）。
///
/// is 「 in will that 」 1 for 1 need ：， or 「 has no has / has 」。
///
/// ── Why「、」 not is 「 for 」─────────
/// 1 need ，path 1 will 1 `flush_live`
/// （`note_first_frame` in up ）—— this 。 is 1 
/// for ， that will ****： when 1 ， need not 
/// （ not when ）、 need （ 1 ），。 this 
/// 「 1 」： 1 path up response， not as 1 
/// 。
///
/// ── Why not `error` ──────────────────────────────────────
/// that 1 1 has ， before not this 1 「 in 」（ `ui/requests-panel.js`
/// `isRunning`）， in failure is before 1 failure —— before 
/// failure， is 「 in 」 this need 。failureReason by 
/// （`last_error`）， not error 。
///
/// ── Why not token ─────────────────────────────────
/// and `RunningProgress` ： in not usage， as 1 not update no has 
/// （usage in up after 1 chunk ， will ）。
///
/// ── Why `phase` not `phase_started_at` ───────────────
/// when is 「 in - 」， 1 in —— will 
/// （path 1 ， is need ）。** in when 
/// **， is 「」 this 1 ： `phase`
/// can ， when 。
#[derive(Default, PartialEq)]
struct LiveStamp {
    provider: String,
    account_id: String,
    account_name: String,
    upstream_model: String,
    upstream_reasoning: String,
    attempts: i64,
    /// ** for when **（ and ， in api ）
    first_response_at: Option<i64>,
    /// current（ 1 —— and when ）
    phase: LogPhase,
    /// + ** after 1 ** （ / has no has error / /
    /// ）。 after 1 ： is ， before 1 not 。
    details: usize,
    last_status: Option<i64>,
    last_error: bool,
    last_retries: usize,
    last_notice: bool,
    last_response_at: Option<i64>,
    last_finished_at: Option<i64>,
    route_revision: u64,
    sensitive: usize,
}

impl LiveStamp {
    fn of(snapshot: &TelemetrySnapshot) -> Self {
        let last = snapshot.attempts_detail.last();
        Self {
            provider: snapshot.provider.clone().unwrap_or_default(),
            account_id: snapshot.account_id.clone(),
            account_name: snapshot.account_name.clone(),
            upstream_model: snapshot.upstream_model.clone(),
            upstream_reasoning: snapshot.upstream_reasoning.clone(),
            attempts: snapshot.attempts,
            first_response_at: snapshot.first_response_at,
            phase: snapshot.phase,
            details: snapshot.attempts_detail.len(),
            last_status: last.and_then(|item| item.status),
            last_error: last.map(|item| item.error.is_some()).unwrap_or(false),
            last_retries: last.map(|item| item.retries.len()).unwrap_or(0),
            last_notice: last.map(|item| item.notice.is_some()).unwrap_or(false),
            last_response_at: last.and_then(|item| item.response_at),
            last_finished_at: last.and_then(|item| item.finished_at),
            route_revision: snapshot.route_revision,
            sensitive: snapshot.sensitive_hits.len(),
        }
    }
}

/// in （ as empty = not enabled，e.g. before failure path ——
/// that request no has in ，none can ）。
#[derive(Default)]
struct LiveWrite {
    sink: Option<LiveSink>,
    /// up 1 ** already ** that （ before is empty： and not ）
    stamp: LiveStamp,
}

/// usage up ：， in when 。
///
/// Why use not is 1 return ： in 
/// **handler return after **（ by response axum ）， that when already return，
/// can 1 can to 。 in 1 request， not in use 。
pub struct RequestTelemetry {
    inner: Mutex<TelemetrySnapshot>,
    /// （`core::debug_traffic`；None = ）。
    ///
    /// in this not is ：path in `ForwardStream::poll_next`
    /// （handler already return）， in ， up has telemetry
    /// —— in ** will before ** ，path from 1 。
    ///
    /// 1 （ not and `inner` use ）： in path up ，
    /// and 「」 can not etc 。
    capture: Mutex<Option<Arc<super::super::debug_traffic::TrafficCapture>>>,
    /// in （ + up 1 already ； [`Self::set_live_sink`]）
    live: Mutex<LiveWrite>,
    /// （`core::upstream::cancellation`；None = this request
    /// no —— before failure path， or use ）。
    ///
    /// and `capture` 1 ： etc （ etc response、、
    /// ） up has telemetry， this ； in handler
    /// （`api::chat` / `api::protocol` in `record_started` after ）。
    cancel: Mutex<Option<Arc<super::cancellation::CancelToken>>>,
}

impl Default for RequestTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestTelemetry {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(TelemetrySnapshot::default()),
            capture: Mutex::new(None),
            live: Mutex::new(LiveWrite::default()),
            cancel: Mutex::new(None),
        }
    }

    /// 1 already id （ for this ： before 1 ）。
    ///
    /// Why not id `new()`：`RequestTelemetry::new()` 
    /// 「 before failure」 path use （`record_early_failure`）， that request no has 
    /// can ，empty 1 id will 1 no has 。
    pub fn with_id() -> Self {
        let telemetry = Self::new();
        telemetry.ensure_id(&super::request::new_request_id());
        telemetry
    }

    /// （ will before 1 ）。
    pub fn set_capture(&self, capture: Arc<crate::server::core::debug_traffic::TrafficCapture>) {
        let mut guard = self
            .capture
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(capture);
    }

    /// （ when as None， use ）
    pub fn capture(
        &self,
    ) -> Option<Arc<crate::server::core::debug_traffic::TrafficCapture>> {
        self.capture
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// （ handler in `record_started` after 1 ）。
    ///
    /// and `ensure_id` 1 when and 1 「 as 」 ： in request
    /// when 1 ， up not （ `cancellation::cancel`，
    /// not from this ）。
    pub fn set_cancel_token(&self, token: Arc<super::cancellation::CancelToken>) {
        let mut guard = self
            .cancel
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = Some(token);
    }

    /// （ when as None， etc 「 not can 」 as ）
    pub fn cancel_token(&self) -> Option<Arc<super::cancellation::CancelToken>> {
        self.cancel
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// request is already （ ； when false）
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token()
            .map(|token| token.is_cancelled())
            .unwrap_or(false)
    }

    /// and down request id（ before 1 ）。
    ///
    /// ** as **： use not —— id 1 ，request and must
    /// ， in 1 will already 。emptyparameter not （ use not to 
    /// id when empty， before not ）。
    pub fn ensure_id(&self, id: &str) {
        if id.is_empty() {
            return;
        }
        let mut guard = self.lock();
        if guard.id.is_empty() {
            guard.id = id.to_string();
        }
    }

    /// request id（ when as empty）
    pub fn id(&self) -> String {
        self.lock().id.clone()
    }

    /// in （** before ** by use 1 ； `LiveSink`）。
    ///
    /// ── ──────────────────────────────────────────────
    /// 「 in 」（`RequestStats::record_started` that ） has id / ts / ：
    /// and in after ， is that 1 not ** in 
    /// 、 is 、 already **。 this in has ，
    /// when that 1 。
    ///
    /// ── Why by use ， not is ────────────────────
    /// in core down 、 not （ by `LiveSink`）； use 
    /// （`api::chat` / `api::protocol`） has `Arc<RequestStats>` has request
    /// id and when ， in `record_started` after 。** not ** path： before 
    /// failure （`record_early_failure`）—— that request no has in ，none can 。
    pub fn set_live_sink(&self, sink: impl Fn(&TelemetrySnapshot) + Send + Sync + 'static) {
        let mut live = self
            .live
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        live.sink = Some(Arc::new(sink));
    }

    /// ** ** to in （ will 1 ）。
    ///
    /// ── Why is 「」 not is when ──────────────────────
    /// path to 1 will `note_first_frame`（ etc ）， when will 「
    /// 」 —— in in when up 。 is ：
    /// 1 up 1 ，。 is up etc **
    /// **（ 1 request 3~4 ）， not is up （ can 1 ）。
    ///
    /// ── use when in `inner` ─────────────────────────────
    /// ： this 「 → → 」 not will 1 up ，
    /// is ， not is in 。 is 
    /// 1 UPDATE（ in that ）， and 1 。
    fn flush_live(&self, snapshot: &TelemetrySnapshot) {
        // and ：、、 new must is 1 ，otherwise
        // up can can 「」 1 （none， no need ）。`live` this in use 
        // ** before ** —— is 「 need not need 」， etc not this 
        // （`inner` by use has ， up 「 use when 」 that ）。
        let sink = {
            let mut live = self
                .live
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(sink) = live.sink.clone() else {
                return;
            };
            let next = LiveStamp::of(snapshot);
            if next == live.stamp {
                return;
            }
            live.stamp = next;
            sink
        };
        sink(snapshot);
    }

    /// （ use is **none**）。
    ///
    /// ── Why not when ──────────────────────────────────
    /// when is 「 in this 」， 1 up 
    /// （e.g.path will to `note_first_frame`） not can ——
    /// otherwise「response in 」 when will in 0 seconds。 has ** **
    /// update， this is and OmniProxy `updateLogPhase` 1 ：
    /// that use `phase_started_at`， use 「 in when 」，
    /// this this not in up 。
    ///
    /// use must has `inner` （ `note_*` `guard`）—— and 
    /// `flush_live` 1 ： and 「 need not need 」 is 1 。
    fn enter_phase(&self, guard: &mut TelemetrySnapshot, phase: LogPhase) {
        if guard.phase == phase {
            return;
        }
        guard.phase = phase;
        guard.phase_started_at = Some(logging::now_ms());
    }

    /// to `phase` and （** to when use **）。
    ///
    /// and `note_*` that `enter_phase` ： that in 
    /// 「 already 」 up （ to 、、）， is to 
    /// 「 will 」 use —— before 1 use is `send_with_retry` 
    /// **request before ** that 1 down ， ： that 1 
    /// failure after is 「 in 」（ etc ）， when must to 「 etc response」，
    /// otherwise will 1 in 「 in 」 to to 。
    pub fn note_phase(&self, phase: LogPhase) {
        let mut guard = self.lock();
        self.enter_phase(&mut guard, phase);
        self.flush_live(&guard);
    }

    /// 1 「 already to this up request」（ 1 ）。
    ///
    /// ** after 1 **：429 after request is after that ，
    /// 「 this request in up 」 this 。
    /// attempts is —— need is 「 this request 」。
    ///
    /// `provider` is ** after 1 **（ after request that ）。
    pub fn note_attempt(&self, account_id: Option<&str>, account_name: &str, provider: &str) {
        let mut guard = self.lock();
        guard.attempts += 1;
        guard.account_id = account_id.unwrap_or("").to_string();
        guard.account_name = account_name.to_string();
        if !provider.is_empty() {
            guard.provider = Some(provider.to_string());
        }
        // in ： 1 ， in this 「 in 」
        self.flush_live(&guard);
    }

    pub fn note_quota_channel(&self, account: Option<&Value>) {
        self.lock().quota_channel = account.map(crate::server::core::limiter::counter::channel_of).unwrap_or_default();
    }

    /// to ** after 1 ** up 。
    ///
    /// and [`Self::note_attempt_started`] for ： that 「」， has provider
    /// in `Vec<u8>` when （ will body：
    /// `max_tokens`、 will id）， by provider ， not is from `Value` 。
    ///
    /// no has can （limit / no ） when —— this is ，
    /// not this 。
    pub fn note_attempt_body_bytes(&self, bytes: i64) {
        let mut guard = self.lock();
        if let Some(last) = guard.current_attempt_mut() {
            last.body_bytes = Some(bytes.max(0));
            self.flush_live(&guard);
        }
    }

    /// to ** after 1 ** 1 （`send_with_retry` 
    /// 1 1 ）。
    ///
    /// empty `reason` ： 1 「， not Why」，
    /// for no has ， will 1 。
    pub fn note_attempt_retry(&self, reason: &str, status: Option<i64>, delay_ms: u64) {
        let debug = self.capture().is_some();
        let reason = reason.trim();
        if reason.is_empty() {
            return;
        }
        let mut guard = self.lock();
        let Some(last) = guard.current_attempt_mut() else {
            return;
        };
        last.retries.push(RetryEvent {
            reason: if debug { reason.to_string() } else { truncate_chars(reason, MAX_ATTEMPT_RETRY_REASON_CHARS) },
            status,
            delay_ms,
        });
        // ： —— down is 1 when ， this etc is 「 in 」
        // （ down 1 `note_attempt_started` will 「 etc response」）
        self.enter_phase(&mut guard, LogPhase::Retrying);
        // in ： is 「 in 」 to 
        // （ before that ， `hasProcessFacts`）
        self.flush_live(&guard);
    }

    /// up in ： 1 「 etc 」 and to 「 in 」。
    ///
    /// ── Why not is `note_attempt_retry` ─────────────────────────
    /// that is 「 in 」。 not is failure， is ** etc up **： need 
    /// 「 in 」， use this 、、none
    /// （ this ）。 —— 「」
    /// and is ， this in that down 1 。
    pub fn note_queued(
        &self,
        reason: &str,
        status: Option<i64>,
        delay_ms: u64,
        wait_number: usize,
        max_waits: usize,
    ) {
        let debug = self.capture().is_some();
        let reason = reason.trim();
        if reason.is_empty() {
            return;
        }
        let mut guard = self.lock();
        let Some(last) = guard.current_attempt_mut() else {
            return;
        };
        // and totalReason： that 1 ， use can 
        // 「 this is etc 、limit」
        let label = if max_waits > 0 {
            format!("{reason}（第 {wait_number}/{max_waits} 次）")
        } else {
            reason.to_string()
        };
        last.retries.push(RetryEvent {
            reason: if debug { label } else { truncate_chars(&label, MAX_ATTEMPT_RETRY_REASON_CHARS) },
            status,
            delay_ms,
        });
        self.enter_phase(&mut guard, LogPhase::Queued);
        self.flush_live(&guard);
    }

    /// to ** after 1 ** 1 （ before has ）。
    ///
    /// ** as **（ and `note_error` 1 to ）： 1 will has 1 ，
    /// after （e.g. when 1 ） not this to that 。
    /// empty not ， by [`Self::note_attempt_retry`]。
    pub fn note_attempt_notice(&self, notice: &str) {
        let debug = self.capture().is_some();
        let notice = notice.trim();
        if notice.is_empty() {
            return;
        }
        let mut guard = self.lock();
        let Some(last) = guard.current_attempt_mut() else {
            return;
        };
        if last.notice.is_none() {
            last.notice = Some(if debug { notice.to_string() } else { truncate_chars(notice, MAX_ATTEMPT_NOTICE_CHARS) });
        }
        // in ： is 「 this 1 」 1 
        self.flush_live(&guard);
    }

    /// to ** after 1 ** up （success need ， when `status` has 、
    /// `error` empty）。
    ///
    /// Why after 1 ： 1 is （`send_with_retry`
    /// + 401 new in 1 ），response when 「 after that 」
    /// is —— in not will has 。 this by `provider_loop`
    /// `'accounts` （ 1 ， down 1 ）。
    ///
    /// `status` as None when （failure）** not ** already has None，`error` ：
    /// is that 1 「A → failure（none）： 」，
    /// not is 1 empty 。
    ///
    /// requesterror，request by need limit。
    pub fn finish_last_attempt(&self, status: Option<i64>, error: Option<&str>) {
        let debug = self.capture().is_some();
        let mut guard = self.lock();
        let Some(last) = guard.current_attempt_mut() else {
            return;
        };
        if status.is_some() {
            last.status = status;
        }
        if last.response_at.is_none() && error.is_none()
            && status.is_none_or(|status| (200..300).contains(&status))
        {
            last.response_at = Some(logging::now_ms());
        }
        // empty need by 「 no has error」（ and RequestEntry.error 1 1 ）：
        // up returnempty body when read_upstream_error will to empty message，
        // that not this 「failure：」 after 1 empty
        if let Some(text) = error.filter(|text| !text.is_empty()) {
            last.error = Some(if debug { text.to_string() } else { truncate_chars(text, MAX_ATTEMPT_ERROR_CHARS) });
        }
        // ： this 1 **failure** → in （ / before ）。
        // its after by down 1 or ： `note_attempt_started`
        // （ etc response）、 `note_attempt_retry`（ in in ）、
        // requestfailure then empty（ no has ）。
        // success（`error` as empty） not ： that 1 success is this request ，
        // will to ， in no has can 。
        if last.error.is_some() || status.is_some_and(|status| !(200..300).contains(&status)) {
            last.finished_at = Some(logging::now_ms());
        }
        if last.error.is_some() {
            self.enter_phase(&mut guard, LogPhase::Retrying);
        }
        // in ： this 1 （success / failure） is 「path」 up to 1 
        self.flush_live(&guard);
    }

    /// 「 this 1 to up 」（， after 1 as ）。
    ///
    /// and `note_attempt` provider 1 ：429 after ， is after 1 
    /// 。empty not （ no has 「empty」 —— is empty，
    /// empty can can use path， that path not this already to ）。
    pub fn note_upstream_model(&self, model: &str) {
        if model.is_empty() {
            return;
        }
        let mut guard = self.lock();
        guard.upstream_model = model.to_string();
        if let Some(last) = guard.current_attempt_mut().filter(|last| last.finished_at.is_none()) { last.model = model.to_string(); }
        // in ： up is that 1 ，request
        // —— can in 「⬆️ up / ⬇️ down 」
        self.flush_live(&guard);
    }

    /// 「 this 1 request to up etc 」（， after 1 as ）。
    ///
    /// and ；None no has etc ， up 1 etc 。
    pub fn note_upstream_reasoning(&self, level: Option<String>) {
        let level = level.filter(|text| !text.is_empty()).unwrap_or_default();
        let mut guard = self.lock();
        guard.upstream_reasoning = level.clone();
        if let Some(last) = guard.current_attempt_mut().filter(|last| last.finished_at.is_none()) { last.reasoning = level; }
        // in ： and 1 when ， in 1 and etc 
        self.flush_live(&guard);
    }

    /// up 1 usage（， after 1 as ； `extract_usage`）。
    ///
    /// ** not in **： in in before not usage（`usageCell` for in to empty），
    /// usage in up after 1 chunk —— will 。
    pub fn report_usage(&self, usage: &Value) {
        let Some(tokens) = extract_usage(usage) else {
            return;
        };
        let mut guard = self.lock();
        guard.prompt_tokens = tokens.prompt;
        guard.completion_tokens = tokens.completion;
        guard.total_tokens = tokens.total;
        guard.cache_read_tokens = tokens.cache_read;
        guard.credit = tokens.credit;
    }

    /// 1 「 up already to 」。
    ///
    /// ** as **（ and note_error 、 and report_usage ）： has 1 ，
    /// will 「 1 when to 」「 after use when 」。
    /// in response up （Streaming RecordingStream / 1 chunk），
    /// 1 up will use ， etc by this None 。
    ///
    /// ── Why not 「 as 」limit ──────────────────────────
    /// has 1 （ that 1 ）， is **current**： after 
    /// new 1 need to 「response in 」。 in if **** ——
    /// and after this （`enter_phase` for is empty，
    /// path use not will ）。
    pub fn note_first_frame(&self) {
        let mut guard = self.lock();
        if guard.first_response_at.is_none() {
            guard.first_response_at = Some(logging::now_ms());
        }
        // ： to → response in 
        self.enter_phase(&mut guard, LogPhase::Streaming);
        // in ： in （ 1 minutes request， its 
        // 1 seconds has ）。** etc **， to this will 1 ——
        // is （`flush_live`）， not is this if
        self.flush_live(&guard);
    }

    /// 1 in / Reason（successrequest not will use ）。
    ///
    /// ** as **（ and usage 「 after 1 as 」）： to that is ，
    /// after to is （e.g. after client 1 ），
    /// 。
    ///
    /// ** not in **：error 1 has ， before not this 1 「 in 」
    /// （ `ui/requests-panel.js` `isRunning`）， in failure is 
    /// before 1 failure —— that failure by ， is 「 in 」。
    pub fn note_error(&self, message: &str) {
        if message.is_empty() {
            return;
        }
        let mut guard = self.lock();
        if guard.error.is_none() {
            guard.error = Some(message.to_string());
        }
    }

    /// 1 in （** and **， `TelemetrySnapshot::sensitive_hits`）。
    ///
    /// is `term_counts` （`&[(String, usize)]`）——
    /// not in this new or ： that already is 「 by 」 （`engine::Counter`
    /// ）， 1 will 。 before need is 。
    ///
    /// 1 （ when 1 in ） when ** and ** not is 
    /// ： 1 `[{word, count}]` 1 ， no has 
    /// will ， will 「 in 3 」 this 「 2 」。
    ///
    /// up ： then is （7 + 3 then + 5 ，
    /// `core::sanitize`）， before that can ， in 
    /// has 。 this not —— in is JSON ，
    /// before when will by （ requests-panel ）。
    pub fn note_sensitive_hits(&self, term_counts: &[(String, usize)]) {
        if term_counts.is_empty() {
            return;
        }
        let mut guard = self.lock();
        for (term, count) in term_counts {
            let count = *count as i64;
            match guard
                .sensitive_hits
                .iter_mut()
                .find(|hit| hit.word == *term)
            {
                Some(hit) => hit.count += count,
                None => guard.sensitive_hits.push(SensitiveHit {
                    word: term.clone(),
                    count,
                }),
            }
        }
        // in ： in in that 1 has ， not etc 
        // （「」 that can in ）
        self.flush_live(&guard);
    }

    /// current（ when 1 ）
    pub fn snapshot(&self) -> TelemetrySnapshot {
        self.lock().clone()
    }

    /// ； in when use —— statistics not 「 not statistics」
    /// need （ and RequestStats / LogStore 1 to ）
    fn lock(&self) -> MutexGuard<'_, TelemetrySnapshot> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
mod usage_extraction {
    //! usage （ and 「 1 」 by module header）。

    use serde_json::json;

    use super::*;

    /// must to down 1 —— ** old in this up will **。
    ///
    /// 1 ： `prompt_tokens_details.cached_tokens` 0、 in 
    /// `cache_read_input_tokens` up 。 old is `.and_then().or_else()`，
    /// 1 in ， is request usage 0； 0 not is 「」 is 
    /// 「 this no 」 （ in ）， 1 。
    #[test]
    fn a_zero_nested_value_yields_to_the_real_top_level_alias() {
        let usage = json!({
            "prompt_tokens": 1200,
            "completion_tokens": 300,
            "total_tokens": 1500,
            "prompt_tokens_details": { "cached_tokens": 0 },
            "cache_read_input_tokens": 4800,
        });
        let tokens = extract_usage(&usage).expect("对象就该出值");
        assert_eq!(4800, tokens.cache_read, "嵌套那份是废值，不该当「命中 0」用掉");
        assert_eq!(1200, tokens.prompt, "其余三项照常");
    }

    /// and need （`"34"`、`12.0`）。
    ///
    /// old `number_field` `as_i64` → `as_f64`， 1 None； up usage
    /// is has （`protocol::commandcode_outbound` 1 before is 
    /// that /）。 this if 0， will 「 1 no request」。
    #[test]
    fn string_and_float_shapes_are_read() {
        let usage = json!({
            "prompt_tokens": "34",
            "completion_tokens": 12.0,
            "input_tokens_details": { "cached_tokens": "7" },
        });
        let tokens = extract_usage(&usage).expect("对象就该出值");
        assert_eq!(34, tokens.prompt, "字符串数字要落数");
        assert_eq!(12, tokens.completion, "12.0 这类浮点截断成 12");
        assert_eq!(46, tokens.total, "total 缺值时按 prompt + completion 补");
        assert_eq!(7, tokens.cache_read, "驼峰/浮点之外的容器键也走同一张别名表");
    }

    /// （0 and ）： not ， not 。
    ///
    /// has ：`completion_tokens: -5` not will down ；
    /// `total_tokens: 0` 0 not prompt + completion ；
    /// `cache_read_tokens: -12` not 「 in 」。 if 「 0 has 」，
    /// after （ before 0）， this 「0 not 」 1 。
    #[test]
    fn non_positive_values_do_not_speak_and_do_not_mask() {
        let usage = json!({
            "prompt_tokens": 90,
            "completion_tokens": -5,
            "total_tokens": 0,
            "cache_read_tokens": -12,
        });
        let tokens = extract_usage(&usage).expect("对象就该出值");
        assert_eq!(0, tokens.completion, "负的输出去落 0，别把负数带进聚合");
        assert_eq!(90, tokens.total, "total 为 0 是废值，回落 prompt + completion");
        assert_eq!(0, tokens.cache_read, "负的命中同样是废值");

        // must： 1 1 has when ， to is not is 0
        let masked = json!({
            "prompt_tokens": 90,
            "completion_tokens": 10,
            "cache_read_tokens": -12,
            "cached_tokens": 33,
        });
        assert_eq!(33, extract_usage(&masked).unwrap().cache_read, "废值不许扮演「有值」");
    }

    /// empty：** is ** when by 1 ， not is and not is 。
    ///
    /// if 「 all」 or 「 after before 」， before use 
    /// （ is 0 or ）， has this can 。
    /// OpenAI `cached_tokens` in Anthropic `cache_read_tokens` before ，
    /// this 1 is before —— is ， this need is 「 1 」。
    #[test]
    fn two_positive_aliases_do_not_add_up() {
        let usage = json!({
            "prompt_tokens": 100,
            "input_tokens": 999,
            "cache_read_tokens": 40,
            "cached_tokens": 77,
        });
        let tokens = extract_usage(&usage).expect("对象就该出值");
        assert_eq!(100, tokens.prompt, "标准写法在前，别把别名加上去");
        assert_eq!(77, tokens.cache_read, "按表序取第一项正的，不相加");
    }

    /// and 1 ； not will 。
    ///
    /// `totalTokens` and `inputTokens + outputTokens` ** not etc ** ：
    /// if ， this not 「 to total」 and 「 prompt+completion 」，
    /// no 。
    #[test]
    fn camel_case_aliases_and_container_precedence() {
        let usage = json!({
            "inputTokens": 500,
            "outputTokens": 20,
            "totalTokens": 610,
            "input_tokens_details": { "cached_tokens": 0 },
            "prompt_tokens_details": { "cacheReadTokens": 300 },
        });
        let tokens = extract_usage(&usage).expect("对象就该出值");
        assert_eq!(500, tokens.prompt, "驼峰的 inputTokens 就是 prompt");
        assert_eq!(20, tokens.completion);
        assert_eq!(610, tokens.total, "读的是 totalTokens，不是兜底加出来的 520");
        assert_eq!(300, tokens.cache_read, "容器内同样按「第一个正数」挑");
    }

    /// for is 1 not （None by use up ）。
    #[test]
    fn a_non_object_usage_reports_nothing() {
        assert!(extract_usage(&json!(null)).is_none());
        assert!(extract_usage(&json!("34")).is_none());
        assert!(extract_usage(&json!([1, 2])).is_none());
        // empty for is 「 for 1 no has 」：、 0， and None in 
        // use not will before that 1 already to （report_usage in Some when ）
        assert_eq!(
            Some(UsageTokens::default()),
            extract_usage(&json!({})),
            "空对象也要出值：None 与「全 0」在调用侧是两条不同分支"
        );
    }
}

#[cfg(test)]
mod body_bytes_observation {
    //! this （ `request_stats::record` and `api::pipeline`）。

    use super::*;

    /// ** after 1 **； no when not empty 1 。
    ///
    /// after is empty： if this in as empty when 1 ， or 「 has 1 can to 」，
    /// before 。path request no has ，must that 1 。
    #[test]
    fn the_body_bytes_observation_lands_on_the_latest_attempt() {
        let telemetry = RequestTelemetry::new();
        telemetry.note_attempt_body_bytes(123);
        assert!(
            telemetry.snapshot().attempts_detail.is_empty(),
            "没有起头时不该凭空多一条明细（本地就拒的请求根本没有发出量）"
        );
        telemetry.note_attempt_started("codearts", "GT-a");
        telemetry.note_attempt_body_bytes(6_372_244);
        telemetry.note_attempt_started("workbuddy", "wb-1");
        telemetry.note_attempt_body_bytes(1_024);
        let details = telemetry.snapshot().attempts_detail;
        assert_eq!(2, details.len());
        assert_eq!(
            Some(6_372_244),
            details[0].body_bytes,
            "第一轮记下的是第一轮真实发出去的字节（串行就等于假归因）"
        );
        assert_eq!(Some(1_024), details[1].body_bytes);
        // can can ： 0， not as 1 
        telemetry.note_attempt_body_bytes(-5);
        assert_eq!(Some(0), telemetry.snapshot().attempts_detail[1].body_bytes);
    }
}
