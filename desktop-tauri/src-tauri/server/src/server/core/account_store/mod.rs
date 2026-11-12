//! （ for Node src/workbuddy-account-store.mjs ）。
//!
//! ── ：SQLite `accounts` （ from accounts.json ）────
//! in `{config_dir}/agent2api.db` `accounts` （
//! `server/db/schema.rs`）。 1 ，`data` ** JSON
//! **（ down not 1 ）， has `id` / `provider` / `priority` /
//! `enabled` / `added_at` is from JSON ****（// 1 
//! use ）， then and `sql.rs` module header。
//! not ， is ：
//!   `[{ id, provider, name, uid, nickname, type, enterpriseId,
//!     enterpriseName, accessToken, refreshToken, expiresAt, refreshExpiresAt,
//!     domain, tokenTail, prefixPath, endpoint, platform, edition, addedAt,
//!     updatedAt, priority, enabled, proxy, rateLimits: { [modelId]: {...} } }]`
//!
//! ── not （ before ）──────────────────────────────
//! 1. **unknown**： use + `extra` ， when and 。
//! use when not can （ old currentAccountId、
//! 、add in this ）。
//! add `provider` is ** already **， not in when 。
//! down this in `data` 「 JSON 」 up 。
//! 2. ** 1 **（all use 1 ）： use （）。
//! 1 （409）， when for has 1 
//! （`migrate_startup`： old by by old and + ）。
//! **（ by ） 1 ： 1 in ** ——
//! by 「 this before 」， up 「 N in before 」 this 
//! and 1 。 need ：add/ 1 in 
//! （`sql::put` that 1 ）， by `store_order::reorder_accounts`
//! by use 。 then in `priority::grouped_queue_order`。
//! 「current」 not is ， is ** by ** ——
//! 1 「 already enabled and has available」 （`pick_current`）， not in 
//! 「 A、 use B」 this 。current 。
//! when 「 has 」。 in `currentAccountIds`
//! to （ has ）。
//! 3. **provider **： when / as empty 1 `"workbuddy"`
//! （ up when ）， by `migrate_startup` and 
//! 1 （ §3.2）。unknown provider **** ——
//! can can is new old to ， 1 will error 。
//!
//! ── provider ： no has （W4a ）─────────────────
//! ** not ** `["workbuddy","raccoon"]` this id ， will not need ：
//! 「 this provider id not 」 1 
//! `providers::is_known_provider_id`（= `kind_from_id(id).is_some()`，
//! is `providers::PROVIDERS` ）。 is 「 1 provider」
//! 1 ，（/// need ） —— W4a CatPaw / AutoClaw
//! when is this 。**** as （ 、 when 、
//! and path） by id ， that 「 this 1 」。
//!
//! ── and ──────────────────────────────────────────────
//! Node is ； this store use 1 `std::sync::Mutex` 
//! 「--」， not etc 。 ** not is file**：
//! before is 「file--」， in is 「 up 1 」—— 
//! 「 1 → in Rust → 」，SQLite not to this 
//! （"" in Rust ）， this required（ `store.rs` ）。
//! ：** not request**（ will all API）—— need need use 
//! （token new 、） 1 in ， after request，
//! 。
//!
//! （ for Node file）：
//! priority.rs then （ 1 ///， by provider use ）
//! state.rs （JSON has + + + provider ）
//! sql.rs SQL ： 1 ↔ StoredAccount 、 by need 、
//! and （ 1 SQL file）
//! store_util.rs JS （`x || y` / `Number(x)` / `JSON.stringify` …）
//! store.rs AccountStore ：//、 and 、current、
//! by id / provider by need 
//! store_view.rs （ + ）、、provider need 
//! store_crud.rs （add/remove/promote/update/move and apply_patch ）
//! store_order.rs （）： by to （`reorder_accounts`）
//! store_batch.rs （batch_update / batch_remove）
//! store_admin.rs token 、、（provider + ）
//! raccoon_accounts.rs （ already to ； this ****
//! ）
//! catpaw_accounts.rs CatPaw and （ already to ；
//! this ）
//! autoclaw_accounts.rs AutoClaw （ already to ； this ****
//! + id）
//! cline_accounts.rs Cline （ ）：、、
//! 、Cline 
//! qoder_accounts.rs Qoder （ + userId ）：、 new 、
//! Qoder 
//! custom_accounts.rs （`custom-` before provider）：
//! （apiKey + baseUrl ）、delete、custom ；
//! and `core::custom_providers`
//! accio_accounts.rs Accio （ 1 provider）：（ /
//! use ）、、（ edition）
//! monkeycode_accounts.rs MonkeyCode （ 1 provider）：
//! session （session + imageId + userId）、（ edition）
//! commandcode_accounts.rs Command Code （ 1 provider）： `user_`
//! API Key 、（ to ）
//! antigravity_accounts.rs Antigravity （ 1 provider）： Google
//! refresh token 、 new （-）、（ to ）

pub(crate) mod account_identity;
pub mod accio_accounts;
pub mod antigravity_accounts;
pub mod autoclaw_accounts;
pub mod catpaw_accounts;
pub mod chatglm_accounts;
pub(crate) mod checkin_state;
pub mod cline_accounts;
pub mod codearts_accounts;
pub mod atomcode_accounts;
pub mod commandcode_accounts;
pub mod custom_accounts;
mod hidden_accounts;
pub mod kuku_accounts;
pub mod lobsterai_accounts;
pub mod loomy_accounts;
pub mod modelscope_accounts;
pub mod monkeycode_accounts;
pub mod officeace_accounts;
pub mod orcaterm_accounts;
pub mod plugin_accounts;
mod plugin_identity;
mod plugin_public;
pub mod priority;
pub mod qoder_accounts;
pub mod raccoon_accounts;
mod read_rows;
mod read_state;
pub mod sql;
mod snapshot;
pub mod state;
pub mod store;
pub mod store_admin;
pub mod store_batch;
pub mod store_crud;
pub mod store_order;
pub mod store_util;
pub mod store_view;
pub mod trae_accounts;
pub mod zcode_accounts;

pub use store::{AccountStore, AccountStoreError};

/// （**-** return， new use ）。
///
/// new is seconds ， can can （ use 、、 new 
/// 、 1 new ）。 not 「none」： use to 
/// ** new before that **，store in ** 1 ** after 。 not 1 when return
/// [`CredentialWrite::Stale`] —— use must use new ， not can old success
/// used for after request。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CredentialWrite {
    /// already （ is new before that ）
    Written,
    /// ： already （ or already delete / id ）， new 
    Stale,
}

/// token limit（ for Node MAX_TOKEN_LENGTH）
pub const MAX_TOKEN_LENGTH: usize = 8192;

/// CatPaw provider id（ need use ；** from **，
/// [`RACCOON_PROVIDER_ID`] —— id， this ）。
pub(crate) const CATPAW_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::CatPaw,
);

/// AutoClaw provider id（ need use ；** from **，
/// [`RACCOON_PROVIDER_ID`] ）。
///
/// this is **** id（， not —— ）；
/// [`AUTOCLAW_INTL_PROVIDER_ID`]。「 is not is AutoClaw 」 use 
/// [`is_autoclaw_family`]。
pub(crate) const AUTOCLAW_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::AutoClaw,
);

/// AutoClaw **** provider id（ [`AUTOCLAW_PROVIDER_ID`] ）。
///
/// is provider（ by `providers::autoclaw::region` module header）：
/// has ， 「 by 」must ——
/// 「 / 」 this use [`is_autoclaw_family`]。
pub(crate) const AUTOCLAW_INTL_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::AutoClawIntl,
);

/// Trae provider id（ need use ；** from **，
/// [`RACCOON_PROVIDER_ID`] ）。 `trae_accounts.rs`。
pub(crate) const TRAE_PROVIDER_ID: &str =
    crate::server::core::providers::kind_id(crate::server::core::providers::ProviderKind::Trae);

/// Loomy（）provider id（ need use ；** from **，
/// [`TRAE_PROVIDER_ID`] ）。 `loomy_accounts.rs`。
pub(crate) const LOOMY_PROVIDER_ID: &str =
    crate::server::core::providers::kind_id(crate::server::core::providers::ProviderKind::Loomy);

/// MonkeyCode **** provider id（ need use ；** from **）。
/// `monkeycode_accounts.rs`；「 is not is MonkeyCode 」 use 
/// [`is_monkeycode_family`]。
pub(crate) const MONKEYCODE_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::MonkeyCode,
);

/// MonkeyCode **** provider id（ [`MONKEYCODE_PROVIDER_ID`] ）。
///
/// is provider（ by `providers::monkeycode::region` module header）：
/// has ， 「 by 」must。
pub(crate) const MONKEYCODE_INTL_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::MonkeyCodeIntl,
);

/// this provider is not is **MonkeyCode **（ 1 ）。
///
/// and [`is_accio_family`] / [`is_zcode_family`] 1 、 1 by ： has 
/// 「 is not is MonkeyCode」， not 。
pub(crate) fn is_monkeycode_family(provider_id: &str) -> bool {
    provider_id == MONKEYCODE_PROVIDER_ID || provider_id == MONKEYCODE_INTL_PROVIDER_ID
}

/// this provider is not is **ModelScope **（ 1 ）。
///
/// and [`is_autoclaw_family`] / [`is_monkeycode_family`] 1 、 1 by ：
/// has 「 is not is ModelScope」（、），
/// not —— that ， is or when this 1 ，
/// not is in file `id == "modelscope"`（ that for as ，
/// is not will ）。
///
/// from `Region` （ [`is_accio_family`] / [`is_zcode_family`]），
/// ** not need ** use `starts_with("modelscope-")` —— id 。
pub(crate) fn is_modelscope_family(provider_id: &str) -> bool {
    crate::server::core::providers::modelscope::Region::from_provider_id(provider_id).is_some()
}

/// Command Code provider id（ need use ；** from **）。
/// `commandcode_accounts.rs`（ 1 provider、 no has ）。
pub(crate) const COMMANDCODE_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::CommandCode,
);

/// Antigravity provider id（ need use ；** from **）。
/// `antigravity_accounts.rs`（ 1 provider、 no has ——
/// `sandbox`/`daily`/`prod` is not is ， §6）。
pub(crate) const ANTIGRAVITY_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::Antigravity,
);

/// OfficeAce（ as / OfficeClaw）provider id（** from **，
/// [`LOOMY_PROVIDER_ID`] ）。 `officeace_accounts.rs`。
pub(crate) const OFFICEACE_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::OfficeAce,
);

/// AtomCode（AtomGit / GitCode）provider id（** from **，
/// [`LOOMY_PROVIDER_ID`] ）。 `atomcode_accounts.rs`。
pub(crate) const ATOMCODE_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::Atomcode,
);

/// LobsterAI（ has 「 has 」）provider id（** from **，
/// [`LOOMY_PROVIDER_ID`] ）。 `lobsterai_accounts.rs`。
pub(crate) const LOBSTERAI_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::LobsterAi,
);

/// （`chatglm.cn`）provider id（** from **，
/// [`LOOMY_PROVIDER_ID`] ）。 `chatglm_accounts.rs`。
///
/// is 1 has **** （`guest/access` ； already to ），
/// **** need 1 `isGuest`
/// （ `store_view::public_account`）， to by excluding
/// （ `billing::checkin::supports_checkin`）。 `isGuest` by 
/// `account_view` ， not this 。
pub(crate) const CHATGLM_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::Chatglm,
);

/// Cline **** provider id（ need use ；** from **，
/// [`RACCOON_PROVIDER_ID`] ）。 `cline_accounts.rs`。
pub(crate) const CLINE_FREE_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::ClineFree,
);

/// Cline **** provider id（ [`CLINE_FREE_PROVIDER_ID`] ）
pub(crate) const CLINE_PASS_PROVIDER_ID: &str = crate::server::core::providers::kind_id(
    crate::server::core::providers::ProviderKind::ClinePass,
);

/// this provider is not is **Cline **（ 1 ）。
///
/// 「 is not is Cline」（ when 、、、
/// in `account` up ）， not —— that ，
/// is will or when need this 1 ， not is in file `id == "cline"`。
///
/// Note `"cline"` this id ** already not in **（ after is `cline-free` /
/// `cline-pass`， `providers::PROVIDERS`）： `id == "cline"` will as ，
/// is not will ， this to 1 。
pub(crate) fn is_cline_family(provider_id: &str) -> bool {
    provider_id == CLINE_FREE_PROVIDER_ID || provider_id == CLINE_PASS_PROVIDER_ID
}

/// this provider is not is **AutoClaw **（ 1 ）。
///
/// and [`is_cline_family`] 1 、 1 by ： has 
/// 「 is not is AutoClaw」（、 in `userId` up ）， not ——
/// that ， is or when this 1 ， not is in file 
/// `id == "autoclaw"`（ that for as ， is not will ）。
pub(crate) fn is_autoclaw_family(provider_id: &str) -> bool {
    provider_id == AUTOCLAW_PROVIDER_ID || provider_id == AUTOCLAW_INTL_PROVIDER_ID
}

/// this provider is not is **Accio **（ 1 ）。
///
/// and [`is_autoclaw_family`] 1 、 1 by ： has 
/// 「 is not is Accio」（ to excluding、）， not ——
/// that ， is or when this 1 ， not is in file 
/// `id == "accio"`（ that for as ， is not will ）。
pub(crate) fn is_accio_family(provider_id: &str) -> bool {
    matches!(provider_id, "accio" | "accio-cn")
}

/// this provider is not is **ZCode **（ 1 ）。
///
/// and [`is_autoclaw_family`] / [`is_accio_family`] 1 、 1 by ：
/// has 「 is not is ZCode」（ to excluding、、
/// ）， not —— that ， is 
/// or when this 1 ， not is in file `id == "zcode"`
/// （ that for as ， is not will ）。
pub(crate) fn is_zcode_family(provider_id: &str) -> bool {
    crate::server::core::providers::zcode::region::Region::from_provider_id(provider_id).is_some()
}

/// this provider is not is **Qoder **（`qoder` in / `qoder-intl` ）。
///
/// and [`is_accio_family`] / [`is_zcode_family`] 1 、 1 by ： has 
/// 「 is not is Qoder」（、）， not ——
/// that ， is or when this 1 ， not is in file 
/// `id == "qoder"`（ that for as ， is not will ）。
pub(crate) fn is_qoder_family(provider_id: &str) -> bool {
    crate::server::core::providers::qoder::endpoints::Region::from_provider_id(provider_id)
        .is_some()
}

/// provider id（ need use ）。
///
/// ** from **（W4a）：`providers::kind_id` is `const fn`， this can 
/// `const` not —— id， this ，
/// not will 「 by `"raccoon"` 、 」 this 。
///
/// ── provider in （W4a ）─────────────────
/// ** no has ** 1 `["workbuddy", "raccoon"]` ， not ：
/// 「 this provider id not 」 1 
/// `providers::is_known_provider_id`（= `kind_from_id(id).is_some()`， is 
/// `providers::PROVIDERS` ）， need need **** id when that 。
/// is after provider（CatPaw / AutoClaw already ），。
///
/// **** （ 、 when ） by id 
/// —— that is 「 this 1 as 」， not is ， new to 
/// （ 1 has 1 ）。
pub(crate) const RACCOON_PROVIDER_ID: &str =
    crate::server::core::providers::kind_id(
        crate::server::core::providers::ProviderKind::Raccoon,
    );
