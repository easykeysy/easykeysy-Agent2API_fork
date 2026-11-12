//! ProviderAdapter （Agent2API W2b-T3， §4.2）。
//!
//! ── this 1 ──────────────────────────────────────
//! before ， 1 default「 up is WorkBuddy」：URL is 
//! `/v2/chat/completions`、 by workbuddy 、429/6004 and 11128 
//! in `upstream/request.rs` and `rotate.rs`。 after ，`upstream/`
//! **none **（、、SSE 、），
//! all「 this 1 」 —— is that 。
//!
//! ── （ not ， §4.2 need + 1 ）──
//! 1. [`UpstreamErrorClass::QuotaLimited`] → for this + down 1 ；
//! 2. [`UpstreamErrorClass::TokenExpired`] → new after ** 1 ** 1 ；
//! 3. [`UpstreamErrorClass::ContentBlocked`] → ** not **： in after 1 
//! 1 ， and （`core::degrade`； 
//! `ErrContentBlocked` —— that is ， not is ）；
//! 4. [`UpstreamErrorClass::Fatal`] → to client。
//!
//! this is 1 error， can in this ——
//! in no has for ， will Fatal，
//! etc 「 this 」 error to use 。
//!
//! ── Why is 「 + match」 not is HashMap ─────────
//! is **none**（ in 、directory in `core::models` ，
//! has and ）。 `&'static dyn ProviderAdapter` ：
//! no has 、 no has 、`adapter_for` is O(1) and not failure。
//! use `OnceLock<HashMap<…>>` need 「 no when use 」 path，
//! that path in panic=abort release can panic —— not 。
//!
//! ── 1 （ is default ， not §4.2 ）──
//! 1. `retry_advice`：「 can error」 and to and 。
//! §4.3 need 「11128 in 」——**** in ，
//! 「 need 、」 is provider （11128 is workbuddy 
//! ）， in this can upstream/ 1 provider not 。
//! 2. `refresh_access_token`：401 after **** new 。`ensure_access_token`
//! is 「available token（ new ）」， 401 token
//! server —— can can to ， ensure not will new 。
//! default `ensure_access_token`，provider need need when 。
//! 3. `allows_anonymous_default_session`： provider is 「 1 
//! no has when use /default」。workbuddy has this 
//! （`WORKBUDDY_TOKEN`）， has （`RACCOON_TOKEN`， in ）。
//! 4. `sse_model_rewrite`（W3-T4 ）：SSE ** need not need down `model`
//! clientrequest that **。 up will 
//! （ this up will chunk `model` ，clientrequest that ），
//! workbuddy up 。 in `upstream::ForwardStream`
//! （ that is SSE down 1 ）， this 「 need not need 」，
//! use not provider —— and `retry_advice` 1 。
//! 5. `supports_refresh` + `credentials_expiring`（）：
//! 「 this can not can 」「 this not 」 is ** **
//! （expiration time in 、 use ， not ）， by ，
//! （`core::credential_maintenance`） and use 。
//! in this not is ： can by ， that 
//! not （ expiration time in `tokenExpiresAt`， no has `expiresAt`）
//! —— 1 will 「none need new 」。
//! ：** not return false**（ no has / not in ），
//! can not need up 。
//! 6. `is_stateful` + `forward_conversation`（W4a ）： provider 
//! up is not is ** will **（round → turn → → completed）。
//! default false； by that will 。`forward_conversation`
//! is will （default 503）， `provider_loop` by 
//! `is_stateful` to 。
//!   7. `supports_web_login` + `build_login_url` + `exchange_login_code`
//! （「」）： → use → 1 code
//! → use code and ****。
//! workbuddy ** not this **： state/authUrl up `auth/state`，
//! by `core::login` after `auth/token` ， is 1 
//! （ `api::session::login_start`）， has `supports_web_login` for 
//! default false， by as can use this 。
//! Why trait not is in `api::session` provider ：
//! 「、、 use 、response is 
//! 、path」 is provider ， and `sse_model_rewrite`
//! 1 ； by this 「 this unsupported」， not this 。
//! 8. `supports_usage` + `query_usage`（ / ）：「 this has no has 」
//! 「 this 」 is provider （ 、、
//! not ：workbuddy 、 need Bearer JWT、
//! CatPaw need will cookie `token2`、AutoClaw userapi ），
//! by ，`api::accounts` and and failure 。
//! before this can ** has workbuddy has **：`resolve_batch_targets` by 
//! provider ， before in workbuddy up 「」 by 。
//! 1 in [`ProviderAdapter::query_usage`] 。
//! 9. `supports_model_refresh` + `refresh_models(force)`（「 new 」）：
//! 「 this has no has directory can 」 and 「 this need not need 」 is ， 
//! （ before can ， after ）。** in true** ——
//! has directory（ down is ， after 
//! `models.rs` module header）：WorkBuddy `GET /v3/config`、
//!      `GET {llmBase}/model_catalog`、Qoder `GET {gateway}algo/api/v2/model/list`、
//!      CatPaw `POST /api/agent/maas/model-types`、AutoClaw
//!      `GET .../proxy/autoclaw-model-config`。
//! （CatPaw and AutoClaw default false， by is 「 up no has directory」——
//! that before after ， 1 in ， is ，
//! `catalog.rs` module header。）`refresh_models` return is 
//! [`ModelRefreshOutcome`]：path not （ old ），
//! path， [`refresh_implemented_forced`]。
//! default is false： that is to 「 will new 、directory no 」 provider 
//! （ and `supports_chat` 1 ）。
//! 10. `reasoning_patch`（ etc ，R7 after ）：「 use 8 
//! up and 」 is ** **（CatPaw 
//! low/high/max， 1 400；Qoder by efforts 1 ；
//! body、 up not this ）， by ，
//! （`upstream::payload`） in when 1 、 by body。
//! **default is 「 not 」**（return `NotSupported`）—— by that ：
//! 1 need has 1 ， no etc unknownparameter to up 。
//! 11. `sse_strip_newline_chunks`（ down 「 has 」 content ）：
//! WorkBuddy and AutoClaw up will in 1 content
//! （ that is ， 34.5% request、request in 
//! 13.4%， `upstream::sse` module header）。 not is provider —— in 
//! can can is ， by ， in `upstream::sse` this 
//! SSE 1 （ and `sse_model_rewrite` 1 、 1 ）。
//! **default false**： no provider down not 。
//! not etc will —— need `stripNewlineKeepalive` this ****
//! （default， `config::KEY_STRIP_NEWLINE_KEEPALIVE`）： can " this 
//! up will not will "，" this need not need as "， and ， and can 
//! not can （ `upstream::sse::FramePolicy::assembled`）。
//!
//! ── in this ──────────────────────────────────────
//! 1 no has ：29 `ProviderKind` **** to 
//! （`agent2api-workers` ， not ）。[`adapter_for`]
//! for kind return `missing_plugin::placeholder` —— 「 need 
//! 」， not is in 1 will when 。
//!
//! trait in by ： is ** **（`plugin_host` 
//! `PluginAdapter` ）， not need need 「 this is is 」。
//! empty + 503 in `catalog` / `accounts` ：`/v1/models`
//! not will as empty that 。

use axum::http::HeaderMap;
use serde_json::Value;

pub use super::retry::RetryDecision;

use std::sync::Arc;

use crate::server::core::account_store::AccountStore;
use crate::server::errors::GatewayError;

use super::ProviderKind;

/// before for 「 + request」 （ §4.2 ChatRequestPlan）。
///
/// ── and `upstream::request::TransportRequest` （ need ）───
/// that is ****： already `payload: String` + ，
/// to `send_chat_request` 。 is **provider ** ：
/// body is `Value`（ not then not ，e.g. workbuddy need 
/// system ）， and ** not ** —— is 、 and provider none，
/// by from will after up 。
pub struct ChatRequestPlan {
    /// up URL（path）
    pub url: String,
    /// request（ not ；Authorization in ）
    pub headers: Vec<(String, String)>,
    /// already by provider then request
    pub body: Value,
    /// up **response** is （default [`UpstreamResponse::Chat`]）
    pub response: UpstreamResponse,
    /// 1 after ； None as ，Some(None) as none。
    pub reasoning: Option<Option<String>>,
    pub policy: super::policy::ChatPolicy,
}

impl ChatRequestPlan {
    /// ： up OpenAI Chat（request and response is chat ）。
    ///
    /// up （ and ） is this 1 ； has 1 use 
    /// ：ZCode Anthropic（[`UpstreamResponse::Anthropic`]）、
    /// AI SDK NDJSON（[`UpstreamResponse::AiSdkNdjson`]）、
    /// Antigravity Gemini v1internal 
    /// （[`UpstreamResponse::AntigravityGemini`]）。 not is 
    /// ， is as 「response」 this 1 new not to use 1 
    /// will 。
    pub fn chat(url: String, headers: Vec<(String, String)>, body: Value) -> Self {
        Self {
            url,
            headers,
            body,
            response: UpstreamResponse::Chat,
            reasoning: None,
            policy: super::policy::ChatPolicy::default(),
        }
    }

    /// AI SDK v5 NDJSON ：request is （ by ），response is 
    /// NDJSON（ [`UpstreamResponse::AiSdkNdjson`]）。
    ///
    /// and [`Self::chat`] 1 ： use 「response is 」，
    /// not `ChatRequestPlan` 。
    pub fn ai_sdk_ndjson(
        url: String,
        headers: Vec<(String, String)>,
        body: Value,
    ) -> Self {
        Self {
            url,
            headers,
            body,
            response: UpstreamResponse::AiSdkNdjson,
            reasoning: None,
            policy: super::policy::ChatPolicy::default(),
        }
    }

    /// Antigravity ：request is Cloud Code Assist v1internal 
    /// （`{project, model, userAgent, requestId, request:{…}}`），response is 
    /// **Gemini SSE**（`data: {"response":{…}}`，
    /// [`UpstreamResponse::AntigravityGemini`]）。
    ///
    /// and 1 （ use 「response is 」），
    /// 1 is as need **request** 1 —— that has 
    /// `antigravity_outbound` will 。
    pub fn antigravity_gemini(
        url: String,
        headers: Vec<(String, String)>,
        body: Value,
    ) -> Self {
        Self {
            url,
            headers,
            body,
            response: UpstreamResponse::AntigravityGemini,
            reasoning: None,
            policy: super::policy::ChatPolicy::default(),
        }
    }
}

/// up response （**request and responsemust**： this by in request when 
/// 1 and to ， by ， not ）。
///
/// ── Why need need ────────────────────────────────────────────
/// before is 「all up Chat」（ `protocol` module header）， is 
/// nonepath down 1 by chat SSE 。ZCode this 
/// before ： is Anthropic Messages（`stream:true` when Anthropic 
/// ）。 and its as 1 new 1 「」 （ that will 、、
/// 、usage and ， `upstream::provider_loop` has path），
/// not 「response need 1 」 1 —— 1 ，
/// its use 。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UpstreamResponse {
    /// OpenAI Chat SSE（default）
    #[default]
    Chat,
    /// Anthropic Messages SSE： down before chat SSE（ `upstream::translate`）
    Anthropic,
    /// **AI SDK v5 NDJSON**（`application/x-ndjson`）： 1 1 JSON 、
    /// **HTTP 200**、error `{"type":"error"}` ， and 
    /// 「 this 1 no 」 is 。 down before chat SSE
    /// （ `upstream::translate` and `protocol::ai_sdk_ndjson`）——
    /// by and Anthropic that ：、、、
    /// usage and in ， has in 。
    ///
    /// this is **** not is 1 has ：，
    /// 「 in use this 、request」 by 。
    AiSdkNdjson,
    /// Antigravity **Gemini v1internal + SSE**： up is Google Cloud Code
    /// Assist `:streamGenerateContent?alt=sse`， is `data: {json}`， and 
    /// Gemini response in `response` down （， §4.2）。
    /// path（`candidates[0].content.parts[*]`、`usageMetadata`、
    /// `finishReason`、`thought` 、`thoughtSignature`） is Gemini ，
    /// and Anthropic / NDJSON and 1 （`upstream::translate` 
    /// `AntigravityToChatStream` ← `protocol::antigravity_stream`）。
    /// by up ：、、、usage and 
    /// in ， has in 。
    AntigravityGemini,
    /// Images JSON or Gemini inlineData ， not Chat 。
    Images(super::images::ImageResponseKind),
    /// **SSE **： is SSE（`data: {json}`） no ， 1 
    /// —— `part.status == "init"` to is **add 1 **，
    /// finish to is ****。 OpenAI need is 1 delta：
    /// will down （），
    /// then will 。
    /// 1 ** by ** （`upstream::translate` 
    /// `ChatglmToChatStream` ← `protocol::chatglm_stream`）。
    ///
    /// and 1 ： **failure HTTP 200 +
    /// `application/json`**（`-1001 remaining not `）， not this ——
    /// that 1 by `is_error_response` in down （「，
    /// not by 」， `plugin_host::rpc`），
    /// has error and （ and ZCode 1 ）。
    ChatglmSnapshot,
}

/// up error（ §4.2； module header）。
///
/// ── ：`message` is **client can **（ before ）────
/// before client error is ` up return {status}: { up }{}`， its in in 
/// workbuddy 11128（） when 。 as 1 then is 
/// provider （ is workbuddy ）， in 、
/// by use —— has use error** not **。
#[derive(Clone, Debug)]
pub enum UpstreamErrorClass {
    /// / → and down 1 （workbuddy 429 / code 6004）
    QuotaLimited {
        /// up to when （ms）； not when None（ use to ）
        reset_at: Option<i64>,
        /// client can （` up return {status}: { up }`）
        message: String,
        /// up （ GatewayError `upstream_code` and ）
        upstream_code: Option<i64>,
        /// HTTP 
        status: u16,
    },
    /// token → new after use 1 1 
    TokenExpired {
        /// client can （` up return 401: { up }`）
        message: String,
    },
    /// （HTTP 400 + ）→ ** not **： in after 
    /// 1 1 ， and （ `core::degrade`）。
    ///
    /// is 「」 not is ：、、session ，
    /// will 1 （ 1 body will ）。
    /// then is use （ `providers::content_block`）， 
    /// `upstream::provider_loop` 「 0」。
    ContentBlocked {
        status: u16,
        /// client can （ 11-128 provider ）
        message: String,
        /// up （ GatewayError `upstream_code`）
        upstream_code: Option<i64>,
    },
    /// to client error
    Fatal {
        status: u16,
        /// client can （ 11-128 provider ）
        message: String,
        /// up （ GatewayError `upstream_code`）
        upstream_code: Option<i64>,
    },
}

/// 1 「 can 」 （module header 1）。
///
/// 「 need not need 」**Reason** 1 to ， this 1 、
/// 1 ， and this request —— is 11128/「」
/// this provider not will `upstream/`。
///
/// ── Why is `reason` not is 1 `log_message` ─────────────
/// before this to is ****（「5 seconds after （ n/N ）」 that ），
/// 1 。 in 1 has ：request 
/// （ need is 「Why」） and （ need is can 1 ）。 if by 
/// to ， that 「 n/N 」 this has by ，or
/// 1 —— before will ， after 。
/// 「 to Reason + 」 after ， is 1 ，
/// and that 1 and before （ `provider_loop::retry_log_line`）。
#[derive(Clone, Debug)]
pub struct RetryAdvice {
    /// when （ms）
    pub delay_ms: u64,
    /// Reason（provider ，「 up （11128）」）。
    /// use and request 。
    pub reason: String,
}

/// 1 etc （`ProviderAdapter::reasoning_patch` return）。
///
/// ── Why is 「 need 1 、 need 1 Reason」 not is `Option<Value>` ──
/// （`upstream::payload`） need ： body、** **。
/// 「 not 」 has （ this unsupported / client already / etc not in can /
/// this etc is ……）， for use Reason not —— use `Option` will 
/// can 1 「」， that is 「Why not 」 。
/// Reasonreturn 1 ，must as to 1 。
#[derive(Clone, Debug)]
pub enum ReasoningPatch {
    /// ： body `field` `value`（ old ）。
    ///
    /// `Value` not is `String`：current this need is ，
    /// up 「 + 」 is （、 for 
    /// ）， `Value` will that not this ****。
    Set {
        /// body （ that resolver ，
        /// `reasoning_effort`）—— ， not 。
        field: &'static str,
        value: Value,
    },
    /// not （body 1 not ， can before as ）。`reason` 。
    Skip {
        /// to use 1 Reason（「 this unsupported etc 」/「clientrequest already 」
        /// /「 this etc not in 」……）。
        reason: &'static str,
    },
}

/// can （ §4.2）。
///
/// must is **none or ** ：`adapter_for` return 
/// `&'static` use will and use ， trait no has `&mut self` 。
///
/// `Send + Sync` is required ： will in await has ，
/// in tokio when up 。
///
/// ── Why `-> Pin<Box<dyn Future>>` ─────────────
/// trait not can `async fn`（Rust 1.77 MSRV unsupported async fn in trait，
/// `async_trait` crate will to 1 Box ）。 Box 
/// in this up ， is trait object-safe and not new 。
pub trait ProviderAdapter: Send + Sync {
    /// for provider
    fn kind(&self) -> ProviderKind;

    /// provider id（`&'static str`）。
    ///
    /// ── Why not can `kind_id(kind())` ─────────────────
    /// `kind()` is （ provider no has ）， 
    /// is id 。 default [`kind_id`]；`RpcAdapter`
    /// id。 new 1 use this ， `kind_id(adapter.kind())`。
    fn provider_id(&self) -> &'static str {
        super::kind_id(self.kind())
    }

    /// （、error、）。
    ///
    /// default from ； `label`（leak 
    /// `&'static`， by and —— `RpcAdapter` ）。
    fn label(&self) -> &'static str {
        super::meta(self.kind()).label
    }

    /// this provider （directory / by ）。
    ///
    /// return is ** up ** （ and `/v3/config` or 
    /// `/model_catalog` 1 ）， use `models::list_item` 。
    ///
    /// ── this return** when **（ before this ）──────
    /// - ** can **（`catalog::providers_for_model` → by ）：
    /// 「 has no has this 」， request to ；
    /// - ** **（`/v1/models` and ，
    /// [`Self::advertise_models`] ）。
    ///
    /// use 1 is （「 no has 、」 by 
    /// ）。** not need ** in this 「 by use 」 ： need by 
    /// when ， [`Self::advertise_models`]。
    ///
    /// ── ** not can use **（2026-09 ）──────────────
    /// （`api::pipeline::resolve_model`）**** as ：client
    /// `/v1/models`， 1 that no has 400
    /// `model_not_found`， not to up 。 [`Self::advertise_models`]
    /// （ Cline by that ）** not can use ** ——
    /// 「 can to ，」。
    fn list_models(&self) -> Vec<Value>;

    /// for before （default as etc ）。
    ///
    /// `manifest` is [`Self::list_models`] return，「 its in 
    /// need in `/v1/models` and 」。
    ///
    /// ── is ****， not is （2026-09 ）────────────
    /// as （`api::pipeline::resolve_model` →
    /// `catalog::advertised_manifest_contains`）： client not to 、
    /// not （400 `model_not_found`）。 this is 「 this can use 
    /// 」， will use can use in 。
    ///
    /// ── `store` Why is parameter（ not is 1 ）──────────
    /// need need by must** use up that store **：
    /// `AccountStore` is `Clone` `with_config_dir()`
    /// 1 ， 1 「 」——
    /// to can can not is new ， and -- and not can 。
    /// by use 1 file，「 is 1 」 this 。
    ///
    /// default etc （ in no has 「 1 」 this ），
    /// defaultignore `store`。
    fn advertise_models(&self, _store: &AccountStore, manifest: Vec<Value>) -> Vec<Value> {
        manifest
    }

    /// use up request（URL / / ）。
    ///
    /// ── `account` （ and use ）────────────────────
    /// is ** this will **（`store.get_session_by_id` /
    /// `auth.get_current_session` return： `auth.accessToken`、
    /// `endpoint`、`edition`、`account.uid`、`proxy`）， not is accounts.json 
    /// —— by ** not ** accessToken（ has tokenTail），
    /// not Authorization。「account as accounts.json for 」
    /// is 「 this is 」； by （ §4.2 
    /// 「 as has parameter」）。
    ///
    /// `body` is clientrequest（）；`client_headers` is client
    /// （ not ，parameter is as will 「client UA / 」
    /// provider not ）。
    fn build_chat_request(
        &self,
        account: &Value,
        body: &Value,
        client_headers: &HeaderMap,
    ) -> Result<ChatRequestPlan, GatewayError>;

    /// [`Self::build_chat_request`] （add）。
    ///
    /// ── Why need need ──────────────────────────────────────
    /// 21 is （ URL / / body），；
    /// in 1 ， can 。 1 will 21 
    /// ，and to use 1 `Box::pin` —— that is 。
    ///
    /// ：**default**， has （`RpcAdapter`）
    /// 。 1 this （ `upstream::provider_loop`）——
    /// is 「、 RPC」 this for 。
    ///
    /// ── must 1 ────────────────────────────────
    /// in return「unsupported」（ to use ），
    /// not is return 1 and not —— down is 「 has 
    /// path use A、 has path use B」， that error no has will 。
    fn build_chat_request_async<'a>(
        &'a self,
        account: &'a Value,
        body: &'a Value,
        client_headers: &'a HeaderMap,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move { self.build_chat_request(account, body, client_headers) })
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    fn supports_image_editing(&self) -> bool {
        false
    }

    fn build_image_request_async<'a>(
        &'a self,
        _account: &'a Value,
        _body: &'a Value,
        _client_headers: &'a HeaderMap,
        _operation: super::images::ImageOperation,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            Err(GatewayError::with_status(501, "当前提供商插件尚未接入图片生成，请更新插件"))
        })
    }

    /// 「 up etc 」 up and （module header 10）。
    ///
    /// ── use when and use ────────────────────────────────────────
    /// `upstream::payload::send_body`： 1 provider already 、 will ，
    /// in by after 1 （** and 1 **， as etc and 
    /// 1 ， `catalog::WireTarget`）。： 
    /// `level` 、 by return body、 `reason` 。
    /// not 1 ， not 「 this 」 。
    ///
    /// ── `level` is 、 not is （must this ）────────────────
    /// is ** up that **（ already `normalize`：empty、
    /// empty、 not 32 ），** no has ** by use —— 「 this can not can 
    /// 」 is ， as can not （CatPaw has ，Qoder
    /// efforts）。
    ///
    /// ** use ** in `model_rules` ， use not is 1 ：
    /// - `model_rules::reasoning_is_off(level)`：`off` / `none` —— 1 not 。
    /// no has 「」（Qoder `enable_thinking = false`
    /// will Qwen3.8 as ，CatPaw no has this 1 ），** not need ** 
    /// 1 「」 ，`Skip` is ；
    /// - `model_rules::reasoning_rank(level)`： use 6 
    /// （0 = minimal … 5 = max）， etc return None。
    /// CatPaw `rank / 2` （ that ）；
    /// no has use ； has directorysupports when can 。
    ///
    /// ── Whydefault is 「 not 」 not is 「 1 use 」────────────
    /// defaultreturn `Skip`。 and must by ， not can only
    /// supportsReasoning can ；WorkBuddy and 1 in 。
    ///
    /// ── must ：client is not is already ──────────────
    /// client in request（CatPaw `reasoning_effort` /
    /// `reasoningEffort` / `effort`，Qoder `reasoning_effort` / `reasoning` /
    /// `thinking`） when ，** not ** —— that is use ， up default
    /// 。** use that resolver**（`catpaw::models::resolve_effort`
    /// 、`qoder::protocol::resolve_thinking` that ）， not need in this 
    /// 1 ： 1 down is up will 1 when ， will 
    /// 1 「client its already 」 request， that error no has will 。
    ///
    /// ── and `build_chat_request` （Why not after parameter）──────
    /// that is 「 + client body」， etc ****、 by 
    /// in by when （ 1 ， `catalog::WireTarget`）；
    /// `build_chat_request` will to 1 and none ，
    /// will `forward_conversation` that has path（CatPaw / Qoder ）
    /// not to this —— this need need 。
    ///
    /// `model` is ** will to up that **（ already by ， `WireTarget.model`）：
    /// need need by （Qoder need `efforts`） use ，
    /// not ignore。
    fn reasoning_patch(&self, _level: &str, _model: &str, _body: &Value) -> ReasoningPatch {
        ReasoningPatch::Skip {
            reason: "该提供商不支持思考等级绑定（上游无对应字段）",
        }
    }

    /// [`Self::reasoning_patch`] （add， by 
    /// [`Self::build_chat_request_async`]）。
    ///
    /// ── default ──────────────────────────────────────────
    /// ，； need （ 
    /// and in ）。
    fn reasoning_patch_async<'a>(
        &'a self,
        level: &'a str,
        model: &'a str,
        body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ReasoningPatch> + Send + 'a>> {
        Box::pin(async move { self.reasoning_patch(level, model, body) })
    }

    /// from ** will to ** etc （request「 up etc 」
    /// ；`None` = this request no has etc ）。
    ///
    /// ── use when and is body ─────────────────────────────
    /// `upstream::payload::send_body` in by 、 etc 
    /// （[`Self::reasoning_patch`]）** after ** use 1 。 when body etc 
    /// need is client （ when ）， need is ——
    /// is up to ： （CatPaw / Qoder） use **
    /// resolver 1 **， and as ；default is 
    /// （ down ）。
    ///
    /// ── Whydefault（ not is None）─────────────────────
    /// `send_body` model 、 etc ，** not client**：
    /// client etc （`reasoning_effort` etc use ） for 1 
    /// up —— that is ， and not is 
    /// （「 not 」 is not ****， [`Self::reasoning_patch`] default `Skip`；
    /// client no has by in up ）。default use 
    /// and ；「」 not （ and Qoder
    /// 1 ）， not 1 「 no 」 。
    ///
    /// ── and ─────────────────────────────────
    /// return is 「request up 」。Qoder will by 
    /// efforts 1 （`minimal` → `low`、unsupported default）， that 1 need need 
    /// directory up down ， not to —— is not is 1 
    /// （CatPaw and in `reasoning_patch` already ，none）。
    fn outbound_reasoning(&self, body: &Value) -> Option<String> {
        crate::server::core::model_rules::read_client_level(body)
            .filter(|level| !crate::server::core::model_rules::reasoning_is_off(level))
    }

    /// response is up error；default use HTTP 2xx 。
    ///
    /// up use HTTP 200 + application/json returnerror， can 
    /// in success to client before will has error、 and 。
    fn is_error_response(&self, status: u16, _headers: &HeaderMap) -> bool {
        !(200..300).contains(&status)
    }

    fn is_error_response_async<'a>(
        &'a self,
        status: u16,
        headers: &'a HeaderMap,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, GatewayError>> + Send + 'a>> {
        Box::pin(async move { Ok(self.is_error_response(status, headers)) })
    }

    /// up error（status + already error）。
    ///
    /// `error_body` is ** already 1 ** error for ： `code`（ up ，
    /// can can as null） and `message`（ up ， message/msg/error.message，
    /// JSON response then is after ）。 1 by （
    /// `upstream::request::read_upstream_error`）， as 「 1 HTTP error」
    /// is 、 and 1 none。
    fn classify_error(&self, status: u16, error_body: &Value) -> UpstreamErrorClass;

    /// [`Self::classify_error`] （add， by 
    /// [`Self::build_chat_request_async`]）。
    ///
    /// ── default ──────────────────────────────────────────
    /// is （ and ），。 need 
    /// （ ， not can ）。
    ///
    /// ── Note: this in errorpath up ────────────────────────────
    /// up will 1 。mustreturn（ not request）——
    /// that is 「 is 」 this has 。
    fn classify_error_async<'a>(
        &'a self,
        status: u16,
        error_body: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = UpstreamErrorClass> + Send + 'a>,
    > {
        Box::pin(async move { self.classify_error(status, error_body) })
    }

    fn error_policy_async<'a>(
        &'a self,
        _status: u16,
        _body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = super::policy::ErrorPolicy> + Send + 'a>> {
        Box::pin(async { super::policy::ErrorPolicy::default() })
    }

    /// will （`attempt_stateful`）failure after 。
    ///
    /// ── Why this must in ─────────────────────────────────
    /// will path error is 「 1 （Fatal ）： not 、 not 、
    /// not 」（ `provider_loop::attempt_stateful`）。「」 this is 
    /// **** default， not this ：CodeArts will has path， 
    /// （HTTP 200 SSE → 403） in this path up to not 
    /// [`Self::classify_error`]， after request from 1 
    /// 、 1 not 。 is provider （403 is 
    /// is ， has ）， by to 。
    ///
    /// ── Whydefault is Fatal ─────────────────────────────────────
    /// has will （CatPaw） as is ，default not ；
    /// has 「 will error has can 」 。
    /// Note this QuotaLimited ****（ after by 
    /// ）—— not will for 1 error use nonepath / new 。
    fn classify_conversation_error(&self, error: &GatewayError) -> UpstreamErrorClass {
        UpstreamErrorClass::Fatal {
            status: u16::try_from(error.status_code).unwrap_or(500),
            message: error.message.clone(),
            upstream_code: error.upstream_code,
        }
    }

    /// 1 need to ****。
    ///
    /// default has up 1 。 CodeArts is ****：
    /// when this （ down 1 request is 1 ） will 
    /// `insufficient quota`， 1 not down 1 ——
    /// this return。returnmust is ** up **（ and 
    /// `routing::CooldownKeys` 1 ）， `wire_model` is 
    /// 。
    ///
    /// `account_id` by directory use （current CodeArts directory is provider
    /// ， not use ； in down 1 this ）。
    fn quota_cooldown_models(&self, _account_id: &str, wire_model: &str) -> Vec<String> {
        vec![wire_model.to_string()]
    }

    /// available access token（ new ； new store）。
    ///
    /// `account_id` as empty「 no has 」： use default
    /// （ or current）。
    fn ensure_access_token<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    >;

    /// new directory。
    ///
    /// ── `force` is （ 1 as ）──────────────
    /// - `force = false`：**path** （`GET /v1/models` after new 、
    /// new ）。 has —— 10 minutes TTL is as 
    /// path ：client 1 up no need 。
    /// - `force = true`：** use 「 new 」** ，/TTL
    /// 、 1 up 。** this parameter not is can has can none **： use by down by 
    /// is 「 in 、 1 」， use path will in 
    /// TTL not return —— 「 new success」 1 no ，
    /// and can none。 this not this by ****， 
    /// to use （parameter）， not is in 。
    ///
    /// ── `account_id` is （ as ）───────────────
    /// use ** ** up directory（「get」
    /// 「」 down ）：
    /// - empty = this default： available（ and default use 1 ），
    /// no has when to / ；
    /// - empty = use that 。** by id **： not to return
    /// failureReason（" not in or unavailable"）， not to —— that will 
    /// 「 A、 use is B」 error。
    /// directory is （ not 、 can can can not ）， up 
    /// this 1 need can 、 can 。
    ///
    /// ── failure and return ──────────────────────────────────────────────
    /// new failure** not returnerror**：directory new is ，failure when has 
    /// （ and before `refresh_with_current_account` to 1 —— ）。
    /// return [`ModelRefreshOutcome`] is to **path** use 
    /// （path not ）：failure need in Reason，otherwise use new can to 
    /// 「 already new 」， its no 。
    fn refresh_models<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        force: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = ModelRefreshOutcome> + Send + 'a>,
    >;

    /// directory new is 「」 this 1 （default true；Cline as false）。
    ///
    /// 「get」 「」 down and ： for not 
    /// （directorynone、 is ）， 1 1 down is 。
    /// and [`Self::refresh_models`] `account_id` parameter for —— that ignoreparameter 
    /// ， this this false（ not `accountId`）。
    fn refresh_uses_account(&self) -> bool {
        true
    }

    /// provider is has ** already can **（ in is true）。
    ///
    /// ── this （， will in has false）─────
    /// Qoder in before return false（ when has can ），
    /// use is down this 「 and `list_models` 」。 that up after 
    /// in already none false ； this is as **** ——
    /// 「 this is empty 」 and 「 this no has can 」 is ， will has new provider
    /// （ up 、 after ） need need 。
    ///
    /// ── and `list_models` （Why not is as empty）──────────
    /// workbuddy and raccoon directory，failure or no when ** can can 
    /// as empty**， can 。 will this in that when 「 not can 」。
    /// is 1 ** can **（）， and `supports_usage` /
    /// `supports_refresh` 1 ； as is ， use （
    /// ） can use not `list_models` 。
    ///
    /// ：`account_store::pick_current`（ = `/api/session` 
    /// `currentAccountId`、 delete、 ★）。 no has can ，
    /// will 「current」、 and 「」 。
    fn supports_chat(&self) -> bool {
        true
    }

    /// provider is has ** can directory**（module header 9）。
    ///
    /// and `supports_refresh` 1 ：** up to has no has that **，
    /// not is 「 not 」。current** true**（ down is ，
    /// after `models.rs` module header）：
    /// ```text
    ///   WorkBuddy   GET  /v3/config
    /// GET {llmBase}/model_catalog
    ///   Qoder       GET  {gateway}algo/api/v2/model/list
    ///   CatPaw      POST /api/agent/maas/model-types
    ///   AutoClaw    GET  .../proxy/autoclaw-model-config
    /// ```
    /// default false is to 「 will new 、directory no 」 provider 。
    ///
    /// ── Why need has this （ not 「 new is empty」）──────────
    /// this is ** use can by **， for unsupported must，
    /// otherwise「supports not to new 」 will use as is bug。
    /// and `supports_usage` 1 。
    ///
    /// ── 1 （ after ）─────────────────────────────
    /// CatPaw and AutoClaw return false， by is 「 up no has directory、 is 」。
    /// that ****： 1 in ， is —— CatPaw 
    /// `tenant/scene/env` is POST body not is query（ query parameter），
    /// AutoClaw directory in host `/proxy/` 1 not is for use `/proxy/autoclaw`
    /// （ `chat/completions` not to ）。**「 this no has 」 
    /// need can is excluding **，otherwise is 1 no to 。
    fn supports_model_refresh(&self) -> bool {
        false
    }

    /// token up （401） after **** new （module header 2）。
    ///
    /// default `ensure_access_token`： for 「401 」 provider use 。
    fn refresh_access_token<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        self.ensure_access_token(store, account_id)
    }

    /// 「 can error」 ；None = none， use then 。
    ///
    /// `attempt` is ** already ** （0 failure、 no ）。
    ///
    /// `budget` is ** **（ can ）， by by 
    /// 「 1 」 that 1 to （ `config::RetrySettings`）。
    /// this not this —— not is set： that 
    /// 「 1 」 this then will 1 。
    fn retry_advice(&self, _error_body: &Value, _attempt: usize, _budget: usize) -> Option<RetryAdvice> {
        None
    }

    /// [`Self::retry_advice`] （ RPC）。
    ///
    /// ── Why need need ──────────────────────────────────────────
    /// no await， in 1 —— no has this ，
    /// to not provider （workbuddy 11-128 that ），
    /// can to use 。，。
    fn retry_advice_async<'a>(
        &'a self,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<RetryAdvice>> + Send + 'a>> {
        Box::pin(async move { self.retry_advice(error_body, attempt, budget) })
    }

    /// optional ；Stop request，Default use 。
    fn retry_decision_async<'a>(
        &'a self,
        _status: u16,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RetryDecision> + Send + 'a>> {
        Box::pin(async move {
            match self.retry_advice_async(error_body, attempt, budget).await {
                Some(advice) => RetryDecision::Retry {
                    delay_ms: advice.delay_ms,
                    reason: advice.reason,
                },
                None => RetryDecision::Default,
            }
        })
    }

    /// `finish_reason` ** up **（HTTP 200 success），defaultempty = not 。
    ///
    /// ── Why need need （ has up ）────────────────────────
    /// has up （ OfficeAce / as AgentArts ） 1 
    /// ：`HTTP 200`、SSE ， is 1 ，
    /// `choices[].finish_reason = content_filter`。 HTTP this is **success**
    /// —— not ，client （：ZCode agentic request to 
    /// ， `upstream::refusal` module header）。
    ///
    /// by ： 1 `UpstreamResponse`/， will 
    /// this `finish_reason` as 1 ** can failure** —— has /，
    /// this 1 when to client 1 ** can error**（ not is that up ）。
    ///
    /// ── Whydefaultempty、 and ──────────────────────────────────
    /// defaultempty = ** not `finish_reason`**， not 。 is ：
    /// 1 ，allrequest will in ** has ** 1 
    /// （ `upstream::refusal`，limit 32 KiB）—— 1 。
    /// this in up ， and in limit ⇒ for in 。
    ///
    /// ⚠️ this 「 up will any」：`length`（） is ****
    /// ， not this ； has 「 up not to this 」 。
    fn stream_refusal_finishes(&self) -> &'static [&'static str] {
        &[]
    }

    /// 「****」： up when ， use 。
    ///
    /// ── Why need 1 by ────────────────────────────
    /// has up ** not in `finish_reason` up **： `200` + 1 SSE
    /// （ is ），`finish_reason` not in ， after `[DONE]`。
    /// ：AtomCode CodingPlan usage after is this （2026-10-18 in dev up 
    /// ，client 426 ，
    /// 「「xxx」 in current CodingPlan usage already limit。current in 
    /// HH:MM and ， after 。」）。 and `finish_reason` ，
    /// this not is failure not —— is 1 **quota** 1 success，
    /// not 、 not ， down 1 。
    ///
    /// ── in after and `finish_reason` that ** not **（）───────
    /// `stream_refusal_finishes` is ****： that is 、 will ，
    /// is 「 not 、」。
    /// is ** **：must****（ by up to when 
    /// ， `rotate::mark_account_limited` ）+ 。
    ///
    /// is ****（、 by for ）—— that ，
    /// 「 can can 」。defaultempty = not ， not 。
    fn stream_refusal_markers(&self) -> &'static [&'static str] {
        &[]
    }

    /// provider is supports**** new （module header 5）。
    ///
    /// default false = this no has （ no has refreshToken、 up no has new ）。
    /// is 「 up to has no has that 」， not is 「 not 」：
    /// supports new not ， will 1 and failure。
    ///
    /// and `refresh_access_token` ： after is 「」（401 after ），
    /// is 「 can not can 」（ can ）。 this 1 ——
    /// for failure use new will 。
    fn supports_refresh(&self) -> bool {
        false
    }

    /// this is **expired or **（ need need new ）（module header 5）。
    ///
    /// is ：workbuddy will `expiresAt`、 JWT 
    /// `exp`、AutoClaw `expiresAt`； ** not **
    /// （5 minutes / 5 minutes / 5 minutes）， not can in this 1 1 。
    ///
    /// ── Why「 not 」 1 return false ────────────────────────
    /// no has expiration time（opaque token、JWT no has `exp`） or not in when return false：
    /// none from 「 need need new 」 will 1 up ， that request
    /// need （token ）， need failure（ no has refreshToken）。
    /// this can etc 401 when that new 。
    ///
    /// is Cline and AutoClaw： in this up 1 ****
    /// （Cline by 25 minutes、AutoClaw by when ），「 not expiration time」
    /// or 「」 can to —— adapter 。
    ///
    /// ** not in after await**： is ，return；
    /// by [`Self::refresh_access_token`] in 。
    fn credentials_expiring(&self, _store: &AccountStore, _account_id: &str) -> bool {
        false
    }

    /// [`Self::credentials_expiring`] （add， by 
    /// [`Self::build_chat_request_async`]）。
    ///
    /// ── default ──────────────────────────────────────────
    /// return； in ， need 。
    ///
    /// ── 「 not false」 this not ────────────────────────────
    /// must：none from 「 need need new 」 will 1 
    /// and failure（ ）。
    fn credentials_expiring_async<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + 'a>> {
        Box::pin(async move { self.credentials_expiring(store, account_id) })
    }

    /// provider is 「 1 no has when use default」
    /// （module header 3；workbuddy `WORKBUDDY_TOKEN` ）。
    fn allows_anonymous_default_session(&self) -> bool {
        false
    }

    /// when ， is ****（default false = ）。
    ///
    /// ── defaultWhy is ────────────────────────────────────
    /// by config， not up `HTTPS_PROXY` / 
    /// （ `egress::build_client` ）—— otherwise up 「」
    /// ， and use to not 。
    ///
    /// ── will true ─────────────────────────────────────────
    /// up in ** has can ** ：Antigravity（Google）。
    /// use up 「 already can Google that 」 in set（ can 
    /// is ）， if no ， is 1 default ——
    /// no when etc ， not new failure。 need in module header
    /// this use （ ）。
    ///
    /// **** client； no has can ，
    /// `egress::client_for_system_proxy()`（ `upstream::request` 1 ）。
    fn system_proxy_when_unset(&self) -> bool {
        false
    }

    /// provider ** in is in **（module header 3 ）。
    ///
    /// and `allows_anonymous_default_session` ： that 「 this supports this 
    /// 」（ can ）， this 「 has no has that 」（ when ）。
    /// directory need use after 「 this in not has available」—— before will 
    /// 1 no `/v1/models` 「available」。
    ///
    /// default false（ not ）。「 in and empty」 ， not 。
    fn env_credentials_present(&self) -> bool {
        false
    }

    /// provider is has 「default」（module header 4）。
    ///
    /// （ §4.4 ）：client** `model`** when ，
    /// config.json `defaultModel`（ that is workbuddy ） —— in 
    /// 「 this default in provider has this 」 when ；
    /// not by 「」， up use default。
    /// otherwise will 「 1 A 、request by to B 」。
    fn supports_default_model(&self) -> bool {
        false
    }

    /// SSE down need not need `model` clientrequest （module header 4）。
    ///
    /// default false（workbuddy up client to ）。 up will 
    /// ， true —— `upstream::sse` 
    /// `model_rewrite` parameter and 。
    fn sse_model_rewrite(&self) -> bool {
        false
    }

    /// SSE down need not need 「 has 」 content （module header 11）。
    ///
    /// default false（ not ）。 before true has WorkBuddy and 
    /// AutoClaw —— that up will in as 
    /// ，client by markdown 1 1 。
    /// and [`crate::server::core::upstream::sse::is_newline_keepalive`]；
    /// ** not in this **：「 need not need 」， in `upstream::sse`
    /// this SSE 1 ， and [`Self::sse_model_rewrite`] 1 。
    fn sse_strip_newline_chunks(&self) -> bool {
        false
    }

    /// provider up is as ** has （conversation ）**（module header 6，
    /// §4.2.1）。
    ///
    /// default false（ 1 HTTP request = 1 for ）。CatPaw true： up is 
    /// round → event(running) → turn(SSE) → → event(completed) 
    /// will （§9），request not 。 to adapter 
    /// ** will ** [`Self::forward_conversation`]； and nonepath 1 
    /// `ForwardOutcome`， is `chat.rs` its （、、error）。
    ///
    /// true is will （CatPaw / Qoder / Accio /
    /// CodeArts / Trae / Kuku / MonkeyCode， `mod.rs` module header）。
    fn is_stateful(&self) -> bool {
        false
    }

    /// ** will **（module header 6 ； §4.2.1）。
    ///
    /// has has provider （`is_stateful` as true ， `mod.rs`）。
    /// defaultreturn 503： to this 
    /// 1 none provider has —— that is **error**，
    /// `build_chat_request` and `forward_conversation` 1 。
    ///
    /// ── and `build_chat_request` ──────────────────────────
    /// nonepath is 「request → → error → by 」；
    /// has path this （CatPaw is round/turn/ + 
    /// ），** use ** that ：、、
    /// provider 、telemetry （ `upstream::provider_loop` ）。
    ///
    /// ── parameter（ provider use ， not is CatPaw ）───────────
    /// - `store`：（；** not await** by ）；
    /// - `account_id`： id（empty = no ， use default）；
    /// - `body`：clientrequest（****， 1 is ）；
    /// - `client_headers`：client（ will `x-session-id` in this ）；
    /// - `proxy`：（ already by ，used for）；
    /// - `stream`：client is need （ `ForwardOutcome` ）；
    /// - `telemetry`：usage / （ by need ， can ignore）。
    fn forward_conversation<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
        _body: &'a Value,
        _client_headers: &'a HeaderMap,
        _proxy: Option<crate::server::core::proxies::ResolvedProxy>,
        _stream: bool,
        _telemetry: &'a std::sync::Arc<crate::server::core::upstream::usage::RequestTelemetry>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<crate::server::core::upstream::ForwardOutcome, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        let kind = self.kind();
        Box::pin(async move {
            Err(GatewayError::with_status(
                503,
                format!(
                    "该提供商不支持会话式转发（{}）：这是内部契约错误，请检查 is_stateful 的分流",
                    super::kind_id(kind)
                ),
            ))
        })
    }
    /// provider is supports「、 by 」 this ****
    /// （module header 7）。
    ///
    /// default false。current** has ** true： in success after will 
    /// to 1 （`office-raccoon://auth/callback?code=…`），
    /// to that 1 code can to 。CatPaw / AutoClaw no has this 
    /// （ after can from auth.json ），default。
    ///
    /// workbuddy ****default false： 「」 is up `auth/state`
    /// that none（`core::login`）， and this is 。 true
    /// will by by `build_login_url`→None， 1 can use can 400。
    fn supports_web_login(&self) -> bool {
        false
    }

    /// ，return `(auth_url, state)`（module header 7）。
    ///
    /// `state` by and （ not ）： is CSRF 1 ， is 
    /// when 「 this by 」 1 ，**must** use not can 
    /// ， and need for 。
    ///
    /// default None = unsupported（ and [`Self::supports_web_login`] default 1 ）。
    fn build_login_url(&self) -> Option<(String, String)> {
        None
    }

    /// [`Self::build_login_url`] （add）。
    ///
    /// ── Why need need ────────────────────────────────────────────
    /// need 1 RPC（`login.start`）， not can await。
    /// and [`Self::build_chat_request_async`] 1 ：default
    /// （）， has 。
    fn build_login_url_async<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<(String, String)>> + Send + 'a>,
    > {
        Box::pin(async move { self.build_login_url() })
    }

    /// （****，add）。
    ///
    /// ── Why need need ， not is use `build_login_url_async` ──────────
    /// `(url, state)` this 「 etc 」 this 。 has ：
    /// - ****：（ or 、 by down to ），
    /// etc `login.callback` to （`poll_interval_ms == 0`）；
    /// - ****： in up 、** not to **（AtomCode /
    /// OfficeAce / ZCode this ）， can by up 「 no 」
    ///     （`poll_interval_ms > 0`）。
    ///
    /// this after ， will 「、 1 etc 1 
    /// not to 」—— this is 1 failure（ no has ）。
    ///
    /// default from `(url, state)` 1 ****， and 
    /// already `build_login_url_async` 。
    fn login_start_async<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<agent2api_provider_protocol::LoginStart>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let (url, state) = self.build_login_url_async().await?;
            Some(
                agent2api_provider_protocol::LoginStart {
                    flow: state,
                    url,
                    instructions: String::new(),
                    // 0 = （ etc `login.callback` to ）
                    poll_interval_ms: 0,
                    external_browser: false,
                    allowed_hosts: Vec::new(),
                },
            )
        })
    }

    /// optional（ not add）。
    /// `auth.methods`： old return 404 / 501 when to None， old not 。
    /// return not can ； by must， not can HTML or 。
    fn login_methods_metadata<'a>(
        &'a self,
        store: &'a AccountStore,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Value>, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            match self.admin_action(store, "", "auth.methods", &serde_json::json!({})).await {
                Ok(metadata) => Ok(Some(metadata)),
                Err(error) if matches!(error.status_code, 404 | 501) => Ok(None),
                Err(error) => Err(error),
            }
        })
    }

    /// parameter：options is already `{methodIndex, inputs, proxy}`。
    /// default old ； options `login.start.payload`，
    /// as old SDK not will methodIndex / inputs / proxy to provider。
    fn build_login_flow_with_options<'a>(
        &'a self,
        _options: Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<agent2api_provider_protocol::LoginStart>, GatewayError>>
                + Send + 'a,
        >,
    > {
        Box::pin(async move { Ok(self.login_start_async().await) })
    }

    /// 1 （** use **，add）。
    ///
    /// has `poll_interval_ms` `> 0` when will use 。defaultreturnerror：
    /// to default（ 1 is no ）—— 
    /// return pending to 1 5 minutes when 。
    fn login_poll_async<'a>(
        &'a self,
        _flow: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<agent2api_provider_protocol::LoginPoll, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async {
            Err(GatewayError::with_status(
                501,
                "该提供商没有实现登录轮询：这是内部契约错误，请检查 poll_interval_ms 的分流",
            ))
        })
    }

    /// 1 already ，return id。
    ///
    /// ── Why 1 （ not is use ）───────────────
    /// then is ** this **：id identity is when 、
    /// `public.name` is label、 `pluginSecret` is 
    /// —— has 。 to `NormalizedAccount` after must this ，
    /// otherwise will and that （ [`Self::exchange_login_code`]，
    /// already ）。
    ///
    /// default 501： ， not will to this 。
    fn persist_login_account(
        &self,
        _account: &agent2api_provider_protocol::NormalizedAccount,
    ) -> Result<String, GatewayError> {
        Err(GatewayError::with_status(
            501,
            "该提供商不支持直接落库登录结果：这是内部契约错误",
        ))
    }

    /// use 1 `code` and ****，successreturn id
    /// （module header 7）。
    ///
    /// `state` must and [`Self::build_login_url`] return that 1 —— need in this 
    /// 1 （： use up already ， in 「 has 
    /// this 」 can ）。
    ///
    /// must** has ** path（ is `add_raccoon_account`），
    /// not 1 —— otherwise「」 and 「」 will in 
    /// 、id 、 up 。
    ///
    /// defaultreturn 501： to this by 1 unsupported provider ，
    /// that is error， not 。
    fn exchange_login_code<'a>(
        &'a self,
        _store: &'a AccountStore,
        _code: &'a str,
        _state: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        let kind = self.kind();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!(
                    "该提供商不支持网页登录（{}）：这是内部契约错误，请检查 supports_web_login 的分流",
                    super::kind_id(kind)
                ),
            ))
        })
    }

    /// not ： URL、 and Pending 。
    /// default None has `exchange_login_code`； after in Done when 
    /// by use `persist_login_account`，Pending already 、 etc 。
    fn login_callback_poll_async<'a>(
        &'a self,
        _store: &'a AccountStore,
        _callback: agent2api_provider_protocol::LoginCallback,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<agent2api_provider_protocol::LoginPoll>, GatewayError>>
                + Send + 'a,
        >,
    > {
        Box::pin(async { Ok(None) })
    }

    /// （ and ）， not delete already 。
    /// defaultnone old ； ， to Done 。
    fn login_cancel_async<'a>(
        &'a self,
        _flow: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), GatewayError>> + Send + 'a>,
    > {
        Box::pin(async { Ok(()) })
    }

    /// provider is supports** / **（module header 8）。
    ///
    /// default false。supports has ， down as ：
    /// - workbuddy： 「」（`core::billing`， before 1 has 1 ）；
    /// - ： + （`raccoon/balance.rs`）；
    /// - CatPaw： credit ，** need need config will token2**
    /// （`catpaw/balance.rs`，not configured when return can 「not configured」 not is failure）；
    /// - AutoClaw： + （`autoclaw/balance.rs`）。
    ///
    /// Why this not is 「 can 」： before by up need not need 「」 by 、
    /// need not need this 。 1 unsupported to ， use down will to 
    /// 1 failure —— and `supports_refresh` 1 （ can ）。
    fn supports_usage(&self) -> bool {
        false
    }

    /// / ，return** 1 after JSON**（module header 8）。
    ///
    /// ── （`api::accounts` `{results:[{id,name,usage,error}]}` 
    /// that `usage` ）────────────────────────────────────
    /// new （raccoon / autoclaw） down this 1 ，
    /// **workbuddy **： `query_credits_summary` is before has has 
    /// （`{kind, unlimited, totalLeft, planLeft, bonusLeft}`）， before 
    /// 1 by ， for workbuddy has 、 not ——
    /// 1 will that （ this is ）。
    /// ```jsonc
    /// {
    /// "available": 1234.5, // availabletotal（； not to null）
    /// "unit": "", // （ before use ）
    /// "wallets": [ // ， can as empty
    /// { "type": "daily_points", "displayName": "daily", "balance": 800 }
    ///   ],
    /// "subscription": { // can empty
    ///     "planName": "…", "status": "…", "expireAt": 1789…,
    ///     "remainQuota": 100, "totalQuota": 1000
    ///   },
    /// "raw": { } // response（ use ； before default not ）
    /// }
    /// ```
    /// up that has **optional**， by 、 before by 
    /// （ not by provider ）：`availableView`（ ）、`balanceView`
    /// （ ， when that ）、`credits[]`（）、
    /// `windows[]`（， OpenCode 5 when / / ，
    /// 1 ）。 not this is up that 。
    ///
    /// ── （ use by this ）────────────────────────────────
    /// 1. ** not HTTP error **：return `Err` 「 this failure」，
    /// by use message `error` （ not failure in ）；
    /// 2. ** when by **： is HTTP，`egress` default
    /// read_timeout is 600 seconds（ to SSE use ）， not total when will 1 
    /// before to （ to 15~20 seconds）；
    /// 3. **401 by **（`GatewayError::with_status(401, …)`）：
    /// use 「 new after 1 」 has ；
    /// 4. not configured this ** use can before **， use 
    /// [`usage_not_configured`] that 400（
    /// [`USAGE_NOT_CONFIGURED_CODE`] ）—— before in 
    /// 「not configured」 not is failure。
    fn query_usage<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        let kind = self.kind();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!(
                    "该提供商不支持余额查询（{}）：这是内部契约错误，请检查 supports_usage 的分流",
                    super::kind_id(kind)
                ),
            ))
        })
    }

    /// 1 「」 1 can （add）。
    ///
    /// ── Why is has ──────────────────────────────
    /// path is `account_store::add_*_account` that ****
    /// （ 1 ）， trait up from not need need this 。 
    /// is not JSON， has 、、 need not need 
    /// —— is must to 。
    ///
    /// ── defaultreturn 501 ────────────────────────────────────────
    /// not this to （`api::accounts` by 「 has no has 」）； to 
    /// ，return 1 not for 。
    fn normalize_account<'a>(
        &'a self,
        _store: &'a AccountStore,
        _input: agent2api_provider_protocol::NormalizeInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<agent2api_provider_protocol::NormalizedAccount, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        let label = self.label();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!("{label}不支持通过网关添加账号：这是内部契约错误，请检查分流"),
            ))
        })
    }

    /// daily to / daily（add ）。
    ///
    /// ── and ────────────────────────────────────
    /// to in `core::billing::checkin::checkin_for`（ 1 
    /// `match provider_id`， 1 up ）。 in ，
    /// that match not this 21 —— this 。
    ///
    /// ── returnWhy is `{success, msg}` ────────────────────────
    /// and `claim_*` return** 1 **（`billing::checkin` 
    /// `claim_result` this ）。 `CheckinReport` this 
    /// （`RpcAdapter` ）， up not 「 / 」。
    ///
    /// default 501： ， to 。
    fn checkin_claim<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        let label = self.label();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!("{label}不支持插件签到入口：这是内部契约错误，请检查分流"),
            ))
        })
    }

    fn checkin_claim_with_options<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        _manual: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        self.checkin_claim(store, account_id)
    }

    fn plugin_account_view<'a>(
        &'a self,
        _secret: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async { Err(GatewayError::with_status(501, "不支持插件账号视图")) })
    }

    /// （ new / when ）。
    ///
    /// `claim` is （；empty = 1 can ）。
    /// return by ， and `claim_all` （`api::onboarding`
    /// to before ）。
    fn claim<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
        _claim: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        let label = self.label();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!("{label}不支持领取入口：这是内部契约错误，请检查分流"),
            ))
        })
    }

    /// ****（`api::onboarding::status` ）。
    ///
    /// `refresh = true` is use 「」（ 1 ）。
    /// default 502 not is 501： use in up will 「 this no has new 」，
    /// 1 「unsupported」return 1 empty not 「」。
    fn claim_status<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
        _refresh: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        let label = self.label();
        Box::pin(async move {
            Err(GatewayError::with_status(502, format!("{label}没有新手任务")))
        })
    }

    /// 「 has daily to 」 can （`None` = ， has ）。
    ///
    /// ── Whyreturn `Option<bool>` not is `bool` ────────────────
    /// is has to ** not is ** ： by `auto_checkin::CHECKIN_PROVIDERS`
    /// and `billing::checkin::supports_checkin` that （ edition / isGuest etc 
    /// ）。 to return `false` will that 。
    /// no has that ， can in —— has to `Some`。
    fn plugin_checkin_capability(&self) -> Option<bool> {
        None
    }

    /// ** **（ / daily / / …）。
    ///
    /// ── Why is 1 ────────────────────────────────────────
    /// this no has （ has is 、 has is 、 has is ），
    /// parameter and return has 。 to 1 trait ，trait will 
    /// none， not this —— 。
    /// `methods::gateway::ADMIN_ACTION` 。
    ///
    /// ── default to 501 ──────────────────────────────────────────
    /// and `claim_status` 502 not ： that is 「 this up no has new 」，
    /// this is ** old **（ new + ）—— 501 use can 
    /// 「 this no this can 」 and 「 this no 」。
    fn admin_action<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
        action: &'a str,
        _payload: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        let label = self.label();
        Box::pin(async move {
            Err(GatewayError::with_status(
                501,
                format!("{label}没有实现管理动作「{action}」（可能需要更新提供商插件）"),
            ))
        })
    }
}

/// 「 this no has config」 ** can **（`GatewayError::code`）。
///
/// ── Why need 1 code not is before ───────────────────
/// before need this in 「not configured」， its failureerror。
/// by will in 1 after （ 1 ）， by will and 
/// 400 parametererror in 1 —— 1 is this 1 can 。
pub const USAGE_NOT_CONFIGURED_CODE: &str = "usage_not_configured";

/// 「 this no has config」 1 （`GatewayError::code` as 
/// [`USAGE_NOT_CONFIGURED_CODE`]）。
///
/// ── Why this need use 1 ， not is ─────────────────
/// before by `code`（ not is ）「 this is not configured、 not is failure」，** will 
/// in up **； 1 will 1 「set 1 down 」 in not 
/// up not 。 will to this is 「 need 1 」 
/// （ CatPaw will `token2`、ZCode Coding Plan JWT —— 
/// `balance.rs`）。
///
/// `field_hint` （「set 」）， message；
/// use **400**： this is before 、 use can ， not is server ——
/// before in not is error。
pub fn usage_not_configured(provider_label: &str, field_hint: &str) -> GatewayError {
    GatewayError::with_status(
        400,
        format!(
            "该账号没有余额查询凭证（{provider_label} 的 {field_hint}），\
             请在账号设置里配置后再查询"
        ),
    )
    .with_code(USAGE_NOT_CONFIGURED_CODE)
}

/// ：provider → 。
///
/// ── this in has 1 ：return ──────────────────────────────
/// 29 `ProviderKind` **** to （`agent2api-providers`，
/// not ）。 no has / can return，
/// for kind return [`missing_plugin::placeholder`] —— 「 need 
/// 」， not is in 1 will when 。
///
/// this （ not is use ） is as is ** 1 **：
/// will if need ， in this ；，
/// 「 up 」 ，must this —— review when 1 can 。
///
/// ：** not need ** in this 。 when is 1 
/// （ `missing_plugin` module header）， not is 。
pub fn adapter_for(kind: ProviderKind) -> &'static dyn ProviderAdapter {
    crate::server::core::providers::missing_plugin::placeholder(kind)
}

/// when ：provider id → （add）。
///
/// ── and [`adapter_for`] ───────────────────────────────
/// `adapter_for` by ****，return `&'static`，O(1) and not failure。
/// no has `ProviderKind`（ provider id is ），
/// can by this 。
///
/// ── Whyreturn is `Option` ────────────────────────────────
/// is ** use ** ： no that 、or disabled，provider not in 。
/// this is ，must —— not can that 「 1 has 」。
///
/// ── ： → old Kind ─────────────────────────────
/// 1 id when and when ，。 by ：
/// 1. already ：21 has ， that 「 after 
/// 」 this 1 use —— etc not ；
/// 2. use new this use new （ and ， that is 
/// 1 ）； will 「」 this invalid；
/// 3. path： / disabled after `lookup` return `None`，
/// （ by 
/// `account_store::plugin_accounts` can ）。
///
/// ── need can **** to ──────────────────────────
/// [`builtin_adapter_by_id`]： need need 「 1 that 」 use 
/// （ for 、 when ） use ， not is this 。
pub fn adapter_by_id(id: &str) -> Option<Arc<dyn ProviderAdapter>> {
    if super::availability::is_suspended(id) { return None; }
    // （ up 3 ： no /disabled when to ）
    if let Some(adapter) = crate::server::core::plugin_host::lookup(id) {
        return Some(adapter);
    }
    // 
    super::kind_from_id(id).map(Into::into)
}

pub async fn adapter_by_id_ready(id: &str) -> Result<Option<Arc<dyn ProviderAdapter>>, GatewayError> {
    if super::availability::is_suspended(id) { return Ok(None); }
    if let Some(adapter) = crate::server::core::plugin_host::lookup_ready(id).await? {
        return Ok(Some(adapter));
    }
    Ok(super::kind_from_id(id).map(Into::into))
}

/// ****（）。
///
/// ── when this use ──────────────────────────────────────────
/// need need 「 that 1 in 」 ： 、 when 
/// for 、 and 「 this has no has 」 。
/// path 1 use [`adapter_by_id`]（）。
pub fn builtin_adapter_by_id(id: &str) -> Option<Arc<dyn ProviderAdapter>> {
    super::kind_from_id(id).map(Into::into)
}

/// `Arc` （ `adapter_by_id` return 1 ）。
///
/// ── Why need need this ──────────────────────────────────────
/// `adapter_for` return `&'static dyn`，return `Arc<dyn>`。 need 
/// 1 return， can before `Arc` —— this not ，
/// to （ is 1 match + 1 use ， and 
/// `adapter_for` ）。
struct BuiltinAdapter(ProviderKind);

impl From<ProviderKind> for Arc<dyn ProviderAdapter> {
    fn from(kind: ProviderKind) -> Self {
        Arc::new(BuiltinAdapter(kind))
    }
}

impl ProviderAdapter for BuiltinAdapter {
    // ── Why use not is 34 ──────────────────────────
    // 34 （ 1 will to trait default，
    // as ）。 「 use 」 and 「parameter」， when 
    // 1 。
    fn kind(&self) -> ProviderKind {
        self.0
    }

    fn list_models(&self) -> Vec<Value> {
        adapter_for(self.0).list_models()
    }

    fn advertise_models(&self, store: &AccountStore, manifest: Vec<Value>) -> Vec<Value> {
        adapter_for(self.0).advertise_models(store, manifest)
    }

    fn build_chat_request(
        &self,
        account: &Value,
        body: &Value,
        client_headers: &HeaderMap,
    ) -> Result<ChatRequestPlan, GatewayError> {
        adapter_for(self.0).build_chat_request(account, body, client_headers)
    }

    fn build_chat_request_async<'a>(
        &'a self,
        account: &'a Value,
        body: &'a Value,
        client_headers: &'a HeaderMap,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).build_chat_request_async(account, body, client_headers)
    }

    fn reasoning_patch(&self, level: &str, model: &str, body: &Value) -> ReasoningPatch {
        adapter_for(self.0).reasoning_patch(level, model, body)
    }

    fn reasoning_patch_async<'a>(
        &'a self,
        level: &'a str,
        model: &'a str,
        body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ReasoningPatch> + Send + 'a>> {
        adapter_for(self.0).reasoning_patch_async(level, model, body)
    }

    fn outbound_reasoning(&self, body: &Value) -> Option<String> {
        adapter_for(self.0).outbound_reasoning(body)
    }

    fn is_error_response(&self, status: u16, headers: &HeaderMap) -> bool {
        adapter_for(self.0).is_error_response(status, headers)
    }

    fn is_error_response_async<'a>(
        &'a self,
        status: u16,
        headers: &'a HeaderMap,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, GatewayError>> + Send + 'a>> {
        adapter_for(self.0).is_error_response_async(status, headers)
    }

    fn classify_error(&self, status: u16, error_body: &Value) -> UpstreamErrorClass {
        adapter_for(self.0).classify_error(status, error_body)
    }

    fn classify_error_async<'a>(
        &'a self,
        status: u16,
        error_body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = UpstreamErrorClass> + Send + 'a>> {
        adapter_for(self.0).classify_error_async(status, error_body)
    }

    fn classify_conversation_error(&self, error: &GatewayError) -> UpstreamErrorClass {
        adapter_for(self.0).classify_conversation_error(error)
    }

    fn quota_cooldown_models(&self, account_id: &str, wire_model: &str) -> Vec<String> {
        adapter_for(self.0).quota_cooldown_models(account_id, wire_model)
    }

    fn ensure_access_token<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).ensure_access_token(store, account_id)
    }

    fn refresh_models<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        force: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ModelRefreshOutcome> + Send + 'a>> {
        adapter_for(self.0).refresh_models(store, account_id, force)
    }

    fn refresh_uses_account(&self) -> bool {
        adapter_for(self.0).refresh_uses_account()
    }

    fn supports_chat(&self) -> bool {
        adapter_for(self.0).supports_chat()
    }

    fn supports_image_generation(&self) -> bool {
        adapter_for(self.0).supports_image_generation()
    }

    fn supports_image_editing(&self) -> bool {
        adapter_for(self.0).supports_image_editing()
    }

    fn build_image_request_async<'a>(
        &'a self,
        account: &'a Value,
        body: &'a Value,
        client_headers: &'a HeaderMap,
        operation: super::images::ImageOperation,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).build_image_request_async(account, body, client_headers, operation)
    }

    fn supports_model_refresh(&self) -> bool {
        adapter_for(self.0).supports_model_refresh()
    }

    fn refresh_access_token<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).refresh_access_token(store, account_id)
    }

    fn retry_advice(&self, error_body: &Value, attempt: usize, budget: usize) -> Option<RetryAdvice> {
        adapter_for(self.0).retry_advice(error_body, attempt, budget)
    }

    fn retry_advice_async<'a>(
        &'a self,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<RetryAdvice>> + Send + 'a>> {
        adapter_for(self.0).retry_advice_async(error_body, attempt, budget)
    }

    fn retry_decision_async<'a>(
        &'a self,
        status: u16,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RetryDecision> + Send + 'a>> {
        adapter_for(self.0).retry_decision_async(status, error_body, attempt, budget)
    }

    fn stream_refusal_finishes(&self) -> &'static [&'static str] {
        adapter_for(self.0).stream_refusal_finishes()
    }

    fn stream_refusal_markers(&self) -> &'static [&'static str] {
        adapter_for(self.0).stream_refusal_markers()
    }

    fn supports_refresh(&self) -> bool {
        adapter_for(self.0).supports_refresh()
    }

    fn credentials_expiring(&self, store: &AccountStore, account_id: &str) -> bool {
        adapter_for(self.0).credentials_expiring(store, account_id)
    }

    fn credentials_expiring_async<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + 'a>> {
        adapter_for(self.0).credentials_expiring_async(store, account_id)
    }

    fn allows_anonymous_default_session(&self) -> bool {
        adapter_for(self.0).allows_anonymous_default_session()
    }

    fn system_proxy_when_unset(&self) -> bool {
        adapter_for(self.0).system_proxy_when_unset()
    }

    fn env_credentials_present(&self) -> bool {
        adapter_for(self.0).env_credentials_present()
    }

    fn supports_default_model(&self) -> bool {
        adapter_for(self.0).supports_default_model()
    }

    fn sse_model_rewrite(&self) -> bool {
        adapter_for(self.0).sse_model_rewrite()
    }

    fn sse_strip_newline_chunks(&self) -> bool {
        adapter_for(self.0).sse_strip_newline_chunks()
    }

    fn is_stateful(&self) -> bool {
        adapter_for(self.0).is_stateful()
    }

    fn forward_conversation<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        body: &'a Value,
        client_headers: &'a HeaderMap,
        proxy: Option<crate::server::core::proxies::ResolvedProxy>,
        stream: bool,
        telemetry: &'a std::sync::Arc<crate::server::core::upstream::usage::RequestTelemetry>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<crate::server::core::upstream::ForwardOutcome, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        adapter_for(self.0).forward_conversation(
            store,
            account_id,
            body,
            client_headers,
            proxy,
            stream,
            telemetry,
        )
    }

    fn supports_web_login(&self) -> bool {
        adapter_for(self.0).supports_web_login()
    }

    fn build_login_url(&self) -> Option<(String, String)> {
        adapter_for(self.0).build_login_url()
    }

    fn build_login_url_async<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<(String, String)>> + Send + 'a>,
    > {
        adapter_for(self.0).build_login_url_async()
    }

    fn login_start_async<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Option<agent2api_provider_protocol::LoginStart>>
                + Send
                + 'a,
        >,
    > {
        adapter_for(self.0).login_start_async()
    }

    fn login_methods_metadata<'a>(
        &'a self,
        store: &'a AccountStore,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<Value>, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).login_methods_metadata(store)
    }

    fn build_login_flow_with_options<'a>(
        &'a self,
        options: Value,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<agent2api_provider_protocol::LoginStart>, GatewayError>>
                + Send + 'a,
        >,
    > {
        adapter_for(self.0).build_login_flow_with_options(options)
    }

    fn login_callback_poll_async<'a>(
        &'a self,
        store: &'a AccountStore,
        callback: agent2api_provider_protocol::LoginCallback,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<agent2api_provider_protocol::LoginPoll>, GatewayError>>
                + Send + 'a,
        >,
    > {
        adapter_for(self.0).login_callback_poll_async(store, callback)
    }

    fn login_cancel_async<'a>(
        &'a self,
        flow: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).login_cancel_async(flow)
    }

    fn login_poll_async<'a>(
        &'a self,
        flow: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<agent2api_provider_protocol::LoginPoll, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        adapter_for(self.0).login_poll_async(flow)
    }

    fn persist_login_account(
        &self,
        account: &agent2api_provider_protocol::NormalizedAccount,
    ) -> Result<String, GatewayError> {
        // in `start_*_login` ， not to this 
        adapter_for(self.0).persist_login_account(account)
    }

    fn normalize_account<'a>(
        &'a self,
        store: &'a AccountStore,
        input: agent2api_provider_protocol::NormalizeInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<agent2api_provider_protocol::NormalizedAccount, GatewayError>,
                > + Send
                + 'a,
        >,
    > {
        adapter_for(self.0).normalize_account(store, input)
    }

    fn exchange_login_code<'a>(
        &'a self,
        store: &'a AccountStore,
        code: &'a str,
        state: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).exchange_login_code(store, code, state)
    }

    fn checkin_claim<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).checkin_claim(store, account_id)
    }

    fn claim<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        claim: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).claim(store, account_id, claim)
    }

    fn claim_status<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        refresh: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).claim_status(store, account_id, refresh)
    }

    fn plugin_checkin_capability(&self) -> Option<bool> {
        adapter_for(self.0).plugin_checkin_capability()
    }

    fn admin_action<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        action: &'a str,
        payload: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).admin_action(store, account_id, action, payload)
    }

    fn supports_usage(&self) -> bool {
        adapter_for(self.0).supports_usage()
    }

    fn query_usage<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        adapter_for(self.0).query_usage(store, account_id)
    }
}

/// already **** for kind （ = ）。
///
/// ── in as empty， and this is **** ────────────────────────────
/// this is 「 」。 and 
/// already to （`agent2api-providers`， not ），
/// is 「 1 not 」。
///
/// empty not is ： （`catalog_refresh` new and 
/// `reschedule`） is 「」 ， will that 
/// `if false` ， not 1 empty。** not need this **
/// —— 1 ， etc this new ， is need 。
///
/// not this ：`catalog_refresh` has `plugin_host::installed_ids()` 
/// ， and use 1 「supports」 。
pub fn implemented_kinds() -> Vec<ProviderKind> {
    Vec::new()
}

/// 1 directory new （module header 9）。
///
/// ── Why not use `core::models::RefreshOutcome` ────────────────────
/// that is **workbuddy ** has （`{refreshed, count, source, reason}`，
/// and Node for ）， in `ModelCatalog` up 。 need not 
/// （ 「TTL 」、 no has directory 「」 etc ）， will this 
/// none `source`/`reason`， need `core::models` has 。 is **
/// **：workbuddy in `RefreshOutcome` can 。
///
/// ── （ and `credential_maintenance` 1 ）──
/// - `refreshed = true`： to new （`count` is after ）。
/// - otherwise `message = None`：** no **（TTL to / up no to available / ）。
/// 「 no 」 not etc 「failure」：path is 「 has use 」。
/// - otherwise `message = Some(..)`：**failure**（）。 by has 
/// as （ not empty）， use this to Why。
///
/// `count` in `refreshed = true` when has （ its as 0）。
#[derive(Clone, Debug, Default)]
pub struct ModelRefreshOutcome {
    /// is 1 new 
    pub refreshed: bool,
    /// after directory（`refreshed = true` when valid）
    pub count: usize,
    /// failureReason（`None` = no failure： or success， or by /TTL ）
    pub message: Option<String>,
}

impl ModelRefreshOutcome {
    /// new success（`count` is after ）
    pub fn refreshed(count: usize) -> Self {
        Self { refreshed: true, count, message: None }
    }

    /// no has （ by /TTL 、 up no to available、 no has available etc ）——
    /// not is failure， is this no has new 
    pub fn unchanged() -> Self {
        Self::default()
    }

    /// new failure（`reason` is will to use Reason）
    pub fn failed(reason: impl Into<String>) -> Self {
        Self { refreshed: false, count: 0, message: Some(reason.into()) }
    }
}

/// for **current** 1 default then （ 1 ）。
///
/// ── this 1 already not need need ──────────────────────────────
/// `raccoon-<hex>` default use and `sn-` before ，「 has this 
/// 」。directory already to ，return （ its in already 
/// `visibility`），default then by in `models.refresh` ——
/// this 1 will and （ in 1 `seeded` ）。
///
/// for **current** WorkBuddy 1 default then （defaultenabled
/// ， `model_rules::seed_workbuddy_defaults`）。
///
/// and `seed_current_qoder_defaults` ： new can can failure / none not 
/// new —— that path up no has can ， after this （ or old ）
/// need has default。 etc ： id not will 。
///
/// ** 1 **（ after has and provider ）： by 
/// `(provider, id)` ， will new in ，
/// can can 。
fn seed_current_workbuddy_defaults() {
    // 1 ： old （hy3 / hy4-preview-f / deepseek-v4.1-flash 
    // ） that 。must in can before ， by and Qoder that 。
    if let Some(summary) = crate::server::core::model_rules::migrate_workbuddy_default_enabled() {
        crate::server::logging::log("[Models]", &summary);
    }
    for region in super::workbuddy::Region::ALL {
        let ids: Vec<String> = super::adapter::adapter_by_id(region.provider_id())
            .map(|adapter| adapter.list_models())
            .unwrap_or_default()
            .iter()
            .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
            .collect();
        if let Some(summary) = crate::server::core::model_rules::seed_workbuddy_defaults(
            region.provider_id(),
            &ids,
        ) {
            crate::server::logging::log("[Models]", &summary);
        }
    }
}

/// for **current** Qoder 1 default then （defaultenabled 
/// ， `model_rules::seed_qoder_defaults`）。
///
/// （`rpc_cache`： or up 1 new ），
/// by 、 by provider id （ after is 
/// `(provider, id)` empty）。 need this 1 by and WorkBuddy ：Qoder in 
/// no has / new failure when ，** use ** 
/// is that —— not ， to is old 。
fn seed_current_qoder_defaults() {
    // 1 ： old （ `Qwen3.8-Flash`） that new 
    // 。must in can ** before ** —— otherwise visibility will 
    // 「 already 」，（ 1 ， will not ）。
    if let Some(summary) = crate::server::core::model_rules::migrate_qoder_default_enabled() {
        crate::server::logging::log("[Models]", &summary);
    }
    seed_qoder_defaults_from_catalog();
}

/// by current id to `model_rules::seed_qoder_defaults`。
///
/// from ****（`adapter_by_id(provider).list_models()`）： not 
/// has Qoder directory，must by `(provider, id)` —— 
/// 。
fn seed_qoder_defaults_from_catalog() {
    for region in super::qoder::endpoints::Region::ALL {
        let provider = region.provider_id();
        let ids: Vec<String> = match super::adapter::adapter_by_id(provider) {
            Some(adapter) => adapter
                .list_models()
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str))
                .map(str::to_string)
                .collect(),
            None => Vec::new(),
        };
        if ids.is_empty() {
            continue;
        }
        if let Some(summary) = crate::server::core::model_rules::seed_qoder_defaults(
            provider,
            &format!("Qoder {}", region.label()),
            &ids,
        ) {
            crate::server::logging::log("[Models]", &summary);
        }
    }
}

/// for **current** Cline 1 default（ before 
/// before ， `deepseek-v4.1-flash → cline-free/deepseek-v4.1-flash`，
/// `model_rules::seed_cline_defaults`）。
///
/// need this 1 by and ：Cline directory in 「 no has / new failure」
/// when has ， that path up no has can —— ** has before 
/// **， not use up to `cline-free/deepseek-v4.1-flash` this 。
/// etc ： id not will （ use after not will ）。
///
/// ── 1 （「 to 」）─────────────
/// has （`deepseek-v4.1-flash`）， before by 「 to 」 to 1 
/// ，** in **。 this by `Pool::ALL` 
/// （Free in before ）—— none（ not need need ），default；
/// 1 by `model_rules::EXTRA_ALIASES` up ，** for can by **，
/// in before ， not available。
///
/// not need need `store`： need by 「 has 」（ is when 
/// ）， in is ， by 。
fn seed_current_cline_defaults() {
    for pool in super::cline::models::Pool::ALL {
        // （`rpc_cache`）： or up 1 new 。
        let provider = pool.provider_id();
        let ids: Vec<String> = super::adapter::adapter_by_id(provider)
            .map(|adapter| {
                adapter
                    .list_models()
                    .iter()
                    .filter_map(|item| item.get("id").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        if ids.is_empty() {
            continue;
        }
        if let Some(summary) =
            crate::server::core::model_rules::seed_cline_defaults(provider, &ids)
        {
            crate::server::logging::log("[Models]", &summary);
        }
    }
}

/// for **current**「 up has 、client not 」 1 default use 。
///
/// ── Why 1 （ not and up that ）──────────────────
/// up is ****（ by 「 up can use 」 ），
/// ** can **： `models.rs` `visibility` （Antigravity
/// by 、Qoder up `enable`、CodeArts 
/// `display_enabled`）。 in ， in that —— 。
///
/// ── Why「default」= unavailable ─────────────────────────
/// by is 「default = this not 」
/// （`catalog::builtin_target` `default_enabled`， not return `None`），
/// default when to ： not in `/v1/models`， not use 。
/// not will down 「 no has 、 can use 」 。
///
/// for ** has **（ its no has `visibility` ， this ）：
/// before no has can ， by 「 no not 」（
/// `officeace/probe.rs` 1 ）。
pub(crate) fn seed_hidden_visibility_defaults() {
    for kind in [
        super::ProviderKind::WorkBuddy,
        super::ProviderKind::WorkBuddyIntl,
        super::ProviderKind::Antigravity,
        super::ProviderKind::Qoder,
        super::ProviderKind::QoderIntl,
    ] {
        let provider = crate::server::core::providers::kind_id(kind);
        let Some(adapter) = adapter_by_id(provider) else { continue };
        let entries: Vec<(String, &str)> = adapter
            .list_models()
            .iter()
            .filter_map(|item| {
                let id = item.get("id").and_then(Value::as_str)?.trim().to_string();
                if id.is_empty() {
                    return None;
                }
                // no has can （ not to —— will this up 
                // when can ）
                let visibility = item.get("visibility").and_then(Value::as_str)?;
                // `'static` ： this ，
                // Json to `seed_hidden_disabled` 
                let owned: &'static str = match visibility {
                    "hidden" => "hidden",
                    "shown" => "shown",
                    _ => "undeclared",
                };
                Some((id, owned))
            })
            .collect();
        if entries.is_empty() {
            continue;
        }
        if let Some(summary) = crate::server::core::model_rules::seed_hidden_disabled(
            provider,
            &crate::server::core::providers::label_of(provider),
            &entries,
        ) {
            crate::server::logging::log("[Models]", &summary);
        }
    }
}

/// all** already ** provider new 1 directory（****path）。
///
/// use ：`api::chat::spawn_catalog_refresh`（client `/v1/models` after new ）
/// and 「directory new 」 when 。 in `providers::catalog_refresh` ——
/// and path use 1 ， in `manual` this 1 up 。
///
/// **path and **： new 、 in 、failure by 
/// `core::task_state` （`modelRefresh:<provider>`），「 1 
/// 1 」「client 1 up 」 not will 。 this is return
/// Reason： when need by statistics（success / / failure） and need 。
///
/// /TTL （`force = false`）：「 can 」 after ，
/// by this is 。 by [`refresh_implemented_forced`]。
pub async fn refresh_implemented(store: &AccountStore) -> Vec<Value> {
    seed_current_workbuddy_defaults();
    seed_current_qoder_defaults();
    seed_current_cline_defaults();
    seed_hidden_visibility_defaults();
    let results = super::catalog_refresh::refresh(store, &serde_json::Map::new(), None, false).await;
    // new after 1 ：add before / in this 1 in 
    // （ and that 1 —— that when is ）
    seed_current_cline_defaults();
    // can must in ** new after ** 1 ： 1 ，「 up has 
    // not 」 that id 1 （ when can can not in ）。
    seed_hidden_visibility_defaults();
    results
}

/// after directory and then 。 by when `set_defaults` ，
/// this in and after use ， default then already 。
pub fn restore_cached_catalogs() {
    for kind in implemented_kinds() {
        let _ = adapter_for(kind).list_models();
    }
    seed_current_workbuddy_defaults();
    seed_current_qoder_defaults();
    seed_current_cline_defaults();
    // this 1 is （ or ）： can need ，
    // otherwise「 after not can available」。
    seed_hidden_visibility_defaults();
}

/// **** new ：「supports new 」 ， and 。
///
/// ：`[{ provider, providerLabel, status, count?, refreshedAt, fixed?, message? }, ...]`，
/// = ，** has 1 **（unsupported in ，status = `skipped`
/// and Reason）—— total（success N / failure M / K） and can for up 
/// total（ and `credential_maintenance::refresh_expiring_accounts` 1 ）。
///
/// ── （ before after ）──────────────────────────────────────
/// - `status`：`"refreshed" | "skipped" | "failed"`（ down ）；
/// - `count`：only in `refreshed` when ， after ；
/// - `refreshedAt`：** this current when **（ms，0 = from success），
/// has 。 is when not is 「request when 」—— failure / 
/// no ， when this is up success that （「update」）；
/// - `fixed`：only in 「 this unsupported new 」 when and as true —— ** can **，
/// before 「 can 」（， not ） and 「 no to new 」
/// ； by `message` will in after 
/// （ and `USAGE_NOT_CONFIGURED_CODE` 1 ）；
/// - `message`：`skipped` / `failed` when to use Reason。
///
/// ── and `refresh_implemented` （ is has ）───────────
/// 1. **`force = true`**： use by down by is 「 in 1 」。
/// TTL will 「 no 、 no 」， and can none。
/// this as after path， use need when 1 。
/// ，** 1 and **（`manual` ， `catalog_refresh`）——
/// new can before ， in and failure（ that is up ，
/// not this by by ）。
/// 2. **supports **： not that request。
/// 3. **return**：pathfailure（ no has in etc ）；pathmust
/// 「 to 、Why no 」 to —— 1 「 already new 」
/// will 「 its failure」 and 「 its 」success。
/// （path in return： when need by statistics need 。）
///
/// ── `skipped` and `failed` （ this ）──
/// - `skipped`：** this no has can ， and not is error**。： this no has 
/// directory（， `fixed: true`）、 or this new no has new 
/// （ up return as empty / no has available）。
/// - `failed`：**， up or **（HTTP 2xx、error、
/// failure）。`message` Reason， use can can need （ / ）。
/// `failed` `skipped` will ； `skipped` `failed`
/// will 「 this no has new 」 this 1 error。
///
/// failure not 、： 1 failure not its （`refresh_models` 
/// failure not returnerror）， by and path（provider 、）。
///
/// `accounts` is 「 this use 」 （`{providerId: accountId}`，
/// 「get」 「」 down ）： or empty = this by default
/// （available， [`AccountStore::current_entry_for_provider`]）。
/// up `accountId`（**** use ， before that 1 ——
/// is ， no is ），「 this use is 」。
///
/// `providers` is ** need **：`None` = already （ when 
/// and not use ）；`Some(list)` = —— 「get」 by 
/// 「 has ∪ has enabled 」（ `api/models.rs`）。
/// and as empty ** not 、 not **： for use in up not to 
/// （ no has enabled、 as empty）， will to 1 「」 。
pub async fn refresh_implemented_forced(
    store: &AccountStore,
    accounts: &serde_json::Map<String, Value>,
    providers: Option<&[String]>,
) -> Vec<Value> {
    seed_current_workbuddy_defaults();
    seed_current_qoder_defaults();
    seed_current_cline_defaults();
    seed_hidden_visibility_defaults();
    let results = super::catalog_refresh::refresh(store, accounts, providers, true).await;
    seed_current_cline_defaults();
    // new new can can 1 「 not can 」 ， need up 
    seed_hidden_visibility_defaults();
    results
}

// ─── （W2b-T3 ，W6 delete）─────────────────
//
// W2b-T3 this has 1 `PlaceholderAdapter`（requestreturn 501）。
// W4a after 「 already provider」（ is ）， 
// parameter `pending::PendingAdapter` and **** `adapter_for` 
// CatPaw / AutoClaw （ from 501 503，
// and up `is_stateful` ）。
// after up （W5-T-d4 / W4b-T-c2） after ， not has use ，
// W6 when file 1 delete —— in provider is ： in 
// `adapter_for` to 1 ， not is new 1 。
