//! directory ****（`/v1/models` and ）。
//!
//! ── this Why shape ────────────────────────────────────
//! WorkBuddy **directory**：、`/v3/config`
//! new 、、 by 。 that is 「 up 」
//! ， WorkBuddy to （ not ）。
//!
//! down is [`shape`]： ** 1 ** down OpenAI 
//! （`id` / `object` / `owned_by` / can / up down ）， and `/v1/models`
//! response。 this is ， and up is none —— 
//! to after ， is this 。
//!
//! ── ────────────────────────────────────────────────
//! · `core::providers::catalog`（directory： 1 ）；
//! · `api::pipeline`（ by when use 1 then ）；
//! · `core::providers::catalog::view`（`/v1/models` ）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod shape;

// for ：directory（`core::providers::catalog`） and from `core::models`
// this ， and before 1 path。
// `shape_value_text` is `shape::value_text` ：directory need by `name`
// ， need need and `shape` 1 JS 。
pub use shape::{list_item, list_response_from, model_id, suggest_from, value_text as shape_value_text};
