//! axum Router ： by 、CORS、API Key check、404/405 、body limit。
//!
//! and Node server.mjs `createRequestHandler`（672-991 ） for ：
//!   - 面板面（`/api/*` and ） response CORS （ 404 and errorresponse）
//! —— Node in none down ；OPTIONS 204（ not need need ）
//! - （`/v1/*`） CORS default****， by set「 → 」
//! （ [`cors_gateway`] and `config::KEY_CORS_ENABLED`）—— is 
//! up that 1 ，`*` not this none
//! - not configured API Key when ；config after by in 1 for 
//! - 404 `Not found: <METHOD> <path>`
//! - error 1 errors::GatewayError → OpenAI payload + 500
//!
//! ── by （ after this ）────────────────────────
//! `public` ：/health、（/api/session already protected）
//! （Node this no checkApiKey）
//! `protected` need ：/api/config、/api/logs*、/api/stats*、/api/retention、
//!               /api/accounts*、/api/proxies*、/api/custom-providers*、
//!               /api/usage、/api/checkin*、/api/activity/*、/api/sanitize、
//!               /api/auto-checkin*、/api/update/*、
//!               /api/session/login/*、/api/session/refresh|logout、/auth/*
//!
//! in in `protected` up ， not is 「 in + 」：
//! Node is `if (!checkApiKey(req)) return unauthorized(res)`，
//! this ，add by when not will check（ 1 can ）。
//!
//! ── by 1 ──────────────────────────────────────
//! `/api/accounts/{id}` and `/api/accounts/export` this path in 1 
//! Router must：matchit is 「 > parameter」， not use Node 
//! that export/import not id —— ** not can **
//! `/api/accounts/{*rest}` this （ will and not check）。
//! this path。

use axum::extract::Request;
use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get, patch, post};
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::server::api;
use crate::server::errors;
use crate::server::logging;
use crate::server::ServerState;

/// requestlimit：32MB（ for Node MAX_BODY_SIZE）。
/// by （/api/* ） and `to_bytes` ；
/// /v1/* not limit —— up down + base64 request not can 
/// （ up will can ）， `gateway_router` disable。
pub const MAX_BODY_SIZE: usize = 32 * 1024 * 1024;

/// and ，、Key、set and when will PATCH。
/// available，request by ； in request in 。
const CORS_METHODS: &str = "GET, POST, PUT, PATCH, OPTIONS, DELETE";
/// CORS request。
///
/// ── Why not is 「 Node 」 ──────────────────────────
/// before and Node 1 。 after is to ** APK** ：APK ，
/// origin is `http://localhost`（Capacitor WebView），request is ，
/// requestmust will —— server cookie is 1 origin ，
/// not will up ， can `x-panel-token` / `x-panel-refresh` （ and 
/// in 1 ， access::access_candidates）。
///
/// `x-panel-auth-mode` is when need 「response」 （ api::panel 
/// `tokens_in_body`）—— APK no has available cookie ，must to body 。
///
/// this not ：「 this 」， can not can use by 
/// `require_api_key` ； and `Allow-Origin: *` down not will request cookie，
/// can request —— this is need 。
const CORS_HEADERS: &str =
    "Content-Type, Authorization, x-api-key, x-panel-token, x-panel-refresh, x-panel-auth-mode, x-agent2api-client, x-title, x-client-type, x-cerebras-3rd-party-integration";

/// by （ + ****：default and ）。
///
/// （headless `AGENT2API_PANEL_PORT`） when not need use ：
/// use [`panel_router`] + [`gateway_router`] 1 （ `mod::start`）。
async fn retired_proxy_feature() -> Response {
    errors::management_error(410, crate::server::core::proxy_pool::RETIRED_HINT)
}

pub fn router(state: ServerState) -> Router {
    panel_router(state.clone()).merge(gateway_router(state))
}

/// by ：（file fallback）+ `/api/*`。
///
/// down `AGENT2API_PANEL_PORT` —— can not 
/// （ / compose not ）， in 。
pub fn panel_router(state: ServerState) -> Router {
    // GET /api/session in （「 before in config Key before need can 」，
    // session.rs）， after protected： request by 1 
    // Key， not ； headless/ down 、 etc 
    // ， not this in none Key （ bridge 401 will ）。
    //
    // `/api/endpoints`（ to ） to after already delete：
    // not has that ， no has can 。
    let public = Router::new()
        // ： use when no has ，must public —— by 
        // failure（ IP 5 5 minutes） and bcrypt 
        // （ api::panel and access module header）。not configured 
        // （） this return 404， etc can not in 。
        .route("/api/panel/login", post(api::panel::panel_login))
        // / will new / ： as 「」 —— use need no has 
        // （setup / login）， need is （refresh 
        // refresh cookie、logout will ）， public by 。
        .route("/api/panel/status", get(api::panel::panel_status))
        .route("/api/panel/setup", post(api::panel::panel_setup))
        .route("/api/panel/refresh", post(api::panel::panel_refresh))
        .route("/api/panel/logout", post(api::panel::panel_logout))
        // ALTCHA ： and status / setup as 「」 —— when 
        // use no has （ when 400， before ）
        .route("/api/panel/captcha", get(api::panel::captcha_challenge))
        // will （）： as public —— in before ，
        // / 1 60 seconds cookie， not will 、 not 
        // （ api::panel::cookie_probe）
        // any not is get： need and ****（POST）—— has in by 
        // for Set-Cookie， GET will 「cookie 」
        .route("/api/panel/cookie-probe", any(api::panel::cookie_probe))
        // CatPaw by already to ： is ****
        // （ bind loopback + up poll-token ）， in 
        // up ， not `/api/session/login/*`。
        // in ： shim use redirect ，
        // GET current。 use is up ， not can 
        // API Key， by 1 state 。parameter not provider
        // —— ， in 。
        .route(
            "/api/session/login/remote-callback",
            get(api::session::login_remote_callback),
        )
        // KukuAI （）： in success.html
        // after WebView2 Cookie （BDUSS is HttpOnly， not to ），
        // state + Cookie 1 and POST to this 。****： use is 
        // （）， not API Key； by 1 state
        // （ and in ）。
        .route(
            "/api/session/login/kuku/complete",
            post(api::session::login_kuku_complete),
        )
        // OrcaTerm （****）： is OAuth —— 
        // `orcaterm://oauth_callback?session_id=…` to 
        // `/api/session/login/callback`， by after `OAuthExchangeToken` 
        // （ `providers/orcaterm/login.rs` module header）。 this to 「 use already has 
        // 1 」 ： state + 1 and POST ，。
        // ****： use is （）， not API Key； by 
        // 1 state （ and kuku/complete ）。
        .route(
            "/api/session/login/orcaterm/complete",
            post(api::session::login_orcaterm_complete),
        )
        // Antigravity ：** 302 to this **（Google 
        // after when redirect_uri）。path `/oauth-callback`
        // is Google / use that （`Antigravity-Manager`
        // `oauth_server.rs`）， not is 。
        // by and CatPaw / Accio loopback 1 ：
        // use is use ， no has API Key； by 
        // 1 state （ for and core::login）。
        .route(
            "/oauth-callback",
            get(api::session::login_antigravity_callback),
        )
        // LobsterAI loopback ：** 302 to this **。
        // ⚠️ this 1 path** no has after **， and up 
        // not 1 —— this is has ：`redirect_uri` is 
        // to up ， by ， is for path has 
        // ** no has **。 use is `/auth/callback`， this that 
        // already ， not as 1 。
        // this path in its use （ its is `-{vendor}` /
        // `/oauth/callback` / `/oauth-callback`）。
        // by and up 1 ： use is use ， no has API Key；
        // by 1 state 。
        .route(
            "/auth/callback",
            get(api::session::login_lobsterai_callback),
        )
        // ── UI （`/plugins/{pack}/ui/*`）──
        // can （ to in this 「 has has 」 can ），
        // by 、client iframe 。** not protected**：iframe
        // request not up ， not （ is HTML/JS/CSS），
        // request `/api/*`（ protected）。 this and `ui/`
        // directory 1 —— `ui/index.html` not 。
        //
        // check in `plugin_host::resolve_ui_file`：** already enabled** 
        // `ui/` directory，`..` and for path 1 。
        .route(
            "/plugins/{pack}/ui/{*rest}",
            get(api::plugin_ui::serve),
        )
        // ── （`/plugins/runtime/*`）──
        // iframe is ： not to `ui/css/tokens.css`、 not to 
        // already React / ， not to `ui/assets/providers/` 。
        // is can 、、 use —— will and 
        // ，「 not 1 」。
        //
        // this by that to iframe（， down 
        // not `ui/` directory in ， this filemodule header）：
        // runtime/theme.css → 
        // runtime/islands/ui.js → + React（`window.wbUiKit`）
        // runtime/islands/ui.css → （Tailwind ）
        // runtime/providers/*.png → 
        //
        // in `{pack}` by before ：`runtime` not can id（ not `/ui/`，
        // by not will ， up not in will ）。
        // not protected by and up that 。
        .route("/plugins/runtime/{*rest}", get(api::plugin_theme::serve));

    // need ：Node for this path use checkApiKey
    let protected = Router::new()
        // GET /api/session（）： from public —— will 
        // 、、 by in etc （ session.rs），
        // request by 1 Key（gateway::request_builder） not 。
        .route("/api/session", get(api::session::get_session))
        .route(
            "/api/config",
            get(api::config_api::get_config)
                .post(api::config_api::post_config)
                // PUT is Agent2API add ****（ §5 
                // GET/PUT）： 1 ， as 1 。 has before POST，
                // 1 not ； new before by §5 PUT can use 。
                .put(api::config_api::post_config),
        )
        .route(
            "/api/logs",
            get(api::logs_api::query_logs).delete(api::logs_api::clear_logs),
        )
        .route("/api/logs/stats", get(api::logs_api::stats_logs))
        .route("/api/logs/download", get(api::logs_api::download_logs))
        // Node for /api/logs* unknownpath（ `/api/logs/` up GET/DELETE）
        // return API 404（`{success:false,error:"Not found: <METHOD> <path>"}`），
        // not is OpenAI 
        .route("/api/logs/", any(api::logs_api::not_found))
        .route("/api/logs/{*rest}", any(api::logs_api::not_found))
        // ── statistics and （ 7 after ， Node for ）──
        // /api/stats/* and /api/retention protected：
        // can to request（、、token usage） and can / ，
        // and /api/logs ，must and 1 API Key check。
        // unknown /api/stats/* pathreturn 404（ logs_api::not_found ，
        // not is OpenAI ）—— 1 before down 404 1 。
        .route("/api/stats/summary", get(api::stats_api::stats_summary))
        .route(
            "/api/stats/requests",
            get(api::stats_api::stats_requests).delete(api::stats_api::clear_stats_requests),
        )
        // down （ / ）。 in `/api/stats/requests`
        // after 、`/api/stats/{*rest}` ** before **： is 404， will this 
        // to 404（ is ，path）
        .route(
            "/api/stats/requests/filters",
            get(api::stats_api::stats_request_filters),
        )
        // request （ for ）。 need in before 
        .route(
            "/api/stats/requests/raw",
            get(api::stats_api::stats_request_raw),
        )
        // 1 in request（ 「request」 by ）。POST and 
        // in before —— by and up 
        .route(
            "/api/stats/requests/terminate",
            post(api::stats_api::stats_request_terminate),
        )
        // statistics（ will / / use / ）
        .route(
            "/api/stats/requests/clear-preview",
            get(api::stats_api::stats_clear_preview),
        )
        // （checkpoint + VACUUM， after ； 409）
        .route(
            "/api/stats/requests/compact",
            post(api::stats_api::compact_stats_db),
        )
        // none `/api/stats` 404： this before down no has 「」
        // （ has path）， 1 before down 404 must 1 ——
        // before path when to if is OpenAI ， will as is 
        .route("/api/stats", any(api::stats_api::not_found))
        .route("/api/stats/", any(api::stats_api::not_found))
        .route("/api/stats/{*rest}", any(api::stats_api::not_found))
        // ：GET ，PUT（）update and ****。
        // and /api/config ：config and ， this ——
        // in is as use （）， not this config 
        // 「none use set」， 1 config not this 。
        .route(
            "/api/retention",
            get(api::stats_api::get_retention).put(api::stats_api::put_retention),
        )
        // request：GET 「 / 」，PUT（）update。
        // and /api/retention 1 not /api/config：
        // config is Node for （apiKey / locale）， is add as 
        // set； this no has 「」 use ， after for down 1 failurerequest。
        .route(
            "/api/retry",
            get(api::retry_api::get_retry).put(api::retry_api::put_retry),
        )
        // up request when （）：GET 、PUT（）update。
        // and /api/retry 1 、 1 by ： after for down 1 request
        // （ when by client， its in when up ）。
        .route(
            "/api/timeouts",
            get(api::timeouts_api::get_timeouts).put(api::timeouts_api::put_timeouts),
        )
        // etc （ / seconds）： up （ before is Qoder ）
        // in when 「 N seconds after 」，。
        // and /api/timeouts 1 ： after for down 1 request。
        .route(
            "/api/queue",
            get(api::queue_api::get_queue).put(api::queue_api::put_queue),
        )
        // by （ / / / / ）：GET 、PUT（）
        // update， after for down 1 request；queue is can 
        // （「 by 」 ， and 1 ）。
        .route(
            "/api/routing",
            get(api::routing_api::get_routing).put(api::routing_api::put_routing),
        )
        .route("/api/routing/queue", get(api::routing_api::get_routing_queue))
        .route("/api/routing/workspace", get(api::routing_workspace::get_workspace))
        .route("/api/routing/groups", get(api::routing_groups::get_groups).put(api::routing_groups::put_group))
        .route("/api/routing/preview", post(api::routing_groups::preview))
        .route("/api/routing/profile", axum::routing::put(api::routing_workspace::put_profile))
        .route("/api/routing/sticky", get(api::routing_workspace::get_sticky))
        .route("/api/routing/sticky/clear", post(api::routing_workspace::clear_sticky))
        .route("/api/routing/replays", get(api::routing_workspace::get_replays))
        .route("/api/routing/flow", get(api::routing_flow::get_flow))
        .route("/api/routing/trace", get(api::routing_trace::get_trace))
        .route("/api/routing/trace/request", get(api::routing_trace::get_trace_request))
        // ── （set「 use → 」）──
        // GET/PUT ；traffic is by id （ not return
        // ， debug_api module header）。 protected： up URL and request。
        .route(
            "/api/debug",
            get(api::debug_api::get_debug).put(api::debug_api::put_debug),
        )
        .route("/api/debug/traffic", get(api::debug_api::get_traffic))
        // ── （set「」）──
        // （ in 、、）。 protected and /api/retention
        // ： can to 「 this up has and request」 this 。
        // before by （relocate / progress）「 1 directory」
        // 1 delete， by `api::storage_api` module header。
        .route("/api/storage", get(api::storage_api::get_storage))
        // ── set「」──
        // 1 ： / / path / / （ api::system_info
        // module header）。 protected —— directory、 can filepath and Key 
        // ； will to can 。
        // `?hideUsername=1` is ****parameter（directory before `~`）， by after 
        // ， this 「」 to is after 。
        .route("/api/system-info", get(api::system_info::get_system_info))
        // ── （ old JSON/JSONL → 1 SQLite ）──
        // when ** not **， by use in 「」（ need ：must use 
        // 、 old ）。 protected：run will 
        // （、、statistics、config）， and /api/config 。
        .route("/api/upgrade", get(api::upgrade_api::get_upgrade))
        .route("/api/upgrade/run", post(api::upgrade_api::run_upgrade))
        // ── （set「」）──
        // need old and its will ； use and enabled。
        // protected：must（ or Key） up 「 」。
        .route(
            "/api/panel/account",
            get(api::panel::panel_account),
        )
        .route(
            "/api/panel/change-password",
            post(api::panel::panel_change_password),
        )
        // ── / （， old SQL ）──
        // protected： path，，
        // is /api/config ；file by after by path
        // （file not HTTP， api::backup_api module header）。
        .route("/api/backup/export", post(api::backup_api::export_backup))
        .route("/api/backup/import", post(api::backup_api::import_backup))
        // （ before ）： is 1 use ， in when return，
        // no has this can 1 not need 「 in …」。
        .route("/api/backup/progress", get(api::backup_api::restore_progress))
        // （ Docker / LAN ） ： can down 、 up file，
        // must HTTP。upload requestlimit —— is 
        // 32MB， up MB； this 1 ， by axum 
        // then （ after ）， `gateway_router` 1 。
        .route("/api/backup/download", get(api::backup_api::legacy_download_backup).post(api::backup_api::download_backup))
        .merge(
            Router::new()
                .route("/api/backup/upload", post(api::backup_api::upload_backup))
                .layer(axum::extract::DefaultBodyLimit::max(
                    api::backup_api::MAX_BACKUP_UPLOAD,
                )),
        )
        // ── （set「 → 」）──
        // protected： up and Key to ， down 
        // ， —— is /api/config 。
        // up / down need MB， as when （
        // `commands::cloud_sync_upload` / `cloud_sync_download`）。
        .route(
            "/api/cloud-sync",
            get(api::cloud_sync::get_settings).put(api::cloud_sync::put_settings),
        )
        .route("/api/cloud-sync/test", post(api::cloud_sync::post_test))
        .route("/api/cloud-sync/remote", get(api::cloud_sync::get_remote))
        .route("/api/cloud-sync/upload", post(api::cloud_sync::post_upload))
        .route("/api/cloud-sync/download", post(api::cloud_sync::post_download))
        .route(
            "/api/cloud-sync/import-key",
            post(api::cloud_sync::post_import_key),
        )
        // ── （ for workbuddy-account-routes.mjs）──
        // use any(...) （none + ），/path to 
        // api::accounts::dispatch —— this is as Node tryHandle 
        // （ that ：`DELETE /api/accounts/export` will id）。
        // axum by + {id} will this 405， and Node 。
        // ：`/api/accounts/` in axum `{*rest}` not up 
        // （ need 1 empty）， not will to 404（OpenAI ），
        // Node for response is API 404
        .route("/api/accounts", any(api::accounts::accounts_entry))
        .route("/api/accounts/", any(api::accounts::accounts_entry))
        .route("/api/accounts/{*rest}", any(api::accounts::accounts_entry))
        // ── Agent（ AI client config）──
        // path：client id body（ and 1 ， by 
        // api::proxies module header —— will and 404 ）。
        // and api::agents module header。
        .route("/api/agents", get(api::agents::list_agents))
        .route("/api/agents/profiles", post(api::agents::save_profiles))
        .route("/api/agents/apply", post(api::agents::apply_profile))
        .route("/api/agents/preview", post(api::agents::preview_profile))
        .route("/api/agents/restore", post(api::agents::restore_client))
        .route("/api/agents/import", post(api::agents::import_client))
        .route("/api/agents/{id}/resources", get(api::agent_resources::get))
        .route("/api/agents/{id}/resources/preview", post(api::agent_resources::preview))
        .route("/api/agents/{id}/resources/apply", post(api::agent_resources::apply))
        // ── （「」 ）──
        // up 、 and when 、 need ，
        // protected。 when （/skill、/mcp、/api/web-search） in ，
        // `gateway_router` 。
        .route("/api/tools/servers", get(api::tools::get_servers))
        .route("/api/tools/servers/{id}", axum::routing::patch(api::tools::patch_server))
        .route("/api/tools/providers", get(api::tools::get_providers))
        .route("/api/tools/api-keys", get(api::tools::list_api_keys))
        .route(
            "/api/tools/provider-selection",
            get(api::tools::get_selection).put(api::tools::put_selection),
        )
        .route(
            "/api/tools/providers/{provider}",
            axum::routing::put(api::tools::put_provider),
        )
        .route(
            "/api/tools/providers/{provider}/key",
            axum::routing::delete(api::tools::delete_provider_key),
        )
        .route("/api/tools/providers/{provider}/test", post(api::tools::test_provider))
        .route(
            "/api/tools/vision/config",
            get(api::tools::get_vision_config)
                .put(api::tools::put_vision_config)
                .delete(api::tools::delete_vision_config),
        )
        .route("/api/tools/vision/config/test", post(api::tools::test_vision))
        .route("/api/tools/install-targets", get(api::tools::install_targets))
        .route("/api/tools/install-info", get(api::tools::install_info))
        .route("/api/tools/install-skill", post(api::tools::install_skill))
        .route("/api/tools/install-mcp", post(api::tools::install_mcp))
        .route("/api/tools/uninstall-skill", post(api::tools::uninstall_skill))
        .route("/api/tools/uninstall-mcp", post(api::tools::uninstall_mcp))
        .route("/api/tools/mcp-bridge/vision.js", get(api::tools::vision_bridge_script))
        // ── （ / / ）──
        // protected： is use config， and will use that 1 。
        // request by Key， will ， not 。
        // api::sidebar_layout module header（`{layout: ...}`）。
        .route(
            "/api/sidebar-layout",
            get(api::sidebar_layout::get_layout).put(api::sidebar_layout::put_layout),
        )
        // ── （Clash Verge when + + ）──
        // /api/proxies and /api/proxies/pool* （ /api/proxies/zzz） not 
        // → to 404 ， and Node 「 before not → 」 1 。
        // pool is path（id body / query， not use {id} ——
        // by api::proxies module header）， in proxies_entry 。
        .route("/api/proxies", any(api::accounts::proxies_entry))
        .route("/api/proxies/test", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool/update", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool/enable", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool/remove", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool/test", any(api::accounts::proxies_entry))
        .route("/api/proxies/pool/sync-clash", any(api::accounts::proxies_entry))
        .route("/api/proxies/settings", any(api::accounts::proxies_entry))
        // ── already down return， not request or ──
        .route("/api/subscriptions", any(retired_proxy_feature))
        .route("/api/subscriptions/{*rest}", any(retired_proxy_feature))
        // ── / to / （ for server.mjs 871-911 ）──
        // in protected（Node checkApiKey），
        // failure when body is OpenAI （ that in server.mjs try ）
        .route("/api/usage", get(api::billing::get_usage))
        .route("/api/checkin/status", get(api::billing::checkin_status))
        .route("/api/checkin", post(api::billing::claim_checkin))
        .route(
            "/api/checkin/claim-and-report",
            post(api::billing::claim_and_report),
        )
        .route("/api/activity/banner", get(api::billing::activity_banner))
        .route(
            "/api/activity/ambassador",
            get(api::billing::activity_ambassador),
        )
        // WorkBuddy （ to in 「 to 」）
        .route(
            "/api/checkin-keepalive",
            get(api::billing::get_keepalive).post(api::billing::save_keepalive),
        )
        // WorkBuddy （ / / +）
        .route("/api/checkin-activity", post(api::billing::run_activity))
        // ── will and ──
        // must； not is any admin.action 。
        .route("/api/session/login/methods", get(api::session::login_methods))
        .route("/api/session/login/start", post(api::session::login_start))
        .route("/api/session/login/wait", get(api::session::login_wait))
        .route("/api/session/login/cancel", post(api::session::login_cancel))
          // ： or 
          // POST to this （Tauri not can Electron that in will ，
          // api::session::login_callback ）。 and its login/* 1 in 
          // protected —— ，must API Key。
        .route("/api/session/login/callback", post(api::session::login_callback))
        // AutoClaw （** not is **， api::session module header）：
        // up no has / this ， is 「 → use token」request，
        // not need need and 。 protected —— ，
        // and up callback 1 。
        .route(
            "/api/session/login/sms/send",
            post(api::session::login_sms_send),
        )
        .route(
            "/api/session/login/sms/verify",
            post(api::session::login_sms_verify),
        )
        // Loomy （、 protected：）。
        // and up that for is ： / / error not ，
        // 1 will need need in handler by provider 。
        .route(
            "/api/session/login/loomy/sms/send",
            post(api::session::login_loomy_sms_send),
        )
        .route(
            "/api/session/login/loomy/sms/verify",
            post(api::session::login_loomy_sms_verify),
        )
        // Tabbit （、 protected）。
        // unsupported， `login.sms-*` 。
        .route(
            "/api/session/login/tabbit/sms/send",
            post(api::session::login_tabbit_sms_send),
        )
        .route(
            "/api/session/login/tabbit/sms/verify",
            post(api::session::login_tabbit_sms_verify),
        )
        // AutoClaw OAuth （**** ）。 protected
        // —— need parameter up 、 and 1 ， and sms/*
        // 。（loopback ）** not **： z.ai
        // down ， by `pollIntervalMs` `login.poll`
        // （ `autoclaw/web_login.rs`）。
        .route(
            "/api/session/login/oauth/captcha-config",
            get(api::session::login_oauth_captcha_config)
                .post(api::session::login_oauth_captcha_config),
        )
        .route(
            "/api/session/login/oauth/start",
            post(api::session::login_oauth_start),
        )
        .route("/api/session/refresh", post(api::session::session_refresh))
        .route("/api/session/logout", post(api::session::session_logout))
        .route("/auth/login", post(api::session::auth_login))
        .route("/auth/logout", post(api::session::auth_logout))
        // ── for （chat/completions / responses / messages /
        // messages/count_tokens） in this ， when 
        // [`gateway_router`]（ API Key，）；
        // /v1/models that 。 `router()` and ，
        // as and before 1 。
        // new （ by ）： will ** up **（ directory），
        // and /v1/chat/completions 1 must API Key；GET /v1/models that 
        // not （ is not ， api::models module header）。
        // return 2xx：， by this module header
        .route("/api/models/refresh", post(api::models::refresh_models))
        // （ / / ） and Key ： is config ， protected
        .route("/api/models/manage", get(api::model_manage::get_manage))
        .route("/api/models/compact-routes", get(api::compact_routes::get_routes).put(api::compact_routes::put_routes))
        .route("/api/models/state", post(api::model_manage::set_state))
        .route("/api/models/mappings", post(api::model_manage::add_mapping))
        .route("/api/models/mappings/remove", post(api::model_manage::remove_mapping))
        // （ up directory no has ）
        .route("/api/models/custom", post(api::model_manage::add_custom))
        .route("/api/models/custom/remove", post(api::model_manage::remove_custom))
        // can （ for down that ；，
        // /api/custom-providers/models ， this handler ）
        .route("/api/models/capabilities", post(api::model_manage::set_capabilities))
        // models.dev （ up no can and by ）：
        // / in set， new that will ****（models.dev JSON），
        // and /api/models/refresh protected。
        .route(
            "/api/models/metadata",
            get(api::model_meta::get_metadata).put(api::model_meta::put_metadata),
        )
        .route("/api/models/metadata/refresh", post(api::model_meta::refresh_metadata))
        // （ 「」）： will ** up 、**，
        // and /v1/chat/completions 1 ，must protected。
        // failurereturn 2xx（ up error in response status / error ，
        // by api::model_test module header）
        .route("/api/models/test", post(api::model_test::run_model_test))
        // ── AI （「AI 」： for + will + set）──
        // for that will ** up 、**（ and /v1/chat/completions、
        // /api/models/test 1 ），set that config， 1 protected。
        // for （SSE）， before must （none and 60 seconds when ）
        // fetch —— in by not is /v1/*， is as not need use 
        // CORS ， by api::assistant module header。
        .route(
            "/api/assistant/config",
            get(api::assistant::get_config).put(api::assistant::put_config),
        )
        .route("/api/assistant/chat", post(api::assistant::chat))
        .route("/api/assistant/images/generations", post(api::assistant_images::generations))
        .route("/api/assistant/images/edits", post(api::assistant_images::edits))
        .route("/api/assistant/images/files/{id}", get(api::assistant_images::download))
        // will （ that for ）： / / / / empty。 is use and 
        // ， and set， 1 protected。
        // `GET|DELETE /api/assistant/sessions` is 1 by （empty
        // DELETE）， `{id}` that —— in axum not 
        // （ `{id}` parameter）。
        .route(
            "/api/assistant/sessions",
            get(api::assistant::list_sessions)
                .post(api::assistant::save_session)
                .delete(api::assistant::clear_sessions),
        )
        .route("/api/assistant/sessions/{id}/fork", post(api::assistant_history::fork))
        .route("/api/assistant/sessions/{id}/title", post(api::assistant_history::title))
        .route(
            "/api/assistant/sessions/{id}",
            get(api::assistant::get_session).delete(api::assistant::delete_session),
        )
        // ── （ use up ： + ）──
        // and /api/models/manage ：config（customProviders ） and 「 new 」
        // will ， protected。 not this —— client
        // /api/accounts（ its add custom- before provider id），
        // in `account_store::custom_accounts` up 。
        .route(
            "/api/custom-providers",
            get(api::custom_providers::get_custom_providers)
                .post(api::custom_providers::create_custom_provider),
        )
        .route(
            "/api/custom-providers/update",
            post(api::custom_providers::update_custom_provider),
        )
        .route(
            "/api/custom-providers/remove",
            post(api::custom_providers::remove_custom_provider),
        )
        // ── 「 from its 」（「」 ）──
        // cc-switch ，response API Key （ need need ），
        // and up ， protected。
        .route(
            "/api/import/cc-switch",
            get(api::import_sources::scan_cc_switch),
        )
        // （）： / server up 。
        // protected by and up ；fetch-models will up 
        // （ 1 GET {baseUrl}/models）， and /api/models/refresh 。
        .route(
            "/api/custom-providers/models",
            post(api::custom_providers::set_custom_models),
        )
        .route(
            "/api/custom-providers/fetch-models",
            post(api::custom_providers::fetch_custom_models),
        )
        .route("/api/model-access/models", get(api::model_access::models))
        .route("/api/model-access/keys", get(api::model_access::list_keys))
        .route("/api/keys", get(api::keys_api::list_keys).post(api::keys_api::create_key))
        .route("/api/keys/{id}", patch(api::keys_api::update_key).delete(api::keys_api::delete_key))
        // ── （ 1 「」）──
        // provider and in use 。 this 
        // ** protected**： etc any， and 。
        // Bytes ： has limit after 32 MiB。
        .route("/api/plugins", get(api::plugins::list))
        .route("/api/plugins/settings", get(api::plugins::settings).patch(api::plugins::save_settings))
        .route("/api/plugins/progress", get(api::plugins::progress))
        .route(
            "/api/plugins/install",
            post(api::plugins::install).route_layer(axum::extract::DefaultBodyLimit::max(
                api::plugins::MAX_PACK_UPLOAD_BYTES,
            )),
        )
        .route("/api/plugins/uninstall", post(api::plugins::uninstall))
        .route("/api/plugins/enable", post(api::plugins::set_enabled))
        .route("/api/plugins/provider", post(api::plugins::set_provider))
        // use update： new / 1 。 protected ——
        // will down and new ， and 。
        .route("/api/plugins/update", get(api::plugins::check_update).post(api::plugins::run_update))
        // ： has can / from （ not need already ）。
        // 「」 can spawn any， and /api/plugins/install 。
        .route("/api/plugins/release", get(api::plugins::release))
        .route("/api/plugins/install-from-release", post(api::plugins::install_from_release))
        // and request，must use 。
        .route(
            "/api/sanitize",
            get(api::sanitize::get_sanitize).put(api::sanitize::put_sanitize),
        )
        // ── ──
        // and /api/debug （GET / PUT ）， protected：
        // request developer need not need 1 system（ is to 
        // up request， and that ）。
        // `core::upstream::payload` send_body 1 。
        .route(
            "/api/developer-role-compat",
            get(api::role_compat::get_role_compat).put(api::role_compat::put_role_compat),
        )
        // ── Cline ──
        // and /api/sanitize （GET / PUT ，response is new ），
        // protected：request「 not client」，。
        // default and and in ****（ use that ），
        // `api::cline_headers` module header。
        .route(
            "/api/cline/headers",
            get(api::cline_headers::get_cline_headers).put(api::cline_headers::put_cline_headers),
        )
        // ── ──
        // and /api/debug （GET / PUT ）， protected：
        // `/v1/*` need not need request（ api::cors module header）。
        // 1 —— etc 「 up 、」 can to ，
        // default， and will 。
        .route(
            "/api/cors",
            get(api::cors::get_cors).put(api::cors::put_cors),
        )
        // ── ──
        // and /api/sanitize、/api/debug （GET / PUT ），
        // protected：「 this can 」 down need not need 
        // content —— is client to ， and that 。
        // default（ config::KEY_STRIP_NEWLINE_KEEPALIVE ）； in can is 
        // as 「 in up 1 、 not for 」 not need need new 。
        .route(
            "/api/keepalive-strip",
            get(api::keepalive::get_keepalive_strip).put(api::keepalive::put_keepalive_strip),
        )
        // ── ──
        // and /api/keepalive-strip 1 （GET / PUT 、default、 is client
        // as ）， 1 ： 1 + 1 has （limit）——
        // limit is that （ is ），
        // 1 request up ， in 1 up 。
        // protected by and that ： will as 1 clientrequest if 
        // up 、（ and core::upstream::continuation）。
        .route(
            "/api/auto-continue",
            get(api::auto_continue::get_auto_continue).put(api::auto_continue::put_auto_continue),
        )
        // ── will （default）──
        // and /api/auto-continue 1 （GET / PUT 、 is to up 
        // ）： + / / 1 in 1 up 。 protected
        // by and /api/sanitize —— is to up request（ can use 
        // in for ， / 、 not ）。
        .route("/api/client-identification", get(api::client_identification::get_settings).put(api::client_identification::put_settings))
        .route("/api/client-identification/preview", post(api::client_identification::preview_settings))
        .route(
            "/api/image-compress",
            get(api::image_compress::get_image_compress)
                .put(api::image_compress::put_image_compress),
        )
        // ── ──
        // and /api/debug （GET / PUT ）， protected：
        // / is need ALTCHA proof-of-work，。
        .route(
            "/api/captcha",
            get(api::captcha::get_captcha).put(api::captcha::put_captcha),
        )
        // ── ──
        // and up that ** not is 1 **： up is （ALTCHA）， this is 
        // up ZCode for need （、
        // to by request use ）， `api::zcode_captcha` module header。
        // protected： is ， and will as 。
        .route(
            "/api/zcode/captcha",
            get(api::zcode_captcha::get_captcha).post(api::zcode_captcha::push_captcha),
        )
        // use ： not ， and 。
        .route(
            "/api/zcode/claim-verification",
            post(api::zcode_claim_verification::verification),
        )
        // set（ / / limit）： and up that 
        // `api::zcode_captcha_pool` module header。stats response will this 
        // to ， after down 1 。 protected。
        .route(
            "/api/zcode/captcha-pool",
            get(api::zcode_captcha_pool::get_captcha_pool)
                .put(api::zcode_captcha_pool::put_captcha_pool),
        )
        // ── ZCode client（set can ）──
        // up by this down not down ，** after when is 
        // empty**（ not is ）—— to use 1 can ，
        // `api::zcode_version` module header。 protected：request。
        .route(
            "/api/zcode/version",
            get(api::zcode_version::get_version).put(api::zcode_version::put_version),
        )
        // ── and ──
        // and /api/sanitize as 「」 ， not （ + file
        // path + 1 need 1 return）， 1 ， api::prompt
        // module header。 protected： can system ，。
        .route(
            "/api/prompt",
            get(api::prompt::get_prompt).put(api::prompt::put_prompt),
        )
        // ── when to （ for server.mjs 726-746 ）──
        // in Node try ，failure OpenAI body（ run 
        // 「 to in in ，」）—— by api::auto_checkin 。
        // Node is 「path === '/api/auto-checkin' || path === '/api/auto-checkin/run'」，
        // `/api/auto-checkin/` and `/api/auto-checkin/xxx` to 404，
        // this this path。
        .route(
            "/api/auto-checkin",
            get(api::auto_checkin::get_state).post(api::auto_checkin::configure),
        )
        .route("/api/auto-checkin/run", post(api::auto_checkin::run_now))
        // ── when to ──
        // total up ； CRUD / in this （ by 
        // api::checkin_schedules module header）。 protected：config + to ，
        // and /api/auto-checkin 。 any(...) ，
        // and /api/scheduled-tasks 1 （ 1 404， not to 405 ）。
        .route("/api/checkin-schedules", any(api::checkin_schedules::entry))
        .route("/api/checkin-schedules/", any(api::checkin_schedules::entry))
        .route(
            "/api/checkin-schedules/{*rest}",
            any(api::checkin_schedules::entry),
        )
        // ── to ──
        // GET / DELETE empty（ all=1， logs then ；delete and 
        // use 1 ）。 protected： has and ，
        // and /api/logs 1 。
        .route(
            "/api/checkin-records",
            get(api::checkin_records::query_records).delete(api::checkin_records::clear_records),
        )
        .route("/api/checkin-records/", any(api::logs_api::not_found))
        .route("/api/checkin-records/{*rest}", any(api::logs_api::not_found))
        // ── to in ──
        // 1 ： and 「 already 」 use to 1 for 
        // （CHECKIN_PROVIDERS ∩ supports_checkin）， api::checkin_center module header。
        // protected： can and its to when ， and /api/accounts 1 。
        .route("/api/checkin-center", get(api::checkin_center::get_center))
        // ── when （）──
        // GET / POST and when / POST run 1 。
        // and /api/auto-checkin 1 （），****：
        // and to not ， and when （ core::growth_scheduler）。
        // protected：POST run will up （ / ）， and to 。
        .route(
            "/api/travel-schedule",
            get(api::growth_schedule::get_state).post(api::growth_schedule::configure),
        )
        .route(
            "/api/travel-schedule/run",
            post(api::growth_schedule::run_now),
        )
        // ── Qoder UMID （）──
        // GET / POST （only Linux， api::qoder_umid module header）。
        // protected： will to npm down and 1 can ， and 
        // /api/scheduled-tasks 。
        .route(
            "/api/qoder-umid",
            get(api::qoder_umid::get_status).post(api::qoder_umid::get_status),
        )
        .route("/api/qoder-umid/install", post(api::qoder_umid::install))
        // ── when （ / directory new / before new ）──
        // protected： can after after （ 1 minutes will 
        // up ）， and up new ， and /api/retention 。
        //
        // is any(...)、 to `api::scheduled_tasks::entry` ——
        // and /api/accounts 1 ： axum by will 
        // 「 already path + 」 405 ， this before down 1 to 404。
        // ：`/api/scheduled-tasks/` in `{*rest}` not up 
        // （ need 1 empty）， not will to 404。
        .route("/api/scheduled-tasks", any(api::scheduled_tasks::entry))
        .route("/api/scheduled-tasks/", any(api::scheduled_tasks::entry))
        .route(
            "/api/scheduled-tasks/{*rest}",
            any(api::scheduled_tasks::entry),
        )
        // ── update（ for server.mjs 749-773 ）──
        // Node is `path.startsWith('/api/update/')` before ： in after for ，
        // not then to 404（ not success ）。 use {*rest} +
        // dispatch this （ by will 「 already path + 」
        // 405， and Node ）。
        // Note: `/api/update`（ no has ） in Node not in before → 404，
        // this `{*rest}`（ need 1 empty）， as 1 。
        .route("/api/update/{*rest}", any(api::update::entry))
        // ── 7 when need ─────────────────────────────
        // API to ****（ 1-6 health/session/config/logs/
        // accounts/proxies/billing/chat/sanitize/auto-checkin/update），
        // by not has 。 7 need ：
        // ① from package.json / resources old node after 
        // （server.cjs + node.exe ）， tauri.conf.json 
        // resources and bundle config；
        // ② file and server/mod.rs `#![allow(dead_code)]` ，
        // by warning use ；
        // ③ and `.zcode/release-notes` update（ and asset 
        // need and update pickInstaller then ： setup .exe ）。
        .layer(middleware::from_fn(require_api_key));

    // in headless （set_ui_dir）： by Tauri ，
    // unknownpath 404。ui_dir is ，。
    let ui_dir = state.ui_dir().cloned();
    Router::new()
        .merge(public)
        .merge(protected)
        .fallback(move |request: Request| {
            let ui_dir = ui_dir.clone();
            async move {
                let Some(dir) = ui_dir else {
                    return not_found(request).await;
                };
                // → （ `panel_boot_redirect`）。
                // must in index.html before ： 1 will by app.js and 
                // and `/api/*`， when 401， etc empty 1 。
                if let Some(response) = panel_boot_redirect(&request) {
                    return response;
                }
                super::static_files::serve(request, dir).await
            }
        })
        // not ：Node no has 405 ， 1 to 404 ，
        // this 405 Not found ， as 1 
        .method_not_allowed_fallback(method_not_allowed)
        // response 1 gzip： islands and /api/session is ，
        // not will etc up seconds（ compression module header）。 ——
        // /v1/* is SSE ， not can in this 。
        .layer(middleware::from_fn(super::compression::gzip))
        // CORS and body limitmust in ：404 and errorresponse need CORS 
        .layer(middleware::from_fn(cors))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_SIZE))
        .with_state(state)
}

/// by ：`/v1/*`（ + + token ） and `/health`。
///
/// down （`AGENT2API_PROXY_PORT`）—— for 
/// OpenAI ， no has and API；， /
/// compose this can 。 [`router`] and and 。
pub fn gateway_router(state: ServerState) -> Router {
    // /health（ and ） and /v1/models ：
    // /v1/models is ，Node not API Key —— OpenAI client in 
    // config key before （README /v1/models in ）。
    //
    // ── Why /health 1 by 、 and none CORS ────────────
    // APK 「」 need 1 /health for （
    // mobile/src/pages/ServersPage.tsx）。APK origin is 
    // `https://localhost`， this 1 is request —— no has `Access-Control-Allow-
    // Origin` when will response， before to fetch failure， and 「」
    // none。 /health not 、 `{"status":"ok"}`， to CORS
    // not new ；`/v1/models` not （），
    // 「」 that 。
    let health = Router::new()
        .route("/health", get(api::health::handle))
        .layer(middleware::from_fn(cors))
        .with_state(state.clone());

    let open = Router::new()
        .route("/v1/models", get(api::chat::list_models))
        .route("/v1/images/files/{id}", get(api::images::download))
        // and REST ：GET is ；POST by handler
        // Key（ and /skill、/mcp 1 ）， in not 1 in 
        // this 1 —— guarded will `require_api_key` 「/api/ need will 
        // or not Key」（ not request，Key none）。
        .route("/api/web-search", get(api::tool_runtime::api_metadata))
        .route("/api/web-search/search", post(api::tool_runtime::api_search))
        .route("/api/web-search/fetch", post(api::tool_runtime::api_fetch))
        .route("/skill/web-search", get(api::tool_runtime::skill_metadata))
        .with_state(state.clone());
    // + Anthropic token ： API Key。
    // and chat/completions ： 1 ， in 
    // （ api::protocol and core::protocol module header）。** is **
    // for not is API —— client base_url /v1 can use 。
    // count_tokens：Claude Code in down will up down ， up no has 
    // for can ， this to `estimated: true` （ this ）—— to 404
    // will client by 「unsupported」， 1 。
    let guarded = Router::new()
        .route("/v1/chat/completions", post(api::chat::chat_completions))
        .route("/v1/responses", post(api::protocol::responses_endpoint))
        .route("/v1/messages", post(api::protocol::messages_endpoint))
        .route("/v1/messages/count_tokens", post(api::chat::count_tokens))
        .route("/v1/images/generations", post(api::images::generations))
        .route("/v1/images/edits", post(api::images::edits))
        // Consumer self-service query：returncurrent Key 、scope、limit and usage/remaining。
        // Read-only; does not consume request quota（middleware uses count=false for this path）。
        .route("/v1/key/info", get(api::model_access::key_info))
        // has ：/skill and /mcp not in /v1 not in /api down ，
        // `require_api_key` for not （ this in ），
        // must by handler —— this is `api::tool_runtime` 。
        .route("/skill/web-search/search", post(api::tool_runtime::skill_search))
        .route("/skill/web-search/fetch", post(api::tool_runtime::skill_fetch))
        .route(
            "/mcp/web-search",
            post(api::tool_runtime::mcp_web_search).get(api::tool_runtime::mcp_unsupported),
        )
        .route(
            "/mcp/vision",
            post(api::tool_runtime::mcp_vision).get(api::tool_runtime::mcp_unsupported),
        )
        .layer(middleware::from_fn(require_api_key))
        .with_state(state);
    health.merge(open.merge(guarded))
        // CORS in ：、404 and errorresponse need up CORS （ by ）。
        // in set「」 after —— cors_gateway 。
        // /health that none CORS in ， not this （ up ）。
        .layer(middleware::from_fn(cors_gateway))
        // must disable：axum for no DefaultBodyLimit by 
        // 2MB（axum-core Request::with_limited_body）， up down + base64 
        // request will 413，clientrequest will 。 this 
        // disable in and down by max —— is insert，
        // after 。limit to up 。
        .layer(axum::extract::DefaultBodyLimit::disable())
}

/// by ： Node `Not found: <METHOD> <path>`。
/// Note use errors::not_found_response GatewayError —— Node 404 no has type 。
async fn not_found(request: Request) -> Response {
    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();
    logging::verbose("[HTTP]", &format!("← 404 {method} {path}"));
    errors::not_found_response(&method, &path)
}

/// not （e.g. GET /api/config to already path up ）：
/// Node will 404 ， this 。
async fn method_not_allowed(request: Request) -> Response {
    not_found(request).await
}

/// CORS in 。
///
/// response not is use tower-http CorsLayer：Node is in ** **response up 
/// none down this （including 404、error、SSE）， use in ，
/// as this 1 tower-http。
async fn cors(request: Request, next: Next) -> Response {
    // 204， not （ for Node `if (req.method === 'OPTIONS')`）
    if request.method() == Method::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        attach_cors(response.headers_mut());
        return response;
    }

    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();
    // down request（ for Node `if (opts.verbose && !path.startsWith('/v1/'))`）
    // —— has AGENT2API_VERBOSE=1（ old WORKBUDDY_VERBOSE can ），
    // is 1 
    if logging::is_verbose() && !path.starts_with("/v1/") {
        let query = request.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
        logging::verbose("[HTTP]", &format!("← {method} {path}{query}"));
    }

    let mut response = next.run(request).await;
    attach_cors(response.headers_mut());
    response
}

/// （`/v1/*`） CORS in ： has set 「」 after 。
///
/// ── Why need 1 in ─────────────────────────────
/// [`cors`] in [`panel_router`] ， axum `Router::layer` **used for use 
/// when already in by **：`router()` is `panel_router().merge(gateway_router())`，
/// and `/v1/*` not to that 1 —— is from not ，client
/// OPTIONS will to API Key in up 401 （ by not 
/// `Authorization` ）， to 「none API」。 this 1 CORS as 
/// to ，** by **（default， `config::KEY_CORS_ENABLED`）： is 
/// up 、 that 1 ，`*` not this none。
///
/// when not （ OPTIONS not ）： as and before 1 —— old to 
/// up 。 this is 「default」 。
async fn cors_gateway(request: Request, next: Next) -> Response {
    // ： this in /v1 request up 1 ， not config（
    // `config::cors_enabled` ）
    if !crate::server::config::cors_enabled() {
        return next.run(request).await;
    }
    cors(request, next).await
}

/// when ，（`None` = file）。
///
/// ── Why need in server ──────────────────────────────────────
/// in in ****（ not this need ，otherwise not to ），
/// is `/` when to index.html， by app.js（`refresh()`）
/// and （ load） and `/api/*` —— when 401。
/// that request for use none use （ down 1 seconds web_shim 401 /login），
/// is 「 will invalid」、 error。
///
/// ── Why use 302 + 1 ， not is will ──────────
/// in （fnOS docker this ） will localStorage 
/// `x-panel-token` request —— ****（、 new 、
/// `location.href`） not can can 。 by `session_valid` ：success →
/// `/`（ not ）→ server → 302 `/login` → 
/// 「 already 」→ `/` …… none， use in 1 to is 。
///
/// is 「 will valid ** or ** this 1 already is from 」
/// （`?from=login`）。 when index.html， API request will 
/// （shim from localStorage ）， by is 。
/// this is ** 1 ** ： when up ，server not ——
/// this parameter is to 1 index.html，`/api/*` 1 no 。
fn panel_boot_redirect(request: &Request) -> Option<Response> {
    // ：（app.js / ui.css / i18n/*） and must，
    // otherwise will ， not 。
    //
    // `/m` and `/m.html` is ， 1 use —— when ，
    // not is 1 will 401 empty。 has 1 （ its is XHR），
    // this after not will has 。
    let path = request.uri().path();
    let is_mobile = path == "/m" || path == "/m.html";
    if path != "/" && path != "/index.html" && !is_mobile {
        return None;
    }
    // ：GET/HEAD。 （curl、check） not in 。
    if !matches!(*request.method(), Method::GET | Method::HEAD) {
        return None;
    }
    // has 「」。must：
    // · ****（） not to this ；
    // · ****（`AGENT2API_PROXY_API_KEY` Key、 no has ）：
    // this localStorage API Key （shim 
    // `x-api-key`）， not —— in this 「 no will 」 will 
    // ， will not Key， etc in 。
    // that down 401 is down 「」， not 。
    if !crate::server::access::panel_auth_enabled() {
        return None;
    }
    if crate::server::access::session_valid(request.headers()) {
        return None;
    }
    // from ： this 1 （ by up ， in ）
    let from_login = request
        .uri()
        .query()
        .is_some_and(|query| query.split('&').any(|pair| pair == "from=login"));
    if from_login {
        return None;
    }
    // 302（ not is 301）： not this —— after 1 must can 。
    // up `next=/m`， to not is 
    // （ in login.html postLoginTarget ）。
    let location = if is_mobile { "/login?next=%2Fm" } else { "/login" };
    Some((StatusCode::FOUND, [(header::LOCATION, location)]).into_response())
}

/// CORS + 1 Private Network Access can （， not will failure；
/// failure when not is panic）
fn attach_cors(headers: &mut axum::http::HeaderMap) {
    if let Ok(value) = HeaderValue::from_str("*") {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
    }
    if let Ok(value) = HeaderValue::from_str(CORS_METHODS) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, value);
    }
    if let Ok(value) = HeaderValue::from_str(CORS_HEADERS) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, value);
    }
    // Private Network Access（Chromium 「 has need to can 」）：
    // origin is （`http://tauri.localhost`）， from fetch to 
    // `http://127.0.0.1:{port}` will WebView2 as private network request，
    // no has this when request will down —— to is 1 
    // `TypeError: Failed to fetch`，Reason not to 。 this 1 for 
    // client is empty（ not CORS ）， for already in by not 
    // —— this by by up `*` ， and this none。
    if let Ok(value) = HeaderValue::from_str("true") {
        headers.insert(
            axum::http::HeaderName::from_static("access-control-allow-private-network"),
            value,
        );
    }
}

/// use mustvalid Key， will not can 。
/// can will ； Key no has 。
/// `/v1/models` is ： not Key can ， Key when mustvalid。
async fn require_api_key(mut request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    if path.starts_with("/v1/") {
        // /v1/key/info is ， not requestquota/RPM（handler count=false ）。
        let count = path != "/v1/key/info";
        let access = match api::model_access::authenticate(request.headers(), "", count) {
            Ok(access) => access,
            Err(error) => return error.payload_response(),
        };
        request.extensions_mut().insert(access.source.api_key);
        if let Some(scope) = access.scope {
            crate::server::core::key_scope::attach(&mut request, scope);
        }
        return next.run(request).await;
    }
    let model_access = api::model_access::authenticate(request.headers(), "", true).ok();
    let panel_session = crate::server::access::session_valid(request.headers());
    if !panel_session {
        if let Some(access) = &model_access {
            let restricted = access.scope.as_ref().is_some_and(|scope| scope.restricts_models() || scope.restricts_providers());
            if restricted && !api::model_access::is_model_endpoint(&path) {
                return errors::GatewayError::with_status(403, " API Key not can ， use will ")
                    .with_code("management_access_denied").payload_response();
            }
        }
    }

    // will and not Key can ； Key。
    let panel_auth = crate::server::access::panel_auth_enabled();
    if panel_auth && path.starts_with("/api/") {
        if panel_session {
            return next.run(request).await;
        }
        if model_access.is_some() {
            // Key can （ / ）
            return next.run(request).await;
        }
        // request：， not client not can 。
        // and response cookie use 1 ， not and then 。
        let live_refresh = crate::server::access::has_live_refresh(request.headers());
        crate::server::access::diagnostics::log_rejection(
            request.headers(), request.method(), &path, live_refresh,
        );
        return errors::panel_login_required_response_with_refresh_state(live_refresh);
    }
    // ── （headless ， not ）────────────────
    // no （ no use ）：/api/* /api/panel/
    // before （status / setup / login / refresh / logout ——
    // ）， its 1 401 panel_login_required，web_shim to 
    // after will to /login 。 no has this ， before to 
    // 200 will ，「」 can 1 。
    //
    // must，otherwise will has ：
    // · ****（AGENT2API_PROXY_API_KEY Key、none）：
    // Key is this （/），must
    // Key ；
    // · in will ： not ， not request。
    // · `/api/panel/` before ： / / in this before 。
    if !panel_auth && crate::server::access::panel_gate() && path.starts_with("/api/") {
        if !path.starts_with("/api/panel/")
            && !crate::server::access::local_session_valid(request.headers())
            && model_access.is_none()
        {
            return errors::panel_login_required_response(request.headers());
        }
        return next.run(request).await;
    }

    if panel_session {
        return next.run(request).await;
    }
    if crate::server::config::current().active_api_keys().is_empty() {
        // none Key can ；， not in use 。
        request.extensions_mut().insert(crate::server::request_stats::ApiKeyIdentity {
            source: "none".to_string(),
            ..Default::default()
        });
        return next.run(request).await;
    }

    if let Some(access) = model_access {
        request.extensions_mut().insert(access.source.api_key);
        if let Some(scope) = access.scope {
            crate::server::core::key_scope::attach(&mut request, scope);
        }
        return next.run(request).await;
    }

    let method = request.method().as_str().to_string();
    logging::log("[Security]", &format!("❌ request: {method} {path}"));
    errors::unauthorized_response()
}

/// `Bearer ` before （ not ，`\s+` 1 empty）。
/// not when **return** —— and Node `String.replace` 1 。
pub(crate) fn strip_bearer_prefix(value: &str) -> &str {
    const PREFIX: &str = "bearer";
    if value.len() < PREFIX.len() {
        return value;
    }
    let Some(head) = value.get(..PREFIX.len()) else {
        return value;
    };
    let rest = &value[PREFIX.len()..];
    if !head.eq_ignore_ascii_case(PREFIX) {
        return value;
    }
    // `\s+` need 1 empty，"BearerX" not 、return
    let trimmed = rest.trim_start();
    if trimmed.len() == rest.len() {
        return value;
    }
    trimmed
}

/// JSON response（ not ）：/health and OpenAI error body use 。
/// Node for this response is for ， not is `{success,data}`。
pub fn raw_json(data: Value) -> Response {
    Json(data).into_response()
}

/// API success：`{ success: true, data: ... }`。
/// all /api/* by use ， and Node sendJson 1 。
pub fn ok_json(data: Value) -> Response {
    Json(json!({ "success": true, "data": data })).into_response()
}

/// API not data successresponse：`{ success: true }`
pub fn ok_empty() -> Response {
    Json(json!({ "success": true })).into_response()
}

/// JSON request；empty body as `{}`（ for Node 
/// `JSON.parse((await readRawBody(req)).toString('utf8') || '{}')`）。
pub fn parse_body(bytes: &[u8]) -> Result<Value, errors::GatewayError> {
    let text = String::from_utf8_lossy(bytes).trim().to_string();
    if text.is_empty() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    serde_json::from_str(&text)
        .map_err(|error| errors::GatewayError::bad_request(format!("request not is JSON: {error}")))
}

/// from 1 parameter（ by `&` 、 by `=` ， after ）。
///
/// return `None`；** not 「parameter not in 」 and 「parameter as empty」**（`?id=` and noneparameter
/// to `Some("")`）—— use if need by empty 1 ， `filter` 1 down ，
/// this is `accounts_usage` `id` when 。
///
/// Why to HTTP ：`/api/update` `current` and has 1 has ，
/// must 1 （ for JS `URLSearchParams`）， 1 will 。
/// has that has （ already path）， new use this 。
pub fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let (name, value) = match pair.split_once('=') {
            Some((name, value)) => (name, value),
            None => (pair, ""),
        };
        if name == key {
            return Some(percent_decode(value));
        }
    }
    None
}

/// （ for URLSearchParams ；`+` empty， and 1 ）。
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                match std::str::from_utf8(&bytes[index + 1..index + 3])
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok())
                {
                    Some(byte) => {
                        out.push(byte);
                        index += 3;
                    }
                    None => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

/// when ms（ `/api/logs` and `/api/stats/*` use ）。
///
/// **Why not in api **： by need by 1 `start` / `end`，
/// 1 will （ 1 、 1 not ），
/// this parameter in is 1 。 and `parse_body` 1 to HTTP 。
///
/// then （** 1 return None， by use ignore this ， not **）：
/// - empty / empty → None（ before empty when will `?start=`， this is ）
/// - by `i64` ，
/// - its `f64`（ JS `Date.now()/1000` ），
/// ****： when no has ， will 
/// - has （NaN / inf） and i64 → None
///
/// Why not ：`start` / `end` is ， as 1 （ / ）
/// ， not this no —— ， is 1 。
pub fn parse_query_ms(value: Option<&String>) -> Option<i64> {
    let text = value?.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(number) = text.parse::<i64>() {
        return Some(number);
    }
    let number = text.parse::<f64>().ok().filter(|item| item.is_finite())?;
    if number.fract() != 0.0 || number.abs() > i64::MAX as f64 {
        return None;
    }
    Some(number as i64)
}

#[cfg(test)]
mod cors_tests {
    //! （`/v1/*`）CORS as use 。
    //!
    //! ── Why need this as （）────────────────────────
    //! `cors` in `panel_router` ， axum `Router::layer` **used for use 
    //! when already in by **：`router()` is `panel_router().merge(gateway_router())`，
    //! and `/v1/*` not to that 1 。 is ：（）
    //! OPTIONS to API Key in up 401 （ by not 
    //! `Authorization`）， to 1 「none API」。 this as 
    //! ：default not （ and before 1 ）， after by 。
    //!
    //! use `oneshot` Router， not serve、 not 。

    use super::*;
    use axum::body::Body;
    use axum::routing::post;
    use tower::ServiceExt;

    /// and `gateway_router` by ：return 200，
    /// use `cors_gateway` in and request up as 。
    fn probe_router() -> Router {
        Router::new()
            .route("/v1/chat/completions", post(|| async { "ok" }))
            .layer(middleware::from_fn(cors_gateway))
    }

    async fn call(router: &Router, method: Method, path: &str) -> Response {
        router
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(path)
                    .body(Body::empty())
                    .expect("request not failure"),
            )
            .await
            .expect("Router error is Infallible")
    }

    /// （default）→ not 、 not CORS ； → 204 + 、
    /// response。
    ///
    /// in 1 use ： is ， `#[tokio::test]`
    /// will and （ 1 true、 1 in false）。
    #[tokio::test]
    async fn gateway_cors_follows_switch() {
        // ── （default）： as and before 1 ──
        crate::server::config::set_cors_enabled(false);
        let router = probe_router();

        let preflight = call(&router, Method::OPTIONS, "/v1/chat/completions").await;
        assert_ne!(
            preflight.status(),
            StatusCode::NO_CONTENT,
            " when in not ， this down to "
        );
        assert!(
            !preflight
                .headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            " when not CORS "
        );

        // ── ： by by ──
        crate::server::config::set_cors_enabled(true);
        let router = probe_router();

        let preflight = call(&router, Method::OPTIONS, "/v1/chat/completions").await;
        assert_eq!(preflight.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            preflight
                .headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .and_then(|value| value.to_str().ok()),
            Some("*")
        );
        assert!(preflight
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
        assert!(preflight
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_HEADERS));

        let ok = call(&router, Method::POST, "/v1/chat/completions").await;
        assert_eq!(ok.status(), StatusCode::OK);
        assert!(
            ok.headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN),
            " after response need CORS "
        );

        // ， its use 
        crate::server::config::set_cors_enabled(false);
    }

    /// `attach_cors` （， not when and config）。
    #[test]
    fn attach_cors_writes_three_headers() {
        let mut headers = axum::http::HeaderMap::new();
        attach_cors(&mut headers);
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "*"
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_METHODS).unwrap(),
            CORS_METHODS
        );
        assert_eq!(
            headers.get(header::ACCESS_CONTROL_ALLOW_HEADERS).unwrap(),
            CORS_HEADERS
        );
    }
}
