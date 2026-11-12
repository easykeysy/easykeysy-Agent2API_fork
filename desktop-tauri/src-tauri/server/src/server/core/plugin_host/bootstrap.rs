//! ： 「 already 」「available 」。
//!
//! ── this 1 in ────────────────────────────────
//! `bootstrap()` before ， is 1 in up 。 this 1 ：
//! 1. `plugins.json`（、 、）；
//! 2. for enabled spawn and ；
//! 3. as provider 1 [`RpcAdapter`]；
//! 4. when —— after `adapter_by_id(id)` can to 。
//!
//! ── Whyfailure not ────────────────────────────────────
//! is **optional**： no 、、 not ，must（
//! 21 ）。 this allerror， not up 。
//!
//! ── Why is after ────────────────────────────────────────
//! can can need seconds（ + 1 directory）。 in `bootstrap()`
//! path will 「 use 」 etc this seconds， this seconds no 
//! request。 spawn ， 1 use to when `ensure()` will etc 。

use std::sync::Arc;
use std::time::Duration;

use agent2api_provider_protocol as proto;

use crate::server::core::account_store::AccountStore;
use crate::server::core::providers::ProviderKind;
use crate::server::logging;

use super::install::{self, PackRegistry};
use super::rpc::{HostOptions, RpcAdapter};
use super::{registry, GatewayCallbacks, Host};

/// ： `store.*` to up 。
///
/// ── up （ before ）────────────────────────────
/// 1. **`secret` is JSON**， not 、 not 。 
/// `secret` ， when —— 「」。
/// 2. **`public` is to **， is 。 will 
/// and （`extra`）， is not need need before 。
pub struct StoreCallbacks {
    store: AccountStore,
    // by ， not parameter in provider/owner as 。
    allowed_providers: std::collections::HashSet<String>,
}

impl StoreCallbacks {
    /// default no has /set； by for_providers 。
    pub fn new(store: AccountStore) -> Self {
        Self { store, allowed_providers: std::collections::HashSet::new() }
    }

    fn for_providers(store: AccountStore, providers: Vec<String>) -> Self {
        Self { store, allowed_providers: providers.into_iter().collect() }
    }

    fn check_provider(&self, provider: &str) -> Result<(), proto::WireError> {
        if provider.is_empty() || !self.provider_allowed(provider) {
            return Err(proto::WireError::with_code(
                "提供商不属于当前插件包", super::CALLBACK_FORBIDDEN,
            ));
        }
        Ok(())
    }

    fn check_account(&self, account: &str, provider: Option<&str>) -> Result<String, proto::WireError> {
        if account.is_empty() {
            return Err(proto::WireError::with_code("账号不存在", proto::WireError::NOT_FOUND));
        }
        let actual = self.store.account_provider_of(account)
            .ok_or_else(|| proto::WireError::with_code("账号不存在", proto::WireError::NOT_FOUND))?;
        self.check_provider(&actual)?;
        if provider.is_some_and(|requested| requested != actual) {
            return Err(proto::WireError::with_code(
                "账号不属于请求的提供商", super::CALLBACK_FORBIDDEN,
            ));
        }
        Ok(actual)
    }
}

/// 1 `secret` / `public` 。
///
/// in is ， is ：`pluginSecret`
/// （ not JSON） and `pluginPublic`（ to ）。 use not is use 
/// `accessToken` that ， is as 「 this 」 this 1 can ——
/// new path to will （ ）。
/// in `account_store::plugin_accounts`（ in that ）。
impl GatewayCallbacks for StoreCallbacks {
    fn account_provider(&self, account: &str) -> Option<String> {
        self.store.account_provider_of(account)
    }

    fn provider_allowed(&self, provider: &str) -> bool {
        self.allowed_providers.contains(provider)
    }

    fn store_get(
        &self,
        provider: String,
        account: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            self.check_provider(&provider)?;
            // empty id = already provider ； not to default WorkBuddy 。
            let id = if account.is_empty() {
                let store = self.store.clone();
                let provider = provider.clone();
                crate::server::core::blocking::run(move || {
                    store.route_account_ids_for_provider(&provider).into_iter().next()
                }).await.map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?
            } else {
                Some(account)
            };
            let Some(id) = id else {
                return Err(proto::WireError::with_code(
                    format!("{provider} 没有可用账号"),
                    proto::WireError::NOT_FOUND,
                ));
            };
            self.check_account(&id, Some(&provider))?;
            let store = self.store.clone();
            let record = crate::server::core::blocking::run(move || store.plugin_account(&id)).await
                .map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?;
            let Some(record) = record else {
                return Err(proto::WireError::with_code(
                    "账号不存在",
                    proto::WireError::NOT_FOUND,
                ));
            };
            // and can can has and ； for ， not secret。
            if record.get("provider").and_then(serde_json::Value::as_str) != Some(provider.as_str()) {
                return Err(proto::WireError::with_code(
                    "账号不属于请求的提供商", super::CALLBACK_FORBIDDEN,
                ));
            }
            Ok(record)
        })
    }

    fn store_list(
        &self,
        provider: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            self.check_provider(&provider)?;
            let store = self.store.clone();
            let list = crate::server::core::blocking::run(move || {
                store.route_account_ids_for_provider(&provider).into_iter()
                    .filter_map(|id| store.plugin_account(&id))
                    .filter(|record| record.get("provider").and_then(serde_json::Value::as_str) == Some(provider.as_str()))
                    .collect::<Vec<_>>()
            }).await.map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?;
            Ok(serde_json::Value::Array(list))
        })
    }

    fn store_update(
        &self,
        account: String,
        expect: serde_json::Value,
        secret: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            let provider = self.check_account(&account, None)?;
            let store = self.store.clone();
            let updated = crate::server::core::blocking::run(move || {
                if store.account_provider_of(&account).as_deref() != Some(provider.as_str()) {
                    return Err("账号归属已变化".to_string());
                }
                store.plugin_update_secret(&account, &expect, &secret)
            }).await.map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?;
            match updated {
                Ok(true) => Ok(serde_json::json!({ "updated": true })),
                // 「 already 」 not is error： 1 new ，
                // new 1 ， not is 。
                Ok(false) => Ok(serde_json::json!({ "updated": false })),
                Err(message) => Err(proto::WireError::with_code(message, proto::WireError::INTERNAL)),
            }
        })
    }

    fn store_update_with_public(
        &self,
        account: String,
        expect: serde_json::Value,
        secret: serde_json::Value,
        public: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            let provider = self.check_account(&account, None)?;
            let view = if public.is_object() {
                Some(public)
            } else if let Some(adapter) = registry::callback_adapter(&provider) {
                adapter.plugin_account_view(&secret).await.ok()
            } else {
                None
            };
            // account_view can can etc RPC； before ， not can etc to 。
            self.check_account(&account, Some(&provider))?;
            let store = self.store.clone();
            crate::server::core::blocking::run(move || {
                if store.account_provider_of(&account).as_deref() != Some(provider.as_str()) {
                    return Err("账号归属已变化".to_string());
                }
                store.plugin_update_account(&account, &expect, &secret, view.as_ref())
            }).await.map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?
                .map(|updated| serde_json::json!({ "updated": updated }))
                .map_err(|message| proto::WireError::with_code(message, proto::WireError::INTERNAL))
        })
    }

    fn store_mark_limited(
        &self,
        account: String,
        model: String,
        reset_at: Option<i64>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), proto::WireError>> + Send + '_>>
    {
        Box::pin(async move {
            let provider = self.check_account(&account, None)?;
            let store = self.store.clone();
            crate::server::core::blocking::run(move || {
                if store.account_provider_of(&account).as_deref() != Some(provider.as_str()) {
                    return Err("账号归属已变化".to_string());
                }
                store.mark_rate_limited(&account, &model, 429, None,
                    reset_at.map(|value| value as f64), "插件上报限额");
                Ok(())
            }).await.map_err(|error| proto::WireError::with_code(error.message, proto::WireError::INTERNAL))?
                .map_err(|message| proto::WireError::with_code(message, super::CALLBACK_FORBIDDEN))
        })
    }

    fn ledger_get(
        &self,
        account: String,
        kind: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            self.check_account(&account, None)?;
            Ok(self.store.plugin_ledger(&account, &kind))
        })
    }

    fn ledger_put(
        &self,
        account: String,
        kind: String,
        value: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), proto::WireError>> + Send + '_>>
    {
        Box::pin(async move {
            self.check_account(&account, None)?;
            self.store
                .plugin_set_ledger(&account, &kind, &value)
                .map_err(|message| proto::WireError::with_code(message, proto::WireError::INTERNAL))
        })
    }

    fn settings_get(
        &self,
        provider: String,
        key: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, proto::WireError>> + Send + '_>,
    > {
        Box::pin(async move {
            self.check_provider(&provider)?;
            // ── ** already has ** （ not is set）─────────────
            // 「」 and in `config.json` 
            // （set「 → by 」 that 1 ， use in that ）。
            // up after ， that in ，must can to 1 
            // use config —— otherwise up 、 request not ， is that 
            // 「 no has 」 1 。
            //
            // use **** not is config to ： need need need 
            // that ， this 。
            match key.as_str() {
                super::rpc_cache::SETTING_MODEL_CATALOG => {
                    return Ok(super::rpc_cache::snapshot(&provider)
                        .and_then(|list| serde_json::to_value(list).ok())
                        .unwrap_or(serde_json::Value::Null));
                }
                crate::server::config::PLUGIN_SETTING_GATEWAY_PROMPT => {
                    let enabled = crate::server::config::gateway_prompt_enabled(&provider);
                    return Ok(serde_json::Value::Bool(enabled));
                }
                crate::server::config::PLUGIN_SETTING_GATEWAY_PROMPT_TEXT => {
                    let blocks = crate::server::config::gateway_prompt_text(&provider);
                    return Ok(match blocks {
                        Some(blocks) => serde_json::json!({
                            "identity": blocks.identity,
                            "stable": blocks.stable,
                            "dynamic": blocks.dynamic,
                        }),
                        None => serde_json::Value::Null,
                    });
                }
                _ => {}
            }
            let path = plugin_settings_key(&provider, &key);
            let value = crate::server::config::with_raw(|raw| raw.get(&path).cloned())
                .flatten()
                .unwrap_or(serde_json::Value::Null);
            Ok(value)
        })
    }

    fn settings_set(
        &self,
        provider: String,
        key: String,
        value: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), proto::WireError>> + Send + '_>>
    {
        Box::pin(async move {
            self.check_provider(&provider)?;
            let path = plugin_settings_key(&provider, &key);
            crate::server::config::update_raw_field(&path, value);
            Ok(())
        })
    }

    fn log(&self, level: String, message: String) {
        super::logs::receive(&self.store, &level, &message, serde_json::Value::Null);
    }

    fn log_context(&self, level: String, message: String, context: serde_json::Value) {
        super::logs::receive(&self.store, &level, &message, context);
    }

    fn notify(&self, kind: String, payload: serde_json::Value) {
        if kind == "models_changed" {
            super::model_sync::notify(&payload);
            return;
        }
        logging::log("Plugin", &format!("通知 {kind}: {payload}"));
    }
}

/// set before （`plugin.<provider>.<key>`）。
///
/// before not is use `key`： is ， can can ，
/// will and config（`apiKey` / `routing` that ）。
fn plugin_settings_key(provider: &str, key: &str) -> String {
    format!("plugin.{provider}.{key}")
}

/// （`bootstrap` ；failure）。
///
/// ── return Why is `Option` ────────────────────────────
/// no is ****（）。return `None` when use 
/// not use 。
pub async fn start(store: AccountStore) -> Option<Arc<Host>> {
    let registry_file = PackRegistry::load();
    let owners = super::ownership::owners(&registry_file);
    let mut first = None;
    for pack in registry_file.packs.iter().filter(|pack| pack.enabled) {
        if let Some(host) = start_pack(store.clone(), pack, &owners).await {
            if first.is_none() {
                first = Some(host);
            }
        }
    }
    if first.is_some() {
        crate::server::core::providers::adapter::restore_cached_catalogs();
        if crate::server::core::checkin_schedules::extend_plugin_providers_once() {
            logging::log("[Checkin]", "新安装插件的每日签到提供商已补入已有签到范围");
        }
        super::schedules::wake();
    }
    first
}

pub(super) async fn start_pack(
    store: AccountStore,
    pack: &install::InstalledPack,
    owners: &std::collections::BTreeMap<String, String>,
) -> Option<Arc<Host>> {
    // path and in directory。 not not —— path
    // has 1 （`InstalledPack::entry`），**must**，otherwise 1 
    // old will in when new use 。
    if let Some(reason) = super::compatibility::version_error(&pack.id, &pack.version) {
        logging::log("[Plugin]", &reason);
        return None;
    }
    let dir = install::plugins_dir().join(&pack.id).join(&pack.version);
    let manifest = match install::read_manifest(&dir) {
        Ok(manifest) => {
            if let Err(reason) = install::validate_manifest(&manifest, proto::PROTOCOL_VERSION) {
                logging::log("Plugin", &format!("插件包「{}」不可用：{reason}", pack.id));
                return None;
            }
            Some(manifest)
        }
        Err(reason) => {
            logging::log("Plugin", &format!("插件包「{}」清单读不出来：{reason}", pack.id));
            return None;
        }
    };

    // ── path：， ────────────────────────────
    // is by （ 1 can ），
    // when that 1 。 as can 「 1 in 1 up 」
    // for file； that is not to when 。
    let entry = manifest
        .as_ref()
        .and_then(|manifest| install::entry_for_target(manifest, &super::rpc::target_triple()))
        .unwrap_or_else(|| pack.entry.clone());
    let exe = dir.join(&entry);
    if !exe.exists() {
        logging::log(
            "Plugin",
            &format!("插件入口不存在（{}），跳过", exe.display()),
        );
        return None;
    }
    if let Err(reason) = super::packages::ensure_entry_executable(&exe) {
        logging::log("Plugin", &format!("插件包「{}」不可用：{reason}", pack.id));
        return None;
    }

    let options = HostOptions {
        cwd: exe.parent().map(std::path::Path::to_path_buf),
        data_dir: install::plugins_dir().join(&pack.id).display().to_string(),
        init_timeout: Duration::from_secs(30),
        loopback_port: loopback_port(),
        ..HostOptions::new(exe)
    };
    // by and currentall； not can only init any ID。
    // ID can and set（disabled ）， and ID not 。
    let allowed: Vec<String> = owners.iter()
        .filter(|(_, owner)| *owner == &pack.id)
        .map(|(provider, _)| provider.clone())
        .collect();
    let callbacks = Arc::new(StoreCallbacks::for_providers(store, allowed));
    let host = Arc::new(Host::new(options, callbacks));
    super::runtime::install_pack(&pack.id, super::ownership::category(&dir), host.clone());
    match host.ensure().await {
        Ok(providers) => {
            let enabled_ids: Vec<String> = providers.iter().map(|info| info.id.0.clone())
                .filter(|id| !crate::server::core::providers::availability::is_suspended(id)
                    && owners.get(id).map(String::as_str) == Some(pack.id.as_str())
                    && pack.providers.get(id).copied().unwrap_or(true)).collect();
            super::runtime::set_providers(&pack.id, enabled_ids);
            let mut loaded = Vec::new();
            let mut adapters = Vec::new();
            let mut restored = Vec::new();
            for info in providers {
                let id = info.id.0.clone();
                if crate::server::core::providers::availability::is_suspended(&id) {
                    logging::log("Plugin", &format!("提供商 {id} 已暂停，跳过执行体登记"));
                    continue;
                }
                if owners.get(&id).map(String::as_str) != Some(pack.id.as_str()) {
                    logging::log("Plugin", &format!("提供商 {id} 不归插件包「{}」接管，跳过", pack.id));
                    continue;
                }
                // provider （ use can 1 ）
                if !pack.providers.get(&id).copied().unwrap_or(true) {
                    logging::log("Plugin", &format!("提供商 {id} 已被用户停用，跳过"));
                    continue;
                }
                if !info.admin_panels.is_empty() {
                    super::schedules::load(&host, &id).await;
                }
                let kind = plugin_kind(&id);
                if let Some((models, at)) = super::rpc_cache::set_defaults(&id, info.default_models.clone()) {
                    restored.push(format!("{id}（{models} 个，{}）", crate::server::core::providers::catalog_cache::age_text(at)));
                }
                let mut adapter = RpcAdapter::new(id.clone(), kind, host.clone(), info);
                adapter.load_image_capability().await;
                adapters.push((id.clone(), Arc::new(adapter)));
                loaded.push(id);
            }
            if !host.ready() { return None; }
            registry::stage_all(adapters);
            let count = loaded.len();
            super::runtime::set_providers(&pack.id, loaded);
            logging::log(
                "Plugin",
                &format!("插件包「{}」v{} 已加载，{} 家提供商可用", pack.id, pack.version, count),
            );
            if !restored.is_empty() {
                logging::log("[Models]", &format!(
                    "📦 已从缓存恢复 {} 份插件模型清单（{}）——刷新成功后覆盖",
                    restored.len(), restored.join("、"),
                ));
            }
            Some(host)
        }
        Err(reason) => {
            logging::log("Plugin", &format!("插件包「{}」启动失败：{reason}", pack.id));
            None
        }
    }
}

/// empty ； in use old request not can 。
pub async fn reload(store: AccountStore) -> Option<Arc<Host>> {
    let _mutation = super::operations::mutation_guard().await;
    let registry = PackRegistry::load();
    let mut ids = super::runtime::loaded_ids();
    ids.extend(registry.packs.iter().map(|pack| pack.id.clone()));
    ids.sort();
    ids.dedup();
    for id in ids {
        let scope = super::switching::ChangeScope::between(&registry, &registry, &id, None);
        let Some(switch) = scope.begin() else {
            logging::log("[Plugin]", &format!("插件包「{id}」仍有任务，未执行重载"));
            continue;
        };
        if let Err(error) = scope.stop().await {
            logging::log("[Plugin]", &error);
            continue;
        }
        switch.clear_adapters();
        if let Err(error) = scope.start(&store).await { logging::log("[Plugin]", &error); }
        drop(switch);
    }
    super::runtime::current()
}

/// （`ServerState::bootstrap` when 1 ）。
///
/// ── Why not is ────────────────────
/// `spawn_bootstrap` use in `ServerState::bootstrap` ， that ** has **；
/// will 「 is optional 」 this 1 none 
/// （ no when that parameternone）。 its loopback is 
/// 1 （`providers::*::oauth::set_loopback_port`）， this 。
static LOOPBACK_PORT: std::sync::OnceLock<u16> = std::sync::OnceLock::new();

/// （ use none）。
pub fn set_loopback_port(port: u16) {
    let _ = LOOPBACK_PORT.set(port);
}

/// （ when as `0`，「 not to 」）。
fn loopback_port() -> u16 {
    LOOPBACK_PORT.get().copied().unwrap_or(0)
}

/// provider `ProviderKind` 。
///
/// ── Why not can ─────────────────────────────────────
/// `ProviderAdapter::kind()` return is ， has 370 by 
/// 。 id can can is （） can can is new （）。
/// id return kind（ this that ）； new id no has 
/// for ， can to 1 —— use **must** use `adapter.provider_id()`
/// not is `kind_id(kind())` （ trait up ）。
///
/// `WorkBuddy`： is 1 ，`meta()` is ， as up 
/// 「 no config」。P6 370 after this 。
fn plugin_kind(id: &str) -> ProviderKind {
    crate::server::core::providers::kind_from_id(id).unwrap_or(ProviderKind::WorkBuddy)
}

/// after （`bootstrap` use this ）。
///
/// ── Why not in use up await ──────────────────────────────
/// need 、 can can need 1 directory。`bootstrap` is ，
/// after has 1 「」 —— etc 1 optional
/// is no 。
pub fn spawn(store: AccountStore) {
    crate::spawn_task(async move {
        let _guard = super::operations::mutation_guard().await;
        let _ = start(store).await;
        super::catalog::spawn();
    });
}
