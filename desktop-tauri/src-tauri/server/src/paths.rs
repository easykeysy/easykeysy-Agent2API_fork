//! 配置目录与相关路径的**唯一事实来源**。
//!
//! 拆分前这段实现住在桌面 crate 的 `gateway.rs`（`crate::gateway::config_dir`
//! 是唯一事实来源，`server::config` 只做转发）。网关拆成独立 crate 后事实
//! 来源随迁到这里：`paths::config_dir` 是唯一实现，桌面侧 `gateway::config_dir`
//! 反过来转发本模块 —— 「两条路径、一个实现」的约定不变，只是方向反了。
//!
//! ── 环境变量 ────────────────────────────────────────────────
//! `AGENT2API_PROXY_HOME` 优先，旧名 `WORKBUDDY_PROXY_HOME` 兼容读（1.x 的
//! 启动脚本/快捷方式里可能还留着旧名）。Docker 部署把它指到挂载卷
//! （compose 示例：`AGENT2API_PROXY_HOME=/data`），数据即可留在容器外。

use std::path::PathBuf;

/// 配置目录：默认 `{用户主目录}/.agent2api`，可被环境变量整体覆盖。
pub fn config_dir() -> PathBuf {
    #[cfg(test)]
    return crate::test_support::config_dir();

    #[cfg(not(test))]
    env_path("AGENT2API_PROXY_HOME")
        .or_else(|| env_path("WORKBUDDY_PROXY_HOME")) // 旧名兼容读
        .unwrap_or_else(default_config_dir)
}

/// 读环境变量里的目录覆盖：未设置 / 全空白一律当未设置
fn env_path(name: &str) -> Option<PathBuf> {
    let custom = std::env::var(name).ok()?;
    let trimmed = custom.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// Default config dir: `{user home}/.agent2api` (upstream behavior; subdirs are
/// auto-created at startup). Primary configuration (the db, `account.key`) must
/// survive reboots, so it never goes on the ramdisk (LAW-9.R4) — only the
/// disposable side dirs do (see `side_dir`).
#[cfg(not(test))]
fn default_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    ensure_layout(PathBuf::from(home).join(".agent2api"))
}

/// Local ramdisk root: only `/mnt/ramdisk` is accepted (used only if it exists and is writable).
/// The `AGENT2API_RAMDISK` env var overrides it (point it at another ramdisk / fast disk).
pub fn ramdisk_root() -> Option<PathBuf> {
    let candidates: [PathBuf; 2] = [
        env_path("AGENT2API_RAMDISK").unwrap_or_else(|| "/mnt/ramdisk".into()),
        "/mnt/ramdisk".into(),
    ];
    for candidate in candidates {
        if candidate.is_dir() && std::fs::write(candidate.join(".a2a-probe"), b"ok").is_ok() {
            return Some(candidate);
        }
    }
    None
}

/// Creates the missing subdirectory layout (the ramdisk is wiped on every reboot; existing dirs are left as-is).
fn ensure_layout(root: PathBuf) -> PathBuf {
    let _ = std::fs::create_dir_all(&root);
    for sub in ["Cache_Data", "cache", "db", "logs", "script-plugins", "debug"] {
        let _ = std::fs::create_dir_all(root.join(sub));
    }
    root
}

/// 旧桌面设置文件的路径（`{config_dir}/desktop-settings.json`）。
///
/// 拆分前由桌面 crate 的 `settings::legacy_file_path()` 提供；本 crate 的
/// 一次性迁移（`db::migrate::settings`）要定位这个文件，而 headless 下
/// 桌面 crate 根本不参与 —— 所以路径事实来源随 `config_dir` 一起迁到本
/// 模块，桌面侧的 `settings::legacy_file_path()` 反过来转发这里。
///
/// 文件名是历史事实：这个文件已不再是设置的真相来源（进了 SQLite 的
/// `kv` 表），名字只被迁移与回落读路径引用，不会随代码演进变化。
pub fn legacy_desktop_settings_file() -> PathBuf {
    config_dir().join("desktop-settings.json")
}

/// Logs dir: defaults to `{config_dir}/logs`.
///
/// On the Linux side (WSL / headless deploys), with **no env override**, this defaults to
/// `/mnt/ramdisk/Agent2API/logs`: logs are high-frequency, disposable, growing data —
/// once split from the main DB they belong on the ramdisk — this path used to depend on a hand-made symlink,
/// now it's the code default. `AGENT2API_PROXY_HOME` (or its old name) being set means the user's entire
/// data directory has moved; the side dirs must follow, without opening a separate ramdisk path.
/// When `/mnt/ramdisk` doesn't exist (plain non-WSL Linux, containers), fall back to `{config_dir}/logs`,
/// never fabricating a mount tree that isn't there.
pub fn logs_dir() -> PathBuf {
    side_dir("logs")
}

/// Cache dir: defaults to `{config_dir}/cache`; the Linux-side rule matches [`logs_dir`] exactly
/// (`/mnt/ramdisk/Agent2API/cache`). Cache is **disposable** data: deleting it never affects correctness —
/// which is exactly why it belongs on the ramdisk.
pub fn cache_dir() -> PathBuf {
    side_dir("cache")
}

/// Temp dir: defaults to `{config_dir}/tmp`; the Linux-side rule matches [`logs_dir`]
/// (`/mnt/ramdisk/Agent2API/tmp`). One-shot data like login windows lands here —
/// it's "use once, throw away" by nature, a perfect fit for the ramdisk.
pub fn tmp_dir() -> PathBuf {
    side_dir("tmp")
}

/// Shared implementation for the three side dirs above: `(env override / non-Linux / no ramdisk) ->
/// `{config_dir}/{name}`, otherwise -> `/mnt/ramdisk/Agent2API/{name}`.
///
/// Only `/mnt/ramdisk` itself is accepted (no other drives tried, no `/mnt/ramdisk1` peeking): it's the actual
/// mount point of this deployment; extra candidates would only add "why did it take that path" confusion when debugging.
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

/// Whether the side dirs are truly externalized (`/mnt/ramdisk` has taken them over).
///
/// The logging module uses this to decide whether event lines go to a file or the DB: the file backend is used only when externalized — Linux /
/// containers / test environments without a ramdisk keep the original DB backend, behavior identical to before the rework.
#[allow(dead_code)]
pub fn side_dirs_external() -> bool {
    logs_dir() != config_dir().join("logs")
}

/// Creates the three side dirs at startup.
///
/// The ramdisk is wiped on every reboot, and each writer's own `create_dir_all` only covers the one
/// it writes; creating them all once at startup means the paths shown on the settings page exist from the start. Failure is not fatal
/// (when a dir isn't writable, the writers degrade on their own anyway).
#[allow(dead_code)]
pub fn ensure_side_dirs() {
    for dir in [logs_dir(), cache_dir(), tmp_dir()] {
        let _ = std::fs::create_dir_all(dir);
    }
}
