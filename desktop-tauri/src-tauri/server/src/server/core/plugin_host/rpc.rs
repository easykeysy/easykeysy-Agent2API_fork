//! `RpcAdapter`： `ProviderAdapter` 1 RPC。
//!
//! ── this 1 Whymust in ────────────────────────────────────
//! up （、directory、 API）
//! [`ProviderAdapter`](crate::server::core::providers::adapter::ProviderAdapter)。
//! after ， that trait not can is Rust ，must
//! is 「 use to 」。`RpcAdapter` is that —— 1 provider
//! 1 ， has `Arc` and id。
//!
//! ── can Why is ──────────────────────────────────
//! `supports_*` this in path up use （`sse_model_rewrite` 1 
//! need ）。must in `init` when 1 and in —— RPC
//! will 「 1 」。 this is can Reason。
//!
//! ── RPC、 not ────────────────────────────────
//! RPC：all need need provider and not in path up （request、、
//! 、directory、、 to 、、）。
//! not RPC： can （）、`kind`（）。
//!
//! ── path up 1 ：`build_chat_request` ─────────────────
//! request 1 RPC。 this is can ： 1 RPC in up 
//! seconds， up request is ms to seconds。 path is SSE 
//! down ， that in `conversation` by 、。

use std::sync::Arc;
use std::time::Duration;

use agent2api_provider_protocol as proto;
use axum::http::HeaderMap;
use serde_json::Value;

use crate::server::core::account_store::AccountStore;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::core::providers::adapter::{
    ChatRequestPlan, ModelRefreshOutcome, ProviderAdapter, ReasoningPatch, RetryAdvice,
    RetryDecision, UpstreamErrorClass, UpstreamResponse,
};
use crate::server::core::providers::ProviderKind;
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::ForwardOutcome;
use crate::server::errors::GatewayError;

use super::Host;
use super::errors::strict;

/// 1 RPC use when 。
///
/// ── Why not use 1 1 ────────────────────────────────────
/// （ can 、directory、）failure； `refresh_models` need 
/// up 、`login.poll` need etc use ， can can is seconds。 use 。
const CONTROL_TIMEOUT: Duration = Duration::from_secs(10);
const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);

/// parameter。
#[derive(Clone, Debug)]
pub struct HostOptions {
    /// can file。
    pub exe: std::path::PathBuf,
    /// parameter。
    pub args: Vec<String>,
    /// directory。
    pub cwd: Option<std::path::PathBuf>,
    /// directory（ can ）。
    pub data_dir: String,
    /// 。
    pub host_version: String,
    /// 。
    pub target: String,
    /// default。
    pub proxy: String,
    /// 。
    pub debug_traffic: bool,
    /// when （ can can need 1 directory）。
    pub init_timeout: Duration,
    /// HTTP （`0` = unknown）。
    ///
    /// ── Why by ────────────────────────────────────
    /// `redirect_uri` need to loopback ，
    /// that is **** （ `HostEnv.loopback_port`）。
    pub loopback_port: u16,
}

impl HostOptions {
    /// 。
    pub fn new(exe: impl Into<std::path::PathBuf>) -> Self {
        Self {
            exe: exe.into(),
            args: Vec::new(),
            cwd: None,
            data_dir: String::new(),
            host_version: env!("CARGO_PKG_VERSION").to_string(),
            target: target_triple(),
            proxy: String::new(),
            debug_traffic: false,
            init_timeout: Duration::from_secs(60),
            loopback_port: 0,
        }
    }
}

/// current（ when ， `build.rs`）。
///
/// ── Why is Rust ， not is `arch-os` ─────────────
/// this has ，must use ** 1 **：
/// 1. `HostEnv.target`（ ）；
/// 2. `pack.json` `entry` ；
/// 3. `target` （ `proto::signing_payload`）；
/// 4. update when （`scripts/pack.mjs`（Agent2Providers ） is 
/// `agent2api-providers-<ver>-<>.zip`）。
///
/// this return `x86_64-windows` this `arch-os` ， `entry_for_target`
/// 「 arch and os」 can use —— is ** for **，
/// not （ is 「 up 」， not 、）。
/// ， 1 。
pub(super) fn target_triple() -> String {
    let arch = std::env::consts::ARCH;
    match std::env::consts::OS {
        // Windows GNU ABI （CI msvc ， will can ）
        "windows" => format!("{arch}-pc-windows-msvc"),
        "macos" => format!("{arch}-apple-darwin"),
        // Linux / its 1 by gnu ABI： and `pack.mjs` default 1 
        _ => format!("{arch}-unknown-linux-gnu"),
    }
}

/// 1 provider。
///
/// ── ────────────────────────────────────────────────
/// and （ after this valid —— has `Arc<Host>` and 
/// id， use when will in ）。
#[derive(Clone)]
pub struct RpcAdapter {
    /// this for provider id（leak `'static`， `provider_id()`）。
    id: &'static str,
    /// （， up ）。
    label: &'static str,
    /// provider id for `ProviderKind`（ use if ）。
    ///
    /// ── Why `ProviderKind` ─────────────────────────────
    /// has 370 `ProviderKind::` use （、 to 、
    /// then ……）。 is P6 ； in that before ，
    /// provider has for ，**add provider no has ** ——
    /// [`Self::kind`] 。
    kind: ProviderKind,
    /// 。
    host: Arc<Host>,
    /// for 1 ； to new after 。
    lease: Option<Arc<super::activity::UsageLease>>,
    binding: Option<Arc<tokio::sync::OnceCell<RpcAdapter>>>,
    /// when can （）。
    capabilities: proto::Capabilities,
    image_generation: bool,
    image_editing: bool,
    image_models: Vec<Value>,
    /// `finish_reason` （ when ）。
    ///
    /// ── Why is `&'static [&'static str]` not is `Box<[...]>` ──
    /// trait need return `&'static [...]`。 `Box<[...]>` can 
    /// `&'a [...]`（ `&self` ）， not `'static`。 need 
    /// leak：`Box::leak` to `&'static mut [T]`， `&'static [T]` can 。
    /// 1 1 ，。
    refusal_finishes: &'static [&'static str],
    /// quota（ up ）。
    refusal_markers: &'static [&'static str],
}

impl RpcAdapter {
    pub fn new(
        id: String,
        kind: ProviderKind,
        host: Arc<Host>,
        info: proto::ProviderInfo,
    ) -> Self {
        Self {
            id: leak_str(id),
            label: leak_str(info.label.clone()),
            kind,
            host,
            lease: None,
            binding: None,
            capabilities: info.capabilities,
            image_generation: false,
            image_editing: false,
            image_models: info.default_models.into_iter()
                .filter(crate::server::core::providers::images::model_is_image).collect(),
            refusal_finishes: leak_slice(info.refusal_finishes),
            refusal_markers: leak_slice(info.refusal_markers),
        }
    }

    pub(super) fn leased(&self) -> Option<Self> {
        let lease = self.host.activity.acquire()?;
        let mut adapter = self.clone();
        adapter.lease = Some(lease);
        Some(adapter)
    }

    pub(super) fn deferred(&self) -> Self {
        let mut adapter = self.clone();
        adapter.lease = None;
        adapter.binding = Some(Arc::new(tokio::sync::OnceCell::new()));
        adapter
    }

    async fn execution(&self) -> Result<&Self, GatewayError> {
        match &self.binding {
            Some(binding) => binding.get_or_try_init(|| async {
                super::registry::acquire(self.id).await?
                    .ok_or_else(|| super::activity::AvailabilityError::Unavailable.gateway())
            }).await,
            None => Ok(self),
        }
    }

    fn metadata(&self) -> &Self {
        self.binding.as_ref().and_then(|binding| binding.get()).unwrap_or(self)
    }

    pub(super) fn host(&self) -> &Arc<Host> { &self.host }

    pub(super) async fn load_image_capability(&mut self) {
        let result = self.control(
            crate::server::core::providers::images::CAPABILITIES_METHOD,
            Value::Object(self.base()),
        ).await;
        let capabilities = result.unwrap_or(Value::Null);
        self.image_generation = capabilities.get("generation").and_then(Value::as_bool).unwrap_or(false);
        self.image_editing = capabilities.get("editing").and_then(Value::as_bool).unwrap_or(false);
        if !self.image_generation {
            self.image_models.clear();
        }
    }

    /// 1 RPC。
    async fn control(&self, method: &str, params: Value) -> Result<Value, GatewayError> {
        self.call(method, params, CONTROL_TIMEOUT).await
    }

    /// 1 need RPC（ when ）。
    async fn network(&self, method: &str, params: Value) -> Result<Value, GatewayError> {
        self.call(method, params, NETWORK_TIMEOUT).await
    }

    async fn call(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, GatewayError> {
        // use ensure： after 1 use will new 。
        let execution = self.execution().await?;
        let lease = execution.lease.clone().or_else(|| execution.host.activity.acquire())
            .ok_or_else(|| super::activity::AvailabilityError::Unavailable.gateway())?;
        execution.host
            .ensure()
            .await
            .map_err(|e| {
                if strict::applies(self.id) { strict::fatal(self.id, 503, "runtime_transport_unavailable") }
                else { GatewayError::with_status(503, format!("提供商插件不可用：{e}")) }
            })?;
        let host = execution.host.clone();
        let method = method.to_string();
        let provider = self.id;
        // use etc not can already ， or when after 。
        crate::spawn_task(async move {
            let _lease = lease;
            host.call_coded(&method, params, timeout).await
                .map_err(|error| rpc_error(provider, error, &method))
        }).await.map_err(|_| GatewayError::with_status(503, "插件调用任务中断"))?
    }

    /// provider parameter before （ need provider）。
    fn base(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("provider".into(), Value::String(self.id.to_string()));
        let timeouts = crate::server::config::timeout_settings();
        map.insert("timeouts".into(), serde_json::json!({
            "connectSeconds": timeouts.connect_seconds,
            "headersSeconds": timeouts.headers_seconds,
            "streamIdleSeconds": timeouts.stream_idle_seconds,
            "bodySeconds": timeouts.body_seconds,
        }));
        map
    }

    /// account parameter。
    fn with_account(&self, account: &str) -> Value {
        let mut map = self.base();
        map.insert("account".into(), Value::String(account.to_string()));
        Value::Object(map)
    }
}

/// and head before will error use 1 ， not 。
pub(super) fn rpc_error(provider: &str, error: proto::WireError, method: &str) -> GatewayError {
    if strict::applies(provider) { return strict::wire_error(provider, &error, method); }
    if let Some(failure) = super::errors::control_error(&error) {
        return failure;
    }
    if error.code.as_deref() == Some(proto::WireError::LOCAL_PRECONDITION) {
        // request，556 ， not 502。
        return GatewayError::with_status(
            i32::from(super::super::providers::readiness::LOCAL_PRECONDITION_STATUS),
            error.message,
        );
    }
    let status = match error.code.as_deref() {
        Some(proto::WireError::NO_METHOD | proto::WireError::UNSUPPORTED) => 501,
        Some(proto::WireError::ABORTED) => 499,
        Some("timeout") => 504,
        _ => 502,
    };
    let message = if method == proto::methods::gateway::CONVERSATION {
        error.message
    } else {
        super::errors::operation_failure(method, &error.message)
    };
    let mut failure = GatewayError::with_status(status, message);
    failure.code = error.code;
    failure
}

impl ProviderAdapter for RpcAdapter {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn provider_id(&self) -> &'static str {
        self.id
    }

    fn label(&self) -> &'static str {
        self.label
    }

    fn list_models(&self) -> Vec<Value> {
        // ── Why this is ──────────────────────────────────
        // trait `list_models` is （ before ）。
        // can ， this **return**， by after 
        // `models.list` 。 as empty when return（ when 
        // `provider.info` already ）。
        let mut models = super::rpc_cache::models_for(self.id).unwrap_or_default();
        for image in &self.metadata().image_models {
            let id = crate::server::core::models::model_id(image);
            if let Some(existing) = models.iter_mut().find(|item| crate::server::core::models::model_id(item).eq_ignore_ascii_case(&id)) {
                existing["kind"] = serde_json::json!("image");
                existing["output_modalities"] = image.get("output_modalities").cloned().unwrap_or(serde_json::json!(["image"]));
                existing["supportsToolCall"] = Value::Bool(false);
                existing["isDefault"] = Value::Bool(false);
            } else if image.get("synthetic").and_then(Value::as_bool) == Some(true) {
                models.push(image.clone());
            }
        }
        models
    }

    fn stream_refusal_finishes(&self) -> &'static [&'static str] {
        self.metadata().refusal_finishes
    }

    fn stream_refusal_markers(&self) -> &'static [&'static str] {
        self.metadata().refusal_markers
    }

    fn build_chat_request_async<'a>(
        &'a self,
        account: &'a Value,
        body: &'a Value,
        client_headers: &'a HeaderMap,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("account".into(), Value::String(account_id_of(account)));
            params.insert("proxy".into(), proxy_of(account));
            params.insert("proxy_override".into(), Value::Bool(account.get("proxy").and_then(|proxy| proxy.get("source"))
                .and_then(Value::as_str) == Some("direct")));
            params.insert("body".into(), body.clone());
            params.insert("headers".into(), headers_json(client_headers));
            params.insert("stream".into(), Value::Bool(true));
            // and by use in body already （`upstream::payload`
            // in this ** before **）， this from body 。
            let wire_model = body
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            params.insert("wire_model".into(), Value::String(wire_model));
            params.insert("reasoning".into(), Value::String(String::new()));

            let value = self.network(proto::methods::gateway::CHAT, Value::Object(params)).await?;
            let url = value.get("url").and_then(Value::as_str).unwrap_or("").to_string();
            if url.is_empty() {
                return Err(GatewayError::with_status(502, "插件没有给出上游地址"));
            }
            let headers = value
                .get("headers")
                .and_then(|v| serde_json::from_value::<Vec<(String, String)>>(v.clone()).ok())
                .unwrap_or_default();
            let mut body = value.get("body").cloned().unwrap_or(Value::Null);
            let upstream = value
                .get("upstream")
                .and_then(|v| serde_json::from_value::<proto::UpstreamKind>(v.clone()).ok())
                .unwrap_or_default();
            if upstream == proto::UpstreamKind::Gemini {
                super::gemini_signatures::fill_cached(&mut body);
            }
            Ok(ChatRequestPlan {
                url,
                headers,
                body,
                response: upstream_response(upstream),
                policy: crate::server::core::providers::policy::ChatPolicy::from_value(
                    value.get("policy").unwrap_or(&Value::Null),
                ),
                reasoning: match value.get("reasoning") {
                    Some(Value::Null) => Some(None),
                    Some(Value::String(level)) => Some(crate::server::core::model_rules::normalize_reasoning(level)),
                    _ => None,
                },
            })
        })
    }

    fn supports_image_generation(&self) -> bool {
        self.metadata().image_generation
    }

    fn supports_image_editing(&self) -> bool {
        self.metadata().image_editing
    }

    fn build_image_request_async<'a>(
        &'a self,
        account: &'a Value,
        body: &'a Value,
        client_headers: &'a HeaderMap,
        operation: crate::server::core::providers::images::ImageOperation,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ChatRequestPlan, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            use crate::server::core::providers::images::ImageRequestPlan;
            let params = serde_json::json!({
                "provider": self.id,
                "account": account_id_of(account),
                "proxy": proxy_of(account),
                "body": body,
                "headers": headers_json(client_headers),
                "wire_model": body.get("model").and_then(Value::as_str).unwrap_or(""),
            });
            // RPC 1 limit 16 MiB； in before ， not to 。
            let size = serde_json::to_vec(&params).map_err(|_| GatewayError::bad_request("图片请求无法序列化"))?.len();
            if size > 14 * 1024 * 1024 {
                return Err(GatewayError::with_status(413, "图片请求内联数据总大小不得超过 14 MiB"));
            }
            let value = self.network(operation.method(), params).await?;
            let plan: ImageRequestPlan = serde_json::from_value(value)
                .map_err(|error| {
                    if strict::applies(self.id) { strict::fatal(self.id, 502, "runtime_protocol_error") }
                    else { GatewayError::with_status(502, format!("插件生图计划无法解析：{error}")) }
                })?;
            if plan.url.trim().is_empty() || !plan.body.is_object() {
                return Err(GatewayError::with_status(502, "插件生图计划缺少上游地址或 JSON 请求体"));
            }
            Ok(ChatRequestPlan {
                url: plan.url,
                headers: plan.headers,
                body: plan.body,
                response: UpstreamResponse::Images(plan.response),
                reasoning: None,
                policy: crate::server::core::providers::policy::ChatPolicy::default(),
            })
        })
    }

    fn is_error_response_async<'a>(
        &'a self,
        status: u16,
        headers: &'a HeaderMap,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, GatewayError>> + Send + 'a>> {
        Box::pin(async move {
            if !(200..300).contains(&status) {
                return Ok(true);
            }
            let mut params = self.base();
            params.insert("status".into(), Value::from(status));
            params.insert("headers".into(), headers_json(headers));
            let value = self.control(
                proto::methods::gateway::IS_ERROR_RESPONSE, Value::Object(params),
            ).await?;
            value.as_bool().ok_or_else(|| GatewayError::with_status(
                502, "插件返回的响应类型判定不是布尔值",
            ))
        })
    }

    fn classify_error_async<'a>(
        &'a self,
        status: u16,
        error_body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = UpstreamErrorClass> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("status".into(), Value::from(status));
            params.insert("body".into(), error_body.clone());
            let value = match self
                .control(proto::methods::gateway::CLASSIFY, Value::Object(params))
                .await
            {
                Ok(value) => value,
                Err(e) => {
                    if strict::applies(self.id) {
                        return strict::cached_class(self.id, &strict::fatal(self.id, status, "runtime_classification_unavailable"));
                    }
                    // not to = 。return Fatal is ： not 、
                    // not ， error —— and 「 」 1 。
                    return UpstreamErrorClass::Fatal {
                        status,
                        message: format!("提供商插件无法分类错误：{}", e.message),
                        upstream_code: None,
                    };
                }
            };
            match serde_json::from_value::<proto::ErrorClass>(value) {
                Ok(class) => {
                    let class = upstream_error_class(class);
                    if strict::applies(self.id) {
                        strict::sdk_class(self.id, status, class, error_body)
                    } else { class }
                }
                Err(_) if strict::applies(self.id) => {
                    strict::cached_class(self.id, &strict::fatal(self.id, status, "runtime_protocol_error"))
                }
                Err(e) => UpstreamErrorClass::Fatal {
                    status,
                    message: format!("插件返回的错误分类无法解析：{e}"),
                    upstream_code: None,
                },
            }
        })
    }

    fn error_policy_async<'a>(
        &'a self,
        status: u16,
        body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::server::core::providers::policy::ErrorPolicy> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("status".into(), Value::from(status));
            params.insert("body".into(), body.clone());
            self.control("error.policy", Value::Object(params)).await.ok()
                .map(crate::server::core::providers::policy::ErrorPolicy::from_value)
                .unwrap_or_default()
        })
    }

    fn reasoning_patch_async<'a>(
        &'a self,
        level: &'a str,
        model: &'a str,
        body: &'a Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ReasoningPatch> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("level".into(), Value::String(level.to_string()));
            params.insert("model".into(), Value::String(model.to_string()));
            params.insert("body".into(), body.clone());
            let value = match self
                .control(proto::methods::gateway::REASONING_PATCH, Value::Object(params))
                .await
            {
                Ok(value) => value,
                // not to not —— and 「 this unsupported」 1 ，body 1 not 。
                Err(_) => {
                    return ReasoningPatch::Skip { reason: "提供商插件未响应，跳过档位注入" }
                }
            };
            match serde_json::from_value::<proto::ReasoningPatch>(value) {
                Ok(proto::ReasoningPatch::Set { field, value }) => ReasoningPatch::Set {
                    field: leak_str(field),
                    value,
                },
                Ok(proto::ReasoningPatch::Skip { reason }) => {
                    let reason = if strict::applies(self.id) { "提供商未注入该思考档位".to_string() } else { reason };
                    ReasoningPatch::Skip { reason: leak_str(reason) }
                }
                Err(_) => ReasoningPatch::Skip { reason: "插件返回的档位结果无法解析" },
            }
        })
    }

    fn credentials_expiring_async<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            self.control(
                proto::methods::gateway::TOKEN_EXPIRING,
                self.with_account(account_id),
            )
            .await
            .ok()
            .and_then(|value| value.as_bool())
            // not false： can ， not need up 。
            .unwrap_or(false)
        })
    }

    fn build_chat_request(
        &self,
        _account: &Value,
        _body: &Value,
        _client_headers: &HeaderMap,
    ) -> Result<ChatRequestPlan, GatewayError> {
        // ── in must ──────────────────────────────
        // in [`Self::build_chat_request_async`]。 1 ；
        // to this use in use —— that is ，
        // return 1 and not （ not will has ）。
        Err(GatewayError::with_status(
            501,
            format!(
                "提供商 {} 是插件实现，必须走 build_chat_request_async（同步签名调用链未切换）",
                self.id
            ),
        ))
    }

    fn classify_error(&self, status: u16, _error_body: &Value) -> UpstreamErrorClass {
        if strict::applies(self.id) {
            return strict::cached_class(self.id, &strict::fatal(self.id, status, "runtime_classification_unavailable"));
        }
        // Fatal（ [`Self::classify_error_async`]）。Fatal not 、
        // not —— ， after is 「 this error no 」，
        // not is 「 」。
        UpstreamErrorClass::Fatal {
            status,
            message: String::new(),
            upstream_code: None,
        }
    }

    fn classify_conversation_error(&self, error: &GatewayError) -> UpstreamErrorClass {
        // add scoped runtime already in error； not can new by HTTP 429 quota。
        if strict::applies(self.id) { return strict::cached_class(self.id, error); }
        // ， not can RPC； old has 429/6004 。
        if error.is_quota_limit() {
            let reset_at = crate::server::errors::parse_quota_reset_at(&error.message);
            UpstreamErrorClass::QuotaLimited {
                reset_at: (reset_at > 0).then_some(reset_at),
                status: 429,
                message: error.message.clone(),
                upstream_code: error.upstream_code,
            }
        } else {
            UpstreamErrorClass::Fatal {
                status: u16::try_from(error.status_code).unwrap_or(500),
                message: error.message.clone(),
                upstream_code: error.upstream_code,
            }
        }
    }

    fn supports_chat(&self) -> bool {
        self.metadata().capabilities.chat
    }

    fn is_stateful(&self) -> bool {
        self.metadata().capabilities.stateful
    }

    fn supports_refresh(&self) -> bool {
        self.metadata().capabilities.refresh
    }

    fn supports_model_refresh(&self) -> bool {
        self.metadata().capabilities.model_refresh
    }

    fn supports_usage(&self) -> bool {
        self.metadata().capabilities.usage
    }

    fn supports_web_login(&self) -> bool {
        self.metadata().capabilities.web_login
    }

    fn sse_model_rewrite(&self) -> bool {
        self.metadata().capabilities.sse_model_rewrite
    }

    fn sse_strip_newline_chunks(&self) -> bool {
        self.metadata().capabilities.sse_strip_newline_chunks
    }

    fn allows_anonymous_default_session(&self) -> bool {
        self.metadata().capabilities.anonymous_default_session
    }

    fn system_proxy_when_unset(&self) -> bool {
        self.metadata().capabilities.system_proxy_when_unset
    }

    fn refresh_uses_account(&self) -> bool {
        self.metadata().capabilities.refresh_uses_account
    }

    /// not 「default」 this （ `Capabilities.supports_default_model`）。
    ///
    /// ── Whymust up （ not can default false）───────────────────
    /// in 「default in provider has this 」 when 
    /// `config.defaultModel`。Accio / AtomCode / LobsterAI / Loomy / Raccoon /
    /// Qoder as true； if default false， use default
    /// in this up will ****（request by 「」， up use 
    /// default， up not ）。
    fn supports_default_model(&self) -> bool {
        self.metadata().capabilities.supports_default_model
    }

    /// has no has 「 / default」（ `Capabilities.env_credentials`）。
    ///
    /// ── and 1 （）──────────────────────
    /// （WorkBuddy / CatPaw / AutoClaw / Raccoon） is **** 
    /// and path —— that is 「 has 」。 not to that ，
    /// can 「 has this 」。 `catalog::provider_available` 
    /// will 1 「 has can can no 」 ； after is client can can in to 
    /// 401，「 has not 」。
    fn env_credentials_present(&self) -> bool {
        self.metadata().capabilities.env_credentials
    }

    /// to can （ that return `None`）。
    fn plugin_checkin_capability(&self) -> Option<bool> {
        Some(self.metadata().capabilities.checkin)
    }

    /// ： 1 `admin.action` RPC。
    ///
    /// ── when use `network` not is `control` ───────────────────────
    /// this need up （、、），control 
    /// when will request。
    fn admin_action<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
        action: &'a str,
        payload: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let activity = self.id == "workbuddy-intl" && action == "activity.run"
                && payload.get("mode").and_then(Value::as_str) != Some("claim");
            let mut activity_payload = payload.clone();
            let access = if activity {
                let (models, identity) = crate::server::core::billing::keepalive::authorized_models()?;
                activity_payload["models"] = serde_json::json!(models);
                Some(identity)
            } else { None };
            let mut params = self.with_account(account_id);
            if let Some(object) = params.as_object_mut() {
                object.insert("action".into(), Value::String(action.to_string()));
                object.insert("payload".into(), activity_payload.clone());
            }
            let started_at = crate::server::logging::now_ms();
            let result = self.network(proto::methods::gateway::ADMIN_ACTION, params).await;
            // old SDK will 「 no this 」 INTERNAL； for this 
            // as 501， not can failure or its error 1 and 。
            let result = result.map_err(|mut error| {
                if action == "auth.methods" && error.message.contains("auth.methods")
                    && ["没有实现管理动作", "不认识管理动作", "不支持管理动作", "unknown action", "unsupported action"]
                        .iter().any(|marker| error.message.contains(marker))
                {
                    error.status_code = 501;
                }
                error
            });
            if let Some(identity) = access {
                crate::server::core::billing::keepalive::record_activity(account_id, started_at, &activity_payload, &result, identity);
            }
            result
        })
    }

    fn ensure_access_token<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let value = self
                .network(proto::methods::gateway::TOKEN_ENSURE, self.with_account(account_id))
                .await?;
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| GatewayError::with_status(502, "插件返回的 token 不是字符串"))
        })
    }

    fn refresh_access_token<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let value = self
                .network(proto::methods::gateway::TOKEN_REFRESH, self.with_account(account_id))
                .await?;
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| GatewayError::with_status(502, "插件返回的 token 不是字符串"))
        })
    }

    fn credentials_expiring(&self, _store: &AccountStore, _account_id: &str) -> bool {
        // in [`Self::credentials_expiring_async`]。return
        // `false`（= 「 not ， not use 」） is **** to ： after this ，
        // new 。return `true` will 
        // 1 。
        false
    }

    fn refresh_models<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
        force: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ModelRefreshOutcome> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("account".into(), Value::String(account_id.to_string()));
            params.insert("force".into(), Value::Bool(force));
            match self
                .network(proto::methods::gateway::MODELS_REFRESH, Value::Object(params))
                .await
            {
                Ok(value) => match serde_json::from_value::<proto::RefreshOutcome>(value.clone()) {
                    Ok(outcome) => {
                        if !outcome.refreshed {
                            return ModelRefreshOutcome {
                                refreshed: false,
                                count: outcome.count,
                                message: if strict::applies(self.id) {
                                    outcome.message.map(|_| "上游目录未能刷新，已保留可用清单".into())
                                } else { outcome.message },
                            };
                        }
                        let list = if crate::server::core::providers::is_scoped_runtime_provider(self.id) {
                            match serde_json::from_value::<proto::ModelList>(value) {
                                Ok(list) if list.meta.remote && outcome.count == list.models.len() => list,
                                _ => return ModelRefreshOutcome::failed("插件刷新缺少同次真实远程模型快照，已保留旧清单"),
                            }
                        } else { match self.control(
                            proto::methods::gateway::MODELS_LIST,
                            self.with_account(account_id),
                        ).await {
                            Ok(value) => match serde_json::from_value::<proto::ModelList>(value) {
                                Ok(list) => list,
                                Err(error) => return ModelRefreshOutcome::failed(
                                    format!("插件刷新后返回的模型清单无法解析: {error}"),
                                ),
                            },
                            Err(error) => return ModelRefreshOutcome::failed(error.message),
                        } };
                        let count = super::rpc_cache::set_remote(self.id, list);
                        if count == 0 && !crate::server::core::providers::is_scoped_runtime_provider(self.id) {
                            return ModelRefreshOutcome::failed("插件刷新后没有可用模型，已保留旧清单");
                        }
                        crate::server::core::providers::adapter::seed_hidden_visibility_defaults();
                        ModelRefreshOutcome {
                            refreshed: true,
                            count,
                            message: if strict::applies(self.id) {
                                outcome.message.map(|_| "提供商模型目录已刷新".into())
                            } else { outcome.message },
                        }
                    }
                    Err(_) if strict::applies(self.id) => ModelRefreshOutcome::failed("插件返回的模型刷新结果无法解析，原始诊断已隐藏"),
                    Err(e) => ModelRefreshOutcome::failed(format!("插件返回形状不对: {e}")),
                },
                Err(e) => ModelRefreshOutcome::failed(e.message),
            }
        })
    }

    fn query_usage<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>>
    {
        Box::pin(async move {
            self.network(proto::methods::gateway::USAGE_QUERY, self.with_account(account_id))
                .await
        })
    }

    fn forward_conversation<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
        body: &'a Value,
        client_headers: &'a HeaderMap,
        proxy: Option<ResolvedProxy>,
        stream: bool,
        telemetry: &'a Arc<RequestTelemetry>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ForwardOutcome, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let execution = self.execution().await?;
            let host = execution.host.clone();
            let lease = execution.lease.clone().or_else(|| host.activity.acquire())
                .ok_or_else(|| super::activity::AvailabilityError::Unavailable.gateway())?;
            let mut params = self.base();
            params.insert("account".into(), Value::String(account_id.to_string()));
            params.insert("proxy".into(), proxy.map_or(Value::Null, |proxy| wire_proxy(proxy)));
            params.insert("body".into(), body.clone());
            params.insert("headers".into(), headers_json(client_headers));
            // ── `stream: true` ────────────────────────────────
            // client need not need is **** ： when 
            // 1 JSON（`aggregate_frame_stream`， and 1 ）。
            // this 1 path（ chat SSE ）， not 1 
            // 「」—— then will ， is 
            // 「 and not 1 」，。
            params.insert("stream".into(), Value::Bool(true));
            params.insert("session".into(), Value::String(session_of(client_headers)));
            // and `build_chat_request_async`：body already is 
            // （ in this before ）。
            let wire_model = body.get("model").and_then(Value::as_str).unwrap_or("").to_string();
            params.insert("wire_model".into(), Value::String(wire_model.clone()));

            let strict = strict::applies(self.id);
            let call = host
                .open_stream(proto::methods::gateway::CONVERSATION, Value::Object(params))
                .await
                .map_err(|e| {
                    if strict { strict::fatal(self.id, 503, "runtime_transport_unavailable") }
                    else { GatewayError::with_status(503, format!("提供商插件不可用：{e}")) }
                })?;
            let id = call.id;
            let mut events = call.events;
            // head before must ， not can use 。
            let mut opening = Some(ErrorBodyScope {
                host: host.clone(), id, abort_on_drop: true, lease: Some(lease.clone()),
            });

            // ── etc head： this when in up request ────────────────
            // when use 「 etc response」 that （set「request when 」）：
            // has in this 1 need round + turn ，none，
            // is 「client no to 」 。
            //
            // ── Why this etc is 「 1 」 not is 「head 」──────────
            // can can in head ** before **failure（、 up not up ），
            // that when to is or chunk（error）。 head will 
            // etc when ， after 1 「 when 」—— Reason in 。
            let budget = Duration::from_millis(crate::server::config::timeout_settings().headers_ms());
            let receive = tokio::time::timeout(budget, events.recv());
            let received = if strict {
                if let Some(token) = telemetry.cancel_token() {
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => return Err(crate::server::core::upstream::cancellation::cancelled_error()),
                        result = receive => result,
                    }
                } else { receive.await }
            } else { receive.await };
            let first = match received {
                Ok(Some(event)) => event,
                // ：（ no ）
                Ok(None) => {
                    host.close_stream(id);
                    return Err(GatewayError::with_status(
                        502,
                        "提供商插件在给出响应头之前退出",
                    ));
                }
                Err(_) => {
                    // when ： this use ，otherwise will 1 to up when ，
                    // client to error —— that up is 。
                    host.abort(id).await;
                    host.close_stream(id);
                    return Err(GatewayError::with_status(
                        if strict { 504 } else { 502 },
                        format!("等待上游响应头超时（{}秒）", budget.as_secs()),
                    ));
                }
            };

            let head = match first {
                super::StreamEvent::Head(head) => head,
                // and 1 /； has provider_http 。
                super::StreamEvent::Done(Err(error)) => {
                    if let Some(scope) = opening.take() { scope.finish(); }
                    host.close_stream(id);
                    if strict {
                        if let Some((status, detail)) = strict::private_control(&error) {
                            let class = self.classify_error_async(status, &detail).await;
                            return Err(strict::conversation(self.id, status, class, &detail, None));
                        }
                        return Err(rpc_error(self.id, error, proto::methods::gateway::CONVERSATION));
                    }
                    let provider_http = super::errors::control_error(&error).is_some();
                    let mut failure = rpc_error(self.id, error.clone(), proto::methods::gateway::CONVERSATION);
                    if provider_http {
                        // has 1 {code: up , message}，。
                        let mut detail = serde_json::json!({
                            "message": failure.message,
                            "code": failure.upstream_code.map(Value::from).unwrap_or(Value::Null),
                            "providerCode": failure.code,
                            "upstreamCode": failure.upstream_code,
                        });
                        if let Some(Value::Object(metadata)) = error.code.as_deref()
                            .and_then(|code| code.strip_prefix("provider_http:"))
                            .and_then(|payload| serde_json::from_str::<Value>(payload).ok())
                        {
                            for key in ["signIn", "credentialScope"] {
                                if let Some(value) = metadata.get(key).and_then(Value::as_str) {
                                    detail[key] = Value::String(value.to_string());
                                }
                            }
                        }
                        let status = failure.status_code as u16;
                        let class = self.classify_error_async(status, &detail).await;
                        let mut classified = conversation_error(status, class, &detail);
                        classified.code = failure.code;
                        classified = classified.with_optional_code(failure.upstream_code);
                        failure = classified;
                    }
                    return Err(failure);
                }
                super::StreamEvent::Done(Ok(_)) => {
                    if let Some(scope) = opening.take() { scope.finish(); }
                    host.close_stream(id);
                    return Err(GatewayError::with_status(
                        502,
                        "插件在没有给出响应头的情况下结束了会话转发",
                    ));
                }
                super::StreamEvent::Failed(message) => {
                    host.close_stream(id);
                    return Err(if strict { strict::fatal(self.id, 502, "runtime_stream_interrupted") }
                        else { GatewayError::with_status(502, message) });
                }
                // to is ： need head， 「 200」
                // has use —— 1 not 。
                super::StreamEvent::Chunk(_) if strict => {
                    return Err(GatewayError::with_status(502, "辅助程序在真实响应头之前发送正文，未猜测成功"));
                }
                super::StreamEvent::Chunk(bytes) => {
                    let status = 200;
                    if let Some(scope) = opening.take() { scope.handoff(); }
                    return execution
                        .finish_stream(id, status, stream, events, Some(bytes), wire_model, telemetry, None, lease)
                        .await;
                }
            };

            if let Some(scope) = opening.take() { scope.handoff(); }
            execution.finish_stream(id, head.status, stream, events, None, wire_model, telemetry, Some(head.headers), lease)
                .await
        })
    }

    fn retry_advice(&self, _error_body: &Value, _attempt: usize, _budget: usize) -> Option<RetryAdvice> {
        // in [`Self::retry_advice_async`]（ no await）。
        // return None = not ： has `transient_retry_advice` /
        // `fallback_retry_advice` use ， not will can 。
        None
    }

    fn retry_advice_async<'a>(
        &'a self,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<RetryAdvice>> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("body".into(), error_body.clone());
            params.insert("attempt".into(), Value::from(attempt));
            params.insert("budget".into(), Value::from(budget));
            let value = match self
                .control(proto::methods::gateway::RETRY_ADVICE, Value::Object(params))
                .await
            {
                Ok(value) => value,
                // not to = not 。 and 「 this not 」 1 ， to 。
                Err(_) => return None,
            };
            if value.is_null() {
                return None;
            }
            serde_json::from_value::<proto::RetryAdvice>(value)
                .ok()
                .map(|advice| RetryAdvice {
                    delay_ms: advice.delay_ms,
                    reason: if strict::applies(self.id) { "提供商建议等待后重试".to_string() } else { advice.reason },
                })
        })
    }

    fn retry_decision_async<'a>(
        &'a self,
        status: u16,
        error_body: &'a Value,
        attempt: usize,
        budget: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = RetryDecision> + Send + 'a>> {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("status".into(), Value::from(status));
            params.insert("body".into(), error_body.clone());
            params.insert("attempt".into(), Value::from(attempt));
            params.insert("budget".into(), Value::from(budget));
            let method = crate::server::core::providers::retry::RETRY_DECISION_METHOD;
            match self.control(method, Value::Object(params)).await {
                Ok(value) => {
                    let decision = serde_json::from_value(value).unwrap_or_else(|_| RetryDecision::Stop {
                        reason: "提供商插件返回的重试决定无法解析，停止本次请求".to_string(),
                    });
                    if strict::applies(self.id) { strict::retry_decision(decision) } else { decision }
                }
                Err(error) if error.status_code == 501 => {
                    match self.retry_advice_async(error_body, attempt, budget).await {
                        Some(advice) => RetryDecision::Retry {
                            delay_ms: advice.delay_ms,
                            reason: advice.reason,
                        },
                        None => RetryDecision::Default,
                    }
                }
                Err(error) => RetryDecision::Stop {
                    reason: format!("提供商插件无法给出重试决定，停止本次请求：{}", error.message),
                },
            }
        })
    }

    fn reasoning_patch(&self, _level: &str, _model: &str, _body: &Value) -> ReasoningPatch {
        // in [`Self::reasoning_patch_async`]。 Skip = not 
        // （body 1 not ）， is 。
        ReasoningPatch::Skip { reason: "插件 provider 的档位翻译要走异步入口" }
    }

    fn build_login_url(&self) -> Option<(String, String)> {
        // none RPC（ need await）。 in 
        // [`Self::build_login_url_async`]，`core::login` already as that 。
        None
    }

    fn build_login_url_async<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<(String, String)>> + Send + 'a>,
    > {
        Box::pin(async move {
            let start = self.login_start_async().await?;
            // `flow` is need for state（、）
            Some((start.url, start.flow))
        })
    }

    /// ：** `poll_interval_ms`** —— is 「 etc 」 and 
    /// 「」 1 （ `ProviderAdapter::login_start_async`
    /// ）。
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
            self.build_login_flow_with_options(serde_json::json!({})).await.ok().flatten()
        })
    }

    fn build_login_flow_with_options<'a>(
        &'a self,
        options: Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<proto::LoginStart>, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            if !options.is_object() {
                return Err(GatewayError::bad_request("登录参数必须是 JSON 对象"));
            }
            // method old browser ； OAuth / API by payload in 
            // methodIndex 。 old SDK will will method and payload to provider。
            let params = serde_json::json!({
                "provider": self.id,
                "method": "browser",
                "proxy": options.get("proxy").cloned().unwrap_or(Value::Null),
                "payload": options,
            });
            let value = self.control(proto::methods::gateway::LOGIN_START, params).await?;
            if value.is_null() {
                return Ok(None);
            }
            let start: proto::LoginStart = serde_json::from_value(value).map_err(|_| {
                GatewayError::with_status(502, "插件返回的登录启动信息无法解析")
            })?;
            if start.flow.trim().is_empty() {
                return Err(GatewayError::with_status(502, "插件没有返回登录流程 id"));
            }
            // empty URL is / ， not success； 1 by by and 。
            Ok(Some(start))
        })
    }

    /// 1 （： in up ， not to ）。
    fn login_poll_async<'a>(
        &'a self,
        flow: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<proto::LoginPoll, GatewayError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // need （ up 「 no 」）， use when 
            let params = serde_json::json!({ "provider": self.id, "flow": flow });
            let value = self
                .network(proto::methods::gateway::LOGIN_POLL, params)
                .await?;
            let poll = serde_json::from_value::<proto::LoginPoll>(value).map_err(|e| {
                if strict::applies(self.id) { strict::fatal(self.id, 502, "runtime_protocol_error") }
                else { GatewayError::with_status(502, format!("插件返回的登录进度无法解析：{e}")) }
            })?;
            Ok(if strict::applies(self.id) { strict::login_poll(poll) } else { poll })
        })
    }

    fn normalize_account<'a>(
        &'a self,
        _store: &'a AccountStore,
        input: proto::NormalizeInput,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<proto::NormalizedAccount, GatewayError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            // can can need （ / ）， use when 
            let params = serde_json::to_value(&input).map_err(|e| {
                if strict::applies(self.id) { GatewayError::bad_request("添加账号的参数无法序列化，原始诊断已隐藏") }
                else { GatewayError::with_status(400, format!("添加账号的参数无法序列化：{e}")) }
            })?;
            let value = self
                .network(proto::methods::gateway::ACCOUNT_NORMALIZE, params)
                .await?;
            serde_json::from_value::<proto::NormalizedAccount>(value).map_err(|e| {
                if strict::applies(self.id) { strict::fatal(self.id, 502, "runtime_protocol_error") }
                else { GatewayError::with_status(502, format!("插件返回的账号形状不对：{e}")) }
            })
        })
    }

    /// to after ： and use [`Self::save_login_account`]。
    ///
    /// ── Whymust use ──────────────────────────────────────────
    /// （ / ） `NormalizedAccount` ， then 
    /// （id identity、 `public.name`、 `plugin_upsert_account`）
    /// can has 1 —— is 「 and in 
    /// not 1 」。
    fn persist_login_account(
        &self,
        account: &proto::NormalizedAccount,
    ) -> Result<String, GatewayError> {
        self.save_login_account(account)
    }

    fn login_callback_poll_async<'a>(
        &'a self,
        _store: &'a AccountStore,
        callback: proto::LoginCallback,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Option<proto::LoginPoll>, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let params = serde_json::json!({
                "provider": self.id,
                "flow": callback.flow,
                "code": callback.code,
                "url": callback.url,
            });
            let value = self.network(proto::methods::gateway::LOGIN_CALLBACK, params).await?;
            let poll = serde_json::from_value::<proto::LoginPoll>(value).map_err(|_| {
                GatewayError::with_status(502, "插件返回的登录结果无法解析")
            })?;
            // not in RPC ：Pending is success，Done by 1 。
            Ok(Some(if strict::applies(self.id) { strict::login_poll(poll) } else { poll }))
        })
    }

    fn login_cancel_async<'a>(
        &'a self,
        flow: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let params = serde_json::json!({ "provider": self.id, "flow": flow });
            self.control(proto::methods::gateway::LOGIN_CANCEL, params).await.map(|_| ())
        })
    }

    fn exchange_login_code<'a>(
        &'a self,
        store: &'a AccountStore,
        code: &'a str,
        state: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // old use can can code， can can URL。 new 
            // ， code after state or parameter。
            let credential = url::Url::parse(code).ok().and_then(|url| {
                url.query_pairs().find(|(key, _)| key == "code" || key == "authorization_code")
                    .map(|(_, value)| value.into_owned())
            }).unwrap_or_else(|| code.to_string());
            let callback = proto::LoginCallback { flow: state.to_string(), code: credential, url: code.to_string() };
            match self.login_callback_poll_async(store, callback).await? {
                Some(proto::LoginPoll::Done { account }) => self.save_login_account(&account),
                Some(proto::LoginPoll::Failed { message }) => Err(GatewayError::with_status(400, message)),
                // old String API none pending； new by 1 callback-poll API。
                Some(proto::LoginPoll::Pending { .. }) | None => Err(GatewayError::with_status(
                    502, "插件尚未完成登录回调，请继续轮询登录任务",
                )),
            }
        })
    }

    /// to ：`checkin.claim` RPC → `{success, msg}`。
    ///
    /// ── Why in this not is to that ────────────────
    /// `CheckinReport` has outcome（`claimed` / `done` /
    /// `ineligible` / `inactive` / `captcha` / `failed`）， that 
    /// ** 1 **（「 already 」 and 「 to 」must： before 
    /// not `checkinAt` ， after need ）。 
    /// `{success, msg}` when outcome，Done use alreadyCompleted ，
    /// and to 、 already and none。
    fn checkin_claim<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        self.checkin_claim_with_options(store, account_id, false)
    }

    fn plugin_account_view<'a>(
        &'a self,
        secret: &'a Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("secret".into(), secret.clone());
            self.control(proto::methods::gateway::ACCOUNT_VIEW, Value::Object(params)).await
        })
    }

    fn checkin_claim_with_options<'a>(
        &'a self,
        store: &'a AccountStore,
        account_id: &'a str,
        manual: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("account".into(), Value::String(account_id.to_string()));
            params.insert("manual".into(), Value::Bool(manual));
            let value = self.network(proto::methods::gateway::CHECKIN_CLAIM, Value::Object(params)).await?;
            let retry_at = value.get("retryAt").cloned();
            let details = value.get("details").cloned();
            let report: proto::CheckinReport = serde_json::from_value(value).map_err(|e| {
                if strict::applies(self.id) { strict::fatal(self.id, 502, "runtime_protocol_error") }
                else { GatewayError::with_status(502, format!("插件返回的签到结果无法解析：{e}")) }
            })?;
            let (success, default_message) = match report.outcome {
                proto::CheckinOutcome::Claimed => (true, "签到成功"),
                proto::CheckinOutcome::Done => (false, "今天已经签过了"),
                proto::CheckinOutcome::Ineligible => (false, "该账号不参与签到"),
                proto::CheckinOutcome::Inactive => (false, "当前没有可领的活动"),
                proto::CheckinOutcome::Captcha => {
                    (false, "上游要求验证码，请到官方客户端手动签到")
                }
                proto::CheckinOutcome::Failed => (false, "签到失败"),
            };
            let message = if strict::applies(self.id) { default_message } else { report
                .message
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .unwrap_or(default_message) };
            let mut claim = serde_json::json!({
                "success": success,
                "alreadyCompleted": report.outcome == proto::CheckinOutcome::Done,
                "msg": message,
                "outcome": format!("{:?}", report.outcome).to_lowercase(),
                "credit": report.credit,
                "streak": report.streak,
            });
            if let Some(retry_at) = retry_at {
                claim["retryAt"] = retry_at;
            }
            if !strict::applies(self.id) {
                if let Some(details) = details {
                    if let Some(results) = details.get("results").and_then(Value::as_array) {
                        claim["results"] = Value::Array(results.clone());
                        if crate::server::core::providers::zcode::region::Region::from_provider_id(self.id).is_some() {
                            for row in results {
                                if row.get("success").and_then(Value::as_bool) == Some(true)
                                    || row.get("alreadyCompleted").and_then(Value::as_bool) == Some(true)
                                {
                                    let plan = row.get("planId").and_then(Value::as_str).unwrap_or("");
                                    let at = row.get("claimedAt").and_then(Value::as_i64)
                                        .filter(|at| *at > 0).unwrap_or_else(crate::server::logging::now_ms);
                                    if !plan.is_empty() && !store.mark_zcode_claim(account_id, plan, at) {
                                        crate::server::logging::verbose("[Checkin]", "套餐领取明细未能落盘");
                                    }
                                }
                            }
                        }
                    }
                    if let Some(state) = details.get("dailyCheckin").filter(|state| state.is_object()) {
                        claim["dailyCheckin"] = state.clone();
                        if let Err(message) = store.plugin_set_ledger(account_id, "dailyCheckin", state) {
                            crate::server::logging::verbose("[Checkin]", &message);
                        }
                    }
                }
            }
            Ok(claim)
        })
    }

    /// ：`claim` RPC， to use 。
    fn claim<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
        claim: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut params = self.base();
            params.insert("account".into(), Value::String(account_id.to_string()));
            params.insert("claim".into(), Value::String(claim.to_string()));
            self.network(proto::methods::gateway::CLAIM, Value::Object(params))
                .await
        })
    }

    /// ：`claim.status` RPC（，）。
    fn claim_status<'a>(
        &'a self,
        _store: &'a AccountStore,
        account_id: &'a str,
        refresh: bool,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Value, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let mut params = self.with_account(account_id);
            if refresh {
                if let Some(object) = params.as_object_mut() {
                    object.insert("refresh".into(), Value::Bool(true));
                }
            }
            self.network(proto::methods::gateway::CLAIM_STATUS, params)
                .await
        })
    }
}

/// `UpstreamKind` to `UpstreamResponse`。
///
/// ── Why this must ──────────────────────────────────
/// 1 in `panic=abort` down is error（ match will ），
/// this not `_ =>` —— will when will 。
pub(crate) fn upstream_response(kind: proto::UpstreamKind) -> UpstreamResponse {
    match kind {
        proto::UpstreamKind::Chat => UpstreamResponse::Chat,
        proto::UpstreamKind::Anthropic => UpstreamResponse::Anthropic,
        proto::UpstreamKind::Ndjson => UpstreamResponse::AiSdkNdjson,
        proto::UpstreamKind::Gemini => UpstreamResponse::AntigravityGemini,
        proto::UpstreamKind::ChatglmSnapshot => UpstreamResponse::ChatglmSnapshot,
    }
}

/// `ErrorClass` to `UpstreamErrorClass`。
///
/// ── 1 ──────────────────────────────────────
/// and is ** 1 **（ that is 1 ，
/// `ProviderAdapter::classify_error` ）。 this 。
fn upstream_error_class(class: proto::ErrorClass) -> UpstreamErrorClass {
    match class {
        proto::ErrorClass::QuotaLimited { reset_at, message, upstream_code, status } => {
            UpstreamErrorClass::QuotaLimited { reset_at, message, upstream_code, status }
        }
        proto::ErrorClass::TokenExpired { message } => {
            UpstreamErrorClass::TokenExpired { message }
        }
        proto::ErrorClass::ContentBlocked { status, message, upstream_code } => {
            UpstreamErrorClass::ContentBlocked { status, message, upstream_code }
        }
        proto::ErrorClass::Fatal { status, message, upstream_code } => {
            UpstreamErrorClass::Fatal { status, message, upstream_code }
        }
    }
}

impl RpcAdapter {
    /// 。
    ///
    /// ── Why id by to ──────────────────────────────────────
    /// in when already （uid / / ）。 use that 
    /// id can 「 1 」 update not is add —— not 
    /// 1 then 。
    fn save_login_account(&self, account: &proto::NormalizedAccount) -> Result<String, GatewayError> {
        let Some(store) = crate::server::core::account_store::plugin_accounts::shared() else {
            return Err(GatewayError::with_status(
                503,
                "账号库尚未就绪，无法保存登录结果",
            ));
        };
        let id = account.identity.clone();
        let name = account
            .public
            .get("name")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(self.label);
        let record = store
            .plugin_upsert_account(&id, self.id, name, &account.secret, &account.public, account.desktop)
            .map_err(|message| {
                if strict::applies(self.id) { GatewayError::with_status(500, "保存提供商登录结果失败，原始诊断已隐藏") }
                else { GatewayError::with_status(500, message) }
            })?;
        Ok(record.get("id").and_then(Value::as_str).unwrap_or(&id).to_string())
    }

    /// head already to after ：error / / 。
    ///
    /// ── Why 1 ──────────────────────────────────────
    /// this need to head， head can can 「 1 」 or 
    /// 「 to 」（ `forward_conversation` ）。 in 1 ，
    /// not will 1 —— that is 「 1 to down 
    /// error 」。
    async fn finish_stream(
        &self,
        id: i64,
        status: u16,
        stream: bool,
        events: tokio::sync::mpsc::UnboundedReceiver<super::StreamEvent>,
        pending: Option<Vec<u8>>,
        wire_model: String,
        telemetry: &Arc<RequestTelemetry>,
        response_headers: Option<std::collections::BTreeMap<String, String>>,
        lease: Arc<super::activity::UsageLease>,
    ) -> Result<ForwardOutcome, GatewayError> {
        // ── failure：（ and 1 ）────────
        // `status >= 400` when will chunk（ that is error ），client
        // need is 1 ** ** JSON error， not is 1 SSE。
        if status >= 400 {
            let scope = ErrorBodyScope {
                host: self.host.clone(), id, abort_on_drop: true, lease: Some(lease.clone()),
            };
            let mut error_private = None;
            let collected = match collect(self.id, &self.host, id, events, pending, telemetry, &mut error_private).await {
                Ok(bytes) => {
                    scope.finish(); // 已收到正常终态，只清登记，不额外 abort。
                    bytes
                }
                Err(error) => {
                    scope.abort().await;
                    if strict::applies(self.id) && telemetry.is_cancelled() {
                        return Err(crate::server::core::upstream::cancellation::cancelled_error());
                    }
                    if let Some((error_status, detail)) = error_private {
                        let error_status = if matches!(status, 499 | 504) { status }
                            else if matches!(error_status, 499 | 504) { error_status }
                            else if status == 556 { status } else { error_status };
                        let class = self.classify_error_async(error_status, &detail).await;
                        return Err(strict::conversation(self.id, error_status, class, &detail, response_headers.as_ref()));
                    }
                    if strict::applies(self.id) && (matches!(status, 499 | 504)
                        || status == 556 && !matches!(error.status_code, 499 | 504)) {
                        let code = if status == 556 { proto::WireError::LOCAL_PRECONDITION } else { "runtime_stream_interrupted" };
                        return Err(strict::fatal(self.id, status, code));
                    }
                    return Err(error);
                }
            };
            let text = String::from_utf8_lossy(&collected);
            let error = match serde_json::from_str::<Value>(&text) {
                Ok(value) => value,
                // not is JSON： message， can by （ 
                // error not ， no has ）。
                Err(_) => Value::String(text.to_string()),
            };
            // 1 RPC （ ）。
            let class = self.classify_error_async(status, &error).await;
            if strict::applies(self.id) {
                return Err(strict::conversation(self.id, status, class, &error, response_headers.as_ref()));
            }
            return Err(conversation_error(status, class, &error));
        }

        let strict = strict::applies(self.id);
        let failure = strict.then(|| Arc::new(std::sync::Mutex::new(None)));
        let private_failure = strict.then(|| Arc::new(std::sync::Mutex::new(None)));
        let source = super::RpcStream::with_pending(
            self.id,
            self.host.clone(),
            id,
            events,
            pending,
            telemetry.clone(),
            failure.clone(),
            private_failure.clone(),
            lease,
        );

        // ── ： （ by stream:true ）───────────
        // client need not need is **** ： when （ and 
        // 1 `aggregate_frame_stream`）。 1 path，
        // not 1 「」—— then ， is 
        // 「 and not 1 」，。
        if !stream {
            let policy = crate::server::core::upstream::sse::FramePolicy::assembled(
                self.metadata().capabilities
                    .sse_model_rewrite
                    .then(|| crate::server::core::upstream::sse::ModelRewrite {
                        requested: wire_model,
                    }),
                self.metadata().capabilities.sse_strip_newline_chunks,
                crate::server::config::strip_newline_keepalive(),
            );
            let aggregated = crate::server::core::upstream::aggregate::aggregate_frame_stream(
                Box::pin(source),
                telemetry.clone(),
                policy,
            )
            .await;
            // in HTTP/ before error； not then 。
            if strict && telemetry.is_cancelled() {
                return Err(crate::server::core::upstream::cancellation::cancelled_error());
            }
            if strict && aggregated.as_ref().err().is_some_and(|error| error.message.starts_with("非流式响应超时(")) {
                return Err(strict::fatal(self.id, 504, "upstream_timeout"));
            }
            let upstream = failure.and_then(|slot| {
                let mut guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.take()
            });
            if let Some(error) = upstream {
                let private = private_failure.and_then(|slot| {
                    let mut guard = slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    guard.take()
                });
                if strict::stopped(&error) || error.conversation_class.is_some() && private.is_none() {
                    return Err(strict::reproject(self.id, &error));
                }
                let status = u16::try_from(error.status_code).unwrap_or(502);
                let detail = strict::classification_input(&error, private.as_ref());
                let class = self.classify_error_async(status, &detail).await;
                let mut classified = strict::conversation(self.id, status, class, &detail, response_headers.as_ref());
                strict::preserve_retry(&mut classified, &error);
                return Err(classified);
            }
            // drop when will `close_stream` + abort（ `RpcStream`）
            return match aggregated {
                Ok(done) => Ok(ForwardOutcome::Completion { body: done.body }),
                Err(error) => {
                    if strict { return Err(strict::reproject(self.id, &error)); }
                    Err(error)
                }
            };
        }

        Ok(ForwardOutcome::Stream { status, stream: Box::new(source) })
    }
}

/// 1 `&'static [&'static str]`。
///
/// ── Why need need ────────────────────────────────────────────
/// trait `stream_refusal_finishes` / `stream_refusal_markers` return
/// `&'static [&'static str]`（return）。 to is 。
/// 、 1 1 —— this is 「 not 21 」 and 
/// 「」 。
fn leak_slice(values: Vec<String>) -> &'static [&'static str] {
    let leaked: Vec<&'static str> = values.into_iter().map(leak_str).collect();
    Box::leak(leaked.into_boxed_slice())
}

/// （ [`leak_all`]）。
fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

/// up 。
///
/// ── Why not use `proxy_of` ──────────────────────────────────
/// that from ** will **（JSON）， `forward_conversation` to 
/// is already （ in when ）。 1 JSON will 
/// 1 can can failure —— failure when ， use 。
fn wire_proxy(proxy: ResolvedProxy) -> Value {
    serde_json::to_value(proto::WireProxy {
        protocol: proxy.protocol,
        host: proxy.host,
        port: proxy.port,
        username: proxy.username,
        password: proxy.password,
    })
    .unwrap_or(Value::Null)
}

/// from client will （`x-session-id`）。
///
/// 1 then and 1 （ `catpaw::conversation::session_id_from_headers`）：
/// empty / 256 1 「 no has will id」—— id will not 
/// client will in before 256 when ， that 「 no has will 」。
fn session_of(headers: &HeaderMap) -> String {
    let value = headers
        .get("x-session-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .trim();
    if value.is_empty() || value.chars().count() > 256 {
        return String::new();
    }
    value.to_string()
}

/// error when ， use Future drop need producer and 。
struct ErrorBodyScope {
    host: Arc<Host>,
    id: i64,
    abort_on_drop: bool,
    lease: Option<Arc<super::activity::UsageLease>>,
}

impl ErrorBodyScope {
    fn handoff(mut self) {
        self.abort_on_drop = false;
        self.id = 0; // 后续 RpcStream/ErrorBodyScope 接管正数 id，不能此刻清登记。
    }

    fn finish(mut self) {
        self.abort_on_drop = false;
    }

    async fn abort(mut self) {
        self.abort_on_drop = false;
        let host = self.host.clone();
        let id = self.id;
        let lease = self.lease.take();
        let abort = crate::spawn_task(async move {
            let _lease = lease;
            host.abort(id).await;
            host.close_stream(id);
        });
        let _ = tokio::time::timeout(CONTROL_TIMEOUT, abort).await;
    }
}

impl Drop for ErrorBodyScope {
    fn drop(&mut self) {
        if self.id == 0 { return; }
        self.host.close_stream(self.id);
        if self.abort_on_drop {
            let host = self.host.clone();
            let id = self.id;
            let lease = self.lease.take();
            crate::spawn_task(async move {
                let _lease = lease;
                host.abort(id).await;
            });
        }
    }
}

/// failureresponse ； in Done(Ok) after return， not / in response。
async fn collect(
    provider: &str,
    host: &Arc<Host>,
    id: i64,
    mut events: tokio::sync::mpsc::UnboundedReceiver<super::StreamEvent>,
    pending: Option<Vec<u8>>,
    telemetry: &RequestTelemetry,
    private_failure: &mut Option<(u16, Value)>,
) -> Result<Vec<u8>, GatewayError> {
    const MAX_ERROR_BODY_BYTES: usize = 1 << 20;
    let settings = crate::server::config::timeout_settings();
    let body = Duration::from_millis(settings.body_ms());
    let idle = Duration::from_millis(settings.stream_idle_ms());
    let started = tokio::time::Instant::now();
    let deadline = started + body;
    let mut idle_deadline = started + idle;
    let body_timeout = || GatewayError::with_status(
        if strict::applies(provider) { 504 } else { 502 }, format!("读取提供商错误响应体超时（{}秒）", body.as_secs()),
    );
    let idle_timeout = || GatewayError::with_status(
        if strict::applies(provider) { 504 } else { 502 }, format!("读取提供商错误响应体空闲超时（{}秒）", idle.as_secs()),
    );
    let token = telemetry.cancel_token();
    let cancelled = async move {
        match token {
            Some(token) => token.cancelled().await,
            None => std::future::pending::<()>().await,
        }
    };
    tokio::pin!(cancelled);
    let mut pending = pending.map(super::StreamEvent::Chunk);
    let mut out = Vec::new();
    loop {
        let event = tokio::select! {
            biased;
            _ = &mut cancelled => return Err(crate::server::core::upstream::cancellation::cancelled_error()),
            _ = tokio::time::sleep_until(deadline) => return Err(body_timeout()),
            _ = tokio::time::sleep_until(idle_deadline) => return Err(idle_timeout()),
            event = async {
                match pending.take() {
                    Some(event) => Some(event),
                    None => events.recv().await,
                }
            } => event,
        };
        match event {
            Some(super::StreamEvent::Chunk(bytes)) => {
                // already chunk（ pending ） 1 ， not id/n。
                // credit ； etc total deadline、empty and 。
                let n = bytes.len() + proto::FRAME_OVERHEAD_BYTES;
                let credit_host = host.clone();
                let credit = tokio::spawn(async move { credit_host.credit(id, n).await });
                if bytes.len() >= MAX_ERROR_BODY_BYTES.saturating_sub(out.len()) {
                    return Err(GatewayError::with_status(502, "提供商错误响应体达到 1 MiB 上限，已停止读取"));
                }
                if !bytes.is_empty() {
                    idle_deadline = tokio::time::Instant::now() + idle;
                }
                out.extend_from_slice(&bytes);
                tokio::select! {
                    biased;
                    _ = &mut cancelled => return Err(crate::server::core::upstream::cancellation::cancelled_error()),
                    _ = tokio::time::sleep_until(deadline) => return Err(body_timeout()),
                    _ = tokio::time::sleep_until(idle_deadline) => return Err(idle_timeout()),
                    result = credit => result.map_err(|_| GatewayError::with_status(
                        502, "提供商错误响应体额度归还任务中断",
                    ))?,
                }
            }
            Some(super::StreamEvent::Done(Ok(_))) => return Ok(out),
            Some(super::StreamEvent::Done(Err(error))) => {
                if strict::applies(provider) { *private_failure = strict::private_control(&error); }
                return Err(rpc_error(provider, error, proto::methods::gateway::CONVERSATION));
            }
            Some(super::StreamEvent::Failed(message)) => {
                return Err(if strict::applies(provider) { strict::fatal(provider, 502, "runtime_stream_interrupted") }
                    else { GatewayError::with_status(502, message) });
            }
            None => return Err(GatewayError::with_status(502, "提供商错误响应体在终态之前中断")),
            // head / empty chunk not none；total deadline not 。
            Some(super::StreamEvent::Head(_)) => {}
        }
    }
}

/// 「 up failure + 」error。
///
/// ── and 1 ────────────────────────────────────
/// `upstream::provider_loop` `OutboundFailure` ：
/// that （ can can ，e.g. 402 → 429）， use ， up up 。
fn conversation_error(status: u16, class: UpstreamErrorClass, body: &Value) -> GatewayError {
    match class {
        UpstreamErrorClass::QuotaLimited { status, message, upstream_code, .. }
        | UpstreamErrorClass::ContentBlocked { status, message, upstream_code }
        | UpstreamErrorClass::Fatal { status, message, upstream_code } => {
            let text = if message.is_empty() { describe_body(body) } else { message };
            GatewayError::with_status(status as i32, text).with_optional_code(upstream_code)
        }
        UpstreamErrorClass::TokenExpired { message } => {
            let text = if message.is_empty() { describe_body(body) } else { message };
            GatewayError::with_status(status as i32, text)
        }
    }
}

/// error no has can when （ and ` up return {status}: …` ）。
fn describe_body(body: &Value) -> String {
    match body {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// from will ；emptydefault。
///
/// up `account.uid` not 1 etc ， not can use 。
fn account_id_of(account: &Value) -> String {
    account
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// from will （ up ）。
fn proxy_of(account: &Value) -> Value {
    let Some(proxy) = account.get("proxy") else {
        return Value::Null;
    };
    if proxy.is_null() {
        return Value::Null;
    }
    match serde_json::from_value::<proto::WireProxy>(proxy.clone()) {
        Ok(wire) => serde_json::to_value(wire).unwrap_or(Value::Null),
        // ： by 「default」（ and `ResolvedProxy::from_json`
        // 1 —— available，）。
        Err(_) => Value::Null,
    }
}

/// client `BTreeMap<String, String>`。
///
/// ── Why need ──────────────────────────────────────────
/// （ and HTTP/2 1 ）。 by ， by to ——
/// 1 then down is 「 not to 」 no has 。
fn headers_json(headers: &HeaderMap) -> Value {
    let mut map = serde_json::Map::new();
    for (name, value) in headers {
        if let Ok(text) = value.to_str() {
            map.insert(name.as_str().to_ascii_lowercase(), Value::String(text.to_string()));
        }
    }
    Value::Object(map)
}
