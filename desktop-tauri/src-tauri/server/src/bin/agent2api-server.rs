//! headless ：Agent2API **none**。
//!
//! ── and ──────────────────────────────────────────
//! 1 lib（`agent2api_server`），HTTP 、directory、
//! 1 。 has ：
//! · ： by `AGENT2API_HOST` （default 0.0.0.0，
//! ）； 127.0.0.1。
//! · ： by Tauri custom-protocol ； by 
//! `ui/` directory（`set_ui_dir`）， and bridge
//! （`web_shim`，UI not ）。
//! · ： to ， ——
//! need 「 or API Key」（ no has then ）；
//!     转发面（/v1/*） no Key when fail-closed（ 1 ；
//! default， 1 not in ； need `AGENT2API_ALLOW_NO_KEY`）。
//!
//! ── （ 1 can file）──────────────────────────────
//! · `agent2api-server` before ： stderr，Ctrl+C ；
//! · `agent2api-server --service` Windows （only Windows）： to SCM
//! ，file，`sc.exe stop` 。
//! Linux up for is systemd （ before can ， by systemd ），
//! not need need this parameter —— `agent2api.service` is this 。
//!
//! ── ────────────────────────────────────────────────
//! AGENT2API_PROXY_HOME config/directory（ old WORKBUDDY_PROXY_HOME ）
//! AGENT2API_HOST ，default 0.0.0.0
//! AGENT2API_PROXY_PORT （ old WORKBUDDY_PROXY_PORT），default 3065
//! AGENT2API_PANEL_PORT optional： —— set after （ + /api/*）
//! this ， /v1/* ；
//! ， can in 
//! AGENT2API_UI_DIR directory，default `ui/`（ for can file）
//! AGENT2API_LOG_FILE filepath（optional）。 down not can ：
//! by `{PROXY_HOME}/logs/server.log` 
//! AGENT2API_ADMIN_USER （ and down 1 when set）
//! AGENT2API_ADMIN_PASSWORD （， when bcrypt ）
//! AGENT2API_ADMIN_PASSWORD_HASH bcrypt （；
//! htpasswd -nBC 10 user can ）
//! AGENT2API_ALLOW_NO_KEY `1` （ Key ， use ）
//! AGENT2API_CAPTCHA_ENABLED ，default as 1，0 as 
//! AGENT2API_SELF_UPDATE `1` in update（Dockerfile default；
//! file after 、 by restart ）
//! AGENT2API_VERBOSE `1` debug （ and 1 ）
//! AGENT2API_IMAGE_CACHE_MB down limit（MiB，default 128，
//! 8~4096；/default）。、
//! not ， and its config 1 
//! AGENT2API_IMAGE_COMPRESS_CACHE_MB will limit
//! （MiB，default 128， 8~4096， up ）

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::ExitCode;

use agent2api_server::server::{config, logging, start, ServerState};

// ── （only Linux）──────────────────────────────────
// 、 and （Docker / Linux server） use jemalloc，
// glibc malloc and RSS （Issue #420）。Why in Linux、
// Why is jemalloc、Why not feature， server/Cargo.toml for 
// —— this "" that 1 。
//
// need ： in can has 1 ， in headless
// ， not `agent2api_server` —— is that ， 1 ，
// will 「duplicate global allocator」。 in this ， no has 
// ， to Rust default， as not 。
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// after parameter。 before and use 1 ，。
struct Launch {
    port: u16,
    host: IpAddr,
    panel_port: Option<u16>,
    ui_dir: PathBuf,
}

/// after ： when need use 。
struct Running {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    /// 「 and 」 （update use ）
    restart_rx: tokio::sync::mpsc::Receiver<()>,
}

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("--systemd-updater") {
        return ExitCode::from(agent2api_server::server::core::update::systemd::run_helper() as u8);
    }
    if std::env::args().nth(1).as_deref() == Some("--windows-updater") {
        let id = std::env::args().nth(2).unwrap_or_default();
        return ExitCode::from(agent2api_server::server::core::update::windows_service::run_helper(&id) as u8);
    }
    // Windows in **parameter before **： no has ，
    // config（） when eprintln will ，「config not 」
    // is need need 。 file， can can 。
    #[cfg(windows)]
    if service::requested() {
        // ：set「」 need can 「Windows 」 and 
        // 「 before headless」。`--service` in this 1 ， after 
        // no has can （SCM and 1 1 ）。
        agent2api_server::server::core::system_info::set_service_mode(true);
        logging::init_file_sink();
        let launch = match parse_launch() {
            Ok(launch) => launch,
            Err(reason) => {
                // when file already ，console_line will when file
                logging::console_line("[Server]", &format!("❌ {reason}"));
                return ExitCode::FAILURE;
            }
        };
        return run_as_windows_service(launch);
    }

    let launch = match parse_launch() {
        Ok(launch) => launch,
        Err(reason) => {
            eprintln!("❌ {reason}");
            return ExitCode::FAILURE;
        }
    };

    run_foreground(launch)
}

/// before ：Ctrl+C / SIGTERM / path。
fn run_foreground(launch: Launch) -> ExitCode {
    let running = match start_gateway(&launch) {
        Ok(running) => running,
        Err(reason) => {
            eprintln!("❌ {reason}");
            return ExitCode::FAILURE;
        }
    };

    // ：Ctrl+C（）+ SIGTERM（unix， stop this ）；
    // 「 」 and 1 path（ `start_gateway` ）。
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("❌ create Tokio when failure: {error}");
            return ExitCode::FAILURE;
        }
    };
    let Running {
        shutdown_tx,
        restart_rx,
    } = running;
    runtime.block_on(wait_for_shutdown_signal(restart_rx));
    finish(shutdown_tx);
    ExitCode::SUCCESS
}

/// ：、、、 HTTP 。
///
/// before and use —— in 「 etc 」， not in this 。
fn start_gateway(launch: &Launch) -> Result<Running, String> {
    // bootstrap：、config、 old 、（ and 1 when ）。
    // failure can can directory、 and 「 schema 」
    // —— ， not can 。
    let mut state = ServerState::bootstrap(launch.port, launch.host)
        .map_err(|reason| format!("failure: {reason}"))?;

    state.panel_port = launch.panel_port;
    // headless ： when /api/* and API Key（
    // ， access::panel_gate and http::require_api_key ）。
    // 1 in down ：AGENT2API_ALLOW_NO_KEY=1 is 「
    // need 」 ， not 。
    agent2api_server::server::access::set_panel_gate(true);
    if let Some(p) = launch.panel_port {
        logging::log(
            "[Server]",
            &format!(
                " already enabled：（ + /api/*） :{p}， /v1/* —— /compose "
            ),
        );
    }

    // and new （ / ）
    agent2api_server::server::access::attach_db(state.db().cloned());
    agent2api_server::server::access::load_refresh_tokens();
    agent2api_server::server::access::load_access_tokens();
    // env （ in before already ，）
    agent2api_server::server::access::sync_env_admin_to_store();

    apply_security_gates();

    state.set_ui_dir(launch.ui_dir.clone());

    // 「 」：「 and 」 after ，update 
    // this 1 ， SIGTERM that and return —— by 
    // （ restart / systemd Restart= / SCM ）。
    //
    // start() before ：request will by in ， need need this 
    // （ in ， `finish`）。
    let (restart_tx, restart_rx) = tokio::sync::mpsc::channel::<()>(1);
    agent2api_server::server::core::update::selfupdate::set_restart_hook(Box::new(move || {
        // in update after ：try_send not 、 not need when up down 
        let _ = restart_tx.try_send(());
    }));

    let shutdown_tx = start(&state).map_err(|conflict| format!("failure：{}", conflict.message))?;
    agent2api_server::server::core::update::systemd::notify_ready();

    logging::log(
        "[Server]",
        &format!(
            "headless in ： http://{}:{}/ ｜ API http://{}:{}/v1",
            display_host(launch.host),
            launch.port,
            display_host(launch.host),
            launch.port
        ),
    );

    Ok(Running {
        shutdown_tx,
        restart_rx,
    })
}

/// ：/v1/* fail-closed and 。
///
/// is 「 127.0.0.1」，（ 1 Key 
/// no → ） in up ； 0.0.0.0、 for can ：
/// · （/v1/*）： API Key。 no has Key when fail-closed
/// （ and 1 ）， not for ；
/// · （/api/*）： after need need 。** **
/// （otherwise「create」 not ）—— this is ： can can 
/// in 「 to use 」， will ；
/// 1 ， need 。 this ， use 
/// （AGENT2API_ADMIN_USER / …_PASSWORD_HASH，）。
/// need none Key ， AGENT2API_ALLOW_NO_KEY=1 。
fn apply_security_gates() {
    if !config::current().active_api_keys().is_empty() {
        logging::log("[Security]", "API Key already enabled");
    } else if std::env::var("AGENT2API_ALLOW_NO_KEY")
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
    {
        // is 「 need 」 ：must，
        // otherwise /api/* will ， and 
        agent2api_server::server::access::set_panel_gate(false);
        logging::log(
            "[Security]",
            "⚠️ AGENT2API_ALLOW_NO_KEY=1：not configured API Key， for all",
        );
    } else {
        agent2api_server::server::access::set_v1_fail_closed(true);
        if agent2api_server::server::access::panel_auth_enabled() {
            logging::log(
                "[Security]",
                "not configured API Key： after in 「 Key」create 1 ，/v1/* in before ",
            );
        } else {
            logging::log(
                "[Security]",
                "⚠️ and not configured API Key： http://<>/login ",
            );
            logging::log(
                "[Security]",
                " /v1/* already fail-closed（）； in before not ， after need ",
            );
        }
    }
}

/// ： HTTP ， to in request when 。
fn finish(shutdown_tx: tokio::sync::oneshot::Sender<()>) {
    let _ = shutdown_tx.send(());
    // `start` will etc graceful shutdown ， this 1 
    std::thread::sleep(std::time::Duration::from_millis(300));
}

// ─── Windows ────────────────────────────────────────

#[cfg(windows)]
pub mod service;

/// Windows ： to SCM 。
///
/// use （main） already file —— no has ，
/// not file etc is 。
#[cfg(windows)]
fn run_as_windows_service(launch: Launch) -> ExitCode {
    // SCM need after up dispatcher；run_service etc 
    // when （and StartPending）。
    let start_fn: service::StartFn = Box::new(move |stop, ready| {
        let Running {
            shutdown_tx,
            mut restart_rx,
        } = start_gateway(&launch)?;
        // already 、request —— this when SCM to Running
        ready.signal();

        // has ，path：
        // · SCM STOP / SHUTDOWN （`sc.exe stop`、）；
        // · 「 and 」 update（file after need ，
        // by SCM new —— and restart ）。
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("create Tokio when failure: {error}"))?;
        runtime.block_on(async move {
            tokio::select! {
                _ = stop.into_async().wait() => {}
                _ = restart_rx.recv() => {}
            }
        });

        finish(shutdown_tx);
        Ok(())
    });

    match service::dispatch(start_fn) {
        Ok(()) => ExitCode::SUCCESS,
        Err(reason) => {
            // no has ：，otherwise this error（ is 
            // 「 SCM failure」） will in none can stderr 
            logging::console_line("[Server]", &format!("❌ {reason}"));
            ExitCode::FAILURE
        }
    }
}

// ─── parameter ──────────────────────────────────────────────

/// parameter（ / / / UI directory）。
fn parse_launch() -> Result<Launch, String> {
    // and 1 ： new > old > default（set）
    let port = env_port("AGENT2API_PROXY_PORT")
        .or_else(|| env_port("WORKBUDDY_PROXY_PORT"))
        .unwrap_or(3065);

    let host: IpAddr = match std::env::var("AGENT2API_HOST") {
        Ok(value) if !value.trim().is_empty() => value
            .trim()
            .parse()
            .map_err(|_| format!("AGENT2API_HOST not is IP : {value}"))?,
        // headless default 0.0.0.0： is / ，
        // will and 。 for by 
        // 「 Key 」 + API Key 。
        _ => IpAddr::from(Ipv4Addr::UNSPECIFIED),
    };

    // （optional）： （ + /api/*） to ，
    // /v1/* —— when ， can 。
    // and no has （ etc no ）， not up 。
    let panel_port = match std::env::var("AGENT2API_PANEL_PORT") {
        Ok(value) if !value.trim().is_empty() => {
            let p: u16 = value
                .trim()
                .parse()
                .map_err(|_| format!("AGENT2API_PANEL_PORT not is : {value}"))?;
            if p == port {
                return Err(format!(
                    "AGENT2API_PANEL_PORT and （{p}）： need not "
                ));
            }
            Some(p)
        }
        _ => None,
    };

    Ok(Launch {
        port,
        host,
        panel_port,
        ui_dir: resolve_ui_dir(),
    })
}

/// ：set / / 0 1 set（ and env_port ）
fn env_port(name: &str) -> Option<u16> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port > 0)
}

/// UI directory：AGENT2API_UI_DIR ，default can file `ui/`。
///
/// this directory** can **（ can path）， not in —— directory when 404，
/// API （ API can not ui/）， not this failure。
fn resolve_ui_dir() -> PathBuf {
    if let Some(dir) = std::env::var("AGENT2API_UI_DIR")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        return PathBuf::from(dir);
    }
    let exe = std::env::current_exe().unwrap_or_default();
    exe.parent()
        .map(|parent| parent.to_path_buf())
        .unwrap_or_default()
        .join("ui")
}

/// and ：0.0.0.0 as 127.0.0.1（「all」，
/// use use or IP， 0.0.0.0 in client not ）
fn display_host(host: IpAddr) -> String {
    if host.is_unspecified() {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

/// etc ：Ctrl+C（）、SIGTERM（unix， stop this ），
/// or （ new ， `start_gateway` ）。
async fn wait_for_shutdown_signal(mut restart: tokio::sync::mpsc::Receiver<()>) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect(" SIGTERM failure");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
            _ = restart.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = restart.recv() => {}
        }
    }
}
