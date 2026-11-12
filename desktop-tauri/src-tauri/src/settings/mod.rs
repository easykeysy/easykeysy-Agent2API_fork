//! use set（ to 、、） 。
//!
//! in configdirectory not is WebView localStorage： this set need as 
//! （ is 、 is 、server bind ），must can in no 
//! 、 no when to 。
//!
//! ── ： 1 `kv` （ from desktop-settings.json ）──
//! before is 1 `{config_dir}/desktop-settings.json`； in set is `kv`
//! `desktopSettings` that 1 （** 1 **， by `sql.rs` module header）。
//! old file by `db::migrate::import_settings` 1 。
//!
//! ── when ： has is has （ need need 1 ）───────────
//! `proxy_port()` server bind ，must in **
//! before ** has ；set in in ， by `Db::open` —— this is 
//! when 。must：
//! 1. `proxy_port()` ** is **（`gateway::ACTIVE_PORT`， 1 
//! 1 ），「 use need 」 this not —— 1 ；
//! 2. ** not is can error**： to default 3065 will and use in use 
//! not （ API client to no ）， 3065 if use then 
//! ** use not **（`backend::ensure_ready` bind failurepath）；
//! 3. in `Db::open` ****， `load()` in ** before ** ——
//! 「 etc set」 is not to 。
//!
//! is ****， 1 ：
//! - ****：。 no has → old file； old file no has → default。
//! is 「 before 」 and 「 after 」 to is 1 。
//! - ****： by （`db::migrate::import_settings`）。
//! ** not ** in to old file when —— by down 「Why no has 」。
//!
//! ── Why no has （ 1 ）──────────────────
//! this has 1 「 from old file to 」 ， to 
//! 。** in down ， is not to **：
//! `lib.rs` setup 1 （ 196 ） `settings::load()` 
//! `closeToTray`， `Db::open` + in `backend::ensure_ready` 
//! （ 274 `spawn`），****。 is 1 is ，
//! after to 「`desktopSettings` already in 」 —— after is old file
//! not as `.migrated`、 that 「✅ set old already 」
//! not ， `import_settings` this will 。
//! is 「」 this use can empty： by 
//! `desktop-settings.json.migrated` will not to ， can to 1 no old file。
//!
//! after ，path 1 ：
//! - ** before **（`load` ）： old file， not —— ，
//! as when this is empty ；
//! - ** after **： in ， not old file；
//! - has 1 → and this 。
//! is 「 this no （e.g. that 1 ）」 when set in file，
//! that is this file （ is 1 ）， down will 
//! —— and not can can ， will 「 has no has 」 this 。
//!
//! Why not 「file、 its 」（ 1 ）： set
//! 、file， will 「set in 」 this for use and ，
//! this is 1 （`change_port` and `save_app_settings` need 
//! 1 set）。 1 （ 1 、 not ） 1 
//! —— that already （ 1 ） to 「 1 1 」。
//!
//! ── ────────────────────────────────────────────────
//! and before 1 ： not （、 、 not ） 1 to default，
//! not as 1 set use not 。** 1 is 「」**： that must，
//! as use no up is need need （ and `load` to ）。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::gateway;

pub(crate) mod sql;

/// set in file（** is file**； and use ）。
///
/// from 「setfilepath」「 this set file」—— and T2~T6 for 
/// `AccountStore::file()` / `LogStore::file()` / `RequestStats::file()` 
/// 1 ， store 1 。
pub fn file_path() -> PathBuf {
    gateway::config_dir().join(crate::server::db::FILE_NAME)
}

/// old setfile path（ and path use ）。
pub(crate) fn legacy_file_path() -> PathBuf {
    agent2api_server::paths::legacy_desktop_settings_file()
}

/// use set。
///
/// before after camelCase JSON （`closeToTray` / `autostart` / `proxyPort`），
/// and renderer 1 ，none need 。
///
/// `default` use in up （）： this after add when ，** old set
/// this not will setfailure**—— Default。
///
/// not `Copy`：`panel_session` is （）， this from 「 
/// and 1 u16」 。all use by / by use ，
/// `Copy` after clone is —— 「 this set in 」。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    /// when to not is 
    pub close_to_tray: bool,
    /// 
    pub autostart: bool,
    /// 。
    ///
    /// 0 = set， to default（release 3065 / debug 3066，
    /// and `gateway::proxy_port()` 1 ）。
    /// this not is after config：****client ， and need in 
    /// server bind before to ，「 use set」config。
    /// this need need （server bind after not ）。
    pub proxy_port: u16,
    /// ： `0.0.0.0` not is `127.0.0.1`（issue #48）。
    ///
    /// default false（， ）。 before is 
    /// 已注册（`change_lan_access` 命令把关）：监听地址一旦出了回环，`/api/*`
    /// not ，must will + API Key —— this 
    /// in `backend::ensure_ready` by 。
    /// this need need （ in bind after not ）。
    pub lan_access: bool,
    /// when is when （ `ui/` directory，
    /// can ； / ）。
    /// default false： is use ， `/v1/*` to ，
    /// by Tauri 。only in as true when has 。
    pub lan_panel: bool,
    /// ： to when ****（ WebView2 that 6 
    /// 、 200MB ）， when by 1 parameter。
    ///
    /// in （`server/` crate）， not and when ；
    /// in WebView 「ZCode 」 ——
    /// this request will 503 request（ will need need ）。
    /// only in `close_to_tray` when has （ when not in 「」 1 ）。
    ///
    /// + when when ，** not **（ and after ，
    /// by 1 parameter）：path not ， if （
    /// ），WebView2 will ， need etc 1 「→」
    /// 。 up （ in before not ）。
    /// default false。
    pub lightweight_mode: bool,
    /// old will ； already has set， not 。
    #[serde(default)]
    pub panel_session: PanelSession,
}

/// will （ is `enc1:` ；empty = no has ）。
///
/// not is 1 JSON：access will （ 2 when ），refresh in 
/// when update —— ， not 1 。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PanelSession {
    /// refresh（30 ） 
    pub refresh: String,
    /// access（2 when ） 
    pub access: String,
    /// access to when （Unix ms；0 = not ）。
    ///
    /// can ： not is ，「 this request before need not need 」。
    /// `serde(default)` before set（ no has this ） by 「 not 」 ——
    /// that 「401 → 」 path， as and before 1 。
    pub expires_at: i64,
}

impl Default for AppSettings {
    fn default() -> Self {
        // to default： in after ，
        // use is ， not is in 
        Self {
            close_to_tray: true,
            autostart: false,
            proxy_port: 0,
            lan_access: false,
            lan_panel: false,
            lightweight_mode: false,
            panel_session: PanelSession::default(),
        }
    }
}

/// set： → old file → default，（ when module header）。
///
/// use ：
/// - ****：path（ after ， 1 ）；
/// - ** old file**：** before ** 1 —— use after 1 when 
/// in file， not to will to 3065（module header 2 ， that can can 
/// use not ）；
/// - **default**： new （ no has ）。
///
/// ** not **： from old file to when not will 。 by 
/// （`db::migrate::import_settings`），module header
/// 「Why no has 」—— ： this in `Db::open` before ， if 
/// will in before ，「 old file as `.migrated`」 and 「 that 
/// already 」 when empty。
pub fn load() -> AppSettings {
    if let Some(text) = sql::load(&file_path()) {
        // that not （）： not in this old file —— after 
        // old file already ， will to 「 no has 」； 1 default， and before 
        // 「file → default」 as 1 。
        return serde_json::from_str(&text).unwrap_or_default();
    }
    let Some(text) = std::fs::read_to_string(legacy_file_path()).ok() else {
        return AppSettings::default();
    };
    // failuredefault（ old file ， not use ）
    serde_json::from_str::<AppSettings>(&text).unwrap_or_default()
}

/// set（），return is success。
///
/// and [`load`] to ：**failuremust** —— use 「」 no up 
/// is need need 。 before not directory：file directory by `Db::open` ，
/// configdirectory in already by `config_migration` ；`sql::write` 
/// will up etc （ by that —— can can in `Db::open` before use ）。
///
/// is ****； old `desktop-settings.json` in after already `.migrated`，
/// not —— if in （ use ）， that is ，
/// not is （ will ， is need that ）。
pub fn save(settings: &AppSettings) -> Result<(), String> {
    let text = serde_json::to_string(settings)
        .map_err(|error| format!("setfailure: {error}"))?;
    sql::write(&file_path(), &text)
}

// ： old file use （ / / ） already to 
// crate（`agent2api_server::server::db::migrate::settings` ），
// its in use `LegacyAppSettings` is `AppSettings` —— **
// when must that **， to 。
