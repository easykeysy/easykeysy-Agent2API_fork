//! ****： 1 in 「 no not 」 when ，。
//!
//! ── need ──────────────────────────────────────────
//! ZCode （`start-plan`） need need **request** 1 by WebView
//! 。headless / Docker no has WebView， that 
//! **empty** —— is 1 request in failure。
//!
//! ── after （2026-10 ）───────────────────────────
//! and 「 need not need 」 ZCode to ，** this 
//! in **： is ****（`upstream::rotate` ）
//! and ****（`provider_loop` 556 ）， that in 。 by 1 
//! （[`LOCAL_PRECONDITION_STATUS`]）「 not 」， old 
//! /； `readiness` 。
//!
//! before this failure will **request**：`provider_loop` failure
//! `return Err`，（ 1 / 3） no has will 。 up down is 
//! `attempts=1`、 has this 1 、 503 —— 1 request in catpaw /
//! codearts / autoclaw up is can 。 use to not is 「 this when unavailable」， is 
//! 「」。
//!
//! ── Why this is provider ， not is 、 not is request ──────
//! · ** and none**： is 1 、 not （`captcha.rs` module header 
//! ）， 1 1 not will ；
//! · ** and requestnone**： in 、、 before ， 1 request 
//! is 1 request；
//! · ****： this up has no has WebView in ， that is ，
//! not is 。
//! ：** can **。 to not this request， this 
//! to down 1 can 。
//!
//! ── in 1 ──────────────────────────────────
//! 1. ****（）：failure to use 
//! [`LOCAL_PRECONDITION_STATUS`]， and 「 up 503」。 no has this ，
//! can **error** this not this ， is to ， 1 
//! will ；
//! 2. ****（）： in failure [`hold`] 1 provider ，
//! [`GATE_TTL_MS`]； when [`release`] before ；
//! 3. ****（ and ）：
//! · `upstream::rotate::accounts_in_providers` 
//! —— is after request** not will this 1 **， 1 failure not ；
//! · `upstream::provider_loop` failure 556 （ and 3
//! 1 ，「」）—— this is ** 1 **： no 
//! when ， that request will to this 1 ， when must not is 。
//!
//! ── Why is 556 ────────────────────────────────────
//! RFC 9110 §7.1 5xx server use ，client by use servererror
//! ，add 1 not 。 556 not is use 503 ** 1 ** by is 
//! ：request `status=503` in has （ up 503、 this 1 no has 
//! 、 and this 「 not 」）， when need 「 from request」 from before 
//! ， can 。 has ， 1 SQL can ， for 
//! （ `deploy/nas` ）。
//!
//! ** will to client**： （ or has this 1 ） when ，errorreturn，
//! not —— that is 1 can （ 「 use 」）， not this as 
//! 。
//!
//! ── Why will ， not is etc ────────────
//! [`release`] in when use 。if can ， that 「
//! 」「 is 1 by 」 this will this 1 —— 
//! is ** this 1 not request**， 。： to 
//! ， down 1 new 1 。 is has and ：headless up 
//! [`GATE_TTL_MS`] 1 failure（** no has up **，`take()` before return），
//! is not need need config、 not need need 。
//!
//! [`hold`] in already when ** not **： ， will seconds when 
//! new 1 ， to not —— that etc up can 。
//!
//! ── in ──────────────────────────────────────────
//! 「none」（）。 is ， not is ： 1 will request to 
//! 、 or this 1 no ， will 1 failure —— 
//! 1 。
//!
//! ── ────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde_json::{json, Value};

use crate::server::logging;

/// 「 this 1 in **** not 」 use （module header 1 ）。
///
/// by 「 not is 」， use 「 from request」。
/// this ， not 。
pub const LOCAL_PRECONDITION_STATUS: u16 = 556;

/// （ms）。
///
/// and TTL （ `captcha::TOKEN_TTL_MS`，120 seconds）： no has 
/// （ no ， = ）， then 「 already 」 need 
/// etc new to 。
pub const GATE_TTL_MS: i64 = 120_000;

/// 1 。
#[derive(Clone, Debug)]
struct Gate {
    /// to that （ and use ；** not and **）
    reason: String,
    /// **** when （ `hold`）。 already when hold not new ， by module header。
    held_at: i64,
}

#[derive(Default)]
struct Registry {
    gates: HashMap<String, Gate>,
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

/// 1 ： this 1 in not ， after request。
///
/// by in **failure** use （ is failure， not can can for not up ）。
/// when current when ； need when use [`hold_at`]。
///
/// return `true` = use ** from **（ use can 1 ，
/// request will ；`false` = ）。
pub fn hold(provider_id: &str, reason: &str) -> bool {
    hold_at(provider_id, reason, logging::now_ms())
}

/// [`hold`] can when 。
///
/// need can when —— otherwise「 not 」 that not ： when down 
/// `hold` seconds，none has no has ，「TTL after 」 
/// （： ， use ）。 use need is ** can ** when 。
pub fn hold_at(provider_id: &str, reason: &str, now: i64) -> bool {
    let Some(provider_id) = non_empty(provider_id) else {
        return false;
    };
    let Ok(mut table) = registry().lock() else {
        return false;
    };
    match table.gates.get_mut(&provider_id) {
        Some(existing) => {
            // ** not new `held_at`**： already to when by 
            existing.reason = reason.to_string();
            false
        }
        None => {
            table.gates.insert(
                provider_id,
                Gate {
                    reason: reason.to_string(),
                    held_at: now,
                },
            );
            true
        }
    }
}

/// （、 or when use ）。
///
/// when this 1 **current in ** ： is ，
/// 1 ， has 1 「 when new 」 ，otherwise「
/// has Why」 can etc to 。
///
/// return `true` = 1 。
pub fn release(provider_id: &str) -> bool {
    let Some(provider_id) = non_empty(provider_id) else {
        return false;
    };
    let Ok(mut table) = registry().lock() else {
        return false;
    };
    let Some(gate) = table.gates.remove(&provider_id) else {
        return false;
    };
    logging::console_line(
        "[Routing]",
        &format!(
            "✅ {provider_id} 的本地闸门已撤（挡了 {}s，原因：{}），该家重新进入候选池",
            ((logging::now_ms() - gate.held_at) / 1000).max(0),
            gate.reason,
        ),
    );
    true
}

/// this 1 current is 。
///
/// **expired **：， not 1 to 
/// [`snapshot`] 「」 。
pub fn gated(provider_id: &str) -> bool {
    gated_at(provider_id, logging::now_ms())
}

/// [`gated`] can when （ by TTL， not is minutes）。
fn gated_at(provider_id: &str, now: i64) -> bool {
    let Some(provider_id) = non_empty(provider_id) else {
        return false;
    };
    let Ok(mut table) = registry().lock() else {
        return false;
    };
    match table.gates.get(&provider_id) {
        Some(gate) if now - gate.held_at < GATE_TTL_MS => true,
        Some(_) => {
            table.gates.remove(&provider_id);
            false
        }
        None => false,
    }
}

/// current （ and use ；expired not in ）。
///
/// `{provider, reason, heldForMs}`。****， not and ， in 
/// when to empty（ not 「 no has 」）。
pub fn snapshot() -> Vec<Value> {
    let now = logging::now_ms();
    let Ok(table) = registry().lock() else {
        return Vec::new();
    };
    let mut items: Vec<Value> = table
        .gates
        .iter()
        .filter(|(_, gate)| now - gate.held_at < GATE_TTL_MS)
        .map(|(provider, gate)| {
            json!({
                "provider": provider,
                "reason": gate.reason,
                "heldForMs": now - gate.held_at,
            })
        })
        .collect();
    // HashMap not ； and need 1 
    items.sort_by(|a, b| {
        a.get("provider")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .cmp(
                b.get("provider")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            )
    });
    items
}

/// use ：empty。
///
/// is ****， 1 cargo test all use 。 no has this 
/// ， use will （ before 1 hold will after 1 1 ），
/// that 。
///
/// ** not etc **： and can ， use need 
/// [`lock_for_tests`]。
#[cfg(test)]
pub fn clear_for_tests() {
    let Ok(mut table) = registry().lock() else {
        return;
    };
    table.gates.clear();
}

/// use ：all this use （ that ）。
///
/// and that `PROBE_SLOT_LOCK` 1 —— + 
/// can this ： by provider ， use use 1 provider id（`"zcode"`
/// is for ， 1 not to `plan.rs` and 1 ）。
#[cfg(test)]
pub fn lock_for_tests() -> std::sync::MutexGuard<'static, ()> {
    static SLOT: Mutex<()> = Mutex::new(());
    // in use 1 ： use panic already failure， this need 
    // is after use ， not is 「」 1 
    SLOT.lock().unwrap_or_else(|error| error.into_inner())
}

/// provider id ：empty 1 「 not this 」。
///
/// Why 1 ： use to is `region.provider_id()` and 
/// `rotate::provider_of(&account)`， can can is empty（ `provider` 
/// when `provider_of` to `""`）。empty 1 ，`gated("")` will **all**
/// provider 1 —— that is ， not is 1 。
fn non_empty(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod local_gates {
    //! ： / / / 。
    //!
    //! when 1 [`gated_at`] —— TTL is 120 seconds， use 
    //! need minutes， need 1 TTL after ** no **。
    //!
    //! use [`lock_for_tests`]： this is ， and use can 
    //! for before （ that and not can ）。

    use super::*;

    #[test]
    fn a_held_gate_gates_only_that_provider() {
        let _slot = lock_for_tests();
        clear_for_tests();
        let now = logging::now_ms();
        assert!(hold("zcode", "活动套餐令牌池为空"));
        assert!(gated_at("zcode", now), "hold 之后当场就该绕开");
        assert!(!gated_at("catpaw", now), "另一家没道理被牵连");
        // 1 is ** 1 provider id**： by id ， not 「 1 」
        assert!(!gated_at("zcode-intl", now));
    }

    #[test]
    fn an_empty_provider_id_gates_nothing() {
        let _slot = lock_for_tests();
        clear_for_tests();
        // `provider` ，`provider_of` to empty；empty will 
        // all 1 （ `non_empty` ）
        assert!(!hold("", "不该登记"), "空 provider 不登记");
        assert!(!gated(""), "空 provider 不绕行");
        assert!(snapshot().is_empty(), "表里不该多出东西");
    }

    #[test]
    fn a_gate_expires_rather_than_waiting_for_a_release() {
        let _slot = lock_for_tests();
        clear_for_tests();
        assert!(hold("zcode", "令牌池为空"));
        // use when TTL ： when down this can ， need 
        // minutes、 need not to TTL（ after etc no ）。
        let base = logging::now_ms();
        assert!(
            gated_at("zcode", base + GATE_TTL_MS - 1),
            "寿命内仍然挡着 —— 比令牌 TTL 还短就放行是白挡"
        );
        assert!(
            !gated_at("zcode", base + GATE_TTL_MS),
            "到点自动放行：下一次构造重新探一次本地条件"
        );
        assert!(
            snapshot().is_empty(),
            "过期的登记项跟着清掉，别报出一个不存在的闸门"
        );
        // after = 1 ** new **（ from that 1 ），
        // not is 
        assert!(
            hold_at("zcode", "令牌池仍然为空", base + GATE_TTL_MS + 1),
            "过期后应当能重新成立"
        );
        assert!(
            gated_at("zcode", base + GATE_TTL_MS + 1),
            "新闸门从它自己成立的那一刻起算"
        );
        // to when = new when + TTL（ `base` already and none）
        assert!(
            !gated_at("zcode", base + 2 * GATE_TTL_MS + 1),
            "新闸门同样只活一个 TTL"
        );
    }

    #[test]
    fn reholding_a_live_gate_does_not_extend_it() {
        let _slot = lock_for_tests();
        clear_for_tests();
        let base = logging::now_ms();
        assert!(hold_at("zcode", "令牌池为空", base));
        // seconds will 1 。 if will ， to not ，
        // up that can etc no has —— this use 「 not 」。
        //
        // when **must** 1 ：seconds when ， and not in down that 
        // up not （ ， use —— that is empty）。
        assert!(
            !hold_at("zcode", "令牌池还是为空", base + GATE_TTL_MS / 2),
            "第二次登记不改变开合状态"
        );
        assert!(
            gated_at("zcode", base + GATE_TTL_MS - 1),
            "第二次登记之后仍在寿命内"
        );
        assert!(
            !gated_at("zcode", base + GATE_TTL_MS),
            "到期时刻由首次登记决定 —— 续期的话这里还挡着，那是「永远挡着」的开始"
        );
    }

    #[test]
    fn releasing_opens_the_gate_immediately() {
        let _slot = lock_for_tests();
        clear_for_tests();
        assert!(hold("zcode", "令牌池为空"));
        assert!(release("zcode"), "撤掉一道真实存在的闸门返回 true");
        assert!(!gated("zcode"), "有令牌了就不该再绕");
        assert!(!release("zcode"), "撤一道不存在的闸门返回 false");
        assert!(snapshot().is_empty());
    }
}
