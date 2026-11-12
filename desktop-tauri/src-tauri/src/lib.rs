/**
 * use — 、 and 、create。
 *
 * before in desktop-tauri/ui/， tauri.conf.json frontendDist 。
 * window.workbuddyDesktop this ， not ，
 * from Electron to Tauri when before 1 。：
 * server/ HTTP （； as node after ）
 * `server/config_migration.rs` is 1.x configdirectory 1 ，
 * by file setup 1 use （mustconfigdirectory）
 * backend.rs （/check/）
 * gateway.rs API HTTP client（ API Key and 1 ）
 * login.rs and 
 * commands.rs to before invoke （ for preload API ）
 * settings.rs use set（ to 、） 
 * tray.rs 、 and 
 *
 * （ to / / ） in in file：
 * this is 「 use 」 as ， to not 。
 *
 * before ： in create when bridge.js， in before 
 * window.workbuddyDesktop ，none need Tauri。
 */

mod account_export;
mod assistant_image;
mod backend;
mod bridge;
mod commands;
mod deepseek_login;
mod doubaowork_login;
mod gateway;
mod legacy_install;
mod login;
mod login_profile;
mod monkeycode_login;
mod panel_session;
mod settings;
mod startup;
mod state;
mod tray;
mod update;

// and already to crate（`server/`， and headless 
// use ）。 in this ：crate all `crate::server::…` /
// `crate::port_conflict::…` path and before 1 ，。
use agent2api_server::port_conflict;
use agent2api_server::server;

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

use server::config_migration::MigrationOutcome;
use state::AppState;

/** default and （ and Electron 1 ） */
const WIN_WIDTH: f64 = 1060.0;
const WIN_HEIGHT: f64 = 800.0;
const WIN_MIN_WIDTH: f64 = 820.0;
const WIN_MIN_HEIGHT: f64 = 600.0;

/// ：、、 by 
pub const MAIN_WINDOW_LABEL: &str = "main";

/// use ：、、 use 1 。
///
/// debug （`tauri dev`） `[Dev]` before —— and can when 
/// （ and identifier ， `gateway::DEV_PORT` and `tauri.dev.conf.json`），
/// and 1 1 ， not not is 。
/// before in before ， that can 1 to 。
///
/// `debug_assertions`， and down 「 not / new 」 1 。
pub fn app_title() -> &'static str {
    if cfg!(debug_assertions) {
        "[Dev]Agent2API·多提供商本地网关"
    } else {
        "Agent2API · 多提供商本地网关"
    }
}

/// when parameter。 when to not ，
/// in after ， use 。
const AUTOSTART_FLAG: &str = "--autostart";

/// is by 
fn launched_by_autostart() -> bool {
    std::env::args().any(|arg| arg == AUTOSTART_FLAG)
}

/// WebView2 use directory(EBWV)**default**: EBWV to 
/// `W:\EBWebView\easykeysy-Agent2API_fork`, its directory `W:\EBWebView` as 
/// Chromium config。 this to `data_directory` is this directory —— wry 
/// this path**** to `CreateCoreWebView2EnvironmentWithOptions` 
/// userDataFolder, not will in `EBWebView`,defaultmust is directorydirectory。
///
/// `AGENT2API_EBWEBVIEW_DIR` can (set / empty use default),
/// and `paths::config_dir` `AGENT2API_PROXY_HOME` 1 。directory not in when 
/// by Tauri when `create_dir_all` create,none need in 。
#[cfg(target_os = "windows")]
fn ebwv_data_dir() -> std::path::PathBuf {
    if let Ok(value) = std::env::var("AGENT2API_EBWEBVIEW_DIR") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return std::path::PathBuf::from(trimmed);
        }
    }
    std::path::PathBuf::from(r"W:\EBWebView\easykeysy-Agent2API_fork")
}

/// error use and ： can Reason → 。
///
/// ── Why not is `return Err(...)` ─────────────────────────────
/// current Tauri（2.11） in `setup` return `Err` when is `panic!("Failed to setup app")`，
/// release is `panic = "abort"` —— use will to 1 ， not to 
/// 「 old in 、Whyfailure、」 this 。 this as path：
/// error（path and error， not file） + error for ，
/// use after ， not configfile。
///
/// ── for Why use `show` ─────────────────────────────
/// setup in up ，`blocking_show` need not can in use 
/// （ will ）； to setup use is `show(callback)`。
/// in use for after ， that request —— ，
/// setup already before return、 and no ， not will has or 。
///
/// ── ─────────────────────────────────────────────────
/// for 1 no can （e.g.failure）， will 「 no has 、
/// no has 、」 ： use will to this 
/// ， not 。failurepath 1 has ； when 
/// （5 minutes）， use when 。
///
/// ── path not will ────────────────────────────────────────
/// failure when `settings::load()` from ，`close_to_tray` default false，
/// `ExitRequested` not will ， will `backend::shutdown` ——
/// that and ， not request、 not configfile（ after from ，
/// when to ）。`app.exit` ExitRequested/Exit，
/// not will 1 setup after ； `process::exit` ：
/// when no has 、、 after or file need need 。
fn report_startup_failure(app: &tauri::AppHandle, error: &str) {
    /// use when not when limit
    const FORCE_EXIT_AFTER: std::time::Duration = std::time::Duration::from_secs(300);

    let message = format!(
        "Agent2API 启动已中止：{error}\n\n\
         本次未改动、未删除任何旧数据，也未创建新配置目录。\
         排除原因（磁盘空间、文件占用、权限）后重新打开本程序即可自动重试迁移。"
    );
    eprintln!("[Startup] ❌ {message}");
    let handle = app.clone();
    app.dialog()
        .message(message)
        .title("Agent2API 无法启动")
        .kind(MessageDialogKind::Error)
        .show(move |_| handle.exit(1));

    std::thread::spawn(|| {
        std::thread::sleep(FORCE_EXIT_AFTER);
        eprintln!("[Startup] ⚠️  错误提示长时间未确认，强制结束本次启动");
        std::process::exit(1);
    });
}

/// create —— 1 ； down after ， by need this 。
///
/// parameter（ / and / / default / /
/// WebView2 after parameter）** in this 1 **： is use 「 new 」
/// ，parameter will as 「 and not 1 」，
/// and must use 1 。
///
/// `visible` by use ： when by 「 is 」（
/// ， not will in 1 down ）， when as true。
fn create_main_window(app: &AppHandle, visible: bool) -> tauri::Result<tauri::WebviewWindow> {
    // ── ： 1 ， in this 1 ────────────────
    // is ** 32px **（titlebar.js create， and 1 
    // ）， in 「 this 、」：
    //
    // · Windows / Linux —— `decorations(false)` ， by 
    // （ up ，Windows 11 ）。
    // · macOS —— ， as （`TitleBarStyle: Overlay`
    // + `hidden_title`）：// by AppKit in up 、 in 
    // up （Overlay will 1 and fullSizeContentView）， that 32px
    // is ， that 
    // （ ui/css/layout.css .titlebar.is-macos）。
    //
    // Why macOS not use decorations(false) ： can is ****，
    // not that as —— by 、 Space、 and 、
    // 、 by Option 、 and 「」
    // （ that 1 can / / not ， not ）。
    // is macOS parameter and Windows not （ this 1 by ）。
    //
    // ：
    // · / ： before to `data-tauri-drag-region`
    // —— Tauri core mousedown， this 
    // （ not not ），（）。Overlay down this 
    // is **must** ： in not 。
    // already limit：Overlay down when not （tauri#4316）——
    // 1 by down ， need 1 。Tauri as ， not 。
    // · ：`resizable(true)`（default）。
    // · ：Windows ✕ window_close → CloseRequested，
    // and by 1 path， not ；macOS use ，
    // is that （ is CloseRequested）。
    // ── WebView2 after parameter（Windows ）──────────────────────
    // or when ，Chromium default will for after enabled when 
    // （Timer Throttling） and ， after （ui/zcode-captcha-pool.js）
    // setTimeout 。 this limit after 
    // （ down ， this —— AppSettings::lightweight_mode）。
    //
    // `--disable-features=…` ** not can **：wry for additional_browser_args is 
    // ****（`unwrap_or_else`， wry webview2/mod.rs）， not to 
    // wry default —— default has WebView2「⋯」 and SmartScreen check
    // that （Tauri in additional_browser_args this 1 ）。
    // after is PR add ， and before not ， can and in 1 。
    #[cfg(target_os = "windows")]
    let builder = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .decorations(false)
        // ── default:WebView2 use directory（EBWV）─────────────────
        // up default in `%LOCALAPPDATA%\com\.workbuddy\.local-proxy`（Tauri by 
        // identifier ）。 is EBWV directory to Chromium
        // config directory， this 1 to `easykeysy-Agent2API_fork`
        // directory， profile ， not use profile。
        // data_directory before identifier ； can （set
        // use default）。
        .data_directory(ebwv_data_dir())
        .additional_browser_args(
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection \
             --disable-background-timer-throttling \
             --disable-backgrounding-occluded-windows \
             --disable-renderer-backgrounding",
        );

    // macOS：`decorations(true)` not can —— tao when if decorations as false
    // will Borderless and **ignore** title_bar_style（ tao 
    // platform_impl/macos/window.rs），Overlay ， not will 。
    // `hidden_title` （ not ，
    // / use ）， not 。
    #[cfg(target_os = "macos")]
    let builder = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .decorations(true)
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        // ：x is to （ is 20），
        // y is wry inset —— by ` + y` 、
        // in its in in ， 20 in that 32px up down 
        // 10px。 not to this default（ by ，Big Sur+ is 
        // 28pt）， in 32px will 2px。
        // Note this is ****， not （set 80%–130%） ——
        // will that 32px ， when not in （
        // 5pt up down ）， can ； no to when this 。
        .traffic_light_position(tauri::LogicalPosition::new(20.0, 20.0));

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let builder = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .decorations(false);

    builder
        .title(app_title())
        .inner_size(WIN_WIDTH, WIN_HEIGHT)
        .min_inner_size(WIN_MIN_WIDTH, WIN_MIN_HEIGHT)
        .center()
        .maximized(true)
        .visible(visible)
        .initialization_script(bridge::bridge_js())
        .build()
}

/// in and return： no has （ after / create） by parameter。
///
/// and use this path —— 「 」，
/// not this 1 in 。failure（WebView not etc ）return None，
/// use by 「unavailable」， not panic。
pub fn ensure_main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        return Some(window);
    }
    create_main_window(app, true).ok()
}

pub fn run() {
    let mut builder = tauri::Builder::default();

    // must 1 ： by ， its when ，
    // can can already / as 
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // already ， this already has to before 
            tray::show_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            // --autostart， when need not need （ not ）。
            // MacosLauncher is parameter， Windows。
            tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![AUTOSTART_FLAG])),
        )
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::api_request,
            commands::api_request_text,
            commands::backend_status,
            commands::wait_for_backend,
            commands::port_occupant,
            commands::end_port_occupant,
            commands::restart_app,
            commands::check_port,
            commands::change_port,
            commands::panel_admin_status,
            commands::panel_register,
            commands::panel_session_status,
            commands::panel_session_store,
            commands::panel_session_clear,
            commands::change_lan_access,
            // when （AI not request， commands.rs）
            commands::direct_credentials,
            commands::local_ip,
            commands::start_login,
            commands::start_autoclaw_oauth_login,
            commands::login_kuku_sms_send,
            commands::login_kuku_sms_verify,
            commands::login_state,
            commands::cancel_login,
            commands::export_logs,
            commands::pick_folder,
            commands::get_app_settings,
            commands::save_app_settings,
            commands::export_accounts,
            account_export::export_account_files,
            assistant_image::save_assistant_image,
            commands::import_accounts,
            commands::export_database,
            commands::import_database,
            commands::cloud_sync_upload,
            commands::cloud_sync_download,
            commands::cloud_sync_import_key,
            commands::check_update,
            commands::system_info,
            commands::download_update,
            commands::update_progress,
            commands::cancel_update,
            commands::run_installer,
            commands::set_window_theme,
            commands::set_zoom,
            commands::open_release_page,
            // （ already decorations(false)，）
            commands::window_minimize,
            commands::window_toggle_maximize,
            commands::window_close,
            commands::window_is_maximized,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // ── 1 ： 1 configdirectory（`~/.workbuddy-proxy` → `~/.agent2api`）──
            // must is **** 1 configdirectory：（set、
            // config.json、、） when ， 1 failure， after will new directory
            // ，`target.exists()` from as ，none， use to is 
            // 「/config no 」。 in this ，failure when down all will not 。
            //
            // this when no ， can （`eprintln!`， logging）。
            match server::config_migration::migrate_config_dir() {
                Ok(MigrationOutcome::Migrated { from, to }) => {
                    eprintln!(
                        "[Config] 📦 配置目录已迁移: {} → {}（旧目录保留，可回退）",
                        from.display(),
                        to.display()
                    );
                }
                Ok(MigrationOutcome::NothingToDo) => {}
                // use use configdirectory： not use ， not this 
                Ok(MigrationOutcome::SkippedEnvOverride) => {}
                Err(error) => {
                    // failure not can ：emptyconfig will new directory after ，
                    // from not will ， and use to is 「」。
                    // this can error（ for ） after ——
                    // old directory， new directory create， down 。
                    report_startup_failure(&handle, &error);
                    return Ok(());
                }
            }

            // set 1 and ： is ， need need to 
            // close_to_tray can is is ；lightweight ——
            // is 「」 is 「」。#188： before ，
            // need etc set/ 1 （get_app_settings / save_app_settings）
            // ， is 「 → 」 this path up as default false，
            // 、 no 。
            let app_settings = settings::load();
            {
                let state = app.state::<AppState>();
                state.window.set_close_to_tray(app_settings.close_to_tray);
                state.window.set_lightweight(app_settings.lightweight_mode);
            }

            // ── old 「current use 」 + new （1.x → perMachine ）──
            // perMachine after ， when `%LOCALAPPDATA%` and 
            // HKCU to ， not to not use old ——
            // has this （ use up down ） can to 。
            //
            // old `%LOCALAPPDATA%\< old >`、 use old ；
            // new in Program Files、 productName， not ， old that 
            // to already delete path， use not set not will ，。
            //
            // ── Why in 1 after ──────────────
            // need / HKCU Run （「 old is already 」、 new 
            // new ）， new to is after ， 
            // 1 。
            //
            // after ： old directory can can 88MB node.exe，`remove_dir_all` 
            // will ， this and after no has 。
            // failure、 down ， not （ legacy_install module header）。
            let autostart_wanted = app_settings.autostart;
            let cleanup_handle = handle.clone();
            tauri::async_runtime::spawn_blocking(move || {
                // ── （debug ） not ────────────────────────
                // this will ： will already 
                // （directory + + ）， new will to 
                // `target\debug` down 。 `tauri dev` is ——
                // 1 use ，（：
                // after ，/）。
                // release （ 1 ） as not ； need 
                // ， use `tauri build` 。
                if cfg!(debug_assertions) {
                    return;
                }
                legacy_install::cleanup_legacy_user_install();
                if !autostart_wanted {
                    return;
                }
                let name = cleanup_handle.package_info().name.clone();
                // etc ： already to current exe when not ； use in 
                // 「」disabled not （ legacy_install）
                if !legacy_install::autostart_needs_refresh(&name) {
                    return;
                }
                match cleanup_handle.autolaunch().enable() {
                    Ok(()) => {
                        eprintln!("[Cleanup] 已按当前安装路径重新登记开机自启（升级迁移）: {name}")
                    }
                    Err(error) => {
                        eprintln!("[Cleanup] 重新登记开机自启失败（下次启动再试）: {error}")
                    }
                }
            });

            // ： not ， is 1 
            if let Err(error) = tray::create(&handle) {
                eprintln!("[tray] 创建托盘图标失败: {error}");
            }

            // and ， after in after ，
            // not will as etc node 。
            // when not （visible(false) ，
            // not will in 1 down ）。
            //
            // ── + ： not ──────────────────────
            // hide not destroy ，WebView2 that 6 （
            // 200MB）—— need etc use 「 1 」，
            // path not to that 1 （ not ）。 this ，
            // and after ； use （ or ） when by 
            // `ensure_main_window` by 1 parameter —— and down 「
            // 」 is 1 path， 1 。
            //
            // and 1 ： in WebView ZCode 
            // in use before not ， this request will 503 request（
            // `AppSettings::lightweight_mode` ）。 or 
            // old ， not this 。
            //
            // parameter（none、WebView2 、 etc ） in 
            // create_main_window 1 ： down ，
            // must 1 1 。
            let autostarted = launched_by_autostart();
            if !(autostarted && app_settings.lightweight_mode) {
                create_main_window(&handle, !autostarted)?;
            }

            // after ， not new 。
            tauri::async_runtime::spawn(async move {
                let started = backend::ensure_ready(&handle).await;
                if let Err(failure) = &started {
                    eprintln!("[backend] 启动失败: {}", failure.message);
                    // 1 to AppState： is 1 ， can can in after 
                    // （ in 、WebView new ）， that when can 
                    // （backend_status） to failureReason，otherwise not Why
                    if let Some(state) = handle.try_state::<AppState>() {
                        if let Ok(mut guard) = state.backend.lock() {
                            guard.failure = Some(failure.clone());
                        }
                    }
                    startup::complete(&started);
                    let _ = handle.emit("backend:error", failure);
                } else {
                    startup::complete(&started);
                    let _ = handle.emit("backend:ready", ());
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Tauri 应用初始化失败")
        .run(|app, event| match event {
            // when by set「 to 」 is 「」。
            // after is ， after ，。
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                if label != MAIN_WINDOW_LABEL {
                    return;
                }
                let state = app.state::<AppState>();
                // use （） when ，otherwisepath 
                if state.window.close_to_tray() && !state.window.is_exiting() {
                    api.prevent_close();
                    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                        if state.window.lightweight() {
                            // ： —— WebView2 that 6 
                            // （ 200MB ）， in 
                            // not 。 after `ExitRequested` by down down 
                            // （close_to_tray → prevent_exit， in ），
                            // when by `ensure_main_window` by 1 parameter。
                            let _ = window.destroy();
                        } else {
                            let _ = window.hide();
                        }
                    }
                }
            }
            // after 1 after Tauri will request， this 1 ：
            // 「 use 」 this 1 ， and exit need 。
            // when after —— e.g. use 「 to 」 after ，
            // this path not will to down Exit ， not in this will down node 
            // （shutdown etc ， and its use not will has use ）。
            tauri::RunEvent::ExitRequested { api, .. } => {
                let state = app.state::<AppState>();
                if state.window.close_to_tray() && !state.window.is_exiting() {
                    api.prevent_exit();
                } else {
                    backend::shutdown(&state);
                }
            }
            // after after （ use not will ）。
            // Note: （、 before is `AppHandle::exit(0)`）
            // not ， this is Tauri already as ， that path
            // already use shutdown， this its 。
            tauri::RunEvent::Exit => {
                let state = app.state::<AppState>();
                backend::shutdown(&state);
            }
            _ => {}
        });
}
