//! will by （ for Node server.mjs 776-869、963-974 ）。
//!
//! GET /api/session before （ need ）
//! GET /api/session/login/methods and prompts（ need ）
//! POST /api/session/login/start （optional methodIndex / inputs / proxy）
//! GET /api/session/login/wait ?state=
//! POST /api/session/login/cancel （/ use ）
//! POST /api/session/login/callback （ or ）
//! GET /api/session/login/remote-callback in （）
//! POST /api/session/refresh new current token
//! POST /api/session/logout （current）
//! POST /auth/login （ etc response）
//! POST /auth/logout 
//!
//! ── （ and Node 1 ）────────────────────────────
//! will and protected； / in by 1 state 。
//! http.rs as ， methods and start 1 。
//!
//! ── （/api/session）────────────────────────────────
//! health ← auth.get_config_summary()（： is in ）
//! session ← auth.get_status()（ new ）
//!   accounts        ← store.list_accounts()
//! lastRequestModel← config.json lastRequestModel
//! routedAccountId ← by lastRequestModel 「 down 1 request will use 」（ its ）
//! defaultModel ← config.json defaultModel
//! proxies ← Clash need （ 3 as when ）
//! models ← directory（ 4 as ， new ）
//! sanitizeBlacklistFingerprints ← （ and GET /api/sanitize ）

use std::time::Duration;

use axum::body::Bytes;
use axum::extract::{Json, OriginalUri, Query, State};
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::api::health::UNCONFIGURED_REASON;
use crate::server::config;
use crate::server::core::login::LOGIN_TIMEOUT_MS;
use crate::server::errors::management_error;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// GET /api/session —— （ http.rs）
pub async fn get_session(State(state): State<ServerState>) -> Response {
    let snapshot = config::current();
    // can and by use 1 ， not and 。
    let accounts = state.store().list_visible_accounts();
    let routed = super::routing_api::queue_for_accounts(
        &state, snapshot.last_request_model().map(str::to_string), &crate::server::core::routing::accounts_of(&accounts),
    )
        .get("pickedId").and_then(Value::as_str).filter(|id| !id.is_empty())
        .map(|id| Value::String(id.to_string())).unwrap_or(Value::Null);
    let summary = state.auth().get_config_summary();
    let configured = summary
        .get("configured")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    ok_json(json!({
        "health": {
            "upstreamConfigured": configured,
            "upstreamBaseUrl": summary.get("baseUrl").cloned().unwrap_or(Value::Null),
            "authSource": summary.get("authSource").cloned().unwrap_or(Value::Null),
            "unavailableReason": if configured {
                Value::Null
            } else {
                summary
                    .get("unavailableReason")
                    .cloned()
                    .unwrap_or_else(|| Value::String(UNCONFIGURED_REASON.to_string()))
            },
        },
        "session": state.auth().get_status(),
        // accounts.currentAccountId = ** not ** （ = enabled + has available +
        // ， core::account_store）。「 1 is 」， 
        // （、 new ） 1 is ， not 。
        // ★ / 「」 need is ** down 1 request will use ** —— that is by 1 
        // request `routedAccountId`（ down ）， in when will not 。
        "accounts": accounts,
        // 1 request use ： 「」default
        "lastRequestModel": snapshot.last_request_model(),
        // and by use none will ， then and 。
        "routedAccountId": routed,
        "defaultModel": snapshot.default_model(),
        "proxies": proxies_summary(),
        // directory（ for server.mjs 802-804 {id, name, isDefault, credits}）。
        // is **directory**（`providers::catalog::session_models`）， and `GET /v1/models`
        // 1 and ： up must etc client can to ，otherwise use will 1 
        // by not to （ before this workbuddy ， in up not can ）。
        // provider / providerLabel， by 。
        "models": crate::server::core::providers::catalog::session_models(state.store()),
        // statistics is in after （ / /
        // ）。 can 1 ； not and 。
        "maintenance": { "running": state.startup_maintenance_running() },
        // （ and GET /api/sanitize 1 、 1 ）
        config::KEY_SANITIZE_FINGERPRINTS: snapshot.sanitize_fingerprints(),
        // **parameter**（`{provider: AccountFeatures}`）。
        //
        // ── Why in /api/session ──────────────────────────────
        // this ， 1 need use this parameter（ 
        // 、valid 、 need not need to ）。 in 1 response，
        // not 1 request， not will 「 already 、parameter no to 」 
        // in （ that will 1 down ）。
        //
        // no not in —— ， old 。
        "accountFeatures": account_features(),
    }))
}

/// already can 。
fn account_features() -> Value {
    Value::Object(crate::server::core::plugin_host::account_features())
}

fn proxies_summary() -> Value {
    let clash = crate::server::core::proxies::clash_proxy_options();
    json!({
        "clashAvailable": clash["available"],
        "clashError": clash["error"],
        "optionCount": clash.get("options").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
    })
}

// ─── GET /api/session/login/methods（protected）──────────────

// 。 HTML / raw / secret / inputs etc 
// not response； this all is JSON ， can by 。
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginMethodsMetadata {
    methods: Vec<LoginMethodMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginMethodMetadata {
    method_index: u64,
    #[serde(rename = "type")]
    kind: LoginMethodType,
    label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
    #[serde(default)]
    prompts: Vec<LoginPromptMetadata>,
    #[serde(default)]
    local_only: bool,
    #[serde(default)]
    manual_callback: bool,
    #[serde(default)]
    callback_input: LoginCallbackInput,
    /// need this OAuth in use （ not ）。
    ///
    /// used for in up （AStudio）： to will 
    /// use 1 no has 。 for oauth valid。
    #[serde(default)]
    embedded_only: bool,
}

#[derive(Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum LoginCallbackInput { #[default] Url, Code }

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum LoginMethodType { Oauth, Api }

#[derive(serde::Deserialize, serde::Serialize)]
struct LoginPromptMetadata {
    key: String,
    #[serde(rename = "type")]
    kind: LoginPromptType,
    label: String,
    #[serde(default)]
    placeholder: String,
    #[serde(default)]
    optional: bool,
    // default； not down default/value。
    #[serde(default)]
    secret: bool,
    #[serde(default, rename = "default", skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    when: Option<LoginPromptCondition>,
    #[serde(default)]
    options: Vec<LoginPromptOption>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum LoginPromptType { Text, Select }

#[derive(serde::Deserialize, serde::Serialize)]
struct LoginPromptCondition {
    key: String,
    op: String,
    value: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct LoginPromptOption {
    value: String,
    label: String,
}

struct SelectedLogin {
    /// methods and login.start use 1 ， not in new owner。
    adapter: std::sync::Arc<dyn crate::server::core::providers::adapter::ProviderAdapter>,
    proxy: Option<crate::server::core::login::LoginProxySelection>,
    options: Value,
    local_only: bool,
    manual_callback: bool,
    callback_input: LoginCallbackInput,
    instructions: Option<String>,
}

/// old （ no has `methodIndex`） `SelectedLogin`： as down 。
///
/// ── Why old need ──────────────────────────────────
/// `start_plugin_web_login` has in `selected` as `Some` when `options` to 
/// （`None` use is empty `options`）。 old not `manualCallback` /
/// `localOnly` —— that is ， old use no has ，default and before 
/// 1 ； 1 is `options.proxy`。
///
/// not to when return `None`， `start_plugin_web_login` has 
/// 「unknown / unsupported」path， not is in this error。
fn legacy_selected(
    provider: &str,
    proxy: Option<crate::server::core::login::LoginProxySelection>,
    options: Value,
) -> Option<SelectedLogin> {
    let adapter = crate::server::core::providers::adapter::adapter_by_id(provider)?;
    Some(SelectedLogin {
        adapter,
        proxy,
        options,
        local_only: false,
        manual_callback: false,
        callback_input: LoginCallbackInput::Url,
        instructions: None,
    })
}

async fn read_login_methods(
    state: &ServerState,
    adapter: &dyn crate::server::core::providers::adapter::ProviderAdapter,
) -> Result<Option<LoginMethodsMetadata>, crate::server::errors::GatewayError> {
    use crate::server::core::login::login_text;
    use crate::server::errors::GatewayError;
    let Some(value) = adapter.login_methods_metadata(state.store()).await? else {
        return Ok(None);
    };
    let value = if value.is_array() { json!({ "methods": value }) } else { value };
    let mut metadata: LoginMethodsMetadata = serde_json::from_value(value)
        .map_err(|_| GatewayError::with_status(502, "插件返回的登录方式元数据无法解析"))?;
    let mut indices = std::collections::HashSet::new();
    for method in &mut metadata.methods {
        // JSON number need can ；， not by 。
        if method.method_index > 9_007_199_254_740_991 || !indices.insert(method.method_index) {
            return Err(GatewayError::with_status(502, "插件登录方式索引无效或重复"));
        }
        if matches!(method.callback_input, LoginCallbackInput::Code) && !method.manual_callback {
            return Err(GatewayError::with_status(502, "code 回调输入只能用于手动登录回调"));
        }
        method.label = login_text(&method.label);
        method.instructions = method.instructions.take().map(|text| login_text(&text));
        let mut keys = std::collections::HashSet::new();
        for prompt in &mut method.prompts {
            if prompt.key.is_empty() || prompt.key.len() > 64 ||
                !prompt.key.bytes().enumerate().all(|(index, ch)| ch.is_ascii_alphabetic() ||
                    (index > 0 && (ch.is_ascii_digit() || matches!(ch, b'_' | b'-')))) ||
                matches!(prompt.key.as_str(), "__proto__" | "prototype" | "constructor" | "signal") ||
                !keys.insert(prompt.key.clone()) {
                return Err(GatewayError::with_status(502, "插件登录输入字段无效或重复"));
            }
            prompt.label = login_text(&prompt.label);
            prompt.placeholder = login_text(&prompt.placeholder);
            for option in &mut prompt.options { option.label = login_text(&option.label); }
            if matches!(prompt.kind, LoginPromptType::Text) { prompt.options.clear(); }
            if matches!(prompt.kind, LoginPromptType::Select) && prompt.options.is_empty() {
                return Err(GatewayError::with_status(502, "插件登录选择字段没有可用选项"));
            }
            if prompt.secret { prompt.default_value = None; }
            if prompt.default_value.as_ref().is_some_and(|value| value.len() > 8192 || value.contains('\0') ||
                (matches!(prompt.kind, LoginPromptType::Select) && !prompt.options.iter().any(|option| &option.value == value))) {
                return Err(GatewayError::with_status(502, "插件登录字段默认值无效"));
            }
        }
        for prompt in &method.prompts {
            if let Some(condition) = &prompt.when {
                if !keys.contains(&condition.key) || !matches!(condition.op.as_str(), "eq" | "neq") ||
                    condition.value.len() > 8192 || condition.value.contains('\0') {
                    return Err(GatewayError::with_status(502, "插件登录字段条件无效"));
                }
            }
        }
        if method.local_only && matches!(method.kind, LoginMethodType::Api) {
            return Err(GatewayError::with_status(502, "本机登录不能进入复制 API 凭证路线"));
        }
        if method.embedded_only && matches!(method.kind, LoginMethodType::Api) {
            return Err(GatewayError::with_status(502, "API 凭证方式不能要求内嵌窗口"));
        }
    }
    metadata.instructions = metadata.instructions.map(|text| login_text(&text));
    Ok(Some(metadata))
}

/// success `{methods:[...], instructions?}`。404 / 501 no has ，
/// use to has ； not methodIndex old POST /start not this 。
pub async fn login_methods(
    State(state): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let provider = params.get("provider").map(|value| value.trim()).unwrap_or("");
    if provider.is_empty() { return management_error(400, "缺少 provider"); }
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider) else {
        return management_error(400, format!("未知的提供商：{provider}"));
    };
    match read_login_methods(&state, adapter.as_ref()).await {
        Ok(Some(metadata)) => match serde_json::to_value(metadata) {
            Ok(value) => ok_json(value),
            Err(_) => management_error(502, "登录方式元数据无法序列化"),
        },
        Ok(None) => management_error(501, "该提供商尚未申报动态登录方式，请使用既有登录入口"),
        Err(error) => management_error(error.status_code, error.message),
    }
}

fn checked_login_inputs(
    method: &LoginMethodMetadata,
    inputs: Option<&Value>,
) -> Result<Value, crate::server::errors::GatewayError> {
    use crate::server::errors::GatewayError;
    let mut inputs = inputs.cloned().unwrap_or_else(|| json!({}));
    let Some(object) = inputs.as_object_mut() else {
        return Err(GatewayError::bad_request("inputs 必须是字符串字段对象"));
    };
    if object.iter().any(|(key, value)| !value.is_string() || !method.prompts.iter().any(|prompt| &prompt.key == key)) {
        return Err(GatewayError::bad_request("inputs 包含未申报的字段或非字符串值"));
    }
    for prompt in &method.prompts {
        if let Some(condition) = &prompt.when {
            let equal = object.get(&condition.key).and_then(Value::as_str) == Some(condition.value.as_str());
            let enabled = if condition.op == "eq" { equal } else { !equal };
            if !enabled {
                if object.contains_key(&prompt.key) { return Err(GatewayError::bad_request("inputs 包含未启用的字段")); }
                continue;
            }
        }
        if !object.contains_key(&prompt.key) {
            if let Some(default) = &prompt.default_value { object.insert(prompt.key.clone(), Value::String(default.clone())); }
        }
        let value = object.get(&prompt.key).and_then(Value::as_str).unwrap_or("");
        if matches!(prompt.kind, LoginPromptType::Select) {
            let supplied = object.get(&prompt.key).and_then(Value::as_str);
            if supplied.is_none() || (prompt.optional && value.is_empty()) {
                if !prompt.optional { return Err(GatewayError::bad_request(format!("请选择 {}", prompt.label))); }
            } else if !prompt.options.iter().any(|option| option.value == value) {
                return Err(GatewayError::bad_request(format!("{}的选项无效", prompt.label)));
            }
        } else if !prompt.optional && value.trim().is_empty() {
            return Err(GatewayError::bad_request(format!("请填写 {}", prompt.label)));
        }
    }
    // 、 not ， and after not inputs to before or 。
    Ok(inputs)
}

fn checked_login_proxy(
    value: Option<&Value>,
) -> Result<Option<crate::server::core::login::LoginProxySelection>, crate::server::errors::GatewayError> {
    use crate::server::core::login::LoginProxySelection;
    use crate::server::core::proxies::{normalize_account_proxy, resolve_account_proxy, ResolvedProxy};
    use crate::server::errors::GatewayError;
    let Some(value) = value else { return Ok(None) };
    // null（ has ） and direct as ， not can and 「 no has 」。
    let direct = value.get("source").and_then(Value::as_str)
        .is_some_and(|source| source.trim().eq_ignore_ascii_case("direct"));
    let config = normalize_account_proxy(if direct { &Value::Null } else { value })
        .map_err(|_| GatewayError::bad_request("登录代理配置无效，请检查代理来源、主机与端口"))?
        .unwrap_or_else(|| json!({ "source": "direct" }));
    if config.get("source").and_then(Value::as_str) == Some("direct") {
        return Ok(Some(LoginProxySelection { config, resolved: Value::Null }));
    }
    if crate::server::core::proxies::is_rotate_config(&config) {
        return Err(GatewayError::bad_request("登录代理不支持轮询出网，请选择一条已保存的代理"));
    }
    let Some(resolution) = resolve_account_proxy(Some(&config)) else {
        return Err(GatewayError::bad_request("所选登录代理未能解析，已拒绝回退默认出口"));
    };
    if resolution.error().is_some() {
        return Err(GatewayError::bad_request("登录代理无法解析，请检查所选代理是否存在且已启用"));
    }
    let proxy = ResolvedProxy::from_json(&resolution.to_json())
        .map_err(|_| GatewayError::bad_request("登录代理出口无效，请检查主机与端口"))?;
    let wire = proxy.map(|proxy| agent2api_provider_protocol::WireProxy {
        protocol: proxy.protocol, host: proxy.host, port: proxy.port,
        username: proxy.username, password: proxy.password,
    });
    let resolved = serde_json::to_value(wire)
        .map_err(|_| GatewayError::bad_request("登录代理无法序列化"))?;
    Ok(Some(LoginProxySelection { config, resolved }))
}

fn login_start_response(
    start: &agent2api_provider_protocol::LoginStart,
    provider: &str,
    edition: &str,
    manual_callback: bool,
    callback_input: LoginCallbackInput,
) -> Value {
    json!({
        "state": start.flow, "flow": start.flow,
        "authUrl": start.url, "url": start.url,
        "edition": edition, "provider": provider,
        "instructions": crate::server::core::login::login_text(&start.instructions),
        "pollIntervalMs": start.poll_interval_ms,
        "externalBrowser": start.external_browser,
        "allowedHosts": start.allowed_hosts,
        "manualCallback": manual_callback,
        "callbackInput": callback_input,
    })
}

// ─── POST /api/session/login/start ──────────────────────────

/// ： methodIndex when methods； old use use 。
/// empty URL available / ，must has empty flow， Done。
///
/// ── `provider` ───────────────────────────────────────
/// body optional `provider`（ = workbuddy）：
/// - ** by **（ up after 、 and provider）→
/// `start_plugin_web_login`： `login.start` `LoginStart`
/// （ + flow + ）， etc `login.callback`、 by 
/// by `pollIntervalMs` `login.poll`；
/// - provider ** no has ** → 503，「 need need 」；
/// - no has id → 400「unknown 」（ not workbuddy：
/// that will 1 workbuddy ）。
///
/// already no has 1 —— this 1 has 「」 and 「 no 」
/// ， not has 。
pub async fn login_start(State(state): State<ServerState>, body: Bytes) -> Response {
    // request `edition` not and ： after up 
    // provider id， that is （`edition` is before ， down Qoder /
    // ZCode ）。failure not is error（Node `catch { edition = null }`）。
    let payload = parse_body(&body).ok();
    let provider = payload
        .as_ref()
        .and_then(|payload| payload.get("provider").and_then(Value::as_str))
        .map(str::trim)
        .unwrap_or("");
    // ， old AutoClaw ； no has old use not 
    // auth.methods， use and supports。
    if let Some(request) = payload.as_ref().filter(|value| value.get("methodIndex").is_some()) {
        let Some(index) = request.get("methodIndex").and_then(Value::as_u64) else {
            return management_error(400, "methodIndex 必须是非负整数");
        };
        let provider = if provider.is_empty() { "workbuddy" } else { provider };
        let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider) else {
            return management_error(400, format!("未知的提供商：{provider}"));
        };
        let metadata = match read_login_methods(&state, adapter.as_ref()).await {
            Ok(Some(metadata)) => metadata,
            Ok(None) => return management_error(501, "该提供商尚未申报动态登录方式，请使用既有登录入口"),
            Err(error) => return management_error(error.status_code, error.message),
        };
        let Some(method) = metadata.methods.iter().find(|method| method.method_index == index) else {
            return management_error(400, "methodIndex 不在该提供商的登录方式清单中");
        };
        let inputs = match checked_login_inputs(method, request.get("inputs")) {
            Ok(inputs) => inputs,
            Err(error) => return management_error(error.status_code, error.message),
        };
        let proxy = match checked_login_proxy(request.get("proxy")) {
            Ok(proxy) => proxy,
            Err(error) => return management_error(error.status_code, error.message),
        };
        let mut options = json!({ "methodIndex": index, "inputs": inputs });
        if let Some(selection) = &proxy {
            options["proxy"] = selection.resolved.clone();
        }
        let instructions = [metadata.instructions.as_deref(), method.instructions.as_deref()]
            .into_iter().flatten().filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>().join("\n");
        let selected = SelectedLogin {
            adapter,
            proxy,
            options,
            local_only: method.local_only,
            manual_callback: method.manual_callback,
            callback_input: method.callback_input,
            instructions: (!instructions.is_empty()).then_some(instructions),
        };
        return start_plugin_web_login(state, provider.to_string(), Some(selected)).await;
    }
    // AutoClaw：** 1 **（`/api/session/login/oauth/start`），
    // must in down all before —— use will 「 can 」，
    // is **supports** ， is not （ need 1 
    // ， by before ）。
    if let Some(region) = crate::server::core::providers::autoclaw::Region::from_provider_id(provider) {
        let message = if region == crate::server::core::providers::autoclaw::Region::Intl {
            "AutoClaw 国际版的网页登录需要先完成一次风控验证，请回到「添加账号」弹窗，             用「网页登录（Zai / Google）」发起"
        } else {
            // no has OAuth（ up oauth-captcha-config return enabled:false），
            // 1 is —— need to that 
            "AutoClaw 国内版没有网页授权登录，请改用「手机验证码登录」或「填写凭证」"
        };
        return management_error(400, message);
    }
    // is provider 1 （ providers::kind_from_id ）
    // old （ no has methodIndex） can ： and old 
    // need （antigravity its —— none up has use can to Google）。
    // this 1 ， options down ； by `payload.proxy` （ antigravity
    // `login::resolve_login_proxy`）， old ignore this 、 as not 。
    let legacy_proxy = match checked_login_proxy(
        payload.as_ref().and_then(|value| value.get("proxy")),
    ) {
        Ok(proxy) => proxy,
        Err(error) => return management_error(error.status_code, error.message),
    };
    let legacy_options = match &legacy_proxy {
        Some(selection) => json!({ "proxy": selection.resolved.clone() }),
        None => json!({}),
    };
    let kind = match provider {
        "" if crate::server::core::plugin_host::serves_provider("workbuddy") => {
            return start_plugin_web_login(
                state,
                "workbuddy".to_string(),
                legacy_selected("workbuddy", legacy_proxy, legacy_options),
            )
            .await;
        }
        "" => crate::server::core::providers::ProviderKind::WorkBuddy,
        other => match crate::server::core::providers::kind_from_id(other) {
            // already ： that 。 down that is ****
            // ， 1 id if from this ， will OAuth 
            // 1 and not will （ is 「success not can use 」）。
            Some(_) if crate::server::core::plugin_host::serves_provider(other) => {
                return start_plugin_web_login(
                    state,
                    other.to_string(),
                    legacy_selected(other, legacy_proxy, legacy_options),
                )
                .await;
            }
            Some(kind) => kind,
            // id not in （ provider no has `ProviderKind`）。
            // `start_plugin_web_login`：`build_login_url_async` 1 
            // `login.start` RPC， to and state after and 1 。
            None => {
                if crate::server::core::providers::adapter::adapter_by_id(other).is_none() {
                    return management_error(400, format!("未知的提供商：{other}"));
                }
                return start_plugin_web_login(
                    state,
                    other.to_string(),
                    legacy_selected(other, legacy_proxy, legacy_options),
                )
                .await;
            }
        },
    };
    // to this ： this is provider， no has ——
    // / CatPaw / Qoder / ZCode / WorkBuddy / Trae / OfficeAce / AtomCode /
    // Cline / AutoClaw etc this 1 。 up this by to 
    // `start_web_login`， that already empty。
    // already 1 not （`implemented_kinds()` as empty）， down 
    // has 「 no 」 this 1 。， not is 1 「 this unsupported
    // 」—— after will use ， this is 。
    management_error(
        503,
        format!(
            "{}的网页登录需要安装提供商插件：本版本不再内置该家的登录实现",
            crate::server::core::providers::meta(kind).label
        ),
    )
}

/// 。
///
/// 1 1 ： by ****（`login.start`
/// RPC）， by has —— when `submit_login_callback` by up 
/// provider id ， `login.callback` 。
///
/// `poll_interval_ms > 0` ； can can in up ， can can in 
/// provider localhost 。manualCallback is need need 。
///
/// this after is 「、 1 etc 1 not will to 」——
/// no has ， is 1 failure。**must** use `login_start_async`
/// `LoginStart`， not can use `(url, state)` that 。
async fn start_plugin_web_login(
    state: ServerState,
    provider: String,
    selected: Option<SelectedLogin>,
) -> Response {
    let selected = match selected {
        Some(selected) => selected,
        None => {
            let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
                return management_error(400, format!("未知的提供商：{provider}"));
            };
            if !adapter.supports_web_login() {
                return management_error(400, format!(
                    "{}不支持网页登录，请改用「填写凭证」或「导入桌面端登录态」添加账号", adapter.label(),
                ));
            }
            SelectedLogin {
                adapter, proxy: None, options: json!({}), local_only: false,
                manual_callback: false, callback_input: LoginCallbackInput::Url, instructions: None,
            }
        }
    };
    let SelectedLogin { adapter, proxy, options, local_only, manual_callback, callback_input, instructions } = selected;
    let label = adapter.label();
    let method_index = options.get("methodIndex").and_then(Value::as_u64);
    let mut start = match adapter.build_login_flow_with_options(options).await {
        Ok(Some(start)) => start,
        Ok(None) => return management_error(400, format!("{label}未能生成登录流程，请重试")),
        Err(error) => return management_error(error.status_code, error.message),
    };
    if start.flow.trim().is_empty() {
        return management_error(502, "插件没有返回登录流程 id");
    }
    start.instructions = crate::server::core::login::login_text(&start.instructions);
    if start.instructions.is_empty() { start.instructions = instructions.unwrap_or_default(); }
    // / must etc Done；empty URL not need need 。
    if start.url.trim().is_empty() { start.url.clear(); }
    if (start.url.is_empty() || local_only) && start.poll_interval_ms == 0 {
        start.poll_interval_ms = 1_000;
    }
    // has WorkBuddy has （`workbuddy` → cn / `workbuddy-intl` → intl），
    // empty。 by provider id ， not —— will 
    // need 。
    let edition = crate::server::core::providers::workbuddy::Region::from_provider_id(&provider)
        .map(|region| region.id())
        .unwrap_or("");
    let handle = state.login().new_handle_for_adapter(edition, adapter.clone(), proxy);
    handle.update(|task| {
        task.state = Some(start.flow.clone());
        task.auth_url = Some(start.url.clone());
        task.instructions = Some(start.instructions.clone());
        task.method_index = method_index;
        task.polling = start.poll_interval_ms > 0;
    });
    state.login().tasks().register(&start.flow, handle.clone());
    // ： after up 「 no 」， after and 
    if start.poll_interval_ms > 0 {
        spawn_plugin_login_poll(
            handle.clone(),
            start.flow.clone(),
            start.poll_interval_ms,
            provider.clone(),
        );
        logging::log(
            "[Login]",
            &format!(
                "发起{label}网页登录（轮询型：每 {}ms 问一次上游）",
                start.poll_interval_ms
            ),
        );
    } else {
        logging::log("[Login]", &format!("发起{label}网页登录（等待浏览器回调…）"));
        // no has consumer need when and session。
        let login = state.login().clone();
        let flow = start.flow.clone();
        crate::spawn_task(async move {
            tokio::time::sleep(Duration::from_millis(LOGIN_TIMEOUT_MS)).await;
            login.cancel_async(&flow).await;
        });
    }
    let mut response = login_start_response(&start, &provider, edition, manual_callback, callback_input);
    if let Some(index) = method_index { response["methodIndex"] = json!(index); }
    if method_index.is_some() { response["localOnly"] = json!(local_only); }
    // allreturn already and LoginStart， not options / inputs 。
    ok_json(response)
}

/// after 。
///
/// ── Why not in request etc ──────────────────────────────────────
/// need minutes（ use in ）。 in `/start` will that HTTP request
/// to when ， before not to state none `/wait`。
///
/// ── failure ────────────────────────────────────────────────
/// - `Failed` → error（ before `/wait` to ）；
/// - error → error。 and 「」 that not ： this 1 not 
/// not can ， etc will 1 「 use no 」；
/// - `Pending` → ， to `LOGIN_TIMEOUT_MS`。
///
/// （`/cancel`） when down 1 （`cancel` will from ，
/// in ， this is up `canceled` ）。
pub(crate) fn spawn_plugin_login_poll(
    handle: crate::server::core::login::LoginTaskHandle,
    flow: String,
    interval_ms: u64,
    provider: String,
) {
    crate::spawn_task(async move {
        let deadline = tokio::time::Instant::now()
            + std::time::Duration::from_millis(crate::server::core::login::LOGIN_TIMEOUT_MS);
        let interval = Duration::from_millis(interval_ms.clamp(1_000, LOGIN_TIMEOUT_MS));
        loop {
            tokio::time::sleep(interval).await;
            let _operation = handle.operation.lock().await;
            let task = handle.snapshot();
            if task.canceled || task.done {
                return;
            }
            if tokio::time::Instant::now() >= deadline {
                crate::server::core::login::fail_plugin_login(
                    &handle,
                    "网页登录超时，请重新发起",
                );
                handle.cancel_provider_flow_locked(&flow).await;
                return;
            }
            let Some(adapter) = handle.adapter() else {
                crate::server::core::login::fail_plugin_login(&handle, "登录任务未绑定执行体，请重新发起");
                return;
            };
            let poll = adapter.login_poll_async(&flow).await;
            // request in await in or by ： to Done not 。
            let task = handle.snapshot();
            if task.canceled || task.done { return; }
            match poll {
                Ok(agent2api_provider_protocol::LoginPoll::Done { account }) => {
                    match crate::server::core::login::persist_plugin_login(&handle, adapter.as_ref(), &account) {
                        Ok(Some(_)) => logging::log("[Login]", &format!("{provider} 登录完成，账号已加入列表")),
                        Ok(None) => {}
                        Err(error) => crate::server::core::login::fail_plugin_login(&handle, &error.message),
                    }
                    handle.cancel_provider_flow_locked(&flow).await;
                    return;
                }
                Ok(agent2api_provider_protocol::LoginPoll::Pending { hint }) => {
                    crate::server::core::login::set_plugin_login_hint(&handle, hint.as_deref());
                }
                Ok(agent2api_provider_protocol::LoginPoll::Failed { message }) => {
                    crate::server::core::login::fail_plugin_login(&handle, &message);
                    handle.cancel_provider_flow_locked(&flow).await;
                    return;
                }
                Err(error) => {
                    crate::server::core::login::fail_plugin_login(&handle, &error.message);
                    handle.cancel_provider_flow_locked(&flow).await;
                    return;
                }
            }
        }
    });
}


// ─── GET /api/session/login/wait ────────────────────────────

/// （response：pending / done+error / done+session）
pub async fn login_wait(
    State(state): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let task_state = params.get("state").cloned().unwrap_or_default();
    let Some(task) = state.login().tasks().get(&task_state) else {
        return management_error(404, "登录任务不存在或已过期");
    };
    ok_json(task.snapshot().to_wait_response())
}

// ─── POST /api/session/login/cancel ─────────────────────────

pub async fn login_cancel(State(state): State<ServerState>, body: Bytes) -> Response {
    let task_state = parse_body(&body)
        .ok()
        .and_then(|payload| {
            payload
                .get("state")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    // LoginService that 1 （ not is `tasks().cancel`）：AutoClaw that 
    // in up ， when 1 （ `LoginService::cancel`）
    let canceled = state.login().cancel_async(&task_state).await;
    ok_json(json!({ "canceled": canceled }))
}

// ─── POST /api/session/login/callback ───────────────────────

/// URL（** use **）。
///
/// ── need need ────────────────────────────────────────
/// in 1 ** or 、 not to ** when ， by this 1 use 
/// 。：
/// - ：`on_navigation` URL、 down 、 POST 
/// （ `office-raccoon://`、`orcaterm://` can this ）；
/// - ： use in URL（`web_shim.rs` 
///     `needsManualCallback`）。
///
/// ── not 1 ──────────────────────────────
/// 「 by state → URL to ** this 1 **」。
/// `scheme / host / path / is ` in （`login.callback`
/// to is URL and `code` ），add 1 not need need this 1 。
///
/// ── and failure ──────────────────────────────────────────
/// body `{state, callbackUrl}`：
/// - state must for 1 ** in ** （404 no has this ， before new ）；
/// - state for and 1 ；failure need error，
/// otherwise before will 1 etc to 5 minutes when 。
pub async fn login_callback(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    let task_state = payload
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let callback_url = payload
        .get("callbackUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if callback_url.trim().is_empty() {
        return management_error(400, "缺少 callbackUrl（登录回调地址）");
    }
    // error in old AutoClaw before 1 ， not /。
    if let Some(error) = url::Url::parse(&callback_url).ok().and_then(|url| {
        url.query_pairs().find(|(key, value)| key == "error" && !value.trim().is_empty())
            .map(|(_, error)| error.into_owned())
    }) {
        let error = state.login().reject_login_callback(&task_state, &error).await;
        return management_error(error.status_code, error.message);
    }
    // ── AutoClaw is ****（ in ）──
    // `submit_login_callback` that 「 etc 」 （ will 
    // 1 code to `login.callback`）。AutoClaw in 
    // ， no has `login.callback` —— use need ** that 
    // **（`login.oauth-callback` ）， after by `login.poll` 。
    if let Some(task) = state.login().tasks().get(&task_state) {
        let snapshot = task.snapshot();
        if snapshot.canceled { return management_error(400, "登录已取消"); }
        if snapshot.done {
            return match snapshot.error {
                Some(error) => management_error(400, error),
                None => ok_json(json!({ "accountId": snapshot.session.as_ref().and_then(|session| session.get("accountUid")) })),
            };
        }
        if snapshot.method_index.is_none()
            && crate::server::core::providers::autoclaw::Region::from_provider_id(&snapshot.provider).is_some()
        {
            let Some(adapter) = task.adapter() else {
                return management_error(503, "AutoClaw 登录任务未绑定执行体，请重新发起");
            };
            let _operation = task.operation.lock().await;
            let current = task.snapshot();
            if current.canceled { return management_error(400, "登录已取消"); }
            if current.done {
                return match current.error {
                    Some(error) => management_error(400, error),
                    None => ok_json(json!({ "accountId": current.session.as_ref().and_then(|session| session.get("accountUid")) })),
                };
            }
            return match adapter
                .admin_action(
                    state.store(),
                    "",
                    "login.oauth-callback",
                    &json!({ "state": task_state, "url": callback_url }),
                )
                .await
            {
                Ok(_) => ok_json(json!({ "accepted": true })),
                Err(error) => management_error(error.status_code, error.message),
            };
        }
    }
    match state
        .login()
        .submit_login_callback(&task_state, &callback_url)
        .await
    {
        Ok(crate::server::core::login::LoginCallbackSubmission::Completed(account_id)) => {
            ok_json(json!({ "accountId": account_id }))
        }
        Ok(crate::server::core::login::LoginCallbackSubmission::Accepted) => {
            ok_json(json!({ "accepted": true, "continue": true }))
        }
        Ok(crate::server::core::login::LoginCallbackSubmission::ContinueTo(next_url)) => {
            ok_json(json!({ "continue": true, "nextUrl": next_url }))
        }
        Err(error) => management_error(error.status_code, error.message),
    }
}

// ─── GET /api/session/login/remote-callback ────────────────

/// ** in **（ can to 、 can 1 
/// HTTP when use ）。
///
/// ── in use ──────────────────────────────────────────────────
/// （`web_shim.rs`）： not to ， shim 
/// `redirect` ， `authorization_code` to up 。
/// is and ** can to ** that 1 ， by down this 1 。
///
/// ── Whypath is use ──────────────────────────────────────
/// parameter（`code` / `authorization_code`）、 need ，
/// is ** that 1 up **， already to 。 this ：
/// ① from query 1 （ this OAuth parameter）；
/// ② code and already state to `submit_login_callback`； URL by POST ；
/// ③ 1 HTML （ use ）。
///
/// this by must： use is ， not will API Key； by 
/// not can state （ for 1 ）。
pub async fn login_remote_callback(
    State(state): State<ServerState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let task_state = params.get("state").cloned().unwrap_or_default();
    if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
        let error = state.login().reject_login_callback(&task_state, error).await;
        return oauth_callback_page(error.status_code, &error.message);
    }
    let code = params
        .get("code")
        .or_else(|| params.get("authorization_code"))
        .cloned()
        .unwrap_or_default();
    drop(params);
    if task_state.trim().is_empty() || code.trim().is_empty() {
        return oauth_callback_page(
            400,
            "登录失败：回调缺少 state 或授权码，请重新发起网页登录。",
        );
    }
    // this is has 「 1 code」 in ， not is provider callback URL。
    // not localhost URL： old will scheme/host/path（e.g.），
    // 1 URL will old 。 URL POST /callback。
    match state
        .login()
        .submit_login_callback(&task_state, &code)
        .await
    {
        Ok(crate::server::core::login::LoginCallbackSubmission::Completed(_)) => {
            oauth_callback_page(200, "登录成功，已返回网关，可以关闭此页面。")
        }
        Ok(crate::server::core::login::LoginCallbackSubmission::Accepted) => {
            oauth_callback_page(200, "回调已接收，登录仍在处理中，请返回面板等待。")
        }
        Ok(crate::server::core::login::LoginCallbackSubmission::ContinueTo(_)) => {
            oauth_callback_page(400, "登录回调仍需继续，请重新发起网页登录。")
        }
        Err(error) => {
            oauth_callback_page(error.status_code, &format!("登录失败：{}", error.message))
        }
    }
}

// ─── POST /api/session/login/sms/send and /verify ────────────

/// from request AutoClaw （`provider` ， and `POST /api/accounts` ）。
///
/// （ no / empty）****： and provider id 1 ——
/// client not this ， that use in use 。
///
/// ── not ：（issue #93 ）──────────────────
/// is 「unknown」， is ** provider none**： 1 
/// id（ Loomy） to this when ， AutoClaw 、
/// AutoClaw， use to is **successresponse**。 has 「 no 」 by ；
/// not 。
fn autoclaw_region_of(
    payload: &Value,
) -> Result<crate::server::core::providers::autoclaw::Region, Response> {
    use crate::server::core::providers::autoclaw::Region;
    let raw = payload
        .get("provider")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if raw.is_empty() {
        return Ok(Region::Cn);
    }
    match Region::from_provider_id(raw) {
        Some(region) => Ok(region),
        None => {
            let known = Region::ALL
                .into_iter()
                .map(Region::provider_id)
                .collect::<Vec<_>>()
                .join(" / ");
            Err(management_error(
                400,
                format!("未知的登录地区「{raw}」（只支持 {known}）"),
            ))
        }
    }
}

/// AutoClaw （ no when to can ）。
fn autoclaw_adapter(
    region: crate::server::core::providers::autoclaw::Region,
) -> Result<std::sync::Arc<dyn crate::server::core::providers::adapter::ProviderAdapter>, Response> {
    let provider = region.provider_id();
    match crate::server::core::providers::adapter::adapter_by_id(provider) {
        Some(adapter) => Ok(adapter),
        None => Err(management_error(
            503,
            format!(
                "AutoClaw {} 的登录需要安装提供商插件：本版本不再携带该家的登录实现",
                region.label()
            ),
        )),
    }
}

/// （**AutoClaw use **， 1 ）。
///
/// ── Why this not is 「」──────────────────────────────────
/// AutoClaw no has ： no has Web use 、 no has ，
/// 1 is + （， has OAuth）。 this not 
/// not ， is ** 1 up use **（ `login.sms-send`）。
///
/// body `{phone, provider?}` → `{deviceId}`。
///
/// ── Why deviceId to before ──────────────────────────────
/// up 「 this 」 in when device_id up ，must 1 ——
/// not use this in （ 1 can HTTP request、 can use 
/// ， in server will 1 need ）。 to before down 1 request。
pub async fn login_sms_send(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    // old before ： `provider: tabbit` to AutoClaw 。
    if payload.get("provider").and_then(Value::as_str) == Some("tabbit") {
        return tabbit_sms_send_inner(&state, &payload).await;
    }
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    let region = match autoclaw_region_of(&payload) {
        Ok(region) => region,
        Err(response) => return response,
    };
    let adapter = match autoclaw_adapter(region) {
        Ok(adapter) => adapter,
        Err(response) => return response,
    };
    // not need need （ no ），`account` empty； that not 。
    match adapter
        .admin_action(state.store(), "", "login.sms-send", &json!({ "phone": phone }))
        .await
    {
        Ok(result) => ok_json(result),
        Err(error) => management_error(error.status_code, error.message),
    }
}

/// use + and ****（**AutoClaw use **）。
///
/// body `{phone, code, deviceId?, name?, provider?}` → `{account, list}` ——
/// response and `POST /api/accounts` ** 1 **： is 1 to ，
/// 、、、 use that 。
pub async fn login_sms_verify(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    // old before ： `provider: tabbit` to AutoClaw 。
    if payload.get("provider").and_then(Value::as_str) == Some("tabbit") {
        return tabbit_sms_verify_inner(&state, &payload).await;
    }
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    let code = payload.get("code").and_then(Value::as_str).unwrap_or("");
    let device_id = payload.get("deviceId").and_then(Value::as_str);
    let name = payload.get("name").and_then(Value::as_str);
    let region = match autoclaw_region_of(&payload) {
        Ok(region) => region,
        Err(response) => return response,
    };
    let adapter = match autoclaw_adapter(region) {
        Ok(adapter) => adapter,
        Err(response) => return response,
    };
    let account = match adapter
        .admin_action(
            state.store(),
            "",
            "login.sms-verify",
            &json!({ "phone": phone, "code": code, "deviceId": device_id, "name": name }),
        )
        .await
    {
        Ok(value) => match serde_json::from_value::<agent2api_provider_protocol::NormalizedAccount>(value)
        {
            Ok(account) => account,
            Err(error) => {
                return management_error(502, format!("插件返回的登录结果无法解析：{error}"))
            }
        },
        Err(error) => return management_error(error.status_code, error.message),
    };
    match adapter.persist_login_account(&account) {
        Ok(account_id) => {
            let label = account
                .public
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("AutoClaw 账号");
            logging::log(
                "[Login]",
                &format!("✅ AutoClaw {}登录成功: {label}", region.label()),
            );
            let list = state.store().list_visible_accounts();
            let account = list
                .get("accounts")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_str) == Some(&account_id))
                })
                .cloned()
                .unwrap_or(account.public);
            ok_json(json!({ "account": account, "list": list }))
        }
        Err(error) => management_error(error.status_code, error.message),
    }
}

// ─── Loomy （）─────────────────────────

/// （**Loomy use **， 1 ）。
///
/// ── Why this not is 「」──────────────────────────────────
/// Loomy （CAccount） no has that —— client 1 
/// is + （ that need need ，
/// not ， `providers/loomy/login.rs` module header）。 this not not 
/// after ， is ** 1 up use **（ HMAC-SHA1 ）。
///
/// body `{phone}` → `{msgid}`。
///
/// ── Why msgid to before （ and AutoClaw deviceId ）──
/// up 「 this 」 in response msgid up ，must 1 ——
/// not use this in （ 1 can HTTP request、 can 
/// ， in server will 1 need ）。
pub async fn login_loomy_sms_send(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id("loomy") else {
        return management_error(400, "Loomy 的提供商插件没有安装，无法发送验证码");
    };
    // not need need （ no ），`account` empty；`store` is 
    // （ this not ）。
    match adapter
        .admin_action(state.store(), "", "login.sms-send", &json!({ "phone": phone }))
        .await
    {
        Ok(result) => ok_json(result),
        Err(error) => management_error(error.status_code, error.message),
    }
}

/// use + and ****（**Loomy use **）。
///
/// body `{phone, code, msgid, name?}` → `{account, list}` —— response and 
/// `POST /api/accounts` ** 1 **： is 1 to ，
/// and use ， before can to 「 already 」。
pub async fn login_loomy_sms_verify(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    let code = payload.get("code").and_then(Value::as_str).unwrap_or("");
    let msgid = payload.get("msgid").and_then(Value::as_str);
    let Some(adapter) =
        crate::server::core::providers::adapter::adapter_by_id("loomy")
    else {
        return management_error(400, "Loomy 的提供商插件没有安装，无法登录");
    };
    let mut account = match adapter
        .admin_action(
            state.store(),
            "",
            "login.sms-verify",
            &json!({ "phone": phone, "code": code, "msgid": msgid }),
        )
        .await
    {
        Ok(value) => match serde_json::from_value::<agent2api_provider_protocol::NormalizedAccount>(value)
        {
            Ok(account) => account,
            Err(error) => {
                return management_error(502, format!("插件返回的登录结果无法解析：{error}"))
            }
        },
        Err(error) => return management_error(error.status_code, error.message),
    };
    if let Some(name) = payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(public) = account.public.as_object_mut() {
            public.insert(
                "name".to_string(),
                Value::String(name.chars().take(100).collect()),
            );
        }
    }
    match adapter.persist_login_account(&account) {
        Ok(account_id) => {
            let label = account
                .public
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Loomy 账号");
            logging::log("[Login]", &format!("✅ Loomy 登录成功: {label}"));
            let list = state.store().list_visible_accounts();
            let account = list
                .get("accounts")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_str) == Some(&account_id))
                })
                .cloned()
                .unwrap_or(account.public);
            ok_json(json!({ "account": account, "list": list }))
        }
        Err(error) => management_error(error.status_code, error.message),
    }
}

// ─── Tabbit （）───────────────────

/// （**Tabbit use **）。
///
/// body `{phone, uuid?, supportsCaptcha?}` → `{uuid}`。uuid is this will 
/// ： up in up ， when must 1 ； not this in ，
/// to before down 1 request can 。
///
/// ── success：`{status:"captcha", captcha:{requestCode, verifyUrl}}` ──
/// can when up not will ， is to 1 。 use in that 
/// after ** new 1 **success（， `challenge_of`）。
/// `supportsCaptcha` will to this 200 + ， and 
/// use 1 ； no （ old before ） to 409 and 1 can 。
pub async fn login_tabbit_sms_send(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    tabbit_sms_send_inner(&state, &payload).await
}

/// ； old `/api/session/login/sms/send` by `provider` 
/// （ old before will `provider: tabbit` to AutoClaw ）。
async fn tabbit_sms_send_inner(state: &ServerState, payload: &Value) -> Response {
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    // when need use 1 uuid（ 1 ）， before will 。
    let uuid = payload.get("uuid").and_then(Value::as_str);
    // new 「 before need 」 this ； not 1 by error，
    // old before 「 already 」 use etc 1 not will 。
    let supports_captcha = payload
        .get("supportsCaptcha")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let Some(adapter) =
        crate::server::core::providers::adapter::adapter_by_id("tabbit")
    else {
        return management_error(503, "Tabbit 的提供商插件没有安装，无法发送验证码");
    };
    match adapter
        .admin_action(
            state.store(),
            "",
            "login.sms-send",
            &json!({ "phone": phone, "uuid": uuid }),
        )
        .await
    {
        Ok(result) => {
            let needs_captcha =
                result.get("status").and_then(Value::as_str) == Some("captcha");
            if needs_captcha && !supports_captcha {
                return management_error(
                    409,
                    "发送验证码前需要先完成滑块验证。请刷新页面后再试，或改用「填写凭证」粘贴 Cookie",
                );
            }
            ok_json(result)
        }
        Err(error) => management_error(error.status_code, error.message),
    }
}

/// use + and ****（**Tabbit use **）。
///
/// body `{phone, code, uuid, name?}` → `{account, list}`，response and 
/// `POST /api/accounts` 1 ；、、 use 。
pub async fn login_tabbit_sms_verify(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    tabbit_sms_verify_inner(&state, &payload).await
}

/// ； old `/api/session/login/sms/verify` by `provider`
/// （ old before `provider: tabbit` to AutoClaw when can ）。
async fn tabbit_sms_verify_inner(state: &ServerState, payload: &Value) -> Response {
    let phone = payload.get("phone").and_then(Value::as_str).unwrap_or("");
    let code = payload.get("code").and_then(Value::as_str).unwrap_or("");
    // new before `uuid`； old before （AutoClaw ） 1 in 
    // `deviceId`， ， old must can 。
    let uuid = payload
        .get("uuid")
        .or_else(|| payload.get("deviceId"))
        .and_then(Value::as_str);
    let Some(adapter) =
        crate::server::core::providers::adapter::adapter_by_id("tabbit")
    else {
        return management_error(503, "Tabbit 的提供商插件没有安装，无法登录");
    };
    let mut account = match adapter
        .admin_action(
            state.store(),
            "",
            "login.sms-verify",
            &json!({
                "phone": phone,
                "code": code,
                "uuid": uuid,
                "name": payload.get("name"),
            }),
        )
        .await
    {
        Ok(value) => {
            match serde_json::from_value::<agent2api_provider_protocol::NormalizedAccount>(value) {
                Ok(account) => account,
                Err(error) => {
                    return management_error(502, format!("插件返回的登录结果无法解析：{error}"))
                }
            }
        }
        Err(error) => return management_error(error.status_code, error.message),
    };
    if let Some(name) = payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if let Some(public) = account.public.as_object_mut() {
            public.insert(
                "name".to_string(),
                Value::String(name.chars().take(100).collect()),
            );
        }
    }
    match adapter.persist_login_account(&account) {
        Ok(account_id) => {
            let label = account
                .public
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Tabbit 账号");
            logging::log("[Login]", &format!("✅ Tabbit 登录成功: {label}"));
            let list = state.store().list_visible_accounts();
            let account = list
                .get("accounts")
                .and_then(Value::as_array)
                .and_then(|items| {
                    items
                        .iter()
                        .find(|item| item.get("id").and_then(Value::as_str) == Some(&account_id))
                })
                .cloned()
                .unwrap_or(account.public);
            ok_json(json!({ "account": account, "list": list }))
        }
        Err(error) => management_error(error.status_code, error.message),
    }
}

// ─── KukuAI （）──────────────────────────

/// `POST /api/session/login/kuku/complete` —— 。
///
/// success after ** not **（ in `.baidu.com` 
/// Cookie ， not will ）， `BDUSS` is **HttpOnly** Cookie，
/// `document.cookie` not to —— by ****
/// ： `success.html` after ， use WebView2 Cookie 
/// （`cookies()` HttpOnly） `BDUSS` / `STOKEN`， Cookie 
/// POST to this （`state` + `cookie` in request， not URL）。
///
/// this by ****： use is （）， not API Key； by 
/// 1 state （ and in ）。
pub async fn login_kuku_complete(
    State(state): State<ServerState>,
    Json(payload): Json<Value>,
) -> Response {
    let task_state = payload
        .get("state")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let cookie = payload
        .get("cookie")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if task_state.is_empty() || cookie.is_empty() {
        return management_error(400, "登录失败：回调缺少 state 或登录态 Cookie，请重新发起短信登录。");
    }
    // **** that use ：Cookie as 1 `code` to ，
    // （`BDUSS` / `STOKEN` / its Cookie）、 `userreport`
    // 、 —— this not Cookie 。
    match state.login().submit_login_callback(task_state, cookie).await {
        Ok(crate::server::core::login::LoginCallbackSubmission::Completed(_)) => ok_json(json!({ "ok": true })),
        Ok(_) => ok_json(json!({ "accepted": true, "continue": true })),
        Err(error) => management_error(error.status_code, &format!("登录失败：{}", error.message)),
    }
}

// ─── OrcaTerm （）──────────────────────────

/// `POST /api/session/login/orcaterm/complete` —— 1 。
///
/// ── not in this （）─────────────────────────────────
/// ** OAuth**： in → → up 
/// `orcaterm://oauth_callback?session_id=…` → URL to 
/// `POST /api/session/login/callback` → after `OAuthExchangeToken` 
/// （ `providers/orcaterm/login.rs` module header， that 「Why not use 
/// `ot_session`」： that Cookie in SPA when not will ）。
///
/// to 「 use already has 1 」 （ if from to 、
/// to ）：`{state, token}` POST ， up ，。
///
/// `ot_session=…` before （ after `normalize_token` will ），
/// `token` `cookie`。
///
/// this by ****： use is （）， not API Key； by 
/// 1 state （ and kuku/complete ）。
pub async fn login_orcaterm_complete(
    State(state): State<ServerState>,
    Json(payload): Json<Value>,
) -> Response {
    let task_state = payload
        .get("state")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    // `token`： is （`ot_session=` before by after 1 ，
    // Cookie ）；`cookie` as 1 and 。
    let token = payload
        .get("token")
        .or_else(|| payload.get("cookie"))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    if task_state.is_empty() || token.is_empty() {
        return management_error(
            400,
            "登录失败：回调缺少 state 或登录态令牌，请重新发起网页登录。",
        );
    }
    match state.login().finish_orcaterm_login(task_state, token).await {
        Ok(_) => ok_json(json!({ "ok": true })),
        Err(error) => management_error(error.status_code, &format!("登录失败：{}", error.message)),
    }
}

// ─── AutoClaw OAuth （，）──────────────────

/// from request OAuth （`vendor` ：`zai` / `google`）。
///
/// not 1 400：** not ** to 1 —— that will use Google
/// Zai （ is not ， is ）。
///
/// by （`login.oauth-config` `region` not ， this 
/// ****：empty and is already 1 —— this is by ，
/// not is up ）。
fn oauth_vendor_of(payload: &Value) -> Result<&'static str, Response> {
    let raw = payload
        .get("vendor")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    match raw {
        "zai" => Ok("zai"),
        "google" => Ok("google"),
        _ => Err(management_error(
            400,
            format!("未知的登录方式「{raw}」（只支持 zai / google）"),
        )),
    }
}

/// `POST /api/session/login/oauth/captcha-config` —— config。
///
/// before SDK（`prefix` / `sceneId` is SDK parameter）。
/// `?provider=` ；。
///
/// ── Why this 1 （ not start）──────────────────
/// need in ****（SDK is JS）， SDK need need 
/// `prefix` / `sceneId` —— that can from up 。 if 「config」 and 「
/// 」 and 1 request，「config、 no 」，
/// before need 1 request。
///
/// `enabled: false`（） not is error： before not OAuth by 。
pub async fn login_oauth_captcha_config(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    let region = match autoclaw_region_of(&payload) {
        Ok(region) => region,
        Err(response) => return response,
    };
    let adapter = match autoclaw_adapter(region) {
        Ok(adapter) => adapter,
        Err(response) => return response,
    };
    match adapter
        .admin_action(state.store(), "", "login.oauth-config", &Value::Null)
        .await
    {
        Ok(config) => ok_json(config),
        Err(error) => management_error(error.status_code, error.message),
    }
}

/// `POST /api/session/login/oauth/start` —— parameter OAuth 。
///
/// body `{provider, vendor, captchaVerifyParam}` → `{state, authUrl, provider,
/// edition, pollIntervalMs, warning?}`（ and ** 1 response**，
/// before and etc not as new ）。
///
/// ── Why this not `start_login` ────────────────────────
/// that is 「 1 」， this before is 「 before already in 
/// **** SDK 」。 parameter from to 、
/// ， etc 1 1 not this by not 。
///
/// ── in （ after ）─────────────────────────────────
/// z.ai 1 and ** down **（`web_login` +
/// `callback_server`）； `pollIntervalMs`， `login.poll`。
/// this ** not has ** `/auth/callback-*` by —— not that in 。
pub async fn login_oauth_start(State(state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).unwrap_or(Value::Null);
    let region = match autoclaw_region_of(&payload) {
        Ok(region) => region,
        Err(response) => return response,
    };
    let vendor = match oauth_vendor_of(&payload) {
        Ok(vendor) => vendor,
        Err(response) => return response,
    };
    let captcha = payload
        .get("captchaVerifyParam")
        .and_then(Value::as_str)
        .unwrap_or("");
    let adapter = match autoclaw_adapter(region) {
        Ok(adapter) => adapter,
        Err(response) => return response,
    };
    if !adapter.supports_web_login() {
        return management_error(
            400,
            format!(
                "AutoClaw {}没有网页授权登录，请改用「填写凭证」或「导入桌面端登录态」",
                region.label()
            ),
        );
    }
    let started = match adapter
        .admin_action(
            state.store(),
            "",
            "login.oauth-start",
            &json!({ "vendor": vendor, "captchaVerifyParam": captcha }),
        )
        .await
    {
        Ok(value) => value,
        Err(error) => return management_error(error.status_code, error.message),
    };
    let flow = started
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let auth_url = started
        .get("authUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    if flow.is_empty() || auth_url.is_empty() {
        return management_error(502, "插件没有返回授权地址或登录流程 id，请重试");
    }
    // ：`/wait` by state ，`pollIntervalMs > 0` when after 。
    let handle = state.login().new_handle_for_adapter("", adapter.clone(), None);
    let interval = started
        .get("pollIntervalMs")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let start = agent2api_provider_protocol::LoginStart {
        flow: flow.clone(), url: auth_url.clone(),
        instructions: crate::server::core::login::login_text(started.get("instructions").and_then(Value::as_str).unwrap_or("")),
        poll_interval_ms: interval,
        external_browser: started.get("externalBrowser").and_then(Value::as_bool).unwrap_or(false),
        allowed_hosts: started.get("allowedHosts").and_then(Value::as_array)
            .map(|hosts| hosts.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default(),
    };
    handle.update(|task| {
        task.state = Some(flow.clone());
        task.auth_url = Some(auth_url.clone());
        task.instructions = Some(start.instructions.clone());
        task.polling = interval > 0;
    });
    state.login().tasks().register(&flow, handle.clone());
    if interval > 0 {
        spawn_plugin_login_poll(
            handle,
            flow.clone(),
            interval,
            region.provider_id().to_string(),
        );
    } else {
        let login = state.login().clone();
        let flow = flow.clone();
        crate::spawn_task(async move {
            tokio::time::sleep(Duration::from_millis(LOGIN_TIMEOUT_MS)).await;
            login.cancel_async(&flow).await;
        });
    }
    logging::log(
        "[Login]",
        &format!(
            "发起 AutoClaw {} {}网页登录（插件侧回调，轮询 {}ms）",
            region.label(),
            vendor,
            interval
        ),
    );
    let mut body = login_start_response(&start, region.provider_id(), "intl", true, LoginCallbackInput::Url);
    if let Some(warning) = started.get("warning").and_then(Value::as_str) {
        if let Some(object) = body.as_object_mut() {
            object.insert("warning".to_string(), Value::String(warning.to_string()));
        }
    }
    ok_json(body)
}

/// OAuth （ in up ， use to can ）。
///
/// not and ： this 1 ，。
/// can can is 410（ up down already ） this 200 —— 1 use `from_u16`
/// 200， 1 responsefailure。
fn oauth_callback_page(status: i32, message: &str) -> Response {
    use axum::response::IntoResponse;

    let escaped = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");
    let html = format!(
        "<!doctype html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\">\
         <title>登录回调</title></head>\
         <body style=\"font-family:system-ui,sans-serif;padding:48px;text-align:center\">\
         <p style=\"font-size:16px\">{escaped}</p></body></html>"
    );
    let status = u16::try_from(status)
        .ok()
        .and_then(|code| axum::http::StatusCode::from_u16(code).ok())
        .unwrap_or(axum::http::StatusCode::OK);
    (
        status,
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}

// ─── /auth/callback（LobsterAI loopback ）─────────────

/// LobsterAI 。** 302 to this **（ after 
/// to up `redirect_uri`， `code` / `state`）。
///
/// path is `/auth/callback`（ and after not 1 ）—— by 
/// `http.rs` that by ：`redirect_uri` is to ， is for 
/// path has no has ， that already 。
///
/// by and antigravity / lobsterai loopback 
/// 1 ： use is ** use **， no has API Key； by 
/// 1 `state` （ `core::login::lobsterai`）。
pub async fn login_lobsterai_callback(
    State(state): State<ServerState>,
    OriginalUri(uri): OriginalUri,
    headers: axum::http::HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let task_state = params.get("state").cloned().unwrap_or_default();
    if let Some(task) = state.login().tasks().get(&task_state) {
        let snapshot = task.snapshot();
        if snapshot.provider != crate::server::core::providers::kind_id(crate::server::core::providers::ProviderKind::LobsterAi) {
            return oauth_callback_page(400, "这次登录不属于 LobsterAI");
        }
        if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
            let error = state.login().reject_login_callback(&task_state, error).await;
            return oauth_callback_page(error.status_code, &error.message);
        }
        if task.adapter().is_some() {
            let callback_url = match original_callback_url(&uri, &headers) {
                Ok(url) => url,
                Err(error) => return oauth_callback_page(error.status_code, &error.message),
            };
            return login_callback_page(state.login().submit_login_callback(&task_state, &callback_url).await);
        }
    }
    if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
        let error = state.login().reject_login_callback(&task_state, error).await;
        return oauth_callback_page(error.status_code, &error.message);
    }
    // old use ， not its and 。
    let code = params.get("code").cloned().unwrap_or_default();
    match state.login().finish_lobsterai_login(&code, &task_state).await {
        Ok(_) => oauth_callback_page(200, "登录成功，已返回网关，可以关闭此页面。"),
        Err(error) => oauth_callback_page(error.status_code, &format!("登录失败：{}", error.message)),
    }
}

// ─── GET /oauth-callback（Antigravity Google OAuth ）────

/// Antigravity （** 302 to this **， `code` / `state`）。
///
/// ── pathWhy is `/oauth-callback`（）──────────────────
/// is when to Google `redirect_uri` path，
/// （`Antigravity-Manager/src-tauri/src/modules/oauth_server.rs`） use 
/// `/oauth-callback`；Google desktop client loopback any +
/// this path。
///
/// ── Why（ public ）──────────────────────────────
/// use is ** use **（Google after ）， no has 
/// API Key。 and CatPaw / Accio loopback 1 ；
/// by not can state （ for 
/// `core::login::submit_login_callback` Antigravity ）。
///
/// ── and 「 URL」 use 1 ────────────────────────
/// / Docker down can can to not ， use will URL 
/// ； that to `submit_login_callback`（Accio by ），
/// to 。
pub async fn login_antigravity_callback(
    State(state): State<ServerState>,
    OriginalUri(uri): OriginalUri,
    headers: axum::http::HeaderMap,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let code = params.get("code").cloned().unwrap_or_default();
    let task_state = params.get("state").cloned().unwrap_or_default();
    if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
        let error = state.login().reject_login_callback(&task_state, error).await;
        return oauth_callback_page(error.status_code, &error.message);
    }
    if task_state.trim().is_empty() || code.trim().is_empty() {
        return oauth_callback_page(400, "登录失败：回调缺少 state 或授权码，请重新发起网页登录。");
    }
    let callback_url = match original_callback_url(&uri, &headers) {
        Ok(url) => url,
        Err(error) => return oauth_callback_page(error.status_code, &error.message),
    };
    login_callback_page(state.login().submit_login_callback(&task_state, &callback_url).await)
}

/// HTTP request not fragment；request origin， path/query ，
/// not from already Query new code/state， not localhost or its parameter。
fn original_callback_url(
    uri: &axum::http::Uri,
    headers: &axum::http::HeaderMap,
) -> Result<String, crate::server::errors::GatewayError> {
    use crate::server::errors::GatewayError;
    if uri.scheme_str().is_some() && uri.authority().is_some() { return Ok(uri.to_string()); }
    let host = headers.get(axum::http::header::HOST).and_then(|value| value.to_str().ok())
        .ok_or_else(|| GatewayError::bad_request("登录回调缺少原始主机地址"))?;
    host.parse::<axum::http::uri::Authority>()
        .map_err(|_| GatewayError::bad_request("登录回调主机地址无效"))?;
    let scheme = uri.scheme_str().unwrap_or("http");
    Ok(format!("{scheme}://{host}{uri}"))
}

fn login_callback_page(
    result: Result<crate::server::core::login::LoginCallbackSubmission, crate::server::errors::GatewayError>,
) -> Response {
    use crate::server::core::login::LoginCallbackSubmission;
    match result {
        Ok(LoginCallbackSubmission::Completed(_)) => oauth_callback_page(200, "登录成功，已返回网关，可以关闭此页面。"),
        Ok(LoginCallbackSubmission::Accepted) => oauth_callback_page(200, "回调已接收，登录仍在处理中，请返回面板等待。"),
        Ok(LoginCallbackSubmission::ContinueTo(_)) => oauth_callback_page(400, "登录回调仍需继续，请重新发起网页登录。"),
        Err(error) => oauth_callback_page(error.status_code, &format!("登录失败：{}", error.message)),
    }
}

// ─── POST /api/session/refresh ──────────────────────────────

/// POST /api/session/refresh
///
/// when ** catch → errorPayload**（OpenAI `{error:{message,type}}`），
/// not is API `{success:false,error}` —— Node this in try ，
/// new failure response and /api/accounts/refresh not ， use 。
///
/// ── （2026-10 ）───────────────────────────────
/// this `AuthService::refresh_stored_session`， that is WorkBuddy 
/// `/auth/token/refresh`。 to ，
/// ** by current to **（`refresh_access_token`）—— and 
/// `/api/accounts/refresh` 1 ， is not id（defaultcurrent）。
pub async fn session_refresh(State(state): State<ServerState>) -> Response {
    let Some(entry) = state.store().get_active_entry() else {
        return session_error(401, "当前没有可刷新的登录态");
    };
    let Some(provider) = state.store().account_provider_of(&entry.id) else {
        return session_error(401, "当前没有可刷新的登录态");
    };
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
        return session_error(400, format!("未知的提供商：{provider}"));
    };
    match adapter.refresh_access_token(state.store(), &entry.id).await {
        Ok(_) => ok_json(json!({ "session": state.auth().get_status() })),
        Err(error) => {
            logging::log("[Auth]", &format!("❌ {}", error.message));
            use axum::response::IntoResponse;
            error.into_response()
        }
    }
}

/// 1 OpenAI errorresponse（`{error:{message,type}}`）。
fn session_error(status: i32, message: impl Into<String>) -> Response {
    use axum::response::IntoResponse;
    crate::server::errors::GatewayError::with_status(status, message).into_response()
}

// ─── POST /api/session/logout and POST /auth/logout ─────────

pub async fn session_logout(State(state): State<ServerState>) -> Response {
    state.auth().clear_session();
    crate::server::http::ok_empty()
}

/// POST /auth/logout —— and /api/session/logout 1 and 1 response
pub async fn auth_logout(State(state): State<ServerState>) -> Response {
    state.auth().clear_session();
    crate::server::http::ok_empty()
}

// ─── POST /auth/login ───────────────────────────────────────

/// （ for server.mjs 601-610 `runLogin`）： etc response。
///
/// 桌面端不用这条（它走 /api/session/login/* ）， is as 
/// and Node / 1 。
///
/// ── already not in （2026-10 ）────────────────────────
/// this `LoginService::run_login`， that is WorkBuddy 
/// `auth/state` → `auth/token` → `login/account` 。 that already to 
/// （`workbuddy::login`），** use **：
/// need → to use → by to `login.poll` →
/// when 。 and is 「」（ this in request）。
///
/// ── ──────────────────────────────────────────────
/// `provider`（`workbuddy` / `workbuddy-intl`， and its 1 ：
/// ）， its `edition`（Node has ， use in use ），
/// no has then 。
///
/// ── supports ────────────────────────────────────────────
/// （`poll_interval_ms == 0`） in none —— no has 
/// 。 that and to ， not is to 5 minutes when 。
pub async fn auth_login(State(_state): State<ServerState>, body: Bytes) -> Response {
    let payload = parse_body(&body).ok();
    let provider = payload
        .as_ref()
        .and_then(|payload| payload.get("provider").and_then(Value::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            // no has `provider` when by `edition` 1 （ use ）
            let edition = payload
                .as_ref()
                .and_then(|payload| payload.get("edition").and_then(Value::as_str))
                .filter(|value| !value.is_empty());
            crate::server::core::providers::workbuddy::Region::from_edition_id(edition)
                .provider_id()
                .to_string()
        });
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
        return management_error(400, format!("unknown ：{provider}"));
    };
    let label = adapter.label();
    if !adapter.supports_web_login() {
        return management_error(
            400,
            format!("{label}unsupported， use 「」 or 「」"),
        );
    }
    let Some(start) = adapter.login_start_async().await else {
        return management_error(400, format!("{label} can ，"));
    };
    if start.poll_interval_ms == 0 {
        return management_error(
            400,
            format!(
                "{label} need etc ，none；\
                  use 「」， or use 「」"
            ),
        );
    }
    logging::log("[Login]", " in in down and ：");
    logging::console_line("[Login]", &format!("  {}", start.url));
    logging::log("[Login]", " after will get and token…");

    let interval = Duration::from_millis(start.poll_interval_ms.max(1_000));
    let deadline = tokio::time::Instant::now() + Duration::from_millis(LOGIN_TIMEOUT_MS);
    loop {
        tokio::time::sleep(interval).await;
        if tokio::time::Instant::now() >= deadline {
            return management_error(
                504,
                format!("{label} when （{} minutes），", LOGIN_TIMEOUT_MS / 60_000),
            );
        }
        match adapter.login_poll_async(&start.flow).await {
            Ok(agent2api_provider_protocol::LoginPoll::Pending { .. }) => continue,
            Ok(agent2api_provider_protocol::LoginPoll::Done { account }) => {
                let public = account.public.clone();
                return match adapter.persist_login_account(&account) {
                    Ok(account_id) => {
                        logging::log(
                            "[Login]",
                            &format!("✅ {label}， already "),
                        );
                        crate::server::http::raw_json(json!({
                            "accountId": account_id,
                            "provider": provider,
                            "account": public,
                        }))
                    }
                    Err(error) => management_error(error.status_code, error.message),
                };
            }
            Ok(agent2api_provider_protocol::LoginPoll::Failed { message }) => {
                return management_error(400, message)
            }
            Err(error) => return management_error(error.status_code, error.message),
        }
    }
}
