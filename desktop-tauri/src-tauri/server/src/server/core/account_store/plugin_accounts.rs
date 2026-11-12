//! and use （`pluginSecret` / `pluginPublic` / ）。
//!
//! in not secret in ， by ； use `accounts`
//! 、 and and 。`source` ，provider runtime id is 。
//! old use `plugin_upsert_account`， use 1 。

use serde_json::{Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store_util::js_truthy;
use crate::server::core::account_store::AccountStore;
use crate::server::core::proxies::{describe_account_proxy, normalize_account_proxy, resolve_account_proxy, ProxyResolution};
use crate::server::logging;

/// （ not JSON；）。
const SECRET_KEY: &str = "pluginSecret";
/// （）。
const PUBLIC_KEY: &str = "pluginPublic";
/// before ：`pluginLedger.<kind>`。
const LEDGER_PREFIX: &str = "pluginLedger.";

/// ： this ** not ** after secret（ is ， not is ）。
///
/// ── Why need need this ────────────────────────────────────────
/// [`flatten_builtin_secret`]： no has `pluginSecret`， need 
/// can 「」。 will `store::record` to 
/// 1 not （`priority` / `enabled` / `addedAt`…）， is when 
/// will secret 1 and 「」， not etc 。
const SKELETON_KEYS: &[&str] = &[
    "id",
    "provider",
    "name",
    "source",
    "priority",
    "enabled",
    "addedAt",
    "updatedAt",
    "rateLimits",
    "upstreamCooldownUntil",
    "upstreamBlockedReason",
    "limiterGroups",
    "limiters",
    "lowBalance",
    "checkinAt",
    "streakDays",
    "claimAt",
    "claimPlanId",
    "claimPlans",
    "needsRelogin",
    "needsReloginReason",
    "nameCustom",
    "pluginIdentity",
    "usageQuery",
];

/// 1 **** secret 。
///
/// ── Why need need this （ ）─────────────────────
/// 1 provider from 「」 to 「」 when ，** already has not will **：
/// is （`refreshToken` / `accessToken` / `projectId` …）， no has 
/// `pluginSecret`。 if `pluginSecret`， 1 all will 
/// 「」（`store.get` NOT_FOUND）。
///
/// is ： secret use （
/// `store.rs` module header）， `refreshToken` this in is 1 。
///
/// when （[`AccountStore::plugin_update_secret`]） will to `pluginSecret`
/// ，** and ** （ [`MIRROR_KEYS`]）—— is can 
/// to that ， after can use ， can when 。
fn flatten_builtin_secret(record: &StoredAccount) -> Value {
    let mut secret = Map::new();
    for (key, value) in record.fields() {
        if SKELETON_KEYS.contains(&key.as_str()) || key.starts_with(LEDGER_PREFIX) {
            continue;
        }
        secret.insert(key.clone(), value.clone());
    }
    Value::Object(secret)
}

/// when **** 。
///
/// ── Why is this ──────────────────────────────────────────
/// path is （ `from_record` in this 
/// up ）。 not after is ： new after is old ，****
/// 1 —— is ， not this 。
///
/// 「 has this 」 that ： new （） not 
/// ， not ， no need empty 1 。
const MIRROR_KEYS: [&str; 3] = ["accessToken", "refreshToken", "expiresAt"];

/// this secret（`pluginSecret` ，otherwise）。
fn secret_of(record: &StoredAccount) -> Value {
    let mut secret = match record.get(SECRET_KEY) {
        Some(value) if !value.is_null() => value.clone(),
        _ => flatten_builtin_secret(record),
    };
    let key = crate::server::core::providers::zcode::PLAN_FIELD;
    if super::is_zcode_family(&record.provider()) {
        if let Some(plan) = record.get(key).and_then(Value::as_str)
            .and_then(crate::server::core::providers::zcode::normalize_plan) {
            if let Some(fields) = secret.as_object_mut() { fields.insert(key.to_string(), Value::String(plan.to_string())); }
        }
    }
    secret
}

/// （bootstrap when install 1 ）。
///
/// ── Why need need ────────────────────────────────────────────
/// **** need ， that （`RpcAdapter` 
/// `exchange_login_code`） not to `ServerState` —— is trait ，
/// has 1 `&AccountStore` parameter，** not use ** that parameter（ in 
/// that ）。 and `sub_kernel::install_store` 1 ：bootstrap 1 。
///
/// `None`（unavailable） when will 503 ， not is 。
static SHARED: std::sync::OnceLock<AccountStore> = std::sync::OnceLock::new();

/// （`ServerState::bootstrap` use ）。 use none use 。
pub fn install_shared(store: AccountStore) {
    let _ = SHARED.set(store);
}

/// （ no = ）。
pub fn shared() -> Option<AccountStore> {
    SHARED.get().cloned()
}

impl AccountStore {
    /// 1 （`{id, provider, secret, public, proxy}`）。
    ///
    /// ── （）───────────────────────────
    /// 1. ** **： in `pluginSecret` ，；
    /// 2. ** **（ after by ）： no has 
    /// `pluginSecret`， in —— by [`flatten_builtin_secret`]
    /// secret。
    ///
    /// ── Why this not will to ─────────────────────────
    /// use （） in ** this id this ** when this （`provider` in 
    /// `accounts_for_provider` that 1 ）； not to not id。 this and 
    /// 「 not to 」 not —— that by provider by ，
    /// not is 「 no has pluginSecret」。
    pub fn plugin_account(&self, id: &str) -> Option<Value> {
        let record = self.read_record(id)?;
        let secret = secret_of(&record);
        let public = self.public_account(&record);
        let config = record
            .get("proxy")
            .or_else(|| secret.get("proxy"))
            .cloned()
            .unwrap_or(Value::Null);
        // not can and secret：update not 。
        let config = config
            .get("config")
            .filter(|value| value.is_object())
            .unwrap_or(&config);
        let source = config.get("source").and_then(Value::as_str).unwrap_or("");
        let proxy = if source == "direct" {
            serde_json::json!({ "state": "static", "config": { "source": "direct" } })
        } else if !matches!(source, "pool" | "rotate" | "clash" | "system")
            && ["url", "proxy", "address", "server"]
                .into_iter()
                .any(|key| {
                    config
                        .get(key)
                        .and_then(Value::as_str)
                        .is_some_and(|text| !text.trim().is_empty())
                })
        {
            serde_json::json!({ "state": "static", "config": config })
        } else {
            match resolve_account_proxy(Some(config)) {
                Some(ProxyResolution::Resolved(proxy)) if proxy.host.trim().is_empty() => {
                    serde_json::json!({ "state": "failed", "message": "代理配置缺少主机地址" })
                }
                Some(ProxyResolution::Resolved(proxy)) if proxy.port.is_none() => {
                    serde_json::json!({ "state": "failed", "message": "代理端口必须是 1-65535 的整数" })
                }
                Some(ProxyResolution::Resolved(proxy)) => serde_json::json!({
                    "state": "resolved",
                    "proxy": agent2api_provider_protocol::WireProxy {
                        protocol: proxy.protocol,
                        host: proxy.host,
                        port: proxy.port,
                        username: proxy.username,
                        password: proxy.password,
                    },
                }),
                Some(ProxyResolution::Failed(message)) => serde_json::json!({
                    "state": "failed",
                    "message": message,
                }),
                None => serde_json::json!({ "state": "unset" }),
            }
        };
        Some(serde_json::json!({
            "id": record.id(),
            "provider": record.provider(),
            "secret": secret,
            "public": public,
            "proxy": proxy,
        }))
    }

    /// 1 provider（ not in when `None`）。
    ///
    /// ── Why need this ────────────────────────────────────────
    /// （ new 、delete before 、 to ） has 1 id，
    /// 「 1 」 provider。 not can 
    /// `*_account_record` （ that that ， for 
    /// `pluginSecret` 1 return None）， by id 。
    ///
    /// is ****（`record.provider()`）， not `pluginSecret`
    /// —— is ， not is 。
    pub fn account_provider_of(&self, id: &str) -> Option<String> {
        self.route_account(id)?.get("provider")?.as_str().map(str::to_string)
    }

    /// id： by (provider, uid) 。
    ///
    /// ── when need need ────────────────────────────────────────
    /// `BillingService` is ** will **（`get_session_by_id` ），
    /// （`admin.action`）must id can by to for 。
    /// will no has `id` （ will `api/assistant` path to before ，
    /// no has ）， can 。
    ///
    /// is 「uid etc and provider 」： 1 uid 1 （ when is this 
    /// ， `*_accounts.rs` `identity`）。 not to （ no uid、
    /// will ）return None， use by 「 no has 」。
    ///
    /// ── uid as empty when Why to id（issue #380）──────────────────
    /// `pluginSecret` / `pluginPublic`， no has 
    /// `uid`。 that id is `user-<uid>`（`accounts::identity_of` 
    /// ）， `id` 1 。empty uid return None will this 
    /// to / 「 not to this will for 」—— 
    /// ， use to is 1 。
    pub fn account_id_of_identity(&self, provider: &str, uid: &str) -> Option<String> {
        let records = self.read_provider(provider);
        let uid = uid.trim();
        if uid.is_empty() {
            return None;
        }
        records
            .into_iter()
            .find(|record| record.uid() == uid)
            .map(|record| record.id().to_string())
    }

    /// by `user-<uid>` id （`account_id_of_identity` ）。
    ///
    /// in this then is `user-<uid>` when has —— to use 
    /// （ in uid as empty when ）， this ， not then 。
    pub fn account_id_of_user_id_suffix(&self, provider: &str, uid: &str) -> Option<String> {
        let uid = uid.trim();
        if uid.is_empty() {
            return None;
        }
        let expected = format!("user-{uid}");
        self.route_account(&expected).filter(|account| {
            account.get("provider").and_then(Value::as_str) == Some(provider)
        }).map(|_| expected)
    }

    /// -。
    ///
    /// ── Why need need 「」 this 1 ──────────────────────────────
    /// can can in and request new 1 。 no has this 1 ， after will 
    /// —— that can can already is up can new 。`expect` is 
    /// to that secret， in 「current etc 」 when 。
    ///
    /// return `Ok(false)` = already （ 1 new ）， not is error。
    ///
    /// ── ： `pluginSecret`， and ──────────
    /// as 「 after can use 」（ [`MIRROR_KEYS`]）；
    /// ** has ** that ， not empty。
    pub fn plugin_update_secret(
        &self,
        id: &str,
        expect: &Value,
        secret: &Value,
    ) -> Result<bool, String> {
        self.plugin_update_account(id, expect, secret, None)
    }

    pub fn plugin_update_account(
        &self,
        id: &str,
        expect: &Value,
        secret: &Value,
        public: Option<&Value>,
    ) -> Result<bool, String> {
        let guard = self.guard();
        self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            let Some(mut record) = crate::server::core::account_store::sql::load_by_id(&tx, id)?
            else {
                return Ok(Err(format!("账号 {id} 不存在")));
            };
            let current = secret_of(&record);
            if &current != expect {
                return Ok(Ok(false));
            }
            record.set(SECRET_KEY, secret.clone());
            if let Some(public) = public.filter(|value| value.is_object()) {
                record.set(PUBLIC_KEY, public.clone());
            }
            // （ for already in ）： after to new 
            for key in MIRROR_KEYS {
                let existed = record.get(key).is_some();
                if !existed {
                    continue;
                }
                if let Some(value) = secret.get(key) {
                    record.set(key, value.clone());
                }
            }
            if let Some(mut public) = record.get(PUBLIC_KEY).and_then(Value::as_object).cloned() {
                super::plugin_public::credential_fields(&record, &mut public);
                public.retain(|key, _| !super::plugin_public::is_private_field(key));
                record.set(PUBLIC_KEY, Value::Object(public));
            }
            // and current 1 and ， not can old new failure new new 。
            if record.provider() == "codearts" {
                if secret.get("needsRelogin").and_then(Value::as_bool) == Some(true) {
                    record.set("needsRelogin", Value::Bool(true));
                    record.set(
                        "needsReloginReason",
                        secret.get("needsReloginReason").cloned().unwrap_or(Value::Null),
                    );
                } else {
                    record.remove("needsRelogin");
                    record.remove("needsReloginReason");
                }
            }
            record.set_updated_at(logging::now_ms());
            crate::server::core::account_store::sql::update_in_place(&tx, &record)?;
            tx.commit()?;
            Ok(Ok(true))
        })
        .map_err(|error| format!("写账号失败: {}", error.message))?
    }

    /// （`kind` is ， use ）。
    pub fn plugin_ledger(&self, id: &str, kind: &str) -> Value {
        let guard = self.guard();
        let Some(record) = self.record_by_id(&guard, id) else {
            return Value::Null;
        };
        let key = format!("{LEDGER_PREFIX}{kind}");
        // null already ， not can to its or 。
        record
            .get(&key)
            .or_else(|| match kind {
                "onboarding" => record
                    .get("pluginLedger.onboardingSettled")
                    .or_else(|| record.get("onboardingSettled")),
                "onboardingSettled" => record
                    .get("pluginLedger.onboarding")
                    .or_else(|| record.get(kind)),
                "onboardingGrants" => record.get(kind),
                _ => None,
            })
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// 。
    pub fn plugin_set_ledger(&self, id: &str, kind: &str, value: &Value) -> Result<(), String> {
        let guard = self.guard();
        let Some(mut record) = self.record_by_id(&guard, id) else {
            return Err(format!("账号 {id} 不存在"));
        };
        let key = format!("{LEDGER_PREFIX}{kind}");
        if kind == super::checkin_state::LEDGER_KIND {
            let previous_at = record.get(&key).and_then(|state| state.get("at")).and_then(Value::as_i64);
            let next_at = value.get("at").and_then(Value::as_i64);
            if previous_at.zip(next_at).is_some_and(|(previous, next)| next < previous) {
                return Ok(());
            }
            let checkin_at = record.checkin_at();
            super::checkin_state::apply(&mut record, value)?;
            if record.get(&key) == Some(value) && record.checkin_at() == checkin_at {
                return Ok(());
            }
        }
        record.set(&key, value.clone());
        // to 1 ， and must。
        let alias = match kind {
            "onboarding" => Some("onboardingSettled"),
            "onboardingSettled" => Some("onboarding"),
            _ => None,
        };
        if let Some(alias) = alias {
            record.set(&format!("{LEDGER_PREFIX}{alias}"), value.clone());
        }
        if kind != super::checkin_state::LEDGER_KIND {
            record.set_updated_at(logging::now_ms());
        }
        self.with_conn(&guard, |conn| {
            crate::server::core::account_store::sql::update_in_place(conn, &record)
        })
        .map_err(|error| format!("写台账失败: {}", error.message))
    }

    /// 1 （success after by `login` that ）。
    ///
    /// ； provider ， old not 。
    /// new by use ， not can identity 。
    pub fn plugin_upsert_account(
        &self,
        id: &str,
        provider: &str,
        name: &str,
        secret: &Value,
        public: &Value,
        desktop: bool,
    ) -> Result<Value, String> {
        self.plugin_upsert_account_with_source(id, provider, name, secret, public, desktop, "plugin")
    }

    /// ； by ， not secret in provider/source。
    /// update old 、 and ，default and then not 。
    /// old not new ， not 。
    pub fn plugin_upsert_account_with_source(
        &self,
        id: &str,
        provider: &str,
        name: &str,
        secret: &Value,
        public: &Value,
        desktop: bool,
        source: &str,
    ) -> Result<Value, String> {
        self.plugin_upsert_account_with_source_and_proxy(id, provider, name, secret, public, desktop, source, None)
            .map(|(account, _)| account)
    }

    /// proxy only API already 1 config。None old ；Some(direct) 。
    /// config and not secret 1 ， 1 return proxySaved=true。
    pub fn plugin_upsert_account_with_source_and_proxy(
        &self,
        id: &str,
        provider: &str,
        name: &str,
        secret: &Value,
        public: &Value,
        desktop: bool,
        source: &str,
        proxy: Option<&Value>,
    ) -> Result<(Value, bool), String> {
        if provider.trim().is_empty() {
            return Err("插件账号缺少提供商".to_string());
        }
        if !matches!(source, "plugin" | "builtin") {
            return Err("标准账号来源必须是 plugin 或 builtin".to_string());
        }
        let account = super::plugin_identity::incoming(provider, id, secret, public, desktop);
        let identity = id;
        let guard = self.guard();
        let (id, existing) = self.plugin_account_for_upsert(&guard, provider, identity, &account)?;
        let (saved, proxy_saved) = self.with_conn_mut(&guard, |conn| {
            let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
            // after in in current， not use old 。
            let current = crate::server::core::account_store::sql::load_by_id(&tx, &id)?;
            if current.is_some() != existing.is_some()
                || current.as_ref().zip(existing.as_ref())
                    .is_some_and(|(current, previous)| current.provider() != previous.provider())
            {
                return Ok(Err("账号身份或存在状态已变化，未覆盖原账号，请重新添加".to_string()));
            }
            let is_new = current.is_none();
            let mut record = if let Some(mut record) = current {
                let zcode_plan = super::is_zcode_family(&record.provider()).then(|| record.zcode_plan());
                record.set_provider(provider);
                // use ； or when current。
                if source == "builtin" || record.get("source").is_none() {
                    record.set("source", Value::String(source.to_string()));
                }
                record.set("desktop", Value::Bool(desktop));
                // old can can in secret.proxy in 。 new when this 
                // to config， not after RAW secret。
                if proxy.is_none() && record.get("proxy").is_none() {
                    if let Some(previous) = secret_of(&record).get("proxy").filter(|value| !value.is_null()).cloned() {
                        let previous = normalize_account_proxy(&previous).ok().flatten().unwrap_or(previous);
                        record.set_proxy(previous);
                    }
                }
                record.set(SECRET_KEY, secret.clone());
                record.set(PUBLIC_KEY, public.clone());
                super::plugin_identity::remember_identity(&mut record, provider, identity);
                if let Some(plan) = zcode_plan.filter(|plan| !plan.trim().is_empty()) { record.set_zcode_plan(&plan); }
                // new to new after ， old 。
                record.remove("needsRelogin");
                record.remove("needsReloginReason");
                record
            } else {
                let mut record = account;
                super::plugin_identity::remember_identity(&mut record, provider, identity);
                record.set("id", Value::String(id.clone()));
                record.set("name", Value::String(name.to_string()));
                record.set("source", Value::String(source.to_string()));
                record.set("addedAt", Value::from(logging::now_ms()));
                let used = crate::server::core::account_store::sql::priorities_all(&tx)?;
                let priority = super::priority::next_free_priority(&used);
                if used.contains(&priority) { return Err(rusqlite::Error::InvalidQuery); }
                record.set_priority(priority);
                record
            };
            // / to （issue #380）： 、 to 、
            // path `uid` / `edition`， new will 
            // 「 not to this will for 」 for 。 new and update need ，
            // otherwise「 new 1 」 not will old up 。
            super::plugin_identity::mirror_identity(&mut record, provider);
            // RAW secret ； old not ，config。
            if let Some(proxy) = proxy { record.set_proxy(proxy.clone()); }
            record.set_updated_at(logging::now_ms());
            if is_new {
                crate::server::core::account_store::sql::put(&tx, &record)?;
            } else {
                crate::server::core::account_store::sql::update_in_place(&tx, &record)?;
            }
            let Some(saved) = crate::server::core::account_store::sql::load_by_id(&tx, &id)? else {
                return Ok(Err("保存账号后未能回读，账号与代理均未提交".to_string()));
            };
            if saved.get(SECRET_KEY) != record.get(SECRET_KEY) || saved.get(PUBLIC_KEY) != record.get(PUBLIC_KEY)
                || saved.get("proxy") != record.get("proxy")
                || proxy.is_some_and(|config| saved.get("proxy") != Some(config))
            {
                return Ok(Err("账号凭证或代理回读不一致，本次保存已回滚".to_string()));
            }
            tx.commit()?;
            Ok(Ok((saved, proxy.is_some())))
        }).map_err(|error| format!("保存账号与代理失败（请确认优先级号段未满且账号库可用）: {}", error.message))??;
        Ok((self.public_account(&saved), proxy_saved))
    }

    /// 。
    ///
    /// `pluginPublic`；、 and use by 
    /// 。 by new ， old and 。
    ///
    /// ── not down ────────────────────────────────────────────
    /// `pluginSecret` 1 not this 。 this is ， before and 
    /// can to 。
    pub(crate) fn to_plugin_public_account(&self, record: &StoredAccount) -> Value {
        let mut public = Map::new();
        public.insert("name".to_string(), record.get("name").cloned().unwrap_or(Value::Null));
        public.insert("tokenTail".to_string(), Value::Null);
        // has ； use as 。
        if let Some(Value::Object(extra)) = record.get(PUBLIC_KEY) {
            for (key, value) in extra {
                if !super::plugin_public::is_private_field(key) {
                    public.insert(key.clone(), value.clone());
                }
            }
        }
        super::plugin_public::credential_fields(record, &mut public);
        // 、enabled and ， not can 。
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        if let Some(region) = crate::server::core::providers::qoder::endpoints::Region::from_provider_id(&record.provider()) {
            public.insert("edition".to_string(), Value::String(region.edition().to_string()));
            public.insert("editionLabel".to_string(), Value::String(region.label().to_string()));
        }
        public.insert("source".to_string(), record.get("source").cloned().unwrap_or(Value::Null));
        public.insert("nameCustom".to_string(), Value::Bool(record.get("nameCustom").map(js_truthy).unwrap_or(false)));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert(
            "rateLimits".to_string(),
            record.get("rateLimits").cloned().unwrap_or_else(|| Value::Object(Map::new())),
        );
        public.insert("proxy".to_string(), describe_account_proxy(Some(&record.proxy())));
        if record.get("nameCustom").map(js_truthy).unwrap_or(false) {
            let name = record.name();
            if !name.trim().is_empty() {
                public.insert("name".to_string(), Value::String(name));
            }
        }
        Value::Object(public)
    }
}
