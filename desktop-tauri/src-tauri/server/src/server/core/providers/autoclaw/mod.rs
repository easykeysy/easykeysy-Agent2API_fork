//! AutoClaw（ autoglm） ****（ this 1 ）。
//!
//! ── Why `region` ─────────────────────────────────────
//! AutoClaw already **** to （ + + ）：
//!
//! ```text
//! / request / plugins/autoclaw/adapter and mod.rs
//! / safeStorage plugins/autoclaw/credentials.rs + crypto.rs
//! token （ + ） plugins/autoclaw/refresh.rs
//! by + directory plugins/autoclaw/models.rs + catalog.rs
//! / plugins/autoclaw/balance.rs
//! to / plugins/autoclaw/checkin.rs + promotion.rs
//! （） plugins/autoclaw/login.rs
//! OAuth （） plugins/autoclaw/oauth.rs
//! plugins/autoclaw/callback_server.rs
//! （） plugins/autoclaw/web_login.rs
//! ```
//!
//! this **** ：
//! 1. ↔ provider id （ + ， [`region`]）；
//! 2. （`core::account_store::autoclaw_accounts`）——
//! that no has `pluginSecret`， not to `account_view`。
//!
//! ── （ ）─────────────────────────────
//! to （UA 、client、 up 、client）
//! not in ： by ，。 1 
//! will that —— directory is ** 1 **。
//!
//! ── ───────────────────────────────────────────────
//! auth.json （safeStorage ）only Windows available —— that in in ，
//! not 。 and none。

pub mod region;

pub use region::Region;
