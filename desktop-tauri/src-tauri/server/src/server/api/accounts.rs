//! by （ for src/workbuddy-account-routes.mjs ）。
//!
//! GET /api/accounts （current、、enabled、、
//! provider and providers need ）
//! POST /api/accounts （accessToken/refreshToken JSON）
//! GET /api/accounts/export （ token， after use ）
//! POST /api/accounts/export-files or file { id? / provider? }
//! POST /api/accounts/import （merge： by uid ， in update、 in ）
//! POST /api/accounts/current （only）{ id }
//! POST /api/accounts/batch （ accounts_batch.rs）
//!                                       enable / disable / proxy / concurrency / usageQuery / limiters / remove
//! POST /api/accounts/reorder （）{ queue: [id, …] }
//! POST /api/accounts/refresh new （ or current） token { id? }
//! POST /api/accounts/refresh-expiring new ****expired/ （）
//! GET /api/accounts/usage /（ and ，failure not ）
//! `?id=` （** not enabled**， usage_query）
//! GET /api/accounts/usage/snapshot 1 ** when ** （ usage）
//! GET /api/accounts/connections （「」，2 seconds）
//! POST /api/accounts/checkin to （， and ）{ id? }
//! PATCH /api/accounts/{id} { name?, priority?, enabled?, proxy? }
//! POST /api/accounts/{id}/move and { direction: 'up' | 'down' }
//! DELETE /api/accounts/{id} delete
//!
//! ── Agent2API in file as ─────────────────────────────
//! - response ** is **： for 1 `provider`、 1 
//! `providers` need （ by `AccountStore::list_accounts` ，file not use ）；
//! - `POST /api/accounts` by payload provider ****（ `add_account`）；
//! - `usage` from 「 workbuddy」 to ****（）：
//! not by provider ， to `ProviderAdapter::query_usage`。
//! `checkin` to ****（WorkBuddy / / AutoClaw， to 
//! not ， `core::billing::checkin::checkin_for`）； its CRUD not 。
//!
//! （/api/proxies、/api/proxies/test） in `api::proxies`，
//! `proxies_entry` in file —— and ， for 
//! Node `tryHandle` → `tryHandleProxies` 。
//!
//! ── ： and Node 「 1 + by path」────────
//! Node is `tryHandle(req,res,path)`： by pathpath， not in 
//! remaining id。 this ** is can as **，e.g.：
//! DELETE /api/accounts/export → `<id>` → 404「 not in 」
//! GET /api/accounts/xxx/yyy → not in → 404「Not found: GET ...」
//! if use axum by + `{id}`（matchit ）， before will 405 ，
//! and Node 。 this Node ： 1 `/api/accounts/{*rest}`
//! + 1 /path， not need ， as for 。
//!
//! ── usage / checkin ─────────────────────────
//! ① `skipped` in path up ** not **（ 1 is 「available in use 」，
//! 1 is 「availabletotal − can to 」）， `resolve_checkin_targets`；
//! ② `usage` 「 no has available」** not name **（Node that return
//! no ），， `query_usage_for`。

use axum::body::Bytes;
use axum::extract::State;
use axum::http::Method;
use axum::response::Response;
use serde_json::{json, Map, Value};

pub use super::accounts_batch::batch_accounts;

use crate::server::core::account_store::AccountStoreError;
use crate::server::core::billing::checkin;
use crate::server::core::proxies::ProxyConfigError;
use crate::server::core::providers::adapter::adapter_for;
use crate::server::core::providers::zcode;
use crate::server::core::providers::ProviderKind;
use crate::server::errors::management_error;
use crate::server::http::{ok_json, parse_body, query_param};
use crate::server::logging;
use crate::server::ServerState;

/// error → （ + ）。
///
/// can is `pub(super)`：`api::session` Loomy need 
/// （`add_loomy_account`），failure when must is 1 response and 1 ——
/// that 1 will 「failure」 and 「failure」 in up not 1 。
pub(super) fn store_error(error: AccountStoreError) -> Response {
    logging::log("[Accounts]", &format!("❌ {}", error.message));
    management_error(error.status_code, error.message)
}

pub(super) fn proxy_error(error: ProxyConfigError) -> Response {
    logging::log("[Accounts]", &format!("❌ {}", error.message));
    management_error(error.status_code, error.message)
}

/// ： payload `NormalizeInput` to adapter。
/// and use secret/public and ， not in API in 。
/// `account_source` is ， not from payload or secret in 。
async fn add_plugin_account(
    state: &ServerState,
    adapter: &dyn crate::server::core::providers::adapter::ProviderAdapter,
    provider: &str,
    payload: &Value,
    import_name: Option<&str>,
    account_source: &str,
) -> Response {
    use crate::server::core::proxies::{normalize_account_proxy, resolve_account_proxy, ResolvedProxy};
    // not ； null/direct is need need 。
    let proxy_config = match payload.get("proxy") {
        Some(value) if !value.is_null() && !value.is_object() => {
            return management_error(400, "账号代理配置必须是对象或 null，未回退直连");
        }
        Some(value) => match normalize_account_proxy(value) {
            Ok(config) => Some(config.unwrap_or_else(|| json!({ "source": "direct" }))),
            Err(error) => return proxy_error(error),
        },
        None => None,
    };
    // normalize and use 1 1 config；pool/Clash failure not can 。
    let selected_proxy = match proxy_config.as_ref() {
        None => None,
        Some(config) if config.get("source").and_then(Value::as_str) == Some("direct") => None,
        Some(config) => {
            if crate::server::core::proxies::is_rotate_config(config) {
                return management_error(400, "添加标准账号不支持轮询代理，请选择直连或一条代理");
            }
            let Some(resolution) = resolve_account_proxy(Some(config)) else {
                return management_error(400, "账号代理未能解析，未回退直连");
            };
            if resolution.error().is_some() {
                return management_error(400, "账号代理无法解析，请检查所选代理是否存在且已启用；未回退直连");
            }
            match ResolvedProxy::from_json(&resolution.to_json()) {
                Ok(Some(proxy)) => Some(proxy),
                _ => return management_error(400, "账号代理出口无效，请检查主机与端口；未回退直连"),
            }
        }
    };
    let proxy = selected_proxy.map(|proxy| agent2api_provider_protocol::WireProxy {
        protocol: proxy.protocol,
        host: proxy.host,
        port: proxy.port,
        username: proxy.username,
        password: proxy.password,
    });

    let source = if payload.get("importDesktop").and_then(Value::as_bool).unwrap_or(false) {
        agent2api_provider_protocol::NormalizeSource::Desktop
    } else if let Some(content) = payload
        .get("authFileContent")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        agent2api_provider_protocol::NormalizeSource::FileText {
            content: content.to_string(),
            file_name: payload
                .get("authFileName")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    } else if let Some(content) = payload
        .get("authFileB64")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        agent2api_provider_protocol::NormalizeSource::FileBase64 {
            content: content.to_string(),
            file_name: payload
                .get("authFileName")
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    } else {
        // new runtime 1 /parameter， not config
        // RAW 。 old to payload， 1 。
        let mut manual_payload = payload.clone();
        if crate::server::core::providers::is_scoped_runtime_provider(provider) {
            if let Some(fields) = manual_payload.as_object_mut() {
                if fields.get("methodIndex").is_some() {
                    for key in ["provider", "name", "proxy"] { fields.remove(key); }
                } else if account_source == "builtin" {
                    for key in ["name", "proxy"] { fields.remove(key); }
                }
            }
        }
        agent2api_provider_protocol::NormalizeSource::Manual { payload: manual_payload }
    };

    let input = agent2api_provider_protocol::NormalizeInput {
        provider: provider.to_string(),
        source,
        name: import_name.map(str::to_string),
        proxy,
    };
    let account = match adapter.normalize_account(&state.store, input).await {
        Ok(account) => account,
        Err(error) => {
            logging::log("[Accounts]", &format!("❌ {}", error.message));
            return management_error(error.status_code, error.message);
        }
    };

    let id = account.identity.clone();
    let name = account
        .public
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .or(import_name)
        .unwrap_or(adapter.label());
    match state.store().plugin_upsert_account_with_source_and_proxy(
        &id,
        provider,
        name,
        &account.secret,
        &account.public,
        account.desktop,
        account_source,
        proxy_config.as_ref(),
    ) {
        Ok((record, proxy_saved)) => ok_json(json!({
            "account": record,
            "created": true,
            "proxySaved": proxy_saved,
            "list": state.store().list_visible_accounts(),
        })),
        Err(message) => {
            logging::log("[Accounts]", &format!("❌ {message}"));
            management_error(500, &message)
        }
    }
}

/// request：empty body as `{}`； JSON by 400。
///
/// Node not not 1 （「 up not is valid JSON」/「request not is valid JSON」），
/// by use ， not 1 1 。
pub(super) fn parse_json_body(body: &Bytes, fallback_message: &str) -> Result<Value, Response> {
    match parse_body(body) {
        Ok(value) => Ok(value),
        Err(_) => Err(management_error(400, fallback_message)),
    }
}

/// `authFileContent` / `authFileB64`（file） in ****：
/// by `add_plugin_account` `NormalizeSource::{FileText,
/// FileBase64}`，file（「 this 1 file is is 」） is 
/// 。 not by 「 file not 」 —— that will and 
/// supports ， use to errormust that 1 。
///
/// （ for Node `decodeURIComponent(path.slice(...))`）。
///
/// axum already in parameter when 1 ； id `user-<uid>`、 not ，
/// this 1 is etc —— is as Node is this ，
/// URL `%2F` this when as not will 。
fn decode_segment(value: &str) -> String {
    let mut out = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
            if let Some(byte) = hex.and_then(|text| u8::from_str_radix(text, 16).ok()) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| value.to_string())
}

// ─── axum ──────────────────────────────────────────────

/// `/api/accounts`（none） and `/api/accounts/{*rest}` use 。
///
/// use `any(...)` （any）， as Node has 「PATCH/DELETE to 
/// `<id>` 」 this path —— to axum by will 。
pub async fn accounts_entry(State(state): State<ServerState>, request: axum::extract::Request) -> Response {
    let method = request.method().clone();
    let full_path = request.uri().path().to_string();
    // （ ， by use to `query_param` ）。
    // before has `GET /api/accounts/usage?id=<id>` use —— that 1 is 「 use 1 
    // by 」， and `core::usage_query::query_all` 。
    let query = request.uri().query().unwrap_or("").to_string();
    let body = match axum::body::to_bytes(request.into_body(), crate::server::http::MAX_BODY_SIZE)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => return management_error(413, format!("请求体读取失败或过大: {error}")),
    };
    // `suffix` as empty = in `/api/accounts`（/add）； its （ `/api/accounts/`
    // this has ） is path —— Node use `path === '/api/accounts'` 
    // etc ， `/api/accounts/` to after 404 ， this must this 
    let suffix = full_path.strip_prefix("/api/accounts").unwrap_or("");
    let rest = suffix.strip_prefix('/').map(str::to_string);
    dispatch(state, method, rest.as_deref(), &full_path, &query, &body).await
}

/// `/api/proxies*` （any）。
///
/// already path：`/api/proxies`、`/api/proxies/test`（Node has can ） and 
/// `/api/proxies/pool`、`/pool/update`、`/pool/enable`、`/pool/remove`、
/// `/pool/test`、`/pool/sync-clash`（「」 CRUD + + 
/// + Clash ， `api::proxies` module header）。 that id body， not 
/// path —— is 1 (, remainingpath) ， and 。
pub async fn proxies_entry(State(state): State<ServerState>, request: axum::extract::Request) -> Response {
    let method = request.method().clone();
    let full_path = request.uri().path().to_string();
    let rest = full_path
        .strip_prefix("/api/proxies")
        .unwrap_or("")
        .trim_start_matches('/')
        .to_string();
    let body = match axum::body::to_bytes(request.into_body(), crate::server::http::MAX_BODY_SIZE)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => return management_error(413, format!("请求体读取失败或过大: {error}")),
    };
    match (method.as_str(), rest.as_str()) {
        ("GET", "") => super::proxies::list_proxies(&state).await,
        ("POST", "test") => super::proxies::test_proxy(&state, &body).await,
        ("GET", "pool") => super::proxies::pool_list(&state).await,
        ("POST", "pool") => super::proxies::pool_create(&state, &body).await,
        ("POST", "pool/update") => super::proxies::pool_update(&state, &body).await,
        ("POST", "pool/enable") => super::proxies::pool_enable(&state, &body).await,
        ("POST", "pool/remove") => super::proxies::pool_remove(&state, &body).await,
        ("POST", "pool/test") => super::proxies::pool_test(&state, &body).await,
        ("POST", "pool/sync-clash") => super::proxies::pool_sync_clash(&state).await,
        ("GET", "settings") => super::proxies::settings_get().await,
        ("POST", "settings") => super::proxies::settings_save(&body).await,
        // already path up its ：Node tryHandleProxies to 404 
        _ => management_error(
            404,
            format!("Not found: {} {full_path}", method.as_str()),
        ),
    }
}

// ─── path（ and Node tryHandle for ）──────

/// by path。
///
/// `rest` as `None` in `/api/accounts`；`Some(...)` is 1 before after 
/// remaining（`/api/accounts/` to is `Some("")` —— Node etc ，path）。
/// `full_path` is path（404 need ），`query` is 
/// （ has `usage` that 1 use `id`， `accounts_usage::accounts_usage`）。
/// ** Node **：
///
/// ① path（none 1 ）：GET/POST ``、GET export、POST import、
///      POST current、POST batch、POST refresh、GET usage、POST checkin
/// ② POST + `/move` → ； `/rate-limits/clear` → 
/// ③ PATCH / DELETE + has remaining → id（** not **，
/// `DELETE /api/accounts/export` is 「 1 export 」）
/// ④ not in → 404「Not found: <METHOD> <path>」（ API ）
///
/// ③ ：Node PATCH `startsWith('/api/accounts/')`，
/// path in GET/POST up will id， to 404「 not in 」。
pub async fn dispatch(
    state: ServerState,
    method: Method,
    rest: Option<&str>,
    full_path: &str,
    query: &str,
    body: &Bytes,
) -> Response {
    // ① in `/api/accounts`
    if rest.is_none() {
        match method.as_str() {
            "GET" => return ok_json(state.store().list_visible_accounts()),
            "POST" => return add_account(&state, body).await,
            // Node for `/api/accounts` up its 404 
            _ => {
                return management_error(404, format!("Not found: {} {full_path}", method.as_str()))
            }
        }
    }
    let rest = rest.unwrap_or("");

    // ② path（Node in `<id>` before Reason）
    match (method.as_str(), rest) {
        ("GET", "sorting") => return super::accounts_sort::get_sorting(&state),
        ("PUT", "sorting") => return super::accounts_sort::save_sorting(&state, body),
        ("GET", "export") => return export_accounts(&state),
        ("POST", "export-files") => return super::accounts_export::account_files(&state, body).await,
        ("POST", "import") => return import_accounts(&state, body).await,
        ("POST", "current") => return set_current(&state, body).await,
        ("POST", "batch") => return batch_accounts(&state, body).await,
        // （： / after new up ）
        ("POST", "reorder") => return reorder_accounts(&state, body).await,
        ("POST", "refresh") => return refresh_account(&state, body).await,
        ("POST", "refresh-expiring") => return refresh_expiring_accounts(&state).await,
        // / （） in `api::accounts_usage`（ that module header）。
        // `?id=` is 「 1 by 」 ， accounts_usage 。
        ("GET", "usage") => {
            return super::accounts_usage::accounts_usage(&state, query).await
        }
        // when that 1 （ usage， 1 `at`）
        ("GET", "usage/snapshot") => {
            return super::accounts_usage::accounts_usage_snapshot(&state).await
        }
        // （「」； `core::upstream::connections`）
        ("GET", "connections") => return account_connections(&state),
        ("POST", "checkin") => return accounts_checkin(&state, body).await,
        _ => {}
    }

    // ③ POST + /move ；POST + /rate-limits/clear 
    if method == Method::POST {
        if let Some(id) = rest.strip_suffix("/move") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return move_account(&state, &id, body).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/rate-limits/clear") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return clear_rate_limits(&state, &id, body);
            }
        }
        // ZCode「 when 」：（、 not need ） and （ need ）。
        // after not （`/zcode-claim/preview` not `/zcode-claim` ），
        // this after not in —— when by 「 after 」，
        // and up use 1 。
        if let Some(id) = rest.strip_suffix("/zcode-claim/captcha-config") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::zcode_claim::captcha_config(&state, &id).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/zcode-claim/preview") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::zcode_claim::preview(&state, &id).await;
            }
        }
        // CodeArts daily：（）（）。
        // up preview must in before —— after not ， when by use 
        // ， and up ZCode that 1 。
        if let Some(id) = rest.strip_suffix("/codearts-welfare/preview") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::codearts_welfare::preview(&state, &id).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/codearts-welfare") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::codearts_welfare::claim(&state, &id, body).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/zcode-claim") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::zcode_claim::claim_plan(&state, &id, body).await;
            }
        }
        // new 1 （ by ：Loomy / 
        // ， `api::onboarding`； etc ， not will ）。
        if let Some(id) = rest.strip_suffix("/onboarding/claim") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::onboarding::claim(&state, &id).await;
            }
        }
        // WorkBuddy （）： / / ，
        // `api::growth`。 after not （ /growth/xxx ）， after not 
        // in ； when by up by （ → → ）。
        // Reason（60 seconds when ） api::growth module header。
        if let Some(id) = rest.strip_suffix("/growth/task") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::growth::run_task(&state, &id, body).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/growth/school") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::growth::school_run(&state, &id, body).await;
            }
        }
        if let Some(id) = rest.strip_suffix("/growth/travel") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::growth::travel_run(&state, &id).await;
            }
        }
    }

    // ③' GET + /onboarding → new （， to after ）。
    // and up POST 1 ： after not ， after not in 。
    // `?refresh=1` up （「」）； not then （ 1 
    // not up ， `api::onboarding` ）。
    if method == Method::GET {
        if let Some(id) = rest.strip_suffix("/onboarding") {
            let id = decode_segment(id);
            if !id.is_empty() {
                let refresh = query_param(query, "refresh").as_deref() == Some("1");
                return super::onboarding::status(&state, &id, refresh).await;
            }
        }
        // WorkBuddy （， `api::growth` module header）。
        // and up onboarding 1 ： GET、 POST ——
        // ， not to 「 1 」 will 。
        if let Some(id) = rest.strip_suffix("/growth") {
            let id = decode_segment(id);
            if !id.is_empty() {
                return super::growth::status(&state, &id).await;
            }
        }
    }

    // ④ PATCH / DELETE → remaining id。
    // id as empty（ `/api/accounts/`） to handler：return 400「 id」，
    // this and Node PATCH `if (!id) throw` 1 。
    // Note and `/api/accounts//` ： that is "/"（empty）→ 404「 not in 」。
    match method.as_str() {
        "PATCH" => return patch_account(&state, &decode_segment(rest), body).await,
        "DELETE" => return delete_account(&state, &decode_segment(rest)),
        _ => {}
    }

    // ⑤ 404（Node `tryHandle` that ）
    management_error(404, format!("Not found: {} {full_path}", method.as_str()))
}

// ─── POST /api/accounts ─────────────────────────────────────

/// ：** by payload provider to path**（W3-T4 ，
/// W4a as ）。
///
/// ── then （W4a： not 「workbuddy/raccoon」）──────────
/// provider `providers::kind_from_id` （** is **，
/// `providers::is_known_provider_id` ）， by kind ：
/// ① `Raccoon` → path：`importDesktop === true` when 
/// （ignore token ）；otherwise（token / refreshToken， or 
/// auth.json ）；
/// ② `CatPaw` → already to ： up when up `serves_provider` 
/// （ by `account.normalize` ）， no when 
/// 503；
/// ③ `AutoClaw` → AutoClaw path（W4b-T-c2）：`importDesktop === true` 
/// when （`%APPDATA%/AutoClaw/auth.json`，DPAPI ）；
/// otherwise（token/refreshToken + deviceId，`enc:` ）；
/// ④ `WorkBuddy` / `WorkBuddyIntl` or **** → has workbuddy path
/// （`store.add_account`， by this provider id to ： and 
/// use 1 and path， in 1 ）。
///
/// ── file（ / Cline / KukuAI， and ）─────────────────
/// payload add optional（，request not ）：
/// - `authFileContent`： use file****（KukuAI not ——
/// is SQLite， to 400 `authFileB64`）；
/// - `authFileB64`：file base64（ before is JSON， and 400；
/// KukuAI Cookies SQLite is ， this is ）；
/// - `authFileName`：file，onlyused forerror and 。
/// `authFileContent` / `authFileB64` 1 in → 「file」path，****
/// `importDesktop`（ ① ② ）； no has when as and 1 。
/// （path use ）：file＝ file ** after **， to 
/// 1 ** can ** （ and 「」 1 ，file can any or 
/// ）；`importDesktop` ＝ 1 「 when when path」 。
///
/// ── AutoClaw in this 400（，）─────────────────
/// W4a–W4b no ， that when this for payload 400： if 
/// ④ ， use will to 1 **200 + workbuddy ** （ is 、
/// is 、failure）， up not 。W4b-T-c2 has 
/// （③）， this 400 ； not —— ** not no **。 use 
/// match：add provider when will to ； no has id（ before after 
/// new 、 or request） ④ —— client not provider ，unknown id not can 。
pub async fn add_account(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(&body, "上传内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let provider = payload
        .get("provider")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let store = state.store();
    let name = payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string);
    let import_name = name.as_deref();
    if let Some(response) = super::accounts_export::import_account_file(state, provider, &payload, import_name) {
        return response;
    }
    if crate::server::core::providers::availability::is_suspended(provider) {
        return management_error(503, "该提供商已暂停接入，暂不支持添加账号；已有数据保留");
    }
    // ── 「file」（ as optional；file）─────────────
    // use in up ****client file when ， before file this ：
    // - `authFileContent`：file（JSON file， before already file）；
    // - `authFileB64`：file base64（file：KukuAI Cookies SQLite ；
    // is JSON and 400）；
    // - `authFileName`：file，onlyused forerror and （ not and ）。
    // 1 in → filepath，**** `importDesktop`： before file
    // ** after **（ to 1 can ，file can any or 
    // ）， after 1 「 when path」 ； when to when file as 
    // （ use ）。 no has when as not （）。
    // ** not **： by `add_plugin_account` 
    // `NormalizeSource::{FileText,FileBase64}`（ by ），
    // Cline / KukuAI etc 。
    // ── （`custom-` before ， not in ）─────────
    // ** not ** `ProviderKind` —— `kind_from_id` for return None is 
    // not is （ `providers` module header），must in match ** before **
    // ： `None` will workbuddy will workbuddy 
    // （ is apiKey、 is ， is W4a need 「」）。
    // is `is_custom_provider_id`（ before + in ）： before will 
    // id down ， no has and ， is 。
    if crate::server::core::custom_providers::is_custom_provider_id(provider) {
        let result = store.add_custom_account(provider, &payload, import_name);
        return match result {
            Ok(upsert) => ok_json(json!({
                "account": upsert.account,
                // etc and （ key / none need ） when as false： before 
                // 「 already update」 not is 「 already 」， CustomAccountUpsert 
                "created": upsert.created,
                "list": store.list_visible_accounts(),
            })),
            Err(error) => store_error(error),
        };
    }
    // and runtime ID use normalize / ，must old path。
    // 1 ID by ；source only， not provider 。
    if crate::server::core::providers::is_executable_provider(provider) {
        let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider) else {
            return management_error(503, format!("提供商执行体尚未就绪：{provider}"));
        };
        let source = if crate::server::core::plugin_host::serves_provider(provider) {
            "plugin"
        } else {
            "builtin"
        };
        return add_plugin_account(state, adapter.as_ref(), provider, &payload, import_name, source).await;
    }
    // ── ：** return**（ no has path）──────
    // not 1 ， this arm is 503 。
    // match not is 1 use 503：add `ProviderKind` when 
    // will to ，「 this to is not is 」 has ——
    // 1 use will 「 1 」 error。
    match crate::server::core::providers::kind_from_id(provider) {
        // Raccoon（）： already to （JWT / when 
        // / file / / directory / / to / new ）， this 
        // path —— up when up `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::Raccoon) => {
            return management_error(
                503,
                "小浣熊的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与桌面端读取实现",
            );
        }
        // CatPaw： already to （ 1 、 when 、、
        // will ）。 up when up `serves_provider` will ，
        // （file / / ） by `account.normalize` 。
        Some(crate::server::core::providers::ProviderKind::CatPaw) => {
            return management_error(
                503,
                "CatPaw 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与桌面端读取实现",
            );
        }
        // AutoClaw（ ）： already to （ / safeStorage /
        // when / / / and OAuth ）， this path
        // —— up when up `serves_provider` will 。
        Some(
            crate::server::core::providers::ProviderKind::AutoClaw
            | crate::server::core::providers::ProviderKind::AutoClawIntl,
        ) => {
            return management_error(
                503,
                "AutoClaw 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与登录实现",
            );
        }
        // CodeArts： already to （AK/SK/STS 、DPoP + PKCE 、
        // directory、、daily）， this path —— up when up 
        // `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::CodeArts) => {
            return management_error(
                503,
                "CodeArts 的账号需要安装提供商插件：本版本不再内置该家的凭据解析与登录实现",
            );
        }
        // Qoder（ ）： already to （、PAT 、
        // 、directory、 will 、、 to 、）， this path
        // —— up when up `serves_provider` will 。
        Some(
            crate::server::core::providers::ProviderKind::Qoder
            | crate::server::core::providers::ProviderKind::QoderIntl,
        ) => {
            return management_error(
                503,
                "Qoder 的账号管理需要安装提供商插件：本版本不再内置该家的设备授权与凭据解析实现",
            );
        }
        // Accio（ ）： already to （ 1 、、
        // and ）， this path —— up when up `serves_provider`
        // will 。
        Some(
            crate::server::core::providers::ProviderKind::Accio
            | crate::server::core::providers::ProviderKind::AccioCn,
        ) => {
            return management_error(
                503,
                "Accio 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与登录实现",
            );
        }
        // Cline： already to （、、、directory、
        // 、、）， this path —— up when up 
        // `serves_provider` will 。
        Some(
            crate::server::core::providers::ProviderKind::ClineFree
            | crate::server::core::providers::ProviderKind::ClinePass,
        ) => {
            return management_error(
                503,
                "Cline 的账号管理需要安装提供商插件：本版本不再内置该家的设备授权登录与凭证解析实现",
            );
        }
        // ZCode（ ）： already to （CLI 、
        // 、、、、、directory、）， this 
        // path —— up when up `serves_provider` will 。
        Some(
            crate::server::core::providers::ProviderKind::Zcode
            | crate::server::core::providers::ProviderKind::ZcodeIntl,
        ) => {
            return management_error(
                503,
                "ZCode 的账号管理需要安装提供商插件：本版本不再内置该家的 CLI 登录与凭证换取实现",
            );
        }
        // Trae： already to （、、、directory、、
        // 、 to ）， this path —— up when up `serves_provider`
        // will 。
        Some(crate::server::core::providers::ProviderKind::Trae) => {
            return management_error(
                503,
                "Trae 的账号管理需要安装提供商插件：本版本不再内置该家的网页登录与凭据解析实现",
            );
        }
        // Loomy（）： already to （ session、、、
        // 、、 to 、 new ）， this path —— up when up 
        // `serves_provider` will 。
        //
        // this 1 `add_loomy_account` path， after is 
        // 「 no can 1 Loomy ， and to 503」——
        // can 、 can none， use not Why。 and its 1 1 503。
        Some(crate::server::core::providers::ProviderKind::Loomy) => {
            return management_error(
                503,
                "Loomy 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与短信登录实现",
            );
        }
        // KukuAI（ AI）： already to （——
        // Cookie / client Cookies / file—— and ），
        // this path。 up when up `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::Kuku) => {
            return management_error(
                503,
                "KukuAI 的账号管理需要安装提供商插件：本版本不再内置该家的 Cookie 解析与校验实现",
            );
        }
        // MonkeyCode（）： already to （session 、image_id
        // 、directory、 will ）， this path。 up when up 
        // `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::MonkeyCode)
        | Some(crate::server::core::providers::ProviderKind::MonkeyCodeIntl) => {
            return management_error(
                503,
                "MonkeyCode 的账号管理需要安装提供商插件：本版本不再内置该家的 session 校验实现",
            );
        }
        // Command Code： already to （API Key 1 / check / /
        // directory / NDJSON ）。 up when up `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::CommandCode) => {
            return management_error(
                503,
                "Command Code 的账号管理需要安装提供商插件：本版本不再内置该家的 API Key 校验与转发实现",
            );
        }
        // Antigravity（Google AI IDE）： already to （ +
        // refresh token 1 + + + ）， this path。
        Some(crate::server::core::providers::ProviderKind::Antigravity) => {
            return management_error(
                503,
                "Antigravity 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与探活实现",
            );
        }
        // OfficeAce（ as / OfficeClaw）： already to （
        // OAuth and ）， this path —— up when up 
        // `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::OfficeAce) => {
            return management_error(
                503,
                "OfficeAce 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与登录实现",
            );
        }
        // OrcaTerm（client）： already to ， this not 
        // —— up when up `serves_provider` will ，
        // no when to this ， not is 1 no has path。
        Some(crate::server::core::providers::ProviderKind::OrcaTerm) => {
            return management_error(
                503,
                "OrcaTerm 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析与探活实现",
            );
        }
        // （`chatglm.cn`）： already to （ / /
        // path、、directory、、、 to 、 in ），
        // this path —— up when up `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::Chatglm) => {
            return management_error(
                503,
                "智谱清言的账号管理需要安装提供商插件：本版本不再内置该家的访客换证、令牌校验与本机登录态读取实现",
            );
        }
        // ModelScope（） ： already to （`ms-` 
        // 、、directory、、 to ）， this path —— up when up 
        // `serves_provider` will 。
        Some(
            crate::server::core::providers::ProviderKind::Modelscope
            | crate::server::core::providers::ProviderKind::ModelscopeIntl,
        ) => {
            return management_error(
                503,
                "ModelScope 的账号管理需要安装提供商插件：本版本不再内置该家的令牌探活与凭证解析实现",
            );
        }
        // AtomCode： already to （ for 1 、directory、、、、
        // ）。 up when up `serves_provider` will 。
        Some(crate::server::core::providers::ProviderKind::Atomcode) => {
            return management_error(
                503,
                "AtomCode 的账号管理需要安装提供商插件：本版本不再内置该家的凭证解析实现",
            );
        }
        // LobsterAI：** no has path**（， not is 1 use not ）。
        // has `keyfrom` and 1 `uuid`， in that 1 by up down ——
        // use from not this ， not 。 has 1 ：
        // to loopback after by ****。
        // up when up `serves_provider` will （ 
        // `normalize_account` for to 1 ）。
        Some(crate::server::core::providers::ProviderKind::LobsterAi) => {
            return management_error(
                400,
                "LobsterAI 只能通过「网页登录」添加：它的续期需要换码时下发的 uuid 与 keyfrom，粘贴给不出这三项",
            );
        }
        // WorkBuddy （ / ）： already to 
        // （ will / / / ）， this path。
        //
        // `None`（client not provider ） by ： that is ** old 
        // ** 。 in no has can path —— by 1 etc 
        // use 1 no has ， not failure，
        // 1 503 and 。
        Some(crate::server::core::providers::ProviderKind::WorkBuddy)
        | Some(crate::server::core::providers::ProviderKind::WorkBuddyIntl)
        | None => {
            management_error(
                503,
                "WorkBuddy 的账号管理需要安装提供商插件：本版本不再内置该家的会话凭证解析与桌面端登录态读取实现",
            )
        }
    }
}

// ─── GET /api/accounts/export ───────────────────────────────

pub fn export_accounts(state: &ServerState) -> Response {
    let data = crate::server::core::account_transfer::export_accounts(state.store());
    let count = data
        .get("accounts")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    logging::log("[Accounts]", &format!("📤 账号已导出: {count} 个"));
    ok_json(data)
}

// ─── POST /api/accounts/import ──────────────────────────────

pub async fn import_accounts(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    // API `{ success, data }` ： before 「」 
    // file， no has accounts。 this file when in this down 1 ——
    // already old file not ；file（ is accounts） not 。
    let payload = match envelope_data(&payload) {
        Some(data) => data,
        None => payload,
    };
    match crate::server::core::account_transfer::import_accounts(state.store(), &payload) {
        Ok(result) => ok_json(result),
        Err(error) => store_error(error),
    }
}

/// if `payload` is 「」（ for 、none accounts、`data` is accounts
/// for ） then return data ，otherwise None。
fn envelope_data(payload: &Value) -> Option<Value> {
    let object = payload.as_object()?;
    if object.contains_key("accounts") {
        return None;
    }
    let data = object.get("data")?;
    data.get("accounts").is_some().then(|| data.clone())
}

// ─── POST /api/accounts/current ─────────────────────────────

pub async fn set_current(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let id = payload.get("id").and_then(Value::as_str).unwrap_or("");
    if id.is_empty() {
        return management_error(400, "缺少账号 id");
    }
    match state.store().promote_to_front(id) {
        Ok(result) => {
            if result.get("changed").and_then(Value::as_bool) == Some(false) {
                logging::log("[Accounts]", &format!("已在全局队列第一位，无需置顶: {id}"));
            }
            ok_json(result)
        }
        Err(error) => store_error(error),
    }
}

// ─── POST /api/accounts/refresh ─────────────────────────────

/// new token（** by provider **）。
///
/// workbuddy has `AuthService::refresh_account`；、AutoClaw、
/// CodeArts、ZCode、Cline `refresh_access_token`；
/// **CatPaw / MonkeyCode / Command Code no has can new **（§9.1：
/// `X-Passport-Token` can in new ， no has refreshToken； after is 
/// session / API Key， up no has ）， this 400 and 
/// —— workbuddy new will 。
///
/// already to （Trae / OfficeAce / AtomCode / LobsterAI …） by up that 
/// 「」； no when in ， will to 
/// （503） not is workbuddy 。
///
/// （ to ）：CodeArts when use 「 new Token」
/// to `client key [] not found`；Trae when to 
/// `12153:refresh token failed:10000:token format error`（2026-10-02 15:47:57 ）——
/// " this "， its is 。** 1 refreshToken
/// when ， this must 1 ** —— in `PROVIDERS` and adapter ，
/// not will ， in use by when 。
///
/// Why not 1 to 1 ： new none，「 new 」 is ****、
/// not is up provider （ that `refresh_access_token` is when 
/// ）。 has this 1 。
pub async fn refresh_account(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|value| !value.is_empty());
    let id = match id {
        Some(value) => value,
        // default new ， not WorkBuddy will 。
        None => match state.store().current_account_id() {
            Some(id) => id,
            None => return management_error(400, "当前没有可用账号"),
        },
    };
    logging::verbose("[Accounts]", &format!("刷新账号 token: {id}"));
    // ： is use apiKey（`custom_accounts`）， no has 
    // can new 。 not will to down workbuddy new ——
    // `get_credentials_by_id` for has `apiKey` return None， use will to 
    // 1 「 not in 」， and can that 。
    if state.store().custom_account_provider(&id).is_some() {
        return management_error(
            400,
            "自定义提供商账号的凭证由用户直接提供，无需刷新（更换凭证请重新添加或直接编辑）",
        );
    }
    // by provider adapter， not secret or kind。
    if let Some(provider) = state.store().account_provider_of(&id) {
        if crate::server::core::providers::is_executable_provider(&provider) {
            let Some(adapter) =
                crate::server::core::providers::adapter::adapter_by_id(&provider)
            else {
                return management_error(400, format!("未知的提供商：{provider}"));
            };
            return match crate::server::core::credential_maintenance::refresh_checked(
                state.store(),
                adapter.as_ref(),
                &id,
            )
            .await
            {
                Ok(_) => ok_json(json!({
                    "refreshedId": id,
                    "list": state.store().list_visible_accounts(),
                })),
                Err(error) => {
                    logging::log("[Accounts]", &format!("❌ {}", error.message));
                    management_error(i32::from(error.http_status().as_u16()), error.message)
                }
            };
        }
    }
    // Qoder（ 1 —— by provider ， 1 id not can can 
    // when ， and Accio / AutoClaw 1 ）
    for region in crate::server::core::providers::qoder::endpoints::Region::ALL {
        if state
            .store()
            .qoder_account_record(region, &id)
            .is_some()
        {
            return refresh_provider_account(state, &id, region.kind()).await;
        }
    }
    // CodeArts： 1 refresh token + ，must（ already to ）。
    // ** this not can **： will to down ， use to 1 and none
    // error（ to to is 
    // `auth/token/refresh failure: client key [] not found`）。
    if state.store().account_provider_of(&id).as_deref()
        == Some(crate::server::core::providers::kind_id(ProviderKind::CodeArts))
    {
        return refresh_provider_account(state, &id, ProviderKind::CodeArts).await;
    }
    // Trae already to ： up when up `serves_provider` will 
    // ； no when to down workbuddy will to and none error，
    // this by 「」。 is provider ， not need need ——
    // not this （ that to ）。
    if state.store().account_provider_of(&id).as_deref()
        == Some(crate::server::core::providers::kind_id(ProviderKind::Trae))
    {
        return management_error(
            503,
            "Trae 的令牌刷新需要安装提供商插件：本版本不再内置该家的续期实现",
        );
    }
    // ZCode： already （ no has —— use can new 
    // ）。 this by 「」： no when to down 
    // workbuddy will to and none error。
    if let Some(provider_id) = state.store().account_provider_of(&id) {
        if crate::server::core::providers::zcode::region::Region::from_provider_id(&provider_id)
            .is_some()
        {
            return management_error(
                503,
                "ZCode 的令牌刷新需要安装提供商插件：本版本不再内置该家的登录链实现",
            );
        }
    }
    // OfficeAce already to ： up when up `serves_provider`
    // will ； no when `refresh_provider_account` to is ，
    // will 「 need need 」。
    if state.store().officeace_account_record(&id).is_some() {
        return refresh_provider_account(state, &id, ProviderKind::OfficeAce).await;
    }
    // AtomCode： （`POST {broker}/oauth/refresh`，** will **
    // refresh token， and must + ）。** this not can **： to workbuddy
    // ， use 「 new Token」 to is that （ and CodeArts/Trae/
    // OfficeAce 1 by ）， AtomGit will to can new 。
    if state.store().atomcode_account_record(&id).is_some() {
        return refresh_provider_account(state, &id, ProviderKind::Atomcode).await;
    }
    // LobsterAI： （`POST /api/auth/refresh`，** will **，
    // and must uuid and keyfrom ）。** this not can **： to 
    // workbuddy ， use 「 new Token」 to is that （ and 
    // CodeArts/Trae/OfficeAce 1 by ）， will 
    // to can new —— that 1 is use 。
    // already to ： no when `refresh_provider_account` to is ，
    // will 「 need need 」。
    if state.store().lobsterai_account_record(&id).is_some() {
        return refresh_provider_account(state, &id, ProviderKind::LobsterAi).await;
    }
    // Accio（ ） already to ： up when up 
    // `serves_provider` will ； no when to is ， will 
    // 「 need need 」。 1 —— by provider 。
    for region in crate::server::core::providers::accio::endpoints::Region::ALL {
        if state.store().account_provider_of(&id).as_deref() == Some(region.provider_id()) {
            return refresh_provider_account(state, &id, region.kind()).await;
        }
    }
    // Cline ： and AutoClaw 1 —— ** when not new **。
    // and Cline client use 1 providers.json refreshToken，
    // new will client that will （ in ， after ）。
    // （ ） new 。
    //
    // 「 1 」 by （ use this ）， is 
    // `cline_account_provider` not is provider id —— is up 
    // ， new for none。
    if let Some(provider_id) = state.store().cline_account_provider(&id) {
        if state.store().cline_is_desktop_account(&id) {
            return management_error(
                400,
                "Cline 桌面端登录态由 Cline 客户端维护，网关不主动刷新：\
                 请在 Cline 客户端重新登录后重新导入，或改用「填写凭证」添加",
            );
        }
        let kind = match crate::server::core::providers::kind_from_id(&provider_id) {
            Some(kind) => kind,
            None => return management_error(400, format!("未知的提供商：{provider_id}")),
        };
        return refresh_provider_account(state, &id, kind).await;
    }
    // CatPaw ： no has new （ ）。 after this 
    // valid —— `refresh` can is false， will to 1 ，
    // this down can 1 RPC， and 。
    if state.store().catpaw_account_record(&id).is_some() {
        return management_error(
            400,
            "CatPaw 登录态没有刷新机制：请在 CatPaw 桌面端重新登录，\
             然后在本页重新导入桌面端登录态（或重新粘贴新的登录凭证）",
        );
    }
    // MonkeyCode ：** no has new **（session 30 limit，
    // up none refresh ， `providers::monkeycode::credentials` module header）。
    // not down will to down workbuddy ， use will to 1 and 
    // error —— and CatPaw when 1 。
    for region in crate::server::core::providers::monkeycode::Region::ALL {
        if state
            .store()
            .monkeycode_account_record(region, &id)
            .is_some()
        {
            return management_error(
                400,
                "MonkeyCode 登录态没有刷新机制（会话 30 天、上游无续期接口）：\
                 请在浏览器重新取一次 session cookie 后在本页重新粘贴",
            );
        }
    }
    // Command Code 「 no has new 」 already to ： 
    // `refresh` will for this to 1 ， not by provider 。
    // Antigravity：** has ** can new refreshToken（Google OAuth）， 
    // new （`grant_type=refresh_token` `oauth2.googleapis.com/token`）。
    // this not can ： to down workbuddy ， use 「 new Token」
    // to is —— and Command Code / Trae when 1 
    // （`refresh_account` this as 「 refreshToken when must 1 」）。
    if state.store().antigravity_account_record(&id).is_some() {
        return refresh_provider_account(state, &id, ProviderKind::Antigravity).await;
    }
    // ── ： by provider ─────────────────────
    //
    // to this up that no in （ by 
    // ）。 after this 1 ** not is WorkBuddy**： up provider，
    // can to （ in up `serves_provider` that already ； this 
    // is 「 no / or no 」 ， will 
    // 「 need need 」）。 not to unknown， not is WorkBuddy 
    // 。
    let provider = match state.store().account_provider_of(&id) {
        Some(provider) => provider,
        None => return management_error(404, "账号不存在"),
    };
    let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(&provider) else {
        return management_error(
            400,
            format!(
                "{} 的令牌刷新需要安装提供商插件：本版本不再内置该家的续期实现",
                crate::server::core::providers::label_of(&provider)
            ),
        );
    };
    match crate::server::core::credential_maintenance::refresh_checked(
        state.store(),
        adapter.as_ref(),
        &id,
    )
    .await
    {
        Ok(_) => ok_json(json!({
            "refreshedId": id,
            "list": state.store().list_visible_accounts(),
        })),
        Err(error) => {
            logging::log("[Accounts]", &format!("❌ {}", error.message));
            management_error(i32::from(error.http_status().as_u16()), error.message)
        }
    }
}

/// 「 → new → 」 this new （ / AutoClaw use ）。
///
/// new not （ `credentials` / `refresh` ），**
/// 1 **： this 、 `refresh_access_token`（force ， not 
/// ）、 by 。 this is 1 use ， in 。
async fn refresh_provider_account(state: &ServerState, id: &str, kind: ProviderKind) -> Response {
    let adapter = adapter_for(kind);
    match crate::server::core::credential_maintenance::refresh_checked(state.store(), adapter, id)
        .await
    {
        Ok(_) => ok_json(json!({
            "refreshedId": id,
            "list": state.store().list_visible_accounts(),
        })),
        Err(error) => {
            logging::log("[Accounts]", &format!("❌ {}", error.message));
            management_error(i32::from(error.http_status().as_u16()), error.message)
        }
    }
}

// ─── POST /api/accounts/refresh-expiring ────────────────────

/// new ****expired / （）。
///
/// and `/api/accounts/refresh` ： that is ** use 1 ** new 
/// （ not ）； this is ****、 need need that 。
/// and new in ， handler ——
/// Why this not can in 、Whyfailure not to 2xx，
/// `core::credential_maintenance` module header and file that 400 。
pub async fn refresh_expiring_accounts(state: &ServerState) -> Response {
    let report =
        crate::server::core::credential_maintenance::refresh_expiring_report(state.store()).await;
    // this 1 already 1 ： when 1 （ and its 
    // 1 ）—— otherwise「、 to or after 1 」。
    crate::server::core::credential_maintenance::note_manual_run();
    ok_json(report)
}

// ─── GET /api/accounts/usage and POST /api/accounts/checkin ──

/// ── path not in file ─────────────────────
/// · ：`core::usage_query::resolve_batch_targets`
/// （ 1 down —— and use 1 ）；
/// · to ：`core::billing::checkin::resolve_checkin_targets`
/// （ when to and to must use 1 ）。
/// file `accounts_checkin` 1 。


/// POST /api/accounts/checkin
///
/// to ： when up 11-128 。
/// id as empty when ， to id then this ；
/// return `{results, succeeded, active, total, skipped}`；`active` is WorkBuddy
/// ， not etc to success。
///
/// is `core::billing::checkin::run_checkin` —— and when to use 1 
/// （Node is `createAutoCheckin({ runCheckin: accountRoutes.runCheckin })`）。
/// has 1 ：Node handler in try ， AccountStoreError
/// errorPayload to **OpenAI ** body；Rust file use is 
/// （`management_error`）。 this **file has ** not ，
/// 6 already —— in 400/404 up ， before message。
pub async fn accounts_checkin(state: &ServerState, body: &Bytes) -> Response {
    let id = match parse_body(body) {
        Ok(payload) => payload
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|value| !value.is_empty()),
        Err(_) => return management_error(400, "请求内容不是有效 JSON"),
    };
    // path****（ after config when ，
    // `batch_providers` ）； id not limit
    // （ resolve_checkin_targets ）
    let providers = state.auto_checkin().batch_providers();
    // reason to （only， checkin_history ）：
    // 「 to 」 and to in from this by ， and when 
    // 「 to / 」 in when up 。
    // source is to （ single / manual）。
    let source = if id.is_some() { "single" } else { "manual" };
    match checkin::run_checkin(
        state.store(),
        state.billing(),
        providers.as_slice(),
        id.as_deref(),
        "手动签到",
        source,
    )
    .await
    {
        Ok(result) => ok_json(result),
        Err(error) => management_error(error.status_code, error.message),
    }
}

// ─── GET /api/accounts/connections ──────────────────────────

/// （「」）。
///
/// response `{ counts: { <accountId>: <n> } }`：`counts` **** ——
/// by 「 0」， is not as empty 1 0，
/// response（2 seconds 1 ）。
///
/// is 「 in use this down request」（ 1 request in when ），
/// `core::upstream::connections` module header。 is ****，
/// not 、 not ， this ， can 2 seconds 1 。
///
/// this `protected` （`http::router` /api/accounts checkApiKey），
/// and usage 1 —— in config Key before not to 。
pub fn account_connections(state: &ServerState) -> Response {
    let counts = state
        .upstream()
        .connections()
        .snapshot()
        .into_iter()
        .map(|(id, count)| (id, Value::from(count)))
        .collect::<Map<String, Value>>();
    ok_json(json!({ "counts": Value::Object(counts) }))
}

// ─── PATCH /api/accounts/{id} ───────────────────────────────

pub async fn patch_account(state: &ServerState, id: &str, body: &Bytes) -> Response {
    if id.is_empty() {
        return management_error(400, "缺少账号 id");
    }
    let patch = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    if !patch.is_object() {
        return management_error(400, "请求内容必须是 JSON 对象");
    }
    // CatPaw （`balanceToken`）： is ** this 1 has ** ，
    // use `update_account` not （ `update_catpaw_balance_token` ：
    // not use apply_patch）。 in use patch before ， is 1 
    // 「set」 and will ， use patch not will as 
    // （ by ，unknown ignore）。
    let mut balance_changes: Vec<String> = Vec::new();
    if let Some(value) = patch.get("balanceToken") {
        if state.store().catpaw_account_record(&id).is_none() {
            // 「 not in 」 and 「 in not is CatPaw」 need ： before use new ，
            // after this not this （ignore will use as ）。
            let known = state
                .store()
                .list_accounts()
                .get("accounts")
                .and_then(Value::as_array)
                .map(|accounts| {
                    accounts
                        .iter()
                        .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
                })
                .unwrap_or(false);
            return if known {
                management_error(400, "只有 CatPaw 账号有余额查询凭证（balanceToken）")
            } else {
                store_error(AccountStoreError::new("账号不存在", 404))
            };
        }
        match state.store().update_catpaw_balance_token(&id, value) {
            Ok(changes) => balance_changes = changes,
            Err(error) => return store_error(error),
        }
    }
    // ZCode 「 use up 」（`zcodePlan`， `providers::zcode::plan`）：
    // and up CatPaw that 1 1 —— has this 1 this ， not use 
    // `apply_patch`， in this and （ not 400， not is 
    // default）。
    if let Some(value) = patch.get(zcode::PLAN_FIELD) {
        if state.store().zcode_account_record(&id).is_none() {
            let known = state
                .store()
                .list_accounts()
                .get("accounts")
                .and_then(Value::as_array)
                .map(|accounts| {
                    accounts
                        .iter()
                        .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
                })
                .unwrap_or(false);
            return if known {
                management_error(400, "只有 ZCode 账号可以选「使用哪个套餐」（zcodePlan）")
            } else {
                store_error(AccountStoreError::new("账号不存在", 404))
            };
        }
        match state.store().update_zcode_plan(&id, value) {
            Ok(changes) => balance_changes.extend(changes),
            Err(error) => return store_error(error),
        }
    }
    // （`apiKey` / `noAuth`）** and up （`baseUrl`）**：
    // and up 1 —— has this ， not use `apply_patch`。
    // ** id by HTTP path to ， not id**（id is ：、request、
    // file by ）， is this not need need 「 key and 」 that 。
    //
    // `baseUrl` is up ****（ when to ）： use 
    // 「」（ null）， is issue #455 that not 、 not 
    // old 。
    if patch.get("apiKey").is_some() || patch.get("noAuth").is_some() || patch.get("baseUrl").is_some() {
        // `custom_account_provider` 1 ： in not in 、 not 
        // （ is `provider` before ， not need in config，
        // that ）—— and balanceToken / zcodePlan 
        if state.store().custom_account_provider(&id).is_none() {
            let known = state
                .store()
                .list_accounts()
                .get("accounts")
                .and_then(Value::as_array)
                .map(|accounts| {
                    accounts
                        .iter()
                        .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
                })
                .unwrap_or(false);
            return if known {
                management_error(400, "只有自定义提供商的账号有 API Key / 无需鉴权 / 上游地址这几项")
            } else {
                store_error(AccountStoreError::new("账号不存在", 404))
            };
        }
        // request** ** down （`apiKey: null` is has ：
        // ）， its by down use `update_account` —— 
        // ， not 
        let mut scoped = Map::new();
        for key in ["apiKey", "noAuth"] {
            if let Some(value) = patch.get(key) {
                scoped.insert(key.to_string(), value.clone());
            }
        }
        if !scoped.is_empty() {
            match state.store().update_custom_credentials(&id, &Value::Object(scoped)) {
                Ok(changes) => balance_changes.extend(changes),
                Err(error) => return store_error(error),
            }
        }
        // 1 path：`null` / empty = （）， =
        // after 。 and not ，failure not this already 1 failure ——
        // in 1 ， by 「 after 」 can 
        if let Some(value) = patch.get("baseUrl") {
            match state.store().update_custom_base_url(&id, value) {
                Ok(changes) => balance_changes.extend(changes),
                Err(error) => return store_error(error),
            }
        }
    }
    // **config**（`balanceQuery`）： and up apiKey that 1 1 
    // —— has 「 」 this （ in core ： not is 
    // 、 or that 1 no ， 1 400， `update_custom_balance_query`）。
    // （ / / ） in this 1 request，
    // （ `account_store::sql` module header）；response
    // has `has*` （ `store_view::balance_query_public`）。
    if let Some(value) = patch.get("balanceQuery") {
        if state.store().custom_account_provider(&id).is_none() {
            let known = state
                .store()
                .list_accounts()
                .get("accounts")
                .and_then(Value::as_array)
                .map(|accounts| {
                    accounts
                        .iter()
                        .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
                })
                .unwrap_or(false);
            return if known {
                management_error(400, "只有自定义提供商的账号有余额查询配置（balanceQuery）")
            } else {
                store_error(AccountStoreError::new("账号不存在", 404))
            };
        }
        match state.store().update_custom_balance_query(&id, value) {
            Ok(changes) => balance_changes.extend(changes),
            Err(error) => return store_error(error),
        }
    }
    match state.store().update_account(&id, &patch) {
        Ok((account, mut changes)) => {
            changes.extend(balance_changes);
            // success after for ： new use 、 old use 
            ok_json(json!({
                "account": account,
                "changes": changes,
                "list": state.store().list_visible_accounts(),
            }))
        }
        Err(error) => store_error(error),
    }
}

// ─── POST /api/accounts/reorder ──────────────────────────────

/// （）：body `{ queue: [ id, …] }`， from to 。
///
/// up —— = ， = ，
/// 1 ； this by to `priority` 0、1、2…。 no 
/// （ is 「 1 by 」， `account_store::priority`），
/// is ， 1 not use 。
///
/// and `POST /api/accounts/{id}/move`（） ： that old ↑/↓ ，
/// 1 ； this ， 1 。（must is 1 ） and 
/// `account_store::store_order::reorder_accounts`。
pub async fn reorder_accounts(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(items) = payload.get("queue").and_then(Value::as_array) else {
        return management_error(400, "缺少 queue 字段（账号 id 数组，从队首到队尾）");
    };
    // ignore（ and batch ids 1 ），empty then 
    let ids: Vec<String> = items
        .iter()
        .filter_map(Value::as_str)
        .map(|value| value.to_string())
        .collect();
    if ids.is_empty() {
        return management_error(400, "queue 不能为空");
    }
    match state.store().reorder_accounts(&ids) {
        Ok(result) => ok_json(result),
        Err(error) => store_error(error),
    }
}

// ─── POST /api/accounts/{id}/move ───────────────────────────

pub async fn move_account(state: &ServerState, id: &str, body: &Bytes) -> Response {
    if id.is_empty() {
        return management_error(400, "缺少账号 id");
    }
    let payload = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    let direction = if payload.get("direction").and_then(Value::as_str) == Some("down") {
        "down"
    } else {
        "up"
    };
    match state.store().move_account(&id, direction) {
        Ok(result) => {
            if result.get("moved").and_then(Value::as_bool) == Some(false) {
                let reason = result
                    .get("reason")
                    .and_then(Value::as_str)
                    .unwrap_or("顺序未变");
                logging::log("[Accounts]", &format!("顺序未变（{reason}）"));
            }
            ok_json(result)
        }
        Err(error) => store_error(error),
    }
}

// ─── POST /api/accounts/{id}/rate-limits/clear ─────────────────

/// ：body `{model?}`， to that 1 ，otherwise。
///
/// this is 「」 up 。 is 「 new this 」：
/// is from up 429 ， after down 1 request if up ，
/// will —— this is ， not need need 。
/// return `{cleared: , list}`， before list new 。
pub fn clear_rate_limits(state: &ServerState, id: &str, body: &Bytes) -> Response {
    if id.is_empty() {
        return management_error(400, "缺少账号 id");
    }
    let payload = if body.is_empty() {
        Value::Object(Default::default())
    } else {
        match parse_json_body(body, "请求内容不是有效 JSON") {
            Ok(value) => value,
            Err(response) => return response,
        }
    };
    let model = payload
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let cleared = match model {
        Some(model) => usize::from(state.store().clear_rate_limit(id, model)),
        None => state.store().clear_all_rate_limits(id),
    };
    if cleared > 0 {
        logging::log(
            "[Accounts]",
            &format!(
                "🧹 已清除账号 {id} 的限流标记（{}）",
                model.map(|value| value.to_string()).unwrap_or_else(|| format!("{cleared} 个模型"))
            ),
        );
    }
    ok_json(json!({ "cleared": cleared, "list": state.store().list_visible_accounts() }))
}

// ─── DELETE /api/accounts/{id} ──────────────────────────────

pub fn delete_account(state: &ServerState, id: &str) -> Response {
    if id.is_empty() {
        return management_error(400, "缺少账号 id");
    }
    // use need in delete** before **： down use need for 
    // （ after 1 use → this this ）
    match state.store().remove_account(id) {
        Ok(()) => {
            ok_json(json!({ "list": state.store().list_visible_accounts() }))
        }
        Err(error) => store_error(error),
    }
}
