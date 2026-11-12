//! **Chat Completions **（OpenAI ，）。
//!
//! ── and and not ────────────────────────────────
//! and 「 has 」（`ProviderAdapter::forward_conversation`）
//! ：（`provider_loop::attempt_custom`）、telemetry ，
//! 「 1 」—— config and → request → → response
//! `ForwardOutcome`。** not is ** `ProviderAdapter` ： not 
//! `ProviderKind`（ by `mod.rs` module header）， in `attempt_queue`
//! for `is_custom_provider_id` up 。
//!
//! slot / connections as parameter（ parameter ）：none
//! path in `ForwardStream`， in `providers::custom` down 、
//! `upstream` —— can （`ForwardStream::new`
//! need need ）。 `&mut` 「 and nonepath when 」：
//! when `slot.take()` + `connections.handoff()` to ， when in 
//! use 、return（ and nonepath 1 then ）。
//!
//! ── （ ， is ）─────────────────
//! up is OpenAI Chat Completions ，：
//! - request：client ****（send_body already /
//! ）， 1 is `model` （ alias → up ） and by need 
//! `reasoning_effort`（ etc ）—— is 「client」 ， not is 
//! 。** 1 ** is `clientEmulation`
//! （ [`emulation`]：OpenCode Zen need request is 「 can 」，
//! need ）—— that is opt-in ，default 1 not ；
//! - response： up `stream:true` request（ already ），
//! client SSE （reasoning and `ForwardStream`），
//! client after JSON（ `aggregate_sse_completion`）—— 
//! `model` **clientrequest **（ AutoClaw 
//! `sse_model_rewrite`： up as ）。
//! - usage in `ForwardStream` / （ is SSE 
//! 1 ）， not need need 1 response。
//!
//! ── （， [`ProviderQuirks`] ）────
//! - has apiKey → `Authorization: Bearer <key>`（ or anthropic 
//!     `x-api-key`）；
//! - none need （`noAuth`）→ ** 1 not **：empty Bearer for 
//! up etc error， and 「 this not need 」 ；
//! - client and no has key → use up `public`
//! （OpenCode ： not Key can ， this is that ）。
//!
//! ── （ ）─────────────────────────────
//! - chat_completions： up is chat ，****（「」）；
//! - responses / anthropic： up 1 ，**** —— chat 
//! [`rewrite_body`]（ + etc in chat up ）， by 
//! `protocol::responses_outbound` / `protocol::anthropic_outbound` 
//! up request； up SSE [`ProtocolTranslateStream`] 
//! chat SSE， after `ForwardStream`（reasoning and / usage /
//! model not ），。。
//! is [`forward`] `protocol` match： up `stream:true`
//! request for 1 use （ request up 1 ）。
//!
//! ── error（ for `UpstreamErrorClass` ，）──
//! not return `UpstreamErrorClass`（ that and in 
//! `providers::adapter`， not in that up ）， is **
//! `GatewayError` **， has ：
//! - 401 / 403 → error（ is apiKey ）。 no has can 
//! new （key is use ），** not TokenExpired**（ that will 
//! 1 failure new ）—— by failure to ： tried →
//! down 1 ；
//! - 429 → （`GatewayError::is_quota_limit` has 429）。
//! when by `Retry-After` （seconds or HTTP ）， not from 
//! error（`errors::parse_quota_reset_at`）—— UTC+8
//! and message， is `rotate::mark_account_limited` not use 
//! can when （ that parse 1 message，）；
//! - 5xx / its 4xx → up and need 。5xx 「 can 」 by 
//! 「 down 1 」（ and has path 1 to ， not ）。
//!
//! ── ──────────────────────────────────────────────────
//! not unwrap/expect（release is panic=abort）；serde failure 1 as 
//! 「 by 」 or 「 1 error」， not panic。

use std::sync::Arc;

use serde_json::Value;

use crate::server::core::account_store::AccountStore;
use crate::server::core::custom_providers;
use crate::server::core::model_rules;
use crate::server::core::protocol::{anthropic_outbound, responses_outbound};
use crate::server::core::proxies::ResolvedProxy;
use crate::server::core::providers::custom::emulation;
use crate::server::core::upstream::connections::ConnectionGuard;
use crate::server::core::upstream::request::{read_upstream_error, send_chat_request, TransportRequest};
// `FramePolicy::with_rewrite(...)`： model ， 
// —— that 1 is WorkBuddy / AutoClaw up ， not can 
// use up 。
use crate::server::core::upstream::sse::{FramePolicy, ModelRewrite};
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::{ForwardOutcome, ForwardStream, InFlightGuard};
use crate::server::errors::GatewayError;
use crate::server::logging;

/// 1 （ `attempt_custom` use ）。
///
/// `account_id` is empty this 1 no has —— ** no has **
/// （， `custom_accounts` module header）， 503，
/// not default。
///
/// `body` is already `send_body` chat request（/ already ）；
/// 「」 after 1 （ + etc ）。
///
/// `pub(crate)` not is `pub`： `InFlightGuard` is `upstream` 
/// （`pub(crate)`）—— can not can 。 use 
/// has `upstream::provider_loop` ，`pub(crate)` is 。
pub(crate) async fn forward(
    store: &AccountStore,
    provider_id: &str,
    account_id: &str,
    body: &Value,
    route_model: Option<&str>,
    proxy: Option<ResolvedProxy>,
    stream: bool,
    telemetry: &Arc<RequestTelemetry>,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
) -> Result<ForwardOutcome, GatewayError> {
    // ── config and （： after ，error to need ）──────
    let provider = custom_providers::get(provider_id).ok_or_else(|| {
        GatewayError::with_status(503, format!("自定义提供商不存在: {}", provider_id.trim()))
    })?;
    let protocol = provider
        .get("protocol")
        .and_then(Value::as_str)
        .unwrap_or(custom_providers::PROTOCOL_CHAT_COMPLETIONS);
    // ── （module header）────────────────────────────────────
    // config，unknown 400 —— 1 「 up 
    // 1 not 」 has use 。
    let kind = match protocol {
        custom_providers::PROTOCOL_CHAT_COMPLETIONS => None,
        custom_providers::PROTOCOL_RESPONSES => Some(OutboundKind::Responses),
        custom_providers::PROTOCOL_ANTHROPIC => Some(OutboundKind::Anthropic),
        other => {
            return Err(GatewayError::with_status(
                400,
                format!(
                    "自定义提供商「{}」的协议 {other} 无法识别，请检查提供商配置",
                    custom_providers::label_of(provider_id).unwrap_or_else(|| provider_id.to_string()),
                ),
            ));
        }
    };
    if account_id.trim().is_empty() {
        return Err(GatewayError::with_status(
            503,
            "没有可用账号，无账号可转发：请在账号页为该自定义提供商添加并启用账号",
        ));
    }
    // （： apiKey and already ）。 not to in 
    // —— 1 ， and 「」 etc can 。
    let credential_store = store.clone();
    let credential_id = account_id.to_string();
    let credential = crate::server::core::blocking::run(move || {
        credential_store.custom_credential_by_id(&credential_id)
    }).await?.ok_or_else(|| {
        GatewayError::with_status(503, format!("自定义账号不存在或已被删除: {account_id}"))
    })?;
    // baseUrl：（ to config）。empty，
    // this 1 —— no has will 1 error
    let base_url = credential
        .base_url_override
        .clone()
        .or_else(|| {
            provider
                .get("baseUrl")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    if base_url.trim().is_empty() {
        return Err(GatewayError::with_status(
            400,
            "该自定义提供商没有可用的 baseUrl（提供商与账号上都没有配置）",
        ));
    }
    // in this （ and already ；chat path down ）
    let quirks = ProviderQuirks::from_provider(&provider);
    // can can OpenCode ** old **（`/zen/v1`，2026-10 
    // 1 403）—— and can can is old ， in **
    // after ** 1 （ already in `custom_providers::item_of`
    // 1 ， in this can to ）。
    let base_url = custom_providers::migrate_base_url(&base_url, quirks.emulate_opencode);
    if let Some(kind) = kind {
        return forward_translated(
            kind,
            provider_id,
            account_id,
            &credential.api_key,
            &base_url,
            &quirks,
            body,
            route_model,
            proxy,
            stream,
            telemetry,
            slot,
            connections,
        )
        .await;
    }
    // chat ：（Anthropic / Responses） （`_wb_*`，
    // `protocol::mod` ） not up —— OpenAI up will 
    // up unknown 400。（ up already return）** not **： that is 
    // need （cache_control / is_error / encrypted_content ）。
    //
    // before ： is （chat up chat 
    // ， `{"type":"web_search","web_search":{…}}`， up ），
    // ** **（ down /v1/messages or /v1/responses 
    // `web_search_20250305` / `web_search` that ） no has chat ， will 
    // up 1 400 —— + ， `protocol::native_tool` module header。
    let cross = crate::server::core::protocol::native_tool::strip_cross_protocol(
        body,
        crate::server::core::protocol::native_tool::ORIGIN_CHAT,
    );
    if let Some((_, dropped)) = &cross {
        logging::log(
            "[CustomProvider]",
            &dropped.describe(
                Some(provider_id),
                Some("该家按 chat 协议透传，承载不了别的协议的原生声明"),
            ),
        );
    }
    let stripped = crate::server::core::protocol::strip_internal_fields(
        cross.as_ref().map(|(next, _)| next).unwrap_or(body),
    );
    let body: &Value = &stripped;
    let url = format!(
        "{}/chat/completions{}",
        base_url.trim_end_matches('/'),
        quirks.url_suffix
    );

    // ── request：（ alias → up ）+ etc ──────
    let requested = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let (mut outbound, rewrite) = rewrite_for_route(body, provider_id, &requested, route_model);
    // client（OpenCode ）：。 when not request。
    // in `rewrite_body` after 、 before —— will ** after ** ，
    // and （ not messages， etc ，
    // not in will ）
    quirks.apply_emulation_to_body(&mut outbound, &base_url);
    let payload = serde_json::to_string(&outbound)
        .map_err(|error| GatewayError::with_status(500, format!("请求体序列化失败: {error}")))?;
    // ── （， not can ）────────────────────────────
    // ① default：** has apiKey empty** —— none need up 
    // （`noAuth` ） to 1 empty `Bearer ` will error，
    // and 「 this not need 」 ；
    // ② （OpenCode ， by ）：client 
    // and `public`；Go will and UA（）；
    // ③ up （`quirks.apply_to`）： use 。
    let api_key = credential.api_key.trim();
    let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
    if !api_key.is_empty() {
        headers.push((
            "Authorization".to_string(),
            format!("Bearer {api_key}"),
        ));
    }
    quirks.apply_emulation_headers(&mut headers, api_key, &outbound, &base_url);
    quirks.apply_to(&mut headers);

    logging::verbose(
        "[CustomProvider]",
        &format!(
            "POST {url} model={} stream={stream} account={account_id} 出口={}{}",
            if requested.is_empty() { "(默认)" } else { &requested },
            describe_proxy(proxy.as_ref()),
            if quirks.emulate_opencode { " 伪装=opencode" } else { "" },
        ),
    );
    let transport = TransportRequest {
        url,
        headers,
        payload,
        proxy,
        // no has can （ use is 1 ：， no ）
        system_proxy_when_unset: false,
    };
    let response = send_chat_request(&transport)
        .await
        .map_err(|error| error.to_gateway_error())?;

    // ── response：2xx outcome； 2xx （module header）────────────────
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        // Retry-After need in body ** before **（read_upstream_error will response）
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string);
        let detail = read_upstream_error(response, telemetry.capture().as_deref()).await;
        return Err(classify_custom_error(
            status,
            &detail.message,
            retry_after.as_deref(),
        ));
    }

    if stream {
        // and to （ and nonepath 1 when ，module header）
        Ok(ForwardOutcome::Stream {
            status,
            stream: Box::new(ForwardStream::new(
                response,
                slot.take(),
                connections.handoff(),
                telemetry.clone(),
                FramePolicy::with_rewrite(rewrite),
            )),
        })
    } else {
        // ： by `rewrite` body model ；
        // in use ，return（ and nonepath）
        let aggregated = crate::server::core::upstream::aggregate::aggregate_sse_completion(
            response,
            telemetry.clone(),
            FramePolicy::with_rewrite(rewrite),
        )
        .await?;
        Ok(ForwardOutcome::Completion {
            body: aggregated.body,
        })
    }
}

/// （[`forward_translated`] in this up ）
#[derive(Clone, Copy)]
enum OutboundKind {
    /// OpenAI Responses（`POST {base}/responses`）
    Responses,
    /// Anthropic Messages（`POST {base}/v1/messages`）
    Anthropic,
}

impl OutboundKind {
    /// and 
    fn label(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::Anthropic => "anthropic",
        }
    }
}

/// up **per-provider **（`urlSuffix` / `headers` /
/// `anthropicToolType` / `clientEmulation`， `custom_providers` module header）。
///
/// this by directory in create when （ 9Router ：
/// GLM / MiniMax `?beta=true` and `Anthropic-Beta`、OpenRouter 、
/// MiniMax type ）， no has this —— when 「none」，
/// old not use 。
struct ProviderQuirks {
    /// to URL （ `?beta=true`）
    url_suffix: String,
    /// and to default up （default）
    headers: Vec<(String, String)>,
    /// anthropic up when to `type: "custom"`（MiniMax none type ）
    tool_type_custom: bool,
    /// is by **OpenCode CLI **request（`clientEmulation`，
    /// [`emulation`]）：none key use `public`、 will 、request
    /// —— OpenCode Zen has this 。** will request**，
    /// in when 。
    emulate_opencode: bool,
}

impl ProviderQuirks {
    fn from_provider(provider: &Value) -> Self {
        // up already 1 （custom_providers::item_of）， this 
        // ： not for by ， not panic
        let headers = provider
            .get("headers")
            .and_then(Value::as_object)
            .map(|object| {
                object
                    .iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|text| (key.clone(), text.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            url_suffix: provider
                .get("urlSuffix")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            headers,
            tool_type_custom: provider
                .get("anthropicToolType")
                .and_then(Value::as_str)
                .is_some_and(|text| {
                    text.eq_ignore_ascii_case(custom_providers::TOOL_TYPE_CUSTOM)
                }),
            emulate_opencode: provider
                .get("clientEmulation")
                .and_then(Value::as_str)
                .is_some_and(|text| {
                    text.eq_ignore_ascii_case(custom_providers::CLIENT_EMULATION_OPENCODE)
                }),
        }
    }

    /// by and default：（ASCII not ） default ，
    /// its in after 。default（Authorization / x-api-key） 1 
    /// —— that is has ， this 。
    fn apply_to(&self, headers: &mut Vec<(String, String)>) {
        for (key, value) in &self.headers {
            match headers.iter_mut().find(|(name, _)| name.eq_ignore_ascii_case(key)) {
                Some(slot) => slot.1 = value.clone(),
                None => headers.push((key.clone(), value.clone())),
            }
        }
    }

    /// and （ in `clientEmulation: "opencode"` when ）。
    ///
    /// `api_key` as empty when use `public` —— this is 「 not Key can use OpenCode
    /// Zen 」 ； key use key（must）。
    ///
    /// ** use **：default → → `apply_to`（）。 in `apply_to`
    /// before is ： in up ， not this use 。
    ///
    /// ── （ by `base_url` ， `custom_providers::is_opencode_go_endpoint`）──
    /// - **Go **（`/zen/go/v1`）： will and UA（
    /// [`emulation::apply_go_headers`]）。** not ** —— that need is 
    /// by need ， not is ； use （ or ），
    /// Go this will 、 this Key ；
    /// - ****（ its ）： will + / ，
    /// in when 。
    fn apply_emulation_headers(
        &self,
        headers: &mut Vec<(String, String)>,
        api_key: &str,
        body: &Value,
        base_url: &str,
    ) {
        let session = emulation::session_id(&emulation::conversation_seed(body));
        if custom_providers::is_opencode_go_endpoint(base_url) {
            emulation::apply_go_headers(headers, &session);
            return;
        }
        if !self.emulate_opencode {
            return;
        }
        emulation::apply_headers(headers, api_key, &session);
    }

    /// request「 can 」（`stream: true` + `bash` / `read` ）。
    /// when ** 1 not ** —— is default。
    ///
    /// **Go 1 not **： request（
    /// #44300/#44382）， that 。
    fn apply_emulation_to_body(&self, body: &mut Value, base_url: &str) {
        if !self.emulate_opencode || custom_providers::is_opencode_go_endpoint(base_url) {
            return;
        }
        let _ = emulation::ensure_agent_shape(body);
    }
}

/// ****（responses / anthropic） 1 。
///
/// and chat 「 1 」， has （ in [`OutboundKind`] up 
/// ）：URL and 、request（`protocol::*_outbound`）、 up SSE 
/// （[`ProtocolTranslateStream`]）。 its —— and etc 
/// then （[`rewrite_body`]， in chat up ）、 up 
/// `stream:true` request、 2xx error —— use 1 。
async fn forward_translated(
    kind: OutboundKind,
    provider_id: &str,
    account_id: &str,
    api_key: &str,
    base_url: &str,
    quirks: &ProviderQuirks,
    body: &Value,
    route_model: Option<&str>,
    proxy: Option<ResolvedProxy>,
    stream: bool,
    telemetry: &Arc<RequestTelemetry>,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
) -> Result<ForwardOutcome, GatewayError> {
    // ── request： in chat up ， ──────
    // （ and etc is 「client」 ， and none；
    // up by from after chat —— responses
    // reasoning_effort reasoning.effort，anthropic thinking）
    let requested = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let (mut outbound_chat, rewrite) = rewrite_for_route(body, provider_id, &requested, route_model);
    // client： and chat 1 when （ after 、 before ）—— need by 
    // 1 up （anthropic tools ）；Go not request
    quirks.apply_emulation_to_body(&mut outbound_chat, base_url);
    // to up = after model （rewrite_body ；request no 
    // model when is empty， and chat 「 no has 」 1 ）
    let wire_model = outbound_chat
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    // （ chat ； this by ）
    let api_key = api_key.trim();
    let has_key = !api_key.is_empty();

    let (url, headers, payload) = match kind {
        OutboundKind::Responses => {
            let url = format!(
                "{}/responses{}",
                base_url.trim_end_matches('/'),
                quirks.url_suffix
            );
            let mut payload = responses_outbound::responses_request_from_chat(
                &outbound_chat,
                &wire_model,
            )
            .map_err(|message| GatewayError::with_status(400, message))?;
            // up stream:true request（ and chat 1 ： is 
            // can ， down by path）—— stream is 
            // chat ， this 1 ， not 
            if let Some(object) = payload.as_object_mut() {
                object.insert("stream".to_string(), Value::Bool(true));
            }
            let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
            if has_key {
                headers.push(("Authorization".to_string(), format!("Bearer {api_key}")));
            }
            quirks.apply_emulation_headers(&mut headers, api_key, &outbound_chat, base_url);
            quirks.apply_to(&mut headers);
            (url, headers, payload)
        }
        OutboundKind::Anthropic => {
            // anthropic baseUrl not /v1（ is https://api.anthropic.com），
            // by OpenAI /v1 —— ： already /v1
            // not 。 /v1/v1/messages 404，
            // 「 up 」， to will ， in this 1 
            let base = base_url.trim_end_matches('/');
            let url = if base.ends_with("/v1") {
                format!("{base}/messages{}", quirks.url_suffix)
            } else {
                format!("{base}/v1/messages{}", quirks.url_suffix)
            };
            let mut payload = anthropic_outbound::anthropic_request_from_chat(
                &outbound_chat,
                &wire_model,
            )
            .map_err(|message| GatewayError::with_status(400, message))?;
            if let Some(object) = payload.as_object_mut() {
                object.insert("stream".to_string(), Value::Bool(true));
            }
            // type （MiniMax Claude none type ，
            // `custom_providers::TOOL_TYPE_CUSTOM`）： is none type
            // ， when up `type: "custom"`
            if quirks.tool_type_custom {
                if let Some(tools) = payload.get_mut("tools").and_then(Value::as_array_mut) {
                    for tool in tools.iter_mut() {
                        if let Some(object) = tool.as_object_mut() {
                            object.insert(
                                "type".to_string(),
                                Value::String("custom".to_string()),
                            );
                        }
                    }
                }
            }
            // anthropic not is Bearer（ and `custom_providers::
            // fetch_upstream_models` 1 ）， by current。
            // ** as empty when not x-api-key**（ by chat ①）
            let mut headers = vec![
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
                ("Content-Type".to_string(), "application/json".to_string()),
            ];
            if has_key {
                headers.push(("x-api-key".to_string(), api_key.to_string()));
            }
            quirks.apply_emulation_headers(&mut headers, api_key, &outbound_chat, base_url);
            quirks.apply_to(&mut headers);
            (url, headers, payload)
        }
    };
    let payload = serde_json::to_string(&payload)
        .map_err(|error| GatewayError::with_status(500, format!("请求体序列化失败: {error}")))?;

    logging::verbose(
        "[CustomProvider]",
        &format!(
            "POST {url} protocol={} model={} stream={stream} account={account_id} 出口={}",
            kind.label(),
            if wire_model.is_empty() { "(默认)" } else { &wire_model },
            describe_proxy(proxy.as_ref()),
        ),
    );
    let transport = TransportRequest {
        url,
        headers,
        payload,
        proxy,
        // no has can （ use is 1 ：， no ）
        system_proxy_when_unset: false,
    };
    let response = send_chat_request(&transport)
        .await
        .map_err(|error| error.to_gateway_error())?;

    // ── response：2xx outcome； 2xx （ and chat 1 ）──────
    // error not （responses is {error:{message}}、anthropic
    // is {type:"error",error:{message}}），read_upstream_error 
    // （message → msg → error.message → before 500 ）
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        // Retry-After need in body ** before **（read_upstream_error will response）
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string);
        let detail = read_upstream_error(response, telemetry.capture().as_deref()).await;
        return Err(classify_custom_error(
            status,
            &detail.message,
            retry_after.as_deref(),
        ));
    }

    // up （→ chat SSE）， to has ：
    // ForwardStream（reasoning and / usage / model ），
    // —— and when and chat 
    let translated =
        Box::pin(ProtocolTranslateStream::new(kind, &wire_model, response, telemetry));
    if stream {
        Ok(ForwardOutcome::Stream {
            status,
            stream: Box::new(ForwardStream::from_translated(
                translated,
                slot.take(),
                connections.handoff(),
                telemetry.clone(),
                FramePolicy::with_rewrite(rewrite),
            )),
        })
    } else {
        let aggregated = crate::server::core::upstream::aggregate::aggregate_frame_stream(
            translated,
            telemetry.clone(),
            FramePolicy::with_rewrite(rewrite),
        )
        .await?;
        Ok(ForwardOutcome::Completion { body: aggregated.body })
    }
}

/// up SSE → chat SSE （responses / anthropic use ）。
///
/// ── Why need need ────────────────────────────────────────────
/// `ForwardStream` and is ** chat SSE**（ is SSE 
/// 1 ，usage / model / reasoning and in that ）。
/// up is ， that before 1 
/// 「 → chat 」 —— （ up → → ），
/// in `protocol::*_outbound` 。
///
/// up when reqwest error `io::Error` up ：path by 
/// `ForwardStream` 「error + [DONE]」，path 502 —— and chat
/// 1 error。 in this ** up **（ before ），
/// and chat path「 up 」 1 。
struct ProtocolTranslateStream {
    /// up 。error in when `io::Error`（ and 
    /// `ForwardStream::new` 1 ）：empty is this ，
    /// （ForwardStream / ） to is 1 。
    inner: futures::stream::BoxStream<'static, Result<bytes::Bytes, std::io::Error>>,
    machine: TranslateMachine,
    /// already down （ 1 up chunk can can ）
    pending: std::collections::VecDeque<bytes::Bytes>,
    /// up already （ not poll up ， pending None）
    upstream_done: bool,
    capture: Option<Arc<crate::server::core::debug_traffic::TrafficCapture>>,
}

/// （， 1 dyn ）
enum TranslateMachine {
    Responses(responses_outbound::ChatFromResponsesStream),
    Anthropic(anthropic_outbound::ChatFromAnthropicStream),
}

impl TranslateMachine {
    fn push(&mut self, chunk: &[u8]) -> Vec<bytes::Bytes> {
        match self {
            Self::Responses(machine) => machine.push(chunk),
            Self::Anthropic(machine) => machine.push(chunk),
        }
    }

    fn finish(&mut self) -> Vec<bytes::Bytes> {
        match self {
            Self::Responses(machine) => machine.finish(),
            Self::Anthropic(machine) => machine.finish(),
        }
    }
}

impl ProtocolTranslateStream {
    fn new(
        kind: OutboundKind,
        model: &str,
        response: reqwest::Response,
        telemetry: &Arc<RequestTelemetry>,
    ) -> Self {
        use futures::StreamExt;
        // up → io::Error（）→ empty（set「request when 」
        // 「responseempty when 」， and chat path 1 、 1 ）
        let described = response.bytes_stream().map(|item| {
            item.map_err(|error| {
                std::io::Error::other(crate::server::core::egress::describe_error_detail(&error))
            })
        });
        let guarded = crate::server::core::upstream::stall::idle_guard(
            Box::pin(described),
            std::time::Duration::from_millis(
                crate::server::config::timeout_settings().stream_idle_ms(),
            ),
        );
        Self {
            inner: guarded,
            machine: match kind {
                OutboundKind::Responses => TranslateMachine::Responses(
                    responses_outbound::ChatFromResponsesStream::new(model),
                ),
                OutboundKind::Anthropic => TranslateMachine::Anthropic(
                    anthropic_outbound::ChatFromAnthropicStream::new(model),
                ),
            },
            pending: std::collections::VecDeque::new(),
            upstream_done: false,
            capture: telemetry.capture(),
        }
    }
}

impl futures::Stream for ProtocolTranslateStream {
    type Item = Result<bytes::Bytes, std::io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::StreamExt;
        loop {
            if let Some(frame) = self.pending.pop_front() {
                return std::task::Poll::Ready(Some(Ok(frame)));
            }
            if self.upstream_done {
                return std::task::Poll::Ready(None);
            }
            match self.inner.poll_next_unpin(cx) {
                std::task::Poll::Pending => return std::task::Poll::Pending,
                std::task::Poll::Ready(None) => {
                    self.upstream_done = true;
                    // up EOF：（finish_reason / usage / [DONE]）
                    for frame in self.machine.finish() {
                        self.pending.push_back(frame);
                    }
                }
                std::task::Poll::Ready(Some(Ok(bytes))) => {
                    // ： up （ in before ， and chat path）
                    if let Some(capture) = &self.capture {
                        capture.push(&bytes);
                    }
                    for frame in self.machine.push(&bytes[..]) {
                        self.pending.push_back(frame);
                    }
                }
                std::task::Poll::Ready(Some(Err(error))) => {
                    self.upstream_done = true;
                    // already in when （ inner ）：reqwest error 
                    // 、empty when ， up 
                    return std::task::Poll::Ready(Some(Err(error)));
                }
            }
        }
    }
}

/// request：return（ need , SSE/parameter）。
///
/// ── （ and for ）────────────────────────
/// - ****：`custom_providers::wire_model_for` 1 「 up +
/// etc 」（， that ）。 and requestignore
/// when ** not **（ is ）； not ；
/// - ** etc **： when `reasoning_effort` —— empty、
/// client no to `reasoning_effort` / `reasoning`（ use ）、
/// etc not is （`model_rules::reasoning_is_off`； not to 
/// up 「」， and use ）。
///
/// `rewrite` is SSE/ model parameter：request model to ——
/// client no when （ up use default） no has 「」 。
fn rewrite_for_route(body: &Value, provider_id: &str, requested: &str, route_model: Option<&str>) -> (Value, Option<ModelRewrite>) {
    if let Some(client_model) = route_model {
        // already by request， not 。
        return (body.clone(), Some(ModelRewrite { requested: client_model.to_string() }));
    }
    rewrite_body(body, provider_id, requested)
}

fn rewrite_body(body: &Value, provider_id: &str, requested: &str) -> (Value, Option<ModelRewrite>) {
    let mut outbound = body.clone();
    if requested.is_empty() {
        return (outbound, None);
    }
    let (wire_model, reasoning) = custom_providers::wire_model_for(provider_id, requested);
    if !wire_model.eq_ignore_ascii_case(requested) {
        logging::verbose(
            "[CustomProvider]",
            &format!("provider={provider_id} 按映射改写模型名 {requested} → {wire_model}"),
        );
        if let Some(object) = outbound.as_object_mut() {
            object.insert("model".to_string(), Value::String(wire_model));
        }
    }
    if let Some(level) = reasoning
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        let explicit =
            outbound.get("reasoning_effort").is_some() || outbound.get("reasoning").is_some();
        if model_rules::reasoning_is_off(level) {
            logging::verbose(
                "[CustomProvider]",
                &format!(
                    "provider={provider_id} 映射 {requested} 绑定的思考等级为「{level}」\
                     （关闭思考），本网关不向任何上游发「关闭思考」字段，跳过注入"
                ),
            );
        } else if explicit {
            logging::verbose(
                "[CustomProvider]",
                &format!(
                    "provider={provider_id} 映射 {requested} 绑定的思考等级 {level} 未注入：\
                     客户端已显式指定思考字段"
                ),
            );
        } else if let Some(object) = outbound.as_object_mut() {
            logging::verbose(
                "[CustomProvider]",
                &format!(
                    "provider={provider_id} 映射 {requested} 注入思考等级 reasoning_effort={level}"
                ),
            );
            object.insert(
                "reasoning_effort".to_string(),
                Value::String(level.to_string()),
            );
        }
    }
    (
        outbound,
        Some(ModelRewrite {
            requested: requested.to_string(),
        }),
    )
}

/// 2xx response → after `GatewayError`（module header）。
///
/// ， and `UpstreamErrorClass` for in module header：
/// - 401/403 → ；
/// - 429 → ；`Retry-After` success when when UTC+8 and 
/// message（`rotate::mark_account_limited` will from 1 ，
/// module header ）；
/// - its → and up need 。
fn classify_custom_error(status: u16, raw: &str, retry_after: Option<&str>) -> GatewayError {
    let raw = raw.trim();
    let raw = if raw.is_empty() { "上游错误" } else { raw };
    match status {
        401 | 403 => GatewayError::with_status(
            i32::from(status),
            format!("上游凭证被拒绝（HTTP {status}）: {raw}"),
        ),
        429 => {
            let mut message = format!("上游返回 {status}: {raw}");
            // when ：Retry-After （seconds / HTTP ）→
            // error （parse_quota_reset_at has ）。
            // 「（<UTC+8 when > ）」 and message ——
            // and rotate reset_hint ， use and can 。
            if let Some(text) = retry_after
                .and_then(retry_after_reset_text)
                .or_else(|| text_of_reset_at(&message))
            {
                message.push_str(&format!("（{text} 恢复）"));
            }
            GatewayError::with_status(429, message)
        }
        _ => GatewayError::with_status(
            i32::from(status),
            format!("上游返回 {status}: {raw}"),
        ),
    }
}

/// `Retry-After` → when （UTC+8，`parse_quota_reset_at` ）。
///
/// ：delay-seconds（seconds） and HTTP-date（RFC 2822）。
/// none（、） when to None —— use 。
fn retry_after_reset_text(retry_after: &str) -> Option<String> {
    let text = retry_after.trim();
    if let Ok(seconds) = text.parse::<i64>() {
        if seconds > 0 {
            return Some(reset_text_at(logging::now_ms() + seconds * 1000));
        }
        return None;
    }
    chrono::DateTime::parse_from_rfc2822(text)
        .ok()
        .map(|date| reset_text_at(date.timestamp_millis()))
        .filter(|text| !text.is_empty())
}

/// error already has when （`errors::parse_quota_reset_at` has ）。
/// not （0） when to None —— not empty。
fn text_of_reset_at(message: &str) -> Option<String> {
    let reset_at = crate::server::errors::parse_quota_reset_at(message);
    (reset_at > 0).then(|| reset_text_at(reset_at))
}

/// ms when → `YYYY-MM-DD HH:MM:SS UTC+8` 。
///
/// and `upstream::format_reset_text` （ up to when by UTC+8 ，
/// use when will use for not up ）， is + UTC+8 ——
/// `parse_quota_reset_at` is this 1 （ not ）。
/// when / failure when to empty（ this is and ， not can request）。
fn reset_text_at(reset_at: i64) -> String {
    let Some(utc) = chrono::DateTime::from_timestamp_millis(reset_at) else {
        return String::new();
    };
    let Some(offset) = chrono::FixedOffset::east_opt(8 * 3600) else {
        return String::new();
    };
    utc.with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S UTC+8")
        .to_string()
}

/// can （ and `upstream::describe_proxy` ； that 1 is 
/// `pub(super)`， as 1 verbose not —— 1 ）。
fn describe_proxy(proxy: Option<&ResolvedProxy>) -> String {
    match proxy {
        Some(proxy) if !proxy.label.is_empty() => proxy.label.clone(),
        Some(proxy) => proxy.host.clone(),
        None => "直连".to_string(),
    }
}

/// this **** is not is 「」（`proxy.config.source == "rotate"`）。
///
/// （`attempt_custom`） use 「429 when need not need 」： 429
/// is ** by IP** （ this use ， `core::proxy_rotate` module header），
/// 1 ； no has this ，429 is / 。
///
/// `proxy.config`（ by to is ，
/// `core::proxies` module header） not is use `ResolvedProxy`：
/// `source` is （pool / clash / custom）， use after already not 
/// 「 this is 」。
pub fn is_rotate_account(account: Option<&Value>) -> bool {
    account
        .and_then(|item| item.get("proxy"))
        .and_then(|proxy| proxy.get("config"))
        .map(crate::server::core::proxies::is_rotate_config)
        .unwrap_or(false)
}

/// this 1 ****（ this to up ； `attempt_custom` use ）。
///
/// and [`rewrite_body`] 1 （`wire_model_for` is ：config、
/// none IO、noneparameter， use ）。 1 not is 
/// `wire_model_for`： that is 「」 ， not this 
/// —— this 「 this is 」。
///
/// request no model when to empty（ and `SendBody::wire_model` empty 1 ）。
pub fn cooldown_model(provider_id: &str, requested: &str) -> String {
    let requested = requested.trim();
    if requested.is_empty() {
        return String::new();
    }
    custom_providers::wire_model_for(provider_id, requested).0
}
