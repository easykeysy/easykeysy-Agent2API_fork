//! （ old JSON/JSONL file → 1 SQLite ） 。
//!
//! ```text
//! GET /api/upgrade has no has old （ need not need ）
//! POST /api/upgrade/run 1 （ etc ；failure not its ）
//! ```
//!
//! ── Why not in this ， by in when ────────────
//! this update from 「 JSON/JSONL file」 SQLite 。
//! in `POST /api/upgrade/run` not is ， is as need 
//! MB old file and 1 —— ** in after ** this 1 ， use can to 
//! （toast）， not will to 「 can 」 when 。
//!
//! （`ui/upgrade-panel.js`）**、 not **： no has 、 not can 
//! （ not = and unavailable）， is 1 ， 「 after 」
//! will 1 empty。** not **「 old not delete」 this 
//! not —— old file （ down 1 ）。
//!
//! ── old Why not delete ────────────────────────────────────
//! success after old file **** `{}.migrated`（
//! `db::migrate::backup` ： is 1 file ，、
//! not 、 1 file not will ）。 and 
//! ** no has delete use path**。 use no to in 
//! ，must in before after 1 「file in 」。
//!
//! ── after need （ 1 ）─────────
//! store not ， not ， （ and ）：
//! - `config`：**must** `reload()`。 in `bootstrap` ， that when 
//! no ， is from old `config.json` **** ； config
//! after ， is ， 1 use up and 1 that 1 
//! （ `config::reload`）。 this （
//! `config::current().sanitize_fingerprints()`）， config 1 。
//! - `AccountStore` / `LogStore` / `RequestStats` / `debug_traffic`：** not need need **
//! 。 `Db` 、（ no has ），
//! down 1 can to 。
//! - `settings`（set，）： not need need 。 is 「 → old file →
//! default」（`settings::load`）， before after to 1 ； 1 
//! need need 「 to 」 is ， that in `Db::open` before ，
//! not （ need ）。
//! - `core::model_rules`： not need need 。 is `config::current().raw()` ****
//! （`ModelRules::from_raw`）， `current()` —— config after 
//! 。
//!
//! ── and and failure ──────────────────────────────────────────────
//! in `spawn_blocking` ： is file IO（ old can can MB） and 
//! 1 ， not this async worker。 when use 1 
//! （ etc ， not will ， will 1 file、 and use 
//! return）。failure not its （ ），failureReason by 
//! ；response**success**， can 。

use std::sync::Mutex;

use axum::extract::State;
use axum::response::Response;
use serde_json::json;

use crate::server::errors;
use crate::server::http::ok_json;
use crate::server::logging;
use crate::server::ServerState;

/// in 。`true` = has 1 in 。
///
/// use `std::sync::Mutex` not is tokio ： has 1 ，
/// no has ； （） in 。
static RUNNING: Mutex<bool> = Mutex::new(false);

/// 「」 can ； already in when return `false`（ use to 409）。
fn begin_run() -> bool {
    match RUNNING.lock() {
        Ok(mut guard) => {
            if *guard {
                return false;
            }
            *guard = true;
            true
        }
        // in （ panic）： use —— and store guard()
        // 1 to ， this no has will panic in 
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            if *guard {
                return false;
            }
            *guard = true;
            true
        }
    }
}

/// 「」 can 。**must** in path up use （ use guard panic
/// allpath；release is panic=abort， no has unwind need ）。
fn end_run() {
    match RUNNING.lock() {
        Ok(mut guard) => *guard = false,
        Err(poisoned) => *poisoned.into_inner() = false,
    }
}

/// GET /api/upgrade —— `{ pending, items }`。
///
/// `items` is can （ `["config","",""]`），
/// 「 will need 」。`pending` as false when `items` as empty。
pub async fn get_upgrade(State(state): State<ServerState>) -> Response {
    let pending = state.upgrade_pending();
    let items = if pending { state.upgrade_items() } else { Vec::new() };
    ok_json(json!({ "pending": pending, "items": items }))
}

/// POST /api/upgrade/run —— 1 old 。
///
/// return `{ outcomes: [{label, imported}], pending }`：`outcomes` ** 
/// ** （ no has old file、 or already has not in ， that is ）；
/// `pending` is after —— as true has file no （ 1 failure
/// and old file in ）， use 。
///
/// etc ： use （ has etc ）， not 「 1 」 
/// ； 1 when 1 （ and will file， and return will ），
/// to 409。
pub async fn run_upgrade(State(state): State<ServerState>) -> Response {
    let Some(db) = state.db().cloned() else {
        // unavailable： no has can 。 not is 「 0 」——
        // after will use as success， no （ and bootstrap 
        // 「unavailable not 」 1 ）。
        return errors::management_error(500, "数据库不可用，无法导入旧数据（详见启动日志）");
    };
    if !begin_run() {
        return errors::management_error(409, "数据升级正在执行中，请稍候");
    }

    // ： MB old file + 1 + 
    // /（ need ）， not this async worker（ and before `relocate` 
    // 1 ）。
    let config_dir = state.config_dir.clone();
    let store = state.store().clone();
    let request_stats = state.request_stats();
    let result = tokio::task::spawn_blocking(move || {
        let outcomes =
            db.with(|conn| crate::server::db::migrate::run_legacy_migrations(conn, &config_dir));
        // has after （ 1 no when config and no ）
        if matches!(outcomes, Some(ref list) if !list.is_empty()) {
            // configmust： config after ， is （module header）。
            crate::server::config::reload();
            // must** 1 **（ `account_bootstrap` 
            // module header）： when （ that when `accounts` empty，
            // will `import_accounts` etc up ）。 in already 
            // 、 before new —— not this 1 down ， use to is 1 no 
            // 、 no provider、 no old （ need etc down 
            // ）。
            //
            // ：must in `config::reload()` ** after ** —— its in 
            // `migrate_cline_split` `modelRules`， that config is this 
            // from old `config.json` ， no not to 。
            crate::server::account_bootstrap::run(&store);
            // statistics （ when upgrade_pending ，
            // `RequestStats::remap_model_dimension_once` use when ）：
            // old ， by request， not 
            // will 1 。
            request_stats.remap_model_dimension_once();
            request_stats.backfill_source_dimensions();
            request_stats.prune();
            if let Some(logs) = logging::store_ref() {
                logs.prune();
            }
        }
        outcomes
    })
    .await;
    end_run();

    let outcomes = match result {
        Ok(Some(outcomes)) => outcomes,
        // `Db::with` return None = in （ panic ）。
        // can can to ， —— this 。
        Ok(None) => return errors::management_error(500, "数据库连接不可用，迁移未执行"),
        Err(error) => return errors::management_error(500, format!("迁移任务执行失败: {error}")),
    };

    let payload: Vec<_> = outcomes
        .iter()
        .map(|item| json!({ "label": item.label, "imported": item.imported }))
        .collect();
    // ： this in ** already after **（ use by when use ），
    // `logging::log` —— use after can in 「」 this 
    // （ and can not ， `db::migrate` module header）。
    if outcomes.is_empty() {
        logging::log("[Storage]", "数据升级：没有可导入的旧数据（可能已升级过）");
    } else {
        for item in &outcomes {
            logging::log(
                "[Storage]",
                &format!("✅ {} 旧数据已导入数据库（{} 条）", item.label, item.imported),
            );
        }
    }

    let pending = state.refresh_upgrade_pending();
    ok_json(json!({ "outcomes": payload, "pending": pending }))
}
