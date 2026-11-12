//! Clash Verge config and （ for Node src/workbuddy-proxy.mjs Clash ）。
//!
//! Why 1 ： （proxies.rs）「」，
//! and directory、 YAML file and 1 TTL ，
//! 「」， and 1 / this in 1 will 。
//!
//! ：Clash Verge is **** —— in Clash Verge ，
//! this from verge.yaml when ， listenerUid。Clash 
//! ， use will failure and （ by describe/resolve to ）。
//!
//! ： need verge.yaml， new ， 1 3 seconds
//! TTL —— Clash 3 seconds， and Node 1 。

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde_json::{json, Value};
use serde_yaml::Value as Yaml;

use crate::server::logging;

/// Clash Verge in config uid（ and not will ）
pub const CLASH_MIXED_UID: &str = "__mixed__";

/// Clash config to when Reason
/// （ and 「 use up no Clash Verge」 when Node 1 ）
pub const CLASH_UNAVAILABLE: &str = "未找到 Clash Verge 配置";

/// Clash Verge use directory（Windows down `%APPDATA%\< this >`）
const CLASH_APP_ID: &str = "io.github.clash-verge-rev.clash-verge-rev";

/// TTL：Clash 3 seconds（ for Node CLASH_CACHE_TTL_MS）
const CLASH_CACHE_TTL_MS: i64 = 3000;

/// Clash Verge 。
///
/// and Node `readClashVergeConfig()` return 1 1 for 
/// （available / dir / mixedPort / socksPort / httpPort / listeners /
/// currentProfileUid / error）。
#[derive(Clone, Debug)]
pub struct ClashSnapshot {
    pub available: bool,
    pub dir: Option<String>,
    pub mixed_port: Option<u16>,
    /// Node `readClashVergeConfig()` return `socksPort` for etc ：
    /// down use to mixedPort / listeners， this 1 
    #[allow(dead_code)]
    pub socks_port: Option<u16>,
    /// Node `readClashVergeConfig()` return `httpPort` for etc （ up ）
    #[allow(dead_code)]
    pub http_port: Option<u16>,
    pub listeners: Vec<ClashListener>,
    pub current_profile_uid: Option<String>,
    pub error: Option<String>,
}

impl ClashSnapshot {
    /// 「 not to config」（ directory no has verge.yaml when ）
    fn unavailable(error: &str, dir: Option<PathBuf>) -> Self {
        Self {
            available: false,
            dir: dir.map(|path| path.to_string_lossy().to_string()),
            mixed_port: None,
            socks_port: None,
            http_port: None,
            listeners: Vec::new(),
            current_profile_uid: None,
            error: Some(error.to_string()),
        }
    }
}

/// Clash Verge 1 
#[derive(Clone, Debug)]
pub struct ClashListener {
    pub uid: String,
    pub name: String,
    /// Node `readClashVergeConfig()` listeners[] `proxy` for etc 
    /// （ when used for name/uid ， after not ； 1 ）
    #[allow(dead_code)]
    pub proxy: String,
    pub port: u16,
    pub profile_uid: Option<String>,
    pub enabled: bool,
}

// ─── configdirectory and YAML ───────────────────────────────────

/// use directory。
///
/// Node `os.homedir()` in Windows up USERPROFILE； this by 1 ，
/// and gateway::config_dir 1 （ not dirs crate）。
fn home_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// Clash Verge configdirectory： Node `clashConfigDir()`。
///
/// ① `%APPDATA%\<appId>`（Windows ）
/// ② `~/AppData/Roaming/<appId>`（APPDATA when ， Linux up Wine / ）
///   ③ `~/.config/<appId>`（macOS / Linux）
///
/// is 「directory in verge.yaml」——directory in will Clash 、
/// no config available。
pub fn clash_config_dir() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        let trimmed = appdata.trim();
        if !trimmed.is_empty() {
            candidates.push(Path::new(trimmed).join(CLASH_APP_ID));
        }
    }
    if let Some(home) = home_dir() {
        candidates.push(home.join("AppData").join("Roaming").join(CLASH_APP_ID));
        candidates.push(home.join(".config").join(CLASH_APP_ID));
    }
    candidates
        .into_iter()
        .find(|dir| dir.join("verge.yaml").exists())
}

/// 1 YAML file；/failure/ not is return None
/// （ for Node `readYaml` `parsed && typeof parsed === 'object' && !Array.isArray(parsed)`）。
///
/// error：Clash configfile can can Clash （ to ），
/// this when as 「 not to 」， not is to use —— Node 。
fn read_yaml(path: &Path) -> Option<Yaml> {
    if !path.exists() {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    let parsed: Yaml = serde_yaml::from_str(&text).ok()?;
    match parsed {
        Yaml::Mapping(_) => Some(parsed),
        _ => None,
    }
}

/// YAML → 。
///
/// for Node `isValidPort(value)`：`Number(value)` after must is 1..65535 。
/// and （Clash verge.yaml has when `"7897"`），
/// / this will `Number()` not —— can can 
/// Clash config， this not 。
fn yaml_port(value: Option<&Yaml>) -> Option<u16> {
    let number = match value? {
        Yaml::Number(number) => number.as_f64()?,
        Yaml::String(text) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    if !number.is_finite() || number.fract() != 0.0 || !(1.0..=65535.0).contains(&number) {
        return None;
    }
    Some(number as u16)
}

/// YAML and empty（ 1 empty）
fn yaml_text(value: Option<&Yaml>) -> String {
    match value {
        Some(Yaml::String(text)) => text.trim().to_string(),
        _ => String::new(),
    }
}

/// Clash Verge 。
///
/// Clash or config not can when `available=false`， not —— this is 
/// （ use up is no Clash）。unavailable and Node 1 ：
/// ① directory no has verge.yaml → dir=null + error= to Clash Verge config
/// ② verge.yaml failure → dir has + error=verge.yaml failure
/// ③ → error=null + （ can can is empty ）
pub fn read_clash_verge_config() -> ClashSnapshot {
    let Some(dir) = clash_config_dir() else {
        return ClashSnapshot::unavailable(CLASH_UNAVAILABLE, None);
    };
    let Some(verge) = read_yaml(&dir.join("verge.yaml")) else {
        return ClashSnapshot::unavailable("verge.yaml 解析失败", Some(dir));
    };

    // config ： 1 
    //   - { uid, name, port, proxy, enabled, profile_uid }
    // （ proxy / / ） 1 ， not is failure ——
    // 1 not this its available from down 。
    let mut listeners: Vec<ClashListener> = Vec::new();
    if let Some(items) = verge.get("verge_mixed_listeners").and_then(Yaml::as_sequence) {
        for item in items {
            let Some(mapping) = item.as_mapping() else {
                continue;
            };
            let proxy = yaml_text(mapping.get("proxy"));
            if proxy.is_empty() {
                continue;
            }
            let Some(port) = yaml_port(mapping.get("port")) else {
                continue;
            };
            // uid when use 「:proxy」（Node ）， down has 
            let uid = {
                let text = yaml_text(mapping.get("uid"));
                if text.is_empty() {
                    format!("{port}:{proxy}")
                } else {
                    text
                }
            };
            let name = {
                let text = yaml_text(mapping.get("name"));
                if text.is_empty() {
                    proxy.clone()
                } else {
                    text
                }
            };
            let profile_uid = {
                let text = yaml_text(mapping.get("profile_uid"));
                if text.is_empty() { None } else { Some(text) }
            };
            listeners.push(ClashListener {
                uid,
                name,
                proxy,
                port,
                profile_uid,
                // Node is `item.enabled !== false`： has false use 
                enabled: !matches!(mapping.get("enabled"), Some(Yaml::Bool(false))),
            });
        }
    }

    // profiles.yaml as 「current」： its when can can ，
    // before in down up 「（ its ， can can ）」
    let current_profile_uid = read_yaml(&dir.join("profiles.yaml")).and_then(|profiles| {
        let current = yaml_text(profiles.get("current"));
        if current.is_empty() { None } else { Some(current) }
    });

    ClashSnapshot {
        available: true,
        dir: Some(dir.to_string_lossy().to_string()),
        mixed_port: yaml_port(verge.get("verge_mixed_port")),
        socks_port: yaml_port(verge.get("verge_socks_port")),
        http_port: yaml_port(verge.get("verge_port")),
        listeners,
        current_profile_uid,
        error: None,
    }
}

// ─── ───────────────────────────────────────────────

/// ：`( when , )`。
///
/// ** to **（including「unavailable」 this ）， this Clash no up 
/// not will request stat directory。file， not IO。
static CLASH_CACHE: Mutex<Option<(i64, ClashSnapshot)>> = Mutex::new(None);

/// （default 3 seconds TTL ）
pub fn clash_snapshot() -> ClashSnapshot {
    clash_snapshot_with(false)
}

/// ，`force=true` when （ before 「 new 」 by use up ）。
///
/// and Node `clashSnapshot({ ttlMs, force })` ：Node TTL，
/// Rust no has use need need TTL， force 1 。
pub fn clash_snapshot_with(force: bool) -> ClashSnapshot {
    let now = logging::now_ms();
    // in （ panic） not ： use ，total Clash
    // failure
    let mut guard = match CLASH_CACHE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if !force {
        if let Some((at, snapshot)) = guard.as_ref() {
            if now - *at < CLASH_CACHE_TTL_MS {
                return snapshot.clone();
            }
        }
    }
    let snapshot = read_clash_verge_config();
    *guard = Some((now, snapshot.clone()));
    snapshot
}

/// Clash optional（GET /api/proxies and /api/session use ）， for Node `clashProxyOptions`。
///
/// unavailablereturn `{available:false, error, dir, options:[]}` —— options is empty
/// not is ， before proxy-form.js `.options.length`。
///
/// `profileActive` this is current（）：
/// profile_uid when as true（ Clash not this ， not can 1 ）。
pub fn clash_proxy_options() -> Value {
    let snapshot = clash_snapshot();
    if !snapshot.available {
        return json!({
            "available": false,
            "error": snapshot.error,
            "dir": snapshot.dir,
            "options": [],
        });
    }
    let mut options = Vec::new();
    if let Some(port) = snapshot.mixed_port {
        options.push(json!({
            "uid": CLASH_MIXED_UID,
            "name": "Clash 混合端口（按规则分流）",
            "port": port,
            "kind": "mixed",
            "enabled": true,
            "profileActive": true,
        }));
    }
    for listener in &snapshot.listeners {
        let active = listener
            .profile_uid
            .as_ref()
            .map(|uid| Some(uid) == snapshot.current_profile_uid.as_ref())
            .unwrap_or(true);
        options.push(json!({
            "uid": listener.uid,
            "name": listener.name,
            "port": listener.port,
            "kind": "listener",
            "enabled": listener.enabled,
            "profileActive": active,
        }));
    }
    json!({
        "available": true,
        "error": Value::Null,
        "dir": snapshot.dir,
        "options": options,
    })
}
