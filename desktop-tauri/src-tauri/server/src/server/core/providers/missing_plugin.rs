//! **** when 。
//!
//! ── this fileWhy in ──────────────────────────────────────
//! 「client up 」 to ** 
//! **（`agent2api-providers`， not ）。 by is ： to 
//! in ， will this 。
//!
//! is ：
//! 1. `ProviderKind` / provider id / / （****）；
//! 2. and use config（****）；
//! 3. （spawn 、JSON RPC、）。
//!
//! and in 。** no when **，[`adapter_for`] for 
//! `ProviderKind` return：， not is in 1 will 
//! when 。
//!
//! ── Whyerror need ────────────────────────────────────
//! use to is 「 this need need 」， not is 「not implemented」——
//! before is ** can **（）， after will bug 。 up provider
//! ， is as 1 use can can when 、or for its in 1 no 。
//!
//! ── Whyreturn 503 not is 500 ──────────────────────────────
//! 503 is 「unavailable」， up is 「 this when no has available 」；500 will 
//! client and as is error。 up after 1 request can success，
//! this 「 can 」。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::Value;

use super::adapter::{
    ChatRequestPlan, ModelRefreshOutcome, ProviderAdapter, UpstreamErrorClass,
};
use super::ProviderKind;
use crate::server::core::account_store::AccountStore;
use crate::server::errors::GatewayError;

/// when use （`label` by use ）。
fn missing_plugin_message(kind: ProviderKind, action: &str) -> String {
    let provider = super::kind_id(kind);
    if let Some(reason) = crate::server::core::plugin_host::provider_compatibility_error(provider) {
        return reason;
    }
    let label = super::label_of(provider);
    format!(
        "{label}的{action}需要安装提供商插件：本版本不再内置反向代理实现，\
         请在「提供商插件」里安装对应插件后重试"
    )
}

/// ： 1 `ProviderKind` 1 （ by kind ，none）。
pub struct MissingPluginAdapter(pub ProviderKind);

impl ProviderAdapter for MissingPluginAdapter {
    fn kind(&self) -> ProviderKind {
        self.0
    }

    /// empty： no when 1 unavailable。
    ///
    /// ── Why not return「」───────────────────────────
    /// that will up ， already no has 。 1 when 
    /// empty：client will to 1 not 。
    fn list_models(&self) -> Vec<Value> {
        Vec::new()
    }

    /// none can ：allerror is 「 no 」。
    ///
    /// return `Fatal` not is `TokenExpired` / `QuotaLimited`： after will 
    /// or new ， is not in —— no use 。
    fn classify_error(&self, status: u16, error_body: &Value) -> UpstreamErrorClass {
        let detail = error_body
            .get("error")
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .or_else(|| error_body.get("message").and_then(Value::as_str))
            .unwrap_or("");
        let message = missing_plugin_message(self.0, "转发");
        UpstreamErrorClass::Fatal {
            status,
            message: if detail.is_empty() {
                message
            } else {
                format!("{message}（上游原文：{detail}）")
            },
            upstream_code: None,
        }
    }

    /// not up request（ no has ， not this has ）。
    fn build_chat_request(
        &self,
        _account: &Value,
        _body: &Value,
        _client_headers: &axum::http::HeaderMap,
    ) -> Result<ChatRequestPlan, GatewayError> {
        Err(missing_plugin_error(self.0, "转发"))
    }

    fn ensure_access_token<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<String, GatewayError>> + Send + 'a>,
    > {
        Box::pin(async move { Err(missing_plugin_error(self.0, "凭证读取")) })
    }

    /// directory new ：「 no 」， not is return「 no new 」。
    ///
    /// `unchanged()` will 「 new ，0 」—— use not to Reason。
    fn refresh_models<'a>(
        &'a self,
        _store: &'a AccountStore,
        _account_id: &'a str,
        _force: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ModelRefreshOutcome> + Send + 'a>> {
        Box::pin(async move { ModelRefreshOutcome::failed(missing_plugin_message(self.0, "模型目录刷新")) })
    }
}

/// use （ new 、、directory）。
///
/// that path not 「 up error」， need is 1 can Reason。
pub fn missing_plugin_error(kind: ProviderKind, action: &str) -> GatewayError {
    GatewayError::with_status(503, missing_plugin_message(kind, action))
}

/// kind （`&'static`， 1 kind 1 ， when create）。
///
/// ── Why need 1 not is `Box::leak` ─────────────────
/// `adapter_for` return `&'static`，none、none need 。
/// by kind 1 、 not —— `Box::leak` path 1 。
pub fn placeholder(kind: ProviderKind) -> &'static dyn ProviderAdapter {
    use std::sync::OnceLock;

    /// `ProviderKind` （ and 1 ）。
    const ALL: &[ProviderKind] = &[
        ProviderKind::WorkBuddy,
        ProviderKind::WorkBuddyIntl,
        ProviderKind::Raccoon,
        ProviderKind::CatPaw,
        ProviderKind::AutoClaw,
        ProviderKind::AutoClawIntl,
        ProviderKind::Qoder,
        ProviderKind::QoderIntl,
        ProviderKind::CodeArts,
        ProviderKind::ClineFree,
        ProviderKind::ClinePass,
        ProviderKind::Accio,
        ProviderKind::AccioCn,
        ProviderKind::Zcode,
        ProviderKind::ZcodeIntl,
        ProviderKind::Trae,
        ProviderKind::Loomy,
        ProviderKind::Kuku,
        ProviderKind::MonkeyCode,
        ProviderKind::MonkeyCodeIntl,
        ProviderKind::CommandCode,
        ProviderKind::Antigravity,
        ProviderKind::OfficeAce,
        ProviderKind::OrcaTerm,
        ProviderKind::Chatglm,
        ProviderKind::Modelscope,
        ProviderKind::ModelscopeIntl,
        ProviderKind::Atomcode,
        ProviderKind::LobsterAi,
    ];

    static SLOTS: OnceLock<Vec<MissingPluginAdapter>> = OnceLock::new();
    let slots = SLOTS.get_or_init(|| ALL.iter().copied().map(MissingPluginAdapter).collect());
    let index = ALL.iter().position(|item| *item == kind).unwrap_or(0);
    slots
        .get(index)
        .map(|slot| slot as &dyn ProviderAdapter)
        .unwrap_or_else(|| {
            // not can can （`position` and `get` use 1 ）；
            // to this `ALL` and not ， 1 not is panic。
            crate::server::logging::log(
                "[Providers]",
                "⚠️ 占位适配器索引越界（ProviderKind 列表与枚举不同步）",
            );
            match slots.first() {
                Some(first) => first as &dyn ProviderAdapter,
                // `ALL` empty is ， this is to 1 
                None => unreachable_placeholder(),
            }
        })
}

/// （`ALL` empty， not use ）。
///
/// 1 is as up `unwrap_or_else` can ；release is 
/// `panic=abort`， this not panic —— return 1 use need need `'static`，
/// use 1 `OnceLock` 1 。
fn unreachable_placeholder() -> &'static dyn ProviderAdapter {
    static FALLBACK: std::sync::OnceLock<MissingPluginAdapter> = std::sync::OnceLock::new();
    FALLBACK.get_or_init(|| MissingPluginAdapter(ProviderKind::WorkBuddy))
}
