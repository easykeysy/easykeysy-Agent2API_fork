//! （、、、 will 、none、、directory、、）。
//!
//! 对照 Node 版 src/*.mjs ：
//! endpoints.rs //UA/ up down （workbuddy-endpoints.mjs）
//! providers/ （id/label）+ directory + by 
//! （Agent2API add；file ）
//! account_store/ 、、、 （workbuddy-account-store.mjs）
//! account_transfer.rs / （workbuddy-account-transfer.mjs）
//! auth_http.rs up request and 、error （workbuddy-auth.mjs api/unwrap）
//! auth.rs will 、getStatus、、 new （workbuddy-auth.mjs will ）
//! login.rs none and （auth.mjs + server.mjs ）
//! clash.rs Clash Verge config and （workbuddy-proxy.mjs Clash ）
//! proxies.rs 1 // （workbuddy-proxy.mjs ）
//! egress.rs （ by Client）+ （workbuddy-proxy.mjs dispatch ）
//! billing/ / to / （workbuddy-billing.mjs）
//! models/ workbuddy directory（ + /v3/config new ）（workbuddy-models.mjs）
//! sanitize.rs request（ then ， workbuddy2api）
//! prompt.rs has （ / / ， workbuddy2api）
//! degrade.rs （ → in to 00:00）
//! routing.rs （ + ） （workbuddy-routing.mjs）
//! upstream/ for （//SSE/） （workbuddy-upstream-client.mjs）
//! auto_checkin.rs when to （ + ） （workbuddy-auto-checkin.mjs）
//! credential_maintenance.rs （ → new ；
//! in ， `providers::adapter` 5）
//! custom_providers.rs （ use up ） and ；
//! API in `api::custom_providers`， in 
//!                        `account_store::custom_accounts`
//! scheduled_tasks.rs when and （ / new /
//! before new ； and config，
//! by `api::scheduled_tasks`）
//! usage_query.rs / （ + and + 
//! to ； in core is as and 
//! use 1 ）
//! usage_records.rs （account_usage_records ） and 
//! use —— not / use 
//! 
//! key_scope.rs request in Key and its available / availablelimit
//! （R9； in request，handler and ）
//! assistant_sessions.rs AI for will （assistant_sessions ，schema v16）——
//! that will ； API in `api::assistant`
//! crypto.rs use for （AES-256-GCM + `enc1:` ）， by 
//! `SecretScope` ；GitHub and use 
//! （ in `update/token.rs`， use ）
//! legacy_api_key.rs old `apiKey`（ Key ） ；
//! and in `api_keys.rs`
//! update/ update（// down ）（workbuddy-update.mjs）
//!
//! ── （Agent2API W2a-T2）─────────────
//! providers/mod.rs and （id/label/default by ）
//! models/ **workbuddy 1 ** （ + new ）
//! providers/catalog.rs directory： and /v1/models 1 
//! providers/router.rs → provider （directory + providerRoute ）
//! before 「 has 」， after 1 「 1 」。
//! ****（ to that in after by ， not to 
//! ） in `providers/catalog_cache.rs` —— up ， not 1 。
//! `model_meta/` is **** that 1 （models.dev can and ）：
//! not 「 has 」， no —— is up 1 、 
//! （`providers::catalog::manifest_for` and 
//! `custom_providers::bindings::public_models`）， and module header。
//!
//! ：core + file + up HTTP， not axum；
//! api/ handler HTTP core use 、 response。

pub mod account_order;
pub mod account_store;
// AI client （「Agent」）：client + config
pub mod agents;
pub mod account_transfer;
pub mod api_keys;
pub mod auth;
pub mod auth_http;
pub mod assistant_sessions;
pub mod assistant_images;
pub mod auto_checkin;
pub mod beijing;
pub mod billing;
pub(crate) mod blocking;
pub mod capability;
pub mod checkin_history;
pub mod checkin_records;
pub mod checkin_schedules;
pub mod clash;
pub mod client;
pub(crate) mod compact_routing;
// （set「 → 」）：WebDAV / S3 up 。
// 、 and this module header
pub mod cloud_sync;
pub mod credential_maintenance;
pub mod crypto;
pub mod custom_providers;
pub mod debug_traffic;
pub mod degrade;
pub mod egress;
pub mod endpoints;
pub mod growth_scheduler;
pub mod import_ccswitch;
pub mod key_scope;
pub mod key_governance;
pub mod legacy_api_key;
pub mod limiter;
pub mod login;
pub mod model_meta;
pub mod model_rules;
pub mod models;
/// ： + RPC client + 。
///
/// this is 「 to use 」 that in ，module header。
/// and `providers` ：`providers` is for 「 1 up 」 
/// （`ProviderAdapter`）， `RpcAdapter`。
pub mod plugin_host;
pub mod prompt;
pub mod protocol;
pub mod providers;
pub mod proxy_pool;
pub mod proxy_rotate;
pub mod proxy_settings;
pub mod proxies;
pub mod routing;
pub(crate) mod routing_groups;
pub mod routing_order;
pub mod routing_scope;
pub mod sanitize;
pub mod scheduled_tasks;
// （ / / ） ： JSON kv 1 。
// by before `ui/sidebar-layout.js` has ， after not ，
// this module header
pub mod sidebar_layout;
// （「」）： / up config、 when and 
// 。 for （Skill / MCP / API） 1 up config， this module header
pub mod tool_services;
// （ / / / / path）：set「」
// ， this module header。， not ServerState。
pub mod system_info;
pub mod task_state;
/// Trae ID（`-solo` / `-solo-max`） 1 ：module header。
pub(crate) mod trae_model_ids;
pub mod update;
pub mod upstream;
pub mod usage_query;
pub mod usage_records;
