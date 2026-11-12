//! Cline（ `api.cline.bot`）：** = provider** 。
//!
//! ** already to **（`agent2api-providers`， not ）：
//! 、、、directory、 up request（default）、
//! 、 in that 。 and config：
//!
//! ```text
//! models.rs ↔ provider id ↔ ProviderKind 、 before 、 then 
//! `account_store::cline_accounts`（ ）
//! config `clineUpstreamHeaders`（ use ）+ `api::cline_headers`
//! （default and and after need ， that ）
//! ```
//!
//! ── Whydefault need ──────────────────────────────
//! that （、、） that ， is as up by 「 use is not is 
//! 」， 1 403。** up **， and 「
//! client」 is 1 —— by need not in 。 use that 
//! is config， in （ not default is ）。
//!
//! ── Why is ──────────────────────────────────────
//! by `models.rs` module header： need as ** has **， not is 
//! 「 has 」 when 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod models;

pub use models::Pool;
