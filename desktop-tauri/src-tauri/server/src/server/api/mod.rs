//! HTTP by （ file 1 ， and Node route 1 1 for ）。
//!
//! ：`*_api.rs` for Node src/workbuddy-*-routes.mjs ；
//! use （health.rs / session.rs）。
//!
//! has ：
//!   health.rs     GET /health
//!   session.rs    GET /api/session、/api/session/login/*、/api/session/refresh|logout、
//!                 POST /auth/login、POST /auth/logout
//!   config_api.rs GET/POST /api/config
//!   logs_api.rs   GET /api/logs、/api/logs/stats、/api/logs/download、DELETE /api/logs
//!   stats_api.rs  GET /api/stats/summary、/api/stats/requests、DELETE /api/stats/requests、
//!                 GET/PUT /api/retention
//! retry_api.rs GET/PUT /api/retry（requestset， / ）
//! queue_api.rs GET/PUT /api/queue（ etc ： and when ）
//! accounts.rs /api/accounts*（ for workbuddy-account-routes.mjs）
//! proxies.rs /api/proxies*（Clash + ）
//! billing.rs / to / （ for workbuddy-billing.mjs + server.mjs 871-911 ）
//! chat.rs POST /v1/chat/completions、GET /v1/models（ for ）
//! models.rs POST /api/models/refresh（ new ； API，
//! and up 1 for is ， this filemodule header）
//! sanitize.rs GET/PUT /api/sanitize（）
//! prompt.rs GET/PUT /api/prompt（ + ）
//! auto_checkin.rs /api/auto-checkin*（ when to set / ）
//! scheduled_tasks.rs /api/scheduled-tasks*（ when / / ）
//! update.rs /api/update/*（updatecheck / down / / ）
//! storage_api.rs GET /api/storage（ 1 、 and ，）
//! zcode_captcha.rs GET/POST /api/zcode/captcha（ ：
//! after ， by request use ）
//! upgrade_api.rs GET /api/upgrade、POST /api/upgrade/run（ old → SQLite ）
//!
//! API already （ 1-6）。stats_api is statisticsadd 1 
//! （ 7 after by ）， ： new file + `http::router` ，
//! and and Node 1 （ need 1 `protected`）。

// 「Agent」： AI clientconfig（ / / / / / ）
pub mod agents;
pub mod agent_resources;
pub mod accounts;
pub mod accounts_batch;
mod accounts_export;
mod accounts_sort;
// `/api/accounts/usage` and （ from accounts.rs ： can from 
// 「 workbuddy」 to when add， this filemodule header）
pub mod accounts_usage;
// AI （GET/PUT /api/assistant/config + POST /api/assistant/chat）：
// set and for 。 and in before ， after not ，
// this filemodule header
pub mod assistant;
pub mod assistant_history;
pub mod assistant_images;
// （「」）： / / when / up / config /
// ， and for when （Skill / MCP / REST）。 up config and 
// in `core::tool_services`， that filemodule header
pub mod tools;
pub mod tool_runtime;
mod image_input;
// / （set「 / 」，SQL ； in 
// `db::backup`， and JSON merge is ）
pub mod backup_api;
pub mod auto_checkin;
// （ + limit）：`GET/PUT /api/auto-continue`。
// and in `core::upstream::continuation`， this is that ；
// file not is config_api —— by this module header。
pub mod auto_continue;
pub mod billing;
pub mod captcha;
pub mod chat;
// to in （GET /api/checkin-center）： in after 1 ，
// by before by need ， this filemodule header
pub mod checkin_center;
// （set「 → 」）：WebDAV / S3 up 。
// and `core::cloud_sync` module header
pub mod cloud_sync;
// to （GET/DELETE /api/checkin-records）： 1 and 
// ， in `core::billing::checkin::run_checkin`， this filemodule header
pub mod checkin_records;
// when to ****（GET/POST /api/checkin-schedules + {id}/delete|run）：
// total in /api/auto-checkin， CRUD in this ， this filemodule header
pub mod checkin_schedules;
pub mod cline_headers;
pub mod codearts_welfare;
pub mod compact_routes;
pub mod client_identification;
pub mod config_api;
// WorkBuddy （ / / / ）： from 
// workbuddy2api task_runner.py ， this filemodule header
pub mod growth;
// ** when **（ / when / ）：
// in `core::growth_scheduler`， this is 
pub mod growth_schedule;
// （`/v1/*`）：`GET/PUT /api/cors`（ and `/api/sanitize`
// ， this filemodule header）
pub mod cors;
// （ new when create ； and 
// `core::custom_providers` and `core::account_store::custom_accounts`）
pub mod custom_providers;
pub mod debug_api;
mod date_range;
// in （ from pipeline.rs ： that already 1000 ， is 
// 「handler axum when 408 」 this 1 ， this filemodule header）
pub mod disconnect_guard;
pub mod health;
// 「 from its 」 （current：cc-switch；
// `core::import_ccswitch`）
pub mod import_sources;
pub mod images;
mod image_assets;
mod image_output;
pub mod keys_api;
/// （`/api/keepalive-strip`）： and `/api/sanitize`、`/api/debug`
/// ，file not is config_api —— by this module header。
pub mod keepalive;
pub mod logs_api;
pub mod model_manage;
pub(crate) mod model_access;
// models.dev （ / / new ）： and 「 up directory」 
// /api/models/refresh ， by this filemodule header
pub mod model_meta;
// （ 「」）：、，
// to 「 this 1 × this 1 」 and ， this filemodule header
pub mod model_test;
pub mod models;
// Loomy new / 1 （ to after in ）， this filemodule header
pub mod onboarding;
pub mod panel;
pub mod pipeline;
mod live_progress;
mod record_persist;
// （ / / / ）： is use ，
// this 1 is 「」 1 ， this filemodule header
pub mod plugins;
// UI （`/plugins/{pack}/ui/*`）： can ， by 
// 、client iframe 。 not protected —— this filemodule header
pub mod plugin_ui;
// （`/plugins/theme.css`）： that `ui/css/tokens.css`
// to iframe ， and 1 / / 
pub mod plugin_theme;
pub mod prompt;
pub mod protocol;
pub mod proxies;
pub mod qoder_umid;
pub mod queue_api;
// will set（GET/PUT /api/image-compress）： + / / ，
// in `core::upstream::payload` send_body（ use in for ， not ），
// this filemodule header
pub mod image_compress;
pub mod retry_api;
pub(crate) mod request_prepare;
// （GET/PUT /api/developer-role-compat）： and /api/sanitize
// ， in `core::upstream::payload` send_body，
// this filemodule header
pub mod role_compat;
pub mod routing_api;
pub mod routing_flow;
pub mod routing_groups;
pub mod routing_trace;
pub mod routing_workspace;
pub mod sanitize;
pub mod scheduled_tasks;
pub mod session;
// （GET/PUT /api/sidebar-layout）： and origin
// 1 ， this filemodule header
pub mod sidebar_layout;
pub mod stats_api;
pub mod storage_api;
pub mod timeouts_api;
pub mod update;
pub mod upgrade_api;
// （ → ）， this filemodule header
pub mod zcode_captcha;
// ****set（ / / limit）， this filemodule header
pub mod zcode_captcha_pool;
pub mod zcode_claim;
pub mod zcode_claim_verification;
// ZCode client（set can ）： up by down not down ，
// after when is **empty**， this filemodule header
pub mod zcode_version;
// set「」： / / path / 1 
// （`GET /api/system-info`， this filemodule header）
pub mod system_info;
