//! use 。
//!
//! can 、 ：、 will ， and 
//! （、 to ）。
//! not in this —— by Tauri `AppHandle::get_webview_window` by ，
//! 1 can can 。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tokio::sync::oneshot;

use crate::port_conflict::StartupFailure;

/// HTTP 。
///
/// old this is `Option<Child>`（ node ）； after after 
/// not has can ， as 「 1 + down 」。
/// up ： in is **none** has —— not has 「 use already 」
/// that 「 no has child not 」 ， as 1 by 。
#[derive(Default)]
pub struct BackendHandle {
    /// ；take after not has （shutdown etc ）
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub port: u16,
    /// 1 failure（success after empty）。
    ///
    /// Why need down ：`backend:error` is 1 ， can can in ** after **
    /// （ no 、 use new WebView）。 ， use 
    /// not to 「Why not 」—— can to 1 。
    /// 1 when can ，「 in 」。
    pub failure: Option<StartupFailure>,
}

/// 1 in ：state used for after ，edition/mode and when 。
#[derive(Clone)]
pub struct ActiveLogin {
    pub state: String,
    pub edition: String,
    pub mode: String,
    /// this 1 （provider id）。
    ///
    /// Why need ： need 「 this URL is not is 」，
    /// not （ is `office-raccoon://auth/callback`，
    /// workbuddy no has —— by after up ）。 provider in 
    /// up ， not 、 not before 1 parameter。
    pub provider: String,
}

/// and 。
///
/// `exiting` must in 「 use 」 and 「 down 」：
/// will to `RunEvent::ExitRequested`， `close_to_tray`
/// will 「」 not 。
///
/// `close_to_tray` / `lightweight` 1 currentset：
/// （ `ExitRequested`） is 、 1 can can ， not this 。
#[derive(Default)]
pub struct WindowState {
    pub exiting: AtomicBool,
    pub close_to_tray: AtomicBool,
    /// ： when （ WebView2 ） not is 
    pub lightweight: AtomicBool,
}

impl WindowState {
    /// use （）： after `ExitRequested` must
    pub fn begin_exit(&self) {
        self.exiting.store(true, Ordering::SeqCst);
    }

    pub fn is_exiting(&self) -> bool {
        self.exiting.load(Ordering::SeqCst)
    }

    pub fn set_close_to_tray(&self, value: bool) {
        self.close_to_tray.store(value, Ordering::SeqCst);
    }

    pub fn close_to_tray(&self) -> bool {
        self.close_to_tray.load(Ordering::SeqCst)
    }

    pub fn set_lightweight(&self, value: bool) {
        self.lightweight.store(value, Ordering::SeqCst);
    }

    pub fn lightweight(&self) -> bool {
        self.lightweight.load(Ordering::SeqCst)
    }
}

#[derive(Default)]
pub struct AppState {
    pub backend: Mutex<BackendHandle>,
    pub login: Mutex<Option<ActiveLogin>>,
    pub window: WindowState,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    /// use ： after `RunEvent::ExitRequested` not 
    pub fn begin_exit(&self) {
        self.window.begin_exit();
    }
}
