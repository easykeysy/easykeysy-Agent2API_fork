//! WorkBuddy（client）：** = provider** 。
//!
//! ** already to **（`agent2api-providers`， not ）：
//! and client、request 1 （ / tool_choice / before /
//! tool for ）、token 、`/v3/config` directory new 、daily、
//! （ / / ） in that 。：
//!
//! ```text
//! region.rs ↔ provider id ↔ ProviderKind 、、
//! `account_store`（default provider id and ）
//! then `core::model_rules`（ use config， and 1 ）
//! ```
//!
//! ── this 1 （ old when will ）─────────────────────
//! is ** 1 up **：`ProviderKind`、、directory is 
//! ， to can to 「workbuddy 」 （default provider id、
//! default、`user-<uid>` id ）。 that is ， not is can 
//! —— as ，`DEFAULT_PROVIDER_ID` to `workbuddy`：
//! `provider` when by 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod region;

pub use region::{is_workbuddy_family, Region};
