//! `{config_dir}/accounts.json` → `accounts` （+ `kv.priorityScope`）。
//!
//! （、 etc 、failure） in down ；
//! then （ etc 、 not 、 1 ） [`super`] module header。

use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

use super::backup::backup_legacy_file;
use super::LegacyOutcome;
use crate::server::core::account_store::state::StoredAccount;
use crate::server::logging;

/// old file（`{config_dir}/accounts.json`）。
///
/// in file 1 not is `account_store` ： that 
/// already ** not in **（ not has " file"）， is 
/// 「 before that file」—— is ， not will 。
/// ， this **** not is ， in this not 。
const ACCOUNTS_FILE_NAME: &str = "accounts.json";

/// can （`LegacyOutcome.label` and 「」 use 1 ）。
pub(super) const LABEL: &str = "账号";

/// old file （ not in when `None`）。
///
/// and 「 has no has old file」** use this 1 **：
/// 1 ， for 「 old file in 」 （ 1 1 ，
/// is 「 has 」 not ）。
pub(super) fn legacy_file(dir: &Path) -> Option<PathBuf> {
    let path = dir.join(ACCOUNTS_FILE_NAME);
    path.is_file().then_some(path)
}

/// old file `{config_dir}/accounts.json` → `accounts` （+ `kv.priorityScope`）。
///
/// file（ `core::account_store::state::AccountState` ）：
/// `{ accounts: [ {id, provider, priority, enabled, addedAt, …}, … ],
/// priorityScope: "global" }`。 this ， its （ old 
/// `currentAccountId` ）**** —— that is `AccountState` has 
/// （file when not ）， 1 " 1 " need ： after 
/// 1 not ， will down 1 has when is not is has use 。
///
/// ── ────────────────────────────────────────────
/// `data` is ** JSON **（ 1 not ， `sql.rs` module header），
/// from ， and 1 `StoredAccount` **** not is JSON
/// —— is 「 `enabled: "false"` 」 this in is 
/// （`enabled()` → 1）， and after 1 。
/// this is must use `from_value` / `provider()` / `priority()` etc Reason 
/// 1 ：** and is 1 then **，otherwise 1 
/// and add， can can not 1 。
///
/// ── etc ────────────────────────────────────────────────────
/// empty（module header then ）。 this is 「 use in already has 
/// ， old file not this 1 」—— none that is is after add 。
/// Notecheck in ** before **：empty when old file not （ not 、 not delete），
/// use can in configdirectory to （ this is 1 " no " can ）。
///
/// ── failure and error ──────────────────────────────────────────
/// all failure 1 and return `None`（module header 「 not 」 then ）：
/// old file not 、JSON 、 not is for 、failure —— no has 1 this 
/// not （ store when is old file， can ）。 in ** 1 **
/// （`unchecked_transaction`， by module headerNote），「 to 1 
/// failure」 not will down —— need empty、 need ， down 「」 can 。
///
/// ── etc ：Why** not can **「empty」（T11 after need need 1 ）──
/// module header then is 「empty」， that in ** in **
/// when is for （ that when can can by ）。 T11 ** use in 「」**
/// ， is in use after —— use can can in that before 
/// （ already has ）， when 「empty」 will 「 already 」，
/// that `accounts.json` ** not **（ old file not ，
/// 1 「 has can 」）。`logs` already up 
/// this ， is 1 。
///
/// ** by etc **：、， use `INSERT OR IGNORE`
/// （ [`write_accounts`]）—— already in id not ， new 。
/// for ：
/// - ****： id and use （ when uuid） not ，
/// ；
/// - ****： old file id already in → IGNORE， not 。
///
/// 「 already in not 」 has 1 **** up ： that can can is use after 
/// （、、）， old file will this ——
/// ignore is need 。
pub(super) fn import_accounts(conn: &Connection, dir: &Path) -> Option<LegacyOutcome> {
    let path = legacy_file(dir)?;

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 账号迁移失败：无法读取 {}（{error}）", path.display()),
            );
            return None;
        }
    };
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 账号迁移失败：{} 不是合法 JSON（{error}）", path.display()),
            );
            return None;
        }
    };
    // must is for （/）， and old path 1 
    if !value.is_object() {
        logging::console_line(
            "[Storage]",
            &format!("❌ 账号迁移失败：{} 的顶层不是对象", path.display()),
        );
        return None;
    }

    // ： and file when `filter_map(from_value)` 1 
    // （ id、id 、id empty 1 ）。、 not 
    // —— and `decode_data` 1 。
    let records: Vec<StoredAccount> = value
        .get("accounts")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .cloned()
                .filter_map(StoredAccount::from_value)
                .collect()
        })
        .unwrap_or_default();
    let priority_scope = value
        .get("priorityScope")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    let (imported, skipped) = match write_accounts(conn, &records, priority_scope.as_deref()) {
        Ok(result) => result,
        Err(error) => {
            logging::console_line(
                "[Storage]",
                &format!("❌ 账号迁移失败：写入数据库失败（{error}）"),
            );
            return None;
        }
    };
    if skipped > 0 {
        logging::console_line(
            "[Storage]",
            &format!("账号迁移：{imported} 条新导入、{skipped} 条已存在（重复导入自动跳过）"),
        );
    }
    // success after （failure when file， down 「」 can ）。
    // Note is **** file to this ： that this file is empty 、
    // no has can —— is for （otherwise will new 1 
    // this emptyfile）， `imported: 0` will " 0 "。
    let backup = backup_legacy_file(&path);
    Some(LegacyOutcome {
        label: LABEL,
        source: path,
        imported: records.len(),
        backup,
    })
}

// this has 1 `count_accounts`（「empty」 etc ）。** already delete**：
// T11 use 「」 after ， that not —— use can can in before 
// ， is 「 already 」， old file not 
// （`logs` already up ）。 in `INSERT OR IGNORE` by etc ，
// not need need 「empty not empty」 this —— `import_accounts` etc 。

/// 1 + `priorityScope` （** 1 **）。
///
/// Why not use `account_store::sql::save_state`： that is 「 by 」 
/// use ， is `&mut Connection`（ need ）。 has 
/// `&Connection`（ ，module header）， use `unchecked_transaction`
/// —— this is module headerNote that 。
/// SQL and `save_state` 1 （ 1 1 ）。
///
/// ── Why is `INSERT OR IGNORE` not is `INSERT` ─────────────
/// etc （ by `import_accounts` etc ）： use 「」 when can can 
/// already has （ in this before ）， that id and old file not when and 、
/// when ** that **（ can can use after —— 、、
/// ）， not can 、 not can 。
///
/// return `( new , already in )`： use use after this 
/// is no ， and 。
fn write_accounts(
    conn: &Connection,
    records: &[StoredAccount],
    priority_scope: Option<&str>,
) -> rusqlite::Result<(usize, usize)> {
    let tx = conn.unchecked_transaction()?;
    let mut imported = 0usize;
    let mut skipped = 0usize;
    for record in records {
        // 1 （ `import_accounts` ）：
        // provider / as empty → default provider；priority → ；
        // enabled bool false → 1；added_at not to → 0（ for schema DEFAULT）
        let changed = tx.execute(
            "INSERT OR IGNORE INTO accounts (id, provider, priority, enabled, added_at, data)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.id(),
                record.provider(),
                record.priority(),
                if record.enabled() { 1 } else { 0 },
                record.added_at(),
                record.to_value().to_string(),
            ],
        )?;
        if changed > 0 {
            imported += 1;
        } else {
            skipped += 1;
        }
    }
    // `priorityScope` has file** has **： no has 「」，
    // that （`migrate_startup`） is 「 not to this 」 need not need 。
    // this if none 1 default， that not will —— use 
    // will in after 。
    //
    // use `DO NOTHING` not is `DO UPDATE`（ and before `DO UPDATE` not ）：
    // already has this when ， can can is down （`migrate_startup` already ），
    // old file will 。 old file in ** not in ** when has 。
    if let Some(scope) = priority_scope {
        tx.execute(
            "INSERT INTO kv (key, value) VALUES ('priorityScope', ?1)
             ON CONFLICT(key) DO NOTHING",
            params![scope],
        )?;
    }
    tx.commit()?;
    Ok((imported, skipped))
}
