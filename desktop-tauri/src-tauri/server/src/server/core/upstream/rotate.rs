//! and 429 （ from mod.rs ，file）。
//!
//! for Node workbuddy-upstream-client.mjs this ：
//! selectTargetAccount （ → / when or by → use 503）
//! withProxyNotice failure when and 
//! requireSession noneavailable when 401
//! markAccountLimited + when 
//! reportLimitEvent 429 up （ 「 A → B」）
//!
//! ── Why is by not is `impl UpstreamService` ──────
//! this none 1 need 「 + / + 」， is for 
//! ， not is as ； `fn(&UpstreamService, ...)`
//! after mod.rs 「 1 」， not no 。
//! as `mod.rs` （`mod rotate;`）， can to 
//! `UpstreamService` has （Rust has for after can ），
//! not need need as 。
//!
//! ── 1 ： = 「 can this that provider」 ──
//! 1 `providers` （`router::route_for_forward` to 、
//! has this ）， by `account.provider ∈ providers` ， by 
//! **** —— in 1 ， use 。
//! 「workbuddy request not will to raccoon 」 this ：raccoon if not 
//! this ， not in `providers` 。 is 
//! `rateLimits[model]`， 1 provider，none need 。
//!
//! ── `model` is ** up **（file 1 need ）──────
//! up that 「 1 provider」 is before ： 1 request
//! in workbuddy and catpaw can can to ** not ** up ，
//! 1 ，「 by 」 has 1 。
//! use [`routing::CooldownKeys`]，（`mark_account_limited`） by use 
//! already —— must，otherwise will in 1 up 、 in 1 
//! up （ that is 2026-09 that 「、request to already 」 ，
//! `routing::CooldownKeys` ）。
//!
//! ── 11128 ───────────────────────────────────────
//! before `request_with_waf_retry` in file（「11128 → 10s/25s」 ）。
//! that is workbuddy ， already 
//! `providers::workbuddy::retry_advice`；**** in 
//! （`provider_loop::send_with_retry`）， as 「、」
//! is （ §4.3）。

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::core::providers::readiness;
use crate::server::core::routing;
use crate::server::errors::{self, GatewayError};
use crate::server::logging;

use super::{account_display, format_reset_text, has_access_token, priority_of, RouteTarget,
            UpstreamService};

/// will ；`account_id` as None when to ** this provider **default。
///
/// no has available when 401（ for Node requireSession，
/// 「node server.mjs --login」 that —— Rust no has that ）。
///
/// `provider` is need use provider id：default must to ，
/// otherwise「 has raccoon 」 up workbuddy request will to raccoon 。
pub(super) async fn session_for(
    service: &UpstreamService,
    provider: &str,
    account_id: Option<&str>,
) -> Result<Value, GatewayError> {
    if let Some(id) = account_id {
        let service = service.clone();
        let id = id.to_string();
        let provider = provider.to_string();
        return crate::server::core::blocking::run(move || {
            let unavailable = || GatewayError::with_status(
                401, "指定账号不存在、无可用凭证或不属于当前提供商；未回退到其他账号",
            );
            if service.store.account_provider_of(&id).as_deref() != Some(provider.as_str()) {
                return Err(unavailable());
            }
            let mut entry = service.store.get_session_by_id(&id)
                .filter(|_| service.store.account_provider_of(&id).as_deref() == Some(provider.as_str()))
                .ok_or_else(unavailable)?;
            if let Some(session) = entry.session.as_object_mut() {
                session.insert("id".to_string(), Value::String(entry.id));
            }
            Ok(entry.session)
        }).await?;
    }
    let session = service
        .auth
        .get_current_session_for(provider)
        .await
        .map_err(|error| error.to_gateway_error())?;
    match session {
        Some(session) if has_access_token(&session) => Ok(session),
        _ => Err(GatewayError::with_status(
            401,
            "当前没有可用登录态：请先在桌面端完成登录",
        )),
    }
}

/// request use （ for Node selectTargetAccount，）。
///
/// 1. by set ** by ** 1 （`routing::plan_route`：
/// = ； = request），
/// use 、 not 、 in and already and limit ， 
/// to before —— no has available （ to empty）；
/// 2. down enabled in / already and limit → in 「 and limit」
/// 1 1 ； 1 no has （ to and limit）→
/// by ****（ down ），
/// up 429（ when ）return to client；
/// 3. use （ or not ）→ 503， not to already use / 。
///
/// will （`settings.sticky` ≠ off and `conv_key` empty） in 1 before ：
/// in available when （`plan_route` already to and Reason）。
///
/// = `providers` （module header）。`providers` empty。
///
/// `keys` is request to up ： by ****（ by 
/// `routing::CooldownKeys`）—— request will request not to 、 already 
/// in 。「 that 」 by `resetAt`。
///
/// `pinned` is 「 this 1 use this 」（ use ，
/// [`super::ForwardRequest::pinned_account`]）： 1 ，
/// up that for ** that 1 ** —— in when will in 1 ，
/// this is need （ up 429 and when ， not is 1 ）。
///
/// `conv_key` is will （`route_state::conversation_key` ；empty =
/// not ）。 in will new this use when —— 「 in up 」 when in 
/// this 1 。
///
/// return `RouteTarget::decision` is ****（ + Reason +
/// ）， by use telemetry request ——「 up 1 requestWhy」
/// 。
pub(super) async fn select_target_account(
    service: &UpstreamService,
    providers: &[&str],
    keys: &routing::CooldownKeys<'_>,
    tried_ids: &[String],
    pinned: Option<&str>,
    conv_key: &str,
    request_routing: &super::route_context::RequestRouting,
) -> Result<RouteTarget, GatewayError> {
    let service = service.clone();
    let providers: Vec<String> = providers.iter().map(|id| id.to_string()).collect();
    let model = keys.request_model().to_string();
    let tried_ids = tried_ids.to_vec();
    let pinned = pinned.map(str::to_string);
    let conv_key = conv_key.to_string();
    let request_routing = request_routing.clone();
    crate::server::core::blocking::run(move || {
        let providers: Vec<&str> = providers.iter().map(String::as_str).collect();
        select_target_account_sync(&service, &providers, &routing::CooldownKeys::new(&model),
            &tried_ids, pinned.as_deref(), &conv_key, &request_routing)
    }).await?
}

fn select_target_account_sync(
    service: &UpstreamService,
    providers: &[&str],
    keys: &routing::CooldownKeys<'_>,
    tried_ids: &[String],
    pinned: Option<&str>,
    conv_key: &str,
    request_routing: &super::route_context::RequestRouting,
) -> Result<RouteTarget, GatewayError> {
    let accounts = scoped_accounts(service, providers, keys.request_model(), pinned, Some(request_routing));
    if accounts.is_empty() && request_routing.group.is_some() {
        let reasons: Vec<_> = request_routing.group_projection.blocked.iter().take(3)
            .filter_map(|entry| entry.get("reason").and_then(Value::as_str)).collect();
        return Err(GatewayError::with_status(503, format!("此路由组没有可用成员，未回退全局或默认提供商{}", if reasons.is_empty() {
            String::new()
        } else { format!("：{}", reasons.join("；")) })).with_code("route_empty"));
    }
    if accounts.is_empty() && accounts_in_providers(service, providers, pinned).iter().any(|account| {
        account.get("upstreamCooldownUntil").and_then(Value::as_i64).is_some_and(|until| until > logging::now_ms())
    }) {
        return Err(GatewayError::with_status(429, "候选账号处于上游账号级冷却，请等待恢复后重试")
            .with_code("account_cooling_down"));
    }
    if pinned.is_some_and(|id| !accounts.iter().any(|account| routing::account_id(account) == Some(id))) {
        return Err(GatewayError::with_status(401,
            "指定账号不存在或不属于当前提供商；未回退到队首或默认登录态")
            .with_code("account_scope_mismatch"));
    }
    if accounts.is_empty() && pinned.is_none()
        && request_routing.profile.is_some()
    {
        return Err(GatewayError::with_status(
            503, "当前模型路由没有参与转发的账号，请在路由页检查提供商与账号范围",
        ));
    }
    // this no has → use 1 default（ etc ）
    if accounts.is_empty() {
        return Ok(RouteTarget {
            provider: providers.first().copied().unwrap_or_default().to_string(),
            account_id: None,
            account: None,
            proxy: None,
            priority: None,
            proxy_notice: None,
            decision: None,
        });
    }

    // limit （ not / Token ， `filter_limiter_blocked`），
    // use 
    let (candidates, balance_blocked_ids) = filter_limiter_blocked(&accounts, keys, pinned);

    // in 1 （ is seconds ， connections.rs）：
    // 1 and use 1 ， to 「 in 」 is 1 when 。
    let counts = request_connection_counts(service, tried_ids, request_routing.group.is_some());
    let now = logging::now_ms();
    // ── and （route_state.rs that ， not ）────
    // in this use when —— 「auto 5 minutes no 」
    // when by 。
    let settings = request_routing.settings;
    let ttl = routing::sticky_ttl_ms(settings.sticky);
    let snap = service.route.snapshot_for_model(conv_key, keys.request_model(), now, ttl);
    // in 「request 1 」 before ：429 after use 1 
    // after ，otherwise 1 request will 。
    //
    // ── 「 in 」 not etc 「 this use 」（2026-11-03 ）─────────
    // has ， not can use ： can can in 、 limit
    // down 、 to and limit，or not in by 。 that down `plan_route`
    // will in available，request not to —— when if by 「 has in 」
    // ， not ， 1 will allrequest will from 1 new ，
    // 。
    //
    // ： has in when by 「 not 」 1 （ not ）， has in to 
    // 1 use ；otherwise、 new 1 。 no has in when is path
    // （， 1 ）。 in 「available / / after 」
    // no to 1 —— that 1 request will to up ， by 
    // 。
    //
    // （`Provider`） use and `Rotate` ——「 1 request
    // 1 」 for is 1 ； in is ** 1 **
    // （ `rotate_take_per_provider`）。
    let may_rotate = tried_ids.is_empty() && pinned.is_none();
    let wants_rotation = may_rotate && settings.strategy != config::RouteStrategy::Order;
    let take_rotation = || match settings.strategy {
        config::RouteStrategy::Rotate => (
            Some(service.route.rotate_take(keys.request_model())),
            None,
        ),
        // 「」 use （`provider_sizes`）， ** not and 
        // ** —— in `plan_route` 、 use is （ use / /
        // and / none） after 。 and must use 1 ：
        // 2026-10-14 before this by ， 1 not etc ， will 
        // 「 1 in 」 not 。
        config::RouteStrategy::Provider => (
            None,
            Some(service.route.rotate_take_per_provider(
                keys.request_model(),
                &routing::provider_sizes(&candidates),
            )),
        ),
        config::RouteStrategy::Order => (None, None),
    };
    let (mut cursor, mut provider_cursors) = if wants_rotation && snap.sticky_hit.is_none() {
        take_rotation()
    } else {
        (None, None)
    };
    let mut plan = routing::plan_route(
        &accounts, keys, &counts, tried_ids, now,
        &routing::PlanMeta {
            strategy: settings.strategy,
            sunk: &snap.sunk,
            resting: &snap.resting,
            open: &snap.open,
            limiter_blocked: &balance_blocked_ids,
            cursor,
            provider_cursors: provider_cursors.as_ref(),
            sticky_account: snap.sticky_hit.as_deref(),
        },
    );
    // in no has to 1 ： this 1 request use not up ， 1 。
    let sticky_first = snap
        .sticky_hit
        .as_deref()
        .is_some_and(|hit| plan.ordered.first().is_some_and(|entry| crate::server::core::routing_groups::attempt_key(&entry.account) == hit));
    if wants_rotation && snap.sticky_hit.is_some() && !sticky_first {
        (cursor, provider_cursors) = take_rotation();
        plan = routing::plan_route(
            &accounts, keys, &counts, tried_ids, now,
            &routing::PlanMeta {
                strategy: settings.strategy,
                sunk: &snap.sunk,
                resting: &snap.resting,
                open: &snap.open,
                limiter_blocked: &balance_blocked_ids,
                cursor,
                provider_cursors: provider_cursors.as_ref(),
                sticky_account: snap.sticky_hit.as_deref(),
            },
        );
    }
    let mut decision = decision_json(&plan, &settings, now);
    decision["scope"] = Value::String(request_routing.scope.clone());
    decision["configSnapshot"] = request_routing.json();
    if let Some(blocked) = decision.get_mut("blocked").and_then(Value::as_array_mut) {
        blocked.extend(request_routing.group_projection.blocked.iter().take(12usize.saturating_sub(blocked.len())).cloned());
    }
    decision["blockedTotal"] = (plan.blocked.len() + request_routing.group_projection.blocked.len()).into();
    if !conv_key.is_empty() {
        decision["sessionId"] = Value::String(super::route_sticky::entry_id(conv_key, keys.request_model()));
        if ttl > 0 { decision["stickyId"] = decision["sessionId"].clone(); }
    }
    let mut excluded: Vec<String> = tried_ids.to_vec();
    for entry in &plan.ordered {
        if let Some(picked_session) = service.store.get_session_by_id(&entry.id) {
            return Ok(with_proxy_notice(
                entry.account.clone(),
                picked_session.proxy,
                picked_session.proxy_error,
                entry.id.clone(),
                Some(decision),
            ));
        }
        excluded.push(entry.id.clone());
    }

    let enabled: Vec<Value> = candidates
        .iter()
        .filter(|account| !matches!(account.get("enabled"), Some(Value::Bool(false))))
        .cloned()
        .collect();
    if enabled.is_empty() {
        // after 1 can no has ： is not is limit —— that and 「 use 」
        // is not ， use need not 1 （ etc / etc / enabled）。
        let enabled_total = accounts
            .iter()
            .filter(|account| !matches!(account.get("enabled"), Some(Value::Bool(false))))
            .count();
        if !balance_blocked_ids.is_empty() && enabled_total > 0 {
            return Err(GatewayError::with_status(
                503,
                format!(
                    "所有启用中的账号都被限制器拦下（已跳过 {} 个账号），无账号可转发：请检查余额读数与 Token 周期用量，或到账号设置的「限制器」里调整规则",
                    balance_blocked_ids.len()
                ),
            ));
        }
        return Err(GatewayError::with_status(
            503,
            "所有账号均已禁用，无账号可转发：请在账号页启用至少一个账号",
        ));
    }

    // ── （ and ）──────────────────────────────────
    // enabled in in / already and limit when ， 1 1 ：
    // ① in 「enabled + + and limit」 **** 
    // —— up if already can success，otherwise 429 and when 
    // return to client（ and before 1 ， and this 1 ）；
    // ② 1 no has （ **** to and limit）→ ****
    // 。
    //
    // ── Why ② not is 503（ ）──────────────────
    // is 「 etc 」 ： has when ， to ； and 
    // then is **** —— in request 1 for after （
    // connections.rs Drop）。 when request（503） will can 
    // request in ， 1-2 （limit
    // `routing::max_concurrent_of`）。： can etc etc （①）， etc not （②），
    // not and limit 503。
    let mut best_effort: Option<Value> = None;
    let mut best_reset = f64::INFINITY;
    for account in &enabled {
        let Some(id) = routing::account_id(account) else {
            continue;
        };
        if tried_ids.iter().any(|tried| tried == &crate::server::core::routing_groups::attempt_key(account)) {
            continue;
        }
        // already and limit not 「」 —— is ② 
        let limit = routing::max_concurrent_of(account);
        if limit > 0 && counts.get(id).copied().unwrap_or(0) >= limit as usize {
            continue;
        }
        let reset = routing::rate_limit_reset_at(account, keys, now);
        let reset = if reset > 0 { reset as f64 } else { f64::INFINITY };
        if reset < best_reset {
            best_reset = reset;
            best_effort = Some(account.clone());
        }
    }
    if best_effort.is_none() {
        // ① none： in 「 to and limit」（ or no has ） when by 。
        // and ① 1 （ will → ）。
        if let Some(account) = squeeze_by_headroom(&enabled, &counts, tried_ids, keys, now) {
            if let Some(id) = routing::account_id(&account).map(str::to_string) {
                if let Some(entry) = service.store.get_session_by_id(&id) {
                    return Ok(with_proxy_notice(
                        account,
                        entry.proxy,
                        entry.proxy_error,
                        id,
                        Some(decision),
                    ));
                }
            }
        }
    }
    if let Some(account) = best_effort {
        if let Some(id) = routing::account_id(&account).map(str::to_string) {
            if let Some(entry) = service.store.get_session_by_id(&id) {
                return Ok(with_proxy_notice(
                    account,
                    entry.proxy,
                    entry.proxy_error,
                    id,
                    Some(decision),
                ));
            }
        }
    }
    if pinned.is_some() {
        return Err(GatewayError::with_status(401,
            "指定账号没有可用凭证；未回退到队首或默认登录态")
            .with_code("account_scope_mismatch"));
    }
    if request_routing.profile.is_some() {
        return Err(GatewayError::with_status(
            503, "当前模型路由中的账号都没有可用凭证，请在路由页检查账号状态",
        ));
    }
    decision["picked"] = Value::String(String::new());
    Ok(RouteTarget {
        provider: providers.first().copied().unwrap_or_default().to_string(),
        account_id: None,
        account: None,
        proxy: None,
        priority: None,
        proxy_notice: None,
        decision: Some(decision),
    })
}

/// up provider id（ by default provider）。
/// and in `routing` —— that need by provider （`pick_for_model`），
/// 1 use ，「default」 this then 、 after 1 。
pub(super) use crate::server::core::routing::provider_of;

/// （request → up ）。
///
/// and `provider_of` 1 ： and in `routing`， this ——
/// （`provider_loop`） need 、 need ，path
/// will ； is **** 1 ， not this in 1 。
pub(super) use crate::server::core::routing::CooldownKeys;

/// ：`providers` （，file）；
/// `pinned` empty when ** that 1 **（，
/// [`select_target_account`] for this parameter ）。
///
/// by and ； use ， use 。
/// ， not 。
///
/// 「」， not not ： not in this （ 、
/// or ） when to is empty —— up that path will to 「 this no has 」
/// has ， not will 1 。
///
/// ── in this ────────────────────────────
/// `readiness` provider not （ and by that module header：
/// that is 1 ** no failure** ，e.g. ZCode in headless up no has 
/// ）。 in this not is in `pick_account_by_priority` ， is as 
/// （ → → by ） is this 1 —— in down 1 ，
/// not ，、 is 「 can can failure need 1 」 。
///
/// ** when old return**： is ， not is 。if `providers` 1 
/// （ or this that 1 ）， can request 1 、 error
/// in this up ， not need in 「 this no has 」—— that will 
/// not in ， use will 1 has 。
pub(super) fn accounts_in_providers(
    service: &UpstreamService,
    providers: &[&str],
    pinned: Option<&str>,
) -> Vec<Value> {
    let snapshot = service.store.routing_snapshot();
    snapshot.accounts.iter()
        .filter(|account| crate::server::core::account_store::store_view::account_visible(provider_of(account)))
        .filter(|account| providers.contains(&provider_of(account)))
        .filter(|account| match pinned {
            Some(id) => routing::account_id(account) == Some(id),
            None => true,
        })
        .map(|account| account.as_ref().clone())
        .collect()
}

fn scoped_accounts(
    service: &UpstreamService,
    providers: &[&str],
    model: &str,
    pinned: Option<&str>,
    request_routing: Option<&super::route_context::RequestRouting>,
) -> Vec<Value> {
    let accounts = accounts_in_providers(service, providers, pinned);
    let accounts = if pinned.is_some() { accounts } else if let Some(request_routing) = request_routing {
        request_routing.accounts(accounts)
    } else {
        crate::server::core::routing_scope::apply_accounts(&accounts, model)
    };
    let accounts: Vec<Value> = accounts.into_iter().filter(|account| {
        pinned.is_some() || !account.get("upstreamCooldownUntil").and_then(Value::as_i64)
            .is_some_and(|until| until > logging::now_ms())
    }).collect();
    let routable: Vec<Value> = accounts.iter()
        .filter(|account| !readiness::gated(provider_of(account)))
        .cloned().collect();
    if routable.is_empty() { accounts } else { routable }
}

/// ；failure when ****（）。
///
/// ── Why is return 1 not is ────────────────
/// before this `logging::log` 1 。 that is **request** ：
/// up request will 1 ，request that not to 「 this its is 」。
/// in in `RouteTarget::proxy_notice` up ， by 
/// （ that ）， not 。
///
/// and 1 ： `proxyError`（ up failure Reason）
/// and failure Reason。** after ** —— is 「 this Why」，
/// `proxyError` can can is old 。
pub(super) fn with_proxy_notice(
    account: Value,
    proxy: Value,
    proxy_error: Option<String>,
    account_id: String,
    decision: Option<Value>,
) -> RouteTarget {
    let mut notice = proxy_error.map(|error| {
        format!(
            "账号「{}」代理不可用，本次直连: {error}",
            account_display(&account)
        )
    });
    let resolved = match ResolvedProxy::from_json(&proxy) {
        Ok(proxy) => proxy,
        Err(reason) => {
            // will proxy by （success will ），
            // this failure in ： by 
            notice = Some(format!("账号代理不可用（{reason}），本次回退直连"));
            None
        }
    };
    let decision = decision.map(|mut decision| {
        if let Some(object) = decision.as_object_mut() {
            object.insert("picked".to_string(), Value::String(account_id.clone()));
            object.insert("pickedCandidate".to_string(), crate::server::core::routing_groups::attempt_key(&account).into());
        }
        decision
    });
    RouteTarget {
        provider: provider_of(&account).to_string(),
        account_id: Some(account_id),
        priority: priority_of(&account),
        account: Some(account),
        proxy: resolved,
        proxy_notice: notice,
        decision,
    }
}

/// and return can when （ for Node markAccountLimited）。
///
/// `account_id` already 1 provider（id in file 1 、 and 
/// 1 ）， = `provider××` 。
///
/// `model` must is ** up to **（ not is clientrequest）—— use from 
/// `SendBody::wire_model` （ `upstream::payload`）。request will in 
/// up not will up ， not to （ `routing::CooldownKeys`）。
pub(super) fn mark_account_limited(
    service: &UpstreamService,
    account_id: &str,
    model: &str,
    status: i32,
    upstream_code: Option<i64>,
    reset_at: Option<i64>,
    message: &str,
) -> String {
    // Node from `error.body` 1 msg when ；Rust error
    // already up up ，`reset_at` by classify_error after ，
    // this 1 （path 1 ： 1 up ）
    let parsed = if let Some(reset_at) = reset_at.filter(|value| *value > 0) {
        reset_at
    } else {
        errors::parse_quota_reset_at(message)
    };
    let entry = crate::server::core::blocking::in_place(|| service.store.mark_rate_limited(
        account_id,
        model,
        status as i64,
        upstream_code,
        if parsed > 0 { Some(parsed as f64) } else { None },
        message,
    ));
    match entry {
        Some(entry) => {
            let reset = entry.get("resetAt").and_then(Value::as_f64).unwrap_or(0.0);
            format_reset_text(reset)
        }
        None => String::new(),
    }
}

/// this for current when （ 0）—— up when use ，
/// and Node `limitEntry?.resetAt` （ from ）。
///
/// `model` is up （ `mark_account_limited` ）， and 1 。
pub(super) fn account_limit_reset_at(service: &UpstreamService, account_id: &str, model: &str) -> i64 {
    service.store.route_account(account_id)
        .map(|account| {
            account
                .get("rateLimits")
                .and_then(|limits| limits.get(model))
                .and_then(|limit| limit.get("resetAt"))
                .and_then(Value::as_f64)
                .map(|value| value as i64)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

/// in **** when （ms； 1 no has when 0）。
///
/// ── use ： 「 need etc 」client ────────────────────────
/// all when ， to client error up this when 
/// （`provider_loop` 「 already or use 」）。 not client can 
/// 、 1 ； up after （Claude Code / Codex this client will 
/// when ） can ** etc **。
///
/// ── Why**** not is ─────────────────────────
/// 「」= 1 can can 、 when 。 will client 1 
/// not when （ is ， not is when ）。
///
/// ── ─────────────────────────────────
/// ， down already new ； not use by 。
///
/// `providers` is request ； by [`routing::CooldownKeys`] 
/// （ and 1 ）。**enabled in **，including already that ——
/// ， when is client this etc that 。
///
/// `accounts_in_providers` by provider 、** not use **（ that is 
/// ）， this 1 ： 1 use 
/// not ， when will use to 1 not will 。
pub(super) fn earliest_limit_reset_at(
    service: &UpstreamService,
    providers: &[&str],
    keys: &routing::CooldownKeys<'_>,
) -> i64 {
    let now = logging::now_ms();
    let accounts: Vec<Value> = scoped_accounts(service, providers, keys.request_model(), None, None)
        .into_iter()
        .filter(|account| !matches!(account.get("enabled"), Some(Value::Bool(false))))
        .collect();
    earliest_limit_reset_at_in(&accounts, keys, now)
}

/// [`earliest_limit_reset_at`] （ and 「 in 」 by use to ，
/// ）。
///
/// not ：`rate_limit_reset_at` **** when or 0
/// （ that ）， this for 。
fn earliest_limit_reset_at_in(
    accounts: &[Value],
    keys: &routing::CooldownKeys<'_>,
    now: i64,
) -> i64 {
    accounts
        .iter()
        .map(|account| routing::rate_limit_reset_at(account, keys, now))
        .filter(|reset| *reset > 0)
        .min()
        .unwrap_or(0)
}

/// to and limit when ****：（`limit - count`）。
///
/// ── （， before ）──────────────────────────
/// · ** not （maxConcurrent == 0）= none**， in has limit before ——
/// use `i64::MAX` ： `limit - count` ；
/// · **** （ and ① 「 1 」 1 ：
/// 「 can can up 」 1 ）；
/// · `>` （ when when ）。
///
/// in 1 verbose（verbose not ， in / when can 
/// —— this is request ，info will ， when must to 「Why
/// limit to this 」 ）。 no has can （ already ） when 
/// return None， by use to 「none」。
fn squeeze_by_headroom(
    enabled: &[Value],
    counts: &HashMap<String, usize>,
    tried_ids: &[String],
    keys: &routing::CooldownKeys<'_>,
    now: i64,
) -> Option<Value> {
    let mut squeezed: Option<Value> = None;
    let mut best_headroom = i64::MIN;
    let mut best_reset = f64::INFINITY;
    for account in enabled {
        let Some(id) = routing::account_id(account) else {
            continue;
        };
        if tried_ids.iter().any(|tried| tried == &crate::server::core::routing_groups::attempt_key(account)) {
            continue;
        }
        let count = counts.get(id).copied().unwrap_or(0) as i64;
        let limit = routing::max_concurrent_of(account) as i64;
        let headroom = if limit > 0 { limit - count } else { i64::MAX };
        let reset = routing::rate_limit_reset_at(account, keys, now);
        let reset = if reset > 0 { reset as f64 } else { f64::INFINITY };
        if headroom > best_headroom || (headroom == best_headroom && reset < best_reset) {
            best_headroom = headroom;
            best_reset = reset;
            squeezed = Some(account.clone());
        }
    }
    if let Some(account) = &squeezed {
        logging::verbose(
            "[Upstream]",
            &format!(
                "全部账号达到并发上限，按余量挤占账号「{}」",
                account_display(account)
            ),
        );
    }
    squeezed
}

/// current in request （ id → request）， and use 。
///
/// from `Connections` 1 （`snapshot` ，seconds）。
/// not to when use **empty** = not and （`pick_for_model`
/// use `api::session` has ，；`describe_route_decision`
/// ，empty）。
pub(super) fn connection_counts(service: &UpstreamService) -> HashMap<String, usize> {
    service.connections().snapshot().into_iter().collect()
}

fn request_connection_counts(service: &UpstreamService, tried: &[String], group: bool) -> HashMap<String, usize> {
    let mut counts = connection_counts(service);
    // when currentrequest has before 1 ； 1 not can this 1 。
    if group {
        if let Some(id) = tried.last().map(|key| crate::server::core::routing_groups::account_key(key)) {
            if let Some(count) = counts.get_mut(id) { *count = count.saturating_sub(1); }
        }
    }
    counts
}

pub(super) fn allows_route_skip(service: &UpstreamService, account_id: &str) -> bool {
    !service.store.route_account(crate::server::core::routing_groups::account_key(account_id))
        .is_some_and(|account| account.get("allowRouteSkip") == Some(&Value::Bool(false)))
}

/// down 1 available（， in `providers` ， already ）。
///
/// `keys` [`select_target_account`]： by 。
/// and 1 ** 1 ** `routing::plan_route`（ 1 、 1 
/// ， is not 、 not before ）—— 429 / failure 
/// 「 down 1 」 and 1 ； and limit by ，
/// not will request 1 already limit 。
///
/// `pinned` [`select_target_account`]： when ， has that 1 、
/// already in `tried_ids` —— return None， is 「 not 」。
/// this is ： is 「 this not 」， will 。
pub(super) fn pick_next_account(
    service: &UpstreamService,
    providers: &[&str],
    keys: &routing::CooldownKeys<'_>,
    tried_ids: &[String],
    pinned: Option<&str>,
    request_routing: &super::route_context::RequestRouting,
) -> Option<Value> {
    if tried_ids.last().is_some_and(|last| !allows_route_skip(service, last)) {
        return None;
    }
    let accounts = scoped_accounts(service, providers, keys.request_model(), pinned, Some(request_routing));
    // and 1 1 ： limit down not will in 429 after 
    // 「 down 1 」 new 
    let (candidates, balance_blocked_ids) = filter_limiter_blocked(&accounts, keys, pinned);
    let counts = request_connection_counts(service, tried_ids, request_routing.group.is_some());
    let now = logging::now_ms();
    let settings = request_routing.settings;
    // not （ is 「 for 」，failure that not this ）；
    // empty will sticky_lookup return None， its （ / / ）
    // and 
    let snap = service.route.snapshot("", now, 0);
    let meta = routing::PlanMeta {
        strategy: settings.strategy,
        sunk: &snap.sunk,
        resting: &snap.resting,
        open: &snap.open,
        limiter_blocked: &balance_blocked_ids,
        cursor: None,
        provider_cursors: None,
        sticky_account: None,
    };
    let plan = routing::plan_route(&candidates, keys, &counts, tried_ids, now, &meta);
    plan.ordered.into_iter().next().map(|entry| entry.account)
}

/// （request「 by 」 ）。
///
/// has ： and 12 —— when up will 
/// request， before 12 already 「Why is 、Why not is that 」。
fn decision_json(
    plan: &routing::RoutePlan,
    settings: &config::RoutingSettings,
    now: i64,
) -> Value {
    let _ = now;
    let entry_json = |entry: &routing::PlanEntry| {
        json!({
            "id": entry.id,
            "name": entry.name,
            "provider": entry.provider,
            "candidateId": crate::server::core::routing_groups::attempt_key(&entry.account),
            "memberId": entry.account.get("routingMemberId"),
            "model": entry.account.get("routingModel"),
            "state": entry.state,
            "reason": entry.reason,
        })
    };
    json!({
        "strategy": settings.strategy.as_str(),
        "sticky": settings.sticky.as_str(),
        "stickyHit": plan.sticky_hit,
        "picked": plan.ordered.first().map(|entry| entry.id.clone()).unwrap_or_default(),
        "ordered": plan.ordered.iter().take(12).map(entry_json).collect::<Vec<_>>(),
        "blocked": plan.blocked.iter().take(12).map(entry_json).collect::<Vec<_>>(),
        "orderedTotal": plan.ordered.len(),
        "blockedTotal": plan.blocked.len(),
    })
}

/// limit ：** not **（limit skip then in ：
/// ） or **Token **（Token skip then in ：current already 
/// ） in ** before ** —— （ / etc /
/// and ） use this 1 。 and not ， is 「 etc not 」 or 「 etc no use 」 ：
/// has when 、 to ， 1 can 429 ； not 
/// will up 402 / 403，Token is in then for 。
///
/// ****（：`usage_records::balance_facts`；Token：
/// `core::limiter::token_facts`， 10 seconds new + request）——
/// IO，seconds。 after down 1 new 、Token after down 1 
/// new ， and ，none need 。
///
/// `pinned`（） not ： need is ** that ** —— limit when 
/// up ， and 「 in will 」 1 。
/// return `( after , id )`： use in 「 1 can no has 」 when 
/// use after 503 Reason（limit vs use ）。
fn filter_limiter_blocked(
    accounts: &[Value],
    keys: &routing::CooldownKeys<'_>,
    pinned: Option<&str>,
) -> (Vec<Value>, std::collections::HashSet<String>) {
    if pinned.is_some() {
        return (accounts.to_vec(), Default::default());
    }
    let facts = crate::server::core::limiter::sources::facts();
    let counts = crate::server::core::limiter::counter::facts();
    let now = logging::now_ms();
    let blocked_ids: std::collections::HashSet<String> = accounts
        .iter()
        .filter(|account| crate::server::core::limiter::evaluate::blocked(
            account, &keys.for_account(account), &facts, &counts, now,
        ))
        .map(crate::server::core::routing_groups::attempt_key)
        .collect();
    if blocked_ids.is_empty() {
        return (accounts.to_vec(), blocked_ids);
    }
    let candidates = accounts
        .iter()
        .filter(|account| {
            !blocked_ids.contains(&crate::server::core::routing_groups::attempt_key(account))
        })
        .cloned()
        .collect();
    (candidates, blocked_ids)
}

/// 429 up to （ for Node reportLimitEvent）。
///
/// data before can 「 A → B」 and when 
/// （logs-panel `data.from` / `data.to` / `data.resetAtText`）。
/// `priority` is **** （Node in `next.priority`，
/// its is `?? null`）。
///
/// `provider` data（ after 1 need can is 1 in ）。
/// before ， 1 not has 。
#[allow(clippy::too_many_arguments)]
pub(super) fn report_limit_event(
    level: &str,
    message: &str,
    from: Option<&str>,
    to: Option<&str>,
    model: &str,
    upstream_code: Option<i64>,
    status_code: i32,
    reset_at: i64,
    priority: Option<i64>,
    provider: &str,
) {
    logging::log_event(
        level,
        "account",
        message,
        Some(json!({
            "model": model,
            "provider": provider,
            "from": from.unwrap_or(""),
            "to": to.unwrap_or(""),
            "priority": priority.map(Value::from).unwrap_or(Value::Null),
            "status": if status_code == 0 { 429 } else { status_code },
            "code": upstream_code.map(Value::from).unwrap_or(Value::Null),
            "resetAt": reset_at,
            "resetAtText": if reset_at > 0 { format_reset_text(reset_at as f64) } else { String::new() },
        })),
    );
}

#[cfg(test)]
mod tests {
    use super::earliest_limit_reset_at_in;
    use crate::server::core::routing::CooldownKeys;
    use serde_json::json;

    /// 1 （ and `mark_rate_limited` 1 ）
    fn limited(id: &str, provider: &str, model: &str, reset_at: i64) -> serde_json::Value {
        json!({
            "id": id,
            "provider": provider,
            "enabled": true,
            "rateLimits": { model: { "status": 429, "resetAt": reset_at } },
        })
    }

    fn unlimited(id: &str, provider: &str) -> serde_json::Value {
        json!({ "id": id, "provider": provider, "enabled": true })
    }

    /// **** when （ not is ）：client this to 1 can can 
    #[test]
    fn the_earliest_reset_wins() {
        let keys = CooldownKeys::new("Qwen/Qwen3.8-Flash-Next");
        let now = 1_000_000i64;
        let accounts = vec![
            limited("a", "modelscope", "Qwen/Qwen3.8-Flash-Next", now + 600_000),
            limited("b", "modelscope", "Qwen/Qwen3.8-Flash-Next", now + 60_000),
            limited("c", "modelscope", "Qwen/Qwen3.8-Flash-Next", now + 300_000),
        ];
        assert_eq!(earliest_limit_reset_at_in(&accounts, &keys, now), now + 60_000);
    }

    /// no has in → 0（ use ， not 「…」）
    #[test]
    fn no_future_reset_reports_zero() {
        let keys = CooldownKeys::new("m");
        let now = 1_000_000i64;
        assert_eq!(earliest_limit_reset_at_in(&[], &keys, now), 0);
        assert_eq!(
            earliest_limit_reset_at_in(&[unlimited("a", "modelscope")], &keys, now),
            0
        );
    }

    /// **expired** not （ in when already = ， and 
    /// `rate_limit_reset_at` 1 ）—— will 1 when 
    #[test]
    fn an_expired_cooldown_is_not_counted() {
        let keys = CooldownKeys::new("m");
        let now = 1_000_000i64;
        let accounts = vec![
            limited("a", "modelscope", "m", now - 1),
            limited("b", "modelscope", "m", now),
        ];
        assert_eq!(earliest_limit_reset_at_in(&accounts, &keys, now), 0);
    }

    /// ** by **： not this 「 need etc 」
    #[test]
    fn another_models_cooldown_is_ignored() {
        let keys = CooldownKeys::new("m");
        let now = 1_000_000i64;
        let accounts = vec![limited("a", "modelscope", "other-model", now + 600_000)];
        assert_eq!(earliest_limit_reset_at_in(&accounts, &keys, now), 0);
    }

    /// when （ this is 「 min not is 1 」）
    #[test]
    fn the_minimum_spans_all_accounts() {
        let keys = CooldownKeys::new("m");
        let now = 1_000_000i64;
        let accounts = vec![
            limited("a", "modelscope", "m", now + 900_000),
            unlimited("b", "modelscope"),
            limited("c", "modelscope", "m", now + 120_000),
            limited("d", "modelscope-intl", "m", now + 500_000),
        ];
        assert_eq!(earliest_limit_reset_at_in(&accounts, &keys, now), now + 120_000);
    }
}
