//! new （SenseNova）****：OAuth2 + PKCE JWT。
//!
//! ── Why need （ this 1 ）──────────────
//! （`platform.sensenova.cn` tokenplan/pool-usage） 
//! Bearer JWT，** use API Key not to ** —— is （
//! API Key ， JWT /）。 use up can is 
//! ， 1 （：OmniProxy 
//! `server/src/utils/sensenovaAuth.ts`， for ）。
//!
//! ── （， and for ）────────────────────────────
//! 1. `GET {platform}/oauth2/auth?...`（PKCE + state）—— to not 
//! location， from **** query `login_challenge`；
//!   2. `POST {iam}/iam/authn/v1/auth/nova/login` `{username, password, challenge}`
//! → response `redirect`；
//! 3. from this redirect （ Cookie）， to `code=`；
//!   4. `POST {platform}/oauth2/token`（authorization_code + code_verifier）
//!      → `access_token`（JWT）。
//! 「config」：signinUrl / callbackUrl is 
//! `platform.sensenova.cn`，IAM is `iam.sensecoreapi.cn`，clientId = `nova`。
//!
//! ── Why、 Cookie ──────────────────────────
//! `auth_http::send_raw` reqwest default（，** not to 
//! Set-Cookie**）， this 1 need will Cookie down 1 。 use 
//! `redirect: 'manual'` + 1 Cookie Jar（ `name=value`， not 
//! Domain / Path / ）—— that in this ： this Cookie is 
//! 1 will ， by 「」 can ， Cookie not 。
//!
//! ── JWT （ and tenant_id）────────────────────
//! need `?account_id=<JWT payload ext.tenant_id>`， need 
//! `exp`（ before 60 seconds as ， and ）。 payload ，
//! ** not **： this is to ， not is need need 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap / expect / panic。
//! failure 1 return can `String`（ use error）。

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use url::Url;

use crate::server::core::egress;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::logging;

/// （）：、、 ** 1 **
pub(super) const PLATFORM_BASE: &str = "https://platform.sensenova.cn";
/// （`/lite` 1 ， and token not path）
pub(super) const PLATFORM_LITE_BASE: &str = "https://platform.sensenova.cn/lite";
/// IAM ：
const IAM_BASE: &str = "https://iam.sensecoreapi.cn";
/// OAuth client（config clientId，）
const CLIENT_ID: &str = "nova";
/// scope（ new that is ）
const SCOPE: &str = "openid offline offline_access";
/// limit（ 12）
const MAX_REDIRECTS: usize = 12;
/// request when ： is （ not ）， to 20 seconds
const STEP_TIMEOUT_MS: u64 = 20_000;
/// UA： for UA has ， 1 UA
const USER_AGENT: &str = "Mozilla/5.0";
/// before （seconds）： and ，
const EXPIRY_SKEW_SECONDS: f64 = 60.0;

/// 1 （ need use ）
pub(super) struct LoginResult {
    /// JWT
    pub access_token: String,
    /// to when （ms； not exp when to 0 = not ， use not can ）
    pub expires_at_ms: i64,
}

/// JWT is can use （`exp` valid and to has [`EXPIRY_SKEW_SECONDS`] up ）。
/// not payload / no has `exp` 1 as not can use —— 1 。
pub(super) fn jwt_reusable(jwt: &str) -> bool {
    let Some(payload) = jwt_payload(jwt) else { return false };
    let Some(exp) = payload.get("exp").and_then(Value::as_f64) else { return false };
    if !(exp > 0.0) {
        return false;
    }
    let now = logging::now_ms() as f64 / 1000.0;
    now < exp - EXPIRY_SKEW_SECONDS
}

/// JWT → `exp` ms when （ not to 0）。
pub(super) fn jwt_expires_at_ms(jwt: &str) -> i64 {
    jwt_payload(jwt)
        .and_then(|payload| payload.get("exp").and_then(Value::as_f64))
        .filter(|exp| *exp > 0.0)
        .map(|exp| (exp * 1000.0) as i64)
        .unwrap_or(0)
}

/// JWT → id（`ext.tenant_id`， `tenant_id`）。
///
/// `account_id` parameter is —— this 1 no ，
/// use to `None` when by 「successunavailable」。
pub(super) fn jwt_tenant_id(jwt: &str) -> Option<String> {
    let payload = jwt_payload(jwt)?;
    let candidate = payload
        .get("ext")
        .and_then(|ext| ext.get("tenant_id"))
        .or_else(|| payload.get("tenant_id"))?;
    let text = match candidate {
        Value::String(text) => text.trim().to_string(),
        Value::Number(number) => number.to_string(),
        _ => String::new(),
    };
    (!text.is_empty()).then_some(text)
}

/// JWT payload → JSON（base64url， `=`； to None）
fn jwt_payload(jwt: &str) -> Option<Value> {
    let segment = jwt.split('.').nth(1)?;
    let text = segment.trim();
    if text.is_empty() {
        return None;
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(text.trim_end_matches('=').as_bytes())
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// ， JWT（failure to can Reason）。
pub(super) async fn login(
    username: &str,
    password: &str,
    proxy: Option<&ResolvedProxy>,
) -> Result<LoginResult, String> {
    let username = username.trim();
    if username.is_empty() || password.is_empty() {
        return Err("商汤日日新的账号 / 密码未配置".to_string());
    }
    let verifier = random_secret()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = random_hex()?;
    // not ： this 1 need Location / Set-Cookie
    // （ `egress::client_without_redirects`）
    let client = egress::client_without_redirects(proxy);
    let mut jar = CookieJar::default();

    // 1) ， to in ， login_challenge
    let auth_url = authorize_url(&challenge, &state)?;
    let followed = follow(&client, &auth_url, &mut jar).await?;
    if followed.status >= 400 {
        return Err(format!("商汤授权页返回 HTTP {}", followed.status));
    }
    let login_challenge = query_param(&followed.url, "login_challenge")
        .ok_or_else(|| "未能从商汤授权页取到登录 challenge".to_string())?;

    // 2) （IAM ）
    let body = json!({
        "username": username,
        "password": password,
        "challenge": login_challenge,
    });
    let response = client
        .post(format!("{IAM_BASE}/iam/authn/v1/auth/nova/login"))
        .header("Content-Type", "application/json")
        .header("User-Agent", USER_AGENT)
        .header("Origin", PLATFORM_BASE)
        .header("Referer", format!("{PLATFORM_BASE}/login"))
        .timeout(std::time::Duration::from_millis(STEP_TIMEOUT_MS))
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("商汤登录接口请求失败：{}", egress::describe_error_detail(&error)))?;
    let status = response.status().as_u16();
    let payload = response_json(response).await;
    if status >= 400 || payload.is_none() {
        let message = payload
            .as_ref()
            .and_then(|value| value.get("message").and_then(Value::as_str))
            .map(str::to_string)
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!("商汤账号登录失败：{}", shorten(&message)));
    }
    let redirect = payload
        .as_ref()
        .and_then(|value| value.get("redirect").and_then(Value::as_str))
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if redirect.is_empty() {
        return Err("商汤账号登录没有返回跳转地址".to_string());
    }

    // 3) redirect to ， 1 code
    let start = Url::parse(PLATFORM_BASE)
        .ok()
        .and_then(|base| base.join(&redirect).ok())
        .map(|url| url.to_string())
        .unwrap_or(redirect);
    let followed = follow(&client, &start, &mut jar).await?;
    let code = query_param(&followed.url, "code")
        .ok_or_else(|| "商汤登录没有拿到一次性授权码".to_string())?;

    // 4) use code access_token
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "authorization_code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("code", &code)
        .append_pair("redirect_uri", PLATFORM_BASE)
        .append_pair("code_verifier", &verifier)
        .append_pair("state", &state)
        .finish();
    let response = client
        .post(format!("{PLATFORM_BASE}/oauth2/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", USER_AGENT)
        .timeout(std::time::Duration::from_millis(STEP_TIMEOUT_MS))
        .body(form)
        .send()
        .await
        .map_err(|error| format!("商汤换码请求失败：{}", egress::describe_error_detail(&error)))?;
    let status = response.status().as_u16();
    let payload = response_json(response).await;
    let access_token = payload
        .as_ref()
        .and_then(|value| value.get("access_token").and_then(Value::as_str))
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if status >= 400 || access_token.is_empty() {
        let message = payload
            .as_ref()
            .and_then(|value| {
                value
                    .get("error_description")
                    .and_then(Value::as_str)
                    .or_else(|| value.get("error").and_then(Value::as_str))
            })
            .map(str::to_string)
            .unwrap_or_else(|| format!("HTTP {status}"));
        return Err(format!("商汤换取登录票失败：{}", shorten(&message)));
    }
    Ok(LoginResult {
        expires_at_ms: jwt_expires_at_ms(&access_token),
        access_token,
    })
}

/// （parameter：response_type / client_id / PKCE / redirect_uri /
/// scope / state / lang）。
fn authorize_url(challenge: &str, state: &str) -> Result<String, String> {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", challenge)
        .append_pair("redirect_uri", PLATFORM_BASE)
        .append_pair("scope", SCOPE)
        .append_pair("state", state)
        .append_pair("lang", "zh-CN")
        .finish();
    Ok(format!("{PLATFORM_BASE}/oauth2/auth?{query}"))
}

/// 3xx to （ Cookie、 Set-Cookie）。
///
/// return that 1 and ；** not **「 2xx 」 ——
/// use by （ in 4xx and need ）。
async fn follow(
    client: &reqwest::Client,
    start: &str,
    jar: &mut CookieJar,
) -> Result<Followed, String> {
    let mut current = start.to_string();
    for _ in 0..=MAX_REDIRECTS {
        let mut request = client
            .get(&current)
            .header("User-Agent", USER_AGENT)
            .timeout(std::time::Duration::from_millis(STEP_TIMEOUT_MS));
        let cookie = jar.header();
        if !cookie.is_empty() {
            request = request.header("Cookie", cookie);
        }
        let response = request
            .send()
            .await
            .map_err(|error| format!("商汤登录跳转请求失败：{}", egress::describe_error_detail(&error)))?;
        jar.capture(&response);
        let status = response.status().as_u16();
        let location = response
            .headers()
            .get("location")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        match location {
            Some(location) if (300..400).contains(&status) => {
                current = resolve_url(&current, &location)?;
            }
            _ => {
                let url = Url::parse(&current).ok();
                return Ok(Followed { status, url });
            }
        }
    }
    Err("商汤登录跳转次数过多（已超过上限）".to_string())
}

/// for / for location → for （， not ）
fn resolve_url(current: &str, location: &str) -> Result<String, String> {
    let base = Url::parse(current).map_err(|_| "商汤登录跳转地址无法解析".to_string())?;
    if let Ok(absolute) = Url::parse(location) {
        return Ok(absolute.to_string());
    }
    base.join(location)
        .map(|url| url.to_string())
        .map_err(|_| "商汤登录跳转地址不是合法 URL".to_string())
}

/// 1 response JSON（ not / not is JSON to None； by use ）
async fn response_json(response: reqwest::Response) -> Option<Value> {
    let text = response.text().await.ok()?;
    if text.trim().is_empty() {
        return None;
    }
    serde_json::from_str::<Value>(&text).ok()
}

/// URL 1 query parameter（ not to None）。 1 `+` → empty and 
/// percent-decode —— `code` / `login_challenge` when by 。
fn query_param(url: &Option<Url>, name: &str) -> Option<String> {
    let url = url.as_ref()?;
    let value = url
        .query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.to_string())?;
    let text = value.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Cookie Jar： `name=value`，（module header）。
#[derive(Default)]
struct CookieJar {
    cookies: Vec<(String, String)>,
}

impl CookieJar {
    fn capture(&mut self, response: &reqwest::Response) {
        for header in response.headers().get_all("set-cookie") {
            let Ok(text) = header.to_str() else { continue };
            let pair = text.split(';').next().unwrap_or("");
            let Some((name, value)) = pair.split_once('=') else { continue };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            let value = value.trim().to_string();
            match self.cookies.iter_mut().find(|(known, _)| known == name) {
                Some(slot) => slot.1 = value,
                None => self.cookies.push((name.to_string(), value)),
            }
        }
    }

    fn header(&self) -> String {
        self.cookies
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// in （ + ；failure when is None， query parameter）
struct Followed {
    status: u16,
    url: Option<Url>,
}

/// PKCE code_verifier：32 → base64url（43 ， in 43~128 ）
fn random_secret() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| "无法生成安全的随机串（系统随机源不可用）".to_string())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// state：16 → hex（ `randomBytes(16).toString('hex')`）
fn random_hex() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)
        .map_err(|_| "无法生成安全的随机串（系统随机源不可用）".to_string())?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// （ up error can can is 1 HTML）
fn shorten(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 160 {
        return trimmed.to_string();
    }
    let head: String = trimmed.chars().take(157).collect();
    format!("{head}...")
}
