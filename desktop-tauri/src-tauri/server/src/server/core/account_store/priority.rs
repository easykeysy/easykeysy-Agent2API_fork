//! then （ for workbuddy-account-store.mjs ）。
//!
//! is ****：must 1 ， and will 「」
//! （ that ， its ）。：
//! - （add/） to 1 （409）， by use 1 empty；
//! - when for has 1 （`renumber_consecutively` + store 
//! ）， has ；
//! - already use —— otherwise use ， new enabled will 。
//!
//! ── 1 use ：****（all use 1 ）─────────
//! by provider （workbuddy and raccoon 0–9999），
//! in ** 1 **： in 1 ， when by from to 
//! ， use / unsupported this / this in 。 is 「 use 1 」
//! by ， not has provider by 。
//! by ** that 1 **， use 。
//!
//! ， not and HTTP。

use serde_json::Value;

/// default： from this ， after 
pub const DEFAULT_PRIORITY: i64 = 100;
pub const MIN_PRIORITY: i64 = 0;
pub const MAX_PRIORITY: i64 = 9999;

/// 1 ： to default， and in 
/// （ for Node `normalizePriority`：null/undefined/'' and NaN ）。
pub fn normalize_priority(value: Option<&Value>, fallback: i64) -> i64 {
    let number = match value {
        None | Some(Value::Null) => return fallback,
        // empty in JS `Number('') === 0`， Node to fallback 
        Some(Value::String(text)) if text.trim().is_empty() => return fallback,
        Some(Value::Number(number)) => match number.as_f64() {
            Some(value) if value.is_finite() => value,
            _ => return fallback,
        },
        Some(Value::String(text)) => match text.trim().parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => return fallback,
        },
        // for // in JS Number() as NaN → 
        _ => return fallback,
    };
    let rounded = number.round();
    // Math.min(MAX, Math.max(MIN, round(num)))
    (rounded as i64).clamp(MIN_PRIORITY, MAX_PRIORITY)
}

/// from up 1 （ 1 Value）
pub fn normalize_priority_value(value: i64) -> i64 {
    value.clamp(MIN_PRIORITY, MAX_PRIORITY)
}

/// ：， by when （ and Node `byPriorityOrder` 1 ）。
/// 1 by ， this is file when 。
pub fn by_priority_order(a: (i64, i64), b: (i64, i64)) -> std::cmp::Ordering {
    let (a_priority, a_added) = a;
    let (b_priority, b_added) = b;
    normalize_priority_value(a_priority)
        .cmp(&normalize_priority_value(b_priority))
        .then(a_added.cmp(&b_added))
}

/// down 1 available： in has after （add not already has ）。
///
/// `used` is already use 。 this is **** not is ：
/// is ， after use in `priority` up 1 
/// （`sql::priorities_except`） can ， not as 20 JSON
/// 1 。
///
/// use when from default 1 empty； not to default
/// （ and Node path）。
pub fn next_free_priority(used: &[i64]) -> i64 {
    let max = used
        .iter()
        .map(|value| normalize_priority_value(*value))
        .max()
        .unwrap_or(DEFAULT_PRIORITY - 1);
    let mut candidate = max + 1;
    while used.contains(&candidate) && candidate <= MAX_PRIORITY {
        candidate += 1;
    }
    if candidate <= MAX_PRIORITY {
        return candidate;
    }
    for priority in MIN_PRIORITY..=MAX_PRIORITY {
        if !used.contains(&priority) {
            return priority;
        }
    }
    DEFAULT_PRIORITY
}

/// 1 （return to before and ）
#[derive(Clone, Debug)]
pub struct PriorityAssignment {
    pub id: String,
    pub name: String,
    pub from: i64,
    pub to: i64,
}

/// 1 ****： has use up 。
///
/// is （ not and HTTP）， not `StoredAccount` —— use 
/// from / JSON this ， then in this 1 。
pub struct QueueEntry<'a> {
    pub id: &'a str,
    pub provider: &'a str,
    pub priority: i64,
    pub added_at: i64,
}

/// 「」 （ ** not **）。
///
/// by ， by —— up 「 2 in 1 
/// after 」 this need ， 1 must****： in ，
/// after and 。 this ：
/// 1. by ；
/// 2. = **current before ** that （， and when ）；
/// 3. by ， and by when （ and [`by_priority_order`] ）。
///
/// not 1 Reason： by 「 before 」 —— empty 1 not will down 
/// empty， new 1 that 1 ， not 。
///
/// 、 not ： by use by return （0、1、2…）。return is ****
/// id（ 1 not ，）。
pub fn grouped_queue_order(entries: &[QueueEntry<'_>]) -> Vec<String> {
    let mut groups: Vec<(&str, Vec<&QueueEntry<'_>>)> = Vec::new();
    for entry in entries {
        match groups
            .iter_mut()
            .find(|(provider, _)| *provider == entry.provider)
        {
            Some((_, list)) => list.push(entry),
            None => groups.push((entry.provider, vec![entry])),
        }
    }
    for (_, list) in groups.iter_mut() {
        list.sort_by(|a, b| {
            by_priority_order((a.priority, a.added_at), (b.priority, b.added_at))
        });
    }
    // use 1 ：`sort_by` is ， and when 
    // （ use by ， and old 1 ）
    groups.sort_by(|(_, a), (_, b)| {
        by_priority_order(
            (a[0].priority, a[0].added_at),
            (b[0].priority, b[0].added_at),
        )
    });
    groups
        .into_iter()
        .flat_map(|(_, list)| list.into_iter().map(|entry| entry.id.to_string()))
        .collect()
}

/// already new ， this not 。
///
/// use has （ this is ， for and 「 N 」 not ）；
/// if from that not down （ to MAX_PRIORITY）， down to down ——
/// not can for `min(MAX, base + i)`， that will after 1 
/// ，「 1 」 this not 。
///
/// `records` as `(id, name, priority)` can ， by use 。
/// return（ not ）。
pub fn renumber_consecutively(records: &mut [(String, String, i64)]) -> Vec<PriorityAssignment> {
    if records.is_empty() {
        return Vec::new();
    }
    let base = records
        .iter()
        .map(|(_, _, priority)| normalize_priority_value(*priority))
        .min()
        .unwrap_or(DEFAULT_PRIORITY);
    let span = records.len() as i64 - 1;
    let start = if base + span <= MAX_PRIORITY {
        base
    } else {
        (MAX_PRIORITY - span).max(MIN_PRIORITY)
    };

    let mut assignments = Vec::new();
    for (index, (id, name, priority)) in records.iter_mut().enumerate() {
        let next = (start + index as i64).clamp(MIN_PRIORITY, MAX_PRIORITY);
        let from = normalize_priority_value(*priority);
        if from != next {
            *priority = next;
            assignments.push(PriorityAssignment {
                id: id.clone(),
                name: name.clone(),
                from,
                to: next,
            });
        }
    }
    assignments
}
