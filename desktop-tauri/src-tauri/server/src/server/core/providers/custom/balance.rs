//! ****： （、 、）。
//!
//! ── Why need has ─────────────────────────────
//! 内置家各有一条自己的余额链路（`providers/*/balance.rs`），
//! （「 API」 that ） before 1 no has —— 「—」。
//! use up ** has **（DeepSeek `/user/balance`、
//! NewAPI `/api/user/self`、OpenCode Go `/v1/usage`…）， ，
//! up and 1 can 、 can 、 can limit by 。
//!
//! ── from （ not is use ）──────────────────────────────
//! in create when **** `balanceScheme`
//! （ `custom_providers` ）， this 1 ：
//! = 「 this 1 is 」 ， use not need need （ not this ） in up 。
//! new no has this ， no has —— will 
//! 。
//!
//! ── （ OmniProxy `server/src/utils/providerBalance.ts`； as ）──
//! | | | | |
//! |---|---|---|---|
//! | `sub2api` | `{base}/v1/usage` | API Key（Bearer） | remaining（`quota.limit/used`，default USD） |
//! | `new_api` | `{base}/api/user/self` + `{base}/api/status` | ****（Bearer + optional `new-api-user` ） | `data.quota`， by `/api/status` |
//! | `deepseek` | `{base}/user/balance` | API Key（Bearer） | `balance_infos[].total_balance`（） |
//! | `opencode` | `{base}/v1/usage` | API Key（Bearer） | rolling / weekly / monthly remaining |
//! | `sensenova` | `platform.sensenova.cn` （ and base none） | **** JWT（ `sensenova_auth`） | use / Flash-Lite 5 when / 7 remaining + |
//! | `zhipu` | `bigmodel.cn` （ and base none） | **** | （） |
//! | `zhipu_zai` | `api.z.ai` （ and base none） | API Key **SIGN **（ `zhipu_sign_jwt`） | （） |
//!
//! before is ** use base**； after is ** **（ in 
//! up ， and not is 1 ）—— this is this 1 1 。
//! 1 ，（ vs API Key ） and 
//! （ vs ） not ， is not is 1 parameter。
//!
//! ── （ and ）────────────────────────────────
//! is （token 1 use ）； this has 
//! need ** 1 **（NewAPI 、 、 
//! ）， in `balanceQuery` （ `custom_accounts`）。
//! sub2api / deepseek / opencode / zhipu_zai use API Key，
//! not need need config —— and use 「DeepSeek not use 」 1 。
//!
//! ── and when ──────────────────────────────────────────────
//! ** **（`credential.proxy`）： is this 
//! up that ， use in that for is 1 （ and 「
//! 1 」 in ： up is use ，none from this not this 
//! ， is not ）。 when 20 seconds —— `egress` default
//! read_timeout is 600 seconds（ to SSE ）， not total when will 1 
//! to 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap / expect / panic。

use serde_json::Value;
use url::Url;

use crate::server::core::account_store::custom_accounts::{CustomBalanceQuery, CustomCredential};
use crate::server::core::account_store::AccountStore;
use crate::server::core::auth_http::send_raw;
use crate::server::core::custom_providers::{
    self, BALANCE_SCHEME_DEEPSEEK, BALANCE_SCHEME_NEW_API, BALANCE_SCHEME_OPENCODE,
    BALANCE_SCHEME_SENSENOVA, BALANCE_SCHEME_SUB2API, BALANCE_SCHEME_ZHIPU,
    BALANCE_SCHEME_ZHIPU_ZAI,
};
use crate::server::core::egress;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::core::providers::adapter::usage_not_configured;
use crate::server::errors::GatewayError;
use crate::server::logging;

use super::balance_parse as parse;
use super::sensenova_auth;

/// request when （ and 1 ：15~20 seconds）
const REQUEST_TIMEOUT_MS: u64 = 20_000;

/// （ in up ， and none）
const ZHIPU_CONSOLE_BASE: &str = "https://bigmodel.cn";

/// （z.ai） 。`/api` this 1 is **path** 1 
/// （biz in `api.z.ai/api` down ）， not is before ；， and 
/// base none（ in up ）。
const ZHIPU_ZAI_BALANCE_URL: &str =
    "https://api.z.ai/api/biz/account/query-customer-account-report";

/// （ and `balanceScheme` 1 1 for ）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scheme {
    Sub2api,
    NewApi,
    DeepSeek,
    Opencode,
    Sensenova,
    Zhipu,
    ZhipuZai,
}

impl Scheme {
    /// id → （unknown id to None；id by `custom_providers` ）
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            BALANCE_SCHEME_SUB2API => Some(Scheme::Sub2api),
            BALANCE_SCHEME_NEW_API => Some(Scheme::NewApi),
            BALANCE_SCHEME_DEEPSEEK => Some(Scheme::DeepSeek),
            BALANCE_SCHEME_OPENCODE => Some(Scheme::Opencode),
            BALANCE_SCHEME_SENSENOVA => Some(Scheme::Sensenova),
            BALANCE_SCHEME_ZHIPU => Some(Scheme::Zhipu),
            BALANCE_SCHEME_ZHIPU_ZAI => Some(Scheme::ZhipuZai),
            _ => None,
        }
    }

    /// （error and is 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Scheme::Sub2api => "Sub2API",
            Scheme::NewApi => "NewAPI",
            Scheme::DeepSeek => "DeepSeek",
            Scheme::Opencode => "OpenCode Go",
            Scheme::Sensenova => " new ",
            Scheme::Zhipu => "",
            Scheme::ZhipuZai => "（z.ai ）",
        }
    }

    /// when use 「、」（ `usage_not_configured`）
    fn credential_hint(self) -> &'static str {
        match self {
            Scheme::Sub2api | Scheme::DeepSeek | Scheme::Opencode | Scheme::ZhipuZai => {
                " API Key"
            }
            Scheme::NewApi => "set （Access Token）",
            Scheme::Sensenova => "set and ",
            Scheme::Zhipu => "set ",
        }
    }
}

/// this 1 has no has can ： + 。
///
/// is `usage_query` and （ before by and 
/// 「set」）—— must， has this 1 。
pub fn supports_usage(provider_id: &str) -> bool {
    custom_providers::is_custom_provider_id(provider_id)
        && custom_providers::balance_scheme_of(provider_id).is_some()
}

/// （ 1 `ProviderAdapter::query_usage` ）。
///
/// failure and 1 ：`Err` this failure（ by use 
/// `error` ， not failure in ）；「 no 」 use 
/// [`usage_not_configured`] 400 + can code， before in 。
pub async fn query_usage(store: &AccountStore, account_id: &str) -> Result<Value, GatewayError> {
    let credential = store
        .custom_credential_by_id(account_id)
        .ok_or_else(|| GatewayError::with_status(404, " not in "))?;
    let Some(provider_id) = store.custom_account_provider(account_id) else {
        return Err(GatewayError::with_status(
            501,
            " this not is ，none use ",
        ));
    };
    let scheme_id = custom_providers::balance_scheme_of(&provider_id).ok_or_else(|| {
        GatewayError::with_status(
            501,
            format!(
                " this no has （{provider_id}）： use 「 API」 ，\
                  or in set up "
            ),
        )
    })?;
    let scheme = Scheme::from_id(&scheme_id).ok_or_else(|| {
        GatewayError::with_status(
            501,
            format!("unknown {scheme_id}（ can can ）"),
        )
    })?;
    let provider = custom_providers::get(&provider_id).unwrap_or(Value::Null);
    let label = provider
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| scheme.label().to_string());
    // can base（ `baseUrl` ）， 
    let base_url = credential
        .base_url_override
        .clone()
        .or_else(|| {
            provider
                .get("baseUrl")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    let query = store
        .custom_balance_query_by_id(account_id)
        .unwrap_or_else(empty_query);
    let proxy = credential.proxy.as_ref();
    let label = label.as_str();
    // error and 1 use ** **（ use in up to that 1 ），
    // not is —— 1 can can has not 
    let what = |kind: &str| format!("{label} {kind}");

    let mut report = match scheme {
        Scheme::Sub2api => {
            let key = require_api_key(&credential, scheme, label)?;
            let url = build_url(&base_url, Api::Usage)?;
            let payload = get_json(&url, &bearer(&key), proxy, &what("")).await?;
            parse::parse_sub2api(&payload).ok_or_else(|| invalid_response(label))
        }
        Scheme::DeepSeek => {
            let key = require_api_key(&credential, scheme, label)?;
            // DeepSeek in down 1 （ not is `/v1` that before ）
            let url = build_url(&base_url, Api::DeepSeekBalance)?;
            let payload = get_json(&url, &bearer(&key), proxy, &what("")).await?;
            parse::parse_deepseek(&payload).ok_or_else(|| invalid_response(label))
        }
        Scheme::Opencode => {
            let key = require_api_key(&credential, scheme, label)?;
            let url = build_url(&base_url, Api::Usage)?;
            let payload = get_json(&url, &bearer(&key), proxy, &what("usage")).await?;
            parse::parse_opencode(&payload).ok_or_else(|| invalid_response(label))
        }
        Scheme::NewApi => query_new_api(&base_url, &query, proxy, label).await,
        Scheme::Zhipu => query_zhipu(&query, proxy, label).await,
        Scheme::ZhipuZai => query_zhipu_zai(&credential, proxy, label).await,
        Scheme::Sensenova => query_sensenova(store, &credential, &query, proxy, label).await,
    }?;
    report["quotaSources"] = Value::Array(super::quota::sources(&provider_id, scheme, &report));
    Ok(report)
}

/// NewAPI：（`/api/user/self`）+ （`/api/status`）。
///
/// is **** （ not ）， not to 「quota」，
/// not this failure —— that when quota + `quota`（）。
async fn query_new_api(
    base_url: &str,
    query: &CustomBalanceQuery,
    proxy: Option<&ResolvedProxy>,
    label: &str,
) -> Result<Value, GatewayError> {
    let scheme = Scheme::NewApi;
    let token = query.access_token.trim();
    if token.is_empty() {
        return Err(usage_not_configured(label, scheme.credential_hint()));
    }
    let self_url = build_url(base_url, Api::NewApiSelf)?;
    let status_url = build_url(base_url, Api::NewApiStatus)?;
    let mut headers = vec![("Authorization".to_string(), format!("Bearer {token}"))];
    let user_id = query.user_id.trim();
    if !user_id.is_empty() {
        headers.push(("new-api-user".to_string(), user_id.to_string()));
    }
    let account = get_json(&self_url, &headers, proxy, &format!("{label} ")).await?;
    let Some((remaining, used)) = parse::parse_new_api_account(&account) else {
        return Err(invalid_response(label));
    };
    let status = get_json(&status_url, &[], proxy, &format!("{label} "))
        .await
        .ok();
    let display = status.as_ref().and_then(parse::new_api_display);
    Ok(parse::new_api_document(
        remaining,
        used,
        display.as_ref(),
        &account,
        status.as_ref(),
    ))
}

/// ：`bigmodel.cn` （ is ）。
///
/// ** in **（ and base none）： in 
/// up ，（`open.bigmodel.cn/api/…` or `api.z.ai`） not 。
/// is Cookie `bigmodel_token_production` （）。
async fn query_zhipu(
    query: &CustomBalanceQuery,
    proxy: Option<&ResolvedProxy>,
    label: &str,
) -> Result<Value, GatewayError> {
    let scheme = Scheme::Zhipu;
    let token = query.token.trim();
    if token.is_empty() {
        return Err(usage_not_configured(label, scheme.credential_hint()));
    }
    let url = format!("{ZHIPU_CONSOLE_BASE}/api/biz/account/query-customer-account-report");
    // this is ****（ not is Bearer），
    let headers = vec![("Authorization".to_string(), token.to_string())];
    let payload = get_json(&url, &headers, proxy, &format!("{label} ")).await?;
    parse::parse_zhipu(&payload, "CNY", "invalid or expired")
        .map_err(|message| GatewayError::with_status(502, message))
}

/// （z.ai）： 1 ， is ****。
///
/// ── Why not is ────────────────────────────────
/// z.ai ****： API Key（`{id}.{secret}` ）
/// SIGN JWT（ [`zhipu_sign_jwt`]）， API Key will 
/// （ up in / Authorization when `code:1001`，）。API Key
/// is need ， this 1 ** not need need config** —— and deepseek / opencode
/// 1 ， not 。
async fn query_zhipu_zai(
    credential: &CustomCredential,
    proxy: Option<&ResolvedProxy>,
    label: &str,
) -> Result<Value, GatewayError> {
    let scheme = Scheme::ZhipuZai;
    let key = require_api_key(credential, scheme, label)?;
    let jwt = zhipu_sign_jwt(&key).ok_or_else(|| {
        GatewayError::with_status(
            400,
            format!(
                "{label} API Key not is 「id.secret」，none use ：\
                  to z.ai new API Key"
            ),
        )
    })?;
    // ： is ****（ not is Bearer）
    let headers = vec![("Authorization".to_string(), jwt)];
    let payload = get_json(
        ZHIPU_ZAI_BALANCE_URL,
        &headers,
        proxy,
        &format!("{label} "),
    )
    .await?;
    parse::parse_zhipu(&payload, "USD", " API Key invalid or expired")
        .map_err(|message| GatewayError::with_status(502, message))
}

/// API Key（`{id}.{secret}` ） ****
/// （ JWT that ：HS256、header `sign_type: SIGN`）。
///
/// is base64url（**none**）：header as 
/// `{"alg":"HS256","sign_type":"SIGN"}`，payload is 
/// `{api_key: <id >, exp, timestamp}`（ms；exp 1 when after —— server
/// 「 no 」， 1 ， not need need and ）， use secret for 
/// `header.payload` HMAC-SHA256。 not （Key no has `id.secret` ）
/// to None —— use 1 can configerror。
fn zhipu_sign_jwt(api_key: &str) -> Option<String> {
    use base64::Engine;
    use hmac::Mac;
    type HmacSha256 = hmac::Hmac<sha2::Sha256>;
    let (id, secret) = api_key.trim().split_once('.')?;
    let (id, secret) = (id.trim(), secret.trim());
    if id.is_empty() || secret.is_empty() {
        return None;
    }
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let header = engine.encode(br#"{"alg":"HS256","sign_type":"SIGN"}"#);
    let now = chrono::Utc::now().timestamp_millis();
    let payload = engine.encode(
        serde_json::to_vec(&serde_json::json!({
            "api_key": id,
            "exp": now + 3_600_000,
            "timestamp": now,
        }))
        .ok()?,
    );
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(format!("{header}.{payload}").as_bytes());
    let signature = engine.encode(mac.finalize().into_bytes());
    Some(format!("{header}.{payload}.{signature}"))
}

/// new ：（）→ usage。
///
/// and 1 ： has ** no **（ before 60 seconds） 
/// use ，otherwise use 1 and new （ down 1 not ）。
/// failure is **failure**（502） not is 「not configured」—— use already ，
/// failure Reason in up （、、）， need 。
async fn query_sensenova(
    store: &AccountStore,
    credential: &CustomCredential,
    query: &CustomBalanceQuery,
    proxy: Option<&ResolvedProxy>,
    label: &str,
) -> Result<Value, GatewayError> {
    let scheme = Scheme::Sensenova;
    let mut jwt = query.sensenova_jwt.trim().to_string();
    if jwt.is_empty() || !sensenova_auth::jwt_reusable(&jwt) {
        let username = query.username.trim();
        if username.is_empty() || query.password.is_empty() {
            return Err(usage_not_configured(label, scheme.credential_hint()));
        }
        let result = sensenova_auth::login(username, &query.password, proxy)
            .await
            .map_err(|message| GatewayError::with_status(502, message))?;
        // ：failure down 1 （ that will 1 ）， not can 
        // 1 success failure
        if let Err(error) = store.store_custom_balance_jwt(
            &credential.account_id,
            &result.access_token,
            result.expires_at_ms,
        ) {
            logging::verbose(
                "[Usage]",
                &format!(
                    "failure（ {}）：{}",
                    credential.account_name, error.message
                ),
            );
        }
        jwt = result.access_token;
    }
    let tenant = sensenova_auth::jwt_tenant_id(&jwt).ok_or_else(|| {
        GatewayError::with_status(502, format!("{label} no has tenant_id，none"))
    })?;
    let account_id_param: String =
        url::form_urlencoded::byte_serialize(tenant.as_bytes()).collect();
    let url = format!(
        "{}/console/v1/tokenplan/pool-usage?account_id={account_id_param}",
        sensenova_auth::PLATFORM_LITE_BASE
    );
    let headers = vec![
        ("Authorization".to_string(), format!("Bearer {jwt}")),
        (
            "Referer".to_string(),
            format!("{}/console", sensenova_auth::PLATFORM_BASE),
        ),
    ];
    let payload = get_json(&url, &headers, proxy, &format!("{label} usage")).await?;
    parse::parse_sensenova(&payload).ok_or_else(|| invalid_response(label))
}

/// this 1 in base up path（ `buildQueryUri` /
/// `buildNewApiUri` 1 ）。
///
/// Why need 「 not is 」： use base not 1 —— has to 
/// `https://host`、 has to `https://host/v1`、 has path up 。
/// need can use ， is not is already is path， is not 。
#[derive(Clone, Copy)]
enum Api {
    /// `{base}/v1/usage`（sub2api and OpenCode Go use ）
    Usage,
    /// `{base}/user/balance`（DeepSeek； before `/v1` need ）
    DeepSeekBalance,
    /// `{base}/api/user/self`（NewAPI ）
    NewApiSelf,
    /// `{base}/api/status`（NewAPI ，）
    NewApiStatus,
}

/// base + → （base when 400， to 「set」）。
fn build_url(base_url: &str, api: Api) -> Result<String, GatewayError> {
    let trimmed = base_url.trim();
    let invalid = || {
        GatewayError::with_status(
            400,
            " this Base URL unavailable（ need need ）： in 「」setcheck",
        )
    };
    if trimmed.is_empty() {
        return Err(invalid());
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = Url::parse(&candidate).map_err(|_| invalid())?;
    if url.host_str().unwrap_or("").is_empty() {
        return Err(invalid());
    }
    let mut parts: Vec<String> = url
        .path()
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    match api {
        Api::Usage => {
            if !ends_with(&parts, &["v1", "usage"]) {
                if parts.last().map(String::as_str) == Some("v1") {
                    parts.push("usage".to_string());
                } else {
                    parts.push("v1".to_string());
                    parts.push("usage".to_string());
                }
            }
        }
        Api::DeepSeekBalance => {
            if !ends_with(&parts, &["user", "balance"]) {
                if parts.last().map(String::as_str) == Some("v1") {
                    parts.pop();
                }
                parts.push("user".to_string());
                parts.push("balance".to_string());
            }
        }
        Api::NewApiSelf => append_api_path(&mut parts, &["api", "user", "self"]),
        Api::NewApiStatus => append_api_path(&mut parts, &["api", "status"]),
    }
    url.set_path(&format!("/{}", parts.join("/")));
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

/// NewAPI `/api/...` path（ already is `api` when not that 1 ）
fn append_api_path(parts: &mut Vec<String>, suffix: &[&str]) {
    if ends_with(parts, suffix) {
        return;
    }
    if parts.last().map(String::as_str) == Some("v1") {
        parts.pop();
    }
    if parts.last().map(String::as_str) == Some("api") && suffix.first() == Some(&"api") {
        parts.extend(suffix.iter().skip(1).map(|part| (*part).to_string()));
    } else {
        parts.extend(suffix.iter().map(|part| (*part).to_string()));
    }
}

/// path is `suffix` 
fn ends_with(parts: &[String], suffix: &[&str]) -> bool {
    if parts.len() < suffix.len() {
        return false;
    }
    parts[parts.len() - suffix.len()..]
        .iter()
        .zip(suffix)
        .all(|(part, expected)| part == expected)
}

/// 1 GET and JSON（ when 、error、error in this ）。
async fn get_json(
    url: &str,
    headers: &[(String, String)],
    proxy: Option<&ResolvedProxy>,
    what: &str,
) -> Result<Value, GatewayError> {
    let response = send_raw("GET", url, None, headers, proxy, Some(REQUEST_TIMEOUT_MS))
        .await
        .map_err(|error| {
            GatewayError::with_status(
                502,
                format!("{what}requestfailure：{}", egress::describe_error_detail(&error)),
            )
        })?;
    if !response.ok {
        // 401 / 403 （「 not for / no 」 and 「 up 」 is ），
        // its by 502 —— and error 1 to 
        let status = if matches!(response.status, 401 | 403) {
            response.status as i32
        } else {
            502
        };
        let detail = describe_payload_error(&response.payload);
        return Err(GatewayError::with_status(
            status,
            format!("{what} up （HTTP {}）{detail}", response.status),
        ));
    }
    response.payload.ok_or_else(|| {
        GatewayError::with_status(502, format!("{what}response not is valid JSON"))
    })
}

/// up errorresponse 1 （`error.message` / `message` / `msg` / `error`）
fn describe_payload_error(payload: &Option<Value>) -> String {
    let Some(payload) = payload else { return String::new() };
    let message = payload
        .get("error")
        .and_then(|error| {
            error
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| error.as_str())
        })
        .or_else(|| payload.get("message").and_then(Value::as_str))
        .or_else(|| payload.get("msg").and_then(Value::as_str))
        .map(str::trim)
        .filter(|text| !text.is_empty());
    match message {
        Some(text) => format!("：{}", parse::shorten(text, 160)),
        None => String::new(),
    }
}

/// API Key（sub2api / deepseek / opencode use ）
fn require_api_key(
    credential: &CustomCredential,
    scheme: Scheme,
    label: &str,
) -> Result<String, GatewayError> {
    let key = credential.api_key.trim();
    if key.is_empty() {
        return Err(usage_not_configured(label, scheme.credential_hint()));
    }
    Ok(key.to_string())
}

/// Bearer 
fn bearer(token: &str) -> Vec<(String, String)> {
    vec![("Authorization".to_string(), format!("Bearer {token}"))]
}

/// 「 up response not 」 1 （502： is up that ， not is config）
fn invalid_response(label: &str) -> GatewayError {
    GatewayError::with_status(
        502,
        format!("{label} response（ not ）"),
    )
}

/// emptyconfig（ no has `balanceQuery` when ）
fn empty_query() -> CustomBalanceQuery {
    CustomBalanceQuery {
        user_id: String::new(),
        access_token: String::new(),
        username: String::new(),
        password: String::new(),
        token: String::new(),
        sensenova_jwt: String::new(),
        sensenova_jwt_expires_at: 0,
    }
}
