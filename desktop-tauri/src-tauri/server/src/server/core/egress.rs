//! 1 ： by reqwest Client + 。
//!
//! for Node src/workbuddy-proxy.mjs `dispatchFor` / `createDispatcher` /
//! `proxyFetch` / `testProxyConnectivity` 。
//!
//! ── Why need need Client ───────────────────────────────────
//! Node use undici dispatcher ： 1 dispatcher（），
//! otherwise request need ** new and 1 TCP **，SSE down 。
//! reqwest etc is `Client`：`Client` ，「 1 1 
//! Client」 is 「 1 1 」。
//!
//! ── when （ need ：Node and reqwest not is 1 1 for ）──────
//! Node （undici） to is **** when ：
//! connectTimeout: 30s TCP （ to that 1 ）
//! headersTimeout: 600s → to response
//! bodyTimeout: 0 response** not when **（SSE use signal ）
//!
//! reqwest has ， and not ：
//! connect_timeout ← for undici connectTimeout（，），
//! set「request when → in when 」
//! read_timeout ← after ：used for** 1 **
//! read（ to 、 and after 1 body），success after 
//! new when 。「 etc response when 」 and 「empty when 」
//! （`TimeoutSettings::read_timeout_backstop_ms`），
//! not will as 。
//! ** not use `.timeout()`（total when ）**： that will SSE in when none，
//! Node is `bodyTimeout: 0`（ not when ）。
//! → and Node ：body limit in 「empty」set（Node is not when ）。
//! this is ** has **：SSE 15-30 seconds 1 ， when no has 
//! already ， when none has use 。
//!
//! in when up （， can use Client
//! when ）： etc response in `upstream::request`、empty in `upstream::ForwardStream`、
//! total in `upstream::aggregate`。 this is 。
//!
//! ── ────────────────────────────────────────────────
//! Node ：`MAX_DISPATCHERS = 24`， when ** use ** that （Map 
//! ， in when to ）。 this use ：`HashMap` + use ，
//! when （ use ）， and after 1 use 。
//! and Node ：Node will `destroyDispatcher` ，
//! reqwest `Client` no has 「」（ can drop，drop when 
//! use ）。 this is drop —— 1 ， when 
//! is has in request has this Client Arc （ in request will ）。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::{json, Value};

use crate::server::config::TimeoutSettings;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::logging;

/// default User-Agent，**mustset**。
///
/// ── Why this not is 「 」 is required ────────────────────────
/// reqwest default** not ** User-Agent ；undici（Node use fetch）default
/// `undici`。（2026-09） in User-Agent when return
/// `HTTP 403 {"code":10085,"msg":"request not ， has "}`，
/// up anyempty UA return `code:0`。
///
/// ： not this ，Rust will in 「Node 」 1 up 403。
/// this `undici` and Node 1 —— up to is 1 UA，
/// not 「 WorkBuddy client UA」 this will up 
/// （ Node not client UA， has banner ）。
///
/// clientdefault** not will **request（reqwest in HeaderMap empty when ），
/// banner `whitelistHeaders` that WorkBuddy UA 。
const DEFAULT_USER_AGENT: &str = "undici";

/// Client limit（ for Node MAX_DISPATCHERS）
const MAX_CLIENTS: usize = 24;

/// IP ： not to ， not 
const IP_ECHO_URL: &str = "https://api.ipify.org?format=json";
/// default：** up ** not is （ test_connectivity）。
/// use can in 「」 set up （`core::proxy_settings::test_url`），
/// not configured（empty） when to this —— and `core::proxy_settings::DEFAULT_TEST_URL`
/// 1 ， 1 need 1 。
const DEFAULT_TEST_URL: &str = "https://copilot.tencent.com/v2/config";
/// default when 
const DEFAULT_TEST_TIMEOUT_MS: u64 = 15_000;
/// IP when limit（ when to ， not need in this ）
const IP_LOOKUP_MAX_MS: u64 = 6_000;

/// 1 Client：`( 1 use , client)`
struct CacheEntry {
    used_at: u64,
    client: Arc<reqwest::Client>,
}

/// → Client 。
///
/// is ** not ** `://:`（ and Node dispatcher 1 ）：
/// will in 1 ， and not not this use 1 。
/// `reqwest::Proxy::basic_auth` in client up ，。
#[derive(Default)]
struct ClientCache {
    entries: HashMap<String, CacheEntry>,
    /// use （ Node use Map LRU）
    clock: u64,
}

static CLIENTS: Mutex<Option<ClientCache>> = Mutex::new(None);

/// ：`://:`（ not ， by ClientCache ）。
///
/// （proxy as None） not in this —— `DIRECT` ，
/// 「 」 not will client 1 。
const DIRECT_KEY: &str = "__direct__";

/// ；（ ） when return None，
/// use （ and Node `NaN:NaN` will 1 not up dispatcher
/// not —— this ， and Reason）。
///
/// `timeouts` in ： when （ / after ） is ** Client when **
/// （reqwest no has request ），config 1 must new Client ——
/// ，「set → down 1 use in new → new client」 is ，
/// old for by LRU （ in request old Client Arc ）。
fn cache_key(proxy: Option<&ResolvedProxy>, timeouts: &TimeoutSettings) -> String {
    let egress = match proxy {
        None => DIRECT_KEY.to_string(),
        Some(proxy) => format!(
            "{}://{}:{}",
            proxy.protocol,
            proxy.host,
            proxy.port.map(|port| port.to_string()).unwrap_or_else(|| "?".to_string())
        ),
    };
    // when ： up 
    format!(
        "{egress}|c{}|r{}",
        timeouts.connect_ms(),
        timeouts.read_timeout_backstop_ms()
    )
}

/// URI：`://[ use [:]@]:`。
///
/// use / by Node `buildProxyUrl` （ for JS 
/// `encodeURIComponent`）—— `@` `:` `/` when not will URI 。
/// IPv6 by Node （`::1` → `[::1]`）。
///
/// **socks5 socks5h**（ has Node URI ， as ）：
/// undici Socks5ProxyAgent by （ATYP=0x03） to ，
/// 「」； reqwest `socks5://` is ****（socks5h is ）。
/// use socks5 is as DNS ， use `socks5://` will this 
/// empty。 this in reqwest URI when socks5h ——
/// 、UI up is socks5， has etc scheme。
fn build_proxy_url(proxy: &ResolvedProxy) -> String {
    let auth = if proxy.username.is_empty() {
        String::new()
    } else if proxy.password.is_empty() {
        format!("{}@", urlencoding(&proxy.username))
    } else {
        format!("{}:{}@", urlencoding(&proxy.username), urlencoding(&proxy.password))
    };
    let needs_brackets = proxy.host.contains(':') && !proxy.host.starts_with('[');
    let host = if needs_brackets {
        format!("[{}]", proxy.host)
    } else {
        proxy.host.clone()
    };
    let scheme = if proxy.protocol == "socks5" { "socks5h" } else { &proxy.protocol };
    format!("{}://{}{}:{}", scheme, auth, host, proxy.port.unwrap_or(0))
}

/// （ for JS `encodeURIComponent`）。
///
/// and core::auth::urlencoding is 1 then ， that in auth 
/// is 「」 ； this as not egress auth（auth already 
/// ）， 1 。 then has 4 ， 。
fn urlencoding(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.as_bytes() {
        let ch = *byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
        {
            out.push(ch);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// URI ，（ not ）
fn describe_proxy(proxy: &ResolvedProxy) -> &str {
    if proxy.label.is_empty() {
        &proxy.host
    } else {
        &proxy.label
    }
}

/// ClientBuilder set（ use that 1 ）。
///
/// is as `build_client`（） and `build_system_proxy_client`
/// （）「」 1 ， when / UA / must 1 。
fn base_builder(timeouts: &TimeoutSettings) -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(timeouts.connect_ms()))
        // when （ etc response + after ，set ）；
        // ** no has **total when （.timeout()）—— that will SSE 。 
        // in when up （ egress and module header）
        .read_timeout(Duration::from_millis(timeouts.read_timeout_backstop_ms()))
        // default UA（ by DEFAULT_USER_AGENT）： not will as 「request not 」
        .user_agent(DEFAULT_USER_AGENT)
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .pool_max_idle_per_host(8)
}

/// 1 for reqwest Client。
///
/// Note `Builder::proxy()` will ** when **（reqwest 
/// "Adding a proxy will disable the automatic usage of the system proxy"）——
/// this is need ： by config， not can `HTTPS_PROXY` 
/// （otherwise「」 can can up ， and UI not ）。
/// ，**** Client when must `.no_proxy()`：reqwest default will 
/// set， not use up `HTTPS_PROXY` will 
/// 「config as 」。
fn build_client(proxy: Option<&ResolvedProxy>, timeouts: &TimeoutSettings) -> Result<reqwest::Client, String> {
    build_client_with(proxy, timeouts, false)
}

/// [`build_client`] ：`no_redirect` as when ** not 3xx**。
///
/// Why need need this ：（ new OAuth2 + PKCE，
/// `providers::custom::sensenova_auth`） need 1 `Location` and 
/// `Set-Cookie`， will this in response。reqwest 0.12 can 
/// in Client when （`RequestBuilder` up no has request `redirect()`），
/// 1 not is requestparameter。
fn build_client_with(
    proxy: Option<&ResolvedProxy>,
    timeouts: &TimeoutSettings,
    no_redirect: bool,
) -> Result<reqwest::Client, String> {
    let mut builder = base_builder(timeouts);
    if no_redirect {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }

    match proxy {
        None => {
            // ：config as must 
            builder = builder.no_proxy();
        }
        Some(proxy) => {
            let port = proxy
                .port
                .ok_or_else(|| format!("代理端口非法（{}）", describe_proxy(proxy)))?;
            let url = build_proxy_url(&ResolvedProxy { port: Some(port), ..proxy.clone() });
            // Proxy::all return Result：release is panic=abort， not can use unwrap
            let parsed = reqwest::Proxy::all(&url)
                .map_err(|error| format!("代理地址无效（{}）: {error}", describe_proxy(proxy)))?;
            // reqwest from URI use / as basic auth（build_proxy_url already ），
            // http/socks5 supports； this not basic_auth，
            // 
            builder = builder.proxy(parsed);
        }
    }

    builder
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

/// 1 **** Client（Windows / macOS set + ）。
///
/// ── Why need need ────────────────────────────────────
/// （ / ） reqwest ：
/// by config， not can —— this for 
/// is for ， for 「 no has can 」 request not ：
/// - ****（Antigravity / userinfo / / project ）：
/// not in ， not up 「」；
/// - **Antigravity request** in when （ new / directory）：
/// up is Google， has can ， use up 「 already can 
/// Google that 」 in set（ can is ）。
/// this is 「 and 」：， no is 
/// —— not is 「 1 」。 `client_for(Some(..))`。
fn build_system_proxy_client(timeouts: &TimeoutSettings) -> Result<reqwest::Client, String> {
    // not `.no_proxy()` not `.proxy()`：reqwest `auto_sys_proxy`
    // default（`system-proxy` feature， server/Cargo.toml ）
    base_builder(timeouts)
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

/// ** not **、 not Client（ use new 1 ）。
///
/// need need Cookie、 `Location` / `Set-Cookie` 
/// （ new ， `providers::custom::sensenova_auth`）—— that 1 1 ，
/// will in response， reqwest can Client when 。
/// not LRU is ： and its key not ， 1 will 
/// 「 1 has when 、 has when not 」 this as as can can ；
/// is （ 1 ）， new Client can ignore。
///
/// failure `client_for(proxy)`（ that ）： that will failure，
/// error will to use before —— is not panic（release is panic=abort）。
pub fn client_without_redirects(proxy: Option<&ResolvedProxy>) -> Arc<reqwest::Client> {
    let timeouts = crate::server::config::timeout_settings();
    match build_client_with(proxy, &timeouts, true) {
        Ok(client) => Arc::new(client),
        Err(error) => {
            logging::log(
                "[Upstream]",
                &format!("⚠️ {error}（不跟跳转的出口回退自动跟转）"),
            );
            client_for(proxy)
        }
    }
}

pub fn image_download_client(
    proxy: Option<&ResolvedProxy>,
    system_proxy_when_unset: bool,
    host: &str,
    addresses: &[std::net::SocketAddr],
) -> Result<reqwest::Client, String> {
    resolved_client(proxy, system_proxy_when_unset, host, addresses)
}

pub fn resolved_client(
    proxy: Option<&ResolvedProxy>,
    system_proxy_when_unset: bool,
    host: &str,
    addresses: &[std::net::SocketAddr],
) -> Result<reqwest::Client, String> {
    let timeouts = crate::server::config::timeout_settings();
    let mut builder = base_builder(&timeouts)
        .redirect(reqwest::redirect::Policy::none());
    if !addresses.is_empty() {
        builder = builder.resolve_to_addrs(host, addresses);
    }
    if let Some(proxy) = proxy {
        if proxy.port.is_none_or(|port| port == 0) { return Err("代理端口无效".to_string()); }
        let parsed = reqwest::Proxy::all(build_proxy_url(proxy))
            .map_err(|_| "代理地址无效".to_string())?;
        builder = builder.no_proxy().proxy(parsed);
    } else if !system_proxy_when_unset {
        builder = builder.no_proxy();
    }
    builder.build().map_err(|_| "无法创建出站连接".to_string())
}

pub fn client_for_github(proxy: Option<&ResolvedProxy>) -> Result<Arc<reqwest::Client>, String> {
    let timeouts = crate::server::config::timeout_settings();
    let key = format!("github|{}", cache_key(proxy, &timeouts));
    {
        let mut guard = lock_clients();
        let cache = guard.get_or_insert_with(ClientCache::default);
        cache.clock += 1;
        let now = cache.clock;
        if let Some(entry) = cache.entries.get_mut(&key) {
            entry.used_at = now;
            return Ok(entry.client.clone());
        }
    }
    // check and ，clientfailure not can to client。
    let client = Arc::new(build_client_with(proxy, &timeouts, true)?);
    let mut guard = lock_clients();
    let cache = guard.get_or_insert_with(ClientCache::default);
    cache.clock += 1;
    let now = cache.clock;
    if let Some(entry) = cache.entries.get_mut(&key) {
        entry.used_at = now;
        return Ok(entry.client.clone());
    }
    while cache.entries.len() >= MAX_CLIENTS {
        let Some(oldest) = cache
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.used_at)
            .map(|(key, _)| key.clone())
        else {
            break;
        };
        cache.entries.remove(&oldest);
    }
    cache.entries.insert(
        key,
        CacheEntry {
            used_at: now,
            client: client.clone(),
        },
    );
    Ok(client)
}

/// Client（ 1 ）。
///
/// and `client_for` not ** not LRU**： no has 「parameter」 this 1 ， 1 
/// can 。 need （reqwest in build when 1 ）——
/// and Antigravity request is ， this not 。
pub fn client_for_system_proxy() -> Arc<reqwest::Client> {
    static CLIENT: OnceLock<Arc<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            let timeouts = crate::server::config::timeout_settings();
            Arc::new(build_system_proxy_client(&timeouts).unwrap_or_else(|error| {
                // this and client_for 1 ：failure can can in 
                // TLS after （ not can can ）， 1 can 。 this 
                // **「」**（Client::new() auto_sys_proxy
                // default）， is when 。
                logging::log("[Upstream]", &format!("⚠️ {error}，系统代理出口回退默认客户端"));
                reqwest::Client::new()
            }))
        })
        .clone()
}

/// （ or create） for Client；`proxy` as None when is client。
///
/// return `Arc<Client>`： use this Arc not is Client 
/// （reqwest Client already is Arc， 1 and ）。
///
/// ── Why in ──────────────────────────────────────
/// `reqwest::Proxy::all(...)` for socks5 URI will `url.socket_addrs()` 1 
/// ** DNS **（http not need need ， socks need ）。 is 
/// 「 not 」—— in ， 1 + 。
/// and 1 when Client（ ），
/// after before —— is 1 ， not 。
pub fn client_for(proxy: Option<&ResolvedProxy>) -> Arc<reqwest::Client> {
    // when set use （config 1 ， down 1 in new and new Client， cache_key）
    let timeouts = crate::server::config::timeout_settings();
    let key = cache_key(proxy, &timeouts);
    // ① （path）
    {
        let mut guard = lock_clients();
        let cache = guard.get_or_insert_with(ClientCache::default);
        cache.clock += 1;
        let now = cache.clock;
        if let Some(entry) = cache.entries.get_mut(&key) {
            entry.used_at = now;
            return entry.client.clone();
        }
    }
    // ② in （， can can ）
    let client = Arc::new(build_client(proxy, &timeouts).unwrap_or_else(|error| {
        // this ** in **（ not 「unavailable」 that request）：
        // 1 —— failure Client will 
        // （ down ③）， after request in 、 not to this ，** not 
        // request**。 and its 1 ： will not will request 1 。
        logging::log("[Upstream]", &format!("⚠️ {error}，本次回退直连"));
        // failure client：TLS after failure when no has ， can return
        // 1 「request when 」 client（ not will failure）。 this `.no_proxy()`：
        // is ****（ / ）， not can 
        // —— `Client::new()` auto_sys_proxy default（ Cargo.toml 
        // `system-proxy` ）， use will 「」 in path up 
        // ； is Client::new()（ build failure）。
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    }));
    // ③ （ and ）
    let mut guard = lock_clients();
    let cache = guard.get_or_insert_with(ClientCache::default);
    cache.clock += 1;
    let now = cache.clock;
    // and down can can already has —— use to that ， 1 
    let client = match cache.entries.get(&key) {
        Some(entry) => entry.client.clone(),
        None => {
            // use （Node idleDispatcher ：Map + in ）
            while cache.entries.len() >= MAX_CLIENTS {
                let Some(oldest) = cache
                    .entries
                    .iter()
                    .min_by_key(|(_, entry)| entry.used_at)
                    .map(|(key, _)| key.clone())
                else {
                    break;
                };
                cache.entries.remove(&oldest);
            }
            cache.entries.insert(key, CacheEntry { used_at: now, client: client.clone() });
            client
        }
    };
    client
}

/// Client ； in （ panic） not ， use 
fn lock_clients() -> std::sync::MutexGuard<'static, Option<ClientCache>> {
    match CLIENTS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

// ─── ─────────────────────────────────────────

/// （ for Node testProxyConnectivity return for ）。
#[derive(Clone, Debug)]
pub struct ConnectivityResult {
    pub success: bool,
    pub status: Option<u16>,
    pub ip: String,
    pub duration_ms: i64,
    pub error: Option<String>,
}

impl ConnectivityResult {
    /// response JSON：`{success, status?, ip?, durationMs, error?}`
    /// （ in has when ， and Node for 1 ）
    pub fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        map.insert("success".to_string(), Value::Bool(self.success));
        if self.success {
            if let Some(status) = self.status {
                map.insert("status".to_string(), Value::from(status));
            }
            map.insert("ip".to_string(), Value::String(self.ip.clone()));
        }
        map.insert("durationMs".to_string(), Value::from(self.duration_ms));
        if let Some(error) = &self.error {
            map.insert("error".to_string(), Value::String(error.clone()));
        }
        Value::Object(map)
    }
}

/// IP （，failurereturnempty）。
///
/// ipify not is from up response —— up not client IP；
/// not to not unavailable， this error 1 。
async fn read_exit_ip(client: &reqwest::Client, proxy: Option<&ResolvedProxy>, timeout_ms: u64) -> String {
    let _ = proxy; // 出口已由 client 决定；保留参数是为了调用处可读
    let request = client
        .get(IP_ECHO_URL)
        .header("Accept", "application/json")
        .timeout(Duration::from_millis(timeout_ms));
    let Ok(response) = request.send().await else {
        return String::new();
    };
    if !response.status().is_success() {
        return String::new();
    }
    let Ok(text) = response.text().await else {
        return String::new();
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(value) => value
            .get("ip")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_default(),
        // return not is JSON（return IP） when to 64 use 
        Err(_) => text.trim().chars().take(64).collect(),
    }
}

/// is available。
///
/// is 「 can up WorkBuddy up 」，** not is ** can ：
/// up in can 、， use will can use 
/// configfailure。 need can to HTTP response（ 401/404 this response）。
///
/// `target_url` as None when use `https://copilot.tencent.com/v2/config`。
pub async fn test_connectivity(
    proxy: Option<&ResolvedProxy>,
    timeout_ms: u64,
    target_url: Option<&str>,
) -> ConnectivityResult {
    let started = logging::now_ms();
    let url = target_url.unwrap_or(DEFAULT_TEST_URL);
    let client = client_for(proxy);
    let request = client
        .get(url)
        .header("Accept", "application/json")
        // request when this （ use is total when ， and Node AbortSignal.timeout ；
        // client up read_timeout not will —— total when ）
        .timeout(Duration::from_millis(timeout_ms));
    match request.send().await {
        Ok(response) => {
            let status = response.status().as_u16();
            let duration_ms = logging::now_ms() - started;
            // 「 up 」， not response（drop can ， will to or ）
            drop(response);
            let ip = read_exit_ip(&client, proxy, timeout_ms.min(IP_LOOKUP_MAX_MS)).await;
            ConnectivityResult {
                success: true,
                status: Some(status),
                ip,
                duration_ms,
                error: None,
            }
        }
        Err(error) => ConnectivityResult {
            success: false,
            status: None,
            ip: String::new(),
            duration_ms: logging::now_ms() - started,
            error: Some(if error.is_timeout() {
                // Node （seconds to seconds）
                format!("连接超时（{} 秒）", (timeout_ms as f64 / 1000.0).round() as i64)
            } else {
                describe_error_detail(&error)
            }),
        },
    }
}

/// use default when and （ by ）。
///
/// ：set （`core::proxy_settings::test_url`），
/// not configured `DEFAULT_TEST_URL` —— 「」set ，
/// `core::proxy_settings` module header。
pub async fn test_proxy(proxy: Option<&ResolvedProxy>) -> ConnectivityResult {
    let url = crate::server::core::proxy_settings::test_url();
    test_connectivity(
        proxy,
        DEFAULT_TEST_TIMEOUT_MS,
        url.as_deref().or(Some(DEFAULT_TEST_URL)),
    )
    .await
}

/// error can （ for Node `proxyErrorDetail`）。
///
/// Node from `error.cause` 1 down ` → `， as undici 
/// error（ECONNREFUSED / ETIMEDOUT） in cause up 。reqwest error use 
/// `source()` （ `error sending request` → `client error (Connect)` →
/// `tcp connect error` → `Connection refused`）， this use 
/// `std::error::Error::source()` down 。
///
/// is need ：reqwest if source will （ `error sending request`）。
pub fn describe_error_detail(error: &reqwest::Error) -> String {
    let mut parts: Vec<String> = Vec::new();
    // reqwest error （ URL and when ）
    let head = error.to_string();
    if !head.is_empty() {
        parts.push(head);
    }
    let mut current: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(error);
    while let Some(source) = current {
        let text = source.to_string();
        if !text.is_empty() && !parts.iter().any(|part| part == &text) {
            parts.push(text);
        }
        current = source.source();
    }
    if parts.is_empty() {
        return "未知错误".to_string();
    }
    parts.join(" → ")
}

/// （`{label, host, port}`， not use /）——
/// by in response， before in is 。
pub fn describe_public(proxy: Option<&ResolvedProxy>) -> Value {
    match proxy {
        None => Value::Null,
        Some(proxy) => json!({
            "label": proxy.label,
            "host": proxy.host,
            "port": proxy.port_json(),
        }),
    }
}
