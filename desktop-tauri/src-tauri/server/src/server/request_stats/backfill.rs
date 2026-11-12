//! old ****： 「 no 」 use 1 。
//!
//! ── when need need （ in is ** 1 **）──────────────────
//! `accountStats` is after 。 in up before no has this 1 ，
//! statistics****（ 365 、`all` can can ，
//! ， will ）。 is after ，
//! that when request「unknown」 —— no ， use to is 
//! 1 unknown，。
//!
//! its ** has **（ old in `accountId` / `accountName`），
//! is when no 。 this is can ：
//! that ， not is to 1 。
//!
//! ── Whyall（ not use ）─────────────────────────
//! 「 」 can can ** old file**： in 1 by 
//! `fold_into_daily` ， as （ is empty），
//! not will this 。 this is 1 ** 1 **，
//! `db::migrate::import_daily` in old when ——
//! in need 1 、 as 「 has no has 」 1 ，
//! this in after 1 is 「 no has 」。
//!
//! ** can not can **： use not 「unknown」 ，
//! this not is ， is use when （ in old after 、
//! and old 1 use ）。（ down 「 not not 」）
//! in when down need ，。
//!
//! ── Why is 「」 not is that 1 ──────────────────
//! `accountStats` `requests` / `tokens` ，
//! （ / provider / ） and total for need ——
//! 「 and = total」 is can for before （ `range_totals`）。
//! `fold_into_daily` ， this not by ** 1 **，
//! and `record` 、`clear_where` 。
//!
//! ── ： this will has ─────────────────────────────
//! use is ** **， and old when can can not ，：
//! ① old failurerequest token（`normalize` that when no has failure），
//! before `normalized_for_aggregate` failure token ；
//! ② success from 「 2xx」 as 「2xx and noneerror need 」， will by successfailure；
//! ③ is ， will after 1 ， is can can 
//! **** —— after total。
//! ① ② 、③ ， in 「 to request」 this 1 。
//! this is ** has **： can this and for ， not 1 not 
//! unknown。 that is when ， ③ 1 。
//!
//! ── not not ────────────────────────────────────────────
//! can can use （empty / by delete）， ——
//! that ** no has ** ， can in 「unknown」。
//! is ` >= requests`： already ，
//! will empty， that unknown。
//! this ** etc **： has `accountStats`，
//! down not ， not will 。

use std::collections::{BTreeMap, BTreeSet};

use super::clock::{date_key, day_of};
use super::fold_into_daily;
use super::record::{DailyEntry, RequestEntry};

/// all「 no 、 and 」 。
///
/// `daily`，return （， use and ）。
/// returnempty = no has 1 need need ， use not 。
///
/// ：、 and this already 1 not ——
/// that is when ， will none 。
pub(super) fn rebuild_legacy_days(
    daily: &mut BTreeMap<String, DailyEntry>,
    entries: &[RequestEntry],
) -> Vec<String> {
    // ① ： has request、 no has 。
    // `account_stats.is_empty()` not `requests > 0` ：
    // request no has ， will down 1 empty id 
    // （`fold_into_daily` ），「empty」 etc 「 when no this 1 」。
    let candidates: BTreeSet<String> = daily
        .iter()
        .filter(|(_, day)| day.requests > 0 && day.account_stats.is_empty())
        .map(|(key, _)| key.clone())
        .collect();
    if candidates.is_empty() {
        return Vec::new();
    }

    // ② by ， 。
    // this ** not ** its ： has already has 、
    // and need need ， will 「 1 」。
    let mut rebuilt: BTreeMap<String, DailyEntry> = BTreeMap::new();
    for item in entries {
        let key = date_key(day_of(item.ts));
        if !candidates.contains(&key) {
            continue;
        }
        let day = rebuilt
            .entry(key.clone())
            .or_insert_with(|| DailyEntry::new(key.clone()));
        fold_into_daily(day, &item.normalized_for_aggregate());
    }

    // ③ 「total」 。
    // `rebuilt` no has （ ） not this 1 ， ——
    // not can 「 as 0 」 「 that no has request」 。
    let mut changed = Vec::new();
    for (key, day) in rebuilt {
        let Some(existing) = daily.get(&key) else {
            continue;
        };
        if day.requests < existing.requests {
            continue;
        }
        changed.push(key.clone());
        daily.insert(key, day);
    }
    changed
}

/// （`[Storage]` —— in by **** in old when use ，
/// that when ， can `console_line`， and its 1 ）。
///
/// not is ： this and use to can can not ，
/// when need has 「 」 this 。
pub(super) fn report_line(changed: &[String]) -> String {
    format!(
        "已按明细回填 {} 天的账号维度（{}）：这些天此前没有账号明细，\
         重算后该日总量以明细为准，与旧值可能有小幅差异",
        changed.len(),
        changed.join(" / ")
    )
}
