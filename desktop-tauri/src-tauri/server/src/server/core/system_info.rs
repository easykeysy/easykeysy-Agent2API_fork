//! （set「」 ）。
//!
//! this 「 this / this is 」 this 1 ，** not ** `ServerState`
//! （、、 this by `api::system_info` ）：core not 
//! axum， not this as 1 to 。
//!
//! ── Why need in after ────────────────────────────────
//! `workbuddyDesktop.platform` ****（windows / macos /
//! linux / web）， not 「」 and 「Windows server」——
//! in Windows up is 1 `platform: 'windows'`。 can is 
//! ****： when will `update::set_desktop_shell(true)`，
//! headless `--service` when service 。「」 this 
//! 1 by after to ， before 「current in 」（ use / / ）。
//!
//! 1 ：「 when 」 after ， use 
//! that when `platform` is `'web'`， after is 。 before will 
//! Docker —— this is must in after Reason。
//!
//! ── and systemd ──────────────────────────────────
//! · ：`/.dockerenv` in ， or `/proc/1/cgroup` docker /
//! kubepods / containerd / lxc。 and use is as before in when not in 
//! （Podman、containerd ）， after in cgroup v2 down can can 。
//! set `AGENT2API_DEPLOY=docker` when （ ）。
//! · systemd：`INVOCATION_ID` by systemd as ， before no has 。
//! · Windows ： can parameter `--service`， by bin in after 
//! [`set_service_mode`] （ no has can ）。
//!
//! ── path ────────────────────────────────────────────────
//! 「 use 」 is to **** use ： directory before `~`， its 
//! （`C:\Users\Bob\.agent2api` → `~\.agent2api`）。 in after not is before ，
//! is as 「」 to is after —— if in before ，
//! that ， use as 、 is path。
//!
//! Windows path need **ignore**： 1 directory can can `C:\Users\Bob` or 
//! `c:\users\bob` （ and `current_exe` not ）。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// when （ms Unix when ）， by `ServerState::bootstrap` 。
///
/// not use `OnceLock::get_or_init(now)` ： that 「 when 」 will from 
/// ** 1 set**， not is from —— 1 
/// will 「」， that 。
static STARTED_AT: OnceLock<i64> = OnceLock::new();

/// is by SCM（Windows ）（`--service`）。
static SERVICE_MODE: AtomicBool = AtomicBool::new(false);

/// when （ etc ； in bootstrap 1 ）。
pub fn record_start() {
    let _ = STARTED_AT.set(now_ms());
}

/// 「 by SCM 」（headless to `--service` when ）。
pub fn set_service_mode(on: bool) {
    SERVICE_MODE.store(on, Ordering::Relaxed);
}

/// when （ms）； when return 0。
pub fn started_at_ms() -> i64 {
    STARTED_AT.get().copied().unwrap_or(0)
}

/// already seconds； when unknown when return 0（「—」 not is 1 ）。
pub fn uptime_seconds() -> i64 {
    let started = started_at_ms();
    if started <= 0 {
        return 0;
    }
    ((now_ms() - started) / 1000).max(0)
}

/// ：`desktop` / `windows-service` / `docker` / `systemd` / `foreground`。
///
/// has ：（ when can can in ， use to 
/// is use ）； its is SCM（Windows not can can is ）； is ； after 
/// systemd and before 。
pub fn deployment() -> &'static str {
    if crate::server::core::update::desktop_shell() {
        return "desktop";
    }
    if SERVICE_MODE.load(Ordering::Relaxed) {
        return "windows-service";
    }
    if in_container() {
        return "docker";
    }
    if std::env::var("INVOCATION_ID")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return "systemd";
    }
    "foreground"
}

/// is in （module header ）。
pub fn in_container() -> bool {
    if std::env::var("AGENT2API_DEPLOY")
        .map(|value| value.trim().eq_ignore_ascii_case("docker"))
        .unwrap_or(false)
    {
        return true;
    }
    if Path::new("/.dockerenv").exists() {
        return true;
    }
    // cgroup v1 and v2 can can in ；， not 
    if let Ok(text) = std::fs::read_to_string("/proc/1/cgroup") {
        let lower = text.to_ascii_lowercase();
        if lower.contains("docker")
            || lower.contains("kubepods")
            || lower.contains("containerd")
            || lower.contains("lxc")
        {
            return true;
        }
    }
    false
}

/// （`std::env::consts::OS` ：windows / macos / linux）。
pub fn os() -> &'static str {
    std::env::consts::OS
}

/// ：`x86_64` → `x64`、`aarch64` → `arm64`， its 。
///
/// and update（`agent2api-server-linux-amd64`） 1 ， use 
/// when can and Release file for up 。
pub fn arch_label() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x64".to_string(),
        "aarch64" => "arm64".to_string(),
        "x86" => "x86".to_string(),
        other => other.to_string(),
    }
}

/// can （ `Windows 11 Pro 24H2 (build 26200)` / `Ubuntu 24.04 LTS`）。
///
/// not to when returnempty —— 「—」， not 。
pub fn os_version() -> String {
    #[cfg(windows)]
    {
        windows_version()
    }
    #[cfg(target_os = "linux")]
    {
        linux_version()
    }
    #[cfg(target_os = "macos")]
    {
        macos_version()
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        String::new()
    }
}

/// use directory（ when use before ）； not to returnempty PathBuf。
pub fn home_dir() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(|value| PathBuf::from(value.trim()))
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_default()
}

/// directory before `~`（module header「path」）。
pub fn mask_path(path: &Path) -> String {
    let text = path.to_string_lossy().to_string();
    let home = home_dir();
    if home.as_os_str().is_empty() {
        return text;
    }
    mask_text(&text, &home.to_string_lossy())
}

/// （ `to_string_lossy` after path use ）。
pub fn mask_text(text: &str, home: &str) -> String {
    // `\\?\C:\...` this before will starts_with ，
    let (prefix, body) = match text.strip_prefix(r"\\?\") {
        Some(rest) => (r"\\?\", rest),
        None => ("", text),
    };
    let home = home.trim_end_matches(['/', '\\']);
    if home.is_empty() {
        return text.to_string();
    }
    let body_matches = if cfg!(windows) {
        body.len() >= home.len() && body[..home.len()].eq_ignore_ascii_case(home)
    } else {
        body.starts_with(home)
    };
    if !body_matches {
        return text.to_string();
    }
    let rest = &body[home.len()..];
    // must in up ：`C:\Users\Bob2` not can `C:\Users\Bob` in 
    if !rest.is_empty() && !rest.starts_with(['/', '\\']) {
        return text.to_string();
    }
    let rest = rest.trim_start_matches(['/', '\\']);
    if rest.is_empty() {
        format!("{prefix}~")
    } else {
        format!("{prefix}~{}{rest}", std::path::MAIN_SEPARATOR)
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

// ─── ───────────────────────────────────────

#[cfg(windows)]
fn windows_version() -> String {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// 1 REG_SZ （ not in / failurereturn None）。
    fn read(name: &str) -> Option<String> {
        let sub = wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
        let value = wide(name);
        let mut size = 0u32;
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(sub.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            )
        };
        if status.is_err() || size == 0 {
            return None;
        }
        let mut buffer = vec![0u16; (size as usize) / 2 + 1];
        let mut got = size;
        let status = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                PCWSTR(sub.as_ptr()),
                PCWSTR(value.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut got),
            )
        };
        if status.is_err() {
            return None;
        }
        let len = (got as usize) / 2;
        let text = String::from_utf16_lossy(&buffer[..len]);
        let text = text.trim_end_matches('\0').trim().to_string();
        (!text.is_empty()).then_some(text)
    }

    let build = read("CurrentBuildNumber").unwrap_or_default();
    let mut product = read("ProductName").unwrap_or_default();
    let display = read("DisplayVersion").unwrap_or_default();

    // Windows 11 ProductName "Windows 10 ..."（ already 
    // ）。 use build ：22000 is Windows 11。
    if let Ok(number) = build.parse::<u32>() {
        if number >= 22000 && product.starts_with("Windows 10") {
            product = product.replacen("Windows 10", "Windows 11", 1);
        }
    }

    let mut parts: Vec<String> = Vec::new();
    if !product.is_empty() {
        parts.push(product);
    }
    if !display.is_empty() {
        parts.push(display);
    }
    if !build.is_empty() {
        parts.push(format!("build {build}"));
    }
    parts.join(" ")
}

#[cfg(target_os = "linux")]
fn linux_version() -> String {
    let Ok(text) = std::fs::read_to_string("/etc/os-release") else {
        return String::new();
    };
    // PRETTY_NAME（`PRETTY_NAME="Ubuntu 24.04.1 LTS"`）， no has NAME + VERSION_ID
    let field = |key: &str| -> Option<String> {
        text.lines()
            .find_map(|line| line.strip_prefix(&format!("{key}=")))
            .map(|value| value.trim().trim_matches('"').to_string())
            .filter(|value| !value.is_empty())
    };
    if let Some(pretty) = field("PRETTY_NAME") {
        return pretty;
    }
    match (field("NAME"), field("VERSION_ID")) {
        (Some(name), Some(version)) => format!("{name} {version}"),
        (Some(name), None) => name,
        _ => String::new(),
    }
}

#[cfg(target_os = "macos")]
fn macos_version() -> String {
    // `/System/Library/CoreServices/SystemVersion.plist` is XML，
    // ProductVersion <string> can 。 not spawn `sw_vers`： spawn
    // in will （ and Windows that to 1 ）。
    let Ok(text) = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist")
    else {
        return String::new();
    };
    let Some(index) = text.find("<key>ProductVersion</key>") else {
        return String::new();
    };
    let rest = &text[index..];
    let Some(start) = rest.find("<string>") else {
        return String::new();
    };
    let rest = &rest[start + "<string>".len()..];
    let Some(end) = rest.find("</string>") else {
        return String::new();
    };
    let version = rest[..end].trim();
    if version.is_empty() {
        String::new()
    } else {
        format!("macOS {version}")
    }
}
