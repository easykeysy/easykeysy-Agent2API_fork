//! to 。
//!
//! to ：「HTTP + 」， not ——
//! and Electron 1 （ 1 in after HTTP ， 1 then ）。
//! before can `api_request(method, path, body)` use ，
//! path and in in bridge.js ， for 。

use serde::Deserialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_dialog::DialogExt;

use crate::backend;
use crate::gateway;
use crate::login::{self, LoginState};
use crate::settings::{self, AppSettings};
use crate::state::AppState;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub body: Option<Value>,
    #[serde(default)]
    pub api_key: Option<String>,
}

/// 1 API ：return after data， when return can 。
#[tauri::command]
pub async fn api_request(request: ApiRequest) -> Result<Value, String> {
    match request.api_key.as_deref() {
        Some(key) => gateway::call_with_api_key(&request.method, &request.path, request.body.as_ref(), key).await,
        None => gateway::call(&request.method, &request.path, request.body.as_ref()).await,
    }
}

/// （ use ）
#[tauri::command]
pub async fn api_request_text(method: String, path: String) -> Result<String, String> {
    gateway::call_text(&method, &path).await
}

/// after ： can in 。
///
/// return not is 「 and 」， has **Why no ** —— 
/// （、 use 、 can ） in `failure` 。
/// is 「 in 」 is to 「 use / 」 。
///
/// can not is 1 ： can can in before 
/// （ in ）， that when use can to 1 ， not 。
#[tauri::command]
pub async fn backend_status(app: AppHandle) -> Result<Value, String> {
    let port = gateway::proxy_port();
    let phase = crate::startup::phase();
    let ready = phase == "ready" && backend::is_ready(port).await;
    // failure in AppState （ by ensure_ready failure）；
    // already （） when ， not this 「」——
    // set not will ， will use 1 
    let failure = app
        .state::<AppState>()
        .backend
        .lock()
        .ok()
        .and_then(|guard| guard.failure.clone());
    Ok(json!({
        "ready": ready,
        "phase": phase,
        "port": port,
        "portFromEnv": gateway::port_from_env().is_some(),
        "failure": failure,
    }))
}

#[tauri::command]
pub async fn wait_for_backend() -> Result<Value, String> {
    crate::startup::wait().await?;
    Ok(json!({ "ready": true, "port": gateway::proxy_port() }))
}

/// use 。
///
/// ── Why is ， not is 「 after 」 ──────────────
/// server has 1 ****（directory、 when to 、update）， use 
/// `OnceLock` ： `ServerState::bootstrap` not ， is new 
/// ServerState will 1 and not —— when `stop_global()` is 
/// old that ， new no 。all from ， not in this 
/// ； is will new （ 1 seconds）， is 「 after 1 is for 」。
///
/// ── up need ────────────────────────────────────────
/// 1. **must `begin_exit()`**：otherwise `RunEvent::ExitRequested` will 
/// 「 to 」 that then down （`api.prevent_exit()`），requestempty；
/// 2. ****： return to before ， can 「 in 」
/// not is none。
pub fn schedule_app_restart(app: &AppHandle) {
    app.state::<AppState>().begin_exit();
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        // `restart()` ExitRequested → Exit（ in ）
        // → new → 。 not return。
        handle.restart();
    });
}

/// use （ in 「 use 」 after ， or use ）。
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<Value, String> {
    let port = gateway::proxy_port();
    schedule_app_restart(&app);
    Ok(json!({ "restarting": true, "port": port }))
}

/// use 。
///
/// no use （ or not to ，e.g.） when `occupant` as null ——
/// to 「」， not 1 failure 「」。
#[tauri::command]
pub async fn port_occupant() -> Result<Value, String> {
    let port = gateway::proxy_port();
    let occupant = backend::occupant_of(port);
    Ok(json!({ "port": port, "occupant": occupant }))
}

/// use ， and in 。
///
/// `port_occupant` to that PID（ not by none：
/// in when ，none will none）。
/// before by and path to use 。
///
/// success after ** not **： use （） need need to ， is 
/// new （`restart_app`）。
#[tauri::command]
pub async fn end_port_occupant(app: AppHandle) -> Result<Value, String> {
    let port = gateway::proxy_port();
    let Some(occupant) = backend::occupant_of(port) else {
        return Err(format!(
            "端口 {port} 上没有查到监听进程，无法结束。\
             若该端口属于系统保留段，请改用「更换端口」。"
        ));
    };
    // is （OpenProcess + TerminateProcess + etc ），
    // to ， when worker
    let result = tauri::async_runtime::spawn_blocking(move || {
        let killed = backend::end_occupant(port, &occupant);
        // none is success 1 ：success need ，
        // failure need use 「 in up is 」
        let release = backend::wait_port_released(port);
        (killed, release)
    })
    .await
    .map_err(|error| format!("结束进程任务失败: {error}"))?;

    let (killed, release) = result;
    // failure（ not / is / is ） to 
    killed.map_err(|conflict| conflict.message)?;
    let released = release.is_ok();
    if released {
        // already ：**failure**。 that is 「 when 」，
        // in use already not in ， will 1 「 use 」——
        // use ， to this will as no 。
        // after to 「 in …」，「empty、 no 」。
        if let Ok(mut guard) = app.state::<AppState>().backend.lock() {
            guard.failure = None;
        }
    }
    Ok(json!({ "port": port, "released": released }))
}

/// 1 can used for（「」 in before ）。
///
/// use bind ， not is netsh ： has 
/// can up ， not not need （ port_conflict module header）。
/// return `{ ok, message, conflict }` —— not ， as 「unavailable」 is 
/// 1 use failure， need in 。
#[tauri::command]
pub async fn check_port(port: u16) -> Result<Value, String> {
    if port == 0 {
        return Ok(json!({ "ok": false, "message": "端口号必须在 1-65535 之间" }));
    }
    // 1024 down in Windows up need need ， can 
    // 1 「success after not 」 
    if port < 1024 {
        return Ok(json!({
            "ok": false,
            "message": "端口 1-1023 是系统保留范围，普通程序无法监听，请用 1024 以上的端口",
        }));
    }
    let current = gateway::proxy_port();
    if port == current {
        return Ok(json!({ "ok": true, "message": "与当前端口相同，无需重启", "same": true }));
    }
    match crate::server::probe_port(port) {
        Ok(()) => Ok(json!({ "ok": true, "message": format!("端口 {port} 可用") })),
        Err(conflict) => Ok(json!({
            "ok": false,
            "message": conflict.message.clone(),
            "conflict": conflict,
        })),
    }
}

/// ： → set → use 。
///
/// in before ， is as 「unavailable」 in setfile —— otherwisefile
/// will down 1 not ， down not 。
///
/// in server bind after not ，must（ `schedule_app_restart`）。
#[tauri::command]
pub async fn change_port(app: AppHandle, port: u16) -> Result<Value, String> {
    if port < 1024 {
        return Err("端口 1-1023 是系统保留范围，请用 1024 以上的端口".to_string());
    }
    // when ，setfile not will —— and its failure，
    // not use this 
    if gateway::port_from_env().is_some() {
        return Err(
            "当前端口由环境变量 AGENT2API_PROXY_PORT 指定，设置里的端口不会生效。\
             请修改该环境变量后重启程序。"
                .to_string(),
        );
    }
    if port == gateway::proxy_port() {
        return Ok(json!({ "changed": false, "port": port }));
    }
    // bind ：unavailable（ check_port ）
    if let Err(conflict) = crate::server::probe_port(port) {
        return Err(conflict.message);
    }

    let mut current = settings::load();
    current.proxy_port = port;
    settings::save(&current)?;

    // after new 。 not in this etc ： will ，
    // （） new is available。
    schedule_app_restart(&app);
    Ok(json!({ "changed": true, "port": port, "restarting": true }))
}

// ── （issue #48）────────────────────────────────────
// set 「」：、、
// 。 and its 「」 to has 1 ：
// server （`access::setup_admin`） not /api/panel/setup —— that HTTP path
// headless ， ALTCHA and ； is 
// ， in ** before **，path not 。
// by `access::hash_password` ， 。

/// is already （set「」 need not need ）。
#[tauri::command]
pub async fn panel_admin_status() -> Result<Value, String> {
    crate::startup::wait().await?;
    Ok(json!({ "registered": crate::server::access::admin_registered() }))
}

/// （ path， up ）。
///
/// already has when return `existed: true` not is ：「 has no has 
/// 」，（ when ） not this use to failure。
#[tauri::command]
pub async fn panel_register(username: String, password: String) -> Result<Value, String> {
    crate::startup::wait().await?;
    let username = username.trim();
    if username.is_empty() || username.chars().count() > 64 {
        return Err("请填写管理员账号（64 字符以内）".to_string());
    }
    if password.len() < 8 {
        return Err("密码至少 8 位".to_string());
    }
    let username = username.to_string();
    // bcrypt is ，， worker
    let hash = tauri::async_runtime::spawn_blocking(move || {
        crate::server::access::hash_password(&password)
    })
    .await
    .map_err(|error| format!("注册任务失败: {error}"))??;

    match crate::server::access::setup_admin(&username, &hash) {
        Ok(true) => {
            crate::server::logging::log(
                "[Security]",
                &format!("✅ 管理员「{username}」注册完成（桌面端）"),
            );
            Ok(json!({ "registered": true }))
        }
        Ok(false) => Ok(json!({ "registered": true, "existed": true })),
        Err(reason) => Err(reason),
    }
}

// ── will ────────────────────────────────────────────
// use will ； not is need need 。

/// after and will after return；failure up ， not can 。
#[tauri::command]
pub async fn panel_session_status() -> Result<Value, String> {
    crate::startup::wait().await?;
    crate::gateway::call("GET", "/api/session", None).await?;
    Ok(json!({
        "registered": crate::server::access::admin_registered(),
        "authenticated": true,
    }))
}

/// old ； not as 。
#[tauri::command]
pub async fn panel_session_store(
    access_token: String,
    refresh_token: String,
    expires_in: Option<i64>,
) -> Result<Value, String> {
    crate::startup::wait().await?;
    if access_token.is_empty() || refresh_token.is_empty() {
        return Err("令牌为空".to_string());
    }
    let _ = expires_in;
    crate::panel_session::ensure_local_access()?;
    Ok(json!({ "saved": true }))
}

/// will ； down 1 will new will 。
#[tauri::command]
pub async fn panel_session_clear() -> Result<Value, String> {
    crate::startup::wait().await?;
    crate::panel_session::clear();
    Ok(json!({ "cleared": true }))
}

/// ： →（ need need when Key）→ set → use 。
///
/// ── before must has ──────────────────────────────────
/// 监听一出回环，`/api/*` is 「 will or API Key」； no has 
/// when this not in （ ）。 is 「、
/// after 」， this 1 。
///
/// ── Why in when 1 default Key ────────────────────────────
/// use mustenabled Key；not configured when 1 and use 。
/// use will ， not this Key。
///
/// enabled [`schedule_app_restart`]： in bind after not ，
/// and `change_port` 1 。
#[tauri::command]
pub async fn change_lan_access(app: AppHandle, enabled: bool, panel: bool) -> Result<Value, String> {
    crate::startup::wait().await?;
    if enabled && !crate::server::access::admin_registered() {
        return Err("：".to_string());
    }
    // in when has ； when 1 and 
    let panel = enabled && panel;

    let mut created_key = None;
    if enabled && crate::server::config::current().active_api_keys().is_empty() {
        let entry = crate::server::core::api_keys::add(
            "default",
            None,
            Vec::new(),
            Vec::new(),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .map_err(|message| format!("create Key failure: {message}"))?;
        crate::server::logging::log(
            "[Config]",
            &format!("✅ already create Key「{}」（）", entry.name),
        );
        created_key = Some(entry.public_json());
    }

    let mut current = settings::load();
    current.lan_access = enabled;
    current.lan_panel = panel;
    settings::save(&current)?;

    schedule_app_restart(&app);
    Ok(json!({
        "changed": true,
        "enabled": enabled,
        "panel": panel,
        "restarting": true,
        "createdKey": created_key,
    }))
}

/// when ； use must and Key。
#[tauri::command]
pub async fn direct_credentials() -> Result<Value, String> {
    crate::startup::wait().await?;
    Ok(Value::Object(crate::panel_session::credential_headers()?))
}

/// in （ to 「」 `http://<IP>:<>/v1` use ）。
///
/// UDP connect not ， is by by default —— 
/// has when is 「 can can to that 」。 not to （ no has / has 
/// ）return null： this not is use failure， by 「 not to 」， not error。
#[tauri::command]
pub fn local_ip() -> Result<Option<String>, String> {
    let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(_) => return Ok(None),
    };
    if socket.connect("8.8.8.8:80").is_err() {
        return Ok(None);
    }
    Ok(socket
        .local_addr()
        .ok()
        .map(|addr| addr.ip().to_string())
        .filter(|ip| ip != "0.0.0.0" && ip != "127.0.0.1"))
}

/// ； to /failure// when 。
///
/// `provider` is Option： not will ，（None） by workbuddy ——
/// that as must。`Option<String>` in Tauri is 「 can not 」，
/// to 1 emptydefault「client no has this 」。
///
/// `social_restore` as false（ not Google / GitHub ），
/// not when to is and new default not 1 as ： by 
/// ， not 。
///
/// ── AutoClaw Why not this ────────────────────────
/// need 1 ****（ SDK）， that is 
/// —— is 「」 this 1 in ，。
/// [`start_autoclaw_oauth_login`]（ is already to state + authUrl），
/// not this 。
/// `options` is add optional（methodIndex / inputs / proxy / manualCallback），
/// not social_restore。 no has methodIndex when old 。
#[tauri::command]
pub async fn start_login(
    app: AppHandle,
    edition: String,
    mode: String,
    provider: Option<String>,
    social_restore: Option<bool>,
    options: Option<Value>,
) -> Result<Value, String> {
    login::start_with_options(
        &app,
        edition,
        mode,
        provider.unwrap_or_default(),
        social_restore.unwrap_or(false),
        options,
    )
    .await
}

/// AutoClaw OAuth ：** already by before **， this and etc 。
///
/// `state` and `authUrl` `POST /api/session/login/oauth/start`
/// （ before after that ）。 by `login::start_autoclaw_oauth`：
/// can in （）， and ——
/// before 、， not is after need 。
///
/// `mode`：`embedded`（， WebView） or `external`（）——
/// available， as by to loopback ，
/// and in none（ and CatPaw ）。
#[tauri::command]
pub async fn start_autoclaw_oauth_login(
    app: AppHandle,
    state: String,
    auth_url: String,
    mode: Option<String>,
) -> Result<Value, String> {
    login::start_autoclaw_oauth(
        &app,
        state,
        auth_url,
        &mode.unwrap_or_else(|| "embedded".to_string()),
    )
    .await
}

/// KukuAI · ：（） after return。
///
/// and its （AutoClaw / Loomy） not ： 「」must in 
/// （request need up ， `login::kuku_sms_script` ），
/// this ** not is ** after API， is 1 use 。
/// before to return `state` after 「 etc 」， use after 
/// [`login_kuku_sms_verify`]。
#[tauri::command]
pub async fn login_kuku_sms_send(
    app: AppHandle,
    phone: String,
) -> Result<Value, String> {
    login::start_kuku_sms(&app, &phone).await
}

/// KukuAI · ： use to ， etc 。
///
/// success after return `{account, list}`（ and `POST /api/accounts` response，
/// before 「 already 」）。 in when already ，
/// this need need 。
#[tauri::command]
pub async fn login_kuku_sms_verify(
    app: AppHandle,
    code: String,
) -> Result<Value, String> {
    login::submit_kuku_sms_code(&app, &code).await
}

#[tauri::command]
pub fn login_state(app: AppHandle) -> LoginState {
    match login::current_login(&app) {
        Some(active) => LoginState {
            active: true,
            mode: Some(active.mode),
            edition: Some(active.edition),
            provider: Some(active.provider),
        },
        None => LoginState { active: false, mode: None, edition: None, provider: None },
    }
}

#[tauri::command]
pub async fn cancel_login(app: AppHandle) -> Result<Value, String> {
    login::cancel(&app).await?;
    Ok(json!({ "canceled": true }))
}

/// ： JSONL ，。
#[tauri::command]
pub async fn export_logs(app: AppHandle, query: Option<String>) -> Result<Value, String> {
    let query = query.unwrap_or_default();
    let query = query.trim_start_matches('?');
    let path = if query.is_empty() { "/api/logs/download".to_string() } else { format!("/api/logs/download?{query}") };
    let text = gateway::call_text("GET", &path).await?;
    let count = text.lines().filter(|line| !line.trim().is_empty()).count();
    if count == 0 {
        return Ok(json!({ "count": 0 }));
    }

    let stamp = timestamp_for_filename();
    let default_name = format!("workbuddy-logs-{stamp}.jsonl");
    let file = tauri_plugin_dialog::DialogExt::dialog(&app)
        .file()
        .set_title("")
        .set_file_name(&default_name)
        .add_filter("JSON Lines", &["jsonl"])
        .add_filter("file", &["*"])
        .blocking_save_file();

    let Some(target) = file else {
        return Ok(json!({ "canceled": true, "count": count }));
    };
    let path = target
        .into_path()
        .map_err(|error| format!("pathinvalid: {error}"))?;
    std::fs::write(&path, text.as_bytes()).map_err(|error| format!("filefailure: {error}"))?;
    Ok(json!({ "count": count, "file": path.to_string_lossy() }))
}

/// file，return in for path（ → `{canceled: true}`）。
///
/// before has 「 down 」 「directory…」 use 。**， not **：
/// directory can not in （ use can in 1 new path），directory is 
/// down when after `create_dir_all` —— in this ，「 after no down 」
/// will down 1 emptydirectory。
///
/// `defaultDir` to for 1 （ is currentdirectory）， not in when ignore。
#[tauri::command]
pub async fn pick_folder(app: AppHandle, default_dir: Option<String>) -> Result<Value, String> {
    let mut builder = tauri_plugin_dialog::DialogExt::dialog(&app)
        .file()
        .set_title("file");
    if let Some(dir) = default_dir.filter(|text| !text.trim().is_empty()) {
        let path = std::path::PathBuf::from(dir);
        if path.is_dir() {
            builder = builder.set_directory(path);
        }
    }
    let Some(folder) = builder.blocking_pick_folder() else {
        return Ok(json!({ "canceled": true }));
    };
    let path = folder
        .into_path()
        .map_err(|error| format!("filepathinvalid: {error}"))?;
    Ok(json!({ "canceled": false, "path": path.to_string_lossy() }))
}

/// use set。
///
/// `autostart` as ， not is setfile ：
/// use can can in 「」 use use ，
/// that down configfile is true， will 「 already 」， and not 。
#[tauri::command]
pub async fn get_app_settings(app: AppHandle) -> Result<AppSettings, String> {
    crate::startup::wait().await?;
    let mut current = settings::load();
    if let Ok(enabled) = app.autolaunch().is_enabled() {
        current.autostart = enabled;
    }
    // and for ： when need use to new 
    app.state::<AppState>().window.set_close_to_tray(current.close_to_tray);
    app.state::<AppState>().window.set_lightweight(current.lightweight_mode);
    Ok(current)
}

/// use set，return after 。
///
/// `autostart` when ；return `autostart` 
/// ，「 already no 」。
///
/// ── Why `proxyPort` need from up ──────────────────────
/// use （set）「 to 」 and 「」 ，
/// patch no has —— is ****。
/// if ，`proxyPort` will 0（= set）， use 
/// 「」 ， down to 3065 up 。
/// not this ， up 。
#[tauri::command]
pub async fn save_app_settings(app: AppHandle, patch: AppSettings) -> Result<AppSettings, String> {
    crate::startup::wait().await?;
    let mut saved = patch;
    // by change_port ， this not 
    saved.proxy_port = settings::load().proxy_port;

    let autolaunch = app.autolaunch();
    let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
    if saved.autostart != currently_enabled {
        let result = if saved.autostart {
            autolaunch.enable()
        } else {
            autolaunch.disable()
        };
        if let Err(error) = result {
            return Err(format!("updatefailure: {error}"));
        }
        // ：success down when ， this can 
        saved.autostart = autolaunch.is_enabled().unwrap_or(saved.autostart);
    }

    settings::save(&saved)?;
    // ：config not use ， down 1 new as 
    app.state::<AppState>().window.set_close_to_tray(saved.close_to_tray);
    app.state::<AppState>().window.set_lightweight(saved.lightweight_mode);
    Ok(saved)
}

/// ：，。
///
/// and ** as 0** when return， not —— use 
/// path 「 no can 」 is 。 has no has （ or ）：
/// file in when （v2 `customProviders` ）。
#[tauri::command]
pub async fn export_accounts(app: AppHandle) -> Result<Value, String> {
    let data = gateway::call("GET", "/api/accounts/export", None).await?;
    let accounts = data
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let custom_providers = data
        .get("customProviders")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if accounts.is_empty() && custom_providers.is_empty() {
        return Ok(json!({ "count": 0 }));
    }

    let stamp = timestamp_for_filename();
    let default_name = format!("workbuddy-accounts-{stamp}.json");
    let file = app
        .dialog()
        .file()
        .set_title("")
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .add_filter("file", &["*"])
        .blocking_save_file();

    let Some(target) = file else {
        return Ok(json!({ "canceled": true }));
    };
    let path = target
        .into_path()
        .map_err(|error| format!("pathinvalid: {error}"))?;
    let text = serde_json::to_string_pretty(&data)
        .map_err(|error| format!("failure: {error}"))?;
    std::fs::write(&path, text.as_bytes()).map_err(|error| format!("filefailure: {error}"))?;
    Ok(json!({
        "count": accounts.len(),
        "customProviders": custom_providers.len(),
        "file": path.to_string_lossy(),
    }))
}

/// from file（merge ： by ， in update、 in ）。
///
/// file `{ version, exportedAt, customProviders, accounts }`、
/// for and `[...]`。`customProviders` （v2 ）
/// to after —— provider to this ，
/// not 。
#[tauri::command]
pub async fn import_accounts(app: AppHandle) -> Result<Value, String> {
    let file = app
        .dialog()
        .file()
        .set_title("")
        .add_filter("JSON", &["json"])
        .add_filter("file", &["*"])
        .blocking_pick_file();

    let Some(target) = file else {
        return Ok(json!({ "canceled": true }));
    };
    let path = target
        .into_path()
        .map_err(|error| format!("filepathinvalid: {error}"))?;
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("filefailure: {error}"))?;

    let parsed: Value = serde_json::from_str(&text)
        .map_err(|error| format!("file not is valid JSON: {error}"))?;
    let document = unwrap_envelope_file(parsed);
    let accounts = match &document {
        Value::Object(map) if map.contains_key("accounts") => map
            .get("accounts")
            .and_then(Value::as_array)
            .cloned()
            .ok_or("file in accounts ")?,
        Value::Object(map) if map.get("id").and_then(Value::as_str)
            .is_some_and(|id| !id.trim().is_empty()) => vec![document.clone()],
        Value::Array(items) => items.clone(),
        _ => return Err("file not is file、file， not is ".to_string()),
    };

    let mut payload = json!({ "accounts": accounts, "mode": "merge" });
    if let Value::Object(map) = &document {
        if let Some(definitions) = map.get("customProviders") {
            payload["customProviders"] = definitions.clone();
        }
    }

    gateway::call("POST", "/api/accounts/import", Some(&payload)).await
}

/// old 「」 API `{ success, data }` response 
/// file： no has accounts data has when down 1 ， old file not 。
/// file（ is accounts） and 。
fn unwrap_envelope_file(parsed: Value) -> Value {
    let Value::Object(map) = &parsed else {
        return parsed;
    };
    if map.contains_key("accounts") {
        return parsed;
    }
    match map.get("data") {
        Some(data) if data.get("accounts").is_some() => data.clone(),
        _ => parsed,
    }
}

/// ： use path， after 。
///
/// and （`export_accounts`） not ：** no has 「empty not 」** —— 
/// down kv has config，empty。file not in ，
/// after by path（ MB not ， `api/backup_api`）。
#[tauri::command]
pub async fn export_database(
    app: AppHandle,
    #[allow(non_snake_case)] includeLogs: Option<bool>,
    password: String,
) -> Result<Value, String> {
    if password.chars().count() < 12 || password.len() > 1024 {
        return Err(" need need 12 ， and not can 1024 ".into());
    }
    let stamp = timestamp_for_filename();
    let default_name = format!("agent2api-backup-{stamp}.a2ab");
    let file = app
        .dialog()
        .file()
        .set_title("")
        .set_file_name(&default_name)
        .add_filter("Agent2API ", &["a2ab"])
        .blocking_save_file();
    let Some(target) = file else {
        return Ok(json!({ "canceled": true }));
    };
    let path = target
        .into_path()
        .map_err(|error| format!("pathinvalid: {error}"))?;
    let payload = json!({
        "filePath": path.to_string_lossy(),
        // before not （ old ） when by 
        "includeLogs": includeLogs.unwrap_or(true),
        "password": password,
    });
    let mut data = gateway::call_with_timeout(
        "POST",
        "/api/backup/export",
        Some(&payload),
        BACKUP_TRANSFER_TIMEOUT_MS,
    )
    .await?;
    if let Value::Object(map) = &mut data {
        map.insert("file".to_string(), Value::String(path.to_string_lossy().to_string()));
    }
    Ok(data)
}

/// / request：30 minutes。
///
/// 60 seconds use for this not —— need SQL、 in up 、
/// 、、、； MB in up minutes
/// （ 1 61 seconds ， after its in ）。** when that 1 must
/// is after **：failure will 1 Reason 500， before will use to 
/// 「 when 」， as no 。
const BACKUP_TRANSFER_TIMEOUT_MS: u64 = 30 * 60 * 1000;

/// ： or old SQL， after after 。
///
/// before by before （「 will to 
/// backups/ directory」「 after need new 」）； this file and 。
#[tauri::command]
pub async fn import_database(app: AppHandle, password: Option<String>) -> Result<Value, String> {
    let file = app
        .dialog()
        .file()
        .set_title("")
        .add_filter("Agent2API ", &["a2ab", "sql"])
        .add_filter("file", &["*"])
        .blocking_pick_file();
    let Some(target) = file else {
        return Ok(json!({ "canceled": true }));
    };
    let path = target
        .into_path()
        .map_err(|error| format!("filepathinvalid: {error}"))?;
    let payload = json!({ "filePath": path.to_string_lossy(), "password": password.unwrap_or_default() });
    // to 30 minutes，request can can after when （ / ）。
    // that when ** not can **failure： after old in ， in 。 when after 1 
    // —— ，「 in …」，
    // not is 1 「failure： when 」。
    match gateway::call_with_timeout(
        "POST",
        "/api/backup/import",
        Some(&payload),
        BACKUP_TRANSFER_TIMEOUT_MS,
    )
    .await
    {
        Ok(data) => Ok(data),
        Err(message) if message.contains(" when ") => {
            let progress = gateway::call("GET", "/api/backup/progress", None).await;
            match progress {
                Ok(snapshot) if snapshot.get("active").and_then(Value::as_bool) == Some(true) => {
                    // after in ： error 1 「 in 」， etc 
                    Err(" in in ， etc ".to_string())
                }
                Ok(snapshot) => {
                    if snapshot.get("done").and_then(Value::as_bool) == Some(true) {
                        if let Some(result) = snapshot.get("result") {
                            return Ok(result.clone());
                        }
                    }
                    let failed = snapshot.get("failed").and_then(Value::as_bool) == Some(true);
                    let detail = snapshot
                        .get("error")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if failed && !detail.is_empty() {
                        Err(detail)
                    } else {
                        Err(message)
                    }
                }
                Err(_) => Err(message),
            }
        }
        Err(message) => Err(message),
    }
}

/// up / down request。
///
/// 60 seconds is API use when ， this need MB： / use up 
/// up 60 seconds not ， before when after its in （ use to 「 when 」，
/// can can already 1 ）。300 seconds and after when （`cloud_sync::webdav`
/// `TRANSFER_TIMEOUT_SECS`） for —— 1 ， to not will 
/// 「failure、 after in 」 。
const CLOUD_SYNC_TIMEOUT_MS: u64 = 300_000;

/// up （set「 → 」 「 up 」）。
#[tauri::command]
pub async fn cloud_sync_upload() -> Result<Value, String> {
    gateway::call_with_timeout(
        "POST",
        "/api/cloud-sync/upload",
        Some(&json!({})),
        CLOUD_SYNC_TIMEOUT_MS,
    )
    .await
}

/// down （ down success will ）。
#[tauri::command]
pub async fn cloud_sync_download() -> Result<Value, String> {
    gateway::call_with_timeout(
        "POST",
        "/api/cloud-sync/download",
        Some(&json!({})),
        CLOUD_SYNC_TIMEOUT_MS,
    )
    .await
}

/// use and to 。
///
/// not is use `call`： as parameter in Tauri IPC ， not 
/// HTTP request（ 1 can can in ）。
#[tauri::command]
pub async fn cloud_sync_import_key(password: String) -> Result<Value, String> {
    let payload = json!({ "password": password });
    gateway::call("POST", "/api/cloud-sync/import-key", Some(&payload)).await
}

/// check new ： use as query parameter to after 。
///
/// must by —— after ， not ，
/// after can 「 new is 」，none「 is has update」。
/// use when package_info（config ）， not is ：
/// in Cargo.toml and tauri.conf.json has 1 ， before can can and not 1 。
#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Value, String> {
    let current = app.package_info().version.to_string();
    let path = format!("/api/update/check?current={}", urlencoding(&current));
    gateway::call("GET", &path, None).await
}

/// set「」： after that + has 。
///
/// ── Why not `/api/system-info` ──────────────────
/// Tauri 、 in `127.0.0.1:{port}`：， and 
/// after fetch will 401 down （ and `direct_credentials` that 
/// is 1 ）。 by `gateway::request_builder` ，
/// （ / no ） not use 。
///
/// ── ────────────────────────────────────────────
/// · `app.version`： when `package_info` after —— and 
/// `check_update` 1 （ in Cargo.toml and tauri.conf.json 1 ，
/// to up that tauri.conf.json as ）；
/// · `runtime.webviewVersion`：WebView2 / WKWebView 。、
/// CSS not 、`structuredClone` API when ， this is 1 need ；
/// not to （Linux up no when etc ） to empty，「—」；
/// · `runtime.debugBuild`： after that is **server crate** ，
/// use is —— as （`tauri dev` and is 1 
/// server crate， 1 `[Dev]` 1 not ）。
#[tauri::command]
pub async fn system_info(app: AppHandle, hide_username: Option<bool>) -> Result<Value, String> {
    let path = if hide_username.unwrap_or(false) {
        "/api/system-info?hideUsername=1"
    } else {
        "/api/system-info"
    };
    let mut info = gateway::call("GET", path, None).await?;

    // 「 has 」：（ after / response ） when after that ，
    // not as 1 for failure。
    if let Some(object) = info.as_object_mut() {
        if let Some(app_section) = object.get_mut("app").and_then(Value::as_object_mut) {
            app_section.insert(
                "version".to_string(),
                Value::String(app.package_info().version.to_string()),
            );
        }
        if let Some(runtime) = object.get_mut("runtime").and_then(Value::as_object_mut) {
            // wry webview_version in when available；Linux webkit2gtk etc 
            // down will failure —— failure to empty， not 。
            let webview = tauri::webview_version().unwrap_or_default();
            runtime.insert("webviewVersion".to_string(), Value::String(webview));
            runtime.insert("debugBuild".to_string(), Value::Bool(cfg!(debug_assertions)));
        }
    }
    Ok(info)
}

/// down （ after and ， this parameter）
#[tauri::command]
pub async fn download_update(url: String, name: Option<String>) -> Result<Value, String> {
    let payload = json!({ "url": url, "name": name.unwrap_or_default() });
    gateway::call("POST", "/api/update/download", Some(&payload)).await
}

/// down （ before ）
#[tauri::command]
pub async fn update_progress() -> Result<Value, String> {
    gateway::call("GET", "/api/update/progress", None).await
}

/// down 
#[tauri::command]
pub async fn cancel_update() -> Result<Value, String> {
    gateway::call("POST", "/api/update/cancel", Some(&json!({}))).await
}

/// already down 。
///
/// pathmust `update::verify_installer` （ in 、 after 、
/// down directory），otherwise this 「any」 。
///
/// `restart` as true when ： ， need file use 
/// （ not will 「 to 」， will in file use up ）。
///
/// ：must in before after 。
/// and not will node （`exit(0)` not `RunEvent::Exit`），
/// NSIS when node.exe can file and 3065 ，filefailure；
/// `backend::shutdown` is kill + wait，return when use already 。
///
/// ── macOS up Why not （ and Windows ）────────────────
/// Windows NSIS is ****： need in that directoryfile，
/// must and after ，otherwisefilefailure。
///
/// macOS dmg is ** + **：`open` is up ， in 
/// use .app 「 use 」 that 1 ， and current in no has file。
/// when if Windows that ， use —— use no ，
/// already （ and dmg after use will 1 ），
/// etc use 1 no has need in 。 this 1 not 、 not ，
/// up ， and `restart: false` to （ not 「 will 」）。
///
/// ── Linux up Why not （ and macOS ， not ）──────
/// `.deb` to （GNOME Software / Discover）： use 「」
/// after by directoryfile —— and current** not in 1 path**（ in 
/// `~/.local/bin` or AppImage directory this use directory）， no has file use ，
/// and when by use 。 this 1 not after 、 not 。
///
/// `.AppImage` is ： is file，update is 「 use new old 」，
/// old file。 need 「、 new 」 that 
/// （ and Windows 1 ）。
#[tauri::command]
pub fn run_installer(app: AppHandle, path: String, restart: Option<bool>) -> Result<Value, String> {
    let target = crate::update::verify_installer(&path)?;

    // macOS： can ， not after 、 not （ by ）
    #[cfg(target_os = "macos")]
    {
        // app down that 1 use to ； 1 ， macOS use 
        let _ = &app;
        let _ = restart;
        crate::update::launch_installer(&target, false)?;
        return Ok(json!({
            "launched": true,
            "path": target.to_string_lossy(),
            "restart": false,
        }));
    }

    // Linux：deb to （ not ），AppImage 「 + 」（ need ）
    #[cfg(target_os = "linux")]
    {
        let is_appimage = target
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("appimage"))
            .unwrap_or(false);

        if is_appimage {
            // AppImage： new file need old can file，
            // （ and Windows 1 ， by ）
            crate::backend::shutdown(&app.state::<AppState>());
            crate::update::launch_installer(&target, true)?;

            let restart = restart.unwrap_or(true);
            if restart {
                app.state::<AppState>().begin_exit();
                let handle = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(400));
                    handle.exit(0);
                });
            }
            return Ok(json!({
                "launched": true,
                "path": target.to_string_lossy(),
                "restart": restart,
            }));
        }

        // deb： to ， not after 、 not （ by ）。
        // and macOS that 1 use parameter， unused 。
        let _ = &app;
        let _ = restart;
        crate::update::launch_installer(&target, false)?;
        Ok(json!({
            "launched": true,
            "path": target.to_string_lossy(),
            "restart": false,
        }))
    }

    #[cfg(target_os = "windows")]
    {
        // node.exe file use and 3065 ，file
        crate::backend::shutdown(&app.state::<AppState>());
        crate::update::launch_installer(&target, true)?;

        let restart = restart.unwrap_or(true);
        if restart {
            let state = app.state::<AppState>();
            state.begin_exit();
            // 1 when return to before ，
            let handle = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(400));
                handle.exit(0);
            });
        }
        Ok(json!({
            "launched": true,
            "path": target.to_string_lossy(),
            "restart": restart,
        }))
    }
}

/// use default Release 。
///
/// http(s)： to when ，file:// will 「 use defaultfile」，
/// etc to 1 file 。
///
/// use login.rs （Windows ShellExecuteW）： 
/// `cmd /C start` will URL `&` ， will ，
/// and will 1 down 。
#[tauri::command]
pub fn open_release_page(url: String) -> Result<Value, String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err(" http(s) ".to_string());
    }
    crate::login::open_in_browser(trimmed)?;
    Ok(json!({ "url": trimmed }))
}

/// set（）。
///
/// Windows up this （tao DWMWA_USE_IMMERSIVE_DARK_MODE）：
/// not set when by by 「」， is to when is ，
/// will 1 。 None ， and Tauri 1 。
#[tauri::command]
pub fn set_window_theme(app: AppHandle, theme: Option<String>) -> Result<(), String> {
    let theme = match theme.as_deref() {
        Some("dark") => Some(tauri::Theme::Dark),
        Some("light") => Some(tauri::Theme::Light),
        _ => None,
    };
    let window = app
        .get_webview_window(crate::MAIN_WINDOW_LABEL)
        .ok_or_else(|| " not in ".to_string())?;
    window.set_theme(theme).map_err(|error| format!("setfailure: {error}"))
}

/// set （：， and ）。
///
/// Tauri `set_zoom` —— Windows up is WebView2 ZoomFactor， and 
/// use by Ctrl +/- 1 （ not is CSS zoom： that ， will and WebView ）。
/// up is 「 down 」（set「 → 」，80%–130%、 1 5%），
/// this 1 ： not —— has 、
/// （ not ， 130% is ）。
///
/// return is ** **（ to ），。 
/// in before localStorage， this not （ and 1 ：，
/// by ）。
#[tauri::command]
pub fn set_zoom(app: AppHandle, scale: f64) -> Result<f64, String> {
    const MIN: f64 = 0.8;
    const MAX: f64 = 1.3;
    if !scale.is_finite() {
        return Err(format!(": {scale}"));
    }
    // ：0.95 this in is 0.9499999…， will 95% 
    let factor = (scale * 100.0).round() / 100.0;
    if !(MIN..=MAX).contains(&factor) {
        return Err(format!(" need in 80%–130% （ to {:.0}%）", factor * 100.0));
    }
    let window = main_window(&app)?;
    window
        .set_zoom(factor)
        .map_err(|error| format!("setfailure: {error}"))?;
    Ok(factor)
}

// ── ─────────────────────────────────────
// already （ lib.rs `.decorations(false)`，
// OmniProxy ）， / / by 
// 。 this is `WebviewWindow` ：
// （bridge.rs） use ， not Tauri —— and file its 1 to 。
//
// ── Why can to window.close ─────────────────────────
// is CloseRequested， and ✕ ** 1 path**：
// lib.rs 「 to 」 （prevent_close + hide） for 1 ，
// as not 。
//
// and not in this ： that is `data-tauri-drag-region` as 
// （Tauri 2 core mousedown）， before ，none need and 。

/// （「」 by ）。
#[tauri::command]
pub fn window_minimize(app: AppHandle) -> Result<(), String> {
    let window = main_window(&app)?;
    window.minimize().map_err(|error| format!("failure: {error}"))
}

/// / （「」 by ）。
///
/// Tauri 2.11 `WebviewWindow` no has toggle_maximize（ that is JS API ），
/// has maximize / unmaximize / is_maximized —— this 「」：
/// already ，otherwise。 after by 
/// [`window_is_maximized`] get（ will ），
/// not return —— 「」 has 1 。
#[tauri::command]
pub fn window_toggle_maximize(app: AppHandle) -> Result<(), String> {
    let window = main_window(&app)?;
    let result = if window
        .is_maximized()
        .map_err(|error| format!("failure: {error}"))?
    {
        window.unmaximize()
    } else {
        window.maximize()
    };
    result.map_err(|error| format!("failure: {error}"))
}

/// （ ✕）。
///
/// and by ： CloseRequested，「 to 」 when 
/// lib.rs （），otherwise —— by up 。
#[tauri::command]
pub fn window_close(app: AppHandle) -> Result<(), String> {
    let window = main_window(&app)?;
    window.close().map_err(|error| format!("failure: {error}"))
}

/// is （ / ）。
///
/// in when 1 ， after `tauri://resize`（ bridge.rs 
/// onWindowResize）： / / will ，
/// after 1 can ， not need need 1 use 。
#[tauri::command]
pub fn window_is_maximized(app: AppHandle) -> Result<bool, String> {
    let window = main_window(&app)?;
    window.is_maximized().map_err(|error| format!("failure: {error}"))
}

/// ： use 1 （ not in when can error）。
fn main_window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window(crate::MAIN_WINDOW_LABEL)
        .ok_or_else(|| " not in ".to_string())
}

/// query ： and ， 1 can 
fn urlencoding(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => ch.to_string(),
            other => format!("%{:02X}", other as u32 & 0xFF),
        })
        .collect()
}

/// when （yyyy-MM-dd-HH-mm-ss）， when 
fn timestamp_for_filename() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // from UTC seconds when ： this used forfile， use UTC+8 can 
    let secs = now + 8 * 3600;
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    // civil_from_days： 1970-01-01 
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{day:02}-{hour:02}-{minute:02}-{second:02}")
}

