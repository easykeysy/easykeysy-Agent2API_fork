//! to ： + （ for Node workbuddy-account-routes.mjs
//! `resolveCheckinTargets` / `checkinFor` / `runCheckin` ）。
//!
//! ── Why down to core ───────────────────────────────────────
//! when to （core::auto_checkin， for workbuddy-auto-checkin.mjs） and 
//! `POST /api/accounts/checkin` must is ** 1 **。Node to this ：
//! `createAutoCheckin({ runCheckin: id => accountRoutes.runCheckin(id) })` ——
//! to is by that ，「、excluding、」
//! then 1 ， not in as 。
//!
//! Rust core not can api（core not axum， core/mod.rs ），
//! is this to this ：api/accounts.rs and core/auto_checkin has 
//! store / billing use ， then old has 1 。 use `CheckinError`
//! response（api use ， message lastResult）。//!
//! ── to not `enabled`（； before ）─────────
//! `enabled` is 「 this 」， to then is use for 
//! 1 （ when to then is for all 1 ）， and none： 1 use 
//! can to 。 and path** not ** `enabled` ——
//! use to ， up has to by 。
//!
//! ── （）────────────────────────────────────────
//! this after has ： is path `enabled`（ when bug ——
//! 「 not 」 ， is for use to will up ），
//! 「 400 / 」； is in this 。 in that 「 use 」
//! and 「 to 」 1 —— to is ****， and quota not is 
//! 1 ， use for use 「 to 」 is ， down 。
//!
//! need ，path 1 ：`available`（path， not ——
//! Node ： when unavailable not ） and `supports_checkin`
//! （ no has to / ，pathexcluding；WorkBuddy by ）。
//!
//! ── `skipped` ────────────────────────────────────────
//! 「availabletotal − can to 」， can can by **unsupported to / ** and **
//! **（`enabled` not and ）， and /api/accounts/usage 「 use 」
//! not —— 「 not use 」 not 1 。

use serde_json::{json, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::core::billing::BillingService;
use crate::server::core::billing::WorkbuddyActivity;
use crate::server::logging;

mod summary;
pub(crate) use summary::failure_of;

/// to path up error。 for Node AccountStoreError：
/// 404「 not in 」 and 400「unsupported to 」。
#[derive(Clone, Debug)]
pub struct CheckinError {
    pub message: String,
    pub status_code: i32,
}

impl CheckinError {
    fn new(message: impl Into<String>, status_code: i32) -> Self {
        Self { message: message.into(), status_code }
    }
}

impl std::fmt::Display for CheckinError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

/// no has to ， to 1 excluding this 
/// （Node: `account.edition !== 'intl'`）。
///
/// **Qoder already not this **（2026-10，issue #140）： has 
/// to （「daily 100 Credits」， `providers::qoder::checkin`
/// and `qoder::risk`）， in down provider —— and WorkBuddy
/// 1 。 after provider id ， by id 。
///
/// Accio （ ）**** no has to ： up client not to 
/// 「 to / checkin / daily」 （ `providers::accio` module header）。
/// by **provider id** excluding not is edition —— no has ， provider
/// is ， not will as 1 。
///
/// this 1 is **required **： not in will to `checkin_for` ， 1 
/// to will （ that after ）。
pub fn supports_checkin(account: &Value) -> bool {
    let provider = account
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID);
    // ── （2026-10）─────────────────────────────────
    // 「 this can not can 」 is provider ， in when （`CheckinInfo`）。
    // —— down that （edition / isGuest / accio ）
    // is to ** no old ** 。
    //
    // excluding（`exclude_guest`）： is `isGuest`，
    // that is `account_view` —— not 。
    if let Some(declared) = crate::server::core::plugin_host::checkin_of(provider) {
        if declared.exclude_guest && account.get("isGuest").and_then(Value::as_bool) == Some(true) {
            return false;
        }
        // `daily: false` = this is to in ** **， no has daily to 。
        // not to this 1 ， will 「 has to 」， that 
        // need 1 failure。（ZCode is this 1 ： has 「
        // 」； daily in already by to ，`daily` as true。）
        return declared.daily;
    }
    // ── ────────────────────────────────────────────
    // down that （edition / isGuest / accio ） is ** by **
    // ： is not JSON， not has no has `edition`。
    // 「 this id by 」 in 、 can ——
    // ， down 1 by to 
    // （ its `edition == "intl"` that 1 ： for is ）。
    if crate::server::core::plugin_host::serves_provider(provider) {
        return crate::server::core::providers::adapter::adapter_by_id(provider)
            .and_then(|adapter| adapter.plugin_checkin_capability())
            .unwrap_or(false);
    }
    // WorkBuddy no has to ， up client to 1 
    // 「daily」 up （ + + ），** and **
    // to ， in `checkin_for` by provider （workbuddy-intl）。
    // provider id is after id， not and use —— must in 
    // down edition before ，otherwise will 「intl 1 」。
    if provider == crate::server::core::providers::workbuddy::Region::Intl.provider_id() {
        return true;
    }
    // Qoder （2026-10 + issue #140）： has to 
    // （「daily 100 Credits」， `providers::qoder::checkin`
    // and `qoder::risk`），must in edition before 。
    if provider == crate::server::core::providers::qoder::endpoints::Region::Global.provider_id() {
        return true;
    }
    if account.get("edition").and_then(Value::as_str) == Some("intl") {
        return false;
    }
    // CodeArts ** and **（2026-10 by use need ， `auto_checkin::CHECKIN_PROVIDERS`
    // that 1 ）： daily is ops （ 、）， and 
    // ， in 「 can not can 」「 need not need 」 this up and to ——
    // up that by `edition` intl 1 and none（ no has ， cn）。
    // Trae and CodeArts excluding；SOLO after use is to 
    // that ， already （ by ）， not need 。
    //
    // ── **** not in to ──────────────────────
    // no has ： up for daily / / in 1 401
    // （； already to ， that in `chatglm/checkin` module header），
    // 
    // ** 1 ** after is —— 「」、 「 to 」 
    // （`checkin_center` by ）， and in to 1 
    // failure（ that in not in `claim_pending` ）。 by 
    // excluding， by to in 「 not and daily to 」Reason
    // （`api::checkin_center::out_of_scope_reason`）—— not up 。
    //
    // `isGuest`（ by `account_view` to ，
    // by `store_view::public_account` ， that ）：**must
    // **， as to in and to is 1 ——
    // 1 ， not will 。
    if account.get("isGuest").and_then(Value::as_bool) == Some(true) {
        return false;
    }
    // ── ： can ─────────────────────────────────
    // to this not in up 1 （`edition` / `isGuest` that 
    // for no has —— up is not JSON， not 
    // has no has `edition`）。 this is that can 。
    if crate::server::core::providers::kind_from_id(provider).is_none() {
        return crate::server::core::providers::adapter::adapter_by_id(provider)
            .and_then(|adapter| adapter.plugin_checkin_capability())
            .unwrap_or(false);
    }
    !crate::server::core::account_store::is_accio_family(provider)
}

/// id（ when by default provider ， and 1 ）。
///
/// to by ：WorkBuddy daily to ， its 
/// （）—— not ， 1 token 1 
/// to will 。
fn provider_of(account: &Value) -> &str {
    account
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or(crate::server::core::providers::DEFAULT_PROVIDER_ID)
}

/// this is in to 
fn matches_provider_filter(account: &Value, providers: &[String]) -> bool {
    providers.iter().any(|id| id == provider_of(account))
}

/// 「available」（Node: `account.available !== false`）
fn is_available(account: &Value) -> bool {
    account
        .get("available")
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

/// this will is not is （`account.enterpriseId` empty）。
///
/// no has / ： up for to 、 this 
/// 400 code 10001（ workbuddy2api-panel A/B ，
/// 2026-10-07）。 that 1 ， 1 failure request ——
/// 1 ， and `AuthService`「enterpriseId as null not 」
/// 1 （empty by ）。
fn is_enterprise_session(session: &Value) -> bool {
    session
        .get("account")
        .and_then(|account| account.get("enterpriseId"))
        .and_then(Value::as_str)
        .is_some_and(|value| !value.is_empty())
}

/// （`store.listAccounts().accounts`）
fn accounts_of(store: &AccountStore) -> Vec<Value> {
    store
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// to 。
///
/// （`id` as empty）：available ∩ ** in `providers` ** ∩ ，
/// `skipped` = availabletotal − can to 。 by config to （WorkBuddy / / AutoClaw
/// can ）， when to and to use 1 。** use and ** ——
/// to and is （module header「 to not enabled」）。
///
/// id： in use （** not available， not provider**），
/// 400 —— use is ， and 「」 has 1 ；
/// pathmust available and provider，otherwise will 1 。
///
/// ── path 「 not 」Why not （ has ）──────────
/// path → ****（ `skipped`）： when will 1 ，
/// use no in ， as 1 「 no has to 」 no has 。
/// path → ** 400 + Reason**： use by ，
/// need need Why not 。success or will as to 。
/// `supports_checkin` in path、 in path ——
/// ** not need as 「 1 」 its in 1 **。
pub fn resolve_checkin_targets(
    store: &AccountStore,
    providers: &[String],
    id: Option<&str>,
) -> Result<(Vec<Value>, usize), CheckinError> {
    let all = accounts_of(store);
    if let Some(id) = id.filter(|value| !value.is_empty()) {
        let found: Vec<Value> = all
            .into_iter()
            .filter(|account| account.get("id").and_then(Value::as_str) == Some(id))
            .collect();
        if found.is_empty() {
            return Err(CheckinError::new("账号不存在", 404));
        }
        // 1 by ： up this 1 no has to /（Qoder etc ）。
        // and to in 「」 that —— not 1 will use as 
        // to is not 。WorkBuddy already by 。
        //
        // to 1 ：「unsupported to 」 Reason and not （ not is this 1 no 
        // ， is no has ）， use to 「 this unsupported to 」
        // will setReason。 and to in Reason。
        if !supports_checkin(&found[0]) {
            if found[0].get("isGuest").and_then(Value::as_bool) == Some(true) {
                return Err(CheckinError::new(
                    "访客账号没有积分体系，仅登录账号可签到",
                    400,
                ));
            }
            return Err(CheckinError::new("该账号暂不支持签到或每日活跃任务", 400));
        }
        return Ok((found, 0));
    }
    let available: Vec<Value> = all.into_iter().filter(is_available).collect();
    let total = available.len();
    let eligible: Vec<Value> = available
        .into_iter()
        .filter(supports_checkin)
        .filter(|account| matches_provider_filter(account, providers))
        .collect();
    let skipped = total - eligible.len();
    Ok((eligible, skipped))
}

/// to 。 already to （ up 0 code） not error，return ——
/// before 「 already to 」 1 warn 。
///
/// ── by （ not ）────────────────────────
/// - **WorkBuddy **： daily to （`billing.claim_daily_checkin`）；
/// - **WorkBuddy **：、 and 
/// （`billing.workbuddy_daily_activity`， can in to in ）；
/// - ****：（ already to ）；
/// - **AutoClaw**： use `daily_signin` （ already to ）；
/// - **Qoder**：（campaign）， in ； need 
/// new （`providers::qoder::checkin` + `qoder::risk`）；
///     （`providers::qoder::checkin::claim_daily_checkin`）；
/// - **Trae**：Trae `checkin_credits` （ already to ）；
/// - **CodeArts**： ops （ already to ，）。
///
/// `manual` in 「 use in up 」 this 1 （ `single` /
/// `manual` as ， when and as ）， RPC to CodeArts etc 
/// need need by requestlimit 。
///
/// 1 token 1 to will ， this is required not is 
/// 。 （claim → + ） 1 ， use [`claim_result`]；
/// claim by for `{success, msg}` 。 after ****
/// default that （WorkBuddy ），unknown「」—— that 。
pub async fn checkin_for(
    store: &AccountStore,
    billing: &BillingService,
    account: &Value,
    manual: bool,
) -> Value {
    let id = account.get("id").and_then(Value::as_str).unwrap_or("").to_string();
    // name is ** to and before **： 1 
    // `displayName`（ and / request 1 ，
    // `store_view::display_name_of`；「」 in this ），
    // （ old / ） by old `name`， id。
    let name = account
        .get("displayName")
        .filter(|value| value.as_str().is_some_and(|text| !text.is_empty()))
        .cloned()
        .or_else(|| account.get("name").cloned())
        .unwrap_or(Value::Null);
    let display = name.as_str().unwrap_or(&id).to_string();
    // is provider id（`provider_of` already 1 ）。 is 1 
    // provider， need by id can 。
    let provider_id = provider_of(account);
    // ── （ and `supports_checkin` 1 ）────────────────
    // down that `match` is 。 up after 1 id has ，
    // is （`adapter_by_id` is ）—— in is 
    // 「 to ， that and no 」。
    if crate::server::core::plugin_host::serves_provider(provider_id) {
        if let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider_id) {
            if adapter.plugin_checkin_capability() == Some(true) {
                let claim = adapter
                    .checkin_claim_with_options(store, &id, manual)
                    .await
                    .map_err(|error| error.message);
                return claim_result(id, name, &display, true, claim);
            }
        }
    }
    match provider_id {
        // already to （`checkin` can ）， up 
        // that （`plugin_checkin_capability`）。
        // AutoClaw ： to already to ， up that 
        // （`plugin_checkin_capability`）。
        // Qoder daily already to ， up that 
        // （`plugin_checkin_capability`）。
        // Trae daily to already to ， up that 
        // （`plugin_checkin_capability`）。
        "workbuddy-intl" => {
            // WorkBuddy ： no has to ，「daily」 ——
            // → → + will （`billing::activity`
            // `workbuddy_daily_activity`，config `billing::keepalive`）。
            // `checkinAt` is [`day_completed`]：success / up success /
            // will 1 「 already 」（ down when as 
            // 「」，、 to ， day_completed
            // ）； in `activity` 。
            let Some(entry) = store.get_session_by_id(&id) else {
                return json!({
                    "id": id,
                    "name": name,
                    "claim": Value::Null,
                    "error": "没有可用凭证",
                });
            };
            let activity = billing
                .workbuddy_daily_activity(&entry.session, WorkbuddyActivity::Full)
                .await;
            let claim = activity
                .get("claim")
                .cloned()
                .unwrap_or_else(|| json!({ "success": false, "code": -1, "msg": "活跃任务未返回领取结果" }));
            let claim_success = claim.get("success").and_then(Value::as_bool).unwrap_or(false);
            let message = claim.get("msg").and_then(Value::as_str).unwrap_or("");
            if claim_success {
                logging::log("[Accounts]", &format!("账号 {display}: 签到成功"));
            } else {
                logging::log("[Accounts]", &format!("账号 {display}: WorkBuddy 国际版 {message}"));
            }
            json!({
                "id": id,
                "name": name,
                "claim": claim,
                "activity": activity.get("activity").cloned().unwrap_or(Value::Null),
                "error": Value::Null,
            })
        }
        // CodeArts daily ops already to （`welfare.claim`， can 
        // `checkin`）， up that 。
        // to （daily + + daily） already to ，
        // up that （`plugin_checkin_capability`）。
        // ZCode daily to （） in ， up that 
        // ； is 19:00 （ `auto_checkin::times`）。
        // excluding in 
        // `supports_checkin` `isGuest` —— that is 。
        // ModelScope（） ： to = 1 
        // （`GET {}/api/v1/users/login/info`，** use **，
        // etc ； `providers::modelscope::checkin` module header）。
        // 、 not use —— by provider id 
        // （ by provider ， 1 id not can can when ）。
        // default that （WorkBuddy ）——** not is **「 down all」。
        // this is none not `_`： 1 provider need no in up ， will 
        // to ， and not Reason（Qoder before is this ）。
        // in to this unknown「」，add 1 when will 
        // （WorkBuddy after is id， up workbuddy-intl ）。
        _ if provider_id == crate::server::core::providers::DEFAULT_PROVIDER_ID => {
            let Some(entry) = store.get_session_by_id(&id) else {
                return json!({
                    "id": id,
                    "name": name,
                    "claim": Value::Null,
                    "error": "没有可用凭证",
                });
            };
            let claim = billing
                .claim_daily_checkin(Some(&entry.session))
                .await
                .map_err(|error| error.message);
            let mut row = claim_result(id.clone(), name, &display, false, claim);
            // ── daily up ： in / ──────────────
            // to （`daily-checkin`）** to ** ， in 
            // and `/v2/report` 1 `chat_request_send` 
            // （2026-10-19 ： to after to 14→15， `growth/streak`
            // days and 1 ； 1 up after 0→1 / 0→2）。
            // `growth::daily_activity` module header。
            //
            // in `growth` up ，** not `activity`**： that is 
            // ，`day_completed` / will 「
            // already 」—— for ， up ， not etc to ，
            // use will to failure already 。
            //
            // 企业版没有个人成长体系（`/activity/growth/*` 1 403），
            // and down 1 1 ； up for （ up 200）。
            if !is_enterprise_session(&entry.session) {
                // ── daily up ： in / ──────────────
                // （`/v2/report` 、 by etc 、）
                // in ****（`workbuddy::admin` `growth.daily`）。
                // this 「 + 」——`mark_streak_days`
                // is ， not to 。
                let growth_row = match crate::server::core::providers::adapter::adapter_by_id(
                    provider_id,
                ) {
                    Some(adapter) => {
                        match adapter
                            .admin_action(store, &id, "growth.daily", &Value::Null)
                            .await
                        {
                            Ok(row) => {
                                let days = row.get("streakDays").and_then(Value::as_i64);
                                if let Some(days) = days {
                                    if !store.mark_streak_days(&id, days) {
                                        logging::verbose(
                                            "[Accounts]",
                                            &format!(" {id} can "),
                                        );
                                    }
                                }
                                let sent =
                                    row.get("reported").and_then(Value::as_bool).unwrap_or(false);
                                if sent {
                                    logging::verbose(
                                        "[Accounts]",
                                        &format!(
                                            " {display}: daily already up {}",
                                            days.map(|days| format!("（ {days} ）"))
                                                .unwrap_or_default()
                                        ),
                                    );
                                }
                                row
                            }
                            // failure：， not to 
                            Err(error) => {
                                logging::verbose(
                                    "[Accounts]",
                                    &format!(" {display} daily up failure: {}", error.message),
                                );
                                json!({
                                    "reported": false,
                                    "streakDays": Value::Null,
                                    "error": error.message,
                                })
                            }
                        }
                    }
                    None => json!({
                        "reported": false,
                        "streakDays": Value::Null,
                        "error": " no has available WorkBuddy （？）",
                    }),
                };
                if let Some(object) = row.as_object_mut() {
                    object.insert("growth".to_string(), growth_row);
                }
            }
            // ── 1 ： in to ────────────────
            // new and （ 2026-10 DeepSeek 500 ）
            // no has ， is 「usage / in 」
            // —— this down （ workbuddy2api-hub 
            // 1 in to after ， 1 ）。 1 ， already / none can 
            // ， not to ； to （ + ）。
            // no has / （ up 400 code 10001），
            // not this 。
            if !is_enterprise_session(&entry.session) {
                let mut rewards: Vec<Value> = Vec::new();
                for reward in [
                    billing.claim_gift(&entry.session).await,
                    billing.claim_compensation(&entry.session).await,
                ] {
                    match reward {
                        Ok(reward) => {
                            let credited = reward.get("success").and_then(Value::as_bool)
                                == Some(true)
                                && reward.get("credit").and_then(Value::as_i64).unwrap_or(0) > 0;
                            if credited {
                                let message =
                                    reward.get("msg").and_then(Value::as_str).unwrap_or("");
                                logging::log(
                                    "[Accounts]",
                                    &format!(" {display}: {message}"),
                                );
                            }
                            rewards.push(reward);
                        }
                        Err(error) => {
                            logging::verbose(
                                "[Accounts]",
                                &format!(" {display} : {}", error.message),
                            );
                        }
                    }
                }
                if !rewards.is_empty() {
                    if let Some(object) = row.as_object_mut() {
                        object.insert("rewards".to_string(), Value::Array(rewards));
                    }
                }
            }
            row
        }
        // AtomCode 「 to 」（ CodingPlan） already to ，
        // up that （`plugin_checkin_capability`）。
        other => {
            // ── （ and already in before ）───────────────────
            // to this can can ： no has 、 and not in up 1 。
            // 1 is as 「 no has to 」 that 
            // （ 1 after can can can ）， and up 1 。
            if let Some(adapter) =
                crate::server::core::providers::adapter::adapter_by_id(other)
            {
                if adapter.plugin_checkin_capability() == Some(true) {
                    let claim = adapter
                        .checkin_claim(store, &id)
                        .await
                        .map_err(|error| error.message);
                    return claim_result(id, name, &display, true, claim);
                }
            }
            json!({
                "id": id,
                "name": name,
                "claim": Value::Null,
                // ：`other` is provider id， will to 
                // 「workbuddy-intl to 」 this not 
                // （ after this will up ）。
                "error": format!(
                    "{} to ",
                    crate::server::core::providers::label_of(other)
                ),
            })
        }
    }
}

/// CodeArts → to claim 。
///
/// in 「 can not can 」 this up ，must and 
/// [`checkin_completed_today`] / [`claim_pending`] for —— that is 
/// （`auto_checkin::tick` need not need after ）、（ need not need `checkinAt`） and 
/// 1 to use 1 （`{id, name, claim, error}`）。
///
/// ── `log_success_msg` Why is 1 parameter not is 1 ─────────
/// and AutoClaw claim `msg` ****（「 +100」
/// 「 to success， 100 」）， has ；WorkBuddy 
/// （ Node ， not in this 「 1 」—— that will has ，
/// is use already in ）。failure 1 。
fn claim_result(
    id: String,
    name: Value,
    display: &str,
    log_success_msg: bool,
    result: Result<Value, String>,
) -> Value {
    match result {
        Ok(claim) => {
            let success = claim.get("success").and_then(Value::as_bool).unwrap_or(false);
            let msg = claim.get("msg").and_then(Value::as_str).unwrap_or("");
            if success {
                if log_success_msg && !msg.is_empty() {
                    logging::log("[Accounts]", &format!(" {display}: to success（{msg}）"));
                } else {
                    logging::log("[Accounts]", &format!(" {display}: to success"));
                }
            } else {
                logging::log("[Accounts]", &format!(" {display}: to （{msg}）"));
            }
            json!({ "id": id, "name": name, "claim": claim, "error": Value::Null })
        }
        Err(message) => {
            logging::verbose("[Accounts]", &format!(" {id} to failure: {message}"));
            json!({
                "id": id,
                "name": name,
                "claim": Value::Null,
                "error": message,
            })
        }
    }
}

/// this to is 「 already 」—— need not need `checkinAt`。
/// `pub`： to （`core::checkin_records`） outcome use ——
/// 「 already 」 in 、 and must is 1 （ 1 ）。
///
/// outcome ； down no has outcome old ：
/// 1. `success === true`： to ；
/// 2. `alreadyCompleted === true`： up already 
/// （AutoClaw `daily_signin` will this ，）；
/// 3. `msg` 「 already to / already 」：WorkBuddy 「 already to 」 in ，
/// no has available， this can 。
///
/// not need need 3 ： 「 already 」 is ** for ** ——
/// has will `granted_today > 0`， is to 1 。
///
/// ── Why not can to 「 need no 」─────────────────────────
/// failure（error / / 5xx） and 「 already to 」 is ： before 
/// can can 1 no up ， already to by will 1 。 this not is ——
/// to 4 、17 seconds after success
/// （ up up ），must to 「 up 」。
pub fn checkin_completed_today(claim: &Value) -> bool {
    if claim.get("error").and_then(Value::as_str).is_some_and(|text| !text.trim().is_empty()) {
        return false;
    }
    if let Some(outcome) = claim.get("outcome").and_then(Value::as_str) {
        return matches!(outcome, "claimed" | "done");
    }
    if claim.get("success").and_then(Value::as_bool) == Some(true)
        || claim.get("alreadyCompleted").and_then(Value::as_bool) == Some(true)
    {
        return true;
    }
    let message = claim.get("msg").and_then(Value::as_str).unwrap_or("");
    !(message.contains(" can can ") || message.contains(" can ") || message.contains("") || message.contains("failure"))
        && (message.contains(" already to ") || message.contains(" already "))
}

/// `activity` is 「 up success」（`report.succeeded`）。
///
/// WorkBuddy daily ： up （`/v2/report`） in 
/// ；「」 is will （[`web_conversation_ok`]，
/// issue #277）。 up 。
pub fn activity_report_ok(row: &Value) -> bool {
    row.get("activity")
        .and_then(|activity| activity.get("report"))
        .and_then(|report| report.get("succeeded"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// `activity` is 「 will 」（`webConversation.succeeded`）。
///
/// up 「」（daily 30 ） in agent will up 
/// （issue #277 ， `billing::activity::create_web_conversation`）。
pub fn web_conversation_ok(row: &Value) -> bool {
    row.get("activity")
        .and_then(|activity| activity.get("webConversation"))
        .and_then(|web| web.get("succeeded"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// is 「 already 」—— （`checkinAt`）、 and use 
/// 1 。
///
/// 1 ：
/// 1. **claim **（[`checkin_completed_today`]）： to / up already ；
/// 2. **daily**： up or will 1 success —— 
/// down （`active:false`，issue #277 code 12302 ） when as 
/// 「」， this 1 、 by up by 
/// （： to Bonus Pack， is 「」）。
/// old claim， is this in to in 「」、 and 
/// already `already/ok` 。
///
/// for success（`pokeSucceeded`）** not **： is 「 has use 」，
/// up not ；failure（error / ） not —— failure old 
/// ， and [`checkin_completed_today`] then 1 。
pub fn day_completed(row: &Value) -> bool {
    if row.get("error").and_then(Value::as_str).is_some_and(|text| !text.trim().is_empty()) {
        return false;
    }
    row.get("claim")
        .map(checkin_completed_today)
        .unwrap_or(false)
        || activity_report_ok(row)
        || web_conversation_ok(row)
}

/// this is 「 in 」—— not is success、 not is already 、 not is failure， is 
/// 「 no has can 」。 not checkinAt， not failure； is need 
/// retryAt（ auto_checkin::retry）， not is allnone can etc 。
pub fn claim_pending(claim: &Value) -> bool {
    if checkin_completed_today(claim) {
        return false;
    }
    // error empty = （ / ）， that not is 「 etc 」 can 
    if claim.get("error").and_then(Value::as_str).is_some_and(|text| !text.trim().is_empty()) {
        return false;
    }
    if let Some(outcome) = claim.get("outcome").and_then(Value::as_str) {
        return matches!(outcome, "inactive" | "ineligible");
    }
    let message = claim.get("msg").and_then(Value::as_str).unwrap_or("");
    message.contains(" no has can to ")
        || message.contains(" down daily")
        || message.contains(" to this down daily")
        || message.contains("none can ")
        || message.contains(" etc ")
}

/// 1 to and total（Node `runCheckin(id)`）。
///
/// `id` as None when （ when to this ）， by `providers`
/// （config ，；** id when not limit**）。
/// `reason` in （id=None） to （`checkin_history`， to in 
/// when ）； to not ，update `checkinAt`。
/// `source` is to （`core::checkin_records`，schema v11） ，
/// 1 when up —— and use this 1 。
/// ****： when up 11-128 。
pub async fn run_checkin(
    store: &AccountStore,
    billing: &BillingService,
    providers: &[String],
    id: Option<&str>,
    reason: &str,
    source: &str,
) -> Result<Value, CheckinError> {
    run_checkin_selected(store, billing, providers, id, reason, source, None).await
}

pub(crate) async fn run_checkin_selected(
    store: &AccountStore,
    billing: &BillingService,
    providers: &[String],
    id: Option<&str>,
    reason: &str,
    source: &str,
    selected: Option<&[String]>,
) -> Result<Value, CheckinError> {
    let (mut targets, mut skipped) = resolve_checkin_targets(store, providers, id)?;
    if let Some(selected) = selected {
        targets.retain(|account| account.get("id").and_then(Value::as_str)
            .is_some_and(|id| selected.iter().any(|selected| selected == id)));
        skipped = selected.len().saturating_sub(targets.len());
    }
    // 「 this is use in up 」—— has CodeArts use up （ is request，
    // has ）。 single and manual is ；
    // schedule / startup is none，must。 `checkin_for` 。
    let manual = matches!(source, "single" | "manual");
    let mut results = Vec::with_capacity(targets.len());
    for account in &targets {
        let row = checkin_for(store, billing, account, manual).await;
        // to when ： and when to is ** this 1 **（），
        // 「 already to 」 in path down will ， not need need 1 。
        // failure、 not to —— up that already to ，
        // as 1 failure success to failure is 。
        if let Some(account_id) = row.get("id").and_then(Value::as_str) {
            // is [`day_completed`]（claim ， or ）——
            // and outcome ； 1 ， in this 1 。
            if day_completed(&row) && !store.mark_checkin(account_id, logging::now_ms()) {
                logging::verbose(
                    "[Accounts]",
                    &format!(" {account_id} to when can （ can can already delete）"),
                );
            }
        }
        // to （schema v11）： 1 （ / error /
        // in ）。failure： 1 not this this 
        // to response —— and up mark_checkin 。provider by 
        // up （ no has this ， need use ）。
        crate::server::core::checkin_records::record_row(
            &row,
            account.get("provider").and_then(Value::as_str).unwrap_or(""),
            source,
            reason,
        );
        results.push(row);
    }
    // 「」 and 「 already 」success to ， succeeded；
    // （ for / up / will ）statistics in `active` ——
    // not 「」， not and succeeded， 1 。
    // Note succeeded is this ： `checkinAt` is 
    // [`day_completed`]（）， not —— before 
    // 「 to 」， after 「 is already 」。
    let succeeded = results
        .iter()
        .filter(|item| {
            day_completed(item) && item.get("claim")
                .map(checkin_completed_today)
                .unwrap_or(false)
        })
        .count();
    let active = results
        .iter()
        .filter(|item| {
            item.get("activity")
                .and_then(|activity| activity.get("pokeSucceeded"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
                // up （/v2/report， that ） and will 
                // （issue #277：「」 ）success「」。
                || activity_report_ok(item)
                || web_conversation_ok(item)
        })
        .count();
    let failures: Vec<String> = results.iter().filter_map(failure_of).collect();
    if skipped > 0 {
        logging::log(
            "[Accounts]",
            &format!(" already {skipped} （unsupported to / or not in to ）"),
        );
    }
    logging::log(
        "[Accounts]",
        &format!(
            " to : {succeeded}/{} success，{active} ",
            results.len()
        ),
    );
    // （ to in when ）。failure： up that already 
    // to ， 1 not this this to response。
    if id.is_none() {
        crate::server::core::checkin_history::record(
            &json!({
                "succeeded": succeeded,
                "active": active,
                "total": results.len(),
                "skipped": skipped,
                "results": results,
            }),
            reason,
        );
    }
    // ── ： → → （ to ， and ）──
    //
    // Why in this not is ： 「 to no has 」
    // （ +1 after can can new ）， to ； to is 
    // and when use 1 （`run_checkin` is ），
    // this path， not 1 。
    //
    // ── Why is ** after ** not is await ────────────────────────
    // need 4~6 up request（ / / / /
    // ）， up 800ms ， is seconds to minutes ——
    // API has 60 secondsrequest when （`gateway::REQUEST_TIMEOUT_MS`）。
    // await will before 「 to 」 when ， after its already ， use as 
    // failure will 1 。 this ** not etc **： to return，
    // and 。
    //
    // failure** not ** to ： in up is etc ， and 
    // （ in `run_for_accounts` ）。 1 ，
    // not 1 success to failure —— and up mark_checkin / record_row 
    // 。
    //
    // in ****（`id.is_none()`）： to is 「 this 1 」，
    // will use as by ；（ when / ） is 
    // 「 1 all」 。
    if id.is_none() {
        spawn_streak_bonus(store.clone(), targets.clone());
    }
    Ok(json!({
        "results": results,
        "succeeded": succeeded,
        "active": active,
        "total": results.len(),
        "skipped": skipped,
        "failed": failures.iter().take(5).cloned().collect::<Vec<_>>(),
        "failedCount": failures.len(),
    }))
}

/// ： after 「 → → 」， not to response。
///
/// Reason has ：
/// 1. ** not **： `run_checkin` that 「Why is after 」；
/// 2. **failure**：error， not will to to ；
/// 3. ****： 1 when 1 in （ to not will 
/// / —— up etc ，request is and ）。
fn spawn_streak_bonus(store: AccountStore, targets: Vec<Value>) {
    /// ：`swap(true)` to ，。
    static RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if RUNNING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        logging::verbose("[Streak]", " up 1 in ，");
        return;
    }
    crate::spawn_task(async move {
        // in （`growth.streak.batch` 、
        // 、）。 `{done, failed}` 。
        // no when —— this is ， not this in error。
        let Some(adapter) =
            crate::server::core::providers::adapter::adapter_by_id("workbuddy")
        else {
            RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
            return;
        };
        let summary = match adapter
            .admin_action(&store, "", "growth.streak.batch", &Value::Null)
            .await
        {
            Ok(summary) => summary,
            Err(error) => {
                logging::verbose("[Streak]", &format!("failure: {}", error.message));
                RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
                return;
            }
        };
        let _ = &targets;
        let done = summary.get("done").and_then(Value::as_u64).unwrap_or(0);
        let failed = summary.get("failed").and_then(Value::as_u64).unwrap_or(0);
        if done > 0 || failed > 0 {
            logging::log(
                "[Streak]",
                &format!(
                    "：{done} {}",
                    if failed > 0 { format!("，{failed} failure") } else { String::new() }
                ),
            );
        }
        RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
    });
}
