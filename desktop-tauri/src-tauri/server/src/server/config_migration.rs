//! 1 configdirectory：`~/.workbuddy-proxy` → `~/.agent2api`（， not ）。
//!
//! ── （ §3.1）────────────────────────────────────
//! - new directory not in 、 old directory in → old directory**** as new directory， old directory not 
//! （ use can when to old ， and when in ）；
//! - in → not （** new directory as **， not and 、 not ——
//! and will 「 as 」 not can ）。 already in new directory 1 
//! use / before 1 ，** not ** is failure、 not ；
//! - not in → not （ after will emptydirectory）；
//! - has （`AGENT2API_PROXY_HOME` / old ）→ not ： use 
//! directory， this when defaultpath will ， can can to error up 。
//!
//! ── Why is not is ───────────────────────────────────
//! （rename） in directory not when not can ， and will already old use 
//! old when not to 。configdirectory has config.json / accounts.json /
//! logs.jsonl this file， can ignore。
//!
//! ── directory： 1 + create， ─────────────────
//! new directory， 1 in failure（ / file use ） will down 1 
//! ** new directory**； down to 「 new directory already in 」「 new directory as 」，
//! not will —— use will its 1 ， and none from 。
//! to directory，success after `rename` directory（，）：
//! new directory need is ， need not in （ down ）。
//!
//! directory and when （`{directory}.migrating-{pid}-{nanos}`），
//! use `create_dir` create、 and allfile， is ：
//! - failure when **create** that 1 path， not for unknown path
//! `remove_dir_all`；
//! - old directory， has in 「 before ** and ** file in 」 when 
//! ， its 1 not —— can can ， not use 。
//!
//! ── / not ───────────────────────────────────
//! when to 、Windows （junction / mount point） 1 and 
//! ： will to configdirectory（ ）， can can none，
//! can can to 。 old directory if is /，。
//!
//! ── failure will （ and can ）──────────────────────────
//! return `Result`： use （ `lib.rs` setup 1 ）failure when ** not config
//! file**、 use after 。 this old directory、 new directory not will ， down 1 
//! will 。 if failure after ， after 1 （set / config.json /
//! ） will new directory， from not will ， use to is 「」。
//!
//! errorpath and error， not file（config has token， not / for ）。

use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

/// directory after （directory + after + 1 ，module header）
const STAGING_SUFFIX: &str = ".migrating";

/// directory allfile： has directory as is create 。
/// ， and use file after 。
const STAGING_MARKER: &str = ".agent2api.migrating.marker";

/// （success when ； use need not need use ）
#[derive(Debug)]
pub enum MigrationOutcome {
    /// already （directory）
    Migrated { from: PathBuf, to: PathBuf },
    /// none need （ new directory already in / old directory not in ）
    NothingToDo,
    /// use use configdirectory， not use 
    SkippedEnvOverride,
}

/// configdirectory。failurereturn can in （ use use and ）。
///
/// etc ：success after new directory in ， after will in 1 up return。
///
/// success not in this ： use can by use （ setup） 1 ，
/// that can to `Migrated` /path。
pub fn migrate_config_dir() -> Result<MigrationOutcome, String> {
    // has （ new or old ） when not ： use directory
    if env_dir_override_set() {
        return Ok(MigrationOutcome::SkippedEnvOverride);
    }

    let target = paths::config_dir();
    // new directory in （ is ）→ as ： not and 、 not 、 not 
    if std::fs::symlink_metadata(&target).is_ok() {
        return Ok(MigrationOutcome::NothingToDo);
    }

    let legacy = crate::server::config::legacy_config_dir();
    let Ok(legacy_metadata) = std::fs::symlink_metadata(&legacy) else {
        return Ok(MigrationOutcome::NothingToDo);
    };
    if is_link_like(&legacy_metadata) {
        return Err(format!(
            "旧配置目录 {} 是链接或 Windows 重解析点，出于安全考虑不跟随迁移。\
             请把它改成真实目录后重启，或手动把数据拷到 {}。",
            legacy.display(),
            target.display()
        ));
    }
    if !legacy_metadata.is_dir() {
        return Err(format!(
            "旧配置路径 {} 不是目录（可能被同名文件占住），无法迁移。\
             请先处理该路径后重启；正式配置目录应为 {}。",
            legacy.display(),
            target.display()
        ));
    }

    let Some(parent) = target.parent() else {
        return Err(format!(
            "配置目录 {} 没有父目录，无法迁移（旧数据仍在 {}）。",
            target.display(),
            legacy.display()
        ));
    };
    let Some(target_name) = target.file_name().and_then(|name| name.to_str()) else {
        return Err(format!(
            "配置目录名 {} 不是合法 UTF-8，无法迁移（旧数据仍在 {}）。",
            target.display(),
            legacy.display()
        ));
    };

    // up in can can down directory： can that 
    sweep_stale_staging(parent, target_name);

    let staging = parent.join(staging_name(target_name));
    // create_dir（ not is create_dir_all）：directorymust by use createsuccess，
    // otherwise/ ， can not can 
    if let Err(error) = std::fs::create_dir(&staging) {
        return Err(format!(
            "创建迁移暂存目录 {} 失败：{error}。旧数据仍完整保留在 {}；\
             修复后重启本程序会自动重试。",
            staging.display(),
            legacy.display()
        ));
    }

    match publish(&legacy, &staging, &target) {
        Ok(Publish::Published) => Ok(MigrationOutcome::Migrated { from: legacy, to: target }),
        // directory ： already in directory as ， not ，
        // not failure（ old directory）。directory can 。
        Ok(Publish::TargetAppeared) => {
            remove_own_staging(&staging);
            Ok(MigrationOutcome::NothingToDo)
        }
        Err(error) => {
            remove_own_staging(&staging);
            Err(error)
        }
    }
}

/// directory is ****（， `ServerState::bootstrap` use ）。
///
/// and `migrate_config_dir` 1 ：none、
/// old directory in 、 new directory not in 。 when 「 this no has 」——
/// when if ， after after 1 （ / config.json / ）
/// will new directory， from none， use to is 「」。
/// return `Some(Reason)` when use 。
pub fn pending_reason() -> Option<String> {
    if env_dir_override_set() {
        return None;
    }
    let target = paths::config_dir();
    if std::fs::symlink_metadata(&target).is_ok() {
        return None;
    }
    let legacy = crate::server::config::legacy_config_dir();
    if std::fs::symlink_metadata(&legacy).is_err() {
        return None;
    }
    Some(format!(
        "检测到旧配置目录 {} 尚未迁移到 {}，但本次启动没有执行目录迁移。\
         为避免以空配置继续运行（那会让迁移再也无法重试、旧数据看似丢失），\
         本次网关初始化已停止。请重启本程序重试；若反复出现请反馈该问题。",
        legacy.display(),
        target.display()
    ))
}

/// ：， or 「directory 」。
enum Publish {
    /// directory already as directory
    Published,
    /// directory in ： as ， not 
    TargetAppeared,
}

/// to directory → → directory。
///
/// 1 failurereturn in ； use create directory。
fn publish(legacy: &Path, staging: &Path, target: &Path) -> Result<Publish, String> {
    // file： is 「 this directory by create」 ，
    // after （ down ）
    let marker = staging.join(STAGING_MARKER);
    if let Err(error) = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker)
    {
        return Err(format!(
            "在暂存目录 {} 写入标记文件失败：{error}。旧数据仍完整保留在 {}；\
             修复后重启本程序会自动重试。",
            staging.display(),
            legacy.display()
        ));
    }

    if let Err(error) = copy_dir_all(legacy, staging, &marker) {
        return Err(format!(
            "拷贝 {} → {} 失败：{error}。旧数据仍完整保留在 {}；\
             修复后（例如关闭占用文件的程序、释放磁盘空间）重启本程序会自动重试，\
             本次未改动或删除旧目录。",
            legacy.display(),
            target.display(),
            legacy.display()
        ));
    }

    // file not can directory（otherwise new configdirectory will 1 none file）
    if let Err(error) = std::fs::remove_file(&marker) {
        return Err(format!(
            "清理暂存目录标记文件 {} 失败：{error}。旧数据仍完整保留在 {}；\
             修复后重启本程序会自动重试。",
            marker.display(),
            legacy.display()
        ));
    }

    // rename is ： need new directory， need no has （ down ）。
    // if directory （ up ， this 1 ），
    // failure not is —— already has directory 1 as ， not use 。
    if std::fs::symlink_metadata(target).is_ok() {
        crate::server::logging::console_line(
            "[Config]",
            &format!(
                "⚠️  拷贝期间配置目录 {} 已出现，本次不改名覆盖（以已存在的目录为准；旧数据仍在 {}）",
                target.display(),
                legacy.display()
            ),
        );
        return Ok(Publish::TargetAppeared);
    }
    std::fs::rename(staging, target)
        .map(|()| Publish::Published)
        .map_err(|error| {
            format!(
                "把暂存目录 {} 改名为 {} 失败：{error}。旧数据仍完整保留在 {}；\
                 修复后重启本程序会自动重试。",
                staging.display(),
                target.display(),
                legacy.display()
            )
        })
}

/// deletecreate directory；failure（ not and down ）。
fn remove_own_staging(staging: &Path) {
    if let Err(error) = std::fs::remove_dir_all(staging) {
        crate::server::logging::console_line(
            "[Config]",
            &format!(
                "⚠️  清理本次迁移的暂存目录 {} 失败（不影响旧数据，下次启动会重试迁移）: {error}",
                staging.display()
            ),
        );
    }
}

/// 1 directory：`{directory}.migrating-{pid}-{nanos}`。
///
/// up and seconds when ，「 this directory 1 is create 」，
/// not will and down 1 （ will create_dir failure， not ）。
fn staging_name(target_name: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or(0);
    format!("{target_name}{STAGING_SUFFIX}-{}-{nanos}", std::process::id())
}

/// ** can ** directory（ up in / down ）。
///
/// 1 directory down 、 before 、 and allfile directory；
/// not path、 not —— use directory not can 。
/// failure： not （ use is 1 new ）。
fn sweep_stale_staging(parent: &Path, target_name: &str) {
    let prefix = format!("{target_name}{STAGING_SUFFIX}");
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !name.starts_with(&prefix) {
            continue;
        }
        let path = entry.path();
        let owned = std::fs::symlink_metadata(&path)
            .map(|metadata| metadata.is_dir() && !is_link_like(&metadata))
            .unwrap_or(false)
            && std::fs::symlink_metadata(path.join(STAGING_MARKER))
                .map(|metadata| metadata.is_file() && !is_link_like(&metadata))
                .unwrap_or(false);
        if !owned {
            crate::server::logging::console_line(
                "[Config]",
                &format!(
                    "⚠️  临时路径 {} 已存在且不是本程序创建的目录，未处理（不影响本次迁移）",
                    path.display()
                ),
            );
            continue;
        }
        match std::fs::remove_dir_all(&path) {
            Ok(()) => crate::server::logging::console_line(
                "[Config]",
                &format!("已清理上次迁移残留的临时目录: {}", path.display()),
            ),
            Err(error) => crate::server::logging::console_line(
                "[Config]",
                &format!(
                    "清理上次迁移残留的临时目录失败（不影响本次迁移）: {} - {error}",
                    path.display()
                ),
            ),
        }
    }
}

/// directory（`std::fs` no has ）。
///
/// file and directory： / Windows （junction、mount point）
/// 1 ** and **， not —— configdirectory not this has ， has when 
/// will to configdirectory（ ）， can can none， can can to 。
/// its （、 etc ）。
///
/// `skip` is directory allfilepath：，directory
/// not this /（ that will not directory）。
fn copy_dir_all(from: &Path, to: &Path, skip: &Path) -> io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let destination = to.join(entry.file_name());
        if destination == skip {
            continue;
        }
        if is_link_like(&entry.metadata()?) {
            crate::server::logging::console_line(
                "[Config]",
                &format!("⚠️  迁移跳过链接/重解析点（不跟随）: {}", source.display()),
            );
            continue;
        }
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            std::fs::create_dir(&destination)?;
            copy_dir_all(&source, &destination, skip)?;
        } else if file_type.is_file() {
            std::fs::copy(&source, &destination)?;
        } else {
            crate::server::logging::console_line(
                "[Config]",
                &format!("⚠️  迁移跳过非普通文件: {}", source.display()),
            );
        }
    }
    Ok(())
}

/// is is directory：， or Windows （junction / mount point）。
///
/// `file_type().is_symlink()` ；Windows junction in 
/// `file_attributes` up `FILE_ATTRIBUTE_REPARSE_POINT` ， need ，
/// otherwise「 junction 」 in 。
fn is_link_like(metadata: &std::fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;

        /// `FILE_ATTRIBUTE_REPARSE_POINT`。 not windows-sys ： that 
        /// in Windows down in ， can file。
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

/// is setconfigdirectory （`AGENT2API_PROXY_HOME` or old ）。
///
/// 「 has no has 」， not path —— 1 is 
/// `paths::config_dir()`； when not use 。
fn env_dir_override_set() -> bool {
    ["AGENT2API_PROXY_HOME", "WORKBUDDY_PROXY_HOME"]
        .iter()
        .any(|name| {
            std::env::var(name)
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        })
}
