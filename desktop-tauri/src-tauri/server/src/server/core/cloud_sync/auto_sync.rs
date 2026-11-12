//! ： → and → up 。
//!
//! ── Why ────────────────────────────────────
//! in ：
//! · ****（`accounts` / `account_usage_records` / `checkin_records`）——
//! use SQLite `update_hook` 。 this is can 1 ： not （
//! 、token new 、 to 、）， will 。
//! · **kv **（config、Agent config、 / ）—— ****， not use 。
//!
//! ── kv Why not can use update_hook ─────────────────────────────
//! `kv` when `logsNextId` this **seconds in ** ，must can 
//! 「 is 」。 `update_hook` is in SQLite **** use 
//! （ when already `Db::with` has ）， in 1 —— then 
//! 1 ， then 。 kv this 1 as in **** use 
//! [`notify_change`]：`config::update`（config， for ）、
//! `agents::save_client`、`proxy_pool::write_items`、`subscriptions::write_items`、
//! `proxy_settings::write_raw` 。 is will add 1 「 kv 」
//! when need 1 —— this in `snapshot::is_synced_kv_key` 。
//!
//! ── Why need + limit ────────────────────────────────────
//! 1 「」 will （ + usage），token new can can in seconds
//! 。 up 1 etc 1 N 。 and cc-switch
//! 1 ：** to 1 after etc 1 seconds**， new and ； if not ，
//! etc 10 secondsmust 1 （otherwise will not ）。
//!
//! ── Why up before need ──────────────────────────────
//! can and 「 when 」。 is ：`update_hook` will in **
//! no ** up （e.g. token new ）。 up is KB to 
//! MB request， down can can up 。 up before 
//! 、： and `lastLocalHash` ** and ** no → ， 1 
//! request not 。 this 「 down 」 not will （ down after will ，
//! and 1 → ）。 is ， not is 1 。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::server::core::crypto::{self, SecretScope};
use crate::server::db::Db;
use crate::server::logging;

use super::{settings, snapshot};

/// 1 after etc （ and to not up ）
const DEBOUNCE_MS: u64 = 1_000;
/// down etc ： to must 1 
const MAX_WAIT_MS: u64 = 10_000;

static CHANGE_TX: OnceLock<Sender<()>> = OnceLock::new();
static SUPPRESS_DEPTH: AtomicUsize = AtomicUsize::new(0);

/// ： use not up （ down use use ）。
///
/// use not is ： will if has ， will `drop` before 。
pub(crate) struct SuppressionGuard;

impl SuppressionGuard {
    pub(crate) fn new() -> Self {
        SUPPRESS_DEPTH.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for SuppressionGuard {
    fn drop(&mut self) {
        let _ = SUPPRESS_DEPTH.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
            Some(value.saturating_sub(1))
        });
    }
}

fn is_suppressed() -> bool {
    SUPPRESS_DEPTH.load(Ordering::SeqCst) > 0
}

/// this need not need （ 1 ， and use ）
pub(crate) fn should_trigger_for_table(table: &str) -> bool {
    snapshot::SYNCED_TABLES.contains(&table)
}

/// 1 「」 。：（ already has ） when ——
/// need is 「」 this ， not is 1 。
///
/// ** can in has use **： not IO、 not 、 not 。
pub(crate) fn notify_change() {
    if is_suppressed() {
        return;
    }
    let Some(tx) = CHANGE_TX.get() else {
        return;
    };
    match tx.try_send(()) {
        Ok(()) => {}
        // 1： already has 1 ， and can 
        Err(TrySendError::Full(_)) | Err(TrySendError::Closed(_)) => {}
    }
}

/// SQLite `update_hook` 。
///
/// ── ： this must ────────────────────────────────────
/// in use （ already has ）。 1 + 1 
/// send，** not **、 not （`logging::log` need 1 → ）。
pub(crate) fn on_table_change(table: &str) {
    if should_trigger_for_table(table) {
        notify_change();
    }
}

/// （`bootstrap` 1 ）。
///
/// ── Whyset ────────────────────────────────────
/// use in set after need ， not 。 from 
/// `settings::load()` new （ 1 kv ， is 「 has when 」）。
pub(crate) fn spawn_worker(db: Db) {
    if CHANGE_TX.get().is_some() {
        return;
    }
    let (tx, rx) = channel::<()>(1);
    if CHANGE_TX.set(tx).is_err() {
        return;
    }
    crate::spawn_task(async move {
        run_worker(db, rx).await;
    });
}

async fn run_worker(db: Db, mut rx: Receiver<()>) {
    while rx.recv().await.is_some() {
        let started = Instant::now();
        // ： etc to 「 1 seconds」 or 「total 10 seconds」， to as 
        loop {
            let elapsed = started.elapsed();
            if elapsed >= Duration::from_millis(MAX_WAIT_MS) {
                break;
            }
            let remaining = Duration::from_millis(MAX_WAIT_MS) - elapsed;
            let wait = Duration::from_millis(DEBOUNCE_MS).min(remaining);
            match tokio::time::timeout(wait, rx.recv()).await {
                Ok(Some(())) => continue,
                // = in 
                Ok(None) => return,
                // to 
                Err(_) => break,
            }
        }
        if let Err(error) = run_once(&db).await {
            super::record_failure(&error, "auto");
        }
    }
}

/// 1 up 。`Ok(false)` = no / enabled，。
async fn run_once(db: &Db) -> Result<bool, String> {
    let settings = settings::load();
    if !settings.enabled || !settings.auto_sync {
        return Ok(false);
    }
    if is_suppressed() {
        return Ok(false);
    }
    settings.validate()?;

    // no ：（， not request）
    let snapshot = snapshot::build(db)?;
    let content_hash = snapshot.content_hash();
    let key_fingerprint = crypto::key_fingerprint(SecretScope::Accounts);
    let unchanged = settings.status.last_local_hash.as_deref() == Some(content_hash.as_str())
        && settings.status.last_key_fingerprint == key_fingerprint;
    if unchanged {
        return Ok(false);
    }

    let _guard = super::sync_lock().lock().await;
    // to after 1 set： use can can 
    let settings = settings::load();
    if !settings.enabled || !settings.auto_sync {
        return Ok(false);
    }
    super::upload_locked(db, &settings).await?;
    logging::verbose("[CloudSync]", "自动同步：已上传本地变更");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_business_tables_trigger() {
        assert!(should_trigger_for_table("accounts"));
        assert!(should_trigger_for_table("checkin_records"));
        assert!(should_trigger_for_table("account_usage_records"));
        assert!(!should_trigger_for_table("logs"));
        assert!(!should_trigger_for_table("requests"));
        assert!(!should_trigger_for_table("debug_traffic"));
        assert!(!should_trigger_for_table("kv"));
    }

    #[test]
    fn suppression_guard_is_scoped() {
        assert!(!is_suppressed());
        {
            let _guard = SuppressionGuard::new();
            assert!(is_suppressed());
            {
                let _nested = SuppressionGuard::new();
                assert!(is_suppressed());
            }
            assert!(is_suppressed(), "内层释放后外层仍应抑制");
        }
        assert!(!is_suppressed());
    }
}
