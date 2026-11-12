//! when to —— by as daily。
//!
//! to 、 and to use `core::billing::checkin`， this when and 。
//! by when and ； from up when new ， or after already to when 、
//! ， not before 。 before ， in after 。
//! 、 and ，config and CRUD in `core::checkin_schedules`。

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};

use chrono::{DateTime, FixedOffset, TimeZone};
use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::beijing;
use crate::server::core::billing::checkin;
use crate::server::core::billing::BillingService;
use crate::server::core::checkin_schedules::{self, Schedule};
use crate::server::logging;

mod execution;
mod retry;
mod scheduler;
mod state;
pub mod times;

/// ：30 seconds to minutes， not will when （Node TICK_MS）
pub const TICK_MS: u64 = 30_000;

/// default when ： 1 （Node DEFAULT_TIME）
pub const DEFAULT_TIME: &str = "00:01";

/// when （`DEFAULT_TIME` owned ， use ）
pub fn default_time() -> String {
    DEFAULT_TIME.to_string()
}

/// can to （ up ）。default。
///
/// - **WorkBuddy **： daily to ；
/// - **WorkBuddy **：daily（ + + ，
/// `billing::activity` `workbuddy_daily_activity`）。 no has 
/// to ， up client this in 1 daily up —— 
/// success not `checkinAt`、statistics in `active`； can in to in 
///     （`billing::keepalive`）。
/// - ****：「」（ already to ）；
/// （ / 1 ） is 1 new ， not in this 
/// daily（ `raccoon::onboarding`， new ）；
/// - **AutoClaw / **： use `daily_signin` 
/// （ already to ）。 **supports** —— in is 1 path、
/// 1 id， is not （ already ），； by 
/// `billing::checkin` by provider to 。
/// - **Qoder in **：（campaign）（`providers::qoder::checkin`）。
/// Free can can no has down （）， that return 1 
/// in （「current no has can to 」）， not failure。
/// - **Qoder **（2026-10，issue #140）： 1 campaign ， up 
/// ** for request** down 「 100 Credits」—— by 
/// Qoder client / qodercli UMID （`providers::qoder::risk`），
/// Linux/Docker can 1 。 when to is not is 
/// （ not ， after ）。
/// - **Trae**：SOLO `checkin_credits` （ already to ）。
/// not is as " has "， is this ** has **：SOLO after ， use 
/// is to that ， not is 1 。 not in " this not this "
/// in ""： new （ use is can ）、requestempty for 、
/// claim `code:0` need use （ etc success）。 in that 
/// 。⚠️ when is **** ：config `autoCheckin`
/// no has `providers` when ， is "including trae"—— up after 
/// will 1 to request。 not need in to in 「 to 」。
/// - **Loomy**：** no has to ** —— 「daily」 by daily
/// new （`POST /api/v1/points/first-login`， `providers::loomy::checkin`）。
/// to for is 1 this ， not can 
/// up 。
/// - **KukuAI**：「」 daily（daily / 1 for ，
/// `freepoint/taskComplete`， `providers::kuku::checkin`）。 etc ，
/// return reward_point=0； will by genflowpro STOKEN
/// （`kuku::engine`）。
/// - **OfficeAce **：** 1 use **（daily to + 1 new 
/// is 1 up ）。 already to ， etc by up 。
/// - **CodeArts**：**ops **（ already to ，`checkin.claim` 
/// `welfare.claim`）。 and （ 、、 1 
/// etc ）， 2026-10 before 1 ** not ** this 、 in to in 1 
/// 「」 by ； by use need after ， is that 1 「 to 」。
/// ⚠️ 「none can / etc 」 is ** in **（ not ， when 
/// after ）， not is failure —— `claim_pending` this 。
/// - **ModelScope / **：** no has to ** —— 1 
/// （`GET /api/v1/users/login/info`）** to **（ etc ），
/// `providers::modelscope::checkin`。 ， not use 
/// （ 401）。 is CPA `modelscope-ratelimit` ，
/// 2026-10 。
/// - **ZCode / **： to = ** that **。 up down 
/// 1 new （`zcode-v3-start-plan-<>` this ）， in client
/// 1 down ， that 1 is to ； 1 `1003 already claimed`，
/// 「 no 」 not need need （ `providers::zcode::checkin`）。
/// ⚠️ is ** 19:00 **（：19:00 can ，23:00 09:00
/// available）， when in `times::PROVIDER_TIMES` is 19:01 ——
/// its 1 in 00:01 will to 「 no 」。
/// ⚠️ by need to already ；noneavailable or return
/// `captcha`， not already ， not etc 。
///
/// this is 「 has to or daily」 ， not is 「 has 」 ：CatPaw has 
/// no has to ， not in —— in to `skipped`。
/// 1 before to in （ 1 no has ）。
pub const CHECKIN_PROVIDERS: [&str; 19] = [
    "workbuddy",
    "workbuddy-intl",
    "raccoon",
    "autoclaw",
    "autoclaw-intl",
    "qoder",
    "qoder-intl",
    "trae",
    "loomy",
    "kuku",
    "officeace",
    // in ****： this is and to in ，
    // in in will all use 1 down （ no has can ， no need ）。
    "codearts",
    // ：「daily」 to （ already to ）。
    // ** not in ** —— no has （ for 1 401），
    // by `billing::checkin::supports_checkin` `isGuest` excluding， in to 
    // in 「 not and daily to 」Reason（ that ）。
    // （id in this ： is ， is 「 to 」 default。）
    "chatglm",
    // ModelScope（） ： to is 1 
    // （`GET {}/api/v1/users/login/info`， use ， etc ），
    // 2026-10-16 in ， `providers::modelscope::checkin` module header。
    // ， not use （ 401）。
    "modelscope",
    "modelscope-intl",
    // AtomCode（AtomGit / GitCode）： 「 to 」 is ** CodingPlan**
    // （ 1 ， `atomcode::checkin` module header）。 in this is as 
    // 「 new must 1 」「 to need can 」 need need 1 、
    // 1 and —— to has 。
    "atomcode",
    // LobsterAI（ has 「 has 」）：daily（ → up down →
    // check_in）， +100 //。 etc client（idempotencyKey + up 
    // `claimedToday` ）， `lobsterai::checkin` module header。
    "lobsterai",
    // ZCode ： to = that （ up that ）。
    // 、 not use ， id 。
    "zcode",
    "zcode-intl",
];

/// to （ = ** already **， not is 「available」）。
///
/// use has ， need ：① when 「 1 no 」 when ；
/// ② default。 will by （`resolve_checkin_targets`
/// `supports_checkin` and has ）， not empty。
/// use [`available_checkin_providers`]。
pub fn default_providers() -> Vec<String> {
    checkin_providers()
}

/// current**available** to ： + already and to can 。
///
/// ── Why not [`CHECKIN_PROVIDERS`] ─────────────────
/// that is （`PROVIDERS_ADDED_IN` and `PROVIDERS_VERSION` 
/// in up ）， is **** —— in when 
/// ， etc 「 use 」。
///
/// ── in ────────────────────────────────────────
/// is to in and 。 is （ in in 
/// will all use 1 down ）， can in after 。
/// `installed_ids()` already ， 1 。
///
/// ── this is 「** already **」， not is 「**available**」（）──
/// when is config （`normalize_providers`） and 。
/// that **must** use ： after use to need ，
/// by 「available」 will empty， down 1 config no 。
/// （ to in 、） 1 use 
/// [`available_checkin_providers`]。
pub fn checkin_providers() -> Vec<String> {
    let mut list: Vec<String> = CHECKIN_PROVIDERS.iter().map(|id| id.to_string()).collect();
    for id in crate::server::core::plugin_host::installed_ids() {
        if list.iter().any(|known| known == &id) {
            continue;
        }
        let supported = crate::server::core::providers::adapter::adapter_by_id(&id)
            .and_then(|adapter| adapter.plugin_checkin_capability())
            == Some(true);
        if supported {
            list.push(id);
        }
    }
    for id in raw_object()
        .get("pluginProvidersSeen")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !list.iter().any(|known| known == id) {
            list.push(id.to_string());
        }
    }
    list
}

/// up ** this ** to （ to in 、、）。
///
/// ── and [`checkin_providers`] （ before this ）────────────
/// `checkin_providers` = 「 already has can to 」——config ，must。
/// = 「 can 」—— to use 。 no when this is empty ，
/// that is ： to in （ WorkBuddy 
/// `billing.checkin-claim` need `billing.admin` to ）， 1 no has 。
///
/// ── has 「」（2026-10 use ）────────────────
/// ** not **「 has no has 」。 use ： not in ， this this not in ；
/// already has in ， up after 。 up 「 has 」 will 
/// no use to 1 failure 。
///
/// ⚠️ in this by 「 1 」（e.g.「 has to 」）： that is 
/// ， will to to when will 。
pub fn available_checkin_providers(_store: &AccountStore) -> Vec<String> {
    checkin_providers()
        .into_iter()
        .filter(|id| crate::server::core::plugin_host::serves_provider(id))
        .collect()
}
/// ── Why need need 1 （ not is when none）───────────
/// 「 to 」 is ** when **（`checkin_schedules`）。 new 
/// not can can in ， use is **default in ** ——
/// 2026-10 use need OfficeAce and CodeArts default。 can is 「」
/// will this 1 not （， up as 「 no 」）。
/// 1 ，config ； 1 up ， use after 
/// 。 `checkin_schedules::extend_providers_once`。
pub const PROVIDERS_VERSION: i64 = 6;

/// 「 1 」—— = this no ⇒ 。
/// 1 = up when （9 ；`qoder-intl` by `normalize_providers`
/// then `qoder` up ， not in this ）。
/// 3 = ； 4 = ModelScope and AtomCode；
/// 5 = LobsterAI（ provider PR use 1 ， and ）；
/// 6 = ZCode （ to = ）。
const PROVIDERS_ADDED_IN: [(&str, i64); 9] = [
    ("officeace", 2),
    ("codearts", 2),
    ("chatglm", 3),
    ("modelscope", 4),
    ("modelscope-intl", 4),
    ("atomcode", 4),
    ("lobsterai", 5),
    ("zcode", 6),
    ("zcode-intl", 6),
];

/// `version` after （ by ）。empty = this 1 no has new ，
/// or config new （update ）——、 not 。
pub fn providers_added_after(version: i64) -> Vec<String> {
    CHECKIN_PROVIDERS
        .iter()
        .filter(|id| {
            PROVIDERS_ADDED_IN
                .iter()
                .any(|(added, at)| added == *id && *at > version)
        })
        .map(|id| id.to_string())
        .collect()
}

/// （ from ， not to id）。
///
/// this is ** to ** （ to in 、 to use this 1 ）。
/// WorkBuddy in 「WorkBuddy / 」（ after 
/// ）， to use can —— is not is 
/// to ， this by to in `PROVIDER_DESC` and ，
/// not in 。Qoder （2026-10） after is 「Qoder in /
/// 」， not need need this 。 its not need need after ： no has ，
/// AutoClaw to in and —— already 「 / 」。
pub fn provider_label(id: &str) -> &str {
    crate::server::core::providers::PROVIDERS
        .iter()
        .find(|meta| meta.id == id)
        .map(|meta| meta.label)
        .unwrap_or(id)
}

/// 1 config ： CHECKIN_PROVIDERS id（、），
/// / empty / is to 「」—— old configfile no has this ，
/// must is default as 。
///
/// ：Qoder （2026-10） before has 9 、 no has `qoder-intl`，
/// not will 「」—— not ， before Qoder use after 
/// to will 。 this by 1 ：
/// `qoder` ， its after up `qoder-intl`（ need in need ， is 
/// 1 can ）； no （ not Qoder） not 。
pub fn normalize_providers(value: Option<&Value>) -> Vec<String> {
    let Some(list) = value.and_then(Value::as_array) else {
        return default_providers();
    };
    // use `checkin_providers()`（） not is ： to can in 
    // ， will use 。
    let known = checkin_providers();
    let picked: Vec<String> = known
        .iter()
        .filter(|id| list.iter().any(|item| item.as_str() == Some(id.as_str())))
        .cloned()
        .collect();
    let mut picked = if picked.is_empty() {
        default_providers()
    } else {
        picked
    };
    if let Some(pos) = picked.iter().position(|id| id == "qoder") {
        if !picked.contains(&"qoder-intl".to_string()) {
            picked.insert(pos + 1, "qoder-intl".to_string());
        }
    }
    picked
}

/// config.json when to 。
/// `pub(crate)`：config（`core::checkin_schedules`） in 1 down 。
pub(crate) const CONFIG_KEY: &str = "autoCheckin";

/// configerror（ for Node AutoCheckinConfigError）。
///
/// **Note statusCode is 400 not is 409**：Node `AutoCheckinConfigError`
/// `statusCode = 400`， is server.mjs `throw new
/// AutoCheckinConfigError(' to in in ，')`（run by ） errorPayload
/// to is **400 + OpenAI body**， not is 409。（2026-09-17） use 
/// time POST /api/auto-checkin to is 
/// `400 {"error":{"message":" when （00:00 - 23:59）","type":"invalid_request_error"}}`。
/// Node as ， this 1 （`CHECKIN_BUSY_STATUS` ，
/// 1 need use 409， 1 can ）。
#[derive(Clone, Debug)]
pub struct AutoCheckinConfigError {
    pub message: String,
    pub status_code: i32,
}

/// run by 「 in in 」 ： and Node AutoCheckinConfigError 1 as 400
pub const CHECKIN_BUSY_STATUS: i32 = 400;

impl AutoCheckinConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            status_code: 400,
        }
    }
}

impl std::fmt::Display for AutoCheckinConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

// ─── when （ for Node ）────────────────────────

/// JS `String(value)` ：used for normalizeTime 。
/// `value ?? ''` in Node null/undefined empty， its String()：
/// /， for / in JS is "[object Object]" / ——
/// this can can request or config.json， will then 。
fn js_text(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(Value::Object(_)) => "[object Object]".to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                Value::String(text) => text.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(","),
    }
}

/// and 1 HH:MM，return after （ for Node normalizeTime）。
///
/// then `^(\d{1,2}):(\d{1,2})$` etc ： 1-2 ASCII ，
/// and ****must（`\d` in JS not 、 not before after empty ——
/// before after empty in Node by `String(value).trim()` before ）。
pub fn normalize_time(value: Option<&Value>) -> Result<String, AutoCheckinConfigError> {
    let text = js_text(value);
    let text = text.trim();
    let parsed = text
        .split_once(':')
        .filter(|(head, tail)| {
            let digits = |part: &str| {
                !part.is_empty()
                    && part.len() <= 2
                    && part.bytes().all(|byte| byte.is_ascii_digit())
            };
            digits(head) && digits(tail) && !tail.contains(':')
        })
        .and_then(|(head, tail)| Some((head.parse::<u32>().ok()?, tail.parse::<u32>().ok()?)));
    let Some((hour, minute)) = parsed else {
        return Err(AutoCheckinConfigError::new(
            "时间格式应为 HH:MM（例如 00:01）",
        ));
    };
    if hour > 23 || minute > 59 {
        return Err(AutoCheckinConfigError::new("时间超出范围（00:00 - 23:59）"));
    }
    Ok(format!("{hour:02}:{minute:02}"))
}

/// YYYY-MM-DD：used for「 is already 」 。
///
/// Why not can use UTC ：UTC+8 00:01 in UTC down is before 1 ， will 
/// 「 no has 」 1 （ Node use when ， in 
/// etc UTC+8 —— this when ， `core::beijing`）。
pub fn date_key(now: DateTime<FixedOffset>) -> String {
    beijing::date_key(now)
}

/// when （`YYYY-MM-DD`）——「 is already 」 1 。
pub fn today_key() -> String {
    beijing::today_key()
}

/// HH:MM when （ when ）；time when None。
///
/// Node use `target.setHours(hour, minute, 0, 0)`（ to ）；
/// chrono no has set_* API， use `with_ymd_and_hms` 。
/// no has when （ as `Single`）， not need need Node that 
/// 「DST this when not in 」 —— `single()` not to None，
/// is this up when to when not 。
fn today_at(time_text: &str, now: DateTime<FixedOffset>) -> Option<DateTime<FixedOffset>> {
    normalize_today_at(time_text, now)
}

/// [`today_at`] （`times` need use 1 ， that ）
pub(crate) fn normalize_today_at(
    time_text: &str,
    now: DateTime<FixedOffset>,
) -> Option<DateTime<FixedOffset>> {
    let normalized = normalize_time(Some(&Value::String(time_text.to_string()))).ok()?;
    let (hour, minute) = normalized.split_once(':')?;
    let hour: u32 = hour.parse().ok()?;
    let minute: u32 = minute.parse().ok()?;
    let naive = now.date_naive().and_hms_opt(hour, minute, 0)?;
    beijing::offset().from_local_datetime(&naive).single()
}

// ─── ───────────────────────────────────────────────

/// `autoCheckin` for （ for / to empty for ）。
/// `pub(crate)`：config（`core::checkin_schedules`） in 1 down ，
/// and has this 1 。
pub(crate) fn raw_object() -> Map<String, Value> {
    match config::current().raw().get(CONFIG_KEY) {
        Some(Value::Object(map)) => map.clone(),
        _ => Map::new(),
    }
}

/// total（`autoCheckin.enabled`）：all not 。
/// in `checkin_schedules::Schedule::enabled`，。
pub fn master_enabled() -> bool {
    raw_object().get("enabled").and_then(Value::as_bool) == Some(true)
}

/// and `autoCheckin`（ for Node writeState `{...current, ...patch}`）。
/// config.rs `update_raw_field`，config.json its 。
/// `pub(crate)`：config in 1 down ， and has this 1 。
pub(crate) fn write_state(patch: Value) {
    update_state(|state| {
        if let Value::Object(patch) = patch {
            state.extend(patch);
        }
    });
}

pub(crate) fn update_state(apply: impl FnOnce(&mut Map<String, Value>)) {
    config::update_raw_field_with(CONFIG_KEY, |value| {
        let mut state = value.as_object().cloned().unwrap_or_default();
        apply(&mut state);
        *value = Value::Object(state);
    });
}

// ─── ───────────────────────────────────────────────

/// ： down / to this in this 「 is in 」 and 。
struct Inner {
    /// is already has 1 to in （ for Node running）
    running: bool,
    /// （ for Node timer）
    task: Option<tokio::task::JoinHandle<()>>,
    /// ：stop() after （ etc ； start will ）
    stopped: Arc<AtomicBool>,
}

/// when to ： 1 + Clone， and update / models 。
///
/// use running and this seconds； to in 
/// （： not request、 not await）。`running` use CAS 
/// 「 when 1 」， and Node down `if (running) return` 1 。
#[derive(Clone)]
pub struct AutoCheckin {
    store: AccountStore,
    billing: BillingService,
    inner: Arc<Mutex<Inner>>,
}

/// to 「 in 」 ：nonereturn is in （ abort），
/// Drop when will running —— for Node `finally { running = false }`，
/// when abort running in true。
struct RunningGuard {
    inner: Arc<Mutex<Inner>>,
}

impl Drop for RunningGuard {
    fn drop(&mut self) {
        match self.inner.lock() {
            Ok(mut guard) => guard.running = false,
            Err(poisoned) => poisoned.into_inner().running = false,
        }
    }
}

impl AutoCheckin {
    pub fn new(store: AccountStore, billing: BillingService) -> Self {
        Self {
            store,
            billing,
            inner: Arc::new(Mutex::new(Inner {
                running: false,
                task: None,
                stopped: Arc::new(AtomicBool::new(false)),
            })),
        }
    }

    /// ； in （ panic） not ， use 
    /// （ and 、 1 ）
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// is in to （getState().running）
    fn is_running(&self) -> bool {
        self.lock().running
    }

    // ─── set ───────────────────────────────────────────────

    /// set（ for Node configure ）。
    ///
    /// after ：
    /// - `enabled` → **total**（`autoCheckin.enabled`， not ）；
    /// - `time` / `providers` → to **default**（`checkin_schedules::upsert`）：
    /// no has id ， is default —— 1 will 
    /// `schedules`（ 1 and ， upsert ）。
    ///
    /// when ， tick to total will return —— and Node 1 
    /// （Node configure start not stop， tick ）。
    pub fn configure(&self, payload: &Value) -> Result<Value, AutoCheckinConfigError> {
        let has_enabled = payload.get("enabled").is_some();
        let has_time = payload.get("time").is_some();
        let has_providers = payload.get("providers").is_some();
        // when / when this （ before ， new can ）
        let has_times = payload.get("timeEnd").is_some()
            || payload.get("providerTimes").is_some()
            || payload.get("retryTime").is_some()
            || payload.get("retryGapMinutes").is_some()
            || payload.get("retryLimit").is_some();
        if !has_enabled && !has_time && !has_providers && !has_times {
            return Err(AutoCheckinConfigError::new("没有需要更新的字段"));
        }
        if has_enabled {
            // Node: `enabled !== undefined` → `patch.enabled = enabled === true`，
            // is null / / 0 will as false（ not is 「ignore」）
            let on = payload.get("enabled") == Some(&Value::Bool(true));
            write_state(json!({ "enabled": on }));
            logging::log(
                "[Checkin]",
                &format!("定时签到总开关已{}", if on { "开启" } else { "关闭" }),
            );
        }
        if has_time || has_providers || has_times {
            let mut entry = Map::new();
            entry.insert(
                "id".to_string(),
                Value::String(checkin_schedules::DEFAULT_SCHEDULE_ID.to_string()),
            );
            if has_time {
                entry.insert(
                    "time".to_string(),
                    Value::String(normalize_time(payload.get("time"))?),
                );
            }
            if let Some(value) = payload.get("providers") {
                // ： id（、 by ）；
                // 1 not no has —— when to none can ， and 。
                let Some(list) = value.as_array() else {
                    return Err(AutoCheckinConfigError::new("providers 必须是字符串数组"));
                };
                let known = checkin_providers();
                let picked: Vec<Value> = known
                    .iter()
                    .filter(|id| list.iter().any(|item| item.as_str() == Some(id.as_str())))
                    .map(|id| Value::String(id.clone()))
                    .collect();
                if picked.is_empty() {
                    let names = known
                        .iter()
                        .map(|id| provider_label(id))
                        .collect::<Vec<_>>()
                        .join("、");
                    return Err(AutoCheckinConfigError::new(format!(
                        "至少勾选一家签到提供商（{names}）"
                    )));
                }
                entry.insert("providers".to_string(), Value::Array(picked));
            }
            for key in [
                "timeEnd",
                "providerTimes",
                "retryTime",
                "retryGapMinutes",
                "retryLimit",
            ] {
                if let Some(value) = payload.get(key) {
                    entry.insert(key.to_string(), value.clone());
                }
            }
            let existing = checkin_schedules::list()
                .into_iter()
                .find(|schedule| schedule.id == checkin_schedules::DEFAULT_SCHEDULE_ID);
            entry.entry("time".to_string()).or_insert_with(|| {
                Value::from(
                    existing
                        .as_ref()
                        .map(|schedule| schedule.time.clone())
                        .unwrap_or_else(default_time),
                )
            });
            entry.entry("providers".to_string()).or_insert_with(|| {
                json!(existing
                    .as_ref()
                    .map(|schedule| schedule.providers.clone())
                    .unwrap_or_else(default_providers))
            });
            entry.insert(
                "label".to_string(),
                Value::from(existing.map(|schedule| schedule.label).unwrap_or_default()),
            );
            // not enabled： （ this when /）
            checkin_schedules::upsert(&Value::Object(entry))
                .map_err(|error| AutoCheckinConfigError::new(error))?;
        }
        // ， and check already to when ； etc when 。
        if master_enabled() {
            self.start();
        }
        Ok(self.state())
    }

    /// 「 to 」 ：****。
    ///
    /// before this return is config （`autoCheckin.providers`）—— that 
    /// in when ， not this ：「 when 」，
    /// this all can （ to has /api/accounts/{id}/checkin）。
    pub fn batch_providers(&self) -> Vec<String> {
        default_providers()
    }

    /// 1 （ up 「 to 」 by this ， and when 1 path）。
    /// in in return None —— by 「 to in in ，」。
    pub async fn run_now(&self) -> Option<Value> {
        self.fire("手动触发").await
    }

    /// ** when **（ when to 「」 by ）。
    ///
    /// and to 1 fire path（ and ）， not need 
    /// or total —— use is need this 1 。 not in return None and 
    /// 「 in 」 not can ， by in （ api::checkin_schedules）。
    pub async fn run_schedule_now(&self, id: &str) -> Option<Value> {
        let schedule = checkin_schedules::list()
            .into_iter()
            .find(|entry| entry.id == id)?;
        let reason = format!(
            "手动触发（{}）",
            checkin_schedules::display_label(&schedule)
        );
        self.fire_for(&reason, Some(id), "manual").await
    }
}

// ─── （path use stop）─────────────────────────

/// when to 。
///
/// Why not use ServerState ： is `backend::shutdown(&AppState)`，
/// that has Tauri use ， not to ServerState； Node when will 
/// `autoCheckin.stop()`（server.mjs 1047-1049 closeAll）。 use 
/// （ and config / logging 1 ）path can to 。
/// ServerState that and this is ** 1 **（bootstrap when ）。
static GLOBAL: OnceLock<AutoCheckin> = OnceLock::new();

/// （ when use 1 ， etc ）
pub fn init_global(service: AutoCheckin) -> AutoCheckin {
    let _ = GLOBAL.set(service.clone());
    service
}

/// ； when not （ by use use ）
pub fn stop_global() {
    if let Some(service) = GLOBAL.get() {
        service.stop();
    }
}
