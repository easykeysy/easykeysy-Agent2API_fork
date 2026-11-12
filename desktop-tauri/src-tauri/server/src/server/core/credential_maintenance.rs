//! ： 「expired / 」 1 。
//!
//! ── this ─────────────────────────────────────
//! has new （ `ensure_access_token` in when ），
//! ** not **。 is 「 no request when 」 this 1 ： up 「valid」
//! 1 expired， use as —— is no 。
//! is that 「 1 down 」 ： → need not need → 。
//!
//! ── Why in not is this ─────────────────────────
//! 「expiration time in 、」 is ** **：workbuddy expiration time in 
//! will `auth.expiresAt`、 in JWT `exp`（ `tokenExpiresAt`）、
//! AutoClaw in `expiresAt`， not （5 minutes / 5 minutes /
//! 5 minutes）。 by 「 `expiresAt`」 this 1 ， is 
//! （ not ）** none need new ** —— not ， and up not Reason。
//! in this `ProviderAdapter::credentials_expiring`， 1 provider
//! not ， not will 「 for not up 」 。
//!
//! ── Why new （ need ）────────────────────────────────
//! `supports_refresh()` and `credentials_expiring()` ，
//! `refresh_access_token`。 by ： that for 「 no has 」 will return Err
//! （ this is ， `api::accounts::refresh_account` use ）——
//! ， will CatPaw（ no has new ）、 no has refreshToken 
//! failure， 1 ， need need failure 。
//!
//! ── Whyfailure not up （HTTP 2xx）──────────────────────
//! this is 1 ****，「， its in no up 」 is 。
//! failure 2xx， will use （ after 、 will ）
//! musterrorresponse can 「 to 」， need is that 
//! 。failure in `results[i].status = "failed"` ， by 
//! 。 will has 「 request not 」 that 1 （ by HTTP client）。
//!
//! ── and panic ─────────────────────────────────
//! 1. ** not await**：， and new 
//! in await；
//! 2. ** not unwrap/expect**： in after and HTTP handler path up ，
//! release is `panic = "abort"`， 1 panic will use 。
//!
//! and `auto_checkin.rs` ：「 when 」， in this ，
//! use （ + ） in `ServerState::bootstrap`。

use serde_json::{json, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::core::providers::adapter::ProviderAdapter;
use crate::server::errors::GatewayError;
use crate::server::logging;

/// （** is to before and **）
pub const STATUS_REFRESHED: &str = "refreshed";
/// not need need new ， or this unsupported new 
pub const STATUS_SKIPPED: &str = "skipped";
/// new failure（ can `message`）
pub const STATUS_FAILED: &str = "failed";

/// **default**：10 minutes。
///
/// Whydefault is 10 minutes（ not is ）： is 2~5 minutes，10 minutes 
/// 「 after etc 1 」—— for 「 for not for 」 this 
/// 。， is empty（ no has when ），
/// is empty， not will ； then will up 
/// 「expired」。
///
/// ── is can ， not has ────────────────────────────
/// by `core::scheduled_tasks` ， config.json 
/// `scheduledTasks.credentialMaintenance.interval`；default（10 minutes） in 
/// `config::DEFAULT_CREDENTIAL_MAINTENANCE_MINUTES`， in when can 。
/// not —— 1 no will as valid。

/// ， 「 this supports new and expired / 」 that new 1 。
///
/// return****：`[{ id, provider, status, message? }, ...]`， and 
/// file 1 （）。`status` 1 ；`message` in 
/// `skipped`（Why） and `failed`（ up / failureReason） when 。
///
/// ── ─────────────────────────────────────────
/// ** has 1 **（ use and unsupported new ）， this total 
/// success / / failure can for up total， not 。
/// is return to —— this is 127.0.0.1 up 1 use ，
/// 「 need new 」 this ， can ignore。
///
/// ── failure ────────────────────────────────────────────────
/// new failure 1 failure and 1 ，** not its **：
/// 1 refreshToken not this 。
///
/// ── return not ────────────────────────────────────────
/// `id` / `provider` / / ， not token 。`GatewayError` 
/// by （ not ）， and `/api/accounts/refresh` to before is 
/// 1 。
/// 1 new and ** for is **（issue #407）。
///
/// return Ok 「 this use no 」： can can no to （ up 
/// old 、failure）， 1 no 。 this in new before after 1 
/// （`expiresAt` / `updatedAt`）， no error —— is 
/// ❌、 new 2xx， 1 will 。
///
/// `expiresAt` or `updatedAt` ** 1 **success： has new after token
/// not valid（ has ）， 1 will 。 not to when ——
/// that is 「 not in 」 ， by new ， not this in this 1 。
pub async fn refresh_checked(
    store: &AccountStore,
    adapter: &dyn ProviderAdapter,
    id: &str,
) -> Result<String, GatewayError> {
    let before = store.credential_stamp(id);
    let token = adapter.refresh_access_token(store, id).await?;
    let after = store.credential_stamp(id);
    if let (Some((expires_before, updated_before)), Some((expires_after, updated_after))) =
        (before, after)
    {
        if expires_after <= expires_before && updated_after <= updated_before {
            return Err(GatewayError::with_status(
                502,
                format!(
                    "账号 {id} 的凭证刷新未产生变化（适配器报成功但凭据未更新），请重新登录该账号"
                ),
            ));
        }
    }
    Ok(token)
}

pub async fn refresh_expiring_accounts(store: &AccountStore) -> Vec<Value> {
    let snapshot = store.list_accounts();
    let accounts: Vec<Value> = snapshot
        .get("accounts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    // 1 ：「 new 」 ， its skipped 。
    // RPC ， down that 1 new 。
    //
    // by ** down **（`slots[i]`）， new that 1 ——
    // is return and 1 ， not is 「 1 、 after is new 」。
    // up for when ， 1 1 by id 。
    let mut slots: Vec<Option<Value>> = vec![None; accounts.len()];
    // new （ not is `ProviderKind`）： no has ，
    // this already 1 ， 1 is 
    let mut pending: Vec<(usize, String, String, std::sync::Arc<dyn crate::server::core::providers::adapter::ProviderAdapter>)> =
        Vec::new();
    for (index, account) in accounts.iter().enumerate() {
        let id = account
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if id.is_empty() {
            continue;
        }
        let provider = account
            .get("provider")
            .and_then(Value::as_str)
            .unwrap_or("");
        if store.needs_relogin(&id) {
            slots[index] = Some(skipped(&id, provider, "登录授权已失效，需要重新登录该账号"));
            continue;
        }
        // has ： this supports new + this 。
        // ** not by enabled **： use is 「 use 」， not is 「」——
        // for use valid， will 1 「expired」，
        // is need that ； new not enabled， no has use 。
        // provider id（file ） no has can ，
        // ── use `adapter_by_id` not is `kind_from_id` ────────────────
        // id not in ，** has can **（`supports_refresh`
        // by ）。 will not —— is 
        // use to 「expired」 not 。
        let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider) else {
            slots[index] = Some(skipped(&id, provider, "未注册的提供商"));
            continue;
        };
        if !adapter.supports_refresh() {
            slots[index] = Some(skipped(
                &id,
                provider,
                "该提供商不支持主动刷新凭证（没有续期手段）",
            ));
            continue;
        }
        // not （ no has / not in / no has refreshToken） 1 false
        if !adapter.credentials_expiring_async(store, &id).await {
            slots[index] = Some(skipped(&id, provider, "凭证未过期或临期"));
            continue;
        }
        pending.push((index, id, provider.to_string(), adapter));
    }

    if pending.is_empty() {
        // to this （ no has ）： 1 verbose，
        // not —— 10 minutes 1 「 no 」 will 
        logging::verbose(
            "[Accounts]",
            &format!("凭证自动维护：无需刷新的账号（共 {} 个）", accounts.len()),
        );
        return slots.into_iter().flatten().collect();
    }

    logging::log(
        "[Accounts]",
        &format!("检测到 {} 个临期账号，开始自动刷新凭证…", pending.len()),
    );
    for (index, id, provider, adapter) in pending {
        if store.needs_relogin(&id) {
            slots[index] = Some(skipped(&id, &provider, "登录授权已失效，需要重新登录该账号"));
            continue;
        }
        // not is ： this already is 1 new 
        // （ / AutoClaw will by -，workbuddy 
        // `AuthService::refresh_account`）， this not need need 1 。
        //
        // await（ not and ）： new is 、 need 1 up ，
        // ；failure match 1 ，
        // not will in （module header failure）。
        slots[index] = Some(match refresh_checked(store, adapter.as_ref(), &id).await {
            Ok(_) => {
                // success 1 ： use in can to 「 new 」，
                // otherwise「token new 」 is none 
                logging::log("[Accounts]", &format!("✅ 已自动刷新账号 {id} 的凭证"));
                json!({
                    "id": id,
                    "provider": provider,
                    "status": STATUS_REFRESHED,
                })
            }
            Err(error) => {
                logging::log(
                    "[Accounts]",
                    &format!("❌ 账号 {id} 凭证自动刷新失败: {}", error.message),
                );
                json!({
                    "id": id,
                    "provider": provider,
                    "status": STATUS_FAILED,
                    "message": error.message,
                })
            }
        });
    }

    let results: Vec<Value> = slots.into_iter().flatten().collect();
    let (refreshed, skipped_count, failed) = summarize(&results);
    // total：「failure 0 」 is ， not can 「failure」 error
    logging::log_with_level(
        "[Accounts]",
        &format!("凭证自动维护完成：成功 {refreshed}，跳过 {skipped_count}，失败 {failed}"),
        if failed > 0 { "error" } else { "info" },
    );
    results
}

/// **response**：`{ results, refreshed, skipped, failed }`。
///
/// total and 1 to ， can 1 （「 already new N 」） can 
/// Why / failure。 by ， by ——
/// handler to 1 ，`api/accounts.rs` not 1 JSON。
pub async fn refresh_expiring_report(store: &AccountStore) -> Value {
    let results = refresh_expiring_accounts(store).await;
    let (refreshed, skipped_count, failed) = summarize(&results);
    json!({
        "results": results,
        "refreshed": refreshed,
        "skipped": skipped_count,
        "failed": failed,
    })
}

/// statistics ：`(refreshed, skipped, failed)`。
///
/// by total use （）—— statistics 1 ，
/// 「success 2、responsesuccess 3」 this for not up 。
pub fn summarize(results: &[Value]) -> (usize, usize, usize) {
    let count = |status: &str| {
        results
            .iter()
            .filter(|item| item.get("status").and_then(Value::as_str) == Some(status))
            .count()
    };
    (
        count(STATUS_REFRESHED),
        count(STATUS_SKIPPED),
        count(STATUS_FAILED),
    )
}

/// 1 `skipped` （`reason` to use ，Why no ）
fn skipped(id: &str, provider: &str, reason: &str) -> Value {
    json!({
        "id": id,
        "provider": provider,
        "status": STATUS_SKIPPED,
        "message": reason,
    })
}

/// after （`POST /api/accounts/refresh-expiring` use ）。
///
/// and 1 ： already 1 up ， when that after 1 ，
/// 「、 to or after 1 」。failure， not 
/// —— use not error（`task_state` already ）。
pub fn note_manual_run() {
    let interval = crate::server::config::scheduled_settings()
        .credential_maintenance
        .interval
        * 60_000;
    if let Err(error) = crate::server::core::task_state::note_external_run(
        crate::server::config::KEY_CREDENTIAL_MAINTENANCE,
        interval,
    ) {
        logging::verbose("[Accounts]", &format!("凭证维护排期顺延失败：{error}"));
    }
}
