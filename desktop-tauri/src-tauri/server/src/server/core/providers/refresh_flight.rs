//! new ****（Raccoon and AutoClaw use ； new ）。
//!
//! ── ──────────────────────────────────────────────
//! 1 and new this 1 request： after to use that 1 ， not is 
//! up 。「」 in Rust has must 1 ， ：
//!
//! 1. ** old not can in **。 old 「 」 and 
//! ， is `force = true`（401 after new ） will in that old ，
//! ** 1 token** —— 「 new after 1 」「 use 1 
//! token 1 」。** no has Done **： by `Arc<Flight>`
//! to ** already this flight etc **（ has `Arc`）， in 
//! （success/failure/） when 。 `force = true` in up 1 after 1 
//! new request，failure not will none。
//! 2. **（future drop）must and etc **。 future
//! when can can drop（client、 when ）。 old leader future 1 
//! drop， in —— after request etc ， to 30 seconds when 
//! to 409， and ** not **。 leader has 1 RAII 
//! （[`LeaderGuard`]）：none、`?` before return is future drop，
//! `Drop` will this 1 from and etc （ etc to 「 already 」，
//! is can ）。
//!
//! ── and （ ）───────────────────────────────
//! - ** not in await has `std::sync::MutexGuard`**：all in 
//! ， in return before —— otherwise future not is `Send`（ in 
//! when up ）， and will 。
//! - ** no has **： etc is join when `watch::Sender::subscribe()`
//! to `Receiver`，**return**；leader when 1 in 
//! 「」 after ，「join when in 」「 this 」，
//! `changed()` not will 。 after ， is etc 。
//! - **、/**： this 「 can 」「 already 」，
//! `force = true` down 1 not will up that 1 use old 。
//! - ** key not **：key ， as `HashMap` in ；
//! id this 。
//!
//! ── Why key by use ─────────────────────────────────
//! not （Raccoon `id` is id，AutoClaw `id` is 
//! `desktop-auth`），key must up 「provider / + this 
//! 」 not will 、 or after new use 。 then in 
//! `credentials.rs`（ that and ， and can 
//! key），「 key 、 key and 」。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::sync::watch;

use crate::server::errors::GatewayError;

/// etc 1 and new limit（150 × 200ms = 30 seconds）。
///
/// new request when is 30 seconds（ `AUTH_REQUEST_TIMEOUT_MS` /
/// `REQUEST_TIMEOUT_MS`），leader limit； this is 「leader 」 when 
/// ， when after etc to 409， by up is 。
const WAIT_STEPS: usize = 150;

/// etc 
const WAIT_STEP_MS: u64 = 200;

/// leader （future drop） when etc to error
pub const CANCELLED_MESSAGE: &str = "刷新已被取消，请稍后重试";

/// etc when error
pub const WAIT_TIMEOUT_MESSAGE: &str = "token 正在刷新中，请稍后重试";

/// ： SHA-256 （32 ）。
///
/// used for key —— 「 1 」 in 1 ， not in key 
/// has token。 and token etc ：** not or error**。
pub fn fingerprint(secret: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(32);
    for byte in digest.iter().take(16) {
        // String not will failure（fmt::Write for String as Ok）
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// 1 in new 。
///
/// leader and ** already etc ** has （`Arc`）， not ：
/// 1 ， after 1 `Arc` has ，。
struct Flight<T> {
    /// ：`None` = in new ；`Some` = already （success or failure）
    done: Mutex<Option<Result<T, GatewayError>>>,
    /// （；used for `watch` ）
    version: AtomicU64,
    /// ：`send` all
    signal: watch::Sender<u64>,
}

impl<T> Flight<T> {
    fn new() -> Self {
        let (signal, _receiver) = watch::channel(0u64);
        Self {
            done: Mutex::new(None),
            version: AtomicU64::new(0),
            signal,
        }
    }

    /// （ in return before ）
    fn outcome(&self) -> Option<Result<T, GatewayError>>
    where
        T: Clone,
    {
        let guard = match self.done.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.clone()
    }

    /// （ to ； 1 flight will has 1 ）
    fn set_outcome(&self, result: Result<T, GatewayError>) {
        let mut guard = match self.done.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if guard.is_none() {
            *guard = Some(result);
        }
    }

    /// （ already after use ）
    fn signal(&self) {
        let next = self.version.fetch_add(1, Ordering::SeqCst) + 1;
        // no has when `send` return Err —— not is error（leader not need need ）
        let _ = self.signal.send(next);
    }
}

/// ：key → in new 。
///
/// ** has in flight**：（success/failure/）， 
/// 「 when in new 」limit， not will request， not will down old 。
pub struct Table<T> {
    entries: Mutex<HashMap<String, Arc<Flight<T>>>>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
        }
    }
}

impl<T> Table<T> {
    /// 1 empty（ use `OnceLock` ）
    pub fn new() -> Self {
        Self::default()
    }

    /// （ in not ： and 1 ）
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Arc<Flight<T>>>> {
        match self.entries.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// （ is that 1 ， not after flight）
    fn detach(&self, key: &str, flight: &Arc<Flight<T>>) {
        let mut entries = self.lock();
        if let Some(current) = entries.get(key) {
            if Arc::ptr_eq(current, flight) {
                entries.remove(key);
            }
        }
    }
}

impl<T: Clone + Send + 'static> Table<T> {
    /// （ or ） 1 new 。
    ///
    /// return [`Join::Leader`] when use to new ：request after **must** use 
    /// `guard.finish(result)`（success and failure need ）； drop etc 「」，
    /// etc will to [`CANCELLED_MESSAGE`]。
    ///
    /// return [`Join::Waiter`] when already has key new in ，`waiter.wait().await` etc 。
    pub fn join(&self, key: &str) -> Join<'_, T> {
        let mut entries = self.lock();
        if let Some(flight) = entries.get(key) {
            // return： in this 1 「」（ before already ），
            // this not will 
            let receiver = flight.signal.subscribe();
            let flight = Arc::clone(flight);
            drop(entries);
            return Join::Waiter(Waiter { flight, receiver });
        }
        let flight = Arc::new(Flight::new());
        entries.insert(key.to_string(), Arc::clone(&flight));
        drop(entries);
        Join::Leader(LeaderGuard {
            key: key.to_string(),
            flight,
            table: self,
            finished: false,
        })
    }
}

/// `Table::join` 
pub enum Join<'a, T> {
    /// use is leader（ has RAII ）
    Leader(LeaderGuard<'a, T>),
    /// use is etc 
    Waiter(Waiter<T>),
}

/// leader RAII ： [`LeaderGuard::finish`]， `Drop`。
///
/// path will 「 + + 」， not will down ，
/// not will etc 。
pub struct LeaderGuard<'a, T> {
    key: String,
    flight: Arc<Flight<T>>,
    table: &'a Table<T>,
    finished: bool,
}

impl<'a, T> LeaderGuard<'a, T> {
    /// （success and failure this ）。
    ///
    /// is **、**： this `force = true` down 1 1 
    /// new request， not is up this 1 use old 。
    pub fn finish(mut self, result: Result<T, GatewayError>) {
        self.table.detach(&self.key, &self.flight);
        self.finished = true;
        self.flight.set_outcome(result);
        self.flight.signal();
    }
}

impl<'a, T> Drop for LeaderGuard<'a, T> {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        // path（future drop / before return）： and etc ，
        // after request can new new 
        self.table.detach(&self.key, &self.flight);
        self.flight
            .set_outcome(Err(GatewayError::with_status(409, CANCELLED_MESSAGE)));
        self.flight.signal();
    }
}

/// etc 
pub struct Waiter<T> {
    flight: Arc<Flight<T>>,
    receiver: watch::Receiver<u64>,
}

impl<T: Clone> Waiter<T> {
    /// etc that 1 （leader success/failure/ will etc ）。
    ///
    /// （）； etc when to 409（leader ， use can ）。
    pub async fn wait(mut self) -> Result<T, GatewayError> {
        for _ in 0..WAIT_STEPS {
            if let Some(outcome) = self.flight.outcome() {
                return outcome;
            }
            let waited = tokio::time::timeout(
                Duration::from_millis(WAIT_STEP_MS),
                self.receiver.changed(),
            )
            .await;
            match waited {
                // to ： down 1 
                Ok(Ok(())) => {}
                // ： not will （leader before 1 has Arc）， by when 
                Ok(Err(_)) => {}
                // when ：（「leader in up 」 ）
                Err(_) => {}
            }
        }
        Err(GatewayError::with_status(409, WAIT_TIMEOUT_MESSAGE))
    }
}
