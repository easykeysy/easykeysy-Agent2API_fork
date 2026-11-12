//! （「」）： and config and 。
//!
//! ── this is ────────────────────────────────────────────────
//! in 「Agent config」（client） and 「」add 1 ： for 
//! optional can ，client can by can ——
//! · Skill： Agent Skill， by 1 to client skills directory；
//! · MCP：Streamable HTTP ， in client MCP config can 。
//! 1 up config， and when by 「 × 」。
//!
//! 网络搜索另有一个固定的 REST 接口（`/api/web-search/*`）： is not is 
//! can ， not 、 not ， in 「」， when 
//! `api::tool_runtime`。
//!
//! ── config in ──────────────────────────────────────────────
//! config in config `toolServices` down （kv 1 ），
//! [`ToolConfig`]。Why not ： this is set， `config` 
//! 「 / / config」 has ， not as 1 schema
//! 。 has [`save`]， `config::update_raw_field` 。
//!
//! ── ────────────────────────────────────────────────────
//! API Key and ： before use `crypto`（AES-256-GCM，
//! `enc1:` ），path in request when 。 API 
//! `configured` and `key_hint`， not 。
//!
//! ── ──────────────────────────────────────
//! config is 「 + up + etc + use Key」。 when use Key
//! `/v1/chat/completions`，、、 and request
//! use （ and OmniProxy 1 ）。

mod account_search;
mod bridge;
mod install;
mod mcp;
mod normalize;
mod providers;
mod search;
mod skill_package;
mod vision;

pub use bridge::VISION_BRIDGE_SCRIPT;
pub use install::{
    install_mcp, install_skill, mcp_targets, skill_targets, uninstall_mcp, uninstall_skill,
    InstallOutcome,
};
pub use mcp::{dispatch as mcp_dispatch, McpRequestContext, McpServerKind, McpSpec};
pub use account_search::test_provider;
pub use search::{batch_items, run_fetch, run_search, SearchError, SearchErrorCode};
pub use skill_package::{
    build_skill_files, is_skill_variant, recommended_skill_variant, skill_variants, SkillFile,
    SkillVariant, WEB_SEARCH_SKILL_NAME, WEB_SEARCH_SKILL_VERSION,
};
pub use vision::{
    clear_vision_config, is_reasoning_level, run_vision, save_vision_config, test_vision,
    vision_config, vision_key_options, VisionConfig, VISION_MAX_IMAGES, VISION_MAX_PROMPT_LENGTH,
    VISION_SERVICE_NAME, VISION_SERVICE_VERSION,
};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::crypto::{self, SecretScope};

/// config in config （kv 1 ）
pub const KEY_TOOL_SERVICES: &str = "toolServices";

/// id（）
pub const SERVICE_IDS: [&str; 2] = [WEB_SEARCH_SERVICE_ID, VISION_SERVICE_ID];
pub const WEB_SEARCH_SERVICE_ID: &str = "web-search";
pub const VISION_SERVICE_ID: &str = "vision";

/// （）。
///
/// API not in this ： is for （`/api/web-search/*`）， not and 
/// 「」 ， in 「」； when by 
/// [`endpoint`] and `tool_runtime` 。
pub const PROTOCOLS: [&str; 2] = ["skill", "mcp"];

/// when supports and ；supports，default not in 。
pub const API_KEY_PROVIDERS: [&str; 4] = ["tavily", "querit", "firecrawl", "ollama"];
pub const ACCOUNT_PROVIDERS: [&str; 3] = ["workbuddy", "workbuddy-intl", "kiro"];
pub const SEARCH_PROVIDERS: [&str; 7] = ["tavily", "querit", "firecrawl", "ollama", "workbuddy", "workbuddy-intl", "kiro"];

/// （ by before by id ， after ）
pub const WEB_SEARCH_PACKAGE: &str = WEB_SEARCH_SKILL_NAME;
pub const VISION_PACKAGE: &str = VISION_SERVICE_NAME;
pub const WEB_SEARCH_VERSION: &str = WEB_SEARCH_SKILL_VERSION;
pub const VISION_VERSION: &str = VISION_SERVICE_VERSION;

pub const CAPABILITY_COUNTS: [(&str, u32); 2] = [
    (WEB_SEARCH_SERVICE_ID, 2),
    (VISION_SERVICE_ID, 1),
];

/// when （seconds）： 1 use ；serverlimit， MCP when client
pub const TIMEOUT_MIN_SECONDS: u64 = 5;
pub const TIMEOUT_MAX_SECONDS: u64 = 600;
pub const DEFAULT_TIMEOUTS: [(&str, u64); 2] = [(WEB_SEARCH_SERVICE_ID, 30), (VISION_SERVICE_ID, 60)];

/// path（： will already client）。
///
/// `"api"` not [`PROTOCOLS`]， this ：REST is ，
/// by `tool_runtime` use ， not is can 「 × 」。
pub fn endpoint(service: &str, protocol: &str) -> &'static str {
    match (service, protocol) {
        (WEB_SEARCH_SERVICE_ID, "skill") => "/skill/web-search",
        (WEB_SEARCH_SERVICE_ID, "mcp") => "/mcp/web-search",
        (WEB_SEARCH_SERVICE_ID, "api") => "/api/web-search",
        (VISION_SERVICE_ID, "mcp") => "/mcp/vision",
        _ => "",
    }
}

pub fn is_service(value: &str) -> bool {
    SERVICE_IDS.contains(&value)
}

pub fn is_protocol(value: &str) -> bool {
    PROTOCOLS.contains(&value)
}

/// this supports （ PROTOCOLS ）
pub fn service_protocols(service: &str) -> Vec<&'static str> {
    PROTOCOLS
        .iter()
        .copied()
        .filter(|protocol| !endpoint(service, protocol).is_empty())
        .collect()
}

pub fn capability_count(service: &str) -> u32 {
    CAPABILITY_COUNTS
        .iter()
        .find(|(id, _)| *id == service)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

pub fn package_name(service: &str) -> &'static str {
    match service {
        WEB_SEARCH_SERVICE_ID => WEB_SEARCH_PACKAGE,
        VISION_SERVICE_ID => VISION_PACKAGE,
        _ => "",
    }
}

pub fn version(service: &str) -> &'static str {
    match service {
        WEB_SEARCH_SERVICE_ID => WEB_SEARCH_VERSION,
        VISION_SERVICE_ID => VISION_VERSION,
        _ => "",
    }
}

/* ─── config ─────────────────────────────────────────────── */

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderConfig {
    /// `enc1:` ；empty = not configured
    pub encrypted_key: String,
    pub key_hint: String,
    /// use ， by and 。
    pub account_id: String,
    pub enabled: bool,
    pub updated_at: i64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self { encrypted_key: String::new(), key_hint: String::new(), account_id: String::new(), enabled: false, updated_at: 0 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Selection {
    pub search: Vec<String>,
    pub fetch: String,
}

impl Default for Selection {
    fn default() -> Self {
        Self { search: API_KEY_PROVIDERS.iter().map(|item| item.to_string()).collect(), fetch: "tavily".into() }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ToolConfig {
    /// service → protocol → 
    pub enabled: BTreeMap<String, BTreeMap<String, bool>>,
    /// service → when seconds
    pub timeouts: BTreeMap<String, u64>,
    /// provider → and enabled
    pub providers: BTreeMap<String, ProviderConfig>,
    pub selection: Selection,
    pub vision: Option<VisionConfig>,
}

impl ToolConfig {
    /// （ old config / JSON this 1 ）
    fn normalized(mut self) -> Self {
        for service in SERVICE_IDS {
            let entry = self.enabled.entry(service.to_string()).or_default();
            for protocol in service_protocols(service) {
                entry.entry(protocol.to_string()).or_insert(true);
            }
            let fallback = DEFAULT_TIMEOUTS
                .iter()
                .find(|(id, _)| *id == service)
                .map(|(_, value)| *value)
                .unwrap_or(30);
            self.timeouts.entry(service.to_string()).or_insert(fallback);
        }
        for provider in SEARCH_PROVIDERS {
            self.providers.entry(provider.to_string()).or_default();
        }
        self.selection.search.retain(|item| is_search_provider(item));
        if self.selection.search.is_empty() {
            self.selection.search = Selection::default().search;
        }
        if !is_fetch_provider(&self.selection.fetch) {
            self.selection.fetch = "tavily".into();
        }
        self
    }
}

pub fn load() -> ToolConfig {
    config::current()
        .raw()
        .get(KEY_TOOL_SERVICES)
        .map(parse_stored)
        .unwrap_or_default()
        .normalized()
}

fn parse_stored(value: &serde_json::Value) -> ToolConfig {
    if value.is_null() {
        return ToolConfig::default();
    }
    serde_json::from_value::<ToolConfig>(value.clone()).unwrap_or_default().normalized()
}

/// in config 1 ： when when not will （ after before ）
fn update(mutate: impl FnOnce(&mut ToolConfig)) -> Result<ToolConfig, String> {
    let mut next: Option<ToolConfig> = None;
    let saved = crate::server::config::update_raw_field_with(KEY_TOOL_SERVICES, |value| {
        let mut current = parse_stored(value);
        mutate(&mut current);
        let current = current.normalized();
        if let Ok(serialized) = serde_json::to_value(&current) {
            *value = serialized;
            next = Some(current);
        }
    });
    match (saved, next) {
        (true, Some(config_value)) => Ok(config_value),
        _ => Err("工具服务配置写入失败".into()),
    }
}

/* ─── and when ───────────────────────────────────────────── */

pub fn is_enabled(service: &str, protocol: &str) -> bool {
    if endpoint(service, protocol).is_empty() {
        return false;
    }
    // API is for ， not is 「」 can ；
    // endpoint and when by ， old config api 1 not 。
    if protocol == "api" {
        return true;
    }
    load()
        .enabled
        .get(service)
        .and_then(|entry| entry.get(protocol))
        .copied()
        .unwrap_or(true)
}

pub fn set_enabled(service: &str, protocol: &str, enabled: bool) -> Result<Value, String> {
    if endpoint(service, protocol).is_empty() {
        return Err(format!("服务 {service} 不支持 {protocol} 协议"));
    }
    update(|config_value| {
        config_value
            .enabled
            .entry(service.to_string())
            .or_default()
            .insert(protocol.to_string(), enabled);
    })?;
    Ok(service_info(service, protocol))
}

pub fn timeout_seconds(service: &str) -> u64 {
    let value = load().timeouts.get(service).copied().unwrap_or(30);
    value.clamp(TIMEOUT_MIN_SECONDS, TIMEOUT_MAX_SECONDS)
}

pub fn set_timeout(service: &str, seconds: u64) -> Result<Value, String> {
    if !is_service(service) {
        return Err("未知的工具服务".into());
    }
    if !(TIMEOUT_MIN_SECONDS..=TIMEOUT_MAX_SECONDS).contains(&seconds) {
        return Err(format!(
            "超时必须是 {TIMEOUT_MIN_SECONDS}~{TIMEOUT_MAX_SECONDS} 秒之间的整数"
        ));
    }
    update(|config_value| {
        config_value.timeouts.insert(service.to_string(), seconds);
    })?;
    Ok(service_info(service, service_protocols(service)[0]))
}

/* ─── ─────────────────────────────────────────────── */

pub fn service_info(service: &str, protocol: &str) -> Value {
    let mut info = json!({
        "id": service,
        "protocol": protocol,
        "key": format!("{service}:{protocol}"),
        "enabled": is_enabled(service, protocol),
        "capability_count": capability_count(service),
        "endpoint": endpoint(service, protocol),
        "package_name": package_name(service),
        "version": version(service),
        "timeout_seconds": timeout_seconds(service),
    });
    if service == WEB_SEARCH_SERVICE_ID && protocol == "skill" {
        info["recommended_variant"] = json!(recommended_skill_variant());
        info["variants"] = serde_json::to_value(skill_variants()).unwrap_or(Value::Null);
    }
    info
}

pub fn list_services(protocol: Option<&str>) -> Vec<Value> {
    let mut rows = Vec::new();
    for service in SERVICE_IDS {
        for supported in service_protocols(service) {
            if protocol.is_none() || protocol == Some(supported) {
                rows.push(service_info(service, supported));
            }
        }
    }
    rows
}

/* ─── ───────────────────────────────────────────── */

pub fn is_search_provider(value: &str) -> bool {
    SEARCH_PROVIDERS.contains(&value)
}

pub fn is_account_provider(value: &str) -> bool {
    ACCOUNT_PROVIDERS.contains(&value)
}

pub fn is_fetch_provider(value: &str) -> bool {
    API_KEY_PROVIDERS.contains(&value)
}

pub fn provider_display_name(provider: &str) -> &'static str {
    match provider {
        "tavily" => "Tavily",
        "querit" => "Querit",
        "firecrawl" => "Firecrawl",
        "ollama" => "Ollama Web Search",
        "workbuddy" => "WorkBuddy 国内版",
        "workbuddy-intl" => "WorkBuddy 国际版",
        "kiro" => "Kiro",
        _ => "",
    }
}

pub fn key_hint(value: &str) -> String {
    if value.chars().count() <= 12 {
        return format!("{}••••", value.chars().take(4).collect::<String>());
    }
    let head: String = value.chars().take(7).collect();
    let tail: String = value.chars().skip(value.chars().count().saturating_sub(4)).collect();
    format!("{head}••••{tail}")
}

pub async fn list_providers(store: &AccountStore) -> Value {
    let config_value = load();
    let mut rows: Vec<Value> = API_KEY_PROVIDERS.iter().map(|provider| {
        let entry = config_value.providers.get(*provider).cloned().unwrap_or_default();
        json!({
            "provider": provider,
            "display_name": provider_display_name(provider),
            "auth_type": "api_key",
            "supports_search": true,
            "supports_fetch": true,
            "available": true,
            "configured": !entry.encrypted_key.is_empty(),
            "key_hint": if entry.key_hint.is_empty() { Value::Null } else { json!(entry.key_hint) },
            "enabled": entry.enabled,
            "updated_at": if entry.updated_at > 0 { json!(entry.updated_at) } else { Value::Null },
        })
    }).collect();
    let account_rows = futures::future::join_all(ACCOUNT_PROVIDERS.iter().map(|provider| {
        let entry = config_value.providers.get(*provider).cloned().unwrap_or_default();
        account_search::provider_row(store, provider, entry)
    })).await;
    rows.extend(account_rows);
    Value::Array(rows)
}

pub fn provider_selection() -> Value {
    let config_value = load();
    json!({
        "search_providers": config_value.selection.search,
        "fetch_provider": config_value.selection.fetch,
    })
}

pub fn update_provider(
    store: &AccountStore,
    provider: &str,
    api_key: Option<&str>,
    enabled: Option<bool>,
    account_id: Option<&str>,
) -> Result<Value, String> {
    if !is_search_provider(provider) {
        return Err("未知的搜索提供商".into());
    }
    if is_account_provider(provider) {
        if api_key.is_some_and(|key| !key.trim().is_empty()) {
            return Err("该搜索来源使用已有账号，不接受 API Key".into());
        }
        let account_id = account_id.map(str::trim);
        if let Some(id) = account_id.filter(|id| !id.is_empty()) {
            account_search::validate_account(store, provider, id).map_err(|error| error.message)?;
        }
        let config_value = update(|config_value| {
            let entry = config_value.providers.entry(provider.to_string()).or_default();
            if let Some(id) = account_id { entry.account_id = id.to_string(); }
            if let Some(enabled) = enabled { entry.enabled = enabled; }
            entry.updated_at = crate::server::logging::now_ms();
        })?;
        let entry = config_value.providers.get(provider).cloned().unwrap_or_default();
        return Ok(json!({ "success": true, "provider": provider, "account_id": entry.account_id }));
    }
    if account_id.is_some() {
        return Err("该搜索来源使用 API Key，不接受账号绑定".into());
    }
    let api_key = api_key.map(str::trim).filter(|value| !value.is_empty());
    let encrypted = match api_key {
        Some(key) => {
            let sealed = crypto::seal(key.as_bytes(), SecretScope::Accounts)
                .map_err(|_| "无法加密 API Key".to_string())?;
            Some((sealed, key_hint(key)))
        }
        None => None,
    };
    let config_value = update(|config_value| {
        let entry = config_value.providers.entry(provider.to_string()).or_default();
        if let Some((sealed, hint)) = encrypted {
            entry.encrypted_key = sealed;
            entry.key_hint = hint;
            entry.updated_at = crate::server::logging::now_ms();
        }
        if let Some(enabled) = enabled {
            entry.enabled = enabled;
        }
    })?;
    let entry = config_value.providers.get(provider).cloned().unwrap_or_default();
    Ok(json!({
        "success": true,
        "provider": provider,
        "configured": !entry.encrypted_key.is_empty(),
        "key_hint": if entry.key_hint.is_empty() { Value::Null } else { json!(entry.key_hint) },
    }))
}

pub fn clear_provider_key(provider: &str) -> bool {
    if !API_KEY_PROVIDERS.contains(&provider) {
        return false;
    }
    update(|config_value| {
        let entry = config_value.providers.entry(provider.to_string()).or_default();
        entry.encrypted_key.clear();
        entry.key_hint.clear();
        entry.updated_at = crate::server::logging::now_ms();
    })
    .is_ok()
}

/// currentconfig（ after ；onlyserver use ， in /response in ）
pub fn provider_key(provider: &str) -> Option<String> {
    decrypt_provider_key(load().providers.get(provider)?)
}

fn decrypt_provider_key(entry: &ProviderConfig) -> Option<String> {
    if entry.encrypted_key.is_empty() {
        return None;
    }
    let bytes = crypto::open(&entry.encrypted_key, SecretScope::Accounts).ok()?;
    String::from_utf8(bytes).ok().filter(|value| !value.is_empty())
}

pub fn configured_provider(provider: &str) -> bool {
    load()
        .providers
        .get(provider)
        .is_some_and(|entry| !entry.encrypted_key.is_empty() && entry.enabled)
}

/// already enabled and already config 
pub fn ready_providers() -> Vec<&'static str> {
    let config_value = load();
    SEARCH_PROVIDERS
        .iter()
        .copied()
        .filter(|provider| {
            config_value
                .providers
                .get(*provider)
                .is_some_and(|entry| entry.enabled && !entry.encrypted_key.is_empty())
        })
        .collect()
}

/// already enabled or not configured （， when return can configerror）
pub fn missing_selected_providers() -> Vec<&'static str> {
    let selected = load().selection.search;
    let ready = ready_providers();
    selected
        .iter()
        .filter(|provider| !ready.contains(&provider.as_str()))
        .map(|provider| provider_display_name(provider))
        .filter(|name| !name.is_empty())
        .collect()
}

pub fn update_selection(
    search: Option<Vec<String>>,
    fetch: Option<String>,
) -> Result<Value, String> {
    let mut next_search = None;
    if let Some(items) = search {
        if items.is_empty() {
            return Err("批量网页搜索至少选择一个提供商".into());
        }
        if items.iter().any(|item| !is_search_provider(item)) {
            return Err("搜索提供商包含未知项".into());
        }
        let mut seen = Vec::new();
        for item in items {
            if !seen.contains(&item) {
                seen.push(item);
            }
        }
        next_search = Some(seen);
    }
    if let Some(provider) = fetch.as_deref() {
        if !is_fetch_provider(provider) {
            return Err("该提供商不支持网页抓取".into());
        }
    }
    update(|config_value| {
        if let Some(items) = next_search {
            config_value.selection.search = items;
        }
        if let Some(provider) = fetch {
            config_value.selection.fetch = provider;
        }
    })?;
    Ok(provider_selection())
}
