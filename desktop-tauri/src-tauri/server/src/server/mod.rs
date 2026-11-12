//! HTTP ：WorkBuddy Rust 。
//!
//! ── Why has this ────────────────────────────────────────
//! old is 「Tauri + node 」： server.cjs and node.exe
//! （88MB） 1 ， when 127.0.0.1:3065。
//! in Node after Rust， as **** HTTP ，
//! 1 、 HTTP and Node 1 —— is 
//! UI（desktop-tauri/ui/）、bridge.rs、commands.rs、gateway.rs 1 not use 。
//! ： from ~95MB to ~10MB， when not has 「 not node 」 。
//!
//! ** as （ need ）**： old 「 to 3065 already in use 」。
//! must bind success —— can in ， use 
//! etc API to 1 not 。 use when ，
//! use old （ `describe_bind_error`）。
//!
//! ── （ 1 ， 2 and ， 3 and ，
//! 4 for ， 5 ， 6 when to and update）──
//! ```text
//! server/
//! mod.rs total：ServerState 、start()/
//! account_bootstrap.rs 1 （auth.json old /
//! / old ）。** use **：
//! when （only no has old file） and use 「」 after ——
//! Why not can none， that file module header（ has 1 not can 
//! path： in before `accounts` ）
//! http.rs axum Router 、CORS、API Key check、404 
//! logging.rs （ + logs ）
//! config.rs config.json （unknown）
//! logs_store/ （append//statistics/empty）：
//! mod.rs and API（ etc 、、）
//! sql.rs SQL + 
//! errors.rs error + OpenAI error payload
//! db/ ** 1 **（file SQLite `{config_dir}/agent2api.db`）：
//! mod.rs and （`Db::open` / `with` / `with_mut` / `file`）
//! schema.rs DDL and `PRAGMA user_version` 
//! migrate/ old file → 1 （ + 1 file）
//! already （config / set / / /
//! request / request / / ）；
//! ** when 、 not ** —— by use in 「」
//! `POST /api/upgrade/run`（ api/upgrade_api.rs）。
//! success after old file as `*.migrated` ，** not delete**
//!   api/
//! mod.rs by 
//!     health.rs     GET /health
//!     session.rs    GET /api/session、/api/session/login/*、/api/session/refresh|logout、
//!                   POST /auth/login、POST /auth/logout
//!     config_api.rs GET/POST /api/config
//!     logs_api.rs   GET /api/logs、/stats、/download、DELETE
//!     stats_api.rs  GET /api/stats/summary、/api/stats/requests、DELETE /api/stats/requests、
//! GET/PUT /api/retention（statistics + ）
//! accounts.rs /api/accounts*（///// new ）
//! proxies.rs /api/proxies*（Clash when + ）
//! billing.rs /api/usage、/api/checkin*、/api/activity/*（ for server.mjs 871-911）
//! chat.rs POST /v1/chat/completions、GET /v1/models（ for ）
//! sanitize.rs /api/sanitize（、 and ）
//! auto_checkin.rs /api/auto-checkin*（ when to set / ）
//! scheduled_tasks.rs /api/scheduled-tasks*（ when ： / / ）
//! update.rs /api/update/*（updatecheck / down / / ）
//! storage_api.rs GET /api/storage（ 1 、 and ，）
//! upgrade_api.rs GET /api/upgrade、POST /api/upgrade/run（ old → SQLite ）
//!   core/
//! endpoints.rs （edition）：cn / intl（ and ）
//! account_store/ （、、CRUD、）。
//! already from `{config_dir}/accounts.json` to 
//! `agent2api.db` `accounts` ；`sql.rs` is ，
//! `store_view.rs` is and 
//! account_transfer.rs 
//! auth.rs will 、getStatus、、token new 
//! auth_http.rs up request and （）+ error
//! login.rs none and 
//! proxies.rs （Clash in clash.rs）
//! clash.rs Clash Verge config and 
//! egress.rs （ by reqwest Client）+ 
//! billing/ / to （checkin.rs） / 
//! models/ （/v1/models and ， use ）
//! routing.rs （ + ）
//! auto_checkin.rs when to （30 seconds + + ）
//! credential_maintenance.rs （ → new ；
//! by `scheduled_tasks` by config and ）
//! scheduled_tasks.rs when and （ / new ；
//! configfile in config.json scheduledTasks， by api::scheduled_tasks）
//! update/ update：
//! mod.rs / down / and 
//! version.rs 、、、file（）
//! client.rs （ → Clash） and GitHub request
//! sanitize.rs request（ can config then ，）
//! prompt.rs has （ / / ，）
//! degrade.rs （ after to 00:00 use in ）
//! upstream/ for ：
//! mod.rs （ / 429 / / SSE ）
//! request.rs request（、URL、system 、error）
//! sse.rs SSE reasoning and （ chunk ）
//! aggregate.rs （SSE → chat.completion）
//! usage.rs usage （token usage / / ）
//! ```
//!
//! ── to after ────────────────────────────────────
//! - add by ： `http::router` `protected` `.route(...)`
//! can API Key check； by `public` 。
//! API already （ 1-6）， 7 。
//! - and ：`ServerState::store()` / `auth()` / `login()` 。
//! - directory and ：directory `core::providers::catalog`（ when 
//! ， no has directory for ）； `ServerState::upstream()`。
//! - ：request `config::current().sanitize_settings()`，
//! by ； then already ，none IO。
//! - ：none，request 1 `config::current().prompt_plan()`
//! to `core::prompt` to up ； when by 
//! `core::degrade`（，none） and in 1 。
//! - when to ：`ServerState::auto_checkin()`；
//! `core::auto_checkin::stop_global()`（backend::shutdown use ）。
//! - when ：`core::scheduled_tasks`（ + ， in 
//! `bootstrap` 1 ）； and `config::scheduled_settings()`
//! （， —— set down 1 ， not ）。
//! not has that ， 1 new need 。
//! - update：`ServerState::update()`。
//! - requeststatistics： is `api::chat` → `RequestStats::record`；
//! is `api::stats_api` by （ / / empty），
//! `ServerState::request_stats()`； when in `start()` serve 
//! `flush()`（ WAL and ， that ）。
//! - ：`config::retention_settings()`（，path use ） and 
//! `config::set_retention()`（）； ****
//! （`RequestStats` / `LogStore` / `PUT /api/retention` ），
//! set not need need 。
//! - config：`config::current()`（ not ）+ `config::apply_update()`（）。
//! - ：`logging::log` / `logging::verbose` / `logging::log_event`。
//! - ：`core::egress::client_for` is 1 （ by use ；
//! `core::auth_http::send_raw/request_via`， for 
//! `core::upstream::request::send_chat_request`，GitHub 
//!     `core::update::client::fetch_with_egress`）。
//!
//! ── dead_code ─────────────────────────────────────────
//! 7 already `#![allow(dead_code)]` （ 1 
//! config/logs_store/errors/egress if use ）。 in ** warning**：
//! no use already ； and by etc has `#[allow(dead_code)]`
//! and by ， after 。

pub mod api;
mod account_bootstrap;
pub mod access;
pub mod altcha;
pub mod config;
pub mod config_migration;
mod compression;
pub mod core;
pub mod db;
pub mod errors;
pub mod http;
pub mod logging;
pub mod logs_store;
pub mod request_stats;
mod static_files;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tokio::sync::oneshot;

use crate::port_conflict::{ConflictKind, PortConflict};
use crate::server::core::account_store::AccountStore;
use crate::server::core::auth::AuthService;
use crate::server::core::auto_checkin::AutoCheckin;
use crate::server::core::billing::BillingService;
use crate::server::core::login::LoginService;
use crate::server::core::update::UpdateManager;
use crate::server::core::upstream::UpstreamService;
use crate::server::db::Db;
use crate::server::request_stats::{RequestStats, Retention};

/// 。handler `axum::extract::State` to 。
///
/// `port` / `config_dir` is 、 not ；
/// can 1 ****（、、、directory、
/// is `Clone` ）， not is 1 Mutex ——
/// this handler to can use ， not 。
#[derive(Clone)]
pub struct ServerState {
    /// （default 3065，available AGENT2API_PROXY_PORT ； old WORKBUDDY_PROXY_PORT can ）
    pub port: u16,
    /// （headless `AGENT2API_PANEL_PORT` ）。
    /// `None` = and （default and ）；`Some(p)` = 
    /// `/v1/*` ， and `/api/*` to `p` —— can not ，
    /// in 。 not set。
    pub panel_port: Option<u16>,
    /// 。default 127.0.0.1（； use after as 
    /// 0.0.0.0）；headless by `AGENT2API_HOST` （default 0.0.0.0，
    /// ）。
    pub host: IpAddr,
    /// in （ Tauri ）。
    ///
    /// true；headless false。 and `host.is_loopback()` is ：
    /// after host is 0.0.0.0， （Tauri
    /// ） in —— OAuth this （
    /// `api::session::login_oauth_start` `local_browser`）。headless not ，
    /// by `host.is_loopback()` （ no has ， false is for ）。
    pub local_panel: bool,
    /// configdirectory（`~/.agent2api`）， and gateway::config_dir() 
    pub config_dir: PathBuf,
    /// headless down （`ui/` directory） ；`None` = not 
    /// （ by Tauri custom-protocol ， API）。
    /// not set；headless [`ServerState::set_ui_dir`] 。
    ui_dir: Option<PathBuf>,
    /// （ 1 Mutex， not in when request）
    store: AccountStore,
    /// （ will / token new ）
    auth: AuthService,
    /// （none + ）
    login: LoginService,
    /// （ / to / ； by session.proxy ）
    billing: BillingService,
    /// directory（ + /v3/config new ； RwLock）
    /// for （ / 429 / SSE / ）
    upstream: UpstreamService,
    /// when to （ + ； Mutex + after ）
    auto_checkin: AutoCheckin,
    /// update（GitHub Release / down ； Mutex）
    update: UpdateManager,
    /// requeststatistics（ + by ， in 1 ）。
    ///
    /// 1 `Arc` not is its store that `Arc<Inner>`：
    /// （`RequestStats::with_db`） already ，
    /// `ServerState` is `Clone` （handler ），
    /// by this `Arc` —— and `AccountStore` `Arc<Inner>`
    /// is 1 ， is in 。
    /// need need `Arc`（ not is `LogStore` that `OnceLock` ） has 1 
    /// Reason：request need this **response**（ in when ），
    /// handler must can to 1 all（ `request_stats()`）。
    request_stats: Arc<RequestStats>,
    /// SQLite 。`None` = failure（ when already ）。
    /// Why as None not is failure：release is panic=abort，
    /// not this use ； can to can error。
    db: Option<Db>,
    /// when 「 has old JSON/JSONL file no 」。
    ///
    /// ── Why is `Arc<AtomicBool>` not is `bool` ──────────────
    /// `ServerState` is `Clone` （handler ）， not will 
    /// handler 。**must** can ：`POST /api/upgrade/run` 
    /// after need false，otherwise new will to 「 has 」，
    /// use will as no 。 use 1 not is `Mutex<bool>`： this has 
    /// 「 1 / 1 」， no has need need 1 。
    ///
    /// ── Why in when 1 and ， not is request ──────────
    /// is ** **（`GET /api/upgrade`），request 
    /// `is_file()` 1 no has ： can can by `POST /api/upgrade/run` ，
    /// that by will new this 。 is ** use old fileconfigdirectory
    /// not will **（ need down ）—— this is can ：file 
    /// in 。
    upgrade_pending: Arc<AtomicBool>,
    /// statistics（ / / ） is 
    /// in after 。
    ///
    /// this can can `requests` ： when seconds， 
    /// and ， not can not can 、 can not can use 。 from 
    /// `bootstrap` to success after （ [`Self::start_background_maintenance`]）。
    /// for ， `/api/session` 「 in 」。
    startup_maintenance: Arc<AtomicBool>,
}

impl ServerState {
    /// ， and 。
    ///
    /// need （ for server.mjs 336-353 ）：
    /// 1. —— config、 and all store ；
    /// 2. config —— （） need in before ，config in 
    /// in ，must in ① after ；
    /// 3. old file（** not **）—— old file in not in ，
    /// `upgrade_pending`， etc use in 「」
    /// （`POST /api/upgrade/run`， `api::upgrade_api` module header）；
    /// 4. —— after all can ；
    /// 5. + old auth.json （only as empty when ）+ 。
    /// ①② is from 「config、」 ：config 
    /// ， need need `Db` 。 new （ no when config from ） by 
    /// `config::read_raw` old file —— that is ，
    /// ①② before **must**。
    ///
    /// ②/④ is up 1 from 「」 ： when will by 
    /// 1 ，config —— if config no ， that will 
    /// default 30 ， use old and （** not can **）。
    /// ①→② ** no has ** this ：config in `logging::init_store` before 
    /// （ is from file + file）。
    ///
    /// ── 1 directory（`~/.workbuddy-proxy` → `~/.agent2api`） not in this ──
    /// must**** will configdirectory ， `bootstrap` already is 「set
    /// already 、 already 、 will 」 —— in this ：failure
    /// after need has 1 （settings::save / config::save_raw / ），
    /// new directory ，`target.exists()` from as ，none。
    /// use up to `lib.rs` setup 1 （`settings::load()` before ），
    /// that failure will use and ， not configfile。
    ///
    /// this 1 ****（`config_migration::pending_reason`）： if old directory
    /// in 、 new directory not in ， this no has （e.g. after has 
    /// use ）。 when returnerror、 not —— down 1 
    /// （`Db::open` → directory；`config::init` → configdirectory；`logging::init_store`
    /// → directory） will new directory， will 。failure `Err`
    /// up （`ensure_ready` to UI `backend:error`）， not panic、
    /// not 。
    pub fn bootstrap(port: u16, host: IpAddr) -> Result<Self, String> {
        if let Some(reason) = config_migration::pending_reason() {
            return Err(reason);
        }
        // when ：set「」 「 already 」 by 。 in 
        // before —— after 1 can can when （、、），
        // that when is 「 this already 」 1 。
        crate::server::core::system_info::record_start();
        // not in ： （ and ）
        // in ， bind empty， or from `HostEnv.loopback_port`
        // 。
        let config_dir = config::config_dir();
        // and Node 1 ：verbose by AGENT2API_VERBOSE=1 
        // （ old WORKBUDDY_VERBOSE can ， new ），
        // debug need not need （default has info up ，）
        let verbose = env_flag(&["AGENT2API_VERBOSE", "WORKBUDDY_VERBOSE"]);
        // ── ──────────────────────────────────────────────
        // is ： in `config::init()` ** before **、`logging::init_store(...)`
        // ** before **。
        // - Whymust in before ：，must；
        // ， no has available can —— that when already by 「available」
        // before ， can 。
        // - Whymust in `config::init()` before （****）：
        // config from `config.json` 1 `kv` ， need need 
        // `Db` 。 after down that `config::init(db.clone())` can 
        // from ；「 no when not to config」 this new by 
        // `config::read_raw` ** old file**（ and Why
        // that ， and `config::init` ）。
        // this 1 not can ： is 「config、」，config old file
        // —— that is need ， and after old file already ，
        // configempty。
        // failure** not **（`db = None`）， by ServerState.db 。
        let db_path = config_dir.join(db::FILE_NAME);
        let db = match db::Db::open(&db_path) {
            Ok(db) => Some(db),
            Err(error) => {
                logging::console_line("[Storage]", &format!("❌ failure: {error}"));
                // ── 1 ：**** → ────────────
                // this is 「 use to old 」。 old not after 
                // `accounts.data`（ will 1 ）， `save_state`
                // will 「 not = no has 」 DELETE —— 「 1 =
                // 」（ `db::schema::migrate` ）。
                // will use to 「、」， that not is 
                // 「」； this return Err failure。
                //
                // ** this 1 **： its DB failure（、、）
                // 1 down has as （`db = None` ， by 
                // `ServerState.db` and `db` module header）。must
                // `Db::open` 1 ：`Db::open` already failure、 no has available。
                if let Some(stored) = db::stored_schema_version(&db_path) {
                    if stored > db::schema::SCHEMA_VERSION {
                        return Err(format!(
                            " schema （{stored}）current（{}）： this is to old 。\
                              use update ， or from —— use old will after delete。",
                            db::schema::SCHEMA_VERSION,
                        ));
                    }
                }
                None
            }
        };
        // config：** in after 、 before **（ by up that ）。
        // after is 「 → old file → default」（`config::read_raw`），
        // is no config when （、 or new ） to is and before 1 
        // —— use after 1 to is old config，
        // down and old filedirectory not will use 。
        let snapshot = config::init(db.clone());
        core::task_state::install(db.clone());
        // （ not / use ）： when 
        // old kv 1 and （ usage_records module header）。
        core::usage_records::install(db.clone());
        // to （ to in 「 to 」）： can ， in 
        // `core::billing::checkin::run_checkin`（ 1 ）， ——
        // to 「empty / by delete」（ checkin_records module header）。
        core::checkin_records::install(db.clone());
        // AI for will （ that will ）： can ， in 
        // after （ `core::assistant_sessions` module header）。
        core::assistant_sessions::install(db.clone());
        core::assistant_images::install(db.clone());
        // when to 「 to 」 1 ： config that 1 
        // （ use is OfficeAce / CodeArts default）。
        // in this is as need config（`config::init` already in up ）， and must
        // 1 `/api/checkin-schedules` —— to this is 。
        // 1 ： up after ， use 。
        if core::checkin_schedules::extend_providers_once() {
            logging::console_line(
                "[Checkin]",
                " already new to when to （OfficeAce / CodeArts）",
            );
        }
        // 「limit」（ / Token then ）：Token 
        // 1 （schema v9 ）， can —— new 
        // by usage_query （ limiter module header）。
        core::limiter::install(db.clone());
        // （「」 ； by id use ）：
        // and as 「kv + 」 ， 1 `Db` 。
        // not and ， need 1 `/api/proxies/pool*` request。
        core::proxy_pool::install(db.clone());
        core::proxy_settings::install(db.clone());
        // （set「 → 」）：set and in kv 
        // `cloudSync`， can 。 in down spawn ——
        // must，otherwise 1 set will to default。
        core::cloud_sync::install(db.clone());
        if let Some(db) = db.clone() {
            core::cloud_sync::auto_sync::spawn_worker(db);
        }
        // ： ** 1 `Db`** （ and config / /
        // 1 ）。 in after by ， not to 
        // （ `core::providers::catalog_cache` module header）。
        // must in this ： down that `restore_cached_catalogs` ——
        // directory when ， to not 。
        core::providers::catalog_cache::install(db.clone());
        // OfficeAce available to （ in ，
        // by in `models.list` ）—— this not has this 。
        // models.dev （ can / ） new 1 
        // （kv `modelsDevCatalog` ）。 and catalog_cache 1 ： install
        // when （ can in ， is not to up new ）， not ；
        // in this is as and kv use in 1 ， for 「 in use this 」。
        core::model_meta::install(db.clone());
        // ── old file 1 ：** not ** ────────────────
        // in by use in 「」（`POST /api/upgrade/run`）。
        // Why： need is 「 use SQLite，」——
        // will not and （ 1 ， use not will 
        // ）， use for 「 」 is this need 。
        //
        // ── not will not will to config ────────────────────
        // not will ，path already ：
        // - `config::init` `config::read_raw`： not in when will 
        // ** old `config.json`**（`read_raw` when ），
        // already is use old config，`storage_dirs()` / `retention_settings()`
        // to for ；
        // - `settings::load()`（ / to ） is 「 → old file →
        // default」（ `settings` module header）。
        // ： before and after ， to config is 1 —— this is 
        // ， is 「」 can before 。
        //
        // when not （），。
        // Note this ** not **「」 ： will ， 1 
        // can （ file in ）， and 1 。
        let upgrade_pending = match db.as_ref() {
            Some(_) => {
                let pending = db::migrate::pending_items(&config_dir);
                if !pending.is_empty() {
                    logging::console_line(
                        "[Storage]",
                        &format!(
                            "⏳ to old （{}）， etc use in 「」 after ",
                            pending.join("、")
                        ),
                    );
                }
                !pending.is_empty()
            }
            // unavailable： no has can ， not this use （failure）。
            // that ❌ already in `Db::open` that 1 ， this not 。
            None => false,
        };
        // ： ** 1 `Db`** —— and 1 （ down 
        // AccountStore ）。`db` is `Option<Db>`： not when ，
        // 、returnempty（ 1 not ，
        // `LogStore` ）。
        // fork：ramdisk directory in after will tmpfs ， 1 ；
        // Windows / no has when is empty，`logging::init_store` in file after when will 。
        crate::paths::ensure_side_dirs();
        logging::init_store(db.clone(), verbose);
        // ：none is —— is **request**
        // （set down 1 request）， no will after 1 
        // requestnone can 。
        // 1 `debug_traffic` （ old `debug-traffic.jsonl`
        // by `migrate::import_debug` 1 ）， is ** 1 `Db`** ——
        // `Option<Db>`： not when （ 1 not ）。
        core::debug_traffic::init(db.clone());
        // Note this ** not has `config::storage_dirs()`**： directory
        // （`logDir` / `requestStatsDir` / `debugDir`） already ——
        // in 1 configdirectory， store `Db`。
        // that and `storage_dirs()` ，****： need 
        // use directory old file（ `db::migrate` 
        // `logs` / `requests` / `debug` ）。set not （T8 
        // that 1 ），file is not this 
        // —— that has 「 old file in 」 to 。

        // + + + （ 1 store ）。
        // in in up that （`accounts` ）， ** 1 `Db`**
        // —— not is 「configdirectory， need when path」。Note `db` is 
        // `Option<Db>`：failure when store ， will 
        // 500「unavailable」（ and by `store.rs` `Inner::db`）。
        // this ** not **「unavailable not 」 ： is ，
        // its （、directory、check、config） not ，
        // available has use —— use can in up to that ❌ and 。
        let store = AccountStore::with_db(db.clone());
        // need ， that not to ServerState（ is 
        // trait ， has 1 not use &AccountStore）。 1 
        // ， `account_store::plugin_accounts::shared`。
        core::account_store::plugin_accounts::install_shared(store.clone());
        let auth = AuthService::new(store.clone());
        let login = LoginService::new(store.clone());
        let billing = BillingService::new(auth.clone());
        // directory：** no has directory for **。 this 1 WorkBuddy 
        // `ModelCatalog`（ + `/v3/config` new + by 
        // ）， this to —— is 「 up 」，
        // not need need 。
        //
        // in 1 is （ `list_models` / 
        // `models.list` RPC）， by `core::providers::catalog` in when 。
        // down is that ****： all 1 （
        // RPC in this ）， is `/v1/models` request not etc 1 RPC。
        // must in up that `install` after —— ， can to 。
        core::providers::adapter::restore_cached_catalogs();
        let upstream = UpstreamService::new(store.clone(), auth.clone());

        // ── when to and update（ 6）──────────────────────────
        // is （ config / logging ）：
        // when to stop() need from **path**（backend::shutdown， has Tauri 
        // AppState） to ，must can from to ； this after ServerState that 
        // and that is 1 。
        let auto_checkin = core::auto_checkin::init_global(AutoCheckin::new(
            store.clone(),
            billing.clone(),
        ));
        // update： down directory `{config_dir}/updates`， and update::download_dir() 
        let update = core::update::init_global(UpdateManager::new(config_dir.clone()));

        // requeststatistics： 1 `requests` / `request_daily` 
        // （ old `requests.jsonl` / `request-daily.jsonl` by `migrate::import_requests`
        // / `migrate::import_daily` 1 ）， not has 「statistics directory」——
        // path by `Db` 1 has ， and `LogStore` 1 。
        // `requestStatsDir` this （directory that old file，
        // `db::migrate::requests`），file not 。
        //
        // ****（`config::retention_settings()`） not is ：
        // is request 1 path， `PUT /api/retention` will new ——
        // is set， down 1 / prune ，** not need need **。
        // default is 99999 （ 274 ，）， use 。
        let request_stats = Arc::new(RequestStats::with_db(db.clone(), || {
            let settings = config::retention_settings();
            Retention {
                request_days: settings.request_days,
                request_count: settings.request_count,
                daily_days: settings.daily_days,
            }
        }));
        // up in （）： not etc to 1 
        // request， when to already is 「 already in 」 
        core::billing::keepalive::install_stats(request_stats.clone());
        request_stats.sweep_stale_running();
        // ── （ 5 minutes 1 ）─────────────────────
        // （1 when ） before in 「 down 1 request」 when ：
        // / empty when no request， will 1 to up 。
        // when up 「 no request need 」 this 1 —— and that use 1 
        // （`sweep_stale_running`）， not then 。
        // ：（`DisconnectGuard`） and up after ， 
        // already ； this 1 「 / etc path」。
        let sweep_stats = request_stats.clone();
        crate::spawn_task(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(300));
            // 1 tick —— when ，
            ticker.tick().await;
            loop {
                ticker.tick().await;
                sweep_stats.sweep_stale_running();
            }
        });

        // ── 1 ：** has when ** ──────
        // that 1 （auth.json old 、`migrate_startup` and 、
        // old ） is 「 that empty not empty」，
        // ** old no when `accounts` is empty ** —— will has 1 not can 
        // path（， `import_accounts` etc up ）。
        // 、 and 「Why after must 1 」 in 
        // `account_bootstrap` module header —— that 1 is ，
        // 1 can to ， to that file。
        if !upgrade_pending {
            account_bootstrap::run(&store);
            // statistics ** not in this **： can can requests 
            // （ / / ）， when will 「
            // 」 1 。 by `start()` in success after 
            // `start_background_maintenance` ；
            // after that 1 by `api::upgrade_api` ， not 。
        }

        let state = Self {
            port,
            panel_port: None,
            host,
            local_panel: false,
            config_dir,
            ui_dir: None,
            store,
            auth,
            login,
            billing,
            upstream,
            auto_checkin,
            update,
            request_stats,
            db,
            upgrade_pending: Arc::new(AtomicBool::new(upgrade_pending)),
            startup_maintenance: Arc::new(AtomicBool::new(false)),
        };
        logging::log("[Server]", "Agent2API （Rust ） in …");
        logging::log("[Config]", &format!("API : {}", port));
        logging::log("[Config]", &format!("API : {}", host));
        logging::log(
            "[Config]",
            &format!(
                "API Key : {}",
                if snapshot.api_key_set() { "✅ already enabled" } else { "❌ enabled" }
            ),
        );
        logging::log("[Config]", &format!("default: {}", snapshot.default_model()));
        logging::log("[Config]", &format!(": {}", snapshot.locale()));
        let sanitize = snapshot.sanitize_settings();
        logging::log(
            "[Config]",
            &format!(
                ": {} · {} enabled then · use ={}",
                if sanitize.enabled { "✅ already enabled" } else { "❌ already " },
                sanitize.rules.entries().iter().filter(|rule| rule.enabled).count(),
                if sanitize.providers.is_empty() {
                    "".to_string()
                } else {
                    sanitize.providers.join(", ")
                },
            ),
        );
        // 1 ： + （default / file）+ file not to when 
        // Reason。 is （ not is ）， in set and ，
        // this not —— that 1 no 。
        logging::log(
            "[Config]",
            &format!(
                ": {}（{}）{}",
                snapshot.prompt_settings().mode.label(),
                snapshot.prompt_settings().source.label(),
                match snapshot.prompt_settings().file_error.as_deref() {
                    Some(reason) => format!("；⚠️  {reason}"),
                    None => String::new(),
                },
            ),
        );
        logging::log("[Config]", &format!("configdirectory: {}", state.config_dir.display())); // 1 （ 1 ： in 、 has no has ）。
        // in 「configdirectory」 after ： when 1 is 「 in 、 can not can 」，
        // `Db::file()` is 1 path for （ not 1 
        // `config_dir.join(FILE_NAME)` —— that is path to ）。
        // failure in `Db::open` that 1 already ❌ ， this not ，
        // 「unavailable」 this after （ store will ）。
        match state.db() {
            Some(db) => {
                logging::log("[Storage]", &format!(": {}", db.file().display()));
            }
            None => {
                logging::log(
                    "[Storage]",
                    "⚠️ unavailable： can will （ up ）",
                );
            }
        }
        let current = state.store.current_account_id();
        logging::log(
            "[Accounts]",
            &format!(
                ": {}（current {}）",
                state.store.file().display(),
                current.as_deref().unwrap_or("none")
            ),
        );
        // for server.mjs 1028-1042： has available when 1 ， and new directory
        let summary = state.auth.get_config_summary();
        if summary
            .get("configured")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            let source = summary
                .get("authSource")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let account = summary
                .get("currentAccountId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            logging::log("[Init]", &format!("✅ : {source}（ {account}）"));
            if let Some(expires_at) = summary
                .get("tokenExpiresAt")
                .and_then(serde_json::Value::as_f64)
            {
                let left = expires_at - logging::now_ms() as f64;
                logging::log(
                    "[Init]",
                    &format!(
                        "   token {}",
                        if left > 0.0 {
                            format!("{} minutes after ", (left / 60_000.0).round() as i64)
                        } else {
                            "expired（ will new ）".to_string()
                        }
                    ),
                );
            }
            // when directory new ** not in this **： already when 
            // （`core::scheduled_tasks` 1 ）， is 「 in when 
            // directory new = not 」 this 1 。 before this is none
            // spawn 1 `refresh_implemented`（ for Node 
            // `void refreshModelCatalog()`）， that as in default when not 。
        } else {
            // 「 old no 」 and 「 1 no has 」 is ： before in 
            // （agent2api.db） —— in ， WorkBuddy 
            // auth.json no 。 old none， will and up 1 `[Accounts] : …`
            // ， 1 （issue #178）。
            let available = available_account_count(&state);
            if available > 0 {
                logging::log(
                    "[Init]",
                    &format!(
                        "：{available} available（WorkBuddy not configured）"
                    ),
                );
            } else {
                let reason = summary
                    .get("unavailableReason")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(core::auth::UNCONFIGURED_REASON);
                logging::log("[Init]", &format!("⚠️ noneavailable: {reason}"));
            }
        }

        // when to ： when ， no and when already then 1 
        // （ for server.mjs 1055-1058： 1 ）。
        // in bootstrap not is start() ： and HTTP ，
        // and start() use （backend::ensure_ready） not to 「config is 」 
        // ；Node is in server.listen before 、 1 main() 。
        //
        // Note** not in ** `scheduled_tasks` ： to is 「」，
        // and that 「 etc 」 not is 1 （ by that module header）。
        let checkin_state = state.auto_checkin.state();
        if checkin_state
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            let time = checkin_state
                .get("time")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(core::auto_checkin::DEFAULT_TIME);
            logging::log("[Checkin]", &format!(" when to already enabled： {time} "));
            state.auto_checkin.start();
        }

        // when （ / directory new ）：
        // 1 ， by config and 。
        //
        // ── before after as for ────────────────────────────────────
        // before this is ： `loop { ; sleep(600s) }`（spawn 
        // 1 ）、directory in 「 has available」 spawn 1 new 。
        // in by ， when that 1 「 to 」——
        // default，**default as 1 **， and after not 。
        //
        // 1 has ：directory in not 「 has none」
        // （ in `configured` ）。 is new 、 no when will 1 
        // —— directory not need need （ its `refresh_models`
        // empty token ），workbuddy none then return「」 not 
        // ，CatPaw / AutoClaw is 。 is 1 
        // none request， is 「 this 1 」 this 1 。
        //
        // ── Why not ────────────────────────────
        // when will ，。 use crate::spawn_task
        // （ and auto_checkin 1 by ） from tokio up down use can when 。
        core::scheduled_tasks::spawn(state.store.clone(), state.update.clone());
        // configupdate and will ， when etc not is 。
        core::growth_scheduler::spawn(state.store.clone());
        // ZCode 「 need not need 」 by up config —— that 
        // to （ in up ，
        // `zcode::config_sync::maybe_sync`）。 this not has 。

        // ： and up 1 （ 1 
        // 1 ， to in 「 + config」 up ）， is by ，
        // not is （ `core::usage_query` module header）。
        core::usage_query::spawn_sweeper(state.store.clone());

        // ── （ use ）─────────────────────────────
        // after ： need spawn ， can can need etc seconds。 this not await ——
        // is optional， no /must（），
        // path error（ `plugin_host::bootstrap`）。
        // use to when `RpcAdapter::call` will `ensure()` etc 。
        // need to （LobsterAI / Accio that 
        // `redirect_uri` by ），。
        core::plugin_host::set_loopback_port(port);
        core::plugin_host::spawn_bootstrap(state.store.clone());

        Ok(state)
    }

    /// 
    pub fn store(&self) -> &AccountStore {
        &self.store
    }

    /// 
    pub fn auth(&self) -> &AuthService {
        &self.auth
    }

    /// 
    pub fn login(&self) -> &LoginService {
        &self.login
    }

    /// （ / to / ）
    pub fn billing(&self) -> &BillingService {
        &self.billing
    }

    /// for 
    pub fn upstream(&self) -> &UpstreamService {
        &self.upstream
    }

    /// when to （/api/auto-checkin* by + ）
    pub fn auto_checkin(&self) -> &AutoCheckin {
        &self.auto_checkin
    }

    /// update（/api/update/* by ）
    pub fn update(&self) -> &UpdateManager {
        &self.update
    }

    /// requeststatistics（ for ；/ API by after ）
    ///
    /// return `Arc<RequestStats>` all（ not is its store that return
    /// `&T`）：request need **response**，response handler 
    /// ， not to use 。
    pub fn request_stats(&self) -> Arc<RequestStats> {
        self.request_stats.clone()
    }

    /// SQLite （ 1 ）。
    ///
    /// return `Option<&Db>`：`None` = failure（ when already 1 can ）。
    /// after store must this 1 —— unavailable when 
    /// （e.g. not ）， not is panic。 by and ：
    /// release is panic=abort， not this use 。
    pub fn db(&self) -> Option<&Db> {
        self.db.as_ref()
    }

    /// has no has old file（ ；）。
    pub fn upgrade_pending(&self) -> bool {
        self.upgrade_pending.load(Ordering::Relaxed)
    }

    /// 「」 and （`POST /api/upgrade/run` after ）。
    ///
    /// Why not is false：** can can success 1 **（ 1 
    /// failure、failure， old file in ）， that when has file in 、 can 1 
    /// （ etc ）。 this 1 ， use not 
    /// 「 that to no has 」。
    ///
    /// unavailable when by 「 no has 」： no has can ， use 
    /// will to 1 failure （ and `bootstrap` 1 ）。
    pub fn refresh_upgrade_pending(&self) -> bool {
        let pending = self.db.is_some() && db::migrate::has_pending(&self.config_dir);
        self.upgrade_pending.store(pending, Ordering::Relaxed);
        pending
    }

    /// can （ use ）。
    pub fn upgrade_items(&self) -> Vec<&'static str> {
        if self.db.is_none() {
            return Vec::new();
        }
        db::migrate::pending_items(&self.config_dir)
    }

    /// statistics is in after （`/api/session` 
    /// `maintenance.running`； can 「 in 」）。
    pub fn startup_maintenance_running(&self) -> bool {
        self.startup_maintenance.load(Ordering::Relaxed)
    }

    /// in success after statistics（ by [`start`] use ， 1 ）。
    ///
    /// ── Why in this not is `bootstrap` ────────────────────────
    /// `bootstrap` return before not can bind ， this （ / 
    /// / ） can can `requests` 。 ，
    /// will 「、 not 」seconds —— use to is 。
    /// in HTTP available， this in 。
    ///
    /// ── Why not 1 ────────────────────────────────
    /// `account_bootstrap` and ， and （
    /// ）；statistics and ， after 。 for 
    /// 「 can not can to 」 not ， after 。
    ///
    /// ── no when ────────────────────────────────────
    /// and `bootstrap` 1 ：。 use after ，`api::upgrade_api`
    /// will by this （ that when HTTP already in ， not will ）。
    pub fn start_background_maintenance(&self) {
        if self.upgrade_pending() {
            logging::log(
                "[Stats]",
                "⏳ old ，statistics（ after ）",
            );
            return;
        }
        // swap use （ after when start can can use ） 1 。
        if self.startup_maintenance.swap(true, Ordering::SeqCst) {
            return;
        }

        let state = self.clone();
        crate::spawn_task(async move {
            logging::log("[Stats]", "statistics already in after （ not ）");
            let started = std::time::Instant::now();
            let stats = state.request_stats();
            let outcome = tokio::task::spawn_blocking(move || {
                stats.remap_model_dimension_once();
                stats.backfill_source_dimensions();
                stats.prune();
            })
            .await;
            state.startup_maintenance.store(false, Ordering::SeqCst);
            match outcome {
                Ok(()) => logging::log(
                    "[Stats]",
                    &format!(
                        "✅ statistics（ use when {} seconds）",
                        started.elapsed().as_secs()
                    ),
                ),
                Err(error) => logging::log(
                    "[Stats]",
                    &format!("⚠️ statistics: {error}"),
                ),
            }
        });
    }

    /// headless ：`ui_dir` to `ui/` directory，
    /// API 1 and （ not use —— by Tauri ）。
    pub fn set_ui_dir(&mut self, dir: PathBuf) {
        self.ui_dir = Some(dir);
    }

    /// headless is （`http::router` fallback）。
    pub fn ui_dir(&self) -> Option<&PathBuf> {
        self.ui_dir.as_ref()
    }
}

/// ：** by 1 set **（ before 1 is new ）。
///
/// and Node 1 ： has as `"1"` ， its （ `"true"`/`"0"`/empty）
/// by —— this 「 0 」 and 「 no 」 not will 。
/// 1 no when return false。
fn env_flag(names: &[&str]) -> bool {
    for name in names {
        if let Ok(value) = std::env::var(name) {
            return value.trim() == "1";
        }
    }
    false
}

/// **available** （`available` = enabled and ）。
///
/// and 、 is 1 （`account_store::store_view` 
/// `public_account` this ）， this not 「enabled + has 」 ——
/// that is ， and before 、。
///
/// use in ： down 1 available no has ， and 「 old no 」
/// this 1 （issue #178）。
fn available_account_count(state: &ServerState) -> usize {
    core::routing::accounts_of(&state.store.list_accounts())
        .iter()
        .filter(|account| {
            account.get("available").and_then(serde_json::Value::as_bool) == Some(true)
        })
        .count()
}

/// ： bind（failurereturn can error）+ serve。
///
/// return： use （state::BackendHandle） has ，
/// when `send(())` graceful shutdown。
///
/// ── Why bind use std 、 tokio to spawn ──
/// `tokio::net::TcpListener::from_std` will to reactor ，
/// **must in Tokio when up down use **，otherwise panic
/// （ release is panic=abort， that will use ）。
/// can can from any use （Tauri setup in 、
/// commands in up down ），：
/// 1. `std::net::TcpListener::bind` —— this 1 not need need when ，
/// and use this failure can to OS errorreturn to use ；
/// 2. need need when from_std and serve `crate::spawn_task`，
/// 1 in when up ，reactor available。
///
/// ── failureWhyreturn PortConflict not is ──
/// bind failure Reason this to ： use can 「」，
/// in then can 「」。（`std::io::ErrorKind`）
/// has in this to ， in this up ，error。
pub fn start(state: &ServerState) -> Result<oneshot::Sender<()>, PortConflict> {
    // ── by by ─────────────────────────────────────────
    // default（panel_port = None， / PANEL_PORT headless）：
    // by （ + and ）， as and before 1 。
    // （headless AGENT2API_PANEL_PORT）：（/v1/* + /health），
    // （ + /api/*） panel_port —— in ，
    // 。（panel_port == port as no ， and 。）
    let panel_port = match state.panel_port {
        Some(port) if port != state.port => Some(port),
        _ => None,
    };
    let main_router = match panel_port {
        Some(_) => http::gateway_router(state.clone()),
        None => http::router(state.clone()),
    };
    let panel_router = panel_port.map(|_| http::panel_router(state.clone()));

    let addr = SocketAddr::new(state.host, state.port);
    let listener = std::net::TcpListener::bind(addr)
        .map_err(|error| PortConflict::from_bind_error(state.port, &error, None))?;
    // ： from this 1 to serve ， if has ，
    // accept will 。 in 。
    listener.set_nonblocking(true).map_err(|error| {
        PortConflict::new(
            ConflictKind::Other,
            state.port,
            None,
            &format!("set as failure: {error}"),
        )
    })?;
    // （）： in bind —— to 
    // serve， 1 failurereturn（ already bind drop ，
    // not will down ）。
    let panel_listener = match panel_port {
        Some(port) => {
            let addr = SocketAddr::new(state.host, port);
            let listener = std::net::TcpListener::bind(addr)
                .map_err(|error| PortConflict::from_bind_error(port, &error, None))?;
            listener.set_nonblocking(true).map_err(|error| {
                PortConflict::new(
                    ConflictKind::Other,
                    port,
                    None,
                    &format!("set as failure: {error}"),
                )
            })?;
            Some(listener)
        }
        None => None,
    };

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    // ：oneshot has 1 receiver， has serve need 
    // to 。 oneshot （send or drop）
    // watch ， serve 1 receiver；watch drop will 
    // `wait_for` return， and oneshot 1 。
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let mut stop_requested = stop_rx.clone();
    let stop_from_signal = stop_tx.clone();
    crate::spawn_task(async move {
        tokio::select! {
            _ = shutdown_rx => {}
            _ = stop_requested.wait_for(|stop| *stop) => {}
        }
        stop_from_signal.send_replace(true);
    });

    // need use statistics：（`state` is use ， not can 
    // async move 1 ）
    let request_stats = state.request_stats();
    let stop_main = stop_rx.clone();
    let stop_from_main = stop_tx.clone();
    crate::spawn_task(async move {
        // when up down in this ，from_std not will panic
        let listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(error) => {
                stop_from_main.send_replace(true);
                logging::log("[Server]", &format!("❌ createfailure: {error}"));
                return;
            }
        };
        // into_make_service_with_connect_info： client handler
        // （ failure by IP ；none ConnectInfo will panic，
        // serve must and 1 ）
        let mut stop = stop_main;
        let server = axum::serve(
            listener,
            main_router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            // to （ or ） accept ，
            // already in in request will 
            let _ = stop.wait_for(|stop| *stop).await;
        });
        let result = server.await;
        // / 1 accept。
        stop_from_main.send_replace(true);
        match result {
            Ok(()) => logging::log("[Server]", " already "),
            Err(error) => logging::log("[Server]", &format!("❌ : {error}")),
        }
        // statistics： in serve return** after **（graceful shutdown already etc in request
        // ）， when not will has new 。
        // Note** not is **「 」—— ，
        // no has （ old 1 T4 ）；
        // in is WAL and 、 `-wal` / `-shm` ，
        // use when for 1 file（ `RequestStats::flush` ）。
        request_stats.flush();
    });

    // serve（ has ）：、 1 。
    // statistics serve（ up that ）， this request。
    if let Some(listener) = panel_listener {
        let router = panel_router.expect("panel_port and panel_router must for ");
        let stop_from_panel = stop_tx;
        crate::spawn_task(async move {
            let listener = match tokio::net::TcpListener::from_std(listener) {
                Ok(listener) => listener,
                Err(error) => {
                    stop_from_panel.send_replace(true);
                    logging::log("[Server]", &format!("❌ createfailure: {error}"));
                    return;
                }
            };
            let server = axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async move {
                let mut stop = stop_rx;
                let _ = stop.wait_for(|stop| *stop).await;
            });
            let result = server.await;
            stop_from_panel.send_replace(true);
            if let Err(error) = result {
                logging::log("[Server]", &format!("❌ : {error}"));
            }
        });
    }

    // already bind、 serve already ： when already can ，
    // can can statistics（ / ）。 in serve before will 
    // after 1 1 （ start_background_maintenance ）。
    state.start_background_maintenance();

    Ok(shutdown_tx)
}

/// before ： can not can bind up this 。
///
/// and `start()` bind is 1 （ `std::net::TcpListener::bind`），
/// ** not **：。 in 「」 when use use 
/// ， in before to 「 this 」 not is etc to after 。
///
/// return Ok(()) available；Err is （ and failure use 1 ）。
pub fn probe_port(port: u16) -> Result<(), PortConflict> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    match std::net::TcpListener::bind(addr) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(error) => Err(PortConflict::from_bind_error(port, &error, None)),
    }
}
