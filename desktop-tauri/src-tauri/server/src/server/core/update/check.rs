//! 「checkupdate」 ****：、 and 。
//!
//! ── this ─────────────────────────────────────
//! before will 1 GitHub（`bootstrap` + ），
//! check in 。 after has ：
//! 1. ****：GitHub is 60 / when ， and ** by IP **
//! （：unauthenticated requests are associated with the
//! originating IP address）—— 1 minutes is request，
//! 1 down its ；
//! 2. ** after no has **：403/429 1 error， down 1 check
//! not ， will 1 。
//!
//! in 、 and in `core::task_state`（ `updateCheck:<repo>`）：
//! ** already to ** that 1 ， from ，403/429 by up 
//! to `Retry-After` / `X-RateLimit-Reset` ， not will before 。
//!
//! ── and use 1 ─────────────────────────────────────
//! set 「checkupdate」 by and when is 1 ， has `manual` 1 ：
//! can （ use by by is need in ），**request**
//! and failure —— by not this 。 in 1 when ， after to 
//! that 1 etc in that （ 75 seconds）， not is and 1 。
//!
//! ── / will failure ────────────────────────────
//! when up （`Retry-After` / `X-RateLimit-Reset`）， for 
//! ** that quota** has （ by IP 、 by use ）。set
//! or after ，`api::update` will 
//! [`UpdateManager::clear_check_cooldown`] `retry_at` ， use 
//! new config； not ， when old （ this ）。
//!
//! ── ： / beta / when ────────────────────
//! default，beta default； not current is 。
//! when when from 1 ， has currentupdate。
//! old check，request and GitHub 。
//!
//! ── down Why in ─────────────────────────────────
//! down （`Inner::task`） is 「 this 1 in file」 ，
//! no has （file in failure/ when already delete， `mod.rs` ）。
//! has **check**（ 1 Release） and 。

use serde_json::{json, Value};

use crate::server::config::{self, UpdateChannels};
use crate::server::core::task_state::{self, Claim, TaskState};
use crate::server::logging;

use super::{
    api_base, client, pick_android_apk, pick_asset_by_name, pick_installer, server_bundle_name,
    source, version, UpdateError, UpdateManager, CURRENT_VERSION,
};

/// GitHub API request total when （ down `None`， `mod.rs` ）。
const REQUEST_TIMEOUT_MS: u64 = 30_000;

/// check （ 1 ）。
///
/// Why need has ： by not ， will —— is by 
/// IP 60 / when 。60 seconds「 no 、 1 」 use 1 ，
/// not use by （ will up 1 check when ）。
const MIN_CHECK_GAP_MS: i64 = 60_000;

/// etc 「 in that 1 」 limit： （90 seconds），
/// when after by / up ， not is use in this 。
const WAIT_RUNNING_MS: i64 = 75_000;

fn waiting_message(state: &TaskState) -> String {
    source::GithubSettings::Update.waiting_message(state)
}

impl UpdateManager {
    /// ：** by **（`WORKBUDDY_UPDATE_REPO` can ，
    /// this new and new ， not can use old and ）。
    /// `pub(super)`： has `update::mod` `global_check_key()`（ when 
    /// need this 1 ） and check use ， not to crate。
    pub(super) fn check_key(&self) -> String {
        self.check_key_for(CURRENT_VERSION)
    }

    fn check_key_for(&self, current: &str) -> String {
        let base = format!("updateCheck:{}", self.repository());
        self.force_update(current).and_then(|value| value["targetVersion"].as_str().map(|target| format!("{base}:required:{target}"))).unwrap_or(base)
    }

    fn check_state(&self, current: &str) -> Result<TaskState, String> {
        task_state::read(&self.check_key_for(current))
    }

    fn current_channel_result(&self, state: &TaskState) -> bool {
        state.value.as_ref().is_some_and(|value| {
            value.get("invalidated").and_then(Value::as_bool) != Some(true)
                && (value.get("channels") == Some(&json!(config::update_channels())) || value.get("required") == Some(&json!(true)))
        })
    }

    /// that Release （ and response）。
    fn restore_release(&self, state: &TaskState) {
        self.lock().latest = state
            .value
            .as_ref()
            .filter(|_| self.current_channel_result(state))
            .and_then(|value| value.get("release"))
            .filter(|value| !value.is_null())
            .cloned();
    }

    pub fn invalidate_channel(&self) {
        self.lock().latest = None;
        if let Err(error) = task_state::store_value(&self.check_key(), json!({ "invalidated": true })) {
            logging::log("[Update]", &format!("更新通道缓存失效失败：{error}"));
        }
        if let Err(error) = task_state::schedule_now(&self.check_key()) {
            logging::log("[Update]", &format!("更新通道重新排期失败：{error}"));
        }
    }

    /// to check。
    ///
    /// by `build_check_result` （ by current ），
    /// this is ** and when **：`checkedAt` must is that check when ——
    /// before use 「check xx:xx」， use when will minutes before 。
    fn checked_result(&self, current_version: &str, state: &TaskState) -> Value {
        self.restore_release(state);
        let mut result = self.build_check_result(current_version);
        let channel_current = self.current_channel_result(state);
        result["checked"] = json!(state.last_success_at > 0 && channel_current);
        result["channelChanged"] = json!(state.value.is_some() && !channel_current);
        result["channels"] = json!(config::update_channels());
        result["checkedAt"] = json!(state.last_success_at);
        result["lastAttemptAt"] = json!(state.last_attempt_at);
        result["lastError"] = json!(state.last_error.as_deref().map(|error| {
            source::GithubSettings::Update.describe_error(error, state.retry_at)
        }));
        result["retryAt"] = json!(state.retry_at);
        result["nextCheckAt"] = json!(state.due_at());
        result["checking"] = json!(state.running());
        result
    }

    /// 1 check（`/api/update/status` and ）。
    ///
    /// ** not request**： no when return `checked:false`，
    /// 「check」—— and before 「 before 1 GitHub」 in this 。
    pub fn last_check(&self) -> Value {
        match self.check_state(CURRENT_VERSION) {
            Ok(state) => self.checked_result(CURRENT_VERSION, &state),
            Err(error) => json!({ "checked": false, "lastError": error }),
        }
    }

    /// 「checkupdate」 failure（`updateCheck:<repo>` `retry_at`）。
    ///
    /// use ： / GitHub success after （`api::update`）。 is 
    /// ** 1 quota** when （ by IP 、 by use ）， this 
    /// will —— not use / need for 「checkupdate」 by empty etc 
    /// minutes（ has 1930 seconds ）。 and when not ，
    /// when old by 。
    pub fn clear_check_cooldown(&self) {
        if let Err(error) = task_state::clear_cooldown(&self.check_key()) {
            // not not is error： to will ， 1 
            logging::log("[Update]", &format!("⚠️ 清除检查失败冷却失败: {error}"));
        }
    }

    /// check（set by ）。
    pub async fn check(&self, current_version: &str) -> Result<Value, UpdateError> {
        self.check_with_mode(current_version, true).await
    }

    /// when check（「check」）。
    pub async fn check_scheduled(&self, current_version: &str) -> Result<Value, UpdateError> {
        self.check_with_mode(current_version, false).await
    }

    async fn check_with_mode(&self, current_version: &str, manual: bool) -> Result<Value, UpdateError> {
        self.resolve_repository().await;
        let required = self.force_update(current_version);
        let key = self.check_key_for(current_version);
        let interval = config::scheduled_settings().update_check.interval * 60_000;
        let started = logging::now_ms();
        // checkfailure（ManualBackoff::Respect）： this is 
        // GitHub quota when ， before will 1 403。 / that 
        // clear_check_cooldown（ up ）。
        let guard = match task_state::claim(&key, interval, manual, task_state::ManualBackoff::Respect, MIN_CHECK_GAP_MS)
            .map_err(|error| UpdateError::new(error))?
        {
            Claim::Acquired(guard) => guard,
            Claim::Deferred(mut state) => {
                // already has 1 in （ 1 ， or 1 / when ）： etc 
                // ， not is and 1 GitHub —— not this and 。
                while state.running() && logging::now_ms() - started < WAIT_RUNNING_MS {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    state = task_state::read(&key).map_err(|error| UpdateError::new(error))?;
                }
                // etc to use ； in （ or failure）Reason。
                if state.running() || (state.last_error.is_some() && state.retry_at > logging::now_ms()) {
                    return Err(UpdateError::new(waiting_message(&state)));
                }
                if state.last_success_at > 0 && self.current_channel_result(&state) {
                    return Ok(self.checked_result(current_version, &state));
                }
                return Err(UpdateError::new(waiting_message(&state)));
            }
        };
        let channels = config::update_channels();
        let outcome = self.fetch_release(channels, required.as_ref()).await;
        let interval = config::scheduled_settings().update_check.interval * 60_000;
        match outcome {
            Ok((release, retry_at)) => {
                self.lock().latest = release.clone();
                let result = self.build_check_result(current_version);
                let summary = if result.get("hasUpdate").and_then(Value::as_bool) == Some(true) {
                    format!("发现新版本 {}", result["latestVersion"].as_str().unwrap_or(""))
                } else {
                    "已完成版本检查".to_string()
                };
                let state = guard.finish(true, summary, Some(json!({ "release": release, "channels": channels, "required": required.is_some() })), retry_at, interval)
                    .map_err(|error| UpdateError::new(error))?;
                if channels != config::update_channels() {
                    task_state::schedule_now(&key).map_err(|error| UpdateError::new(error))?;
                }
                Ok(self.checked_result(current_version, &state))
            }
            Err((error, retry_at)) => {
                guard.finish(false, format!("检查失败：{}", error.message), None, retry_at, interval)
                    .map_err(|error| UpdateError::new(error))?;
                Err(error)
            }
        }
    }

    /// ， and up 。
    ///
    /// `/releases/latest`； beta when from by 。
    /// 404 or current no has as noneupdate。
    async fn fetch_release(
        &self,
        channels: UpdateChannels,
        required: Option<&Value>,
    ) -> Result<(Option<Value>, i64), (UpdateError, i64)> {
        let repository = self.resolve_repository().await;
        let beta = channels.beta;
        // update and GitHub use Release 。
        let target = required.and_then(|rule| rule["targetVersion"].as_str());
        let url = if let Some(target) = target {
            format!("{}/repos/{repository}/releases/tags/v{}", api_base(), target.trim_start_matches(['v', 'V']))
        } else if beta {
            format!(
                "{}/repos/{repository}/releases?per_page={RELEASE_LIST_PER_PAGE}",
                api_base()
            )
        } else {
            format!("{}/repos/{repository}/releases/latest", api_base())
        };
        let response = client::fetch_json(
            &url,
            &client::github_headers(),
            REQUEST_TIMEOUT_MS,
            |payload| {
                if beta && target.is_none() {
                    payload.is_array()
                } else {
                    payload.get("tag_name").and_then(Value::as_str).is_some()
                }
            },
        )
        .await
        .map_err(|error| (error, 0))?;
        let status = response.status.as_u16();
        let retry_at = retry_deadline(&response.headers, status);
        if status == 404 {
            // 404 has ： 「 no has Release」（GitHub 
            // as ）、update no request that path（`WORKBUDDY_UPDATE_API_BASE`
            // `/releases/latest`、beta need when will this ）。 by 
            // 「noneupdate」， not is checkfailure —— after will 1 
            // error、 that available 。
            return Ok((None, retry_at));
        }
        if status == 403 || status == 429 {
            return Err((
                UpdateError::new(source::GithubSettings::Update.rate_limit_message(retry_at)),
                retry_at,
            ));
        }
        if !response.status.is_success() {
            return Err((
                UpdateError::new(format!("GitHub 返回 HTTP {status}")),
                retry_at,
            ));
        }
        let from_official = response.official && source::is_official_api(&url);
        let payload = response.value;
        let raw = if let Some(target) = target {
            if payload["draft"] == true || version::compare_versions(payload["tag_name"].as_str().unwrap_or(""), target) != Some(0) {
                return Err((UpdateError::new("索引指定的强制更新版本与发布信息不匹配"), retry_at));
            }
            Some(payload)
        } else if beta {
            version::pick_listed_release(&payload, channels).cloned()
        } else {
            let prerelease = payload.get("prerelease").and_then(Value::as_bool) == Some(true)
                || version::is_prerelease_version(payload["tag_name"].as_str().unwrap_or(""));
            (payload["draft"] != true && channels.accepts(prerelease)).then_some(payload)
        };
        if from_official {
            if let Some(release) = raw.as_ref() {
                super::integrity::remember_release(release, &repository);
            }
        }
        Ok((raw.map(|item| release_json(&item, &repository)), retry_at))
    }
}

/// 1 request Release。
///
/// 30 「 1 」 this ： ，
/// response（ not file）。** to ** —— will to 
/// old ， will that 1 。
const RELEASE_LIST_PER_PAGE: u32 = 30;

/// GitHub Release for → （`build_check_result` and update）。
///
/// use ： to is for （`/releases/latest`），beta is 
/// 1 —— by Node for ，** in this 1 **。
fn release_json(payload: &Value, repository: &str) -> Value {
    let text = |key: &str| {
        payload
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let raw_tag = text("tag_name");
    let tag = raw_tag
        .strip_prefix('v')
        .or_else(|| raw_tag.strip_prefix('V'))
        .unwrap_or(&raw_tag)
        .to_string();
    let name = text("name");
    let page_url = text("html_url");
    let published_at = text("published_at");
    // server and optionalfile：Linux used forupdate，Windows used for。
    // and server and return， by can 。
    let assets = payload
        .get("assets")
        .and_then(Value::as_array)
        .map(|items| {
            Value::Array(
                items
                    .iter()
                    .filter(|asset| {
                        let Some(url) = asset.get("browser_download_url").and_then(Value::as_str)
                        else {
                            return false;
                        };
                        if source::is_resource(url) {
                            return url.starts_with(&format!(
                                "https://github.com/{repository}/releases/download/{raw_tag}/"
                            ));
                        }
                        super::custom_api_base().is_some()
                            && super::assert_downloadable(url).is_ok()
                    })
                    .cloned()
                    .collect(),
            )
        });
    let assets = assets.as_ref();
    // 「 has new no has can down 」 when need can **Why**： 「 this 1 to 
    // file」 down 。 is ** before ** —— after as empty when need 
    // 「 up its has .dmg / .deb， is no has this 」； this in 
    // `filteredBySource` （ is 「 file」，
    // and 「 up no 」 not ）。
    let raw_assets: &[Value] = payload
        .get("assets")
        .and_then(Value::as_array)
        .map(|list| list.as_slice())
        .unwrap_or(&[]);
    let kept = assets.and_then(Value::as_array).map(Vec::len).unwrap_or(0);
    let asset_probe = json!({
        "assetsTotal": raw_assets.len(),
        "suffixes": version::asset_suffixes(raw_assets),
        "filteredBySource": kept < raw_assets.len(),
    });
    let bundle = server_bundle_name();
    let bundle_asset = bundle
        .as_deref()
        .and_then(|name| pick_asset_by_name(assets, name));
    let bundle_checksums = bundle
        .as_deref()
        .and_then(|name| pick_asset_by_name(assets, &format!("{name}.sha256")));
    json!({
        "tag": tag,
        "name": if name.is_empty() { raw_tag } else { name },
        "notes": text("body").chars().take(4000).collect::<String>(),
        "publishedAt": if published_at.is_empty() { text("created_at") } else { published_at },
        "pageUrl": if page_url.is_empty() { format!("https://github.com/{repository}/releases") } else { page_url },
        "prerelease": payload.get("prerelease").and_then(Value::as_bool) == Some(true)
            || version::is_prerelease_version(&tag),
        "asset": pick_installer(assets),
        // this 1 up file（ after / total / has no has update）——
        // 「asset as empty」 when Reason， [`asset_probe`] and 
        // `build_check_result` expected* that 1 
        "assetProbe": asset_probe,
        // （APK） 。 and `asset` and not is 1 ： can can when 
        // and ， that ， not this for that 。
        "androidAsset": pick_android_apk(assets),
        "serverBundle": bundle_asset.unwrap_or(Value::Null),
        "serverChecksums": bundle_checksums.unwrap_or(Value::Null),
    })
}

/// from response「 can request when 」（ms；0 = up no to limit）。
///
/// ， 1 （）：
/// - `Retry-After`：seconds or HTTP （ use ）；
/// - `X-RateLimit-Remaining: 0` when `X-RateLimit-Reset`：UTC seconds
/// （ use ； need etc to this when after request）；
/// - 403 / 429 ** 1 minutes**： no to when 
/// （ for is 「 etc 1 minutes」）。
///
/// this will （`retry_at`），** not will before **。
fn retry_deadline(headers: &reqwest::header::HeaderMap, status: u16) -> i64 {
    let now = logging::now_ms();
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    let retry = header("retry-after").and_then(|value| {
        value.parse::<i64>().ok().map(|seconds| now.saturating_add(seconds.max(0).saturating_mul(1000)))
            .or_else(|| chrono::DateTime::parse_from_rfc2822(value).ok().map(|date| date.timestamp_millis()))
    }).unwrap_or(0);
    let reset = if header("x-ratelimit-remaining") == Some("0") {
        header("x-ratelimit-reset").and_then(|value| value.parse::<i64>().ok())
            .map(|seconds| seconds.saturating_mul(1000).saturating_add(1000)).unwrap_or(0)
    } else { 0 };
    let minimum = if status == 403 || status == 429 { now + 60_000 } else { 0 };
    retry.max(reset).max(minimum)
}
