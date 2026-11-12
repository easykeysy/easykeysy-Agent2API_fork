//! **** and by provider （ from store.rs ，file）。
//!
//! to_public_account workbuddy （ for Node toPublicAccount）
//! public_account by provider **** to ， and 
//! snapshot /api/accounts （ providers need ）
//! provider_summary provider need `{id,label,count}`
//! accounts_for_provider provider enabled（directoryavailable）
//! list_accounts （ + ）
//!
//! ── Why ──────────────────────────────────────────
//! store.rs in after need （、、、/ 
//! ），「 1 to before / to 」 is 1 ：
//! before and and ， after 。 by file
//! and has `store_*` （`store_crud` / `store_batch` / `store_admin`）
//! file， when not in 。
//!
//! this is `impl AccountStore` （ 1 ，file not can ）。

use std::collections::HashMap;

use serde_json::{json, Map, Value};

use crate::server::core::account_store::state::{AccountState, StoredAccount};
use crate::server::core::account_store::store::{forwards_requests, AccountStore};
use crate::server::core::account_store::store_util::{js_truthy, max_concurrent_public_for, value_or, value_or_nullish};
use crate::server::core::custom_providers;
use crate::server::core::endpoints::{resolve_edition, EditionInfo};
use crate::server::core::proxies::describe_account_proxy;

/// this ， is in up 。
///
/// and has ； old ， in 。
/// ， not ； its ， 
/// not delete。
pub(crate) fn account_visible(provider: &str) -> bool {
    provider.starts_with(custom_providers::ID_PREFIX)
        || crate::server::core::providers::is_executable_provider(provider)
}

/// use ： need ， need can use 。
///
/// ── Whymust is not is 1 parameter ────────────────────
/// （、、directory new 、） is ** **，
/// in not in —— this ， use 、
/// new will 1 ， that is no has 。： not in ，
/// this not this （ use need ）。
///
/// use 1 `snapshot` 1 （`providers` need and `accounts` 
/// ）， in this 。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Visibility {
    /// （ use ）。
    All,
    /// can use （ use ， [`account_visible`]）。
    UserFacing,
}

impl AccountStore {
    // ─── ────────────────────────────────────────────

    /// → （ for Node `toPublicAccount`， for ）。
    ///
    /// Note `||` / `??` ：`prefixPath` is `??`（emptyvalid），
    /// `endpoint` is `||`（emptydefault）；`name` in Node no has ，
    /// this is 「 then not 」 not is empty。
    pub(crate) fn to_public_account(&self, record: &StoredAccount) -> Value {
        let edition: &'static EditionInfo = resolve_edition(record.edition().as_deref());
        let fields = record.fields();
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        // （Agent2API add）： when by workbuddy ，
        // to can ， not 「 no has provider = 」
        public.insert(
            "provider".to_string(),
            Value::String(record.provider()),
        );
        // name none：（ ）， when not this 
        if let Some(value) = fields.get("name") {
            public.insert("name".to_string(), value.clone());
        }
        public.insert(
            "uid".to_string(),
            value_or(fields.get("uid"), Value::String(String::new())),
        );
        public.insert(
            "nickname".to_string(),
            value_or(fields.get("nickname"), Value::String(String::new())),
        );
        public.insert(
            "type".to_string(),
            value_or(fields.get("type"), Value::String("personal".to_string())),
        );
        public.insert(
            "enterpriseId".to_string(),
            value_or(fields.get("enterpriseId"), Value::String(String::new())),
        );
        public.insert(
            "enterpriseName".to_string(),
            value_or(fields.get("enterpriseName"), Value::String(String::new())),
        );
        public.insert(
            "tokenTail".to_string(),
            value_or(fields.get("tokenTail"), Value::String(String::new())),
        );
        public.insert(
            "expiresAt".to_string(),
            value_or(fields.get("expiresAt"), Value::Null),
        );
        public.insert(
            "hasRefreshToken".to_string(),
            Value::Bool(
                fields
                    .get("refreshToken")
                    .map(js_truthy)
                    .unwrap_or(false),
            ),
        );
        // 「 need need new 」（issue #215）： up will 、refresh token 1 and 
        // when by new 。 「 new failure」「 need need new 」——
        // for use down 1 not （ before 1 ， after must）。
        // in （false ）， before not 「」。
        public.insert(
            "needsRelogin".to_string(),
            Value::Bool(
                fields
                    .get("needsRelogin")
                    .map(js_truthy)
                    .unwrap_or(false),
            ),
        );
        public.insert(
            "needsReloginReason".to_string(),
            value_or(fields.get("needsReloginReason"), Value::String(String::new())),
        );
        public.insert(
            "prefixPath".to_string(),
            value_or_nullish(fields.get("prefixPath"), Value::String(String::new())),
        );
        public.insert(
            "endpoint".to_string(),
            value_or(fields.get("endpoint"), Value::String(String::new())),
        );
        public.insert("edition".to_string(), Value::String(edition.id.to_string()));
        public.insert(
            "editionLabel".to_string(),
            Value::String(edition.label.to_string()),
        );
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert(
            "proxy".to_string(),
            describe_account_proxy(Some(&value_or(fields.get("proxy"), Value::Null))),
        );
        // Node is `record.rateLimits || {}`：
        // （/ when ， before by for will to undefined，
        // and Node as 1 「」 need ）
        public.insert(
            "rateLimits".to_string(),
            value_or(fields.get("rateLimits"), Value::Object(Map::new())),
        );
        // all as available（/available 3 ）
        public.insert("available".to_string(), Value::Bool(true));
        Value::Object(public)
    }

    /// （** not **）。
    ///
    /// accounts by ****return（ and Node by filereturn 1 ）—— by 
    /// ， not need in this ，otherwise and Node as 。
    /// 「」 in SQLite down is `rowid` ， and old accounts.json 
    /// in down for （ by 、add to 、 not ），
    /// `sql.rs` module header「」 1 。
    ///
    /// ── this is **** use ， ──────────────────
    /// 、directory new 、 this must —— no when 
    /// need can already has ，otherwise「 not in 」 will 、
    /// 1 。** need [`Self::list_visible_accounts`]**（ by in ）。
    ///
    /// ── 「current」 ──────────────────────────────
    /// · `currentAccountId`：****（`pick_current`：enabled + has available +
    /// ）， and `/api/session` `session.currentAccountId` 。
    /// ★ / 「」 this 。
    /// · `currentAccountIds`：provider → this id（，
    /// `pick_current_for_provider`）。 after 「 this 1 has no has available」
    /// ， is as not old client。
    ///
    /// `providers` is provider need （`{id,label,count}`）：
    /// use and 「 use 」，**must in 
    /// up to **， before not request。 is provider **total**
    /// （ use ）—— and 「N 」 1 。
    pub fn list_accounts(&self) -> Value {
        let state = self.read_state();
        self.snapshot(&state, Visibility::All)
    }

    /// use ：** can use that **（ [`account_visible`]）。
    ///
    /// all to use this 1 （`/api/session`、`/api/accounts`、
    /// to in 、…）。 no when not —— use need is 
    /// 「 not in ， this not in 」； in ， up 。
    pub fn list_visible_accounts(&self) -> Value {
        let state = self.read_state();
        self.snapshot(&state, Visibility::UserFacing)
    }

    /// proxy to id （）。
    ///
    /// is `proxy.source == "pool"`（** **，`normalize_account_proxy`
    /// —— `{source:'pool', proxyId}`；`config` is `describe_account_proxy`
    /// after ， before not in —— can can ，
    /// ）。failure use （ already etc ）statistics to —— that is need need 
    /// 「 has 」 。 is lazy listeners
    /// （`sub_kernel::runtime` to use ，
    /// `proxy_pool::subscription_node_keys`）， this not enabled：
    /// use use 「 in use 」—— new enabled not this 1 etc 。
    ///
    /// ── this （）──────────────────────────
    /// by `proxy.config.source` —— that is ****， up 
    /// no has this ， use as empty：「 use 」 that 1 from ，lazy
    /// listeners 「」 in ， 1 up 、pinned after 
    /// （all use not to ，）。
    ///
    /// ── （rotate） ──────────────────────────────────
    /// use ** when can can use **， 「 in use 」：
    /// `all: true` when enabled id return（ is 「 
    /// enabled can can to 」， that will in use when 「」 ，
    /// etc ）；return id（ not in not 
    /// ，）。 this 1 use `proxy_rotate::candidate_ids` ——
    /// 「 can can use to 」 in use and must is 1 。
    pub fn referenced_pool_ids(&self) -> std::collections::HashSet<String> {
        let state = self.read_state();
        let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for record in &state.accounts {
            let proxy = record.proxy();
            // ； `config` （ ）
            let config = proxy
                .get("config")
                .filter(|value| value.is_object())
                .unwrap_or(&proxy);
            if config.get("source").and_then(Value::as_str) == Some("pool") {
                if let Some(id) = config
                    .get("proxyId")
                    .and_then(Value::as_str)
                    .filter(|id| !id.is_empty())
                {
                    ids.insert(id.to_string());
                }
            }
            if crate::server::core::proxies::is_rotate_config(config) {
                ids.extend(crate::server::core::proxy_rotate::candidate_ids(config));
            }
        }
        ids
    }

    /// id → this 1 when （`max(addedAt, updatedAt)`，ms）。
    ///
    /// to 「 this in not 」 when use （ `core::usage_query` 
    /// ）。 when ： new 、 new 、token 
    /// new 、set will before ，** not as this **
    /// —— ， not to 「 token 1 new 」 1 。
    ///
    /// is not （ and is 「」）：
    /// by 「 old 」， 1 failure， 1 
    /// —— use 1 can to new （ `usage_query` ）。
    pub fn account_change_times(&self) -> HashMap<String, i64> {
        let state = self.read_state();
        state
            .accounts
            .iter()
            .map(|record| {
                (
                    record.id().to_string(),
                    record.added_at().max(record.updated_at()),
                )
            })
            .collect()
    }

    /// already when （CRUD need in 1 「 + 」）
    ///
    /// ── in in when （2026-10 use ）────────────
    /// / / to already 。 no when ，
    /// that ** in not in ** —— use need is 「 not in etc 
    /// this not in 」， not is 「、 no 」。 1 not ：
    /// （`account_transfer::export_accounts`）、 not this ；
    /// up after this 。
    ///
    /// ； and by `account_visible` 
    /// and 。 old ， and not 。
    pub(crate) fn snapshot(&self, state: &AccountState, visibility: Visibility) -> Value {
        // 1 、 1 （ need `&[StoredAccount]`）
        let mut visible: Vec<StoredAccount> = Vec::with_capacity(state.accounts.len());
        // by provider （ use 「 etc 」）。
        // in `UserFacing` when —— `All` is 「 need 」， that 
        // 「」 as empty， is 1 。
        let mut hidden: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for record in &state.accounts {
            if visibility == Visibility::All || account_visible(record.provider().as_str()) {
                visible.push(record.clone());
            } else {
                *hidden.entry(record.provider()).or_insert(0) += 1;
            }
        }
        // ：provider is 
        // （ unknown id —— has ， not this ignore）
        let mut current_ids = Map::new();
        for record in &visible {
            let provider = record.provider();
            if current_ids.contains_key(&provider) {
                continue;
            }
            let head = Self::pick_current_for_provider(&visible, &provider)
                .map(|record| Value::String(record.id().to_string()))
                .unwrap_or(Value::Null);
            current_ids.insert(provider, head);
        }
        let global_current = Self::pick_current(&visible)
            .map(|record| Value::String(record.id().to_string()))
            .unwrap_or(Value::Null);
        // need can ， has when must can 。
        let hidden = super::hidden_accounts::summary(hidden);
        let mut snapshot = json!({
            "currentAccountId": global_current,
            "currentAccountIds": Value::Object(current_ids),
            "accounts": visible
                .iter()
                .map(|record| self.public_account(record))
                .collect::<Vec<_>>(),
            "providers": self.provider_summary(&visible),
            "hiddenAccounts": hidden,
        });
        crate::server::core::account_order::enrich_snapshot(&mut snapshot);
        snapshot
    }

    /// → （** by provider **）。
    ///
    /// not ：workbuddy has uid/nickname/edition/enterprise…，
    /// has userId/tokenExpiresAt/desktop and ** no has **（ §5/§6），
    /// CatPaw has uid/loginName/tokenTail/desktop and ** no has **/ to （W5-T-d4）。
    /// in this ， use （、、）
    /// 1 not use —— to is provider this has 。
    ///
    /// to is **workbuddy **， this for **unknown** provider id is 
    /// （ and `StoredAccount::provider()` 1 ：file id
    /// can in up ）。AutoClaw in ** not **
    /// （`api::accounts::add_account` 400）， will in 
    /// 1 can —— that before ， 1 has this ，
    /// workbuddy total。
    ///
    /// ── `hasCredentials` / `chatSupported`： 1 **** ──────
    /// in this ** 1 ** not is ： has 1 
    /// （`has_credentials` / `supports_chat`），「 has 」「 can 」
    /// is and 「current」 use that 。 before need by 
    /// 「 this 1 will 」 when （ after to not ），must to 1 
    /// ，otherwise will 「 ★ its when will none 」 。
    /// in **、**， before not by provider 。
    ///
    /// `nameCustom` / `checkinAt` / `usageQuery` / `lowBalance` / `limiters` /
    /// `maxConcurrent` 1 。`maxConcurrent` its not can down to ：
    /// is ** has **， need 1 ， to etc 
    /// 0 = not （limit）， not 。
    ///
    /// （ and 「 / updateresponse」）must 1 ， that response
    /// 1 [`Self::public_account`] ， not is `to_*_public_account`
    /// —— after will this 。
    ///
    /// add： has 1 not ， old clientignore can 。
    pub(crate) fn public_account(&self, record: &StoredAccount) -> Value {
        // ── （must in all before ）─────────────────
        // up after ， new by （ in `pluginSecret` ，
        // no has ）。 if by provider id down ， that 
        // will 「uid empty、tokenTail empty、hasRefreshToken false」 empty
        // —— can use ， up 。
        //
        // is ** up `pluginSecret`** not is 「 this id by 」：
        // no when ，（ no has `pluginSecret`） old 
        // ，path not 。
        let shaped = if record.has_plugin_secret() {
            self.to_plugin_public_account(record)
        } else if record.provider() == super::RACCOON_PROVIDER_ID {
            self.to_raccoon_public_account(record)
        } else if record.provider() == super::CATPAW_PROVIDER_ID {
            self.to_catpaw_public_account(record)
        } else if super::is_autoclaw_family(&record.provider()) {
            // （`autoclaw` / `autoclaw-intl`） use this 1 ：
            // 、、deviceId 1 ，
            // in （ that is and ， not ）
            self.to_autoclaw_public_account(record)
        } else if super::is_qoder_family(&record.provider()) {
            // （`qoder` / `qoder-intl`） use this 1 ： by 
            // provider id（ `mode` ） `edition` / `editionLabel`
            self.to_qoder_public_account(record)
        } else if record.provider() == super::codearts_accounts::CODEARTS_PROVIDER_ID {
            self.to_codearts_public_account(record)
        } else if super::is_cline_family(&record.provider()) {
            // （`cline-free` / `cline-pass`） use this 1 
            self.to_cline_public_account(record)
        } else if super::is_accio_family(&record.provider()) {
            // （`accio` / `accio-cn`） use this 1 ：、
            // 1 ， in and （ that is and 
            // ， not ）
            self.to_accio_public_account(record)
        } else if super::is_zcode_family(&record.provider()) {
            // （`zcode` / `zcode-intl`） use this 1 ：
            // 1 ， in and when up provider （ that is 
            // and ， use `edition` ）
            self.to_zcode_public_account(record)
        } else if record.provider() == super::TRAE_PROVIDER_ID {
            // Trae has 1 （ SOLO）： is ** 1 ** not is 1 ，
            // this no has `is_trae_family` —— will when kind、
            // 1 ， not need up `region` 。
            // this 1 ****： new has `pluginSecret`，
            // up 1 already （`account_view` by to ）。
            self.to_trae_public_account(record)
        } else if record.provider() == super::LOOMY_PROVIDER_ID {
            // Loomy（）： 1 （）， userId /
            // phone / session （ `loomy_accounts.rs`）
            self.to_loomy_public_account(record)
        } else if super::is_monkeycode_family(&record.provider()) {
            // MonkeyCode （`monkeycode` / `monkeycode-intl`） use this 1 
            // ： 1 ， in （ that is and 
            // ， use `edition` ）
            self.to_monkeycode_public_account(record)
        } else if record.provider() == super::kuku_accounts::KUKU_PROVIDER_ID {
            // KukuAI（ AI）： 1 （ Cookie / ），
            // uid(=uk) / loginName(=) / tokenTail（
            // `kuku_accounts.rs`）
            self.to_kuku_public_account(record)
        } else if record.provider() == super::COMMANDCODE_PROVIDER_ID {
            // Command Code： 1 （ `user_` API Key）， key
            // 、 and 「 has no has key」（ `commandcode_accounts.rs`）
            self.to_commandcode_public_account(record)
        } else if record.provider() == super::ANTIGRAVITY_PROVIDER_ID {
            // Antigravity（Google）： 1 （ Google refresh token），
            // token 、 refresh token and 
            // 「 has no has refreshToken / accessToken」（ `antigravity_accounts.rs`）
            self.to_antigravity_public_account(record)
        } else if record.provider() == super::OFFICEACE_PROVIDER_ID {
            // OfficeAce（ as ）： and to when ， Basic that 1 for 
            // and when not this （ `to_officeace_public_account`）。
            // this 1 when will down workbuddy ： 1 uid/edition
            // and none empty，「 new Token」 by hasRefreshToken
            // is **** in —— that not is ， is 。
            self.to_officeace_public_account(record)
        } else if record.provider() == super::orcaterm_accounts::ORCATERM_PROVIDER_ID {
            // OrcaTerm（client）： 1 （ accessToken /
            // ）， uid(=userId) / loginName(=) /
            // tokenTail / tokenExpireAt（ is in can ，
            // `orcaterm_accounts.rs`）
            self.to_orcaterm_public_account(record)
        } else if super::is_modelscope_family(&record.provider()) {
            // ModelScope（） （`modelscope` / `modelscope-intl`） use 
            // this 1 ： 1 ， in （ that is and 
            // ， not ）。， 4 and 「 has no has 」
            // （ `modelscope_accounts.rs`）。
            self.to_modelscope_public_account(record)
        } else if record.provider() == super::ATOMCODE_PROVIDER_ID {
            // AtomCode（AtomGit）： need ** **
            // （accessToken and refreshToken）—— before that in this is ，
            // refreshToken is valid ， not can down 。
            self.to_atomcode_public_account(record)
        } else if record.provider() == super::LOBSTERAI_PROVIDER_ID {
            // LobsterAI： need ** + **
            // （refreshToken / uuid / keyfrom）。 is 「 can 」 ，
            // 1 down etc can ； need is canRefresh 。
            self.to_lobsterai_public_account(record)
        } else if record
            .provider()
            .starts_with(crate::server::core::custom_providers::ID_PREFIX)
        {
            // 。 use ** before ** not is `custom_providers::
            // is_custom_provider_id`（ after need in config）：
            // is ， after （/）
            // ， not is workbuddy uid/edition etc 
            // and none 
            self.to_custom_public_account(record)
        } else {
            self.to_public_account(record)
        };
        match shaped {
            Value::Object(mut fields) => {
                for key in ["upstreamCooldownUntil", "upstreamBlockedReason"] {
                    if let Some(value) = record.get(key) { fields.insert(key.to_string(), value.clone()); }
                    else { fields.remove(key); }
                }
                if !record.has_plugin_secret() {
                    super::plugin_public::credential_fields(record, &mut fields);
                }
                fields.insert(
                    "hasCredentials".to_string(),
                    Value::Bool(record.has_credentials()),
                );
                // 「 need need new 」（issue #215）： up will 、refresh token
                // 1 and when by new 。 「 new failure」「 need need new 
                // 」—— for use down 1 not （ before 1 ， after must
                // ）。
                //
                // ── Why in this 1 1 ──────────────────────────────
                // `to_public_account` that 1 ，** not 
                // that **（ by `account_view` to ， up 1 
                // ）—— is up `needsRelogin`， not to 。
                // 「」 is ****， and by none：Loomy this 
                // none its can （ up 020002 / 100002 when by 
                // `upstream::provider_loop` this ， that ）。
                // in （false ）， before not 「」； 
                // this none —— 。
                fields.insert(
                    "needsRelogin".to_string(),
                    Value::Bool(
                        record
                            .fields()
                            .get("needsRelogin")
                            .map(js_truthy)
                            .unwrap_or(false),
                    ),
                );
                fields.insert(
                    "needsReloginReason".to_string(),
                    value_or(
                        record.fields().get("needsReloginReason"),
                        Value::String(String::new()),
                    ),
                );
                // no has can ：「enabled not will 」，
                // not is 1 enabled。
                // in can ，config down this as true ——
                // this is as 「 can use 、 no up 」 this 
                // will will ， need need has 。
                fields.insert(
                    "chatSupported".to_string(),
                    Value::Bool(forwards_requests(record)),
                );
                // 1 to success when （0 = from ）。 1 by and up 
                // ： is 「 this to 」 this 1 ， 
                // 1 （`checkinAt`）， not by provider 。
                //
                // to is ** when ** not is 「 no 」 ： need by 
                // use when ， that in up already has （ when 
                // 「 / 」 is ， accounts-model.js startOfDay）。
                // when can 「 08:30 already to 」 this 。
                //
                // as （none when 0） not is ： by `Number(...) || 0` ，
                // can ， 「」 and 「 as 0」 not need need 。
                if super::is_zcode_family(&record.provider()) {
                    let plan = crate::server::core::providers::zcode::normalize_plan(&record.zcode_plan())
                        .unwrap_or(crate::server::core::providers::zcode::PLAN_CODING);
                    fields.insert(crate::server::core::providers::zcode::PLAN_FIELD.to_string(), Value::String(plan.to_string()));
                }
                for key in ["claimAt", "claimPlanId", "claimPlans"] {
                    if let Some(value) = record.get(key) {
                        fields.insert(key.to_string(), value.clone());
                    }
                }
                fields.insert("checkinAt".to_string(), Value::from(record.checkin_at()));
                fields.insert("checkedInToday".to_string(), Value::Bool(super::checkin_state::completed(record)));
                fields.insert("dailyCheckin".to_string(), super::checkin_state::record_status(record)
                    .cloned().unwrap_or(Value::Null));
                // in （WorkBuddy ； to after ，
                // `AccountStore::mark_streak_days`）。 as （0 = no to ），
                // and checkinAt 1 by ： by ， not 「」。
                // has will this ， 0 —— not by provider is ：
                // ， not 。
                fields.insert(
                    "streakDays".to_string(),
                    Value::from(
                        record
                            .fields()
                            .get("streakDays")
                            .and_then(Value::as_i64)
                            .unwrap_or(0),
                    ),
                );
                // and limit（all use ）： 1 by and up 
                // —— （`routing::max_concurrent_of`） and before 
                // is ** 1 **。 as （0 = not ）， and ** in this 1 **：
                // from before is `insert`， is that 
                // （monkeycode / antigravity / officeace） —— 
                // 0 = not ，limit****。CodeArts 「0 = by default and 」
                // by `max_concurrent_public_for` 1 ， 0 is not 。
                fields.insert(
                    "maxConcurrent".to_string(),
                    Value::from(max_concurrent_public_for(
                        &record.provider(),
                        record.get("maxConcurrent"),
                    )),
                );
                fields.insert(
                    "allowRouteSkip".to_string(),
                    Value::Bool(record.get("allowRouteSkip").and_then(Value::as_bool).unwrap_or(true)),
                );
                // use set（update_account to name when ，
                // apply_patch）。 as not is ：「
                // / etc default」 is 「 as 」， not 
                // 「」。
                fields.insert(
                    "nameCustom".to_string(),
                    Value::Bool(
                        record
                            .fields()
                            .get("nameCustom")
                            .map(js_truthy)
                            .unwrap_or(false),
                    ),
                );
                // set（ + not ）： 1 
                // by and hasCredentials —— 、 / use and before 
                // 、 is ** 1 **， not can 1 。
                // as for （not configured = ， and 
                // `apply_patch` 1 1 ）， not 「」。
                fields.insert(
                    "usageQuery".to_string(),
                    usage_query_public(record.fields().get("usageQuery")),
                );
                fields.insert(
                    "lowBalance".to_string(),
                    low_balance_public(&record.provider(), record.fields().get("lowBalance")),
                );
                // 「limit」valid then （ / Token then ）： as ——
                // up no has `limiters` （ old ） when by lowBalance / provider
                // ****（ `limiter::effective_rules`）， before not 
                // ，、 and after to is 1 then 。
                fields.insert(
                    "limiters".to_string(),
                    crate::server::core::limiter::effective_rules_json_in(record.fields()),
                );
                fields.insert(
                    "limiterGroups".to_string(),
                    crate::server::core::limiter::types::json_of(&crate::server::core::limiter::effective_groups_in(record.fields())),
                );
                // 1 ****： and before `displayNameOf` 
                // 1 after （ then file `display_name_of`）。
                // as empty（`displayName` when id），request /
                // / / to etc after 1 —— 
                // ， is ， not 「、」
                // this （issue #249）。
                fields.insert(
                    "displayName".to_string(),
                    Value::String(display_name_of(&fields, &record.id().to_string())),
                );
                // ****： to 
                // （`billing::checkin::supports_checkin`） and to in 「」
                // Reason，must to 1 —— in 
                // （ is ， not will up `isGuest`）。
                //
                // for ： no has ， to all 1 false 
                // will 1 「 need not need 」 。
                // `chatglm_accounts::guest_of_record`（ ，
                // and new that to ）。
                if record.provider() == super::CHATGLM_PROVIDER_ID && !record.has_plugin_secret() {
                    fields.insert(
                        "isGuest".to_string(),
                        Value::Bool(super::chatglm_accounts::guest_of_record(record)),
                    );
                }
                Value::Object(fields)
            }
            // as for ； when ， not in this 
            other => other,
        }
    }

    /// provider need ： + provider total。
    ///
    /// `records` ** already by use ** that （`snapshot` is `visible`）：
    /// and 1 —— not will 「 3 
    /// 、 1 no has 」。
    fn provider_summary(&self, records: &[StoredAccount]) -> Value {
        let counts: Vec<(String, usize)> = records.iter().fold(Vec::new(), |mut acc, record| {
            let provider = record.provider();
            match acc.iter_mut().find(|(id, _)| *id == provider) {
                Some((_, count)) => *count += 1,
                None => acc.push((provider, 1)),
            }
            acc
        });
        let ids: Vec<String> = counts.iter().map(|(id, _)| id.clone()).collect();
        let value = crate::server::core::providers::summary_json(
            |id| {
                counts
                    .iter()
                    .find(|(known, _)| known == id)
                    .map(|(_, count)| *count)
                    .unwrap_or(0)
            },
            &ids,
        );
        Value::Array(value)
    }

    // ─── by provider ────────────────────────────────────

    /// provider enabled（， by priority、addedAt ）。
    ///
    /// ：directory「 this 1 in has no has available」
    /// （`catalog::provider_available`）—— will 「 has no 」
    /// to client，must can by provider available。
    ///
    /// ** not use **：`rotate::provider_accounts` is 
    /// （ that need ** use **，「 use → 503」 ）。
    ///
    /// 「enabled」 and 1 ：`enabled !== false`** and ** has 
    /// （`has_credentials`： when no has token， in 
    /// auth.json，「 has 」， `state.rs` ）。return is ** after ** ：
    /// use by can ， not 1 —— for is 。
    pub fn accounts_for_provider(&self, provider: &str) -> Vec<Value> {
        let records = self.read_provider(provider);
        let mut records: Vec<StoredAccount> = records
            .into_iter()
            .filter(|record| record.enabled() && record.has_credentials())
            .collect();
        records.sort_by_key(StoredAccount::order_key);
        records
            .iter()
            .map(|record| self.public_account(record))
            .collect()
    }

    /// （`custom_accounts` → ）。
    ///
    /// ── Why not is workbuddy ────────────────────────────
    /// （`to_public_account`） will uid / nickname / edition /
    /// enterprise this ** for WorkBuddy has ** —— to 
    /// will in up 1 empty， will before 1 not in edition。
    /// 「 this has 」「 this 」。
    ///
    /// ── `baseUrl` ────────────────────────────────────
    /// `baseUrl` is ****（ when to baseUrl），
    /// no has this when not —— before 「 is in 」
    /// 「（）」 and 「 」， not can empty/null 
    /// in 1 。
    ///
    /// `hasCredentials` / `chatSupported` / `checkinAt` by [`Self::public_account`]
    /// 1 （， that ）； `hasCredentials`
    /// is 「`apiKey` empty ** or ** none need 」（ `state::no_auth`），
    /// no has = not configured，「 will 」。
    pub(crate) fn to_custom_public_account(&self, record: &StoredAccount) -> Value {
        let fields = record.fields();
        let mut public = Map::new();
        public.insert("id".to_string(), Value::String(record.id().to_string()));
        public.insert("provider".to_string(), Value::String(record.provider()));
        public.insert("name".to_string(), Value::String(record.name()));
        // apiKey （ and tokenTail 1 ；empty key when is empty）
        public.insert(
            "tokenTail".to_string(),
            value_or(fields.get("tokenTail"), Value::String(String::new())),
        );
        // ── （「set」 need use ）──────────
        // `hasApiKey` to 1 ： need 「 already config，empty then 
        // not 」 and 「 no 」；`noAuth` to （ false）—— 
        // by 。**apiKey not **（ and ，
        // `custom_credential_by_id` ）。
        public.insert("hasApiKey".to_string(), Value::Bool(record.has_api_key()));
        public.insert("noAuth".to_string(), Value::Bool(record.no_auth()));
        // ：****（ = ，）
        if let Some(base_url) = fields.get("baseUrl") {
            public.insert("baseUrl".to_string(), base_url.clone());
        }
        // ── config（`balanceQuery`）────────────────────────────
        // ** in when **（ `balanceScheme`，
        // `providers::custom::balance`）。 before 「 has no has this 」
        // 「set」、 and by —— and after `supports_usage` 1 
        // （ `balance_scheme_of`， not will 1 ）。
        //
        // （`accessToken` / `password` / `token`） and 
        // ** not **， to `has*` （ and 「」 by need use ）；
        // （`userId` / `username`） —— and apiKey 
        // 「 and 」 1 （ `custom_credential_by_id`）。
        if let Some(scheme) = custom_providers::balance_scheme_of(&record.provider()) {
            public.insert(
                "balanceQuery".to_string(),
                balance_query_public(fields, &scheme),
            );
        }
        public.insert("source".to_string(), Value::String(record.source()));
        public.insert("priority".to_string(), Value::from(record.priority()));
        public.insert("enabled".to_string(), Value::Bool(record.enabled()));
        public.insert("addedAt".to_string(), Value::from(record.added_at()));
        public.insert("updatedAt".to_string(), Value::from(record.updated_at()));
        public.insert("proxy".to_string(), describe_account_proxy(Some(&record.proxy())));
        // 「available」= （ and 1 ， `accounts-shared` available ）。
        // this 1 before true， is 「 no Key no none need 」 in 
        // 「」 down （ by `available !== false` ， models-fetch-modal）
        // ——「」， that is 。
        // in by `has_credentials` to ： this in up as 「not configured」。
        public.insert(
            "available".to_string(),
            Value::Bool(record.enabled() && record.has_credentials()),
        );
        Value::Object(public)
    }
}

/// up `balanceQuery` → （ id + + 
/// `has*` ；** not **）。
///
/// as for （ no has `balanceQuery` when to empty、`has*` to false），
/// before not 「」——「 this 1 has no has 」 by ** that in not in **
/// （ has has ）， and 「 no 」 is 。
fn balance_query_public(fields: &Map<String, Value>, scheme: &str) -> Value {
    let query = fields.get("balanceQuery").and_then(Value::as_object);
    let text_of = |key: &str| {
        query
            .and_then(|map| map.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string()
    };
    let configured = |key: &str| !text_of(key).is_empty();
    json!({
        "scheme": scheme,
        "userId": text_of("userId"),
        "username": text_of("username"),
        "hasAccessToken": configured("accessToken"),
        "hasPassword": configured("password"),
        "hasToken": configured("token"),
    })
}

/// up `usageQuery` → （ as for ）。
///
/// ****、 not ： by （`apply_patch` 
/// `normalize_usage_query`）， this / ，
/// and before by 「」。 and `usage_records::query_interval_of`
/// 1 —— **not configured = 、1 minutes**（ for when 「default in 」 
/// as ， after not ）， has `{enabled:false}` is 。
fn usage_query_public(value: Option<&Value>) -> Value {
    use crate::server::core::usage_records::{
        DEFAULT_QUERY_INTERVAL_SECONDS, MAX_QUERY_INTERVAL_SECONDS, MIN_QUERY_INTERVAL_SECONDS,
    };
    let Some(fields) = value.and_then(Value::as_object) else {
        return json!({ "enabled": true, "interval": DEFAULT_QUERY_INTERVAL_SECONDS });
    };
    let enabled = fields.get("enabled").map(js_truthy).unwrap_or(true);
    let interval = fields.get("interval").and_then(Value::as_i64).unwrap_or(0);
    // / （）→ ； then （0 = ）
    let interval = if enabled
        && !(MIN_QUERY_INTERVAL_SECONDS..=MAX_QUERY_INTERVAL_SECONDS).contains(&interval)
    {
        DEFAULT_QUERY_INTERVAL_SECONDS
    } else {
        interval
    };
    json!({ "enabled": enabled, "interval": interval })
}

/// up `lowBalance` → 。
///
/// （noneconfig） by provider （ and `usage_records::balance_blocked` 1 ，
/// `default_low_balance_mode`）：Cline = `off`（ not ）， its = `skip`、
/// 1；config 1 —— that is use ， not can 。skip / disable
/// down or when 1。
fn low_balance_public(provider: &str, value: Option<&Value>) -> Value {
    use crate::server::core::usage_records::{default_low_balance, default_low_balance_mode, DEFAULT_LOW_BALANCE_THRESHOLD};
    let default_mode = default_low_balance_mode(provider);
    let Some(fields) = value.and_then(Value::as_object) else {
        return default_low_balance(provider);
    };
    let mode = fields.get("mode").and_then(Value::as_str).unwrap_or(default_mode);
    let threshold = fields
        .get("threshold")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(DEFAULT_LOW_BALANCE_THRESHOLD);
    match mode {
        "skip" => json!({ "mode": "skip", "threshold": threshold }),
        "disable" => json!({ "mode": "disable", "threshold": threshold }),
        _ => json!({ "mode": "off", "threshold": 0.0 }),
    }
}

/// → （`displayName`， as empty）。
///
/// this is before `displayNameOf`（ui-islands accounts-domain.ts） ** 1 after **，
/// and for ：
/// 1. use set（`nameCustom`）→ as ；
/// 2. → 「」 （Qoder / AutoClaw / Accio
/// ）**** —— this is ： this to is 、
/// or not can （issue #249 Qoder is 「、」 
/// ）； its ， name / / id。
/// before displayNameOf is `nickname || name || identifierOf || id`，
/// this `identifier` by can （ and before `identifier` 1 ，
/// after no has that ， has not 1 ）。
///
/// ****（ / to / telemetry）， is 
/// （`name` / `nickname` / `email` / `nameCustom` / `provider`），
/// not 。
fn display_name_of(public: &Map<String, Value>, id: &str) -> String {
    let text = |key: &str| -> String {
        public
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let name = text("name");
    // use set（update_account to name when ）→ as 
    if public.get("nameCustom").map(js_truthy).unwrap_or(false) && !name.is_empty() {
        return name;
    }
    let email = text("email");
    let provider = text("provider");
    // 「」 ： and before emailAsName 1 ——
    // Qoder / AutoClaw / Accio 
    let email_as_name = super::is_qoder_family(&provider)
        || provider == super::AUTOCLAW_INTL_PROVIDER_ID
        || super::is_accio_family(&provider);
    if email_as_name && !email.is_empty() {
        return email;
    }
    let nickname = text("nickname");
    let identifier = identifier_key_of(&provider).map(text).unwrap_or_default();
    let fallback = [nickname, name, identifier]
        .into_iter()
        .find(|value| !value.is_empty())
        .unwrap_or_else(|| id.to_string());
    fallback
}

/// up 「」（ and before can `identifier` 1 ；
/// by userId —— and before GENERIC_FEATURES 1 ）。
fn identifier_key_of(provider: &str) -> Option<&'static str> {
    match provider {
        "workbuddy" | "workbuddy-intl" | "catpaw" | "trae" | "kuku" => Some("uid"),
        "cline-free" | "cline-pass" => Some("account"),
        "officeace" => Some("id"),
        _ => Some("userId"),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::display_name_of;

    fn fields(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().cloned().unwrap_or_default()
    }

    #[test]
    fn qoder_shows_email_like_the_accounts_page() {
        // issue #249：Qoder 、request —— 1 after 
        // 「」 in after 
        let value = fields(json!({
            "provider": "qoder", "name": "昵称", "email": "a@b.com", "userId": "u1",
        }));
        assert_eq!(display_name_of(&value, "id-1"), "a@b.com");
        let intl = fields(json!({
            "provider": "qoder-intl", "email": "c@d.com", "userId": "u2",
        }));
        assert_eq!(display_name_of(&intl, "id-2"), "c@d.com");
    }

    #[test]
    fn custom_note_wins_over_email() {
        let value = fields(json!({
            "provider": "qoder", "name": "备注名", "nameCustom": true,
            "email": "a@b.com", "userId": "u1",
        }));
        assert_eq!(display_name_of(&value, "id-1"), "备注名");
    }

    #[test]
    fn other_providers_prefer_nickname() {
        let value = fields(json!({
            "provider": "workbuddy", "nickname": "昵称", "name": "种子名", "uid": "uid-9",
        }));
        assert_eq!(display_name_of(&value, "id-1"), "昵称");
    }

    #[test]
    fn falls_back_to_identifier_then_id() {
        let value = fields(json!({ "provider": "raccoon", "userId": "u-77" }));
        assert_eq!(display_name_of(&value, "id-1"), "u-77");
        let empty = fields(json!({ "provider": "raccoon" }));
        assert_eq!(display_name_of(&empty, "id-9"), "id-9");
    }
}
