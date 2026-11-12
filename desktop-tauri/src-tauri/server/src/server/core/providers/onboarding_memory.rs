//! 1 new ****（Loomy / CodeArts use ）。 
//! 「 already 」 by 1 to `pluginLedger.onboarding`，
//! —— not （ is when ， `api::onboarding`）。
//!
//! ── ──────────────────────────────────────────────────
//! up `onboardingSettled`：`{ at, tasks: [...], earned, total }`，
//! `tasks` and return （ that 1 `done`）。 has 1 ：
//! ** this new （ 1 ） already ， after not will 。**
//!
//! ── Why need has ────────────────────────────────────────────
//! new is 1 ： no has 。 no has this ， to in 、
//! to after need as new 1 up （Loomy 、CodeArts 
//! ）， is 「 already 」。 already use 
//! （ `raccoon::onboarding`）， this 1 to ：
//! ⇒ and 1 ** up request**， use 。
//!
//! ── when / when / when ────────────────────────────────
//! · ：[`remember`]， or to 「 no has 」 that 1 。including「
//! （client）」 —— that this 1 already ；
//! · ：[`snapshot`] / [`serve`]，`?refresh=1` and ；
//! · ：[`remember`] to `unclaimed > 0` when old —— up new 1 
//! new ，， to 「」 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{json, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::logging;

/// from ； no has / not （ for ）→ None。
pub fn snapshot(record: Option<&Value>) -> Option<Value> {
    record
        .and_then(|record| record.get("onboardingSettled"))
        .filter(|value| value.is_object())
        .cloned()
}

/// use 1 ：`tasks / earned / total` ，`unclaimed` 0
/// （ already ）， `settled` / `settledAt` can 「 this is 、 not is 
/// 」。 use that （provider / note） and 。
pub fn serve(snapshot: &Value) -> Value {
    json!({
        "tasks": snapshot.get("tasks").cloned().unwrap_or(Value::Null),
        "earned": snapshot.get("earned").cloned().unwrap_or(Value::Null),
        "total": snapshot.get("total").cloned().unwrap_or(Value::Null),
        "unclaimed": 0,
        "settled": true,
        "settledAt": snapshot.get("at").cloned().unwrap_or(Value::Null),
    })
}

/// by **response **， and response（`payload` 
/// tasks / settled）。
///
/// `settled == true`（ this is 「 1 to 」）⇒ and return，
/// when in response `settledAt`（ when ）—— nonepath is path to 
/// is 1 。otherwise**** old and return None —— up new 
/// （ new 1 、），， to 「」 。 is 
/// `payload.settled` not is 1 「unclaimed is not is 0」： for this 1 
/// not 1 （CodeArts `blocked` not not to ， has that 
/// not ）， by that 。
///
/// `previous` is ** before ** that （ use use [`snapshot`] ， no has `None`）。
/// and old 1 （） when use old `at`：`refresh=1` will use 
/// 1 new 1 ， if new when ，「」「」，
/// that not is （ this is when ）。
///
/// failure 1 、return —— up that already ， not this as 1 
/// failure「 no 」。
pub fn remember(
    store: &AccountStore,
    account_id: &str,
    label: &str,
    payload: &mut Value,
    previous: Option<&Value>,
) -> Option<Value> {
    let settled = payload.get("settled").and_then(Value::as_bool) == Some(true);
    if !settled {
        if !store.mark_onboarding_settled(account_id, Value::Null) {
            logging::verbose(
                "[Onboarding]",
                &format!("{label} 账号 {account_id}：结算记忆清除失败（不影响本次结果）"),
            );
        }
        return None;
    }
    let tasks = payload.get("tasks").cloned().unwrap_or(Value::Null);
    let at = match previous {
        // 1 already in ⇒ use when 
        Some(old) if old.get("tasks") == Some(&tasks) => old
            .get("at")
            .and_then(Value::as_i64)
            .unwrap_or_else(logging::now_ms),
        _ => logging::now_ms(),
    };
    let snapshot = json!({
        "at": at,
        "tasks": tasks,
        "earned": payload.get("earned").cloned().unwrap_or(Value::Null),
        "total": payload.get("total").cloned().unwrap_or(Value::Null),
    });
    if !store.mark_onboarding_settled(account_id, snapshot.clone()) {
        logging::log(
            "[Onboarding]",
            &format!(
                "⚠️ {label} 账号 {account_id}：新手任务结算记忆落库失败（本次结果不受影响，下次查询会重试）"
            ),
        );
    }
    if let Some(object) = payload.as_object_mut() {
        object.insert("settledAt".to_string(), json!(at));
    }
    Some(snapshot)
}
