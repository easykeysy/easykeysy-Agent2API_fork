//! GET/PUT /api/routing + GET /api/routing/queue —— by set and ** can **。
//!
//! ── /api/routing（ and /api/timeouts 1 ）─────────────────
//! 、、return after 、none use （ not ）。
//! after for down 1 request（request `config::routing_settings()`
//! ， not ）。
//!
//! ── /api/routing/queue（ can ， ）────────
//! 「 by current， this will 、 、Why」。
//! ** and is 1 **（`routing::plan_route`）—— that 
//! ， this ； if 1 ， and 
//! （ this is 「 can 」 need that ）。
//!
//! and `/api/session` `routedAccountId` ： that 「 is 」
//! （★ ， not ）， to is ** + Reason** —— need 
//! 1 ， （ use / / and / limit /
//! none has Reason， use not use ）。
//!
//! `model` parameter can ：（ or empty） when by 「 not 」 
//! （ not to 「 this 」）—— in no has up down when has 。

use std::collections::{HashMap, HashSet};

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{self, RoutingPatch, RoutingSettings};
use crate::server::core::providers::router;
use crate::server::core::routing;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::ServerState;

/// GET /api/routing
pub async fn get_routing(State(_state): State<ServerState>) -> Response {
    ok_json(routing_json(config::routing_settings()))
}

/// PUT /api/routing —— body optional（ not to = not ）
pub async fn put_routing(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let mut patch = RoutingPatch::default();
    if let Some(value) = object.get(config::KEY_ROUTING_STRATEGY) {
        let Some(text) = value.as_str() else {
            return errors::management_error(400, "strategy 必须是字符串");
        };
        let known = [
            config::ROUTING_STRATEGY_ORDER,
            config::ROUTING_STRATEGY_ROTATE,
            config::ROUTING_STRATEGY_PROVIDER,
        ];
        if !known.contains(&text) {
            return errors::management_error(
                400,
                format!("未知的路由策略「{text}」（只支持 {}）", known.join(" / ")),
            );
        }
        patch.strategy = Some(config::RouteStrategy::from_id(text));
    }
    if let Some(value) = object.get(config::KEY_ROUTING_STICKY) {
        let Some(text) = value.as_str() else {
            return errors::management_error(400, "sticky 必须是字符串");
        };
        let known = [
            config::STICKY_AUTO,
            config::STICKY_SESSION,
            config::STICKY_TURN,
            config::STICKY_OFF,
        ];
        if !known.contains(&text) {
            return errors::management_error(
                400,
                format!("未知的会话粘性档位「{text}」（只支持 {}）", known.join(" / ")),
            );
        }
        patch.sticky = Some(config::StickyMode::from_id(text));
    }
    let bool_field = |key: &str| -> Result<Option<bool>, Response> {
        match object.get(key) {
            None => Ok(None),
            Some(Value::Bool(value)) => Ok(Some(*value)),
            Some(_) => Err(errors::management_error(400, format!("{key} 必须是布尔值"))),
        }
    };
    let failover_backoff = match bool_field(config::KEY_ROUTING_FAILOVER_BACKOFF) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let sink = match bool_field(config::KEY_ROUTING_SINK) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let breaker_enabled = match bool_field(config::KEY_ROUTING_BREAKER_ENABLED) {
        Ok(value) => value,
        Err(response) => return response,
    };
    patch.failover_backoff = failover_backoff;
    patch.sink = sink;
    patch.breaker_enabled = breaker_enabled;
    let int_field = |key: &str, min: i64, max: i64| -> Result<Option<i64>, Response> {
        match object.get(key) {
            None => Ok(None),
            Some(value) => match value.as_i64() {
                Some(count) if (min..=max).contains(&count) => Ok(Some(count)),
                _ => Err(errors::management_error(
                    400,
                    format!("{key} 必须是 {min}~{max} 的整数"),
                )),
            },
        }
    };
    patch.breaker_threshold = match int_field(
        config::KEY_ROUTING_BREAKER_THRESHOLD,
        config::ROUTING_BREAKER_MIN_THRESHOLD,
        config::ROUTING_BREAKER_MAX_THRESHOLD,
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    patch.breaker_timeout_seconds = match int_field(
        config::KEY_ROUTING_BREAKER_TIMEOUT,
        config::ROUTING_BREAKER_MIN_TIMEOUT,
        config::ROUTING_BREAKER_MAX_TIMEOUT,
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    patch.breaker_success_threshold = match int_field(
        config::KEY_ROUTING_BREAKER_SUCCESS,
        config::ROUTING_BREAKER_MIN_SUCCESS,
        config::ROUTING_BREAKER_MAX_SUCCESS,
    ) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let previous_sticky = config::routing_settings().sticky;
    if !config::set_routing(patch) {
        return errors::management_error(500, "路由设置落盘失败");
    }
    if previous_sticky != config::routing_settings().sticky {
        _state.upstream().clear_route_sticky(None, None);
    }
    ok_json(routing_json(config::routing_settings()))
}

pub(crate) fn routing_json(settings: RoutingSettings) -> Value {
    json!({
        config::KEY_ROUTING_STRATEGY: settings.strategy.as_str(),
        config::KEY_ROUTING_STICKY: settings.sticky.as_str(),
        config::KEY_ROUTING_FAILOVER_BACKOFF: settings.failover_backoff,
        config::KEY_ROUTING_SINK: settings.sink,
        config::KEY_ROUTING_BREAKER_ENABLED: settings.breaker_enabled,
        config::KEY_ROUTING_BREAKER_THRESHOLD: settings.breaker_threshold,
        config::KEY_ROUTING_BREAKER_TIMEOUT: settings.breaker_timeout_seconds,
        config::KEY_ROUTING_BREAKER_SUCCESS: settings.breaker_success_threshold,
    })
}

/// GET /api/routing/queue?model=
pub async fn get_routing_queue(
    State(state): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let model = params
        .get("model")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    ok_json(queue_json(&state, model))
}

pub(crate) fn queue_json(state: &ServerState, model: Option<String>) -> Value {
    let accounts = routing::accounts_of(&state.store().list_visible_accounts());
    queue_for_accounts(state, model, &accounts)
}

pub(crate) fn queue_for_accounts(state: &ServerState, model: Option<String>, accounts: &[Value]) -> Value {
    if let Some(group) = model.as_deref().and_then(|model| config::routing_profiles().group_for_model(model).cloned()) {
        return super::routing_groups::group_queue(state, group);
    }
    let snapshot = config::current();
    // ： → 「 has 」（ and 1 ，
    // `route_for_forward`）； no → 。 and session.rs ★ 1 。
    let providers: Vec<String> = match &model {
        Some(model) => router::route_for_forward(model),
        None => Vec::new(),
    };
    let scoped: Vec<Value> = match model.as_deref() {
        Some(_) => accounts.iter()
            .filter(|account| providers.contains(&routing::provider_of(account).to_string()))
            .cloned().collect(),
        None => accounts.to_vec(),
    };
    let scoped = crate::server::core::routing_scope::apply_accounts(
        &scoped, model.as_deref().unwrap_or(""),
    );
    let routable: Vec<Value> = scoped.iter()
        .filter(|account| !crate::server::core::providers::readiness::gated(routing::provider_of(account)))
        .cloned().collect();
    let scoped = if routable.is_empty() { scoped } else { routable };
    let now = crate::server::logging::now_ms();
    let counts: HashMap<String, usize> = state
        .upstream()
        .connections()
        .snapshot()
        .into_iter()
        .collect();
    let keys = routing::CooldownKeys::new(model.as_deref().unwrap_or(""));
    // limit and 1 （ in up is 「」，
    // Reason is limit not is —— ）
    let facts = crate::server::core::limiter::sources::facts();
    let limiter_counts = crate::server::core::limiter::counter::facts();
    let limiter_blocked: HashSet<String> = scoped
        .iter()
        .filter(|account| crate::server::core::limiter::evaluate::blocked(
            account, &keys.for_account(account), &facts, &limiter_counts, now,
        ))
        .filter_map(|account| routing::account_id(account).map(str::to_string))
        .collect();
    let settings = config::routing_settings_for_model(model.as_deref().unwrap_or(""));
    // ****current， not 、 not before —— not this new 
    // will when ， not this down 1 。 ，
    // `None`： to is 「 from 0 」 。
    let (sunk, resting, open) = state.upstream().route_queue_facts(now);
    let meta = routing::PlanMeta {
        strategy: settings.strategy,
        sunk: &sunk,
        resting: &resting,
        open: &open,
        limiter_blocked: &limiter_blocked,
        cursor: None,
        provider_cursors: None,
        sticky_account: None,
    };
    let plan = routing::plan_route(&scoped, &keys, &counts, &[], now, &meta);
    let entry_json = |entry: &routing::PlanEntry| {
        json!({
            "id": entry.id,
            "name": entry.name,
            "provider": entry.provider,
            "providerLabel": crate::server::core::providers::label_of(&entry.provider),
            "state": entry.state,
            "reason": entry.reason,
            "resetAt": if entry.reset_at > 0 { Value::from(entry.reset_at) } else { Value::Null },
            "priority": entry.account.get("priority").cloned().unwrap_or(Value::Null),
        })
    };
    json!({
        "strategy": settings.strategy.as_str(),
        "sticky": settings.sticky.as_str(),
        "model": model.clone().map(Value::String).unwrap_or(Value::Null),
        "modelKnown": model.as_ref().map(|_| !providers.is_empty()),
        "lastRequestModel": snapshot.last_request_model(),
        "pickedId": plan.ordered.first().map(|entry| entry.id.clone()).unwrap_or_default(),
        "ordered": plan.ordered.iter().map(entry_json).collect::<Vec<_>>(),
        "blocked": plan.blocked.iter().map(entry_json).collect::<Vec<_>>(),
    })
}
