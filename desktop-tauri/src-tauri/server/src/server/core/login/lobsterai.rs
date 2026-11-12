//! LobsterAI （**loopback by use **）。
//!
//! ── this 1 ────────────────────────────────────────
//! and Accio / AutoClaw ：** use HTTP **
//! （`GET /auth/callback?code=…&state=…`）。 and in ****
//! （`agent2api-providers` `lobsterai/oauth.rs`）， this ：
//! this → to → 。
//!
//! ── and `submit_login_callback` ───────────────────────
//! that 「 URL 」 use （`POST
//! /api/session/login/callback`）， in that 1 
//! `exchange_login_code`。file is ****： to 
//! `/auth/callback`， by not to `ServerState` ，。
//! to `login.callback` RPC。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::{Value, json};

use crate::server::core::providers::{kind_id, ProviderKind};
use crate::server::errors::GatewayError;
use crate::server::logging;

use super::{LoginService, finish_task_error};

impl LoginService {
    /// ： → use → → 。
    ///
    /// ── （ and accio 1 ， has by ）───────────────
    /// in HTTP up ， can 1 GET，：
    /// 1. `state` must for 1 ** in and LobsterAI** ；
    /// 2. that 1 pending by （`login.callback` 
    /// `take_pending`）——「 can 1 」 in ；
    /// 3. already when returnsuccess： use new / will ，
    /// to 「 already 」， 1 success 1 failure。
    pub async fn finish_lobsterai_login(
        &self,
        code: &str,
        state: &str,
    ) -> Result<String, GatewayError> {
        let code = code.trim();
        let state = state.trim();
        if code.is_empty() {
            return Err(GatewayError::with_status(400, "回调没有携带授权码"));
        }
        if state.is_empty() {
            return Err(GatewayError::with_status(
                400,
                "回调没有携带 state，无法确认这次登录归属",
            ));
        }
        let Some(handle) = self.tasks.get(state) else {
            return Err(GatewayError::with_status(
                404,
                "这次登录已取消或已过期，请重新发起",
            ));
        };
        let provider = kind_id(ProviderKind::LobsterAi);
        if handle.snapshot().provider != provider {
            return Err(GatewayError::with_status(400, "这次登录不属于 LobsterAI"));
        }
        if handle.snapshot().done {
            // etc ： not is error（ 3）
            return Ok(String::new());
        }
        let Some(adapter) = crate::server::core::providers::adapter::adapter_by_id(provider)
        else {
            return Err(GatewayError::with_status(
                400,
                "LobsterAI 的提供商插件没有安装，无法完成登录",
            ));
        };
        match adapter.exchange_login_code(&self.store, code, state).await {
            Ok(account_id) => {
                handle.update(|task| {
                    task.done = true;
                    task.session = Some(json!({
                        "accountUid": account_id,
                        "nickname": Value::Null,
                        "provider": provider,
                    }));
                    task.finished_at = Some(logging::now_ms());
                });
                logging::log("[Login]", "✅ LobsterAI 网页登录完成，账号已加入列表");
                Ok(account_id)
            }
            Err(error) => {
                finish_task_error(&handle, &error.message);
                logging::log(
                    "[Login]",
                    &format!("❌ LobsterAI 网页登录换取凭证失败: {}", error.message),
                );
                Err(error)
            }
        }
    }
}
