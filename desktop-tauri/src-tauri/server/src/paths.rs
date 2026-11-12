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

/// 默认配置目录：`{用户主目录}/.agent2api`
fn default_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".agent2api")
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

/// 日志目录:默认 `{config_dir}/logs`。
///
/// Linux 侧(WSL / headless 部署)在**没有环境变量覆盖时**默认落到
/// `/mnt/ramdisk/Agent2API/logs`:日志是高频写、可丢弃、且体积会长大的数据,
/// 与主库分家之后就该归内存盘管 —— 这条路径原本靠用户手挂的符号链接,
/// 现在落实成代码默认。`AGENT2API_PROXY_HOME`(或旧名)在设时说明用户整个
/// 数据目录搬了家,侧目录必须跟着走,不另开内存盘路径。
/// `/mnt/ramdisk` 不存在(非 WSL 的普通 Linux、容器)时回落 `{config_dir}/logs`,
/// 不硬造一棵不存在的挂载树。
pub fn logs_dir() -> PathBuf {
    side_dir("logs")
}

/// 缓存目录:默认 `{config_dir}/cache`;Linux 侧规则与 [`logs_dir`] 完全一致
/// (`/mnt/ramdisk/Agent2API/cache`)。缓存是**可丢弃**数据:清掉不影响正确性,
/// 这正是它该去内存盘的理由。
pub fn cache_dir() -> PathBuf {
    side_dir("cache")
}

/// 临时目录:默认 `{config_dir}/tmp`;Linux 侧规则与 [`logs_dir`] 一致
/// (`/mnt/ramdisk/Agent2API/tmp`)。登录窗口之类的一次性数据落这里 ——
/// 它本来就是「用完就丢」的,归内存盘管正合适。
pub fn tmp_dir() -> PathBuf {
    side_dir("tmp")
}

/// 上面三个侧目录的共用实现:`(env 覆盖 / 非 Linux / 无内存盘) →
/// `{config_dir}/{name}`,否则 → `/mnt/ramdisk/Agent2API/{name}`。
///
/// 只认 `/mnt/ramdisk` 本身(不试别的盘、不看 `/mnt/ramdisk1`):这是本机部署
/// 的实际挂载点,多一个候选只会在排查时多一种「为什么走了那条」的困惑。
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

/// 侧目录是否真的外置(`/mnt/ramdisk` 接管了它们)。
///
/// 日志库据此决定事件行写文件还是写库:只有外置时才切文件后端 —— 没有内存盘
/// 的 Linux / 容器 / 测试环境保持原来的库后端,行为与改造前逐字一致。
#[allow(dead_code)]
pub fn side_dirs_external() -> bool {
    logs_dir() != config_dir().join("logs")
}

/// 启动时补建三个侧目录。
///
/// ramdisk 每次重启都会被清空,而写侧各自 `create_dir_all` 只能兜住「写的那
/// 一个」;启动时统一补一遍,设置页展示的路径从一开始就是存在的。失败不致命
/// (目录不可写时各写侧本来就会降级)。
#[allow(dead_code)]
pub fn ensure_side_dirs() {
    for dir in [logs_dir(), cache_dir(), tmp_dir()] {
        let _ = std::fs::create_dir_all(dir);
    }
}
