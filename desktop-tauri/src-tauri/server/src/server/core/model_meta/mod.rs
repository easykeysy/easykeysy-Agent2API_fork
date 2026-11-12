//! models.dev ： ** can ** and **** from 。
//!
//! ── ──────────────────────────────────────────
//! for down can （ up down 、、 use 、、、）
//! and 「 this 」， before has ： up directory（ has 
//! to 、 has not to ）、 （ not up up ， has is ）、
//! and use （`modelRules.capabilities`）。 is 「 up no to 、
//! no 」 that 1 —— down to is 「」， can 。
//!
//! （magpie） ：**， and etc by 
//! models.dev **（ README ）。 is this after 。
//!
//! ── （ 1 then ， has this 1 ）─────────────────────
//! ** use > up > models.dev > **。
//! models.dev 「 up no 」 ； up 1 not ， use 
//! after （ in use after ）。 by `fill` module header。
//!
//! ── from （， `snapshot`）────────────────────────
//! `builtin` （`modelsdev.json`， by 
//! `scripts/sync-modelsdev.mjs` ， when ）——
//! 、Docker 、 not when ；
//! `cache` 1 `kv` up 1 success new （ not ）；
//! `network` new to （`refresh`）。
//! cache > builtin； new success，failure old 。
//!
//! ── has （ is 1 1 ）─────────────
//! · `providers::catalog::manifest_for` 
//! · `custom_providers::bindings::public_models` 
//! in not will 「 has 、 down no has 」 （ and can 1 by ）。
//!
//! ── this after ** no has ** （ has ，）─────────────
//! · not up already （ up ）；
//! · not （ up in use ；magpie use models.dev name is 
//! as not ）；
//! · not / / / is etc （ not use ）；
//! · not that 「」 —— that is 1 （ no has 
//! 「」）， will has as ， need need 。
//!
//! ── path ────────────────────────────────────────────
//! `fill::apply` in when for 1 ： 1 （）
//! 1 。 already has 、、 this O(n) ， this not 
//! ； in new when （ 1 ）。

mod fill;
mod refresh;
mod snapshot;

pub use fill::{apply, mark_user};
pub use refresh::refresh;
pub use snapshot::{install, status};

/// scope（ and `modelRefresh:<provider>` 1 empty）。
pub(crate) use refresh::SCHEDULE_SCOPE;
