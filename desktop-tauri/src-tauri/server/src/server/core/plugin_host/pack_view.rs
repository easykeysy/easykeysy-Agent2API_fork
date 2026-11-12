//! already need 、 and path。

use std::path::{Path, PathBuf};

use agent2api_provider_protocol as proto;
use base64::Engine;
use serde::Serialize;

use super::install::{plugins_dir, read_manifest, PackRegistry};

/// already need （ use ）。
///
/// and [`super::InstalledPack`] ： this is ** to **， 
/// （directory is in 、 is in 、current is in ），
/// 。 「file no 」， not is 1 no 。
///
/// `rename_all = "camelCase"`： this is ** before **（`plugins-page.tsx`
/// ）， `InstalledPack` is （ snake_case and has 
/// ）。 use ， as 「 1 」 and 1 。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSummary {
    /// id。
    pub id: String,
    /// （ `name`； not to when id）。
    pub name: String,
    pub description: Option<String>,
    /// 。
    pub version: String,
    pub required_version: Option<String>,
    pub update_required: bool,
    pub compatibility_error: Option<String>,
    /// is enabled。
    pub enabled: bool,
    /// is already （ when as true —— 「 already / 」）。
    pub signed: bool,
    /// when （ms）。
    pub installed_at: i64,
    /// provider（id / / is enabled）。
    pub providers: Vec<PackProviderSummary>,
    pub host: Option<serde_json::Value>,
    pub provider_category: super::ProviderCategory,
    /// is in （false = / ）。
    pub entry_present: bool,
    /// this （ and not 1 when need ）。
    pub protocol: u32,
    /// （optional）。
    pub homepage: Option<String>,
    /// （optional，`data:image/png;base64,…` ； no has then as null）。
    ///
    /// ── Why data URI not is to 1 URL ─────────────────
    /// in directory（`plugins/<id>/<version>/assets/…`）， not is 
    /// `ui/` down —— `ui_dir` file， as 1 
    /// can anydirectory by ， etc 「 」「 can 」。
    /// response not use add， not file 。
    ///
    /// limit [`ICON_MAX_BYTES`]： after need ， MB 
    /// will 。
    pub icon: Option<String>,
    /// this （empty = after ）。
    ///
    /// ── client use ────────────────────────────────────────────
    /// from this ： no no has this ， after 
    /// （ and 「 not in ， this not in 」 1 then ， is from provider to 
    /// ）。`url` is ，client use iframe 。
    pub ui: Vec<PackUiSummary>,
}

/// 1 need 。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackUiSummary {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    /// （`/plugins/<pack>/ui/<entry>`）。
    pub url: String,
    /// default（client id， `g-overview`）。
    ///
    /// ** not ** this ： id is client （ and has 
    /// default）， that to client。 and 
    /// in client（ not by 1 ）。
    pub group: Option<String>,
    pub order: i32,
}

/// 1 provider need 。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackProviderSummary {
    pub id: String,
    pub label: String,
    /// this 1 is enabled（ use in up ）。
    pub enabled: bool,
    /// this 1 is when （ in and success）。
    pub loaded: bool,
}

/// filelimit（2MB）。
///
/// is use 44px ， KB。 this is 
/// 4K —— response will new KB
/// to MB， can not （）， not 。
const ICON_MAX_BYTES: u64 = 2 * 1024 * 1024;

/// from directory， data URI。
///
/// ── file ──────────────────────────────────────────────
/// `icons` is 1 **directory**（ for ）。directory no has ，
/// by 1 in ：`logo.*` → `icon.*` → directory down any
/// 1 。 not is 「 1 file」， is as can use `logo.png`
/// ， not is 。
///
/// 1 failurereturn `None`（）， not —— is ，
/// not this 「」 this can failure。
fn pack_icon(dir: &Path, manifest: Option<&proto::PackManifest>) -> Option<String> {
    let icons_dir = manifest?.icons.as_ref()?;
    if icons_dir.starts_with(['/', '\\'])
        || icons_dir.contains(':')
        || icons_dir.split(['/', '\\']).any(|part| part == "..")
    {
        return None;
    }
    let icons_dir = dir.join(icons_dir);
    let mut candidates: Vec<PathBuf> = Vec::new();
    for stem in ["logo", "icon"] {
        for ext in ["png", "webp", "jpg", "jpeg", "svg"] {
            candidates.push(icons_dir.join(format!("{stem}.{ext}")));
        }
    }
    // up no in when ，directoryany 1 （ 1 1 ）
    if let Ok(entries) = std::fs::read_dir(&icons_dir) {
        let mut rest: Vec<PathBuf> = entries
            .filter_map(|item| item.ok())
            .map(|item| item.path())
            .filter(|path| path.is_file() && is_image_path(path))
            .collect();
        rest.sort();
        candidates.extend(rest);
    }
    let path = candidates
        .into_iter()
        .find(|path| path.is_file())?
        .canonicalize()
        .ok()?;
    if !path.starts_with(dir.canonicalize().ok()?) {
        return None;
    }
    if std::fs::metadata(&path).ok()?.len() > ICON_MAX_BYTES {
        return None;
    }
    let bytes = std::fs::read(&path).ok()?;
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "webp" => "image/webp",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

/// is not is can 。
fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "png" | "webp" | "jpg" | "jpeg" | "svg"
    )
}

/// UI client can use （`url` to ）。
///
/// ── Why need `entry` ────────────────────────────────────
/// `entry` is 。 not ， 1 can `../../../../etc/passwd`
/// file —— that is path。 this in ****：must is 
/// for path、 not `..`、 `.html` 。 that will 1 （）。
fn pack_ui_pages(pack_id: &str, manifest: Option<&proto::PackManifest>) -> Vec<PackUiSummary> {
    let Some(manifest) = manifest else {
        return Vec::new();
    };
    manifest
        .ui
        .iter()
        .filter(|page| ui_entry_ok(&page.entry))
        .map(|page| PackUiSummary {
            id: page.id.clone(),
            title: page.title.clone(),
            icon: page.icon.clone(),
            // id and URL， need URL-safe： id by 
            // （ `[a-z0-9-]`）， this already `..` and for path
            url: format!("/plugins/{pack_id}/ui/{}", page.entry.replace('\\', "/")),
            // default， not （ by 
            // `PackUiSummary::group` ）
            group: page.group.clone(),
            order: page.order,
        })
        .collect()
}

/// UI path is （ for 、 not `..`、 `.html` ）。
fn ui_entry_ok(entry: &str) -> bool {
    let normalized = entry.replace('\\', "/");
    if normalized.is_empty() || normalized.starts_with('/') || normalized.contains(':') {
        return false;
    }
    if normalized.split('/').any(|segment| segment == "..") {
        return false;
    }
    normalized.to_ascii_lowercase().ends_with(".html")
}

/// already （enabled ）。
///
/// not not ：`InstalledPack` `entry` and ， can 
/// 1 「， new 」。failurereturnempty —— that is 「 no 」。
pub fn list_packs() -> Vec<PackSummary> {
    let registry = PackRegistry::load();
    registry
        .packs
        .iter()
        .map(|pack| {
            let dir = plugins_dir().join(&pack.id).join(&pack.version);
            let compatibility_error = super::compatibility::version_error(&pack.id, &pack.version);
            let update_required = compatibility_error.is_some();
            let manifest = read_manifest(&dir).ok();
            let name = manifest
                .as_ref()
                .map(|item| item.name.clone())
                .unwrap_or_else(|| pack.id.clone());
            let description = std::fs::read_to_string(dir.join("pack.json"))
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                .and_then(|value| value["description"].as_str().map(str::to_string));
            let protocol = manifest.as_ref().map(|item| item.protocol).unwrap_or(0);
            let homepage = manifest.as_ref().and_then(|item| item.homepage.clone());
            let declared = manifest
                .as_ref()
                .map(|item| item.providers.clone())
                .unwrap_or_else(|| pack.manifest_providers.clone());
            let providers = declared
                .into_iter()
                .filter(|item| !crate::server::core::providers::availability::is_suspended(item.id.as_str()))
                .map(|item| {
                    let id = item.id.0.clone();
                    let enabled = pack.providers.get(&id).copied().unwrap_or(true);
                    PackProviderSummary {
                        loaded: pack.enabled && enabled && super::serves_provider(&id)
                            && super::provider_pack(&id).as_deref() == Some(pack.id.as_str()),
                        enabled,
                        label: item.label,
                        id,
                    }
                })
                .collect();
            PackSummary {
                id: pack.id.clone(),
                name,
                description,
                version: pack.version.clone(),
                required_version: super::compatibility::minimum_version(&pack.id).map(str::to_string),
                update_required,
                compatibility_error,
                enabled: pack.enabled,
                signed: pack.signed,
                installed_at: pack.installed_at,
                providers,
                host: super::host_for_pack(&pack.id).map(|host| {
                    let status = host.status();
                    serde_json::json!({
                        "running": status.running, "providers": status.providers,
                        "crashes": status.crashes, "disabled": status.disabled,
                        "lastError": status.last_error,
                    })
                }),
                provider_category: super::ownership::category(&dir),
                entry_present: dir.join(&pack.entry).exists(),
                protocol,
                homepage,
                icon: pack_icon(&dir, manifest.as_ref()),
                ui: if update_required { Vec::new() } else { pack_ui_pages(&pack.id, manifest.as_ref()) },
            }
        })
        .collect()
}

// ─── UI ─────────────────────────────────────────────

/// `/plugins/{pack}/ui/{rest}` up filepath（`None` = not in / ）。
///
/// ── （， 1 is ）──────────────────────────
/// 1. ** already enabled directory**： id must in and `enabled`，
/// directory by to （ not is URL to ）—— is URL `{pack}` is 1 
/// ， not can to anydirectory；
/// 2. **path 1 **：`..` / for path / 1 ， not 
///      `ui/` directory；
/// 3. ** `ui/` before down file**： `bin/`、`pack.json` not for 
/// （ that is after and ， before no has by to ）。
///
/// and `static_files::serve` not is use ： that is `ui_dir`、 need 
/// web shim（）， not 。 use will 1 。
pub fn resolve_ui_file(pack_id: &str, rest: &str) -> Option<PathBuf> {
    if pack_id.is_empty() || rest.is_empty() {
        return None;
    }
    // ① must already and enabled（disabled = not this can ）
    let registry = PackRegistry::load();
    let pack = registry
        .packs
        .iter()
        .find(|item| item.id == pack_id && item.enabled)?;
    if super::compatibility::version_error(&pack.id, &pack.version).is_some() {
        return None;
    }
    let root = plugins_dir().join(&pack.id).join(&pack.version).join("ui");
    // ② 1 （ and `static_files::resolve_file` 1 then ）
    let mut segments: Vec<&str> = Vec::new();
    for segment in rest.split('/') {
        if segment.contains(':') {
            return None;
        }
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            other => {
                // ：Windows ， 1 this （`..\..\x` this ）
                if other.contains('\\') {
                    return None;
                }
                segments.push(other);
            }
        }
    }
    if segments.is_empty() {
        return None;
    }
    let mut path = root.clone();
    for segment in &segments {
        path.push(segment);
    }
    // ③ 1 after pathmust in `ui/` down （：`..` already in up ）
    let canonical_root = root.canonicalize().ok()?;
    let canonical = path.canonicalize().ok()?;
    if !canonical.starts_with(&canonical_root) {
        return None;
    }
    canonical.is_file().then_some(canonical)
}
