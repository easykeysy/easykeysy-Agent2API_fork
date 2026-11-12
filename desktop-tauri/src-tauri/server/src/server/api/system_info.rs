//! `GET /api/system-info` —— set「」 1 。
//!
//! ── Why 1 ， not is has ───────────
//! this use is 「 when 1 / 1 」：must** 1 **，
//! and need can in when to 1 。 ，
//! and `/api/session`、`/api/storage`、`/api/update/status`、
//! `/api/panel/status`、`/api/config` ， 1 failure in up 
//! 「—」—— that 「—」 can can is need need 1 。 this in server 1 ，
//! failure null，。
//!
//! ── protected ────────────────────────────────────────────
//! response has directory、 can filepath、Key 、 is fail-closed —— is 
//! ， and `/api/storage` 。 will to can 
//! 。
//!
//! ── 「 has no has 」， not ──────────────────────────────────
//! Key to 、GitHub to filled / origin、 to 「 is 」。
//! this and `/api/update/token` 1 ： this will to issue ，
//! 1 will 1 。
//!
//! ── path by after ────────────────────────────────────────
//! `hideUsername` parameter（`?hideUsername=1`）， in path in directory before 
//! `~`。 after is as 「」 to is after ——
//! before in that up （ `core::system_info` module header）。
//! is ****， not ： this 1 request return， new 
//! to default（path）——「 this need to 」 is 1 ， in 
//! config will down 。

use std::path::Path;

use axum::extract::{Query, State};
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::access;
use crate::server::config;
use crate::server::core::system_info;
use crate::server::core::update::{self, selfupdate, status_json, CURRENT_VERSION};
use crate::server::db;
use crate::server::http::ok_json;
use crate::server::ServerState;

/// parameter：`hideUsername=1` / `true` when directory before `~`。
#[derive(Default, serde::Deserialize)]
pub struct Params {
    #[serde(rename = "hideUsername")]
    hide_username: Option<String>,
}

/// GET /api/system-info
pub async fn get_system_info(
    State(state): State<ServerState>,
    Query(params): Query<Params>,
) -> Response {
    let hide = matches!(
        params.hide_username.as_deref().map(str::trim),
        Some("1") | Some("true") | Some("yes") | Some("on")
    );
    let show = |path: &Path| -> String {
        if hide {
            system_info::mask_path(path)
        } else {
            path.to_string_lossy().to_string()
        }
    };

    let config_dir = config::config_dir();
    let db_file = match state.db() {
        Some(db) => db.file().to_path_buf(),
        None => config_dir.join(db::FILE_NAME),
    };
    let exe = std::env::current_exe().ok();
    let db_available = state.db().is_some();
    let db_bytes = std::fs::metadata(&db_file).map(|meta| meta.len()).unwrap_or(0);

    // schema ：unavailable when as null（ not —— that will 
    // 「 no 」「 for up 」）。
    let db_schema = if db_available {
        db::stored_schema_version(&db_file)
    } else {
        None
    };

    let snapshot = config::current();
    let accounts = state.store().list_accounts();
    let account_list = accounts
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let account_count = account_list.len();
    let enabled_accounts = account_list
        .iter()
        .filter(|item| item.get("enabled").and_then(Value::as_bool).unwrap_or(false))
        .count();
    let limited_accounts = account_list
        .iter()
        .filter(|item| is_rate_limited(item))
        .count();

    let in_flight: usize = state
        .upstream()
        .connections()
        .snapshot()
        .into_iter()
        .map(|(_, count)| count)
        .sum();

    let stats = state.request_stats().stats();
    let requests = stats.get("total").and_then(Value::as_u64).unwrap_or(0);
    let logs = crate::server::logging::store_ref()
        .map(|store| store.stats().total)
        .unwrap_or(0);

    let token_status = status_json();
    let has_update_proxy = config::update_proxy().is_some();

    ok_json(json!({
        "app": {
            // ： will use when package_info ，
            // its this is 1 （ and `/api/update/check` 1 ）
            "version": CURRENT_VERSION,
            "license": "AGPL-3.0-only",
            "sourceUrl": "https://github.com/aimod-cc/Agent2API",
            "licenseUrl": "https://github.com/aimod-cc/Agent2API/blob/main/LICENSE",
            "copyright": "Copyright (c) 2026 Agent2API",
            "channel": config::update_channels().key(),
            "schemaVersion": db::schema::SCHEMA_VERSION,
            "dbSchemaVersion": db_schema,
            // 「 up client / CLI 」 already to ：
            // not has 1 client，none from 。
            // （ not is to empty）—— empty will 1 「—」，
            // failure； after before by 「 no has this 」 this 。
        },
        "runtime": {
            "deployment": system_info::deployment(),
            "container": system_info::in_container(),
            "os": system_info::os(),
            "osVersion": system_info::os_version(),
            "arch": system_info::arch_label(),
            "startedAt": system_info::started_at_ms(),
            "uptimeSeconds": system_info::uptime_seconds(),
            "pid": std::process::id(),
            "debugBuild": cfg!(debug_assertions),
            "verbose": crate::server::logging::is_verbose(),
        },
        "deployment": {
            "host": state.host.to_string(),
            "port": state.port,
            "panelPort": state.panel_port,
            "uiHosted": state.ui_dir().is_some(),
            "localPanel": state.local_panel,
            "configDir": show(&config_dir),
            "dbFile": show(&db_file),
            "dbBytes": db_bytes,
            "dbAvailable": db_available,
            "logFile": show(&crate::server::logging::log_file_path_public()),
            "exePath": exe.as_deref().map(show).unwrap_or_default(),
            "installerKind": update::installer_kind(),
            "selfUpdate": selfupdate::supported(),
            "updateCapability": update::capabilities::current(),
        },
        "security": {
            "adminRegistered": access::admin_registered(),
            "panelAuth": access::panel_auth_enabled(),
            "apiKeyCount": snapshot.active_api_keys().len(),
            "allowNoKey": env_flag("AGENT2API_ALLOW_NO_KEY"),
            "v1FailClosed": access::v1_fail_closed(),
            "panelGate": access::panel_gate(),
            "githubTokenFilled": token_status.get("filled").and_then(Value::as_bool).unwrap_or(false),
            "githubTokenOrigin": token_status.get("origin").cloned().unwrap_or(Value::Null),
            // 「 has no has 」：， will 
            "updateProxy": has_update_proxy,
        },
        "features": {
            "debugMode": snapshot.debug_mode(),
            "sanitizeFingerprints": snapshot.sanitize_fingerprints(),
            "cors": snapshot.cors_enabled(),
            "developerRoleCompat": snapshot.developer_role_compat(),
            "keepaliveStrip": config::strip_newline_keepalive(),
            "autoContinue": config::auto_continue(),
            "autoContinueMaxRounds": config::auto_continue_max_rounds(),
            "modelsDev": config::models_dev_enabled(),
            // current this 1 （ / headless ）， and /api/captcha
            // ； old before this ， as not 
            "captchaEnabled": snapshot.captcha_enabled(state.local_panel),
            "updateBeta": config::update_beta(),
        },
        "stats": {
            "accounts": account_count,
            "enabledAccounts": enabled_accounts,
            "limitedAccounts": limited_accounts,
            "models": crate::server::core::providers::catalog::managed_manifests(state.store())
                .into_iter()
                .map(|(_, models)| models.into_iter()
                    .map(|model| crate::server::core::models::model_id(&model).to_lowercase())
                    .filter(|id| !id.is_empty())
                    .collect::<std::collections::HashSet<_>>().len())
                .sum::<usize>(),
            "inFlight": in_flight,
            "requests": requests,
            "logs": logs,
            "upgradePending": state.upgrade_pending(),
        },
    }))
}

/// is （ and before `isRateLimited` 1 ： 1 
/// `resetAt` current when ）。
fn is_rate_limited(account: &Value) -> bool {
    let Some(limits) = account.get("rateLimits").and_then(Value::as_object) else {
        return false;
    };
    let now = crate::server::logging::now_ms();
    limits.values().any(|info| {
        info.get("resetAt")
            .and_then(Value::as_i64)
            .map(|reset| reset > now)
            .unwrap_or(false)
    })
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}
