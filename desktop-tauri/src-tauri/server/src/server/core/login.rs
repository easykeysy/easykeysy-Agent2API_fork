//! none（device-flow ） and 。
//!
//! ── in this ────────────────────────────────────────
//! ** 1 not **： already to ， this 
//! ****（`/wait` pending/done/error 、、10 minutes） and 
//! use （`submit_login_callback` → `login.callback`）。
//! 1 `api::session` `start_plugin_web_login`（ / 
//! use ）。
//!
//! ── and ──────────────────────────────────────────
//! is 1 `LoginTask`（`Arc<Mutex<...>>`），`/start` to after etc 
//! 15 seconds authUrl， to by state `/wait` ； not to 
//! 502（ in after ， and Node 1 ）。
//!
//! not AbortController is up `canceled` —— 1 
//! check 1 ， etc （Node signal.aborted is check）。
//! after 10 minutes， in 「 when check」，
//! not after when 。

mod lobsterai;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{json, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::core::endpoints::DEFAULT_EDITION;
use crate::server::core::providers::adapter::{adapter_for, ProviderAdapter};
use crate::server::core::providers::{kind_from_id, kind_id, ProviderKind};
use crate::server::errors::GatewayError;
use crate::server::logging;

/// total when （ SIGN_IN_PENDING_TIMEOUT； use ）
pub const LOGIN_TIMEOUT_MS: u64 = 5 * 60 * 1000;
/// `/start` etc authUrl limit（ for server.mjs `Date.now() + 15000`）
pub const AUTH_URL_WAIT_MS: u64 = 15_000;
/// after in 10 minutes（Node setTimeout ）
const TASK_RETENTION_MS: i64 = 10 * 60 * 1000;

/// 1 （ for Node `loginTasks` task for ）。
#[derive(Clone, Debug, Default)]
pub struct LoginTaskState {
    pub state: Option<String>,
    pub auth_url: Option<String>,
    /// and ； not use inputs or 。
    pub instructions: Option<String>,
    pub hint: Option<String>,
    /// already has after when ， Pending not can consumer。
    pub polling: bool,
    /// has ， old AutoClaw use not this 。
    pub method_index: Option<u64>,
    pub done: bool,
    pub error: Option<String>,
    /// success after will need `{ accountUid, nickname, edition }`
    pub session: Option<Value>,
    pub edition: String,
    /// this 1 （provider id）。
    ///
    /// Whymust in up ：`/wait` by state ，**** that 1 
    /// （`submit_login_callback`）must this `exchange_login_code`。
    /// is ， not can use 1 provider —— that etc client
    /// 「 use this state」， 1 。
    ///
    /// （default） as empty，`new_handle_for_provider` will up 1 ；
    /// empty in by 「 not 」， not will to 1 。
    pub provider: String,
    pub canceled: bool,
    finished_at: Option<i64>,
}

impl LoginTaskState {
    /// `/api/session/login/wait` response：
    /// `{pending:true}` / `{done:true,error}` / `{done:true,session}`
    pub fn to_wait_response(&self) -> Value {
        if !self.done {
            let mut response = json!({ "pending": true });
            if let Some(instructions) = &self.instructions { response["instructions"] = json!(instructions); }
            if let Some(hint) = &self.hint { response["hint"] = json!(hint); }
            return response;
        }
        if let Some(error) = &self.error {
            return json!({ "done": true, "error": error });
        }
        json!({ "done": true, "session": self.session.clone().unwrap_or(Value::Null) })
    }
}

/// in has up down ， not 、response or 。
/// Some + source:direct ；None old no has 。
pub(crate) struct LoginProxySelection {
    pub(crate) config: Value,
    pub(crate) resolved: Value,
}

struct LoginTaskContext {
    /// flow and its owner；/ after not by id new 。
    adapter: Mutex<Option<Arc<dyn ProviderAdapter>>>,
    provider: &'static str,
    store: AccountStore,
    proxy: Option<LoginProxySelection>,
}

/// ： and `/start`、`/wait`、`/cancel` 1 。
#[derive(Clone)]
pub struct LoginTaskHandle {
    inner: Arc<Mutex<LoginTaskState>>,
    /// only when ， old has 。
    context: Option<Arc<LoginTaskContext>>,
    /// and after 1 provider 。
    /// callback and poll 1 / 1 。
    pub(crate) operation: Arc<tokio::sync::Mutex<()>>,
    /// 1 （ by state ， to state before use and ）
    ticket: u64,
}

/// after 。
///
/// `ContinueTo` to 「 1 not 、 need 1 」 （ not can 
/// public callback by ， down 1 use ）。
/// before no has this ， to use 。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoginCallbackSubmission {
    Completed(String),
    /// to ， need by login.poll::Done 。
    Accepted,
    ContinueTo(String),
}

impl LoginTaskHandle {
    fn lock(&self) -> MutexGuard<'_, LoginTaskState> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub fn snapshot(&self) -> LoginTaskState {
        self.lock().clone()
    }

    pub(crate) fn update(&self, action: impl FnOnce(&mut LoginTaskState)) {
        let mut guard = self.lock();
        action(&mut guard);
    }

    pub fn ticket(&self) -> u64 {
        self.ticket
    }

    pub(crate) fn adapter(&self) -> Option<Arc<dyn ProviderAdapter>> {
        self.context.as_ref().and_then(|context| {
            context.adapter.lock().unwrap_or_else(|error| error.into_inner()).clone()
        })
    }

    /// not await； and poll/callback use 1 。
    pub(crate) async fn cancel_provider_flow(&self, flow: &str) {
        let _operation = self.operation.lock().await;
        self.cancel_provider_flow_locked(flow).await;
    }

    pub(crate) async fn cancel_provider_flow_locked(&self, flow: &str) {
        let Some(adapter) = self.adapter() else { return };
        if let Err(error) = adapter.login_cancel_async(flow).await {
            if error.status_code != 404 {
                // 501 need can ： not can already flow。
                // not error： can can in 。
                logging::log("[Login]", &format!("{} 的登录流程清理未确认，请更新提供商或重新发起登录", adapter.provider_id()));
            }
        }
        if let Some(context) = &self.context {
            context.adapter.lock().unwrap_or_else(|error| error.into_inner()).take();
        }
    }

    /// config use has ； old secret.proxy 。
    /// success Done；failure not or secret 。
    fn persist_proxy(&self, account_id: &str) -> Result<(), GatewayError> {
        let Some(context) = &self.context else { return Ok(()) };
        let Some(proxy) = &context.proxy else { return Ok(()) };
        let builtin = crate::server::core::providers::availability::BUILTIN_SUBSCRIPTION_IDS
            .contains(&context.provider);
        // secret is ， not （otherwise identityStamp/CAS will ）。
        // secret.proxy ；config and 1 。
        let saved = context.store.with_lock(|guard| {
            context.store.with_conn_mut(guard, |conn| {
                let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
                let Some(mut record) = crate::server::core::account_store::sql::load_by_id(&tx, account_id)? else {
                    return Ok(Err(GatewayError::with_status(500, "登录账号不存在，未能保存登录代理")));
                };
                if record.provider() != context.provider {
                    return Ok(Err(GatewayError::with_status(500, "登录账号归属不匹配，未能保存登录代理")));
                }
                if !builtin {
                    let mut secret = record.get("pluginSecret").cloned().unwrap_or(Value::Null);
                    let Some(fields) = secret.as_object_mut() else {
                        return Ok(Err(GatewayError::with_status(500, "登录账号凭证不支持保存代理出口，请更新提供商后重试")));
                    };
                    fields.insert("proxy".to_string(), proxy.resolved.clone());
                    record.set("pluginSecret", secret);
                }
                record.set_proxy(proxy.config.clone());
                record.set_updated_at(logging::now_ms());
                crate::server::core::account_store::sql::update_in_place(&tx, &record)?;
                tx.commit()?;
                Ok(Ok(()))
            })
        }).map_err(|error| GatewayError::with_status(error.status_code, "保存登录代理配置与出口失败，请重新发起"))?;
        saved
    }
}

/// ：`state → `。
///
/// 「 already to state」 —— and Node 1 （ in onAuthUrl 
/// `loginTasks.set(state, task)`）， not to authUrl failure not will 
/// `/wait` to ， before will to 404「 not in or expired」。
#[derive(Clone)]
pub struct LoginTasks {
    inner: Arc<Mutex<TaskTable>>,
}

struct TaskTable {
    by_state: HashMap<String, LoginTaskHandle>,
    next_ticket: u64,
}

impl LoginTasks {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TaskTable { by_state: HashMap::new(), next_ticket: 1 })),
        }
    }

    fn lock(&self) -> MutexGuard<'_, TaskTable> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// （ after 10 minutes）。 when ， not after when ——
    /// after not will has /wait ， when for is ，
    /// down 1 will 。
    fn sweep(table: &mut TaskTable) {
        let now = logging::now_ms();
        table.by_state.retain(|_, handle| match handle.lock().finished_at {
            Some(finished) => now - finished < TASK_RETENTION_MS,
            None => true,
        });
    }

    pub fn get(&self, state: &str) -> Option<LoginTaskHandle> {
        if state.is_empty() {
            return None;
        }
        let mut table = self.lock();
        Self::sweep(&mut table);
        table.by_state.get(state).cloned()
    }

    /// （`/start` to state after use ）
    pub fn register(&self, state: &str, handle: LoginTaskHandle) -> bool {
        if state.is_empty() {
            return false;
        }
        let mut table = self.lock();
        Self::sweep(&mut table);
        table.by_state.insert(state.to_string(), handle);
        true
    }

    /// ： canceled （ down 1 ） and ** from **。
    ///
    /// is ：Node `cancelLoginTask` `loginTasks.delete(state)`，
    /// is before that `/wait` will to 404「 not in or expired」——
    /// login.rs is this 「 use already 」， etc 
    /// （ its `error.contains(" not in ")` ）。 will 
    /// that /wait to 1 error ，「error」 。
    ///
    /// return is 。 already / not in return false。
    pub fn cancel(&self, state: &str) -> bool {
        let Some(handle) = self.get(state) else {
            return false;
        };
        {
            let mut task = handle.lock();
            if task.done {
                return false;
            }
            task.canceled = true;
            task.done = true;
            task.session = None;
            task.instructions = None;
            task.hint = None;
            task.polling = false;
            task.auth_url = None;
            task.error = Some("登录已取消".to_string());
            task.finished_at = Some(logging::now_ms());
        }
        {
            let mut table = self.lock();
            // by state or by （state when use ticket ， its 1 ）
            let ticket = handle.ticket();
            table
                .by_state
                .retain(|_, item| item.ticket() != ticket);
        }
        logging::log("[Login]", "登录任务已取消（用户放弃等待）");
        true
    }

    /// 1 （`/start` and use ，onlyused for）
    fn next_ticket(table: &mut TaskTable) -> u64 {
        let ticket = table.next_ticket;
        table.next_ticket = table.next_ticket.wrapping_add(1);
        ticket
    }
}

impl Default for LoginTasks {
    fn default() -> Self {
        Self::new()
    }
}

/// ： and in 1 。
///
/// ── Why no has `AuthService` ──────────────────────────────
/// WorkBuddy （ `login/account` / `accounts`、
/// ）， that already to 。 in this 「 + + 
/// 」， not need need will 。
#[derive(Clone)]
pub struct LoginService {
    store: AccountStore,
    tasks: LoginTasks,
}

impl LoginService {
    pub fn new(store: AccountStore) -> Self {
        Self {
            store,
            tasks: LoginTasks::new(),
        }
    }

    pub fn tasks(&self) -> &LoginTasks {
        &self.tasks
    }

    /// 1 ：， for will and 。
    /// （ already has use none need ），HTTP use cancel_async etc 。
    pub fn cancel(&self, state: &str) -> bool {
        let Some(handle) = self.cancel_task(state) else { return false };
        let flow = state.to_string();
        crate::spawn_task(async move { handle.cancel_provider_flow(&flow).await; });
        true
    }

    pub async fn cancel_async(&self, state: &str) -> bool {
        let Some(handle) = self.cancel_task(state) else { return false };
        handle.cancel_provider_flow(state).await;
        true
    }

    fn cancel_task(&self, state: &str) -> Option<LoginTaskHandle> {
        let handle = self.tasks.get(state)?;
        self.tasks.cancel(state).then_some(handle)
    }

    /// OAuth failure： can /wait ， use flow 。
    pub(crate) async fn reject_login_callback(
        &self,
        state: &str,
        error: &str,
    ) -> GatewayError {
        let Some(handle) = self.tasks.get(state.trim()) else {
            return GatewayError::with_status(404, "登录任务不存在或已过期，请重新发起网页登录");
        };
        let message = oauth_callback_error_message(error);
        {
            let mut task = handle.lock();
            if task.canceled { return GatewayError::bad_request("登录已取消，请重新发起"); }
            if task.done {
                return GatewayError::bad_request(task.error.clone().unwrap_or_else(|| "登录任务已结束".to_string()));
            }
            // etc ， in await poll not use to Done 。
            mark_task_error(&mut task, message);
        }
        handle.cancel_provider_flow(state.trim()).await;
        GatewayError::bad_request(message)
    }

    /// up ， provider（ need ， `LoginTaskState::provider`）。
    ///
    /// `edition` is （`cn` / `intl`）。** has WorkBuddy has 
    /// **，empty can —— this will to before ， by 「empty = not 
    /// 」。
    pub(crate) fn new_handle_for_provider(
        &self,
        edition: &str,
        provider: &str,
    ) -> LoginTaskHandle {
        let ticket = {
            let mut guard = self.tasks.lock();
            LoginTasks::next_ticket(&mut guard)
        };
        LoginTaskHandle {
            inner: Arc::new(Mutex::new(LoginTaskState {
                state: None,
                auth_url: None,
                instructions: None,
                hint: None,
                polling: false,
                method_index: None,
                done: false,
                error: None,
                session: None,
                edition: edition.to_string(),
                provider: provider.to_string(),
                canceled: false,
                finished_at: None,
            })),
            context: None,
            operation: Arc::new(tokio::sync::Mutex::new(())),
            ticket,
        }
    }

    /// in adapter； after not provider id new owner。
    pub(crate) fn new_handle_for_adapter(
        &self,
        edition: &str,
        adapter: Arc<dyn ProviderAdapter>,
        proxy: Option<LoginProxySelection>,
    ) -> LoginTaskHandle {
        let mut handle = self.new_handle_for_provider(edition, adapter.provider_id());
        handle.context = Some(Arc::new(LoginTaskContext {
            provider: adapter.provider_id(), adapter: Mutex::new(Some(adapter)),
            store: self.store.clone(), proxy,
        }));
        handle
    }

    /// 1 「」（trait 7 ， this ）。
    ///
    /// and [`Self::start`] ： this ** not up ** —— authUrl and state by 
    /// provider （ is ，state ），
    /// to code before no has 。 no has after 、 no has ，
    /// after etc `submit_login_callback` （ or etc `cancel`）。
    ///
    /// return `Err(Reason)` this unsupported（ use 400，）。
    ///
    /// ── Why is async ──────────────────────────────────────────
    /// by ** 1 **（`login.start` RPC），
    /// not can await。 `build_login_url_async` default（），
    /// as 1 not 。
    pub async fn start_web_login(&self, kind: ProviderKind) -> Result<LoginTaskHandle, String> {
        let label = crate::server::core::providers::meta(kind).label;
        let adapter = adapter_for(kind);
        // can ： has （`supports_web_login` is can ，
        // `build_login_url` is this ）。 can 「 this no has this 」
        // and 「 this has this not 」 in and 。
        if !adapter.supports_web_login() {
            return Err(format!(
                "{label}不支持网页登录，请改用「填写凭证」或「导入桌面端登录态」添加账号"
            ));
        }
        let Some((auth_url, state)) = adapter.build_login_url_async().await else {
            return Err(format!("{label}未能生成网页登录授权地址，请重试"));
        };
        // （ / AutoClaw） no has has ， 1 empty ——
        // `new_handle_for_provider` 。
        let handle = self.new_handle_for_provider(DEFAULT_EDITION, kind_id(kind));
        handle.update(|task| {
            task.state = Some(state.clone());
            task.auth_url = Some(auth_url);
        });
        self.tasks.register(&state, handle.clone());
        logging::log("[Login]", &format!("发起{label}网页登录（等待浏览器回调…）"));
        Ok(handle)
    }

    // KukuAI already to ： Cookie as 
    // `login.callback` `code` to ， and after 
    // （ `start_plugin_web_login` ）。

    /// OrcaTerm （****）： → 1 → up 
    /// → 。
    ///
    /// ── in not in this ──────────────────────────────────
    /// ** OAuth**： `orcaterm://oauth_callback?session_id=…`
    /// → `submit_login_callback` OrcaTerm → `exchange_login_code`
    /// （ `providers/orcaterm/login.rs` module header）。 is ****path：
    /// use already has 1 （ if from to 、 or 
    /// to ）， POST `{state, token}` to 
    /// `/api/session/login/orcaterm/complete` can —— to 1 
    /// 「」 up ， not 。
    ///
    /// ── and KukuAI and ──────────────────────────────────
    /// ： by POST ，、 etc 、 `/wait` when 1 
    /// （ `finish_kuku_login`）。
    /// ： this must** up **（`complete_login` 
    /// `GetUserInfo`）—— / down will to 401 ；
    /// KukuAI that is 「valid、」，。
    ///
    /// failure** not **： can can is to 、 up ， use 
    /// can ； error by return `Err` ， by `/wait` when 。
    pub async fn finish_orcaterm_login(
        &self,
        state: &str,
        token: &str,
    ) -> Result<String, GatewayError> {
        let state = state.trim();
        if state.is_empty() {
            return Err(GatewayError::with_status(400, "缺少 state，无法确认这次回调归属"));
        }
        let Some(handle) = self.tasks.get(state) else {
            return Err(GatewayError::with_status(
                404,
                "登录任务不存在或已过期，请重新发起网页登录",
            ));
        };
        let _operation = handle.operation.lock().await;
        let task = handle.snapshot();
        if task.canceled {
            return Err(GatewayError::with_status(400, "登录已取消，请重新发起"));
        }
        if task.done {
            return match task.error {
                Some(error) => Err(GatewayError::bad_request(error)),
                None => Ok(String::new()),
            };
        }
        if task.provider != crate::server::core::providers::kind_id(crate::server::core::providers::ProviderKind::OrcaTerm)
        {
            return Err(GatewayError::with_status(
                400,
                "该登录任务不属于 OrcaTerm，请重新发起",
            ));
        }
        // ── and to ────────────────────────────────
        // `complete_login` already delete（ and in ）。
        // this `account.normalize`： 1 （ GetUserInfo、
        // userId / ）， and `pluginSecret` 。
        let provider = "orcaterm";
        // already when ； old ，
        // not by current new owner。
        let adapter = handle.adapter().unwrap_or_else(|| ProviderKind::OrcaTerm.into());
        let input = agent2api_provider_protocol::NormalizeInput {
            provider: provider.to_string(),
            source: agent2api_provider_protocol::NormalizeSource::Manual {
                payload: json!({ "accessToken": token }),
            },
            name: None,
            proxy: handle.context.as_ref().and_then(|context| context.proxy.as_ref())
                .and_then(|proxy| serde_json::from_value(proxy.resolved.clone()).ok()),
        };
        let normalized = adapter.normalize_account(&self.store, input).await?;
        let outcome = persist_plugin_login(&handle, adapter.as_ref(), &normalized)
            .and_then(|id| id.ok_or_else(|| GatewayError::bad_request("登录任务已取消或结束")));
        handle.cancel_provider_flow_locked(state).await;
        if outcome.is_ok() {
            logging::log("[Login]", "OrcaTerm 网页登录完成，账号已加入列表");
        }
        outcome
    }

    /// etc authUrl（ for Node `/api/session/login/start` 15 seconds etc ）。
    ///
    /// return `(state, authUrl, edition)`； when or before failure when returnerrorReason。
    /// not ， in after （ and Node 1 ：return 502 not ）。
    pub async fn wait_for_auth_url(
        &self,
        handle: &LoginTaskHandle,
        timeout: Duration,
    ) -> Result<(String, String, String), String> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let task = handle.snapshot();
            if let (Some(state), Some(url)) = (task.state.clone(), task.auth_url.clone()) {
                // authUrl to ，/wait can by state to 
                self.tasks.register(&state, handle.clone());
                return Ok((state, url, task.edition));
            }
            if task.done {
                return Err(task
                    .error
                    .clone()
                    .unwrap_or_else(|| "上游登录服务未返回登录链接".to_string()));
            }
            if tokio::time::Instant::now() >= deadline {
                return Err("上游登录服务未返回登录链接".to_string());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// 1 ****： state → → → 。
    ///
    /// ── Why need by ──────────────────────────
    /// is `office-raccoon://auth/callback`。Electron
    /// can in will `protocol.handle('office-raccoon', …)` ，Tauri/WebView2
    /// ** no has etc **：WebView2 NavigationStarting is 「 before 1 need not need 
    /// 」， 1 will to （ no this when is 
    /// 1 failure）。 is ：** in URL、 down 、
    /// URL POST to **（ `src-tauri/src/login.rs`）。
    ///
    /// ── state （）───────────────────────────────────
    /// this 1 is up ** 1 ** can 「 this code is 」 ：
    /// by state ， and state is not can 。，
    /// can 1 URL use ** ** code and 
    /// （ etc to use ）。：
    /// 1. must in （ not in → 404「 new 」）；
    /// 2. state must and 1 （`parse_callback_code`）；
    /// 3. must no （ 1 → returnsuccess， not ，
    /// as is 1 ： use will to 200035「 already 」 error，
    /// 1 success 1 failure）。
    ///
    /// returnsuccess when id（ before not use ， to and will can can 「 after new 」）。
    pub async fn submit_login_callback(
        &self,
        state: &str,
        callback_url: &str,
    ) -> Result<LoginCallbackSubmission, GatewayError> {
        let state = state.trim();
        if state.is_empty() {
            return Err(GatewayError::with_status(400, "缺少 state，无法确认这次回调归属"));
        }
        let Some(handle) = self.tasks.get(state) else {
            return Err(GatewayError::with_status(
                404,
                "登录任务不存在或已过期，请重新发起网页登录",
            ));
        };
        let parsed_callback = url::Url::parse(callback_url).ok();
        if let Some(error) = parsed_callback.as_ref().and_then(|url| {
            url.query_pairs().find(|(key, value)| key == "error" && !value.trim().is_empty())
                .map(|(_, error)| error.into_owned())
        }) {
            // used for， URL / and code not 。
            return Err(self.reject_login_callback(state, &error).await);
        }
        let _operation = handle.operation.lock().await;
        let task = handle.snapshot();
        if task.canceled {
            return Err(GatewayError::with_status(400, "登录已取消，请重新发起"));
        }
        if task.done {
            if let Some(error) = task.error {
                return Err(GatewayError::with_status(400, error));
            }
            // etc used forsuccess，failure not can 。
            return Ok(LoginCallbackSubmission::Completed(String::new()));
        }
        // /must use when ； old not when 。
        let plugin_adapter = handle.adapter();
        // when （ or ） not callback-poll API。
        // empty URL / id not success； has this Done can path。
        if let Some(adapter) = &plugin_adapter {
            let code = parsed_callback.as_ref().and_then(|url| {
                url.query_pairs().find(|(key, _)| key == "code" || key == "authorization_code")
                    .map(|(_, value)| value.into_owned())
            }).unwrap_or_else(|| callback_url.to_string());
            let callback = agent2api_provider_protocol::LoginCallback {
                flow: state.to_string(), code, url: callback_url.to_string(),
            };
            let poll = adapter.login_callback_poll_async(&self.store, callback).await;
            // / 1 consumer await ， not to 。
            let current = handle.snapshot();
            if current.canceled { return Err(GatewayError::bad_request("登录已取消")); }
            if current.done {
                return match current.error {
                    Some(error) => Err(GatewayError::bad_request(error)),
                    None => Ok(LoginCallbackSubmission::Completed(String::new())),
                };
            }
            match poll {
                Ok(Some(agent2api_provider_protocol::LoginPoll::Done { account })) => {
                    let outcome = match persist_plugin_login(&handle, adapter.as_ref(), &account) {
                        Ok(Some(id)) => Ok(LoginCallbackSubmission::Completed(id)),
                        Ok(None) if handle.snapshot().canceled => Err(GatewayError::bad_request("登录已取消")),
                        Ok(None) => Ok(LoginCallbackSubmission::Completed(String::new())),
                        Err(error) => { finish_task_error(&handle, &error.message); Err(error) }
                    };
                    handle.cancel_provider_flow_locked(state).await;
                    return outcome;
                }
                Ok(Some(agent2api_provider_protocol::LoginPoll::Pending { hint })) => {
                    set_plugin_login_hint(&handle, hint.as_deref());
                    let mut start_polling = false;
                    handle.update(|task| {
                        if !task.canceled && !task.done && !task.polling {
                            task.polling = true;
                            start_polling = true;
                        }
                    });
                    if start_polling {
                        crate::server::api::session::spawn_plugin_login_poll(
                            handle.clone(), state.to_string(), 1_000, task.provider.clone(),
                        );
                    }
                    return Ok(LoginCallbackSubmission::Accepted);
                }
                Ok(Some(agent2api_provider_protocol::LoginPoll::Failed { message })) => {
                    finish_task_error(&handle, &message);
                    handle.cancel_provider_flow_locked(state).await;
                    return Err(GatewayError::bad_request(message));
                }
                Err(error) => {
                    finish_task_error(&handle, &error.message);
                    handle.cancel_provider_flow_locked(state).await;
                    return Err(error);
                }
                Ok(None) => {} // 旧适配器仍使用下面的 exchange_login_code。
            }
        }
        if let Some(adapter) = plugin_adapter {
            // old String API URL， and use when 。
            let result = adapter.exchange_login_code(&self.store, callback_url, state).await;
            let current = handle.snapshot();
            if current.canceled { return Err(GatewayError::bad_request("登录已取消")); }
            let outcome = match result {
                Ok(account_id) => {
                    match finish_adapter_login(&handle, &account_id) {
                        Ok(true) => Ok(LoginCallbackSubmission::Completed(account_id)),
                        Ok(false) if handle.snapshot().canceled => Err(GatewayError::bad_request("登录已取消")),
                        Ok(false) => Ok(LoginCallbackSubmission::Completed(String::new())),
                        Err(error) => { finish_task_error(&handle, &error.message); Err(error) }
                    }
                }
                Err(error) => {
                    finish_task_error(&handle, &error.message);
                    let task = handle.snapshot();
                    Err(task.error.map(GatewayError::bad_request).unwrap_or(error))
                }
            };
            handle.cancel_provider_flow_locked(state).await;
            return outcome;
        }
        let Some(kind) = kind_from_id(&task.provider) else {
            return Err(GatewayError::with_status(500, "登录任务未绑定提供商执行体，请重新发起"));
        };
        // Accio already to ： loopback 、
        // （`accio::login`）， this 「」 that use （`login_poll`）。
        // LobsterAI is `code` / `state` （ and Accio / Antigravity
        // ）， use this 1 ：
        // ① to `/auth/callback`（ this by not ）；
        // ② / use in ** URL** ——
        // and not and no has when ， not to ， by 
        // `web_shim.rs` `needsManualCallback` to this 。
        //
        // this 1 ，② will 1 to that 「 this not 」，
        // 1 can use can 400 —— `needsManualCallback` 
        // this need 。 need in ， need not in 。
        //
        // `state` use request that （ ）， state used for for 。
        if kind == ProviderKind::LobsterAi {
            let parsed = url::Url::parse(callback_url).map_err(|_| {
                GatewayError::with_status(400, "LobsterAI 登录回调地址无效")
            })?;
            let params: std::collections::HashMap<String, String> = parsed
                .query_pairs()
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect();
            if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
                finish_task_error(&handle, oauth_callback_error_message(error));
                return Err(GatewayError::bad_request(oauth_callback_error_message(error)));
            }
            let callback_state = params.get("state").map(String::as_str).unwrap_or("");
            let code = params.get("code").map(String::as_str).unwrap_or("");
            if callback_state.trim() != state {
                return Err(GatewayError::with_status(400, "回调地址与当前 LobsterAI 登录任务不匹配"));
            }
            return match self.finish_lobsterai_login(code, state).await {
                Ok(account_id) => Ok(LoginCallbackSubmission::Completed(account_id)),
                // failure when not by pending： that 1 in ****
                // （`take_pending` ）， this no has can 。
                Err(error) => Err(error),
            };
        }
        // Antigravity is `code` / `state` （Google OAuth
        // + loopback）。 and Accio ， this when ：
        // ① to `/oauth-callback` by （public handler
        // URL ）；
        // ② / Docker use in URL（
        // to not loopback when ）。
        // 1 ：state and for → 。
        if kind == ProviderKind::Antigravity {
            let parsed = url::Url::parse(callback_url).map_err(|_| {
                GatewayError::with_status(400, "Antigravity 登录回调地址无效")
            })?;
            let params: std::collections::HashMap<String, String> = parsed
                .query_pairs()
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect();
            if let Some(error) = params.get("error").filter(|value| !value.trim().is_empty()) {
                finish_task_error(&handle, oauth_callback_error_message(error));
                return Err(GatewayError::bad_request(oauth_callback_error_message(error)));
            }
            let callback_state = params.get("state").map(String::as_str).unwrap_or("");
            let code = params.get("code").map(String::as_str).unwrap_or("");
            if callback_state.trim() != state {
                return Err(GatewayError::with_status(400, "回调地址与当前 Antigravity 登录任务不匹配"));
            }
            return match adapter_for(kind)
                .exchange_login_code(&self.store, code, state)
                .await
            {
                Ok(account_id) => {
                    let session = json!({
                        "accountUid": account_id,
                        "nickname": Value::Null,
                        "edition": task.edition,
                        "provider": task.provider,
                    });
                    handle.update(|task| {
                        task.done = true;
                        task.session = Some(session);
                        task.finished_at = Some(logging::now_ms());
                    });
                    Ok(LoginCallbackSubmission::Completed(account_id))
                }
                Err(error) => {
                    // and Accio / Raccoon 1 ：failure need error，
                    // otherwise before will 1 etc to 5 minutes when 
                    finish_task_error(&handle, &error.message);
                    logging::log(
                        "[Login]",
                        &format!("❌ Antigravity 网页登录换取凭证失败: {}", error.message),
                    );
                    Err(error)
                }
            };
        }
        // OrcaTerm is 
        // （`orcaterm://oauth_callback?session_id=…`）：URL is `session_id`
        // not is `code`， CGI `OAuthExchangeToken`
        // （ `providers/orcaterm/login.rs` module header）。 and Antigravity ：
        // failure need error，otherwise before will 1 etc to 5 minutes when 。
        if kind == ProviderKind::OrcaTerm {
            return match adapter_for(kind)
                .exchange_login_code(&self.store, callback_url, state)
                .await
            {
                Ok(account_id) => {
                    let session = json!({
                        "accountUid": account_id,
                        "nickname": Value::Null,
                        "edition": task.edition,
                        "provider": task.provider,
                    });
                    handle.update(|task| {
                        task.done = true;
                        task.session = Some(session);
                        task.finished_at = Some(logging::now_ms());
                    });
                    Ok(LoginCallbackSubmission::Completed(account_id))
                }
                Err(error) => {
                    finish_task_error(&handle, &error.message);
                    logging::log(
                        "[Login]",
                        &format!("❌ OrcaTerm 网页登录换取凭证失败: {}", error.message),
                    );
                    Err(error)
                }
            };
        }
        // in this is 「 supports_web_login、 no has as 
        // 」 。 1 not ， by up that 
        // 「provider by 」 use —— config down to not this 。
        // not is 1 path： will code 1 from no 
        // （ is 「 1 in etc 」）。
        Err(GatewayError::with_status(
            400,
            format!(
                "该登录任务不接收授权码回调（{}）",
                crate::server::core::providers::label_of(&task.provider)
            ),
        ))
    }
}

/// OAuth failure to in ， not up error/error_description or response。
fn oauth_callback_error_message(error: &str) -> &'static str {
    if error == "access_denied" {
        "登录授权已取消或被拒绝，请重新发起登录"
    } else {
        "登录授权未完成，请重新发起登录"
    }
}

/// only as 。 and ， not HTML、 or 。
/// /option.value not this ，must to 。
pub(crate) fn login_text(text: &str) -> String {
    let mut in_tag = false;
    text.chars().filter(|ch| {
        if *ch == '<' { in_tag = true; return false; }
        if *ch == '>' { in_tag = false; return false; }
        !in_tag && (!ch.is_control() || matches!(*ch, '\n' | '\r' | '\t'))
    }).take(16_384).collect()
}

/// Done： and use 1 ， RPC return after ，
/// after poll and callback when return Done when 。
/// and 1 failureerror， not success need 。
pub(crate) fn persist_plugin_login(
    handle: &LoginTaskHandle,
    adapter: &dyn ProviderAdapter,
    account: &agent2api_provider_protocol::NormalizedAccount,
) -> Result<Option<String>, GatewayError> {
    let mut task = handle.lock();
    if task.canceled { return Ok(None); }
    if task.done {
        return match &task.error {
            Some(error) => Err(GatewayError::bad_request(error)),
            None => Ok(None),
        };
    }
    if handle.adapter().is_some_and(|owner| !std::ptr::eq(owner.as_ref(), adapter)) {
        return Err(GatewayError::with_status(500, "登录流程执行体不匹配，已拒绝保存账号"));
    }
    let account_id = match adapter.persist_login_account(account) {
        Ok(id) => id,
        Err(error) => { mark_task_error(&mut task, &error.message); return Err(error); }
    };
    if let Err(error) = handle.persist_proxy(&account_id) {
        mark_task_error(&mut task, &error.message);
        return Err(error);
    }
    mark_adapter_login_done(&mut task, &account_id);
    Ok(Some(account_id))
}

fn finish_adapter_login(handle: &LoginTaskHandle, account_id: &str) -> Result<bool, GatewayError> {
    let mut task = handle.lock();
    if task.canceled { return Ok(false); }
    if task.done {
        return match &task.error {
            Some(error) => Err(GatewayError::bad_request(error)),
            None => Ok(false),
        };
    }
    if let Err(error) = handle.persist_proxy(account_id) {
        mark_task_error(&mut task, &error.message);
        return Err(error);
    }
    mark_adapter_login_done(&mut task, account_id);
    Ok(true)
}

fn mark_adapter_login_done(task: &mut LoginTaskState, account_id: &str) {
    task.done = true;
    task.polling = false;
    task.error = None;
    task.hint = None;
    task.session = Some(json!({
        "accountUid": account_id, "nickname": Value::Null,
        "edition": task.edition, "provider": task.provider,
    }));
    task.finished_at = Some(logging::now_ms());
}

pub(crate) fn set_plugin_login_hint(handle: &LoginTaskHandle, hint: Option<&str>) {
    handle.update(|task| {
        if !task.canceled && !task.done { task.hint = hint.map(login_text); }
    });
}

/// and will need 
fn finish_task(handle: &LoginTaskHandle, session: &Value) {
    let summary = json!({
        "accountUid": session
            .get("account")
            .and_then(|account| account.get("uid"))
            .cloned()
            .unwrap_or(Value::Null),
        "nickname": session
            .get("account")
            .and_then(|account| account.get("nickname"))
            .cloned()
            .unwrap_or(Value::Null),
        "edition": session
            .get("edition")
            .and_then(Value::as_str)
            .unwrap_or_default(),
    });
    handle.update(|task| {
        task.done = true;
        task.session = Some(summary);
        task.finished_at = Some(logging::now_ms());
    });
}

fn mark_task_error(task: &mut LoginTaskState, message: &str) {
    task.done = true;
    task.polling = false;
    task.session = None;
    task.hint = None;
    task.error = Some(login_text(message));
    task.finished_at = Some(logging::now_ms());
}

/// failure（ cancel and to use ）
fn finish_task_error(handle: &LoginTaskHandle, message: &str) {
    handle.update(|task| {
        if task.canceled || task.done { return; }
        mark_task_error(task, message);
    });
}

/// success and will need （****， to that use ）。
///
/// ── Why need need ────────────────────────────────────────────
/// after in `api::session` （ need adapter 
/// `login.poll`）， need `LoginTaskState` has 
/// `finished_at`。 that 1 `login.rs` is 「」 this has 
/// 1 —— （ / / ） 1 ，
/// before `/wait` need 1 。
///
/// will and that （ before ）：`accountUid` / `nickname`
/// / `edition` / `provider`。
pub fn finish_plugin_login(handle: &LoginTaskHandle, account_id: &str) {
    if let Err(error) = finish_adapter_login(handle, account_id) {
        finish_task_error(handle, &error.message);
    }
}

/// failure（`finish_task_error` ，）。
pub fn fail_plugin_login(handle: &LoginTaskHandle, message: &str) {
    finish_task_error(handle, message);
}
