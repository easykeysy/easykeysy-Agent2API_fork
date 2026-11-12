//! config and when 。
//!
//! and Node `loadConfig` / `saveConfig` / `applyConfig` for （server.mjs 242-264 ）。
//!
//! ── ： 1 `kv` （ from config.json ）────────
//! before is 1 `{config_dir}/config.json`； in ** is `kv`
//! 1 **（ and configfile 1 not ）， is this that JSON 。
//! is 1 （ `RESERVED_KV_KEYS` ）， is 1 
//! 「 UPSERT + new config already no has 」。 old file by 
//! `db::migrate::import_config` 1 。
//!
//! ── 「unknown」 this not in down ──────────
//! before ： JSON for is **** ， not struct ， 1 is 
//! `serde_json::Map`； can that ， when ，**unknown
//! from up not can can **。
//! after this not ** 1 ， not **：
//! - **all**（ not ）→ `raw` unknown；
//! - by `raw` UPSERT → unknown****；
//! - 1 will delete is 「 not in `raw` that 」， `raw` is ——
//! 「 not 」 not will 。
//! is ：**unknown not 「」 this ，「 +
//! already 」**。 this is down 1 ， `read_conn`
//! or `stale_keys` will —— has to this 。
//! 1 after ：`providerRoute` this ** already down in config** ，
//! will 、， not will as no use 。
//!
//! ── and kv （ 1 ）─────────────────────────
//! config ** not can and `db::schema::RESERVED_KV_KEYS` **： after is 
//! config 「config already 」。current for 
//! （ and for `db::schema` module header）。
//!
//! ── can ──────────────────────────────────────────────
//! Node config `opts` for ，requestpath not ；Rust use 
//! `RwLock<Option<RuntimeConfig>>` has 1 「current」， is 
//! POST /api/config API Key after in （ not 、 not ）。
//!
//! ：config > > default。
//! Note `WORKBUDDY_PROXY_API_KEY` is 「 when 」 —— will ，
//! POST /api/config null can （ for Node `opts.apiKey = null`）。
//!
//! ── when （ 1 ）─────────────────────
//! `init` in `bootstrap` **** use ， and must：`db::migrate` 
//! `import_logs` / `import_requests` / `import_debug` need `storage_dirs()` and 
//! `retention_settings()`。config in 、 need `Db::open` after in ——
//! `read_raw` use 「、 need when old file」 this ， that 。
//!
//! ── file ────────────────────────────────────────────────
//! ```text
//! server/config/
//! mod.rs and （file）
//! types.rs 、set、default and （ for ）
//! parse.rs from raw ：、、
//! sql.rs `kv` （ 1 SQL ）
//! ```
//!
//! configdirectory（`~/.agent2api`） in `crate::paths::config_dir`，
//! ； from 1.x up 1 directory in `config_migration`，
//! this old directory and old directorypath（`LEGACY_DIR_NAME` is 
//! 1 ）。

use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde_json::{Map, Value};

use crate::server::db::Db;

mod account_sort;
mod assistant_config;
pub(crate) mod compact_routes;
pub(crate) mod client_identification;
mod github_access;
mod mutations;
pub(crate) use mutations::try_update_raw_field;
pub(crate) mod parse;
pub(crate) mod routing_groups;
mod routing_profiles;
pub(crate) mod routing_workspace;
mod sanitize_config;
pub(crate) mod sql;
mod types;
mod update_policy;

pub use account_sort::{
    account_sort_settings, save_account_sort, validate_account_sort, AccountSortRule, AccountSortSettings,
    AddedSortDirection, CreditSortDirection,
};
pub use assistant_config::*;
pub use github_access::{
    github_access, set_github_access, GithubAccessMode, GithubAccessPatch,
    GithubAccessSettings, DEFAULT_GITHUB_MIRROR,
};
pub use sanitize_config::{
    set_sanitize_settings, SanitizePatch, SanitizeSettings, KEY_SANITIZE_PROVIDERS,
    KEY_SANITIZE_RULES, MAX_SANITIZE_PROVIDERS,
};
pub use routing_groups::{RoutingGroup, RoutingMember};
pub use routing_profiles::{
    normalize_model as normalize_routing_model, routing_profiles, routing_snapshot, save_routing_profile,
    settings_for_model as routing_settings_for_model,
    validate_shape as validate_routing_profile, RoutingProfile, RoutingProfiles, RoutingProviderScope,
    KEY_ROUTING_PROFILES,
};
pub use types::*;
pub use update_policy::{
    set_update_channels, update_beta, update_channels, UpdateChannels, UpdateChannelsPatch,
    KEY_UPDATE_STABLE,
};
pub(crate) use update_policy::{sync_installed_update_channel, KEY_INSTALLED_UPDATE_VERSION};

use parse::*;

/// config。
///
/// is 「 will use to 」， its unknown in `raw` ，
/// when 1 。
#[derive(Clone, Debug, Default)]
pub struct RuntimeConfig {
    /// old `apiKey` ****（ from can can is `enc1:` ）。
    /// 1 is `masked_api_key`， in that —— Why not in ，
    /// that 。
    api_key: Option<String>,
    locale: String,
    default_model: String,
    last_request_model: Option<String>,
    /// set（ 1 ；request / by already ）。
    ///
    /// Why not is use `raw` ： need ****
    /// use （）， from `Value` need not 、、，
    /// in this 1 can ；`raw` is when 1 。
    retention: RetentionSettings,
    /// when and （set「 when 」）。
    ///
    /// and 1 by ： and new ** 1 need **
    /// （set down 1 ， not ）， from `Value` 1 need 1 
    /// and ， 1 down 。
    scheduled: ScheduledSettings,
    /// request and （set「 use → request」）。
    ///
    /// and 1 by ：**** need （set down 1 
    /// failurerequest use new ， not ）， 1 down 。
    retry: RetrySettings,
    /// up request when （set「 use → request when 」）。
    ///
    /// and retry 1 by ：** / ** need （set
    /// down 1 request use new ， not ）， 1 down 。
    timeouts: TimeoutSettings,
    /// etc and when （set「 use → etc 」）。
    ///
    /// and timeouts 1 by ： ** to ** need 
    /// （set down 1 request use new ， not ）， 1 down 。
    queue: QueueSettings,
    /// ZCode （set「 → 」）。
    ///
    /// and queue 1 by ：stats ** ** need （set
    /// down 1 ， not ）， 1 down 。
    captcha_pool: CaptchaPoolSettings,
    /// will （set「 → 」）。
    ///
    /// and timeouts 1 by ：** before ** need （set down 1 
    /// request， not ）， 1 down 。default，
    /// `KEY_IMAGE_COMPRESS_ENABLED` 。
    image_compress: ImageCompressSettings,
    /// by / / / / （「 by 」）。
    ///
    /// and timeouts 1 by ：**** need （set down 1 
    /// request， not ）， 1 down 。
    routing: RoutingSettings,
    routing_profiles: std::sync::Arc<RoutingProfiles>,
    compact_routes: std::sync::Arc<compact_routes::CompactRoutesSnapshot>,
    client_identification: std::sync::Arc<client_identification::ClientIdentificationSnapshot>,
    account_sort: std::sync::Arc<AccountSortSettings>,
    model_rules: std::sync::Arc<crate::server::core::model_rules::ModelRules>,
    /// directory（config；None = set， use configdirectory）。
    /// （ + set）， not as ， can 。
    log_dir: Option<String>,
    /// request directory（config；None = set）
    request_stats_dir: Option<String>,
    /// directory（config；None = set）
    debug_dir: Option<String>,
    /// （set「 use → 」）。
    ///
    /// and 1 by ：** before ** need 1 （ down 1 request
    /// ， not ）， from `Value` 1 need ， 1 down 
    /// —— this in path up 。
    debug_mode: bool,
    /// then in config / when ，request， 1 use 。
    sanitize: SanitizeSettings,
    /// （set「 → 」）。
    ///
    /// and `sanitize_fingerprints` 1 by ：** before ** need 1 
    /// （ down 1 request， not ）， 1 down 。
    /// and default `KEY_DEVELOPER_ROLE_COMPAT`。
    developer_role_compat: bool,
    /// **Cline **（set「Cline 」）。
    ///
    /// is ** use **（），default in 
    /// `core::providers::cline::headers`（ and client for that 1 ）。
    /// and `debug_mode` 1 by ：Cline ** up request** need 
    /// （ down 1 request， not ）， 1 down 。
    /// ：empty = by /add；empty = this not （delete）。
    cline_upstream_headers: std::collections::BTreeMap<String, String>,
    /// 网关面（`/v1/*`）（set「 → 」）。
    ///
    /// and `debug_mode` 1 by ：CORS in request 1 （ down 1 request
    /// ， not ）， 1 down 。default `false`， `KEY_CORS_ENABLED`。
    /// used for， by not 。
    cors_enabled: bool,
    /// AI config（「set」：default as / / ）。
    ///
    /// and `prompt` 1 ： + （ has set and for 
    /// ）， 1 down can 。 in 1 config down （
    /// `assistant_config` module header）。
    assistant: AssistantConfig,
    /// 「」（`/api/keepalive-strip`，default）。
    ///
    /// and `sanitize_fingerprints` 1 by ： down when need 1 ，
    /// 1 down 。** not can **： not is 
    /// 「 × 」 and （
    /// [`crate::server::core::upstream::sse::FramePolicy::assembled`]）。
    strip_newline_keepalive: bool,
    /// 「」（`/api/auto-continue`，default）。
    ///
    /// and `strip_newline_keepalive` 1 by ：** request** need 1 
    /// 「 this 1 need not need 」（ down 1 request， not ），
    /// 1 down 。 and in 
    /// [`crate::server::core::upstream::continuation`]，。
    auto_continue: bool,
    /// limit（`autoContinueMaxRounds`，default 3、 1~8）。
    ///
    /// and `auto_continue` 1 by （request 1 ）， and **must and 1 **：
    /// this is that （ is ），default
    /// （ not ）， by `api::auto_continue` 400 —— use 
    /// `AUTO_CONTINUE_MIN/MAX_ROUNDS`。
    ///
    /// by `build()` to ： `Default` is ， that 0 and 
    /// `locale` / `default_model` that empty 1 ，** not is **，
    /// `Debug` and （default `auto_continue_max_rounds()`
    /// that ， when will `parse` ）。
    auto_continue_max_rounds: i64,
    /// （set「 → 」，ALTCHA proof-of-work）。
    ///
    /// and `debug_mode` 1 by ： / request 1 （ down 1 
    /// request）， 1 down 。default `true`， `KEY_CAPTCHA_ENABLED`；
    /// config when can by `AGENT2API_CAPTCHA_ENABLED` （default 1 、0 ）。
    ///
    /// ** **： and headless use 1 after ， not ——
    /// is （ can can ），headless is 。 ，
    /// use by `ServerState.local_panel` for that 1 。
    captcha_desktop: bool,
    captcha_web: bool,
    /// models.dev （set「 use → models.dev 」）。
    ///
    /// is 「 this can / can not can by 」 ****total，default `true`
    /// （ `KEY_MODELS_DEV_ENABLED`）。（ in new when 
    /// 1 ）， and `captcha_enabled` 1 1 down 。
    models_dev_enabled: bool,
    /// set（set「 use → 」）。
    ///
    /// and `sanitize_fingerprints` 1 by （request 1 ， down 1 request
    /// ）， 1 ：**file in when **（ `prompt_from`），
    /// is path up 1 IO no has ；file not to when this already is 「default +
    /// 1 Reason」 ， not failurepath。
    prompt: PromptSettings,
    /// 「update」 （`updateProxy`：None = ，default）。
    ///
    /// is 1 after config for （ and 1 ）， to 
    /// `core::proxies::resolve_account_proxy`（ 1 ）。（
    /// check / down when 1 ）， not as ， can 。
    update_proxy: Option<Value>,
    /// and beta ； and currentnone。
    update_channels: UpdateChannels,
    /// GitHub ****（`githubToken`：None = in ）。
    ///
    /// Note this from config already is `enc1:` ，** not 
    /// config **（ in `core::update::token` ），
    /// not in this 。 and update_proxy as ， can 。
    github_token: Option<String>,
    /// up that JSON for （unknown）， when 
    raw: Map<String, Value>,
}

impl RuntimeConfig {
    /// current is in enabled Key config（ old and ）； not can 。
    pub fn api_key_set(&self) -> bool {
        !self.active_api_keys().is_empty()
    }

    /// currentenabled Key config； not can ， as none Key。
    pub fn active_api_keys(&self) -> Vec<String> {
        crate::server::core::api_keys::active_keys_from(&self.raw)
    }

    /// （Accept-Language）
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// default
    pub fn default_model(&self) -> &str {
        &self.default_model
    }

    /// 1 （「」 default）
    pub fn last_request_model(&self) -> Option<&str> {
        self.last_request_model.as_deref()
    }

    /// is （ before 1 ，）
    pub fn debug_mode(&self) -> bool {
        self.debug_mode
    }

    /// is （ before 1 ，）
    pub fn sanitize_fingerprints(&self) -> bool {
        self.sanitize.enabled
    }

    /// is （ before 1 ，）
    pub fn developer_role_compat(&self) -> bool {
        self.developer_role_compat
    }

    /// will set（ before 1 `Copy` ，）
    pub fn image_compress_settings(&self) -> &ImageCompressSettings {
        &self.image_compress
    }

    /// （`/v1/*`） is （CORS in request 1 ，）
    pub fn cors_enabled(&self) -> bool {
        self.cors_enabled
    }

    /// 「」（ and can and after ，）
    pub fn strip_newline_keepalive(&self) -> bool {
        self.strip_newline_keepalive
    }

    /// 「」（
    /// [`crate::server::core::upstream::continuation`]，）
    pub fn auto_continue(&self) -> bool {
        self.auto_continue
    }

    /// limit（ when already by `AUTO_CONTINUE_MIN_ROUNDS..=AUTO_CONTINUE_MAX_ROUNDS`
    /// ，）
    pub fn auto_continue_max_rounds(&self) -> i64 {
        self.auto_continue_max_rounds
    }

    /// （ / request 1 ）。
    ///
    /// `local_panel = true` that ，otherwise that —— and 
    /// `ServerState.local_panel` 1 （ true，headless false）。
    pub fn captcha_enabled(&self, local_panel: bool) -> bool {
        if local_panel {
            self.captcha_desktop
        } else {
            self.captcha_web
        }
    }

    /// （set use ）。
    pub fn captcha_settings(&self) -> (bool, bool) {
        (self.captcha_desktop, self.captcha_web)
    }

    /// models.dev ** new **（`core::model_meta::refresh` ）。
    pub fn models_dev_enabled(&self) -> bool {
        self.models_dev_enabled
    }

    /// set（ / use ； need is down use ）
    pub fn prompt_settings(&self) -> &PromptSettings {
        &self.prompt
    }

    /// need ****：default + use 。
    ///
    /// use not is ： can can has ， in ** request** up use 1 
    /// （`upstream::forward` ）， 1 。 in 
    /// `&self` up —— use must（ `upstream::forward`
    /// that `config` ）。
    pub fn prompt_plan(&self) -> crate::server::core::prompt::PromptPlan<'_> {
        crate::server::core::prompt::PromptPlan::new(
            crate::server::core::prompt::PromptChoice {
                mode: self.prompt.mode,
                text: &self.prompt.text,
                source: self.prompt.source,
            },
            &self.prompt.providers,
        )
    }

    /// after API Key， server.mjs 920 ： before 6 after 4。
    /// key will before after —— Node slice(0,6)/slice(-4) is this ， 1 。
    ///
    /// ── Why in ** this **， not in or ─────────────
    /// `apiKey` from can can is `enc1:` （ old 
    /// `core::legacy_api_key`）， before must by `api_keys` ——
    /// ， up will `enc1:b...xxxx`， Key 。
    /// in this 1 not is `build`：`build` in `config::init` ， that when 
    /// no （ `save_raw` ），「file、 this Key not 」 that 
    /// 1 will in up （ down use not to ）； by 
    /// （GET /api/config） use ， that when already 。 not 。
    pub fn masked_api_key(&self) -> Option<String> {
        let stored = self.api_key.as_ref().filter(|key| !key.is_empty())?;
        let key = crate::server::core::api_keys::stored_plaintext(stored);
        let head: String = key.chars().take(6).collect();
        let total = key.chars().count();
        let tail: String = key.chars().skip(total.saturating_sub(4)).collect();
        Some(format!("{head}...{tail}"))
    }

    /// JSON （ after use ）
    pub fn raw(&self) -> &Map<String, Value> {
        &self.raw
    }
}

/// configdirectory：`paths::config_dir` is 1 （ `gateway::config_dir` this ），
/// 「 key」 and 「server key」 to 1 directory，path need paths 1 。
pub fn config_dir() -> PathBuf {
    crate::paths::config_dir()
}

/// config.json path
pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

/// old configdirectory（onlyused for 1 directory）。
///
/// ** this is 1 1 `.workbuddy-proxy` ** ——
/// path 1 `config_dir()`（ in `gateway::config_dir`），
/// otherwise will 。
const LEGACY_DIR_NAME: &str = ".workbuddy-proxy";

/// old configdirectory path（`{ use directory}/.workbuddy-proxy`）， use 。
pub(crate) fn legacy_config_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(LEGACY_DIR_NAME)
}

// （ 1 `config_migration::migrate_config_dir`） in 
// `server/config_migration.rs`； up this old directorypath，
// `.workbuddy-proxy` has 1 。

/// up `providerRoute` ：`[(providerId, )]`，file
/// that 。 「 by 」 old and when ，
/// use old （ by ， `store_admin`）。
///
/// not ： this is 1 use ， and can can 
/// `config::init`， not 。
pub fn legacy_provider_route() -> Vec<(String, u32)> {
    let raw = read_raw();
    let Some(Value::Object(table)) = raw.get(KEY_PROVIDER_ROUTE) else {
        return Vec::new();
    };
    table
        .iter()
        .filter_map(|(id, value)| number_u32(value).map(|rank| (id.clone(), rank)))
        .collect()
}

/// JSON → u32：（）/ ， and has not 。
fn number_u32(value: &Value) -> Option<u32> {
    let number = match value {
        Value::Number(number) => number.as_f64()?,
        Value::String(text) => text.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    if !number.is_finite() || number < 0.0 {
        return None;
    }
    Some(number.min(u32::MAX as f64) as u32)
}

/// config： 1 ，** need when old file**。
///
/// ── Why need need （ when ： has is has ）─────────────────
/// `config::init()` in `bootstrap` **** `Db::open` use ， and must
/// ：`db::migrate` `import_logs` / `import_requests` / `import_debug`
/// need `storage_dirs()`（ old file directory） and `retention_settings()`
/// （），config。， that will defaultdirectory
/// old file（ not to → use in old file） and by default 30 1 
/// （** not can old **）。configmust in before 。
/// config in ， need `Db::open` after in —— this is that 。
///
/// is **path**， not is ：
/// - `db` already → from （path， 1 ）；
/// - `db` / unavailable / ** no ** → old 
/// `{config_dir}/config.json`， and before as 。
/// is ： new when no has config， to is empty for （ old file not in ），
/// and before 1 ； use after 1 when ， to is old config ——
/// is no ， already is ，、 old filedirectory for 。
/// old file and `.migrated` after ， this 1 
/// （file no ）， down that 。
///
/// Why use 「」 not is 「 has no has config」 need not need ： 1 
/// config will config，** no **（ old file can can has no ）。
/// and in 1 （ `sql::import_conn`），「 in 」
/// etc 「 this old file already 」， is 1 can 。
fn read_raw() -> Map<String, Value> {
    if let Some(db) = db() {
        if let Some((raw, migrated)) = sql::read_snapshot(&db) {
            // ： can can has ， its in old file，
            // need （ old file ， is use old config）。
            // after （ in ） as 1 ， not file。
            if migrated {
                return raw;
            }
            return merge_legacy(raw);
        }
    }
    // not （ / in / SQL ）： old file， and before 1 。
    // not in this ： this in will use ， use （`init`）
    // already can `db()` is as empty。
    read_legacy_file()
}

/// old `{config_dir}/config.json`（ / / not empty for ，
/// for Node `loadConfig` catch ）。
///
/// in to ：`db` （ / not ）， and 
/// （ when is use old config in ）。 after this file already 
/// `.migrated`， not to —— that is 「 not this 」 。
fn read_legacy_file() -> Map<String, Value> {
    let Ok(text) = std::fs::read_to_string(config_file()) else {
        return Map::new();
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    }
}

/// and 「 」 and 「 old file 」： old file（ when ）。
///
/// has `kv` and old file when has will use to （ 1 、 or use in before 
/// config）。 old file by ： is use **** that config， can can is 
/// ； ， use will old file 。
fn merge_legacy(store: Map<String, Value>) -> Map<String, Value> {
    let legacy = read_legacy_file();
    if legacy.is_empty() {
        return store;
    }
    let mut merged = store;
    for (key, value) in legacy {
        merged.insert(key, value);
    }
    merged
}

/// by + config（ for Node applyConfig ）
fn build(raw: Map<String, Value>) -> RuntimeConfig {
    let retention = retention_from(&raw);
    let scheduled = scheduled_from(&raw);
    let retry = retry_from(&raw);
    let timeouts = timeouts_from(&raw);
    let queue = queue_from(&raw);
    let captcha_pool = captcha_pool_from(&raw);
    let image_compress = image_compress_from(&raw);
    let routing = routing_from(&raw);
    RuntimeConfig {
        // file has use file ，otherwise（ for `if (config.apiKey && !opts.apiKey)`）。
        // this is ****：`apiKey` from can can is `enc1:` （ old 
        // `core::legacy_api_key`）。 in 
        // `masked_api_key` —— Why not in ， that 。
        api_key: string_field(&raw, "apiKey").or_else(env_api_key),
        locale: env_text("WORKBUDDY_LOCALE")
            .or_else(|| string_field(&raw, "locale"))
            .unwrap_or_else(|| DEFAULT_LOCALE.to_string()),
        default_model: env_text("WORKBUDDY_DEFAULT_MODEL").unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        last_request_model: string_field(&raw, "lastRequestModel"),
        retention,
        scheduled,
        retry,
        timeouts,
        queue,
        captcha_pool,
        image_compress,
        routing,
        routing_profiles: routing_profiles::from_raw(&raw),
        compact_routes: compact_routes::from_raw(&raw),
        client_identification: client_identification::from_raw(&raw),
        account_sort: account_sort::from_raw(&raw),
        model_rules: std::sync::Arc::new(crate::server::core::model_rules::ModelRules::from_raw(&raw)),
        log_dir: string_field(&raw, KEY_LOG_DIR),
        request_stats_dir: string_field(&raw, KEY_REQUEST_STATS_DIR),
        debug_dir: string_field(&raw, KEY_DEBUG_DIR),
        // has `true` （file "1" / "yes" 1 ）： and 
        // 「」 1 to —— this is ， can 
        debug_mode: raw.get(KEY_DEBUG_MODE).and_then(Value::as_bool).unwrap_or(false),
        sanitize: sanitize_config::from_raw(&raw),
        // has `false` ：**default**（ → true）。none
        // （developer and system in OpenAI up etc ），default will developer
        // client（ new OpenAI SDK、Claude Code etc ） 1 up not this 
        // up —— and sanitize_fingerprints 1 to ：default in use 
        // 1 request up 。
        developer_role_compat: raw
            .get(KEY_DEVELOPER_ROLE_COMPAT)
            .and_then(Value::as_bool)
            .unwrap_or(true),
        // Cline ： string → string （/
        // ignore —— will is ）， = default
        cline_upstream_headers: cline_headers_from(&raw),
        // has `true` ：**default**（ → false）。 `*` 
        // etc 「 up 、」 can to ， can use 
        // （ KEY_CORS_ENABLED ）； and sanitize 「default」 to ，
        // as default not 。
        cors_enabled: raw
            .get(KEY_CORS_ENABLED)
            .and_then(Value::as_bool)
            .unwrap_or(false),
        // AI config： and cline_upstream_headers 1 （、
        // not ），default assistant_config module header
        assistant: assistant_config::from_raw(&raw),
        // has `true` ：**default**。 and debug_mode 1 to （ 
        // is 「 1 no 」）， and sanitize_fingerprints
        // （ that default will new use 1 up 400）。 this default by 
        // KEY_STRIP_NEWLINE_KEEPALIVE ： will 「 」。
        // config when （Docker 1 can ）。
        strip_newline_keepalive: raw
            .get(KEY_STRIP_NEWLINE_KEEPALIVE)
            .and_then(Value::as_bool)
            .unwrap_or_else(env_strip_newline_keepalive),
        // has `true` / `false` this 1 ：**default**。 and up that 1 to 
        // （ is client can as ）， to not ： this is ** up **、
        // is ， need can no 。（file > 
        // `AGENT2API_AUTO_CONTINUE` > ） in `parse::auto_continue_from`，
        // and down that 1 —— is 「」， in parse.rs。
        auto_continue: auto_continue_from(&raw),
        // limit：file > `AGENT2API_AUTO_CONTINUE_MAX` > default 3。
        // ** not is **（ and `bounded_int_field` 1 ）： 99 
        // this to 99 no ， not is by to 8 —— after will as 
        // 「8 limit」 is to 。
        auto_continue_max_rounds: auto_continue_rounds_from(&raw),
        // has `false` ：**default**。 / and 
        // can 1 not can 1 （ KEY_CAPTCHA_ENABLED ）。
        //
        // （ → old → → default）： before use 
        // `captchaEnabled`， that must when as new default —— otherwise use 
        // will in after 。`env_captcha_enabled` 。
        captcha_desktop: captcha_switch_from(
            &raw,
            KEY_CAPTCHA_DESKTOP,
            env_captcha_enabled(),
        ),
        captcha_web: captcha_switch_from(&raw, KEY_CAPTCHA_WEB, env_captcha_enabled()),
        // has `false` ：**default**（ by KEY_MODELS_DEV_ENABLED）。
        // no has ： not is 「 when need not need 」 （Docker and 
        // 1 ， can in when has ）。
        models_dev_enabled: raw
            .get(KEY_MODELS_DEV_ENABLED)
            .and_then(Value::as_bool)
            .unwrap_or(true),
        // ：/ → passthrough（default），file not to → default
        // + 1 Reason（ `prompt_from`）
        prompt: prompt_from(&raw),
        // update：null / not （None = ）
        update_proxy: raw
            .get(KEY_UPDATE_PROXY)
            .filter(|value| !value.is_null())
            .cloned(),
        update_channels: UpdateChannels::from_raw(&raw),
        // GitHub ：empty by set（ empty not this ）
        github_token: string_field(&raw, KEY_GITHUB_TOKEN)
            .filter(|envelope| !envelope.is_empty()),
        raw,
    }
}

/// `clineUpstreamHeaders` ：`{"<header>": "<value>"}`，。
///
/// and `gateway_text_from` ： not is ** not **
/// （ will is ， / no has ）， not is for = no 。
fn cline_headers_from(raw: &Map<String, Value>) -> std::collections::BTreeMap<String, String> {
    let mut table = std::collections::BTreeMap::new();
    let Some(object) = raw.get(KEY_CLINE_UPSTREAM_HEADERS).and_then(Value::as_object) else {
        return table;
    };
    for (key, value) in object {
        if let Some(text) = value.as_str() {
            table.insert(key.trim().to_string(), text.to_string());
        }
    }
    table
}

/// from configset（** in this file**，）。
///
/// and has config 1 ：
/// - / → `passthrough`（default）。file** not **，
/// and 「」 has to ； by `api::prompt` 400；
/// - `passthrough`： not file、 not （`text` as empty，）；
/// - `custom` / `append`：fileempty；failure（ not in / UTF-8 / emptyfile）
/// **default** and Reason `file_error` —— use not can as 1 
/// file not 、 not can ，Reason will in set up ， use can ；
/// - file → default。
fn prompt_from(raw: &Map<String, Value>) -> PromptSettings {
    use crate::server::core::prompt::PromptMode;

    let mode = raw
        .get(KEY_PROMPT_MODE)
        .and_then(Value::as_str)
        .and_then(PromptMode::parse)
        .unwrap_or_default();
    let file = string_field(raw, KEY_PROMPT_FILE);
    let inline = string_field(raw, KEY_PROMPT_TEXT);
    let base = resolve_choice(mode, file, inline);
    PromptSettings {
        mode: base.mode,
        file: base.file,
        inline: base.inline,
        text: base.text,
        source: base.source,
        file_error: base.file_error,
        providers: providers_from(raw),
        gateway: gateway_from(raw),
        gateway_text: gateway_text_from(raw),
    }
}

/// `promptGatewayText` ：`{"<providerId>": {"identity"?, "stable"?, "dynamic"?}}`。
///
/// and `gateway_from` 1 to ： not is for → this 1 ； not is →
/// this no （ use ）。 not （ use ） when ** not not ** ——
/// is ignore， old ， when in 1 。
fn gateway_text_from(
    raw: &Map<String, Value>,
) -> std::collections::BTreeMap<String, crate::server::core::prompt::GatewayBlocks> {
    let mut table = std::collections::BTreeMap::new();
    let Some(object) = raw.get(KEY_PROMPT_GATEWAY_TEXT).and_then(Value::as_object) else {
        return table;
    };
    for (id, value) in object {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        let Some(entry) = value.as_object() else { continue };
        let mut blocks = crate::server::core::prompt::GatewayBlocks::default();
        for field in crate::server::core::prompt::GatewayBlocks::FIELDS {
            if let Some(text) = entry.get(field).and_then(Value::as_str) {
                if !text.trim().is_empty() {
                    blocks.set(field, text.to_string());
                }
            }
        }
        // empty = use all：config not empty（ and 1 ）
        if !blocks.is_empty() {
            table.insert(id.to_string(), blocks);
        }
    }
    table
}

/// `promptGateway` ：`{"<providerId>": bool}`，** **。
///
/// config `"false"` this when ** not **（`as_bool()` `true`/`false`）：
/// to is 「 use as 、」， by is to default ——
/// down ， 1 no 1 。
fn gateway_from(raw: &Map<String, Value>) -> std::collections::BTreeMap<String, bool> {
    let mut gateway = std::collections::BTreeMap::new();
    let Some(object) = raw.get(KEY_PROMPT_GATEWAY).and_then(Value::as_object) else {
        return gateway;
    };
    for (id, value) in object {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        if let Some(flag) = value.as_bool() {
            gateway.insert(id.to_string(), flag);
        }
    }
    gateway
}

/// `promptProviders` ： by ** 1 **（ [`resolve_choice`]）。
///
/// config when to and 1 （、 not 、 not ）：
/// · not is for → this 1 （ no ）；
/// · `mode` / → by default `passthrough`（ and 1 `.unwrap_or_default()`）；
/// · `file` → no （ use default）；
/// · id not is → （ to ， not ——
/// use can can config、 after ）。
fn providers_from(raw: &Map<String, Value>) -> std::collections::BTreeMap<String, ProviderPrompt> {
    use crate::server::core::prompt::PromptMode;

    let mut providers = std::collections::BTreeMap::new();
    let Some(object) = raw.get(KEY_PROMPT_PROVIDERS).and_then(Value::as_object) else {
        return providers;
    };
    for (id, value) in object {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        let Some(entry) = value.as_object() else { continue };
        let mode = entry
            .get(KEY_PROMPT_MODE)
            .and_then(Value::as_str)
            .and_then(PromptMode::parse)
            .unwrap_or_default();
        let file = entry
            .get(KEY_PROMPT_FILE)
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|text| !text.trim().is_empty());
        let inline = entry
            .get(KEY_PROMPT_TEXT)
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|text| !text.trim().is_empty());
        let choice = resolve_choice(mode, file, inline);
        providers.insert(
            id.to_string(),
            ProviderPrompt {
                mode: choice.mode,
                file: choice.file,
                inline: choice.inline,
                text: choice.text,
                source: choice.source,
                file_error: choice.file_error,
            },
        );
    }
    providers
}

/// 「 + filepath + 」→ ** 1 **（default and use ）。
///
/// ： > file > default；`passthrough` 1 not （ no 
/// will that —— is not system）。
///
/// `passthrough` down ** **（`text` = use that ， is `none`）：
/// is use down ，set must —— otherwise「 ，
/// is empty 」 is 。file not in ： not this not （ 1 I/O，
/// 「 in 」）。
fn resolve_choice(
    mode: crate::server::core::prompt::PromptMode,
    file: Option<String>,
    inline: Option<String>,
) -> ResolvedChoice {
    use crate::server::core::prompt::{PromptMode, PromptSource, BUILT_IN_PROMPT};

    if !matches!(mode, PromptMode::Custom | PromptMode::Append) {
        let stored = inline.clone().filter(|text| !text.trim().is_empty());
        return ResolvedChoice {
            mode,
            file,
            text: stored.clone().unwrap_or_default(),
            inline: stored,
            source: PromptSource::None,
            file_error: None,
        };
    }
    // use in up ： that as （** not file**， is file not to not is 
    // 1 —— that file no in use ， 1 not as will ）
    if let Some(text) = inline.clone().filter(|text| !text.trim().is_empty()) {
        return ResolvedChoice {
            mode,
            file,
            inline: Some(text.clone()),
            text,
            source: PromptSource::Inline,
            file_error: None,
        };
    }
    let built_in = || ResolvedChoice {
        mode,
        file: file.clone(),
        inline: None,
        text: BUILT_IN_PROMPT.to_string(),
        source: PromptSource::BuiltIn,
        file_error: None,
    };
    let Some(path) = file.as_deref().map(str::trim).filter(|text| !text.is_empty()) else {
        return built_in();
    };
    match read_prompt_file(path) {
        Ok(text) => ResolvedChoice {
            mode,
            file,
            inline: None,
            text,
            source: PromptSource::File,
            file_error: None,
        },
        Err(error) => ResolvedChoice { file_error: Some(error), ..built_in() },
    }
}

/// [`resolve_choice`] （ and [`ProviderPrompt`] ， mode）。
struct ResolvedChoice {
    mode: crate::server::core::prompt::PromptMode,
    file: Option<String>,
    inline: Option<String>,
    text: String,
    source: crate::server::core::prompt::PromptSource,
    file_error: Option<String>,
}

/// file（UTF-8 ）；`Err` is ** can to use ** Reason。
///
/// emptyfile（ or has empty）failure： in `custom` down etc 「 not 」
/// （empty `prompt::rewrite` 「 not 」）， ——
/// by 「 no 」，default and Reason。
pub fn read_prompt_file(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("filepath as empty".to_string());
    }
    match std::fs::read_to_string(trimmed) {
        Ok(text) if text.trim().is_empty() => {
            Err(format!("file is empty ：{trimmed}（ already default）"))
        }
        Ok(text) => Ok(text),
        Err(error) => Err(format!(" not to file {trimmed}: {error}")),
    }
}

/// config：all use ， request
static CONFIG: RwLock<Option<RuntimeConfig>> = RwLock::new(None);

/// config（ when use 1 ； use will new ， etc ）。
///
/// ── Why from `init()` `Db` ───────────────────────
/// config from `config.json` 1 `kv` ，path this by `Db`
/// 1 has （`Db::file()`）。 and `LogStore::with_db` / `RequestStats::with_db`
/// / `AccountStore::with_db` 1 ， store 
/// 1 ；`Option<Db>` 1 （ not when can ， is ）。
///
/// when is `bootstrap` 1 （`Db::open` after 、 before ）——
/// Whymust， `read_raw` （/statistics/）。
/// when 1 and ；** not new **： is 「 old file 
/// 」， already from old file to （`merge_legacy`），
/// is 1 。
pub fn init(db: Option<Db>) -> RuntimeConfig {
    let db = db.filter(|db| {
        // not not this （`Db` in success when has ， this is ）
        let _ = db;
        true
    });
    let _ = DB.set(db);
    let snapshot = build(read_raw());
    crate::server::core::providers::catalog::invalidate();
    if let Ok(mut guard) = CONFIG.write() {
        *guard = Some(snapshot.clone());
    }
    crate::server::core::providers::catalog::invalidate();
    snapshot
}

/// config and （ after by use 1 ）。
///
/// Why need need ：`init` in before after to can can is ****（ before from old file
/// 、 after from ）。 in down etc （ is old file），
/// can can **failure**（ not 、）， when config and 
/// will has 。 when use 1 ， and for —— otherwise will use 1 
/// 「 no 」 ， use down to is 1 。
///
/// in when by use （`db::migrate` returnempty）；
/// no has old file when not 1 。
pub fn reload() -> RuntimeConfig {
    let snapshot = build(read_raw());
    crate::server::core::providers::catalog::invalidate();
    if let Ok(mut guard) = CONFIG.write() {
        *guard = Some(snapshot.clone());
    }
    crate::server::core::providers::catalog::invalidate();
    snapshot
}

/// （ and `logging` / `core::debug_traffic` 1 ）。
///
/// `OnceLock<Option<Db>>` not is `OnceLock<Db>`：`Db::open` can can failure，
/// `init` need `Option<Db>`（ use is ）—— in `Option` 
/// 「」 and 「unavailable」 use 1 path（ is `None`）。
static DB: std::sync::OnceLock<Option<Db>> = std::sync::OnceLock::new();

/// （ or unavailable when as `None`）
fn db() -> Option<Db> {
    DB.get().and_then(|slot| slot.as_ref()).cloned()
}

/// currentconfig 。
///
/// when by 「empty + 」 when 1 ， not will panic。
/// return not is use ： use has await and file IO。
pub fn current() -> RuntimeConfig {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.clone();
        }
    }
    build(Map::new())
}

pub(crate) fn model_rules_snapshot() -> std::sync::Arc<crate::server::core::model_rules::ModelRules> {
    if let Some(rules) = CONFIG.read().ok().and_then(|config| config.as_ref().map(|config| config.model_rules.clone())) {
        return rules;
    }
    std::sync::Arc::new(crate::server::core::model_rules::ModelRules::default())
}

/// in config 1 **、** ， `raw` （** not **）。
///
/// ── Why need need （ and up `*_enabled` 1 ）──────
/// `current()` use will `raw`（ that ）。 has 
/// need `raw` ** 1 **， in path up ： is when 
/// by provider 1 （ 1 ）。 as 
/// no need ， 1 `*_of()` will 
/// 。
///
/// ── （ has ，）────────────────────────
/// config ** is has **： `update()` will 。
/// 「、、 1 」 this —— not file / IO，
/// not get its （ has RwLock， is ）。
///
/// when return `None`（ and `current()` emptyconfig not ： this no has can ，
/// use by 「 not to 」）。
pub fn with_raw<R>(reader: impl FnOnce(&Map<String, Value>) -> R) -> Option<R> {
    let guard = CONFIG.read().ok()?;
    let config = guard.as_ref()?;
    Some(reader(config.raw()))
}

/// （** not raw**）。
///
/// and [`retention_settings`] 1 by ： this in ** `/v1/*` request** up 1 
/// （CORS in request）， `current()` will `raw` Map
/// （ that ）—— as 1 this no need 。
///
/// is not is ：`update()` after will new ，
/// 「set → down 1 request use new as 」， and not file。
/// （ up has ） when to default（）。
pub fn cors_enabled() -> bool {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.cors_enabled;
        }
    }
    false
}

/// AI config（** not raw**）。
///
/// and [`cors_enabled`] 1 by ： for and set 1 ， `current()`
/// will `raw` Map（ that ）。（ up has 
/// ） when to [`AssistantConfig::default`]。
pub fn assistant_config() -> AssistantConfig {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.assistant.clone();
        }
    }
    AssistantConfig::default()
}

/// set （** not raw**）。
///
/// Why not use use `current().retention()`： is in ****
/// up use （`LogStore::append` → ）， `current()` will 
/// `raw` Map —— path up no need 。 this 1 `Copy` 。
///
/// is not is ：`update()` after will new ，
/// 「set → down 1 use new 」， and not file。
/// （ up has ） when to default。
pub fn retention_settings() -> RetentionSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.retention;
        }
    }
    RetentionSettings::default()
}

/// ** 1 ** （** not raw**）。
///
/// Why not `current().prompt_plan()`： in 「 1 up request」 when ，
/// `current()` use `raw` Map（ that ）——
/// as 1 this no need 。 and `PromptPlan::gateway_prompt` 1 ：
/// ** not in = true（）**， this is for as 。
///
/// （ up has ） when to `true`。
pub fn gateway_prompt_enabled(provider_id: &str) -> bool {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config
                .prompt
                .gateway
                .get(provider_id.trim())
                .copied()
                .unwrap_or(true);
        }
    }
    true
}

/// ** 1 ** （** not raw**）。
///
/// and [`gateway_prompt_enabled`] 1 ： in up request when ，
/// `current()` will `raw`。：** not in = `None`（ use ）**，
/// has when empty that by use to （
/// `core::prompt::GatewayBlocks::or`）—— this not and ，config that 。
pub fn gateway_prompt_text(
    provider_id: &str,
) -> Option<crate::server::core::prompt::GatewayBlocks> {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.prompt.gateway_text.get(provider_id.trim()).cloned();
        }
    }
    None
}

/// 「」 （** not raw**）。
///
/// and `retention_settings()` 1 ： in ** down ** when 
/// （`upstream::provider_loop::frame_policy_of`， and 1 ），
/// `current()` 1 `raw` Map —— path up no need 。
/// （ / no config） when ， and `current()`
/// in down `build(empty)` 1 ； is **default**： no has 
/// not 。
pub fn strip_newline_keepalive() -> bool {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.strip_newline_keepalive;
        }
    }
    env_strip_newline_keepalive()
}

/// 「」 （** not raw**）。
///
/// and [`strip_newline_keepalive`] 1 ：** request** need 1 
/// 「 this 1 need not need 」， `current()` 1 `raw` Map
/// （ that ）—— path up no need 。
/// （ / no config） when ， and `current()`
/// in down `build(empty)` 1 ； is **default**： no not 
/// use 1 up 。
pub fn auto_continue() -> bool {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.auto_continue;
        }
    }
    env_auto_continue()
}

/// limit （** not raw**）。
///
/// and [`auto_continue`] 1 （request 1 ， 1 `i64` can ）。
/// when by 「file no this + 」（ and `current()` `build(empty)`
/// 1 path， [`auto_continue_rounds_from`]）， this and 
/// 1 —— not will 「 by 3 、 by 」。
pub fn auto_continue_max_rounds() -> i64 {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.auto_continue_max_rounds;
        }
    }
    auto_continue_rounds_from(&Map::new())
}

/// models.dev （** not raw**）。
///
/// and `strip_newline_keepalive` 1 ：`core::model_meta::refresh` 1 ，
/// `current()` 1 `raw` Map。 when to `true` —— and 
/// `build(empty)` 1 （ this **default**， `KEY_MODELS_DEV_ENABLED`）。
pub fn models_dev_enabled() -> bool {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.models_dev_enabled;
        }
    }
    true
}

/// when set （** not raw**）。
///
/// and `retention_settings()` 1 ： and new ** 1 ** need 
/// 1 「 in 、」（ this is 「set down 1 」 ），
/// `current()` will `raw` Map —— no need 。
/// 1 `Copy` can 。 when to default。
pub fn scheduled_settings() -> ScheduledSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.scheduled;
        }
    }
    ScheduledSettings::default()
}

/// requestset （** not raw**）。
///
/// and `retention_settings()` 1 ： failurerequest need 1 
/// 「 can 、」， `current()` will `raw` Map
/// —— path up no need 。 1 （error is `Arc`，
/// 1 ）。 when to default。
pub fn retry_settings() -> RetrySettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.retry.clone();
        }
    }
    RetrySettings::default()
}

/// Cline （** not raw**）。
///
/// and [`gateway_prompt_enabled`] 1 ：Cline up request need 
/// 1 「 use 」， `current()` will `raw` Map。return is 
/// ****（ use ），default and and in 
/// `core::providers::cline::headers`。 when to empty（= default）。
pub fn cline_upstream_overrides() -> Vec<(String, String)> {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config
                .cline_upstream_headers
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
        }
    }
    Vec::new()
}

/// use 1 updateconfig（ → → → ）。
///
/// `mutate` ； by 1 ，。
///
/// 「」 has ： is 1 （`CONFIG.write()`）， that 1 
/// by `sql::write_all` 1 （ UPSERT + delete need 、 need not ）。
static CONFIG_UPDATE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn update<F>(mutate: F) -> bool
where
    F: FnOnce(&mut RuntimeConfig),
{
    // must，otherwise config will 。
    let _update_guard = CONFIG_UPDATE_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut next = current();
    let previous = next.clone();
    let before = synced_fingerprint(&next.raw);
    mutate(&mut next);
    if previous.raw().get(crate::server::core::model_rules::KEY_MODEL_RULES)
        != next.raw().get(crate::server::core::model_rules::KEY_MODEL_RULES) {
        next.model_rules = std::sync::Arc::new(crate::server::core::model_rules::ModelRules::from_raw(&next.raw));
    }
    let saved = save_raw(&next.raw);
    // after all：`next` need 
    let changed = before != synced_fingerprint(&next.raw);
    let catalog_changed = crate::server::core::providers::catalog::config_changed(&previous, &next);
    if catalog_changed { crate::server::core::providers::catalog::invalidate(); }
    if let Ok(mut guard) = CONFIG.write() {
        *guard = Some(next);
    }
    if catalog_changed { crate::server::core::providers::catalog::invalidate(); }
    // up ： has ** ** 。
    //
    // ── Why in this not is in `save_raw` ─────────────────────
    // `save_raw` etc path use ， that path not this up （ in ，
    // no ）。`update` is 「config」 1 （all setter ），
    // 。
    //
    // ── Why need in before 1 ───────────────────────────
    // `set_debug_mode` this setter will in 「 no 」 when `update`（）。
    // not for will up 1 ； for after has 。
    if changed {
        crate::server::core::cloud_sync::auto_sync::notify_change();
    }
    saved
}

/// kv ： + 。
///
/// used for「 this `update` has no has 」， not is 
/// —— use sha256 can config（） in 。
fn synced_fingerprint(raw: &Map<String, Value>) -> String {
    use crate::server::core::cloud_sync::snapshot::is_synced_kv_key;
    use sha2::{Digest as _, Sha256};

    let mut hasher = Sha256::new();
    // `raw` is `serde_json::Map`（default BTreeMap），
    for (key, value) in raw {
        if !is_synced_kv_key(key) {
            continue;
        }
        hasher.update(key.as_bytes());
        hasher.update(b"\0");
        hasher.update(value.to_string().as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

/// config 1 （ for Node `saveConfig`）。
///
/// ── and file （must）─────────────────────────
/// before this is 「file」：`raw` no has not in file， is 
/// 「 1 config」（`set_api_key(None)` will `raw.remove`） can 。
/// after is **** in ，if UPSERT， will in ——
/// use down will to that 。
/// `sql::write_all` in 1 「 UPSERT + not in `raw` config」，
/// delete**excluding** `db::schema::RESERVED_KV_KEYS`（ that is ）。
///
/// ── Whysuccess and need ────────────────────────
/// ； by use （`update`） in after 1 new 。failure
/// （unavailable、）return `false`，** ** ——
/// and before 「filefailure、 not in request」 1 ：failure is 
/// 「 after in not in 」， not this in request or config。
///
/// ── Why in {@link sql::write_all} ──────────────
/// `Db::with` is ** 1 that **， `logging::log` that 1 
/// need 1 `logs` —— `std::sync::Mutex` not can ， in 
/// etc 。 `write_all` `Result` ， this in 
/// （ and ）。
///
/// Note** in before can can use **（`bootstrap` 
/// `config::init` `logging::init_store`）： that when `logging::log` 
/// that 1 will ， —— this is this use （`console_line`
/// and `log` is 1 ）。
pub fn save_raw(raw: &Map<String, Value>) -> bool {
    let Some(db) = db() else {
        crate::server::logging::log(
            "[Config]",
            "❌ failure: unavailable（only）",
        );
        return false;
    };
    match sql::write_all(&db, raw) {
        Ok(()) => true,
        Err(error) => {
            crate::server::logging::log("[Config]", &format!("❌ failure: {error}"));
            false
        }
    }
}

/// set API Key：`None` delete（ for Node `body.apiKey === null` ）。
/// return is success；nonesuccess and already update（）。
///
/// ── is `enc1:` ， not is ───────────────────────
/// （ POST /api/config） is ， `raw`（= that 1 ） 
/// `crypto::seal(…, SecretScope::Accounts)` —— and `apiKeys` 
/// `key`、 old ** use 1 **（ by `api_keys` module header， not add）。
/// not in this after is 1 can not in ： up Key will in 
/// ， to down （`core::legacy_api_key`） —— 
/// before is 「 that 」， this 1 「file」。
///
/// `config.api_key` ****（ up ）， and `update_raw_field`
/// （） and `build` 1 ： this 1 is 
/// `RuntimeConfig::masked_api_key`， in that 。 in this 1 or use 1 ， 1 
/// will （ / ），、 for 、 by 。
///
/// ── failure：fail closed， not ─────────────────────
/// failure in this （file or configdirectory not 、
/// unavailable）， when etc 「 this 」， up not 。
/// failure `Err` up 、** 1 not **（`raw` and ， old Key
/// available）， use （`api::config_api::post_config`） error to use ——
/// and `api_keys::save` 1 to （ that `ApiKeyEntry::to_value`）。
/// and for is 「failure」： that 「 after in not in 」， use config has 
/// （`Ok(false)`， already update， not 1 requesterror， `save_raw` ）。
pub fn set_api_key(api_key: Option<String>) -> Result<bool, String> {
    let Some(key) = api_key else {
        // delete and ： not is empty（ use has ，
        // `set_update_proxy` / `set_prompt` 「 and set_api_key(None) 1 」）。
        // this not ， `Err` not can can 
        return Ok(update(|config| {
            config.raw.remove(crate::server::core::api_keys::KEY_API_KEY);
            config.api_key = None;
        }));
    };
    // must in `update` ** before **：`update` can 、 no in ，
    // ，failure when 「」 or 「empty」 ——
    // is error。 in then failure when 1 not （ down this 1 not will ）
    let envelope = crate::server::core::crypto::seal(
        key.as_bytes(),
        crate::server::core::crypto::SecretScope::Accounts,
    )?;
    Ok(update(move |config| {
        config.raw.insert(
            crate::server::core::api_keys::KEY_API_KEY.to_string(),
            Value::String(envelope.clone()),
        );
        // （） not is ： in 1 `masked_api_key`，
        // 
        config.api_key = Some(envelope);
    }))
}

/// `apiKeys` ， and old Key `apiKey`（ from has 1 ；
/// Key injected via environment variable not in file，`active_api_keys` will ）。
pub fn replace_api_keys(list: Value) -> bool {
    update(move |config| {
        config
            .raw
            .insert(crate::server::core::api_keys::KEY_API_KEYS.to_string(), list.clone());
        config.raw.remove(crate::server::core::api_keys::KEY_API_KEY);
        config.api_key = None;
    })
}

/// update（empty， for Node `typeof body.locale === 'string' && body.locale`）
pub fn set_locale(locale: &str) -> bool {
    let locale = locale.to_string();
    update(|config| {
        config.raw.insert("locale".to_string(), Value::String(locale.clone()));
        config.locale = locale.clone();
    })
}

/// request use （ for Node rememberRequestModel：only in when ）
pub fn remember_request_model(model: &str) {
    let trimmed = model.trim();
    if trimmed.is_empty() || current().last_request_model.as_deref() == Some(trimmed) {
        return;
    }
    let value = trimmed.to_string();
    update(|config| {
        config
            .raw
            .insert("lastRequestModel".to_string(), Value::String(value.clone()));
        config.last_request_model = Some(value.clone());
    });
}

/// config.json any（ its ）。path use 。
pub fn update_raw_field(key: &str, value: Value) -> bool {
    update_raw_field_with(key, move |current| *current = value)
}

/// 1 config（ raw to use ）。used for 1 ：
/// `update_raw_field` will 1 ， and ；
/// by use 1 `reload()`，all and for 。
pub(crate) fn update_raw_fields(mutate: impl FnOnce(&mut Map<String, Value>)) -> bool {
    update(move |config| mutate(&mut config.raw))
}

pub(crate) fn update_raw_field_with(key: &str, mutate: impl FnOnce(&mut Value)) -> bool {
    let key = key.to_string();
    update(move |config| {
        mutate(config.raw.entry(key.clone()).or_insert(Value::Null));
        // apiKey 「」， when need 。 this is ****
        // （ will use this `enc1:` ， `core::legacy_api_key`
        // ）， to in `RuntimeConfig::masked_api_key` 
        // —— Why not in this 1 ， that has 。
        if key == crate::server::core::api_keys::KEY_API_KEY {
            config.api_key = string_field(&config.raw, crate::server::core::api_keys::KEY_API_KEY);
        }
        if key == routing_profiles::KEY_ROUTING_PROFILES {
            config.routing_profiles = routing_profiles::from_raw(&config.raw);
        }
        if key == compact_routes::KEY_COMPACT_ROUTES {
            config.compact_routes = compact_routes::from_raw(&config.raw);
        }
        if key == client_identification::KEY_CLIENT_IDENTIFICATION {
            config.client_identification = client_identification::from_raw(&config.raw);
        }
        if key == account_sort::KEY_ACCOUNT_SORT {
            config.account_sort = account_sort::from_raw(&config.raw);
        }
    })
}

/// updateset（`None` = this not ），return is success。
///
/// use （`stats_api::put_retention`）**must**： by 「 already 」
/// ， will `days_field` （ that will use as set）。
///
/// and raw 1 （ and `set_api_key` 1 ）： before down 1 
/// use new ， after when not will 。nonesuccess and already update
/// （ and its setter 1 ），「set → 」 not IO。
pub fn set_retention(patch: RetentionPatch) -> bool {
    update(|config| {
        let mut next = config.retention;
        // ， not default to config。
        let mut apply = |key: &str, value: Option<i64>, slot: &mut i64| {
            if let Some(value) = value {
                config.raw.insert(key.to_string(), Value::from(value));
                *slot = value;
            }
        };
        apply(KEY_LOG_RETENTION_DAYS, patch.log_days, &mut next.log_days);
        apply(KEY_LOG_RETENTION_COUNT, patch.log_count, &mut next.log_count);
        apply(
            KEY_REQUEST_RETENTION_DAYS,
            patch.request_days,
            &mut next.request_days,
        );
        apply(
            KEY_REQUEST_RETENTION_COUNT,
            patch.request_count,
            &mut next.request_count,
        );
        apply(KEY_DAILY_RETENTION_DAYS, patch.daily_days, &mut next.daily_days);
        config.retention = next;
    })
}

/// update 1 （`None` = this not ），return is success。
///
/// `key` must is `KEY_CREDENTIAL_MAINTENANCE` etc 1 ——
/// is `scheduledTasks` down ，** not in this **： use 
/// （`scheduled_tasks::configure`） already by id ， not id
/// in that 1 。
///
/// use **must**（ and `set_retention` 1 ）： by 
/// 「 already 」， will `interval_field` 。
///
/// and `set_retention` 1 ： and raw 1 —— before in 
/// down 1 use new （ not ）， after when not 
/// （ 1 when ，`scheduledTasks` down its must）。
pub fn set_scheduled_task(
    key: &str,
    patch: IntervalTaskPatch,
    min: i64,
    max: i64,
) -> bool {
    let key = key.to_string();
    update(move |config| {
        // in raw this for （ not in 1 ），。
        // use `entry` not is 「 scheduledTasks」： after will its 
        // set， and will can can 。
        let root = config
            .raw
            .entry(KEY_SCHEDULED_TASKS.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !root.is_object() {
            // file for （）： for 。
            // not ignore —— that will 「success」 no ，。
            *root = Value::Object(Map::new());
        }
        let Some(tasks) = root.as_object_mut() else {
            return;
        };
        let entry = tasks
            .entry(key.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        let Some(task) = entry.as_object_mut() else {
            return;
        };
        if let Some(enabled) = patch.enabled {
            task.insert("enabled".to_string(), Value::Bool(enabled));
        }
        if let Some(interval) = patch.interval {
            // before by ： use already ， this 1 is 
            //（file and path not this to up ）
            task.insert(
                "interval".to_string(),
                Value::from(interval.clamp(min, max)),
            );
        }
        // 1 ：。** not can ** raw ——
        // is `scheduled_settings()` ， not new after is 
        // 「 up 、 is by old 」（ and need etc down ），
        // and that 「」 not 1 。
        config.scheduled = scheduled_from(&config.raw);
    })
}

/// updaterequestset（`None` = this not ），return is success。
///
/// use （`retry_api::put_retry`）**must**： by 「 already 」
/// ， will `bounded_int_field` （ that will use as 
/// set）。
///
/// and `set_retention` 1 ： and raw 1 —— before down 1 
/// failurerequest use new ， after when not config.json its 。
pub fn set_retry(patch: RetryPatch) -> bool {
    update(|config| {
        let mut next = config.retry.clone();
        if let Some(count) = patch.count {
            config.raw.insert(KEY_RETRY_COUNT.to_string(), Value::from(count));
            next.count = count;
        }
        if let Some(count) = patch.account_switch_count {
            config
                .raw
                .insert(KEY_RETRY_ACCOUNT_SWITCH_COUNT.to_string(), Value::from(count));
            next.account_switch_count = count;
        }
        if let Some(seconds) = patch.interval_seconds {
            config
                .raw
                .insert(KEY_RETRY_INTERVAL_SECONDS.to_string(), Value::from(seconds));
            next.interval_seconds = seconds;
        }
        if let Some(codes) = &patch.no_retry_codes {
            // raw 「 and API 」 （ already ，
            // this ）； new 
            let array: Vec<Value> = codes.iter().map(|code| Value::from(*code)).collect();
            config
                .raw
                .insert(KEY_RETRY_NO_RETRY_CODES.to_string(), Value::Array(array));
            next.no_retry_codes = std::sync::Arc::from(codes.as_slice());
        }
        config.retry = next;
    })
}

// ─── up request when （connectTimeoutSeconds / headersTimeoutSeconds /
//     streamIdleTimeoutSeconds / bodyTimeoutSeconds）──────────────────

/// when （** not raw**）。
///
/// and `retry_settings()` 1 ：、 need 1 ，
/// 1 `Copy` can 。 when to default。
pub fn timeout_settings() -> TimeoutSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.timeouts;
        }
    }
    TimeoutSettings::default()
}

/// update when （`None` = this not ），return is success。
///
/// use （`timeouts_api::put_timeouts`）**must**： by 
/// 「 already 」， will `bounded_int_field` 。
/// and `set_retry` 1 ： and raw 1 。
pub fn set_timeouts(patch: TimeoutPatch) -> bool {
    update(|config| {
        let mut next = config.timeouts;
        let mut write = |key: &str, value: i64, slot: &mut i64| {
            config.raw.insert(key.to_string(), Value::from(value));
            *slot = value;
        };
        if let Some(seconds) = patch.connect_seconds {
            write(KEY_TIMEOUT_CONNECT_SECONDS, seconds, &mut next.connect_seconds);
        }
        if let Some(seconds) = patch.headers_seconds {
            write(KEY_TIMEOUT_HEADERS_SECONDS, seconds, &mut next.headers_seconds);
        }
        if let Some(seconds) = patch.stream_idle_seconds {
            write(
                KEY_TIMEOUT_STREAM_IDLE_SECONDS,
                seconds,
                &mut next.stream_idle_seconds,
            );
        }
        if let Some(seconds) = patch.body_seconds {
            write(KEY_TIMEOUT_BODY_SECONDS, seconds, &mut next.body_seconds);
        }
        config.timeouts = next;
    })
}

// ─── etc （queueMaxWaits / queueWaitSeconds）──────────────

/// etc （** not raw**）。
///
/// and `timeout_settings()` 1 ： to need 1 ，
/// 1 `Copy` can 。 when to default。
pub fn queue_settings() -> QueueSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.queue;
        }
    }
    QueueSettings::default()
}

/// update etc （`None` = this not ），return is success。
///
/// use （`queue_api::put_queue`）**must**： by 「 already 」，
/// will `bounded_int_field` 。 and `set_timeouts` 1 ：
/// and raw 1 。
pub fn set_queue(patch: QueuePatch) -> bool {
    update(|config| {
        let mut next = config.queue;
        if let Some(count) = patch.max_waits {
            config
                .raw
                .insert(KEY_QUEUE_MAX_WAITS.to_string(), Value::from(count));
            next.max_waits = count;
        }
        if let Some(seconds) = patch.wait_seconds {
            config
                .raw
                .insert(KEY_QUEUE_WAIT_SECONDS.to_string(), Value::from(seconds));
            next.wait_seconds = seconds;
        }
        config.queue = next;
    })
}

// ─── ZCode （zcodeCaptchaPoolTarget / PollMs / MaxPerRound）──

/// （** not raw**）。
///
/// and `queue_settings()` 1 ：stats need 1 ，
/// 1 `Copy` can 。 when to default。
pub fn captcha_pool_settings() -> CaptchaPoolSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.captcha_pool;
        }
    }
    CaptchaPoolSettings::default()
}

/// update（`None` = this not ），return is success。
///
/// use （`zcode_captcha_pool_api::put_captcha_pool`）**must**：
/// by 「 already 」， will `bounded_int_field` 。
/// and `set_queue` 1 ： and raw 1 。
pub fn set_captcha_pool(patch: CaptchaPoolPatch) -> bool {
    update(|config| {
        let mut next = config.captcha_pool;
        if let Some(target) = patch.target {
            config
                .raw
                .insert(KEY_CAPTCHA_POOL_TARGET.to_string(), Value::from(target));
            next.target = target;
        }
        if let Some(poll_ms) = patch.poll_ms {
            config
                .raw
                .insert(KEY_CAPTCHA_POOL_POLL_MS.to_string(), Value::from(poll_ms));
            next.poll_ms = poll_ms;
        }
        if let Some(max_per_round) = patch.max_per_round {
            config.raw.insert(
                KEY_CAPTCHA_POOL_MAX_PER_ROUND.to_string(),
                Value::from(max_per_round),
            );
            next.max_per_round = max_per_round;
        }
        config.captcha_pool = next;
    })
}

// ─── will （imageCompressEnabled / MaxEdge / Quality / ThresholdKb）──

/// set（** not raw**）。
///
/// and `timeout_settings()` 1 ： before need 1 ， 1 
/// `Copy` can 。 when to default（****）。
pub fn image_compress_settings() -> ImageCompressSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.image_compress;
        }
    }
    ImageCompressSettings::default()
}

/// update will set（`None` = this not ），return is success。
///
/// use （`api::image_compress::put_image_compress`）**must**：
/// by 「 already 」， will `bounded_int_field` 。 and 
/// `set_captcha_pool` 1 ： and raw 1 。
pub fn set_image_compress(patch: ImageCompressPatch) -> bool {
    update(|config| {
        let mut next = config.image_compress;
        if let Some(enabled) = patch.enabled {
            config
                .raw
                .insert(KEY_IMAGE_COMPRESS_ENABLED.to_string(), Value::Bool(enabled));
            next.enabled = enabled;
        }
        let mut write_int = |key: &str, value: i64, slot: &mut i64| {
            config.raw.insert(key.to_string(), Value::from(value));
            *slot = value;
        };
        if let Some(max_edge) = patch.max_edge {
            write_int(KEY_IMAGE_COMPRESS_MAX_EDGE, max_edge, &mut next.max_edge);
        }
        if let Some(quality) = patch.quality {
            write_int(KEY_IMAGE_COMPRESS_QUALITY, quality, &mut next.quality);
        }
        if let Some(threshold_kb) = patch.threshold_kb {
            write_int(
                KEY_IMAGE_COMPRESS_THRESHOLD_KB,
                threshold_kb,
                &mut next.threshold_kb,
            );
        }
        config.image_compress = next;
    })
}

// ─── by （routingStrategy / routingSticky / / / ）──

/// by set （** not raw**）。
///
/// and `timeout_settings()` 1 ： need 1 ， 1 
/// `Copy` can 。 when to default。
pub fn routing_settings() -> RoutingSettings {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.routing;
        }
    }
    RoutingSettings::default()
}

/// update by set（`None` = this not ），return is success。
///
/// use （`routing_api::put_routing`）**must**： by 「 already 」
/// ， will `bounded_int_field` 。 and `set_timeouts`
/// 1 ： and raw 1 。
pub fn set_routing(patch: RoutingPatch) -> bool {
    update(|config| {
        let mut next = config.routing;
        if let Some(strategy) = patch.strategy {
            config
                .raw
                .insert(KEY_ROUTING_STRATEGY.to_string(), Value::from(strategy.as_str()));
            next.strategy = strategy;
        }
        if let Some(sticky) = patch.sticky {
            config
                .raw
                .insert(KEY_ROUTING_STICKY.to_string(), Value::from(sticky.as_str()));
            next.sticky = sticky;
        }
        let mut write_bool = |key: &str, value: bool, slot: &mut bool| {
            config.raw.insert(key.to_string(), Value::Bool(value));
            *slot = value;
        };
        if let Some(enabled) = patch.failover_backoff {
            write_bool(KEY_ROUTING_FAILOVER_BACKOFF, enabled, &mut next.failover_backoff);
        }
        if let Some(enabled) = patch.sink {
            write_bool(KEY_ROUTING_SINK, enabled, &mut next.sink);
        }
        if let Some(enabled) = patch.breaker_enabled {
            write_bool(KEY_ROUTING_BREAKER_ENABLED, enabled, &mut next.breaker_enabled);
        }
        if let Some(count) = patch.breaker_threshold {
            config
                .raw
                .insert(KEY_ROUTING_BREAKER_THRESHOLD.to_string(), Value::from(count));
            next.breaker_threshold = count;
        }
        if let Some(seconds) = patch.breaker_timeout_seconds {
            config
                .raw
                .insert(KEY_ROUTING_BREAKER_TIMEOUT.to_string(), Value::from(seconds));
            next.breaker_timeout_seconds = seconds;
        }
        if let Some(count) = patch.breaker_success_threshold {
            config
                .raw
                .insert(KEY_ROUTING_BREAKER_SUCCESS.to_string(), Value::from(count));
            next.breaker_success_threshold = count;
        }
        config.routing = next;
    })
}

// ─── （debugMode）────────────────────────────────────

/// 。
///
/// and `set_retry` 1 ： and raw 1 —— before down 1 request
/// use new （request）， after when not config.json 
/// its 。return is success（failure when already update， use ）。
pub fn set_debug_mode(enabled: bool) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_DEBUG_MODE.to_string(), Value::Bool(enabled));
        config.debug_mode = enabled;
    })
}

// ─── （sanitizeBlacklistFingerprints）─────────────

/// 。
///
/// and `set_debug_mode` 1 ： and raw 1 —— before down 1 request
/// use new （request）， after when not config.json 
/// its 。return is success（failure when already update， use ）。
pub fn set_sanitize_fingerprints(enabled: bool) -> bool {
    set_sanitize_settings(SanitizePatch {
        enabled: Some(enabled),
        ..Default::default()
    })
}

// ─── （developerRoleCompat）────────────────────

/// 。
///
/// and `set_debug_mode` 1 ： and raw 1 —— before down 1 request
/// use new （request）， after when not config.json 
/// its 。return is success（failure when already update， use ）。
pub fn set_developer_role_compat(enabled: bool) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_DEVELOPER_ROLE_COMPAT.to_string(), Value::Bool(enabled));
        config.developer_role_compat = enabled;
    })
}

// ─── Cline （clineUpstreamHeaders）────────────────

/// Cline ****（PUT = ）。
///
/// and `set_debug_mode` 1 ： and raw 1 —— before down 1 
/// request use new （request）， after when not config 
/// its 。empty = all（default）。 use （`api::cline_headers`）
/// （empty、 is 、 and has ）。
pub fn set_cline_upstream_headers(overrides: std::collections::BTreeMap<String, String>) -> bool {
    let value = Value::Object(
        overrides
            .iter()
            .map(|(key, text)| (key.clone(), Value::String(text.clone())))
            .collect(),
    );
    update(|config| {
        config
            .raw
            .insert(KEY_CLINE_UPSTREAM_HEADERS.to_string(), value.clone());
        config.cline_upstream_headers = overrides;
    })
}

// ─── （corsEnabled）───────────────────────────

/// （set「 → 」）。
///
/// and `set_sanitize_fingerprints` 1 ：（CORS in request
/// ， down 1 request use new as ， not ）， when not config.json 
/// its 。
pub fn set_cors_enabled(enabled: bool) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_CORS_ENABLED.to_string(), Value::Bool(enabled));
        config.cors_enabled = enabled;
    })
}

/// AI config（）。
///
/// and its setter 1 ：（ down 1 for request use new ），raw 
/// 1 not its 。return `false` **failure**（）。
pub fn set_assistant_config(config: AssistantConfig) -> bool {
    let value = config.to_value();
    update(|current| {
        current
            .raw
            .insert(KEY_ASSISTANT_CONFIG.to_string(), value);
        current.assistant = config;
    })
}

// ─── （stripNewlineKeepalive）───────────────────

/// 「」。
///
/// and `set_sanitize_fingerprints` 1 ：（ down 1 request 
/// use new ），raw 1 not its 。 and 1 
/// （ use ）—— that 1 ， down not is up ，
/// this must in to 。
pub fn set_strip_newline_keepalive(enabled: bool) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_STRIP_NEWLINE_KEEPALIVE.to_string(), Value::Bool(enabled));
        config.strip_newline_keepalive = enabled;
    })
}

// ─── （autoContinue / autoContinueMaxRounds）────

/// ** 1 **path：`None` = this not 。
///
/// and `set_retention` / `set_routing` 1 ： and raw 1 ——
/// before down 1 request use new （request）， after when not 
/// config its 。
///
/// ── Why setter need this ──────────────────────
/// 1 PUT if `update()`， is 「 → 」， in that 
/// failure will down 「 already 、limit is old 」 ， this need 1 has ；
/// 「 1 request」 this in use 。 1 ，
/// `set_auto_continue` / `set_auto_continue_max_rounds` is 。
///
/// use （`api::auto_continue` and setter use ）**must**：
/// by 「 already 」， will `auto_continue_rounds_from` 
/// （ that will use as set）。
pub fn set_auto_continue_patch(patch: AutoContinuePatch) -> bool {
    update(|config| {
        if let Some(enabled) = patch.enabled {
            config
                .raw
                .insert(KEY_AUTO_CONTINUE.to_string(), Value::Bool(enabled));
            config.auto_continue = enabled;
        }
        if let Some(rounds) = patch.max_rounds {
            config
                .raw
                .insert(KEY_AUTO_CONTINUE_MAX_ROUNDS.to_string(), Value::from(rounds));
            config.auto_continue_max_rounds = rounds;
        }
    })
}

/// 「」。
///
/// and `set_strip_newline_keepalive` 1 （ + raw 1 ），
/// and 1 （ use ）： that 1 ， 1 clientrequest can can for 
/// up 、 1 ， this must in to 。
pub fn set_auto_continue(enabled: bool) -> bool {
    set_auto_continue_patch(AutoContinuePatch {
        enabled: Some(enabled),
        max_rounds: None,
    })
}

/// limit（`1..=8`， use already ）。
///
/// and [`set_auto_continue_patch`] 1 ； this is to and 
/// （、 not ）， and PUT up 
/// `autoContinueMaxRounds` 1 —— is 1 path。
pub fn set_auto_continue_max_rounds(rounds: i64) -> bool {
    set_auto_continue_patch(AutoContinuePatch {
        enabled: None,
        max_rounds: Some(rounds),
    })
}

/// ： → old → 。
///
/// old 「default」 use ， not and —— 1 ， not 
/// this 1 。 this before after as （ use is ）， use in new 
/// 1 when not will 1 。
fn captcha_switch_from(
    raw: &serde_json::Map<String, Value>,
    key: &str,
    fallback: bool,
) -> bool {
    raw.get(key)
        .and_then(Value::as_bool)
        .or_else(|| raw.get(KEY_CAPTCHA_ENABLED).and_then(Value::as_bool))
        .unwrap_or(fallback)
}

/// （set「 → 」）。
///
/// `desktop = None` not that 1 （PUT ）。 when when 
/// 1 、 1 new —— will 「 already 、 no 」
/// in and request to 。
///
/// and `set_sanitize_fingerprints` 1 ：（ / 
/// request）， when not config.json its 。
pub fn set_captcha_patch(desktop: Option<bool>, web: Option<bool>) -> bool {
    if desktop.is_none() && web.is_none() {
        return true;
    }
    update(|config| {
        if let Some(enabled) = desktop {
            config
                .raw
                .insert(KEY_CAPTCHA_DESKTOP.to_string(), Value::Bool(enabled));
            config.captcha_desktop = enabled;
        }
        if let Some(enabled) = web {
            config
                .raw
                .insert(KEY_CAPTCHA_WEB.to_string(), Value::Bool(enabled));
            config.captcha_web = enabled;
        }
    })
}

/// models.dev （set「 use → models.dev 」）。
///
/// and `set_captcha_enabled` 1 ：（ down new when ），
/// when not config.json its 。** not already **： is 
/// 「」， not is 「 already to 」（ `KEY_MODELS_DEV_ENABLED`）。
pub fn set_models_dev_enabled(enabled: bool) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_MODELS_DEV_ENABLED.to_string(), Value::Bool(enabled));
        config.models_dev_enabled = enabled;
    })
}

// ─── update（updateProxy）─────────────────────────────

/// 「update」 （** not raw**）。
///
/// check and down in ** when ** 1 （）， not as 
/// `raw` Map —— that 1 can 。 when to None（）。
pub fn update_proxy() -> Option<Value> {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.update_proxy.clone();
        }
    }
    None
}

/// 「update」 （None = ）。
///
/// by use 1 （`api::update` `normalize_account_proxy`）， this not 
/// —— failure（ after / ） by ， not can 
/// （ `core::update::client`）。
/// and `set_sanitize_fingerprints` 1 ：（ down 1 check / down 
/// use new ， when ， not ）， when not its 。
pub fn set_update_proxy(proxy: Option<Value>) -> bool {
    update(move |config| {
        match proxy.clone() {
            Some(value) => {
                config.raw.insert(KEY_UPDATE_PROXY.to_string(), value);
                config.update_proxy = proxy.clone();
            }
            None => {
                // **** not is null： and set_api_key(None) 1 ，
                // config not no empty（ by = ）
                config.raw.remove(KEY_UPDATE_PROXY);
                config.update_proxy = None;
            }
        }
    })
}

// ─── GitHub （githubToken）────────────────────────

/// GitHub （** not raw**）。
///
/// in `core::update::token`（ this ， by ）。
/// when to None（ in ）。
pub fn github_token_envelope() -> Option<String> {
    if let Ok(guard) = CONFIG.read() {
        if let Some(config) = guard.as_ref() {
            return config.github_token.clone();
        }
    }
    None
}

/// GitHub （None = ， 1 ）。
///
/// ** not **： use （`core::update::token::set_token`）
/// and 。 and `set_update_proxy` 1 ：（ down 1 checkupdate use 
/// new ）， when not its 。
pub fn set_github_token_envelope(envelope: Option<String>) -> bool {
    update(move |config| {
        match envelope.clone() {
            Some(value) => {
                config.raw.insert(KEY_GITHUB_TOKEN.to_string(), Value::String(value));
                config.github_token = envelope.clone();
            }
            None => {
                config.raw.remove(KEY_GITHUB_TOKEN);
                config.github_token = None;
            }
        }
    })
}

// ─── （promptMode / promptFile / promptText）──────────

/// set（ + file + ）， and ** by new new 1 **。
///
/// and `set_sanitize_fingerprints` 1 ， 1 ： after 1 
/// [`prompt_from`]。 not will 「、`text` is up 1 」 
/// （`custom` empty = not ）， 
/// 「file not to 」 Reason 1 new —— use path can in up to 。
///
/// `file` / `inline` as empty/None when ** for **（ not is empty）： and 
/// `set_api_key(None)` 1 ，config not down no empty。 not ——
/// after ，file（ if ）。
///
/// return is success（failure when already update， use ）。
pub fn set_prompt(
    mode: crate::server::core::prompt::PromptMode,
    file: Option<String>,
    inline: Option<String>,
) -> bool {
    update(|config| {
        config
            .raw
            .insert(KEY_PROMPT_MODE.to_string(), Value::String(mode.as_str().to_string()));
        match file.filter(|text| !text.trim().is_empty()) {
            Some(path) => {
                config
                    .raw
                    .insert(KEY_PROMPT_FILE.to_string(), Value::String(path));
            }
            None => {
                config.raw.remove(KEY_PROMPT_FILE);
            }
        }
        match inline.filter(|text| !text.trim().is_empty()) {
            Some(text) => {
                config
                    .raw
                    .insert(KEY_PROMPT_TEXT.to_string(), Value::String(text));
            }
            None => {
                config.raw.remove(KEY_PROMPT_TEXT);
            }
        }
        config.prompt = prompt_from(&config.raw);
    })
}

/// /delete** 1 ** ， and 1 （ and [`set_prompt`] 1 by ：
/// not will 「、`text` is up 1 」 ）。
///
/// `patch` as `None` = ** this 1 **（default，config not empty for ）；
/// as `Some(ProviderPromptPatch)` = this 1 （`file` / `inline` as empty/None =
/// use default；`gateway` that ）。
///
/// return is success（failure when already update， use ）。
pub fn set_prompt_provider(provider_id: &str, patch: Option<ProviderPromptPatch>) -> bool {
    let id = provider_id.trim().to_string();
    if id.is_empty() {
        return false;
    }
    update(|config| {
        // this 1 old ， by patch not —— 1 ，
        // not will 「 map 」 this not 1 
        let mut entry = config
            .raw
            .get(KEY_PROMPT_PROVIDERS)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        entry.remove(&id);
        if let Some(patch) = patch {
            let mut one = Map::new();
            one.insert(
                KEY_PROMPT_MODE.to_string(),
                Value::String(patch.mode.as_str().to_string()),
            );
            if let Some(path) = patch.file.filter(|text| !text.trim().is_empty()) {
                one.insert(KEY_PROMPT_FILE.to_string(), Value::String(path));
            }
            if let Some(text) = patch.inline.filter(|text| !text.trim().is_empty()) {
                one.insert(KEY_PROMPT_TEXT.to_string(), Value::String(text));
            }
            entry.insert(id.clone(), Value::Object(one));
        }
        // 1 not when ：config not down no empty for 
        // （ and `set_prompt` for emptyfilepath 1 to ）
        if entry.is_empty() {
            config.raw.remove(KEY_PROMPT_PROVIDERS);
        } else {
            config
                .raw
                .insert(KEY_PROMPT_PROVIDERS.to_string(), Value::Object(entry));
        }
        config.prompt = prompt_from(&config.raw);
    })
}

/// WorkBuddy （2026-10） ** 1 **（config ）。
///
/// ── Why this need need 1 （ and `migrate_cline_split` ）────
/// Cline that is ****： old not in ， in 1 。WorkBuddy 
/// and is ****（ then 、 Key 、 by ）：
/// 「 1 has 、 1 no has 1 」 this in after ** 1 ** —— use after 
/// in new 1 、 new 1 workbuddy Key、 new 1 ，
/// will in down to ， use 。
///
/// use 1 、 by `account_bootstrap` 1 （ that use ）：
/// 1 not ， and `raccoonImported` / `autoclawImported` that 
/// 1 1 。
///
/// is **config** not is `db::schema::RESERVED_KV_KEYS` ： that 1 
/// is 「 not config 」， by config API 。
const WORKBUDDY_SPLIT_FLAG: &str = "workbuddySplitMigrated";

/// is already WorkBuddy 
pub fn workbuddy_split_migrated() -> bool {
    current()
        .raw()
        .get(WORKBUDDY_SPLIT_FLAG)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// down 「 already 」 。
///
/// **none has no has can need **（ new 1 ）： no has can not 
/// no —— not ， use after config when that will that 
/// config（ is need ）。
///
/// 1 ** not ** is then no （ `account_bootstrap` use ）：
/// that when down 1 in 。
pub fn mark_workbuddy_split_migrated() -> bool {
    update_raw_field(WORKBUDDY_SPLIT_FLAG, Value::Bool(true))
}

/// WorkBuddy （2026-10） ： `promptProviders.workbuddy`
/// **** 1 to `workbuddy-intl`。
///
/// ── Whymust（ not after is ）──────────────────────
/// before `workbuddy` this 1 id ，「 to WorkBuddy 1 
/// / or 」 use set for 。 after is 
/// 1 id： not ， that set for ** to default**
/// （ is passthrough）， this set use is system 
/// —— is 「 not 11-128、」， use not to need set
/// to 1 1 。
///
/// ── Why is not is （ and `migrate_cline_split` ）──────
/// old id `workbuddy` in （ is ）， will configempty。
///
/// ── ────────────────────────────────────────────────
/// ****（ not ）： this 1 by `api::prompt` ，
/// this ， can can will add 。 no has `workbuddy` when 
/// not （ not empty 1 empty）。
///
/// ** 1 **： use （`account_bootstrap`） and in 1 
/// 1 down —— not can ， that will use after to 
/// new to （ by [`WORKBUDDY_SPLIT_FLAG`]）。
pub fn migrate_workbuddy_split_prompt_providers() -> Option<String> {
    let intl = crate::server::core::providers::workbuddy::Region::Intl.provider_id();
    let mut summary: Option<String> = None;
    update(|config| {
        let table = config
            .raw
            .get(KEY_PROMPT_PROVIDERS)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if table.contains_key(intl) {
            return;
        }
        let Some(inherited) = table.get("workbuddy").cloned() else {
            return;
        };
        let mut next = table;
        next.insert(intl.to_string(), inherited);
        config
            .raw
            .insert(KEY_PROMPT_PROVIDERS.to_string(), Value::Object(next));
        config.prompt = prompt_from(&config.raw);
        summary = Some(format!(
            "💬 already WorkBuddy to （ before for ，set not ）：{intl}"
        ));
    });
    summary
}

/// ** 1 ** ****（`None` = 、 to ）。
///
/// and [`set_prompt_gateway`]（ not ）： and not ——
/// use in that 、 to when ， not this 。
/// empty** not config**（ by 「 this no 」 use ）， is 「」
/// in config is 1 `identity` ， 1 is 1 。
/// return is success（failure when already update， use ）。
pub fn set_prompt_gateway_text(
    provider_id: &str,
    blocks: Option<crate::server::core::prompt::GatewayBlocks>,
) -> bool {
    let id = provider_id.trim().to_string();
    if id.is_empty() {
        return false;
    }
    update(|config| {
        let mut table = config
            .raw
            .get(KEY_PROMPT_GATEWAY_TEXT)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        table.remove(&id);
        if let Some(blocks) = blocks.filter(|blocks| !blocks.is_empty()) {
            let mut one = Map::new();
            for field in crate::server::core::prompt::GatewayBlocks::FIELDS {
                let text = blocks.get(field);
                if !text.trim().is_empty() {
                    one.insert(field.to_string(), Value::String(text.to_string()));
                }
            }
            table.insert(id.clone(), Value::Object(one));
        }
        if table.is_empty() {
            config.raw.remove(KEY_PROMPT_GATEWAY_TEXT);
        } else {
            config
                .raw
                .insert(KEY_PROMPT_GATEWAY_TEXT.to_string(), Value::Object(table));
        }
        config.prompt = prompt_from(&config.raw);
    })
}

/// ** 1 ** （`None` = 、 to default「」）。
///
/// and [`set_prompt_provider`] by `KEY_PROMPT_GATEWAY` ：
/// 1 down not this this / file。return is success
/// （failure when already update， use ）。
pub fn set_prompt_gateway(provider_id: &str, flag: Option<bool>) -> bool {
    let id = provider_id.trim().to_string();
    if id.is_empty() {
        return false;
    }
    update(|config| {
        let mut table = config
            .raw
            .get(KEY_PROMPT_GATEWAY)
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        match flag {
            Some(value) => {
                table.insert(id.clone(), Value::Bool(value));
            }
            None => {
                table.remove(&id);
            }
        }
        if table.is_empty() {
            config.raw.remove(KEY_PROMPT_GATEWAY);
        } else {
            config
                .raw
                .insert(KEY_PROMPT_GATEWAY.to_string(), Value::Object(table));
        }
        // use 1 ：`gateway` and `providers` 1 new ， not will 
        // 「 1 update、 1 is up 1 」 
        config.prompt = prompt_from(&config.raw);
    })
}

// ─── directory（logDir / requestStatsDir / debugDir）────────//
// ── Why this ****， no has for （T8 1 ）──────
// in 1 after ****：、requeststatistics、
// in `{config_dir}/agent2api.db` ， not has 「 directory」。 that 
// `set_storage_dir(key, dir)` is to set 「…」 by use ，
// in ** 1 use no has **（ that by already delete），
// is —— 1 no use ， down has to will as 
// 「 can 」， after not will 。
//
// that Why `storage_dirs()` and **must**： has ，
// and is **** —— `db::migrate` `logs` / `requests` / `debug` need 
// use `storage_dirs()` ** use directory** old file（ that 
// `legacy_file`）。 use before `logDir: "D:\\logs"` is that in ，
// this ， that directory old not to （ will as 「 no has old file」，
// use will in up to 「 already 」empty）。
//
// 1 total this not for ：** is ， is already not in can **。
// will if supports「」， that this is 「 new + + configdirectory」，
// not is 1 —— that when not this this 。

/// by current directory（** old file**）。
#[derive(Clone, Debug)]
pub struct StorageDirs {
    /// old file（`logs.jsonl`） directory
    pub log_dir: PathBuf,
    /// request old file（`requests.jsonl` + `request-daily.jsonl`） directory
    pub request_stats_dir: PathBuf,
    /// old file（`debug-traffic.jsonl`） directory
    pub debug_dir: PathBuf,
}

/// directory：config** for path** use ，otherwiseconfigdirectory。
///
/// 「 not is 」 ：directory is need use （ old file 
/// directory）， this will —— for path / empty is 
/// 「file not 」， to defaultdirectory is down as 。
pub fn storage_dirs() -> StorageDirs {
    let base = config_dir();
    // from （`init` after has ； when default —— and 
    // `retention_settings()` 1 to ）。 1 ，。
    let (log_dir, request_stats_dir, debug_dir) = match CONFIG.read() {
        Ok(guard) => match guard.as_ref() {
            Some(config) => (
                resolve_dir(config.log_dir.as_deref(), &base),
                resolve_dir(config.request_stats_dir.as_deref(), &base),
                resolve_dir(config.debug_dir.as_deref(), &base),
            ),
            None => (base.clone(), base.clone(), base.clone()),
        },
        // in ：config not ，defaultdirectory（ and 1 to ）
        Err(_) => (base.clone(), base.clone(), base.clone()),
    };
    StorageDirs {
        log_dir,
        request_stats_dir,
        debug_dir,
    }
}

/// directory ：empty + for path， its `base`
fn resolve_dir(raw: Option<&str>, base: &Path) -> PathBuf {
    raw.map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| base.to_path_buf())
}

#[cfg(test)]
mod auto_continue_tests {
    //! in **** this 1 ： default、raw 
    //! not 、 and `set_*` must when to and 。
    //!
    //! ── Why this ────────────────────────────────────
    //! - **default**： 1 、 no has when ， to must is 
    //! 「 + 3 」。default not is 「 1 can 」， is use up 、
    //! ；
    //! - ** not default raw**：`raw` is ，default 1 ，
    //! 「 use from 」 and 「 use default」 not —— that 1 
    //! （ is 「config no has this 1 when 」）；
    //! - ** 1 **： `raw` will old （ up 、
    //! as not ， need etc down for up ）， then this 1 not 。
    //! is 「success」 failure， can 。
    //!
    //! ── Why use not ────────────────────────────
    //! `std::env::set_var` in 1 can 、 and use （
    //! `core::providers::trae::forward::forward_at` ）。 this 
    //! ** and none** ： need to （ that 1 
    //! ）， need use 「 and 1 」 this 。
    //! 「file」 that 1 in `parse.rs` `resolve_auto_continue` /
    //! `resolve_rounds` use 。

    use super::*;
    use serde_json::json;

    /// is ， `update()` is 「 → → 」 ****
    /// ： use and when ， 1 can can 1 「 old 」。
    /// use （ and `api::panel` that 1 ）。
    static CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn hold_config() -> std::sync::MutexGuard<'static, ()> {
        CONFIG_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// 1 to 。
    fn raw_of(pairs: &[(&str, Value)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect()
    }

    /// empty（ new ）＝「file no this 」，must is that 1 ；
    /// no when is 「 + 3」， and ** 1 default not **。
    ///
    /// ：`unwrap_or(true)`（default）、limit 0（ 1 not ，
    /// up is 「 already 」）， and when default `raw`。
    #[test]
    fn an_empty_backup_means_off_and_three_rounds_without_seeding_defaults() {
        let fresh = build(Map::new());
        // ：empty ≡ 「file no to 」， that 1 can by 
        assert_eq!(fresh.auto_continue(), env_auto_continue(), "empty must by ");
        assert_eq!(
            fresh.auto_continue_max_rounds(),
            auto_continue_rounds_from(&Map::new()),
            "empty limitmust by and default"
        );
        if std::env::var_os("AGENT2API_AUTO_CONTINUE").is_none() {
            assert!(!fresh.auto_continue(), "file and no → must is ");
        }
        if std::env::var_os("AGENT2API_AUTO_CONTINUE_MAX").is_none() {
            assert_eq!(
                fresh.auto_continue_max_rounds(),
                DEFAULT_AUTO_CONTINUE_MAX_ROUNDS,
                "file and no → must is default 3 "
            );
        }
        assert!(fresh.raw().is_empty(), "default， not ");
    }

    /// to as —— including「 `false` 」 and 「 1 / 8 this 」。
    ///
    /// has ， is this up will ：
    /// - `raw.get(key).is_some()` （ or `!= false` ）： up 
    /// 「」 by ， already is false as old ；
    /// - ： 99 to set 8， will as 8 is ，
    /// 「99 from 」 this not ；
    /// - 0 / false： that 「 」 and 「 down 」 not 。
    #[test]
    fn a_value_in_the_backup_is_the_value_that_gets_used() {
        let on = build(raw_of(&[(KEY_AUTO_CONTINUE, json!(true))]));
        assert!(on.auto_continue(), "file true must is ");
        assert_eq!(
            on.auto_continue_max_rounds(),
            auto_continue_rounds_from(&Map::new()),
            " no limit → limit that 1 ， not "
        );

        let off = build(raw_of(&[(KEY_AUTO_CONTINUE, json!(false))]));
        assert!(!off.auto_continue(), "file false must is （ need can ）");

        for bound in [AUTO_CONTINUE_MIN_ROUNDS, AUTO_CONTINUE_MAX_ROUNDS] {
            let at_edge = build(raw_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(bound))]));
            assert_eq!(at_edge.auto_continue_max_rounds(), bound, " {bound} must is available");
        }

        let too_many = build(raw_of(&[(KEY_AUTO_CONTINUE_MAX_ROUNDS, json!(99))]));
        assert_ne!(too_many.auto_continue_max_rounds(), 99, " not ");
        assert_ne!(
            too_many.auto_continue_max_rounds(),
            AUTO_CONTINUE_MAX_ROUNDS,
            " not limit， can to down 1 "
        );
        assert_eq!(
            too_many.auto_continue_max_rounds(),
            build(Map::new()).auto_continue_max_rounds(),
            " must and 「file no this 」"
        );

        // = no （ and 1 ）， not is 「 by down 」
        let junk = build(raw_of(&[
            (KEY_AUTO_CONTINUE, json!("true")),
            (KEY_AUTO_CONTINUE_MAX_ROUNDS, json!("8")),
        ]));
        assert_eq!(junk.auto_continue(), build(Map::new()).auto_continue());
        assert_eq!(junk.auto_continue_max_rounds(), build(Map::new()).auto_continue_max_rounds());
    }

    /// `set_auto_continue` / `set_auto_continue_max_rounds` must when to and 
    /// ， and ** not **。
    ///
    /// ：
    /// - `raw`： down 1 request old （`set_scheduled_task` that 
    /// 「** not can ** raw —— is `scheduled_settings()` 」
    /// is 1 ， is that in when up 、 this in path up ）；
    /// - 1 when 1 by default： use 「 to 1 」， 
    /// ；
    /// - （`Value::String("5")`）： for ，
    /// after by 「 not 」default —— use will as set no 。
    #[test]
    fn the_setters_write_the_backup_and_the_snapshot_together() {
        let _guard = hold_config();
        // ： to ， use down this 
        let _ = set_auto_continue_patch(AutoContinuePatch {
            enabled: Some(false),
            max_rounds: Some(DEFAULT_AUTO_CONTINUE_MAX_ROUNDS),
        });

        let saved = set_auto_continue(true);
        assert_eq!(saved, db().is_some(), "return this by 「 not success」（ no has ）");
        assert!(auto_continue(), "must can in （ is ）");
        assert_eq!(current().raw()[KEY_AUTO_CONTINUE], json!(true), "must up this 1 ，otherwise not ");
        assert!(current().auto_continue(), " getter and must is 1 ");
        assert_eq!(
            auto_continue_rounds_from(current().raw()),
            DEFAULT_AUTO_CONTINUE_MAX_ROUNDS,
            " not limit 1 and default "
        );

        set_auto_continue_max_rounds(AUTO_CONTINUE_MAX_ROUNDS);
        assert_eq!(auto_continue_max_rounds(), AUTO_CONTINUE_MAX_ROUNDS, "limitmust can in to ");
        assert_eq!(current().raw()[KEY_AUTO_CONTINUE_MAX_ROUNDS], json!(8), " need 1 JSON ");
        assert!(auto_continue(), "limit when must（update not is ）");

        // 1 （PUT ）： 1 update this for 
        set_auto_continue_patch(AutoContinuePatch {
            enabled: Some(false),
            max_rounds: Some(AUTO_CONTINUE_MIN_ROUNDS),
        });
        assert!(!auto_continue());
        assert_eq!(auto_continue_max_rounds(), AUTO_CONTINUE_MIN_ROUNDS);

        // ： 「 + 1 」 to after use （ its to that ）
        let _ = set_auto_continue_patch(AutoContinuePatch {
            enabled: Some(false),
            max_rounds: Some(DEFAULT_AUTO_CONTINUE_MAX_ROUNDS),
        });
    }
}
