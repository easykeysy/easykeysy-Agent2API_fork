//! configdirectory and path ** 1 **。
//!
//! before this in crate `gateway.rs`（`crate::gateway::config_dir`
//! is 1 ，`server::config` ）。 crate after 
//! to this ：`paths::config_dir` is 1 ， `gateway::config_dir`
//! —— 「path、 1 」 not ， is to 。
//!
//! ── ────────────────────────────────────────────────
//! `AGENT2API_PROXY_HOME` ， old `WORKBUDDY_PROXY_HOME` （1.x 
//! / can can old ）。Docker to 
//! （compose ：`AGENT2API_PROXY_HOME=/data`）， can in 。

use std::path::PathBuf;

/// configdirectory：default `{ use directory}/.agent2api`， can 。
pub fn config_dir() -> PathBuf {
    env_path("AGENT2API_PROXY_HOME")
        .or_else(|| env_path("WORKBUDDY_PROXY_HOME")) // 旧名兼容读
        .unwrap_or_else(default_config_dir)
}

/// directory：set / empty 1 set
fn env_path(name: &str) -> Option<PathBuf> {
    let custom = std::env::var(name).ok()?;
    let trimmed = custom.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// defaultconfigdirectory：`{ use directory}/.agent2api`
fn default_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent2api")
}

/// old setfile path（`{config_dir}/desktop-settings.json`）。
///
/// before by crate `settings::legacy_file_path()` ； crate 
/// 1 （`db::migrate::settings`） need this file， headless down 
/// crate not and —— path `config_dir` 1 to 
/// ， `settings::legacy_file_path()` this 。
///
/// file is ： this file already not is set （ SQLite 
/// `kv` ）， and path use ， not will 。
pub fn legacy_desktop_settings_file() -> PathBuf {
    config_dir().join("desktop-settings.json")
}

/// directory:default `{config_dir}/logs`。
///
/// Linux (WSL / headless ) in ** no has when **default to 
/// `/mnt/ramdisk/Agent2API/logs`: is 、 can 、 and will ,
/// and after this —— this path use ,
/// in default。`AGENT2API_PROXY_HOME`( or old ) in when use 
/// directory,directorymust, not path。
/// `/mnt/ramdisk` not in ( WSL Linux、) when `{config_dir}/logs`,
/// not 1 not in 。
pub fn logs_dir() -> PathBuf {
    side_dir("logs")
}

/// directory:default `{config_dir}/cache`;Linux then and [`logs_dir`] 1 
/// (`/mnt/ramdisk/Agent2API/cache`)。 is ** can **: not ,
/// this is this by 。
pub fn cache_dir() -> PathBuf {
    side_dir("cache")
}

/// when directory:default `{config_dir}/tmp`;Linux then and [`logs_dir`] 1 
/// (`/mnt/ramdisk/Agent2API/tmp`)。 1 this ——
/// is 「 use 」 ,。
pub fn tmp_dir() -> PathBuf {
    side_dir("tmp")
}

/// up directory use :`(env / Linux / none) →
/// `{config_dir}/{name}`,otherwise → `/mnt/ramdisk/Agent2API/{name}`。
///
/// `/mnt/ramdisk` ( not 、 not `/mnt/ramdisk1`): this is 
/// , 1 will in when 1 「Why that 」 。
fn side_dir(name: &str) -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if env_path("AGENT2API_PROXY_HOME").is_some()
            || env_path("WORKBUDDY_PROXY_HOME").is_some()
        {
            return config_dir().join(name);
        }
        let ramdisk = std::path::Path::new("/mnt/ramdisk");
        if ramdisk.is_dir() {
            return ramdisk.join("Agent2API").join(name);
        }
    }
    config_dir().join(name)
}

/// directory is (`/mnt/ramdisk` )。
///
/// file is : has when file after —— no has 
/// Linux / / after , as and before 1 。
#[allow(dead_code)]
pub fn side_dirs_external() -> bool {
    logs_dir() != config_dir().join("logs")
}

/// when directory。
///
/// ramdisk will empty, `create_dir_all` can 「 that 
/// 1 」; when 1 1 ,set path from 1 is in 。failure not 
/// (directory not can when will )。
#[allow(dead_code)]
pub fn ensure_side_dirs() {
    for dir in [logs_dir(), cache_dir(), tmp_dir()] {
        let _ = std::fs::create_dir_all(dir);
    }
}
