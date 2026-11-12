//! and ：`GET/PUT /api/prompt`。
//!
//! ── this ──────────────────────────────────────────
//! （ 1 ， as is 1 and ）：
//!
//! ① ****（`promptMode` / `promptFile` / `promptText`）：
//! `passthrough`（default，client system）/ `custom`（ use has 
//! system ）/ `append`（ in system after 1 ）。 is 
//! —— from system ， `core::prompt` module header。
//! need use ** that ** has ：file（`promptFile`） and 
//! （`promptText`， use in set that 1 ，）；empty
//! then use default。 has `promptProviders`（** by ** 1 set，：
//! no 1 use up that set）—— 1 up 「 1 」 this by up 
//! ， `config::KEY_PROMPT_PROVIDERS`。
//! ② ** and **（`promptGateway` / `promptGatewayText`）：
//! up that （ has ZCode ）
//! need not need 、 that 。 and ① is ：① client system
//! ，② （ `config::KEY_PROMPT_GATEWAY`）。
//! ③ ****（ + 1 ）：`passthrough` / `append` down 
//! （HTTP 400 + ） when ， will in 1 ，
//! and 「」 to 00:00（UTC+8）—— allrequest in 
//! ， not 1 400（ `core::degrade`）。
//! `degradeActive` / `degradeUntil` use this ；`clearDegrade`
//! is no has 1 （ can etc to ）： use 
//! （e.g. to `custom`） after ， not this in 。
//!
//! ── Why and /api/sanitize ───────────────────────
//! client and is not ，config。
//!
//! ── fileWhy in （ and not ）─────────────
//! （`config::prompt_from`） to not to file will **default + 1 Reason**：
//! use not can as 1 file not 、 not can 。 then ——
//! use in this ，path need （400 + can Reason）， not is success after 
//! in up 「：default」 no 。

use axum::body::Bytes;
use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::config::{
    self, PromptSettings, ProviderPromptPatch, KEY_PROMPT_FILE, KEY_PROMPT_GATEWAY,
    KEY_PROMPT_GATEWAY_TEXT, KEY_PROMPT_MODE, KEY_PROMPT_PROVIDERS, KEY_PROMPT_TEXT,
};
use crate::server::core::degrade;
use crate::server::core::prompt::{GatewayBlocks, PromptMode};
use crate::server::core::providers;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// request（`PUT` ）
const KEY_CLEAR_DEGRADE: &str = "clearDegrade";

/// GET /api/prompt —— current / / 
pub async fn get_prompt(State(state): State<ServerState>) -> Response {
    ok_json(prompt_json(&config::current(), &state).await)
}

/// PUT /api/prompt —— body `{promptMode?, promptFile?, promptText?, promptProviders?,
/// promptGateway?, promptGatewayText?, clearDegrade?}`
///
/// （ ，**`null` **）—— and `/api/retry` 1 。
/// need ****file / empty（ or empty ）：`promptFile: ""`
/// 「default」，`promptText: ""` 「 this not need ，
/// file / default」。 this is need —— （`bridge.rs`） in 「」
/// when will up `promptFile: null`， if `null` ， 1 down 「」 will 
/// use path。
/// ：、 or new need use file not to → 400 and not 。
pub async fn put_prompt(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = match parse_body(&body) {
        Ok(value) => value,
        Err(error) => return errors::management_error(400, error.message),
    };
    let Some(object) = payload.as_object() else {
        return errors::management_error(400, "请求体必须是 JSON 对象");
    };
    let current = config::current();
    let settings = current.prompt_settings();

    // ── ──────────────────────────────────────────────
    let mode = match object.get(KEY_PROMPT_MODE) {
        // / null as 「 this 1 not 」
        None | Some(Value::Null) => settings.mode,
        Some(Value::String(text)) => match PromptMode::parse(text) {
            Some(mode) => mode,
            None => {
                return errors::management_error(
                    400,
                    format!(
                        "{KEY_PROMPT_MODE} 只认 {}（收到: {text}）",
                        allowed_modes()
                    ),
                )
            }
        },
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_MODE} 必须是字符串（收到: {value}）"),
            )
        }
    };

    // ── file（empty = ，null = not ）─────────────────
    let file = match object.get(KEY_PROMPT_FILE) {
        None | Some(Value::Null) => settings.file.clone(),
        Some(Value::String(text)) => Some(text.clone()).filter(|text| !text.trim().is_empty()),
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_FILE} 必须是字符串或 null（收到: {value}）"),
            )
        }
    };
    // ── （empty = ，null = not ）───────────────────
    // file，** not file** this in empty when （ down that ）：
    // use in up ，file not and ，path not this down this 。
    let inline = match object.get(KEY_PROMPT_TEXT) {
        None | Some(Value::Null) => settings.inline.clone(),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(text.clone())
            }
        }
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_TEXT} 必须是字符串或 null（收到: {value}）"),
            )
        }
    };
    // new need use file when ：passthrough not file（ and 1 ），
    // that when path not as ， down will 「、 after 」 use 
    if matches!(mode, PromptMode::Custom | PromptMode::Append) && inline.is_none() {
        if let Some(path) = file.as_deref() {
            if let Err(message) = config::read_prompt_file(path) {
                return errors::management_error(400, message);
            }
        }
    }

    // ── by （optional；）──────────────
    // ：`{"<providerId>": {"promptMode"?, "promptFile"?, "promptText"?} | null}`
    // （ and that ）； as `null` = ** this 1 **（default）。
    // is 「update」：`{"zcode": {"promptMode": "append"}}` this 1 ，
    // file and 。
    //
    // ： 1 request、 when before not this ——
    // and up 「」 is 1 。
    let mut provider_patches: Vec<(String, Option<ProviderPromptPatch>)> = Vec::new();
    match object.get(KEY_PROMPT_PROVIDERS) {
        None | Some(Value::Null) => {}
        Some(Value::Object(entries)) => {
            for (id, value) in entries {
                let id = id.trim();
                if id.is_empty() {
                    return errors::management_error(400, "提供商 id 不能为空串");
                }
                if !providers::is_known_provider_id(id) {
                    return errors::management_error(
                        400,
                        format!("未知的提供商 id：{id}（只能配已登记的提供商）"),
                    );
                }
                // is null → delete this 1 
                if value.is_null() {
                    provider_patches.push((id.to_string(), None));
                    continue;
                }
                let Some(entry) = value.as_object() else {
                    return errors::management_error(
                        400,
                        format!("{KEY_PROMPT_PROVIDERS}.{id} 必须是对象或 null（收到: {value}）"),
                    );
                };
                // this 1 ****； this 1 no when by ****
                // —— is 「 to this file」 not 1 
                let previous = settings.providers.get(id);
                let entry_mode = match entry.get(KEY_PROMPT_MODE) {
                    None | Some(Value::Null) => previous
                        .map(|item| item.mode)
                        .unwrap_or(settings.mode),
                    Some(Value::String(text)) => match PromptMode::parse(text) {
                        Some(mode) => mode,
                        None => {
                            return errors::management_error(
                                400,
                                format!(
                                    "{KEY_PROMPT_PROVIDERS}.{id}.{KEY_PROMPT_MODE} 只认 {}（收到: {text}）",
                                    allowed_modes()
                                ),
                            )
                        }
                    },
                    Some(other) => {
                        return errors::management_error(
                            400,
                            format!(
                                "{KEY_PROMPT_PROVIDERS}.{id}.{KEY_PROMPT_MODE} 必须是字符串（收到: {other}）"
                            ),
                        )
                    }
                };
                let entry_file = match entry.get(KEY_PROMPT_FILE) {
                    None | Some(Value::Null) => previous.and_then(|item| item.file.clone()),
                    Some(Value::String(text)) => {
                        Some(text.clone()).filter(|text| !text.trim().is_empty())
                    }
                    Some(other) => {
                        return errors::management_error(
                            400,
                            format!(
                                "{KEY_PROMPT_PROVIDERS}.{id}.{KEY_PROMPT_FILE} 必须是字符串或 null（收到: {other}）"
                            ),
                        )
                    }
                };
                // ： 1 「update」（ = this 1 ；
                // empty = this 、file / default）
                let entry_inline = match entry.get(KEY_PROMPT_TEXT) {
                    None | Some(Value::Null) => previous.and_then(|item| item.inline.clone()),
                    Some(Value::String(text)) => {
                        if text.trim().is_empty() {
                            None
                        } else {
                            Some(text.clone())
                        }
                    }
                    Some(other) => {
                        return errors::management_error(
                            400,
                            format!(
                                "{KEY_PROMPT_PROVIDERS}.{id}.{KEY_PROMPT_TEXT} 必须是字符串或 null（收到: {other}）"
                            ),
                        )
                    }
                };
                // has when file not and （ and that 1 ， up ）
                if matches!(entry_mode, PromptMode::Custom | PromptMode::Append)
                    && entry_inline.is_none()
                {
                    if let Some(path) = entry_file.as_deref() {
                        if let Err(message) = config::read_prompt_file(path) {
                            return errors::management_error(400, format!("{id}: {message}"));
                        }
                    }
                }
                provider_patches.push((
                    id.to_string(),
                    Some(ProviderPromptPatch {
                        mode: entry_mode,
                        file: entry_file,
                        inline: entry_inline,
                    }),
                ));
            }
        }
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_PROVIDERS} 必须是对象或 null（收到: {value}）"),
            )
        }
    }

    // ── （ 1 ， is ）──────────
    // ** / `null`** = this 1 not （ in no has this 1 when `null`，
    // and up `promptMode` 1 ）； `null` = to default（）。
    // and up / file**** by `KEY_PROMPT_GATEWAY`：
    // 1 down not this this 。
    let mut gateway_patches: Vec<(String, Option<bool>)> = Vec::new();
    match object.get(KEY_PROMPT_GATEWAY) {
        None | Some(Value::Null) => {}
        Some(Value::Object(entries)) => {
            for (id, value) in entries {
                let id = id.trim();
                if id.is_empty() {
                    return errors::management_error(400, "提供商 id 不能为空串");
                }
                if !providers::is_known_provider_id(id) {
                    return errors::management_error(
                        400,
                        format!("未知的提供商 id：{id}（只能配已登记的提供商）"),
                    );
                }
                match value {
                    Value::Null => gateway_patches.push((id.to_string(), None)),
                    Value::Bool(flag) => gateway_patches.push((id.to_string(), Some(*flag))),
                    other => {
                        return errors::management_error(
                            400,
                            format!(
                                "{KEY_PROMPT_GATEWAY}.{id} 必须是 true / false / null（收到: {other}）"
                            ),
                        )
                    }
                }
            }
        }
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_GATEWAY} 必须是对象或 null（收到: {value}）"),
            )
        }
    }

    // ── ****（： not / ）──────
    // `{"<providerId>": {"identity"?, "stable"?, "dynamic"?} | null}`：
    // `null` = this 1 to ； is **update**（ 
    // ，empty = this 1 、 to ）。 and config、 and `GatewayBlocks`
    // （`GatewayBlocks::FIELDS`）， has 1 。
    let mut gateway_text_patches: Vec<(String, Option<GatewayBlocks>)> = Vec::new();
    match object.get(KEY_PROMPT_GATEWAY_TEXT) {
        None | Some(Value::Null) => {}
        Some(Value::Object(entries)) => {
            for (id, value) in entries {
                let id = id.trim();
                if id.is_empty() {
                    return errors::management_error(400, "提供商 id 不能为空串");
                }
                if !providers::is_known_provider_id(id) {
                    return errors::management_error(
                        400,
                        format!("未知的提供商 id：{id}（只能配已登记的提供商）"),
                    );
                }
                if value.is_null() {
                    gateway_text_patches.push((id.to_string(), None));
                    continue;
                }
                let Some(entry) = value.as_object() else {
                    return errors::management_error(
                        400,
                        format!("{KEY_PROMPT_GATEWAY_TEXT}.{id} 必须是对象或 null（收到: {value}）"),
                    );
                };
                let mut blocks = settings
                    .gateway_text
                    .get(id)
                    .cloned()
                    .unwrap_or_default();
                for field in GatewayBlocks::FIELDS {
                    match entry.get(field) {
                        // = this 1 not （config ）
                        None | Some(Value::Null) => {}
                        Some(Value::String(text)) => blocks.set(field, text.clone()),
                        Some(other) => {
                            return errors::management_error(
                                400,
                                format!(
                                    "{KEY_PROMPT_GATEWAY_TEXT}.{id}.{field} 必须是字符串或 null（收到: {other}）"
                                ),
                            )
                        }
                    }
                }
                // empty = no has （ will this 1 from config）
                gateway_text_patches.push((id.to_string(), Some(blocks)));
            }
        }
        Some(value) => {
            return errors::management_error(
                400,
                format!("{KEY_PROMPT_GATEWAY_TEXT} 必须是对象或 null（收到: {value}）"),
            )
        }
    }

    // ── （optional， and up config）──────────────
    let clear_degrade = object
        .get(KEY_CLEAR_DEGRADE)
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if clear_degrade && degrade::active() {
        degrade::clear();
        logging::log(
            "[Config]",
            "内容拦截降级已手动解除：后续请求恢复本模式自己的系统提示词",
        );
    }

    // ── ─────────────────────────────────────────────
    // noneconfig has no has 1 ： **file ** new 
    // 1 （ use file、 or file， 1 PUT can ）。
    if !config::set_prompt(mode, file, inline) {
        logging::log(
            "[Config]",
            "⚠️  系统提示词设置写入失败，本次运行内仍生效",
        );
    }
    // ： 1 1 （failure that 1 ， its ）
    for (id, patch) in &provider_patches {
        if !config::set_prompt_provider(id, patch.clone()) {
            logging::log(
                "[Config]",
                &format!("⚠️  提供商 {id} 的提示词设置写入失败，本次运行内仍生效"),
            );
        }
    }
    // ： 1 1 
    for (id, flag) in &gateway_patches {
        if !config::set_prompt_gateway(id, *flag) {
            logging::log(
                "[Config]",
                &format!("⚠️  提供商 {id} 的网关提示词开关写入失败，本次运行内仍生效"),
            );
        }
    }
    // ： 1 1 
    for (id, blocks) in &gateway_text_patches {
        if !config::set_prompt_gateway_text(id, blocks.clone()) {
            logging::log(
                "[Config]",
                &format!("⚠️  提供商 {id} 的网关提示词正文写入失败，本次运行内仍生效"),
            );
        }
    }
    let after = config::current();
    let next = after.prompt_settings();
    // in ** ** when ： that 1 PUT will to this ，
    // 1 config no —— as 1 「 already update」 is 。
    // and this ：otherwise「 zcode 1 」 in 1 no has 。
    // and is **** and ： 1 （ use will 
    // for 「 that to down no has 」）。
    let changed = next.mode != settings.mode
        || next.file != settings.file
        || next.inline != settings.inline
        || next.source != settings.source
        || next.file_error != settings.file_error
        || !provider_patches.is_empty()
        || !gateway_patches.is_empty()
        || !gateway_text_patches.is_empty();
    if changed {
        let mut detail = format!("{}（{}）", next.mode.label(), next.source.label());
        if !provider_patches.is_empty() {
            let names: Vec<String> = provider_patches
                .iter()
                .map(|(id, patch)| match patch {
                    Some(patch) => {
                        let mut marks = vec![format!("模式={}", patch.mode.as_str())];
                        if patch.inline.is_some() {
                            marks.push("界面正文".to_string());
                        }
                        format!("{id}（{}）", marks.join("/"))
                    }
                    None => format!("{id}=默认"),
                })
                .collect();
            detail.push_str(&format!("；按提供商：{}", names.join(", ")));
        }
        if !gateway_patches.is_empty() {
            // is ** use up need ** ，must（ when 
            // 「Why this 405」 1 is ）
            let names: Vec<String> = gateway_patches
                .iter()
                .map(|(id, flag)| match flag {
                    Some(true) => format!("{id}=装"),
                    Some(false) => format!("{id}=**已关闭**"),
                    None => format!("{id}=默认"),
                })
                .collect();
            detail.push_str(&format!("；网关自带提示词：{}", names.join(", ")));
            if gateway_patches.iter().any(|(_, flag)| *flag == Some(false)) {
                detail.push_str("（关闭后能否通过上游校验取决于上游当前口径）");
            }
        }
        if !gateway_text_patches.is_empty() {
            // need ： is up need ，「 1 」 and 
            // 「」 in when not 
            let names: Vec<String> = gateway_text_patches
                .iter()
                .map(|(id, blocks)| match blocks {
                    Some(blocks) => {
                        let edited: Vec<&str> = GatewayBlocks::FIELDS
                            .into_iter()
                            .filter(|field| !blocks.get(field).trim().is_empty())
                            .collect();
                        format!("{id}=改{}段", edited.join("/"))
                    }
                    None => format!("{id}=官方原文"),
                })
                .collect();
            detail.push_str(&format!("；网关自带提示词正文：{}", names.join(", ")));
        }
        if let Some(reason) = next.file_error.as_deref() {
            detail.push_str(&format!("；注意: {reason}"));
        }
        logging::log("[Config]", &format!("系统提示词已更新: {detail}"));
    }
    ok_json(prompt_json(&after, &state).await)
}

/// response（GET and PUT use ； before use response new ， not GET 1 ）。
///
/// for before is ****（`settings-panel.js` for ）， use 
/// not is （ and `retry_json` 1 ）。
async fn prompt_json(config: &config::RuntimeConfig, state: &ServerState) -> Value {
    let settings: &PromptSettings = config.prompt_settings();
    json!({
        KEY_PROMPT_MODE: settings.mode.as_str(),
        KEY_PROMPT_FILE: settings.file.clone().unwrap_or_default(),
        // ****（`passthrough` down is empty）。 this 1 ：
        // when ， 1 「」（ inline）。
        // not —— can need is this ，「」 in set this 1 
        // （ / new when request 1 ） not is 。
        KEY_PROMPT_TEXT: settings.text,
        // ：none（passthrough）/ builtin（default）/ file（file）/
        // inline（ that ）
        "promptSource": settings.source.as_str(),
        // file not to when Reason（default，Reason in this to use ）
        "promptFileError": settings.file_error,
        // （「N 」 use ； up promptText）
        "promptLines": settings.text.lines().count(),
        // （ `core::degrade`）：active = in 
        "degradeActive": degrade::active(),
        "degradeUntil": degrade::until_ms(),
        "degradeUntilText": degrade::until_text(),
        // ── by （ has in this ）──────────────
        // and up ****： use 1 ， not 1 
        KEY_PROMPT_PROVIDERS: providers_json(settings),
        // ：`{"<id>": true|false}`，** **
        // （ = default）。， by 「」。
        KEY_PROMPT_GATEWAY: gateway_json(settings),
        // ：`{"<id>": {identity?, stable?, dynamic?}}`，
        // ** **（ = use ， and `GatewayBlocks::or`）。
        // in `promptProviderOptions[].gatewayBlocks` to （
        // ）， this 1 down that —— and config 1 。
        KEY_PROMPT_GATEWAY_TEXT: gateway_text_json(settings),
        // can config （ down use ）： 1 no 
        "promptProviderOptions": provider_options(state).await,
    })
}

/// JSON ：`{ "<id>": {mode, file, promptText, promptSource, promptLines, promptFileError} }`
fn providers_json(settings: &PromptSettings) -> Value {
    let mut map = serde_json::Map::new();
    for (id, entry) in &settings.providers {
        map.insert(
            id.clone(),
            json!({
                KEY_PROMPT_MODE: entry.mode.as_str(),
                KEY_PROMPT_FILE: entry.file.clone().unwrap_or_default(),
                KEY_PROMPT_TEXT: entry.text,
                "promptSource": entry.source.as_str(),
                "promptLines": entry.text.lines().count(),
                "promptFileError": entry.file_error,
            }),
        );
    }
    Value::Object(map)
}

/// JSON （ ； `KEY_PROMPT_GATEWAY`）
fn gateway_json(settings: &PromptSettings) -> Value {
    let mut map = serde_json::Map::new();
    for (id, flag) in &settings.gateway {
        map.insert(id.clone(), Value::Bool(*flag));
    }
    Value::Object(map)
}

/// JSON （ and ； `KEY_PROMPT_GATEWAY_TEXT`）
fn gateway_text_json(settings: &PromptSettings) -> Value {
    let mut map = serde_json::Map::new();
    for (id, blocks) in &settings.gateway_text {
        let mut one = serde_json::Map::new();
        for field in GatewayBlocks::FIELDS {
            let text = blocks.get(field);
            if !text.trim().is_empty() {
                one.insert(field.to_string(), Value::String(text.to_string()));
            }
        }
        map.insert(id.clone(), Value::Object(one));
    }
    Value::Object(map)
}

/// can config ：
/// （`[{id,label,gatewayNote,gatewayChars,gatewayBlocks}]`）。
///
/// · `gatewayNote` empty = this has ； in or already has when ；
/// · `gatewayChars` = that （ 「 N 」）；
/// · `gatewayBlocks` = that ****（；Environment 
/// `{cwd}` this ）。 and 「」 。
///
/// ── in by **** ────────────────────────────────
/// and 「 up Why need 」 is up ， to 
/// （ has ZCode 1 has ）。 this `prompt.info`
/// ； not in when is null/0， is not that 1 —— that e.g.
/// ， 1 when （ use 1 etc ）。
async fn provider_options(state: &ServerState) -> Value {
    let mut items = Vec::with_capacity(providers::PROVIDERS.len());
    for meta in providers::PROVIDERS.iter() {
        let info = plugin_prompt_info(state, meta.id).await;
        items.push(json!({
            "id": meta.id,
            "label": meta.label,
            "gatewayNote": info.as_ref().and_then(|info| info.get("gatewayNote")).cloned()
                .unwrap_or(Value::Null),
            "gatewayChars": info.as_ref().and_then(|info| info.get("gatewayChars")).cloned()
                .unwrap_or(Value::Null),
            "gatewayBlocks": info.as_ref().and_then(|info| info.get("gatewayBlocks")).cloned()
                .unwrap_or(Value::Null),
        }));
    }
    Value::Array(items)
}

/// need this 1 （`None` = this no has / not in ）。
///
/// ── Why for 1 1 ──────────────────────────────────
/// 「 has 」 this in （， not 
/// ）。 1 will —— 1 ， 1 。
/// 1 RPC 1 「 no has 」， can ignore（ this in set when 1 ）。
async fn plugin_prompt_info(state: &ServerState, provider: &str) -> Option<Value> {
    let adapter = crate::server::core::providers::adapter::adapter_by_id(provider)?;
    let info = adapter
        .admin_action(state.store(), "", "prompt.info", &Value::Null)
        .await
        .ok()?;
    // no to （empty for ） etc 「 this no has 」
    let object = info.as_object()?;
    if object
        .get("gatewayNote")
        .and_then(Value::as_str)
        .map_or(true, |note| note.trim().is_empty())
    {
        return None;
    }
    Some(info)
}

/// 400 
fn allowed_modes() -> String {
    PromptMode::ALL
        .iter()
        .map(|mode| mode.as_str())
        .collect::<Vec<_>>()
        .join(" / ")
}
