//! / to / ****。
//!
//! ── this Why（2026-10 ）───────────────────────
//! path（`/v2/billing/meter/daily-checkin`）、 use 、 
//! up （「 already to ，」）、（`www.codebuddy.cn`） and 
//! —— is 「 this up 」 ， already to 
//! （`workbuddy::billing`）。：
//!
//! 1. ** will **：current（ or use ）→ 1 available by will ；
//! 2. ****： and id to `admin.action` / `usage.query`；
//! 3. ****：`checkin_and_report` this 「 + + 」 ，
//! and daily 。
//!
//! ── id from ──────────────────────────────────────────
//! `admin.action` must id can by to that ， if 
//! is ** will **（`store.get_session_by_id` ， not id）。
//! by `(provider, uid)` 1 —— from will `edition` 
//! （ will when is from ）。
//!
//! ── file ────────────────────────────────────────────────
//! mod.rs 、error、 and （file）
//! checkin.rs to （ + ， to when to 1 path）
//! keepalive.rs **config**（ use config， in ）
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod checkin;
pub mod keepalive;

use serde_json::{json, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::auth::AuthService;
use crate::server::core::providers::adapter::adapter_by_id;
use crate::server::core::providers::workbuddy::Region;
use crate::server::logging;

/// / error（ for Node WorkBuddyBillingError）。
///
/// `status_code` will HTTP （504 when / 401 /
/// up HTTP ），must is Option 1 ——
/// Node default 502。
#[derive(Clone, Debug)]
pub struct BillingError {
    pub message: String,
    pub status_code: i32,
    pub upstream_code: Option<i64>,
}

impl BillingError {
    pub fn new(message: impl Into<String>, status_code: i32) -> Self {
        Self { message: message.into(), status_code, upstream_code: None }
    }

    pub fn with_code(
        message: impl Into<String>,
        status_code: i32,
        upstream_code: Option<i64>,
    ) -> Self {
        Self { message: message.into(), status_code, upstream_code }
    }

    /// 1 error。
    ///
    /// by failure body OpenAI （`/api/usage` that in server.mjs
    /// try → errorPayload）—— by by use ，
    /// this 。
    pub fn to_gateway_error(&self) -> crate::server::errors::GatewayError {
        let mut error = crate::server::errors::GatewayError::with_status(
            self.status_code,
            self.message.clone(),
        );
        if let Some(code) = self.upstream_code {
            error = error.upstream_code(code);
        }
        error
    }

    /// error → error（）。
    fn from_gateway(error: crate::server::errors::GatewayError) -> Self {
        Self::with_code(error.message, error.status_code, error.upstream_code)
    }
}

impl std::fmt::Display for BillingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

/// daily ****（ for to in by ；
/// when and to [`WorkbuddyActivity::Full`]， is ）。
///
/// is 「 need up 」：
/// * `Full`： + + —— 1 1 ；
/// * `Claim`： + —— need ， not that for ；
/// * `Keepalive`： —— no when ， not and 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkbuddyActivity {
    Full,
    Claim,
    Keepalive,
}

impl WorkbuddyActivity {
    /// API mode → 。unknown Full —— 
    /// as is 「 1 」，「 1 」 。
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("claim") => Self::Claim,
            Some("keepalive") => Self::Keepalive,
            _ => Self::Full,
        }
    }

    /// mode（ before 「 this 」 use ）
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Claim => "claim",
            Self::Keepalive => "keepalive",
        }
    }
}

/// client： has （ will use ）。
///
/// Node is `createWorkBuddyBilling({ auth, ... })`， `auth.getCurrentSession()`
/// default will 。 by （/api/accounts/usage、/api/accounts/checkin） need need 
/// **** will ， that path by `billing::checkin` store，
/// this not has 。
#[derive(Clone)]
pub struct BillingService {
    auth: AuthService,
}

impl BillingService {
    pub fn new(auth: AuthService) -> Self {
        Self { auth }
    }

    /// currentconfig locale（/api/usage and claim-and-report use ）
    fn current_locale() -> String {
        config::current().locale().to_string()
    }

    /// default will （current）； no has available when 401。
    ///
    /// Node ，「node server.mjs --login」 that ——
    /// Rust no has that ， to not in will use 。
    async fn require_session(&self) -> Result<Value, BillingError> {
        let session = self.auth.get_current_session().await.map_err(|error| {
            let status = error.http_status();
            BillingError::with_code(error.message, status, error.upstream_code)
        })?;
        match session {
            Some(session) if has_access_token(&session) => Ok(session),
            _ => Err(BillingError::new("当前没有可用登录态：请先在桌面端完成登录", 401)),
        }
    }

    /// will → `(provider_id, account_id)`。
    ///
    /// from will `edition` （ will when is from ），
    /// use `(provider, uid)` id —— `admin.action` need is id。
    ///
    /// ── uid not to when （issue #380）────────────────────────────
    /// will is `session_from_record` ， that path in will 
    /// `pluginPublic` / `pluginSecret`， down this can to uid。
    /// if will use （ already JSON），uid can can as empty ——
    /// that when by `user-<uid>` id 1 ：WorkBuddy 
    /// id is that （`accounts::identity_of`）。
    fn account_of(&self, session: &Value) -> Option<(String, String)> {
        let uid = session
            .get("account")
            .and_then(|account| account.get("uid"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let region = Region::from_edition_id(session.get("edition").and_then(Value::as_str));
        let provider = region.provider_id();
        let store = self.auth.store();
        store
            .account_id_of_identity(provider, uid)
            .or_else(|| store.account_id_of_user_id_suffix(provider, uid))
            .map(|id| (provider.to_string(), id))
    }

    /// 1 to （`admin.action`）。
    async fn admin(
        &self,
        session: &Value,
        action: &str,
        payload: &Value,
    ) -> Result<Value, BillingError> {
        let Some((provider, account_id)) = self.account_of(session) else {
            return Err(BillingError::new(
                "找不到该会话对应的账号记录，无法执行该操作",
                400,
            ));
        };
        let Some(adapter) = adapter_by_id(&provider) else {
            return Err(BillingError::new(
                format!(
                    "{} 的计费接口需要安装提供商插件：本版本不再内置反向代理实现",
                    crate::server::core::providers::label_of(&provider)
                ),
                503,
            ));
        };
        let store: AccountStore = self.auth.store();
        adapter
            .admin_action(&store, &account_id, action, payload)
            .await
            .map_err(BillingError::from_gateway)
    }

    // ─── to ───────────────────────────────────────────────

    /// to （`AuthService.getCheckinStatus`）。
    ///
    /// success data as empty when return null（ and 1 ）。daily，
    /// response 1 1 （`active` / `todayCheckedIn`）， use not use 
    /// —— 1 in （`billing::ops::checkin_status`）。
    pub async fn get_checkin_status(&self, session: Option<&Value>) -> Result<Value, BillingError> {
        let active = match session {
            Some(session) => session.clone(),
            None => self.require_session().await?,
        };
        self.admin(&active, "billing.checkin-status", &Value::Null).await
    }

    /// daily to （`AuthService.claimDailyCheckin`）。
    ///
    /// etc ： already when up return 0 code（ or HTTP 400）， that return
    /// `{success:false, code, msg}` ——「 already to 」 not is error， before will 
    /// 1 warn 。
    ///
    /// no has 「 to 」 this ：daily
    /// （[`Self::workbuddy_daily_activity`]）， and 、 and 
    /// 1 and claim return —— and to from this ，
    /// 1 use not up 。
    pub async fn claim_daily_checkin(&self, session: Option<&Value>) -> Result<Value, BillingError> {
        let active = match session {
            Some(session) => session.clone(),
            None => self.require_session().await?,
        };
        if is_international(&active) {
            let activity = self
                .workbuddy_daily_activity(&active, WorkbuddyActivity::Full)
                .await;
            let mut claim = activity.get("claim").cloned().unwrap_or_else(|| {
                json!({
                    "success": false,
                    "code": -1,
                    "msg": "活跃任务未返回领取结果",
                })
            });
            if let Some(object) = claim.as_object_mut() {
                object.insert(
                    "activity".to_string(),
                    activity.get("activity").cloned().unwrap_or(Value::Null),
                );
                object.insert(
                    "status".to_string(),
                    activity.get("status").cloned().unwrap_or(Value::Null),
                );
            }
            return Ok(claim);
        }
        self.admin(&active, "billing.checkin-claim", &Value::Null).await
    }

    /// 1 （`claim-gift` / `claim-compensation`）。
    ///
    /// （ for ）。 1 ——「 already 」 and 
    /// 「 no has can 」 is ，return not is `Err`。
    async fn claim_reward(&self, kind: &str, session: &Value) -> Result<Value, BillingError> {
        self.admin(session, "billing.reward", &json!({ "kind": kind }))
            .await
    }

    /// （ 2026-10 DeepSeek 500 ）
    pub async fn claim_compensation(&self, session: &Value) -> Result<Value, BillingError> {
        self.claim_reward("compensation", session).await
    }

    /// new （ 1 new use ， no has ）
    pub async fn claim_gift(&self, session: &Value) -> Result<Value, BillingError> {
        self.claim_reward("gift", session).await
    }

    // ─── ───────────────────────────────────────────────

    /// （`{kind, unlimited, totalLeft, planLeft, bonusLeft}` + + ）。
    ///
    /// （ up → before ） in ； this 
    /// 「 will → id → `usage.query`」。
    pub async fn query_credits_summary(
        &self,
        session: Option<&Value>,
        _locale: Option<&str>,
    ) -> Result<Value, BillingError> {
        let active = match session {
            Some(session) => session.clone(),
            None => self.require_session().await?,
        };
        let Some((provider, account_id)) = self.account_of(&active) else {
            return Err(BillingError::new(
                "找不到该会话对应的账号记录，无法查询积分",
                400,
            ));
        };
        let Some(adapter) = adapter_by_id(&provider) else {
            return Err(BillingError::new(
                format!(
                    "{} 的积分查询需要安装提供商插件：本版本不再内置反向代理实现",
                    crate::server::core::providers::label_of(&provider)
                ),
                503,
            ));
        };
        let store = self.auth.store();
        adapter
            .query_usage(&store, &account_id)
            .await
            .map_err(BillingError::from_gateway)
    }

    /// default down （ by ， for server.mjs
    /// `billing.queryCreditsSummary({ locale: opts.locale })`）
    pub async fn query_credits_summary_default(&self) -> Result<Value, BillingError> {
        let locale = Self::current_locale();
        self.query_credits_summary(None, Some(&locale)).await
    }

    // ─── ───────────────────────────────────────────

    /// banner： need client，otherwise （return null， and 1 ）。
    ///
    /// **allerror and return null** —— banner is up 1 ， not to not 
    /// ， not this 1 （ and Node 1 ）。
    pub async fn get_activity_banner(&self, session: Option<&Value>) -> Value {
        let active = match session {
            Some(session) => session.clone(),
            None => match self.require_session().await {
                Ok(session) => session,
                Err(error) => {
                    logging::log("[Activity]", &format!("banner 拉取失败: {}", error.message));
                    return Value::Null;
                }
            },
        };
        match self.admin(&active, "billing.banner", &Value::Null).await {
            Ok(value) => value,
            Err(error) => {
                logging::log("[Activity]", &format!("banner 拉取失败: {}", error.message));
                Value::Null
            }
        }
    }

    /// （）。return null（verbose ）。
    pub async fn get_ambassador_status(&self, session: Option<&Value>) -> Value {
        let active = match session {
            Some(session) => session.clone(),
            None => match self.require_session().await {
                Ok(session) => session,
                Err(error) => {
                    logging::verbose(
                        "[Activity]",
                        &format!("ambassador 状态拉取失败: {}", error.message),
                    );
                    return Value::Null;
                }
            },
        };
        match self.admin(&active, "billing.ambassador", &Value::Null).await {
            Ok(value) => value,
            Err(error) => {
                logging::verbose(
                    "[Activity]",
                    &format!("ambassador 状态拉取失败: {}", error.message),
                );
                Value::Null
            }
        }
    }

    /// to + （" to and new "）。
    ///
    /// already to when not ，returncurrent。
    ///
    /// and ：Node use `Promise.all` and 「 + 」。
    /// this must**** —— 1 no ，
    /// and will in up down when to to request， Node 
    /// `checkinAndReport` 「 can can to ， etc 1 down 」，
    /// `usage` is after 。
    pub async fn checkin_and_report(
        &self,
        session: Option<&Value>,
        locale: Option<&str>,
    ) -> Result<Value, BillingError> {
        let active = match session {
            Some(session) => session.clone(),
            None => self.require_session().await?,
        };

        // no has to ； use 1 ，
        // and // 1 return， up 。
        if is_international(&active) {
            let activity = self
                .workbuddy_daily_activity(&active, WorkbuddyActivity::Full)
                .await;
            let usage = match self.query_credits_summary(Some(&active), locale).await {
                Ok(value) => value,
                Err(error) => json!({ "error": error.message }),
            };
            return Ok(json!({
                "checkinStatus": activity.get("status").cloned().unwrap_or(Value::Null),
                "claim": activity.get("claim").cloned().unwrap_or(Value::Null),
                "activity": activity.get("activity").cloned().unwrap_or(Value::Null),
                "usage": usage,
            }));
        }

        // and ：Node and ， this （ by up ）。
        // failure not —— `{success:false, code:-1, msg}`， and Node 
        // `.catch(error => ({ success:false, code:-1, msg: error.message }))` 1 
        let status = self.get_checkin_status(Some(&active)).await?;
        let claim = match self.claim_daily_checkin(Some(&active)).await {
            Ok(value) => value,
            Err(error) => json!({ "success": false, "code": -1, "msg": error.message }),
        };
        // can can to ， etc 1 down （ for Node sleep(500)）
        if claim.get("success").and_then(Value::as_bool).unwrap_or(false) {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        let usage = match self.query_credits_summary(Some(&active), locale).await {
            Ok(value) => value,
            Err(error) => json!({ "error": error.message }),
        };
        Ok(json!({
            "checkinStatus": status,
            "claim": claim,
            "usage": usage,
        }))
    }

    /// default down to （ by ）
    pub async fn checkin_and_report_default(&self) -> Result<Value, BillingError> {
        let locale = Self::current_locale();
        self.checkin_and_report(None, Some(&locale)).await
    }

    // ─── daily ─────────────────────────────────────

    /// daily： → → → will 。
    ///
    /// ── this ────────────────────────────────────────────
    /// in ****（`workbuddy::keepalive`， `activity.run`）。
    /// is 「 up 」 ：、`chat_request_send`
    /// 、 will `conversationOrigin` and UA。 this 
    /// 「config、、、」。
    ///
    /// ── ──────────────────────────────────────────────────
    /// `keepalive::models()`（ to in can ） as payload to ： that is 
    /// ** use **config， in config.json ， not has 。
    /// use no when is empty —— in （「 up has 」 is 
    /// up ， not this in 1 will when ）。
    pub async fn workbuddy_daily_activity(&self, session: &Value, mode: WorkbuddyActivity) -> Value {
        let mut blocked = None;
        let mode = if mode != WorkbuddyActivity::Claim {
            match keepalive::authorized_models() {
                Ok(_) => mode,
                Err(error) if mode == WorkbuddyActivity::Keepalive => return activity_failure(&error.message),
                Err(error) => {
                    blocked = Some(error.message);
                    WorkbuddyActivity::Claim
                }
            }
        } else { mode };
        let payload = json!({
            "mode": mode.as_str(),
            "models": keepalive::models(),
        });
        match self.admin(session, "activity.run", &payload).await {
            Ok(mut result) => {
                if let Some(reason) = blocked {
                    if let Some(activity) = result.get_mut("activity").and_then(Value::as_object_mut) {
                        activity.insert("error".to_string(), json!(reason));
                        activity.insert("modelCallsSkipped".to_string(), Value::Bool(true));
                    }
                }
                result
            }
            Err(error) => activity_failure(&error.message),
        }
    }
}

/// 1 daily** no ** when （ and successreturn 1 ）。
///
/// ── Why not is `Err` ────────────────────────────────────────
/// use （ to 、`claim_daily_checkin`、 by ） by 「`activity` 
/// has 」： this return Err will 「 up no up 」 to failure。 and 
/// 「、failure as 」 is 1 —— in 
/// this request no （ no / will for not up ），Reason `claim.msg`
/// and `activity.report.msg`， use 。
fn activity_failure(reason: &str) -> Value {
    logging::verbose("[Checkin]", &format!("WorkBuddy 国际版每日活跃未执行: {reason}"));
    json!({
        "status": Value::Null,
        "claim": {
            "success": false,
            "code": -1,
            "msg": reason,
        },
        "activity": {
            "mode": Value::Null,
            "active": Value::Null,
            "todayCheckedIn": Value::Null,
            "statusAvailable": Value::Null,
            "report": {
                "succeeded": false,
                "streakDays": Value::Null,
                "msg": reason,
            },
            "pokeSucceeded": Value::Null,
            "pokeModel": Value::Null,
            "webConversation": Value::Null,
        },
    })
}

/// will is accessToken（Node: `session?.auth?.accessToken` ）
fn has_access_token(session: &Value) -> bool {
    session
        .get("auth")
        .and_then(|auth| auth.get("accessToken"))
        .and_then(Value::as_str)
        .map(|token| !token.trim().is_empty())
        .unwrap_or(false)
}

/// will （ after `edition`，session from when ）。
/// and to not （ before has to 、 after has ），
/// this is 1 。
fn is_international(session: &Value) -> bool {
    Region::from_edition_id(session.get("edition").and_then(Value::as_str)) == Region::Intl
}
