//! AI API（「AI 」 for + set「AI 」 ）。
//!
//! GET /api/assistant/config → `{ config: {...} }`（default， down ）
//! PUT /api/assistant/config `{ config: {...} }` → after 
//!   POST /api/assistant/chat     `{ model?: {provider,id}, messages, tools? }`
//! → **SSE **（OpenAI delta， tool_calls）
//!
//! ── Why for not in before /v1/chat/completions ─────────────
//! Tauri （`tauri://localhost`）， before fetch to 
//! `http://127.0.0.1:{port}/v1/*` is ****request，default not CORS
//! （ `KEY_CORS_ENABLED` ）—— that need use CORS ，
//! etc 「 use 」 1 and none 。 to by （`/api/*`，
//! none CORS） not that ， and for is 1 。
//!
//! ── and /v1/chat/completions ─────────────────────────
//! ** use 1 **（`UpstreamService::forward`）：token new 、429 、
//! 、、 in that up 。file：
//! ① 「 use 」 up —— is for （`/v1/models` 
//! `data[].id`，）， `resolve_model`（default /
//! / directory and /v1/* ）；** not **， in when by 
//! —— by and client 1 ；
//! ② by set（ × is as empty， `AssistantConfig::system_prompt`）；
//! ③ （OpenAI ， before delta / tool_calls）。
//!
//! ── and in before ───────────────────────────────────
//! `tools` is before by OpenAI tools ； after for ID and after 
//! to up ， not parameter。`tool_calls` before ， by before 、 
//! `role: "tool"` as down 1 messages up 。 use already has 
//! /api/* ；addconfig in ID，parameter and in before 。
//!
//! ── ────────────────────────────────────────────────────
//! and 1 ：request（ use in 「request」 to 1 
//! ），`is_test = true` excluding（ 
//! will ）。 as `assistant`， Key request。

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{json, Map, Value};

use crate::server::config::{self, AssistantConfig, PermissionMode, MAX_SYSTEM_PROMPT_CHARS};
use crate::server::core::assistant_sessions;
use crate::server::core::upstream::cancellation;
use crate::server::core::upstream::usage::RequestTelemetry;
use crate::server::core::upstream::{ForwardOutcome, ForwardRequest};
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

use super::disconnect_guard::DisconnectGuard;
use super::pipeline::{self, json_response, RecordContext, RecordingStream, sse_response};

/// for limit（ and `model_test` `text_field` limit 1 ）
const MAX_MODEL_CHARS: usize = 128;
/// for messages limit。
///
/// not is use ， is 「 before bug up down to 」 and up 
/// up down 1 ：200 1 will （ is 1 1 ）。
const MAX_MESSAGES: usize = 200;
/// tools limit（ before before 15 ，）
const MAX_TOOLS: usize = 64;
/// etc limit（ and `model_rules::normalize_reasoning` 1 ）
const MAX_REASONING_CHARS: usize = 32;

/* ─── set（ 「set」）───────────────────────── */

/// GET /api/assistant/config
///
/// use `status_value`： 15 （ in after ， not 
/// 1 default， `AssistantConfig::status_value`）。
pub async fn get_config(State(_state): State<ServerState>) -> Response {
    ok_json(json!({ "config": config::assistant_config().status_value() }))
}

/// PUT /api/assistant/config
///
/// ： before GET response 1 ， by default
/// （** not is **「 = 」—— before ， use will 
/// 「 this 1 」 not ）。
pub async fn put_config(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(payload) => payload,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(block) = payload.get("config").and_then(Value::as_object) else {
        return errors::management_error(400, "缺少 config 对象");
    };
    let config = match parse_config(block) {
        Ok(config) => config,
        Err(message) => return errors::management_error(400, message),
    };
    if !config::set_assistant_config(config.clone()) {
        logging::log(
            "[Config]",
            "⚠️  AI 助手设置写入失败（本次运行内仍生效）",
        );
    }
    // to before and GET 1 （ ）——「」 and 
    // 「 to 」 use 1 ， before after that that and after 
    ok_json(json!({ "config": config::assistant_config().status_value() }))
}

/// and set（failure to ， not ）
fn parse_config(block: &Map<String, Value>) -> Result<AssistantConfig, String> {
    let model = match block.get("model") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            let id = text.trim();
            if id.is_empty() {
                None
            } else {
                if id.chars().count() > MAX_MODEL_CHARS {
                    return Err("模型名过长".to_string());
                }
                Some(id.to_string())
            }
        }
        Some(_) => return Err("model 必须是字符串或 null".to_string()),
    };
    let reasoning = match block.get("reasoning") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            let level = text.trim();
            if level.is_empty() {
                None
            } else if level.chars().count() > MAX_REASONING_CHARS {
                return Err(format!("思考等级最长 {MAX_REASONING_CHARS} 个字符"));
            } else {
                Some(level.to_string())
            }
        }
        Some(_) => return Err("reasoning 必须是字符串或 null".to_string()),
    };
    let system_prompt = match block.get("systemPrompt") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => {
            if text.chars().count() > MAX_SYSTEM_PROMPT_CHARS {
                return Err(format!("系统提示词最长 {MAX_SYSTEM_PROMPT_CHARS} 字"));
            }
            text.clone()
        }
        Some(_) => return Err("systemPrompt 必须是字符串".to_string()),
    };
    let tools = match block.get("tools") {
        None | Some(Value::Null) => Default::default(),
        Some(Value::Object(tools)) => tools
            .iter()
            .filter_map(|(id, on)| on.as_bool().map(|on| (id.clone(), on)))
            .collect(),
        Some(_) => return Err("tools 必须是对象".to_string()),
    };
    let title_model = match block.get("titleModel") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => {
            let text = text.trim();
            if text.chars().count() > MAX_MODEL_CHARS { return Err("标题总结模型名过长".into()); }
            (!text.is_empty()).then(|| text.to_string())
        }
        Some(_) => return Err("titleModel 必须是字符串或 null".into()),
    };
    let title_prompt = match block.get("titlePrompt") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) if text.chars().count() <= MAX_SYSTEM_PROMPT_CHARS => text.clone(),
        Some(Value::String(_)) => return Err(format!("标题总结提示词最长 {MAX_SYSTEM_PROMPT_CHARS} 字")),
        Some(_) => return Err("titlePrompt 必须是字符串".into()),
    };
    let api_key_id = match block.get("apiKeyId") {
        None | Some(Value::Null) => None,
        Some(Value::String(id)) if id.trim().len() <= 128 => {
            let id = id.trim();
            (!id.is_empty()).then(|| id.to_string())
        }
        Some(_) => return Err("apiKeyId 必须是有效的 Key ID 或 null".to_string()),
    };
    Ok(AssistantConfig {
        model,
        api_key_id,
        reasoning,
        title_model,
        title_prompt,
        // ： confirm / full， and unknown to 「 before 」
        // （ to and assistant_config::from_raw 1 ）
        mode: match block.get("mode") {
            None | Some(Value::Null) => PermissionMode::default(),
            Some(Value::String(text)) => PermissionMode::from_str_lenient(text),
            Some(_) => return Err("mode 必须是字符串".to_string()),
        },
        system_prompt_enabled: block
            .get("systemPromptEnabled")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        system_prompt,
        tools,
    })
}

/* ─── for （SSE）────────────────────────────────────────── */

/// POST /api/assistant/chat
pub async fn chat(State(state): State<ServerState>, headers: HeaderMap, body: Bytes) -> Response {
    let access = match super::model_access::authenticate(&headers, super::model_access::ASSISTANT, true) {
        Ok(access) => access,
        Err(error) => return chat_error(error),
    };
    let payload = match parse_body(&body) {
        Ok(payload) => payload,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(messages) = payload.get("messages").and_then(Value::as_array).cloned() else {
        return errors::management_error(400, "缺少 messages 数组");
    };
    if messages.is_empty() {
        return errors::management_error(400, "messages 不能为空");
    }
    if messages.len() > MAX_MESSAGES {
        return errors::management_error(400, format!("messages 最多 {MAX_MESSAGES} 条"));
    }
    // request ； no use set 。 no has when down 1 ——
    // 「 1 」「」 this has use 。
    // this model is ** for **（/v1/models data[].id，），
    // by to —— in when by ， and client 1 。
    let config = config::assistant_config();
    let model = match model_from(&payload) {
        Ok(Some(model)) => model,
        Ok(None) => match config.model.clone() {
            Some(model) => model,
            None => {
                return errors::management_error(
                    400,
                    "还没有选择对话模型：去助手页的「助手设置」里选一个，或先在模型管理页确认有已启用的模型",
                )
            }
        },
        Err(message) => return errors::management_error(400, message),
    };
    let tools = match payload.get("tools") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(tools)) => {
            if tools.len() > MAX_TOOLS {
                return errors::management_error(400, format!("tools 最多 {MAX_TOOLS} 个"));
            }
            for tool in tools {
                let Some(id) = tool.get("function").and_then(|function| function.get("name")).and_then(Value::as_str) else {
                    return errors::management_error(400, "工具缺少 function.name");
                };
                if !config::ALL_TOOL_IDS.contains(&id) || !config.tool_enabled(id) {
                    return errors::management_error(403, format!("工具「{id}」未获授权"));
                }
            }
            tools.clone()
        }
        Some(_) => return errors::management_error(400, "tools 必须是数组"),
    };

    // ： × is as empty in config （「 1 
    // no not 」 this ）； this to messages before 
    let system = config.system_prompt(!tools.is_empty());
    let mut outbound_messages = Vec::with_capacity(messages.len() + 1);
    if let Some(text) = system {
        outbound_messages.push(json!({ "role": "system", "content": text }));
    }
    outbound_messages.extend(messages);

    // etc ：request ， no use set （ and 1 「
    // > default」）。 `reasoning_effort` —— that is client 
    // ， by has ** up etc **（ `model_rules::reasoning`
    // module header）； up 。
    let reasoning = reasoning_from(&payload).or_else(|| config.reasoning.clone());

    let mut upstream = json!({
        "model": model,
        "messages": outbound_messages,
        // ： use and need to ， for for no has 
        "stream": true,
    });
    if !tools.is_empty() {
        upstream["tools"] = Value::Array(tools);
    }
    if let Some(level) = reasoning.as_deref() {
        upstream["reasoning_effort"] = Value::String(level.to_string());
    }

    // 模型路由：对外名 → 上游名（默认值 / 映射别名 / 目录校验与 /v1/* ）
    let started_at = logging::now_ms();
    let requested_model = match super::request_prepare::resolve_model(&state, &mut upstream, access.scope.as_ref()).await {
        Ok(name) => name,
        Err(error) => {
            pipeline::record_early_failure(&state, started_at, &model, reasoning.as_deref().unwrap_or(""), &access.source, &error);
            return chat_error(error);
        }
    };

    if upstream.get("tools").and_then(Value::as_array).is_some_and(|tools| tools.iter().any(|tool| {
        tool.pointer("/function/name").and_then(Value::as_str) == Some("edit_images")
    })) {
        let catalog = crate::server::core::providers::catalog::models_response(state.store(), access.scope.as_ref());
        let supports_images = catalog.get("data").and_then(Value::as_array)
            .and_then(|models| models.iter().find(|item| {
                item.get("id").and_then(Value::as_str).is_some_and(|id| id.eq_ignore_ascii_case(&requested_model))
            })).is_some_and(|model| model.get("supports_images").and_then(Value::as_bool) == Some(true));
        if !supports_images {
            super::assistant_images::preserve_tool_references(&mut upstream);
        }
    }
    let telemetry = Arc::new(RequestTelemetry::with_id());
    let telemetry_id = telemetry.snapshot().id;
    // in ： and request 1 when 。`is_test = true` —— excluding、request
    // （ and 1 ，module header）； down use that for 
    super::request_prepare::record_started(
        state.request_stats(),
        &telemetry_id,
        started_at,
        &requested_model,
        &model,
        reasoning.as_deref().unwrap_or(""),
        &access.source,
        true,
    ).await;
    if let Some(token) = cancellation::register(&telemetry_id) {
        telemetry.set_cancel_token(token);
    }
    let mut guard = DisconnectGuard::new(state.request_stats(), telemetry_id.clone());

    let outcome = state
        .upstream()
        .forward_assistant(ForwardRequest {
            body: upstream,
            stream: true,
            // not ： for not is 「 body 」， 1 1 this 1 
            dedupe_key: String::new(),
            // empty： not is client ， not this to up 
            client_headers: HeaderMap::new(),
            telemetry: telemetry.clone(),
            // ** not **： for is for ， by and /v1/chat/completions
            // 1 —— in when by ，failure down 1 
            allowed_providers: access.scope.clone(),
            pinned_account: None,
            // ： use is 「 already enabled 」， use not this 
            // as （ and request 1 ）
            ignore_model_gate: false,
            proxy_override: None,
        })
        .await;

    match outcome {
        Ok(ForwardOutcome::Stream { status, stream: source }) => {
            let status = StatusCode::from_u16(status).unwrap_or(StatusCode::OK);
            let context = RecordContext {
                stats: state.request_stats(),
                telemetry,
                started_at,
                model: requested_model,
                client_model: model.clone(),
                client_reasoning: reasoning.clone().unwrap_or_default(),
                status: i64::from(status.as_u16()),
                raw_request: None,
                raw_response: None,
                is_test: true,
                source: access.source.clone(),
            };
            // to response（ and ）； not 
            guard.handoff();
            sse_response(
                status,
                Box::new(RecordingStream::with_terminals(
                    source,
                    context,
                    pipeline::terminal::CHAT,
                )),
            )
            .into_response()
        }
        // need ， this up to not ； to by JSON ，
        // before for SSE response has 1 （ 1 assistant ）
        Ok(ForwardOutcome::Completion { body }) => {
            let raw_response = pipeline::capture_raw_json(&body);
            pipeline::record_entry(
                &RecordContext {
                    stats: state.request_stats(),
                    telemetry,
                    started_at,
                    model: requested_model,
                    client_model: model.clone(),
                    client_reasoning: reasoning.clone().unwrap_or_default(),
                    status: 200,
                    raw_request: None,
                    raw_response,
                    is_test: true,
                    source: access.source.clone(),
                },
                None,
            );
            guard.complete();
            json_response(body)
        }
        Err(error) => {
            let status = i64::from(error.http_status().as_u16());
            let message = error.message.clone();
            pipeline::record_entry(
                &RecordContext {
                    stats: state.request_stats(),
                    telemetry,
                    started_at,
                    model: requested_model,
                    client_model: model.clone(),
                    client_reasoning: reasoning.clone().unwrap_or_default(),
                    status,
                    raw_request: None,
                    raw_response: None,
                    is_test: true,
                    source: access.source.clone(),
                },
                Some(message.clone()),
            );
            guard.complete();
            chat_error(error)
        }
    }
}

fn chat_error(error: errors::GatewayError) -> Response {
    let status = error.http_status();
    (
        status,
        axum::Json(json!({
            "success": false,
            "error": error.message,
            "code": error.code,
        })),
    )
        .into_response()
}

/// from request etc （； / null / empty → None）。
///
/// and empty，** not **： etc （`model_rules::REASONING_LEVELS`）
/// is 「 up 」， not is —— use can in etc ，
/// this if ， 1 in 「」 and 「」 up will has 
/// 。 by （ not not will ）。
fn reasoning_from(payload: &Value) -> Option<String> {
    let text = payload.get("reasoning").and_then(Value::as_str)?.trim();
    if text.is_empty() || text.chars().count() > 32 {
        return None;
    }
    Some(text.to_string())
}

/// from request ：** for **（； / null → None）。
///
/// old before `{provider, id}` for —— `id`，provider not and by 
/// （ for already and /v1/models for ， in by ）。
fn model_from(payload: &Value) -> Result<Option<String>, String> {
    match payload.get("model") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let id = text.trim();
            if id.is_empty() {
                return Err("model not can as empty".to_string());
            }
            Ok(Some(id.to_string()))
        }
        Some(Value::Object(model)) => {
            let id = model.get("id").and_then(Value::as_str).unwrap_or("").trim();
            if id.is_empty() {
                return Err("model.id not can as empty".to_string());
            }
            Ok(Some(id.to_string()))
        }
        Some(_) => Err("model must is ".to_string()),
    }
}

/* ─── will （ that for ）────────────────────────────── */

/// GET /api/assistant/sessions → `{sessions: [{id, title, model, reasoning,
/// createdAt, updatedAt}, …]}`（ by update）。
///
/// ** not messages**： use to and when ， will 
/// 1 is 。 1 when down 。
pub async fn list_sessions(State(_state): State<ServerState>) -> Response {
    ok_json(json!({ "sessions": assistant_sessions::list() }))
}

/// GET /api/assistant/sessions/{id} → `{session: {…, messages: [...]}}`
pub async fn get_session(State(_state): State<ServerState>, Path(id): Path<String>) -> Response {
    match assistant_sessions::get(&id) {
        Some(session) => ok_json(json!({ "session": session })),
        None => errors::management_error(404, "会话不存在"),
    }
}

/// POST /api/assistant/sessions → `{session: {…}}`（ new or id will ）。
///
/// body：`{id, title?, messages, model?, reasoning?}`。id by before （
/// `core::assistant_sessions` module header）， this when 「 new 」 and 
/// 「」—— before after ， not is 。
pub async fn save_session(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(payload) => payload,
        Err(error) => return errors::management_error(400, error.message),
    };
    match assistant_sessions::save(&payload) {
        Ok(session) => ok_json(json!({ "session": session })),
        Err(message) => errors::management_error(400, message),
    }
}

/// DELETE /api/assistant/sessions/{id} → `{removed: true}`
pub async fn delete_session(State(_state): State<ServerState>, Path(id): Path<String>) -> Response {
    if assistant_sessions::remove(&id) {
        ok_json(json!({ "removed": true }))
    } else {
        errors::management_error(404, "会话不存在")
    }
}

/// DELETE /api/assistant/sessions → `{removed: n}`（empty will ）。
///
/// and delete by （ before DELETE to this ）： before 「empty」
/// ，request。
pub async fn clear_sessions(State(_state): State<ServerState>) -> Response {
    let removed = assistant_sessions::clear();
    ok_json(json!({ "removed": removed }))
}
