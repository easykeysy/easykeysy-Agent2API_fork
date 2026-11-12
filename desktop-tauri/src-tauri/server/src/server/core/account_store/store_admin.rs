//! 、 and （ from store.rs ，file）。
//!
//! update_account_tokens new success after new token
//! mark_rate_limited × （ up 429 / code 6004）
//! clear_rate_limit 
//! import_legacy_session old auth.json （only as empty when ）
//! clear_all_rate_limits （「」）
//! migrate_startup when 1 （provider + 
//! + + Cline ）
//! renumber_globally （** 1 **，migrate_startup has ）
//!
//! this is 「、」： in file is as CRUD file in 
//! use can up ， in 1 when not is 。
//!
//! ── ：/ 1 （ ）──────────
//! file 「 1 」 （token 、、
//! 、、 to ） in before 「 → 1 → 」，
//! is **** —— token new 、 429、 to 
//! will 。 in 1 `record_by_id` + `sql::update_in_place`：
//! 1 、 1 。
//!
//! ── Why 1 ──────────────────────────────
//! `provider` （ §3.2）、（ `migrate_priorities`）
//! and Cline in when 、 in 1 up 。 if 1 、
//! 1 ， after that to is before —— no ， will 
//! and ， and 「 provider when already and 」 this 
//! not 。 by `migrate_startup` ：** 1 、 1 、
//! 1 **。
//!
//! will ****（`id` from `cline-…` `cline-free-…`）。
//! state（`migrate_cline_accounts` not ）， 1 to `save`：
//! `save_state` is ** by id for ** —— new id in not to for ， INSERT；
//! old id in not to ， DELETE。 is as 「 old + new 」，
//! not need need as 1 UPDATE。
//! in ** 1 **（`save_state` ）： and need 、
//! need not —— in down 「 id、 no 」 in will 1 
//! in provider 。

use std::collections::{HashMap, HashSet};

use serde_json::{json, Map, Value};

use crate::server::config;
use crate::server::core::account_store::priority::{
    normalize_priority_value, renumber_consecutively, PriorityAssignment,
};
use crate::server::core::account_store::sql;
use crate::server::core::account_store::state::{
    json_number, AccountState, StoredAccount, PRIORITY_SCOPE_GLOBAL,
};
use crate::server::core::account_store::store::AccountStore;
use crate::server::core::account_store::store_util::{token_tail_of, truncate_chars};
use crate::server::core::providers::{provider_index, DEFAULT_PROVIDER_ID};
use crate::server::logging;

/// ── parameter（2026-11 magpie / CLIProxyAPI ）──────────
/// up to when not ：seconds `Retry-After` will in 1 
/// request、 new in 、 1 （CLIProxyAPI #5265 ）。
const RATE_LIMIT_FLOOR_MS: f64 = 30_000.0;
/// up no to when when ， is （ use old as 10 minutes，
/// is ； by [`RATE_LIMIT_CAP_MS`] ）。
const RATE_LIMIT_FALLBACK_MS: f64 = 10.0 * 60_000.0;
/// limit： when 。
const RATE_LIMIT_CAP_MS: f64 = 30.0 * 60_000.0;
/// ： after this 「 1 」（ +1）； after 
/// new 1 ， from （）。
const RATE_LIMIT_FORGET_MS: f64 = 30.0 * 60_000.0;
/// up ** to ** when limit： up to （magpie 
/// `longestQuota` ）。used for「 up 」， has limit。
const RATE_LIMIT_GIVEN_MAX_MS: f64 = 8.0 * 24.0 * 60.0 * 60_000.0;

impl AccountStore {
    // ─── and ──────────────────────────────────

    /// / 「 need need new 」（`needsRelogin` + `needsReloginReason`）。
    ///
    /// ── this （issue #215）────────────────────────────
    /// up will refresh token 1 （12153） after ，
    /// not will 。 no has this when ，、failure，， use 
    /// not 「 this new 」。 after ：
    /// - （ not up ）；
    /// - 「 need need new 」 not is 「 new failure」；
    /// - use new success when （ `AuthService::clear_needs_relogin`）。
    ///
    /// `reason` to `None` = 。 not in return false。
    pub fn set_needs_relogin(&self, id: &str, reason: Option<&str>) -> bool {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        match reason.filter(|text| !text.is_empty()) {
            Some(reason) => {
                record.set("needsRelogin", Value::Bool(true));
                record.set("needsReloginReason", Value::String(reason.to_string()));
            }
            None => {
                record.remove("needsRelogin");
                record.remove("needsReloginReason");
            }
        }
        record.set_updated_at(logging::now_ms());
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    pub fn suspend_upstream_account(&self, id: &str, reason: &str) -> Result<(), String> {
        let guard = self.guard();
        let mut record = self.record_by_id(&guard, id).ok_or_else(|| "账号不存在".to_string())?;
        record.set_enabled(false);
        record.set("upstreamBlockedReason", Value::String(truncate_chars(reason, 300)));
        record.set_updated_at(logging::now_ms());
        self.with_conn(&guard, |conn| sql::update_in_place(conn, &record)).map_err(|error| error.to_string())
    }

    pub fn cooldown_upstream_account(&self, id: &str, until: i64, reason: &str) -> Result<(), String> {
        let guard = self.guard();
        let mut record = self.record_by_id(&guard, id).ok_or_else(|| "账号不存在".to_string())?;
        if until <= logging::now_ms() { return Ok(()); }
        record.set("upstreamCooldownUntil", Value::from(until));
        record.set("upstreamBlockedReason", Value::String(truncate_chars(reason, 300)));
        record.set_updated_at(logging::now_ms());
        self.with_conn(&guard, |conn| sql::update_in_place(conn, &record)).map_err(|error| error.to_string())
    }

    /// is as 「 need need new 」（）。
    pub fn needs_relogin(&self, id: &str) -> bool {
        self.read_record(id).and_then(|record| record.get("needsRelogin").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    /// 1 「 up 12153」 and return****（ not in return 0）。
    ///
    /// ── Why not is 1 （issue #215 ）────────
    /// （workbuddy2api-panel） in this ： old 「 1 12153
    /// Disable」， after **13 use refresh success** —— is 
    /// 。12153 will 、 up 、refresh when ， 1 failure
    /// will 。
    ///
    /// this 、 not ； in use （`AuthService`），
    /// `needsRelogin`。
    pub fn note_refresh_token_invalid(&self, id: &str) -> i64 {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return 0;
        };
        let streak = record
            .get("refreshTokenInvalidStreak")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .saturating_add(1);
        record.set("refreshTokenInvalidStreak", Value::from(streak));
        record.set_updated_at(logging::now_ms());
        let _ = self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record));
        streak
    }

    /// 12153 （ new success = ）。
    ///
    /// and `note_refresh_token_invalid` for ： need has 1 new success，
    /// before that 12153 is ，，「 1 」 to 。
    pub fn clear_refresh_token_invalid(&self, id: &str) {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return;
        };
        if record.get("refreshTokenInvalidStreak").is_none() {
            return;
        }
        record.remove("refreshTokenInvalidStreak");
        record.set_updated_at(logging::now_ms());
        let _ = self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record));
    }

    /// new success after new token（ not in return false）
    pub fn update_account_tokens(
        &self,
        id: &str,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        expires_at: Option<f64>,
        refresh_expires_at: Option<f64>,
    ) -> bool {
        let _guard = self.guard();
        // that 1 ： this is new （token 1 to ），
        // before need JSON 1 as its in 1 
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        if let Some(token) = access_token.filter(|value| !value.is_empty()) {
            record.set("accessToken", Value::String(token.to_string()));
            record.set("tokenTail", Value::String(token_tail_of(token)));
        }
        if let Some(token) = refresh_token.filter(|value| !value.is_empty()) {
            record.set("refreshToken", Value::String(token.to_string()));
        }
        if let Some(value) = expires_at.filter(|value| *value > 0.0) {
            record.set("expiresAt", json_number(value));
        }
        if let Some(value) = refresh_expires_at.filter(|value| *value > 0.0) {
            record.set("refreshExpiresAt", json_number(value));
        }
        record.set_updated_at(logging::now_ms());
        // update： not in （ and old 「 in down up 」 1 ）
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    /// for （ up 429 / code 6004）。
    ///
    /// ── when （，2026-11 magpie / CLIProxyAPI）──────
    /// 1. ** not **： new not 1 no old —— 1 1 minutes 429
    /// before when ， 1 new in 、、
    /// ， 1 1 （magpie #3289 ， is #451 ）。
    /// 1 4 。
    /// 2. ** down **： up to when not [`RATE_LIMIT_FLOOR_MS`] ——
    /// seconds `Retry-After` will in 1 request（CLIProxyAPI #5265）。
    /// 3. ****： up no to when when by （[`RATE_LIMIT_FALLBACK_MS`]
    /// ×2ⁿ， [`RATE_LIMIT_CAP_MS`]）。 = up 1 after 
    /// [`RATE_LIMIT_FORGET_MS`] ； after new 1 ， from 。
    /// 4. ****： up ** to ** when ** **（ / ）
    /// —— is up ， up to as （）。
    ///
    /// resetAt as when ； when by 3 （ is old 10 minutes）。
    ///
    /// ── ：provider × × （Agent2API W2b-T3 ）──
    /// is `accounts[i].rateLimits[model] = {status, code, resetAt, message, at, strikes, guessed}`，
    /// ** in **， 1 provider
    /// （`record.provider()` is ）。「provider 」 already ，
    /// ** not need need **：`accounts[i]` this 1 down when provider and ，
    /// up `rateLimits` ， `(provider, account_id, model)`。
    /// （`strikes` / `guessed` is add ：、 is 
    /// 「 up no to when 」 。 old no has when by 0 / false ，「、
    /// 」， as not + 10 minutes。）
    ///
    /// when （）， is 「 this for this 
    /// 」—— not will to up 。
    pub fn mark_rate_limited(
        &self,
        id: &str,
        model: &str,
        status: i64,
        code: Option<i64>,
        reset_at: Option<f64>,
        message: &str,
    ) -> Option<Value> {
        let _guard = self.guard();
        // that 1 。 this 1 in before is **** 1 ： 1 429
        // 20 1 ， is to its in 1 1 rateLimits 。
        let mut record = self.record_by_id(&_guard, id)?;
        let now = logging::now_ms();
        let now_f = now as f64;
        let prev = record
            .get("rateLimits")
            .and_then(|limits| limits.get(model))
            .cloned();
        // up ** to ** when （ down ； when no has ）
        let given = reset_at.filter(|value| value.is_finite() && *value > now_f);
        // up 1 and its （ old this when by 0 / false ）
        let prev_reset_at = prev
            .as_ref()
            .and_then(|entry| entry.get("resetAt"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let prev_guess = prev
            .as_ref()
            .and_then(|entry| entry.get("guessed"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        // ： up 1 （ no or not ） after FORGET → +1
        let strikes = if prev.is_some() && now_f - prev_reset_at <= RATE_LIMIT_FORGET_MS {
            prev.as_ref()
                .and_then(|entry| entry.get("strikes"))
                .and_then(Value::as_u64)
                .unwrap_or(0)
                .saturating_add(1)
        } else {
            1
        };
        // when （ and old and before ）
        let fresh = match given {
            // up to ： down and limit， its up 
            Some(given_at) => {
                given_at.clamp(now_f + RATE_LIMIT_FLOOR_MS, now_f + RATE_LIMIT_GIVEN_MAX_MS)
            }
            // no to ：（2ⁿ use not ，30 already limit）
            None => {
                let steps = (strikes.saturating_sub(1)).min(30) as i32;
                let backoff = RATE_LIMIT_FALLBACK_MS * 2f64.powi(steps);
                now_f + backoff.clamp(RATE_LIMIT_FLOOR_MS, RATE_LIMIT_CAP_MS)
            }
        };
        // not （ 1 ）， this is up 、 old that is 
        // （ 4 ： up ，）
        let until = if given.is_some() && prev_guess {
            fresh
        } else {
            fresh.max(prev_reset_at)
        };
        let entry = json!({
            "status": if status == 0 { 429 } else { status },
            "code": code.map(Value::from).unwrap_or(Value::Null),
            "resetAt": json_number(until),
            "message": truncate_chars(message, 300),
            "at": now,
            "strikes": strikes,
            "guessed": given.is_none(),
        });
        let mut limits = match record.get("rateLimits") {
            Some(Value::Object(map)) => map.clone(),
            _ => Map::new(),
        };
        limits.insert(model.to_string(), entry.clone());
        record.set("rateLimits", Value::Object(limits));
        record.set_updated_at(now);
        if self
            .with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_err()
        {
            return None;
        }
        Some(entry)
    }

    /// for （ this requestsuccess when use ）。
    ///
    /// `mark_rate_limited`： in ，provider by 1 。
    pub fn clear_rate_limit(&self, id: &str, model: &str) -> bool {
        if !self.route_account(id).is_some_and(|account| {
            account.get("rateLimits").and_then(|limits| limits.get(model)).is_some()
        }) {
            return false;
        }
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        let Some(Value::Object(limits)) = record.get("rateLimits") else {
            return false;
        };
        if !limits.contains_key(model) {
            return false;
        }
        let mut limits = limits.clone();
        limits.remove(model);
        if limits.is_empty() {
            record.remove("rateLimits");
        } else {
            record.set("rateLimits", Value::Object(limits));
        }
        record.set_updated_at(logging::now_ms());
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    /// **** （ 「」）。
    /// return ； not in or no has return 0。
    pub fn clear_all_rate_limits(&self, id: &str) -> usize {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return 0;
        };
        let count = match record.get("rateLimits") {
            Some(Value::Object(limits)) => limits.len(),
            _ => 0,
        } + usize::from(record.get("upstreamCooldownUntil").is_some());
        if count == 0 {
            return 0;
        }
        record.remove("rateLimits");
        record.remove("upstreamCooldownUntil");
        if record.enabled() { record.remove("upstreamBlockedReason"); }
        record.set_updated_at(logging::now_ms());
        if self
            .with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_err()
        {
            return 0;
        }
        count
    }

    /// new for use ：`expiresAt` and `updatedAt`（ms）。
    ///
    /// `refresh_access_token` return Ok 「 this use no 」—— can can 
    /// no to returnsuccess（AutoClaw 28 when empty，issue #407）。
    /// use in new before after 1 this ， no by empty。
    ///
    /// use not is `expiresAt`：expiration time in not 1 in 
    /// （ in JWT `exp`、 not ）， 1 will 
    /// `updatedAt`。
    pub fn credential_stamp(&self, id: &str) -> Option<(f64, f64)> {
        let _guard = self.guard();
        let record = self.record_by_id(&_guard, id)?;
        let expires_at = record
            .get("expiresAt")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        Some((expires_at, record.updated_at() as f64))
    }

    /// 1 ：** by request（） down **（ etc ， can use ）。
    ///
    /// ── is （2026-09 ）─────────────────────────────
    /// （`modelRules.mappings`） will in before request up ， up by 
    /// ****。 old in and use request， is ：
    ///
    /// - request after ， in up （ `gpt-5.6-luna`）；
    /// - after use （`deepseek-v4.1-flash`）request when is —— not to that 
    /// ， is in this already ， 1 429；
    /// - that ，「 this for 」，
    /// its has 1 （ `resetAt` ）。
    ///
    /// （`routing::CooldownKeys` / `payload::SendBody::wire_model`）
    /// after ， this **** will not in ： not ，
    /// will in 「」（ before after to ），
    /// use to is 1 not 。 when must 1 。
    ///
    /// ── Why is 「delete」 not is 「」──────────────────────
    /// 「」， that will ** when **：
    /// up `resetAt` 「 use request when 」 up return that 429， and 
    /// up that is 1 （ `resetAt` 1 ）。 will 
    /// in already in when 1 （ 1 or 1 ）， not delete。
    /// is 「 use this 」 —— after is ** 1 **
    /// up 、 1 429 new 1 ； 1 。
    ///
    /// ── Whymust** by **（ this ）──────
    /// is 「 in alias → 」， that will ** 
    /// **：`alias` and up id （ is use ，
    /// `model_rules` module header）， `deepseek-v4.1-flash` is raccoon / cline
    /// that ，** when ** is WorkBuddy 。 for WorkBuddy 
    /// to this ， will 1 —— that after will 
    /// in 、 429， is need that 1 。
    ///
    /// is ** that **： request，
    /// `wire_target_for_provider(, this )` will ——
    ///
    /// - is ****（ this ， or in ）→ is ，
    /// this is ，；
    /// - is ** **（ ）→ not is up ，
    /// this ，。
    ///
    /// and / 1 （ `wire_target_for_provider`），
    /// not in 「」 。
    ///
    /// not `resetAt` expired —— that not 、 not and ，
    /// （ and `rate_limit_reset_at` 「」 1 ，
    /// not as 1 ）。
    ///
    /// return `(, )` need （）， no has can returnempty。
    ///
    /// ── Why（、） not is ─────────
    /// need `wire_target_for_provider`， will 。 that 
    /// current is **** （`global_catalog` / `models::list`），
    /// down not will —— that is 1 ****： will 
    /// `list_models` 1 ， this will 「 etc 」 （`std`
    /// Mutex not can ， and release is panic=abort）。 1 in ，
    /// 「 by id 」 this 1 。
    pub fn migrate_rate_limit_keys(&self) -> Vec<String> {
        // ① （）： has need 
        let snapshot = self.list_accounts();
        let accounts = crate::server::core::routing::accounts_of(&snapshot);
        if accounts.is_empty() {
            return Vec::new();
        }
        let now = logging::now_ms();
        // ② ： this in this is not is （）
        let mut plan: Vec<(String, String, Vec<String>)> = Vec::new();
        for account in &accounts {
            let Some(id) = account.get("id").and_then(Value::as_str).filter(|id| !id.is_empty())
            else {
                continue;
            };
            let Some(Value::Object(limits)) = account.get("rateLimits") else {
                continue;
            };
            let provider = crate::server::core::routing::provider_of(account).to_string();
            let stale: Vec<String> = limits
                .iter()
                .filter(|(key, entry)| {
                    let future = entry
                        .get("resetAt")
                        .and_then(Value::as_f64)
                        .map(|value| value > now as f64)
                        .unwrap_or(false);
                    if !future || key.is_empty() {
                        return false;
                    }
                    // request 1 ： not is → not is 
                    let wire = crate::server::core::providers::catalog::wire_target_for_provider(
                        key,
                        &provider,
                        None,
                    )
                    .model;
                    !wire.eq_ignore_ascii_case(key)
                })
                .map(|(key, _)| key.clone())
                .collect();
            if !stale.is_empty() {
                let who = account
                    .get("name")
                    .and_then(Value::as_str)
                    .filter(|name| !name.is_empty())
                    .unwrap_or(id)
                    .to_string();
                plan.push((id.to_string(), who, stale));
            }
        }
        if plan.is_empty() {
            return Vec::new();
        }
        // ③ ： by ， not 
        let _guard = self.guard();
        let mut state = self.load(&_guard);
        let mut cleaned: Vec<String> = Vec::new();
        for record in state.accounts.iter_mut() {
            let Some((_, who, stale)) = plan.iter().find(|(id, _, _)| id == record.id()) else {
                continue;
            };
            // 1 ：`record.get` is use ， after `record.remove` /
            // `record.set` need can use ， not can when （ and `clear_rate_limit`
            // 1 ： that is clone limits ）
            let Some(Value::Object(current)) = record.get("rateLimits") else {
                continue;
            };
            let mut limits = current.clone();
            let mut hit = false;
            for key in stale {
                // that ； can can already path（ and has 
                // when ），`remove` return use 
                if limits.remove(key).is_some() {
                    hit = true;
                    cleaned.push(format!("{who}: {key}"));
                }
            }
            if !hit {
                continue;
            }
            if limits.is_empty() {
                record.remove("rateLimits");
            } else {
                record.set("rateLimits", Value::Object(limits));
            }
            record.set_updated_at(now);
        }
        if cleaned.is_empty() {
            return cleaned;
        }
        if let Err(error) = self.save(&state, &_guard) {
            logging::log(
                "[Accounts]",
                &format!("❌ 限流键迁移落库失败（下次启动会重试）: {error}"),
            );
            return Vec::new();
        }
        cleaned
    }

    /// Trae ID ： up 、limit 
    /// by directory to old ID。
    ///
    /// provider as `trae` ；** old **，
    /// not by after —— up `-solo` when not can 。
    /// 1 、 1 （ and `migrate_startup` 1 to ）； no has when not 。
    pub fn migrate_trae_model_ids(&self, mapping: &HashMap<String, String>) -> usize {
        if mapping.is_empty() {
            return 0;
        }
        let _guard = self.guard();
        let mut state = self.load(&_guard);
        let now = logging::now_ms();
        let mut changed = 0usize;
        for record in state.accounts.iter_mut() {
            if !record.provider().eq_ignore_ascii_case("trae") {
                continue;
            }
            let mut touched = false;
            if let Some(Value::Object(limits)) = record.get("rateLimits").cloned() {
                let mut next = Map::new();
                for (key, value) in limits {
                    let base = mapping.get(&key.trim().to_ascii_lowercase());
                    if base.is_some() {
                        touched = true;
                    }
                    next.entry(base.cloned().unwrap_or(key)).or_insert(value);
                }
                if touched {
                    if next.is_empty() {
                        record.remove("rateLimits");
                    } else {
                        record.set("rateLimits", Value::Object(next));
                    }
                }
            }
            for field in ["limiterGroups", "limiters"] {
                let Some(mut value) = record.get(field).cloned() else {
                    continue;
                };
                if rewrite_models_arrays(&mut value, mapping) {
                    record.set(field, value);
                    touched = true;
                }
            }
            if touched {
                record.set_updated_at(now);
                changed += 1;
            }
        }
        if changed > 0 {
            if let Err(error) = self.save(&state, &_guard) {
                logging::log(
                    "[Accounts]",
                    &format!("❌ Trae 旧模型 ID 迁移落库失败（下次启动会重试）: {error}"),
                );
                return 0;
            }
        }
        changed
    }

    /// ** already to **（ that by ）。
    ///
    /// `at` is this to success ms when 。「 no 」 by **** by 
    /// （`checkinAt` in ）， this when —— not need need 
    /// in when ， 0 after 1 ， not when 。
    ///
    /// and ： is 「 、、failure not 」。 use is to 
    /// ，failure** not ** to （ up that already to ），return
    /// bool not is Result —— use to false when 1 ， not 
    /// 1 success to failure。
    pub fn mark_checkin(&self, id: &str, at: i64) -> bool {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        record.set_checkin_at(at);
        // Note** not ** updatedAt： that is 「 」 when ， will in 
        // 「update …」 up ； to not is use config that 「update」， will this 1 
        // in to after 1 。
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    /// in ****（WorkBuddy ； ）。
    ///
    /// down is as to in / not use 1 up 
    /// （`GET /activity/growth/streak` is ， is 1 request）。
    /// and [`Self::mark_checkin`] 1 ：、、failure not 
    /// （return bool， use 1 ）。
    ///
    /// ** not ** `updatedAt`： is up ， not is use config。
    /// no when not —— to will ， 1 to not this 。
    pub fn mark_streak_days(&self, id: &str, days: i64) -> bool {
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        let existing = record
            .fields()
            .get("streakDays")
            .and_then(Value::as_i64);
        if existing == Some(days) {
            return true;
        }
        record
            .fields_mut()
            .insert("streakDays".to_string(), Value::from(days));
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    /// down 「 this new （ 1 ） already 」 
    /// （ up `onboardingSettled`； and 
    /// [`StoredAccount::set_onboarding_settled`] and 
    /// `crate::server::core::providers::onboarding_memory`）。
    ///
    /// and `mark_checkin` / `mark_onboarding_grant_batch` 1 ：、
    /// 、failure not （return bool； up that already ， use 
    /// 1 ）。** not ** updatedAt（ by up ）。
    ///
    /// to 「 for or Null」 as ： for = （ is 
    /// `{at, tasks, earned, total}`， this not —— 1 use is that 
    /// onboarding ， in that ）；**Null = **（ up 
    /// new 1 new when ， need can ， Null 「 no has 」）。
    /// its （ / / ）， not 
    /// 。
    ///
    /// and when （「 no can 」 Null）**return true， not **：
    /// no will `unclaimed > 0` to 「」 that 1 ，
    /// no 1 is —— this need has that 1 。
    pub fn mark_onboarding_settled(&self, id: &str, snapshot: Value) -> bool {
        if !(snapshot.is_object() || snapshot.is_null()) {
            return false;
        }
        let _guard = self.guard();
        let Some(mut record) = self.record_by_id(&_guard, id) else {
            return false;
        };
        let existing = record.fields().get("onboardingSettled");
        if existing == Some(&snapshot) || (snapshot.is_null() && existing.is_none()) {
            return true;
        }
        record.set_onboarding_settled(snapshot);
        self.with_conn(&_guard, |conn| sql::update_in_place(conn, &record))
            .is_ok()
    }

    // ─── ────────────────────────────────────────────────

    /// when 1 （** 1 **，bootstrap ）。
    ///
    /// in 1 up 、 1 ：
    /// ① provider （/empty 1 workbuddy）；
    /// ② use from 「 by provider 」 to 「 1 」——
    /// no has `priorityScope: "global"` when 1 ： by old 
    /// （provider by → → when ），，
    /// is before after use 1 ， use not will to ；
    /// ③ （ and ）；
    /// ④ Cline （`provider: "cline"` + `pool` → 1 ）；
    /// ⑤⑥ WorkBuddy / Qoder （ provider）；
    /// ⑦ OfficeAce （ from refreshToken ）；
    /// ⑧ from `auth` to 。
    ///
    /// up ④ must in ①②③ ** after **： before by provider when to is old id
    /// （ 1 `cline` ）， after this 1 ； if ，
    /// will in new up 1 ， and before not for 。 after then and 
    /// 「 old 、」 etc —— ， and 。
    /// return `{providerAdded, priorityChanged, assignments}` 。
    ///
    /// ── Whymust in 1 （ need ）──────────────
    /// ②③④ will `priority`（ `id`，）。 this is 
    /// 「 1 」， in in 1 will down ** or ** in 
    /// —— 「 1 」 this not ， down 1 will up 
    /// 409， use to is 「 no use 」。
    /// `save`（`sql::save_state` 1 ）： need ，
    /// need 1 not ， in after down from 1 （ etc ）。
    ///
    /// ── ────────────────────────────────────────────────
    /// is （①②③④ need ），**** is by ：
    /// `save_state` for `data` UPDATE，「 already 」
    /// is （`provider_added == 0 && assignments.is_empty()
    /// && !scope_migrated && cline_renamed == 0` not ）。
    pub fn migrate_startup(&self) -> Value {
        let _guard = self.guard();
        let mut state = self.load(&_guard);

        // ① provider （ §3.2）：/empty 1 workbuddy
        let mut provider_added = 0usize;
        for record in state.accounts.iter_mut() {
            if record.provider_explicit().is_none() {
                record.set_provider(DEFAULT_PROVIDER_ID);
                provider_added += 1;
            }
        }
        if provider_added > 0 {
            logging::log(
                "[Accounts]",
                &format!("🏷️  已为 {provider_added} 个历史账号补全 provider 字段（{DEFAULT_PROVIDER_ID}）"),
            );
        }

        // ② / ③ ： when by old ； after in has and when 
        let scope_migrated = state.priority_scope.as_deref() != Some(PRIORITY_SCOPE_GLOBAL);
        let assignments = Self::renumber_globally(&mut state, scope_migrated);
        if scope_migrated {
            state.priority_scope = Some(PRIORITY_SCOPE_GLOBAL.to_string());
        }

        // ④ Cline （ `migrate_cline_accounts`）。 in after ：
        // is id， ②③ need by ** old ** id 
        let cline_renamed = Self::migrate_cline_accounts(&mut state);

        // ⑤ WorkBuddy （ `migrate_workbuddy_intl_accounts`）。
        // `provider`、** not id**， in ②③ after no has ；
        // in Cline after is 「」 in 1 。
        let workbuddy_moved = Self::migrate_workbuddy_intl_accounts(&mut state);

        // ⑥ Qoder ： old ， and has 。
        // provider， not as request use id。
        let qoder_moved = super::qoder_accounts::migrate_accounts(&mut state.accounts);

        // ⑦ OfficeAce （ name is id， in 
        // refreshToken claims ） already to ： in when ，
        // not this claims。 —— is 
        // use ， not this as 1 。

        // ⑧ from `auth` to 
        // （ `chatglm_accounts::migrate_nested_tokens`）： 
        // `auth`， `has_credentials()` —— not that will 1 
        // （ not to this 1 、「 no has available」）。
        let chatglm_healed =
            super::chatglm_accounts::migrate_nested_tokens(&mut state.accounts);
        if chatglm_healed > 0 {
            logging::log(
                "[Accounts]",
                &format!(
                    "🔧 已修复 {chatglm_healed} 个智谱清言账号的令牌存储位置（嵌套 auth → 顶层，\
                     修复后即可正常选路与查询余额）"
                ),
            );
        }

        let codearts_limiters = super::codearts_accounts::migrate_limiters(&mut state.accounts);

        if provider_added == 0
            && assignments.is_empty()
            && !scope_migrated
            && cline_renamed == 0
            && workbuddy_moved == 0
            && qoder_moved == 0
            && chatglm_healed == 0
            && codearts_limiters == 0
        {
            return json!({
                "providerAdded": 0,
                "priorityChanged": false,
                "assignments": [],
            });
        }
        // 1 （ is + 1 ， up ）。
        // will ， `save_state` 「delete + add」 will 
        // use to （ old id 、 new id ）—— this is by id for not is 
        // UPDATE Reason。 use is will to ； 
        // no has （ and by ）， can 。
        if let Err(error) = self.save(&state, &_guard) {
            logging::log("[Accounts]", &format!("❌ 启动迁移落库失败: {error}"));
            return json!({
                "providerAdded": 0,
                "priorityChanged": false,
                "assignments": [],
            });
        }
        if codearts_limiters > 0 {
            logging::log("[Accounts]", &format!("已移除 {codearts_limiters} 个 CodeArts 账号的过期活动 Token 限制；无其他规则时改用总积分保护"));
        }
        if cline_renamed > 0 {
            logging::log(
                "[Accounts]",
                &format!(
                    "🔀 Cline 已拆为 Cline Free / Cline Pass 两家，{cline_renamed} 个账号已按额度池归位"
                ),
            );
        }
        if workbuddy_moved > 0 {
            logging::log(
                "[Accounts]",
                &format!(
                    "🔀 WorkBuddy 已拆为国内版 / 国际版两家，{workbuddy_moved} 个国际版账号已归位\
                     （凭证、优先级、限流记录与账号 id 均未改动）"
                ),
            );
        }
        if qoder_moved > 0 {
            logging::log(
                "[Accounts]",
                &format!(
                    "🔀 Qoder 已按地区归位 {qoder_moved} 个账号\
                     （凭证、优先级、限流记录与账号 id 均未改动）"
                ),
            );
        }
        if !assignments.is_empty() {
            logging::log(
                "[Accounts]",
                &format!(
                    "🔢 优先级已{}（{} 个账号重新编号，转发顺序保持不变）",
                    if scope_migrated { "合并为全局一条队列" } else { "去重" },
                    assignments.len()
                ),
            );
        } else if scope_migrated {
            logging::log("[Accounts]", "🔢 优先级已标记为全局一条队列（号码无需调整）");
        }
        json!({
            "providerAdded": provider_added,
            "priorityChanged": !assignments.is_empty(),
            "assignments": assignments
                .iter()
                .map(|item| json!({
                    "id": item.id,
                    "name": item.name,
                    "from": item.from,
                    "to": item.to,
                }))
                .collect::<Vec<_>>(),
        })
    }

    /// ****（ etc ， can use ）。
    ///
    /// ── is ────────────────────────────────────────────
    /// `accounts.data` is `enc1:` （ `sql.rs` module header），
    /// ** before already ** is JSON。`sql::decode_data` ， that 
    /// can —— that is ， not is ： need has ，file
    /// 。 when 1 「」， 。
    ///
    /// ── Why in `migrate_startup` after （ `account_bootstrap::run`）────
    /// that 1 will by （ provider、、…）， 
    /// already is ； down no to 、 is by 。 
    /// can to 1 （`decode_data` ）， after is 「
    /// 、」 this 。
    ///
    /// ── etc and failure by SQL ───────────────────────────
    /// is （`sql::encrypt_legacy_rows` `data` is not is `enc1:`），
    /// no has kv ； in 1 ， need 、 need 1 not 。**failure not 
    /// **：file not failure in this 1 、return， down 
    /// —— as `decode_data` ，available， not use in 。
    pub fn encrypt_legacy_accounts(&self) {
        let _guard = self.guard();
        // use `with_conn_mut` not is `with_conn`： need in 1 
        // （ `sql::encrypt_legacy_rows` ）， need need can use 。
        match self.with_conn_mut(&_guard, |conn| sql::encrypt_legacy_rows(conn)) {
            // 0 = no has （ new / already ）， not 
            Ok(0) => {}
            Ok(count) => {
                logging::log(
                    "[Accounts]",
                    &format!("🔒 已加密 {count} 个历史账号的凭证"),
                );
            }
            Err(error) => {
                // this must in `with_conn_mut` ****： need 
                // （logs ）， `with_conn_mut` that 1 ——
                // in is 「 etc 」 （`std` Mutex
                // not can ，release is panic=abort）。
                logging::log(
                    "[Accounts]",
                    &format!(
                        "⚠️  历史账号凭证加密未完成（下次启动会自动重试，本次账号照常可用）: {error}"
                    ),
                );
            }
        }
    }

    /// 「」 （ etc ， can use ）。
    ///
    /// ── is （issue #380）──────────────────────────────
    /// / is `pluginSecret` + `pluginPublic`， no has 
    /// `uid` / `edition` / `endpoint` etc 。 、 to 、
    /// path —— that 「 for 、 to not to 」。
    ///
    /// （`plugin_identity::mirror_identity`） already in new when ，
    /// ** before already in ** that 。（`StoredAccount::uid`
    /// etc ）， this 1 no ， can is —— is to up ，
    /// JSON and its by 1 。
    ///
    /// ── etc and failure ──────────────────────────────────────────
    /// is ： has 「、 for has 」 when ， no has kv 。
    /// 1 1 （`migrate_startup` ）； no has need when not 。
    pub fn backfill_plugin_identity(&self) {
        let guard = self.guard();
        let mut state = self.load_locked(&guard);
        let mut changed = 0usize;
        for record in state.accounts.iter_mut() {
            // （ has pluginSecret or pluginPublic）：
            // is ， not this for 。
            let has_plugin_side = record.get("pluginSecret").is_some_and(|value| !value.is_null())
                || record.get("pluginPublic").is_some_and(|value| !value.is_null());
            if !has_plugin_side {
                continue;
            }
            let before = record.to_value();
            let provider = record.provider();
            super::plugin_identity::mirror_identity(record, &provider);
            if record.to_value() != before {
                changed += 1;
            }
        }
        if changed == 0 {
            return;
        }
        match self.save_locked(&state, &guard) {
            Ok(()) => {
                drop(guard);
                logging::log(
                    "[Accounts]",
                    &format!("🔧 已为 {changed} 个插件账号补齐顶层身份字段（uid / edition 等）"),
                );
            }
            Err(error) => {
                drop(guard);
                logging::log(
                    "[Accounts]",
                    &format!(
                        "⚠️  插件账号身份字段回填未完成（下次启动会自动重试，本次账号照常可用）: {}",
                        error.message
                    ),
                );
            }
        }
    }

    /// Cline ：`provider: "cline"` + `pool` → 1 ，
    /// state（ not ）。return 。
    ///
    /// ── Why this 1 not can ──────────────────────────────────────
    /// `"cline"` this provider id ** already not in **（ after is `cline-free` /
    /// `cline-pass`）。 not after not is 「 1 set」 is ****：
    /// by provider path（`accounts_for_provider` 、
    /// 、 by id ） not to —— in file， use will to 
    /// 「 in ， no has available」。
    ///
    /// ── ────────────────────────────────────────────────
    /// `pool` ， and ** use before **
    /// （`cline::models::Pool::parse`：`free` → ， its // → ）。
    /// use 1 not is 1 match is ： use can to 
    /// ， before after must 1 ，otherwise 1 。
    ///
    /// ── id Why need 1 ──────────────────────────────────────
    /// old id `cline-usr-…`，** not **。 if not ， 1 Cline after 
    /// 1 when ， will 1 id 「update 1 」，
    /// not 。 `<provider>-< after >`（`cline-free-usr-…`）
    /// after can and —— and `cline_accounts::record_id` 1 。
    ///
    /// is ** old provider ** not is id before ：id is use can ，
    /// has can can （ will ）， has provider 
    /// is this 1 。
    ///
    /// `pool` in after ****： already is provider 1 ，
    /// 1 can can and provider not 1 （ already ）。
    ///
    /// etc ： provider not in 1 ， 1 none。
    fn migrate_cline_accounts(state: &mut AccountState) -> usize {
        /// before Cline provider id（ in this ）
        const LEGACY_CLINE_ID: &str = "cline";
        let mut renamed = 0usize;
        for record in state.accounts.iter_mut() {
            if record.provider() != LEGACY_CLINE_ID {
                continue;
            }
            let pool = crate::server::core::providers::cline::models::Pool::parse(
                record.get("pool").and_then(Value::as_str).unwrap_or(""),
            );
            let provider = pool.provider_id();
            let id = record.id().to_string();
            // already before （ not can can ， for ） provider，
            // not before —— otherwise will `cline-free-free-…` this id
            let new_id = if id.starts_with(&format!("{provider}-")) {
                id
            } else if let Some(rest) = id.strip_prefix(&format!("{LEGACY_CLINE_ID}-")) {
                format!("{provider}-{rest}")
            } else {
                format!("{provider}-{id}")
            };
            record.set_provider(provider);
            record.set("id", Value::String(new_id));
            record.remove("pool");
            renamed += 1;
        }
        renamed
    }

    /// WorkBuddy ： `provider == "workbuddy"` and `edition == "intl"`
    /// to `workbuddy-intl`（2026-10， `providers::workbuddy::region`）。
    ///
    /// ── Why not id（ and `migrate_cline_accounts` ）──────────
    /// Cline that must id： use `cline-...` id empty， not not 。
    /// not need need —— uid is up uuid， not will ； id is 
    /// `requests.account_id` and `request_daily` use ， will use in request
    /// and for not up that （ `Region` module header that ）。
    ///
    /// ── Why is `edition` not is ────────────────
    /// before 「 」 this ** has ** `edition` 1 （、
    /// UA、 by ）， is this 。
    /// `edition` （ old / ） 1 by —— and `resolve_edition`
    /// （`StoredAccount::edition()` return None， this not ）。
    ///
    /// ── etc ────────────────────────────────────────────────────
    /// is （`provider == workbuddy && edition == intl`）： no has 
    /// this ， 1 return 0、 not 。 and `migrate_startup` its 
    /// 1 then （ `db::migrate` module header「 has no has 」 that ）。
    fn migrate_workbuddy_intl_accounts(state: &mut AccountState) -> usize {
        /// before 1 WorkBuddy provider id（ in this ）
        const LEGACY_WORKBUDDY_ID: &str = "workbuddy";
        let mut moved = 0usize;
        for record in state.accounts.iter_mut() {
            if record.provider() != LEGACY_WORKBUDDY_ID || record.has_plugin_secret() {
                continue;
            }
            let region = crate::server::core::providers::workbuddy::Region::from_edition_id(
                record.edition().as_deref(),
            );
            let provider = region.provider_id();
            if provider == LEGACY_WORKBUDDY_ID {
                continue;
            }
            record.set_provider(provider);
            moved += 1;
        }
        moved
    }

    /// new ，** state**（ not ）。
    ///
    /// `legacy_order = true`（ from 「 by 」 to ）： is old 
    /// —— ( old by , , when )。 old by config.json 
    /// `providerRoute` ， no has then by （ and old default 10/20/30/40 ）。
    /// this 1 **none has no has and **： not （ has P100），
    /// not ， etc old 。
    ///
    /// `legacy_order = false`（ after ）： in in and when by 
    /// (, when ) —— and up when 1 。
    ///
    /// after by new 。return（ not ）。
    fn renumber_globally(state: &mut AccountState, legacy_order: bool) -> Vec<PriorityAssignment> {
        if state.accounts.is_empty() {
            return Vec::new();
        }
        if !legacy_order {
            let mut unique: Vec<i64> = state
                .accounts
                .iter()
                .map(|record| normalize_priority_value(record.priority()))
                .collect();
            let total = unique.len();
            unique.sort_unstable();
            unique.dedup();
            if unique.len() == total {
                return Vec::new();
            }
        }

        let legacy_route = config::legacy_provider_route();
        let route_rank = |record: &StoredAccount| -> u32 {
            let provider = record.provider();
            legacy_route
                .iter()
                .find(|(id, _)| *id == provider)
                .map(|(_, rank)| *rank)
                .unwrap_or_else(|| (provider_index(&provider).unwrap_or(usize::MAX / 2) as u32 + 1) * 10)
        };
        let mut sorted = state.accounts.clone();
        if legacy_order {
            sorted.sort_by_key(|record| (route_rank(record), record.order_key()));
        } else {
            sorted.sort_by_key(StoredAccount::order_key);
        }
        let mut numbering: Vec<(String, String, i64)> = sorted
            .iter()
            .map(|record| (record.id().to_string(), record.name(), record.priority()))
            .collect();
        let assignments = renumber_consecutively(&mut numbering);
        // 「」 ： no has priority after no has 
        // （ is default 100）， not will empty 1 
        for assignment in &assignments {
            if let Some(record) = sorted.iter_mut().find(|record| record.id() == assignment.id) {
                record.set_priority(assignment.to);
            }
        }
        state.accounts = sorted;
        assignments
    }

    /// old auth.json ：onlynone when 1 。
    ///
    /// return after （ when return None， for Node return null）。
    pub fn import_legacy_session(&self, legacy: &Value) -> Option<Value> {
        let access_token = legacy
            .get("auth")
            .and_then(|auth| auth.get("accessToken"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if access_token.is_empty() {
            return None;
        }
        {
            let _guard = self.guard();
            // 「 is as empty」—— COUNT， not 。
            // failure when by 「empty」（）： is 1 ，
            // unavailable when not this （`add_account` will return 500），「」 and 
            // old 「 not to fileempty、 is 」 in when ——
            // that when unavailable， by ❌ 。
            match self.with_conn(&_guard, |conn| sql::count_all(conn)) {
                Ok(count) if count == 0 => {}
                Ok(_) => return None,
                Err(error) => {
                    logging::log(
                        "[Accounts]",
                        &format!("⚠️  读取账号数失败，跳过旧登录态迁移: {error}"),
                    );
                    return None;
                }
            }
        }
        let uid = legacy
            .get("account")
            .and_then(|account| account.get("uid"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if uid.is_empty() {
            logging::log("[Accounts]", "旧登录态缺少 uid，跳过迁移");
            return None;
        }
        let nickname = legacy
            .get("account")
            .and_then(|account| account.get("nickname"))
            .and_then(Value::as_str)
            .map(str::to_string);
        let payload = json!({
            "auth": legacy.get("auth").cloned().unwrap_or(Value::Null),
            "account": legacy.get("account").cloned().unwrap_or(Value::Null),
            "prefixPath": legacy.get("prefixPath").cloned().unwrap_or(Value::Null),
            "endpoint": legacy.get("endpoint").cloned().unwrap_or(Value::Null),
            "platform": legacy.get("platform").cloned().unwrap_or(Value::Null),
            "edition": legacy.get("edition").cloned().unwrap_or(Value::Null),
        });
        // old （`auth.json`） is up when ： that when has 1 up ，
        // by `WORKBUDDY_EDITION` ， this ** by payload edition
        // **（， as not ）。
        let provider = crate::server::core::providers::workbuddy::Region::from_edition_id(
            payload.get("edition").and_then(Value::as_str),
        )
        .provider_id();
        match self.add_account(&payload, nickname.as_deref(), Some(provider)) {
            Ok(account) => Some(account),
            Err(error) => {
                logging::log("[Accounts]", &format!("❌ 旧登录态迁移失败: {error}"));
                None
            }
        }
    }
}

/// `limiterGroups` / `limiters` all `models` ， and 。
fn rewrite_models_arrays(value: &mut Value, mapping: &HashMap<String, String>) -> bool {
    match value {
        Value::Object(object) => {
            let mut changed = false;
            for (key, child) in object.iter_mut() {
                if key == "models" {
                    if let Some(items) = child.as_array_mut() {
                        for item in items.iter_mut() {
                            if let Some(text) = item.as_str() {
                                if let Some(base) =
                                    mapping.get(&text.trim().to_ascii_lowercase())
                                {
                                    *item = Value::String(base.clone());
                                    changed = true;
                                }
                            }
                        }
                        let before = items.len();
                        let mut seen = HashSet::new();
                        items.retain(|item| {
                            item.as_str()
                                .map(|text| seen.insert(text.to_ascii_lowercase()))
                                .unwrap_or(true)
                        });
                        changed |= items.len() != before;
                    }
                } else if rewrite_models_arrays(child, mapping) {
                    changed = true;
                }
            }
            changed
        }
        Value::Array(items) => {
            let mut changed = false;
            for item in items {
                changed |= rewrite_models_arrays(item, mapping);
            }
            changed
        }
        _ => false,
    }
}
