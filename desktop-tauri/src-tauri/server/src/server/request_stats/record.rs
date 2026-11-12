//! requeststatistics and JSON （ 1 ）。
//!
//! ── Why `#[serde(rename)]` ─────────────────────
//! before statistics and 「request」 this ，** is **。
//! this not use `rename_all = "camelCase"` is ： when 
//! diff `rename` that 1 ， not will as 「 rename 」
//! up 。
//!
//! ── `RequestEntry` use ， no ──────────
//! before JSONL 1 ； in `requests` ****
//! （ 1 ， `db/schema.rs`）， use ：
//! - `report::entry_json` `/api/stats/requests` response
//! （ before is this ， `rename` 1 not can ）；
//! - ** old file**（`parse_requests_jsonl`）。
//! ： in is 「API + old 」， not is 。
//!
//! ── Why `default` ───────────────────────────
//! need ， in ：
//! - ** old file**（）： old file can can ，
//! new old must can （ by 0 / empty），otherwise 1 
//! will use already has ；
//! - ** JSON **（`model_tokens` / `provider_stats` /
//! `account_stats`）：， can can ，
//! `default` 「 1 」 not failure。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::source::{ApiKeyAccum, ClientAccum, RequestSource, SOURCE_STATS_VERSION};

/// `request_raw` limit；，only old 。
pub const MAX_RAW_ROWS: usize = 1000;

/// default and limit。
/// `RequestStats::query_requests`（ by `/api/stats/requests`）
/// and `api::stats_api` `limit` use 。
pub const DEFAULT_LIMIT: usize = 50;
pub const MAX_LIMIT: usize = 500;

/// 1 request。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestEntry {
    /// request when （ms Unix when ）
    pub ts: i64,
    /// request id（ by ， `core::debug_traffic`）。
    ///
    /// old no has this （ before ），`default` empty ——
    /// before not 「」。
    #[serde(default)]
    pub id: String,
    pub model: String,
    #[serde(rename = "accountId", default)]
    pub account_id: String,
    #[serde(rename = "accountName", default)]
    pub account_name: String,
    /// HTTP ； no requestfailure when use 0
    #[serde(default)]
    pub status: i64,
    #[serde(rename = "durationMs", default)]
    pub duration_ms: i64,
    /// up to for request when （ms）。
    ///
    /// and OmniProxy request `ttfb_ms` ： 「 etc up 」 and 「
    /// 」 when —— has durationMs when ， 1 30 seconds request
    /// not is up is 。
    ///
    /// ── Why is `Option`（null） not is 0 ─────────────────────────
    /// 「 no to 」 and 「 to 0ms」 is 。None in 「 no has 
    /// to 」 request up （ before failure）； in failurerequest**** ——
    /// to ， and 「 this is not is failure」none（ is when ，
    /// not is ， and 「failure token」 not ）。
    /// old no has this ，`default` None， before 「-」。
    #[serde(rename = "firstResponseMs", default)]
    pub first_response_ms: Option<i64>,
    /// （）， ≥1
    #[serde(default = "one")]
    pub attempts: i64,
    /// error need ，success as null
    #[serde(default)]
    pub error: Option<String>,
    #[serde(rename = "promptTokens", default)]
    pub prompt_tokens: i64,
    #[serde(rename = "completionTokens", default)]
    pub completion_tokens: i64,
    #[serde(rename = "totalTokens", default)]
    pub total_tokens: i64,
    #[serde(rename = "cacheReadTokens", default)]
    pub cache_read_tokens: i64,
    /// up by request up ****（e.g. WorkBuddy `usage.credit`）。
    ///
    /// and token ： this is **** not is ， can as 
    /// （ WorkBuddy in 0.03～0.31 ）， can empty REAL。
    /// `None` = this 1 no up （ old 、 not this ， or failure request），
    /// and 「 up 0」 is —— before for before 「—」， for after 「0」。
    #[serde(rename = "credit", default)]
    pub credit: Option<f64>,
    /// request provider id（ §3.6）。
    ///
    /// ── Why is `String` + empty not is `Option<String>` ─────────
    /// and `accountId` / `accountName` 1 「empty is no has 」 ：
    /// 「 old no has this 」「 1 no （ before failure）」
    /// this in up is 「 not is 」， 1 ，
    /// down （ by provider 、API 、 before ） not 。
    /// empty in 「unknown」，「—」，** not to old **
    /// （ old up when not workbuddy： that is ， not is ）。
    ///
    /// ── Why no has `rename` `default` ────────────────────
    /// is `provider`， and （file `rename` in 
    /// not 1 up ）。`default` old ；empty** to **
    /// （ and `accountId` / `accountName` 1 ： in 、empty no has ），
    /// is 「 has no has this 」 not is before need 。
    #[serde(default)]
    pub provider: String,
    /// ** down request **（clientrequest ， / default before ；
    /// empty = client no ， or this no has old ）。
    ///
    /// request use and `upstreamModel` 「⬆️ / ⬇️ request 
    /// 」。** not need ** and `model` use ：`model` is request （default
    /// already ； not —— after down to by ，
    /// `pipeline::resolve_model` ），client when is 。
    #[serde(rename = "clientModel", default)]
    pub client_model: String,
    /// to up （ + by after ；empty =
    /// 1 no failure， or this no has old ）。
    ///
    /// and `model` in 「」 when ： down request `gpt-4o` to 
    /// `deepseek-v4-pro`、 or request to when ，`model` request，
    /// this up to 、 。
    ///
    /// ** by as statistics**（empty `model`，
    /// `fold_into_daily` `model_stat_key`）： is ，request is 
    /// up that 1 ，usage need in down —— otherwise 1 up will in 
    /// 「usage」「 + 」。
    #[serde(rename = "upstreamModel", default)]
    pub upstream_model: String,
    /// ** down request**client etc （ `max`；empty = no ，
    /// or this no has old ）。 and 1 then 
    /// `core::model_rules::reasoning::read_client_level`。
    ///
    /// request use to down up `( etc )` after ； and `upstream_reasoning`
    /// ， is as 「client need 」 and 「 to up 」 is 
    /// （client no when can 1 ； can can and ）。
    #[serde(rename = "clientReasoning", default)]
    pub client_reasoning: String,
    /// ** up request** etc （empty = no has etc ：client no and 
    /// no 、「」、 or 1 no failure； or this no has 
    /// old ）。client etc up （ not 
    /// client）， for 1 「request etc 」。
    /// `core::upstream::payload::send_body` —— and `upstream_model` when 。
    #[serde(rename = "upstreamReasoning", default)]
    pub upstream_reasoning: String,
    /// ** 1 up **（`[{provider, status, error}]`， by ）。
    ///
    /// ── and `attempts` （ this is in by ）────────────
    /// `attempts` 「」，`provider` 「 after is 」——
    /// in that 、Why， in before ** no has **，request 
    /// 「」 can 1 no has 「」。 is that 。
    ///
    /// limit： `attempt_details.len() <= attempts`， `attempts`
    /// up is `MAX_ROUTE_ATTEMPTS + 1`（32+1）。 has 
    /// `MAX_ATTEMPT_DETAILS`（24） —— when will `attempts` ，
    /// before 「 before N 」（ requests-panel）。
    ///
    /// ── Why JSON not is ──────────────────────────
    /// is 「 1 」，****（ when to 、 when ），
    /// no has by need 。 1 `request_attempts` need 
    /// 、delete and join， is 「 can SQL 1 」——
    /// that is ，`sqlite3` for JSON use `json_each` 1 can 。
    /// and `request_daily` `*Stats` JSON 1 （ schema.rs）。
    ///
    /// ── old ──────────────────────────────────────────────
    /// `default` empty： old no has this ，empty is 「 not 」， and 「 1 no 」
    /// not is 1 ， for （ as empty when this in has in 
    /// when has ）。 and `provider` empty 1 ：** not to old **。
    #[serde(rename = "attemptDetails", default)]
    pub attempt_details: Vec<AttemptDetail>,
    /// ****（`{strategy, sticky, stickyHit, picked, ordered[], blocked[]}`）。
    ///
    /// 「 by 」 ：request by 「 when Why this 」—— and 
    /// `attemptDetails` ， that 「、 no 」， this 「
    /// 、Why is up 」。 in （`rotate::select_target_account`
    /// 、telemetry 、）， and 1 「 has has 」。
    ///
    /// old no has this ，`default` Null； for Null and 1 
    /// （ not that 1 ）。 JSON and attemptDetails 1 （， not ）。
    #[serde(rename = "route", default)]
    pub route: Value,
    /// **request in **（`[{word, count}]`， by ；empty = no in ）。
    ///
    /// and `core::upstream::usage::SensitiveHit` ****（`word` /
    /// `count`）： that is ， this is 。 not 、
    /// —— `request_stats` is ， not this as 1 
    /// `core::upstream` when （ that this will 1 and none 
    /// ， will will ）。
    ///
    /// ：request「」 （ in ）， and OmniProxy
    /// `SensitiveMaskedTag` 。 in ** not in **「」 —— request
    /// has this after ， that （ use ）
    /// 。
    ///
    /// ── `word` is （ then ）───────────────────
    /// before is 「 in 」（ can ）； in is ** in 
    /// then **（ `11128` / `cc_*=` / ， `core::sanitize`）。
    /// and 1 ， before and 。
    ///
    /// need 「 up as this request」， + 
    /// （request 「」） that 。
    #[serde(rename = "sensitiveHits", default)]
    pub sensitive_hits: Vec<SensitiveHit>,
    /// ** in requestcurrent **（`connecting` / `waiting` / `streaming` /
    /// `retrying` / `queued`；empty = not in —— 、 old 、 before failure ）。
    ///
    /// is `core::upstream::usage::LogPhase::as_str()` ， and OmniProxy
    /// `LogPhase` ； before by （ in / etc response / response in /
    /// in ） and 。
    ///
    /// ── Why and status=0 ──────────────────────────────
    /// in in ：（success、failure、、） 1 
    /// empty。「 has no has 」 can 「 this 1 in 」 ，
    /// is `status` that 1 （ before `isRunning`）—— is ****， not is ：
    /// 1 「status=0 as empty」 （ ），
    /// use 「 in 」， not will 。
    ///
    /// ── Why in ──────────────────────────────────────
    /// and `REQUEST_COLUMNS` / `decode_request` 1 ： this is schema v6
    /// ， has in after not before all `row.get(n)` （ that 
    /// sql.rs ）。
    #[serde(default)]
    pub phase: String,
    /// **current** when （ms when ， and `ts` 1 ；None = not in ）。
    ///
    /// not ：`/api/stats/requests` response is ****
    /// `phaseElapsedMs`（「 in - this 」， `report::entry_json`，
    /// and `providerLabel` 1 ）—— when and server when ，
    /// when not will 。
    #[serde(rename = "phaseStartedAt", default)]
    pub phase_started_at: Option<i64>,
    /// ** this request is not is **（schema v7 ； old `default` 
    /// false = ）。
    ///
    /// ── Why not is 「」 ─────────────────────────────────
    /// request and request is 1 ， 1 （、、
    /// 、usage） and —— 1 is 「 」， that has 
    /// （`api::model_test`），must as 1 down 。
    ///
    /// use has ， and to ：**excluding**（ 
    /// will ），request**** and to 1 「」
    /// （ need can for 、 need can ）。 not this this 1 —— its not need use 
    /// or ： is 1 request ， not is or 。
    ///
    /// and `phase` 1 ： new in ， not before all
    /// `row.get(n)` 。
    #[serde(rename = "isTest", default)]
    pub is_test: bool,
    /// request when statistics and ； old no has ， not new 。
    #[serde(flatten)]
    pub source: RequestSource,
}

/// 1 in and its （）。
///
/// and `core::upstream::usage::SensitiveHit` ， by 
/// `RequestEntry::sensitive_hits`。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SensitiveHit {
    /// in 
    #[serde(default)]
    pub word: String,
    /// request in 
    #[serde(default)]
    pub count: i64,
}

/// up （）。
///
/// and `core::upstream::usage::AttemptDetail` ， by up 。
/// `provider` empty「 this 1 no down 」—— empty
/// （`note_attempt_started` is provider_id），empty can can 
/// ， to 。
///
/// ── `account` / `retries` / `notice` is schema v3 before has ──────
/// not is ： and `account` 1 in 「request」 this ，
/// `attemptDetails` that **JSON **，** not need need schema**
/// （ in ， is JSON ）。 old JSON no has this ，
/// `default` empty / empty / None， 1 not that 。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttemptDetail {
    /// this 1 provider id
    #[serde(default)]
    pub provider: String,
    /// this 1 （empty = this no has ）
    #[serde(default)]
    pub account: String,
    #[serde(rename = "accountId", default)]
    pub account_id: String,
    #[serde(rename = "candidateId", default)]
    pub candidate_id: String,
    #[serde(rename = "memberId", default)]
    pub member_id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub reasoning: String,
    #[serde(default)]
    pub reason: String,
    #[serde(rename = "stickyHit", default)]
    pub sticky_hit: bool,
    #[serde(rename = "startedAt", default)]
    pub started_at: Option<i64>,
    #[serde(rename = "responseAt", default)]
    pub response_at: Option<i64>,
    #[serde(rename = "finishedAt", default)]
    pub finished_at: Option<i64>,
    /// this 1 HTTP （None = or failure， ）
    #[serde(default)]
    pub status: Option<i64>,
    /// this 1 failure need （success when as None）
    #[serde(default)]
    pub error: Option<String>,
    /// this 1 **** （empty = no ， ）
    #[serde(default)]
    pub retries: Vec<RetryEvent>,
    /// this 1 （ before has ）
    #[serde(default)]
    pub notice: Option<String>,
    /// this 1 to up （None = no ）。 by and 
    /// 「 not 」 `core::upstream::usage::AttemptDetail::body_bytes`。
    #[serde(rename = "bodyBytes", default)]
    pub body_bytes: Option<i64>,
}

/// 1 （， and ）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetryEvent {
    /// Reason
    #[serde(default)]
    pub reason: String,
    /// HTTP （None = failure）
    #[serde(default)]
    pub status: Option<i64>,
    /// when （ms）
    #[serde(rename = "delayMs", default)]
    pub delay_ms: u64,
}

/// `attempts` serde default（ this old when by 1 ）
fn one() -> i64 {
    1
}

impl RequestEntry {
    /// is success —— 2xx ** and ** no has error need 。
    ///
    /// ── Why not can ───────────────────────────────────
    /// request HTTP 200 in 「response already 」 when ， after up 、
    /// up error、failure in response（ Reason
    /// `RequestTelemetry::error`）。 by 2xx will this 1 requestsuccess，
    /// success and by provider success —— this is CatPaw has 
    /// 。 has error need = this request no has success。
    ///
    /// failure（4xx/5xx） not is 2xx， in ， not 。
    /// old no has error by None ，2xx success（ not ）。
    pub(super) fn is_success(&self) -> bool {
        (200..300).contains(&self.status) && self.error.is_none()
    }

    /// by **current** 1 after ：failurerequest token 1 。
    ///
    /// ── Why need need ────────────────────────────────────────────
    /// path use not to —— before already `NewRequestEntry::normalize`。
    /// ** from old file can can **：1.x `normalize` 
    /// ， no has 「failure」 this 1 ， that when failure request token
    /// （ old file has this ：`status: 200` + down need + token）。
    /// need this old ，must 1 ，
    /// otherwise will 「failureusage」 this old then token ——
    /// current is 「failure request not usage」。
    ///
    /// 1 use ：`backfill::rebuild_legacy_days`（ by **** in old when 
    /// use ）。 not has use —— `normalize`。
    ///
    /// etc ： already 1 1 not （failure that as 0）。
    pub(super) fn normalized_for_aggregate(&self) -> Self {
        if self.is_success() {
            return self.clone();
        }
        let mut copy = self.clone();
        copy.prompt_tokens = 0;
        copy.completion_tokens = 0;
        copy.total_tokens = 0;
        copy.cache_read_tokens = 0;
        copy.credit = None;
        copy
    }
}

/// `record` 。
///
/// use 1 parameter： in ， will 
/// （error、 up Reason…）， not already has use 。
/// `ts` as None when current when ； will to ≥0，`attempts` to ≥1。
#[derive(Clone, Debug)]
pub struct NewRequestEntry {
    pub ts: Option<i64>,
    /// request id（ ；empty = not ）
    pub id: String,
    pub model: String,
    pub account_id: String,
    pub account_name: String,
    pub status: i64,
    pub duration_ms: i64,
    /// up to for request when （ for when ，；
    /// `None` = no has to ）。 `RequestEntry::first_response_ms`。
    pub first_response_ms: Option<i64>,
    pub attempts: i64,
    pub error: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cache_read_tokens: i64,
    /// up by request up （ can empty：`None` = no up ， not is 0；
    /// `RequestEntry::credit`）
    pub credit: Option<f64>,
    /// request provider id（Agent2API W2b-T3 add，
    /// W4 up ： `normalize` ）。
    ///
    /// `None` = 1 no failure（request / not in / noneavailable），
    /// when 1 empty，「unknown」。
    pub provider: Option<String>,
    /// down request （client；empty = / ）
    pub client_model: String,
    /// to up （empty = 1 no / ）
    pub upstream_model: String,
    /// down request etc （empty = no ； `RequestEntry::client_reasoning`）
    pub client_reasoning: String,
    /// up request etc （empty = no has etc ；
    /// `RequestEntry::upstream_reasoning`）
    pub upstream_reasoning: String,
    /// 1 up （ `RequestEntry::attempt_details`）。
    /// empty = 1 no （ before failure） or no up 。
    pub attempt_details: Vec<AttemptDetail>,
    /// （ `RequestEntry::route`）。Null = no to 。
    pub route: Value,
    /// request in （ `RequestEntry::sensitive_hits`）。empty = no in 。
    pub sensitive_hits: Vec<SensitiveHit>,
    /// this request is not is （ `RequestEntry::is_test`；default false）。
    pub is_test: bool,
    /// requestclient（ by `User-Agent` after ， `RequestEntry::client`；
    /// not path as empty）。 no has down UA，empty —— 「 」
    /// that already by `is_test` ， this not 1 `test` 。
    pub source: RequestSource,
}

impl NewRequestEntry {
    /// use ， its ——
    /// can can has /，token need etc up response after has 。
    pub fn new(model: impl Into<String>, status: i64) -> Self {
        Self {
            ts: None,
            model: model.into(),
            account_id: String::new(),
            account_name: String::new(),
            status,
            duration_ms: 0,
            first_response_ms: None,
            attempts: 1,
            error: None,
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            cache_read_tokens: 0,
            credit: None,
            provider: None,
            client_model: String::new(),
            upstream_model: String::new(),
            client_reasoning: String::new(),
            upstream_reasoning: String::new(),
            id: String::new(),
            // is 「 has has 」： before failure request `record_early_failure`，
            // that telemetry is empty ， is empty「 no /
            // no in 」—— not is 1 need need 1 None
            attempt_details: Vec::new(),
            // no to when Null（ and old 1 ， not ）
            route: Value::Null,
            sensitive_hits: Vec::new(),
            // default is ； that 1 by `api::model_test` true
            // （`RecordContext::is_test` 1 ）
            is_test: false,
            // client「 has has 」： to UA （ `RequestEntry::client`），
            // not to UA pathempty， before 
            source: RequestSource::default(),
        }
    }

    /// 1 can 。
    ///
    /// when and in in this ， is `record` 「 already 」 。
    ///
    /// **failurerequest token 1 **（ need ）： up returnerror when usage is 
    /// ； 1 errorresponse usage，「failure request
    /// token」 —— that token not will ， in will 
    /// 「failure」「usage」。
    pub(super) fn normalize(self) -> RequestEntry {
        let ts = self.ts.unwrap_or_else(super::clock::now_ms);
        // 「failure」= 2xx ** or **error need （ and `RequestEntry::is_success` 1 
        // ）：request HTTP 200 in response， after up /
        // error can `error` 。empty need by no has error（ down filter ）。
        let failed = !(200..300).contains(&self.status)
            || self.error.as_deref().is_some_and(|text| !text.is_empty());
        let token = |value: i64| if failed { 0 } else { value.max(0) };
        // and token 1 「failure」：failure request in not usage
        // （`usageCell` for failure to 「-」）， will this 1 and 
        // 「failurerequest not usage」 has 。 has / can can 
        // or up ， by 「 no up 」 not is 0。
        let credit = if failed {
            None
        } else {
            self.credit.filter(|value| value.is_finite() && *value >= 0.0)
        };
        // provider from to （W2b in up ）。
        // `None`（ no to failure） and empty（ up returnempty id） 1 
        // **empty**： in up is 「unknown」， 1 will 
        // and before 1 「None empty」。trim 1 down ， old file or 
        // empty 1 not 。
        let provider = self
            .provider
            .map(|id| id.trim().to_string())
            .filter(|id| !id.is_empty())
            .unwrap_or_default();
        RequestEntry {
            ts,
            // id （trim by provider）：empty = no / old 
            id: self.id.trim().to_string(),
            model: self.model,
            account_id: self.account_id,
            account_name: self.account_name,
            status: self.status,
            duration_ms: self.duration_ms.max(0),
            // ：（ when ） 0 in already ，
            // this 「 has no has 」，None（ no to ）
            first_response_ms: self.first_response_ms,
            attempts: self.attempts.max(1),
            error: self.error.filter(|text| !text.is_empty()),
            prompt_tokens: token(self.prompt_tokens),
            completion_tokens: token(self.completion_tokens),
            total_tokens: token(self.total_tokens),
            cache_read_tokens: token(self.cache_read_tokens),
            credit,
            provider,
            // （trim by and provider ：empty not this 1 
            // 「 not 」）；empty = no has / no has 
            client_model: self.client_model.trim().to_string(),
            upstream_model: self.upstream_model.trim().to_string(),
            // etc 1 ：trim after ，empty = no has etc 
            client_reasoning: self.client_reasoning.trim().to_string(),
            upstream_reasoning: self.upstream_reasoning.trim().to_string(),
            // （Null or for ， —— by ）
            route: self.route,
            // ── 1 （）───────────────────────
            // 「empty + to 」，（SQL false 、 before ）
            // to is can ：
            // · provider empty ****（「 this 1 no down is 」，
            // and before `provider` empty 1 ， need ）；
            // · error empty None（ and `error` 1 ：empty not is error）；
            // · count to ≥0（ can can ， not this 
            // as ）；word empty ****（ 1 no has in no has 
            // ， will 1 「× 3」）。
            attempt_details: self
                .attempt_details
                .into_iter()
                .map(|item| AttemptDetail {
                    provider: item.provider.trim().to_string(),
                    // trim by and provider （empty not this 1 
                    // 「 not 」）；empty = this no has 
                    account: item.account.trim().to_string(),
                    account_id: item.account_id.trim().to_string(),
                    candidate_id: item.candidate_id,
                    member_id: item.member_id,
                    model: item.model.trim().to_string(),
                    reasoning: item.reasoning.trim().to_string(),
                    reason: item.reason,
                    sticky_hit: item.sticky_hit,
                    started_at: item.started_at,
                    response_at: item.response_at,
                    finished_at: item.finished_at,
                    status: item.status,
                    error: item.error.filter(|text| !text.is_empty()),
                    // ：reason as empty （ word empty ——
                    // 「 not Why」 for no has ， will 1 ），
                    // delay_ms （ ） 0
                    retries: item
                        .retries
                        .into_iter()
                        .filter(|retry| !retry.reason.trim().is_empty())
                        .map(|retry| RetryEvent {
                            reason: retry.reason.trim().to_string(),
                            status: retry.status,
                            delay_ms: retry.delay_ms,
                        })
                        .collect(),
                    // notice empty None， by error
                    notice: item.notice.filter(|text| !text.is_empty()),
                    // （None = this 1 no ； by usage ）
                    body_bytes: item.body_bytes,
                })
                .collect(),
            sensitive_hits: self
                .sensitive_hits
                .into_iter()
                .map(|hit| SensitiveHit {
                    word: hit.word.trim().to_string(),
                    count: hit.count.max(0),
                })
                .filter(|hit| !hit.word.is_empty())
                .collect(),
            // as empty： is ** in **（ has 
            // `LogPhase` ）， 1 already no has 「current」——
            // not to ， not as 「」
            phase: String::new(),
            phase_started_at: None,
            // （ not ）： is use （ handler）
            // ， 
            is_test: self.is_test,
            // in ， when not set new 。
            source: self.source.normalized(),
        }
    }
}

/// 1 「 in 」 ** in **（ already 、 before this that ）。
///
/// ── Why and `NewRequestEntry` ──────────────────────────
/// when and not ：
/// - `NewRequestEntry` is ****： 1 request 1 ，all，failure
/// token in `normalize` ；
/// - this is ** in **： 1 request will if （ when ），
/// that ， and ** not ** status / error —— that 1 has ，
/// before not this 1 「 in 」（ `ui/requests-panel.js` 
/// `isRunning`）， in 1 failure is before 。
///
/// is up （`core::upstream::usage` telemetry ），
/// 1 has ： and in that 1 、 up in that 1 、
/// in 1 and when 、 in up 1 to when 、 in in 
/// when 。
///
/// ── （ after 1 as ）───────────────────────────────────
/// and telemetry `note_attempt` 1 ：429 after request is after 
/// that ， in this is ， not is 1 in that 。
/// **** this ， not （ will 「」 ）。
///
/// ── Why no has token ─────────────────────────────────
/// in in before ** not usage**（`usageCell` for in to empty）， usage 
/// in up after 1 chunk —— as 1 not update 1 no has ，
/// will 1 。
#[derive(Clone, Debug)]
pub struct RunningProgress {
    /// to as this 1 provider id（empty = no to ）
    pub provider: String,
    /// id / （empty = default， or no ）
    pub account_id: String,
    pub account_name: String,
    /// to up （ after has ，empty = no ）
    pub upstream_model: String,
    /// up request etc （ and `upstream_model` when ，
    /// empty = no has etc ； `RequestEntry::upstream_reasoning`）
    pub upstream_reasoning: String,
    /// to as ** already ** （ `TelemetrySnapshot::attempts`，
    /// ≥1； before 「」 and 「 N 」）
    pub attempts: i64,
    /// ： up to for request ms（None = no to ）
    pub first_response_ms: Option<i64>,
    /// current（`connecting` / `waiting` / `streaming` / `retrying` / `queued`，
    /// `RequestEntry::phase`）—— 1 1 。
    ///
    /// Why** 1 **： is 「 in in 1 」 ，must
    /// 1 1 ；（ 、 ） will 
    /// has will to 「 up 1 + this 1 」 this not in 。
    pub phase: String,
    /// current when （ms when ）。「 in 」， has ；
    /// 「 in 」 is **request when **， by in use `started_at`
    /// （ `api::pipeline::live_row_sink`）。
    pub phase_started_at: Option<i64>,
    /// to as （** no ** that 1 ：`status` / `error` as empty
    /// —— before after 1 「 in 」， `ui/request-hover.js`）
    pub attempt_details: Vec<AttemptDetail>,
    pub route: Option<Value>,
    /// request in then （empty = no in ）
    pub sensitive_hits: Vec<SensitiveHit>,
}

/// by （ and 1 ）。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyEntry {
    /// when `YYYY-MM-DD`
    pub date: String,
    #[serde(default)]
    pub requests: i64,
    #[serde(default)]
    pub successful: i64,
    #[serde(default)]
    pub tokens: i64,
    /// in token 
    #[serde(rename = "cacheHitTokens", default)]
    pub cache_hit_tokens: i64,
    /// token （ in ）
    #[serde(rename = "cacheInputTokens", default)]
    pub cache_input_tokens: i64,
    /// by （** **）。
    ///
    /// Whymust in ：`topModel` need by ， from 
    /// 7 （`SUMMARY_WINDOW_MS`），`"all"` / `"month"` this 
    /// can ； no has ， tokens 1 。
    /// before ， 1 `modelTokens` not has 。
    /// none when （`skip_serializing_if`）——
    /// this in ** old file** when （ is old ）， when that by 
    /// `sql::encode_accum` 1 JSON （empty then is `[]`，
    /// and DDL default）。
    #[serde(rename = "modelTokens", default, skip_serializing_if = "Vec::is_empty")]
    pub model_tokens: Vec<ModelAccum>,
    /// by provider （** **，W4 add）。
    ///
    /// Why not use `modelTokens` ：provider need 
    /// （success / failure）， `ModelAccum` will that no 
    /// ，「」 this need has 。
    ///
    /// and `modelTokens` ， this must in ：`topModel` 
    /// statistics need （`SUMMARY_WINDOW_MS`，`all` / `month`
    /// can can ）， will 1 。
    /// not use `*Tokens` is as tokens （request and success）。
    #[serde(rename = "providerStats", default, skip_serializing_if = "Vec::is_empty")]
    pub provider_stats: Vec<ProviderAccum>,
    /// by （** **）。
    ///
    /// and `providerStats` 1 by ： usage need 
    /// ， has `all` / `month` will 1 。
    #[serde(rename = "accountStats", default, skip_serializing_if = "Vec::is_empty")]
    pub account_stats: Vec<AccountAccum>,
    #[serde(rename = "clientStats", default, skip_serializing_if = "Vec::is_empty")]
    pub(in crate::server::request_stats) client_stats: Vec<ClientAccum>,
    #[serde(rename = "apiKeyStats", default, skip_serializing_if = "Vec::is_empty")]
    pub(in crate::server::request_stats) api_key_stats: Vec<ApiKeyAccum>,
    /// ， old default 0；used for 1 。
    #[serde(rename = "sourceStatsVersion", default)]
    pub source_stats_version: i64,
}

impl DailyEntry {
    /// empty（ new 1 when use ）
    pub(super) fn new(date: String) -> Self {
        Self {
            date,
            requests: 0,
            successful: 0,
            tokens: 0,
            cache_hit_tokens: 0,
            cache_input_tokens: 0,
            model_tokens: Vec::new(),
            provider_stats: Vec::new(),
            account_stats: Vec::new(),
            client_stats: Vec::new(),
            api_key_stats: Vec::new(),
            source_stats_version: SOURCE_STATS_VERSION,
        }
    }
}

/// in （ ）
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelAccum {
    pub model: String,
    #[serde(default)]
    pub requests: i64,
    #[serde(default)]
    pub tokens: i64,
}

/// provider in （ ，W4 add）。
///
/// 「success」** not failure**：failure = `requests - successful`，
/// will 「 for ， for not up 」 this none ，
/// need `failures` by 1 ， no has 。
///
/// `provider` as empty has 「unknown」 request（ old 、
/// or before failure request）—— and `RequestEntry.provider` 1 。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderAccum {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub requests: i64,
    #[serde(default)]
    pub successful: i64,
    #[serde(default)]
    pub tokens: i64,
}

/// in （ ）。
///
/// and `ProviderAccum` （requests / successful / tokens + ），
/// as is 1 ：「 」 and 「 」。
/// failure by ， not 1 （ by `ProviderAccum`）。
///
/// ── Why when id and name ──────────────────────────────────
/// `accountId` is 、`accountName` is ** when **： can in 
/// ， not this ， id（ `push_account_accum`
/// then ）； need 1 down —— delete after （ and 
/// ） need 1 can ， not is 1 id。
///
/// as empty「 not is 」：default（not configured）
/// or old 。 and provider empty 1 ， by to 。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountAccum {
    #[serde(rename = "accountId", default)]
    pub account_id: String,
    #[serde(rename = "accountName", default)]
    pub account_name: String,
    #[serde(default)]
    pub requests: i64,
    #[serde(default)]
    pub successful: i64,
    #[serde(default)]
    pub tokens: i64,
}

/// （「request」 use ）——
/// by `/api/stats/requests` （ `api::stats_api`）
#[derive(Clone, Debug, Default)]
pub struct RequestQuery {
    pub offset: usize,
    /// default `DEFAULT_LIMIT`， in `[1, MAX_LIMIT]`
    pub limit: Option<usize>,
    /// （`model` = after ， by ）；
    /// None not 
    pub model: Option<String>,
    /// provider id（`provider` ）；None not 。
    ///
    /// empty id （ 1 no failure request）** not to ** —— that not is 1 ，
    /// use `status=error` 。 down empty id。
    pub provider: Option<String>,
    /// Some("") client，None not 。
    pub client: Option<String>,
    /// key:<id> / env / none / unknown， and filterValue 。
    pub api_key: Option<String>,
    pub production_only: bool,
    pub completed_only: bool,
    /// `"ok"` = 2xx / `"error"` = 2xx / None not 
    pub status: Option<String>,
    /// ms when （ down ）
    pub start: Option<i64>,
    /// ms when （**** up ）
    pub end: Option<i64>,
}
