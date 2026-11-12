//! update in 「」 this 1 need 。
//!
//! new 、 down in after （ reqwest as TLS 
//! default， not GitHub HTTPS request）。 this 
//! can ：
//! 1. current use （ after not ，must has ）
//! 2. after down path in directory
//! 3. ， and by need need （ before need file use ）
//!
//! path not is ：`run_installer` will 1 can file，
//! if path， etc to any 。
//!
//! ── （ after and ）────────────────────────────────
//! after by （ [`allowed_installer_suffixes`]）：Windows is NSIS 
//! `.exe`，macOS is `.dmg`，Linux is `.deb` / `.AppImage`。
//! not —— Windows need `runas` ，macOS is 「 → use app 
//! use 」，Linux then deb（ to ） and AppImage（file）。

use std::path::{Path, PathBuf};

use crate::gateway;

/// **** after （，** not **）， by 。
///
/// and after `update::version` is ** 1 **： after use 
/// Release ， this use 「 need this file is not is 」。 by 
/// `cfg!(target_os)` ， not will not 1 。
///
/// and Linux： when deb and AppImage ，
/// use is has ， after （ when deb ，
/// after `pick_installer`）。Windows / macOS has 1 。
pub const fn allowed_installer_suffixes() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["dmg"]
    } else if cfg!(target_os = "windows") {
        &["exe"]
    } else {
        // for ：AppImage after is `.AppImage`
        &["deb", "appimage"]
    }
}

/// down directory： and after `createUpdateManager` downloadDir 1 
pub fn download_dir() -> PathBuf {
    gateway::config_dir().join("updates")
}

/// path， `..` and after before 。
/// not in when canonicalize will failure， for in directory。
fn normalize(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return path.canonicalize().ok();
    }
    let parent = path.parent()?;
    let name = path.file_name()?;
    Some(parent.canonicalize().ok()?.join(name))
}

/// ：must in 、 after 、 and down directory。
///
/// 1 not can —— 「directory」 this 1 ， can anypath
/// ； after check， then can can use file。
pub fn verify_installer(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw.trim());
    if raw.trim().is_empty() {
        return Err("安装包路径为空".to_string());
    }
    if !path.is_absolute() {
        return Err("安装包路径必须是绝对路径".to_string());
    }

    let suffix = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let allowed = allowed_installer_suffixes();
    if !allowed.contains(&suffix.as_str()) {
        // after ：Linux up has （deb / AppImage），
        // 1 will 1 use as not supports
        let list = allowed.iter().map(|s| format!(".{s}")).collect::<Vec<_>>().join(" / ");
        return Err(format!("只允许运行 {list} 安装包"));
    }

    let target = normalize(&path).ok_or_else(|| "无法解析安装包路径".to_string())?;
    let root = normalize(&download_dir())
        .ok_or_else(|| "无法解析下载目录".to_string())?;
    if !target.starts_with(&root) {
        return Err("安装包不在受控的下载目录内，已拒绝执行".to_string());
    }
    if !target.is_file() {
        return Err("安装包不存在或不是文件".to_string());
    }
    Ok(target)
}

/// 。
///
/// Windows：`silent` as true when up NSIS `/S /R`（ + ）。
/// macOS：dmg no has etc parameter， to `open` （ use after 
/// app 「 use 」）—— this is dmg ， will 
/// use not to 。
/// Linux： by ， down `cfg` 。
///
/// not etc ： need need use ， in this will 
/// 1 。 use after is 。
pub fn launch_installer(path: &Path, silent: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        launch_installer_elevated(path, silent)
    }
    #[cfg(target_os = "macos")]
    {
        // dmg by ：`open` will to DiskImages and 。
        // silent in this no has for （），ignore not is supports。
        let _ = silent;
        std::process::Command::new("open")
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("打开磁盘映像失败: {error}"))
    }
    #[cfg(target_os = "linux")]
    {
        launch_installer_linux(path, silent)
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        let mut command = std::process::Command::new(path);
        if silent {
            command.arg("/S").arg("/R");
        }
        command
            .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动安装包失败: {error}"))
    }
}

/// Linux ：deb and AppImage is not 。
///
/// ── deb：must to ────────────────────────────────
/// `.deb` not can ， is ar 。 is to 
/// （`xdg-open` will to GNOME Software / Discover this ， use down 
/// ）。** not `dpkg -i`**： that need need root， is use —— 
/// `sudo` will 「update」 1 ， not this by ， no has 
/// can 。
///
/// `silent` for deb no has for （ not 、 when by use in 
/// ），ignore not is supports。
///
/// ── AppImage： can ───────────────────────────────────
/// AppImage is can file： up can as 「」（
/// when ，file）。 this ** not **： by use or 
/// （ new AppImage to old file up ）； old 
/// can file， will failure。
#[cfg(target_os = "linux")]
fn launch_installer_linux(path: &Path, silent: bool) -> Result<(), String> {
    let is_appimage = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("appimage"))
        .unwrap_or(false);

    if is_appimage {
        // AppImage need need can can （ down when default no has ）。
        // failure： 1 not can AppImage will to 
        // Permission denied， not in this 。
        if let Err(error) = make_executable(path) {
            return Err(format!("无法为 AppImage 添加可执行权限: {error}"));
        }
        let _ = silent;
        std::process::Command::new(path)
            .current_dir(path.parent().unwrap_or_else(|| Path::new(".")))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("启动 AppImage 失败: {error}"))
    } else {
        // deb： to （ as not use dpkg）
        let _ = silent;
        std::process::Command::new("xdg-open")
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| {
                format!("调用系统安装器失败（需要桌面环境提供 xdg-open）: {error}")
            })
    }
}

/// to file can （`chmod +x`）：AppImage before need 。
///
/// use `std::os::unix::fs::PermissionsExt` not is spawn `chmod`： 1 、
/// 1 PATH ， and failureReason is io error（ can ）。
#[cfg(target_os = "linux")]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path)?;
    let mut permissions = metadata.permissions();
    // in has up can （rw- → rwx）， its not 
    permissions.set_mode(permissions.mode() | 0o111);
    std::fs::set_permissions(path, permissions)
}

/// `canonicalize` to path shell can 。
///
/// `verify_installer` return is `canonicalize` ， Windows up 
/// `canonicalize` 1 `\\?\` verbatim before （UNC then is `\\?\UNC\...`）。
/// shell API not this ： `ShellExecuteW` for `\\?\C:\...\x.exe`
/// return SE_ERR_FNF(2)， `Command::spawn`（CreateProcess） can down ——
/// ShellExecute after this 1 is required ，otherwiseupdate will 「 not to 」。
#[cfg(windows)]
fn shell_path(path: &Path) -> String {
    let text = path.to_string_lossy().to_string();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return format!(r"\\{rest}");
    }
    match text.strip_prefix(r"\\?\") {
        Some(rest) => rest.to_string(),
        None => text,
    }
}

/// Windows up ：must `ShellExecuteW` + `runas` 。
///
/// ── Why not can use `Command::spawn`（ after to ）────────────────
/// perMachine requireAdministrator 。`Command::spawn` is 
/// CreateProcess， CreateProcess ** not will UAC**：failure and return
/// `ERROR_ELEVATION_REQUIRED (740)`。 has ShellExecute will shell path
/// UAC ， this is 1 can use 。
///
/// and `login.rs` `open_in_browser` （ not add）。
/// `SW_SHOWNORMAL` UAC after to in use before 。
#[cfg(windows)]
fn launch_installer_elevated(path: &Path, silent: bool) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    const SW_SHOWNORMAL: i32 = 1;
    /// use in UAC 「」：ShellExecuteW when return SE_ERR_ACCESSDENIED(5)
    /// （`ShellExecuteEx` then is GetLastError to ERROR_CANCELLED(1223)，
    /// —— 「request no 」）。
    const SE_ERR_ACCESSDENIED: isize = 5;
    const ERROR_CANCELLED: isize = 1223;

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(
            hwnd: *mut std::ffi::c_void,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> *mut std::ffi::c_void;
    }

    let to_wide = |text: &str| -> Vec<u16> {
        std::ffi::OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
    };
    // runas = ； requireAdministrator 
    let operation = to_wide("runas");
    let file = to_wide(&shell_path(path));
    // /S ，/R （ and before as 1 ）
    let parameters = if silent { to_wide("/S /R") } else { to_wide("") };
    let directory = path
        .parent()
        .map(|dir| to_wide(&shell_path(dir)))
        .unwrap_or_else(|| to_wide(""));

    // ShellExecuteW return <= 32 failure（ this is Win32 ）
    let result = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file.as_ptr(),
            parameters.as_ptr(),
            directory.as_ptr(),
            SW_SHOWNORMAL,
        )
    };
    let code = result as isize;
    if code > 32 {
        return Ok(());
    }
    // 5 / 1223 is 「 no 」： use in UAC 「」， or 
    // not （ and none can ）。 to 1 can ，
    // not is 1 error —— this is new down 1 failure。
    if code == SE_ERR_ACCESSDENIED || code == ERROR_CANCELLED {
        return Err(
            "安装已取消：没有获得管理员权限（UAC 确认被拒绝）。新版本安装在 Program Files，\
             安装程序需要管理员权限；可重新点击「安装并重启」，并在系统弹窗中选择「是」"
                .to_string(),
        );
    }
    Err(format!("启动安装包失败: ShellExecute 返回 {code}"))
}
