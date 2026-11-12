//! 1 and ****（ from `provider_loop.rs` ，file）。
//!
//! ── Why in before ────────────────────────────────
//! request from `api::chat` ****（request）。 in 
//! ** already 、 will before **， by **** to up ：
//!
//! ⓪ ** 1 **（ `developerRoleCompat`）： 
//! `messages` `role: "developer"` 1 as `"system"`（OpenAI
//! new ， OpenAI up when not ）。 or no has 
//! developer → ，path not 。
//! ① ****（`core::prompt`）： by config （ / / ）
//! has body； and ** can by config**
//! （`config::KEY_PROMPT_PROVIDERS`），config that 1 。
//! （`core::degrade`） use in 。`passthrough` + when ** 1 not **（defaultpath）。
//! ② ****（`core::sanitize`）： by request when config is 
//! used forcurrent， by / 。defaultused for。
//! ③ ** will **（`core::upstream::image_compress`）： when 
//! base64 to limit and JPEG，（
//! 1 when use ）。** in for ** —— / in 
//! before return， not to this 1 。
//!
//! ①② not can （ and 1 ）： in after ， is 1 in 
//! will 。⓪ in before ： 1 ， after and to 1 
//! （ developer→system for 1 after is etc ）。 in 
//! **** up ，client（`ctx.body`） not 。
//!
//! is and to to is 1 ； 1 provider ****
//! use 1 （ not 、 not statistics —— is 「 × ×
//! is 」， as in 、 will in request in ）；
//! 「 this 1 no has available」 when not to ， not 1 already 。
//!
//! ── model by （）────────────────────────────
//! after ， up directory can can is **** not is request
//! （WorkBuddy `deepseek-v4.1-flash` vs `sn-deepseek-v4-1-flash`）。
//! up directory ， this 1 will before ，body model need 
//! this that （`catalog::wire_target_for_provider`）—— 
//! （Cline ） when by **current in **， in 
//! ** ** up ：`ctx.body`（ / / ）client
//! request not 。 and 1 when and ： 1 use 1 。
//!
//! ── 「」 is ：must is after ──────────────────
//! up that 「request not 」 in 2026-09 before is this ， is that 
//! ： is ** up by **， use request will in 1 up 
//! not up （）， is not to 、 already in 。
//! in [`send_body`] 1 and （[`SendBody::wire_model`]），
//! use —— / is request， has `rateLimits`
//! up 。 `routing::CooldownKeys`。
//!
//! ── etc （`mappings[].reasoning`）Why in this 1 ─────
//! etc and 「 this 1 」 is ** 1 ** up ， in 1 
//! 1 （`catalog::wire_target_for_provider` return `WireTarget`
//! when `model` and `reasoning`）—— this is 「A etc not will use to B 」 ：
//! need 1 ， then 。 etc after to **
//! that ** `ProviderAdapter::reasoning_patch` （ can by ），
//! not 1 。 not （、、
//! client already 、 this not ） `model_rules::reasoning` module header。
//!
//! not in this ： in `core::sanitize`， in 
//! `core::providers::catalog`（「 in when 、 for 1 、 use 1 」）。

use std::borrow::Cow;
use std::sync::Arc;

use axum::http::HeaderMap;
use serde_json::Value;

use crate::server::logging;

use super::usage::RequestTelemetry;

/// 1 （，parameter in ）。
pub(super) struct ProviderContext<'a> {
    /// clientrequest（****： already stream:true；； system
    /// —— that is ）。 provider by [`send_body`] 。
    pub body: &'a Value,
    pub routing: super::route_context::RequestRouting,
    /// and request； already in when 。
    pub compact_routed: bool,
    /// client is need （success after ：SSE or ）
    pub stream: bool,
    pub images: Option<crate::server::core::providers::images::ImageOperation>,
    /// clientrequest（ 1 ； not ）
    pub client_headers: &'a HeaderMap,
    /// usage / 
    pub telemetry: &'a Arc<RequestTelemetry>,
    /// 、 and must use 1 request， in request。
    pub sanitize: &'a crate::server::config::SanitizeSettings,
    /// request ****（request when 1 ， and 
    /// `sanitize` 1 ： 1 request must 1 ）。
    ///
    /// when [`send_body`] in before `messages` `role: "developer"` 
    /// 1 as `"system"`（OpenAI new API ； OpenAI up is request
    /// ， not 1 will developer ）。default，
    /// `config::KEY_DEVELOPER_ROLE_COMPAT`。
    pub developer_role_compat: bool,
    /// request ** will **set（request when by request 1 ）。
    ///
    /// `Copy` （ not `sanitize` / `prompt` that use ）：、
    /// none， use 。 when [`send_body`] in for up 
    /// base64 （ `upstream::image_compress` module header）； /
    /// in already before return， not 。
    pub image_compress: crate::server::config::ImageCompressSettings,
    /// request ****（ + ，request when from config 1 ）。
    ///
    /// and `sanitize` 1 ： and in 1 requestmust 1 ，
    /// otherwise will 「 before 1 、 after 1 no 」。 use request
    /// （ can can ，request）， by 
    /// `upstream::forward` config has 。
    ///
    /// ** not is ** ： after request will to in （），
    /// that not in —— by by request has and to [`send_body`]。
    pub prompt: crate::server::core::prompt::PromptPlan<'a>,
    /// system not 、 or 。
    pub preserve_system_prompt: bool,
    /// request in Key **available**（R9；`None` = unlimited，
    /// `core::key_scope` module header）。
    ///
    /// Why in this （ up down ） not is request： is 
    /// 「 1 」 （、client、telemetry in this ），
    /// and —— （、） 1 ，
    /// not will in 。 use is as by `upstream::forward` has ，
    /// 。
    pub key_scope: Option<&'a crate::server::core::key_scope::KeyScope>,
    /// use this （；`None` = ）。
    ///
    /// and `key_scope` 1 ： is 「 」， and 
    /// 1 ， not will in 。 
    /// [`super::ForwardRequest::pinned_account`] and `rotate::accounts_in_providers`。
    pub pinned_account: Option<&'a str>,
    /// by by （ 「enabled can 」； and 
    /// [`super::ForwardRequest::ignore_model_gate`]）。 from `key_scope`
    /// —— is 1 ， is 1 。
    pub ignore_model_gate: bool,
    /// request when （ use ；`None` = config，
    /// `Some(None)` = ，`Some(Some(proxy))` = ）。
    pub proxy_override: Option<&'a Option<crate::server::core::proxies::ResolvedProxy>>,
}

/// 1 provider need request（** before **， not use ），
/// and this use up 。
///
/// ── Why 「 up 」 and request in 1 return（）──
/// this is ** **： up by ，`rateLimits` by 
/// （ `routing::CooldownKeys`）。must and ** ** ——
/// if use 1 `wire_target_for_provider` ， has 、
/// can can ， is that error：
/// 「 in 1 up 、 in 1 up ， is already in 」。
/// 1 、 （ + ）， this in up not can can 。
///
/// `wire_model` in request no has `model` when is empty（ that when up to is 
/// default，none from ， by empty —— and before 1 ）。
pub(super) struct SendBody<'a> {
    /// need request： in and none need when use client
    /// body（），otherwise is 。
    pub body: Cow<'a, Value>,
    /// this to up —— is `rateLimits` 。
    pub wire_model: String,
    pub wire_reasoning: Option<String>,
}

/// 1 provider need request（** before **， not use ）。
///
/// return [`SendBody`]： in and none need when client
/// body； in or need need model this when 。 and in 
/// `core::sanitize` / `core::providers::catalog`，「 in 
/// when 1 」—— when is 「 this 1 will before 」， 1 
/// not will 、statistics。
///
/// ** up **： to this in this （none is ），
/// telemetry（， after 1 as —— and provider 1 ，
/// 429 after down is that 1 ）。request 「 up 」
/// not need need 。
///
/// ** in **：`sanitize_body` return in telemetry，
/// is 「 this request in then 」request 1 request `sensitiveHits`。
/// can is this —— has to that in 。
///
/// ── is **** not is ─────────────────────────────────
/// and `note_attempt` 「 after 1 as 」 not ， in by ** and **： up 
/// A 、 to B 1 ， 1 then will in 1 。
/// 「 this request in 」 is use need （B is 1 1 ），
/// 1 when not （`send_cache` already this 1 ： 1 
/// 1 ）， then and 。
/// when up has need ： in in ** 1 will when **， that when request no ，
/// telemetry （ in after ）。
///
/// ── Why is async（）─────────────────────────────
/// need 「 this etc 」，
/// in 1 （`reasoning_patch_async`）。 use （
/// `provider_loop` path） in async ， async not 
/// 。
pub(super) async fn send_body<'a>(
    ctx: &'a ProviderContext<'_>,
    provider_id: &str,
    account: Option<&Value>,
    degraded: bool,
) -> SendBody<'a> {
    if ctx.images.is_some() {
        let requested = ctx.body.get("model").and_then(Value::as_str).unwrap_or("");
        let wire = crate::server::core::providers::catalog::wire_target_for_provider(
            requested, provider_id, account,
        );
        let mut body = Cow::Borrowed(ctx.body);
        rewrite_model(&mut body, requested, &wire.model, provider_id);
        ctx.telemetry.note_upstream_model(&wire.model);
        ctx.telemetry.note_upstream_reasoning(None);
        return SendBody { body, wire_model: wire.model, wire_reasoning: None };
    }
    // ── ⓪ 1 （）───────────────────────────
    // in all** before **： developer 1 system， after （、
    // 、） to is 1 —— developer→system 
    // for 1 after is etc ， not will 。 or no has developer when 
    // down （）， has need 1 。
    let role_base: Cow<'a, Value> =
        match normalize_developer_role(ctx.body, ctx.developer_role_compat) {
            Some((next, changed)) => {
                logging::verbose(
                    "[Upstream]",
                    &format!(
                        "角色归一层：developer → system × {changed} 条 provider={provider_id}（开发者角色兼容）"
                    ),
                );
                Cow::Owned(next)
            }
            None => Cow::Borrowed(ctx.body),
        };
    // ── ① （ has ： / / ）──────────────
    // in ** before **： and （ → ）， is 
    // 1 in will after 1 ； ，「 up that 」
    // no 1 。
    //
    // ** 1 ** is by （ `config::KEY_PROMPT_PROVIDERS`）：
    // is 「 1 will before 」 that 1 ，`provider_id` in up ， not need need 
    // new when ， is 「 1 」 from this that 1 。
    //
    // `degraded` = request is already （request when already ， or 
    // after by ）： use in （`custom` ，
    // `PromptChoice::text_for`）。
    let prompt = ctx.prompt.for_provider(provider_id);
    let processed = if ctx.preserve_system_prompt { None } else { prompt.apply(role_base.as_ref(), degraded) };
    let after_prompt = match processed {
        Some(next) => {
            logging::verbose(
                "[Upstream]",
                &format!(
                    "系统提示词层：{}（{}）provider={} messages {} → {}",
                    prompt.mode.label(),
                    if degraded && prompt.mode.degradable() {
                        "降级期：中性提示词"
                    } else {
                        prompt.source.label()
                    },
                    provider_id,
                    message_count(ctx.body),
                    message_count(&next),
                ),
            );
            Cow::Owned(next)
        }
        None => role_base,
    };
    // ── ② （ then `core::sanitize`）─────────────────────
    let body = if ctx.sanitize.applies_to(provider_id) {
        match crate::server::core::sanitize::sanitize_body(after_prompt.as_ref(), &ctx.sanitize.rules) {
            Some((scrubbed, hits)) => {
                if !hits.is_empty() {
                    ctx.telemetry.note_sensitive_hits(&hits);
                }
                Cow::Owned(scrubbed)
            }
            None => after_prompt,
        }
    } else {
        after_prompt
    };
    // ── ③ will （only for ）─────────────────────────────────
    // base64 to limit and （parameter and 
    // `config::ImageCompressSettings`）， ——
    // for client 1 when in ， not 。
    //
    // / in （`ctx.images.is_some()`） before return， not to 
    // this —— need can use in for 。
    //
    // CPU （ 1 MB can can ms）， `blocking::in_place` 
    // current tokio worker（ when down `block_in_place`；otherwise）。 in 
    // has when request（`has_compressible` ，request）。
    let body = if ctx.image_compress.enabled
        && crate::server::core::upstream::image_compress::has_compressible(body.as_ref())
    {
        let settings = ctx.image_compress;
        let mut owned = body.into_owned();
        let stats = crate::server::core::blocking::in_place(|| {
            crate::server::core::upstream::image_compress::compress_images_in_place(
                &mut owned, &settings,
            )
        });
        if stats.changed() {
            logging::verbose(
                "[Upstream]",
                &format!(
                    "会话图片压缩：{} 张 provider={provider_id} {}KB -> {}KB",
                    stats.compressed,
                    stats.bytes_before / 1024,
                    stats.bytes_after / 1024,
                ),
            );
        }
        Cow::Owned(owned)
    } else {
        body
    };
    // ── ④ （server） ──────────────────────────────
    // is client after ：client `type:"function"` 
    // （client ）， up for from 「 400」
    // （CatPaw） to 「」（Qoder / Trae） has —— 1 in this ：
    // + ， to has 。
    //
    // ** not ** this （ is `is_custom_provider_id`）：chat 
    // （ is ， `providers::custom::forward`），
    // in 。 this will client chat 
    // （ that ） 1 and ， and 。
    // after if supports ， in this （ 1 ） by / by can 。
    // `core::protocol::native_tool` module header。
    let mut body = if crate::server::core::custom_providers::is_custom_provider_id(provider_id) {
        body
    } else {
        match crate::server::core::protocol::native_tool::downgrade(body.as_ref()) {
            Some((next, dropped)) => {
                logging::log(
                    "[Upstream]",
                    &dropped.describe(
                        Some(provider_id),
                        Some("本上游只承载 type:\"function\" 的函数工具"),
                    ),
                );
                Cow::Owned(next)
            }
            None => body,
        }
    };
    let requested = body
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if requested.is_empty() {
        // no has model ： not ， no has available （empty， and before 1 ）
        ctx.telemetry.note_upstream_reasoning(None);
        return SendBody { body, wire_model: requested, wire_reasoning: None };
    }
    // 1 ： this need + that etc 
    // （，module header「 etc Why in this 1 」）
    let direct = account.and_then(crate::server::core::routing_groups::candidate_model);
    let wire = direct.map(|model| crate::server::core::providers::catalog::WireTarget { model: model.to_string(), reasoning: None })
        .unwrap_or_else(|| crate::server::core::providers::catalog::wire_target_for_provider(&requested, provider_id, account));
    ctx.telemetry.note_upstream_model(&wire.model);
    rewrite_model(&mut body, &requested, &wire.model, provider_id);
    if ctx.compact_routed {
        crate::server::core::compact_routing::parameters::cap_output(&mut body, provider_id, &wire.model);
    }
    let injected = apply_reasoning(
        &mut body,
        provider_id,
        &requested,
        &wire.model,
        wire.reasoning.as_deref(),
    )
    .await;
    // 「 up request etc 」（request `( etc )`）。
    // （ when ，CatPaw already in patch and ）；
    // no when `outbound_reasoning` —— client this 
    // （， in body request up ）。empty = no has etc ，
    // None（empty）， not to 「 no etc 」 1 。
    // path already 1 ， this 1 is ， can ignore。
    let upstream_reasoning = injected.or_else(|| outbound_reasoning_of(provider_id, &body));
    ctx.telemetry.note_upstream_reasoning(upstream_reasoning.clone());
    SendBody { body, wire_model: wire.model, wire_reasoning: upstream_reasoning }
}

/// [`ProviderAdapter::outbound_reasoning`]（ etc ）。
///
/// unknown provider id return None： and [`apply_reasoning`] 1 ——
/// already ， to this not to use ， not panic 。
///
/// ── Whyreturn None ────────────────────────────────────
/// this in up down （ ）， need RPC， no in this 
/// await。 1 use is ****「 this etc 」， not 
/// —— `None`（ by 「 no 」）， etc P6 
/// `ChatPlan` reasoning when 。
pub(super) fn outbound_reasoning_of(provider_id: &str, body: &Value) -> Option<String> {
    // （）； in up down none await，
    // `adapter_by_id` is to （`outbound_reasoning` is ）。
    match crate::server::core::providers::kind_from_id(provider_id) {
        Some(kind) => crate::server::core::providers::adapter::adapter_for(kind).outbound_reasoning(body),
        None => crate::server::core::providers::adapter::adapter_by_id(provider_id)?
            .outbound_reasoning(body),
    }
}

/// request （ use ； no has messages when to 0）。
fn message_count(body: &Value) -> usize {
    body.get("messages")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0)
}

/// 1 ： `messages` `role: "developer"` as `"system"`
/// （， `config::KEY_DEVELOPER_ROLE_COMPAT`）。
///
/// return `None` = 1 no （、 no has messages 、 or no has 
/// developer ）—— use use ，path not ；`Some`
/// is after **** and 。 by （ and 
/// `responses_outbound` / `anthropic_outbound` has 1 ）， is 
/// `Developer` this 1 and 1 。
///
/// has to developer clone request：messages is 1 request
/// 1 ， this in need when —— this is Why not 
/// `&mut Cow`（ `rewrite_model` that ） is return Option： when 
/// `to_mut()` not use 。
fn normalize_developer_role(body: &Value, enabled: bool) -> Option<(Value, usize)> {
    if !enabled {
        return None;
    }
    let messages = body.get("messages")?.as_array()?;
    let hits = messages
        .iter()
        .filter(|message| {
            message
                .get("role")
                .and_then(Value::as_str)
                .is_some_and(|role| role.eq_ignore_ascii_case("developer"))
        })
        .count();
    if hits == 0 {
        return None;
    }
    let mut next = body.clone();
    let mut changed = 0usize;
    if let Some(list) = next.get_mut("messages").and_then(Value::as_array_mut) {
        for message in list.iter_mut() {
            if message
                .get("role")
                .and_then(Value::as_str)
                .is_some_and(|role| role.eq_ignore_ascii_case("developer"))
            {
                if let Some(slot) = message.get_mut("role") {
                    *slot = Value::String("system".to_string());
                    changed += 1;
                }
            }
        }
    }
    (changed > 0).then_some((next, changed))
}

/// model this provider （only need need when ）。
///
/// `requested` is request、`wire` is `catalog::wire_target_for_provider` already 
/// this （ use 1 ， this not directory）； has **** when 
/// use —— not **must**：directoryreturn is up 
/// id（Loomy directory is `GLM-5.3-Flash`，client）， when not ，
/// up will unknown（：`glm-5.3-flash` 
/// `deepseek-v4-flash-0731`， can ）。
fn rewrite_model(body: &mut Cow<'_, Value>, requested: &str, wire: &str, provider_id: &str) {
    if wire == requested {
        return;
    }
    logging::verbose(
        "[Upstream]",
        &format!("provider={provider_id} 按该家目录改写模型名 {requested} → {wire}（备援名）"),
    );
    let object = body.to_mut().as_object_mut();
    if let Some(object) = object {
        object.insert("model".to_string(), Value::String(wire.to_string()));
    }
}

/// up etc to ** this ** ， and by 。
///
/// ── Why in this （ not is ）─────────────
/// 「 this 1 」（`ProviderAdapter::reasoning_patch` return
/// and or 1 「 not 」），、、 and 「 etc not this 」
/// this is use ， in this 1 ： 1 ， 1 
/// will 「 」。
///
/// ── in ** before ** down （ has ）─────────────
/// 1. `level = None`： this no etc （ or this is and no has 
/// for ）—— request this 1 ，return；
/// 2. `off` / `none`：****。 no has （Qoder 
/// `enable_thinking = false` will Qwen3.8 ，CatPaw no has this 1 ），
/// not —— will 「 1 false」，
/// that is 。 and by in `model_rules::reasoning`。
///
/// its （ etc 、client already 、 this not ） by return `Skip`
/// and to Reason —— that is ** **， this not 。
///
/// ── and 「 not 」 ──────────────────────────────────
/// success and 1 verbose（`Skip` that 1 its need ： use not when ，
/// can to Why）。`body` in `Set` `to_mut()` ——
/// path、， and `rewrite_model` 1 。
///
/// ── 「」 this （ when ）─────────────────────
/// this 1 is 「** request**」， not is 「 up 1 will 」：
/// to `Set` when already this （ that is ，
/// `reasoning_patch` ），**** can can ——
/// Qoder `protocol::resolve_thinking` will by efforts 1 and 
/// （e.g. `minimal` → `low`、unsupported this default）。
/// that 1 up down （） has to ， in that is for ；
/// this not ， 1 。
///
/// up `requested → wire_model`（ and `rewrite_model` that 1 ）：
/// （ for and up id ） when ， use can from this 1 
/// 「 this etc 」； not when is 「 this 」 。
///
/// ** not client **： and is （ use that 
/// resolver ）， this `field` up 。
///
/// return is **success when **（`Set` body that ；
/// `value` not is when to None）： use 「 up etc 」 1 
/// —— when is （CatPaw and already in patch ）。
/// its （ no / / Skip / request not is for ） 1 return None，
/// by use `outbound_reasoning`。
async fn apply_reasoning(
    body: &mut Cow<'_, Value>,
    provider_id: &str,
    requested: &str,
    wire_model: &str,
    level: Option<&str>,
) -> Option<String> {
    let Some(level) = level.map(str::trim).filter(|text| !text.is_empty()) else {
        return None;
    };
    if crate::server::core::model_rules::reasoning_is_off(level) {
        logging::verbose(
            "[Upstream]",
            &format!(
                "provider={provider_id} 映射 {requested} → {wire_model} 绑定的思考等级为\
                 「{level}」（关闭思考），本网关不向任何上游发「关闭思考」字段，跳过注入"
            ),
        );
        return None;
    }
    // unknown provider id return： already by （`provider_loop` for unknown
    // id 503）， to this use —— not panic 
    // （release is panic=abort）。
    //
    // in this ****：`reasoning_patch_async` is as 
    // （，）， 1 `adapter_by_id`
    // —— 、， can to 。
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider_id) else {
        return None;
    };
    // ：（）， RPC。
    match adapter.reasoning_patch_async(level, wire_model, body.as_ref()).await {
        crate::server::core::providers::adapter::ReasoningPatch::Set { field, value } => {
            // can for ：request not is JSON for when not （path not will 
            // —— chat already is for ）， when ** not can **「 already 」——
            // 「 is has 」 is this 1 can ，
            // 1 and not 「 already 」 no has 。
            let Some(object) = body.to_mut().as_object_mut() else {
                logging::verbose(
                    "[Upstream]",
                    &format!(
                        "provider={provider_id} 映射 {requested} → {wire_model} \
                         绑定的思考等级 {level} 未注入：请求体不是 JSON 对象"
                    ),
                );
                return None;
            };
            logging::verbose(
                "[Upstream]",
                &format!(
                    "provider={provider_id} 映射 {requested} → {wire_model} \
                     注入思考等级 {level} → {field}={value}"
                ),
            );
            // use in move before ： is 「」， its （ will 
            // / for ） to use that outbound_reasoning 
            let injected = value.as_str().map(str::to_string);
            object.insert(field.to_string(), value);
            injected
        }
        crate::server::core::providers::adapter::ReasoningPatch::Skip { reason } => {
            logging::verbose(
                "[Upstream]",
                &format!(
                    "provider={provider_id} 映射 {requested} → {wire_model} \
                     绑定的思考等级 {level} 未注入：{reason}"
                ),
            );
            None
        }
    }
}