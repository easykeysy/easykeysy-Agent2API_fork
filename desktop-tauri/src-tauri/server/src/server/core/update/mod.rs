//! update： GitHub 、check and down 。

mod check;
mod download;
pub mod capabilities;
pub mod systemd;
pub mod windows_service;
mod service_task;
#[cfg(any(target_os = "linux", windows))]
#[path = "systemd/trust.rs"]
mod service_trust;
pub(crate) mod source;
mod source_state;
mod selection;
mod probe;
mod integrity;
// update： from `app.json` 「 in 」。
// and use 、 file（module header）。
mod index;
// （`fetch_public`） `core::model_meta`（models.dev
// new ） use ： that is 「 this 」， and 
// none， as 1 will 1 will 。 for crate
// can ， for not 。
pub(crate) mod client;
// update：headless （`set_restart_hook`） need use ，
// for crate need can （ not ， this in Linux up 
// 1 return「unsupported」）。
pub mod selfupdate;
mod token;
mod version;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use crate::server::{config, logging};

pub use version::{
    api_base, asset_suffixes, assert_downloadable, compare_versions, custom_api_base, desktop_shell,
    installer_kind, installer_suffixes, is_prerelease_version, parse_version, pick_android_apk,
    pick_asset_by_name, pick_installer, safe_file_name, server_bundle_name, set_desktop_shell,
    UpdateError, DEFAULT_REPO, GITHUB_API, MAX_INSTALLER_BYTES, SOURCE_REPO,
};
pub use integrity::download_plan;
/// 「updateset」 GitHub （api::update this ；
/// in token ， module header）
pub use token::{set_token, status_json};

/// request GitHub UA（Node ，）
const USER_AGENT: &str = "agent2api-local-proxy";

/// update use check，minutes new 1 。
const INDEX_RETRY_MS: i64 = 5 * 60 * 1000;

/// use current（ Cargo.toml， and tauri.conf.json
/// update）。 `checkUpdate` use is when package_info —— 
/// 1 ； after when check（scheduled_tasks） not to Tauri ， use this 1 。
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ─── down ───────────────────────────────────────────

/// down （ for Node `task` for ）。
#[derive(Clone, Debug)]
struct Task {
    active: bool,
    done: bool,
    canceled: bool,
    error: Option<String>,
    filename: String,
    /// path； or failure after None（ and Node `current.path = null` 1 ）
    path: Option<String>,
    received: u64,
    total: u64,
    selecting: bool,
    source: Option<String>,
}

impl Task {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("active".to_string(), Value::Bool(self.active));
        map.insert("done".to_string(), Value::Bool(self.done));
        map.insert("canceled".to_string(), Value::Bool(self.canceled));
        map.insert(
            "error".to_string(),
            self.error.clone().map(Value::String).unwrap_or(Value::Null),
        );
        map.insert("filename".to_string(), Value::String(self.filename.clone()));
        map.insert(
            "path".to_string(),
            self.path.clone().map(Value::String).unwrap_or(Value::Null),
        );
        map.insert("received".to_string(), Value::from(self.received));
        map.insert("total".to_string(), Value::from(self.total));
        map.insert("selecting".to_string(), Value::Bool(self.selecting));
        map.insert("source".to_string(), json!(self.source));
        // percent：totalunknown when as 0； already when to up to 100 
        // （Node: `task.total > 0 ? Math.min(100, Math.round(received/total*100)) : 0`）
        let percent = if self.total > 0 {
            let raw = (self.received as f64 / self.total as f64 * 100.0).round() as i64;
            raw.clamp(0, 100)
        } else {
            0
        };
        map.insert("percent".to_string(), Value::from(percent));
        Value::Object(map)
    }
}

/// ： down 、、 1 to Release、。
struct Inner {
    task: Option<Task>,
    /// and 1 1 for （Node use AbortController，Rust use ）
    cancel_flag: Option<Arc<std::sync::atomic::AtomicBool>>,
    /// 1 to Release（）
    latest: Option<Value>,
    repository: String,
    /// `repository` `WORKBUDDY_UPDATE_REPO` （）。
    ///
    /// as when ** not **： use not can 。
    repository_explicit: bool,
    /// `repository` is from （`true`） is default（`false`）。
    /// used for。
    repository_from_index: bool,
    /// up when （ms，0 = no ）。 and [`INDEX_RETRY_MS`]
    /// 1 。
    index_checked_at: i64,
    app_index: index::AppIndex,
    download_dir: PathBuf,
    /// update already use 、 no （ in ，）。
    ///
    /// 「 1 」： new already 、 when not file。
    pending_restart: bool,
    service_update_active: bool,
}

/// update： 1 + Clone（ and auto_checkin / models ）。
#[derive(Clone)]
pub struct UpdateManager {
    inner: Arc<Mutex<Inner>>,
}

impl UpdateManager {
    /// 。`directory` is configdirectory（ down in `{directory}/updates`）。
    ///
    /// and Node 1 ：parameter > `WORKBUDDY_UPDATE_REPO` > DEFAULT_REPO
    /// （`String(repo || env || DEFAULT).trim()`）。
    pub fn new(directory: PathBuf) -> Self {
        let explicit = std::env::var("WORKBUDDY_UPDATE_REPO")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let app_index = if explicit.is_some() { index::AppIndex::default() } else { index::cached() };
        let repository = explicit.clone().unwrap_or_else(|| app_index.repo.clone());
        Self {
            inner: Arc::new(Mutex::new(Inner {
                task: None,
                cancel_flag: None,
                latest: None,
                repository,
                repository_explicit: explicit.is_some(),
                repository_from_index: false,
                index_checked_at: 0,
                app_index,
                download_dir: directory.join("updates"),
                pending_restart: false,
                service_update_active: false,
            })),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// （`owner/repo`）
    pub fn repository(&self) -> String {
        self.lock().repository.clone()
    }

    pub fn reset_source_preference(&self) {
        source::clear_preferences();
        self.lock().index_checked_at = 0;
    }

    /// check need use ， need when 1 。
    ///
    /// ── ──────────────────────────────────────────────────
    /// 1. `WORKBUDDY_UPDATE_REPO` ：，，** and not **
    /// —— / need can ；
    /// 2. `app.repo`（ [`index::resolve_app_repo`]）；
    /// 3. [`DEFAULT_REPO`]。
    ///
    /// ── Why ────────────────────────────────────────────
    /// when check can can minutes 1 ， is 1 not file。
    /// is raw.githubusercontent.com（ ）。success after 
    /// [`INDEX_RETRY_MS`] not ；**failure** 1 etc this —— failure
    /// is not ， will failure。
    ///
    /// Note「 1 」， not 「 after 」： 1 to ， use 
    /// `repository` and 1 use down （ after new ）。
    async fn resolve_repository(&self) -> String {
        let (current, explicit) = { let guard = self.lock(); (guard.repository.clone(), guard.repository_explicit) };
        if explicit { return current; }
        let now = logging::now_ms();
        {
            let mut guard = self.lock();
            if guard.index_checked_at != 0 && now.saturating_sub(guard.index_checked_at) < INDEX_RETRY_MS { return current; }
            guard.index_checked_at = now;
        }
        let resolved = index::resolve().await;
        let repository = resolved.repo.clone();
        let changed = {
            let mut guard = self.lock();
            let changed = guard.repository != repository;
            guard.repository = repository.clone();
            guard.repository_from_index = true;
            guard.app_index = resolved;
            changed
        };
        if changed { logging::verbose("[Update]", &format!("更新源仓库改为 {repository}（来自索引）")); }
        repository
    }

    pub fn force_update(&self, current: &str) -> Option<Value> {
        let guard = self.lock();
        if guard.repository_explicit { return None; }
        guard.app_index.required(current)
    }

    pub async fn refresh_policy(&self, current: &str) -> Value {
        self.resolve_repository().await;
        json!({ "currentVersion": current, "forceUpdate": self.force_update(current), "repository": self.repository() })
    }

    /// down directory（`{config_dir}/updates`）—— and `update::download_dir()` 
    pub fn download_dir(&self) -> PathBuf {
        self.lock().download_dir.clone()
    }

    /// check response（ for buildCheckResult， for ）
    fn build_check_result(&self, current_version: &str) -> Value {
        let version = current_version.trim().to_string();
        let (latest, repository, download_dir, task, pending_restart) = {
            let guard = self.lock();
            (
                guard.latest.clone(),
                guard.repository.clone(),
                guard.download_dir.clone(),
                guard.task.clone(),
                guard.pending_restart,
            )
        };
        let channels = config::update_channels();
        let force_update = self.force_update(current_version);
        let latest = latest.filter(|release| {
            let prerelease = release["prerelease"] == true
                || version::is_prerelease_version(release["tag"].as_str().unwrap_or(""));
            force_update.is_some() || channels.accepts(prerelease)
        });
        let latest_tag = latest
            .as_ref()
            .and_then(|value| value.get("tag"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let comparison = if latest.is_some() && !version.is_empty() {
            compare_versions(latest_tag, &version)
        } else {
            None
        };
        // has url 「 can down 」
        let asset = latest
            .as_ref()
            .and_then(|value| value.get("asset"))
            .filter(|value| {
                value.get("url").and_then(Value::as_str).map(|url| !url.is_empty()).unwrap_or(false)
            })
            .cloned();
        // update and .sha256 file（ Linux、 or this no this 
        // when is None —— 「」 ）
        let downloadable = |value: Option<&Value>| -> Option<Value> {
            value
                .filter(|item| {
                    item.get("url").and_then(Value::as_str).map(|url| !url.is_empty()).unwrap_or(false)
                })
                .cloned()
        };
        let server_asset = downloadable(latest.as_ref().and_then(|value| value.get("serverBundle")));
        let server_checksum =
            downloadable(latest.as_ref().and_then(|value| value.get("serverChecksums")));
        let latest_text = |key: &str| -> Value {
            match latest.as_ref().and_then(|value| value.get(key)) {
                Some(Value::String(text)) if !text.is_empty() => Value::String(text.clone()),
                Some(Value::Bool(flag)) => Value::Bool(*flag),
                _ => Value::Null,
            }
        };
        // 「 this 1 has no has to 」 ：release that （total / after /
        // has no has update） check.rs release_json； expected* is 
        // ****， in this —— and servernone， not this 
        // that by Release （ will to after ）。
        let probe = latest.as_ref().and_then(|value| value.get("assetProbe"));
        let probe_field = |key: &str| -> Value {
            probe.and_then(|value| value.get(key)).cloned().unwrap_or(Value::Null)
        };
        let asset_probe = json!({
            "assetsTotal": probe_field("assetsTotal"),
            "suffixes": probe_field("suffixes"),
            "filteredBySource": probe_field("filteredBySource"),
            // need after （ / headless is empty）；update need file
            // （ Linux is null）—— by selfUpdate 1 ， has 
            "expectedSuffixes": version::installer_suffixes(),
            "expectedName": version::server_bundle_name(),
        });
        let capability = capabilities::current();
        let systemd_task = (capability.deployment == "systemd").then(systemd::status).flatten();
        let service_task = if capability.deployment == "windows-service" {
            windows_service::status()
        } else { systemd_task.clone() };
        json!({
            "updateCapability": capability,
            "serviceTask": service_task,
            "systemdTask": systemd_task,
            "forceUpdate": force_update,
            "currentVersion": if version.is_empty() { Value::Null } else { Value::String(version) },
            "latestVersion": if latest_tag.is_empty() { Value::Null } else { Value::String(latest_tag.to_string()) },
            // none when not 「 has update」， by null as 「none」
            "hasUpdate": match comparison {
                Some(order) => Value::Bool(order > 0),
                None => Value::Null,
            },
            "comparison": match comparison {
                Some(order) if order > 0 => Value::String("newer".to_string()),
                Some(order) if order < 0 => Value::String("older".to_string()),
                Some(_) => Value::String("same".to_string()),
                None => Value::Null,
            },
            "releaseName": latest_text("name"),
            "notes": latest_text("notes"),
            "publishedAt": latest_text("publishedAt"),
            "pageUrl": latest_text("pageUrl"),
            "prerelease": latest
                .as_ref()
                .and_then(|value| value.get("prerelease"))
                .and_then(Value::as_bool)
                == Some(true),
            "asset": asset.clone().unwrap_or(Value::Null),
            // ── （APK）────────────────────────────────────
            // and `asset` and ： when and ， need 。
            // this GitHub down down （， not in ）。
            // this no apk when is null，「update」。
            "androidAsset": downloadable(latest.as_ref().and_then(|value| value.get("androidAsset")))
                .unwrap_or(Value::Null),
            // ── update（Linux / Docker）──────────────────────
            // selfUpdate is 「 this can not can update」（ + ），
            // serverAsset is 「 this 1 has no has can down 」—— ， can 
            // 「unsupported」 and 「 this no update」 this not 。
            "selfUpdate": selfupdate::supported(),
            "serverAsset": server_asset.unwrap_or(Value::Null),
            "serverChecksumUrl": server_checksum.unwrap_or(Value::Null),
            // 「 has new no has can down 」 Reason（ in ， update-panel
            // and update-modal assetMissingReason）
            "assetProbe": asset_probe,
            // already use 、： by 「」，
            // not is to 1 400 「 down and update」
            "pendingRestart": pending_restart,
            // in （ down ），used for「update from 」 。
            // / Star ** not need ** use —— down 1 。
            "repository": repository,
            // ： up 「」「」 by 。
            // is Actions ， up no has ， not can 。
            "sourceRepository": SOURCE_REPO,
            "downloadSupported": asset.is_some(),
            // （"nsis" / "dmg"）： ——
            // Windows need UAC 、macOS is after use 
            "installerKind": installer_kind(),
            "downloadDir": download_dir.to_string_lossy(),
            "task": task.map(|task| task.to_json()).unwrap_or(Value::Null),
        })
    }

    // ─── update（Docker / Linux headless）────────────────────
    //
    // and up 「 down 」 is ： that to down （ down by 
    // ）， this to file（ down by + ）。
    // is POST /api/update/{install,restart} 1 ，
    // and selfupdate module header。

    /// use already down update： → → and ui directory。
    ///
    /// down directory file by **file**（`safe_file_name()`）， not 
    /// use path —— 1 「 anyfileupdate」 。
    pub async fn install_downloaded(&self, ui_dir: Option<&Path>) -> Result<Value, UpdateError> {
        if !selfupdate::supported() {
            return Err(UpdateError::with_status(
                400,
                "当前部署不支持容器内更新：需要在容器中运行并设置 AGENT2API_SELF_UPDATE=1",
            ));
        }
        // down in when file is ： can etc ， not need 
        if self.lock().task.as_ref().map(|task| task.active).unwrap_or(false) {
            return Err(UpdateError::with_status(400, "更新包还在下载中，请等下载完成后再安装"));
        }
        // already use 1 、 no ：， not file。
        if self.lock().pending_restart {
            return Err(UpdateError::with_status(
                400,
                "本次更新已经应用，请先重启服务（面板上的重启，或宿主机 docker compose restart）",
            ));
        }
        let bundle = selfupdate::bundle_name().expect("supported 已判定平台与架构");
        // from ** that release JSON** ：、file、。
        // is check.rs that （`serverBundle` / `serverChecksums` / `tag`），
        // ** not is ** check response to use （`serverAsset` / `serverChecksumUrl` /
        // `latestVersion`）—— use after is empty、「 no has file」。
        let (bundle_url, checksum_url, version) = {
            let guard = self.lock();
            let latest = guard.latest.clone();
            let asset_url = |key: &str| -> String {
                latest
                    .as_ref()
                    .and_then(|value| value.get(key))
                    .and_then(|value| value.get("url"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            };
            let tag = latest
                .as_ref()
                .and_then(|value| value.get("tag"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            (asset_url("serverBundle"), asset_url("serverChecksums"), tag)
        };
        // this no （ Release）： to 1 can ， not is failure
        if bundle_url.is_empty() {
            return Err(UpdateError::with_status(
                400,
                "本次发布没有提供容器内更新包，请在宿主机执行 docker compose pull && docker compose up -d 升级",
            ));
        }
        match compare_versions(&version, CURRENT_VERSION) {
            Some(order) if order < 0 => {
                return Err(UpdateError::with_status(400, "容器内更新不支持降级到较旧版本"));
            }
            None => {
                return Err(UpdateError::with_status(400, "更新包版本号无法比较，已放弃安装"));
            }
            _ => {}
        }
        if checksum_url.is_empty() && !source::is_resource(&bundle_url) {
            return Err(UpdateError::with_status(
                400,
                "本次发布没有提供校验文件（.sha256），已放弃安装",
            ));
        }
        let archive = self.download_dir().join(safe_file_name(&bundle));
        if !archive.is_file() {
            return Err(UpdateError::with_status(400, "更新包尚未下载，请先点击「下载并更新」"));
        }
        let expected = match integrity::trusted_digest(&bundle_url).await {
            Some((hash, _)) => hash,
            None if !checksum_url.is_empty() => selfupdate::fetch_checksum(&checksum_url).await?,
            None => return Err(UpdateError::new("未取得官方校验值，已放弃安装")),
        };
        let actual = selfupdate::file_sha256(&archive)?;
        if expected != actual {
            logging::log(
                "[Update]",
                &format!("❌ 更新包 sha256 校验失败（期望 {expected}，实际 {actual}）"),
            );
            return Err(UpdateError::new(
                "更新包校验失败（sha256 不匹配），已放弃安装；可删掉该文件后重新下载",
            ));
        }
        let exe = selfupdate::current_exe_path()?;
        let outcome = selfupdate::apply(&archive, &exe, ui_dir)?;
        self.lock().pending_restart = true;
        logging::log(
            "[Update]",
            &format!(
                "✅ 已在容器内替换程序{}（v{version}），重启后生效",
                if outcome.ui_updated { "与界面文件" } else { "" }
            ),
        );
        Ok(json!({
            "applied": true,
            "version": version,
            "needRestart": true,
            "uiUpdated": outcome.ui_updated,
            "archive": archive.to_string_lossy(),
        }))
    }

    /// request（ by restart ； selfupdate ）。
    pub fn request_restart(&self) -> Result<Value, UpdateError> {
        if !selfupdate::supported() {
            return Err(UpdateError::with_status(
                400,
                "当前部署不支持在网页端重启：请在宿主机执行 docker compose restart",
            ));
        }
        if !selfupdate::request_restart() {
            return Err(UpdateError::with_status(
                400,
                "重启通道未就绪：本进程不是由 headless 容器入口启动的",
            ));
        }
        Ok(json!({ "restarting": true }))
    }
}

// ─── ─────────────────────────────────────────────

/// update（ and config / logging 1 ）：
/// bootstrap when 1 ， by and path use 1 。
static GLOBAL: std::sync::OnceLock<UpdateManager> = std::sync::OnceLock::new();

/// updatecheck by ， use current。
pub fn global_check_key() -> String {
    GLOBAL.get().map(UpdateManager::check_key)
        .unwrap_or_else(|| format!("updateCheck:{DEFAULT_REPO}"))
}

pub fn required_update() -> Option<Value> {
    GLOBAL.get().and_then(|manager| manager.force_update(CURRENT_VERSION))
}

/// （ etc ）
pub fn init_global(manager: UpdateManager) -> UpdateManager {
    let plugins_before = crate::server::core::plugin_host::settings::current();
    if config::sync_installed_update_channel(CURRENT_VERSION) {
        manager.invalidate_channel();
        crate::server::core::plugin_host::settings::notify_changed(plugins_before);
    }
    if GLOBAL.set(manager.clone()).is_ok() {
        let updater = manager.clone();
        // must `crate::spawn_task`： by `ServerState::bootstrap` use ，
        // headless（ / ） bootstrap in ** Tokio when before **
        // up —— `tokio::spawn` in that panic，release is 
        // panic=abort，，restart is 1 
        // （v3.3.0-beta.5 update after none ）。
        crate::spawn_task(async move {
            loop {
                let _ = updater.refresh_policy(CURRENT_VERSION).await;
                tokio::time::sleep(std::time::Duration::from_millis(INDEX_RETRY_MS as u64)).await;
            }
        });
    }
    manager
}
