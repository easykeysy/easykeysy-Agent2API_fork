//! Qoder：** = provider** 。
//!
//! ** already to **（`agent2api-providers`， not ）：
//! COSY and request、SSE 、、、、daily to 、
//! （UMID ）、directory and will in that 。 and config：
//!
//! ```text
//! endpoints.rs ↔ provider id ↔ ProviderKind 、
//! `account_store::qoder_accounts`（ ）
//! `api::qoder_umid`（ `admin.action`）
//! ```
//!
//! ── Why will （ in ）────────────────────────────
//! nonepath「request by use after 、response is OpenAI SSE」。
//! Qoder not ：request need ，response need 1 —— 
//! `conversation`， 「 → → 」。、
//! 、telemetry by 1 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod endpoints;

pub use endpoints::Region;
