//! update： can file and `ui/` directory， after 。
//!
//! ── Why is 「file + 」， not is Docker API ──────────
//! `docker.sock` etc root to 1 to up ； this 
//! （ sub2api ） not config： can file by ，
//! **** —— Docker `restart: unless-stopped` will 
//! ** 1 **（ can ）， new 。 root ，
//! can file（`/usr/local/bin`） and directory（`/app/ui`） can 。
//!
//! ── ： and will ──────────────────────
//! update after ，`docker inspect` / compose is old tag， is 
//! up new ； 1 `docker compose pull && up -d`（）， to 
//! 。 this is 「 not Docker can update」 ，，
//! up update after will 。
//!
//! ── ： ────────────────────────────────────────
//! `AGENT2API_SELF_UPDATE=1`（Dockerfile default），onlyenabled。
//! systemd （systemd ），。
//! before not “”， after none。
//!
//! ── ──────────────────────────────────────────────────
//! not can ：① down `assert_downloadable` （ in down 
//! that 1 ）；② **must** `.sha256`（ when by CI ， not to 
//! ， not as 「none」）；③ 
//! （`agent2api-server` and `ui/` before ），path 1 、total。
//!
//! ── （CI by ， need 1 ）──────────────────
//! `agent2api-server-linux-<arch>.tar.gz`， down ：
//! · `agent2api-server` can file
//!   · `ui/**` directory
//! up `.github/workflows/build.yml` `linux-server` job。

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::server::logging;

use super::client;
use super::version::{UpdateError, MAX_INSTALLER_BYTES};
use super::USER_AGENT;

/// can file（ and CI 1 ）
const BINARY_ENTRY: &str = "agent2api-server";
/// directory before （： `ui/` down ）
const UI_PREFIX: &str = "ui/";
/// totallimit： already has 600MB down limit， this 1 ，
/// （gzip can KB to GB）
const MAX_EXTRACTED_BYTES: u64 = 1024 * 1024 * 1024;
/// tar （`L` / `x`） limit： has to 
const MAX_META_BYTES: u64 = 1024 * 1024;
/// filerequest when （ ）
const CHECKSUM_TIMEOUT_MS: u64 = 30_000;

/// ：`AGENT2API_SELF_UPDATE=1`（ etc 1， and ALLOW_NO_KEY ）。
/// `0` or empty，Dockerfile default `1`。
pub fn enabled() -> bool {
    std::env::var("AGENT2API_SELF_UPDATE")
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

/// is availableupdate：Linux + + has for 。
pub fn supported() -> bool {
    cfg!(target_os = "linux") && !super::desktop_shell() && enabled() && bundle_name().is_some()
        && crate::server::core::system_info::in_container()
}

/// for update（ Linux / unknown as None）。
pub fn bundle_name() -> Option<String> {
    super::version::server_bundle_name()
}

/// current can file path（；`/proc/self/exe` to is ）。
pub fn current_exe_path() -> Result<PathBuf, UpdateError> {
    let exe = std::env::current_exe()
        .map_err(|error| UpdateError::new(format!("nonecurrent can file：{error}")))?;
    Ok(std::fs::canonicalize(&exe).unwrap_or(exe))
}

/// failure use when path：directory、 `.backup` after ，success。
fn backup_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "agent2api".to_string());
    target.with_file_name(format!("{name}.backup"))
}

// ─── and ─────────────────────────────────────────────────

/// `.sha256` file 。
///
/// by `sha256sum` （`<64 >␣␣<file>`）， has 
/// 。 not for —— this 1 is 「 up is 」 
/// 1 ， etc no 。
pub async fn fetch_checksum(url: &str) -> Result<String, UpdateError> {
    let headers = vec![
        ("User-Agent".to_string(), USER_AGENT.to_string()),
        ("Accept".to_string(), "text/plain".to_string()),
    ];
    super::assert_downloadable(url)?;
    let response = client::fetch_official(url, &headers, Some(CHECKSUM_TIMEOUT_MS)).await?;
    if !response.status().is_success() {
        return Err(UpdateError::new(format!(
            " down filefailure：GitHub return HTTP {}",
            response.status().as_u16()
        )));
    }
    let text = response
        .text()
        .await
        .map_err(|error| UpdateError::new(format!("filefailure：{error}")))?;
    parse_checksum(&text)
}

/// from file（ + 64 ）。
fn parse_checksum(text: &str) -> Result<String, UpdateError> {
    let token = text.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    if token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(token)
    } else {
        Err(UpdateError::new("file（ not to sha256 ）， already "))
    }
}

/// file sha256（，， not ）。
pub fn file_sha256(path: &Path) -> Result<String, UpdateError> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ─── ───────────────────────────────────────────────────

/// （ is to 、file ）。
pub struct Extracted {
    pub binary: bool,
    pub ui_files: usize,
}

/// to directory： can file → `bin_stage/agent2api-server`，
/// `ui/**` → `ui_stage/**`（ before ）。
///
/// tar （ `providers/qoder/risk.rs` `tar_entry`， that 
/// 、 this need ）： GNU longname（`L`） and pax （`x`） 
/// path —— Rust tar when default pax ， will path。
///
/// ， its ****（tar is ， not 
/// will ）。path（`..`、 for path、） 1 。
pub fn extract_bundle(
    archive: &Path,
    bin_stage: &Path,
    ui_stage: Option<&Path>,
) -> Result<Extracted, UpdateError> {
    let file = std::fs::File::open(archive)
        .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
    let mut tar = flate2::read::GzDecoder::new(file);

    let mut header = [0u8; 512];
    let mut long_name: Option<String> = None;
    let mut total: u64 = 0;
    let mut seen = std::collections::HashSet::new();
    let mut entries = 0usize;
    let mut out = Extracted { binary: false, ui_files: 0 };

    loop {
        if !read_header(&mut tar, &mut header)? {
            break;
        }
        if header.iter().all(|byte| *byte == 0) {
            break; // 
        }
        entries += 1;
        if entries > 100_000 {
            return Err(UpdateError::new("update， already in "));
        }
        let recorded = octal(&header[148..156]);
        let calculated: u64 = header.iter().enumerate()
            .map(|(index, byte)| if (148..156).contains(&index) { 32 } else { u64::from(*byte) }).sum();
        if recorded != calculated {
            return Err(UpdateError::new("update already （tar failure）"));
        }
        let size = octal(&header[124..136]);
        total = total.saturating_add(size);
        if total > MAX_EXTRACTED_BYTES {
            return Err(UpdateError::new("update after ， already in "));
        }
        let entry_type = header[156] as char;
        let prefix = cstr(&header[345..500]);
        let mut name = long_name.take().unwrap_or_else(|| {
            if prefix.is_empty() {
                cstr(&header[..100])
            } else {
                format!("{prefix}/{}", cstr(&header[..100]))
            }
        });

        // ：GNU longname to ；pax `path=` 
        match entry_type {
            'L' => {
                let body = read_meta(&mut tar, size)?;
                name = String::from_utf8_lossy(&body).trim_end_matches('\0').to_string();
                read_padding(&mut tar, size)?;
                long_name = Some(name);
                continue;
            }
            'x' => {
                let body = read_meta(&mut tar, size)?;
                let text = String::from_utf8_lossy(&body).to_string();
                if let Some(value) = text.lines().filter_map(|line| line.split_once(' '))
                    .find_map(|(_, record)| record.strip_prefix("path="))
                {
                    long_name = Some(value.to_string());
                }
                read_padding(&mut tar, size)?;
                continue;
            }
            _ => {}
        }

        let relative = safe_relative(&name)
            .ok_or_else(|| UpdateError::new("update not path"))?;
        if !seen.insert(relative.clone()) {
            return Err(UpdateError::new("update"));
        }
        if !matches!(entry_type, '0' | '\0' | '5') {
            return Err(UpdateError::new("updateunsupported or file"));
        }
        let target: Option<PathBuf> = match (Some(relative), entry_type) {
            (Some(relative), _) if relative == Path::new(BINARY_ENTRY) => {
                Some(bin_stage.join(BINARY_ENTRY))
            }
            // `ui/` down ： before to directory（`Path` 
            // `starts_with` by ，`"ui/"` and `"ui"` etc ）
            (Some(relative), _) if relative.starts_with(UI_PREFIX) => ui_stage.map(|stage| {
                let tail = relative.strip_prefix(UI_PREFIX).unwrap_or(&relative);
                stage.join(tail)
            }),
            _ => None,
        };

        match (target, entry_type) {
            // directory： can （ as empty）
            (Some(target), '5') => {
                let _ = std::fs::create_dir_all(&target);
                read_padding(&mut tar, size)?;
            }
            // file：； not in （tar is ）
            (target, '0' | '\0') => {
                match target {
                    Some(target) => {
                        write_entry(&mut tar, &target, size)?;
                        if target.starts_with(bin_stage) {
                            out.binary = true;
                        } else {
                            out.ui_files += 1;
                        }
                    }
                    None => {
                        skip_exact(&mut tar, size)?;
                    }
                }
                read_padding(&mut tar, size)?;
            }
            // its （、file etc ）：， not 
            _ => {
                skip_exact(&mut tar, size)?;
                read_padding(&mut tar, size)?;
            }
        }
    }
    Ok(out)
}

/// 1 ： `./` before ， for path / `..` / / empty。
/// return can `join` to directory for path。
fn safe_relative(name: &str) -> Option<PathBuf> {
    let mut trimmed = name;
    while let Some(rest) = trimmed.strip_prefix("./") { trimmed = rest; }
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.contains(['\\', '\0']) {
        return None;
    }
    let mut out = PathBuf::new();
    for part in trimmed.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return None;
        }
        out.push(part);
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

/// 1 512 ：（）return false。
fn read_header(reader: &mut impl Read, buffer: &mut [u8; 512]) -> Result<bool, UpdateError> {
    let mut filled = 0usize;
    while filled < buffer.len() {
        let read = reader
            .read(&mut buffer[filled..])
            .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
        if read == 0 {
            if filled == 0 {
                return Ok(false);
            }
            return Err(UpdateError::new("update already （tar not ）"));
        }
        filled += read;
    }
    Ok(true)
}

/// 1 （ has limit，）。
fn read_meta(reader: &mut impl Read, size: u64) -> Result<Vec<u8>, UpdateError> {
    if size > MAX_META_BYTES {
        return Err(UpdateError::new("update already （）"));
    }
    let mut buffer = vec![0u8; size as usize];
    reader
        .read_exact(&mut buffer)
        .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
    Ok(buffer)
}

/// 1 （limit and file）。
fn write_entry(reader: &mut impl Read, target: &Path, size: u64) -> Result<(), UpdateError> {
    if size > MAX_INSTALLER_BYTES {
        return Err(UpdateError::new("update ， already in "));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| UpdateError::new(format!("createdirectoryfailure（{}）：{error}", parent.display())))?;
    }
    let mut file = std::fs::File::create(target)
        .map_err(|error| UpdateError::new(format!("filefailure（{}）：{error}", target.display())))?;
    let mut remaining = size;
    let mut buffer = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let chunk = buffer.len().min(remaining as usize);
        reader
            .read_exact(&mut buffer[..chunk])
            .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
        file.write_all(&buffer[..chunk])
            .map_err(|error| UpdateError::new(format!("filefailure（{}）：{error}", target.display())))?;
        remaining -= chunk as u64;
    }
    file.flush()
        .map_err(|error| UpdateError::new(format!("filefailure（{}）：{error}", target.display())))?;
    set_mode(target, 0o644);
    Ok(())
}

/// size （ not in ）。
fn skip_exact(reader: &mut impl Read, size: u64) -> Result<(), UpdateError> {
    let mut remaining = size;
    let mut buffer = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let chunk = buffer.len().min(remaining as usize);
        reader
            .read_exact(&mut buffer[..chunk])
            .map_err(|error| UpdateError::new(format!("updatefailure：{error}")))?;
        remaining -= chunk as u64;
    }
    Ok(())
}

/// after for （tar by 512 for ）。
fn read_padding(reader: &mut impl Read, size: u64) -> Result<(), UpdateError> {
    let padding = (512 - (size % 512)) % 512;
    if padding > 0 {
        skip_exact(reader, padding)?;
    }
    Ok(())
}

/// （tar is ASCII ， can can empty / NUL）。
fn octal(bytes: &[u8]) -> u64 {
    let text = cstr(bytes);
    u64::from_str_radix(text.trim(), 8).unwrap_or(0)
}

/// tar C 
fn cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

/// setfile（ for unix has ； its is empty）。
fn set_mode(path: &Path, mode: u32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
}

// ─── file ───────────────────────────────────────────────

/// deletefile or directory（ not in 、failure not —— use is 「」）。
fn remove_path(path: &Path) {
    if path.is_dir() {
        let _ = std::fs::remove_dir_all(path);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

/// directory（file）：`target` → `backup`，`staged` → `target`。
///
/// filemust and **directory**（ 1 file），`rename` is ；
/// failure when ， not 。
///
/// file rename is ：overlayfs up down file will copy-up to can ，
/// （`/usr/local/bin/agent2api-server`） can 。**directory not can this ** —— 
/// [`replace_dir`]。
fn replace_file(target: &Path, staged: &Path, backup: &Path) -> Result<(), UpdateError> {
    remove_path(backup);
    std::fs::rename(target, backup).map_err(|error| {
        UpdateError::new(format!("filefailure（{}）：{error}", target.display()))
    })?;
    if let Err(error) = std::fs::rename(staged, target) {
        let _ = std::fs::rename(backup, target);
        return Err(UpdateError::new(format!("failure（ already ）：{error}")));
    }
    Ok(())
}

/// directory： directory****，、 directory。
///
/// ── Why not is `rename(directory → )` ─────────────────────
/// directory in ****（ down ） when ，overlayfs rename will 
/// `EXDEV: Invalid cross-device link` —— Docker Desktop up `/app/ui`
/// is this directory（ 1 down 、 to directory this 1 ）。
/// directory 1 「 → delete → 」：
/// · delete whiteout， is overlayfs supports path；
/// · in can （directory and is create ）。
fn replace_dir(target: &Path, staged: &Path, backup: &Path) -> Result<(), UpdateError> {
    remove_path(backup);
    copy_dir_all(target, backup).map_err(|error| {
        UpdateError::new(format!("directoryfailure（{}）：{error}", target.display()))
    })?;
    remove_path(target);
    if let Err(error) = std::fs::rename(staged, target) {
        // ： in can ，
        let _ = std::fs::rename(backup, target);
        return Err(UpdateError::new(format!("directoryfailure（ already ）：{error}")));
    }
    Ok(())
}

/// directory（std no has ）。file and directory ——
/// `ui/` no has ，CI is 。
fn copy_dir_all(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = to.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            std::fs::copy(entry.path(), &destination)?;
        }
    }
    Ok(())
}

/// （directory has no has 1 ）。
pub struct ApplyOutcome {
    pub ui_updated: bool,
}

/// use update： → → `ui/` directory。
///
/// has ：（ this 1 failure then no ）， ——
/// failure will ，「 need ， need not 」。
/// `ui_dir` as None（ API no has directory） when 。
pub fn apply(
    archive: &Path,
    exe: &Path,
    ui_dir: Option<&Path>,
) -> Result<ApplyOutcome, UpdateError> {
    let parent = exe
        .parent()
        .ok_or_else(|| UpdateError::new("none can file in directory"))?;
    let id = format!("{}-{}", std::process::id(), logging::now_ms());
    let bin_stage = parent.join(format!(".a2a-update-{id}"));
    // directorymust and directory（rename need file）
    let ui_stage = match ui_dir {
        Some(dir) if dir.is_dir() => match dir.parent() {
            Some(parent) => Some(parent.join(format!(".a2a-ui-{id}"))),
            None => None,
        },
        _ => None,
    };

    // directory：file / not in this ，
    // not is to 1 not 
    std::fs::create_dir_all(&bin_stage).map_err(|error| {
        UpdateError::new(format!(
            "none in {} down createdirectory（file can can ）：{error}",
            parent.display()
        ))
    })?;
    if let Some(stage) = &ui_stage {
        if let Err(error) = std::fs::create_dir_all(stage) {
            let _ = std::fs::remove_dir_all(&bin_stage);
            return Err(UpdateError::new(format!("nonecreatedirectory：{error}")));
        }
    }

    let outcome = (|| {
        let extracted = extract_bundle(archive, &bin_stage, ui_stage.as_deref())?;
        if !extracted.binary {
            return Err(UpdateError::new("update no has agent2api-server， already "));
        }
        let staged_bin = bin_stage.join(BINARY_ENTRY);
        set_mode(&staged_bin, 0o755);

        replace_file(exe, &staged_bin, &backup_path(exe))?;

        let mut ui_updated = false;
        if let (Some(dir), Some(stage)) = (ui_dir, ui_stage.as_deref()) {
            if dir.is_dir() {
                if let Err(error) = replace_dir(dir, stage, &backup_path(dir)) {
                    // failure：， not 「 new + old 」 
                    if let Err(restore_error) = std::fs::rename(backup_path(exe), exe) {
                        return Err(UpdateError::new(format!(
                            "{}；failure：{restore_error}", error.message
                        )));
                    }
                    return Err(error);
                }
                ui_updated = true;
            }
        }
        remove_path(&backup_path(exe));
        if ui_updated {
            if let Some(dir) = ui_dir {
                remove_path(&backup_path(dir));
            }
        }
        Ok(ApplyOutcome { ui_updated })
    })();

    let _ = std::fs::remove_dir_all(&bin_stage);
    if let Some(stage) = &ui_stage {
        let _ = std::fs::remove_dir_all(stage);
    }
    outcome
}

// ─── ───────────────────────────────────────────────────

/// ： by headless （ not —— that ）。
static RESTART_HOOK: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

/// （ignore；「」 ）。
pub fn set_restart_hook(hook: Box<dyn Fn() + Send + Sync>) {
    let _ = RESTART_HOOK.set(hook);
}

/// request： 1 will ， successresponse to 。
///
/// return false = no has （ not is headless ）—— use 
/// request， not is not 。
pub fn request_restart() -> bool {
    let Some(hook) = RESTART_HOOK.get() else {
        return false;
    };
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(400));
        hook();
    });
    true
}
