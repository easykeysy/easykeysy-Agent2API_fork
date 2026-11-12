use serde_json::{json, Value};

use super::{
    beijing, checkin, checkin_schedules, default_providers, master_enabled, retry, times,
    today_key, write_state, AutoCheckin, RunningGuard,
};
use crate::server::logging;

impl AutoCheckin {
    // ─── ───────────────────────────────────────────────

    /// 1 to and ；failure， not down 1 。
    ///
    /// return None （Node ）：① already has 1 in （）；
    /// ② 。 by ① 「 to in in ，」。
    ///
    /// （`run_now`，none） this ： is ****——
    /// after not has 「」 can ， that when 。
    pub async fn fire(&self, reason: &str) -> Option<Value> {
        self.fire_for(reason, None, "manual").await
    }

    /// 1 to ， can to when 。
    ///
    /// `schedule_id` to Some： use ** that **，
    /// （`checkin_schedules::mark_fired` / `mark_result`）； to None
    /// （）： is ，config。
    /// `source` is to （schedule / makeup / manual /
    /// startup， `core::checkin_records` module header）。
    pub async fn fire_for(
        &self,
        reason: &str,
        schedule_id: Option<&str>,
        source: &str,
    ) -> Option<Value> {
        self.fire_scoped(reason, schedule_id, source, None, None)
            .await
    }

    /// [`fire_for`] ：`only` ，`retry_round` 。
    /// ， this 、 and 。
    pub async fn fire_scoped(
        &self,
        reason: &str,
        schedule_id: Option<&str>,
        source: &str,
        only: Option<&[String]>,
        retry_round: Option<&str>,
    ) -> Option<Value> {
        // 「 in 」； not to up 1 in 
        let guard = {
            let mut inner = self.lock();
            if inner.running {
                drop(inner);
                logging::verbose("[Checkin]", "上一次签到仍在执行，本次跳过");
                return None;
            }
            inner.running = true;
            RunningGuard {
                inner: self.inner.clone(),
            }
        };

        let today = today_key();
        let schedule = schedule_id.and_then(|id| {
            self.planned_schedules()
                .into_iter()
                .find(|schedule| schedule.id == id)
        });
        if schedule_id.is_some() && schedule.is_none() {
            return None;
        }
        // use ， use 。
        let mut providers = schedule
            .as_ref()
            .map(|schedule| schedule.providers.clone())
            .unwrap_or_else(default_providers);
        if let Some(only) = only {
            providers.retain(|id| only.iter().any(|picked| picked == id));
        }
        if retry_round.is_some() && matches!(source, "schedule" | "makeup" | "startup") {
            let current = schedule.as_ref()?;
            if !master_enabled() || !current.enabled {
                return None;
            }
            let due = times::retry_due_providers(current, beijing::now(), &today);
            providers.retain(|provider| due.contains(provider));
        }
        if providers.is_empty() {
            return None;
        }
        let fired_scope = providers.clone();
        // ， to when ； not limit。
        // has retryLimit，、 and not 。
        let window_retry =
            retry_round.is_none() && only.is_none() && matches!(source, "schedule" | "startup");
        if window_retry
            && (!master_enabled() || !schedule.as_ref().is_some_and(|schedule| schedule.enabled))
        {
            return None;
        }
        let (mut selected, round) = if retry_round.is_some() {
            (
                Some(retry::unsigned_accounts(&self.store)),
                retry::Round::Other,
            )
        } else if window_retry {
            (
                retry::due_accounts_scoped(
                    schedule
                        .as_ref()
                        .and_then(|schedule| schedule.last_result.as_ref()),
                    logging::now_ms(),
                    retry::gap_ms_of(schedule.as_ref()),
                    retry::limit_of(schedule.as_ref()),
                ),
                retry::Round::Window,
            )
        } else {
            (None, retry::Round::Other)
        };
        if window_retry {
            let selected = selected.as_mut()?;
            selected.retain(|id| {
                self.store.account_provider_of(id).is_some_and(|provider| {
                    providers.contains(&provider)
                        && crate::server::core::plugin_host::serves_provider(&provider)
                })
            });
        }
        if window_retry && selected.is_none() {
            return None;
        }
        if let (Some(id), Some(_)) = (schedule_id, retry_round) {
            checkin_schedules::mark_retried_providers(id, &fired_scope, &today);
        }
        if selected.as_ref().is_some_and(Vec::is_empty) {
            // no has can ：（ this 1 ）， 30 seconds
            // 1 
            if let (Some(_), Some(_)) = (schedule_id, retry_round) {
                logging::log(
                    "[Checkin]",
                    &format!("补签轮无需执行（今天都已签到）：{reason}"),
                );
            }
            drop(guard);
            return None;
        }
        if let Some(id) = schedule_id {
            let selected_targets = checkin::resolve_checkin_targets(&self.store, &providers, None)
                .ok()
                .map(|(targets, _)| {
                    targets
                        .into_iter()
                        .filter_map(|account| {
                            let account_id = account.get("id").and_then(Value::as_str)?;
                            selected
                                .as_ref()
                                .map_or(true, |selected| selected.iter().any(|id| id == account_id))
                                .then(|| account_id.to_string())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            checkin_schedules::account_plan::mark_attempted(id, &selected_targets, &today);
        }
        // ： to in ， not will in after 
        write_state(json!({ "lastFiredDate": today }));
        if let Some(id) = schedule_id {
            checkin_schedules::mark_fired(id, Some(&today));
        }
        logging::log("[Checkin]", &format!("⏰ 定时签到开始（{reason}）"));

        let outcome = match checkin::run_checkin_selected(
            &self.store,
            &self.billing,
            providers.as_slice(),
            None,
            reason,
            source,
            selected.as_deref(),
        )
        .await
        {
            Ok(result) => {
                let summary = retry::merge(
                    &self.store,
                    self.record_success(&result, &today, reason),
                    &result,
                    schedule
                        .as_ref()
                        .and_then(|schedule| schedule.last_result.as_ref()),
                    selected.as_deref(),
                    retry::RetryConfig::of(schedule.as_ref(), round),
                );
                write_state(json!({ "lastResult": summary }));
                if let Some(id) = schedule_id {
                    checkin_schedules::account_plan::mark_result(id, &result, &today);
                    checkin_schedules::mark_result(id, Some(summary.clone()));
                }
                Some(summary)
            }
            Err(error) => {
                // failure lastResult（failed error）， and Node catch 1 
                let failure = json!({
                    "at": logging::now_ms(),
                    "date": today,
                    "reason": reason,
                    "succeeded": 0,
                    "active": 0,
                    "total": 0,
                    "skipped": 0,
                    "failed": [error.message.clone()],
                    "failedCount": 1,
                });
                write_state(json!({ "lastResult": failure.clone() }));
                if let Some(id) = schedule_id {
                    checkin_schedules::mark_result(id, Some(failure));
                }
                logging::log("[Checkin]", &format!("❌ 定时签到失败: {}", error.message));
                None
            }
        };
        // ：successfailure（failure not —— by `retryAccounts` that 
        // by ， not 「 1 」；otherwise 1 up will N ）
        if let Some(id) = schedule_id {
            checkin_schedules::account_plan::finish_providers(id, &fired_scope, &today);
        }
        drop(guard);
        outcome
    }

    /// success：total + lastResult + （ for Node fire try ）
    pub(super) fn record_success(&self, result: &Value, today: &str, reason: &str) -> Value {
        let number = |key: &str| result.get(key).and_then(Value::as_u64).unwrap_or(0);
        let succeeded = number("succeeded");
        let active = number("active");
        let total = number("total");
        let skipped = number("skipped");
        // failure：`（error）`， when to id， to 「unknown」
        let failures: Vec<String> = result
            .get("results")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(checkin::failure_of).collect())
            .unwrap_or_default();
        let summary = json!({
            "at": logging::now_ms(),
            "date": today,
            "reason": reason,
            "succeeded": succeeded,
            "active": active,
            "total": total,
            "skipped": skipped,
            // before 5 ： use ， config.json （Node ）
            "failed": failures.iter().take(5).cloned().collect::<Vec<_>>(),
            "failedCount": failures.len(),
        });
        write_state(json!({ "lastResult": summary }));
        logging::log(
            "[Checkin]",
            &format!(
                "定时签到完成: {succeeded}/{total} 个账号成功领取{}{}{}",
                if skipped > 0 {
                    format!("，跳过 {skipped} 个")
                } else {
                    String::new()
                },
                if failures.is_empty() {
                    String::new()
                } else {
                    format!("，失败 {} 个", failures.len())
                },
                if active > 0 {
                    format!("，活跃保活 {active} 个")
                } else {
                    String::new()
                },
            ),
        );
        summary
    }
}
