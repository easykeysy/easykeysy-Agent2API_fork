use serde_json::{json, Value};

use super::{
    available_checkin_providers, beijing, checkin_schedules, date_key, default_providers,
    default_time, master_enabled, provider_label, retry, times, AutoCheckin, Schedule,
    DEFAULT_TIME,
};

impl AutoCheckin {
    // ─── for ───────────────────────────────────────────

    /// for （ for Node getState ）。
    ///
    /// ：
    /// - `schedules`：（ enabled / time / providers /
    ///     providerTimes / retryTime / lastFiredToday / nextRunAt / lastResult），
    /// to in 「 when to 」；
    /// - `time` / `providers` / `lastFiredDate` / `lastFiredToday` /
    /// `lastResult` / `nextRunAt`：****—— nextRunAt is allenabled
    /// 1 （ use ）， its default（ or 
    /// id="default"； to ）， old （） not use 。
    pub fn state(&self) -> Value {
        let schedules = self.planned_schedules();
        let master = master_enabled();
        let now = beijing::now();
        let today = date_key(now);
        let entries: Vec<Value> = schedules
            .iter()
            .map(|schedule| {
                // 「」 by ****：all
                // （ no has when to old lastFiredDate， times::main_fired）
                let pending = times::pending_providers(schedule, &today);
                let fired_today = pending.is_empty();
                let next_run_at = if master && schedule.enabled {
                    [if schedule.retry_gap_minutes > 0 { retry::next_at(schedule.last_result.as_ref(), &today, schedule.retry_limit) } else { None },
                        times::next_run_at(schedule, now, &today)].into_iter().flatten().min()
                } else {
                    None
                };
                // when ： up 1 need can to 「 this to 」
                // to no （「」）； no to is 「 etc 」，
                // not can 1 —— up 08:00 1 00:01 ，Qoder
                // （10:01） not is ， is in etc 。
                let provider_times: Vec<Value> = schedule
                    .providers
                    .iter()
                    .map(|id| {
                        let mut row = checkin_schedules::account_plan::row_state(schedule, id, now.timestamp_millis());
                        if let Some(fields) = row.as_object_mut() {
                            fields.extend(json!({
                                "id": id,
                                "label": provider_label(id),
                                "time": times::effective_time(schedule, id),
                                "timeEnd": times::effective_end(schedule, id),
                                "builtin": times::provider_time(id),
                                "firedToday": !pending.iter().any(|item| item == id),
                            }).as_object().cloned().unwrap_or_default());
                        }
                        row
                    })
                    .collect();
                json!({
                    "id": schedule.id,
                    "label": schedule.label,
                    // ：label as empty when to time， not 
                    "displayLabel": checkin_schedules::display_label(schedule),
                    "enabled": schedule.enabled,
                    "time": schedule.time,
                    "timeEnd": schedule.time_end,
                    "providers": schedule.providers,
                    "providerTimes": schedule.provider_times,
                    "providerRows": provider_times,
                    "retryTime": schedule.retry_time,
                    // when （ + etc ， not retryTime）
                    "providerRetryTimes": schedule.providers.iter()
                        .filter_map(|id| times::provider_retry_time(schedule, id)
                            .map(|time| (id.clone(), Value::from(time))))
                        .collect::<serde_json::Map<String, Value>>(),
                    "retryGapMinutes": schedule.retry_gap_minutes,
                    "retryLimit": schedule.retry_limit,
                    "lastFiredDate": schedule.last_fired_date,
                    "lastFiredToday": fired_today,
                    // no （「：Qoder 10:01」）
                    "pendingProviders": pending,
                    "nextRunAt": next_run_at,
                    // in 、 etc （ up need 「 will 」）
                    "retryCount": schedule.last_result.as_ref()
                        .filter(|_| schedule.retry_gap_minutes > 0 && schedule.retry_limit > 0)
                        .filter(|value| value.get("date").and_then(Value::as_str) == Some(today.as_str()))
                        .and_then(|value| value.get("retryAccounts"))
                        .and_then(Value::as_object)
                        .map(|map| map.len())
                        .unwrap_or(0),
                    // already use ： already not in `retryAccounts` ，
                    // not to ，`retryCount` 「 no 」，
                    // is 「 not 、」
                    "retryExhausted": retry::exhausted_accounts(
                        schedule.last_result.as_ref(),
                        schedule.retry_limit,
                    ),
                    "lastResult": schedule.last_result,
                })
            })
            .collect();
        // 「 down 」：enabled 1 （total no ）
        let next_run_at = if master {
            entries
                .iter()
                .filter_map(|entry| entry.get("nextRunAt").and_then(Value::as_i64))
                .min()
        } else {
            None
        };
        // 「default」： default → 1 → （）
        let fallback = Schedule {
            id: checkin_schedules::DEFAULT_SCHEDULE_ID.to_string(),
            label: String::new(),
            enabled: false,
            time: default_time(),
            time_end: times::default_time_end(DEFAULT_TIME),
            providers: default_providers(),
            provider_times: std::collections::BTreeMap::new(),
            retry_time: times::DEFAULT_RETRY_TIME.to_string(),
            retry_gap_minutes: 10,
            retry_limit: 3,
            last_fired_date: None,
            last_fired_providers: std::collections::BTreeMap::new(),
            last_retried_providers: std::collections::BTreeMap::new(),
            account_plan: Default::default(),
            last_result: None,
        };
        let default_entry = schedules
            .iter()
            .find(|schedule| schedule.id == checkin_schedules::DEFAULT_SCHEDULE_ID)
            .or_else(|| schedules.first())
            .unwrap_or(&fallback);
        json!({
            "enabled": master,
            // （default ； and before 1 ）
            "time": default_entry.time,
            "timeEnd": default_entry.time_end,
            "providers": default_entry.providers,
            "lastFiredDate": default_entry.last_fired_date,
            "lastFiredToday":
                times::pending_providers(default_entry, &today).is_empty(),
            "lastResult": default_entry.last_result,
            "nextRunAt": next_run_at,
            // （ to ），（ in ）。
            // use **available** not is `checkin_providers()`： no when that 17 
            // is failure （2026-10 use ）。 already 
            // not —— is id ， this 「 can 」。
            "providerOptions": available_checkin_providers(&self.store).iter().map(|id| {
                json!({
                    "id": id,
                    "label": provider_label(id),
                    // when （None = when ）， in 
                    "builtinTime": times::provider_time(id),
                })
            }).collect::<Vec<_>>(),
            // when ：「 when 」 placeholder use 
            "defaultRetryTime": times::DEFAULT_RETRY_TIME,
            "schedules": entries,
            "running": self.is_running(),
        })
    }
}
