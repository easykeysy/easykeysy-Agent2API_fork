//! ： + RPC client。
//!
//! ── this in ──────────────────────────────────
//! is **** ，「provider 1 」
//! that 。 up （`providers::adapter::adapter_for`）
//! [`crate::server::core::providers::adapter::ProviderAdapter`]； this 
//! [`RpcAdapter`]（ that trait ） and [`Host`]（）。
//!
//! ── ： 1 ──────────────────────────────
//! and magpie ： 1 = 1 = if provider。 by ：
//! - 21 tokio when and in （ MB ）；
//! - provider （HTTP client、） 1 ；
//! - —— 1 provider panic is its ，
//! ** not will **（ that is need ）。
//!
//! ── ────────────────────────────────────────────────
//! → all in use failure（ not is ）→ by 「 up unavailable」
//! （ / / ）→ down 1 use when new spawn。
//!
//! ** not none**： by （1s → 2s → 4s … limit 60s），
//! and to 5 after 「」 —— 「 this ， already disabled」，
//! use can 。none will 1 CPU 。
//!
//! ── （ before ）──────────────────────────────────
//! not in await 。 will in request when 
//! （`store.get` etc ）， that by ； if in etc 
//! ， not to 。：
//! - 「 → to pending / task」；
//! - all await in task ；
//! - ， not etc IO；
//! - already ， use JSON 。

mod activity;
mod bootstrap;
mod callbacks;
pub mod catalog;
mod compatibility;
mod errors;
mod gemini_signatures;
mod install;
mod leased_stream;
mod lifecycle;
mod logs;
mod management;
mod model_sync;
pub mod operations;
mod ownership;
mod runtime;
mod packages;
mod pack_view;
mod process;
mod registry;
mod rpc;
pub(crate) mod rpc_cache;
pub mod schedules;
pub mod settings;
mod stream;
mod switching;
pub mod update;

#[cfg(test)]
mod tests;

pub use callbacks::GatewayCallbacks;
pub use bootstrap::{
    reload as reload_host, set_loopback_port, spawn as spawn_bootstrap, start as bootstrap_host,
    StoreCallbacks,
};
pub use install::{
    install_zip, list_packs, resolve_ui_file, set_pack_enabled, set_provider_enabled, uninstall,
    InstalledPack, PackProviderSummary, PackRegistry, PackSummary, PackUiSummary,
};
pub use registry::{install as install_provider, installed_ids, lookup, lookup_ready, remove, clear};
pub use registry::{serves as serves_provider};
pub use rpc::{HostOptions, RpcAdapter};
pub(crate) use rpc_cache::meta as model_catalog_meta;
pub use management::snapshot as management_snapshot;
pub(crate) use packages::prepare as prepare_pack;
pub(crate) use compatibility::provider_error as provider_compatibility_error;

/// current （「 down 」 use ）。
pub fn host_target() -> String {
    rpc::target_triple()
}

/// provider ****（ by already ， not is in ）。
///
/// ── Why not `lookup(id).label()` ─────────────────────────
/// that **success** 。disabled （ `demo`） not and ，
/// is 「 has 」 when can id —— use to 1 `demo`
/// ， not that is 。 label， can 。
///
/// is use 1 and `pack.json`。 this 1 is 
/// provider need （ 1 `/api/session` 1 、 for 「 has no in 」 
/// ）， can ignore； need ， in this before 。
pub fn provider_label(id: &str) -> Option<String> {
    for pack in install::list_packs() {
        if !pack.enabled {
            continue;
        }
        if let Some(found) = pack.providers.iter().find(|item| item.id == id) {
            return Some(found.label.clone());
        }
    }
    None
}

/// directory path（「 to 」 use ）。
pub fn plugins_dir_display() -> String {
    install::plugins_dir().display().to_string()
}

/// empty provider （ / before ）。
///
/// and [`clear`] is 1 ， is as api use is 
/// 「 already provider 」， not is 「 1 」—— not ：
/// before is use can （）， after is 。
pub fn clear_providers() {
    clear();
    rpc_cache::clear_all();
}

pub(crate) use leased_stream::hold_adapter;
pub(crate) use stream::RpcStream;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use agent2api_provider_protocol as proto;
use tokio::sync::{oneshot, Mutex};

use crate::server::logging;

pub use ownership::ProviderCategory;
pub use runtime::{current, detach, host_for_pack, host_for_provider, install, provider_category, provider_pack, shutdown_all};
use runtime::provider_info;

/// current already provider ****（）。
///
/// ── Why need need ────────────────────────────────────────
/// 、、、daily this in ，
/// in 。 by provider id ， no when 、 is 
/// 1 「failure」。 as by this ： no → empty → not 。
///
/// no （ no / in ） when returnempty， and 「 1 no has 」。
pub fn admin_panels() -> Vec<proto::AdminPanel> {
    let providers = runtime::provider_infos();
    let mut panels: Vec<proto::AdminPanel> = Vec::new();
    for info in providers {
        for panel in info.admin_panels {
            // `Unknown` is 「 new old 」 ， not ，
            // in 
            if panel != proto::AdminPanel::Unknown && !panels.contains(&panel) {
                panels.push(panel);
            }
        }
    }
    panels
}

/// 1 is （ need by provider when use ）。
pub fn has_admin_panel(provider: &str, panel: proto::AdminPanel) -> bool {
    provider_info(provider).is_some_and(|info| info.admin_panels.contains(&panel))
}

/// **parameter**（`provider id → AccountFeatures`）。
///
/// ── Why need need ────────────────────────────────────────
/// 1 、 need not need to 、 need not need after ， is up 
/// 。 has 1 by provider id ， 1 
/// 。 as by this ：， by 。
///
/// ** **： no （ old ） id not in ， 
/// —— this is ， not etc all when 。
pub fn account_features() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    let providers = runtime::provider_infos();
    for info in providers {
        let Some(features) = info.account_features else {
            continue;
        };
        // failure can can is not can （ not will ）； that when 
        // —— ， not will parameter
        if let Ok(value) = serde_json::to_value(&features) {
            map.insert(info.id.as_str().to_string(), value);
        }
    }
    map
}

/// 1 parameter（ no = `None`）。
pub fn account_features_of(provider: &str) -> Option<proto::AccountFeatures> {
    provider_info(provider)?.account_features
}

/// 1 to can （ no = `None`）。
pub fn checkin_of(provider: &str) -> Option<proto::CheckinInfo> {
    provider_info(provider)?.checkin
}

/// to can （`provider id → `）。
///
/// —— old not when not in ， use 
/// （`Capabilities::checkin`）， is 。
pub fn checkin_features() -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();
    let providers = runtime::provider_infos();
    for info in providers {
        let Some(checkin) = info.checkin else {
            continue;
        };
        if let Ok(value) = serde_json::to_value(&checkin) {
            map.insert(info.id.as_str().to_string(), value);
        }
    }
    map
}

/// this failure is not is 「」（issue #379）。
///
/// ── Why by not is new error ──────────────────────────
/// by crate `write_frame` to （`io::ErrorKind::InvalidData` +
/// 「 … limit」）， `InvalidData` when used for「none」。
/// in —— is **failure**， no has ——
/// this `InvalidData` 1 and 。 in ， not 
/// that 。
fn is_frame_too_large(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::InvalidData
        && error.to_string().contains("帧超过")
}

/// （ to ）。
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct HostStatus {
    /// is in 。
    pub running: bool,
    /// already provider id 。
    pub providers: Vec<String>,
    /// （> 0 this not ）。
    pub crashes: u32,
    /// already disabled（）。
    pub disabled: bool,
    /// failureReason； in in 。
    pub last_error: Option<String>,
}

/// 1 （ for 1 already ）。
pub struct Host {
    /// parameter。
    options: HostOptions,
    activity: Arc<activity::Activity>,
    /// current ； by ProcessRun has 。
    inner: Mutex<Option<Inner>>,
    /// and use use ； not await has 。
    lifecycle: StdMutex<()>,
    generation: AtomicU64,
    active: StdMutex<Option<Arc<process::ProcessRun>>>,
    /// etc （ → request）。
    ///
    /// ── Why use std Mutex ────────────────────────────────────
    /// this in 「」 and 「」， no has 
    /// await。 use tokio Mutex will 1 await （ 
    /// then is 「 not await」）。
    pending: StdMutex<HashMap<i64, PendingCall>>,
    /// down 1 id。
    next_id: AtomicI64,
    /// in use （`abort` and by ）。
    streams: StdMutex<HashMap<i64, StreamState>>,
    /// and ； not can has inner 。
    starting: Mutex<()>,
    /// is already （`init` success）。
    ///
    /// and `running` ：failure when ，`running` as `ready`
    /// as —— when `ensure()` must not is empty provider 。
    ready: AtomicBool,
    /// when （ after to before not spawn）。
    retry_after: StdMutex<Option<std::time::Instant>>,
    /// 。
    crashes: StdMutex<u32>,
    /// is already disabled（）。
    disabled: AtomicBool,
    /// after this Host not ； new enabled by create new Host。
    closing: AtomicBool,
    /// already and in ； has can 。
    running: AtomicBool,
    /// failureReason。
    last_error: StdMutex<Option<String>>,
    /// already provider（`init` after ）。
    providers: StdMutex<Vec<proto::ProviderInfo>>,
    /// （， [`GatewayCallbacks`]）。
    callbacks: Arc<dyn GatewayCallbacks>,
}

/// and 。
struct Inner {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    run: Arc<process::ProcessRun>,
}

struct PendingCall {
    tx: oneshot::Sender<Result<serde_json::Value, proto::WireError>>,
    _lease: Option<Arc<activity::UsageLease>>,
}

/// 1 in 。
///
/// ── Why use none not is has ──────────────────────────────
/// ** not can ** await（module header ）， has when can or ，
/// not can 。limit is ** **： not 
/// `credit`， 1 （512 KiB）。none
/// 1 —— ， not 。
struct StreamState {
    /// head / chunk / to 。
    tx: tokio::sync::mpsc::UnboundedSender<StreamEvent>,
    _lease: Arc<activity::UsageLease>,
    closed: bool,
}

/// to 。
///
/// ── Why head and chunk use 1 ────────────────────────
/// head 1 oneshot， is ： in head ** before **failure
/// （、 up not up ） when ， head ， to error in 
/// 1 no —— client etc 「response when 」 to 1 「 when 」，
/// Reason（ already ） 。 1 「 to 
/// to 」， chunk in head after 。
pub(crate) enum StreamEvent {
    /// up response（ 1 ； and ）。
    Head(proto::StreamHead),
    /// 1 up （ already is OpenAI chat SSE ）。
    Chunk(Vec<u8>),
    /// no / ： IO error， not is 。
    Failed(String),
    /// to （`result: null` or error）。
    ///
    /// and `Failed` ： this is ** **（Whyfailure），
    /// `Failed` is （ already no ）。
    Done(Result<serde_json::Value, proto::WireError>),
}

/// 1 use （`conversation` use ）。
pub(crate) struct StreamCall {
    /// use RPC id（、 when use ）。
    pub id: i64,
    /// head / chunk / （）。
    pub events: tokio::sync::mpsc::UnboundedReceiver<StreamEvent>,
}

const CALLBACK_FORBIDDEN: &str = "forbidden";

impl Host {
    /// 1 （ not ）。
    pub fn new(options: HostOptions, callbacks: Arc<dyn GatewayCallbacks>) -> Self {
        Self {
            options,
            activity: Arc::new(activity::Activity::default()),
            inner: Mutex::new(None),
            lifecycle: StdMutex::new(()),
            generation: AtomicU64::new(0),
            active: StdMutex::new(None),
            pending: StdMutex::new(HashMap::new()),
            next_id: AtomicI64::new(1),
            streams: StdMutex::new(HashMap::new()),
            starting: Mutex::new(()),
            ready: AtomicBool::new(false),
            retry_after: StdMutex::new(None),
            crashes: StdMutex::new(0),
            disabled: AtomicBool::new(false),
            closing: AtomicBool::new(false),
            running: AtomicBool::new(false),
            last_error: StdMutex::new(None),
            providers: StdMutex::new(Vec::new()),
            callbacks,
        }
    }

    pub(crate) fn ready(&self) -> bool {
        self.ready.load(Ordering::Acquire) && self.running.load(Ordering::Acquire)
            && !self.closing.load(Ordering::Acquire)
    }

    /// current（ to ）。
    pub fn status(&self) -> HostStatus {
        let _state = self.lifecycle.lock().unwrap_or_else(|error| error.into_inner());
        let providers = self
            .providers
            .lock()
            .map(|list| list.iter().map(|p| p.id.0.clone()).collect())
            .unwrap_or_default();
        HostStatus {
            // this use try_lock 「 in 」：`inner` is tokio Mutex，
            // up down not can await。 use 1 —— `running` 。
            running: self.running.load(Ordering::Acquire),
            providers,
            crashes: self.crashes.lock().map(|c| *c).unwrap_or(0),
            disabled: self.disabled.load(Ordering::Acquire),
            last_error: self.last_error.lock().ok().and_then(|e| e.clone()),
        }
    }


    /// 1 etc in use 。
    ///
    /// ── Why ────────────────────────────────────────
    /// use must** in chunk after **（ [`StreamEvent`]），
    /// not `pending` ， is that 。 by id 
    /// （`open_stream` 、`call` pending ）， 
    /// not ； is as in before 1 in 。
    fn settle(&self, frame: proto::Frame) {
        let outcome = match frame.error {
            Some(err) => Err(err),
            None => Ok(frame.result.unwrap_or(serde_json::Value::Null)),
        };
        let streamed = {
            let guard = self.streams.lock();
            match guard {
                Ok(mut guard) => guard.remove(&frame.id),
                Err(_) => None,
            }
        };
        if let Some(stream) = streamed {
            let _ = stream.tx.send(StreamEvent::Done(outcome));
            return;
        }
        let waiter = self.pending.lock().ok().and_then(|mut map| map.remove(&frame.id));
        if let Some(call) = waiter {
            let _ = call.tx.send(outcome);
        }
    }

    /// / in 。
    ///
    /// ── Why this （ not in read_loop ）──────────
    /// （`log` / `head` / `chunk`） is 「 → 」，
    /// no has await， in 1 can 1 to 。
    fn handle_event(&self, frame: proto::Frame) {
        match frame.event.as_deref() {
            Some("log") => {
                let level = frame.level.unwrap_or_else(|| "info".to_string());
                let message = frame.message.unwrap_or_default();
                self.callbacks.log_context(level, message, frame.params.unwrap_or(serde_json::Value::Null));
            }
            Some("head") => self.deliver_head(frame),
            Some("chunk") => self.deliver_chunk(frame),
            // `end` etc ： old ignore， not （ to before ）
            _ => {}
        }
    }

    /// `head`（ up response）。
    fn deliver_head(&self, frame: proto::Frame) {
        let status = frame.status.unwrap_or(200);
        let headers = frame.headers.unwrap_or_default();
        // by up （snake_case）。 not by Chat —— that is 
        // 「 not this 」 （ and `UpstreamKind` default）。
        let upstream = frame
            .upstream
            .as_deref()
            .and_then(|name| {
                serde_json::from_value::<proto::UpstreamKind>(serde_json::Value::String(
                    name.to_string(),
                ))
                .ok()
            })
            .unwrap_or_default();
        let guard = match self.streams.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if let Some(state) = guard.get(&frame.id).filter(|state| !state.closed) {
            // head is ， by 「 1 」
            let _ = state.tx.send(StreamEvent::Head(proto::StreamHead { status, headers, upstream }));
        }
    }

    /// `chunk`（ 1 up ）。
    fn deliver_chunk(&self, frame: proto::Frame) {
        let Some(data) = frame.data.as_deref() else { return };
        let bytes = match proto::decode_chunk(data) {
            Ok(bytes) => bytes,
            Err(e) => {
                // base64 = this not can after ， and etc 
                logging::log("Plugin", &format!("流式帧的 chunk 无法解码: {e}"));
                self.fail_stream(frame.id, format!("插件发出的 chunk 无法解码: {e}"));
                return;
            }
        };
        let guard = match self.streams.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if let Some(state) = guard.get(&frame.id).filter(|state| !state.closed) {
            // failure = already not in （client）。 not in this abort：
            // drop when `RpcStream` Drop will abort， that 1 。
            let _ = state.tx.send(StreamEvent::Chunk(bytes));
        }
    }

    /// 1 in failure（ / ）。
    fn fail_stream(&self, id: i64, message: String) {
        let guard = match self.streams.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if let Some(state) = guard.get(&id) {
            let _ = state.tx.send(StreamEvent::Failed(message));
        }
    }

    /// 1 use and etc 。
    pub async fn call(
        &self,
        method: &str,
        params: serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value, String> {
        self.call_coded(method, params, timeout).await.map_err(|error| error.message)
    }

    /// and [`Self::call`] 1 ，** error**（`WireError.code`）。
    ///
    /// ── Why need need this 1 ────────────────────────────────────────
    /// has 1 error **** not in in ：「 before not 」
    /// （`LOCAL_PRECONDITION`） need down 1 、 when 
    /// this 1 ， its error need request。 message ， can 
    /// before —— 1 will 。
    pub async fn call_coded(
        &self,
        method: &str,
        params: serde_json::Value,
        timeout: Duration,
    ) -> Result<serde_json::Value, proto::WireError> {
        let lease = if matches!(method, proto::methods::gateway::INIT | proto::methods::gateway::SHUTDOWN | proto::methods::gateway::ABORT) {
            None
        } else {
            Some(self.activity.acquire().ok_or_else(|| proto::WireError::with_code(
                "插件正在切换或已关闭", "plugin_updating",
            ))?)
        };
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        let run = {
            let _state = self.lifecycle.lock().unwrap_or_else(|error| error.into_inner());
            let run = self.current_run().filter(|run| self.is_current_run(run)
                && self.running.load(Ordering::Acquire) && !run.failed.load(Ordering::Acquire)
                && (!run.is_expected_exit() || method == proto::methods::gateway::SHUTDOWN))
                .ok_or_else(|| proto::WireError::with_code("插件进程不可用", proto::WireError::INTERNAL))?;
            self.pending.lock().map_err(|_| proto::WireError::with_code("回调表被污染", proto::WireError::INTERNAL))?
                .insert(id, PendingCall { tx, _lease: lease });
            run
        };
        let frame = proto::Frame {
            id,
            method: Some(method.to_string()),
            params: Some(params),
            ..Default::default()
        };
        if let Err(e) = self.write_for_run(frame, run).await {
            if let Ok(mut map) = self.pending.lock() {
                map.remove(&id);
            }
            // ── 「 before failure」（issue #379）──────────────────
            // 「request、 this 1 not 」 is ** before failure**：、
            // 、 not will ， will etc 1 。 already has 556
            // （`readiness::LOCAL_PRECONDITION_STATUS`） this 1 ，
            // （ `provider_loop` for 556 ）。 to INTERNAL
            // will 1 failure request up 。
            let code = if is_frame_too_large(&e) {
                proto::WireError::LOCAL_PRECONDITION
            } else {
                proto::WireError::INTERNAL
            };
            return Err(proto::WireError::with_code(
                format!("写插件失败: {e}"),
                code,
            ));
        }
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(err))) => Err(err),
            Ok(Err(_)) => Err(proto::WireError::with_code(
                "插件没有应答（宿主已退出）",
                proto::WireError::INTERNAL,
            )),
            Err(_) => {
                // when etc ； before ， not unknownempty。
                if let Ok(mut map) = self.pending.lock() {
                    if map.get(&id).is_some_and(|call| call._lease.is_none()) { map.remove(&id); }
                }
                Err(proto::WireError::with_code(
                    format!("插件调用超时（{method}）"),
                    proto::WireError::INTERNAL,
                ))
            }
        }
    }

    /// 1 to 。
    async fn write(&self, frame: proto::Frame) -> std::io::Result<()> {
        let run = self.current_run().ok_or_else(process::disconnected)?;
        self.write_for_run(frame, run).await
    }

    async fn write_for_run(&self, frame: proto::Frame, run: Arc<process::ProcessRun>) -> std::io::Result<()> {
        let stdin = {
            let guard = self.inner.lock().await;
            let inner = guard.as_ref().filter(|inner| Arc::ptr_eq(&inner.run, &run))
                .ok_or_else(process::disconnected)?;
            inner.stdin.clone()
        };
        let mut stdin = stdin.lock_owned().await;
        // write_all can can already ； use not can 。
        tokio::spawn(async move {
            if run.retired.load(Ordering::Acquire) || run.failed.load(Ordering::Acquire) {
                return Err(process::disconnected());
            }
            proto::write_frame(&mut *stdin, &frame).await
        }).await.map_err(|error| std::io::Error::other(format!("插件写入任务未完成：{error}")))?
    }


    /// use ；head、chunk and use ，。
    pub(crate) async fn open_stream(
        self: &Arc<Self>,
        method: &str,
        params: serde_json::Value,
    ) -> Result<StreamCall, String> {
        let lease = self.activity.acquire().ok_or_else(|| "插件正在切换或已关闭".to_string())?;
        self.ensure().await?;
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

        let run = {
            let _state = self.lifecycle.lock().unwrap_or_else(|error| error.into_inner());
            let run = self.current_run().filter(|run| self.is_current_run(run) && self.ready()
                && !run.failed.load(Ordering::Acquire) && !run.is_expected_exit())
                .ok_or_else(|| "插件进程不可用".to_string())?;
            self.streams.lock().map_err(|_| "流式表被污染".to_string())?
                .insert(id, StreamState { tx: event_tx, _lease: lease, closed: false });
            run
        };

        let (written, write_result) = oneshot::channel();
        let mut opening = activity::OpeningStream {
            host: self.clone(), id: Some(id), written: Some(write_result),
        };
        let frame = proto::Frame {
            id,
            method: Some(method.to_string()),
            params: Some(params),
            ..Default::default()
        };
        let host = self.clone();
        let write = crate::spawn_task(async move {
            let result = host.write_for_run(frame, run).await;
            let _ = written.send(result.is_ok());
            result
        }).await.map_err(|error| std::io::Error::other(format!("插件写入任务中断：{error}")))
            .and_then(|result| result);
        if let Err(e) = write {
            if let Ok(mut guard) = self.streams.lock() {
                guard.remove(&id);
            }
            // return is `String`（ no has error）， this in 
            // when 「 this is 、 no use 」 —— use ，
            // not 1 in this 1 error。
            if is_frame_too_large(&e) {
                return Err(format!("写插件失败: {e}（请求体过大，换账号或重试都不会成功）"));
            }
            return Err(format!("写插件失败: {e}"));
        }
        opening.id = None;
        Ok(StreamCall { id, events: event_rx })
    }

    /// not already ； and by or 。
    pub fn close_stream(&self, id: i64) -> bool {
        if let Ok(mut guard) = self.streams.lock() {
            if let Some(stream) = guard.get_mut(&id) {
                stream.closed = true;
                return true;
            }
        }
        false
    }

    /// 1 in use 。
    pub async fn abort(&self, id: i64) {
        let _ = self.call_coded(
            proto::methods::gateway::ABORT,
            serde_json::json!({ "id": id }),
            Duration::from_secs(10),
        ).await;
    }

    /// 。
    pub async fn credit(&self, id: i64, n: usize) {
        let frame = proto::Frame {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            method: Some(proto::methods::gateway::CREDIT.to_string()),
            params: Some(serde_json::json!({ "id": id, "n": n })),
            ..Default::default()
        };
        let _ = self.write(frame).await;
    }
}

impl HostStatus {
    /// up （ in ， before i18n）。
    pub fn describe(&self) -> String {
        if self.disabled {
            return format!(
                "插件包已停用（连续崩溃 {} 次）：{}",
                self.crashes,
                self.last_error.as_deref().unwrap_or("原因未知")
            );
        }
        if self.running {
            return format!("插件运行中，已加载 {} 家提供商", self.providers.len());
        }
        match &self.last_error {
            Some(error) => format!("插件未运行：{error}"),
            None => "插件未启动".to_string(),
        }
    }
}
