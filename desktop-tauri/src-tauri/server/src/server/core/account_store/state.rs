//! JSON and 。
//!
//! already from `accounts.json` to SQLite `accounts` （ `sql.rs`），
//! ** 1 **： is 「 has JSON for + 1 
//! 」，`data` is file `to_value()`。
//!
//! ── Why use `serde_json::Map` not is struct ──
//! Node for （`record.anything`），，
//! is 「 use not can 」。 use struct has ：
//! 1. not → when unknown ，；
//! 2. → `enabled: "false"` this will not 。
//! this JSON （`Map<String, Value>`）， in up ** **：
//! not for when by Node （`''`/`0`/`null`/defaultenabled）， not is 。
//!
//! is this no has check —— all in file 1 ，
//! up 1 ， not need 。

use serde_json::{Map, Value};

use crate::server::core::account_store::priority::normalize_priority_value;
use crate::server::core::providers::DEFAULT_PROVIDER_ID;

/// 1 （ and 「 need need 」 path use ）。
///
/// `accounts` and `priorityScope`； its （ old 
/// currentAccountId ） not —— from old file when ， after not in 。
///
/// `priority_scope` is use ：`"global"` already by 
/// 「 1 」。 old by provider （file no has this ），
/// need not need 1 new （ `store_admin::migrate_startup`）。
#[derive(Clone, Debug, Default)]
pub struct AccountState {
    pub accounts: Vec<StoredAccount>,
    pub priority_scope: Option<String>,
}

/// `priority_scope` current： 1 
pub const PRIORITY_SCOPE_GLOBAL: &str = "global";

/// 1 ： has JSON for + 。
#[derive(Clone, Debug)]
pub struct StoredAccount {
    fields: Map<String, Value>,
}

/// ： 1 empty（ for JS `String(x || '')` use ）
fn as_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    }
}

/// optional：/empty「set」
fn as_optional_text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) if !text.is_empty() => Some(text.clone()),
        _ => None,
    }
}

fn normalize_plugin_value(value: Value) -> Value {
    if let Value::String(text) = &value {
        if let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(text) {
            return Value::Object(fields);
        }
    }
    value
}

impl StoredAccount {
    /// use 1 （ use id already ）
    pub fn from_map(mut fields: Map<String, Value>) -> Self {
        // can can 1 ； to 。
        for key in ["pluginSecret", "pluginPublic"] {
            if let Some(value) = fields.remove(key) {
                fields.insert(key.to_string(), normalize_plugin_value(value));
            }
        }
        Self { fields }
    }

    /// from ； id（ or id ） when return None
    /// （ for Node `filter(item => typeof item.id === 'string')`）。
    pub fn from_value(value: Value) -> Option<Self> {
        let Value::Object(fields) = value else {
            return None;
        };
        match fields.get("id") {
            Some(Value::String(id)) if !id.is_empty() => Some(Self::from_map(fields)),
            _ => None,
        }
    }

    /// JSON（unknown）
    pub fn to_value(&self) -> Value {
        Value::Object(self.fields.clone())
    }

    /// use （、 and etc need need use ）
    pub fn fields(&self) -> &Map<String, Value> {
        &self.fields
    }

    /// can use （ and by key ）
    pub fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.fields
    }

    // ── ────────────────────────────────────────────────

    pub fn id(&self) -> &str {
        self.fields.get("id").and_then(Value::as_str).unwrap_or("")
    }

    /// 、 `pluginPublic` / `pluginSecret` （issue #380）。
    ///
    /// ── Why need 1 （ already ）────────────────────
    /// in ** new ** when ： before is empty ，
    /// 「」 not can all（e.g. use from 1 old ）。
    /// this not etc can use —— is to up ，
    /// not is before 。
    ///
    /// ： > `pluginPublic` > `pluginSecret`。`pluginPublic` in 
    /// `pluginSecret` before ， as is `account_view` 、 already 1 
    /// （e.g. `edition` `cn` / `intl`， secret can can is `international`）。
    pub fn identity_field(&self, key: &str) -> Option<&Value> {
        if let Some(value) = self.fields.get(key).filter(|value| !value.is_null()) {
            if !value.as_str().is_some_and(|text| text.trim().is_empty()) {
                return Some(value);
            }
        }
        for holder in ["pluginPublic", "pluginSecret"] {
            if let Some(value) = self
                .fields
                .get(holder)
                .and_then(|object| object.get(key))
                .filter(|value| !value.is_null())
            {
                if !value.as_str().is_some_and(|text| text.trim().is_empty()) {
                    return Some(value);
                }
            }
        }
        None
    }

    /// [`Self::identity_field`] （ / to empty）。
    fn identity_text(&self, key: &str) -> String {
        self.identity_field(key).map(|value| as_text(Some(value))).unwrap_or_default()
    }

    /// id（`"workbuddy"` / `"raccoon"`， `core::providers`）。
    ///
    /// ****：、 as empty、 or not is when 1 
    /// `DEFAULT_PROVIDER_ID`（workbuddy）—— workbuddy up when ，
    /// 「unknown provider 」 then return（ can can is new old to ，
    /// 1 workbuddy will in error ； can in up ）。
    /// and Node `record.provider || 'workbuddy'` 1 。
    pub fn provider(&self) -> String {
        match self.fields.get("provider") {
            Some(Value::String(text)) if !text.trim().is_empty() => text.trim().to_string(),
            _ => DEFAULT_PROVIDER_ID.to_string(),
        }
    }

    /// provider （ and add use ）
    pub fn set_provider(&mut self, provider: &str) {
        self.fields
            .insert("provider".to_string(), Value::String(provider.to_string()));
    }

    /// **** provider （ need not need ）。
    ///
    /// and `provider()` ： after for default，，
    /// 「」 and 「 is workbuddy」 can —— in when 。
    pub fn provider_explicit(&self) -> Option<&str> {
        match self.fields.get("provider") {
            Some(Value::String(text)) if !text.trim().is_empty() => Some(text.trim()),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        as_text(self.fields.get("name"))
    }

    pub fn uid(&self) -> String {
        self.identity_text("uid")
    }

    pub fn nickname(&self) -> String {
        self.identity_text("nickname")
    }

    /// ： as （ for Node `type || 'personal'`）
    pub fn account_type(&self) -> String {
        let value = self.identity_text("type");
        if value.is_empty() { "personal".to_string() } else { value }
    }

    pub fn enterprise_id(&self) -> String {
        as_text(self.fields.get("enterpriseId"))
    }

    pub fn enterprise_name(&self) -> String {
        as_text(self.fields.get("enterpriseName"))
    }

    // ── ────────────────────────────────────────────────

    pub fn access_token(&self) -> String {
        as_text(self.fields.get("accessToken"))
    }

    pub fn refresh_token(&self) -> String {
        as_text(self.fields.get("refreshToken"))
    }

    /// expiration time（ms）。/「」——
    /// and Node `Number(x) || null` 1 （0 to ）。
    pub fn expires_at(&self) -> Option<f64> {
        positive_number(self.fields.get("expiresAt"))
    }

    pub fn refresh_expires_at(&self) -> Option<f64> {
        positive_number(self.fields.get("refreshExpiresAt"))
    }

    pub fn domain(&self) -> String {
        as_text(self.fields.get("domain"))
    }

    // ── and ──────────────────────────────────────────

    /// id； when return None（ by use by default）。
    ///
    /// when `pluginPublic` / `pluginSecret`（issue #380）：
    /// that for ， `edition` is after 。
    /// `resolve_edition(None)` will `cn`， 。
    pub fn edition(&self) -> Option<String> {
        let value = self.identity_text("edition");
        if value.is_empty() { None } else { Some(value) }
    }

    /// prefixPath。
    /// Note and edition ：Node use `??`（null/undefined ），
    /// empty is **valid**（ has empty before ）， not can set。
    pub fn prefix_path(&self) -> Option<String> {
        match self.fields.get("prefixPath") {
            Some(Value::String(text)) => Some(text.clone()),
            // no has this （） when for ；empty is valid。
            _ => ["pluginPublic", "pluginSecret"]
                .iter()
                .find_map(|holder| {
                    self.fields
                        .get(*holder)
                        .and_then(|object| object.get("prefixPath"))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                }),
        }
    }

    pub fn endpoint(&self) -> Option<String> {
        let value = self.identity_text("endpoint");
        if value.is_empty() { None } else { Some(value) }
    }

    pub fn platform(&self) -> Option<String> {
        let value = self.identity_text("platform");
        if value.is_empty() { None } else { Some(value) }
    }

    // ── ────────────────────────────────────────────

    pub fn priority(&self) -> i64 {
        let fallback = crate::server::core::account_store::priority::DEFAULT_PRIORITY;
        let value = self.fields.get("priority");
        let parsed = match value {
            Some(Value::Number(number)) => number.as_f64(),
            Some(Value::String(text)) => text.trim().parse::<f64>().ok(),
            _ => None,
        };
        match parsed {
            Some(number) if number.is_finite() => {
                normalize_priority_value(number.round() as i64)
            }
            _ => fallback,
        }
    }

    pub fn set_priority(&mut self, priority: i64) {
        self.fields
            .insert("priority".to_string(), Value::from(priority));
    }

    /// is enabled： as enabled（ for Node `enabled !== false`）
    pub fn enabled(&self) -> bool {
        !matches!(self.fields.get("enabled"), Some(Value::Bool(false)))
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.fields
            .insert("enabled".to_string(), Value::Bool(enabled));
    }

    pub fn added_at(&self) -> i64 {
        integer_of(self.fields.get("addedAt"))
    }

    pub fn updated_at(&self) -> i64 {
        integer_of(self.fields.get("updatedAt"))
    }

    pub fn set_updated_at(&mut self, value: i64) {
        self.fields
            .insert("updatedAt".to_string(), Value::from(value));
    }

    /// 1 ** to success** when （ms when ，0 = from or no has ）。
    ///
    /// and `updatedAt` is must ：`updatedAt` in 1 （、
    /// 、） when will new ，「 no 」 will 「set」
    /// 「」。 to is by etc ， need need is ** this ** 
    /// when ， 1 。
    pub fn checkin_at(&self) -> i64 {
        integer_of(self.fields.get("checkinAt"))
    }

    pub fn set_checkin_at(&mut self, value: i64) {
        self.fields
            .insert("checkinAt".to_string(), Value::from(value));
    }

    /// 1 ** ZCode success** when （ms when ，0 = from ）。
    ///
    /// and `checkinAt` ：ZCode no has to ， is when 
    /// （2026 that is 「 1 」， `providers::zcode::claim` module header），
    /// 「 already 」 and in by 。 use to that will 
    /// —— ZCode will to by 。
    pub fn claim_at(&self) -> i64 {
        integer_of(self.fields.get("claimAt"))
    }

    pub fn set_claim_at(&mut self, value: i64) {
        self.fields.insert("claimAt".to_string(), Value::from(value));
    }

    /// 1 to id（`zcode-v3-start-plan-trust-0928` this ）。
    ///
    /// used for and ：「 no 」 use is [`Self::claim_at`] ，
    /// id is ** up **（ when and 1 ），
    /// will in when when 。
    pub fn claim_plan_id(&self) -> String {
        as_text(self.fields.get("claimPlanId"))
    }

    pub fn set_claim_plan_id(&mut self, value: &str) {
        self.fields
            .insert("claimPlanId".to_string(), Value::String(value.to_string()));
    }

    /// ：`{ "<plan_id>": <ms when > }` —— in when 
    /// （ `AccountStore::mark_zcode_claim` 「Why 1 」 that ）。
    ///
    /// not （ / this ） 1 empty：
    /// 「 this no 」 ， by 「 no 」 use 1 （ up will 
    /// 「 already 」 and up ）， will 。
    pub fn claim_plans(&self) -> Map<String, Value> {
        let mut ledger = match self.fields.get("claimPlans") {
            Some(Value::Object(map)) => map.clone(),
            _ => Map::new(),
        };
        // ： is after ， 「 1 to 」
        // （`claimPlanId` + `claimAt`）。 this and —— otherwise after 1 
        // ， will already that 1 new optional in ， down 1 
        // 「 already 」， use as 1 。
        if ledger.is_empty() {
            let last_plan = as_optional_text(self.fields.get("claimPlanId")).unwrap_or_default();
            let last_at = self.claim_at();
            if !last_plan.is_empty() && last_at > 0 {
                ledger.insert(last_plan, Value::from(last_at));
            }
        }
        ledger
    }

    pub fn set_claim_plans(&mut self, ledger: Map<String, Value>) {
        self.fields
            .insert("claimPlans".to_string(), Value::Object(ledger));
    }

    /// new ****：`{ "<task_key>": <ms when > }`。
    ///
    /// is `raccoon::onboarding` key（`desktop_login_grant` /
    /// `mobile_login_grant`）， is this ** 1 ** when —— 「
    /// to 」（granted=true） and 「 already 」（granted=false）：
    /// 1 after not will ， after by 「 already 」。 not ，
    /// client is 「」， 1 
    /// 1 「 already 」—— and `mark_zcode_claim` up that is 1 。
    ///
    /// not （ / this ） 1 empty：「 this no 」
    /// ， by 「 no 」 use 1 （ up etc ， not will ），
    /// 1 will up 。
    pub fn onboarding_grants(&self) -> Map<String, Value> {
        match self.fields.get("onboardingGrants") {
            Some(Value::Object(map)) => map.clone(),
            _ => Map::new(),
        }
    }

    pub fn set_onboarding_grants(&mut self, ledger: Map<String, Value>) {
        self.fields
            .insert("onboardingGrants".to_string(), Value::Object(ledger));
    }

    /// new ** 1 **（Loomy 8 / CodeArts new ）：
    /// `{ "at": <ms>, "tasks": [...], "earned": n, "total": n }`，`tasks` and 
    /// return （ `done`）。
    ///
    /// and [`Self::onboarding_grants`] ： that is **** when （ 
    /// has ）， this is **** —— to that 「
    /// no has 」 use ： after and 1 not up （request），
    /// by 。 1 not will ， no has ；
    /// 「」 new （`?refresh=1`）， to new when 。
    ///
    /// in [`crate::server::core::providers::onboarding_memory::snapshot`]：
    /// that is ** JSON**（ `*_account_record()` return），
    /// has 1 ，。
    pub fn set_onboarding_settled(&mut self, snapshot: Value) {
        self.fields
            .insert("onboardingSettled".to_string(), snapshot);
    }

    /// config（return Value::Null）
    pub fn proxy(&self) -> Value {
        self.fields.get("proxy").cloned().unwrap_or(Value::Null)
    }

    pub fn set_proxy(&mut self, proxy: Value) {
        self.fields.insert("proxy".to_string(), proxy);
    }

    // ── ZCode ──────────────────────────────────────

    /// （`X-Device-Mid`）： and need ， up need **UUID **
    /// and **request**（ 1 request， 1 
    /// will ）。 when 1 ，。
    pub fn device_mid(&self) -> String {
        as_text(self.fields.get("deviceMid"))
    }

    pub fn set_device_mid(&mut self, value: &str) {
        self.fields
            .insert("deviceMid".to_string(), Value::String(value.to_string()));
    }

    /// JWT（ `zcode.z.ai` / / use ； can can as empty）。
    ///
    /// and `accessToken` ： not can （ `zcode::credentials` module header）。
    pub fn jwt(&self) -> String {
        as_text(self.fields.get("jwt"))
    }

    /// ZCode use up （`zcode::PLAN_FIELD`， `zcode::{PLAN_CODING,
    /// PLAN_START}`）。
    ///
    /// ****， 1 （ not ） in `zcode::normalize_plan` 1 ：
    /// not ，otherwise will need 。
    pub fn zcode_plan(&self) -> String {
        let key = crate::server::core::providers::zcode::PLAN_FIELD;
        let value = as_text(self.fields.get(key));
        if !value.trim().is_empty() { value }
        else { as_text(self.fields.get("pluginSecret").and_then(|secret| secret.get(key))) }
    }

    pub fn set_zcode_plan(&mut self, value: &str) {
        let key = crate::server::core::providers::zcode::PLAN_FIELD;
        self.fields.insert(key.to_string(), Value::String(value.to_string()));
        for field in ["pluginSecret", "pluginPublic"] {
            if let Some(Value::Object(fields)) = self.fields.get_mut(field) {
                fields.insert(key.to_string(), Value::String(value.to_string()));
            }
        }
    }

    /// is 「 has available」（current、 as ）
    pub fn has_token(&self) -> bool {
        !self.access_token().is_empty()
    }

    // ── （Agent2API W3-T4）─────────────────────

    /// is as 「 when 」（ §3.2）。
    ///
    /// is `desktop: true` ， not is id ——
    /// id is （`raccoon-desktop`）， is 。
    pub fn is_desktop(&self) -> bool {
        matches!(self.fields.get("desktop"), Some(Value::Bool(true)))
    }

    /// is ** has available** —— 1 。
    ///
    /// and `has_token()` ： when by ** not token**
    /// （ in `~/.box-agent/config/auth.json`， when ），
    /// 「 has accessToken」 for is 。 `has_token()` will this 
    /// in when ， 401。
    ///
    /// workbuddy and its provider not （ `desktop`
    /// not in → `has_token()`， and before ）。
    ///
    /// is （`account_store::custom_accounts`）：
    /// is `apiKey` not is `accessToken` —— not this 1 ， will 
    /// in 「current」「 hasCredentials」 none
    /// 。「 has no has empty `apiKey`」， not provider：
    /// no has this ， not （ not in this 
    /// `custom_providers`）。
    ///
    /// `has_jwt()` for ZCode valid（`jwt` is has ）：
    /// that up 「」 JWT（ use can can ），
    /// `plan: start-plan` ** use JWT**（ `zcode::plan`）—— 
    /// `accessToken` will this in when ，「 no has available」，
    /// and 「 can use 」。
    ///
    /// `no_auth()` for valid： up not need when 
    /// （ Ollama、OpenCode Zen 、none），「 no has key」
    /// not is is ** this **。must**** in up ——
    /// 「apiKey as empty」 will use key 1 ， 1 
    /// not up 401（ [`Self::no_auth`]）。
    ///
    /// `has_officeace_key()` for OfficeAce valid（`modelAppKey` /
    /// `modelAppSecret` is has ）： that not use Bearer token， is 
    /// 1 for Basic key， this 1 will （
    /// [`Self::has_officeace_key`]）。
    ///
    /// `has_plugin_secret()` for valid（ is not 
    /// `pluginSecret`， not —— `account_store::plugin_accounts`）。
    /// ， will in 1 「 no has 」。
    pub fn has_credentials(&self) -> bool {
        self.has_token() || self.is_desktop() || self.has_api_key() || self.has_jwt()
            || self.no_auth() || self.has_officeace_key() || self.has_plugin_secret()
    }

    /// is has empty `pluginSecret`（ not ）。
    ///
    /// 「 in not in 、 is not is null」 not ： that JSON by ，
    /// is in 。empty for has —— （e.g.
    /// ） will 1 empty for 「 no has can valid」。
    pub fn has_plugin_secret(&self) -> bool {
        matches!(self.fields.get("pluginSecret"), Some(value) if !value.is_null())
    }

    /// is **「 this up none need 」**（ `noAuth: true`）。
    ///
    /// and `apiKey` （[`super::custom_accounts`] 
    /// this not ）：none need no has key can ， key 
    /// this 。 is 「 has no has 」 and 「 not 」 has 1 ，
    /// not 。
    ///
    /// no has this （ has and new ）， not 
    /// —— and `has_api_key` 1 「 not provider」 。
    pub fn no_auth(&self) -> bool {
        matches!(self.fields.get("noAuth"), Some(Value::Bool(true)))
    }

    /// is has **empty** `jwt`（ZCode ）
    pub fn has_jwt(&self) -> bool {
        !self.jwt().is_empty()
    }

    /// is has **empty** `apiKey`（ ， up ）
    pub fn has_api_key(&self) -> bool {
        self.fields
            .get("apiKey")
            .and_then(Value::as_str)
            .is_some_and(|key| !key.trim().is_empty())
    }

    /// is has OfficeAce ****（`modelAppKey` + `modelAppSecret`）。
    ///
    /// ── Why need has this 1 ────────────────────────────────────────
    /// OfficeAce is ** 1 for **（`Bearer`/`accessToken` that not use ）：
    /// Basic `modelAppKey` : `modelAppSecret`。 no has this 1 when 
    /// `has_credentials()` for ⇒ in `accounts_for_provider` /
    /// `current_entry_for_provider` / `get_session_by_id` ，
    /// as 「success、directory to ， `/v1/models` this 1 
    /// no has 、 no has available」—— 1 。
    ///
    /// key need empty： 1 401， 「 has 」
    /// will failure to up （ and `has_api_key` 1 ）。
    pub fn has_officeace_key(&self) -> bool {
        let non_empty = |key: &str| {
            self.fields
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        };
        non_empty("modelAppKey") && non_empty("modelAppSecret")
    }

    /// use ID（`userId`；workbuddy for `uid`）
    pub fn user_id(&self) -> String {
        as_text(self.fields.get("userId"))
    }

    /// token expiration time（`tokenExpiresAt`，ms）
    pub fn token_expires_at(&self) -> Option<f64> {
        positive_number(self.fields.get("tokenExpiresAt"))
    }

    /// （`imported` / `manual` / …； to empty）
    pub fn source(&self) -> String {
        as_text(self.fields.get("source"))
    }

    /// ：`(, when )`
    pub fn order_key(&self) -> (i64, i64) {
        (self.priority(), self.added_at())
    }

    // ── use （ and 、 etc need need ）──

    pub fn set(&mut self, key: &str, value: Value) {
        let value = if matches!(key, "pluginSecret" | "pluginPublic") {
            normalize_plugin_value(value)
        } else {
            value
        };
        self.fields.insert(key.to_string(), value);
    }

    pub fn remove(&mut self, key: &str) {
        self.fields.remove(key);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }
}

/// in up 「 use set」（`nameCustom`）。
///
/// （ / / uid ） and use after is `name`，none
/// after —— can in ** use to when **：set to name when 
/// （`apply_patch`）， and （ provider path）。
/// ： has as ，none（ / ，
/// displayNameOf）—— no has ，「」 will 
/// when 。
///
/// （ id 1 ） no to new when ， old need ****：
/// will from old ， default。
pub(crate) fn mark_name_custom(
    record: &mut Map<String, Value>,
    explicit_name: bool,
    existing: Option<&StoredAccount>,
) {
    let carried = existing
        .and_then(|item| item.get("nameCustom"))
        .is_some_and(|value| matches!(value, Value::Bool(true)));
    if explicit_name || carried {
        record.insert("nameCustom".to_string(), Value::Bool(true));
    }
}

// ─── （ 1 ）────────────────────

// before this has 1 `priority_peers(&[StoredAccount]) -> Vec<(id, name, priority)>`
// ， in 「 + 」。 to SQLite
// after this by 1 （`sql::priorities_except` 、
// `sql::priority_holder` ）， not need need ——
// is 1 （ 1 none use will ）。
// 「 1 」 this not no ， is from Rust SQL。

/// JSON：** **。
///
/// Whymust this ：`serde_json::Value::from(1e12_f64)` will `1000000000000.0`，
/// Node `JSON.stringify(1730000000000)` `1730000000000`。
/// when 1 is ms， will `1730000000000.0` this 
/// none （ old accounts.json diff will is ），
/// can can 「 by for config」 use 。
pub fn json_number(value: f64) -> Value {
    if !value.is_finite() {
        return Value::Null;
    }
    if value.fract() == 0.0 && value.abs() < 9_007_199_254_740_992.0 {
        return Value::from(value as i64);
    }
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

/// （>0 ，/0/）
fn positive_number(value: Option<&Value>) -> Option<f64> {
    let number = match value {
        Some(Value::Number(number)) => number.as_f64()?,
        Some(Value::String(text)) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    if number.is_finite() && number > 0.0 {
        Some(number)
    } else {
        None
    }
}

/// when ：Node `addedAt || 0` is 「（ms）， 0」，
/// this by i64 （ms when use can ）。
fn integer_of(value: Option<&Value>) -> i64 {
    match value {
        Some(Value::Number(number)) => number.as_i64().unwrap_or_else(|| {
            number.as_f64().map(|float| float as i64).unwrap_or(0)
        }),
        Some(Value::String(text)) => text.trim().parse::<i64>().unwrap_or(0),
        _ => 0,
    }
}

// ─── （ for Node getCurrentEntry / getCredentialsById / getSessionById）──

/// current（ available） and its will 
#[derive(Clone, Debug)]
pub struct CurrentEntry {
    pub id: String,
    pub session: Value,
}

/// `getCredentialsById` return（ token and ）。
///
/// and Node `workbuddy-account-store.mjs` 1 for ： this is this 
/// for etc ，（refresh_account） already use name/uid/refresh_token/endpoint etc ；
/// down currentnone， and Node return for 。
#[derive(Clone, Debug)]
pub struct CredentialsById {
    /// Node `getCredentialsById().id` for etc （ use already has id， for ）
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub uid: String,
    /// Node `getCredentialsById().accessToken` for etc （ use refreshToken）
    #[allow(dead_code)]
    pub access_token: String,
    pub refresh_token: String,
    /// Node `getCredentialsById().expiresAt` for etc 
    #[allow(dead_code)]
    pub expires_at: Option<f64>,
    pub endpoint: String,
    pub prefix_path: String,
    pub platform: String,
    pub edition: String,
    /// Node `getCredentialsById().priority` for etc 
    #[allow(dead_code)]
    pub priority: i64,
    /// Node `getCredentialsById().enabled` for etc 
    #[allow(dead_code)]
    pub enabled: bool,
    pub proxy: Value,
    pub proxy_error: Option<String>,
}

/// `getSessionById` return（ will + this ）
#[derive(Clone, Debug)]
pub struct SessionById {
    /// Node `getSessionById()` return for id（ use already has id， for ）
    #[allow(dead_code)]
    pub id: String,
    pub session: Value,
    pub proxy: Value,
    pub proxy_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_name_or_carried_flag_marks_the_record() {
        // to ：
        let mut record = Map::new();
        mark_name_custom(&mut record, true, None);
        assert_eq!(record.get("nameCustom"), Some(&Value::Bool(true)));

        // （、 old none）： not —— need 
        let mut record = Map::new();
        mark_name_custom(&mut record, false, None);
        assert_eq!(record.get("nameCustom"), None);

        // no to new ： old already need （ from old ， not can ）
        let mut previous = Map::new();
        previous.insert("nameCustom".to_string(), Value::Bool(true));
        let existing = StoredAccount::from_map(previous);
        let mut record = Map::new();
        mark_name_custom(&mut record, false, Some(&existing));
        assert_eq!(record.get("nameCustom"), Some(&Value::Bool(true)));
    }

    #[test]
    fn officeace_gateway_pair_counts_as_credentials() {
        // 1 for Basic ⇒ has （otherwise will ）
        let mut fields = Map::new();
        fields.insert("modelAppKey".to_string(), Value::String("K".to_string()));
        fields.insert("modelAppSecret".to_string(), Value::String("S".to_string()));
        let record = StoredAccount::from_map(fields);
        assert!(record.has_officeace_key());
        assert!(record.has_credentials());

        // 1 ： not has （ 401， failure to up ）
        let mut only_key = Map::new();
        only_key.insert("modelAppKey".to_string(), Value::String("K".to_string()));
        assert!(!StoredAccount::from_map(only_key).has_officeace_key());

        // empty / empty not 
        let mut blank = Map::new();
        blank.insert("modelAppKey".to_string(), Value::String("K".to_string()));
        blank.insert("modelAppSecret".to_string(), Value::String("  ".to_string()));
        assert!(!StoredAccount::from_map(blank).has_officeace_key());

        // not ： has accessToken is 「 has 」
        let mut plain = Map::new();
        plain.insert("accessToken".to_string(), Value::String("T".to_string()));
        let plain = StoredAccount::from_map(plain);
        assert!(plain.has_credentials());
        assert!(!plain.has_officeace_key(), "别家不该被判成有 OfficeAce 凭据");
    }
}
