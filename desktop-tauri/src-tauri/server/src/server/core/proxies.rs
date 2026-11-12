//! ：config、Clash Verge and use 1 、 and 。
//! old ， as unavailable。
//! request by `core::egress` ； use `core::proxy_rotate`。

use serde_json::{json, Map, Value};

pub use crate::server::core::clash::{clash_proxy_options, CLASH_MIXED_UID};

const MAX_HOST_LENGTH: usize = 255;
const MAX_USER_LENGTH: usize = 200;
const MAX_LABEL_LENGTH: usize = 100;

/// source （`core::proxy_rotate` config）
pub const ROTATE_SOURCE: &str = "rotate";

/// this proxy config is not is （ source； is by 1 ）
pub fn is_rotate_config(config: &Value) -> bool {
    config.get("source").and_then(Value::as_str) == Some(ROTATE_SOURCE)
}

/// configerror（ for Node ProxyConfigError， 400）。
///
/// 1 not is GatewayError： by need by to 400，
/// GatewayError default is 500（ up /error）。
#[derive(Clone, Debug)]
pub struct ProxyConfigError {
    pub message: String,
    pub status_code: i32,
}

impl ProxyConfigError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), status_code: 400 }
    }
}

impl std::fmt::Display for ProxyConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

// ─── 1 and ─────────────────────────────────────────────

/// empty and （ for Node `cleanString`： 1 empty）
fn clean_string(value: Option<&Value>, max: usize) -> String {
    let Some(Value::String(text)) = value else {
        return String::new();
    };
    let trimmed = text.trim();
    trimmed.chars().take(max).collect()
}

/// Node `Number(value)` down 「 is 1..65535 」
fn valid_port(value: Option<&Value>) -> Option<u16> {
    let number = match value {
        Some(Value::Number(number)) => number.as_f64()?,
        // and Node Number('8080') 1 ：
        Some(Value::String(text)) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    if !number.is_finite() || number.fract() != 0.0 || !(1.0..=65535.0).contains(&number) {
        return None;
    }
    Some(number as u16)
}

/// 1 config（ API ）。
///
/// return `Ok(None)`（none） or `Ok(Some())`； ProxyConfigError
/// （→ HTTP 400）。`source` when by ，
/// （ `data` ， or file JSON）。
pub fn normalize_account_proxy(input: &Value) -> Result<Option<Value>, ProxyConfigError> {
    if input.is_null() || input.as_str() == Some("") {
        return Ok(None);
    }
    let Some(object) = input.as_object() else {
        return Err(ProxyConfigError::new("代理配置必须是对象或 null"));
    };

    let explicit_source = clean_string(object.get("source"), 20).to_lowercase();
    if explicit_source == "direct" {
        return Ok(Some(json!({ "source": "direct" })));
    }
    let has_listener = object
        .get("listenerUid")
        .and_then(Value::as_str)
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);
    let has_host = object
        .get("host")
        .and_then(Value::as_str)
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false);
    let source = if !explicit_source.is_empty() {
        explicit_source
    } else if has_listener {
        "clash".to_string()
    } else if has_host {
        "custom".to_string()
    } else {
        String::new()
    };
    if source.is_empty() {
        return Err(ProxyConfigError::new(
            "代理配置缺少 source（clash / custom / pool / rotate）",
        ));
    }

    // ：「」（ or ）， in 。
    // and pool use 1 ： is in / is use to **** when ，
    // not down （ no when down this config）。
    if source == ROTATE_SOURCE {
        let all = match object.get("all") {
            None | Some(Value::Null) => true,
            Some(Value::Bool(flag)) => *flag,
            Some(_) => return Err(ProxyConfigError::new("轮询的 all 必须是布尔值")),
        };
        if all {
            // 「」 not proxyIds： will 「 after add need not need 」
            // 1 no （ as ）。 1 1 
            return Ok(Some(json!({ "source": ROTATE_SOURCE, "all": true })));
        }
        let mut ids: Vec<String> = Vec::new();
        if let Some(items) = object.get("proxyIds") {
            let Some(items) = items.as_array() else {
                return Err(ProxyConfigError::new("轮询的 proxyIds 必须是数组"));
            };
            for item in items {
                let id = clean_string(Some(item), MAX_LABEL_LENGTH);
                if id.is_empty() || ids.iter().any(|existing| existing == &id) {
                    continue;
                }
                ids.push(id);
                if ids.len() >= crate::server::core::proxy_rotate::MAX_IDS {
                    break;
                }
            }
        }
        if ids.is_empty() {
            return Err(ProxyConfigError::new(
                "轮询至少要选择一条代理（或改选「全部」）",
            ));
        }
        return Ok(Some(json!({
            "source": ROTATE_SOURCE,
            "all": false,
            "proxyIds": ids,
        })));
    }

    if source == "subscription" {
        return Err(ProxyConfigError::new(crate::server::core::proxy_pool::RETIRED_HINT));
    }
    if source == "clash" {
        let listener_uid = clean_string(object.get("listenerUid"), MAX_LABEL_LENGTH);
        if listener_uid.is_empty() {
            return Err(ProxyConfigError::new("缺少 Clash 监听器 uid"));
        }
        return Ok(Some(json!({ "source": "clash", "listenerUid": listener_uid })));
    }
    // use ： id（； is in / is use to **** when 
    // —— and 1 ， not 「」 ）
    if source == "pool" {
        let proxy_id = clean_string(object.get("proxyId"), MAX_LABEL_LENGTH);
        if proxy_id.is_empty() {
            return Err(ProxyConfigError::new("缺少代理池条目 id"));
        }
        return Ok(Some(json!({ "source": "pool", "proxyId": proxy_id })));
    }
    if source != "custom" {
        return Err(ProxyConfigError::new(format!("不支持的代理来源: {source}")));
    }

    let protocol = {
        let cleaned = clean_string(object.get("protocol"), 10).to_lowercase();
        if cleaned.is_empty() { "http".to_string() } else { cleaned }
    };
    if protocol != "http" && protocol != "socks5" {
        return Err(ProxyConfigError::new("代理协议只支持 http 或 socks5"));
    }
    let host = clean_string(object.get("host"), MAX_HOST_LENGTH);
    if host.is_empty() {
        return Err(ProxyConfigError::new("缺少代理主机地址"));
    }
    let Some(port) = valid_port(object.get("port")) else {
        return Err(ProxyConfigError::new("代理端口必须是 1-65535 的整数"));
    };
    let label = clean_string(object.get("label"), MAX_LABEL_LENGTH);
    // label when by `://:` ， before this 
    let label = if label.is_empty() {
        format!("{protocol}://{host}:{port}")
    } else {
        label
    };

    let mut normalized = Map::new();
    normalized.insert("source".to_string(), Value::String("custom".to_string()));
    normalized.insert("protocol".to_string(), Value::String(protocol));
    normalized.insert("host".to_string(), Value::String(host));
    normalized.insert("port".to_string(), Value::from(port));
    normalized.insert(
        "username".to_string(),
        Value::String(clean_string(object.get("username"), MAX_USER_LENGTH)),
    );
    normalized.insert(
        "password".to_string(),
        Value::String(clean_string(object.get("password"), MAX_USER_LENGTH)),
    );
    normalized.insert("label".to_string(), Value::String(label));
    Ok(Some(Value::Object(normalized)))
}

/// ：success when to available ，failure when `error` Reason
/// （ use and ， for Node `{ error }` ）。
///
/// `port` is `Option<u16>`：Node custom `Number(config.port)`，
/// will `NaN` → JSON `null`。 this that null，
/// not is 1 0 —— before to null 「 this 」。
#[derive(Clone, Debug)]
pub struct ResolvedProxy {
    pub source: String,
    pub protocol: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: String,
    pub label: String,
}

impl ResolvedProxy {
    /// JSON （/ → null， and Node NaN → null 1 ）
    pub fn port_json(&self) -> Value {
        self.port.map(Value::from).unwrap_or(Value::Null)
    }

    /// from JSON （ will `proxy` is this ，
    /// by account_store `session_from_record` use `to_json` success）。
    ///
    /// ：`Ok(None)` = no has （）；`Ok(Some(...))` = available；
    /// `Err(Reason)` = ****（ / ）。
    /// must and 「 no 」：（ `data` 
    /// `proxy` ， or file） `port: "abc"` when ，Node will `NaN`
    /// ProxyAgent and failure（「❌ none」）—— if this 
    /// by ， use will to 「✅ available」， that is in 。
    /// use to Err when by （available），
    /// **** need Reason。
    pub fn from_json(value: &Value) -> Result<Option<Self>, String> {
        if value.is_null() {
            return Ok(None);
        }
        let text = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let host = text("host");
        if host.is_empty() {
            return Err("代理配置缺少主机地址".to_string());
        }
        let Some(port) = valid_port(value.get("port")) else {
            return Err("代理端口必须是 1-65535 的整数".to_string());
        };
        let protocol = {
            let protocol = text("protocol");
            if protocol.is_empty() { "http".to_string() } else { protocol }
        };
        Ok(Some(ResolvedProxy {
            source: text("source"),
            protocol,
            host,
            port: Some(port),
            username: text("username"),
            password: text("password"),
            label: text("label"),
        }))
    }
}

/// will （`session.proxy`）。
///
/// will already up `proxy`（success） or `proxyError`（failure，
/// when `proxy` as null）—— to None 「」： no 、
/// failure（ use already Clash ）、 and ，
/// this in is 1 。 will 1 。
pub fn session_proxy(session: &Value) -> Option<ResolvedProxy> {
    match ResolvedProxy::from_json(session.get("proxy").unwrap_or(&Value::Null)) {
        Ok(proxy) => proxy,
        Err(reason) => {
            crate::server::logging::log(
                "[Upstream]",
                &format!("⚠️ 账号代理不可用（{reason}），本次回退直连"),
            );
            None
        }
    }
}

/// JS （`Boolean(x)`）：null/false/0/"" as ， its （/ for ） as 。
/// used for Node `config.label || ...` this 。
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().map(|item| item != 0.0).unwrap_or(false),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JS etc ：`a` is `undefined`、 null is `null`、
/// its by `String(a)` （ for / in JS is `[object Object]` /
/// ， this use JSON —— this can can ，
/// is 「 not need 」， not is JS toString）。
/// used for Node label / as （ `resolve_account_proxy`）。
fn js_interpolation(value: Option<&Value>) -> String {
    match value {
        None => "undefined".to_string(),
        Some(Value::Null) => "null".to_string(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(Value::Object(_)) => "[object Object]".to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(","),
    }
}

/// failure Reason（ and success）
#[derive(Clone, Debug)]
pub enum ProxyResolution {
    Resolved(ResolvedProxy),
    Failed(String),
}

impl ProxyResolution {
    pub fn error(&self) -> Option<&str> {
        match self {
            ProxyResolution::Resolved(_) => None,
            ProxyResolution::Failed(message) => Some(message.as_str()),
        }
    }

    pub fn resolved(&self) -> Option<&ResolvedProxy> {
        match self {
            ProxyResolution::Resolved(proxy) => Some(proxy),
            ProxyResolution::Failed(_) => None,
        }
    }

    /// Node resolveAccountProxy return（JSON， error ）。
    /// `describe_account_proxy`（ config/label ），
    /// this and for ， when for 。
    #[allow(dead_code)]
    pub fn to_json(&self) -> Value {
        match self {
            ProxyResolution::Resolved(proxy) => json!({
                "source": proxy.source,
                "protocol": proxy.protocol,
                "host": proxy.host,
                "port": proxy.port_json(),
                "username": proxy.username,
                "password": proxy.password,
                "label": proxy.label,
            }),
            ProxyResolution::Failed(message) => json!({ "error": message }),
        }
    }
}

/// configavailable 。
///
/// Node `resolveAccountProxy`，**including for **：
/// - protocol `socks5`， its （ undefined） http
/// - host （Node not ， will undefined → ）
/// - port `Number()`： NaN → JSON null（ not is 0）
/// - label when by JS ， `undefined://bad:undefined` this 
/// is ** as ** —— use this （ before as 
/// 「 undefined://…」 1 default）
pub fn resolve_account_proxy(config: Option<&Value>) -> Option<ProxyResolution> {
    let config = config?;
    if config.is_null() {
        return None;
    }
    // for （//） in JS `.source` undefined，
    // is to 「unsupported : undefined」—— 
    // when is this ， not 「」 
    let Some(object) = config.as_object() else {
        return Some(ProxyResolution::Failed("不支持的代理来源: undefined".to_string()));
    };
    // source when is undefined（Node is `config.source` ）。
    // this will in 「」，must 1 ——
    // to empty will use to 「unsupported : 」 this is 
    let source = object
        .get("source")
        .map(|value| js_interpolation(Some(value)))
        .unwrap_or_else(|| "undefined".to_string());

    if source == "direct" {
        return None;
    }

    // ： 1 available（** not **）。request 
    // `resolve_account_proxy_rotating`； this to and 「 1 use not 1 」
    // （module header）—— is 「 this config in can not can use 」， not is 「 this this 」。
    if source == ROTATE_SOURCE {
        return Some(
            match crate::server::core::proxy_rotate::first_usable(config) {
                Ok(proxy) => ProxyResolution::Resolved(proxy),
                Err(reason) => ProxyResolution::Failed(reason),
            },
        );
    }

    // use by 1 ； not in 、 already use or already down Reason，
    // this —— 「」、 when 。
    if source == "pool" {
        let proxy_id = object.get("proxyId").and_then(Value::as_str).unwrap_or("");
        if proxy_id.is_empty() {
            return Some(ProxyResolution::Failed("代理引用缺少条目 id".to_string()));
        }
        return Some(
            match crate::server::core::proxy_pool::resolve_reference(proxy_id) {
                Ok(proxy) => ProxyResolution::Resolved(proxy),
                Err(reason) => ProxyResolution::Failed(reason),
            },
        );
    }

    if source == "custom" {
        let protocol = if object.get("protocol").and_then(Value::as_str) == Some("socks5") {
            "socks5"
        } else {
            "http"
        };
        // host ： when Node will / to JSON， that 
        // none use ， this 1 to empty —— use （session_proxy） to empty host
        // and ， `host: 123` 1 
        let host = object.get("host").and_then(Value::as_str).unwrap_or("").to_string();
        let port = valid_port(object.get("port"));
        // label when by JS （set undefined）；
        // to use （Node is `config.label || \`...\``，/ this 
        // will use ， this use js_interpolation ）
        let label = match object.get("label") {
            Some(value) if js_truthy(value) => js_interpolation(Some(value)),
            _ => format!(
                "{}://{}:{}",
                js_interpolation(object.get("protocol")),
                js_interpolation(object.get("host")),
                js_interpolation(object.get("port")),
            ),
        };
        return Some(ProxyResolution::Resolved(ResolvedProxy {
            source: "custom".to_string(),
            protocol: protocol.to_string(),
            host,
            port,
            username: object
                .get("username")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            password: object
                .get("password")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            label,
        }));
    }
    if source != "clash" {
        return Some(ProxyResolution::Failed(if source == "subscription" {
            crate::server::core::proxy_pool::RETIRED_HINT.to_string()
        } else {
            format!("不支持的代理来源: {source}")
        }));
    }

    let snapshot = crate::server::core::clash::clash_snapshot();
    if !snapshot.available {
        let detail = snapshot.error.as_ref()
            .map(|error| format!("（{error}）"))
            .unwrap_or_default();
        return Some(ProxyResolution::Failed(format!("Clash Verge 配置不可用{detail}")));
    }
    let listener_uid = object.get("listenerUid").and_then(Value::as_str).unwrap_or("");
    let (port, label) = if listener_uid == CLASH_MIXED_UID {
        let Some(port) = snapshot.mixed_port else {
            return Some(ProxyResolution::Failed("Clash Verge 未启用混合端口".to_string()));
        };
        (port, format!("Clash 混合端口 {port}"))
    } else {
        let Some(listener) = snapshot.listeners.iter().find(|item| item.uid == listener_uid) else {
            return Some(ProxyResolution::Failed(format!(
                "Clash Verge 中找不到监听器「{listener_uid}」（可能已在 Clash 中删除）"
            )));
        };
        if !listener.enabled {
            return Some(ProxyResolution::Failed(format!(
                "Clash Verge 监听器「{}」已禁用", listener.name
            )));
        }
        (listener.port, format!("{}（:{}）", listener.name, listener.port))
    };
    Some(ProxyResolution::Resolved(ResolvedProxy {
        source: "clash".to_string(),
        protocol: "http".to_string(),
        host: "127.0.0.1".to_string(),
        port: Some(port),
        username: String::new(),
        password: String::new(),
        label,
    }))
}

/// ****： and [`resolve_account_proxy`] rotate —— by `rotate_key`
/// down 1 （ before ）， its source to that 。
///
/// `rotate_key` is （** id**）： 1 request to 
/// not ， not （ `core::proxy_rotate` module header）。
/// has 1 （ will 、） this this ：、
/// this 「 1 use not 1 」 use ，otherwise
/// 1 new （ is 「 in not IP」， 
/// 1 ， etc ）。
pub fn resolve_account_proxy_rotating(
    config: Option<&Value>,
    rotate_key: &str,
) -> Option<ProxyResolution> {
    let config = config?;
    if config.is_null() {
        return None;
    }
    if is_rotate_config(config) {
        return Some(match crate::server::core::proxy_rotate::next(config, rotate_key) {
            Ok(proxy) => ProxyResolution::Resolved(proxy),
            Err(reason) => ProxyResolution::Failed(reason),
        });
    }
    resolve_account_proxy(Some(config))
}

/// （， not —— and Node 1 ，
/// in config ）。
///
/// nonereturn Null；failure when `label` as 「failure」 and `error`
/// （ before 「 when will 」）。
///
/// 1 （ down ）： 「current」 is ， need is 
/// 「、available」， not can use 1 label 「current」。
pub fn describe_account_proxy(config: Option<&Value>) -> Value {
    let Some(config) = config.filter(|value| !value.is_null()) else {
        return Value::Null;
    };
    if config.get("source").and_then(Value::as_str) == Some("direct") {
        return json!({
            "source": "direct",
            "config": { "source": "direct" },
            "label": "直连",
            "error": Value::Null,
        });
    }
    if is_rotate_config(config) {
        return describe_rotate_proxy(config);
    }
    let Some(resolution) = resolve_account_proxy(Some(config)) else {
        return Value::Null;
    };
    let source = config
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    match &resolution {
        ProxyResolution::Failed(message) => json!({
            "source": source,
            "label": "解析失败",
            "error": message,
            "config": config,
        }),
        ProxyResolution::Resolved(proxy) => json!({
            // source use ** that **（pool / clash / custom）：pool use 
            // success after is clash / custom， up need 「 use 
            // 」—— before in up 。 
            // （source ） to source
            "source": if source.is_empty() { proxy.source.clone() } else { source.clone() },
            "protocol": proxy.protocol,
            "host": proxy.host,
            "port": proxy.port,
            "label": proxy.label,
            "error": Value::Null,
            "config": config,
        }),
    }
}

/// （`describe_account_proxy` rotate ）。
///
/// and its （`source` / `label` / `error` / `config`），** not ** to 
/// host / port： no has 「current」 this （ down 1 request can can ）， to 1 
/// will as 「 this is that 」。 `rotate` for is to ：
/// total、available（ `proxy_rotate::entry_ready`）。
///
/// `label` is and ；`error` in ** 1 use 
/// not ** when to —— that when will ， by has 
/// （ and use 「unavailable」 1 ）。
fn describe_rotate_proxy(config: &Value) -> Value {
    let summary = crate::server::core::proxy_rotate::summary(config);
    let label = if summary.all {
        format!("轮询全部代理（{} 个出口）", summary.total)
    } else {
        format!("轮询指定代理（{} 个出口）", summary.total)
    };
    json!({
        "source": ROTATE_SOURCE,
        "label": label,
        "error": summary.error,
        "config": config,
        "rotate": {
            "all": summary.all,
            "total": summary.total,
            "usable": summary.usable,
        },
    })
}
