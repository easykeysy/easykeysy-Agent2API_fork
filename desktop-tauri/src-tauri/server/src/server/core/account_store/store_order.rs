//! （）： 「 + 」 `priority`。
//!
//! ── Why need need 1 「」 ──────────────────────────
//! by after ， not by use ， is ** **：
//! = ， = 。 1 is 1 
//! （ → ）， `priority` must 1 （ not 2），client 
//! up 、 by this by —— not is client PATCH 
//! （ that in failure will down /， 1 ）。
//!
//! ── and `move_account`（） ──────────────────────
//! that old ↑/↓ ： 。 down not use （ by 
//! ）， —— client/ in use ， no 。 new 1 this 。
//!
//! ── ： or can 1 ────────────────────
//! not to ， can when 。
//! can 、 or 400，and 1 not 。

use std::collections::HashSet;

use serde_json::{json, Value};

use crate::server::core::account_store::sql;
use crate::server::core::account_store::store_view::{account_visible, Visibility};
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};

impl AccountStore {
    /// by to ：`ids` is new （ from to ）。
    ///
    /// return `{ ok, changed, list }`：`changed` is （ before 
    /// need not need 「 already 」），`list` is after （ `GET /api/accounts`，
    /// and `move_account` / `promote_to_front` 1 ）。
    ///
    /// or can ； after can ， not 。
    /// by use ， by `priority` 。
    pub fn reorder_accounts(&self, ids: &[String]) -> Result<Value, AccountStoreError> {
        let guard = self.guard();
        let state = self.load(&guard);

        let known: HashSet<&str> = state.accounts.iter().map(|record| record.id()).collect();
        let visible: HashSet<&str> = state.accounts.iter()
            .filter(|record| account_visible(&record.provider()))
            .map(|record| record.id()).collect();
        let mut seen: HashSet<&str> = HashSet::with_capacity(ids.len());
        for id in ids {
            let id = id.as_str();
            if !known.contains(id) {
                return Err(AccountStoreError::bad_request(format!(
                    "队列里出现了不存在的账号：{id}"
                )));
            }
            if !seen.insert(id) {
                return Err(AccountStoreError::bad_request(format!(
                    "队列里有重复的账号：{id}"
                )));
            }
        }
        let ordered_ids = if seen.len() == known.len() {
            ids.to_vec()
        } else if seen == visible {
            let mut records: Vec<_> = state.accounts.iter().collect();
            records.sort_by_key(|record| record.order_key());
            let mut replacements = ids.iter();
            records.into_iter().map(|record| {
                if visible.contains(record.id()) {
                    replacements.next().cloned().unwrap_or_else(|| record.id().to_string())
                } else {
                    record.id().to_string()
                }
            }).collect()
        } else {
            return Err(AccountStoreError::bad_request(format!(
                "队列不完整：可见账号 {} 个，全部账号 {} 个，队列里只有 {} 个",
                visible.len(), known.len(), seen.len()
            )));
        };

        // by to （0、1、2…）； （ `sql::apply_priorities`）
        let updates: Vec<(String, i64)> = ordered_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.clone(), index as i64))
            .collect();
        let changed = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction()?;
            let changed = sql::apply_priorities(&tx, &updates)?;
            tx.commit()?;
            Ok(changed)
        })?;

        // use ** after ** ： 1 （ and ），
        // not 1 
        let mut state = state;
        for record in state.accounts.iter_mut() {
            if let Some(index) = ordered_ids.iter().position(|id| id == record.id()) {
                record.set_priority(index as i64);
            }
        }
        Ok(json!({
            "ok": true,
            "changed": changed,
            "list": self.snapshot(&state, Visibility::UserFacing),
        }))
    }
}
