//! 管理面板的注册 / 登录 / 刷新 / 登出（`/api/panel/*`）。
//!
//! ── （， OmniProxy ）────────────────
//! is 「」 ，API Key is 「」 ， 1 ：
//! · success —— access（2 when ，path=/）+ refresh
//! （30 ，path=/api/panel）；`/api/*` by `http::require_api_key`
//! access will or API Key；
//! · access after before `POST /api/panel/refresh` new 
//! （： old refresh 、 new refresh will ； old refresh
//! will as and ）；
//! · （ no has ） when ：
//! `POST /api/panel/setup` create and 。
//!
//! ── ──────────────────────────────────────────────────
//! 1 failure 5 5 minutes（ by IP， not is —— will 
//! 「」 1 ）。 public ： use 
//! no has ， by and bcrypt 。

use axum::body::Bytes;
use axum::extract::{ConnectInfo, Query, State};
use axum::http::{header::SET_COOKIE, HeaderMap};
use axum::response::Response;
use std::net::SocketAddr;

use crate::server::access;
use crate::server::altcha;
use crate::server::config;
use crate::server::errors;
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

fn body_str<'a>(payload: &'a serde_json::Value, field: &str) -> String {
    payload
        .get(field)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

/// （ALTCHA proof-of-work， `server::altcha`）： when ，
/// / requestmust payload，otherwise 400。failure** not **
/// failure —— that for 「」， no no to that 。
///
/// `local_panel` 1 （ and headless is set）。
fn check_captcha(payload: &serde_json::Value, local_panel: bool) -> Result<(), Response> {
    if !config::current().captcha_enabled(local_panel) {
        return Ok(());
    }
    let Some(token) = payload.get("captcha").and_then(serde_json::Value::as_str) else {
        return Err(errors::management_error(400, " after "));
    };
    altcha::verify(token).map_err(|message| errors::management_error(400, &message))
}

/// `GET /api/panel/captcha` —— 1 ALTCHA challenge（ when ）。
///
/// public： when use no has 。response is ** challenge JSON**
/// （ not API success/data ）—— widget ，
/// failure（altcha-lib challengeHandler for ）。
/// when 400 —— before 。
pub async fn captcha_challenge(State(state): State<ServerState>) -> Response {
    match altcha::challenge(state.local_panel) {
        Some(challenge) => crate::server::http::raw_json(challenge),
        None => errors::management_error(400, "enabled"),
    }
}

/// client need 「response」 request（ `body`）。server not 
/// /： by before in （`cookie_probe`）failure after to 。
const TOKEN_BODY_MODE: &str = "x-panel-auth-mode";

/// 1 （ + when ）—— 。
///
/// ── Why before not ────────────────────────────────────
/// （2026-09-29 ）： this in for Set-Cookie / request / URL
/// parameter none from ， and before down 90 
/// old —— old 、 、query ， 1 will 「 before 
/// 」 。 ： / new when to cookie must
/// ** etc that and new **，「 this can 」；
/// its 1 （、 old 、 for not up ） 1 response —— can to ， not can 
/// 。 when （ → seconds） in ；
/// after will response， can none（client）。
///
/// ── Why is 1 ───────────────────────────────────────────
/// need is 「 that response cookie can not can down 」，response in 
/// ** 1 ** `Set-Cookie`（ will cookie 1 ， `access::ACCESS_COOKIE`）。
/// from before is as —— 1 will in 、
/// 「」， 1 ， is 、 will 。 
/// must and for 1 ： in is 1 。
static LAST_ISSUED_PROBE: std::sync::OnceLock<
    std::sync::Mutex<Option<(String, std::time::Instant)>>,
> = std::sync::OnceLock::new();
const PROBE_FRESH_WINDOW: std::time::Duration = std::time::Duration::from_secs(30 * 60);

fn remember_issued_probe(value: &str) {
    let Ok(mut slot) = LAST_ISSUED_PROBE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
    else {
        return;
    };
    *slot = Some((value.to_string(), std::time::Instant::now()));
}

/// to cookie is 「currentvalid」： in 、 etc that 、 and in 。
/// （ has ） when by 「 in not in its in 」， not 1 ——
/// and `access::cookie_values` 1 。
fn probe_is_currently_valid(received: &[String]) -> bool {
    let Ok(slot) = LAST_ISSUED_PROBE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
    else {
        return false;
    };
    match &*slot {
        Some((expected, at)) if at.elapsed() < PROBE_FRESH_WINDOW => received.contains(expected),
        _ => false,
    }
}

fn tokens_in_body(
    headers: &HeaderMap,
    query: &std::collections::HashMap<String, String>,
) -> bool {
    // （ before to cookie not after need ），****：
    // request + URL parameter —— has in request，query not 
    // （request can to ， query ）。
    let header_marked = headers
        .get(TOKEN_BODY_MODE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("body"));
    let query_marked = query
        .get("auth-mode")
        .map(|value| value.eq_ignore_ascii_case("body"))
        .unwrap_or(false);
    if header_marked || query_marked {
        return true;
    }
    // cookie must is 「 that 」（ `probe_is_currently_valid`
    // ） cookie available —— 、 old （ before / its 
    // down ）、 for not up ， 1 response。 this is ，
    // not before in ， not old 。
    !probe_is_currently_valid(&access::cookie_values(headers, access::PROBE_COOKIE))
}

/// can （ use ）：。
fn marker_source(headers: &HeaderMap, query: &std::collections::HashMap<String, String>) -> &'static str {
    if headers
        .get(TOKEN_BODY_MODE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("body"))
    {
        ""
    } else if query
        .get("auth-mode")
        .map(|value| value.eq_ignore_ascii_case("body"))
        .unwrap_or(false)
    {
        "parameter"
    } else {
        "none"
    }
}

/// cookie up （ use ）：` 1 in ` / ` no has `。
///
/// this 1 is 「Why this response」 in 1 ， not 
/// response。 from before this is （` in ` / ` has 1 ` / ` no has `），
/// 「 has 1 」 is and in ；response 1 will cookie after ，
/// 1 ，" 1 " this not in —— need 「 this 1 can not can 」。
fn probe_shape(headers: &HeaderMap) -> &'static str {
    if access::cookie_values(headers, access::PROBE_COOKIE).is_empty() {
        " no has "
    } else {
        " 1 in "
    }
}

/// `ANY /api/panel/cookie-probe` —— will （ when ）。
///
/// public： in before 。 1 ** no has ** cookie：** 1 ** 90 
/// cookie（`PROBE_COOKIE`， and response that `Set-Cookie` ） and 
/// `roundtrip:false`； if down 、 and for up ，server
/// `roundtrip:true` —— cookie ，response not need need body。
/// `false`（ in Set-Cookie / cookie / ）＝
/// cookie not ， before `x-panel-auth-mode: body` 。 is ，
/// not as ： in cookie ，。
pub async fn cookie_probe(
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    // ── for ： must is ** that ** ───────────────────
    // cookie by 、 not （RFC 6265）——fnOS （:5666） and 
    // （:3065） use 1 cookie ： use 1 ，
    // down 1 ； from :5666 in when this ** old ** will up 
    // （2026-10-13 ：dev up is that 63 ）。
    // 「 has no has 」 will （ 2026-09-29： cookie in 、
    // not 、）。must `expect=`， for 。
    let received = access::cookie_values(&headers, access::PROBE_COOKIE);
    if let Some(expected) = non_empty(query.get("expect")) {
        let roundtrip = received.iter().any(|value| value == &expected);
        // not ： will that old cookie， down 1 
        return ok_json(serde_json::json!({ "roundtrip": roundtrip }));
    }
    // 1 （none expect）： 1 new ，response（，
    // none will ；HttpOnly JS not to ， body for ）
    let first = access::random_hex(16);
    // （90 ）： down after is 「 this cookie 」 
    // —— / new request，response and old 1 （ `tokens_in_body`）。
    let cookie = format!(
        "{}={first}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax",
        access::PROBE_COOKIE,
        90 * 24 * 3600
    );
    remember_issued_probe(&first);
    let mut response = ok_json(serde_json::json!({
        "roundtrip": false,
        "token": first,
    }));
    if let Ok(value) = axum::http::HeaderValue::from_str(&cookie) {
        response.headers_mut().append(SET_COOKIE, value);
    }
    response
}

fn non_empty(value: Option<&String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn issue_response(session: access::IssuedSession, tokens_in_body: bool) -> Response {
    // not response by client （ `tokens_in_body`）：
    // `x-panel-auth-mode: body` is 「 to cookie not 」 
    // （fnOS docker this in ）， before body 
    // localStorage、 after request `x-panel-token` （ access::session_valid
    // and web_shim PANEL_AUTH ）； not and old 
    // 1 —— in HttpOnly cookie ，JS not to 。 
    // JS can ： is can ， use available that 
    // XSS in 「cookie unavailable」 。
    let mut payload = serde_json::json!({ "loggedIn": true });
    // remaining** down **（）： not is ，cookie down 
    // 「 when this 」 has use —— 1 will up 2 when 1 
    // access ， is before can etc 401 （ `IssuedSession::expires_in`）。
    payload["expiresIn"] = session.expires_in.into();
    if tokens_in_body {
        payload["accessToken"] = session.access_token.into();
        payload["refreshToken"] = session.refresh_token.into();
    }
    let mut response = ok_json(payload);
    // ** 1 ** `Set-Cookie`（ 1 access+refresh）： and in will 
    // 1 ， as `path|domain|max-age|…` cookie， new will not 
    // —— that is 「 in need 」 。 1 no has can for 。
    if let Ok(value) = axum::http::HeaderValue::from_str(&session.session_cookie) {
        response.headers_mut().append(SET_COOKIE, value);
    }
    response
}

/// `GET /api/panel/status` —— use 「」 is 「」。
///
/// public： when use no has 。
pub async fn panel_status() -> Response {
    ok_json(serde_json::json!({
        "registered": access::admin_registered(),
    }))
}

/// `POST /api/panel/setup` —— （ in none when success）。
///
/// need ： 8 。bcrypt （`kv` `panelAdmin`），
/// not 。success will —— use ， not 1 。
pub async fn panel_setup(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Response {
    if access::admin_registered() {
        return management_error(409, " already in ，none need ");
    }
    let payload = parse_body(&body).unwrap_or(serde_json::Value::Null);
    if let Err(response) = check_captcha(&payload, state.local_panel) {
        return response;
    }
    let username = body_str(&payload, "username");
    let password = payload
        .get("password")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if username.is_empty() || username.len() > 64 {
        return management_error(400, "（64 ）");
    }
    if password.len() < 8 {
        return management_error(400, " 8 ");
    }
    let hash = match access::hash_password(password) {
        Ok(hash) => hash,
        Err(error) => {
            logging::log("[Security]", &format!("❌ failure：{error}"));
            return management_error(500, "failure，");
        }
    };
    match access::setup_admin(&username, &hash) {
        Ok(true) => {
            logging::log(
                "[Security]",
                &format!("✅ 「{username}」（{}）", addr.ip()),
            );
            issue_response(access::IssuedSession::new_session(), tokens_in_body(&headers, &query))
        }
        Ok(false) => management_error(409, " already in ，none need "),
        Err(reason) => {
            logging::log("[Security]", &format!("❌ failure：{reason}"));
            management_error(500, &reason)
        }
    }
}

/// `POST /api/panel/login` —— 。
pub async fn panel_login(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Response {
    if !access::admin_registered() {
        return management_error(400, "： in ");
    }
    if access::login_locked(addr.ip()) {
        logging::log("[Security]", &format!("❌ already （{}）", addr.ip()));
        return management_error(429, "failure， 5 minutes after ");
    }
    let payload = parse_body(&body).unwrap_or(serde_json::Value::Null);
    if let Err(response) = check_captcha(&payload, state.local_panel) {
        return response;
    }
    let username = body_str(&payload, "username");
    let password = payload
        .get("password")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if !access::verify_login(&username, password) {
        access::record_login_failure(addr.ip());
        logging::log("[Security]", &format!("❌ failure（{}）", addr.ip()));
        return management_error(401, " or not ");
    }
    access::clear_login_failures(addr.ip());
    // ： in down 「 to no response」 is 
    // 「success 」 1 —— has no has 、
    // ， 1 is in is cookie。
    let body_mode = tokens_in_body(&headers, &query);
    let session = access::IssuedSession::new_session();
    // is this ： down 1 request 「 will invalid」 will ** to **
    // access cookie ， 1 for is " new cookie no up " is " up 
    // server not "—— cookie not this （" access cookie"）。
    logging::log(
        "[Security]",
        &format!(
            "✅ success（{}）response={} ={} ={} will ={}",
            addr.ip(),
            body_mode,
            marker_source(&headers, &query),
            probe_shape(&headers),
            access::access_fingerprint(&session),
        ),
    );
    issue_response(session, body_mode)
}

/// `POST /api/panel/change-password` —— （set「」）。
///
/// body：`{oldPassword, newPassword}`。 need old （ not can 1 already will ：
/// will can can already ）；success after ** its ** will ，current this （
/// `access::change_password`）。
///
/// protected： use must already —— this is 「 already 」，
/// not 「 when 」 （ that is ， docs ）。
pub async fn panel_change_password(headers: HeaderMap, body: Bytes) -> Response {
    if !access::admin_registered() {
        return management_error(400, "");
    }
    let payload = parse_body(&body).unwrap_or(serde_json::Value::Null);
    let old_password = body_str(&payload, "oldPassword");
    let new_password = body_str(&payload, "newPassword");
    if old_password.is_empty() {
        return management_error(400, "current");
    }
    if new_password.len() < 8 {
        return management_error(400, " new 8 ");
    }
    if new_password == old_password {
        return management_error(400, " new not can and current");
    }
    let hash = match access::hash_password(&new_password) {
        Ok(hash) => hash,
        Err(error) => {
            logging::log("[Security]", &format!("❌ failure：{error}"));
            return management_error(500, "failure，");
        }
    };
    match access::change_password(&headers, &old_password, &hash) {
        Ok(true) => {
            logging::log("[Security]", "✅ already （ its will already ）");
            ok_json(serde_json::json!({ "changed": true }))
        }
        Ok(false) => management_error(400, "current not "),
        Err(reason) => {
            logging::log("[Security]", &format!("❌ failure：{reason}"));
            management_error(500, &reason)
        }
    }
}

/// `GET /api/panel/account` —— current（set「」）。
///
/// use and 「 is by 」 this ，** not **。
pub async fn panel_account() -> Response {
    let Some(username) = access::admin_username() else {
        return ok_json(serde_json::json!({ "registered": false }));
    };
    ok_json(serde_json::json!({
        "registered": true,
        "username": username,
        // in when and ； has as （valid）
        "loginEnabled": access::panel_auth_enabled(),
    }))
}

/// `POST /api/panel/refresh` —— use refresh new 。
///
/// before in access （401） after this ，success then request、 use none；
/// refresh （ / ）。
pub async fn panel_refresh(
    headers: HeaderMap,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let Some(session) = access::rotate_session(&headers) else {
        return management_error(401, "expired， new ");
    };
    issue_response(session, tokens_in_body(&headers, &query))
}

/// `POST /api/panel/logout` —— current will （ will cookie ）。
pub async fn panel_logout(headers: HeaderMap) -> Response {
    access::revoke_session(&headers);
    let mut response = ok_json(serde_json::json!({ "loggedIn": false }));
    let clear = access::clear_cookie();
    if let Ok(value) = axum::http::HeaderValue::from_str(&clear) {
        response.headers_mut().append(SET_COOKIE, value);
    }
    response
}

fn management_error(status: i32, message: &str) -> Response {
    errors::management_error(status, message)
}

#[cfg(test)]
mod tokens_in_body_tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderName, HeaderValue};
    use std::collections::HashMap;

    /// （`LAST_ISSUED_PROBE`） is ， will ——
    /// and will （A B ，A validempty false）。
    /// use this 「 + 」 that ； `.await` has is ：
    /// `#[tokio::test]` 1 current_thread when ， will ** 1 
    /// ** up etc ， not in 。
    static PROBE_SLOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn hold_probe_slot() -> std::sync::MutexGuard<'static, ()> {
        PROBE_SLOT_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn headers_with(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        headers
    }

    fn query_with(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// cookie up （ 1 ）。
    fn probe_cookies(value: &str) -> String {
        format!("{PROBE}={value}")
    }

    /// will cookie up ： 1 （ and `access::issue_cookie` ）。
    fn session_cookie_of(session: &access::IssuedSession) -> String {
        format!("{}={}.{}", access::ACCESS_COOKIE, session.access_token, session.refresh_token)
    }

    /// from 1 `Set-Cookie` will cookie （）。
    fn cookie_value_of(set_cookie: &str) -> Option<&str> {
        set_cookie
            .split(';')
            .next()?
            .strip_prefix(&format!("{}=", access::ACCESS_COOKIE))
    }

    // new ：must and `access` that ，
    // otherwise（ cookie no for up ）。
    const PROBE: &str = access::PROBE_COOKIE;

    /// is ** for up that **（，2026-09-29 ）：
    /// · in and etc → cookie ，response and old 1 
    /// （ in HttpOnly cookie ，JS not to ）；
    /// · / is its down old （cookie not ，fnOS :5666 in 
    /// and :3065 use 1 ）→ not ，response。
    /// from before this need ** for up **， as response Set-Cookie；
    /// 1 after ， 1 —— need and for 1 。
    #[test]
    fn the_probe_cookie_must_match_the_recently_issued_value() {
        let _serial = hold_probe_slot();
        remember_issued_probe("fresh-a");

        let headers = headers_with(&[("cookie", probe_cookies("fresh-a").as_str())]);
        assert!(
            !tokens_in_body(&headers, &HashMap::new()),
            " for up when not response（ as ）"
        );

        // （ before that down 、 or 1 down ）→ not 
        let stale = headers_with(&[("cookie", probe_cookies("stale-a").as_str())]);
        assert!(tokens_in_body(&stale, &HashMap::new()));

        // （ up 1 ）： in its in for up ，
        // and `access::cookie_values` 1 
        let repeated = headers_with(&[(
            "cookie",
            format!("{PROBE}=old-a; {PROBE}=fresh-a").as_str(),
        )]);
        assert!(!tokens_in_body(&repeated, &HashMap::new()));
    }

    /// in cookie（ not in ）→ response，** before no 
    /// ** —— can cookie ， not request in 。
    #[test]
    fn a_cookie_stripping_environment_gets_tokens_in_the_body() {
        let _serial = hold_probe_slot();
        remember_issued_probe("a");
        let headers = headers_with(&[]);
        assert!(tokens_in_body(&headers, &HashMap::new()));
    }

    /// before （ + URL parameter）： in request when ，
    /// query up —— request can to ，query 。
    #[test]
    fn the_marker_travels_by_header_or_by_query() {
        let _serial = hold_probe_slot();
        remember_issued_probe("fresh-a");
        let pair = probe_cookies("fresh-a");

        // ****「」： before need response when ，
        // for up （client， can none）。
        let headers = headers_with(&[
            ("x-panel-auth-mode", "body"),
            ("cookie", pair.as_str()),
        ]);
        assert!(tokens_in_body(&headers, &HashMap::new()));
        // parameter： 
        let headers = headers_with(&[("cookie", pair.as_str())]);
        assert!(tokens_in_body(
            &headers,
            &query_with(&[("auth-mode", "body")])
        ));
        // not not ： to valid up —— for up ，
        // is this is 「 cookie」（ and up 1 as for ， that 
        // no " body"）
        assert!(
            !tokens_in_body(&headers, &query_with(&[("auth-mode", "cookie")])),
            "unknown not this ， to "
        );
        // for not up when ， not not 「response」 this default
        let stale = headers_with(&[("cookie", probe_cookies("nope-a").as_str())]);
        assert!(tokens_in_body(
            &stale,
            &query_with(&[("auth-mode", "cookie")])
        ));
    }

    /// （**checkresponse**： Set-Cookie is this 
    /// ， not to ）。
    #[tokio::test]
    // await is ** this ** ：`#[tokio::test]` 1 current_thread
    // when ， will 1 up etc ， not in ；
    // async Mutex need await ， as 1 use not 。
    #[allow(clippy::await_holding_lock)]
    async fn the_probe_endpoint_plants_one_cookie_and_needs_it_back() {
        let _serial = hold_probe_slot();

        // 1 （ no has cookie）：** 1 ** Set-Cookie + response
        let response = cookie_probe(HeaderMap::new(), Query(HashMap::new())).await;
        let planted: Vec<String> = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_string))
            .collect();
        assert_eq!(
            1,
            planted.len(),
            " this down 1 Set-Cookie —— and response； is in that 1 "
        );
        let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("response can ");
        let payload: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(false, payload["data"]["roundtrip"]);
        let first = payload["data"]["token"].as_str().unwrap().to_string();
        assert!(
            planted[0].starts_with(&format!("{PROBE}={first};")),
            " and need down is ， {:?}",
            planted[0]
        );

        // （：）→ roundtrip true
        let headers = headers_with(&[("cookie", probe_cookies(&first).as_str())]);
        let response = cookie_probe(headers, Query(query_with(&[("expect", &first)]))).await;
        let payload: serde_json::Value = payload_of(response).await;
        assert_eq!(true, payload["data"]["roundtrip"]);

        // （ in cookie / is old ）→ false
        let headers = headers_with(&[("cookie", probe_cookies("an-old-value").as_str())]);
        let response = cookie_probe(headers, Query(query_with(&[("expect", &first)]))).await;
        let payload: serde_json::Value = payload_of(response).await;
        assert_eq!(
            false, payload["data"]["roundtrip"],
            " for not up when must not （ after response that ）"
        );

        // no expect（ 1 ） not ： use is down 
        let response = cookie_probe(HeaderMap::new(), Query(HashMap::new())).await;
        let planted: Vec<String> = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_string))
            .collect();
        assert_eq!(1, planted.len(), " 1 ， not ");
    }

    async fn payload_of(response: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("response can ");
        serde_json::from_slice(&bytes).unwrap()
    }

    /// **** not response（2026-09-29 when 
    /// 1 has ）：`web_shim` `?auth-mode=body` none， is 
    /// when 1 will JS can 、 and localStorage ——
    /// 「 in HttpOnly cookie 」 this in 1 after 。
    /// client in when ； this 1 server that 1 ：
    /// ** no has + for up = cookie = response no has **，
    /// new ** 1 ** `Set-Cookie` down （ will ）。
    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn a_direct_session_refreshes_without_putting_tokens_in_the_body() {
        let _serial = hold_probe_slot();
        remember_issued_probe("direct-a");
        let session = access::IssuedSession::new_session();
        let cookie = format!("{PROBE}=direct-a; {}", session_cookie_of(&session));

        let response = panel_refresh(
            headers_with(&[("cookie", cookie.as_str())]),
            Query(HashMap::new()),
        )
        .await;
        assert!(response.status().is_success(), "cookie success");
        let set_cookies: Vec<String> = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_string))
            .collect();
        let payload = payload_of(response).await;
        assert!(
            payload["data"]["accessToken"].is_null(),
            " response not ， {:?}",
            payload["data"]
        );
        assert!(payload["data"]["refreshToken"].is_null());
        assert_eq!(
            1,
            set_cookies.len(),
            " this down 1 Set-Cookie（ will and in ）"
        );

        // for ： 1 （ already ），client**** when mustresponse ——
        // this 1 ， up " no has " can can is 「 no new 」
        let rotated = cookie_value_of(&set_cookies[0]).expect("Set-Cookie has new will for ");
        let marked = format!("{PROBE}=direct-a; {}={rotated}", access::ACCESS_COOKIE);
        let response = panel_refresh(
            headers_with(&[
                ("cookie", marked.as_str()),
                ("x-panel-auth-mode", "body"),
            ]),
            Query(HashMap::new()),
        )
        .await;
        assert!(response.status().is_success());
        let payload = payload_of(response).await;
        assert!(
            payload["data"]["accessToken"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "mustresponse（ in 1 ）"
        );
    }
}
