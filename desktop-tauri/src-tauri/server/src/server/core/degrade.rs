//! （ workbuddy2api `internal/server/degrade.go`）。
//!
//! ── ───────────────────────────────────────────────
//! `passthrough` / `append` down ，request up （HTTP 400 + 
//! ） when ， is ****：client system down ，
//! （、 no 、session no ）。 when is ：
//!
//! ```text
//! ① in （`prompt::DEGRADED_PROMPT`） 1 ；
//! ② when ****： down 1 when ，passthrough/append request
//! in ， not 1 400 。
//! ```
//!
//! to ** 00:00（UTC+8）** —— and 1 ：
//! and 「」（ up by statistics）， is 1 ，
//! 「none」 will use in after not to system 。
//!
//! ── 、（ and 1 ）─────────────────────
//! is **** not is config： etc 1 （ use is 
//! as 400 ）， no has 。 will 「 when this 」
//! （？？）， not 。
//!
//! ── not ───────────────────────────────────────────────
//! ** not ** when （ that ）：otherwise 1 
//! client will none after ， etc 。 this is down 
//! （`Trigger` 「 already in then not 」）。
//!
//! ── in ─────────────────────────────────────────────────
//! - `upstream::provider_loop`：request when 1 （ ），
//! when and 1 ；
//! - `api::prompt`：set need 「 in is not is 、 to when 」——
//! use not to when ， this is 。

use std::sync::atomic::{AtomicI64, Ordering};

/// UTC+8 （ms）。
///
/// use not is when ： in when no has when ，`Asia/Shanghai` in 
/// 1970 after as +08:00 —— is use `time.FixedZone("CST", 8*3600)`
/// （ by ： not when config）。
const CST_OFFSET_MS: i64 = 8 * 3600 * 1000;

/// 1 ms
const DAY_MS: i64 = 86_400_000;

/// when （UTC ms）；`0` = 。
///
/// use `AtomicI64` not is `Mutex<Option<...>>`： is 1 ， is 
/// 「 1 / 1 」， no has need need not 。 and is 
/// 1 （ 1 、 1 ），none。
static UNTIL_MS: AtomicI64 = AtomicI64::new(0);

/// current is 。
pub fn active() -> bool {
    until_ms() > 0
}

/// when （UTC ms）；`0` = current not in 。
///
/// in this as 「 no has 」—— is use not when ，
/// not will 「 in 、 already 」 this 。
pub fn until_ms() -> i64 {
    let until = UNTIL_MS.load(Ordering::SeqCst);
    if until > 0 && crate::server::logging::now_ms() < until {
        until
    } else {
        0
    }
}

/// （return when ，UTC ms）。
///
/// already in ** not **：return when （module header）。
pub fn trigger() -> i64 {
    let now = crate::server::logging::now_ms();
    let current = UNTIL_MS.load(Ordering::SeqCst);
    if current > now {
        return current;
    }
    let next = next_midnight_cst_ms(now);
    UNTIL_MS.store(next, Ordering::SeqCst);
    next
}

/// （ when ）。
///
/// no has this （ can etc to ）； 1 ， is as 
/// use can can ** **（e.g. to `custom` ）， this when need etc to 
/// system ，。 by set use 。
pub fn clear() {
    UNTIL_MS.store(0, Ordering::SeqCst);
}

/// `now` after UTC+8 （， for ）。
///
/// （ and 1 ）：
/// - `23:59` → `00:00`（minutes after ）；
/// - `00:00` → **** `00:00`（， down is ）。
///
/// ： when to CST （+8h）， is `div_euclid(DAY) * DAY`，
/// 1 as 「 」；`div_euclid` 1970 before when 
/// in up （ in Rust default to ， `/` will ）。
fn next_midnight_cst_ms(now_ms: i64) -> i64 {
    let shifted = now_ms + CST_OFFSET_MS;
    let start_of_today = shifted.div_euclid(DAY_MS) * DAY_MS;
    start_of_today + DAY_MS - CST_OFFSET_MS
}

/// when （UTC+8， and up to when ）。
/// `0` or none when to empty —— this is ， not can as 
/// （release is panic=abort）。
pub fn until_text() -> String {
    let until = until_ms();
    if until <= 0 {
        return String::new();
    }
    let Some(utc) = chrono::DateTime::from_timestamp_millis(until) else {
        return String::new();
    };
    let Some(offset) = chrono::FixedOffset::east_opt(8 * 3600) else {
        return String::new();
    };
    utc.with_timezone(&offset)
        .format("%Y/%m/%d %H:%M")
        .to_string()
}
