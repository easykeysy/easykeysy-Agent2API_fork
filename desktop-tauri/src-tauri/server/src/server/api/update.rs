//! update by （ for server.mjs 749-773 ）。
//!
//! GET /api/update/check?current=1.0.0 check new （return new ）
//! POST /api/update/download down { url, name }
//! GET /api/update/progress down 
//! POST /api/update/cancel down 
//! GET /api/update/proxy update（null = ）
//! POST /api/update/proxy update { proxy: null | {…} }
//! GET /api/update/token GitHub { filled, origin }（ not ）
//! POST /api/update/token / GitHub { token: string | null }
//! GET /api/update/beta 「 beta 」 { enabled }
//! POST /api/update/beta 「 beta 」{ enabled: bool }
//! GET /api/update/channels { stable, beta }
//! POST /api/update/channels update { stable?, beta? }
//!
//! ── update（Linux headless / Docker has ）───────────────
//! POST /api/update/systemd systemd 1 down 、、 and { version }
//! POST /api/update/install and use already down update（ and ui）
//! POST /api/update/restart （ by restart ）
//! need `AGENT2API_SELF_UPDATE=1`，otherwise 400； `core::update::selfupdate`。
//! `check` `current` （ not ） when use server —— 1 
//! will （ after not ， that not can ）， has 。
//!
//! ── by ────────────────────────────────────────────────
//! Node is `path.startsWith('/api/update/')` before ： in before after 
//! and path， not then to 404（** not ** `success` 
//! `{"error":{"message":"Not found: <METHOD> <path>"}}`，）。
//! this use 1 `{*rest}` + dispatch： axum by will 
//! `/api/update/zzz` this path to —— that its is 1 404 ，
//! `DELETE /api/update/check` this 「 already path + 」 will 405 ，
//! and Node 404 （ http.rs method_not_allowed ）。
//!
//! ── error ────────────────────────────────────────────────
//! Node in try → failure errorPayload **OpenAI **
//! body， UpdateError （400 parameter/、502 and GitHub ）。
//! ：`{"url":"http://evil.com/a.exe"}` → 400「 down must is https」。
//! after `/releases` use 1 （return UpdateError）。

use axum::body::Bytes;
use axum::extract::State;
use axum::http::Method;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::proxies::{describe_account_proxy, normalize_account_proxy};
use crate::server::core::update::{set_token, status_json, UpdateError};
use crate::server::errors::{management_error, GatewayError};
use crate::server::http::{ok_json, parse_body};
use crate::server::logging;
use crate::server::ServerState;

/// updateerror → response（OpenAI body，）
fn update_error(error: UpdateError) -> Response {
    logging::log("[Update]", &format!("❌ {}", error.message));
    error.to_gateway_error().into_response()
}

/// `/api/update/{*rest}` （any、anypath）。
pub async fn entry(State(state): State<ServerState>, request: axum::extract::Request) -> Response {
    let method = request.method().clone();
    let full_path = request.uri().path().to_string();
    let query = request.uri().query().unwrap_or("").to_string();
    let body = match axum::body::to_bytes(request.into_body(), crate::server::http::MAX_BODY_SIZE)
        .await
    {
        Ok(bytes) => bytes,
        Err(error) => return management_error(413, format!("请求体读取失败或过大: {error}")),
    };
    dispatch(&state, method, &full_path, &query, &body).await
}

/// path（ by has if ）
async fn dispatch(
    state: &ServerState,
    method: Method,
    full_path: &str,
    query: &str,
    body: &Bytes,
) -> Response {
    if method == Method::GET && full_path == "/api/update/policy" {
        let current = query.split('&').filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == "current").map(|(_, value)| percent_decode(value)).unwrap_or_default();
        let current = if current.is_empty() { crate::server::core::update::CURRENT_VERSION } else { &current };
        return ok_json(state.update().refresh_policy(current).await);
    }
    if method == Method::GET && full_path == "/api/update/check" {
        // `url.searchParams.get('current') || ''`： is empty（ not is null）
        let current = query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == "current")
            .map(|(_, value)| percent_decode(value))
            .unwrap_or_default();
        // （web_shim ） not ： no has 「 」 this 
        // ，server as 。， as not 。
        let current = if current.trim().is_empty() {
            crate::server::core::update::CURRENT_VERSION.to_string()
        } else {
            current
        };
        return match state.update().check(&current).await {
            Ok(data) => ok_json(data),
            Err(error) => update_error(error),
        };
    }

    if method == Method::POST && full_path == "/api/update/systemd" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return management_error(400, error.message),
        };
        let version = payload.get("version").and_then(Value::as_str).unwrap_or("");
        return match state.update().start_systemd_update(version) {
            Ok(data) => ok_json(data),
            Err(error) => update_error(error),
        };
    }

    if method == Method::POST && full_path == "/api/update/windows-service" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return management_error(400, error.message),
        };
        let version = payload.get("version").and_then(Value::as_str).unwrap_or("").to_string();
        let updater = state.update().clone();
        return match tokio::task::spawn_blocking(move || updater.start_windows_service_update(&version)).await {
            Ok(Ok(data)) => ok_json(data),
            Ok(Err(error)) => update_error(error),
            Err(error) => update_error(UpdateError::new(format!("Windows 更新任务启动失败：{error}"))),
        };
    }

    if method == Method::GET && full_path == "/api/update/status" {
        // 1 「checkupdate」 （ when and ， before 60 seconds
        // this ， not GitHub —— 60 / when ）。
        // no when is {checked:false}， before by 「unknown」， not 。
        return ok_json(state.update().last_check());
    }

    if method == Method::GET && full_path == "/api/update/github" {
        let mut value = json!(config::github_access());
        value["observations"] = crate::server::core::update::source::status();
        return ok_json(value);
    }

    if method == Method::POST && full_path == "/api/update/github" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::bad_request(error.message).into_response(),
        };
        let patch = match serde_json::from_value::<config::GithubAccessPatch>(payload) {
            Ok(patch) => patch,
            Err(error) => return GatewayError::bad_request(format!("GitHub 访问设置无效：{error}")).into_response(),
        };
        return match config::set_github_access(patch) {
            Ok((settings, saved, changed)) => {
                if changed {
                    state.update().reset_source_preference();
                    crate::server::core::plugin_host::catalog::schedule_now();
                }
                let mut value = json!(settings);
                value["saved"] = json!(saved);
                value["observations"] = crate::server::core::update::source::status();
                ok_json(value)
            }
            Err(message) => GatewayError::bad_request(message).into_response(),
        };
    }

    if method == Method::POST && full_path == "/api/update/download-plan" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::bad_request(error.message).into_response(),
        };
        let Some(url) = payload.get("url").and_then(Value::as_str) else {
            return GatewayError::bad_request("下载方案缺少 url").into_response();
        };
        return match crate::server::core::update::download_plan(url).await {
            Ok(value) => ok_json(value),
            Err(error) => update_error(error),
        };
    }

    if method == Method::GET && full_path == "/api/update/proxy" {
        // 「update」 （setupdate「checkupdate」 down ）。
        // return and ** 1 **（label / config / error）：
        // before listenerUid down 、 label 、 error ——
        // already has ， for not use 。
        return ok_json(json!({
            "proxy": describe_account_proxy(config::update_proxy().as_ref()),
        }));
    }

    if method == Method::POST && full_path == "/api/update/proxy" {
        // ：`{proxy: null}` = ，`{proxy: {source:'clash', listenerUid}}`
        // = Clash 。 1 use normalize_account_proxy
        // （custom supports）， 400 —— ignore will 
        // 「 already 」 no up ，。
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::new(error.message).into_response(),
        };
        let raw = payload.get("proxy").cloned().unwrap_or(Value::Null);
        return match normalize_account_proxy(&raw) {
            // ****（request， `core::proxy_rotate`）：
            // update is 1 check / down ， no has this 。 1 （ that 1 
            // and use ） after must in this down —— otherwise will 1 not 
            // config，checkupdate（ up 「 already 」）。
            Ok(Some(value)) if crate::server::core::proxies::is_rotate_config(&value) => {
                GatewayError::bad_request(
                    "更新出网线路不支持「轮询」档（轮询用于 OpenCode Zen 账号的出网）",
                )
                .into_response()
            }
            Ok(normalized) => {
                // 「」： in current not this （otherwise
                // failure ， down clear_check_cooldown ）
                let changed = config::update_proxy() != normalized;
                let saved = config::set_update_proxy(normalized);
                if changed {
                    state.update().clear_check_cooldown();
                    state.update().reset_source_preference();
                    crate::server::core::plugin_host::update::clear_cooldown();
                }
                let current = config::update_proxy();
                // failurereturn new （ already ， new ），
                // saved up ， use need use 
                ok_json(json!({
                    "proxy": describe_account_proxy(current.as_ref()),
                    "saved": saved,
                }))
            }
            Err(error) => {
                logging::log("[Update]", &format!("❌ 更新出网线路保存失败: {}", error.message));
                GatewayError::bad_request(error.message).into_response()
            }
        };
    }

    if method == Method::GET && full_path == "/api/update/token" {
        // 「updateset」 GitHub 1 ：** has no has 、**，
        // not —— after 「 already 」， this API is that 
        // 1 （ = ）。 and token 。
        return ok_json(status_json());
    }

    if method == Method::POST && full_path == "/api/update/token" {
        // / ：`{token: "ghp_…"}` 、`{token: null}` 。
        // token by requesterror not is 「」—— must null，
        // use `{}` 。 in token::set_token
        // （request）， 400。
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::new(error.message).into_response(),
        };
        let Some(raw) = payload.get("token").cloned() else {
            return GatewayError::bad_request("请求体缺少 token 字段（清除请传 null）").into_response();
        };
        if !raw.is_null() && raw.as_str().is_none() {
            return GatewayError::bad_request("token 必须是字符串或 null").into_response();
        }
        return match set_token(raw.as_str()) {
            Ok(update) => {
                // 「checkupdate」 failure：quota by （ by 
                // IP、 by use ）， old not use ； 1 
                // not （token::stored_token ， TokenUpdate ）
                if update.changed {
                    state.update().clear_check_cooldown();
                    state.update().reset_source_preference();
                    crate::server::core::plugin_host::update::clear_cooldown();
                }
                // failurereturn new （ already ， use ），saved 
                let mut status = status_json();
                status["saved"] = json!(update.saved);
                ok_json(status)
            }
            Err(message) => {
                logging::log("[Update]", &format!("❌ GitHub 令牌保存失败: {message}"));
                GatewayError::bad_request(message).into_response()
            }
        };
    }

    if method == Method::GET && full_path == "/api/update/channels" {
        return ok_json(json!(config::update_channels()));
    }

    if method == Method::POST && full_path == "/api/update/channels" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::bad_request(error.message).into_response(),
        };
        let patch = match serde_json::from_value::<config::UpdateChannelsPatch>(payload) {
            Ok(patch) => patch,
            Err(error) => return GatewayError::bad_request(format!("更新通道设置无效：{error}")).into_response(),
        };
        return save_channels(state, patch, false);
    }

    if method == Method::GET && full_path == "/api/update/beta" {
        return ok_json(json!({ "enabled": config::update_beta() }));
    }

    if method == Method::POST && full_path == "/api/update/beta" {
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::bad_request(error.message).into_response(),
        };
        let Some(enabled) = payload.get("enabled").and_then(Value::as_bool) else {
            return GatewayError::bad_request("请求体缺少 enabled 字段（须为 true / false）")
                .into_response();
        };
        // old beta when ， not can down 。
        let patch = config::UpdateChannelsPatch {
            beta: Some(enabled),
            stable: if enabled { None } else { Some(true) },
        };
        return save_channels(state, patch, true);
    }

    if method == Method::POST && full_path == "/api/update/download" {
        // JSON in Node is JSON.parse → errorPayload 500 proxy_error
        // （ and /api/auto-checkin 1 path）， this 1 
        let payload = match parse_body(body) {
            Ok(value) => value,
            Err(error) => return GatewayError::new(error.message).into_response(),
        };
        let text = |key: &str| -> String {
            payload
                .get(key)
                .filter(|value| !value.is_null())
                .map(js_string)
                .unwrap_or_default()
        };
        return match state.update().start_download(&text("url"), &text("name")) {
            Ok(data) => ok_json(data),
            Err(error) => update_error(error),
        };
    }

    if method == Method::GET && full_path == "/api/update/progress" {
        return ok_json(state.update().get_progress());
    }

    if method == Method::POST && full_path == "/api/update/cancel" {
        return ok_json(state.update().cancel_download());
    }

    // ── update（Linux headless / Docker）────────────────────
    // not request：、， by server
    // （update by from down directory，path not from request —— core::update::mod）。
    // directory ServerState（headless set_ui_dir ），
    // no has （None）—— not will to this 。

    if method == Method::POST && full_path == "/api/update/install" {
        let ui_dir = state.ui_dir().map(|path| path.as_path());
        return match state.update().install_downloaded(ui_dir).await {
            Ok(data) => ok_json(data),
            Err(error) => update_error(error),
        };
    }

    if method == Method::POST && full_path == "/api/update/restart" {
        return match state.update().request_restart() {
            Ok(data) => ok_json(data),
            Err(error) => update_error(error),
        };
    }

    // in 1 → 404 （Node to `sendJson(res, 404, …)`），
    // Note this body ** no has success **
    crate::server::errors::not_found_response(method.as_str(), full_path)
}

fn save_channels(
    state: &ServerState,
    patch: config::UpdateChannelsPatch,
    legacy: bool,
) -> Response {
    let plugins_before = crate::server::core::plugin_host::settings::current();
    match config::set_update_channels(patch) {
        Ok((channels, saved, changed)) => {
            if changed {
                state.update().invalidate_channel();
                crate::server::core::plugin_host::settings::notify_changed(plugins_before);
            }
            let mut result = if legacy { json!({ "enabled": channels.beta }) } else { json!(channels) };
            result["saved"] = json!(saved);
            ok_json(result)
        }
        Err(message) => GatewayError::bad_request(message).into_response(),
    }
}

/// JS `String(value)`：（ request name can can is ）。
/// for / will request， this to JSON can ——
/// will safe_file_name file。
fn js_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// （`current=1.0.0-beta%2B1` this ）。
/// failure when return（ not is error， to ）。
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3])
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok());
                match hex {
                    Some(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    None => {
                        out.push(b'%');
                        index += 1;
                    }
                }
            }
            // `+` in empty（ and URLSearchParams 1 ）
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}
