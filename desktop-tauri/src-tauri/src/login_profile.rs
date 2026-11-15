//! 每次网页登录独享一个临时数据目录。
//!
//! WebView2 的 Cookie / localStorage / 缓存都存放在**数据目录**里，共用默认目录
//! 就等于共用登录态：添加第二个账号时官方登录页会直接把上一个账号放行过去，
//! 用户根本没有机会换账号。因此每次登录都新建一个随机目录，窗口销毁后再删掉。
//!
//! 登录临时目录的根：`AGENT2API_LOGIN_DIR` 环境变量优先；未设置时回落
//! 系统临时目录的叉形子目录 `easykeysy-Agent2API_fork`（Windows 下
//! 即用户目录下的同名子目录，子目录形式 `agent2api-login-*`）：
//! 一次性的，删不掉时也应由系统回收，不该混进 `~/.agent2api` 的持久化数据里。
//! 本机部署（数据全放非系统盘）时设 `AGENT2API_LOGIN_DIR` 指向不盘位置即可。

use std::path::{Path, PathBuf};
use std::time::Duration;

/// 删除目录的尝试次数与间隔（WebView2 关窗后要过一小段时间才释放目录句柄）
const CLEANUP_ATTEMPTS: usize = 40;
const CLEANUP_INTERVAL: Duration = Duration::from_millis(250);

pub struct LoginProfile {
    path: PathBuf,
}

impl LoginProfile {
    pub fn new() -> Result<Self, String> {
        let mut random = [0u8; 8];
        getrandom::getrandom(&mut random)
            .map_err(|_| "生成独立登录环境标识失败".to_string())?;
        let suffix: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
        // fork：登录临时目录的根。
        // 1. `AGENT2API_LOGIN_DIR` 环境变量优先（本机部署指向非系统盘，例如 Y:，
        //    避免 WebView2 数据写进 C: 用户目录）；
        // 2. 未设置时回落默认：系统临时目录的父目录（Windows: C 盘用户目录）下
        //    名为 `easykeysy-Agent2API_fork` 的子目录。
        let base = match std::env::var("AGENT2API_LOGIN_DIR") {
            Ok(value) if !value.trim().is_empty() => PathBuf::from(value.trim()),
            _ => std::env::temp_dir()
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("easykeysy-Agent2API_fork"),
        };
        let path = base.join(format!("agent2api-login-{suffix}"));
        std::fs::create_dir_all(&path)
            .map_err(|error| format!("创建独立登录环境失败: {error}"))?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for LoginProfile {
    fn drop(&mut self) {
        let path = self.path.clone();
        // 在独立线程里重试删除：这里可能运行在 tokio 运行时上下文中，而删除要
        // 睡一会儿等 WebView2 释放句柄 —— 不阻塞调用方，也不依赖运行时的
        // spawn_blocking（应用退出时那条路径会 panic）。
        // 只删本次创建的目录（名字带随机串），清不掉时也只记一行日志：
        // 下次登录用的是新目录，残留一份旧登录态不会造成串号。
        std::thread::spawn(move || {
            for attempt in 0..CLEANUP_ATTEMPTS {
                match std::fs::remove_dir_all(&path) {
                    Ok(()) => return,
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
                    Err(error) if attempt + 1 == CLEANUP_ATTEMPTS => {
                        eprintln!("[Login] 临时登录目录未能清理（不影响下次登录）: {error}");
                    }
                    Err(_) => std::thread::sleep(CLEANUP_INTERVAL),
                }
            }
        });
    }
}
