//! （ from `mod.rs` ）。
//!
//! ── 1 ， ────────────────────────────────────
//! `mod.rs` `forward()` **none 1 **： → to 
//! → （ / ） axum。****：
//!
//! ```text
//! = has this provider（router::route_for_forward）
//! ： in by **** 1 
//! └ 1 （ this provider ； + 401 new after 1 ）
//! └ 429 → this for this ， to down 1 （ can can 1 ）
//! ```
//!
//! is 「 by provider by 、 in this 」；
//! in in 1 ， use 1 by ，provider
//! is （ use ）。 by provider 1 
//! ： not not this 。
//!
//! ── error（ §4.2 ）────────
//! 1. `QuotaLimited` → this for this + down 1 ；
//! 2. `TokenExpired` → `refresh_access_token` new ，** 1 **
//! 1 （ 1 ：failure by failure， not will empty）；
//! 3. `Fatal` → to client。
//!
//! by to ， by this —— is 「11128 need 、6004 is 」
//! this provider in ，file for 「 up is 1 」 1 none。
//!
//! ── （11128）Why in this ─────────────────────────
//! §4.3 need 「11128 in 」：**** in this 
//! （、、），「 need 、、」 by 
//! `retry_advice` to 。 this can ， not will errorfile。
//!
//! ── has provider （W5-T-d4， §4.2.1）─────────
//! has provider（ CatPaw； `mod.rs` `is_stateful`）
//! **「 1 」， not **：、
//! telemetry use ； not `build_chat_request` that ， as will 
//! 「 1 」 is round + turn + ， is `ForwardOutcome`
//! not is `reqwest::Response`。 is 1 `if adapter.is_stateful()`，
//! `attempt_stateful`（「 → → → error」；
//! nonepath / new not use ， to `classify_conversation_error`
//! its in ****， after by ）。
//!
//! ── （ + ） in 1 ─────────────────
//! `payload.rs`： in 1 will before to **** up 。file
//! in when [`send_body`]（ and after 、request before ）， 1 
//! in request 1 （ to 1 1 when use ）。
//!
//! ── （ 0）Why in file ──────────────────
//! up by ****， up is HTTP 400 —— that is client system 
//! （ `core::sanitize`）。：** in 
//! 1 **（ in down ， and 401 new 1 ）， and 
//! （`core::degrade`） after request in 。 not 、
//! not —— not is （ `ErrContentBlocked`）。
//!
//! ── （usage）────────────────────────────────────────
//! 1 `telemetry.note_attempt(...)`， and up provider id ——
//! by 「 this request 」， not is by clientrequest 。
//!
//! 1 need 1 `note_attempt_started(provider_id, account)`， and in when 
//! `finish_last_attempt(status, error)` —— that 1 for is ****
//! （request「」 need 「A → B → C」）， and `note_attempt` 
//! 「 after 1 as 」 not ， is 。 （success `break response`、
//! failure `Err(failure)`） 1 ， etc `attempts`。
//! ** this 1 path when need 1 and check that **： 1 will 
//! attempts 1 （ before will 1 「none、noneerror」 empty）。
//!
//! ── request 1 not （）───────────────────
//! file is 「 1 will 」 （、、401 new 、
//! 、 up 、）****（`console_line`） or **request
//! **（ `error` / `retries` / `notice`）—— 「 not 
//! request 」（、、、、
//! directory new 、 when 、 ）。 1 this ，
//! ** will not will use request 1 **。

use std::collections::HashMap;
use std::time::Duration;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::custom_providers;
use crate::server::core::protocol::strip_internal_fields;
use crate::server::core::providers::adapter::{
    ProviderAdapter, RetryAdvice, RetryDecision, UpstreamErrorClass,
};
use crate::server::core::providers::custom::forward as custom_forward;
use crate::server::core::providers::readiness;
use crate::server::core::providers::{kind_from_id, meta};
#[cfg(test)]
use crate::server::core::providers::adapter::adapter_for;
// `ProviderKind` `provider_loop/tests.rs` `super::*` use to ；
// already not need need （ `adapter_by_id`）。 this re-import
// path has 。
#[allow(unused_imports)]
use crate::server::core::providers::ProviderKind;
use crate::server::errors::{parse_quota_reset_at, GatewayError};
use crate::server::logging;

use super::payload::{send_body, ProviderContext, SendBody};
use super::request::{read_upstream_error, TransportRequest};
use super::usage::LogPhase;
use super::{
    account_display, account_label, cancellation, connections::ConnectionGuard, describe_proxy,
    format_reset_text, refusal, reset_hint, rotate, ForwardOutcome, InFlightGuard, RouteTarget,
    UpstreamService,
    MAX_ROUTE_ATTEMPTS,
};

#[cfg(test)]
mod tests;

/// up 1 request failure（ already + already to client error）。
struct OutboundFailure {
    /// to （）
    class: UpstreamErrorClass,
    /// to client error（ / / up ）。
    ///
    /// and up `message` —— before `markAccountLimited`
    /// is `error.message`（` up return 429: { up }`），
    /// accounts.json that not 。
    error: GatewayError,
    /// request； not new 、 or 。
    stop: bool,
    policy: crate::server::core::providers::policy::ErrorPolicy,
}

/// **** （ `send_with_retry` ）。
///
/// request use 1 ： from 1 ， after not 。
/// this is 「 1 N 」 in （ `config::RetrySettings`）。
///
/// and 「 can 」 is ** **： that `attempt_queue` 
/// `switches_left` ， not 。
///
/// `total` used for 「 n/N 」， not and —— `remaining`。
#[derive(Clone, Copy, Debug)]
struct RetryBudget {
    /// total（ use ）
    total: usize,
    /// 
    remaining: usize,
}

impl RetryBudget {
    fn new(total: usize) -> Self {
        Self { total, remaining: total }
    }

    /// already use （ need this ）
    fn used(&self) -> usize {
        self.total.saturating_sub(self.remaining)
    }
}

/// before 1 ： can return true， already return false。
///
/// when 1 —— this is 「Why this N 」 
/// 1 （request ， not Why in that ）。
///
/// path（ will failure、error） use ，「 」 in 
/// up 1 。**429 not this **： that is 「 this down 1 」 
/// ， and 「」 is （set that ）。
fn take_switch(switches_left: &mut usize, total: usize) -> bool {
    if *switches_left == 0 {
        logging::console_line(
            "[Upstream]",
            &format!("⚠️ 换账号次数已用尽（最多 {total} 个），不再顺延，返回本次错误"),
        );
        return false;
    }
    *switches_left -= 1;
    true
}

/// failure after down 1 ****： 3 and 「failure」 use 1 。
///
/// ， and 1 ： this 1 `tried_ids` → has no has 
/// no → 1 → 1 。return `true` = use 
/// `continue 'accounts`；return `false` = 「 no has down 1 」 or 「 use 」，
/// use errorreturnclient。
///
/// ── Why in 「 has no has down 1 」 after ────────────────
/// no when this failure is ， not this 1 。 ，「 after 1 
/// failure」 will ， 1 request after is this 。
///
/// ── Why this use use 1 ─────────────────────────
/// and must —— 「 1 request 」 this in 1 
/// 。 1 ， need when 。
/// （** not ** 429 and will / that ：429 that not 、 need 
/// ， will and and when not —— 。）
///
/// `stage` （「failure」/「 in not 」）， not and 。
#[allow(clippy::too_many_arguments)]
fn defer_to_next_account(
    service: &UpstreamService,
    provider_ids: &[&str],
    cooldown_keys: &rotate::CooldownKeys<'_>,
    tried_ids: &mut Vec<String>,
    target: &RouteTarget,
    session: &Value,
    provider_id: &str,
    model: &str,
    stage: &str,
    status_code: i32,
    switches_left: &mut usize,
    switch_total: usize,
    pinned: Option<&str>,
    request_routing: &super::route_context::RequestRouting,
) -> bool {
    if let Some(account_id) = target.attempt_key() {
        if !tried_ids.contains(&account_id) {
            tried_ids.push(account_id);
        }
    }
    let Some(next) =
        rotate::pick_next_account(service, provider_ids, cooldown_keys, tried_ids, pinned, request_routing)
    else {
        return false;
    };
    if !take_switch(switches_left, switch_total) {
        return false;
    }
    let next_home = {
        let next_provider = rotate::provider_of(&next);
        if next_provider == provider_id {
            String::new()
        } else {
            format!(
                "，切换提供商 → {}",
                kind_from_id(next_provider)
                    .map(|kind| meta(kind).label)
                    .unwrap_or(next_provider)
            )
        }
    };
    // in ：request that by 「（ + error）+ down 1 」
    // this —— this this no has 。
    logging::console_line(
        "[Upstream]",
        &format!(
            "⚠️ 账号 {} 对模型 {model} {stage}（HTTP {status_code}），\
             按队列顺延 → {}（优先级 {}{next_home}）",
            account_label(target.account.as_ref(), &target.account_id.clone().unwrap_or_default(), &session),
            account_display(&next),
            next.get("priority")
                .and_then(Value::as_i64)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ),
    );
    true
}

/// when HTTP ： no when ， by set 1 。
///
/// 408（request when ） and 5xx（ / error）—— is up or 
/// ， has 。** not 429**： that is ，「 + 」 
/// （module header ）， for 1 will etc 1 。
/// ** not [`readiness::LOCAL_PRECONDITION_STATUS`]（556）**： that is 
/// 「 this 1 from 」， and 1 is 1 failure ——
/// has 1 （ [`defer_to_next_account`]）。
const TRANSIENT_RETRY_STATUSES: &[u16] = &[408, 500, 502, 503, 504];

/// by **request** ： this 1 by 「 this body has 」，
/// 1 to 1 failure ⇒ not （ [`fallback_retry_advice`]）。
///
/// and [`TRANSIENT_RETRY_STATUSES`] ： that is 「 up or 1 down 」，
/// 1 has ；413 no has will 。
const PAYLOAD_SIZE_REJECTION_STATUSES: &[u16] = &[413];

/// by **request by / ** ： up in request when no to 
/// （ not in this 1 、 by not in ）， by 「 this body 」，
/// 1 1 body to 1 404 ⇒ not ，
/// is has （ can can has this ）。
///
/// （issue #255）：OfficeAce for directory has 、 no has 
/// `404 {"code":null,"message":"Invalid model."}`， old Fatal，
/// 3 × 5 seconds——client etc 15 seconds， is 1 404。
///
/// and [`PAYLOAD_SIZE_REJECTION_STATUSES`] not is 1 ：
/// 1 is 「」， 1 is 「」， and not ， will excluding can can 。
const DETERMINISTIC_NOT_FOUND_STATUSES: &[u16] = &[404];

/// （`core::proxy_rotate`） in 429 when **** limit。
///
/// Why is 3：OpenCode Zen 429 by IP （ this use ），
/// has will ； 1 request not this none down —— 1 
/// ，client etc 。 3 429， by 「 this up 」
/// （ has by + down 1 ）， and no has when as 1 。
const MAX_ROTATE_EXIT_RETRIES: usize = 3;

/// this failure is in 「error」（set「 use → request」）。
///
/// in failure****： not [`send_with_retry`] ， not 
/// （、401 new —— that is 「 1 
/// 1 」）， down 1 （ and 3 1 ，「
/// 」）。 or no has no when ，error to client。
/// in that before 。
///
/// failure（DNS / / ） no has up ， not 。
/// `status_code` is `i64` （GatewayError is `i32`， this 1 ），
/// / not can can is HTTP ， by 「 not in 」。
fn direct_switch_status(status: i64) -> bool {
    u16::try_from(status)
        .map(|code| config::retry_settings().no_retry(code))
        .unwrap_or(false)
}

/// when HTTP error 1 （set「request」 ）。
///
/// `remaining` is ****（ [`RetryBudget`] ）；
/// use return None， by use error。
fn transient_retry_advice(status: u16, remaining: usize) -> Option<RetryAdvice> {
    if remaining == 0 || !TRANSIENT_RETRY_STATUSES.contains(&status) {
        return None;
    }
    Some(RetryAdvice {
        delay_ms: config::retry_settings().delay_ms(),
        reason: format!("上游瞬时错误（HTTP {status}）"),
    })
}

/// failure（DNS / / ） 1 ： and when HTTP error 1 set。
///
/// Reasonerror （`UpstreamRequestError::reason`）： when and 
/// etc response when set and seconds， its 1 「 up failure」。
fn transport_retry_advice(error: &super::request::UpstreamRequestError, remaining: usize) -> Option<RetryAdvice> {
    if remaining == 0 {
        return None;
    }
    Some(RetryAdvice {
        delay_ms: config::retry_settings().delay_ms(),
        reason: error.reason.clone(),
    })
}

/// ：**** up error， in before in 1 up 。
///
/// ── Why need need this 1 ────────────────────────────────────────
/// before has use ： that 
/// （workbuddy 11128 ）， when 408/5xx。 not in when ，
/// request will to 「」—— set that 「 1 」** 1 no use up **。
/// use ： 3， 1 400 to 6 1 、
/// `retries` empty（2026-09）。
///
/// up this is this set ：** in this up ， not **。
/// in 「 after 」 not is 「 by 」， is as 400 this 
/// by up （ 1 400 can can is 、 can can is up not 1 ），
/// none from —— is 1 ， is 1 
/// and 1 can can 。。
///
/// ── excluding（ has ）─────────────────────────────
/// - `QuotaLimited`（429 / ）： that is ，「 + 」，
/// in will etc 1 （ and [`TRANSIENT_RETRY_STATUSES`] not 429
/// 1 by ）；
/// - `TokenExpired`（401）： has （ new after 1 ），
/// to this already 1 failure， no has new ；
/// - `ContentBlocked`（）： has （ in 1 ，
/// `attempt_queue` 0）， to this that already use ——
/// 1 body not （ is ， not is ）；
/// - `Fatal` + **413**（ [`PAYLOAD_SIZE_REJECTION_STATUSES`]）： up by request
/// （ Loomy `{"error_code":"APIG.0201","error_msg":
/// "Request entity too large"}`）， no has will 。 not for ：
/// 13 seconds + 1 5 seconds， down client 60 seconds etc to ，
/// down 「408 client in response before 」， in attempt_details。
fn fallback_retry_advice(
    class: &UpstreamErrorClass,
    remaining: usize,
    status: u16,
) -> Option<RetryAdvice> {
    if remaining == 0 {
        return None;
    }
    match class {
        UpstreamErrorClass::QuotaLimited { .. }
        | UpstreamErrorClass::TokenExpired { .. }
        | UpstreamErrorClass::ContentBlocked { .. } => None,
        // ： 1 will to 1 413（ 1 ， that is 
        // ， this 「 1 」 this 1 ）
        UpstreamErrorClass::Fatal { .. }
            if PAYLOAD_SIZE_REJECTION_STATUSES.contains(&status)
                || DETERMINISTIC_NOT_FOUND_STATUSES.contains(&status) =>
        {
            None
        }
        UpstreamErrorClass::Fatal { .. } => Some(RetryAdvice {
            delay_ms: config::retry_settings().delay_ms(),
            reason: format!("上游错误（HTTP {status}），换账号前先原地重发"),
        }),
    }
}

/// ：`⚠️ {Reason}；{n} seconds after （ {i}/{N} ）`。
///
/// Reason `RetryAdvice::reason`（provider ）， by this ——
/// 「 / 」 has （ [`RetryBudget`]， to 
/// 1 already ）。 before this by and ，
/// and in 1 ， is 1 「；」（ when error and failure that use 「，」）。
fn retry_log_line(reason: &str, delay_ms: u64, used: usize, total: usize) -> String {
    format!("⚠️ {reason}；{} 秒后重试（第 {used}/{total} 次）", delay_ms / 1000)
}

/// ： in by 。
///
/// `slot` is in （`&mut` is as in **success as ** when ，
/// failure when by has ； `InFlightGuard` ）。
/// `connections` is ： by in ，
/// success when to response。
pub(super) async fn forward_with_providers(
    service: &UpstreamService,
    ctx: ProviderContext<'_>,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
) -> Result<ForwardOutcome, GatewayError> {
    let model = model_of(ctx.body);
    if ctx.routing.group.is_some() {
        ctx.telemetry.set_route_decision(json!({ "scope": ctx.routing.scope, "configSnapshot": ctx.routing.json(),
            "strategy": ctx.routing.settings.strategy.as_str(), "sticky": ctx.routing.settings.sticky.as_str(),
            "ordered": [], "blocked": ctx.routing.group_projection.blocked.iter().take(12).collect::<Vec<_>>(),
            "orderedTotal": ctx.routing.group_projection.accounts.len(), "blockedTotal": ctx.routing.group_projection.blocked.len() }));
    }
    // `route_for_forward`（directory no has this when will default
    // provider 1 ； has use when ** not **， that ）——
    // is 1 ， in this 。
    //
    // ：`ignore_model_gate`（ ） not by by —— 
    // can can no enabled（、 not enabled is use ），「 → empty」
    // will for in 。 Key （
    // `provider_only`， is that 1 ）； by ，
    // default not 、， `ForwardRequest` 。
    let candidates = if ctx.routing.group.is_some() {
        if ctx.images.is_some() { return Err(GatewayError::bad_request("对话路由组不能用于图片生成")); }
        ctx.routing.providers(Vec::new())
    } else if ctx.ignore_model_gate {
        ctx.key_scope
            .map(crate::server::core::key_scope::KeyScope::allowed_provider_ids)
            .unwrap_or_default()
    } else if let Some(operation) = ctx.images {
        ctx.routing.providers(crate::server::core::providers::images::operation_providers(&model, operation))
    } else {
        let available = crate::server::core::providers::catalog::forwarding_providers(&model);
        // in after delete when ， not can to request default。
        let available = if !ctx.compact_routed && available.is_empty() && crate::server::core::providers::catalog::providers_for_model(&model).is_empty() {
            vec![crate::server::core::providers::DEFAULT_PROVIDER_ID.to_string()]
        } else { available };
        ctx.routing.providers(available)
    };
    let candidates: Vec<_> = if ctx.images.is_some() || ctx.routing.group.is_some() {
        candidates
    } else {
        candidates.into_iter().filter(|provider| {
            !crate::server::core::providers::images::provider_model_is_image(provider, &model)
        }).collect()
    };
    if !ctx.compact_routed && ctx.routing.group.is_none() && crate::server::core::providers::catalog::providers_for_model(&model).is_empty() {
        logging::verbose(
            "[Upstream]",
            &format!(
                "模型 {} 不在聚合目录中，按默认提供商转发",
                if model.is_empty() { "(未指定)" } else { &model },
            ),
        );
    }
    if candidates.is_empty() {
        if ctx.routing.group.is_some() {
            return Err(GatewayError::with_status(503, "此路由组没有参与成员，请在路由页加入模型；未回退全局或默认提供商").with_code("route_empty"));
        }
        // empty has （ `route_for_forward`）：default provider 
        // no has ，or this ** **（ that when not default）。 after is 
        // use can ， need ， not can 「 no has available 」。
        if crate::server::core::providers::catalog::model_blocked_everywhere(&model) {
            return Err(GatewayError::with_status(
                404,
                format!(
                    "模型已在网关中关闭: {model}。完整列表见 GET /v1/models"
                ),
            )
            .with_code("model_not_found"));
        }
        if let Some(profile) = &ctx.routing.profile {
            let hint = if !ctx.routing.scope.is_empty() {
                "请在路由页该模型的独立配置中添加或启用提供商"
            } else if profile.scope() == config::RoutingProviderScope::Selected {
                "请在路由页「全局配置 → 参与范围」中改为自动参与，或将提供商加入指定名单"
            } else {
                "请在路由页「全局配置」中恢复已停用的提供商"
            };
            return Err(GatewayError::with_status(
                503,
                format!("模型 {model} 没有可参与转发的提供商：当前路由的参与范围将候选全部排除。{hint}"),
            ));
        }
        if ctx.compact_routed {
            return Err(GatewayError::with_status(404, format!(
                "压缩目标模型 {model} 已不可用，未回退默认提供商或原模型"
            )).with_code("model_not_found"));
        }
        return Err(GatewayError::new("没有可用的提供商，无法转发"));
    }
    // ── Key available（R9， OmniProxy `filterRoutesForKey`）──
    // OmniProxy：** in and ** —— is 「
    // can this 」（ can ）， its in not that 。
    // and 「 excluding when error」 in `filter_by_key_scope` （ need can before return
    // error，return Result）。
    let candidates = filter_by_key_scope(candidates, ctx.key_scope, &model)?;
    // already is id empty（`route_for_forward` return， router module header）：
    // and id ， id to and 。
    let provider_ids: Vec<&str> = candidates.iter().map(String::as_str).collect();
    // ：， （ is 
    // ）。 use to request to when ， this and for up 。
    let with_mapping = crate::server::core::model_rules::shared()
        .mappings_of(&model)
        .len() > 0;
    logging::verbose(
        "[Upstream]",
        &format!(
            "候选提供商 {}（按账号全局优先级选路{}）",
            provider_ids.join(" / "),
            if with_mapping { "，含映射" } else { "" },
        ),
    );
    attempt_queue(service, &ctx, &provider_ids, slot, connections).await
}

/// by Key **available**（R9， OmniProxy 
/// `filterRoutesForKey` in 「 and 」 ）。
///
/// not this `route_for_forward`： that return 
///（「unknown」 and 「 use 」 ， module header）， is **request** 
/// ， will 「 by 」 and request 。
///
/// is id empty（ + ）：`KeyScope` 
/// is id （ `core::key_scope`）， id can —— id
/// in this not need need ，、 no ， and 1 。
///
/// ── excluding when to 1 ** can error**（ need need 1 ）──
/// OmniProxy「empty has 404」 （ that is 
/// `fail(res, ..., 404, 'model_not_found')`）， need Reason：
/// not is not in ， is this Key not that —— after will 
/// 「」 ， need is Key available。
///
/// and file its errorpath 1 ：** not failure、 not panic**
///（release is panic=abort， to that will use ）。
fn filter_by_key_scope(
    candidates: Vec<String>,
    scope: Option<&crate::server::core::key_scope::KeyScope>,
    model: &str,
) -> Result<Vec<String>, GatewayError> {
    let Some(scope) = scope.filter(|scope| scope.restricts_providers()) else {
        // no has scope / this unlimited —— （request this ）
        return Ok(candidates);
    };
    let kept: Vec<String> = candidates
        .iter()
        .filter(|id| scope.allows_provider(id))
        .cloned()
        .collect();
    if !kept.is_empty() {
        return Ok(kept);
    }
    // in ****： this will as GatewayError 1 ，
    // by `api::chat` / `api::protocol` request 「error」（ 1 Reason、
    // ）， not as 1 request 1 。
    logging::console_line(
        "[Security]",
        &format!(
            "🚫 请求被网关 Key 的可用提供商拒绝: 模型 {model}（{}）",
            scope.describe()
        ),
    );
    Err(GatewayError::with_status(
        404,
        format!(
            "模型 '{model}' 不可用：提供它的提供商都不在这把网关 Key 的可用提供商列表里（{}）",
            scope.describe()
        ),
    )
    .with_code("model_not_found"))
}

/// ： 1 from by 1 ， use 1 。
///
/// by `is_stateful` （ §4.2.1）：
/// - none（workbuddy / / AutoClaw）→ down this 「request → →
/// by 」；
/// - has （CatPaw）→ [`attempt_stateful`]（ 1 will ； and ）。
async fn attempt_queue(
    service: &UpstreamService,
    ctx: &ProviderContext<'_>,
    provider_ids: &[&str],
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
) -> Result<ForwardOutcome, GatewayError> {
    let model = model_of(ctx.body);
    let model_label = if model.is_empty() { "(默认)".to_string() } else { model.clone() };
    // ： request** up **（ `routing::CooldownKeys`）。
    // 1 、request use —— 、429 、success must is 1 ，
    // otherwise will in 1 up 、 in 1 up 。
    let cooldown_keys = rotate::CooldownKeys::new(&model);
    // will （`route_state::conversation_key`）：request ** will 
    // **（trace / `prompt_cache_key` / Claude Code `metadata.user_id`
    // etc ）， no has to 「 + use 」 。empty = no has 
    // 、 no has available will （none messages）， not 。 request
    // use this 1 —— 429 after is 1 will ， failure after 
    // （ not etc none）。
    let conv_key = super::route_state::RouteState::conversation_key(ctx.client_headers, &ctx.body);
    let sticky_generation = service.route.sticky_generation();
    let sticky_source = super::route_state::RouteState::conversation_source(ctx.client_headers, &ctx.body);
    if !conv_key.is_empty() {
        let mut decision = json!({
            "sessionId": super::route_sticky::entry_id(&conv_key, &model), "sessionSource": sticky_source,
            "strategy": ctx.routing.settings.strategy.as_str(), "sticky": ctx.routing.settings.sticky.as_str(),
            "scope": ctx.routing.scope, "configSnapshot": ctx.routing.json(),
        });
        if ctx.routing.settings.sticky.ttl_ms().is_some() { decision["stickyId"] = decision["sessionId"].clone(); }
        if let Some(group) = &ctx.routing.group {
            decision["blocked"] = json!(ctx.routing.group_projection.blocked.iter().take(12).collect::<Vec<_>>());
            decision["blockedTotal"] = ctx.routing.group_projection.blocked.len().into();
            decision["groupId"] = group.id.clone().into();
        }
        ctx.telemetry.set_route_decision(decision);
    }
    let mut conversation_ids = None;
    let mut tried_ids: Vec<String> = Vec::new();
    // ── （ config::RetrySettings）─────────────────────
    // - `budget`： 1 up can ****。request use 1 ，
    // by `send_with_retry` —— is in 「 1 
    // 」 up ， this is 「 1 N 」 this has 。
    // - `switches_left`： this request can ** **。 1 1 ， by down 
    // path（ will failure / error）。
    //
    // not ： use not ， not ——
    // 「 in current， in not 」 this 。
    //
    // by ****、 not ： to 1 down 1 ， and to 1 ，
    // 1 。 old by （ 1 down use 1 、 has ），
    // is 「 can 」 no —— 9 when 1 request will 1 
    // to 10 ， use that need etc 。
    let settings = config::retry_settings();
    let mut budget = RetryBudget::new(settings.resend_budget());
    let switch_total = settings.switch_budget();
    let mut switches_left = switch_total;
    // ── （ for `degradedApplied`）────────
    // `true` = ** already **：request when already （ in request
    // ）， or request after by down 「 0」。 after not 
    // —— 1 request 1 。
    //
    // this 「 no has 」，** not ** and ： in can ** by **
    // not （`config::KEY_PROMPT_PROVIDERS`）， no to 1 。
    // 「 this 1 need not need in 」 is `PromptChoice::text_for`（ that 
    // not can when return ）， down 「 0」 by **
    // **。
    let mut degraded = crate::server::core::degrade::active();
    // ： 1 will before by use 1 ， to ** 1 ** 
    // 1 when use （ not 、 not statistics）。 （Cline 
    // has `free`/`pass`）： in 
    // （`wire_target_for_provider`）， not ， 1 。
    // use ， use body。
    //
    // is ****： after request will in （「 0」），
    // that 1 is 1 ， not 1 —— not 
    // ， not will 「 before that 」 to after 。
    //
    // when ** this to up **（`SendBody::wire_model`）——
    // is ， and （ `payload::SendBody`）。
    // not 1 ：429 and success from this ， not 1 。
    let mut send_cache: HashMap<(&'static str, String, String, bool), SendBody<'_>> = HashMap::new();

    // is required ： down 「429 to down 1 」 in ****，
    // `continue` will to （ use 1 1 ， is need ）。
    // `continue 'accounts` 「 down 1 」。
    'accounts: for _ in 0..=MAX_ROUTE_ATTEMPTS {
        // ── ： 1 before ────────────────────────
        // 「 to 」 all when ：、、、 up 
        // response after down 1 。 etc （ etc response、） by `send_with_retry`
        // select ， this its （ will / ）。
        // no has in need need （ up 1 already ），return。
        if ctx.telemetry.is_cancelled() {
            return Err(cancellation::cancelled_error());
        }
        let mut target = rotate::select_target_account(
            service,
            provider_ids,
            &cooldown_keys,
            &tried_ids,
            ctx.pinned_account,
            &conv_key,
            &ctx.routing,
        )
        .await?;
        if let Some(override_proxy) = ctx.proxy_override {
            target.proxy = override_proxy.clone();
            target.proxy_notice = None;
        } else if crate::server::core::providers::is_scoped_runtime_provider(&target.provider)
            && target.proxy_notice.is_some()
        {
            return Err(GatewayError::with_status(400,
                "指定账号的代理不可用；未回退直连、系统代理或其他账号")
                .with_code("account_proxy_unavailable"));
        }
        // telemetry（ after 1 as ：request 「 by 」
        // is that —— and attempts ，
        // that 「」， this 「 when Why this 」）。
        if let Some(mut decision) = target.decision.clone() {
            decision["sessionSource"] = Value::String(sticky_source.to_string());
            ctx.telemetry.set_route_decision(decision);
        }
        // to this 1 in ：failure when ，
        // is 「 1 requestany when 1 」 this not need need 1 
        // （429 、401 new after 、 will failurepath this ）。
        connections.rebind(target.account_id.clone());
        let attempt_key = target.attempt_key().unwrap_or_default();
        let selected_model = target.account.as_ref().and_then(crate::server::core::routing_groups::candidate_model)
            .map(str::to_string).unwrap_or_else(|| model.clone());
        // ── （； and down is_stateful ）──
        // in `kind_from_id` before ：custom id ** not ** `ProviderKind`
        // （「 not is not 」 ， `custom_providers` module header），
        // has 「 1 」（`providers::custom::forward`）。 is use 
        // —— in （`RouteTarget.provider` for is 
        // custom id，`provider_of` ），。
        if custom_providers::is_custom_provider_id(&target.provider) {
            let custom_provider_id = target.provider.clone();
            let custom_account_id = target.account_id.clone();
            match attempt_custom(service, ctx, target, slot, connections, degraded).await {
                Ok(outcome) => {
                    if let Some(account_id) = custom_account_id.as_deref() {
                        service.route.sticky_bind(
                            &conv_key, &model, &attempt_key, sticky_source, logging::now_ms(),
                            ctx.routing.settings.sticky.ttl_ms().unwrap_or(0), sticky_generation,
                        );
                        service.route.backoff_clear(account_id);
                        service.route.sink_clear(account_id);
                    }
                    return Ok(outcome);
                },
                Err(error) => {
                    // （ and nonepath 1 and by ）： not already 
                    // 「 this 1 failure」 down 1 
                    if ctx.telemetry.is_cancelled() {
                        ctx.telemetry.finish_last_attempt(
                            Some(cancellation::MANUAL_TERMINATED_STATUS),
                            Some(cancellation::MANUAL_TERMINATED),
                        );
                        return Err(cancellation::cancelled_error());
                    }
                    // 「error」 in this no has ： in when 
                    // `attempt_custom` already （
                    // `direct_switch_status`）， to down is and its error 1 
                    // —— down 1 ，failurereturn。
                    // failure tried， to —— and will 
                    // failure（is_stateful ） 1 。
                    if let Some(account_id) = custom_account_id {
                        let first_failure = !tried_ids.contains(&attempt_key);
                        if first_failure {
                            tried_ids.push(attempt_key.clone());
                            // 429 → （ and nonepath 1 1 
                            // ： is ** up **， when by attempt_custom
                            // and message、`mark_account_limited` ）。
                            // no has 「」 ，
                            // and （`custom::forward::cooldown_model`）。
                            if error.is_quota_limit() {
                                let wire_model = if ctx.routing.group.is_some() { selected_model.clone() } else {
                                    custom_forward::cooldown_model(&custom_provider_id, &model)
                                };
                                rotate::mark_account_limited(
                                    service,
                                    &account_id,
                                    &wire_model,
                                    error.status_code,
                                    error.upstream_code,
                                    None,
                                    &error.message,
                                );
                                // and 1 ： has when 429 
                                // （ that ）
                                if ctx.routing.settings.sink && ctx.pinned_account.is_none() {
                                    service.route.sink_mark(&account_id, logging::now_ms());
                                }
                            } else {
                                // and 3 ：failure + 
                                let route_settings = ctx.routing.settings;
                                if route_settings.failover_backoff {
                                    service.route.backoff_mark(&account_id, logging::now_ms());
                                }
                                if route_settings.breaker_enabled {
                                    service.route.breaker_failure(
                                        &account_id,
                                        route_settings.breaker_threshold,
                                        route_settings.breaker_timeout_seconds,
                                        logging::now_ms(),
                                    );
                                }
                            }
                        }
                    }
                    match rotate::pick_next_account(
                        service,
                        provider_ids,
                        &cooldown_keys,
                        &tried_ids,
                        ctx.pinned_account,
                        &ctx.routing,
                    ) {
                        Some(next) => {
                            // use → has not 
                            // （`take_switch` already that ），error
                            // return： and 「 no has down 1 available」 1 。
                            if !take_switch(&mut switches_left, switch_total) {
                                return Err(error);
                            }
                            logging::console_line(
                                "[Upstream]",
                                &format!(
                                    "⚠️ 自定义转发失败，按队列顺延 → {}（优先级 {}）",
                                    account_display(&next),
                                    next.get("priority").and_then(Value::as_i64)
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "-".to_string()),
                                ),
                            );
                            continue 'accounts;
                        }
                        None => return Err(error),
                    }
                }
            }
        }
        // and ； etc new ， not up 。
        let ready = crate::server::core::providers::adapter::adapter_by_id_ready(&target.provider);
        let adapter_handle = if let Some(token) = ctx.telemetry.cancel_token() {
            tokio::select! {
                biased;
                _ = token.cancelled() => return Err(cancellation::cancelled_error()),
                result = ready => result?,
            }
        } else { ready.await? }.ok_or_else(|| {
            GatewayError::with_status(
                503,
                format!("账号所属提供商「{}」未知，无法转发", target.provider),
            )
        })?;
        let adapter = adapter_handle.as_ref();
        let provider_id = adapter.provider_id();
        // no has 「 by 」 this 1 ： in 、request use 
        // （ by up ）。`provider_id` need —— down 、
        // and use 。
        // will 「 1 for request is 1 will 」 ；request
        // （`build_image_request_async`）， not can will 。
        // （ will for + for ）， this 
        // when will for `conversation`， by for 
        // 「messages must」—— and none， 1 。
        if adapter.is_stateful() && ctx.images.is_none() {
            // will （CatPaw） no has ，** **：
            // failure already 、 to down 1 —— can can already 1 。
            // no has down 1 available when ， this error（ Reason）。
            let stateful_account_id = target.account_id.clone();
            let strict_runtime = crate::server::core::providers::is_scoped_runtime_provider(provider_id);
            match attempt_stateful(
                service,
                ctx,
                provider_id,
                adapter,
                target,
                slot,
                connections,
                degraded,
            )
            .await
            {
                Ok(outcome) => {
                    if let Some(account_id) = stateful_account_id.as_deref() {
                        service.route.sticky_bind(
                            &conv_key, &model, &attempt_key, sticky_source, logging::now_ms(),
                            ctx.routing.settings.sticky.ttl_ms().unwrap_or(0), sticky_generation,
                        );
                        service.route.backoff_clear(account_id);
                        service.route.sink_clear(account_id);
                    }
                    return Ok(crate::server::core::plugin_host::hold_adapter(outcome, adapter_handle));
                },
                Err(error) => {
                    // （ and none / path 1 and by ）：
                    // will this 1 already ， use need is —— not 
                    // failure down 1 （ that will 1 ）
                    if ctx.telemetry.is_cancelled() {
                        ctx.telemetry.finish_last_attempt(
                            Some(cancellation::MANUAL_TERMINATED_STATUS),
                            Some(cancellation::MANUAL_TERMINATED),
                        );
                        return Err(cancellation::cancelled_error());
                    }
                    if stateful_account_id.is_some() {
                        if !tried_ids.contains(&attempt_key) {
                            tried_ids.push(attempt_key.clone());
                        }
                    }
                    // ── will path （ and none「 1」 1 ）──
                    // by to （default Fatal， has not ）； to 
                    // QuotaLimited use **** —— after down 
                    // ， not use none / new 。 from 
                    // `cooldown_keys` （ and ）； will in 
                    // before ， not to `SendBody::wire_model`。
                    //
                    // `class` to `mark_conversation_limited` not is in this 
                    // —— after 「 1 」 and not 
                    // （ before `reset_at` `None`， that ）。
                    let class = adapter.classify_conversation_error(&error);
                    // Scoped runtime loader already has token ； not 
                    // refresh， not Fatal//WAF// when 。
                    if strict_runtime && !matches!(&class, UpstreamErrorClass::QuotaLimited { .. }) {
                        return Err(error);
                    }
                    if let (UpstreamErrorClass::QuotaLimited { .. }, Some(account_id)) =
                        (&class, &stateful_account_id)
                    {
                        let wire = if ctx.routing.group.is_some() { selected_model.clone() } else { cooldown_keys.for_provider(provider_id) };
                        let group = adapter.quota_cooldown_models(account_id, &wire);
                        mark_conversation_limited(service, account_id, &group, &class, &error.message);
                        let label = if group.len() > 1 {
                            format!("{} 等 {} 个模型（福利池按账号记账）",
                                    group.first().cloned().unwrap_or_else(|| wire.clone()),
                                    group.len())
                        } else {
                            wire.clone()
                        };
                        logging::console_line(
                            "[Upstream]",
                            &format!(
                                "⚠️ 会话式账号 {} 对模型 {} 已限额，标记冷却后按队列顺延",
                                account_id, label,
                            ),
                        );
                    }
                    match rotate::pick_next_account(
                        service,
                        provider_ids,
                        &cooldown_keys,
                        &tried_ids,
                        ctx.pinned_account,
                        &ctx.routing,
                    ) {
                        Some(next) => {
                            // use → has not 
                            // （`take_switch` already that ），error
                            // return： and 「 no has down 1 available」 1 。
                            if !take_switch(&mut switches_left, switch_total) {
                                return Err(error);
                            }
                            // this by request （ already 
                            // as failure、 down 1 will new ）， this 1 
                            logging::console_line(
                                "[Upstream]",
                                &format!(
                                    "⚠️ 会话式转发失败，按队列顺延 → {}（优先级 {}）",
                                    account_display(&next),
                                    next.get("priority").and_then(Value::as_i64)
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "-".to_string()),
                                ),
                            );
                            continue 'accounts;
                        }
                        None => return Err(error),
                    }
                }
            }
        }
        // no has （ to default） when ， has 
        // provider can —— otherwise down 1 session_for will 「 this provider 
        // current」， not to is 401。 before can Reason。
        if target.account_id.is_none() && !adapter.allows_anonymous_default_session() {
            return Err(GatewayError::with_status(
                503,
                "没有可用账号，无账号可转发：请在账号页添加并启用账号",
            ));
        }
        let mut session = match rotate::session_for(service, provider_id, target.account_id.as_deref()).await {
            Ok(session) => session,
            Err(error) => {
                // ── default （Agent2API W3-T4）──────────────────
                // `session_for(None)` 「auth default」：workbuddy is 
                // `WORKBUDDY_TOKEN` + file current。 
                // （`RACCOON_TOKEN`） not in that 1 ， **file can can 
                // 1 no has **（/CI use use ）—— that down 
                // up use 401。
                //
                // for **default will ** provider 
                // （`allows_anonymous_default_session`）， and only in no has when ：
                // to token 1 will （ has Authorization
                // need need ）， not workbuddy that already can to will path ——
                // has as not 。
                if target.account_id.is_none() && adapter.allows_anonymous_default_session() {
                    match adapter.ensure_access_token(&service.store, "").await {
                        Ok(token) if !token.is_empty() => json!({
                            "auth": {
                                "accessToken": token,
                                "tokenType": "Bearer",
                            },
                        }),
                        // not to ：return**error**（ 401「」
                        // Reason，e.g. as empty、auth.json ）
                        _ => return Err(error),
                    }
                } else {
                    return Err(error);
                }
            }
        };
        // ── available（ §4.2 ensure_access_token）──────────
        // in in this 1 「 new 」： before has default
        // get_current_session when new ， up 1 will token
        // will to up 401。 new by store， after will 。
        //
        // **failure not **：ensure （e.g. this new 1 in ，
        // auth to 409） when use store has token —— token
        // by 401 → refresh_access_token that path， this before 
        // will 1 can success requestfailure。
        if let Some(account_id) = target.account_id.clone() {
            match adapter.ensure_access_token(&service.store, &account_id).await {
                Ok(_) => {
                    // new can can already ： will ， token is new 
                    if let Ok(fresh) =
                        rotate::session_for(service, provider_id, Some(&account_id)).await
                    {
                        session = fresh;
                    }
                }
                Err(error) => logging::verbose(
                    "[Upstream]",
                    &format!(
                        "账号 {account_id} 的凭证准备失败（沿用现有 token）: {}",
                        error.message
                    ),
                ),
            }
        }
        // ── ： provider + is this 1 ──────────
        // 1 、 use （`note_attempt` account_name and 
        // `note_attempt_started` account）：「 this 1 in 」 and 
        // not can can for not up 。
        let attempt_account = account_label(
            target.account.as_ref(),
            target.account_id.as_deref().unwrap_or(""),
            &session,
        );
        ctx.telemetry.note_quota_channel(target.account.as_ref());
        ctx.telemetry.note_attempt(
            target.account_id.as_deref(),
            &attempt_account,
            provider_id,
        );
        // 「」： ， after by down up 
        // （success / failure）。 and note_attempt must for and in after ——
        // etc attempts， before 「 N 」 and for up 。
        // in this not is send_with_retry ： that （）
        // ** not 1 new **（ TelemetrySnapshot::attempts ），
        // if in will 「」 。
        ctx.telemetry.note_attempt_started(provider_id, &attempt_account);
        // ： when down 「unavailable、」 this 1 
        // （ before is 1 ， `with_proxy_notice`）
        if let Some(notice) = target.proxy_notice.as_deref() {
            ctx.telemetry.note_attempt_notice(notice);
        }

        // ── ： already 、 this 1 ** will **， ────
        // in / after ： no has available（ up 503/401 before return） request
        // not to this ， not will 1 「 already 」statistics。 ，
        // （ use ， 1 ）。
        //
        // ** in down that **（ not is in this 1 ）：
        // after is 「 in 1 」， that 1 need `degraded` after new 
        // （「 0」）。 in this 1 ， use 。
        let account_pool = target
            .account
            .as_ref()
            .and_then(|account| account.get("pool"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        // ── 1 ：（ + 1 + 401 new 1 ）──
        // Why new and 1 ：** can can is ** 429
        // （ use ）。 if return， will 1 error
        // to client —— is 「 + to down 1 」。
        // and 1 after ， error 1 。
        //
        // 1 「 → in 1 」（ 0）： is 
        // 「 not 、 1 」， is body not is 。 `continue`
        // has 1 1 （`degraded` / `refreshed`）， has up 
        // —— not in 「 1 」 path。
        let started_at = logging::now_ms();
        let mut refreshed = false;
        // is up response （ in request when to ）：chat 
        // need in down 1 response， `UpstreamResponse` 。 is 
        // Antigravity request will （ ，issue #291；
        // its is empty —— 1 that success break）
        let mut native_tools;
        let (response, wire_model, response_protocol, antigravity_session) = loop {
            // in this 1 before 1 （ 1 down use ）：
            // use in valid，`continue`（401 new / ） when 
            // new —— is 「 body that 」 to 1 is new 1 。
            //
            // ── Why not use `or_insert_with`（）───────────
            // `send_body` in is async（ need 「 this 」），
            // `or_insert_with` not can await。「、miss when await
            // 、」—— etc ， is miss that 1 。
            let cache_key = (provider_id, account_pool.clone(), selected_model.clone(), degraded);
            if !send_cache.contains_key(&cache_key) {
                let built = send_body(ctx, provider_id, target.account.as_ref(), degraded).await;
                send_cache.insert(cache_key.clone(), built);
            }
            let send = match send_cache.get(&cache_key) {
                Some(send) => send,
                None => {
                    // up ； to this and empty（ not this ）。
                    // panic （release is panic=abort）。
                    return Err(GatewayError::with_status(
                        500,
                        "发送体缓存异常：插入后立刻读不到",
                    ));
                }
            };
            // is 「 chat to up 」 ：（Anthropic /
            // Responses） （_wb_*， protocol::mod ） not can 
            // to this —— up will up unknown 400。 no has 
            // when （request ）。
            let stripped = strip_internal_fields(&send.body);
            let body: &serde_json::Value = &stripped;
            // this 1 to up = （ and ， `SendBody`）。
            // 1 （，），success when return to 
            // （`cap_cleared` need ）—— is break 。
            let wire_model = send.wire_model.clone();
            ctx.telemetry.note_upstream_model(&wire_model);
            ctx.telemetry.note_upstream_reasoning(send.wire_reasoning.clone());
            // request** can can failure**（ ，e.g.
            // accessToken → 401）。 this not is use `?` ：
            // up `note_attempt_started` already as this 1 ，return will 
            // that in 「none、noneerror」 empty（ before 
            // will 「none」）—— has 1 failureReason。
            // to ， error： and attempts 1 1 for 
            // in （ that is before ，module header）。
            // ：（）， RPC。
            // `ProviderAdapter::build_chat_request_async` 。
            if ctx.images.is_none() {
                if let Some(override_proxy) = ctx.proxy_override {
                    session["proxy"] = override_proxy.as_ref().map(|proxy| json!({
                        "protocol": proxy.protocol, "host": proxy.host, "port": proxy.port,
                        "username": proxy.username, "password": proxy.password,
                    })).unwrap_or_else(|| json!({ "source": "direct" }));
                    session["proxyError"] = Value::Null;
                }
            }
            let planned = if let Some(operation) = ctx.images {
                adapter.build_image_request_async(&session, body, ctx.client_headers, operation).await
            } else {
                adapter.build_chat_request_async(&session, body, ctx.client_headers).await
            };
            let mut plan = match planned {
                Ok(plan) => plan,
                Err(error) => {
                    ctx.telemetry.finish_last_attempt(
                        Some(i64::from(error.status_code)),
                        Some(&error.message),
                    );
                    // ── ****failure： this 1 no not ──────
                    // is not is （556， `readiness` module header）：
                    // in **request when ** before not （current 1 use is ZCode
                    // ）， 1 up no 。
                    //
                    // before this 1 `return Err`， is this failure will **request**
                    // —— up is `attempts=1`、 has this 1 、503， 1 
                    // in can 。 not this 「 after 
                    // up 」：unavailable， down 1 has will 。
                    //
                    // its error（400 / failure、401 、
                    // 500 ）****： that is 「 this request」，
                    // is 1 request， will 1 400 
                    // 400， and Reason。
                    let account_unavailable = error.code.as_deref().is_some_and(|code| {
                        matches!(code, "account_model_unavailable" | "account_model_permissions_unknown")
                    });
                    if i32::from(readiness::LOCAL_PRECONDITION_STATUS) != error.status_code && !account_unavailable {
                        return Err(error);
                    }
                    if defer_to_next_account(
                        service,
                        provider_ids,
                        &cooldown_keys,
                        &mut tried_ids,
                        &target,
                        &session,
                        provider_id,
                        &model,
                        "在本机就发不出去",
                        error.status_code,
                        &mut switches_left,
                        switch_total,
                        ctx.pinned_account,
                        &ctx.routing,
                    ) {
                        continue 'accounts;
                    }
                    return Err(error);
                }
            };
            if plan.policy.stable_conversation {
                if conversation_ids.is_none() {
                    conversation_ids = Some(match super::conversation_ids::resolve(ctx.client_headers, ctx.body) {
                        Ok(ids) => ids,
                        Err(error) => {
                            ctx.telemetry.finish_last_attempt(Some(i64::from(error.status_code)), Some(&error.message));
                            return Err(error);
                        }
                    });
                }
                if let Some(ids) = &conversation_ids { ids.apply(&mut plan.headers); }
            }
            native_tools = if plan.policy.repair_native_tools {
                super::native_tools::declared_tools(&plan.body)
            } else { Vec::new() };
            if ctx.images.is_none() {
                let reasoning = plan.reasoning.clone().unwrap_or_else(|| {
                    super::payload::outbound_reasoning_of(provider_id, &plan.body)
                        .or_else(|| send.wire_reasoning.clone())
                });
                ctx.telemetry.note_upstream_reasoning(reasoning);
            }
            // failure can can is （ to body not can 
            // ）， by 500 。 need to （ by up 1 ）。
            let payload = match serde_json::to_string(&plan.body) {
                Ok(payload) => payload,
                Err(error) => {
                    let gateway = GatewayError::new(format!("请求体序列化失败: {error}"));
                    ctx.telemetry.finish_last_attempt(
                        Some(i64::from(gateway.status_code)),
                        Some(&gateway.message),
                    );
                    return Err(gateway);
                }
            };
            logging::verbose(
                "[Upstream]",
                &format!(
                    "POST {} model={} stream={} uid={} priority={} 出口={} msgs={} provider={provider_id}",
                    plan.url,
                    model_label,
                    ctx.stream,
                    session
                        .get("account")
                        .and_then(|account| account.get("uid"))
                        .and_then(Value::as_str)
                        .unwrap_or("-"),
                    target
                        .priority
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                    describe_proxy(target.proxy.as_ref()),
                    ctx.body
                        .get("messages")
                        .and_then(Value::as_array)
                        .map(|items| items.len().to_string())
                        .unwrap_or_else(|| "?".to_string()),
                ),
            );
            let transport = TransportRequest {
                url: plan.url,
                headers: plan.headers,
                payload,
                proxy: target.proxy.clone(),
                // can by to ； already or / when not and 
                system_proxy_when_unset: ctx.proxy_override.is_none()
                    && adapter.system_proxy_when_unset(),
            };
            // ── ： 1 will ──────────────────
            // in `build_chat_request` after （URL / / body already ）。
            // before new 1 （ and 401 new will to this ），
            // is down is **** that 1 。 when `capture`
            // as None， this 1 not （）。
            let capture = ctx.telemetry.capture();
            if let Some(capture) = capture.as_deref() {
                capture.reset_request(&transport.url, provider_id, &transport.headers, &plan.body);
            }
            match send_with_retry(
                adapter,
                &transport,
                &mut budget,
                capture.as_deref(),
                ctx.telemetry,
                degraded,
                match plan.response {
                    crate::server::core::providers::adapter::UpstreamResponse::Images(kind) => Some(kind),
                    _ => None,
                },
                plan.policy.network.as_ref(),
            )
            .await
            {
                Ok(response) => {
                    // success： this 1 is 「success + 」， in this 。
                    // failure in down （`Err(failure)` that 1 ）—— 
                    // 1 ， 1 for 1 。
                    //
                    // ** up return that **（ not is down to client ）： need 
                    // 「 up 」，client 200 in response
                    // ， after can can failure —— that 1 （） 
                    // ， `RequestEntry::is_success` 。
                    ctx.telemetry
                        .finish_last_attempt(Some(i64::from(response.status().as_u16())), None);
                    // ── by success（route_state.rs）───────────────
                    // / / / 。 in 
                    // 「config and has will 」 when —— Turn / Off ttl as None。
                    // none： will to new （ before 1 failure）
                    // when ， must is in this can 。
                    let route_settings = ctx.routing.settings;
                    if let Some(account_id) = target.account_id.as_deref() {
                        if route_settings.breaker_enabled {
                            service
                                .route
                                .breaker_success(account_id, route_settings.breaker_success_threshold);
                        }
                        service.route.backoff_clear(account_id);
                        service.route.sink_clear(account_id);
                        // this success = **available**： can can 
                        // in 「 need need new 」（ use can can already in new 
                        // ，or before that 401 is up ）。
                        //
                        // Why in success、 not is etc new ：none
                        // （Loomy） no has 「 new success」 this available， 1 success 
                        // is 。 not after is in 
                        // 「 need need new 」 up ， use already new 、 can use ，
                        // 1 。
                        // CodeArts when can can valid，success not new 。
                        if adapter.provider_id() != "codearts" && target.account.as_ref()
                            .is_some_and(|account| account.get("needsRelogin").and_then(Value::as_bool) == Some(true)) {
                            crate::server::core::blocking::in_place(|| service.store.set_needs_relogin(account_id, None));
                            logging::log(
                                "[Upstream]",
                                &format!("账号 {account_id} 转发成功，已清除「需要重新登录」标记"),
                            );
                        }
                    }
                    if let (Some(_ttl), true, Some(_)) = (
                        route_settings.sticky.ttl_ms(),
                        !conv_key.is_empty(),
                        target.account_id.as_deref(),
                    ) {
                        service.route.sticky_bind(
                            &conv_key, &model, &attempt_key, sticky_source,
                            logging::now_ms(), route_settings.sticky.ttl_ms().unwrap_or(0),
                            sticky_generation,
                        );
                    }
                    // Antigravity will （ ，
                    // issue #291）： has has `request.sessionId`， its 
                    // to empty。`plan.body` in this plan all， can 。
                    let antigravity_session = if plan.response
                        == crate::server::core::providers::adapter::UpstreamResponse::AntigravityGemini
                    {
                        plan.body
                            .pointer("/request/sessionId")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string()
                    } else {
                        String::new()
                    };
                    break (response, wire_model, plan.response, antigravity_session);
                }
                Err(failure) => {
                    // ── 1 ────────────────────────
                    // use already 「request」： / / new no has 
                    // 。 in before —— otherwise 1 「 when 」
                    // up error will by Fatal ， already to down 1 
                    // （ down 1 will down ， 1 and ）。
                    // in this （ and down that ： return）。
                    if ctx.telemetry.is_cancelled() {
                        ctx.telemetry.finish_last_attempt(
                            Some(cancellation::MANUAL_TERMINATED_STATUS),
                            Some(cancellation::MANUAL_TERMINATED),
                        );
                        return Err(cancellation::cancelled_error());
                    }
                    if let Some(account_id) = target.account_id.as_deref() {
                        if failure.policy.suspend_account {
                            let _ = crate::server::core::blocking::in_place(|| {
                                service.store.suspend_upstream_account(account_id, &failure.policy.reason)
                            });
                        } else if let Some(until) = failure.policy.account_cooldown_until {
                            let _ = crate::server::core::blocking::in_place(|| {
                                service.store.cooldown_upstream_account(account_id, until, &failure.policy.reason)
                            });
                        }
                    }
                    if failure.stop || failure.policy.stop {
                        ctx.telemetry.finish_last_attempt(
                            Some(i64::from(failure.error.status_code)),
                            Some(&failure.error.message),
                        );
                        return Err(failure.error);
                    }
                    // ── error ──────────────────────────────
                    // use （default 402） not 「 1 1 」：
                    // already in `send_with_retry` ， this 
                    // （ 0 、 2 new 
                    // ） 1 and —— no has 。 down 
                    // 「 this 1 already 」 that ， after and its error
                    // 1 by 1 / 3 ： down 1 ，
                    // or no has when errorreturnclient。
                    let named_switch = failure.policy.suspend_account
                        || failure.policy.account_cooldown_until.is_some()
                        || direct_switch_status(i64::from(failure.error.status_code));
                    // ── 0： → in ， 1 ──
                    // up by ， in ； this is **** not is 
                    // （、、session ）， not 
                    // not —— 1 in 1 has 
                    // （ `ErrContentBlocked` ）。
                    //
                    // in 「 no + **** can 」 when ：`custom`
                    // system already by ， is use 
                    // ， not （ that to down by error）。
                    // by （`ctx.prompt.for_provider`）—— this 1 is 
                    // this 1 ， this use this 1 config。
                    //
                    // and 401 new 1 （ 1 、 1 ，
                    // is body not is ）：** not ** in this to ，
                    // success when this 1 is success。`degraded` after not 
                    // ， 1 request 1 。
                    let attempt_prompt = ctx.prompt.for_provider(provider_id);
                    if ctx.images.is_some() && matches!(failure.class, UpstreamErrorClass::ContentBlocked { .. }) {
                        ctx.telemetry.finish_last_attempt(
                            Some(i64::from(failure.error.status_code)), Some(&failure.error.message),
                        );
                        return Err(failure.error);
                    }
                    if matches!(failure.class, UpstreamErrorClass::ContentBlocked { .. })
                        && !ctx.preserve_system_prompt
                        && !named_switch
                        && !degraded
                        && attempt_prompt.mode.degradable()
                    {
                        degraded = true;
                        // （ already in then not ，return when ）
                        crate::server::core::degrade::trigger();
                        let until_text = crate::server::core::degrade::until_text();
                        let reason = format!(
                            "内容策略拦截（疑似 system 指纹误报），换中性提示词重试一次；\
                             降级持续到 {until_text}（届时恢复「{}/{}」）",
                            attempt_prompt.mode.label(),
                            provider_id,
                        );
                        ctx.telemetry.note_attempt_retry(
                            &reason,
                            Some(i64::from(failure.error.status_code)),
                            0,
                        );
                        // （ is 1 ， after allrequest）——
                        // and 「 」，； that 1 
                        // request in request can to Reason。
                        logging::log("[Upstream]", &format!("⚠️ {reason}"));
                        continue;
                    }
                    // ── 2：token → new after 1 1 ──────
                    // in 「failure + is token + no new 」 when 。
                    // new failure、 or failure（ when `refreshed` already as true）
                    // to down 1 ， not will empty。
                    //
                    // Note this 1 in ** before **：401 new 1 
                    // （ 1 、 1 ）， is not is ，
                    // not this new 、 not this in this 1 this 1 failure ——
                    // if success， this 1 is success（`break response` that 
                    // will up ）。 this and `attempts` 1 ： that 1 +1。
                    if matches!(failure.class, UpstreamErrorClass::TokenExpired { .. })
                        && !named_switch
                        && !refreshed
                    {
                        refreshed = true;
                        let account_id = target.account_id.clone().unwrap_or_default();
                        // 401 new is 「 1 」（ not ），
                        // and 1 —— request can 
                        // 「 this 1 no has 、Why」， not this 1 。
                        ctx.telemetry.note_attempt_retry(
                            &format!("token 被上游拒绝（401），刷新后重试一次（账号 {account_id}）"),
                            Some(i64::from(failure.error.status_code)),
                            0,
                        );
                        logging::console_line(
                            "[Upstream]",
                            &format!(
                                "token 被上游拒绝（401），尝试刷新后重试一次（账号 {account_id}）"
                            ),
                        );
                        if adapter
                            .refresh_access_token(&service.store, &account_id)
                            .await
                            .is_ok()
                        {
                            // new already store： will （ new accessToken） 1 
                            if let Ok(fresh) = rotate::session_for(
                                service,
                                provider_id,
                                target.account_id.as_deref(),
                            )
                            .await
                            {
                                session = fresh;
                                continue;
                            }
                        } else {
                            // new **failure**： this is 「401 、、 no 」 
                            // 1 。 before this not ， use to 1 401，
                            // none from to is 「 no 」 is 「failure」——
                            // that is 「token not 」 when 1 。
                            // use log（ not is verbose）： in 401 when ，
                            // 、， not will requestpath that 。
                            logging::log(
                                "[Upstream]",
                                &format!(
                                    "账号 {account_id} 的凭证续期失败，本次 401 无法通过重试恢复\
                                     （请检查该账号的 refreshToken 是否仍有效）"
                                ),
                            );
                        }
                    }
                    // ── 2b：** ** → to 「 need need new 」 ──
                    //
                    // ── Why is 「 new no 」 not is 「 to 401」──────
                    // 401 not already ： has new can （ up 
                    // 2 already ）。 has ** no has ** （Loomy this ，
                    // `supports_refresh() == false`） in 401 / 020002 when is 
                    // no —— `refresh_access_token` no ， 2
                    // that 1 not will ， is error can to client。
                    // for this ， to `TokenExpired` is 「 up this session
                    // not 」， this is 1 can 、 can （ by when 
                    // 1 to when ， not will available ）。
                    //
                    // Why in ** this 1 ** not is ：`classify` has status and 
                    // body， not to id； is 。 in up ，
                    // `needsRelogin`（`store_view::public_account` 1 ），
                    // will ， not up 。
                    if matches!(failure.class, UpstreamErrorClass::TokenExpired { .. })
                        && !adapter.supports_refresh()
                    {
                        if let Some(account_id) = target.account_id.clone() {
                            // in this when 1 ：401 will request
                            // ， 1 etc 。`set_needs_relogin`
                            // is etc ， this is 。
                            if !crate::server::core::blocking::in_place(|| service.store.needs_relogin(&account_id)) {
                                crate::server::core::blocking::in_place(|| service.store.set_needs_relogin(
                                    &account_id,
                                    Some("上游拒绝了这个登录态（会话已失效）"),
                                ));
                                logging::log(
                                    "[Upstream]",
                                    &format!(
                                        "账号 {account_id} 的登录态已被上游注销（该家无续期手段），\
                                         已标记为「需要重新登录」"
                                    ),
                                );
                            }
                        }
                    }
                    // ── this 1 already （ not will 1 ）→ ────
                    // in 1 / 3 before ： that will `continue 'accounts`
                    // （ down 1 、 new ） or `return Err`（），
                    // after not to this 1 `failure` 。
                    // 1 `note_attempt_started` for 1 this （success then by 
                    // down `break response` that ）， etc 
                    // attempts， not in 。
                    ctx.telemetry.finish_last_attempt(
                        Some(i64::from(failure.error.status_code)),
                        Some(&failure.error.message),
                    );
                    // ── 1： → + down 1 ──────────
                    if let (
                        UpstreamErrorClass::QuotaLimited {
                            status,
                            upstream_code,
                            reset_at,
                            ..
                        },
                        Some(account_id),
                    ) = (&failure.class, target.account_id.clone())
                    {
                        if !tried_ids.contains(&attempt_key) {
                            tried_ids.push(attempt_key.clone());
                            // = ** up **（ not is request）： up by ，
                            // （`cooldown_keys`） by ，
                            // `routing::CooldownKeys`。
                            let reset_text = rotate::mark_account_limited(
                                service,
                                &account_id,
                                &wire_model,
                                *status as i32,
                                *upstream_code,
                                *reset_at,
                                &failure.error.message,
                            );
                            let limit_at =
                                rotate::account_limit_reset_at(service, &account_id, &wire_model);
                            // （Sink）：** has ** when 429 （ is 
                            // ， not ）， to all no 429 after —— 
                            // ， not is to 1 。success 1 or 
                            // after （ route_state.rs）。
                            if ctx.routing.settings.sink && ctx.pinned_account.is_none() {
                                service.route.sink_mark(&account_id, logging::now_ms());
                            }
                            // and 「」： as ，
                            // when `request →` before （ `limit_model_label`）
                            let limit_label = limit_model_label(&model, &wire_model);
                            let from_label =
                                account_label(target.account.as_ref(), &account_id, &session);
                            if !rotate::allows_route_skip(service, &account_id) {
                                rotate::report_limit_event(
                                    "warn",
                                    &format!("账号「{from_label}」对模型 {limit_label} 已限额，已关闭失败换号"),
                                    Some(&from_label), None, &wire_model, *upstream_code,
                                    *status as i32, limit_at, None, provider_id,
                                );
                                return Err(failure.error);
                            }
                            match rotate::pick_next_account(
                                service,
                                provider_ids,
                                &cooldown_keys,
                                &tried_ids,
                                ctx.pinned_account,
                                &ctx.routing,
                            ) {
                                Some(next) => {
                                    let next_label = account_display(&next);
                                    let next_priority =
                                        next.get("priority").and_then(Value::as_i64);
                                    let next_provider = rotate::provider_of(&next);
                                    let next_home = if next_provider == provider_id {
                                        String::new()
                                    } else {
                                        format!(
                                            "，切换提供商 → {}",
                                            kind_from_id(next_provider)
                                                .map(|kind| meta(kind).label)
                                                .unwrap_or(next_provider)
                                        )
                                    };
                                    let reset_hint = reset_hint(&reset_text);
                                    // this 1 in ：request that by 「 
                                    // error + down 1 」 1 
                                    // （ is ）。
                                    logging::console_line(
                                        "[Upstream]",
                                        &format!(
                                            "⚠️ 账号 {from_label} 对模型 {limit_label} 已限额{reset_hint}，\
                                             按优先级降级 → {next_label}（优先级 {}{next_home}）",
                                            next_priority
                                                .map(|value| value.to_string())
                                                .unwrap_or_else(|| "-".to_string()),
                                        ),
                                    );
                                    // is ****（ 「」
                                    // will 、 use can ）， not is request 
                                    // —— is in that 1 ，
                                    // and up that in this 。
                                    rotate::report_limit_event(
                                        "warn",
                                        &format!(
                                            "账号「{from_label}」对模型 {limit_label} 已限额{reset_hint}，\
                                             按优先级降级 → 「{next_label}」",
                                        ),
                                        Some(&from_label),
                                        Some(&next_label),
                                        &wire_model,
                                        *upstream_code,
                                        *status as i32,
                                        limit_at,
                                        next_priority,
                                        provider_id,
                                    );
                                    // this 1 down 1 （ to ）
                                    continue 'accounts;
                                }
                                None => {
                                    // all in / use —— ** when **
                                    // ：client（Claude Code / Codex etc will this 
                                    // ） can etc ， not is 
                                    // 1 。 not to （ no has when ） when 
                                    // ， and before 。
                                    let earliest_reset = rotate::earliest_limit_reset_at(
                                        service,
                                        provider_ids,
                                        &cooldown_keys,
                                    );
                                    let wait_hint = if earliest_reset > 0 {
                                        let text = format_reset_text(earliest_reset as f64);
                                        if text.is_empty() {
                                            String::new()
                                        } else {
                                            format!("，最早 {text} 恢复")
                                        }
                                    } else {
                                        String::new()
                                    };
                                    let message = format!(
                                        "{}（所有候选账号对模型 {model} 均已限额或禁用{wait_hint}）",
                                        failure.error.message
                                    );
                                    rotate::report_limit_event(
                                        "error",
                                        &format!(
                                            "模型 {limit_label} 在所有候选账号均已限额或禁用，\
                                             无法继续转发（尝试过 {} 个账号）",
                                            tried_ids.len(),
                                        ),
                                        Some(&from_label),
                                        None,
                                        &wire_model,
                                        *upstream_code,
                                        *status as i32,
                                        limit_at,
                                        None,
                                        provider_id,
                                    );
                                    return Err(GatewayError::with_status(
                                        *status as i32,
                                        message,
                                    )
                                    .with_optional_code(*upstream_code));
                                }
                            }
                        }
                    }
                    // ── 3： its error → down 1 ──────────
                    // old （ by ） to 1 failure will down 1 ； 
                    // this 「 down 1 」—— can can is down 1 ， can can 
                    // 1 。 has 「 no 」（otherwise will 
                    // 1 defaultempty）， no has error。
                    //
                    // 「」（`switches_left`）： use 
                    // when has not —— this is 「 1 request 
                    // 」 1 （429 that path not ，set）。
                    //
                    // ── Why not `defer_to_next_account` ──────────────
                    // that has 「failure」（ as empty 556） in ，
                    // that is before not 、 not is this —— 
                    // and will 1 by 「failure」 down 。
                    if let Some(account_id) = target.account_id.clone() {
                        if !tried_ids.contains(&attempt_key) {
                            tried_ids.push(attempt_key.clone());
                            // ── by failure（route_state.rs）──────────
                            // ： this failure 1 will （failure
                            // ，limit 1 when ）；：failure to 。
                            // 429（） not this —— has ，
                            // and 「 etc 」「」。
                            let route_settings = ctx.routing.settings;
                            if route_settings.failover_backoff {
                                service.route.backoff_mark(&account_id, logging::now_ms());
                            }
                            if route_settings.breaker_enabled {
                                let opened = service.route.breaker_failure(
                                    &account_id,
                                    route_settings.breaker_threshold,
                                    route_settings.breaker_timeout_seconds,
                                    logging::now_ms(),
                                );
                                if opened {
                                    logging::console_line(
                                        "[Upstream]",
                                        &format!(
                                            "⛔ 账号 {} 连续失败 {} 次，熔断 {} 秒（到点放行试一次，\
                                             连续成功 {} 次恢复）",
                                            account_display(target.account.as_ref().unwrap_or(&serde_json::Value::Null)),
                                            route_settings.breaker_threshold,
                                            route_settings.breaker_timeout_seconds,
                                            route_settings.breaker_success_threshold,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                    //
                    // and in [`defer_to_next_account`]： and 「
                    // failure」 use 1 ， not this has 。
                    if defer_to_next_account(
                        service,
                        provider_ids,
                        &cooldown_keys,
                        &mut tried_ids,
                        &target,
                        &session,
                        provider_id,
                        &model,
                        "转发失败",
                        failure.error.status_code,
                        &mut switches_left,
                        switch_total,
                        ctx.pinned_account,
                        &ctx.routing,
                    ) {
                        continue 'accounts;
                    }
                    return Err(failure.error);
                }
            }
        };

        // requestsuccess： this for this （ has ） already ，。
        // is ** up **（ and 、 1 ， `routing::CooldownKeys`）。
        cap_cleared(
            service,
            &target,
            &wire_model,
            &limit_model_label(&model, &wire_model),
            &session,
            provider_id,
        ).await;
        logging::verbose(
            "[Upstream]",
            &format!(
                "上游响应 HTTP {}（{}ms）",
                response.status().as_u16(),
                logging::now_ms() - started_at
            ),
        );

        let response = match response {
            super::images::ImageResponse::Complete(body) => {
                return Ok(ForwardOutcome::Completion { body });
            }
            super::images::ImageResponse::Live(response) => response,
        };

        // ── ：response to （must in consume response before ）──────────
        // and response in this ；response by after / 1 
        // （ `ForwardStream` / `aggregate_sse_completion`）。
        if let Some(capture) = ctx.telemetry.capture() {
            capture.attach_response(response.status().as_u16(), response.headers());
        }

        // ── （ has ， `upstream::refusal`）──────
        // has up 1 ****：HTTP 200、SSE ，
        // is 1 （ in `finish_reason`）。 HTTP
        // this is **success** —— not ，client （：
        // ZCode agentic request to OfficeAce ）。
        //
        // in response after 、 down before ： 1 （ has ，
        // `refusal` module header），， not 。** no `finish_reason`
        // 1 not **（`stream_refusal_finishes` defaultempty ⇒ ）。
        //
        // when ： not （ this is ， not is ），** this **——
        // has /， this 1 when to client 1 ** can error**，
        // not is that up 。
        //
        // `Gate`：`Live` is `Response`（ / 200
        // this ， 1 not ）；`Held` is （ already `Response`， use 
        // + remaining）。 use not is `Option`， is as `response` in **
        // 1 ** —— otherwise use checknone「None that no 」。
        let refusal_finishes = adapter.stream_refusal_finishes();
        // （ `ProviderAdapter::stream_refusal_markers`）： has up 
        // **quota** 1 200 + ，`finish_reason` not in ——
        // and `finish_reason` will 1 success（ not 、 not ）。
        let refusal_markers = adapter.stream_refusal_markers();
        let gate = if (!refusal_finishes.is_empty() || !refusal_markers.is_empty())
            && response_protocol == crate::server::core::providers::adapter::UpstreamResponse::Chat
            && response.status().is_success()
        {
            let head = refusal::hold_head(response).await;
            // 1 `( to , is not is quota, to client)`：
            // · quota（**** in ）⇒ ****（ to when ）+ ；
            // · （`finish_reason` in ）⇒ not （， will ）+ 。
            let refusal_hit =
                match refusal::matched_refusal_marker(&head.buffered, refusal_markers) {
                    Some(marker) => Some((
                        format!("上游以正文拒绝本次答复（命中标记「{marker}」）：本次答复不采纳"),
                        true,
                        "命中正文配额标记".to_string(),
                    )),
                    None => head
                        .finish_reason
                        .as_deref()
                        .filter(|finish| refusal::is_refusal(refusal_finishes, finish))
                        .map(|finish| {
                            (
                                format!("上游拒答（finish_reason={finish}，疑似内容策略拦截）：本次答复不采纳"),
                                false,
                                format!("finish_reason={finish}"),
                            )
                        }),
                };
            if let Some((message, is_quota, refusal_label)) = refusal_hit {
                    // up that ：quota ** when ** need from 
                    // （「 HH:MM」 and 「… HH:MM:SS UTC+8」，
                    // `errors::parse_quota_reset_at`）。SSE no ——
                    // in when ， not JSON 。
                    let upstream_text = String::from_utf8_lossy(&head.buffered).to_string();
                    // to client etc when （seconds）：0 = not when ， that 429 not 
                    let mut refusal_reset_at: i64 = 0;
                    // quota： this 1 use ⇒ 。 when to 
                    // `mark_account_limited` when 
                    // （「…current in HH:MM and …」）； not when 
                    // is 10 minutes。
                    if is_quota {
                        if let Some(account_id) = &target.account_id {
                            refusal_reset_at = parse_quota_reset_at(&upstream_text);
                            let reset_text = rotate::mark_account_limited(
                                service,
                                account_id,
                                &model,
                                429,
                                None,
                                if refusal_reset_at > 0 { Some(refusal_reset_at) } else { None },
                                &message,
                            );
                            logging::console_line(
                                "[Upstream]",
                                &format!(
                                    "⚠️ {provider_id} 账号 {account_id} 配额拒绝（正文标记命中），已限流{reset_text}"
                                ),
                            );
                        }
                    }
                    ctx.telemetry.finish_last_attempt(Some(i64::from(head.status)), Some(&message));
                    logging::console_line(
                        "[Upstream]",
                        &format!(
                            "⚠️ {provider_id} 对模型 {model_label} 以「{refusal_label}」拒绝本次答复，\
                             本家不采纳"
                        ),
                    );
                    // and its failure 1 ： this 1 as failure、 by down 1 。
                    // is 「 has no has down 1 /」， not is 「 this 1 can not can 」——
                    // 1 body 1 not （ by ）， not 。
                    // that and up failure （file this 
                    // already has ， not ）。
                    if target.account_id.is_some() && !tried_ids.contains(&attempt_key) {
                        tried_ids.push(attempt_key.clone());
                    }
                    // no / use when to client that ： need is 1 ——
                    // is 「 1 or 」，quota is 「 etc after 」。
                    let client_text = if is_quota {
                        // is 1 ， and ** when ** body（ and `Retry-After`
                        // 1 ）—— body client need etc 。
                        let when = if refusal_reset_at > 0 {
                            format!(
                                "，额度预计在 {} 重置",
                                crate::server::errors::format_reset_at_text(refusal_reset_at)
                            )
                        } else {
                            String::new()
                        };
                        format!(
                            "上游配额拒绝（该账号的用量窗口已耗尽{when}）：没有其他可用的账号或提供商可接管。\
                             请在窗口重置后重试，或在设置中更换该模型的服务商"
                        )
                    } else {
                        format!(
                            "上游拒答（{refusal_label}）：该答复疑似被上游内容策略拦截，\
                             且没有其他可用的账号或提供商可接管。请调整内容后重试，或在设置中更换该模型的服务商"
                        )
                    };
                    match rotate::pick_next_account(
                        service,
                        provider_ids,
                        &cooldown_keys,
                        &tried_ids,
                        ctx.pinned_account,
                        &ctx.routing,
                    ) {
                        Some(next) => {
                            if !take_switch(&mut switches_left, switch_total) {
                                return Err(quota_refusal_error(
                                    client_text,
                                    refusal_reset_at,
                                    is_quota,
                                ));
                            }
                            let next_home = {
                                let next_provider = rotate::provider_of(&next);
                                if next_provider == provider_id {
                                    String::new()
                                } else {
                                    format!(
                                        "，切换提供商 → {}",
                                        kind_from_id(next_provider)
                                            .map(|kind| meta(kind).label)
                                            .unwrap_or(next_provider)
                                    )
                                }
                            };
                            logging::console_line(
                                "[Upstream]",
                                &format!(
                                    "⚠️ 账号 {} 对模型 {model} 上游拒答（HTTP {}），\
                                     按队列顺延 → {}（优先级 {}{next_home}）",
                                    account_label(
                                        target.account.as_ref(),
                                        &target.account_id.clone().unwrap_or_default(),
                                        &session
                                    ),
                                    head.status,
                                    account_display(&next),
                                    next.get("priority")
                                        .and_then(Value::as_i64)
                                        .map(|value| value.to_string())
                                        .unwrap_or_else(|| "-".to_string()),
                                ),
                            );
                            continue 'accounts;
                        }
                        None => {
                            return Err(quota_refusal_error(client_text, refusal_reset_at, is_quota))
                        }
                    }
            }
            Gate::Held(head)
        } else {
            Gate::Live(response)
        };

        let mut response_policy = frame_policy_of(adapter, &model);
        if ctx.routing.group.is_some() { response_policy.rewrite = Some(super::sse::ModelRewrite { requested: model.clone() }); }
        response_policy.native_tools = native_tools;
        // ── up response（ in request when 1 and to ）──────────────────
        // up chat SSE（`ForwardStream` / default）；
        // has 1 （ `upstream::translate` module header）：
        // - ZCode Anthropic SSE；
        // - Command Code NDJSON（`application/x-ndjson`，**HTTP 200**、
        // error in ）， that return 200 not this 1 success —— by 
        // error， and use 1 down ；
        // - Antigravity Gemini v1internal SSE（`data: {"response":{…}}`
        // ，path is Gemini ）—— up is ，
        // （errorerror）。
        // in ** before **， is and use 1 down ：
        // reasoning and 、usage 、model 、 not need need 。
        // chat （） this 1 ， down path。
        //
        // in Chat up （ up that `response_protocol` ），
        // to when `gate` is `Live`、`response` is 。
        if response_protocol
            != crate::server::core::providers::adapter::UpstreamResponse::Chat
        {
            use crate::server::core::providers::adapter::UpstreamResponse;
            let Gate::Live(response) = gate else {
                // not can ： in Chat up （ up already ）。
                return Err(GatewayError::with_status(
                    500,
                    "内部错误：非 Chat 通道不应进入拒绝门控",
                ));
            };
            // need in consume response before （ and chat path 1 when ）
            let status = response.status().as_u16();
            let translated: futures::stream::BoxStream<
                'static,
                Result<bytes::Bytes, std::io::Error>,
            > = match response_protocol {
                UpstreamResponse::Anthropic => Box::pin(
                    super::translate::AnthropicToChatStream::new(
                        response,
                        &wire_model,
                        ctx.telemetry,
                    ),
                ),
                UpstreamResponse::AiSdkNdjson => Box::pin(
                    super::translate::AiSdkNdjsonToChatStream::new(
                        response,
                        &wire_model,
                        ctx.telemetry,
                    ),
                ),
                UpstreamResponse::AntigravityGemini => Box::pin(
                    super::translate::AntigravityToChatStream::new(
                        response,
                        &wire_model,
                        &antigravity_session,
                        ctx.telemetry,
                    ),
                ),
                // ：SSE and ， by 
                // （ that module header）
                UpstreamResponse::ChatglmSnapshot => {
                    Box::pin(super::translate::ChatglmToChatStream::new(
                        response,
                        &wire_model,
                        ctx.telemetry,
                    ))
                }
                // `if` already excluding， this 1 as match in ，** not can can 
                // to **。 to ：error not is panic
                // （release is `panic=abort`， 1 panic ）。
                UpstreamResponse::Chat | UpstreamResponse::Images(_) => {
                    return Err(GatewayError::with_status(
                        500,
                        "内部错误：Chat 协议不该进入上游响应翻译分派",
                    ))
                }
            };
            if ctx.stream {
                return Ok(crate::server::core::plugin_host::hold_adapter(ForwardOutcome::Stream {
                    status,
                    stream: Box::new(super::ForwardStream::from_translated(
                        translated,
                        slot.take(),
                        connections.handoff(),
                        ctx.telemetry.clone(),
                        response_policy.clone(),
                    )),
                }, adapter_handle));
            }
            let aggregated = super::aggregate::aggregate_frame_stream(
                translated,
                ctx.telemetry.clone(),
                response_policy.clone(),
            )
            .await?;
            return Ok(ForwardOutcome::Completion {
                body: aggregated.body,
            });
        }

        // Chat ： by （ up `Gate` ）。 
        // `return`， this is 1 「 after not can 」 match。
        match gate {
            Gate::Live(response) => {
                if ctx.stream {
                    return Ok(crate::server::core::plugin_host::hold_adapter(ForwardOutcome::Stream {
                        status: response.status().as_u16(),
                        // to ： / client / drop when etc ；
                        // （`handoff` all， ，
                        // 1 1 ）
                        stream: Box::new(super::ForwardStream::new(
                            response,
                            slot.take(),
                            connections.handoff(),
                            ctx.telemetry.clone(),
                            response_policy.clone(),
                        )),
                    }, adapter_handle));
                }
                let aggregated = super::aggregate::aggregate_sse_completion(
                    response,
                    ctx.telemetry.clone(),
                    response_policy.clone(),
                )
                .await?;
                log_aggregated(&aggregated);
                return Ok(ForwardOutcome::Completion { body: aggregated.body });
            }
            Gate::Held(head) => {
                let status = head.status;
                let capture = ctx.telemetry.capture();
                let source = head.into_stream(capture);
                if ctx.stream {
                    return Ok(crate::server::core::plugin_host::hold_adapter(ForwardOutcome::Stream {
                        status,
                        stream: Box::new(super::ForwardStream::from_translated(
                            source,
                            slot.take(),
                            connections.handoff(),
                            ctx.telemetry.clone(),
                            response_policy.clone(),
                        )),
                    }, adapter_handle));
                }
                let aggregated = super::aggregate::aggregate_frame_stream(
                    source,
                    ctx.telemetry.clone(),
                    response_policy.clone(),
                )
                .await?;
                log_aggregated(&aggregated);
                return Ok(ForwardOutcome::Completion { body: aggregated.body });
            }
        }
    }
    Err(GatewayError::with_status(500, "上游转发重试次数超限"))
}

/// after Chat response：`Live` = `Response`（）；
/// `Held` = （ use + remaining， `upstream::refusal`）。
enum Gate {
    Live(reqwest::Response),
    Held(refusal::HeldHead),
}

/// （ use ， has 1 ）。
fn log_aggregated(aggregated: &super::aggregate::AggregatedCompletion) {
    let choice = aggregated.body.get("choices").and_then(|value| value.get(0));
    let content_chars = choice
        .and_then(|choice| choice.pointer("/message/content"))
        .and_then(Value::as_str)
        .map(|text| text.chars().count())
        .unwrap_or(0);
    let finish = choice
        .and_then(|choice| choice.get("finish_reason"))
        .and_then(Value::as_str)
        .unwrap_or("");
    logging::verbose(
        "[Upstream]",
        &format!(
            "聚合完成: chunks={} content={content_chars} 字符 finish={finish}",
            aggregated.chunk_count
        ),
    );
}

/// will path ： ** when ** to 。
///
/// ── Why ──────────────────────────────────────────
/// as 「 when not in 」 this 1 can （ tests 
/// `stateful_quota_bookkeeping_keeps_the_adapter_reset_at`）。 before this is 
/// `for` ，`mark_account_limited` `reset_at` `None`：
/// that for （CodeArts ⇒ down 1 ，
/// `providers::codearts::daily_pool_reset_at`； will to when 
/// is 1 ）， in parameter up ， 「 10 minutes」
/// （`account_store::store_admin::mark_rate_limited`）。
///
/// this not ， is ** 1 already by new **：
/// need etc has ， 10 minutes to 1 ⇒ 2.9.9 up when 
/// 8 （ 0.2–0.3 seconds、 not 、request 200 —— response not ，
/// in requests.attempt_details ）。
/// none from 1 is `*reset_at` （file「 1」），path
/// to this is 1 。
///
/// ── ：， not ──────────────────
/// `reset_at` is 1 「 no 」 that （client will to 
/// response、 will empty， has can after ）。 `class` 、 in 
/// destructure， use no has 「 1 」 this can error。
///
/// `QuotaLimited` return —— by use `if let` ， this is 
/// ： path use when not this 。
fn mark_conversation_limited(
    service: &UpstreamService,
    account_id: &str,
    group: &[String],
    class: &UpstreamErrorClass,
    message: &str,
) {
    let UpstreamErrorClass::QuotaLimited { status, upstream_code, reset_at, .. } = class else {
        return;
    };
    for name in group {
        rotate::mark_account_limited(
            service,
            account_id,
            name,
            i32::from(*status),
            *upstream_code,
            *reset_at,
            message,
        );
    }
}

/// **** 1 （； and [`attempt_stateful`] ）。
///
/// ── use and not use （ and has path for ）───────────────
/// ```text
/// use ：（ `target` by use ）、telemetry 、
/// 429 （ in use Err ）、 in and 
/// not use ：「 1 」 —— `providers::custom::forward`
/// （ no has ， not ProviderKind， mod.rs module header）
/// ```
///
/// ── and has path （ is ）──────────────────────
/// 1. ** no has **： ，`account_id`
/// as empty 503（ and in `custom::forward` ， this not 
/// —— Reason， not will ）；
/// 2. ** not `ensure_access_token`**： is use apiKey，
/// no has 「 new 」 —— is （`custom_credential_by_id`）。
///
/// ── and （Why this not `wire_target_for_provider`）──
/// `send_body` for id will （ that modelRules ），
/// 「alias → up 」 and 「 etc 」 in `custom::forward` 
/// （ is up `models` / `mappings`）。 in use Err
/// use `custom_forward::cooldown_model` —— and 1 、 1 
/// ， use not will （ that ）。
///
/// ── （`core::proxy_rotate`） 429 ─────────────
/// this path is ** 1 ** will 「429」 1 ：（ before has OpenCode Zen
/// ） down 429 is 「 this IP use 」， not is 「 this not 」
/// —— 1 。 in up 
/// [`MAX_ROTATE_EXIT_RETRIES`] ， and Reason 
/// （`note_attempt_retry`， and 「 0 」 1 ： 1 、 1 
/// ， not 1 ）； 429 error to use ， has 「 by +
/// down 1 」。 、 error 1 not （ by ）。
///
/// and nonepath for ：success when ** up **（ in 
/// `ForwardOutcome::Stream.status`、 as 200 —— 2xx in forward
/// already error），failure when error and 。
async fn attempt_custom(
    service: &UpstreamService,
    ctx: &ProviderContext<'_>,
    target: RouteTarget,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
    degraded: bool,
) -> Result<ForwardOutcome, GatewayError> {
    let provider_id = target.provider.clone();
    // ： provider + is this 1 （attempts +1）。
    // and has path（ → id）； no has will for can 。
    let attempt_account = account_label(
        target.account.as_ref(),
        target.account_id.as_deref().unwrap_or(""),
        &Value::Null,
    );
    ctx.telemetry.note_quota_channel(target.account.as_ref());
    ctx.telemetry
        .note_attempt(target.account_id.as_deref(), &attempt_account, &provider_id);
    ctx.telemetry
        .note_attempt_started(&provider_id, &attempt_account);
    if let Some(notice) = target.proxy_notice.as_deref() {
        ctx.telemetry.note_attempt_notice(notice);
    }
    let started_at = logging::now_ms();
    // （ + ） and 1 when ： already 、 this 1 
    // ** will **。`send_body` for id is （ 
    // modelRules）， is 「/ after clientrequest」—— 
    // （ alias → 、 etc ） in forward 。
    let send = send_body(ctx, &provider_id, target.account.as_ref(), degraded).await;
    // 「 up 」**** as ：send_body for id is （ 
    // is request）， and in forward —— this by 
    // 1 （note_upstream_model is ， after 1 as ；empty ）。
    let wire_model = if ctx.routing.group.is_some() { send.wire_model.clone() } else { custom_forward::cooldown_model(&provider_id, &model_of(ctx.body)) };
    ctx.telemetry.note_upstream_model(&wire_model);
    logging::verbose(
        "[Upstream]",
        &format!(
            "自定义转发 model={} stream={} account={} priority={} 出口={} provider={provider_id}",
            limit_model_label(&model_of(ctx.body), &custom_forward::cooldown_model(&provider_id, &model_of(ctx.body))),
            ctx.stream,
            target.account_id.as_deref().unwrap_or("-"),
            target
                .priority
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            describe_proxy(target.proxy.as_ref()),
        ),
    );
    // ── 429 ： down 1 ────────────────────
    // OpenCode Zen 429 is ** by IP** （ this use ，
    // `core::proxy_rotate` module header）—— 1 ， by 「
    // 」 will this 。 this in up 
    // [`MAX_ROTATE_EXIT_RETRIES`] ； 429 error to use ， has 
    // 「 by + down 1 」。
    //
    // for 429（`is_quota_limit`）：error / 5xx / 4xx and none，
    // is 1 （ that failure by use down 1 ）。
    let mut proxy = target.proxy.clone();
    let mut rotate_left = if custom_forward::is_rotate_account(target.account.as_ref()) {
        MAX_ROTATE_EXIT_RETRIES
    } else {
        0
    };
    loop {
        match custom_forward::forward(
            &service.store,
            &provider_id,
            target.account_id.as_deref().unwrap_or(""),
            &send.body,
            ctx.routing.group.as_ref().map(|group| group.model.as_str()),
            proxy.clone(),
            ctx.stream,
            ctx.telemetry,
            slot,
            connections,
        )
        .await
        {
            Ok(outcome) => {
                // success：** up **（ in outcome 、 200）。
                // for is empty —— by use 429
                // ，success `cap_cleared` 1 path（ this and has 
                // path 1 in outcome to after use ， attempt_stateful ）。
                let status = match &outcome {
                    ForwardOutcome::Stream { status, .. } => i64::from(*status),
                    ForwardOutcome::Completion { .. } => 200,
                };
                cap_cleared(
                    service,
                    &target,
                    &custom_forward::cooldown_model(&provider_id, &model_of(ctx.body)),
                    &limit_model_label(
                        &model_of(ctx.body),
                        &custom_forward::cooldown_model(&provider_id, &model_of(ctx.body)),
                    ),
                    &Value::Null,
                    &provider_id,
                ).await;
                logging::verbose(
                    "[Upstream]",
                    &format!("自定义转发完成: HTTP {status}（{}ms）", logging::now_ms() - started_at),
                );
                ctx.telemetry.finish_last_attempt(Some(status), None);
                return Ok(outcome);
            }
            Err(error) => {
                // （ 1 、 1 ： 1 ，
                // and 「 0 」 1 ， `note_attempt_retry`）。
                if rotate_left > 0 && error.is_quota_limit() {
                    if let Some(account_id) = target.account_id.as_deref() {
                        let store = service.store.clone();
                        let account_id = account_id.to_string();
                        let next = crate::server::core::blocking::run(move || store.next_rotate_proxy(&account_id)).await?;
                        match next {
                            Ok(Some(next)) => {
                                rotate_left -= 1;
                                let reason = format!(
                                    "轮询换出口重试（下一个 {label}）：上游 {status} 表示该出口已用完额度",
                                    label = describe_proxy(Some(&next)),
                                    status = error.status_code,
                                );
                                logging::verbose("[Upstream]", &format!("🔁 {reason}"));
                                ctx.telemetry.note_attempt_retry(
                                    &reason,
                                    Some(i64::from(error.status_code)),
                                    0,
                                );
                                proxy = Some(next);
                                continue;
                            }
                            // not is （ up not will to ： up already by ）——
                            // by error， not 
                            Ok(None) => {}
                            // is 1 not （empty / use ）：
                            // error has （ up Why）， that 
                            // 1 to ， when 
                            Err(reason) => {
                                ctx.telemetry.note_attempt_retry(&reason, None, 0);
                            }
                        }
                    }
                }
                // in ： this error will as GatewayError （ or by use 
                // Err ）， by `api::chat` / `api::protocol` request。
                logging::console_line(
                    "[CustomProvider]",
                    &format!("❌ {}", error.message),
                );
                ctx.telemetry
                    .finish_last_attempt(Some(i64::from(error.status_code)), Some(&error.message));
                return Err(error);
            }
        }
    }
}

/// ** has provider** 1 （ §4.2.1；`is_stateful` as true this ）。
///
/// ── use and not use （ and nonepath for ）───────────────
/// ```text
/// use ：（ `target` by use ）、telemetry 、
/// success after 、 in （SlotHoldingStream）
/// not use ：build_chat_request / send_chat_request / classify_error 
/// ```
///
/// ── Why this ** no has **（ for ，W5-T-d4）────────
/// nonepath is `loop`：401/429 after down 1 ，「 up to 
/// can error」。 CatPaw ** no has **：
/// - `catpaw-local-proxy` no has 429 （`grep -rn "429" *.mjs` in ），
/// no has ；
/// - is 「 use in in that 1 」（`account-store.mjs` 
/// `getCurrentCredentials`）， old conversation
/// （`account-routes.mjs` `notifySwitch` → `clearClientToolSessions()`），
/// no has 「failure → 」 path；
/// - 「 will in in ，nonecreate new 」** not is **： that is 1 
/// conversation in up running， is `turn/stop` + 
/// （ `conversation::submit_round_with_self_heal`）， and none。
/// for up error** not not **（CatPaw no has ）；
/// by `classify_conversation_error` （default Fatal），，
/// by （ `attempt_queue`）。
/// use —— to CatPaw when 。
///
/// ── in （）─────────────────────────────────────
/// nonepath to `ForwardStream`； has path 1 
/// [`SlotHoldingStream`]，「 to response」 in up 1 。
/// （`connections`） and 1 ：success as when 1 to 。
async fn attempt_stateful(
    service: &UpstreamService,
    ctx: &ProviderContext<'_>,
    provider_id: &str,
    adapter: &dyn ProviderAdapter,
    target: RouteTarget,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
    degraded: bool,
) -> Result<ForwardOutcome, GatewayError> {
    let model = model_of(ctx.body);
    // no has （default） when ， has provider can 
    // —— and nonepath 1 and （`allows_anonymous_default_session`）
    if target.account_id.is_none() && !adapter.allows_anonymous_default_session() {
        return Err(GatewayError::with_status(
            503,
            format!(
                "{} 没有可用账号，无账号可转发：请在账号页添加并启用账号",
                adapter.label()
            ),
        ));
    }
    // 。failure** not **： will by `forward_conversation` 
    // to （e.g.「auth.json ， in CatPaw 」），
    // this before will 「 its can to 」 request。Note CatPaw
    // no has new （§9.1）：`ensure_access_token` in 。
    if let Some(account_id) = target.account_id.clone() {
        if let Err(error) = adapter.ensure_access_token(&service.store, &account_id).await {
            if crate::server::core::providers::is_scoped_runtime_provider(provider_id) {
                let name = account_label(target.account.as_ref(), &account_id, &Value::Null);
                ctx.telemetry.note_attempt(Some(&account_id), &name, provider_id);
                ctx.telemetry.note_attempt_started(provider_id, &name);
                ctx.telemetry.finish_last_attempt(Some(i64::from(error.status_code)), Some(&error.message));
                return Err(error); // FULL CAS/来源/出口失败不能沿用旧凭证再发送。
            }
            logging::verbose(
                "[Upstream]",
                &format!(
                    "账号 {account_id} 的凭证准备失败（继续尝试转发）: {}",
                    error.message
                ),
            );
        }
    }
    // ── ： already 、 this 1 ** will **， ────────
    // and nonepath 1 when and 1 ：failure（503/401） request not to this ，
    // not will 1 「 already 」； to is clientrequest。
    // `degraded` by use （） to ：path** no has **（ has 
    // provider 1 is 1 will ， no has 「」 this 1 ），
    // （ already ） need 。
    let send = send_body(ctx, provider_id, target.account.as_ref(), degraded).await;
    // ： and nonepath 1 by （ that ）
    let stripped = strip_internal_fields(&send.body);
    let body: &serde_json::Value = &stripped;
    // this 1 to up = （ and ， `SendBody`）
    let wire_model = &send.wire_model;
    // ： provider + is this 1 （attempts +1）。
    // and nonepath（ → will → id）；
    // this no has will for can （ will no ）， use 。
    // 1 、 use ， by nonepath。
    let attempt_account = account_label(
        target.account.as_ref(),
        target.account_id.as_deref().unwrap_or(""),
        &Value::Null,
    );
    ctx.telemetry.note_quota_channel(target.account.as_ref());
    ctx.telemetry
        .note_attempt(target.account_id.as_deref(), &attempt_account, provider_id);
    // ： and note_attempt for （ up 1 ）。
    // path in down match —— has provider no has ，
    // 1 is 1 ， 1 （`provider_loop` `'accounts` 
    // can can in to down 1 ， that will use ）。
    ctx.telemetry.note_attempt_started(provider_id, &attempt_account);
    ctx.telemetry.note_upstream_model(wire_model);
    ctx.telemetry.note_upstream_reasoning(send.wire_reasoning.clone());
    if let Some(notice) = target.proxy_notice.as_deref() {
        ctx.telemetry.note_attempt_notice(notice);
    }
    let started_at = logging::now_ms();
    logging::verbose(
        "[Upstream]",
        &format!(
            "会话式转发 model={} stream={} account={} priority={} 出口={} provider={provider_id}",
            limit_model_label(&model, wire_model),
            ctx.stream,
            target.account_id.as_deref().unwrap_or("-"),
            target
                .priority
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            describe_proxy(target.proxy.as_ref()),
        ),
    );
    match adapter
        .forward_conversation(
            &service.store,
            target.account_id.as_deref().unwrap_or(""),
            body,
            ctx.client_headers,
            target.proxy.clone(),
            ctx.stream,
            ctx.telemetry,
        )
        .await
    {
        Ok(outcome) => {
            // success： already has ， and will for 。
            cap_cleared(
                service,
                &target,
                wire_model,
                &limit_model_label(&model, wire_model),
                &Value::Null,
                provider_id,
            ).await;
            logging::verbose(
                "[Upstream]",
                &format!("会话式转发完成（{}ms）", logging::now_ms() - started_at),
            );
            // ： this path success is 「outcome to 」（ will 
            // by ， not to up HTTP ）， status to None、
            // not error —— before 「success」（none）。
            ctx.telemetry.finish_last_attempt(None, None);
            // will to up ；client by can request。
            let outcome = match outcome {
                ForwardOutcome::Stream { status, stream } if adapter.sse_model_rewrite() || ctx.routing.group.is_some() => {
                    return Ok(ForwardOutcome::Stream {
                        status,
                        stream: Box::new(super::ForwardStream::from_translated(
                            Box::pin(stream), slot.take(), connections.handoff(),
                            ctx.telemetry.clone(),
                            super::sse::FramePolicy::with_rewrite(Some(super::sse::ModelRewrite {
                                requested: model,
                            })),
                        )),
                    });
                }
                ForwardOutcome::Completion { mut body } if adapter.sse_model_rewrite() || ctx.routing.group.is_some() => {
                    if let Some(current) = body.get_mut("model").filter(|value| value.as_str() != Some(model.as_str())) {
                        *current = Value::String(model);
                    }
                    ForwardOutcome::Completion { body }
                }
                other => other,
            };
            Ok(attach_slot(outcome, slot, connections))
        }
        // returnerror；、 and by attempt_queue 。
        Err(error) => {
            // in ： this error will as GatewayError ， by `api::chat` /
            // `api::protocol` request（ error is 1 ）。
            logging::console_line("[Upstream]", &format!("❌ {}", error.message));
            // ：error （ has path error by ，
            // 502/500/ up can can ），error need use 1 —— and 
            // 「error」 is 1 。
            ctx.telemetry
                .finish_last_attempt(Some(i64::from(error.status_code)), Some(&error.message));
            Err(error)
        }
    }
}

/// has path ：** to up 、**（ `attempt_stateful`）。
///
/// Why need to up （ not is outcome ）： is 「 body 
/// in etc before 1 ****」，「」 is 「response down 」
/// （`InFlightGuard` ）。 has provider down by after ，
/// for is use `Box<dyn Stream>`， is this 1 as ** has that ** ——
/// not 、 not 、 not ， not 。
///
/// （`connections`）** 1 then **： when 1 to up ，
/// when return。 1 ， not need need 
/// when 。
fn attach_slot(
    outcome: ForwardOutcome,
    slot: &mut Option<InFlightGuard>,
    connections: &mut ConnectionGuard,
) -> ForwardOutcome {
    match outcome {
        ForwardOutcome::Stream { status, stream } => ForwardOutcome::Stream {
            status,
            stream: Box::new(SlotHoldingStream {
                inner: stream,
                _slot: slot.take(),
                // handoff all： （ its ）
                _connection: connections.handoff(),
            }),
        },
        other => {
            // ： already ， in this （ and nonepath）
            drop(slot.take());
            other
        }
    }
}

/// as 1 in ：** has in and **（ `attach_slot`）。
///
/// `inner` `Item`（ `Err`）， no has and 。
struct SlotHoldingStream {
    inner: Box<dyn futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Send + Unpin>,
    /// ： drop（ / client / ） when and etc 
    _slot: Option<InFlightGuard>,
    /// ： 1 when ， this -1
    _connection: ConnectionGuard,
}

impl futures::Stream for SlotHoldingStream {
    type Item = Result<bytes::Bytes, std::io::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::StreamExt;
        // is Unpin（Box / Option）， Unpin，get_mut 
        self.get_mut().inner.poll_next_unpin(cx)
    }
}

/// requestsuccess after （「 already available」）。
///
/// is as successpath in 401 new after to ， that when `target` and 
/// `session` in use —— 「success to 」 1 can 。
async fn cap_cleared(
    service: &UpstreamService,
    target: &RouteTarget,
    model: &str,
    model_label: &str,
    session: &Value,
    provider_id: &str,
) {
    let Some(account_id) = target.account_id.as_deref() else {
        return;
    };
    let store = service.store.clone();
    let id = account_id.to_string();
    let wire_model = model.to_string();
    let cleared = crate::server::core::blocking::run(move || store.clear_rate_limit(&id, &wire_model)).await;
    if !matches!(cleared, Ok(true)) {
        return;
    }
    let label = account_label(target.account.as_ref(), account_id, session);
    rotate::report_limit_event(
        "info",
        &format!("账号「{label}」对模型 {model_label} 已恢复可用"),
        None,
        Some(&label),
        model,
        None,
        0,
        0,
        None,
        provider_id,
    );
}

/// up request （/ to empty， and before 1 ）
fn model_of(body: &Value) -> String {
    body.get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// ：** up ** as 。
///
/// is up ， 「」 is —— if request，
/// will 「 `gpt-5.6-luna`、 `deepseek-v4.1-flash`」 this 
/// 。 use ，only in request not （） when 
/// 1 `request →` before ， use can for up 「 that 」。
fn limit_model_label(requested: &str, wire: &str) -> String {
    let requested = requested.trim();
    let wire = wire.trim();
    if wire.is_empty() {
        // no has （request no model）：request， and before 1 
        return if requested.is_empty() { "(默认)".to_string() } else { requested.to_string() };
    }
    if requested.is_empty() || requested.eq_ignore_ascii_case(wire) {
        return wire.to_string();
    }
    format!("{requested} → {wire}")
}

/// down ： can by ， by config， use not 
/// provider 。
///
/// - model ： / AutoClaw / Cline up will ，
/// `raccoon` and `sse.rs` module header；
/// - ：WorkBuddy / AutoClaw up will in 
/// 1 content（ `sse::is_newline_keepalive`），
/// ** and ** need `stripNewlineKeepalive` （default，
/// `config::KEY_STRIP_NEWLINE_KEEPALIVE`）—— then and after 
/// in [`super::sse::FramePolicy::assembled`]。
///
/// its provider（ and no ） to `FramePolicy`， down 
/// not （workbuddy that 「 not 」 need in when 1 ）。
fn frame_policy_of(adapter: &dyn ProviderAdapter, model: &str) -> super::sse::FramePolicy {
    super::sse::FramePolicy::assembled(
        adapter
            .sse_model_rewrite()
            .then(|| super::sse::ModelRewrite { requested: model.to_string() }),
        adapter.sse_strip_newline_chunks(),
        crate::server::config::strip_newline_keepalive(),
    )
}

/// in `send_with_retry` return（ [`cancelled_error`] ：
/// use by ， not this `class`）。
fn cancelled_failure() -> OutboundFailure {
    OutboundFailure {
        class: UpstreamErrorClass::Fatal {
            status: cancellation::MANUAL_TERMINATED_STATUS as u16,
            message: cancellation::MANUAL_TERMINATED.to_string(),
            upstream_code: None,
        },
        error: cancellation::cancelled_error(),
        stop: true,
        policy: Default::default(),
    }
}

/// 1 up request； when **** etc （ not etc response when ）。
///
/// up future （ and client when is 1 ），
/// error use —— not will failure（ use in 
/// failure before ）。
async fn send_or_cancel(
    network: &mut super::network_fallback::Fallback,
    telemetry: &crate::server::core::upstream::usage::RequestTelemetry,
) -> Result<reqwest::Response, super::request::UpstreamRequestError> {
    let Some(token) = telemetry.cancel_token() else {
        return network.send().await;
    };
    tokio::select! {
        result = network.send() => result,
        _ = token.cancelled() => Err(super::request::UpstreamRequestError {
            message: cancellation::MANUAL_TERMINATED.to_string(),
            reason: cancellation::MANUAL_TERMINATED.to_string(),
            safe_to_retry: false,
            network_failure: false,
        }),
    }
}

/// ； when （ use not this etc 1 ）。
///
/// after by use check and —— this not ，「」。
async fn sleep_or_cancel(
    telemetry: &crate::server::core::upstream::usage::RequestTelemetry,
    delay_ms: u64,
) {
    let sleep = tokio::time::sleep(Duration::from_millis(delay_ms));
    tokio::pin!(sleep);
    match telemetry.cancel_token() {
        Some(token) => tokio::select! {
            _ = &mut sleep => {}
            _ = token.cancelled() => {}
        },
        None => sleep.await,
    }
}

/// 1 up request，「 can 」（ / set set）。
///
/// default HTTP 2xx success； can by response 2xx error。
///
/// ：
/// - ****（`retry_advice`）：provider （workbuddy 11128），
/// need not need 、Reason， by （module header）；
/// - ** 1 **（[`transient_retry_advice`] / [`transport_retry_advice`]）：
/// when HTTP and failure by 1 set —— this is 「set
/// for 」 ， no can to 1 config。
///
/// ── Whyrequest、 not ─────────────────
/// is **request** （ 1 11128 will 3 ）， before 
/// can 「」 —— request that 1 request up not 
/// （ not `attempts`， not ）。 in by 
/// `note_attempt_retry` ** that ** down ，request 1 
/// can to 「Why」； 1 （`console_line`），
/// use when when to 。
///
/// ── `budget` Why need `&mut`（ this 1 ）────────
/// 「 1 N 」 is **request use 1 **： by `attempt_queue`
/// in by `RetrySettings::resend_budget()` ， 1 1 。
/// is in 1 up —— that use ， after 
/// 1 （ not ）， this is 「 1 3 」 this has 。
///
/// and 「 can 」 is ： that `attempt_queue` `switches_left`
/// （ [`take_switch`]）， not 。
async fn send_with_retry(
    adapter: &dyn ProviderAdapter,
    transport: &TransportRequest,
    budget: &mut RetryBudget,
    capture: Option<&crate::server::core::debug_traffic::TrafficCapture>,
    telemetry: &std::sync::Arc<crate::server::core::upstream::usage::RequestTelemetry>,
    degraded: bool,
    images: Option<crate::server::core::providers::images::ImageResponseKind>,
    network: Option<&crate::server::core::providers::policy::NetworkPolicy>,
) -> Result<super::images::ImageResponse, OutboundFailure> {
    let mut network = super::network_fallback::Fallback::new(transport, if images.is_some() { None } else { network });
    loop {
        // ： before （ / up 1 failure after to this ）。
        // is `cancelled_failure`， use （attempt_queue） in before 
        // will 1 and return —— this class is return。
        if telemetry.is_cancelled() {
            return Err(cancelled_failure());
        }
        // ： this 1 down is **** up request when → etc response。
        // Why not can that 1 （`note_attempt_started`）： 
        // （ not ）， is 「 in 」，
        // when must —— otherwise that 1 etc when will 
        // 「 in 」， its already in etc up 。
        telemetry.note_phase(LogPhase::Waiting);
        if let Some(capture) = capture {
            let actual = network.transport();
            if let Ok(body) = serde_json::from_str::<Value>(&actual.payload) {
                capture.reset_request(&actual.url, adapter.provider_id(), &actual.headers, &body);
            }
        }
        let response = match send_or_cancel(&mut network, telemetry).await {
            Ok(response) => {
                telemetry.note_attempt_response();
                response
            }
            Err(error) => {
                // ： not failure（Reason not is ）
                if telemetry.is_cancelled() {
                    return Err(cancelled_failure());
                }
                if images.is_some() && !error.safe_to_retry {
                    telemetry.note_attempt_notice("生图请求可能已发送，未自动重发，以免重复消耗额度");
                    let gateway = super::images::output_error(format!(
                        "{}；上游可能已消耗生图额度，未自动重试", error.message,
                    ));
                    return Err(OutboundFailure {
                        class: UpstreamErrorClass::Fatal {
                            status: 502, message: gateway.message.clone(), upstream_code: None,
                        },
                        error: gateway,
                        stop: true,
                        policy: Default::default(),
                    });
                }
                // failure（DNS//）： by set，；
                // use 502， and before 1 
                if let Some(reason) = network.after_transport(&error).await {
                    telemetry.note_attempt_retry(&reason, None, 0);
                    continue;
                }
                if let Some(advice) = transport_retry_advice(&error, budget.remaining) {
                    budget.remaining -= 1;
                    let used = budget.used();
                    telemetry.note_attempt_retry(&advice.reason, None, advice.delay_ms);
                    logging::console_line(
                        "[Upstream]",
                        &retry_log_line(&advice.reason, advice.delay_ms, used, budget.total),
                    );
                    sleep_or_cancel(telemetry, advice.delay_ms).await;
                    continue;
                }
                let gateway = error.to_gateway_error();
                return Err(OutboundFailure {
                    class: UpstreamErrorClass::Fatal {
                        status: 502,
                        message: gateway.message.clone(),
                        upstream_code: None,
                    },
                    error: gateway,
                    stop: false,
                    policy: Default::default(),
                });
            }
        };
        if let Some(kind) = images.filter(|_| response.status().is_success()) {
            match super::images::collect(response, kind, transport, telemetry).await {
                Ok(body) => return Ok(super::images::ImageResponse::Complete(body)),
                Err(failure) => {
                    let message = failure.body.get("message").and_then(Value::as_str).unwrap_or("图片生成失败");
                    let class = if failure.body.get("code").and_then(Value::as_str) == Some("content_policy_violation") {
                        UpstreamErrorClass::ContentBlocked {
                            status: 400, message: message.to_string(), upstream_code: None,
                        }
                    } else {
                        adapter.classify_error_async(failure.status, &failure.body).await
                    };
                    let error = match &class {
                        UpstreamErrorClass::QuotaLimited { status, message, upstream_code, .. }
                        | UpstreamErrorClass::ContentBlocked { status, message, upstream_code }
                        | UpstreamErrorClass::Fatal { status, message, upstream_code } => {
                            GatewayError::with_status(i32::from(*status).max(400), message.clone())
                                .with_optional_code(*upstream_code)
                        }
                        UpstreamErrorClass::TokenExpired { message } => GatewayError::with_status(401, message.clone()),
                    };
                    let stop = failure.status >= 500
                        && matches!(class, UpstreamErrorClass::Fatal { .. });
                    if stop {
                        telemetry.note_attempt_notice("生图结果不确定，未自动重发，以免重复消耗额度");
                    }
                    return Err(OutboundFailure {
                        class,
                        error: if error.message.is_empty() { super::images::output_error(message) } else { error },
                        stop,
                        policy: Default::default(),
                    });
                }
            }
        }
        let is_error = adapter.is_error_response_async(
            response.status().as_u16(), response.headers(),
        ).await.map_err(|error| OutboundFailure {
            class: UpstreamErrorClass::Fatal {
                status: error.status_code as u16,
                message: error.message.clone(),
                upstream_code: error.upstream_code,
            },
            error,
            stop: true,
            policy: Default::default(),
        })?;
        if !is_error {
            network.success();
            return Ok(super::images::ImageResponse::Live(response));
        }
        let status = response.status().as_u16();
        // errorresponse 「response when 」（ for OmniProxy
        // readBodyWithStallGuard use 1 ）： up errorresponse not 
        // when ， not can 「error」 request —— not up no to ，
        // by （classify_error status can to ）。
        let detail = {
            let budget = Duration::from_millis(config::timeout_settings().body_ms());
            match tokio::time::timeout(budget, read_upstream_error(response, capture)).await {
                Ok(detail) => detail,
                Err(_elapsed) => super::request::UpstreamErrorDetail {
                    code: None,
                    message: format!("非流式响应超时({}秒)", budget.as_secs()),
                    payload: None,
                },
            }
        };
        let body = detail.to_value();
        let class = adapter.classify_error_async(status, &body).await;
        let policy = adapter.error_policy_async(status, &body).await;
        let decision = adapter
            .retry_decision_async(status, &body, budget.used(), budget.total)
            .await;
        let uncertain_image = images.is_some()
            && matches!(class, UpstreamErrorClass::Fatal { .. })
            && (status >= 500 || matches!(status, 408 | 409 | 425));
        let stop = policy.stop || uncertain_image || matches!(&decision, RetryDecision::Stop { .. });
        if let Some(reason) = network.after_status(status, &body,
            !stop && !policy.suspend_account && policy.account_cooldown_until.is_none()
                && matches!(class, UpstreamErrorClass::Fatal { .. })).await {
            telemetry.note_attempt_retry(&reason, Some(i64::from(status)), 0);
            continue;
        }
        if uncertain_image {
            telemetry.note_attempt_notice("图片请求结果不确定，未自动重发，以免重复消耗额度");
        }
        if let RetryDecision::Stop { reason } = &decision {
            telemetry.note_attempt_notice(reason);
            logging::console_line("[Upstream]", reason);
        }
        // ：provider ， no when for when 1 
        //
        // ── Why in `!degraded` when this 1 ──────────────
        // when ， is 「 1 in 」（ use 
        // 0）—— 1 、 ** 1 ** body 1 will 1 
        // （ by ， no not will ），。
        // after （`degraded`） to has ： by 
        // （11-128 「 will 1 when 」 is ）， not 。
        //
        // ── 「error」Why is ────────────────────
        // use （default 402）「 1 」 not ： 1 body
        // not 。 this return None will down errorpath， not 
        // —— that by ： in failure not in 
        // up ， down 1 （ `direct_switch_status` and 3），
        // failure error to client。 is 「 this 」 
        // 。
        let advice = if stop || policy.suspend_account || policy.account_cooldown_until.is_some()
            || (!degraded && matches!(class, UpstreamErrorClass::ContentBlocked { .. }))
            || direct_switch_status(i64::from(status))
        {
            None
        } else {
            match decision {
                RetryDecision::Retry { delay_ms, reason } => Some(RetryAdvice { delay_ms, reason }),
                RetryDecision::Default => transient_retry_advice(status, budget.remaining)
                    .or_else(|| fallback_retry_advice(&class, budget.remaining, status)),
                RetryDecision::Stop { .. } => None,
            }
        };
        if let Some(advice) = advice {
            budget.remaining = budget.remaining.saturating_sub(1);
            let used = budget.used();
            telemetry.note_attempt_retry(&advice.reason, Some(i64::from(status)), advice.delay_ms);
            logging::console_line(
                "[Upstream]",
                &retry_log_line(&advice.reason, advice.delay_ms, used, budget.total),
            );
            sleep_or_cancel(telemetry, advice.delay_ms).await;
            continue;
        }
        // up error： this 1 in ****。request that by 
        // `error`（ 1 message）， not 1 。
        logging::console_line(
            "[Upstream]",
            &format!("上游错误 HTTP {status}: {}", detail.message),
        );
        // by to （ provider ），error
        let error = match &class {
            UpstreamErrorClass::QuotaLimited { status, message, upstream_code, .. } => {
                GatewayError::with_status(*status as i32, message.clone())
                    .with_optional_code(*upstream_code)
            }
            // and Fatal client（ + up + up ）：
            // in ****（ before not 、）， not in 。
            UpstreamErrorClass::ContentBlocked { status, message, upstream_code } => {
                GatewayError::with_status(*status as i32, message.clone())
                    .with_optional_code(*upstream_code)
            }
            UpstreamErrorClass::Fatal { status, message, upstream_code } => {
                GatewayError::with_status(*status as i32, message.clone())
                    .with_optional_code(*upstream_code)
            }
            UpstreamErrorClass::TokenExpired { message } => {
                GatewayError::with_status(401, message.clone())
                    .with_optional_code(detail.code)
            }
        };
        let error = if uncertain_image {
            GatewayError::with_status(error.status_code, format!("{}；图片请求可能已消耗额度，未自动重试", error.message))
                .with_optional_code(error.upstream_code)
        } else { error };
        return Err(OutboundFailure { class, error, stop, policy });
    }
}

/// to client when error。
///
/// ── Whyquota is 429 is 502 ───────────────────────
/// quota：** use ** —— none use ， etc after 1 request
/// can 。 this is 429 + `Retry-After` （ and Trae 
/// `SoftRate | PlanLimit → 429` 1 ），client not is 。
/// ：** this up **， 1 will ； 429 etc client by 
/// 1 —— that is 。 502， is 「 or 」。
///
/// `reset_at > 0` `Retry-After`： not when not to （
/// `GatewayError::with_retry_after_seconds` ）。
fn quota_refusal_error(client_text: String, reset_at: i64, is_quota: bool) -> GatewayError {
    if !is_quota {
        return GatewayError::with_status(502, client_text);
    }
    let error = GatewayError::with_status(429, client_text);
    let seconds = (reset_at - logging::now_ms()) / 1000;
    if seconds > 0 {
        error.with_retry_after_seconds(seconds as u64)
    } else {
        error
    }
}

#[cfg(test)]
mod local_failure {
    //! 「 not 」 this 1 in 。
    //!
    //! （failure → down 1 ） need and ， to 
    //! for （NAS up -14 and -15 `attempts` is ）。 this is 
    //! 。

    use super::readiness;
    use super::TRANSIENT_RETRY_STATUSES;
    use super::direct_switch_status;

    #[test]
    fn a_local_failure_is_never_resent_in_place() {
        // 556 not in when ： and 1 to 1 failure
        // （ not will as etc 5 seconds has ）， 1 need client etc 1 。
        assert!(
            !TRANSIENT_RETRY_STATUSES.contains(&readiness::LOCAL_PRECONDITION_STATUS),
            "本地前置条件的出路只有换号一条"
        );
        // for ： up 503 —— this empty
        assert!(TRANSIENT_RETRY_STATUSES.contains(&503));
    }

    #[test]
    fn the_local_status_is_not_a_user_configurable_switch_code() {
        // 「error」 is use （default 402）。556 not this ：
        // use empty not failure， use 556 not this has 。
        // this use in is as 「」 this up 。
        assert!(!direct_switch_status(i64::from(readiness::LOCAL_PRECONDITION_STATUS)));
    }

    #[test]
    fn a_quota_refusal_reaches_the_client_as_429_with_the_wait_it_knows() {
        // quota：client need can ****， 429 +（ already when ）`Retry-After`。
        let now = crate::server::logging::now_ms();
        let error = super::quota_refusal_error("配额拒绝".to_string(), now + 90_000, true);
        assert_eq!(429, error.status_code);
        assert_eq!(Some(90), error.retry_after_secs);
        // not when （0）： is 429，** not ** —— not to etc when 
        let unknown = super::quota_refusal_error("配额拒绝".to_string(), 0, true);
        assert_eq!(429, unknown.status_code);
        assert_eq!(None, unknown.retry_after_secs);
        // already when （ already ）， not to 1 /
        let stale = super::quota_refusal_error("配额拒绝".to_string(), now - 60_000, true);
        assert_eq!(None, stale.retry_after_secs);
    }

    #[test]
    fn a_content_refusal_stays_502_so_clients_do_not_queue_on_the_same_body() {
        // ： 1 will ⇒ not can use 「 etc 1 will 」 client。
        let now = crate::server::logging::now_ms();
        let error = super::quota_refusal_error("内容拒答".to_string(), now + 90_000, false);
        assert_eq!(502, error.status_code);
        assert_eq!(None, error.retry_after_secs, "内容型不该带 Retry-After");
    }
}