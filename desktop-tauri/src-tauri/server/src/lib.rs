//! Agent2API （ crate）。
//!
//! ── crate ────────────────────────────────────
//! · Tauri ： crate ， as HTTP
//! in tauri Tokio when up （`server::start`）；
//! · headless / Docker：`agent2api-server` （`src/bin/`）
//! Tokio when 、（`server::static_files`），
//! not GUI 。
//!
//! is ** 1 **：`server/`、`port_conflict.rs` directory
//! and file and before （ crate `src/server`） 1 ，all
//! `crate::server::…` / `crate::port_conflict::…` path —— 
//! use `use agent2api_server as server` after ， has path。
//!
//! ── Why spawn must by [`spawn_task`] ─────────────────────
//! before use is `tauri::async_runtime::spawn`（ when ，
//! any can ）； crate not tauri，headless `#[tokio::main]`
//! when 。 1 [`spawn_task`] ：
//! · already in Tokio when up down （： in tauri when 
//! up ；headless：`#[tokio::main]`）→ `Handle::spawn`，
//! in **** when up ， and before as 1 ；
//! · not in when up down （ from any after ）→ to create 
//! when ，** not will panic** —— release is panic=abort，
//! 1 use will ， this is before 1 
//! `tauri::async_runtime::spawn` Reason， 1 。

pub mod paths;
pub mod port_conflict;
pub mod server;
pub mod web_shim;

use std::future::Future;
use std::sync::OnceLock;

use tokio::runtime::{Handle, Runtime};

/// when ：「 in Tokio up down after 」，
/// path（handler / has ） not will to 。
static GLOBAL_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// 1 spawn ： and headless use （
/// `tauri::async_runtime::spawn` ， crate ）。
pub fn spawn_task<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    match Handle::try_current() {
        Ok(handle) => handle.spawn(future),
        Err(_) => global_runtime().spawn(future),
    }
}

fn global_runtime() -> &'static Runtime {
    GLOBAL_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("创建后台 Tokio 运行时失败")
    })
}
