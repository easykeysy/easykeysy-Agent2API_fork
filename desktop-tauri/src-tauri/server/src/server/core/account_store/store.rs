//! ：CRUD、current、/ will 、、。
//!
//! for workbuddy-account-store.mjs `createAccountStore` return。
//! in `super::mod` ：unknown、** 1 **
//! （all use 1 —— not is in provider 1 ， that is provider before 
//! old ）、provider when by workbuddy 。
//! 「 1 」 in `sql::priority_holder` SQL up （** not ** provider
//! ）， 1 409， `priority.rs` module header。
//!
//! ── ：`accounts.json` → SQLite `accounts` （ ）────
//! from `{config_dir}/accounts.json` to `{config_dir}/agent2api.db` 
//! `accounts` （ `server/db/schema.rs`）。** for 1 **：
//! file `pub fn` 、return、error and before ， use 
//! （`api::accounts`、`auth`、`routing`、 provider ）。
//!
//! SQL and in in `sql.rs`（ 1 SQL file）。
//! this ： and 、`AccountState` ↔ 、 and 。
//!
//! ── in ， ──────────────────────────────
//! `guard()` is that `Mutex<()>`，`with_lock` / `load_locked` / `save_locked`
//! **must** not （`account_transfer` use ， that module header）。
//! in ** not is file**： before 「file--」，
//! in 「 up 1 」。 etc —— is 「 1 when 
//! has 1 」， 。
//! Why need this not is SQLite： **--**（ 1 →
//! in Rust → ）， in must has 1 ；
//! SQLite not to this ， as 「」 in Rust 、 not in 。
//!
//! not ： not request（ super::mod ）。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::{json, Value};

use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::{
    AccountState, CredentialsById, CurrentEntry, SessionById, StoredAccount,
};
use crate::server::core::endpoints::resolve_edition;
use crate::server::core::providers::DEFAULT_PROVIDER_ID;
use crate::server::core::proxies::{
    resolve_account_proxy, resolve_account_proxy_rotating, ProxyResolution, ResolvedProxy,
};
use crate::server::db::Db;

/// error（ for Node AccountStoreError， → by use ）。
#[derive(Clone, Debug)]
pub struct AccountStoreError {
    pub message: String,
    pub status_code: i32,
}

impl AccountStoreError {
    pub fn new(message: impl Into<String>, status_code: i32) -> Self {
        Self { message: message.into(), status_code }
    }

    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::new(message, 400)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new(message, 404)
    }

    /// unavailable（failure， or after in ）。
    ///
    /// **500 + 1 can Reason**： is ， no has no has available，
    /// this not is 「 no for 」 that 400， not is 「 this not in 」 that 404 ——
    /// is server no has available 。 and before 「filefailure: …」
    /// 1 （ that is 500）， before and by 1 not use 。
    pub(crate) fn storage_unavailable(reason: &str) -> Self {
        Self::new(
            format!("账号数据库不可用（{reason}）：请检查磁盘空间与配置目录权限后重启应用"),
            500,
        )
    }
}
impl std::fmt::Display for AccountStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

/// 。
///
/// use `Arc<Inner>` ：`ServerState` can clone， handler to is 
/// 1 store。 is ** 1 Mutex** —— Node is ，
/// this use 1 and etc ， not 。
/// ： not request（ super::mod ）。
#[derive(Clone)]
pub struct AccountStore {
    inner: Arc<Inner>,
}

struct Inner {
    pub(super) cache: super::snapshot::SnapshotCache,
    validation: Mutex<Option<crate::server::core::providers::catalog::ValidationCache>>,
    /// SQLite 。`None` = failure（ when already ）。
    ///
    /// ── Why not 「file」─────────────────────────────────
    /// 1 is ：unavailable when `accounts.json`， can 。
    /// not ， by has ：
    /// 1. **path no **。 not is （、、
    /// ）， in that when —— is 1 in up 
    /// up 1 ， is new 。 to that 1 ， use need is 
    /// 「 1 can error」， not is 「 can use 、 1 file、
    /// down after for not up 」。
    /// 2. ** will **。 not （unknown、
    /// 1 、provider ） in up has ， 1 1 is 
    /// ； is ** not can ** use 。
    /// 3. **failure need ， not need **。`None` 500，
    /// use in up to 「unavailable」， not is 
    /// 「 no 」。
    /// and for ，`Db::open` failure** not **（ can 、、）——
    /// this is `ServerState::bootstrap` already has ， is after 。
    db: Option<Db>,
    /// file **path**（`{config_dir}/agent2api.db`）。
    ///
    /// Why 1 ：`file()` / `file_string()` is 「 to in 
    /// file」， has need this empty —— `/api/session` 
    /// `authFile`（ in 「file」 1 ） and `ServerState::bootstrap`
    /// 。failure when if returnempty or emptypath， use to is 「
    /// is empty 」 not is 「 no 」， to will 。
    /// when this is （`Db::file()` is ）， not this use 
    /// path —— in has 1 。
    file_path: PathBuf,
    /// 「--」 ； ， not request
    lock: Mutex<()>,
}

impl AccountStore {
    /// use （** **）。
    ///
    /// `db` as `None` not ： not （`ServerState::bootstrap`
    /// need need can ），all will 500 ， by 
    /// `Inner::db` 。
    ///
    /// `file_path` by `db` to （`Db::file()`）， is 「 in 」 this has 
    /// `Db` 1 ；`db` as `None` when path（ `Inner::file_path`）。
    pub fn with_db(db: Option<Db>) -> Self {
        let file_path = match db.as_ref() {
            Some(db) => db.file().to_path_buf(),
            None => crate::server::config::config_dir().join(crate::server::db::FILE_NAME),
        };
        Self {
            inner: Arc::new(Inner { db, file_path, lock: Mutex::new(()), cache: Default::default(), validation: Mutex::new(None) }),
        }
    }

    // ── in **** ：`with_config_dir()` ──────────────
    // is 「configdirectory， when accounts.json path」。 to 
    // after no has in ， and has ****： will `Db::open` 1 ，
    // is 1 up ** + ** —— --
    // ， not can ， store will for 。
    // already has 1 for this （`providers::adapter` 
    // 「 1 」）， this etc that 
    // in 。 1 use （`ServerState::bootstrap`） already as 
    // `Db` ， this ， not is 1 none use 
    // `#[allow(dead_code)]` 。

    /// in **file**path（`/api/session` authFile use ）。
    ///
    /// from 「file」「 file」—— this in up 
    /// 「file」 1 ， use in 、 this 。 no when to 
    /// is **path**（emptypath）， by `Inner::file_path`。
    pub fn file(&self) -> &Path {
        &self.inner.file_path
    }

    pub fn file_string(&self) -> String {
        self.inner.file_path.to_string_lossy().to_string()
    }

    /// is already （Node 
    /// `existsSync: () => existsSync(filePath)` for etc ）。
    ///
    /// 1 ： from 「`accounts.json` this file in not in 」
    /// 「 file in not in 」。current API use ， as this for etc ，
    /// after 「 is 」。
    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.file().exists()
    }

    pub(super) fn snapshot_cache(&self) -> &super::snapshot::SnapshotCache {
        &self.inner.cache
    }

    pub(crate) fn validation_cache(&self) -> &Mutex<Option<crate::server::core::providers::catalog::ValidationCache>> {
        &self.inner.validation
    }

    pub(super) fn database(&self) -> Option<&Db> {
        self.inner.db.as_ref()
    }

    /// ； in （ panic） not ： use ，
    /// totalall API 500。
    ///
    /// ── Why this 、 `Db::with` that return None ─────────────
    /// to ， its not ： this is ** need that 
    /// **（ not ）， in panic ， （ this 
    /// no has ） no has in can —— after can ，
    /// is 。
    /// `Db::with` that not ： is ****，panic can can 1 ，
    /// after will to （ that ）。 that ， this 。
    ///
    /// unavailable ** not in this **（ this no has return can ）—— 
    /// `with_conn` / `with_conn_mut` ， that returnerror。
    pub(crate) fn guard(&self) -> MutexGuard<'_, ()> {
        match self.inner.lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// in already before down 1 ；unavailable or SQL when return Err。
    ///
    /// this is file ** 1 **：`_guard` parameter not is —— 
    /// 「must」 this （Rust not empty 1 `MutexGuard`），
    /// is 「」 in not 。parameter down not 。
    ///
    /// SQL error 1 500： and before 「filefailure: {io error}」
    /// （ that is 500）—— failure is server， not is use 
    /// 。error up Reason， use and can to is 1 。
    pub(crate) fn with_conn<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Result<T, AccountStoreError> {
        let Some(db) = self.inner.db.as_ref() else {
            return Err(AccountStoreError::storage_unavailable("数据库未打开"));
        };
        db.with(|conn| action(conn))
            .ok_or_else(|| AccountStoreError::storage_unavailable("连接不可用（已标记中毒）"))?
            .map_err(|error| AccountStoreError::new(format!("账号数据库操作失败: {error}"), 500))
    }

    /// and [`with_conn`] ， to `&mut Connection` —— ** need need use this **。
    ///
    /// `save_state`（ ） and `batch_update` / `batch_remove` /
    /// `migrate_startup` ： need 、 need not 。
    pub(crate) fn with_conn_mut<T>(
        &self,
        _guard: &MutexGuard<'_, ()>,
        action: impl FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Result<T, AccountStoreError> {
        let Some(db) = self.inner.db.as_ref() else {
            return Err(AccountStoreError::storage_unavailable("数据库未打开"));
        };
        db.with_mut(|conn| action(conn))
            .ok_or_else(|| AccountStoreError::storage_unavailable("连接不可用（已标记中毒）"))?
            .map_err(|error| AccountStoreError::new(format!("账号数据库操作失败: {error}"), 500))
    }

    /// （ use need need in 1 when use ）。
    ///
    /// unavailable when return**empty** not is ： this （`pub`， and 
    /// `load_locked` ） no has failure ， is 「
    /// /」—— not to is 「 no has available」， use need 
    /// this （、noneavailable）。，****path 1 return
    /// `Result`，unavailable when 500， not success。
    pub(crate) fn load(&self, _guard: &MutexGuard<'_, ()>) -> AccountState {
        let Some(db) = self.inner.db.as_ref() else {
            return AccountState::default();
        };
        let (rows, priority_scope) = db.with(|conn| {
            (super::read_rows::all(conn), sql::load_priority_scope(conn))
        }).unwrap_or_else(|| (Ok(Vec::new()), Ok(None)));
        let accounts = super::read_rows::decode_many(rows.unwrap_or_default(), "读取全部账号")
            .into_iter().map(|(_, record)| record).collect();
        AccountState { accounts, priority_scope: priority_scope.ok().flatten() }
    }

    /// （** by **， `sql::save_state`）。
    ///
    /// this 「 is 」， down use ：
    /// - `account_transfer::import_accounts`（ and ， `save_locked`）；
    /// - `promote_to_front`（ need new ， `store_crud.rs`）；
    /// - `migrate_startup`（ and ， `store_admin.rs`）。
    /// ** not **：（ 1 ， `sql::put` /
    /// `sql::update_in_place` / `sql::delete`）； and old 
    /// （ need need 「 that 」， in `with_conn_mut` 、
    /// `sql::*`， not this ）。
    /// ： to this is 「 up `AccountState` is 」
    /// ，`save_state` by id for has 。
    ///
    /// ── errorWhy not is 「filefailure」──────────────
    /// that （ and 500） is before 1 1 500 ， in 
    /// not has 「file」 this —— will use by 「file」 to 
    /// （ accounts.json、file、），Reason in that 。
    /// is 500（ before and by not by ）， by `with_conn_mut`
    /// 1 to 「failure: {Reason}」，Reason SQLite 
    /// error and —— need need that old 。
    pub(crate) fn save(
        &self,
        state: &AccountState,
        guard: &MutexGuard<'_, ()>,
    ) -> Result<(), AccountStoreError> {
        self.with_conn_mut(guard, |conn| sql::save_state(conn, state))
    }

    /// use ： in already before down 
    pub fn load_locked(&self, guard: &MutexGuard<'_, ()>) -> AccountState {
        self.load(guard)
    }

    /// use ： in already before down 
    pub fn save_locked(
        &self,
        state: &AccountState,
        guard: &MutexGuard<'_, ()>,
    ) -> Result<(), AccountStoreError> {
        self.save(state, guard)
    }

    /// use ： after 1 「--」
    pub fn with_lock<T>(&self, action: impl FnOnce(&MutexGuard<'_, ()>) -> T) -> T {
        let guard = self.guard();
        action(&guard)
    }

    /// by id 1 （** 1 **； not in or unavailable when None）。
    ///
    /// to 「 already id」 path use ：`get_credentials_by_id` /
    /// `get_session_by_id` and `*_account_record`。 before need 
    /// 「 → in find」， in is 1 —— up this 
    /// is ** request** will to （`rotate` by id will 、 by id ），
    /// is 「 20 JSON」 is this 1 。
    pub(crate) fn record_by_id(&self, _guard: &MutexGuard<'_, ()>, id: &str) -> Option<StoredAccount> {
        let db = self.inner.db.as_ref()?;
        let result = db.with(|conn| super::read_rows::by_id(conn, id));
        match result {
            Some(Ok(Some(raw))) => match super::read_rows::decode(raw) {
                Ok((_, record)) => Some(record),
                Err(error) => {
                    sql::warn_load_failure("按 id 读取账号", &format!("账号 {id} 读不出来：{}", error.describe()));
                    None
                }
            },
            Some(Err(error)) => {
                sql::warn_load_failure("按 id 读取账号", &error.to_string());
                None
            }
            _ => None,
        }
    }

    /// provider （** this 1 **）。
    ///
    /// `current_entry_for_provider` / `accounts_for_provider` use ： 
    /// in 1 （「 this 1 」/「 this 1 enabled」），
    /// no has by 。 in `provider` up 
    /// （ this `sql.rs` module header）。
    pub(crate) fn records_for_provider(
        &self,
        _guard: &MutexGuard<'_, ()>,
        provider: &str,
    ) -> Vec<StoredAccount> {
        let Some(db) = self.inner.db.as_ref() else {
            return Vec::new();
        };
        let rows = db.with(|conn| super::read_rows::provider(conn, provider))
            .and_then(Result::ok).unwrap_or_default();
        super::read_rows::decode_many(rows, &format!("读取 {provider} 的账号"))
            .into_iter().map(|(_, record)| record).collect()
    }

    // ─── ：current ──────────────────────────────────────

    /// provider 「current」= ** this ** 1 「 already enabled and has available」 。
    ///
    /// and （ is `current_entry_for_provider` ）：enabled +
    /// [`StoredAccount::has_credentials`]（ when by no has token，
    /// in clientfile，「 has 」）+ by (, when ) 。
    /// no has available when return None（ not is to ）—— when this 1 not to 
    /// ， not this to 「current」。
    ///
    /// ── use ： this 1 ──────────────────────────────────
    /// after not ，「 no has when to this default」
    /// and `currentAccountIds` use 。 and `pick_current` ，
    /// is to this provider。 up this （ already 1 ）：
    /// 1. （raccoon/catpaw/autoclaw `desktop: true`） ——
    /// will ， not to 「 is current」 this ；
    /// 2. has up ，workbuddy 「current」 will is 。
    /// （ up ★） by use ** use ** to 
    /// （ `snapshot` `currentAccountIds`）， not need not 1 。
    pub(crate) fn pick_current_for_provider(
        accounts: &[StoredAccount],
        provider: &str,
    ) -> Option<StoredAccount> {
        let mut candidates: Vec<StoredAccount> = accounts
            .iter()
            .filter(|item| item.provider() == provider && item.enabled() && item.has_credentials())
            .cloned()
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next()
    }

    /// ****：all in 1 ， 1 「 already enabled and has available
    /// 」 。 this is ★ / 「」 and `/api/session` `currentAccountId`
    /// 。 for use need is supports this 、 is in ，
    /// that is by request （`routing::pick_account_by_priority`）。
    ///
    /// ── Whyexcluding** no has can ** （ use ， already can ）──
    /// this 「 will request that 」 use ：`/api/session` 
    /// `currentAccountId` /expiration time，`clear_session`（）
    /// by **delete**， ★ / 「 as 」 by 。Qoder has 
    /// can that will （return 501）mustexcluding： will current，
    /// 「」 will 。 up after ， this for ——
    /// ** can **（`supports_chat`）， not provider id，
    /// when not need need this 。
    pub(crate) fn pick_current(accounts: &[StoredAccount]) -> Option<StoredAccount> {
        let mut candidates: Vec<StoredAccount> = accounts
            .iter()
            .filter(|item| item.enabled() && item.has_credentials() && forwards_requests(item))
            .cloned()
            .collect();
        candidates.sort_by_key(StoredAccount::order_key);
        candidates.into_iter().next()
    }

    /// 「current」id（none or unavailable when as None）—— ， `pick_current`。
    ///
    /// this must****： is （`pick_current` need in all
    /// provider ）， to 1 no has 。
    pub fn current_account_id(&self) -> Option<String> {
        let snapshot = self.routing_snapshot();
        snapshot.accounts.iter().filter(|account| {
            account.get("enabled").and_then(Value::as_bool) != Some(false)
                && account.get("hasCredentials").and_then(Value::as_bool) == Some(true)
                && snapshot.forwarding_providers.contains(crate::server::core::routing::provider_of(account))
        }).min_by_key(|account| route_order_key(account))
            .and_then(|account| account.get("id")?.as_str().map(str::to_string))
    }

    /// workbuddy default and will （ workbuddy when use ：
    /// directory、 new etc ）；noneavailable when None。
    ///
    /// **workbuddy **（ to default provider）。 not use ——
    /// by ， has 「 no has 」 when to default。
    pub fn get_current_entry(&self) -> Option<CurrentEntry> {
        self.current_entry_for_provider(DEFAULT_PROVIDER_ID)
    }

    /// （ and get_current_entry 1 ， as ）
    pub fn get_active_entry(&self) -> Option<CurrentEntry> {
        self.get_current_entry()
    }

    /// provider 「current」 and will （Agent2API W2b-T3）。
    ///
    /// and `pick_current_for_provider` （enabled + has + by (, when )
    /// ）， is in ** this provider ** 1 。 when use 
    /// 「 this 1 default」， and `accounts_for_provider` 1 
    /// （ after used for ， this used for「 no has /」 when will ）。
    ///
    /// ── when （W3-T4）─────────────────────────
    /// ** no has accessToken**（ in `~/.box-agent/config/auth.json`），
    /// from `has_token()` `has_credentials()`： `desktop: true`
    /// raccoon 「 has 」， token by will when when 
    /// （ `session_from_record`）。 `has_token()` will this up 
    /// not in ， 401。
    ///
    /// return None this provider in no has available（ when 
    /// `auth::get_current_session_for` will this provider ）。
    pub fn current_entry_for_provider(&self, provider: &str) -> Option<CurrentEntry> {
        let ids = self.route_account_ids_for_provider(provider);
        let record = self.read_record(ids.first()?)?;
        if record.provider() != provider || !record.enabled() || !record.has_credentials() {
            return None;
        }
        Some(CurrentEntry {
            id: record.id().to_string(),
            session: self.session_from_record(&record),
        })
    }

    /// id will （directory new by use in 「get」 this ）。
    ///
    /// and [`Self::current_entry_for_provider`] （enabled + has ），
    /// has 「 by id not is 」：** use not ** ——
    /// not to return None， by use to 「 this not in or unavailable」 ，
    /// not is to （ that will 「 A、 use is B」 error）。
    pub fn session_for_account(&self, account_id: &str) -> Option<CurrentEntry> {
        let record = self.read_record(account_id)?;
        if !record.enabled() || !record.has_credentials() {
            return None;
        }
        Some(CurrentEntry {
            id: record.id().to_string(),
            session: self.session_from_record(&record),
        })
    }

    /// → auth will （/prefixPath/platform by edition ）。
    ///
    /// `proxy` is this （null = ），/ to etc 「 session
    /// request」 use none need parameter；`proxyError` emptyconfig 
    /// failure， use and 。
    ///
    /// ── ───────────────────────────────────
    /// up 「 not token、 when clientfile」 （ / Cline /
    /// AutoClaw） already to ，`live_desktop_credentials` in `None`；
    /// this in empty token —— up 「 need 」Reason，
    /// not error（ is CRUD and use path）。
    fn session_from_record(&self, record: &StoredAccount) -> Value {
        // ── edition （issue #380 / #381）─────────────────────
        // ① edition（ → pluginPublic → pluginSecret，
        // `StoredAccount::edition` ）；
        // ② no has when by **provider id** ：`workbuddy-intl` is 。
        // this 1 ， old （`account_view` not edition） will 
        // `resolve_edition(None)` —— provider
        // `workbuddy`， / to 「 not to this 」。
        let edition = match record.edition() {
            Some(explicit) => resolve_edition(Some(&explicit)),
            None => crate::server::core::providers::workbuddy::Region::from_provider_id(
                &record.provider(),
            )
            .map(|region| region.edition())
            .unwrap_or_else(|| resolve_edition(None)),
        };
        let resolution = resolve_account_proxy(Some(&record.proxy()));
        let (proxy, proxy_error) = split_resolution(resolution);
        let proxy_error_value = proxy_error
            .clone()
            .map(Value::String)
            .unwrap_or(Value::Null);
        let live = live_desktop_credentials(record);
        let (access_token, refresh_token, expires_at) = match live {
            Some(credentials) => credentials,
            None => (
                record.access_token(),
                record.refresh_token(),
                record.expires_at().unwrap_or(0.0),
            ),
        };
        // ── ZCode （`zcode::plan` use ）────────────────────
        // has ** up **（、 `zcode.z.ai`
        // Anthropic ）， can to will —— 、 use 
        // from this 。 is 「 ZCode 」， for will is **
        // empty**（ not will ）：
        // - `jwt`： Bearer（ and `accessToken` not can ）；
        // - `deviceMid`：request `metadata.user_id.device_id`；
        // - `zcodePlan`：（ = ， `zcode::plan_of`）。
        // empty not ： will empty will 「 has no has this 」 
        // 「 in not in 」， that is not 。
        // from ****， no has is empty： not 1 up 
        // this （ that to ）。 not this ——
        // from `store.get` to secret has that 。
        let mut session = json!({
            "endpoint": record.endpoint().unwrap_or_default(),
            "prefixPath": record.prefix_path().unwrap_or_default(),
            "platform": record.platform().unwrap_or_default(),
            "edition": edition.id,
            "proxy": proxy,
            "proxyError": proxy_error_value,
            "auth": {
                "accessToken": access_token,
                "refreshToken": refresh_token,
                "tokenType": "Bearer",
                "expiresAt": expires_at,
                "refreshExpiresAt": record.refresh_expires_at().unwrap_or(0.0),
                "domain": record.domain(),
            },
            "account": {
                "uid": record.uid(),
                "nickname": record.nickname(),
                "type": record.account_type(),
                "enterpriseId": record.enterprise_id(),
                "enterpriseName": record.enterprise_name(),
            },
        });
        if crate::server::core::providers::zcode::region::Region::from_provider_id(
            &record.provider(),
        )
        .is_some()
        {
            if let Some(object) = session.as_object_mut() {
                for (key, value) in [
                    ("jwt", record.jwt()),
                    ("deviceMid", record.device_mid()),
                    (
                        crate::server::core::providers::zcode::PLAN_FIELD,
                        record.zcode_plan(),
                    ),
                ] {
                    if !value.trim().is_empty() {
                        object.insert(key.to_string(), Value::String(value));
                    }
                }
            }
        }
        // ── Antigravity （`antigravity::adapter` use ）──────
        // and up ZCode 1 、 1 by ： can to will ， this 
        // in （`antigravity_accounts.rs` ）：
        // - `projectId`：cloudaicompanionProject —— `project` 
        // （ §3.3），OAuth new / / directory new after ；
        // - `email`： `userAgent` （ gmail/googlemail →
        // `jetski`，Manager ）。 will `account.uid` for 
        // Antigravity is email， that is 「」 use ，
        // use 。
        // is provider id（ and ZCode Region 1 ）， for will 
        // is empty（ not will ）。
        if record.provider() == super::ANTIGRAVITY_PROVIDER_ID {
            if let Some(object) = session.as_object_mut() {
                for key in ["projectId", "email"] {
                    let value = record
                        .fields()
                        .get(key)
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .unwrap_or("");
                    if !value.is_empty() {
                        object.insert(key.to_string(), Value::String(value.to_string()));
                    }
                }
            }
        }
        // OfficeAce already to ： from `store.get` 
        // secret（ not will ）， not need need Basic that 1 for 
        // will 。
        session
    }

    /// （Node getCredentialsById）；none/ not in when None
    ///
    /// when path already to （ `live_desktop_credentials`），
    /// this —— 「 has no has available」 in is 1 。
    pub fn get_credentials_by_id(&self, id: &str) -> Option<CredentialsById> {
        let record = self.read_record(id)?;
        let live = live_desktop_credentials(&record);
        let (access_token, refresh_token, expires_at) = match live {
            Some(credentials) => credentials,
            None => (
                record.access_token(),
                record.refresh_token(),
                record.expires_at().unwrap_or(0.0),
            ),
        };
        if access_token.is_empty() {
            return None;
        }
        let edition = resolve_edition(record.edition().as_deref());
        // ** down 1 **（「 this 」）： is 
        // 1 ， and ， `resolve_account_proxy_rotating`。
        let resolution = resolve_account_proxy_rotating(Some(&record.proxy()), record.id());
        let (proxy, proxy_error) = split_resolution(resolution);
        Some(CredentialsById {
            id: record.id().to_string(),
            name: record.name(),
            uid: record.uid(),
            access_token,
            refresh_token,
            expires_at: if expires_at > 0.0 { Some(expires_at) } else { None },
            endpoint: record.endpoint().unwrap_or_default(),
            prefix_path: record.prefix_path().unwrap_or_default(),
            platform: record.platform().unwrap_or_default(),
            edition: edition.id.to_string(),
            priority: record.priority(),
            enabled: record.enabled(),
            proxy,
            proxy_error,
        })
    }

    /// will （Node getSessionById）； not in /none when None
    ///
    /// `resolve_account_proxy_rotating`： 1 from this ，
    /// is 「 1 request 1 」（ `core::proxy_rotate` module header）。
    pub fn get_session_by_id(&self, id: &str) -> Option<SessionById> {
        let record = self.read_record(id)?;
        if !record.has_credentials() {
            return None;
        }
        let resolution = resolve_account_proxy_rotating(Some(&record.proxy()), record.id());
        let (proxy, proxy_error) = split_resolution(resolution);
        Some(SessionById {
            id: record.id().to_string(),
            session: self.session_from_record(&record),
            proxy,
            proxy_error,
        })
    }

    /// ** down 1 **（ not will 、 by config 1 ）。
    ///
    /// is 「429 」 that ：OpenCode Zen 429 is 
    /// ** by IP** （ this IP use ）， 1 can ——
    /// to IP 1 （ by ） up 。
    ///
    /// return `Ok(None)` = this not is （ use not this ）；
    /// `Err(Reason)` = is 1 not （Reason to ）。
    pub fn next_rotate_proxy(&self, id: &str) -> Result<Option<ResolvedProxy>, String> {
        let Some(record) = self.read_record(id) else {
            return Ok(None);
        };
        let config = record.proxy();
        if !crate::server::core::proxies::is_rotate_config(&config) {
            return Ok(None);
        }
        crate::server::core::proxy_rotate::next(&config, record.id())
            .map(Some)
            .map_err(|reason| reason)
    }
}

/// when （ that 1 ）； not is or not to when None。
///
/// ── in as None（ by ）──────────────────────────
/// up 「 not token、 when clientfile」 has ：
/// （`~/.box-agent/config/auth.json`）、AutoClaw（`%APPDATA%/AutoClaw/
/// auth.json` safeStorage ） and Cline（client `providers.json`）。
/// path** up client**， already to ——
/// not has 1 「client」 path。
///
/// not is ： will 、、 by 「 when 、
/// 」 use 。 will if （ not in ）， in this 1 
/// can ， use 1 not use ； will 「 this has 」 this 
/// from ， will to 1 empty match。
///
/// `desktop: true` in 、； up after 
/// by when ， no when this will 「 need need 」。
pub(crate) fn live_desktop_credentials(_record: &StoredAccount) -> Option<(String, String, f64)> {
    None
}

/// this provider ** is can **（`ProviderAdapter::supports_chat`）。
///
/// and `chatSupported` / can ，
/// not old MissingPlugin or ID can 。。
pub(crate) fn forwards_requests(record: &StoredAccount) -> bool {
    provider_forwards_requests(&record.provider())
}

pub(super) fn provider_forwards_requests(provider: &str) -> bool {
    if provider.starts_with(crate::server::core::custom_providers::ID_PREFIX) {
        return true;
    }
    crate::server::core::providers::is_executable_provider(provider)
        && crate::server::core::providers::adapter::adapter_by_id(provider)
            .is_some_and(|adapter| adapter.supports_chat())
}

pub(super) fn route_order_key(account: &Value) -> (i64, i64) {
    (account.get("priority").and_then(Value::as_i64).unwrap_or(super::priority::DEFAULT_PRIORITY),
        account.get("addedAt").and_then(Value::as_i64).unwrap_or(0))
}

/// `(proxy, proxyError)`：failure when proxy as null、
/// error Reason（ use and ）。
///
/// `proxy` is JSON（ will for ）；`proxy_error` is `Option<String>`，
/// will `proxyError` by use null/。
fn split_resolution(resolution: Option<ProxyResolution>) -> (Value, Option<String>) {
    match resolution {
        None => (Value::Null, None),
        Some(ProxyResolution::Resolved(ref proxy)) => {
            (json!(proxy_json(proxy)), None)
        }
        Some(ProxyResolution::Failed(message)) => (Value::Null, Some(message)),
    }
}

/// → JSON（ and ProxyResolution::to_json success）
fn proxy_json(proxy: &crate::server::core::proxies::ResolvedProxy) -> Value {
    json!({
        "source": proxy.source,
        "protocol": proxy.protocol,
        "host": proxy.host,
        "port": proxy.port,
        "username": proxy.username,
        "password": proxy.password,
        "label": proxy.label,
    })
}
