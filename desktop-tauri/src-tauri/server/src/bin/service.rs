//! Windows ： headless to SCM（）。
//!
//! ── 1 exe， ──────────────────────────────────────
//! · `agent2api-server.exe` before （ / ）；
//! · `agent2api-server.exe --service` by SCM （install-service.ps1
//! ImagePath this parameter）。
//!
//! `--service` this 1 ， its parameter not —— allconfig
//! （ ImagePath 1 ， install-service.ps1）。
//!
//! ── Why not 「 is not is SCM 」──────────────────
//! 「 StartServiceCtrlDispatcher、failure before 」 is ， for 
//! not ： use in exe when that failure， before 
//! ** no has **（headless no ）， use to is 「 no 
//! 」。 this need `--service`： = before 、 to stderr
//! （ from cmd / PowerShell when ）， = parameter。。
//!
//! ── and before ────────────────────────────────────────
//! · ： no has ，`eprintln!` empty —— use in 
//! SCM before file（`logging::init_file_sink`），otherwise
//! will 「success not 」；
//! · ：SCM STOP / SHUTDOWN ， and Ctrl+C / SIGTERM 1 
//! path（ to → `shutdown_tx` → in request）。
//!
//! ── directory ────────────────────────────────────────────────
//! CWD is `C:\Windows\system32`， not is exe in directory：
//! · ui/ `current_exe()` （`resolve_ui_dir` already ， not ）；
//! · directory `AGENT2API_PROXY_HOME`（**must** in when set，
//! otherwise will to LocalSystem `USERPROFILE` down —— install-service.ps1）。
//!
//! ── Why not in ────────────────────────────────
//! SCM in SCM up ， when will 
//! （「」 not to ）。：、 by SCM
//! need up ； by 。
//!
//! ── Why by ────────────────────────────────
//! 「」 not etc 「available」：、 1 failure，SCM to 
//! failure not is Running （ after will ）。
//! in **、request** that 1 `Ready::signal()`，
//! up Running； in signal before returnerror， up Stopped。
//!
//! in `cfg(windows)` down ， use by `bin/agent2api-server.rs`
//! use `#[cfg(windows)]` 。

use std::ffi::OsString;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
    ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

/// ： and install-service.ps1 `sc.exe create` **must 1 **
/// （SCM by and for up ； not 1 when will 1053 failure）。
pub const SERVICE_NAME: &str = "Agent2API";

/// ： + config + ， to SCM 。
/// used for StartPending `wait_hint`， not will etc this 。
const START_HINT: Duration = Duration::from_secs(30);

/// ： in request（SSE can can no ）。
const STOP_HINT: Duration = Duration::from_secs(20);

/// （`--service`， not ， before after empty）
pub fn requested() -> bool {
    std::env::args()
        .skip(1)
        .any(|arg| arg.trim().eq_ignore_ascii_case("--service"))
}

/// ：SCM to this 。
/// and Ctrl+C / SIGTERM 1 select。
pub struct StopSignal(mpsc::Receiver<()>);

impl StopSignal {
    /// etc SCM need 。
    ///
    /// （ already ）： can SCM by ，
    /// not need 「 not 」。
    pub fn wait(self) {
        let _ = self.0.recv();
    }

    /// can and select 1 use 。
    /// is `tokio::select!`， use `recv()` future ：
    /// this async etc 。
    pub fn into_async(self) -> StopSignalAsync {
        StopSignalAsync(self.0)
    }
}

/// `StopSignal` async ：`wait()` in up 。
/// not use tokio::sync::mpsc：SCM is ****， not can in 
/// await， tokio `blocking_send` need when up down —— use 
/// is can use 。
pub struct StopSignalAsync(mpsc::Receiver<()>);

impl StopSignalAsync {
    /// etc to （ in tokio up ， not worker）。
    pub async fn wait(self) {
        let receiver = self.0;
        let _ = tokio::task::spawn_blocking(move || receiver.recv()).await;
    }
}

/// ： in 「 already 、request」 that 1 `signal()`。
pub struct Ready(mpsc::Sender<()>);

impl Ready {
    pub fn signal(self) {
        let _ = self.0.send(());
    }
}

/// ： to and ， and to 。
pub type StartFn = Box<dyn FnOnce(StopSignal, Ready) -> Result<(), String> + Send>;

/// ：SCM service_main is `extern "system" fn`， not can 
/// ， can （ will use 1 ）。
static START_FN: Mutex<Option<StartFn>> = Mutex::new(None);

/// to SCM and 。 will to 。
pub fn dispatch(start: StartFn) -> Result<(), String> {
    *START_FN.lock().map_err(|_| "启动槽位被污染".to_string())? = Some(start);
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
        .map_err(|error| format!("连接服务控制管理器失败：{error}"))
}

windows_service::define_windows_service!(ffi_service_main, service_main);

/// SCM use ：、 up 、、 up 。
fn service_main(_arguments: Vec<OsString>) {
    if let Err(reason) = run_service() {
        // no has ， this will file（ use already file）
        eprintln!("❌ Windows 服务运行失败：{reason}");
    }
}

fn run_service() -> Result<(), String> {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();

    let event_handler = move |control| -> ServiceControlHandlerResult {
        match control {
            ServiceControl::Stop | ServiceControl::Shutdown | ServiceControl::Preshutdown => {
                // send failure = already （ STOP or ）。
                // NoError， SCM already ， not will to when 。
                let _ = stop_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            // STOP / SHUTDOWN（ down controls_accepted）， its by 
            // to SCM。 no has / / parameter 。
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
        .map_err(|error| format!("注册服务控制回调失败：{error}"))?;

    // StartPending： + can can SCM ，
    // not pending will 「 when 」（1053）。
    status_handle
        .set_service_status(status(ServiceState::StartPending, 1, START_HINT))
        .map_err(|error| format!("上报 StartPending 失败：{error}"))?;

    let start = START_FN
        .lock()
        .map_err(|_| "启动槽位被污染".to_string())?
        .take()
        .ok_or_else(|| "未找到网关启动逻辑（dispatch 未被调用？）".to_string())?;

    // + ：SCM need service_main return，
    // block_on will 1 to 。
    let (ready_tx, ready_rx) = mpsc::channel::<()>();
    let worker = std::thread::Builder::new()
        .name("agent2api-gateway".to_string())
        .spawn(move || start(StopSignal(stop_rx), Ready(ready_tx)))
        .map_err(|error| format!("创建网关线程失败：{error}"))?;

    // etc 。 when or before failure —— not can SCM in 
    // StartPending 1 （ will by wait_hint and ）。
    let ready = ready_rx.recv_timeout(START_HINT).is_ok();

    if !ready {
        // can can in before failure： etc to Reason
        let reason = worker
            .join()
            .err()
            .map(|_| "网关线程异常退出".to_string())
            .unwrap_or_else(|| "网关启动未在宽限期内就绪".to_string());
        let _ = status_handle.set_service_status(status_with_exit(
            ServiceState::Stopped,
            0,
            Duration::default(),
            ServiceExitCode::ServiceSpecific(1),
        ));
        return Err(reason);
    }

    status_handle
        .set_service_status(status(ServiceState::Running, 0, Duration::default()))
        .map_err(|error| format!("上报 Running 失败：{error}"))?;
    agent2api_server::server::core::update::windows_service::notify_ready();

    // etc （ to STOP after ）
    let outcome = worker
        .join()
        .map_err(|_| "网关线程异常退出".to_string())?;

    let _ = status_handle.set_service_status(status(ServiceState::Stopped, 0, STOP_HINT));
    outcome
}

/// up 。`checkpoint` in pending 。
fn status(state: ServiceState, checkpoint: u32, wait_hint: Duration) -> ServiceStatus {
    status_with_exit(state, checkpoint, wait_hint, ServiceExitCode::NO_ERROR)
}

fn status_with_exit(
    state: ServiceState,
    checkpoint: u32,
    wait_hint: Duration,
    exit_code: ServiceExitCode,
) -> ServiceStatus {
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: state,
        // ： no has / 
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code,
        checkpoint,
        wait_hint,
        process_id: None,
    }
}
