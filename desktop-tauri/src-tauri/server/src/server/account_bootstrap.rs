//! ** 1 **： 1 ， use 。
//!
//! Why 1 file（ not is in `server/mod.rs` ）
//! ────────────────────────────────────────────────────
//! in 1 ：`server/mod.rs` already file（800 ），
//! this 1 ** use when **（ when 、 after ） is 
//! 1 ， 1 can to 。 after `mod.rs` 1 
//! `if !upgrade_pending { account_bootstrap::run(&store); }`， will 
//! that to this ， not is in that 。
//!
//! ── this 1 Why not can none（file need ）─────────
//! in 「 that is current」 this before up ， is 
//! 「empty not empty / has no has 」。** old JSON/JSONL no when this 
//! before not **： `accounts` is empty （ in `accounts.json` 
//! no ）， is ：
//! 1. [`AccountStore::import_legacy_session`]（ old `auth.json`） will as 
//! 「empty」**** 1 ， `db::migrate::import_accounts` 
//! etc （**empty**）** up ** —— use after 「」， 
//! `accounts.json` not ， up that 1 old 。
//! this is 1 path，must in ** this before **，
//! not is 。
//! 2. old is 「 as empty」， will 、 
//! 。** this already to **（raccoon / CatPaw /
//! AutoClaw is client has file and safeStorage ）， this 
//! not has need need in before —— this 1 in is 。
//! 3. [`AccountStore::migrate_startup`] will in empty up down 
//! `priorityScope: "global"`（「 use not is global」 this ，
//! empty not ）—— `import_accounts` need ** this not in **
//! 「 by old 1 」。 before ， that not will ，
//! use will in after （ is that need ）。
//!
//! ── is by use ， for 「 before new 」 1 when ──────────
//! - `ServerState::bootstrap`：** no has ** old file when 1 （path，
//! and before as 1 ）；
//! - `POST /api/upgrade/run`： use 、 old already after ** 1 **
//! （ `api::upgrade_api`）。 not this 1 down ， use to is 1 
//! no 、 no provider、 no old ——
//! need etc to down ， that when already up empty「 
//! 」。
//! etc （`migrate_startup` none when not 、 
//! and ）， not will 。
//!
//! ── （ has 1 ，）───────────────────────────
//! and before 1 ， its in after has ：
//! `auth.json` → `migrate_startup`（//）→ 
//! → Key → `apiKey` → `migrate_cline_split`
//! → Key 
//! - 「」（ / Key / Key old ）
//! `migrate_startup` after 、 and in ： 1 no to ，
//! after kv `apiKeys` and `apiKey`， not 、 not after 
//! （ from is ）。
//! - `migrate_cline_split` must in `migrate_startup` ** after **： after will 
//! Cline id `cline-free` / `cline-pass`， before is then 
//! （`modelRules` by provider id disabled / hidden / mappings /
//! seeded）—— ， then will in 「 is old id」 down 1 ，
//! for not up 。
//! - Key and 1 （ `kv` `apiKeys`）， and **
//! provider id **（`workbuddy` → `workbuddy` + `workbuddy-intl`），
//! and after none； in this is as 「 」
//! （、 then 、Key ） in 1 1 。
//! - must** already config** that ：path use already 
//! `config::reload()`（ `api::upgrade_api`），path `config::init`
//! （`read_raw` will old file），。
//! - old in `migrate_startup` after ： that when already ，
//! new not will this 1 （ by provider ）。

use crate::server::core::account_store::AccountStore;
use crate::server::core::model_rules;
use crate::server::logging;

/// 1 （ etc ， can use ）。
pub(crate) fn run(store: &AccountStore) {
    import_legacy_auth_json(store);
    // （ 1 、 1 ， store_admin::migrate_startup）：
    // ① `provider: "workbuddy"`（Agent2API ，§3.2）
    // ② —— provider ， for not 
    // （ after not ）
    // ③ Cline （`provider: "cline"` + `pool` → cline-free / cline-pass）
    // ④ WorkBuddy （`provider: "workbuddy"` + `edition: "intl"` → workbuddy-intl）
    store.migrate_startup();
    // （ store_admin::encrypt_legacy_accounts）：
    // `accounts.data` from is `enc1:` ， before already in is 
    // JSON（ `decode_data` ， that available）。 this 1 
    // —— after 「file」 not to 。
    //
    // in `migrate_startup` ** after **： that 1 will by （ 
    // already ）， down no to by this 1 。 etc （ is ，
    // no has kv ）、failure not （unavailable when return， down ）。
    store.encrypt_legacy_accounts();
    // （ store_admin::backfill_plugin_identity）：
    // pluginSecret / pluginPublic， uid / edition，、
    // to 、path（issue #380）。 already in new when ，
    // this 1 before in that 。 etc （ is ：、 for has 
    // ）、failure not （， can available）。
    // in `migrate_startup` after ： that 1 can can provider（ / ），
    // by new provider edition for 。
    store.backfill_plugin_identity();
    // Key （ `core::api_keys::encrypt_legacy_keys`）：
    // `apiKeys` Key `key` from is `enc1:` ， before 
    // already in config is （ `from_value` ， that Key available）。
    // this 1 —— after 「file」 not to 。
    //
    // and up 1 ** and **（ that 1 after ）： not ， is as 
    // 「 1 」， in 1 this 1 1 。
    // `config::current().raw()`，**must etc config**：path
    // `config::init` in bootstrap already （ use ），path
    // use `config::reload()`（ `api::upgrade_api`）， ——
    // and in this `add_intl_to_workbuddy_allowlists` is 1 。
    // etc （ is ： has no has key）、failure not （unavailable when 
    // return， Key available， down ）， and up 1 1 。
    crate::server::core::api_keys::encrypt_legacy_keys();
    // old `apiKey` ****（ `core::legacy_api_key`）： and up 1 
    // 「」—— `apiKeys` and `apiKey` is 1 「」 
    // ， in 1 this 1 1 。 and failure up 1 ：
    // `config::current().raw()`（must etc config，，
    // up 1 ）、 etc （ is ： not is `enc1:` ）、
    // failure not （ Key available， down ）。
    // ****： in `apiKey`、 not —— will 「 1 
    // `apiKeys` and `apiKey`」 this has （ in new module header）。
    crate::server::core::legacy_api_key::encrypt_legacy_api_key();
    // then 1 （module header ）： is `kv` 
    // `modelRules`，must** already config** that 。
    if let Some(summary) = model_rules::migrate_cline_split() {
        logging::log("[Models]", &summary);
    }
    // WorkBuddy （2026-10） ： 1 in 1 down 。
    // then / Key / by before by `workbuddy` 1 id
    // ， need that config**** to 。
    // **must 1 **（ `config::workbuddy_split_migrated` ）：
    // 「 1 has 、 1 no has 1 」 in after 1 ， not will in 
    // use after new config。
    // ： then in before （ can not can ）， not 。
    //
    // and after none（ has ）： can can already in 
    // `restore_cached_catalogs` and default use ， then 
    // is 「 as 」 not is 「」—— that 。
    if !crate::server::config::workbuddy_split_migrated() {
        let (outcome, rules_summary) = model_rules::migrate_workbuddy_split();
        let patched_keys = crate::server::core::api_keys::add_intl_to_workbuddy_allowlists();
        let prompt_summary = crate::server::config::migrate_workbuddy_split_prompt_providers();
        if let Some(summary) = rules_summary {
            logging::log("[Models]", &summary);
        }
        if !patched_keys.is_empty() {
            logging::log(
                "[Keys]",
                &format!(
                    "🔑 已为 {} 把网关 Key 补上「WorkBuddy 国际版」的可用提供商（拆家前 workbuddy 覆盖两地，限制范围保持不变）：{}",
                    patched_keys.len(),
                    patched_keys
                        .iter()
                        .map(|entry| entry.name.as_str())
                        .collect::<Vec<_>>()
                        .join("、"),
                ),
            );
        }
        if let Some(summary) = prompt_summary {
            logging::log("[Prompt]", &summary);
        }
        // then ** no ** when not （ down 1 ， use 
        // in ）。，failure。
        if outcome.allows_marking_done() {
            if !crate::server::config::mark_workbuddy_split_migrated() {
                logging::log(
                    "[Models]",
                    "⚠️  WorkBuddy 拆家迁移标记未能落盘：下次启动会再核对一遍（三条迁移都幂等，不会重复改动）",
                );
            }
        }
    }
    // Trae old ID（`-solo` / `-solo-max`）： 1 ， is 
    // config and directory， not down in request（ old in will 1 ）。
    // must in before ： will Trae up ID，
    // after 1 「 this is not is up 」 。
    if !crate::server::core::trae_model_ids::migrated() {
        if crate::server::core::trae_model_ids::migrate(store) {
            if !crate::server::core::trae_model_ids::mark_migrated() {
                logging::log(
                    "[Models]",
                    "⚠️  Trae 历史模型 ID 清理标记未能落盘：下次启动会再核对一遍（迁移幂等）",
                );
            }
        }
    }
    // ： by **request** down （）。
    //
    // must in this —— `model_rules::current()` （ need already config，
    // and `migrate_cline_split` 1 ）， and need （otherwise that 
    // will in up 「 in 」， use as ）。
    // is ** already in **， new in `routing::CooldownKeys`
    // and `payload::SendBody` ， not need need 。
    let stale_keys = store.migrate_rate_limit_keys();
    if !stale_keys.is_empty() {
        logging::log(
            "[Accounts]",
            &format!(
                "🧹 已清理 {} 条错位的限流记录（旧版把映射别名当成冷却键，实际额度按上游真名记）：{}",
                stale_keys.len(),
                stale_keys.join("；"),
            ),
        );
    }
    // old 1 ** already to **（ and down CatPaw / AutoClaw
    // 1 ）： `~/.raccoon-proxy/accounts.json` and 
    // `~/.box-agent/config/auth.json` is old file and client，
    // after up client。 use by `account.normalize` in 「
    // 」 when ， old file `authFileContent` 1 。
    // CatPaw old 1 （ §9，W5-T-d4）** already to **：
    // （`~/.meituan-catpaw/catpaw-proxy-accounts.json` and 
    // `~/.meituan-catpaw/auth.json`） is CatPaw client has file，
    // 「 up 」， not this in 。 use by `account.normalize`
    // in 「」 when ； old file by use 1 
    // （`POST /api/accounts` `authFileContent` path）。
    // AutoClaw old 1 ** already to **： 
    // （`~/.autoclaw-proxy/accounts.json` and `%APPDATA%/AutoClaw/auth.json`）
    // is AutoClaw client has file + safeStorage ，「 up 」。
    // use by `account.normalize`（） or 
    // `POST /api/accounts` `authFileContent`（ 1 ）。
}

/// old `auth.json` → 1 workbuddy （only as empty when 1 ）。
///
/// file [`read_legacy_session`]（/ no has ），
/// `import_legacy_session`（「empty not empty」 and return after ）。
/// this ** and **（ `accounts` empty not empty），
/// is 「Why not can none」 that 1 1 —— module header。
fn import_legacy_auth_json(store: &AccountStore) {
    let Some(legacy) = read_legacy_session() else {
        return;
    };
    let Some(account) = store.import_legacy_session(&legacy) else {
        return;
    };
    let name = account
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("旧登录态");
    logging::log("[Accounts]", &format!("✅ 旧版登录态已迁移为账号: {name}"));
}

/// old auth.json（/ no has ， for Node `loadStoredSession`）。
///
/// 「 has empty `auth.accessToken`」 this 1 ： its （file not in 、JSON 、
/// not for 、token as empty） 1 no has —— and Node to 1 ，
/// not as 1 not old filepath。
fn read_legacy_session() -> Option<serde_json::Value> {
    let path = crate::server::core::endpoints::legacy_auth_file();
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&text).ok()?;
    let token = value
        .get("auth")
        .and_then(|auth| auth.get("accessToken"))
        .and_then(serde_json::Value::as_str)?;
    if token.is_empty() {
        return None;
    }
    Some(value)
}
