//! update：、、、file（）。
//!
//! for Node src/workbuddy-update.mjs /。
//! and `mod.rs`（ and ）， is as this is ****：
//! Reason not （ this 「 and 」，mod.rs 「HTTP and file」），
//! when use 。
//!
//! ── 1 has ： ──────────────────────────
//! Node 、 after 1 ignore； this by semver `-` after 
//! （ [`compare_versions`]）， [`pick_listed_release`]（ from Release 
//! 1 ）—— is 「 beta 」 this need ：
//! after `3.1.0-beta.1` and `3.1.0-beta.2` must can after ， as then 1 not 
//! （`/releases/latest` not will return）。

use serde_json::Value;

use crate::server::config::UpdateChannels;

/// default： （ down ）。
///
/// ── Why not is ────────────────────────────────────────
/// already to use （ `Agent2Builder/agent2api-releases` 
/// `release.yml`）： job 1 is ， in tag 
/// as Actions use 。** not Release** —— this 
/// if to ，updatecheck will 「 already is new 」（ not ， is ）。
///
/// ── this is can ────────────────────────────────
/// is ****。 down use is `app.repo`
/// （ `core::update::index`）：，file can ， not 。
/// this 1 available， is as 「 not to can update」。
///
/// `WORKBUDDY_UPDATE_REPO` can （， and not ）。
pub const DEFAULT_REPO: &str = "Agent2Builder/agent2api-releases";

/// （`owner/repo`）。
///
/// ── Why not can use [`DEFAULT_REPO`] ──────────────────────────
/// up 「 / 」 is ****， not is 。
/// （`Agent2Builder`） is Actions ， no has 、
/// no has issue、Star no has —— `repository` will 。
///
/// this not ： is 「 in 」， this is 「 in 」，
/// has 「」 when need need 1 （ not this ）。
///
/// / to `sourceRepository` is （ `build_check_result`）。
pub const SOURCE_REPO: &str = "aimod-cc/Agent2API";

/// GitHub API （Node GITHUB_API）—— [`api_base`] default
pub const GITHUB_API: &str = "https://api.github.com";

/// update （`WORKBUDDY_UPDATE_API_BASE`，empty/ no ）。
///
/// use is 「 not GitHub can update」：、 this 。config after 
/// **updatecheck and down **（、 1 not ）：
/// · checkupdate `{base}/repos/{repo}/releases/latest`，return and GitHub ；
/// 「 beta 」 when `{base}/repos/{repo}/releases?per_page=…`
/// （ is ****，）—— need supports beta this ；
/// · down `base` host（ [`assert_downloadable`]）——
/// this is **config** ， no is has GitHub that 。
pub fn custom_api_base() -> Option<String> {
    std::env::var("WORKBUDDY_UPDATE_API_BASE")
        .ok()
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty())
}

/// update API ： `WORKBUDDY_UPDATE_API_BASE` use ，otherwise is GitHub。
pub fn api_base() -> String {
    custom_api_base().unwrap_or_else(|| GITHUB_API.to_string())
}

/// down ：Release will in github.com or *.githubusercontent.com
/// （Node ALLOWED_HOSTS）
pub const ALLOWED_HOSTS: &[&str] = &[
    "github.com",
    "www.github.com",
    "objects.githubusercontent.com",
    "codeload.github.com",
];

/// down after （Node ALLOWED_HOST_SUFFIX）
pub const ALLOWED_HOST_SUFFIX: &str = ".githubusercontent.com";

/// limit：used for response， （Node 600MB）
pub const MAX_INSTALLER_BYTES: u64 = 600 * 1024 * 1024;

/// update error（ for Node UpdateError，default 502）。
///
/// `status_code` will HTTP （400 parameter/、502 and GitHub ）。
#[derive(Clone, Debug)]
pub struct UpdateError {
    pub message: String,
    pub status_code: i32,
}

impl UpdateError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), status_code: 502 }
    }

    pub fn with_status(status_code: i32, message: impl Into<String>) -> Self {
        Self { message: message.into(), status_code }
    }

    /// 1 error。update by failure body OpenAI 
    /// （ this error in Node try errorPayload ）
    pub fn to_gateway_error(&self) -> crate::server::errors::GatewayError {
        crate::server::errors::GatewayError::with_status(self.status_code, self.message.clone())
    }
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

// ─── ───────────────────────────────────────────────

/// as 。
///
/// for Node parseVersion：empty、optional `v` before ，
/// need `d+.d+.d+` ****（ after / not ）。
/// return None（ when not 「 has new 」，）。
pub fn parse_version(text: &str) -> Option<[u64; 3]> {
    split_version(text).map(|(core, _)| core)
}

/// this is not is ****（`-beta.1` this semver ）？
///
/// ── use ， not Release `prerelease` ──────────
/// 「tag `-` ↔ pre-release」 is 1 （
/// `.agents/skills/release-flow/SKILL.md` ），
/// —— if 1 ， pre-release when 「
/// 」 and 「 is not is beta」 will （ and and not 1 ）。
/// not 1 by ， and [`split_version`] 1 。
pub fn is_prerelease_version(text: &str) -> bool {
    split_version(text)
        .map(|(_, prerelease)| prerelease.is_some())
        .unwrap_or(false)
}

/// ：`(, )`。 and [`parse_version`] 1 
/// （ `v` before 、 need before ）， is `-` after that （ not `-`）。
///
/// ── has `-` ────────────────────────────────────
/// `3.1.0.1` / `3.1.0_beta` this semver 1 「 no has 」，
/// and before 「 after 1 ignore」 as 1 ： not this in Release ，
/// when by for （ by ） by for 。
/// （`+` after ） by semver not and ， this 。
fn split_version(text: &str) -> Option<([u64; 3], Option<&str>)> {
    let trimmed = text.trim();
    let body = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    // `^(\d+)\.(\d+)\.(\d+)`： need before ，（-beta.1 etc ） to down 
    let mut parts = [0u64; 3];
    let mut rest = body;
    for (index, slot) in parts.iter_mut().enumerate() {
        let digits_end = rest.bytes().take_while(u8::is_ascii_digit).count();
        if digits_end == 0 {
            return None;
        }
        // use ASCII ：JS \d in u down is [0-9]
        *slot = rest[..digits_end].parse().ok()?;
        rest = &rest[digits_end..];
        if index < 2 {
            rest = rest.strip_prefix('.')?;
        }
    }
    let head = rest.split('+').next().unwrap_or(rest);
    let prerelease = head.strip_prefix('-').filter(|tail| !tail.is_empty());
    Some((parts, prerelease))
}

/// after （semver 11 ）：return [`compare_versions`]。
///
/// is `-` after （`beta.1` / `rc.2` this ）， by `.` ：
/// · is → by （`beta.2` > `beta.10` in this ）；
/// · < （`alpha.1` < `alpha.beta`）；
/// · is → by ASCII ；
/// · before → （`1.0.0-alpha` < `1.0.0-alpha.1`）。
fn compare_prerelease(left: &str, right: &str) -> i32 {
    let mut left_parts = left.split('.');
    let mut right_parts = right.split('.');
    loop {
        match (left_parts.next(), right_parts.next()) {
            (None, None) => return 0,
            (None, Some(_)) => return -1,
            (Some(_), None) => return 1,
            (Some(a), Some(b)) => {
                // `u64` success「」： before （01） by semver
                // ， this not 
                let order = match (a.parse::<u64>(), b.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    (Ok(_), Err(_)) => std::cmp::Ordering::Less,
                    (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
                    (Err(_), Err(_)) => a.cmp(b),
                };
                if order != std::cmp::Ordering::Equal {
                    return if order == std::cmp::Ordering::Less { -1 } else { 1 };
                }
            }
        }
    }
}

/// ：a > b return 1， etc 0，a < b return -1；none when None。
///
/// ── Whymust ─────────────────────────────────────
/// 「 beta 」 will `3.1.0-beta.1` and `3.1.0-beta.2` 1 to ：
/// this is 「 1 」，beta use beta.1 not to 
/// beta.2（ when 「noneupdate」）。 by semver： 
/// （`1.0.0-beta.1` < `1.0.0`）， is 「beta.1 → beta.2 → 」
/// this 1 「 has new 」， when beta use can up 。
pub fn compare_versions(a: &str, b: &str) -> Option<i32> {
    let (left, left_pre) = split_version(a)?;
    let (right, right_pre) = split_version(b)?;
    for index in 0..3 {
        if left[index] > right[index] {
            return Some(1);
        }
        if left[index] < right[index] {
            return Some(-1);
        }
    }
    Some(match (left_pre, right_pre) {
        (None, None) => 0,
        // ： no has that （`3.1.0` > `3.1.0-beta.1`）
        (None, Some(_)) => 1,
        (Some(_), None) => -1,
        (Some(left), Some(right)) => compare_prerelease(left, right),
    })
}

/// from Release in already 1 。
///
/// and not and ； when and after 。
/// and current by use ， when not after 。
pub fn pick_listed_release(payload: &Value, channels: UpdateChannels) -> Option<&Value> {
    let list = payload.as_array()?;
    let mut best: Option<&Value> = None;
    for item in list {
        if item.get("draft").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let tag = item.get("tag_name").and_then(Value::as_str).unwrap_or("");
        if parse_version(tag).is_none() {
            continue;
        }
        let prerelease = item.get("prerelease").and_then(Value::as_bool) == Some(true)
            || is_prerelease_version(tag);
        if !channels.accepts(prerelease) {
            continue;
        }
        let take = match best {
            None => true,
            Some(current) => {
                let current_tag = current.get("tag_name").and_then(Value::as_str).unwrap_or("");
                compare_versions(tag, current_tag) == Some(1)
            }
        };
        if take {
            best = Some(item);
        }
    }
    best
}

// ─── URL and ─────────────────────────────────────────────

/// ： https and in GitHub down （ for assertDownloadable）。
///
/// Node use `new URL(url)` ： URL 400「 down not is URL」，
/// https 400「 down must is https」， not in 400
/// 「 down not in : <host>」。
pub fn assert_downloadable(url: &str) -> Result<url::Url, UpdateError> {
    let parsed = url::Url::parse(url)
        .map_err(|_| UpdateError::with_status(400, "下载地址不是合法 URL"))?;
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(UpdateError::with_status(400, "下载地址不能包含凭证"));
    }
    let host = parsed.host_str().unwrap_or("").to_lowercase();

    // ── update（WORKBUDDY_UPDATE_API_BASE）───────────────
    // 「config that 」：host or 。scheme config ——
    // http:// http down （ / ）， https://
    // https。 GitHub that not ， old https + 。
    //
    // ： http etc ， sha256 this 1 
    // （file 1 ）。** and can ** use no ，
    // https。
    if let Some(base) = custom_api_base() {
        if let Ok(parsed_base) = url::Url::parse(&base) {
            let base_host = parsed_base.host_str().unwrap_or("").to_lowercase();
            let same_host = !base_host.is_empty()
                && (host == base_host || host.ends_with(&format!(".{base_host}")));
            if same_host {
                if parsed_base.scheme() == "https" && parsed.scheme() != "https" {
                    return Err(UpdateError::with_status(400, "下载地址必须是 https"));
                }
                return Ok(parsed);
            }
        }
    }

    if parsed.scheme() != "https" {
        return Err(UpdateError::with_status(400, "下载地址必须是 https"));
    }
    let allowed = ALLOWED_HOSTS.contains(&host.as_str()) || host.ends_with(ALLOWED_HOST_SUFFIX);
    if !allowed {
        return Err(UpdateError::with_status(
            400,
            format!("下载地址域名不在允许列表内: {host}"),
        ));
    }
    Ok(parsed)
}

/// **** after （，** not **）：Windows is NSIS 
/// `exe`、macOS is `dmg`、Linux is `deb`（AppImage when is 
/// `AppImage` —— [`desktop_shell_suffixes`]）。
///
/// 1 Release will when （ up ），
/// 「 」must by —— otherwise macOS will down exe， 
/// `verify_installer` down ， as 「 has new not 」。
///
/// ── Linux Why need in ──────────────────────────────
/// Linux up 「」 and 「 / headless」 is ** 1 **
/// （`cfg!(target_os = "linux")` ）， path not ：
/// need down to ， need down server。 can is 
/// 「 has no has GUI 」—— by when [`set_desktop_shell`] ，
/// headless not 。
///
/// and [`installer_kind`] not can use `const fn`； down this 
/// not will use to （[`pick_installer`] in that return None， by ）。
pub fn installer_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        "dmg"
    } else if cfg!(target_os = "windows") {
        "exe"
    } else if running_as_appimage() {
        "AppImage"
    } else {
        "deb"
    }
}

/// （ to use ： need not need UAC / / 
/// by ）。 and [`installer_suffix`] 1 ，Linux up 
/// [`running_as_appimage`] —— 「 to 」 is 「file」。
pub fn installer_kind() -> &'static str {
    if !desktop_shell() {
        return "server";
    }
    if cfg!(target_os = "macos") {
        "dmg"
    } else if cfg!(target_os = "windows") {
        "nsis"
    } else if running_as_appimage() {
        "appimage"
    } else {
        "deb"
    }
}

/// Linux **** after ， by （ when ，
/// 1 in use ）； Linux is empty。
///
/// ── by 「 use is 」， not is ─────────────
/// for ， use not （deb use to AppImage
/// can chmod +x ，AppImage use to deb need ）：
/// · is AppImage （`APPIMAGE` empty，
/// [`running_as_appimage`]）→ AppImage： that is 1 update
/// （file）；
/// · otherwise（deb 、 or from ）→ deb ，AppImage 
/// （「 AppImage 」 and ）。
fn desktop_shell_suffixes() -> &'static [&'static str] {
    if cfg!(target_os = "linux") {
        // AppImage `.AppImage` ， for when 1 （ pick_installer）
        if running_as_appimage() {
            &["AppImage"]
        } else {
            &["deb", "AppImage"]
        }
    } else {
        &[]
    }
}

/// need after （、， by ）。
///
/// and [`pick_installer`] ****（ that ）： in 「 has new 
/// no has can down 」 when need 1 after Reason， 1 
/// 「 not to .deb、 its is .AppImage」。
///
/// Linux up by ： to deb / AppImage， / headless to **empty**
/// —— that when [`server_bundle_name`] that server， and after none。
pub fn installer_suffixes() -> Vec<String> {
    if !desktop_shell() {
        return Vec::new();
    }
    if cfg!(target_os = "linux") {
        desktop_shell_suffixes()
            .iter()
            .map(|suffix| format!(".{}", suffix.to_lowercase()))
            .collect()
    } else {
        vec![format!(".{}", installer_suffix())]
    }
}

/// is not is by AppImage 。
///
/// AppImage when ： when set `APPIMAGE` to this AppImage
/// file（`runtime` ， use none need ）。empty「 is AppImage」，
/// this is 「 this to use 」 1 —— path can （ use can can 
/// file、 or from ）。
///
/// Why this need ：AppImage is 「 use new file old file」，
/// to deb etc use 1 、 that AppImage in old 。
fn running_as_appimage() -> bool {
    std::env::var("APPIMAGE")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

// ── 「 in 」────────────────────────────────

/// 。
///
/// default false = headless / （** when as must and before 1 **：
/// Linux up not ， this and 「Linux return None」 etc ）。
/// in path（`desktop-tauri/src/backend.rs`），
/// `agent2api-server` not 。
static DESKTOP_SHELL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// when （ etc ； and `access::set_panel_gate` 1 ）。
pub fn set_desktop_shell(on: bool) {
    DESKTOP_SHELL.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// is in （ [`DESKTOP_SHELL`]）。
pub fn desktop_shell() -> bool {
    DESKTOP_SHELL.load(std::sync::atomic::Ordering::Relaxed)
}

/// `name` （ or when to empty， and Node `String(item?.name || '')` ）
fn asset_name(item: &Value) -> &str {
    item.get("name").and_then(Value::as_str).unwrap_or("")
}

/// （ by ）。
///
/// ── Windows： NSIS ────────────────────────────────
/// NSIS `-setup`，Release will 1 exe（ 
/// ）， that not is —— not will 、 not will 。
///
/// ── macOS： by ─────────────────────────────────────
/// 1 macOS can can up aarch64 / x86_64 / universal dmg：
/// - `universal` can ，；
/// - `aarch64` is Apple Silicon ；
/// - `x86_64` in Intel up can （Apple Silicon need Rosetta ）。
/// no has （ use ） when 1 —— is dmg。
///
/// ── Linux（）： by ， its ──────────────────
/// 1 Linux will amd64 and arm64 （deb / AppImage 1 ），
/// by （`amd64` and `x86_64` / `arm64` and `aarch64` is 1 
/// ， need ）， when by after ， not in 
/// 1 —— is after 。
fn pick_preferred<'a>(candidates: &[&'a Value]) -> Option<&'a Value> {
    if cfg!(target_os = "macos") {
        for keyword in ["universal", "aarch64", "arm64", "x86_64", "x64"] {
            if let Some(found) = candidates
                .iter()
                .find(|item| asset_name(item).to_lowercase().contains(keyword))
            {
                return Some(*found);
            }
        }
        return candidates.first().copied();
    }
    if cfg!(target_os = "linux") {
        // ：`std::env::consts::ARCH` to is x86_64 / aarch64，
        // Docker and use amd64 / arm64（ 1 ， selfupdate）。
        let keywords: &[&str] = match std::env::consts::ARCH {
            "aarch64" => &["aarch64", "arm64"],
            "x86_64" => &["x86_64", "amd64"],
            // unknown： not ， down `.first()` 
            _ => &[],
        };
        for keyword in keywords {
            if let Some(found) = candidates
                .iter()
                .find(|item| asset_name(item).to_lowercase().contains(keyword))
            {
                return Some(*found);
            }
        }
        return candidates.first().copied();
    }
    candidates
        .iter()
        .find(|item| {
            let name = asset_name(item).to_lowercase();
            name.contains("setup") || name.contains("install")
        })
        .copied()
        .or_else(|| candidates.first().copied())
}

/// from Release **** 。
///
/// after by [`installer_suffix`] （Windows → exe，macOS → dmg，Linux 
/// → deb / AppImage）， when by [`pick_preferred`] 1 。
/// no has return None（ 「 down and 」 by ， not is to 1 
/// by ）。
///
/// ── Linux：， as None ────────────────────────
/// Linux up for **** has （ update
/// [`server_bundle_name`] that tar.gz， selfupdate ）。 is 1 
/// ， this by [`desktop_shell`] ：
/// · （ when ）→ by deb / AppImage ；
/// · / headless（）→ None， and before as 1 。
///
/// not up will 1 「 down and 」 by ， will to 
/// 「update in unavailable」。
pub fn pick_installer(assets: Option<&Value>) -> Option<Value> {
    // / headless is empty（ [`installer_suffixes`]）：server，
    // up not this 「 down and 」 by ， will to 「update in unavailable」
    let suffixes = installer_suffixes();
    if suffixes.is_empty() {
        return None;
    }
    let list = assets.and_then(Value::as_array)?;
    // after by ：Windows / macOS 1 ，Linux is 
    // [deb, AppImage]（ [`desktop_shell_suffixes`]）—— 1 can to 
    // after ， deb 、AppImage 。
    for suffix in &suffixes {
        let candidates: Vec<&Value> = list
            .iter()
            .filter(|item| asset_name(item).to_lowercase().ends_with(suffix.as_str()))
            .collect();
        if let Some(chosen) = pick_preferred(&candidates) {
            return Some(serde_json::json!({
                "name": asset_name(chosen),
                // `Number(installer.size) || 0`：/NaN/0 0
                "size": chosen.get("size").and_then(Value::as_u64).unwrap_or(0),
                "url": chosen.get("browser_download_url").and_then(Value::as_str).unwrap_or(""),
            }));
        }
    }
    None
}

/// update in Release ：`agent2api-server-linux-<arch>.tar.gz`。
///
/// ： can file `agent2api-server` and directory `ui/`（
/// selfupdate ）。** this is ** ——
/// CI （.github/workflows/build.yml linux-server job）、 this 、
/// down after by `safe_file_name` to down directory， 1 ； need 1 。
///
/// server；Windows zip only，Linux tar.gz used forupdate。
/// use ， not this 。
pub fn server_bundle_name() -> Option<String> {
    if cfg!(target_os = "windows") && std::env::consts::ARCH == "x86_64" {
        return Some("agent2api-server-windows-amd64.zip".to_string());
    }
    if !cfg!(target_os = "linux") {
        return None;
    }
    // Release use amd64 / arm64 （ and docker TARGETARCH 1 ）
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return None,
    };
    Some(format!("agent2api-server-linux-{arch}.tar.gz"))
}

/// Release after （、、， by ）。
///
/// in 「 has new no has can down 」 when use 「 this 1 has .dmg / .deb」——
/// and [`installer_suffixes`] for ： 1 is 「 need 」， 1 is 「 up has 」。
/// is ** before ** ： fileupdate ， need in 。
pub fn asset_suffixes(assets: &[Value]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for item in assets {
        let name = asset_name(item).to_lowercase();
        // after 1 after ：`.tar.gz` to `.gz`（update by 
        // file， [`server_bundle_name`]， not this ）
        let Some(index) = name.rfind('.') else { continue };
        let suffix = &name[index..];
        if suffix.len() > 1 && !seen.iter().any(|known| known == suffix) {
            seen.push(suffix.to_string());
        }
    }
    seen
}

/// by **** in Release `{name, size, url}`。
///
/// and [`pick_installer`] ： that by after 「 」， this need is 
/// 1 need file（serverupdate / `.sha256` file）。 not to return None
/// —— Release no has this when ，「」 。
pub fn pick_asset_by_name(assets: Option<&Value>, wanted: &str) -> Option<Value> {
    let list = assets.and_then(Value::as_array)?;
    let chosen = list.iter().find(|item| asset_name(item) == wanted)?;
    Some(serde_json::json!({
        "name": asset_name(chosen),
        "size": chosen.get("size").and_then(Value::as_u64).unwrap_or(0),
        "url": chosen.get("browser_download_url").and_then(Value::as_str).unwrap_or(""),
    }))
}

/// Android APK ：`Agent2API-<>.apk`（CI android job by this 
/// ， is ）。
///
/// and [`pick_installer`] Reason： that by ****， need 
/// is 「 1 」 —— in up ， use in up update。
/// this not `cfg!(target_os)`， by after `.apk` 。
///
/// （e.g. when universal and ） when 1 ：current CI 1 ，
/// etc 。
pub fn pick_android_apk(assets: Option<&Value>) -> Option<Value> {
    let list = assets.and_then(Value::as_array)?;
    let chosen = list
        .iter()
        .find(|item| asset_name(item).to_lowercase().ends_with(".apk"))?;
    Some(serde_json::json!({
        "name": asset_name(chosen),
        "size": chosen.get("size").and_then(Value::as_u64).unwrap_or(0),
        "url": chosen.get("browser_download_url").and_then(Value::as_str).unwrap_or(""),
    }))
}

/// JS `String.prototype.trim` empty = White_Space ∪ {U+FEFF}。
/// this as safe_file_name `.trim()` 1 ： then has 1 ， 
/// （ has ， can ）。
fn js_trim(value: &str) -> &str {
    value.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// file：，path and can （ for Node safeFileName）。
///
/// Node is `basename(name).replace(/[\\/:*?"<>|]/g, '_').trim()`，
/// empty to `workbuddy-update.exe`（ by after ， [`installer_suffix`]）。
/// Note `basename` in Windows up 
/// when `\` and `/`， and Node path.basename（POSIX ， `/`）
/// has —— will `\` `_`，file not path，
/// 1 （ this is ** has ** use Windows ， `..\evil.exe` down ）。
pub fn safe_file_name(name: &str) -> String {
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("")
        .to_string();
    let cleaned: String = base
        .chars()
        .map(|ch| {
            if matches!(ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let cleaned = js_trim(&cleaned);
    if cleaned.is_empty() {
        // need after ： down `verify_installer` will after ，
        // to 1 after etc 「」「 not 」。
        // Linux 1 （deb）—— ，
        // `allowed_installer_suffixes`。
        let suffix = if cfg!(target_os = "linux") {
            desktop_shell_suffixes().first().copied().unwrap_or("deb")
        } else {
            installer_suffix()
        };
        format!("agent2api-update.{suffix}")
    } else {
        cleaned.to_string()
    }
}
