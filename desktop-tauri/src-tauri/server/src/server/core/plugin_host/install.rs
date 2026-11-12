//! 、 and 。
//!
//! ── （ not can ）──────────────────────────────────────
//! 1. **sha256**：zip need ，used for（`InstalledPack::sha256`）。
//! 「file」 and 「 after 」—— in [`verify_installed`]；
//! 2. ****：`pack.json` `protocol` must etc 
//! [`proto::PROTOCOL_VERSION`]。「 not 」；
//! 3. ****：`pack.json` `signature` when ed25519 （，
//! [`OFFICIAL_PUBKEYS`]）。 is ****（ [`signing_payload`]），
//! not is zip —— zip / when can ， is need 。
//!
//! and 。** **（/must
//! has ）， —— and 「 / 」 1 。
//! not ** 1 **： that ，or not is that 。
//!
//! ── from ──────────────────────────────────────────────
//! [`OFFICIAL_PUBKEYS`]（ 32 ， can ）。
//! not can —— 1 can config etc no has 。
//!
//! ── directory ────────────────────────────────────────────────
//! ```text
//! {config_dir}/plugins/
//! plugins.json （、）
//! agent2api-providers/ id
//! 0.1.0/ directory（ can and ，）
//!       pack.json
//!       bin/providers.exe
//! current -> 0.1.0 current（Windows up is current.txt）
//! ```
//!
//! ── Why need directory not is ───────────────────────────
//! 1 failure when need can 。 will 「 new not 」「 this can 
//! no 」。 and + `current` is mihomo that already in use 
//! （`sub_kernel::layout`）， use 1 1 。
//!
//! ── from （zip up ）────────────────────────────────────
//! use to is zip（`scripts/pack.mjs`（Agent2Providers ） ， 13MB）。
//! [`install_zip`] ： to ** when directory**， and ，
//! `rename` directory —— not will in `plugins/` 
//! （otherwise `bootstrap` will spawn 1 not ）。
//!
//! zip slip in up 1 ： when path， when 1 
//! `..`（ [`validate_manifest`]）。

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use agent2api_provider_protocol as proto;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::server::logging;

pub use super::pack_view::{list_packs, resolve_ui_file, PackProviderSummary, PackSummary, PackUiSummary};

/// ：`(key_id, 32 ed25519 )`。
///
/// ── Why is ──────────────────────────────────────
/// 1 can config etc no has —— need can config， can 
/// 1 。****update， and 。
///
/// ── ──────────────────────────────────────────────
/// can has ： new （ old can ）， new after old 。
/// `key_id` is to this use ， not is —— 「 has no has this 」。
///
/// ── in this 1 is ────────────────────────────────────────
/// `official-2026`：2026-10 ， in and GitHub
/// Actions secret（`AGENT2API_PLUGIN_KEY`），CI after 。
/// when new 、 etc after old 。
pub const OFFICIAL_PUBKEYS: &[(&str, &str)] = &[(
    "official-2026",
    "7cdac378ff9bf23b72ae1b686f19859b3c802c26eed13c699f7df39f027352d6",
)];

/// 。
///
/// not is bool：`Unsigned`（ no ） and `Invalid`（ not ） 
/// not —— before and ， after must。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureState {
    /// used for ， and 。
    Verified,
    /// no has used for （， or is ）。
    Unsigned,
    /// not （ not / not in / not ）。
    Invalid,
}

impl SignatureState {
    /// is 「 already 」（ use ）。
    pub fn is_signed(self) -> bool {
        matches!(self, Self::Verified)
    }
}

/// 1 。
///
/// `dir` is ** already ** directory（ need sha256），`target` is current。
/// return `(, failureReason)` —— Reason in `Invalid` when has ，used forfailure 。
pub fn verify_signature(
    dir: &Path,
    manifest: &proto::PackManifest,
    target: &str,
) -> (SignatureState, Option<String>) {
    let Some(signature) = manifest.signature.as_ref() else {
        return (SignatureState::Unsigned, None);
    };
    // and not 1 ：「 no has used for 」。
    // 1 ， has that can —— signing_payload。
    if signature.target.as_deref() != Some(target) {
        return (SignatureState::Unsigned, None);
    }
    if !signature.algorithm.eq_ignore_ascii_case("ed25519") {
        return (
            SignatureState::Invalid,
            Some(format!("不支持的签名算法：{}", signature.algorithm)),
        );
    }
    let Some((_, public_hex)) = OFFICIAL_PUBKEYS
        .iter()
        .find(|(id, _)| *id == signature.key_id)
    else {
        return (
            SignatureState::Invalid,
            Some(format!(
                "签名密钥 {} 不在本版本的信任列表里（可能需要更新网关）",
                signature.key_id
            )),
        );
    };
    let Some(public_bytes) = hex_to_32(public_hex) else {
        // （， has can ）： by not and down Reason
        return (
            SignatureState::Invalid,
            Some(format!("内置公钥 {} 的格式无效", signature.key_id)),
        );
    };
    let Some(entry) = manifest.entry.get(target) else {
        return (
            SignatureState::Invalid,
            Some("包里没有本平台的入口，无法验签".to_string()),
        );
    };
    let Some(binary_hash) = sha256_file(&dir.join(entry)) else {
        return (
            SignatureState::Invalid,
            Some("读不到入口二进制，无法验签".to_string()),
        );
    };
    let payload = proto::signing_payload(manifest, target, &binary_hash, &collect_ui_files(dir));
    let Ok(signature_bytes) = base64::engine::general_purpose::STANDARD.decode(signature.value.trim())
    else {
        return (SignatureState::Invalid, Some("签名不是合法的 base64".to_string()));
    };
    match verify_ed25519(&public_bytes, payload.as_bytes(), &signature_bytes) {
        true => (SignatureState::Verified, None),
        false => (
            SignatureState::Invalid,
            Some("签名与本包内容不匹配（包可能被改过，或不是它自称的发布者签的）".to_string()),
        ),
    }
}

/// UI ：`[( for POSIX path, sha256), …]`， by path。
///
/// ── Why need UI ──────────────────────────────────
/// by 、 in origin down ， can to （ etc can 
/// all protected API）。 ， 1 can exe ****
/// 1 js， —— that etc to 1 after 。
///
/// and `scripts/sign.mjs` `collectUiFiles` ： `ui/` before down 
/// file。path 1 `/`（Windows up need in Linux up ）。
fn collect_ui_files(dir: &Path) -> Vec<(String, String)> {
    let root = dir.join("ui");
    let mut files: Vec<(String, String)> = Vec::new();
    collect_ui_files_into(&root, &root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

/// （directory not in when not —— no has UI is ）。
fn collect_ui_files_into(root: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            collect_ui_files_into(root, &path, out);
            continue;
        }
        let Some(hash) = sha256_file(&path) else {
            continue;
        };
        let Some(relative) = path.strip_prefix(root.parent().unwrap_or(root)).ok() else {
            continue;
        };
        // 1 POSIX ： need 1 ， Windows up 
        // `Path::display()` to is 
        let name = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/");
        out.push((name, hash));
    }
}

/// ed25519 （`Ok` valid）。
fn verify_ed25519(public: &[u8; 32], message: &[u8], signature: &[u8]) -> bool {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let Ok(key) = VerifyingKey::from_bytes(public) else {
        return false;
    };
    let Ok(signature) = Signature::from_slice(signature) else {
        return false;
    };
    key.verify(message, &signature).is_ok()
}

/// （64 ）→ 32 。
fn hex_to_32(text: &str) -> Option<[u8; 32]> {
    let text = text.trim();
    if text.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (index, chunk) in text.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk).ok()?;
        out[index] = u8::from_str_radix(pair, 16).ok()?;
    }
    Some(out)
}

/// file sha256（）。 not to return None。
fn sha256_file(path: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(
        hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

/// 1 。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstalledPack {
    /// id。
    pub id: String,
    /// current。
    pub version: String,
    /// is enabled（ use can in up ）。
    #[serde(default = "yes")]
    pub enabled: bool,
    /// provider （ use can its in 1 ）。
    #[serde(default)]
    pub providers: BTreeMap<String, bool>,
    /// for path（ for directory）。
    pub entry: String,
    /// sha256（ when ，used for）。
    #[serde(default)]
    pub sha256: String,
    /// is already and 。
    #[serde(default)]
    pub signed: bool,
    /// when （ms）。
    #[serde(default)]
    pub installed_at: i64,
    /// provider （， pack.json）。
    #[serde(default)]
    pub manifest_providers: Vec<proto::PackProvider>,
}

fn yes() -> bool {
    true
}

/// file（`plugins.json`）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PackRegistry {
    #[serde(default)]
    pub packs: Vec<InstalledPack>,
}

/// directory。
pub fn plugins_dir() -> PathBuf {
    crate::paths::config_dir().join("plugins")
}

/// path。
pub fn registry_path() -> PathBuf {
    plugins_dir().join("plugins.json")
}

impl PackRegistry {
    /// （ not in = empty； = empty and ）。
    ///
    /// ── Why not ────────────────────────────────────────
    /// is 「」 ， in directory。 not when 
    /// empty， use new can —— will can unavailable，
    /// is config。
    pub fn load() -> Self {
        let path = registry_path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        match serde_json::from_str(&text) {
            Ok(registry) => registry,
            Err(e) => {
                logging::log("Plugin", &format!("注册表损坏，按空表处理：{e}"));
                Self::default()
            }
        }
    }

    /// （）。
    pub fn save(&self) -> Result<(), String> {
        let path = registry_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("建目录失败: {e}"))?;
        }
        let text =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {e}"))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|e| format!("写临时文件失败: {e}"))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("替换注册表失败: {e}"))
    }

    /// 1 already enabled 。
    pub fn enabled(&self) -> Option<&InstalledPack> {
        self.packs.iter().find(|pack| pack.enabled)
    }

    /// provider is by already enabled 。
    pub fn provider_enabled(&self, provider: &str) -> bool {
        if crate::server::core::providers::availability::is_suspended(provider) { return false; }
        self.packs.iter().any(|pack| {
            pack.enabled
                && pack
                    .manifest_providers
                    .iter()
                    .any(|p| p.id.as_str() == provider)
                && pack.providers.get(provider).copied().unwrap_or(true)
        })
    }
}

/// 1 。
///
/// ── return error need can to use ──────────────────────────
/// failure when this 。 need 「Why」 and 「」，
/// not need 「failure」。
pub fn validate_manifest(manifest: &proto::PackManifest, host_protocol: u32) -> Result<(), String> {
    if manifest.protocol != host_protocol {
        return Err(format!(
            "这个插件包要求协议版本 {}，而当前网关是 {}。请更新插件包（或更新网关）。",
            manifest.protocol, host_protocol
        ));
    }
    if manifest.providers.is_empty() {
        return Err("插件包里没有声明任何提供商。".to_string());
    }
    if let Some(provider) = manifest.providers.iter().find(|provider| super::ownership::reserved(provider.id.as_str())) {
        return Err(format!("提供商「{}」是本体保留身份，插件包不得声明，请移除后重新安装。", provider.id.as_str()));
    }
    if manifest.entry.is_empty() {
        return Err("插件包没有声明入口二进制。".to_string());
    }
    // pathmust is for path and not （ zip slip）
    for (target, entry) in &manifest.entry {
        // ── Why to need ────────────────────────────────
        // `Path::new(x).is_absolute()` is ** by current** ：Windows up 
        // `/usr/bin/evil` not for path（ no has ）， in is 
        // 「 to 」 。， need `/`
        // or `\` path。
        let path = Path::new(entry);
        if path.is_absolute()
            || entry.starts_with('/')
            || entry.starts_with('\\')
            || entry.chars().nth(1) == Some(':')
        {
            return Err(format!("入口路径必须是相对路径（{target}: {entry}）"));
        }
        if entry.split(['/', '\\']).any(|part| part == "..") {
            return Err(format!("入口路径不能包含 ..（{target}: {entry}）"));
        }
    }
    Ok(())
}

/// as current 1 。
pub fn entry_for_target(manifest: &proto::PackManifest, target: &str) -> Option<String> {
    // ， by 「-」（ not not 1 ）
    if let Some(entry) = manifest.entry.get(target) {
        return Some(entry.clone());
    }
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    manifest
        .entry
        .iter()
        .find(|(key, _)| key.contains(arch) && key.contains(os))
        .map(|(_, entry)| entry.clone())
}

/// 1 directory `pack.json`。
pub fn read_manifest(dir: &Path) -> Result<proto::PackManifest, String> {
    let path = dir.join("pack.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("读不到 pack.json（{}）: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("pack.json 不是合法 JSON: {e}"))
}

// ─── / / ─────────────────────────────────────

/// before ； by use 。
pub fn install_zip(bytes: &[u8]) -> Result<PackSummary, String> {
    super::packages::prepare(bytes, None, None, false)?.install()
}

/// zip to directory（path）。
pub(super) fn extract_zip(bytes: &[u8], target: &Path) -> Result<(), String> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("不是合法的 zip 包: {e}"))?;
    if archive.len() > 4096 {
        return Err("插件包文件数量超过上限".to_string());
    }
    let mut expanded = 0_u64;
    let mut actual_expanded = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|e| format!("读 zip 条目失败: {e}"))?;
        let declared_size = file.size();
        expanded = expanded.saturating_add(declared_size);
        if expanded > 128 * 1024 * 1024 {
            return Err("插件包解压后超过 128 MB 上限".to_string());
        }
        // `enclosed_name` already 「 not can directory」 （ `..` and 
        // for path）；return None 1 ， not path
        let Some(relative) = file.enclosed_name() else {
            return Err(format!("包里有越界路径，已拒绝安装：{}", file.name()));
        };
        let out_path = target.join(&relative);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("建目录失败: {e}"))?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("建目录失败: {e}"))?;
        }
        let mut out = std::fs::File::create(&out_path)
            .map_err(|e| format!("写文件失败（{}）: {e}", out_path.display()))?;
        let mut file_expanded = 0_u64;
        loop {
            let remaining = 128 * 1024 * 1024 - actual_expanded;
            // after 1 ， EOF， zip CRC 。
            let read_len = buffer.len().min((remaining + 1) as usize);
            let count = match file.read(&mut buffer[..read_len]) {
                Ok(count) => count,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return Err("解包失败：读取条目或 CRC 校验失败".to_string()),
            };
            if count == 0 {
                break;
            }
            if count as u64 > remaining {
                return Err("插件包解压后超过 128 MB 上限".to_string());
            }
            actual_expanded += count as u64;
            file_expanded += count as u64;
            if file_expanded > declared_size {
                return Err("插件包条目实际解压尺寸与声明不一致".to_string());
            }
            out.write_all(&buffer[..count])
                .map_err(|_| "解包失败：写入文件失败".to_string())?;
        }
        if file_expanded != declared_size {
            return Err("插件包条目实际解压尺寸与声明不一致".to_string());
        }
        // Unix up need can （zip external attributes 16 is mode）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = file.unix_mode().unwrap_or(0o644);
            std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))
                .map_err(|error| format!("设置文件权限失败（{}）：{error}", out_path.display()))?;
        }
    }
    Ok(())
}

/// directory ：， 1.4 seconds。
///
/// ── Why is 「」 not is 「 」──────────────────────────
/// `MoveFileEx` return `ERROR_ACCESS_DENIED`（os error 5） when ，Windows no has 
/// 「」 （`RestartManager` can ， need API and ， and for 
/// invalid）。 this use ** is when **：Defender 。
/// is is 。
const DIR_RETRY_DELAYS_MS: [u64; 3] = [80, 250, 1000];

/// directory， to 「 use 」。
pub(super) fn move_dir_retrying(from: &Path, to: &Path) -> std::io::Result<()> {
    retry_dir_op(|| std::fs::rename(from, to), from)
}

/// directory， to 「 use 」（ [`move_dir_retrying`]）。
pub(super) fn remove_dir_retrying(dir: &Path) -> std::io::Result<()> {
    retry_dir_op(|| std::fs::remove_dir_all(dir), dir)
}

/// directory 1 。
///
/// for 「 when 」error：Windows `PermissionDenied`（os error 5，
/// / ） and `ResourceBusy`。 its error（path not in 、、
/// ）return —— that 1 1 ， will 1 failure 
/// seconds etc 。
fn retry_dir_op<F>(mut op: F, subject: &Path) -> std::io::Result<()>
where
    F: FnMut() -> std::io::Result<()>,
{
    let mut last = match op() {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    if !is_transient_io_error(&last) {
        return Err(explain_dir_error(last, subject));
    }
    for delay in DIR_RETRY_DELAYS_MS {
        std::thread::sleep(std::time::Duration::from_millis(delay));
        match op() {
            Ok(()) => return Ok(()),
            Err(error) => {
                if !is_transient_io_error(&error) {
                    return Err(explain_dir_error(error, subject));
                }
                last = error;
            }
        }
    }
    Err(explain_dir_error(last, subject))
}

/// this IO error not （「 when use 」 this 1 ）。
fn is_transient_io_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ResourceBusy
    )
}

/// to failure directoryerror up can 。
///
/// `os error 5` for use no has —— not is 、 is 、 is 
/// configdirectory to up 。 this by Windows that Reason to 。
fn explain_dir_error(error: std::io::Error, subject: &Path) -> std::io::Error {
    if !is_transient_io_error(&error) {
        return error;
    }
    let hint = if cfg!(windows) {
        "（重试 3 次仍被拒绝：多半是杀毒软件的实时保护正锁着刚解压出来的 exe。\
         把配置目录加入杀软白名单，或关掉实时保护后重试）"
    } else {
        "（重试 3 次仍被拒绝：请检查该目录的写权限）"
    };
    std::io::Error::new(
        error.kind(),
        format!("{}{}：{subject:?}", error, hint),
    )
}

/// 1 ：directory + from 。
///
/// by use （api ） —— this 。Windows up 
/// use when not ， that returnerror（「disabled」）。
pub fn uninstall(id: &str) -> Result<(), String> {
    let mut registry = PackRegistry::load();
    let Some(index) = registry.packs.iter().position(|pack| pack.id == id) else {
        return Err(format!("没有安装这个插件包：{id}"));
    };
    let pack = registry.packs.remove(index);
    let dir = plugins_dir().join(&pack.id);
    if dir.exists() {
        // ： when will up （ its is no ）
        remove_dir_retrying(&dir).map_err(|e| {
            format!("删除包目录失败（{e}）：如果插件进程还在运行，请先停用插件再卸载")
        })?;
    }
    registry.save()?;
    logging::log("Plugin", &format!("插件包「{id}」已卸载"));
    Ok(())
}

/// enabled / disabled 。
pub fn set_pack_enabled(id: &str, enabled: bool) -> Result<(), String> {
    let mut registry = PackRegistry::load();
    let Some(pack) = registry.packs.iter_mut().find(|pack| pack.id == id) else {
        return Err(format!("没有安装这个插件包：{id}"));
    };
    pack.enabled = enabled;
    registry.save()?;
    Ok(())
}

/// enabled / disabled 1 provider。
pub fn set_provider_enabled(id: &str, provider: &str, enabled: bool) -> Result<(), String> {
    let mut registry = PackRegistry::load();
    let Some(pack) = registry.packs.iter_mut().find(|pack| pack.id == id) else {
        return Err(format!("没有安装这个插件包：{id}"));
    };
    if !pack
        .manifest_providers
        .iter()
        .any(|item| item.id.0 == provider)
    {
        return Err(format!("这个包里没有提供商 {provider}"));
    }
    pack.providers.insert(provider.to_string(), enabled);
    registry.save()?;
    Ok(())
}

/// zip sha256（）。
pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(protocol: u32) -> proto::PackManifest {
        proto::PackManifest {
            id: "test".into(),
            name: "Test".into(),
            version: "1.0.0".into(),
            protocol,
            providers: vec![proto::PackProvider {
                id: proto::ProviderId::new("demo"),
                label: "Demo".into(),
                order: 0,
                default_enabled: true,
            }],
            entry: BTreeMap::from([("x86_64-pc-windows-msvc".to_string(), "bin/x.exe".to_string())]),
            icons: None,
            ui: Vec::new(),
            homepage: None,
            signature: None,
        }
    }

    #[test]
    fn rejects_protocol_mismatch() {
        let err = validate_manifest(&manifest(999), 1).unwrap_err();
        assert!(err.contains("999"), "错误里要带上对方的版本号: {err}");
    }

    #[test]
    fn rejects_absolute_entry() {
        let mut m = manifest(1);
        m.entry.insert("t".into(), "/usr/bin/evil".into());
        assert!(validate_manifest(&m, 1).is_err());
    }

    #[test]
    fn rejects_parent_traversal() {
        let mut m = manifest(1);
        m.entry.insert("t".into(), "../../evil".into());
        assert!(validate_manifest(&m, 1).is_err());
    }

    #[test]
    fn accepts_matching_protocol() {
        assert!(validate_manifest(&manifest(1), 1).is_ok());
    }

    // ── ────────────────────────────────────────────────────
    //
    // this 1 is 「 」， not is 「 」：
    // · no has → Unsigned（，）；
    // · for not up → Unsigned（ 1 ）；
    // · not / not in / not → Invalid（）。
    //
    // use `ed25519_dalek::SigningKey` 1 for ， not is to ：
    // to in after will its failure， this need is 。

    /// 1 already when directory（ ）。
    fn staged_dir(tag: &str, binary: &[u8]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "a2a-sign-test-{tag}-{}",
            crate::server::logging::now_ms()
        ));
        let bin = dir.join("bin");
        std::fs::create_dir_all(&bin).expect("建临时目录");
        std::fs::write(bin.join("x.exe"), binary).expect("写入口二进制");
        dir
    }

    #[test]
    fn unsigned_pack_is_allowed() {
        let dir = staged_dir("unsigned", b"binary");
        let (state, reason) = verify_signature(&dir, &manifest(1), "x86_64-pc-windows-msvc");
        assert_eq!(state, SignatureState::Unsigned);
        assert!(reason.is_none(), "未签名不该带失败原因");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// and not 1 → （ not is invalid）。
    #[test]
    fn signature_for_other_target_is_unsigned() {
        let dir = staged_dir("othertarget", b"binary");
        let mut m = manifest(1);
        m.signature = Some(proto::PackSignature {
            algorithm: "ed25519".into(),
            key_id: "test".into(),
            value: "AAAA".into(),
            target: Some("aarch64-apple-darwin".into()),
        });
        let (state, _) = verify_signature(&dir, &m, "x86_64-pc-windows-msvc");
        assert_eq!(state, SignatureState::Unsigned);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_algorithm_is_invalid() {
        let dir = staged_dir("alg", b"binary");
        let mut m = manifest(1);
        m.signature = Some(proto::PackSignature {
            algorithm: "rsa-sha256".into(),
            key_id: "test".into(),
            value: "AAAA".into(),
            target: Some("x86_64-pc-windows-msvc".into()),
        });
        let (state, reason) = verify_signature(&dir, &m, "x86_64-pc-windows-msvc");
        assert_eq!(state, SignatureState::Invalid);
        assert!(reason.unwrap_or_default().contains("rsa-sha256"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_key_id_is_invalid() {
        let dir = staged_dir("keyid", b"binary");
        let mut m = manifest(1);
        m.signature = Some(proto::PackSignature {
            algorithm: "ed25519".into(),
            key_id: "not-in-table".into(),
            value: "AAAA".into(),
            target: Some("x86_64-pc-windows-msvc".into()),
        });
        let (state, reason) = verify_signature(&dir, &m, "x86_64-pc-windows-msvc");
        assert_eq!(state, SignatureState::Invalid);
        assert!(reason.unwrap_or_default().contains("not-in-table"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// must need ：， 1 not 。
    ///
    /// this is —— ，，
    /// `bin/` 。
    #[test]
    fn payload_covers_binary_and_target() {
        let m = manifest(1);
        let left = proto::signing_payload(&m, "x86_64-pc-windows-msvc", "aaaa", &[]);
        let right = proto::signing_payload(&m, "x86_64-pc-windows-msvc", "bbbb", &[]);
        assert_ne!(left, right, "二进制摘要变了，载荷必须变");
        let other_target = proto::signing_payload(&m, "aarch64-apple-darwin", "aaaa", &[]);
        assert_ne!(left, other_target, "平台变了，载荷必须变");
        assert!(left.starts_with("agent2api-pack-v1\n"));
        assert!(left.contains("binary=aaaa"));
    }

    /// 「 → 」： use 1 for must，
    /// 1 must not 。
    #[test]
    fn sign_and_verify_roundtrip() {
        use ed25519_dalek::{Signer, SigningKey};
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let public_hex: String = signing
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();

        let target = "x86_64-pc-windows-msvc";
        let dir = staged_dir("roundtrip", b"the-real-binary");
        let binary_hash = sha256_file(&dir.join("bin/x.exe")).expect("算摘要");
        let mut m = manifest(1);
        let payload = proto::signing_payload(&m, target, &binary_hash, &[]);
        let signature = signing.sign(payload.as_bytes());
        m.signature = Some(proto::PackSignature {
            algorithm: "ed25519".into(),
            key_id: "test".into(),
            value: base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()),
            target: Some(target.into()),
        });

        // is ， when not —— this 
        // verify_ed25519 （ that 1 by up Invalid use ）
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(m.signature.as_ref().map(|s| s.value.clone()).unwrap_or_default())
            .expect("解 base64");
        assert!(verify_ed25519(
            &hex_to_32(&public_hex).expect("公钥十六进制"),
            payload.as_bytes(),
            &decoded
        ));

        // 1 （ ）→ must not 
        let tampered = proto::signing_payload(&m, target, "0000", &[]);
        assert!(!verify_ed25519(
            &hex_to_32(&public_hex).expect("公钥十六进制"),
            tampered.as_bytes(),
            &decoded
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// ： or not for need return None（ not panic）。
    #[test]
    fn hex_parsing_rejects_bad_input() {
        assert!(hex_to_32("zz").is_none());
        assert!(hex_to_32(&"a".repeat(63)).is_none());
        assert!(hex_to_32(&"a".repeat(64)).is_some());
    }
}
