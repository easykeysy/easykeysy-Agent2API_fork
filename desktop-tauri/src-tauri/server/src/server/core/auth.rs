//! will 、 and （****）。
//!
//! ── this has ──────────────────────────────────────────
//! for Node `workbuddy-auth.mjs` ： will 、`buildAuthHeaders`、
//! `refreshSession`（ up `/auth/token/refresh`）、12153 and 
//! `needsRelogin` 、`fetchAccounts` / `fetchLoginAccount`、
//! （`WORKBUDDY_TOKEN`）、`Context` 。
//!
//! that is 「client up 」 ， by need already 
//! to （`agent2api-providers` `workbuddy` ）。 in 
//! ：
//!
//! 1. ** will **： from 「current」 will （`get_current_session`
//! / `get_current_session_for`）， `/api/session`、 and use ；
//! 2. ****：`get_status` / `unconfigured_status` / `get_config_summary`
//! —— is **** ， not ；
//! 3. ****：`clear_session`（current）。
//!
//! ── ──────────────────────────────────────────────
//! is （、、、）， in by 
//! （） `refresh_access_token` 。`api/accounts.rs` new by 
//! 「 provider is by 」，WorkBuddy that use 。
//!
//! ── （`WORKBUDDY_TOKEN`） ─────────────────
//! to ： in `provider.info` 
//! `anonymous_default_session` / `env_credentials`， in 「 1 
//! no has 」 when to need default token（`ensure_access_token(store, "")`）。
//! not that —— 1 has this 。

use serde_json::{json, Value};

use crate::server::core::account_store::state::json_number;
use crate::server::core::account_store::AccountStore;
use crate::server::core::proxies::ResolvedProxy;
use crate::server::logging;

// and error for use 1 （`autoclaw` etc use 
// `send_raw`；`WorkBuddyAuthError` is failure ）。
#[allow(unused_imports)]
pub use crate::server::core::auth_http::{
    send_raw, ApiResponse, WorkBuddyAuthError, REFRESH_WINDOW_MS,
};

// ─── will and ─────────────────────────────────────────────

/// ：。
///
/// `AuthService` is as not `ServerState` and ——
/// in is 1 「 + will 」 。
#[derive(Clone)]
pub struct AuthService {
    store: AccountStore,
}

/// new （ and by use ）。
///
/// already not ， this to in （Raccoon / AutoClaw /
/// CodeArts） and will path use 。
#[derive(Clone, Debug)]
pub struct RefreshedAuth {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<f64>,
    pub refresh_expires_at: Option<f64>,
}

impl AuthService {
    pub fn new(store: AccountStore) -> Self {
        Self { store }
    }

    /// and `new` 1 （ old ， before is 「 use default up down 」）。
    pub fn for_store(store: AccountStore) -> Self {
        Self::new(store)
    }

    /// （）。
    ///
    /// to 「 up already has will 、 need by id 」 path use —— before is 
    /// `BillingService`： by will ， `admin.action` must id can 
    /// by to this provider 。
    pub fn store(&self) -> AccountStore {
        self.store.clone()
    }

    /// in **file**（Node `auth.authFile` for etc ）。
    ///
    /// `AccountStore::file_string()`， after is **SQLite file**
    /// （`{config_dir}/agent2api.db`）—— in `accounts` ，
    /// not is that `accounts.json`。 up 「file」 1 is ，
    /// use this （WAL down has `-wal` / `-shm` file）；
    /// no when will to that **path**， not is empty。
    ///
    /// Windows down 1 `\`：Node `path.join` in Windows up is this 
    /// ， this path will to use （ up 「file」） and to 
    /// directory use ， and Node 1 。
    pub fn auth_file(&self) -> String {
        normalize_display_path(&self.store.file_string())
    }

    /// currentavailable；** not new **（ in ，module header）。
    ///
    /// ：current（WorkBuddy ）。
    /// return None no has available（）。
    ///
    /// ** not use ** —— must provider ，
    /// `get_current_session_for`。
    pub async fn get_current_session(&self) -> Result<Option<Value>, WorkBuddyAuthError> {
        self.session_of(None).await
    }

    /// provider currentavailable。
    ///
    /// 「current」 in this provider （`current_entry_for_provider`），
    /// not is —— otherwise「 has raccoon 」 up ，
    /// workbuddy default will to raccoon （ 401）。
    pub async fn get_current_session_for(
        &self,
        provider: &str,
    ) -> Result<Option<Value>, WorkBuddyAuthError> {
        self.session_of(Some(provider)).await
    }

    /// will use ；`scope` as None current。
    async fn session_of(&self, scope: Option<&str>) -> Result<Option<Value>, WorkBuddyAuthError> {
        let store = self.store.clone();
        let scope = scope.map(str::to_string);
        let entry = super::blocking::run(move || match scope {
            Some(provider) => store.current_entry_for_provider(&provider),
            None => store.get_current_entry(),
        }).await.map_err(|error| WorkBuddyAuthError::new(error.message))?;
        Ok(entry.map(|entry| entry.session))
    }

    /// will ； by and by need 。
    pub fn get_status(&self) -> Value {
        let session = self.store.get_current_entry().map(|entry| entry.session);
        let Some(session) = session else {
            return self.unconfigured_status();
        };
        let expires_at = session
            .get("auth")
            .and_then(|auth| auth.get("expiresAt"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let refresh_token = session
            .get("auth")
            .and_then(|auth| auth.get("refreshToken"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let account = session.get("account").cloned().unwrap_or(Value::Null);
        // this Node use is `|| null` —— empty will null（ not only when ）。
        // 「 is not is 」（enterpriseId as null not ），
        // to empty will all up 1 empty 。
        let nullable_text = |key: &str| -> Value {
            account
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(|value| Value::String(value.to_string()))
                .unwrap_or(Value::Null)
        };
        json!({
            "loggedIn": true,
            "endpoint": session.get("endpoint").cloned().unwrap_or(Value::Null),
            "prefixPath": session.get("prefixPath").cloned().unwrap_or(Value::Null),
            "edition": session.get("edition").cloned().unwrap_or(Value::Null),
            "platform": session.get("platform").cloned().unwrap_or(Value::Null),
            "accountUid": nullable_text("uid"),
            "nickname": nullable_text("nickname"),
            "accountType": account
                .get("type")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .unwrap_or("personal"),
            "enterpriseId": nullable_text("enterpriseId"),
            "currentAccountId": self.store.current_account_id(),
            "tokenExpiresAt": if expires_at > 0.0 {
                crate::server::core::account_store::state::json_number(expires_at)
            } else {
                Value::Null
            },
            "canRefresh": !refresh_token.is_empty(),
            "authFile": self.auth_file(),
        })
    }

    /// （`/health`、`/api/session` login use ）
    pub fn unconfigured_status(&self) -> Value {
        json!({
            "loggedIn": false,
            "endpoint": Value::Null,
            "prefixPath": Value::Null,
            "edition": Value::Null,
            "platform": Value::Null,
            "authFile": self.auth_file(),
            "reason": "尚未登录或凭证缺失",
        })
    }

    /// up config need （ for workbuddy-upstream-client.mjs `getConfigSummary`）。
    ///
    /// ****： is in ， not request。 not token new ，
    /// otherwise /health will up —— is ， 1 
    /// will 「 is 」 up 。
    ///
    /// `authApiBase` / `chatApiUrl` already delete： is ** up **， not 
    /// has that （`baseUrl` ， that is ）。
    pub fn get_config_summary(&self) -> Value {
        let Some(entry) = self.store.get_current_entry() else {
            return json!({
                "configured": false,
                "baseUrl": Value::Null,
                "unavailableReason": UNCONFIGURED_REASON,
            });
        };
        let auth = entry.session.get("auth").cloned().unwrap_or(Value::Null);
        let expires_at = auth.get("expiresAt").and_then(Value::as_f64).unwrap_or(0.0);
        let refresh_token = auth
            .get("refreshToken")
            .and_then(Value::as_str)
            .unwrap_or("");
        let account_uid = entry
            .session
            .get("account")
            .and_then(|account| account.get("uid"))
            .cloned()
            .unwrap_or(Value::Null);
        json!({
            "configured": true,
            "baseUrl": entry
                .session
                .get("endpoint")
                .cloned()
                .unwrap_or(Value::Null),
            "authSource": if refresh_token.is_empty() {
                "环境变量/静态 token"
            } else {
                "auth.json（可自动刷新）"
            },
            "currentAccountId": account_uid,
            "tokenExpiresAt": if expires_at > 0.0 { json_number(expires_at) } else { Value::Null },
            "canRefresh": !refresh_token.is_empty(),
        })
    }

    // ─── （only use ）──────────────────────────────────

    /// （ `refresh_flight` module header）。
    ///
    /// in （Raccoon / AutoClaw / CodeArts） `refresh.rs`
    /// use `refresh_flight::Table`， this no has —— this 
    /// as up `RefreshedAuth` has 。

    /// empty。
    ///
    /// store down 「empty」= current（Node `clearSession` ）：
    /// current by ， after down 1 available。
    /// no has available when （none can ）return false， use can 。
    pub fn clear_session(&self) -> bool {
        let Some(entry) = self.store.get_current_entry() else {
            logging::verbose("[Auth]", "当前没有可用账号，无需清理登录态");
            return false;
        };
        match self.store.remove_account(&entry.id) {
            Ok(()) => {
                logging::log(
                    "[Auth]",
                    &format!("已清除当前登录态（账号 {} 已从列表移除）", entry.id),
                );
                true
            }
            Err(error) => {
                logging::log("[Auth]", &format!("清理登录态失败: {error}"));
                false
            }
        }
    }
}

/// path 1 Windows （`C:/a/b` → `C:\a\b`）。
///
/// `PathBuf::to_string_lossy` in Windows up to is `C:\a\b`， config_dir can can is 
/// from （AGENT2API_PROXY_HOME， old WORKBUDDY_PROXY_HOME can ） 
/// 。 this will to use 、 will use directory， 1 Node 
/// `path.join` not 。
fn normalize_display_path(path: &str) -> String {
    if cfg!(windows) {
        path.replace('/', "\\")
    } else {
        path.to_string()
    }
}

/// Reason（getConfigSummary use ）。
///
/// Node this is 「（node server.mjs --login）」； Rust no has 
/// 那个命令行入口（登录走 UI 的 /api/session/login/*）， use will to 1 not in 
/// ， and 1 。
pub const UNCONFIGURED_REASON: &str = "noneavailable";

/// in auth for up expiresAt / refreshExpiresAt / lastRefreshTime。
/// Node `withExpiresAt`： already has ，otherwise expiresIn（seconds）。
pub fn with_expires_at(mut auth: Value) -> Value {
    let now = logging::now_ms() as f64;
    let Some(object) = auth.as_object_mut() else {
        return auth;
    };
    let expires_in = object
        .get("expiresIn")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let refresh_expires_in = object
        .get("refreshExpiresIn")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let expires_at = object
        .get("expiresAt")
        .and_then(Value::as_f64)
        .filter(|value| *value != 0.0)
        .unwrap_or_else(|| if expires_in != 0.0 { now + expires_in * 1000.0 } else { 0.0 });
    let refresh_expires_at = object
        .get("refreshExpiresAt")
        .and_then(Value::as_f64)
        .filter(|value| *value != 0.0)
        .unwrap_or_else(|| {
            if refresh_expires_in != 0.0 {
                now + refresh_expires_in * 1000.0
            } else {
                0.0
            }
        });
    let last_refresh = object
        .get("lastRefreshTime")
        .and_then(Value::as_f64)
        .filter(|value| *value != 0.0)
        .unwrap_or(now);
    // when （json_number）：Node `JSON.stringify` for 
    // `Date.now() + n*1000` this not will `.0`， will accounts.json
    // and when 「 1 not 」 
    object.insert("expiresAt".to_string(), json_number(expires_at));
    object.insert("refreshExpiresAt".to_string(), json_number(refresh_expires_at));
    object.insert("lastRefreshTime".to_string(), json_number(last_refresh));
    auth
}

/// token is （ before 5 minutes as need need new ）
pub fn is_token_expiring(expires_at: f64) -> bool {
    if expires_at <= 0.0 {
        return false;
    }
    expires_at - REFRESH_WINDOW_MS <= logging::now_ms() as f64
}

/// new response RefreshedAuth（ when to None， use ）
pub fn refresh_result_from(data: &Value) -> RefreshedAuth {
    let enriched = with_expires_at(data.clone());
    RefreshedAuth {
        access_token: enriched
            .get("accessToken")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        refresh_token: enriched
            .get("refreshToken")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        expires_at: enriched
            .get("expiresAt")
            .and_then(Value::as_f64)
            .filter(|value| *value > 0.0),
        refresh_expires_at: enriched
            .get("refreshExpiresAt")
            .and_then(Value::as_f64)
            .filter(|value| *value > 0.0),
    }
}

/// new use （ new use ）。
pub type RefreshProxy = ResolvedProxy;
