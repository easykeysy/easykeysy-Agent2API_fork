//! old 「current use 」（1.x） ：directory、、，
//! and old 。
//!
//! ── Whymust in use when ， not is NSIS ──────────────
//! from currentUser as perMachine after ，： when 
//! `%LOCALAPPDATA%` and HKCU to is ****， not to not 
//! that use old 。 new use when is use up down ，`%LOCALAPPDATA%`
//! and HKCU current use ， use directory down file not need need —— this is 1 
//! can to old 。
//!
//! ── 「 is 」 ────────────────────────────────────
//! 1 must， 1 not （ can down ， not 
//! use its ）：
//! directory must in （`agent2api.exe` / `workbuddy-proxy-desktop.exe`）；
//! and DisplayName need etc / use ， and 
//! need in for old directory、 this directory already not in ；
//! file etc ， and .lnk old directory for path；
//! etc ， old directorypath， and to can file already not in 。
//! ，**current in directory not will **（ use new old directory when not can ）。
//!
//! ── failure， down ────────────────────────────────────
//! deletefailure（file use 、 not ） not 。 is etc ： not in when 
//! allempty，。
//!
//! ── is 1 1 （ `autostart_needs_refresh`）──────────
//! old to `%LOCALAPPDATA%` down old directory、 use old ； new in Program
//! Files and use productName ， not ， old that will 。
//!
//! ── Why use eprintln! ──────────────────────────────────────
//! use in setup and in after ， when can can no ；
//! `server::logging::log` is eprintln!， this use 。

#[cfg(windows)]
pub use imp::{autostart_needs_refresh, cleanup_legacy_user_install};

/// Windows ： has Windows， is as its 
/// can （ and `update.rs` `launch_installer` 1 ）
#[cfg(not(windows))]
pub fn cleanup_legacy_user_install() {}

/// Windows etc （ by ，none need new ）
#[cfg(not(windows))]
pub fn autostart_needs_refresh(_current_name: &str) -> bool {
    false
}

#[cfg(windows)]
mod imp {
    //! Windows 。

    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};

    use super::registry::{self, HKEY_CURRENT_USER};

    /// use 。NSIS in currentUser down this when use 
    /// directory（`%LOCALAPPDATA%\<>`）、 and 。
    const PRODUCT_NAMES: [&str; 2] = ["Agent2API", "WorkBuddy 本地代理"];

    /// old directory can can file。
    ///
    /// `workbuddy-proxy-desktop.exe` is use ：Cargo.toml package
    /// name 1 is `workbuddy-proxy-desktop`， tauri.conf.json no 
    /// mainBinaryName，NSIS MAINBINARYNAME and up exe is （ already by 
    /// installer.nsi and directory for ）。 in tauri.conf.json mainBinaryName
    /// `agent2api`， new directory is `agent2api.exe`。
    ///
    /// ： for is ****directory， is 1 is 
    /// ， not this as 。
    const BINARY_NAMES: [&str; 2] = ["workbuddy-proxy-desktop.exe", "agent2api.exe"];

    /// current use （NSIS currentUser this ）
    const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall";

    /// （tauri-plugin-autostart auto-launch use 1 ）
    const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    /// 「」 enabled： and Run 1 
    const STARTUP_APPROVED_KEY: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

    /// old use 。、 etc 、failure。
    pub fn cleanup_legacy_user_install() {
        let dirs = legacy_dirs();
        if dirs.is_empty() {
            return;
        }
        // directory， after / / by 「directory is already not in 」，
        // available old （、）
        let cleaned: Vec<(&'static str, PathBuf, bool)> = dirs
            .into_iter()
            .map(|(name, dir)| {
                let removed = remove_legacy_dir(&dir);
                (name, dir, removed)
            })
            .collect();

        cleanup_shortcuts(&cleaned);
        cleanup_uninstall_keys(&cleaned);
        cleanup_autostart_entries(&cleaned);
    }

    /// is need need by current exe path（ as `current_name`）。
    ///
    /// ：1.x in `%LOCALAPPDATA%\< old >` down 、 use old ；
    /// perMachine after new in Program Files， use productName。
    /// old to already delete directory（ by ）， new then ** not in ** ——
    /// `is_enabled()` will 「 no 」， use not set
    /// not will ，。 1 。
    ///
    /// （return true， etc and not use ）：
    /// 1. current Run not in ， or can file not is current exe；
    /// 2. 「」 no has for this disabled —— use 
    /// not this （disabled in when Run in ， 1 
    /// will 「 need need 」）。
    ///
    /// this 、 not setfile： use already use need 。
    pub fn autostart_needs_refresh(current_name: &str) -> bool {
        if startup_disabled(current_name) {
            return false;
        }
        let Some(value) = registry::read_string(HKEY_CURRENT_USER, RUN_KEY, current_name) else {
            return true;
        };
        match (exe_path_from(&value), std::env::current_exe()) {
            // is current this can file： already is new ，none need 
            (Some(registered), Ok(current)) => {
                !same_path(&registered, &current)
            }
            // not can file（ ）： 1 1 invalid
            (None, _) => true,
            // not to current exe path（ up not will ）： not 
            (Some(_), Err(_)) => false,
        }
    }

    /// path is to 1 file（ not ； can canonicalize when use 
    /// 8.3 path and `\\?\` before ）
    fn same_path(left: &Path, right: &Path) -> bool {
        if let (Ok(a), Ok(b)) = (left.canonicalize(), right.canonicalize()) {
            return a == b;
        }
        left.to_string_lossy().eq_ignore_ascii_case(&right.to_string_lossy())
    }

    /// old directory：`%LOCALAPPDATA%\<>`， for 
    fn legacy_dirs() -> Vec<(&'static str, PathBuf)> {
        let Some(local) = env_path("LOCALAPPDATA") else {
            return Vec::new();
        };
        PRODUCT_NAMES.iter().map(|name| (*name, local.join(name))).collect()
    }

    /// directory（`%LOCALAPPDATA%` / `%APPDATA%` / `%USERPROFILE%`）
    fn env_path(name: &str) -> Option<PathBuf> {
        let value = std::env::var(name).ok()?;
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    }

    /// directory is has ； no has as not is directory
    fn looks_like_our_install(dir: &Path) -> bool {
        BINARY_NAMES.iter().any(|name| dir.join(name).is_file())
    }

    /// current is in this directory（ or use new old directory when not can ）
    fn running_inside(dir: &Path) -> bool {
        let (Ok(exe), Ok(target)) = (std::env::current_exe(), dir.canonicalize()) else {
            return false;
        };
        exe.canonicalize().map(|exe| exe.starts_with(&target)).unwrap_or(false)
    }

    /// delete 1 old directory；return「 this directory is already not in 」（deletesuccess or no has ）
    fn remove_legacy_dir(dir: &Path) -> bool {
        if !dir.exists() {
            return true;
        }
        if !dir.is_dir() {
            eprintln!("[Cleanup] 跳过旧安装路径（同名但不是目录）: {}", dir.display());
            return false;
        }
        if running_inside(dir) {
            eprintln!("[Cleanup] 跳过旧安装目录（当前程序正在其中运行）: {}", dir.display());
            return false;
        }
        if !looks_like_our_install(dir) {
            eprintln!(
                "[Cleanup] 跳过旧安装目录（未发现本产品主程序，可能不是本产品安装的）: {}",
                dir.display()
            );
            return false;
        }
        match std::fs::remove_dir_all(dir) {
            Ok(()) => {
                eprintln!(
                    "[Cleanup] 已清理旧用户级安装目录（新版装在 Program Files）: {}",
                    dir.display()
                );
                true
            }
            Err(error) => {
                eprintln!(
                    "[Cleanup] 清理旧安装目录失败（下次启动再试）: {} — {error}",
                    dir.display()
                );
                false
            }
        }
    }

    /// old ： `%APPDATA%\Microsoft\Windows\Start Menu\Programs\<>.lnk`
    /// and `%USERPROFILE%\Desktop\<>.lnk`。
    ///
    /// use 「file + .lnk old directory for path」， not .lnk 
    /// （ that need COM IShellLink，）：.lnk path UTF-16 
    /// in file， when ， use its not will 。
    fn cleanup_shortcuts(dirs: &[(&'static str, PathBuf, bool)]) {
        let mut roots: Vec<PathBuf> = Vec::new();
        if let Some(appdata) = env_path("APPDATA") {
            roots.push(appdata.join("Microsoft\\Windows\\Start Menu\\Programs"));
        }
        if let Some(profile) = env_path("USERPROFILE") {
            roots.push(profile.join("Desktop"));
            // to to OneDrive when （） in this ；
            // 1 ，file and not will 
            if let Some(onedrive) = env_path("OneDrive") {
                roots.push(onedrive.join("Desktop"));
            }
        }
        for root in roots {
            for (name, dir, removed) in dirs {
                // old directory in （ not or not is ）： is 1 ，
                if !removed {
                    continue;
                }
                let shortcut = root.join(format!("{name}.lnk"));
                if !shortcut.is_file() {
                    continue;
                }
                if !points_into(&shortcut, dir) {
                    eprintln!(
                        "[Cleanup] 跳过快捷方式（未指向旧安装目录，可能已被改过）: {}",
                        shortcut.display()
                    );
                    continue;
                }
                match std::fs::remove_file(&shortcut) {
                    Ok(()) => eprintln!("[Cleanup] 已清理旧快捷方式: {}", shortcut.display()),
                    Err(error) => eprintln!(
                        "[Cleanup] 清理旧快捷方式失败（下次启动再试）: {} — {error}",
                        shortcut.display()
                    ),
                }
            }
        }
    }

    /// .lnk is this directory for path（UTF-16LE ， not ）
    fn points_into(shortcut: &Path, dir: &Path) -> bool {
        let Ok(bytes) = std::fs::read(shortcut) else {
            return false;
        };
        let needle: Vec<u8> = dir.as_os_str().encode_wide().flat_map(u16::to_le_bytes).collect();
        if needle.is_empty() || bytes.len() < needle.len() {
            return false;
        }
        bytes.windows(needle.len()).any(|window| window.eq_ignore_ascii_case(&needle))
    }

    /// old 。 has 「directory already not in 」 when ：directory in （ not or not is ）
    /// this ， use can use 。
    fn cleanup_uninstall_keys(dirs: &[(&'static str, PathBuf, bool)]) {
        for (name, dir, removed) in dirs {
            if !removed {
                continue;
            }
            let sub_key = format!("{UNINSTALL_KEY}\\{name}");
            // not ：DisplayName must etc （ can can is ）
            let Some(display) = registry::read_string(HKEY_CURRENT_USER, &sub_key, "DisplayName")
            else {
                continue;
            };
            if display.trim() != *name {
                eprintln!("[Cleanup] 跳过卸载注册项（DisplayName 不匹配）: {sub_key} → {display}");
                continue;
            }
            // must in this old directory； not to 「 no 」——
            // and DisplayName already is 
            if let Some(recorded) = recorded_install_dir(&sub_key) {
                if !path_within(&recorded, dir) {
                    eprintln!(
                        "[Cleanup] 跳过卸载注册项（记录的安装位置不在旧目录内）: {sub_key} → {}",
                        recorded.display()
                    );
                    continue;
                }
            }
            if registry::delete_key(HKEY_CURRENT_USER, &sub_key) {
                eprintln!("[Cleanup] 已清理旧卸载注册项: {sub_key}");
            } else {
                eprintln!("[Cleanup] 清理旧卸载注册项失败（下次启动再试）: {sub_key}");
            }
        }
    }

    /// ： InstallLocation， its from UninstallString 
    fn recorded_install_dir(sub_key: &str) -> Option<PathBuf> {
        if let Some(text) = registry::read_string(HKEY_CURRENT_USER, sub_key, "InstallLocation") {
            let path = PathBuf::from(trim_quotes(&text));
            if !path.as_os_str().is_empty() {
                return Some(path);
            }
        }
        let command = registry::read_string(HKEY_CURRENT_USER, sub_key, "UninstallString")?;
        exe_path_from(&command)?.parent().map(Path::to_path_buf)
    }

    /// old ： etc 、 old directorypath、 and to can file
    /// already not in —— when （ old in up when valid）。
    fn cleanup_autostart_entries(dirs: &[(&'static str, PathBuf, bool)]) {
        let paths: Vec<PathBuf> = dirs.iter().map(|(_, dir, _)| dir.clone()).collect();
        for name in PRODUCT_NAMES {
            let Some(value) = registry::read_string(HKEY_CURRENT_USER, RUN_KEY, name) else {
                continue;
            };
            let Some(target) = autostart_target(&value, &paths) else {
                continue;
            };
            if target.is_file() {
                continue;
            }
            // when 「」 ，otherwise will down 1 
            // to already delete、 not 
            let removed = registry::delete_value(HKEY_CURRENT_USER, RUN_KEY, name);
            registry::delete_value(HKEY_CURRENT_USER, STARTUP_APPROVED_KEY, name);
            if removed {
                eprintln!("[Cleanup] 已清理旧开机自启登记: {name}");
            }
        }
    }

    /// to old directory can filepath
    fn autostart_target(value: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
        let exe = exe_path_from(value)?;
        dirs.iter().any(|dir| path_within(&exe, dir)).then_some(exe)
    }

    /// 「」 is this 1 disabled。
    ///
    /// and tauri-plugin-autostart auto-launch 1 （ this down 
    /// 8 as 0 enabled）， this 「 is this new 」 and 
    /// `is_enabled()` not will 。 not in from disabled。
    fn startup_disabled(name: &str) -> bool {
        let Some(bytes) = registry::read_binary(HKEY_CURRENT_USER, STARTUP_APPROVED_KEY, name)
        else {
            return false;
        };
        bytes.len() >= 8 && !bytes[bytes.len() - 8..].iter().all(|byte| *byte == 0)
    }

    /// from can file for path： .exe （ not ），
    /// after parameter 1 ； not to return None（ use by 「 not to 」， can not ）。
    ///
    /// use after ****、 in up ****： this must use `to_ascii_lowercase`
    /// not is `to_lowercase` —— after for ASCII （ 'İ'） will ，
    /// down will ；`to_ascii_lowercase` A-Z， not 。
    fn exe_path_from(command: &str) -> Option<PathBuf> {
        let text = trim_quotes(command);
        let end = text.to_ascii_lowercase().find(".exe")? + 4;
        Some(PathBuf::from(&text[..end]))
    }

    /// NSIS （`"C:\...\uninstall.exe"`， can can in after parameter）
    fn trim_quotes(text: &str) -> &str {
        let trimmed = text.trim();
        let Some(rest) = trimmed.strip_prefix('"') else {
            return trimmed;
        };
        rest.split('"').next().unwrap_or(rest)
    }

    /// path is in old directory（ not ）。 can canonicalize when 
    /// use ， 8.3 path and `\\?\` before ；directory already not in when 。
    fn path_within(candidate: &Path, dir: &Path) -> bool {
        if let (Ok(left), Ok(right)) = (candidate.canonicalize(), dir.canonicalize()) {
            return left.starts_with(&right);
        }
        let left = candidate.to_string_lossy().to_lowercase();
        let right = dir.to_string_lossy().to_lowercase();
        left == right || left.starts_with(&format!("{right}\\"))
    }
}

/// use to 「 / / / 」 。
///
/// Why extern not is to windows-sys Registry ：Cargo.toml in 
/// file（ and use ）。 this API 、 use ，
/// by `login.rs` `ShellExecuteW` 1 can ， new 。
#[cfg(windows)]
mod registry {
    use std::ffi::{c_void, OsStr};
    use std::os::windows::ffi::OsStrExt;

    /// 。 etc `(LONG)0x80000001` in 64 down 
    /// （ and windows-sys `HKEY_CURRENT_USER` 1 ， 0x80000001）。
    pub const HKEY_CURRENT_USER: *mut c_void = -2147483647i32 as isize as *mut c_void;

    const KEY_QUERY_VALUE: u32 = 0x0001;
    const KEY_SET_VALUE: u32 = 0x0002;
    const ERROR_SUCCESS: i32 = 0;
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;
    const REG_BINARY: u32 = 3;

    /// limit： this is ， not is need 
    const MAX_VALUE_BYTES: u32 = 8 * 1024;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hkey: *mut c_void,
            sub_key: *const u16,
            options: u32,
            sam: u32,
            result: *mut *mut c_void,
        ) -> i32;
        fn RegQueryValueExW(
            hkey: *mut c_void,
            value_name: *const u16,
            reserved: *mut u32,
            kind: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        fn RegDeleteKeyW(hkey: *mut c_void, sub_key: *const u16) -> i32;
        fn RegDeleteValueW(hkey: *mut c_void, value_name: *const u16) -> i32;
        fn RegCloseKey(hkey: *mut c_void) -> i32;
    }

    fn wide(text: &str) -> Vec<u16> {
        OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
    }

    /// already ：Drop when ，failure need 
    struct Key(*mut c_void);

    impl Drop for Key {
        fn drop(&mut self) {
            unsafe { RegCloseKey(self.0) };
        }
    }

    fn open(root: *mut c_void, sub_key: &str, sam: u32) -> Option<Key> {
        let mut handle: *mut c_void = std::ptr::null_mut();
        let name = wide(sub_key);
        let status = unsafe { RegOpenKeyExW(root, name.as_ptr(), 0, sam, &mut handle) };
        if status == ERROR_SUCCESS && !handle.is_null() {
            Some(Key(handle))
        } else {
            None
        }
    }

    /// 1 and ；/ not in 、 not 、return None。
    ///
    /// not when `RegQueryValueExW` return ERROR_MORE_DATA and need size，
    /// by this 1 （ and winreg `get_raw_value` 1 ： not 
    /// 「lpData NULL 」 that ， that in up as not 1 ）。
    fn query(key: &Key, value: &str, allowed: &[u32]) -> Option<Vec<u8>> {
        const ERROR_MORE_DATA: i32 = 234;

        let name = wide(value);
        let mut kind: u32 = 0;
        // this is ，1KB 1 ； by down 
        let mut size: u32 = 1024;
        let mut buffer = vec![0u8; size as usize];
        loop {
            let status = unsafe {
                RegQueryValueExW(
                    key.0,
                    name.as_ptr(),
                    std::ptr::null_mut(),
                    &mut kind,
                    buffer.as_mut_ptr(),
                    &mut size,
                )
            };
            if status == ERROR_MORE_DATA {
                // size already need ；limit not is need ，
                if size > MAX_VALUE_BYTES {
                    return None;
                }
                buffer = vec![0u8; size as usize];
                continue;
            }
            if status != ERROR_SUCCESS || !allowed.contains(&kind) {
                return None;
            }
            buffer.truncate(size as usize);
            return Some(buffer);
        }
    }

    /// （REG_SZ / REG_EXPAND_SZ）
    pub fn read_string(root: *mut c_void, sub_key: &str, value: &str) -> Option<String> {
        let key = open(root, sub_key, KEY_QUERY_VALUE)?;
        let bytes = query(&key, value, &[REG_SZ, REG_EXPAND_SZ])?;
        let units: Vec<u16> =
            bytes.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        Some(String::from_utf16_lossy(&units).trim_end_matches('\0').to_string())
    }

    /// （「」 enabled use ）
    pub fn read_binary(root: *mut c_void, sub_key: &str, value: &str) -> Option<Vec<u8>> {
        let key = open(root, sub_key, KEY_QUERY_VALUE)?;
        query(&key, value, &[REG_BINARY])
    }

    /// （ need this no has ， is ）
    pub fn delete_key(root: *mut c_void, sub_key: &str) -> bool {
        let name = wide(sub_key);
        unsafe { RegDeleteKeyW(root, name.as_ptr()) == ERROR_SUCCESS }
    }

    /// 
    pub fn delete_value(root: *mut c_void, sub_key: &str, value: &str) -> bool {
        let Some(key) = open(root, sub_key, KEY_SET_VALUE) else {
            return false;
        };
        let name = wide(value);
        unsafe { RegDeleteValueW(key.0, name.as_ptr()) == ERROR_SUCCESS }
    }
}
