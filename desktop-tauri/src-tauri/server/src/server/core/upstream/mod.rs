//! for （ for Node workbuddy-upstream-client.mjs ）。
//!
//! ── 1 ────────────────────────────────────────
//! ① ： body in etc before 1 （）
//! ② ：provider （）→ 
//! （ → 429 → 11128 ）→ failure
//! ③ ：SSE （reasoning and ）；： JSON
//!
//! ── file（file）────────────────────────────
//! mod.rs ：、SSE （ForwardStream）、
//! /provider 
//! payload.rs 1 （ProviderContext）+ （ by provider
//! use use body is ）
//! provider_loop.rs provider + + error + 
//! rotate.rs and 429 ：selectTargetAccount / /
//! 429 and provider up 
//! request.rs ：request、 up error、 id（** not provider**）
//! sse.rs SSE reasoning and （ chunk ）+ usage 
//! aggregate.rs （SSE → chat.completion）+ usage 
//! translate.rs up response（Anthropic / NDJSON / Gemini ）
//! ——ZCode 、Command Code、Antigravity 1 
//! usage.rs usage ：token usage / provider+ / 
//!
//! ── provider （Agent2API W2b-T3）───────────────
//! directory** not provider **：request/URL/system /error/
//! token new in `core::providers` （`workbuddy.rs`），
//! directory `ProviderAdapter` （`providers::adapter`） use 。
//! this `ProviderKind` **** use （ 、
//! provider id、 provider ）， no has 
//! 「if provider is X 」 —— provider in `adapter_for`
//! already up （ that match is ， new kind will in ），
//! not need need as 1 。
//!
//! ── and Node （ is as Rust all）────
//! 1. Node is 「 writeHead、 1 up 1 res」 ；
//! Rust must 1 `Response` to axum， 
//! 「 up response + and + in 」 1 `Stream`， by axum 。
//! is **client**：response drop when Stream drop，
//! reqwest `bytes_stream` drop，hyper to body after 
//! will `close_read()` up （ `ForwardStream` ）。
//! 2. Node `pipeSse` in res close when abort controller；Rust not need need 
//! that controller —— drop already ， and no has 「 abort」 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：** not ** unwrap/expect/panic。
//! SSE in （client、 up ） is **path**， 1 use Result/Option。

pub mod aggregate;
pub mod cancellation;
pub mod connections;
pub mod continuation;
mod image_tasks;
mod images;
pub mod image_compress;
mod multipart;
mod native_tools;
mod http_dns;
mod network_fallback;
pub mod refusal;
pub mod request;
mod payload;
mod provider_loop;
mod rotate;
mod route_state;
mod route_context;
mod conversation_ids;
mod route_sticky;
pub mod sse;
pub mod stall;
pub mod translate;
pub mod usage;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use futures::Stream;
use serde_json::{json, Value};

use axum::http::HeaderMap;

use crate::server::core::account_store::AccountStore;
use crate::server::core::auth::AuthService;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::errors::GatewayError;
use crate::server::logging;

use self::connections::{ConnectionGuard, Connections};
use self::sse::{FramePolicy, ReasoningCoalescer};

#[cfg(test)]
mod capture_tests;

/// etc limit（ for Node INFLIGHT_WAIT_MS）
const INFLIGHT_WAIT_MS: u64 = 45_000;

/// （ down none）。
///
/// Node `pickNextAccount` return null —— will triedIds 
/// 1 ，limit is 20，。 this 1 limit：
/// 1 「 1 id 」 ， can not need empty。
pub(super) const MAX_ROUTE_ATTEMPTS: usize = 32;

/// in request （ use ）。
///
/// ── Why not is 1 `Notify` ──────────────────────────────
/// `notify_waiters()` ** when already ** etc 、 and not ，
/// 「 etc in notify after 」 will etc to when 。 and 
/// （、 after ） after ， etc 「 etc 、」 can path：
/// - in before → can return；
/// - in after → notify 1 can already etc 。
/// up not can can empty， this is none 。
struct InFlight {
    done: AtomicBool,
    signal: tokio::sync::Notify,
}

impl InFlight {
    fn new() -> Self {
        Self { done: AtomicBool::new(false), signal: tokio::sync::Notify::new() }
    }

    /// and all etc （ use ：、 after ）
    fn complete(&self) {
        self.done.store(true, Ordering::SeqCst);
        self.signal.notify_waiters();
    }

    fn is_done(&self) -> bool {
        self.done.load(Ordering::SeqCst)
    }
}

/// in has ：**drop **（「response in down 」 ）。
///
/// ── Why need has （ and Node for ）────────────────────
/// Node `trackInFlight(key, promise)` is `doForwardChatCompletions()`
/// this **async return promise**， that need etc `pipeSse` resolve
/// —— is 1 to response。if Rust in 「 to response」
/// ， 1 failure can can and in up 1 request and to up ，
/// is need 。
///
/// this 「」 from to ： to 
/// `ForwardStream`，（ or client、 drop） when ，
/// 。 in `forward()` return when —— and Node 1 。
///
/// can is `pub(crate)`（ `pub(super)`）： 
/// （`providers::custom::forward`，pub ） —— 
/// use will `private_interfaces` ； is `upstream`
/// ， this no has 。
pub(crate) struct InFlightGuard {
    table: Arc<Mutex<HashMap<String, Arc<InFlight>>>>,
    key: String,
    signal: Arc<InFlight>,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        // （ is InFlight ，）
        self.signal.complete();
        // can can already after body request —— in to when 
        let mut guard = lock_table(&self.table);
        let is_self = guard
            .get(&self.key)
            .map(|entry| Arc::ptr_eq(entry, &self.signal))
            .unwrap_or(false);
        if is_self {
            guard.remove(&self.key);
        }
    }
}

/// ： + + in request。
#[derive(Clone)]
pub struct UpstreamService {
    pub(super) store: AccountStore,
    pub(super) auth: AuthService,
    /// in request：sha256(body) → （， `wait_for_in_flight`）
    in_flight: Arc<Mutex<HashMap<String, Arc<InFlight>>>>,
    /// （「」 ， `connections.rs`）
    connections: Connections,
    /// by （ / / / / ，
    /// `route_state.rs`）。 、request。
    /// ** has **：（rotate / provider_loop） to ，
    /// 1 `route_queue_facts` —— this 
    /// path。Arc is to `#[derive(Clone)]` ：（request up down ）
    /// when must is 1 —— and 。
    route: std::sync::Arc<route_state::RouteState>,
}

/// 1 
pub struct ForwardRequest {
    /// clientrequest（ is default system in ）
    pub body: Value,
    /// client is need （`body.stream === true`）
    pub stream: bool,
    /// request sha256（；empty not ）
    pub dedupe_key: String,
    /// clientrequest（ 1 ， `ProviderAdapter::build_chat_request`）
    pub client_headers: HeaderMap,
    /// usage / 。
    ///
    /// Why「 use 、request」 not is by responder 1 ：
    /// in handler return after （ axum ），
    /// that when must in ** use has ** to 。 use （）
    /// 1 `Arc` 1 ， can in when to 。
    pub telemetry: Arc<usage::RequestTelemetry>,
    /// request in Key **available**limit（R9）。
    ///
    /// `None` = already and no has limit（Environment variable key， `core::key_scope`）。
    /// Why** in ** not is 
    /// request 1 ： not this 「 Key」 this （ up and 
    /// ）， and handler already has `KeyScope`（ in request that ），
    /// 1 is 1 。（`Option<KeyScope>`） not is use ：
    /// need ， handler can can in path up return。
    ///
    /// ** has this **： is ****（handler in 
    /// when 、 404 ）， not this etc to by —— that 
    /// will request to up failure， is 「 this for this Key not in 」。
    ///
    /// path `crate::server::core::key_scope::KeyScope` not is `super::`：
    /// file in `server::core::upstream` down ，`super::key_scope` is 
    /// `server::core::upstream::key_scope`（ not in ）—— this 1 。
    pub allowed_providers: Option<crate::server::core::key_scope::KeyScope>,
    /// ** use this **（`None` = ）。
    ///
    /// before 1 use is （`api::model_test`）： is 「 this 1 this 
    /// 、 use this ， in to not 」，must to 1 up ，
    /// and** not **（`pick_next_account` in not to ）。
    /// 1 `None` —— this not is 「」 use ，
    /// no has 「 use / already delete when 1 」 （
    /// `rotate::accounts_in_providers` ）。
    pub pinned_account: Option<String>,
    /// ** by by **， [`Self::allowed_providers`]
    /// （ before is use ， and `pinned_account` 1 use ）。
    ///
    /// ── Why need has ─────────────────────────────────────────
    /// that 「」 use is 「、 need not need enabled」： 
    /// is 。 up 「 」 404
    /// 「 already in in 」（`route_for_forward` not default），
    /// that for is for in —— already and ，
    /// is 「 this 1 、 this 、 this in not 」， by in up 
    /// not and this 。
    ///
    /// by ****： default not （
    /// `builtin_target` need enabled），「」—— this is need 
    /// （directory ID to that ）， etc not ，
    /// and 「 = this not in 」 1 。
    ///
    /// 1 `false`； as empty（limit） when to not ，
    /// will 「 no has available 」—— 1 use `provider_only`， not will 。
    pub ignore_model_gate: bool,
    /// request when （ before only `api::model_test` use ）。
    ///
    /// - `None`（ as ）： in config；
    /// - `Some(None)`：（ignore ， and not ）；
    /// - `Some(Some(proxy))`：。
    pub proxy_override: Option<Option<ResolvedProxy>>,
}

/// ： need is can down ， need is JSON
pub enum ForwardOutcome {
    /// ： up + SSE （ already reasoning and ）
    Stream {
        status: u16,
        stream: Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send + Unpin>,
    },
    /// ： after JSON
    Completion { body: Value },
}

/// 1 （ for Node `{ accountId, account, proxy, proxyError }`）
pub(super) struct RouteTarget {
    /// need use provider id： this 1 to ；
    /// no has when is 「 1 」（ to default）
    pub provider: String,
    pub account_id: Option<String>,
    /// （ and use ；none when as null）
    pub account: Option<Value>,
    pub proxy: Option<ResolvedProxy>,
    pub priority: Option<i64>,
    /// （ before has 「unavailable、」）。
    ///
    /// ── Why by to ， not is 1 ──────────
    /// is **request** （ up ， 1 request will 1 ），
    /// before by request 1 —— use to is ，request that 
    /// not to 「 this request its is 」。 to this after ， 
    /// **** `notice`，request 1 can to ， not 。
    pub proxy_notice: Option<String>,
    /// ****（ 、 、Why in ——
    /// `routing::plan_route` ；request 「 by 」）。
    ///
    /// `None` = no to （ as empty、 use default）。
    /// by telemetry request； this is ****， not and 。
    pub decision: Option<Value>,
}

impl RouteTarget {
    pub(super) fn attempt_key(&self) -> Option<String> {
        self.account.as_ref().map(crate::server::core::routing_groups::attempt_key).or_else(|| self.account_id.clone())
    }
}

impl UpstreamService {
    pub fn new(store: AccountStore, auth: AuthService) -> Self {
        Self {
            store,
            auth,
            in_flight: Arc::new(Mutex::new(HashMap::new())),
            connections: Connections::new(),
            route: std::sync::Arc::new(route_state::RouteState::new()),
        }
    }

    /// by ****（`api::routing` use ）。
    ///
    /// return std （ / / ）， not has 
    /// ； not ， not in （ not this new will when ）。
    pub fn route_queue_facts(
        &self,
        now: i64,
    ) -> (
        HashMap<String, i64>,
        HashMap<String, i64>,
        std::collections::HashSet<String>,
    ) {
        let snapshot = self.route.snapshot("", now, 0);
        (snapshot.sunk, snapshot.resting, snapshot.open)
    }

    pub fn route_sticky_sessions(&self, model: Option<&str>, now: i64) -> Vec<Value> {
        self.route.sticky_list(model, now)
    }

    pub fn clear_route_sticky(&self, id: Option<&str>, model: Option<&str>) -> usize {
        self.route.sticky_clear(id, model)
    }

    /// （`api::accounts` `/api/accounts/connections` ）。
    ///
    /// return（ `Arc`） not is use ：`ServerState` handler 1 ，
    /// to is 1 。
    pub fn connections(&self) -> Connections {
        self.connections.clone()
    }

    /// 1 for request。
    ///
    /// （ for Node waitForInFlight / trackInFlight）：
    /// - ** etc 1 ** in request（ not is ）： body 2、3 request
    /// etc to 1 。Node `waitForInFlight` is and 「current 
    /// that promise」， after etc is 1 promise，。
    /// - etc has 45 secondslimit（Node ）， when after request —— not can use as 
    /// 1 before request none。
    /// - ** 1 to response**（request 1 ， InFlightGuard）。
    pub async fn forward(&self, request: ForwardRequest) -> Result<ForwardOutcome, GatewayError> {
        self.forward_request(request, None, false).await
    }

    pub async fn forward_assistant(&self, request: ForwardRequest) -> Result<ForwardOutcome, GatewayError> {
        self.forward_request(request, None, true).await
    }

    pub async fn generate_images(&self, request: ForwardRequest) -> Result<ForwardOutcome, GatewayError> {
        self.forward_request(request, Some(crate::server::core::providers::images::ImageOperation::Generate), false).await
    }

    pub async fn edit_images(&self, request: ForwardRequest) -> Result<ForwardOutcome, GatewayError> {
        self.forward_request(request, Some(crate::server::core::providers::images::ImageOperation::Edit), false).await
    }

    async fn forward_request(&self, request: ForwardRequest, images: Option<crate::server::core::providers::images::ImageOperation>, preserve_system_prompt: bool) -> Result<ForwardOutcome, GatewayError> {
        if let Some(rule) = super::update::required_update() {
            return Err(GatewayError::with_status(426, rule["message"].as_str().unwrap_or("当前版本需要强制更新")).with_code("update_required"));
        }
        // ── ： as request 1 ─────────────────
        // in this （） not is ：path（ / ×
        // none / has ） need ， 1 to 。 when ** not create**
        // —— after all `capture()` return None，。
        // id：URL / / etc when by `reset_request` up 。
        if crate::server::core::debug_traffic::enabled() {
            let id = request.telemetry.id();
            if !id.is_empty() {
                request.telemetry.set_capture(std::sync::Arc::new(
                    crate::server::core::debug_traffic::TrafficCapture::begin(&id),
                ));
            }
        }
        // （ handler ；None = ）。 1 、
        // ： and need use ， in not will 
        let cancel = request.telemetry.cancel_token();
        // already （： after 、 before ）——return，
        // not 1 
        if cancel.as_ref().is_some_and(|token| token.is_cancelled()) {
            return Err(cancellation::cancelled_error());
        }
        let route_model = request.body.get("model").and_then(Value::as_str).unwrap_or("").to_string();
        let route_store = self.store.clone();
        let route_body = request.body.clone();
        let route_scope = request.allowed_providers.clone();
        let ignore_route = request.ignore_model_gate;
        let routing = crate::server::core::blocking::run(move || {
            route_context::RequestRouting::capture_request(&route_model, &route_store, &route_body, route_scope.as_ref(), ignore_route)
        }).await??;
        let mut slot = self.begin_slot(&request.dedupe_key, cancel.as_ref()).await;
        // ：**request** ， to use 。
        // in this （ not is ） is as must to response —— path by 
        // `handoff()` response，pathreturn。
        // is 「」 and 1 ， not will has path。
        let mut connections = ConnectionGuard::new(self.connections.clone());
        // for 1 to up request；Images request。
        let mut upstream_body = request.body.clone();
        let compact_route = super::compact_routing::take_route(&mut upstream_body);
        if let Some(object) = upstream_body.as_object_mut() {
            object.remove("_wb_route_group_id");
            object.remove("_wb_route_continue");
        }
        if images.is_none() {
            if let Some(object) = upstream_body.as_object_mut() {
                object.insert("stream".to_string(), Value::Bool(true));
            }
        }
        // ── sanitize（client ）─────────────────
        // client will up use （empty name、 for 、 
        // in use and ） will ， after request this 
        // —— up for after 1 use return 400， will 。 in this 
        // （、 before ） not is ：all、all、all
        // to 1 。 then `protocol::history`。
        let history = crate::server::core::protocol::history::sanitize_history(&mut upstream_body);
        if history.notable() {
            logging::verbose(
                "[Upstream]",
                &format!("历史 sanitize：{}", history.describe()),
            );
        }
        // ** by request 1 **： 1 request provider use 1 
        // （request in set not will ）。 up that （stream 1 、
        // sanitize） after body 「 」—— / etc 
        // in 「 1 will before 」， payload.rs。
        //
        // configmust in to ： is ** use **
        // `ProviderContext` down （ `ProviderContext::prompt`）， before 
        // will empty —— 1 、 from 。
        let config = crate::server::config::current();
        let sanitize = config.sanitize_settings();
        let developer_role_compat = config.developer_role_compat();
        let prompt = config.prompt_plan();
        // set is `Copy`， 1 up down down （ not use config，）
        let image_compress = *config.image_compress_settings();
        // Key request up down ： need 
        //（ —— not need need ， in and when ）。
        // this request **** to context：`request` its 
        // （body / headers） use context， `&request.allowed_providers`
        // will as 「 when has request can use （ up body）」 not ——
        // after all， not 1 。
        let key_scope = request.allowed_providers;
        // and ：ownership 、 to context，
        // is `request` in down not use （ by up 1 ）
        let pinned_account = request.pinned_account;
        let proxy_override = request.proxy_override;
        let context = payload::ProviderContext {
            body: &upstream_body,
            routing,
            compact_routed: compact_route.is_some(),
            stream: request.stream,
            images,
            client_headers: &request.client_headers,
            telemetry: &request.telemetry,
            sanitize,
            developer_role_compat,
            image_compress,
            prompt,
            preserve_system_prompt,
            key_scope: key_scope.as_ref(),
            pinned_account: pinned_account.as_deref(),
            // bool is Copy，： not in up that use 
            ignore_model_gate: request.ignore_model_gate,
            proxy_override: proxy_override.as_ref(),
        };
        provider_loop::forward_with_providers(self, context, &mut slot, &mut connections).await
    }

    /// etc body in request， after 。
    ///
    /// return None not need need （dedupe_key as empty）。
    ///
    /// `cancel` is （None = ）： etc before return
    /// and ** not ** —— will in to and ， not is as 
    /// not to 1 already down need need 。
    async fn begin_slot(
        &self,
        dedupe_key: &str,
        cancel: Option<&Arc<cancellation::CancelToken>>,
    ) -> Option<InFlightGuard> {
        if dedupe_key.is_empty() {
            return None;
        }
        if cancel.is_some_and(|token| token.is_cancelled()) {
            return None;
        }
        self.wait_for_in_flight(dedupe_key, cancel).await;
        if cancel.is_some_and(|token| token.is_cancelled()) {
            return None;
        }
        let signal = Arc::new(InFlight::new());
        lock_table(&self.in_flight).insert(dedupe_key.to_string(), signal.clone());
        Some(InFlightGuard {
            table: self.in_flight.clone(),
            key: dedupe_key.to_string(),
            signal,
        })
    }

    /// etc body in request（ 45 seconds； then return）
    async fn wait_for_in_flight(
        &self,
        key: &str,
        cancel: Option<&Arc<cancellation::CancelToken>>,
    ) {
        let signal = lock_table(&self.in_flight).get(key).cloned();
        let Some(signal) = signal else {
            return;
        };
        // in ： this is ** **（ body request in 1 ），
        // not is —— request that no has （ etc in before ，
        // that when no has can ）， not request、 not 
        // ， in 1 when 「 this requestWhy」。
        logging::console_line("[Upstream]", "⏳ 检测到相同请求正在处理，排队等待（防重试风暴）");
        // etc 、check：notify_waiters 「 when already 」 
        // etc ， this 「 in before 」 and 「 in after 」 not will 
        let mut notified = std::pin::pin!(signal.signal.notified());
        if notified.as_mut().enable() {
            return;
        }
        if signal.is_done() {
            return;
        }
        let wait = tokio::time::timeout(Duration::from_millis(INFLIGHT_WAIT_MS), notified);
        tokio::pin!(wait);
        match cancel {
            // ：（return after by begin_slot 、
            // in ）
            Some(token) => tokio::select! {
                _ = &mut wait => {}
                _ = token.cancelled() => {}
            },
            None => {
                let _ = wait.await;
            }
        }
    }
}

/// in ； in not （ and 1 ）
fn lock_table<'a>(
    table: &'a Mutex<HashMap<String, Arc<InFlight>>>,
) -> std::sync::MutexGuard<'a, HashMap<String, Arc<InFlight>>> {
    match table.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// SSE ： up → reasoning and → client。
///
/// ── client（ and ）───────────────
/// this ** not ** up request， is drop ， is ：
/// 1. `reqwest::Response::bytes_stream()` return has hyper body 
/// （`Incoming` data channel）。 this drop when ，hyper 
/// h1 dispatch in `try_send_data` up to `Err(_canceled)`，
/// `self.conn.close_read()`（hyper 1.x `proto/h1/dispatch.rs` 
/// 「body receiver dropped before eof, closing」 that ）， up TCP 
/// —— up to is 「client」。
/// 2. axum in client when will in response future （），
/// `poll_next` not use 、 after drop。
/// 「 down → up 」 is and none 。
/// and Node ：Node use AbortController abort（undici ），
/// 1 （ up will to ）， is path not 。
///
/// ── up in （headers already ）────────────────────────
/// Node when 1 `data: {"error": {...}}` + `data: [DONE]`（
/// server.mjs 551-554）。 this 1 ： ——
/// for OpenAI SDK ， this 「 」 1 failure 。
pub struct ForwardStream {
    /// up （ already consume Response， is 'static）。
    ///
    /// error 1 `io::Error`：`new`（reqwest ） in map when error use 
    /// `describe_error_detail` （ that reqwest::Error，
    /// after poll_next to ）；`from_translated`（） 
    /// up io::Error。 in poll_next use 1 「error + [DONE]」
    /// ， 1 。
    inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
    coalescer: ReasoningCoalescer,
    /// up already （ not poll up ， pending ）
    upstream_done: bool,
    /// down （ 1 up chunk can can ）
    pending: std::collections::VecDeque<Bytes>,
    /// in ： drop when （client、 up path）。
    /// `Option` is as can in optional，total is Some。
    _slot: Option<InFlightGuard>,
    /// ： and 1 —— / drop when 
    /// this -1。** not is ** `Option` ，total is Some，
    /// is 「none 」（） that `account_id` is None，
    /// is empty， not 。
    _connection: ConnectionGuard,
    /// usage / ： and and use 1 （ `ForwardStream::new`）
    telemetry: Arc<usage::RequestTelemetry>,
    /// （ when 1 ，None = ）。
    ///
    /// in this not is （`telemetry.capture()`）： in 
    /// ** up chunk** up ， 1 is ； 1 request 
    /// in when ， in not will 。
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

impl ForwardStream {
    /// `policy` by `sse_model_rewrite()` / `sse_strip_newline_chunks()`
    /// （ `sse::FramePolicy`）： when down and before 1 
    /// （workbuddy not is need ）。
    pub(super) fn new(
        response: reqwest::Response,
        slot: Option<InFlightGuard>,
        connection: ConnectionGuard,
        telemetry: Arc<usage::RequestTelemetry>,
        policy: FramePolicy,
    ) -> Self {
        use futures::StreamExt;
        // reqwest error in this （`describe_error_detail` is 
        // reqwest::Error； io::Error after poll_next can to ）
        let inner = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        // responseempty when （set「request when 」）： when ，
        // to new ； when in when （ stall module header）
        let guarded = stall::idle_guard(
            Box::pin(inner),
            std::time::Duration::from_millis(
                crate::server::config::timeout_settings().stream_idle_ms(),
            ),
        );
        let capture = telemetry.capture();
        let mut stream = Self::from_translated(guarded, slot, connection, telemetry, policy);
        stream.capture = capture;
        stream
    }

    /// ：`inner` already is ** chat SSE** 。
    ///
    /// responses / anthropic up `providers::custom` 
    /// `ProtocolTranslateStream`（ up → chat ）， 
    /// reasoning and / usage / model / —— this 
    /// chat ， not need need up is 。
    pub(super) fn from_translated(
        inner: futures::stream::BoxStream<'static, Result<Bytes, std::io::Error>>,
        slot: Option<InFlightGuard>,
        connection: ConnectionGuard,
        telemetry: Arc<usage::RequestTelemetry>,
        policy: FramePolicy,
    ) -> Self {
        // ── （add）────────────────────────────
        // etc 1 「 1 error」 ： after poll 
        // to Err，** not etc down 1 up **（ up when is this ，
        // check will to down 1 ）。
        // not —— 1 ， no has 「」 
        // empty（ for ：spawn 1 etc need need Weak ）。
        // error is ，poll_next error not 
        // 「 up in 」 before （ that will up ）。
        //
        // is `cancellation::cancellable` ** not is ** `stream::select`：
        // after is 「」， in up when not ，
        // is `poll_next` not to `Ready(None)` —— client after 
        // not 、 1 etc in that （ `cancellable` ）。
        let inner = cancellation::cancellable(inner, telemetry.cancel_token());
        Self {
            inner,
            coalescer: ReasoningCoalescer::with_telemetry(telemetry.clone())
                .with_policy(policy),
            upstream_done: false,
            pending: std::collections::VecDeque::new(),
            _slot: slot,
            _connection: connection,
            telemetry,
            // already by ， not can chat 。
            capture: None,
        }
    }
}

impl Stream for ForwardStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            if let Some(frame) = self.pending.pop_front() {
                return std::task::Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return std::task::Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(context) {
                std::task::Poll::Pending => return std::task::Poll::Pending,
                std::task::Poll::Ready(None) => {
                    self.upstream_done = true;
                    for frame in self.coalescer.finish() {
                        self.pending.push_back(frame);
                    }
                }
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    // ： ** up ** to —— in and 
                    // before ， as use need is up ， not is 
                    // / and after （ that is 「 up to 」 need ）
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.coalescer.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                std::task::Poll::Ready(Some(Err(error))) => {
                    // up in ： that is **path**（client、 up ），
                    // not panic。 already reasoning ， up 
                    // 「error + [DONE]」（ and Node 1 ）。
                    self.upstream_done = true;
                    for frame in self.coalescer.finish() {
                        self.pending.push_back(frame);
                    }
                    // error already in when io::Error（ inner ）
                    // to is （ from_translated）；
                    // empty to is before （ stall::IDLE_TIMEOUT_PREFIX）
                    // —— not 「 up in 」 before ： that not is up not in 
                    let text = error.to_string();
                    let message = if text == cancellation::MANUAL_TERMINATED
                        || text.starts_with(stall::IDLE_TIMEOUT_PREFIX)
                    {
                        text
                    } else {
                        format!("上游流式传输中断: {error}")
                    };
                    // in ： this Reason by down `note_error` request
                    // （client when already to ，HTTP is 200，
                    // has request 「error」 can 「Why this is failure 」）。
                    logging::console_line("[Model]", &format!("❌ {message}"));
                    // ：Reason need request（client when already to ，
                    // HTTP is 200， has this can 「Why this is failure 」）
                    self.telemetry.note_error(&message);
                    self.pending.push_back(self::sse::sse_frame(&json!({
                        "error": {
                            "message": message,
                            "type": "proxy_error",
                        }
                    })));
                    self.pending.push_back(Bytes::from_static(b"data: [DONE]\n\n"));
                }
            }
        }
    }
}

/// will is accessToken（Node: `session?.auth?.accessToken` ）
pub(super) fn has_access_token(session: &Value) -> bool {
    session
        .get("auth")
        .and_then(|auth| auth.get("accessToken"))
        .and_then(Value::as_str)
        .map(|token| !token.is_empty())
        .unwrap_or(false)
}

/// （Node `account.name || account.id`）。
///
/// ** `displayName`**（`store_view::public_account` 1 ，
/// and before `displayNameOf` —— 「」 in ）；
/// this （ old / for ） when by old `name || id`。
pub(super) fn account_display(account: &Value) -> String {
    account
        .get("displayName")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| {
            account
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .or_else(|| account.get("id").and_then(Value::as_str).map(str::to_string))
        .unwrap_or_default()
}

/// ： → will → id
/// （ for Node `target.account?.name || session.account?.nickname || accountId`）。
///
/// `displayName` by [`account_display`]：，
/// is 。 will in for （ will ） when —— 
/// `session.account.nickname` not 。
pub(super) fn account_label(account: Option<&Value>, account_id: &str, session: &Value) -> String {
    account
        .map(account_display)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            session
                .get("account")
                .and_then(|account| account.get("nickname"))
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| account_id.to_string())
}

/// can （ use ）
pub(super) fn describe_proxy(proxy: Option<&ResolvedProxy>) -> String {
    match proxy {
        Some(proxy) if !proxy.label.is_empty() => proxy.label.clone(),
        Some(proxy) => proxy.host.clone(),
        None => "直连".to_string(),
    }
}

/// （ `priority=N`）
pub(super) fn priority_of(account: &Value) -> Option<i64> {
    account.get("priority").and_then(Value::as_i64)
}

/// when ：`（< when > ）`，empty no has when 
pub(super) fn reset_hint(reset_text: &str) -> String {
    if reset_text.is_empty() {
        String::new()
    } else {
        format!("（{reset_text} 恢复）")
    }
}

/// when （ for Node `toLocaleString('zh-CN', { hour12:false })`）。
/// and errors.rs reset_at_text ： 1 by UTC+8 ，
/// as up to when is UTC+8 ， use when will use for not up 。
pub(super) fn format_reset_text(reset_at: f64) -> String {
    if !(reset_at > 0.0) {
        return String::new();
    }
    let Some(utc) = chrono::DateTime::from_timestamp_millis(reset_at as i64) else {
        return String::new();
    };
    // +8 1 can success； 1 failure to empty（ this is ，
    // not can as 1 failure request —— release is panic=abort）
    let Some(offset) = chrono::FixedOffset::east_opt(8 * 3600) else {
        return String::new();
    };
    utc.with_timezone(&offset)
        .format("%Y/%m/%d %H:%M:%S")
        .to_string()
}