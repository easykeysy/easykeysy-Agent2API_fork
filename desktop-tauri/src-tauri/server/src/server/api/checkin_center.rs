//! to in （GET /api/checkin-center）。
//!
//! ── Why 1 ──────────────────────────────────
//! to in 1 need ：daily to 、 to set、 to 、
//! 1 /（ new / / ） 。 before already has 
//! （/api/accounts、/api/auto-checkin），「 by + supports to +
//! already 」 in after （`supports_checkin` / `checkin_providers`）， before 
//! will 1 —— that is 「 as 」 。 this in server 1 ，
//! 1 request， this 1 。
//!
//! ** up **（Loomy / new 、ZCode can ）
//! ** not in ** this ： 1 need 1 up ，must
//! 「 up request」。 to ， before to after by need has 
//! （/onboarding、/zcode-claim/preview）—— that use ， 1 not 。
//! （CodeArts `/codearts-welfare/preview` not in ： daily
//! 2026-10 already and daily to ， `/api/accounts/checkin` that 1 ——
//! in ，。）
//!
//! ── and to 1 ─────────────────────────────
//! 「daily to 」 = ∈ can （`auto_checkin::checkin_providers()`，
//! + when ）** and ** `supports_checkin` as —— and `resolve_checkin_targets` is 1 for ，
//! to in to 「 can 9 」 and 「 to 」 to not will 。
//! its `outOfScope`（ by ，Reason）。
//!
//! ── response ───────────────────────────────────────────────
//! ```json
//! {
//!   "daily": {
//!     "providers": [ { "id", "label",
//!                      "accounts": [ { "id", "name", "available", "checkinAt",
//!                                      "checkedInToday", "streakDays" } ],
//!                      "doneCount", "totalCount" } ],
//!     "outOfScope": [ { "label", "reason", "count" } ],
//!     "todayDone": 6, "todayEligible": 9
//!   },
//!   "extras": {
//!     "onboarding": [ { "id", "name", "provider" } ],
//!     "plans":      [ { "id", "name", "provider", "claimAt", "claimPlans" } ]
//!   },
//! "auto": { … and GET /api/auto-checkin … },
//!   "history": [ { "at", "date", "reason", "succeeded", "total", "skipped",
//!                  "failed", "failedCount" } ]
//! }
//! ```
//! `history` new in before （`checkin_history` ）， before by when 。

use std::collections::BTreeMap;

use axum::extract::State;
use axum::response::Response;
use serde_json::{json, Value};

use crate::server::core::auto_checkin::{self};
use crate::server::core::billing::checkin;
use crate::server::core::checkin_history;
use crate::server::core::providers::{kind_id, ProviderKind};
use crate::server::http::ok_json;
use crate::server::ServerState;

/// when use ； old to to when 。
fn checked_in_today(account: &Value) -> bool {
    crate::server::core::account_store::checkin_state::checked_in_today(account)
}

/// （ by empty， and billing::checkin 1 ）
fn account_text(account: &Value, key: &str) -> String {
    account
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

/// ： 1 （ and / request，
/// `store_view::display_name_of`），（ old ） by old `name`。
/// file use this 1 ， to in not 。
fn account_display_name(account: &Value) -> String {
    let display = account_text(account, "displayName");
    if display.is_empty() {
        account_text(account, "name")
    } else {
        display
    }
}

/// outOfScope Reason（ by `supports_checkin` excluding 1 for ）：
/// this need —— is 「 this Why not can to 」。
///
/// ── ────────────────────────────────────────────
/// 「 no has 」 this by （`CheckinInfo::exclude_guest`）：
/// 1 has 、 can not can ， is provider 。 down by `isGuest` /
/// `edition` / Accio ** no old **。
fn out_of_scope_reason(account: &Value) -> &'static str {
    let provider = account_text(account, "provider");
    let guest = account.get("isGuest").and_then(Value::as_bool) == Some(true);
    // ： has 「excluding」 1 will to 
    // （ its excluding is ， not will to ）
    if let Some(declared) = crate::server::core::plugin_host::checkin_of(&provider) {
        if declared.exclude_guest && guest {
            return "访客账号没有积分体系";
        }
        // `plans` / `onboarding` / `growth` no daily to 
        // （ZCode has 「」）： by need ， as this no 。
        if !declared.daily && (declared.plans || declared.onboarding || declared.growth) {
            return "本家没有每日签到，福利在下方对应卡片里";
        }
        return "没有签到链路";
    }
    // ****： and 「 no has to 」 is —— up this 1 has to 
    // （ can ）， is no has （ for 1 401）。
    // in provider / edition before ： edition cn， not will and down that ；
    // and `resolve_checkin_targets` when that 。
    if guest {
        return "访客账号没有积分体系";
    }
    // WorkBuddy / Qoder in `supports_checkin` 
    //（ before 、 after ）， in intl can can is Accio。
    if account.get("edition").and_then(Value::as_str) == Some("intl") {
        return "国际版账号没有签到活动";
    }
    if crate::server::core::account_store::is_accio_family(&provider) {
        return "未接入签到链路";
    }
    "没有签到链路"
}

/// to in （ not up request —— by before by need ）
pub async fn get_center(State(state): State<ServerState>) -> Response {
    // ： not in not （ in ， up ）
    let accounts: Vec<Value> = state
        .store()
        .list_visible_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|account| {
            account.get("provider").and_then(Value::as_str) != Some(kind_id(ProviderKind::Chatglm))
                || account.get("isGuest").and_then(Value::as_bool) != Some(true)
        })
        .collect();

    // ── daily to ： ∈ **available** and supports_checkin ──
    // by （）； by 「 + Reason」
    // （label use ， by excluding ）。
    //
    // use `available_checkin_providers`（ not is `checkin_providers`）： after is 
    // config 、 already ， will in no when 
    // failure empty（2026-10 use ）。available is 「
    // this ， or already has 」—— `providers::serves_or_has_accounts`。
    let checkin_providers = auto_checkin::available_checkin_providers(state.store());
    let mut buckets: Vec<Vec<Value>> = vec![Vec::new(); checkin_providers.len()];
    let mut out_of_scope: BTreeMap<String, (String, String, i64)> = BTreeMap::new();
    for account in &accounts {
        let provider = account_text(account, "provider");
        let provider = if provider.is_empty() {
            crate::server::core::providers::DEFAULT_PROVIDER_ID.to_string()
        } else {
            provider
        };
        let slot = checkin_providers
            .iter()
            .position(|id| id == &provider)
            .filter(|_| checkin::supports_checkin(account));
        if let Some(index) = slot {
            buckets[index].push(json!({
                "id": account_text(account, "id"),
                "name": Value::from(account_display_name(account)),
                "available": account.get("available").and_then(Value::as_bool).unwrap_or(true),
                "checkinAt": account.get("checkinAt").cloned().unwrap_or(Value::Null),
                "checkedInToday": checked_in_today(account),
                "dailyCheckin": account.get("dailyCheckin").cloned().unwrap_or(Value::Null),
                // （cn / intl， null）：WorkBuddy in this is 
                // 「」（）， by and need 「 to 」
                "edition": account.get("edition").cloned().unwrap_or(Value::Null),
                // in （WorkBuddy ； to after ，
                // `AccountStore::mark_streak_days`）。 as （0 = no to ），
                // and 1 —— by ， not 「」。
                "streakDays": account.get("streakDays").cloned().unwrap_or(Value::from(0)),
            }));
        } else {
            let label = crate::server::core::providers::label_of(&provider);
            let reason = out_of_scope_reason(account);
            let entry = out_of_scope
                .entry(format!("{provider}|{reason}"))
                .or_insert_with(|| (label, reason.to_string(), 0));
            entry.2 += 1;
        }
    }

    let mut provider_rows: Vec<Value> = checkin_providers
        .iter()
        .zip(buckets.into_iter())
        .map(|(id, rows)| {
            let done = rows
                .iter()
                .filter(|row| row.get("checkedInToday").and_then(Value::as_bool) == Some(true))
                .count();
            // （ / / UI ）。
            // no old not this ， （ and before 1 ）。
            let declared = crate::server::core::plugin_host::checkin_of(id);
            let manual_plans = declared.as_ref().is_some_and(|declared| declared.daily && declared.plans);
            let mut row = json!({
                "id": id,
                "label": auto_checkin::provider_label(id),
                "accounts": rows,
                "doneCount": done,
                "totalCount": rows.len(),
                "manualPlans": manual_plans,
            });
            if let Some(declared) = declared {
                if let Some(object) = row.as_object_mut() {
                    object.insert(
                        "description".to_string(),
                        declared.description.map(Value::String).unwrap_or(Value::Null),
                    );
                    object.insert(
                        "modes".to_string(),
                        serde_json::to_value(&declared.modes).unwrap_or(Value::Null),
                    );
                    object.insert(
                        "uiHooks".to_string(),
                        serde_json::to_value(&declared.ui_hooks).unwrap_or(Value::Null),
                    );
                }
            }
            row
        })
        .collect();
    provider_rows.sort_by_key(|row| row.get("totalCount").and_then(Value::as_u64).unwrap_or(0) == 0);
    let today_done: u64 = provider_rows
        .iter()
        .map(|row| row.get("doneCount").and_then(Value::as_u64).unwrap_or(0))
        .sum();
    let today_eligible: u64 = provider_rows
        .iter()
        .map(|row| row.get("totalCount").and_then(Value::as_u64).unwrap_or(0))
        .sum();
    let out_rows: Vec<Value> = out_of_scope
        .into_values()
        .map(|(label, reason, count)| json!({ "label": label, "reason": reason, "count": count }))
        .collect();

    // new ：** by **（`CheckinInfo::onboarding`）。
    // is 1 `onboarding_rows_of("loomy")` —— that is provider
    // （ this 1 has no has 1 ）， 1 。 after ，
    // new need this 1 ， in this 。
    //
    // excluding（`exclude_guest`）： no has 
    // （ in 1 401）， will 。
    let mut onboarding_rows: Vec<Value> = Vec::new();
    for account in &accounts {
        let provider = account_text(account, "provider");
        let Some(declared) = crate::server::core::plugin_host::checkin_of(&provider) else {
            continue;
        };
        if !declared.onboarding {
            continue;
        }
        if declared.exclude_guest && account.get("isGuest").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let id = account_text(account, "id");
        onboarding_rows.push(json!({
            "id": id,
            "name": Value::from(account_display_name(account)),
            "provider": provider,
            // `settled` is ** 1 **（ already empty）：、
            // up request，「 already 」 and in / to after 
            // ， not as 1 up 。
            "settled": super::onboarding::settled_view(state.store(), &id)
                .unwrap_or(Value::Null),
        }));
    }
    // ZCode ****（`account.claimPlans`）—— after 
    // ， up request， not 「 up 」； 「」 by 
    // for （ when in before ， not in after 1 ）。
    // is `plans` （ by zcode / zcode-intl id ）。
    let plan_rows: Vec<Value> = accounts
        .iter()
        .filter(|account| {
            crate::server::core::plugin_host::checkin_of(&account_text(account, "provider"))
                .is_some_and(|declared| declared.plans)
        })
        .map(|account| {
            json!({
                "id": account_text(account, "id"),
                "name": Value::from(account_display_name(account)),
                "provider": account_text(account, "provider"),
                "claimAt": account.get("claimAt").cloned().unwrap_or(Value::Null),
                // （{planId: ms}）：「」 by 
                // for （claimPlans in before already has ， use ，
                // after not 1 when ）
                "claimPlans": account.get("claimPlans").cloned().unwrap_or(Value::Null),
            })
        })
        .collect();

    // ： is `growth` （ by `supports_growth` 
    // 「provider == workbuddy and 」）。 excluding in 
    // `supports_growth` —— that is ****， not is provider can 。
    // and new 1 ： to ， by before when by need 
    // `GET /api/accounts/{id}/growth`（ that is 1 up ）。
    let growth_rows: Vec<Value> = accounts
        .iter()
        .filter(|account| {
            crate::server::core::plugin_host::checkin_of(&account_text(account, "provider"))
                .is_some_and(|declared| declared.growth)
                && crate::server::api::growth::supports_growth(account)
        })
        .map(|account| {
            json!({
                "id": account_text(account, "id"),
                "name": Value::from(account_display_name(account)),
                "provider": account_text(account, "provider"),
            })
        })
        .collect();

    ok_json(json!({
        "daily": {
            "providers": provider_rows,
            "outOfScope": out_rows,
            "todayDone": today_done,
            "todayEligible": today_eligible,
        },
        "extras": {
            "onboarding": onboarding_rows,
            "plans": plan_rows,
            "growth": growth_rows,
        },
        "auto": state.auto_checkin().state(),
        // WorkBuddy （ to in can ；empty）
        "keepalive": crate::server::core::billing::keepalive::state(),
        // when （）： / when / already when / up 1 。
        // and `auto`（ to ） and —— is ， and when 。
        "travel": crate::server::core::growth_scheduler::state(),
        "history": checkin_history::list(),
        // ：「」「」 this 
        // not 。 can by WorkBuddy not （
        // has 、 has daily）， no when not this 。
        "adminPanels": crate::server::core::plugin_host::admin_panels(),
        // to （ / / UI ）， by provider id 。
        // `daily.providers[]` 1 （ by use ）； this to 1 ，
        // for 「 no 」 to （e.g. no when ）。
        "checkinFeatures": crate::server::core::plugin_host::checkin_features(),
    }))
}
