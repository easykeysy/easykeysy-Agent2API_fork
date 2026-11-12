//! after （ HTTP ）。
//!
//! ── and old ──────────────────────────────────────
//! old is 「 3065， no in node.exe + server.cjs」：
//! can use use `npm start` ， in when 。
//!
//! in is HTTP （ `crate::server`），：
//! 1. **must bind success**。「 to already use use 」 ——
//! can in ， use etc API to 1 
//! not （ can can not ， as none from ）。 use when 
//! and use old ，「success、 is old 」。
//! 2. ** when not **， 1 axum 
//! （ not has 「 not node 」failure ——
//! this is need 1 ）。
//!
//! for not ：`ensure_ready(app)` and `shutdown(&AppState)` 
//! and （success/failurereturn、 etc ） and old 1 ，lib.rs / tray.rs /
//! commands.rs use 1 not use 。
//!
//! ── （1.0.x → 1.1.0）───────────────────────────────
//! already old use after ， has need in ** new ** when ：
//! 1. old node 3065 → `reclaim_port_from_legacy`
//! in is 「 old 」 after （）；
//! 2. old directory resources\node.exe（88MB）+ server.cjs →
//! `cleanup_legacy_resources` in after 。
//! in after 1 has ，。

#[cfg(windows)]
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use tauri::{AppHandle, Manager};

// and use Win32 API Reason Cargo.toml windows-sys
// ：（taskkill / powershell） and create as will 
// in 。 this by need ， Windows not this 。
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_SUCCESS, INVALID_HANDLE_VALUE,
};
#[cfg(windows)]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP6TABLE_OWNER_PID, MIB_TCP6ROW_OWNER_PID, MIB_TCP_STATE_LISTEN,
    MIB_TCPROW_OWNER_PID, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_CLASS, TCP_TABLE_OWNER_PID_LISTENER,
};
#[cfg(windows)]
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
#[cfg(windows)]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_TERMINATE,
};

use crate::gateway::{local_client, proxy_port};
use crate::port_conflict::{ConflictKind, Occupant, PortConflict, StartupFailure};
use crate::settings;
use crate::server;
use crate::state::AppState;

/// etc limit。
///
/// old to node 12 seconds； no has and ，
/// bind + response is ms ，5 seconds「/」 this 。
const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);
const HEALTH_TIMEOUT: Duration = Duration::from_millis(1500);
const HEALTH_INTERVAL: Duration = Duration::from_millis(100);

/// is already （ can to /health response）。
///
/// when `commands::backend_status` use （ before 「 already 」），
/// as 、 not ： can to 2xx 。
pub async fn is_ready(port: u16) -> bool {
    let Ok(client) = local_client(HEALTH_TIMEOUT) else {
        return false;
    };
    matches!(
        client.get(format!("http://127.0.0.1:{port}/health")).send().await,
        Ok(response) if response.status().is_success()
    )
}

/// after available： → → accept → 。
///
/// failurereturn [`StartupFailure`]（ use to UI `backend:error` 
/// and `backend_status` ）， not panic —— release profile is panic=abort。
///
/// Why need not is 1 ： has Reason（ use 
/// vs in ）， to not ， need OS error
/// has in bind failure that 1 to 。 `crate::port_conflict`。
pub async fn ensure_ready(app: &AppHandle) -> Result<(), StartupFailure> {
    let port = proxy_port();

    // ── ：（default） or （，issue #48）──────
    // and 「bind before must to 」 set， settings::load。
    // after in down bootstrap after by 1 set。
    let lan = settings::load();
    let host = if lan.lan_access {
        std::net::IpAddr::from([0, 0, 0, 0])
    } else {
        std::net::IpAddr::from([127, 0, 0, 1])
    };

    // ： and config in this （bootstrap ）。
    // failure 1 return：`bootstrap` in 「 this directory no has 」（
    // ， that ） or 「 schema 」（ is to old ，
    // use old will after delete） when return Err。 before down will new config
    // directory、none； after must —— this must in 
    // not is error down 。
    // default： API not 127.0.0.1； use 
    // 「」 after 0.0.0.0， when by down 
    // （headless by agent2api-server by AGENT2API_HOST ）。
    // and will ， not can IPC and etc 。
    let mut state = tauri::async_runtime::spawn_blocking(move || {
        server::ServerState::bootstrap(port, host)
    })
    .await
    .map_err(|error| StartupFailure::other(format!("网关初始化任务失败: {error}")))?
    .map_err(StartupFailure::other)?;

    // and use 1 will 。
    server::access::attach_db(state.db().cloned());

    // （Tauri ） in ：OAuth can ，
    // and —— （0.0.0.0） after 。
    state.local_panel = true;

    // ── （Linux up 「 update」 ）──────────────
    // Linux and / headless is ** 1 **， after updatecheck
    // must「 has no has GUI 」 can ： deb / AppImage 
    // to ，server（ `update::version::desktop_shell`）。
    // not when as and before 1 （ not ）， this need 。
    server::core::update::set_desktop_shell(true);

    // ── （headless ， bin/agent2api-server）──
    // 1 ，「 1 Key no has → 」 not ：
    //   · /api/*： need will or Key（panel_auth ）；
    // no by （，fail-closed）。
    // · /v1/*： no has enabled Key when （ not for ）。
    // path down 「、 after 、」， to this already ；
    // for 。
    if lan.lan_access {
        server::access::set_panel_gate(true);
        if !server::config::current().active_api_keys().is_empty() {
            server::logging::log("[Security]", " already ：API Key already enabled");
        } else {
            server::access::set_v1_fail_closed(true);
            server::logging::log(
                "[Security]",
                "⚠️ already not configured API Key：/v1/* when ， in 「 Key」create 1 ",
            );
        }
        // （optional）： ui_dir in ui/ directory
        //（ bridge）， http://<IP>:<> ；
        // not then —— API， by Tauri 。
        if lan.lan_panel {
            match resolve_ui_dir(app) {
                Some(dir) => state.set_ui_dir(dir),
                None => server::logging::log(
                    "[Server]",
                    "⚠️ ：none（ can ），API not ",
                ),
            }
        }
    }

    // already use ： 1 use is 「 old node 」（），
    // ； not （ / use ）。
    // this not use （module header）—— use etc API
    // to 1 not old 。
    if is_ready(port).await && !reclaim_port_from_legacy(port).await {
        // up has failure：「 use 」 this 1 ，
        // 1 and to （ can when use not netstat）
        let occupant = occupant_of(port);
        return Err(StartupFailure::port(PortConflict::new(
            ConflictKind::Occupied,
            port,
            occupant,
            " up already has in response",
        )));
    }
    let shutdown_tx = server::start(&state).map_err(|conflict| {
        // bind failure： up 「」 to （ is ， backend module header）
        StartupFailure::port(conflict.with_occupant(occupant_of(port)))
    })?;

    // state： after failure，path can ，
    // not will down 1 「 already no 」 
    {
        let app_state = app.state::<AppState>();
        let mut guard = app_state
            .backend
            .lock()
            .map_err(|_| StartupFailure::other("unavailable"))?;
        guard.shutdown_tx = Some(shutdown_tx);
        guard.port = port;
    }

    // ：bind success not by available（e.g. by ），
    // by old 「 /health」 to available
    let deadline = std::time::Instant::now() + STARTUP_TIMEOUT;
    while std::time::Instant::now() < deadline {
        if is_ready(port).await {
            let scope = if lan.lan_access { "， already " } else { "" };
            server::logging::log("[Server]", &format!(" already （127.0.0.1:{port}{scope}）"));
            crate::panel_session::ensure_local_access().map_err(StartupFailure::other)?;
            // up 1 failure： that is 「 before not 」，
            // will in when 「 use 」——
            // use ， to this will as no 。
            if let Ok(mut guard) = app.state::<AppState>().backend.lock() {
                guard.failure = None;
            }
            // ： old directory node.exe / server.cjs。
            // in after —— available is in ，。
            cleanup_legacy_resources();
            return Ok(());
        }
        tokio::time::sleep(HEALTH_INTERVAL).await;
    }
    Err(StartupFailure::other(format!(
        " when （{port} ）： already check，"
    )))
}

/// use `ui/` directory（ and headless 1 file）。
///
/// release：tauri.conf.json `../ui` bundle resources —— will 
/// use directory `..` `_up_`（ tauri-utils `resources.rs` 
/// path then ）， after in directory `_up_/ui` down （Windows NSIS as 
/// `<install>/_up_/ui`）。 `ui/` will not to ，
/// 「 no 」—— 1 ， this in this 。
/// debug： not ，directory（`tauri dev` when ui-islands 
/// in that ）。directory not in return None —— is 「 no 」，
/// API （headless for ui is this ）。
fn resolve_ui_dir(app: &AppHandle) -> Option<std::path::PathBuf> {
    if cfg!(debug_assertions) {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui");
        return dir.is_dir().then_some(dir);
    }
    let base = app.path().resource_dir().ok()?;
    [base.join("_up_").join("ui"), base.join("ui")]
        .into_iter()
        .find(|dir| dir.is_dir())
}

/// up （PID、、path， and ）。
///
/// not to return None —— this can can is as not （ ）， can can as 
/// no （ is this ：`netstat` emptynone 1 ）。
/// not this 1 「 use 」。
pub fn occupant_of(port: u16) -> Option<Occupant> {
    let (pid, path) = listener_process(port)?;
    let name = Path::new(&path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let self_pid = std::process::id();
    Some(Occupant {
        pid,
        name,
        path: path.clone(),
        is_self: pid == self_pid,
        system: inside_system_dir(&path),
    })
}

/// use （ its ）， and in 。
///
/// ** `occupant` that PID**， not 「 by none」： in 
/// when ，none will 1 none 。 before by use （）
/// and path to use —— this not ， is 。
///
/// return after ：`Ok(())` already can bind（ can ）；
/// Err is in （e.g. —— that no has can ）。
pub fn end_occupant(port: u16, occupant: &Occupant) -> Result<(), PortConflict> {
    // ： etc in ， this is use none as ，
    // 1 （ in this down not this to this by ， this is ）。
    if occupant.is_self {
        return Err(PortConflict::new(
            ConflictKind::Other,
            port,
            None,
            " this is ， not can ",
        ));
    }
    if occupant.system {
        return Err(PortConflict::new(
            ConflictKind::Other,
            port,
            None,
            " this ， not by ",
        ));
    }

    if !kill_process_tree(occupant.pid) {
        return Err(PortConflict::new(
            ConflictKind::Other,
            port,
            None,
            &format!(
                "failure（PID {}，{}）： can can not ，",
                occupant.pid, occupant.name
            ),
        ));
    }

    server::logging::log(
        "[Server]",
        &format!(
            " already use （PID {}，{}）",
            occupant.pid, occupant.name
        ),
    );
    Ok(())
}

/// after etc ： after ， to 、
/// this 1 。
///
/// return Ok(()) already can bind；Err is in （current use ）。
pub fn wait_port_released(port: u16) -> Result<(), PortConflict> {
    const RELEASE_TIMEOUT: Duration = Duration::from_secs(5);
    let deadline = std::time::Instant::now() + RELEASE_TIMEOUT;
    loop {
        match server::probe_port(port) {
            Ok(()) => return Ok(()),
            Err(conflict) => {
                if std::time::Instant::now() >= deadline {
                    // when ： when to use up ， can 
                    return Err(conflict.with_occupant(occupant_of(port)));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// ： from 「 old node 」。
///
/// old （1.0.x） NSIS when ， node will 
/// 3065 —— new bind failure， use can 。
/// this is up 「 when not Node 」 path， this 。
///
/// ** 1 must** will ， 1 not return false
/// （ use use ）：
/// 1. up /health and `product == "WorkBuddy"` —— 
/// ； 3065 when not ；
/// 2. this ， its can file** use directory**
/// （ old resources\node.exe）。 use `npm start`
/// node in directory —— 、 not ；
/// 3. success， and in （ old /health not ）。
async fn reclaim_port_from_legacy(port: u16) -> bool {
    if !health_reports_workbuddy(port).await {
        return false;
    }
    let Some((pid, exe_path)) = listener_process(port) else {
        server::logging::log(
            "[Server]",
            &format!(" {port} up old ， not to ， need "),
        );
        return false;
    };
    if !inside_install_dir(&exe_path) {
        server::logging::log(
            "[Server]",
            &format!(
                " {port} WorkBuddy use ， not in use directory（{exe_path}），\
                  not —— can can is use npm start "
            ),
        );
        return false;
    }

    server::logging::log(
        "[Server]",
        &format!(" to old in （PID {pid}）， in （）…"),
    );
    let killed = kill_process_tree(pid);
    if !killed {
        server::logging::log("[Server]", " old failure， need ");
        return false;
    }

    // etc ： after ， to 、
    // this 5 seconds
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        if !is_ready(port).await {
            server::logging::log("[Server]", " already old （）");
            return true;
        }
        if std::time::Instant::now() >= deadline {
            server::logging::log("[Server]", " old already ");
            return false;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

/// （ etc `taskkill /PID <pid> /T /F`）。
///
/// use Toolhelp + TerminateProcess not is spawn taskkill： after will in 
/// down `taskkill` and create as ， and 「」 in 
/// for （ Cargo.toml in windows-sys ）。
#[cfg(windows)]
fn kill_process_tree(pid: u32) -> bool {
    // ： 1 ， will （ PID to 
    // 1 already ）， not this 。
    // and taskkill /T 1 ：、 after 。
    let mut descendants = descendant_pids(pid);
    descendants.reverse();
    for child in descendants {
        // failure not ： old taskkill /T 
        // 「 already / not not 」 this 。
        terminate_process(child);
    }
    terminate_process(pid)
}

#[cfg(not(windows))]
fn kill_process_tree(_pid: u32) -> bool {
    // Windows ；file can 。
    false
}

/// `root` after PID（ not `root` ）。
#[cfg(windows)]
fn descendant_pids(root: u32) -> Vec<u32> {
    // 1 ， when ； need is 「 already 、
    // 」。
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        // not to can 「」—— can can no has ，
        // this not can failure。
        return Vec::new();
    }

    // dwSize must，otherwise Process32FirstW will failure。
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    let mut children_by_parent: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut has_entry = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while has_entry {
        children_by_parent
            .entry(entry.th32ParentProcessID)
            .or_default()
            .push(entry.th32ProcessID);
        has_entry = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };

    // HashSet ：PID use can can ， 1 。
    // can can root ，mustexcluding —— otherwise root will after 1 ，
    // after that terminate_process(root) will as already as failure。
    let mut seen: HashSet<u32> = HashSet::new();
    let mut pending = vec![root];
    let mut result = Vec::new();
    while let Some(current) = pending.pop() {
        let Some(children) = children_by_parent.get(&current) else {
            continue;
        };
        for &child in children {
            if child == root || child == 0 || !seen.insert(child) {
                continue;
            }
            result.push(child);
            pending.push(child);
        }
    }
    result
}

/// ；failure（ not / ）return false。
#[cfg(windows)]
fn terminate_process(pid: u32) -> bool {
    let handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    if handle.is_null() {
        return false;
    }
    let ok = unsafe { TerminateProcess(handle, 1) };
    unsafe { CloseHandle(handle) };
    ok != 0
}

/// this （PID + can filepath）。
///
/// use GetExtendedTcpTable `powershell Get-NetTCPConnection`： is 
/// （powershell + create）， 1 。
/// use TCP_TABLE_OWNER_PID_LISTENER（）， ALL 
/// already 。IPv4 not to IPv6 —— and old is IPv4，
/// IPv6 「 use use ::1 old 」 this 。
#[cfg(windows)]
fn listener_process(port: u16) -> Option<(u32, String)> {
    let pid = listener_pid(AF_INET as u32, port).or_else(|| listener_pid(AF_INET6 as u32, port))?;
    let path = process_image_path(pid)?;
    Some((pid, path))
}

#[cfg(not(windows))]
fn listener_process(_port: u16) -> Option<(u32, String)> {
    // Windows ；file can 。
    None
}

/// in use `port` PID。
#[cfg(windows)]
fn listener_pid(family: u32, port: u16) -> Option<u32> {
    let buffer = tcp_listener_table(family)?;

    if family == AF_INET as u32 {
        let table = buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_PID;
        // table has 1 ， by dwNumEntries to ，
        // by （IPv4 is 6 u32， and 1 ）。
        let rows = unsafe {
            std::slice::from_raw_parts(
                std::ptr::addr_of!((*table).table).cast::<MIB_TCPROW_OWNER_PID>(),
                (*table).dwNumEntries as usize,
            )
        };
        return rows
            .iter()
            .find(|row| {
                row.dwState == MIB_TCP_STATE_LISTEN as u32 && tcp_port(row.dwLocalPort) == port
            })
            .map(|row| row.dwOwningPid);
    }

    let table = buffer.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID;
    // IPv4：IPv6 by MIB_TCP6ROW_OWNER_PID ， ntohs。
    let rows = unsafe {
        std::slice::from_raw_parts(
            std::ptr::addr_of!((*table).table).cast::<MIB_TCP6ROW_OWNER_PID>(),
            (*table).dwNumEntries as usize,
        )
    };
    rows.iter()
        .find(|row| row.dwState == MIB_TCP_STATE_LISTEN as u32 && tcp_port(row.dwLocalPort) == port)
        .map(|row| row.dwOwningPid)
}

/// （`TCP_TABLE_OWNER_PID_LISTENER`） 。
///
/// is ， can use ： 1 empty（return
/// ERROR_INSUFFICIENT_BUFFER， when need size）， by 
/// 。 use `Vec<u32>` not is `Vec<u8>`： u32 
/// （ for need 4）， use u32 can when for is 。
#[cfg(windows)]
fn tcp_listener_table(family: u32) -> Option<Vec<u32>> {
    let class: TCP_TABLE_CLASS = TCP_TABLE_OWNER_PID_LISTENER;
    let mut size: u32 = 0;
    // ：null failure， size
    let probe = unsafe { GetExtendedTcpTable(std::ptr::null_mut(), &mut size, 0, family, class, 0) };
    if probe != ERROR_INSUFFICIENT_BUFFER || size == 0 {
        return None;
    }

    let mut buffer = vec![0u32; size as usize / 4 + 1];
    // by size（ ）， API
    // as 
    let mut capacity = std::mem::size_of_val(buffer.as_slice()) as u32;
    let filled =
        unsafe { GetExtendedTcpTable(buffer.as_mut_ptr().cast(), &mut capacity, 0, family, class, 0) };
    if filled != ERROR_SUCCESS {
        return None;
    }
    Some(buffer)
}

/// is 「 in u32 」 ， 16 ntohs。
#[cfg(windows)]
fn tcp_port(raw: u32) -> u16 {
    u16::from_be((raw & 0xFFFF) as u16)
}

/// can file for path（ not to pathreturn None， not to ）。
#[cfg(windows)]
fn process_image_path(pid: u32) -> Option<String> {
    // PROCESS_QUERY_LIMITED_INFORMATION：Vista after ， for 「 use 
    // not 」 （ old is this ） can success，path。
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        return None;
    }

    let mut buffer = [0u16; 1024];
    let mut length = buffer.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) };
    unsafe { CloseHandle(handle) };
    if ok == 0 {
        return None;
    }

    // success when length is not NUL 。
    Some(String::from_utf16_lossy(&buffer[..length as usize]))
}

/// up is is WorkBuddy （ /health product ）。
async fn health_reports_workbuddy(port: u16) -> bool {
    let Ok(client) = local_client(HEALTH_TIMEOUT) else {
        return false;
    };
    let Ok(response) = client.get(format!("http://127.0.0.1:{port}/health")).send().await
    else {
        return false;
    };
    let Ok(payload) = response.json::<serde_json::Value>().await else {
        return false;
    };
    payload.get("product").and_then(|value| value.as_str()) == Some("WorkBuddy")
}

/// can file is use directory。
///
/// canonicalize before ：、8.3 path and 
/// `\\?\` verbatim before （pathreturn is path，
/// canonicalize after 1 ，starts_with can ）。
fn inside_install_dir(candidate: &str) -> bool {
    let Ok(current) = std::env::current_exe() else {
        return false;
    };
    let Ok(current) = current.canonicalize() else {
        return false;
    };
    let Some(install_dir) = current.parent() else {
        return false;
    };
    let Ok(target) = std::fs::canonicalize(candidate) else {
        // can can in this —— file no not 
        return false;
    };
    target.starts_with(install_dir)
}

/// can file is directory（`%SystemRoot%`）。
///
/// use down 「」 by ：svchost this can can ，
/// not this by 。 not to directory when return true（ can 1 ，
/// not need —— this is not for ）。
fn inside_system_dir(candidate: &str) -> bool {
    let Ok(root) = std::env::var("SystemRoot") else {
        return true;
    };
    let Ok(root) = std::fs::canonicalize(root) else {
        return true;
    };
    let Ok(target) = std::fs::canonicalize(candidate) else {
        return false;
    };
    target.starts_with(root)
}

/// old directory Node when 。
///
/// NSIS file， old resources\node.exe（ 88MB）
/// and resources\server.cjs will 1 in directory。 this in after 1 ；
/// deletefailure（ use / not ）， down 。
///
/// current can file directorydirectory（ not is Tauri resource_dir，
/// after can can `\\?\` verbatim before ， and up path not 1 ）。
/// （target/debug down ） not in resources directory，。
fn cleanup_legacy_resources() {
    let Some(install_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    else {
        return;
    };
    let resources = install_dir.join("resources");
    for name in ["node.exe", "server.cjs"] {
        let target = resources.join(name);
        if !target.exists() {
            continue;
        }
        match std::fs::remove_file(&target) {
            Ok(()) => {
                server::logging::log("[Server]", &format!(" already old file（）: {name}"))
            }
            Err(error) => {
                server::logging::log(
                    "[Server]",
                    &format!(" old {name} failure（ down ）: {error}"),
                )
            }
        }
    }
}

/// when ： 1 ， axum 。
///
/// and old 1 （、 `&AppState`）， lib.rs use and 
/// commands.rs / tray.rs none need 。** etc **： use when already take，
/// return —— this 1 and old `child.take()` as 1 。
pub fn shutdown(state: &AppState) {
    crate::startup::stop();
    crate::panel_session::clear();
    // when to ： in before （ for Node closeAll 
    // `autoCheckin.stop(); closeDispatchers();`）。
    //
    // ** in etc before **： is HTTP after 
    // （ by auto_checkin has ）， need in this ； if in down 
    // `shutdown_tx.take()` after ， use （ already → already take）
    // will ， down 1 in 。stop is etc （task after 
    // return）， not 「 use none use 」 has 。
    server::core::auto_checkin::stop_global();

    let Ok(mut guard) = state.backend.lock() else {
        return;
    };
    if let Some(sender) = guard.shutdown_tx.take() {
        // can can already when 1 （failure）—— that ，none need 
        let sent = sender.send(()).is_ok();
        server::logging::log(
            "[Server]",
            if sent {
                " already ， in "
            } else {
                " already （none need ）"
            },
        );
        // from ：Node when is 1 ， this 1 can 。
        // not 「 etc 」——shutdown is and can can from path use ，
        // etc will 。
        server::logging::console_line("[Server]", " in …");
    }
}

/// already ； when return None（ after when use ）。
#[allow(dead_code)]
pub fn running_port(state: &AppState) -> Option<u16> {
    state
        .backend
        .lock()
        .ok()
        .and_then(|guard| guard.shutdown_tx.is_some().then_some(guard.port))
}
