//! new （ / ）， by 。
//!
//! ```text
//! GET /api/accounts/{id}/onboarding （）
//! POST /api/accounts/{id}/onboarding/claim 1 
//! ```
//!
//! ── Why is GET、 is POST ───────────────────────────
//! and 「 + 」 after by has 1 ： GET（ can 、 can 、
//! none use ）， POST（`onboarding/claim` not `/onboarding` ，
//! after not ，none）。 in `api::accounts::dispatch`
//! GET after and POST after 1 。
//!
//! ── by ────────────────────────────────────────────
//! all already to ， 1 down `plugin_status` /
//! `plugin_claim` ：（ —— / 
//! 1 ）、CodeArts（ ** new **）、OfficeAce（ 
//! 「 new 」 and daily to is 1 up ）、（ in `newbie`
//! ， click + complete）。
//!
//! this ****： 1 provider， request to this 。
//! no has 1 or up path —— that in 。
//! is provider ； not to or 1 Loomy ——
//! and has as 1 （`loomy_account_record` will for id 404， not ）。
//!
//! "" need ： after ** when to **（`auto_checkin`） not codearts，
//! up to after ** new ** —— and Loomy / ，
//! and as up after not `claimable`， that pathtotal will 1 request。
//! in `ui-islands/src/islands/checkin-state.ts`。
//!
//! ── and to （Why not in checkin ）──────────────
//! to is daily and and when to use 1 （`core::billing::checkin`），
//! 1 new ， will after 1 when to 。
//! in to after 1 、 has （ ui-islands 
//! checkin-state）， not 。
//!
//! ── 1 ─────────────────────────────────────
//! is ** 1 **： no 。 after up 1 
//! （`onboardingSettled`； is `onboardingGrants` ），
//! after and ** up request**（ `providers::onboarding_memory`）。
//! 「」 by `?refresh=1` —— that is 1 " new up "
//! ， is / path。
//!
//! response（ for ）：
//!   · status  → `{tasks:[{key,title,group,points,done,blocked?}], earned, total,
//!                 unclaimed, settled}`
//!   · claim   → `{results:[{key,ok,already?,error?}], claimed, failed,
//!                 claimedPoints, tasks, earned, total, unclaimed, settled}`

use serde_json::Value;

use axum::response::Response;

use crate::server::core::account_store::AccountStore;
use crate::server::errors::management_error;
use crate::server::http::ok_json;
use crate::server::ServerState;

/// from 1 provider id（ not to to empty —— will to 
/// Loomy ， by `loomy_account_record` 404， and has as 1 ）。
fn provider_of_account(state: &ServerState, account_id: &str) -> String {
    state
        .store()
        .list_accounts()
        .get("accounts")
        .and_then(Value::as_array)
        .and_then(|accounts| {
            accounts
                .iter()
                .find(|account| account.get("id").and_then(Value::as_str) == Some(account_id))
                .and_then(|account| account.get("provider").and_then(Value::as_str))
        })
        .unwrap_or("")
        .to_string()
}

/// 。
///
/// ── Why ────────────────────────────────────────────
/// （`status` and `claim`） need in 「 this provider by 」 when this ，
/// not （`status` in provider as empty when Loomy 404）。 1 
/// ， is as 「 has 1 」 this in up 。
async fn plugin_status(
    state: &ServerState,
    provider: &str,
    account_id: &str,
    refresh: bool,
) -> Result<Value, crate::server::errors::GatewayError> {
    match crate::server::core::providers::adapter::adapter_by_id(provider) {
        Some(adapter) => {
            adapter
                .claim_status(state.store(), account_id, refresh)
                .await
        }
        None => Err(crate::server::errors::GatewayError::with_status(
            400,
            format!("未知的提供商：{provider}"),
        )),
    }
}

/// （`claim` ， by up ）。
async fn plugin_claim(
    state: &ServerState,
    provider: &str,
    account_id: &str,
) -> Result<Value, crate::server::errors::GatewayError> {
    match crate::server::core::providers::adapter::adapter_by_id(provider) {
        Some(adapter) => adapter.claim(state.store(), account_id, "").await,
        None => Err(crate::server::errors::GatewayError::with_status(
            400,
            format!("未知的提供商：{provider}"),
        )),
    }
}

/// already ； not use ， as 
/// `{at, tasks, earned, total}`。 by `plugin_ledger` 。
pub fn settled_view(store: &AccountStore, account_id: &str) -> Option<Value> {
    let snapshot = store.plugin_ledger(account_id, "onboarding");
    snapshot.is_object().then_some(snapshot)
}

/// `GET /api/accounts/{id}/onboarding` —— 。
///
/// `refresh`（`?refresh=1`） up ： in use 「」 when ，
/// its （、 to after ）。
pub async fn status(state: &ServerState, account_id: &str, refresh: bool) -> Response {
    let provider = provider_of_account(state, account_id);
    let result = plugin_status(state, &provider, account_id, refresh).await;
    match result {
        Ok(data) => ok_json(data),
        Err(error) => management_error(error.status_code, error.message),
    }
}

/// `POST /api/accounts/{id}/onboarding/claim` —— 。
/// etc ， use not will 。
pub async fn claim(state: &ServerState, account_id: &str) -> Response {
    let provider = provider_of_account(state, account_id);
    let result = plugin_claim(state, &provider, account_id).await;
    match result {
        Ok(data) => ok_json(data),
        Err(error) => management_error(error.status_code, error.message),
    }
}
