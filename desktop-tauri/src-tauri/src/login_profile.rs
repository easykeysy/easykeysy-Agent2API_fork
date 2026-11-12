//! 1 when directory。
//!
//! WebView2 Cookie / localStorage / in **directory**， use defaultdirectory
//! etc use ： when will up 1 ，
//! use no has will 。 new 1 directory， after 。
//!
//! directory in when directory down （ not is configdirectory）： is 1 ， not when this by 
//! ， not this `~/.agent2api` 。

use std::path::{Path, PathBuf};
use std::time::Duration;

/// deletedirectory and （WebView2 after need 1 when directory）
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
        let path = std::env::temp_dir().join(format!("agent2api-login-{suffix}"));
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
        // in delete： this can can in tokio when up down in ，delete need 
        // 1 will etc WebView2 —— not use ， not when 
        // spawn_blocking（ use when that path will panic）。
        // create directory（）， not when 1 ：
        // down use is new directory， 1 old not will 。
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
