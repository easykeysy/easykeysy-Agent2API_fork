//! ** SQL ** —— 1 SQL 。
//!
//! ── Why 1 ──────────────────────────────────────────
//! before in `{config_dir}/accounts.json`： is 「 JSON →
//! 1 → new 」。 not ，「 1 need file」
//! ： need 、 1 failure is 、 and need 
//! JSON 1 （ rateLimits this ）。
//!
//! to SQLite after ， this ** and that 1 **。file is this ：
//! 「 1 ↔ `StoredAccount`」 、 and by need in in this ，
//! up （`store_*`、 provider ） not SQL。
//!
//! ── `data` is ， is ───────────────────────────
//! （ `server/db/schema.rs`） `data` ** JSON **，
//! this is 「 use not can 」 this not —— unknown、
//! use 、 old 。`id` / `provider` / `priority` / `enabled` /
//! `added_at` is from JSON **** （、、 1 use ）。
//!
//! by 1 then ：** when `data` as **， not 。
//! file 1 `record.provider()` / `record.priority()` / `record.enabled()`
//! / `record.added_at()` this ****（ not is JSON ）， is 
//! 「 `enabled: "false"` 」 this in and is 1 ：
//! is ** **， not is 。 when —— 
//! `data`， not and （`decode_data` data 1 ）。
//!
//! ── `data` in is ****（ ）─────────────────
//! `data` not is that JSON ， is `core::crypto` `enc1:` 
//! —— `AES-256-GCM` after base64（nonce 、 1 to not ）。
//! to file 1 for ： [`encode_data`]（`StoredAccount` → ），
//! [`decode_data`]（ → `StoredAccount`）。** has file `data` is **。
//!
//! not ， this file when not need ：
//! 1. ** is **。`state.rs` `StoredAccount::to_value` / `from_value`
//! 1 ， provider `record.to_value()` 
//! （`Credentials::from_payload(&record.to_value())`）。 in 「」
//! this 1 ， not 。
//! 2. ** in **（`{config_dir}/account.key`，
//! `AGENT2API_ACCOUNT_KEY`， `core::crypto` module header）。 this is 「file 
//! / / 」 this 1 —— `agent2api.db` not to ；
//! not to `config_dir` or （ `core::crypto`）。
//! 3. **`decode_data` **。 before is JSON，（
//! [`encrypt_legacy_rows`]） ； can can failure、 can can in ，
//! must「 `enc1:` 、 not is JSON 」——otherwise 1 
//! no etc 。
//! failure（file not etc ）** not panic**（file ），
//! **failure and up error**（fail closed）： is 「 when not to 」，
//! in this etc ， [`encode_data`] return
//! `Result`、 use `?` up —— use to is 「failure」， not is 「success
//! is 」。（ up 3 ）， this not 。
//!
//! ── ：`ORDER BY rowid` ──────────────────────────────────
//! `list_accounts` is 「 by **file**return， by 」。
//! no has 「 down 」 this 1 （ not this as 1 ： down is file ，
//! not is ）， use SQLite `rowid` as ， and old 
//! accounts.json in down for ：
//! - ****： by INSERT → rowid == ；
//! - **add/**： old is `retain(≠id)` + `push`（ to ），
//! this [`put`] is `DELETE` + `INSERT`（ to new rowid）→ to ；
//! - ****： old in down up （ not ）， this 
//! [`update_in_place`] is `UPDATE`（rowid not ）→ not 。
//! 1 ** has **：`promote_to_front` / `migrate_startup` in old will 
//! by after （file）， this UPDATE
//! （rowid not ）、 not 。
//! ****：`migrate_startup` Cline will `id`（）， that in 
//! `save_state` 「 old id + new id」 ， is ** will ** to ——
//! and old 「」 not ，none can （ by down ）。
//! up this not will use to —— （、）
//! by （`accounts-view.js:144` and `accounts-groups.js:351` 
//! `byPriorityOrder`、`routing::pick_account_by_priority` `sort_by`），
//! ** no has **（ not by 、 not by down ）。
//!
//! ── ──────────────────────────────────────────────────
//! file not request、 not （ by use `AccountStore::guard` ），
//! not panic：all SQL failure `rusqlite::Result` up ， by use 
//! `AccountStoreError`（500）。

use rusqlite::{params, Connection, OptionalExtension};

use crate::server::core::account_store::priority::{grouped_queue_order, QueueEntry};
use crate::server::core::account_store::state::{AccountState, StoredAccount};
use crate::server::core::crypto::{self, SecretScope};

/// 1 `data` not **Reason**。
///
/// ── Why need Reason ──────────────────────────────────────
/// from before this has 1 `None`， is 「file no 」 and 「 1 JSON
/// 」 in use is 1 ， as 「 this not in 」 `filter_map`
/// 。 use to is 「 / empty」， not Reason —— 
/// after not is this （ `db::backup` 1 ）。
///
/// Reason for ** not **，must in is 1 ：
/// · [`Key`](Self::Key)：`account.key` or not is that 1 —— file
/// 1 can （ no ）；
/// · [`Json`](Self::Json)： not is JSON —— this 1 ；
/// · [`Shape`](Self::Shape)：JSON not is for （ `id` etc ）——
/// not is ， or bug。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecodeError {
    /// not ： / not ， or 
    Key,
    /// （ or ） not is JSON
    Json,
    /// JSON not is 1 
    Shape,
}

impl DecodeError {
    /// to use 1 Reason（ use can ， need can ）
    pub(crate) fn describe(self) -> &'static str {
        match self {
            Self::Key => "账号加密密钥不可用或与库不匹配（account.key 缺失 / 被换过）",
            Self::Json => "记录内容不是合法 JSON",
            Self::Shape => "记录不是一条账号（缺少 id 等必需字段）",
        }
    }
}

/// 1 `data` → JSON `Value`（** and can **）。
///
/// this is 「`data` ↔ 」 this ，[`decode_data`] and 
/// [`save_state`] use 。：
/// - is （`crypto::is_envelope`， has `core::crypto` 1 ）→ 
/// `core::crypto::open` （ already is UTF-8）， JSON ；
/// - its → by **** JSON （ ，module header）。
/// failurereturn**Reason error**（ [`DecodeError`]）， by use is up ——
/// this not ： 1 failureReason can can 20 seconds 1 ， up will 。
fn decode_value(text: &str) -> Result<serde_json::Value, DecodeError> {
    if crypto::is_envelope(text) {
        let plaintext = crypto::open(text, SecretScope::Accounts).map_err(|_| DecodeError::Key)?;
        serde_json::from_slice(&plaintext).map_err(|_| DecodeError::Json)
    } else {
        serde_json::from_str(text).map_err(|_| DecodeError::Json)
    }
}

/// 1 `data` → `StoredAccount`； not or not is for / id when returnReason error。
///
/// this not （ and old filepath 1 ： that when is 
/// `filter_map(from_value)` ， not ）—— is 
/// ** not is **： use to error after by Reason up （ [`warn_undecodable`]）。
pub(crate) fn decode_data(text: &str) -> Result<StoredAccount, DecodeError> {
    let value = decode_value(text)?;
    StoredAccount::from_value(value).ok_or(DecodeError::Shape)
}

/// up ： 1 `( up down , need )` in `THROTTLE` 1 。
///
/// is ： will new 1 —— that when use file，
/// need need to new 。`Mutex` in （ panic） when up ：
/// not not this 。
fn throttled_warn(context: &str, subject: &str, text: &str) {
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, Instant};
    const THROTTLE: Duration = Duration::from_secs(600);

    static LAST: OnceLock<Mutex<Vec<(String, String, Instant)>>> = OnceLock::new();
    let now = Instant::now();
    let Ok(mut last) = LAST.get_or_init(|| Mutex::new(Vec::new())).lock() else {
        return;
    };
    if let Some((_, _, at)) = last
        .iter()
        .find(|entry| entry.0 == context && entry.1 == subject)
    {
        if now.duration_since(*at) < THROTTLE {
            return;
        }
    }
    last.retain(|entry| !(entry.0 == context && entry.1 == subject));
    last.push((context.to_string(), subject.to_string(), now));
    drop(last);
    crate::server::logging::log("[Accounts]", text);
}

/// 1 「 not 」 by Reason** 1 **。
///
/// ── Why need not is ────────────────────────────────
/// when ， 20 seconds 1 。 not when 
/// not ， is —— will this ，
/// has use 。 this by Reason， 1 1 。
fn warn_undecodable(context: &str, errors: &[DecodeError]) {
    // by Reason（Reason， not use HashMap）
    for kind in [DecodeError::Key, DecodeError::Json, DecodeError::Shape] {
        let count = errors.iter().filter(|item| **item == kind).count();
        if count == 0 {
            continue;
        }
        throttled_warn(
            context,
            kind.describe(),
            &format!(
                "{context}：{count} 条账号记录读不出来，已跳过 —— {}。\
                 数据仍在库里，修好之后重新读取即可恢复（不需要重新导入）。",
                kind.describe()
            ),
        );
    }
}

/// failure up （`load_by_id` error already in up Reason）。
///
/// 1 not is use `throttled_warn`：path up use 
/// 「 this not this not this 1 」， not this 。
pub(crate) fn warn_load_failure(context: &str, message: &str) {
    throttled_warn(
        context,
        message,
        &format!("{context}失败：{message}（数据仍在库里，修好之后重新读取即可恢复）"),
    );
}

/// `StoredAccount` → 1 `data` （** `enc1:` **）。
///
/// `to_value().to_string()` to that JSON （ and before 
/// is ** 1 **，「JSON is 、unknown」 this not 
/// 1 ）， UTF-8 。 is 、 has this 1 down 
/// ，module header 1 not 。
///
/// ── failureWhy** up ** not is ────────────────────
/// is 「file when not to 」，failure in this 
/// （`config_dir` not can 、file not etc ）。 when if ，
/// etc 「 this 」， use from up not （ before that verbose default
/// not ）。**failure this failure**（fail closed）： use （`store_*` →
/// API） error， use to is 「failure」 not is 「success is 」。
/// error use `ToSqlConversionFailure` （ has ， its `Display` is that 
/// ， `db::migrate::logs::assign_log_id` ）。
///
/// `save_state` is 「、」， has ** need ** will to this —— no 
/// not will encode， not will as failure（ this as has ）。
fn encode_data(record: &StoredAccount) -> rusqlite::Result<String> {
    let json = record.to_value().to_string();
    crypto::seal(json.as_bytes(), SecretScope::Accounts).map_err(|error| {
        rusqlite::Error::ToSqlConversionFailure(
            format!("账号凭证加密失败（{error}）：为避免把凭证明文落库，本次保存已中止")
                .into(),
        )
    })
}

/// by 1 （ not in return None）。path 1 。
///
/// not when ** up ** not is return `None`： this use is 「 this 
/// 」 path（、、、）， 「 not 」「 not in 」
/// will 「 new 1 」 —— that etc use 1 empty not ，
/// is not can 。error to Reason（ [`DecodeError::describe`]），
/// use can to 「 not 」 not is 1 「 not in 」。
pub(crate) fn load_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<StoredAccount>> {
    let mut stmt = conn.prepare("SELECT data FROM accounts WHERE id = ?1")?;
    let mut rows = stmt.query(params![id])?;
    while let Some(row) = rows.next()? {
        let text: String = row.get(0)?;
        return match decode_data(&text) {
            Ok(record) => Ok(Some(record)),
            Err(error) => Err(rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                format!("账号 {id} 读不出来：{}", error.describe()).into(),
            )),
        };
    }
    Ok(None)
}

/// provider （ by ）。
///
/// use `provider` ： is when from `record.provider()` ，
/// `provider()` for /empty/ 1 default provider ——
/// 「 no provider 」 this in already is `workbuddy`，
/// by `workbuddy` to ， and old in 1 。
pub(crate) fn load_by_provider(
    conn: &Connection,
    provider: &str,
) -> rusqlite::Result<Vec<StoredAccount>> {
    let mut stmt =
        conn.prepare("SELECT data FROM accounts WHERE provider = ?1 ORDER BY rowid")?;
    let mut rows = stmt.query(params![provider])?;
    let mut records = Vec::new();
    let mut errors = Vec::new();
    while let Some(row) = rows.next()? {
        let text: String = row.get(0)?;
        match decode_data(&text) {
            Ok(record) => records.push(record),
            Err(error) => errors.push(error),
        }
    }
    warn_undecodable(&format!("读取 {provider} 的账号"), &errors);
    Ok(records)
}

/// total —— statistics and use （empty before ）。
pub(crate) fn count_all(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
}

/// —— `next_free_priority` 。
///
/// and [`priorities_except`] in 「 need not need excluding id」：add when 
/// need excluding that id not in （ new id ）， use this ；update has 
/// when need excluding in ， use 1 。
pub(crate) fn priorities_all(conn: &Connection) -> rusqlite::Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT priority FROM accounts")?;
    let mut rows = stmt.query([])?;
    let mut values = Vec::new();
    while let Some(row) = rows.next()? {
        values.push(row.get(0)?);
    }
    Ok(values)
}

/// 「 id 」 —— `next_free_priority` 。
///
/// 1 ： is ， no need 20 JSON（
/// `rateLimits` this ）。 [`priority_holder`]。
pub(crate) fn priorities_except(conn: &Connection, id: &str) -> rusqlite::Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT priority FROM accounts WHERE id <> ?1")?;
    let mut rows = stmt.query(params![id])?;
    let mut values = Vec::new();
    while let Some(row) = rows.next()? {
        values.push(row.get(0)?);
    }
    Ok(values)
}

/// use （excluding `exclude_id` ）， no has then None。
///
/// for old `find_priority_holder(&peers, priority, None)`：`peers` is 
/// 「 」， is ** 1 after etc **。 `priority`
/// is `normalize_priority_value(record.priority())`， SQL 
/// `priority = ?` and etc ， not need need 1 。
///
/// `ORDER BY rowid LIMIT 1`： 1 （ ） when ，
/// before that —— is old in to 1 ，
/// is 409 「」 and before 1 。
pub(crate) fn priority_holder(
    conn: &Connection,
    priority: i64,
    exclude_id: &str,
) -> rusqlite::Result<Option<String>> {
    let text: Option<String> = conn
        .query_row(
            "SELECT data FROM accounts WHERE priority = ?1 AND id <> ?2 ORDER BY rowid LIMIT 1",
            params![priority, exclude_id],
            |row| row.get(0),
        )
        .optional()?;
    // not 「 no has 」： this use 409 ，
    // not is （ `priorities_*` ）。 as 1 
    // failure is —— this error， not 
    // （ will by Reason 1 up ， not in this ）。
    Ok(text
        .as_deref()
        .and_then(|text| decode_data(text).ok())
        .map(|record| record.name()))
}

/* ─── （ ）────────── */

/// 1 「」： need need （ `priority::QueueEntry`）。
pub(crate) struct QueueRow {
    pub id: String,
    pub provider: String,
    pub priority: i64,
    pub added_at: i64,
}

/// （id / provider / priority / added_at ）。
///
/// ， not `data`： is + ， no need 
/// JSON（ rateLimits this ） 1 —— and `priorities_all` 1 to 。
/// `added_at` use + `COALESCE(…, 0)`： and JSON `addedAt` is 1 
/// （ when by `record.added_at()` ）， when and when after ， not 。
pub(crate) fn queue_rows(conn: &Connection) -> rusqlite::Result<Vec<QueueRow>> {
    let mut stmt = conn
        .prepare("SELECT id, provider, priority, COALESCE(added_at, 0) FROM accounts")?;
    let mut rows = stmt.query([])?;
    let mut values = Vec::new();
    while let Some(row) = rows.next()? {
        values.push(QueueRow {
            id: row.get(0)?,
            provider: row.get(1)?,
            priority: row.get(2)?,
            added_at: row.get(3)?,
        });
    }
    Ok(values)
}

/// by `(id, new )` ：** and JSON priority 1 **。
///
/// ：
/// · （ and `save_state` 1 to ）—— 1 
/// has 1 ；
/// · ** not `updatedAt`**： not is 「」， up 「update」 is 
/// / after 1 when （`move_account` that 1 as 
/// will ， if ， 1 will 「update」）。
pub(crate) fn apply_priorities(
    conn: &Connection,
    updates: &[(String, i64)],
) -> rusqlite::Result<usize> {
    let mut changed = 0usize;
    for (id, priority) in updates {
        let data: Option<String> = conn
            .query_row("SELECT data FROM accounts WHERE id = ?1", params![id], |row| {
                row.get(0)
            })
            .optional()?;
        let Some(record) = data.as_deref().and_then(|text| decode_data(text).ok()) else {
            continue;
        };
        if record.priority() == *priority {
            continue;
        }
        let mut record = record;
        record.set_priority(*priority);
        update_in_place(conn, &record)?;
        changed += 1;
    }
    Ok(changed)
}

/// new is （ need need ）。
///
/// use to is 「 + 1」， has 1 ：** that 1 is not is 
/// 1 **。 is 1 → in ，none need ； not is → 。
pub(crate) fn queue_needs_normalize(
    conn: &Connection,
    provider: &str,
    id: &str,
) -> rusqlite::Result<bool> {
    let last_provider: Option<String> = conn
        .query_row(
            "SELECT provider FROM accounts WHERE id <> ?1 ORDER BY priority DESC, rowid DESC LIMIT 1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(matches!(last_provider.as_deref(), Some(last) if last != provider))
}

/// 「 + 0、1、2…」（ then `priority::grouped_queue_order`）。
///
/// used for：add after （[`put`] that 1 ）， and `AccountStore::reorder_accounts`
/// 。 is need （ by ），**** 。
pub(crate) fn normalize_grouped_queue(conn: &Connection) -> rusqlite::Result<usize> {
    let rows = queue_rows(conn)?;
    let entries: Vec<QueueEntry<'_>> = rows
        .iter()
        .map(|row| QueueEntry {
            id: &row.id,
            provider: &row.provider,
            priority: row.priority,
            added_at: row.added_at,
        })
        .collect();
    let ordered = grouped_queue_order(&entries);
    let updates: Vec<(String, i64)> = ordered
        .into_iter()
        .enumerate()
        .map(|(index, id)| (id, index as i64))
        .collect();
    apply_priorities(conn, &updates)
}

/// （upsert） 1 ，** and to **。
///
/// and old `state.accounts.retain(≠id); push(record)` for ：
/// id old 、 new → new to current rowid， to 。
/// all「/update」 path use （workbuddy / / CatPaw / AutoClaw /
/// Cline / Qoder and ），"update has in after "
/// this has as 。
///
/// Why not use `INSERT OR REPLACE`： its will 「 old + new 」（ to ），
/// that in SQLite REPLACE ， when not ； 
/// `DELETE` + `INSERT` this in up ， after has `INSERT OR IGNORE`
/// as as not 。
///
/// ── add that 1 need in ** **（）────────────────
/// use to new is 「 + 1」， that is 「 in after 」。
/// down 1 in must（ `priority::grouped_queue_order`）， is ：
/// new if not **current in after that 1 **， —— up 
/// in ， in that 1 after ， for not up 。 this 1 
/// （[`normalize_grouped_queue`]）： after in （ is 
/// ）， not 。
///
/// in ** need need ** when （ [`queue_needs_normalize`]）： to that 1 
/// 、 and 1 1 ， in not to this ，。
///
/// 1 already ： use if `priority`（`POST /api/accounts` 
/// `payload.priority`，client has ）， will that **** ——
/// down 「 in in 」 not is 1 has request（ up no has this ），
/// in is 1 。
pub(crate) fn put(conn: &Connection, record: &StoredAccount) -> rusqlite::Result<()> {
    let existed = delete(conn, record.id())?;
    insert_row(conn, record)?;
    if !existed && queue_needs_normalize(conn, &record.provider(), record.id())? {
        normalize_grouped_queue(conn)?;
    }
    Ok(())
}

/// update 1 （** not **）。
///
/// and [`put`] in ：`update_account`（//enabled/）、
/// `move_account`（ ）、、token this 「
/// 」 this —— old in down up ， not ，
/// up （） not this as 1 「」。
///
/// not in when （ to ）： that is 「 after and 」 
/// —— old in that is 「 file」， is 
/// this new 。 this 1 （ not is this ），
/// as use already 「success」 before down 。
pub(crate) fn update_in_place(conn: &Connection, record: &StoredAccount) -> rusqlite::Result<()> {
    let changed = conn.execute(
        "UPDATE accounts SET provider = ?2, priority = ?3, enabled = ?4, added_at = ?5, data = ?6
         WHERE id = ?1",
        params![
            record.id(),
            record.provider(),
            record.priority(),
            enabled_flag(record),
            record.added_at(),
            encode_data(record)?,
        ],
    )?;
    if changed == 0 {
        insert_row(conn, record)?;
    }
    Ok(())
}

/// delete 1 ，return is （`false` = no has this 1 ）。
pub(crate) fn delete(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])? > 0)
}

/// 1 INSERT（[`put`] / [`update_in_place`] ）
fn insert_row(conn: &Connection, record: &StoredAccount) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO accounts (id, provider, priority, enabled, added_at, data)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            record.id(),
            record.provider(),
            record.priority(),
            enabled_flag(record),
            record.added_at(),
            encode_data(record)?,
        ],
    )?;
    Ok(())
}

/// `enabled` ：SQLite no has ， by 0/1。
///
/// is `record.enabled()`（`enabled !== false` ） not is JSON 
/// —— is `enabled: "false"` this in is 1（enabled），
/// and to up to 1 。
fn enabled_flag(record: &StoredAccount) -> i64 {
    if record.enabled() { 1 } else { 0 }
}

/// 1 `AccountState` ** by **（ 1 ）。
///
/// used for「 is 」：`batch_update` / `batch_remove`、
/// `migrate_startup`、 old 、 and `account_transfer` and 
/// （ `save_locked` to this ）。 this path need 
/// （、、）， is ** need ** not is ；
/// **** not this —— this has JSON， and for ：
/// - has 、 no has → INSERT
/// - has JSON not → UPDATE（ 1 and new ）
/// - has 、 no has → DELETE
/// - JSON → ** 1 not **（ this is 「 1 not its 19 」
/// ）
///
/// in 「add」 when （INSERT by `state.accounts` ， is new rowid
/// for and old push 1 ）； has 1 UPDATE， not 。
///
/// Why `&mut Connection`： use need already has can use （`Db::with_mut`），
/// need in 。 by （`conn.transaction()`）， 
/// in —— use not need need 1 。
///
/// `priorityScope` in `kv` （ `server/db/schema.rs` module header）：
/// not 1 ， in `Some` when （ old file is 「 no has this 」
/// ）， [`load_priority_scope`]。
pub(crate) fn save_state(conn: &mut Connection, state: &AccountState) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    // has ：id → data ** after **。 can ： if has 1 
    // （empty、） not will 「」 1 。
    //
    // this **must**（`decode_value`） not is `serde_json::from_str`：
    // · → failure → to None → down INSERT → ；
    // · → GCM nonce 、 1 not →
    // not etc → `save_state` 1 。
    // after `Value` not ： no after and `record.to_value()`
    // etc （`serde_json::Value` by ， and none）， is ** 1 not **；
    // UPDATE。** already and not will 。**
    let mut existing: Vec<(String, Option<serde_json::Value>)> = Vec::new();
    {
        let mut stmt = tx.prepare("SELECT id, data FROM accounts")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let text: String = row.get(1)?;
            existing.push((id, decode_value(&text).ok()));
        }
    }

    // delete： in 、 no has —— **「」 **。
    //
    // `existing` id （total can to ）， `state.accounts` ，
    // that not will `decode_data` 。 is 「 / not / no 
    // `account.key`」 this down ， not → `state.accounts` as **empty** ——
    // if by 「 in 、 no has 」， 1 bulk （`batch_update` /
    // `batch_remove` / and / `migrate_startup` has when ） will **
    // DELETE **。 this is need old ， new in when will 。
    //
    // ：** has `decode_value` success（`value.is_some()`） and 
    // delete**；`None`（ not / ） 1 ** not ** —— not not ，
    // when ， can not can 。
    for (id, value) in &existing {
        if value.is_none() {
            continue;
        }
        if !state.accounts.iter().any(|record| record.id() == *id) {
            tx.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        }
    }
    // add / update
    for record in &state.accounts {
        let current = existing
            .iter()
            .find(|(id, _)| id == record.id())
            .map(|(_, value)| value.as_ref())
            .unwrap_or(None);
        match current {
            Some(value) if *value == record.to_value() => {}
            Some(_) => {
                tx.execute(
                    "UPDATE accounts SET provider = ?2, priority = ?3, enabled = ?4,
                     added_at = ?5, data = ?6 WHERE id = ?1",
                    params![
                        record.id(),
                        record.provider(),
                        record.priority(),
                        enabled_flag(record),
                        record.added_at(),
                        encode_data(record)?,
                    ],
                )?;
            }
            None => {
                tx.execute(
                    "INSERT INTO accounts (id, provider, priority, enabled, added_at, data)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        record.id(),
                        record.provider(),
                        record.priority(),
                        enabled_flag(record),
                        record.added_at(),
                        encode_data(record)?,
                    ],
                )?;
            }
        }
    }

    if let Some(scope) = &state.priority_scope {
        // etc ： no when SQLite will this 1 ， has 1 、 and this path
        // is 「」， not as 1 
        tx.execute(
            "INSERT INTO kv (key, value) VALUES ('priorityScope', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![scope],
        )?;
    }
    tx.commit()
}

/// `priorityScope`（`kv` ； no has this when None）。
///
/// is **** not is JSON ： `server/db/schema.rs`
/// module header（「 its use 」 1 ），`priorityScope` is its in 1 ，
/// is `"global"` —— is use `sqlite3` when 1 can 。
pub(crate) fn load_priority_scope(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM kv WHERE key = 'priorityScope'",
        [],
        |row| row.get(0),
    )
    .optional()
}

/* ─── （ 1 ）────────── */

/// `accounts` ** not is ** `data` ，returnsuccess 。
///
/// ── is ────────────────────────────────────────────
/// `data` is ，** before already in ** is JSON
/// （module header 3 not ）。`decode_data` ， that can ——
/// that is ， not is ： need has ，file。
/// in when 。
///
/// ── etc ： ──────────────────────────────────────
/// is 「 has no has `data` not `enc1:` 」，** not use kv **：
/// is can ， no has ， 1 in 1 、 not 。
/// 1 「 this no has 」 ， will and 「 use 
/// 」 this 。
///
/// ── Why 1 （ and in ）────────────
/// to 1 in will down 「、」 in 。 this 、
/// （ 1 `seal` failure not 、return Err）， after 
/// UPDATE 1 —— is 「 need 、 need 1 not 」， in after 
/// down from 1 （ etc ）。
///
/// failure use `ToSqlConversionFailure` 1 can （ has ，
/// `db::migrate::logs::assign_log_id` ）： this `Display` is that 
/// ，`Db::open` / `with_conn_mut` will 。**failure not **
/// by use （ `store_admin::encrypt_legacy_accounts`）。
pub(crate) fn encrypt_legacy_rows(conn: &mut Connection) -> rusqlite::Result<usize> {
    // ① 「 not is 」 ， not 
    let mut pending: Vec<(String, String)> = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, data FROM accounts")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let text: String = row.get(1)?;
            if !crypto::is_envelope(&text) {
                pending.push((id, text));
            }
        }
    }
    if pending.is_empty() {
        return Ok(0);
    }
    // ② in ： 1 failure（ when 1 ）
    let mut updates: Vec<(String, String)> = Vec::with_capacity(pending.len());
    for (id, plaintext) in pending {
        let envelope = crypto::seal(plaintext.as_bytes(), SecretScope::Accounts)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(error.into()))?;
        updates.push((id, envelope));
    }
    // ③ 1 （ need 、 need not ）
    let tx = conn.transaction()?;
    for (id, envelope) in &updates {
        tx.execute(
            "UPDATE accounts SET data = ?2 WHERE id = ?1",
            params![id, envelope],
        )?;
    }
    tx.commit()?;
    Ok(updates.len())
}
