//! ：、、。
//!
//! is use 1 「 after 」： after in （ lib.rs to ），
//! use can or ， and use 
//! 1 `show_main_window` / ，「 not 」 this 。
//! down will （ lib.rs `ensure_main_window`）， when by 
//! 1 parameter —— file「」， not is 、
//! is not in 。
//!
//! use `app.default_window_icon()`（ tauri.conf.json config bundle ），
//! not use 。

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

use crate::state::AppState;

pub const TRAY_ID: &str = "main-tray";
const MENU_SHOW: &str = "tray-show";
const MENU_QUIT: &str = "tray-quit";

/// create。 use when use 1 can 。
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, MENU_SHOW, "显示主窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, MENU_QUIT, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        // and 1 （ "Dev" ， lib.rs app_title）：
        // when when ， is 1 can 1 is 
        .tooltip(crate::app_title())
        .menu(&menu)
        // to 「」， not ；
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => show_main_window(app),
            MENU_QUIT => request_exit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // 「」：Windows down by down and 1 ，
            // not will when 
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    // not to default ： can is none ，
    // not can after use 1 
    builder.build(app)?;
    Ok(())
}

/// and 。 can can 「」「」，
/// can can in down already —— need can ：
/// before by unminimize → show → set_focus 1 ；
/// by `ensure_main_window` by parameter after 。
pub fn show_main_window(app: &AppHandle) {
    let Some(window) = crate::ensure_main_window(app) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

/// use 。
///
/// 「 in 」 `exit(0)`： to when ，`exit(0)` will 
/// `RunEvent::ExitRequested`， no has this will 「 use 」 1 ，
/// as 「 in 」。
///
/// this need after ：`exit(0)` this not `RunEvent::Exit`
/// （Tauri already as ）， lib.rs will ，node will 3065 
/// in after 。shutdown is etc ， 1 none。
pub fn request_exit(app: &AppHandle) {
    app.state::<AppState>().begin_exit();
    crate::backend::shutdown(&app.state::<AppState>());
    app.exit(0);
}
