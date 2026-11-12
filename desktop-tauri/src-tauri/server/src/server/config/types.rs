//! config ** for **：default、、set and 。
//!
//! ── Why 1 ──────────────────────────────────────────
//! this when is ** **：`config.json`（ in is `kv` ）、
//! HTTP API response、 and before `config.X` 。must has 1 ——
//! and 1 ， 1 not ， will to default。
//! file after 「config 1 has 、」 1 can ，
//! `mod.rs` 「」。
//!
//! ── Why 1 not （camelCase ）─────────────────────
//! before is `config.json` ； `kv` after ** use **，
//! not snake_case： this when is HTTP API （`GET /api/config` 
//! response、 before `config.X` ）， will before 、、 then 
//! when 。 and JSON 1 1 for ， when can 
//! configfile for （ `db::schema` module header）。
//! 1 ： this ** not can and `db::schema::RESERVED_KV_KEYS` ** ——
//! after is config （ that ）。
//!
//! ── ────────────────────────────────────────────────
//! - ****（`KEY_*`）： use ；
//! - ****（`DEFAULT_*` / `*_MIN_*` / `*_MAX_*`）： and 
//! use 1 ，「 60 」 this 。

use std::collections::BTreeMap;
use std::sync::Arc;

/// default：client when use （ for Node `--default-model` default）
pub const DEFAULT_MODEL: &str = "auto";
/// default（ for Node `--locale` default）
pub const DEFAULT_LOCALE: &str = "zh-CN";

// ─── set and （config.json ** is **）─────
// and has （apiKey / locale / lastRequestModel / autoCheckin） 1 ：
// camelCase。 this ， is as ** and must use 1 ** ——
// 1 not will ， will to default。

/// （ 1 `logs` ）
pub const KEY_LOG_RETENTION_DAYS: &str = "logRetentionDays";
/// request（ 1 `requests` ）。
///
/// Why not is old `requestRetentionDays`： that in 「」 when 
/// （`4a0e104`）， / old config.json is ** old default**（30 ）。
/// this set when default 99999 —— use old will that ，
/// after 1 。 new from new default； old 
/// not 、none（ and when 1 ）。
pub const KEY_REQUEST_RETENTION_DAYS: &str = "requestLogRetentionDays";
/// by （ 1 `request_daily` ）（ by up ）
pub const KEY_DAILY_RETENTION_DAYS: &str = "dailyAggregateRetentionDays";
pub const KEY_LOG_RETENTION_COUNT: &str = "logRetentionCount";
pub const KEY_REQUEST_RETENTION_COUNT: &str = "requestLogRetentionCount";

/// directory（config.json ）。
///
/// is ** for path**； / empty / for path（ as ）configdirectory ——
/// 1 ， not （ can to 1 directory，file not ）。
pub const KEY_LOG_DIR: &str = "logDir";
/// request（ + by ） directory（config.json ）， `KEY_LOG_DIR`
pub const KEY_REQUEST_STATS_DIR: &str = "requestStatsDir";
/// directory（config.json ）， `KEY_LOG_DIR`
pub const KEY_DEBUG_DIR: &str = "debugDir";

/// 「update」 （config.json ，** = **）。
///
/// is 1 after config for （ and 1 ：`{source:'clash',
/// listenerUid}` / `{source:'custom',…}`），null / 。
/// checkupdate and down use （`core::update::client` ），
/// when check 1 —— not use ， down 1 check。
/// 1 and use that （`normalize_account_proxy` /
/// `resolve_account_proxy`）， this 。
pub const KEY_UPDATE_PROXY: &str = "updateProxy";

/// 「 beta update」（config.json ，）。
///
/// and `updateStable` Release，current not 。
/// beta must and tag 1 ，otherwisenoneupdate after 。
pub const KEY_UPDATE_BETA: &str = "updateBeta";

/// GitHub （config.json ，** not is **）。
///
/// is `core::update::token` use AES-256-GCM `enc1:<base64>` ，
/// 、 in request when use ； in file（ that 
/// ）。API 「 has no has 、」， not 。
/// `GITHUB_TOKEN` is （： this > ）。
pub const KEY_GITHUB_TOKEN: &str = "githubToken";

/// （config.json ）。
///
/// after will ** to up request（） and request、 up return response and 
/// response** to 1 `debug_traffic` （ `core::debug_traffic`），request
/// 「」。default —— can KB， will directory
/// ； when path not （， `if enabled`）。
pub const KEY_DEBUG_MODE: &str = "debugMode";

/// request（config.json ， for workbuddy2api 
/// `features.sanitize_blacklist_fingerprints`）。
///
/// after in before up （
/// `core::sanitize`）：delete、 。
/// **default**（ and default）： etc client system to 
/// up ， that is （400 code=11128） Reason。
/// request， down 1 request， not 。
pub const KEY_SANITIZE_FINGERPRINTS: &str = "sanitizeBlacklistFingerprints";

/// （config.json ，set「 → 」）。
///
/// after in before request `messages` `role: "developer"` 
/// as `role: "system"`（OpenAI new API use developer system 
/// ）， not developer OpenAI up —— is 
/// request ， not 1 this will up 400。
/// **default**（ and OmniProxy default）：none（ in OpenAI 
/// up etc ），default will developer client 1 up not up 。
/// request， down 1 request， not 。
/// `core::upstream::payload`（send_body 1 ）。
pub const KEY_DEVELOPER_ROLE_COMPAT: &str = "developerRoleCompat";

/// **Cline **（config.json ）。
///
/// ：`{"User-Agent": "Cline/3.0.62", ...}`（string → string）。 is 
/// ****config：Cline up request default in 
/// `core::providers::cline::headers`（ and client for that 1 ，
/// `X-CLIENT-TYPE: cline-sdk` this ）， this **empty** by 
/// default（ can add），**empty** 「 this not need 」——
/// is 「 to default」 and 「 」 is 1 PUT can 。
///
/// Whydefault not config： and `promptGatewayText` 1 to —— default is 
/// 1 etc （ up as when ），config use ** ** that ，
/// not 、 not 。
pub const KEY_CLINE_UPSTREAM_HEADERS: &str = "clineUpstreamHeaders";

/// 网关面（`/v1/*`） （config.json ，set「 → 」）。
///
/// **default**： not when not to —— request will to 
/// API Key in up 401 （ by not `Authorization` ），
/// can to 1 「none API」。 need use （ Web UI、file
/// before use etc ） when 。 after by by ：
/// 204， `Access-Control-Allow-*` response down （ as `*`）。
///
/// ── Whydefault， by 1 is `*` ────────────────────────
/// by `*` is has as （ and `/api/*` ， not need need ），
/// 1 ； is ** up 、** that 1 ——
/// `*` no API Key when ， can up 。default + 
/// ， use 。
///
/// by （`/api/*`） not ， has none CORS
/// （ `http::panel_router` and `http::cors`）。
pub const KEY_CORS_ENABLED: &str = "corsEnabled";

/// 「」 （ `core::upstream::sse::is_newline_keepalive`）。
///
/// after ，** this can **（ before has WorkBuddy and AutoClaw） in down 
/// SSE when 「 by 」 content —— that is that up 
/// （ 1 `: heartbeat` in ），client by markdown 
/// 1 1 。 58 WorkBuddy 200 request 20 this （34.5%），
/// request in 13.4%（ 1.2%–30.8%）。
///
/// **default**， and that `sse_strip_newline_chunks()` is ** and ** ：
/// can 「 this up will not will this 」， this 「 need not need as 
/// down 」。default by not is ， is to —— after ，
/// has 1 「」 will 1 ； that is 1 ** **，
/// is 1 。 1 not can ， is as this can in 
/// up and when （ `api::keepalive` `/api/keepalive-strip`）。
///
/// ：config no has this when `AGENT2API_STRIP_NEWLINE_KEEPALIVE`
/// （`1` / `true` / `on` ， its empty， `parse::env_strip_newline_keepalive`）
/// —— Docker from 1 ， not 1 。
pub const KEY_STRIP_NEWLINE_KEEPALIVE: &str = "stripNewlineKeepalive";

/// （config.json ，ALTCHA proof-of-work， `server::altcha`）。
///
/// **default**： / is ， can none（、
/// ）；ALTCHA request 1 ，failure 
/// no 。 for none —— in after 。
/// login / setup ， and `/v1/*` API Key none。
///
/// ：config no has this when `AGENT2API_CAPTCHA_ENABLED`
/// （，default as 1，0 as ，
/// `parse::env_captcha_enabled`）—— Docker from 1 ，
/// in compose / `.env` can ；set 1 after as 
/// （「config > 」）。
///
/// ── （2026-10 ）──────────────────────────────────────
/// is ** old **， in as default： when 
/// [`KEY_CAPTCHA_DESKTOP`] / [`KEY_CAPTCHA_WEB`]， no has 。
/// when old —— use not will 。
pub const KEY_CAPTCHA_ENABLED: &str = "captchaEnabled";

/// （Tauri ） 。
///
/// default****（ and old 1 ）： / for 
/// its ，ALTCHA is 1 。 use can in set ——
/// when ，。
pub const KEY_CAPTCHA_DESKTOP: &str = "captchaDesktop";

/// （headless / Docker / ） 。
///
/// headless default****： this is in up ，。
pub const KEY_CAPTCHA_WEB: &str = "captchaWeb";

/// models.dev （config.json ， `core::model_meta`）。
///
/// **default**： is 「 up no that can by up 」 this total
/// ——** new **（，），default
/// is when 1 MB request；default is 「 this can for 
/// use etc not in 」（ no will set 1 not need need ）。
///
/// after ：`core::model_meta::refresh` return skipped（ not ）， already 
/// use （ that + that not will ）——「」 is 
/// 「」， not is 「 already has 」。
pub const KEY_MODELS_DEV_ENABLED: &str = "modelsDevEnabled";

/// （config.json ， for workbuddy2api `prompt.mode`）。
///
/// `passthrough` / `custom` / `append`（ `core::prompt::PromptMode`）；
/// **default `passthrough`**（ and default）： not client system， as and before 
/// 。`custom` / `append` use has / system ——
/// that is ： from **** system ， not is etc 
/// before （、 not ， `core::prompt` module header）。
pub const KEY_PROMPT_MODE: &str = "promptMode";

/// file （config.json ， for workbuddy2api `prompt.file`）。
///
/// empty / = use default（`core::prompt::BUILT_IN_PROMPT`）；
/// otherwise this path（UTF-8 ）。 in `custom` / `append` down has ——
/// `passthrough` not file。
pub const KEY_PROMPT_FILE: &str = "promptFile";

/// **** （config.json ）。
///
/// empty = ** as **， [`KEY_PROMPT_FILE`] and default；empty /
/// = to 「file > default」 path。 is this （ and 
/// `promptMode` / `promptFile` 1 ， and use 1 ）。
///
/// ── Why need has （ and file ）────────────────────────
/// filepath「 has 1 」； 1 need file、
/// 、set， for 「 1 『 use in 』」 this need 。
/// is ** that 1 **，（`set_prompt` will ， down 1 request
/// use new ）。 not ：empty after ，file and default old ——
/// use not as 1 filepath。
pub const KEY_PROMPT_TEXT: &str = "promptText";

/// ** by **set （config.json ）。
///
/// ：`{"<providerId>": {"promptMode": "...", "promptFile": "..."}}` ——
/// and that ****（ has 1 ：config、、
/// response）， is **** ——
/// config that ， its 1 use up 。delete = this from 
/// for （ `null`）。
///
/// ── Why need need （ and ）────────────────────────────
/// 「 use 」 is ** by up ** not ： 1 by 、
/// 1 （ZCode ） ， 1 all
/// not is is 。 as **default**（ not this when as and before ），
/// in this 。
pub const KEY_PROMPT_PROVIDERS: &str = "promptProviders";

/// **** （config.json ）：`{"<providerId>": false}`。
///
/// up that is **client system **； this 1 is ** need not need 
/// that **（ has ZCode ，
/// `zcode::OFFICIAL_PROMPT_NOTE` ）。
///
/// ── Why 1 ， not and `promptProviders` ──────────────
/// and default not 1 ：`promptProviders` is 「 this ** has ** 
/// / file」（ = ）， this 1 is 「 this need not need 」
/// （ = ****， as that is up down need ）。 and 1 ， use 1 down 
/// / file 1 —— is 「」 will 
/// this in when up ， after not ： 1 after ，
/// and after use not 。 after not ， 。
///
/// `true` / `false`（ use that 1 ）；** = default（）**。
/// `null` = 、 to default（ `/api/prompt` ）—— use 
/// 1 `true` is has ：config 1 「 this need 」，
/// and 「 from no 」， when 1 。
pub const KEY_PROMPT_GATEWAY: &str = "promptGateway";

/// ** **（config.json ）：
/// `{"<providerId>": {"identity": "...", "stable": "...", "dynamic": "..."}}`。
///
/// up that is 「 not 」， this 1 is 「 that 」—— use 
/// that 1 config as （ and ： that can ， use ，
/// `core::prompt::GatewayBlocks::or`）。 is ： / /
/// —— is up need ， not 「 1 」 。
///
/// `{cwd}` / `{platform}` / `{shell}` / `{os_version}` / `{git}` /
/// `{provider}` / `{model}` is ****：request when （directory、
/// 、……）。 `{provider}` / `{model}` ， this is 
/// Environment that ， use 1 、 can 。
///
/// = this from for （ `null`）= to 。
pub const KEY_PROMPT_GATEWAY_TEXT: &str = "promptGatewayText";

/// 「」 when use ****（`settings.get` `key`）。
///
/// ── Why not can `KEY_PROMPT_GATEWAY` ────────────────
/// `settings.get` default is 「 `plugin.<provider>.<key>` this ** has **
/// config」， is use config（ in `promptGateway` 、
/// by provider id ）。 to this 1 use ， can to 1 use 
/// config —— otherwise up 、 request not 。
pub const PLUGIN_SETTING_GATEWAY_PROMPT: &str = "gatewayPrompt";

/// 「」 when use （ and 
/// [`PLUGIN_SETTING_GATEWAY_PROMPT`] 1 ）。
///
/// is `{identity, stable, dynamic}`（ use ， 
/// `Null` 「 use 」）。
pub const PLUGIN_SETTING_GATEWAY_PROMPT_TEXT: &str = "gatewayPromptText";

/// ** 1 ** when （`set_prompt_provider` ）。
///
/// and [`ProviderPrompt`] before 1 1 for ； use not is ， is as 
/// will when not use 。
#[derive(Clone, Debug)]
pub struct ProviderPromptPatch {
    /// （ / / ）
    pub mode: crate::server::core::prompt::PromptMode,
    /// file（`None` / empty = use default）
    pub file: Option<String>,
    /// （`None` / empty = no has this 1 ，file / default）
    pub inline: Option<String>,
}

/// set（set「 use → 」）。
///
/// and `RetrySettings` 1 ： total is 1 use （request 1 、
/// 1 return）， 1 use 1 to ， not 。
/// and set not ， this ** not is `Copy`**：`text` is （ can can ），
/// request —— is use （`core::prompt::PromptPlan`）。
#[derive(Clone, Debug)]
pub struct PromptSettings {
    /// （ / / ）
    pub mode: crate::server::core::prompt::PromptMode,
    /// use file（`None` = ）
    pub file: Option<String>,
    /// （`None` = no this 1 ； has when is ** **）
    pub inline: Option<String>,
    /// **** ：`custom` / `append` down is / file /
    /// default；`passthrough` down ** no has **， will use down 
    /// （set need ， `resolve_choice` ）
    pub text: String,
    /// （ and need can 「 this use to is 1 」）
    pub source: crate::server::core::prompt::PromptSource,
    /// file not to when Reason（`None` = no this ）；failure when `text`
    /// default —— and 「」 has to 1 ： use not can as 
    /// 1 file not or not 
    pub file_error: Option<String>,
    /// ** by ** （`KEY_PROMPT_PROVIDERS`）； not in this up that 。
    /// `BTreeMap` not is `HashMap`： and response need has （ 1 config
    /// not ， use will as set ）
    pub providers: BTreeMap<String, ProviderPrompt>,
    /// ** **（`KEY_PROMPT_GATEWAY`）：`false` = this 1 not ，
    /// not in = （default）。 and `providers` by that 。
    pub gateway: BTreeMap<String, bool>,
    /// ** **（`KEY_PROMPT_GATEWAY_TEXT`）： 
    /// ； not in = use 。 and ： and is 。
    pub gateway_text: BTreeMap<String, crate::server::core::prompt::GatewayBlocks>,
}

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            mode: crate::server::core::prompt::PromptMode::default(),
            file: None,
            inline: None,
            text: String::new(),
            source: crate::server::core::prompt::PromptSource::None,
            file_error: None,
            providers: BTreeMap::new(),
            gateway: BTreeMap::new(),
            gateway_text: BTreeMap::new(),
        }
    }
}

/// 1 up （`KEY_PROMPT_PROVIDERS` 1 ）。
///
/// and [`PromptSettings`] before ** 1 **： has 1 
/// （`prompt_from` that `resolve_choice`）， is 「default、
/// 」 not will 。 not in 「 this no file is file」 this 
/// ： 1 in map ， mode / file / inline **** this as 
/// （`file: None` = use default）—— is use not ，
/// up 「 this 」 and 「 use 」must 1 for up 。
#[derive(Clone, Debug)]
pub struct ProviderPrompt {
    /// （ / / ）
    pub mode: crate::server::core::prompt::PromptMode,
    /// use file（`None` = ， use default）
    pub file: Option<String>,
    /// （`None` = no this 1 ）
    pub inline: Option<String>,
    /// **** （`passthrough` down that ： no has ，
    /// has ）
    pub text: String,
    /// 
    pub source: crate::server::core::prompt::PromptSource,
    /// file not to when Reason（`None` = no this ）
    pub file_error: Option<String>,
}

/// default（ when use ）。
///
/// request and by default is 99999 （ 274 ）： up etc 「」，
/// when is 1 —— use can ，
/// not 「empty」（ old 「」 as is ：
/// empty / by delete ）。
pub const DEFAULT_LOG_RETENTION_DAYS: i64 = 30;
pub const DEFAULT_REQUEST_RETENTION_DAYS: i64 = 99999;
pub const DEFAULT_DAILY_RETENTION_DAYS: i64 = 99999;
pub const DEFAULT_LOG_RETENTION_COUNT: i64 = 10_000;
pub const DEFAULT_REQUEST_RETENTION_COUNT: i64 = 1_000_000;
pub const RETENTION_MIN_COUNT: i64 = 1;
pub const RETENTION_MAX_COUNT: i64 = 100_000_000;

/// **** by ：`{"workbuddy": 10, "raccoon": 20}`。
///
/// provider by already 「 1 」 down （ use 1 by 
/// ）。 this in 1 （`legacy_provider_route`），
/// use old 「 by 」 by old and ； not has ，
/// file not will （unknown）。
pub const KEY_PROVIDER_ROUTE: &str = "providerRoute";

/// ： down 1 （ 0 etc not ， not is validconfig），
/// limit「none」 default 99999 —— up no has （99999 already 274 ），
/// can config.json empty。
///
/// **（`stats_api::parse_retention_value`） and （`days_field`） use this **：
/// if 1 ，file and will 
/// （e.g. 5000 ）。 not is has ：
/// to 400（ use can ），file to default（ not ）。
pub const RETENTION_MIN_DAYS: i64 = 1;
pub const RETENTION_MAX_DAYS: i64 = 99999;

/// set。 and request when and limit， by by 。
/// after ；request not will or delete by 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetentionSettings {
    /// 
    pub log_days: i64,
    pub log_count: i64,
    /// request
    pub request_days: i64,
    pub request_count: i64,
    /// by 
    pub daily_days: i64,
}

impl Default for RetentionSettings {
    fn default() -> Self {
        Self {
            log_days: DEFAULT_LOG_RETENTION_DAYS,
            log_count: DEFAULT_LOG_RETENTION_COUNT,
            request_days: DEFAULT_REQUEST_RETENTION_DAYS,
            request_count: DEFAULT_REQUEST_RETENTION_COUNT,
            daily_days: DEFAULT_DAILY_RETENTION_DAYS,
        }
    }
}

/// ****update（PUT /api/retention its in ）。
/// `None` = this 1 not （ for 「」 ）。
#[derive(Clone, Copy, Debug, Default)]
pub struct RetentionPatch {
    pub log_days: Option<i64>,
    pub log_count: Option<i64>,
    pub request_days: Option<i64>,
    pub request_count: Option<i64>,
    pub daily_days: Option<i64>,
}

// ─── when set and （config.json ** is **）─────
//
// （ / new / check / before new ） in 
// `scheduledTasks` for down ； to not in its in —— is ****， when and up 
// by `core::auto_checkin` （`autoCheckin` ）， not has 。
// already config （`core::usage_query`）， not in its in 。
// up this is 「 1 」，config 1 ，
// need by 1 。

/// config for 
pub const KEY_SCHEDULED_TASKS: &str = "scheduledTasks";
/// in `scheduledTasks` down 
pub const KEY_CREDENTIAL_MAINTENANCE: &str = "credentialMaintenance";
/// directory when new in `scheduledTasks` down 
pub const KEY_MODEL_REFRESH: &str = "modelRefresh";
/// new in `scheduledTasks` down （** before ** when ， after config）
pub const KEY_LOGS_AUTO_REFRESH: &str = "logsAutoRefresh";
/// request new in `scheduledTasks` down （ up ）
pub const KEY_REQUESTS_AUTO_REFRESH: &str = "requestsAutoRefresh";
/// new in `scheduledTasks` down （ up ）
pub const KEY_REPORT_AUTO_REFRESH: &str = "reportAutoRefresh";
/// check in `scheduledTasks` down （ after when to GitHub new ）
pub const KEY_UPDATE_CHECK: &str = "updateCheck";

/// default（minutes）： and before 600 seconds 1 
pub const DEFAULT_CREDENTIAL_MAINTENANCE_MINUTES: i64 = 10;
/// directory when new default（minutes）。
///
/// ：WorkBuddy `/v3/config` ** no has TTL **， 1 is 
/// up （ `providers::workbuddy` `refresh_models`）， etc to up 
/// none 。 1 when for 「 no 」 this 。
pub const DEFAULT_MODEL_REFRESH_MINUTES: i64 = 60;
/// before new default（seconds）：seconds 1 。
///
/// before 10 seconds， this is has ： in ** for can when **
/// request（`document.hidden` and current），，
/// 「」 in use that 1 when —— that is need when when 。
/// is （ 1 、 1 statistics）， not 。
pub const DEFAULT_LOGS_AUTO_REFRESH_SECONDS: i64 = 1;
pub const DEFAULT_REQUESTS_AUTO_REFRESH_SECONDS: i64 = 1;
pub const DEFAULT_REPORT_AUTO_REFRESH_SECONDS: i64 = 1;
/// checkdefault（minutes）： 20 minutes 1 GitHub new 。
///
/// 20 minutes = 3 / when ， for （60 / when ， and ** by IP ** —— 1 
/// down its use this ）， can new in 1 
/// 。** this default「 no 」 **： already 
/// `scheduledTasks.updateCheck.interval` config by （`interval_field`
/// in or when to default）， not will use set。
pub const DEFAULT_UPDATE_CHECK_MINUTES: i64 = 20;

/// 。 up down （minutes / seconds）， as 
/// ： after by minutes（1 minutes～1 ）， before new by seconds（1 seconds～10 minutes）。
///
/// seconds down to 1 seconds： this is ** can ** （ not 、 not up ），
/// 1 is 「 1 」， not will to 。
/// 5 seconds down no has by ， is before 10 seconds 。
///
/// and ：**（`scheduled_tasks::parse_interval`） and 
/// （`interval_field`） use this **，otherwise will 「 60 file」。
pub const INTERVAL_MIN_MINUTES: i64 = 1;
pub const INTERVAL_MAX_MINUTES: i64 = 1440;
pub const INTERVAL_MIN_SECONDS: i64 = 1;
pub const INTERVAL_MAX_SECONDS: i64 = 600;

/// 1 config： + （ by ）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntervalTask {
    pub enabled: bool,
    /// ，（minutes or seconds）
    pub interval: i64,
}

/// config（set「 when 」）。
///
/// and `RetentionSettings` 1 ： total is 1 use （GET 1 return、
/// need ）， 1 `Copy` use 1 to 、 not 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduledSettings {
    pub credential_maintenance: IntervalTask,
    pub model_refresh: IntervalTask,
    pub logs_auto_refresh: IntervalTask,
    pub requests_auto_refresh: IntervalTask,
    pub report_auto_refresh: IntervalTask,
    pub update_check: IntervalTask,
}

impl Default for ScheduledSettings {
    fn default() -> Self {
        Self {
            credential_maintenance: IntervalTask {
                enabled: true,
                interval: DEFAULT_CREDENTIAL_MAINTENANCE_MINUTES,
            },
            model_refresh: IntervalTask {
                enabled: true,
                interval: DEFAULT_MODEL_REFRESH_MINUTES,
            },
            logs_auto_refresh: IntervalTask {
                enabled: true,
                interval: DEFAULT_LOGS_AUTO_REFRESH_SECONDS,
            },
            requests_auto_refresh: IntervalTask {
                enabled: true,
                interval: DEFAULT_REQUESTS_AUTO_REFRESH_SECONDS,
            },
            report_auto_refresh: IntervalTask {
                enabled: true,
                interval: DEFAULT_REPORT_AUTO_REFRESH_SECONDS,
            },
            update_check: IntervalTask {
                enabled: true,
                interval: DEFAULT_UPDATE_CHECK_MINUTES,
            },
        }
    }
}

/// 1 ****update（`None` = this not ）。
#[derive(Clone, Copy, Debug, Default)]
pub struct IntervalTaskPatch {
    pub enabled: Option<bool>,
    pub interval: Option<i64>,
}

// ─── requestset and （config.json ** is **）─────
//
// this （ `upstream::provider_loop::send_with_retry`
// and `attempt_queue`）。
//
// ── Why（ / ）──────────────
// 1 failure after has ， and not ：
// - ** in 1 up **：， can can is when or ，
// etc 1 1 —— this is `retryCount`；
// - ** 1 **： need 1 and ， can can is 1 ，
// use 「 in current， in not 」—— this is 
//     `retryCrossProviderCount`。
//
// （`provider_loop::attempt_queue` ）：
// - request** in that ** use `retryCount`，；
// - ** 1 **（ not is down 1 is 1 ） 1 
// `retryCrossProviderCount`， after 1 error，
// not down 。
//
// ── Why by 「」 not is by 「」（2026-09 ）────
// old by ： 1 down all 1 、 use 1 ，
// has new to 1 。 this has after ， and use for this not ：
// - 「 can 」 no has set —— totallimit，
// 9 when ， 1 request will 1 to 10 （ use ）；
// - use 5 in 「 after 」， already is ，
// etc to that when no has available。
// in 「」： not to 1 ， 1 1 。
// `retryCrossProviderCount` not （ will config not to 、default，
// ， not ）—— is ， as 。

/// ** 1 ** （0 = failure， not ）
pub const KEY_RETRY_COUNT: &str = "retryCount";
/// **** （0 = not ，）。
///
/// ── and JSON Why for not up ─────────────────────────
/// （`retryCrossProviderCount`） is **config**： will config
/// not to 、default， 1 ， not —— 
/// 。（Rust ） is ：** by ， not **，
/// to down 1 and to 1 1 。 。
pub const KEY_RETRY_ACCOUNT_SWITCH_COUNT: &str = "retryCrossProviderCount";
/// etc seconds
pub const KEY_RETRY_INTERVAL_SECONDS: &str = "retryIntervalSeconds";

/// 1 default：failure after 3 （ 4 ）
pub const DEFAULT_RETRY_COUNT: i64 = 3;
/// default：failure after 5 。
///
/// that is ： need in 「 etc to down 1 available」，
/// 1 failure this not ， failure。
pub const DEFAULT_RETRY_SWITCH_COUNT: i64 = 5;
/// default：5 seconds
pub const DEFAULT_RETRY_INTERVAL_SECONDS: i64 = 5;

/// 「error」 config（ is HTTP ， `[402, 429]`）。
///
/// `NoRetry` is （ is 「 in 」）， as after 
/// 「 not in 1 、 down 1 」—— is config，
/// will config not to ，。 in up failure： and 
/// （ / 401 new ） not ， by down 1 
/// ，failure error to client。
pub const KEY_RETRY_NO_RETRY_CODES: &str = "noRetryStatusCodes";
/// default：402（WorkBuddy not ）。 not ，
/// client to 402 can 「 this no 」。
pub const DEFAULT_NO_RETRY_CODES: &[u16] = &[402];
/// ：HTTP in 100–599
pub const RETRY_CODE_MIN: u16 = 100;
pub const RETRY_CODE_MAX: u16 = 599;
/// limit：total 500 ，50 config，
/// 1 and config.json 
pub const RETRY_MAX_NO_RETRY_CODES: usize = 50;

/// and 。
///
/// limit 10 / 300 seconds： or will client etc （ is 「 1 」，
/// is 「 1 」， not is request）。 down 0： 0 = 
/// this （ not / not ）， 0 = 。
pub const RETRY_MIN_COUNT: i64 = 0;
pub const RETRY_MAX_COUNT: i64 = 10;
pub const RETRY_MIN_INTERVAL_SECONDS: i64 = 0;
pub const RETRY_MAX_INTERVAL_SECONDS: i64 = 300;

/// requestset（set「 use → request」）。
///
/// and `RetentionSettings` 1 ： total is 1 use （
/// ）， 1 use 1 to 、 not 。
/// is `Copy` ；`no_retry_codes` after can `Clone`
/// （`Arc` not is Copy）—— 1 ，pathnone。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetrySettings {
    /// ** 1 ** （0 = failure）
    pub count: i64,
    /// **** limit（0 = not ，）。
    ///
    /// old 、JSON no （
    /// [`KEY_RETRY_ACCOUNT_SWITCH_COUNT`]）—— is config， will config
    /// not to 。 is ：** by ， not **， to down 1 
    /// and to 1 1 。
    pub account_switch_count: i64,
    /// （seconds）
    pub interval_seconds: i64,
    /// **error**（[`KEY_RETRY_NO_RETRY_CODES`]）。
    ///
    /// Why is `Arc<[u16]>` not is `Vec<u16>`： failurerequest use ，
    /// `Arc` 1 ；（`no_retry`） is ，
    /// not need need 。
    pub no_retry_codes: Arc<[u16]>,
}

impl RetrySettings {
    /// ms（ sleep use ）
    pub fn delay_ms(&self) -> u64 {
        self.interval_seconds.max(0) as u64 * 1000
    }

    /// 1 （ and [`Self::switch_budget`] is ）。
    ///
    /// by 0 ：`bounded_int_field` already ， this is 
    /// （ `usize` will ， that will ）。
    pub fn resend_budget(&self) -> usize {
        self.count.max(0) as usize
    }

    /// limit（ 1 1 ，）。
    pub fn switch_budget(&self) -> usize {
        self.account_switch_count.max(0) as usize
    }

    /// this up is in 「」。
    pub fn no_retry(&self, status: u16) -> bool {
        self.no_retry_codes.contains(&status)
    }
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            count: DEFAULT_RETRY_COUNT,
            account_switch_count: DEFAULT_RETRY_SWITCH_COUNT,
            interval_seconds: DEFAULT_RETRY_INTERVAL_SECONDS,
            no_retry_codes: Arc::from(DEFAULT_NO_RETRY_CODES),
        }
    }
}

/// request ****update（`None` = this not ）。
#[derive(Clone, Debug, Default)]
pub struct RetryPatch {
    pub count: Option<i64>,
    pub account_switch_count: Option<i64>,
    pub interval_seconds: Option<i64>,
    pub no_retry_codes: Option<Vec<u16>>,
}

// ─── up request when （ ， for OmniProxy set）──────────────

/// when ： up TCP/TLS or etc when （seconds）
pub const KEY_TIMEOUT_CONNECT_SECONDS: &str = "connectTimeoutSeconds";
/// etc response when ：request after etc up response when （seconds）
pub const KEY_TIMEOUT_HEADERS_SECONDS: &str = "headersTimeoutSeconds";
/// responseempty when ：response empty when （seconds）， to new after new when 
pub const KEY_TIMEOUT_STREAM_IDLE_SECONDS: &str = "streamIdleTimeoutSeconds";
/// response when ：response when （seconds）
pub const KEY_TIMEOUT_BODY_SECONDS: &str = "bodyTimeoutSeconds";

/// when default：30 seconds（ and egress 1 ）
pub const DEFAULT_TIMEOUT_CONNECT_SECONDS: i64 = 30;
/// etc response when default：300 seconds（ and request.rs HEADERS_TIMEOUT_MS 1 ）
pub const DEFAULT_TIMEOUT_HEADERS_SECONDS: i64 = 300;
/// empty when default：300 seconds（ and OmniProxy stream_idle_timeout 1 ）
pub const DEFAULT_TIMEOUT_STREAM_IDLE_SECONDS: i64 = 300;
/// response when default：300 seconds（ and OmniProxy body_timeout 1 ）
pub const DEFAULT_TIMEOUT_BODY_SECONDS: i64 = 300;

/// when （seconds）： and OmniProxy 1~3600 1 。
///
/// down 1 not is 0：0 in this no has （「 when 」 etc use ，
/// use 1 its need is to limit）。
pub const TIMEOUT_MIN_SECONDS: i64 = 1;
pub const TIMEOUT_MAX_SECONDS: i64 = 3600;

/// up request when （set「 use → request when 」）。
///
/// 1 ， and OmniProxy connect / headers / stream_idle / body
/// 1 1 for ；request 1 （`Copy`， i64）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeoutSettings {
    /// （ to that 1 ）
    pub connect_seconds: i64,
    /// request → response to 
    pub headers_seconds: i64,
    /// response empty
    pub stream_idle_seconds: i64,
    /// response total
    pub body_seconds: i64,
}

impl TimeoutSettings {
    /// / 0 by （： already by bounded_int_field ，
    /// `u64` before must，otherwise）
    fn ms(seconds: i64) -> u64 {
        seconds.max(TIMEOUT_MIN_SECONDS) as u64 * 1000
    }

    pub fn connect_ms(&self) -> u64 {
        Self::ms(self.connect_seconds)
    }

    pub fn headers_ms(&self) -> u64 {
        Self::ms(self.headers_seconds)
    }

    pub fn stream_idle_ms(&self) -> u64 {
        Self::ms(self.stream_idle_seconds)
    }

    pub fn body_ms(&self) -> u64 {
        Self::ms(self.body_seconds)
    }

    /// read_timeout after limit：「 etc response」 and 「empty」 。
    ///
    /// reqwest `read_timeout` used for 1 （ before 、
    /// ）， this is —— after ，** not ** as 
    /// （ in when ，
    /// `upstream::request` and `upstream::ForwardStream`）。
    pub fn read_timeout_backstop_ms(&self) -> u64 {
        self.headers_ms().max(self.stream_idle_ms())
    }
}

impl Default for TimeoutSettings {
    fn default() -> Self {
        Self {
            connect_seconds: DEFAULT_TIMEOUT_CONNECT_SECONDS,
            headers_seconds: DEFAULT_TIMEOUT_HEADERS_SECONDS,
            stream_idle_seconds: DEFAULT_TIMEOUT_STREAM_IDLE_SECONDS,
            body_seconds: DEFAULT_TIMEOUT_BODY_SECONDS,
        }
    }
}

/// when ****update（`None` = this not ）。
#[derive(Clone, Copy, Debug, Default)]
pub struct TimeoutPatch {
    pub connect_seconds: Option<i64>,
    pub headers_seconds: Option<i64>,
    pub stream_idle_seconds: Option<i64>,
    pub body_seconds: Option<i64>,
}

// ─── etc （ up ： before has Qoder ）─────────────

/// when etc （0 = not etc ， as errorreturn）
pub const KEY_QUEUE_MAX_WAITS: &str = "queueMaxWaits";
/// etc seconds（0 = up ）
pub const KEY_QUEUE_WAIT_SECONDS: &str = "queueWaitSeconds";

/// default etc ：2 。
///
/// and （CLIProxyAPI qoder2api `queue_max_waits` default） 1 
/// ： up to 9～30 seconds， etc 1 minutes —— in 「 use can 
/// etc 」 and 「default 300 seconds etc response when 」。
pub const DEFAULT_QUEUE_MAX_WAITS: i64 = 2;
/// default etc seconds：0 = up （ up no to when to 15 seconds）
pub const DEFAULT_QUEUE_WAIT_SECONDS: i64 = 0;

/// etc 0~10：0 = etc （）。limit 10 is as 
/// 「 etc 1 will 」 etc not client —— will to 1 
/// 「 in 、 not is or 」 error， not is 1 not when 。
pub const QUEUE_MIN_MAX_WAITS: i64 = 0;
pub const QUEUE_MAX_MAX_WAITS: i64 = 10;
/// etc seconds 0~120（0 = up ）
pub const QUEUE_MIN_WAIT_SECONDS: i64 = 0;
pub const QUEUE_MAX_WAIT_SECONDS: i64 = 120;

/// etc set（set「 use → etc 」）。
///
/// and `TimeoutSettings` 1 ： total is 1 use （ 1 ），
/// 1 `Copy` use 1 to 。
///
/// ── ────────────────────────────────────────────────
/// has ** **（ before is Qoder： when up use 403 +
/// 10605 1 「 not can ， N seconds after 」）。 its no has ，
/// this set for none。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QueueSettings {
    /// etc （0 = not etc ）
    pub max_waits: i64,
    /// etc seconds（0 = up ）
    pub wait_seconds: i64,
}

impl QueueSettings {
    /// etc （ by 0： already ， this is ）
    pub fn wait_budget(&self) -> usize {
        self.max_waits.clamp(QUEUE_MIN_MAX_WAITS, QUEUE_MAX_MAX_WAITS) as usize
    }

    /// etc when （ms）；`None` = up 
    pub fn forced_wait_ms(&self) -> Option<u64> {
        let seconds = self.wait_seconds.clamp(QUEUE_MIN_WAIT_SECONDS, QUEUE_MAX_WAIT_SECONDS);
        if seconds > 0 {
            Some(seconds as u64 * 1000)
        } else {
            None
        }
    }
}

impl Default for QueueSettings {
    fn default() -> Self {
        Self {
            max_waits: DEFAULT_QUEUE_MAX_WAITS,
            wait_seconds: DEFAULT_QUEUE_WAIT_SECONDS,
        }
    }
}

/// etc ****update（`None` = this not ）。
#[derive(Clone, Copy, Debug, Default)]
pub struct QueuePatch {
    pub max_waits: Option<i64>,
    pub wait_seconds: Option<i64>,
}

// ─── ZCode （ ， api/zcode_captcha）──

/// （`zcodeCaptchaPoolTarget`）： before 。
pub const KEY_CAPTCHA_POOL_TARGET: &str = "zcodeCaptchaPoolTarget";
/// ms（`zcodeCaptchaPoolPollMs`）。
pub const KEY_CAPTCHA_POOL_POLL_MS: &str = "zcodeCaptchaPoolPollMs";
/// 1 （`zcodeCaptchaPoolMaxPerRound`）。
pub const KEY_CAPTCHA_POOL_MAX_PER_ROUND: &str = "zcodeCaptchaPoolMaxPerRound";

/// default：3 （ `POOL_TARGET` ）。 1 「 down 1 request
/// has use 」， not 1 in 120 seconds TTL use not （quota）。
pub const DEFAULT_CAPTCHA_POOL_TARGET: i64 = 3;
/// default：2500 ms（ `POLL_MS` ），「 to up 」 。
pub const DEFAULT_CAPTCHA_POOL_POLL_MS: i64 = 2_500;
/// defaultlimit：1 （ `MAX_PER_ROUND` ）。SDK 1 
/// will ，default 1 1 。
pub const DEFAULT_CAPTCHA_POOL_MAX_PER_ROUND: i64 = 1;

/// 1~10：limit「」 use —— ，
/// none when （120 seconds TTL）， is quota。
pub const CAPTCHA_POOL_MIN_TARGET: i64 = 1;
pub const CAPTCHA_POOL_MAX_TARGET: i64 = 10;
/// 1000~30000 ms： down down ，limit up 
/// 「 to up 」 will to 。 not ， not will 。
pub const CAPTCHA_POOL_MIN_POLL_MS: i64 = 1_000;
pub const CAPTCHA_POOL_MAX_POLL_MS: i64 = 30_000;
/// limit 1~5： 500ms，5 already is 1 
/// not to ； up up 。
pub const CAPTCHA_POOL_MIN_MAX_PER_ROUND: i64 = 1;
pub const CAPTCHA_POOL_MAX_MAX_PER_ROUND: i64 = 5;

/// ZCode （set「 → 」）。
///
/// and `QueueSettings` 1 ： stats、
/// request need 1 「 not 」， 1 `Copy` can 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaptchaPoolSettings {
    /// （ before ）
    pub target: i64,
    /// （ms）
    pub poll_ms: i64,
    /// 1 
    pub max_per_round: i64,
}

impl Default for CaptchaPoolSettings {
    fn default() -> Self {
        Self {
            target: DEFAULT_CAPTCHA_POOL_TARGET,
            poll_ms: DEFAULT_CAPTCHA_POOL_POLL_MS,
            max_per_round: DEFAULT_CAPTCHA_POOL_MAX_PER_ROUND,
        }
    }
}

/// ****update（`None` = this not ）。
#[derive(Clone, Copy, Debug, Default)]
pub struct CaptchaPoolPatch {
    pub target: Option<i64>,
    pub poll_ms: Option<i64>,
    pub max_per_round: Option<i64>,
}

// ─── will （ before base64 ，set「 → 」）──

/// total（`imageCompressEnabled`，**default**）。
///
/// ：** has `false` **（ `parse::image_compress_from`），
/// or by —— is up config（ no has this ） 1 is 
/// 。
///
/// Whydefault： is ** to up **，（ request、 up 
/// ） for for ，「default this has 」 can ； has 「
/// 」 this request ， after use ，
/// **default、 can 1 ** ， to 「request 1 not 」。
pub const KEY_IMAGE_COMPRESS_ENABLED: &str = "imageCompressEnabled";
/// limit（`imageCompressMaxEdge`）： then etc 。
pub const KEY_IMAGE_COMPRESS_MAX_EDGE: &str = "imageCompressMaxEdge";
/// JPEG （`imageCompressQuality`，40~95）。
pub const KEY_IMAGE_COMPRESS_QUALITY: &str = "imageCompressQuality";
/// KB（`imageCompressThresholdKb`）： after （0 = ）。
pub const KEY_IMAGE_COMPRESS_THRESHOLD_KB: &str = "imageCompressThresholdKb";

/// limitdefault：2048（** OmniProxy default**）。
///
/// limit；default and OmniProxy 1 ，
/// use need can in set（ CatPaw up limit use 1568）。
pub const DEFAULT_IMAGE_COMPRESS_MAX_EDGE: i64 = 2048;
/// default：80（ OmniProxy）。
pub const DEFAULT_IMAGE_COMPRESS_QUALITY: i64 = 80;
/// default：512 KB（ OmniProxy）—— after 512KB not 。
pub const DEFAULT_IMAGE_COMPRESS_THRESHOLD_KB: i64 = 512;

/// limit 256~8192： down down not ，limit up etc not 。
pub const IMAGE_COMPRESS_MIN_MAX_EDGE: i64 = 256;
pub const IMAGE_COMPRESS_MAX_MAX_EDGE: i64 = 8192;
/// 40~95： 40 ， 95 as 。
pub const IMAGE_COMPRESS_MIN_QUALITY: i64 = 40;
pub const IMAGE_COMPRESS_MAX_QUALITY: i64 = 95;
/// 0~10240 KB：0 = not 、 1 。
pub const IMAGE_COMPRESS_MIN_THRESHOLD_KB: i64 = 0;
pub const IMAGE_COMPRESS_MAX_THRESHOLD_KB: i64 = 10240;

/// will set（set「 → 」）。
///
/// and `QueueSettings` 1 ： total is 1 use （ before 1 ），
/// 1 `Copy` use 1 to —— this request down 。
///
/// ── use in ──────────────────────────────────────────────
/// in ** for **（chat / responses / anthropic 1 after Chat request）
/// ； and ， in `send_body` before return， not this 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageCompressSettings {
    /// total（ = request 1 not ）
    pub enabled: bool,
    /// limit（ etc ， not ）
    pub max_edge: i64,
    /// JPEG 
    pub quality: i64,
    /// （KB， after ）
    pub threshold_kb: i64,
}

impl Default for ImageCompressSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_edge: DEFAULT_IMAGE_COMPRESS_MAX_EDGE,
            quality: DEFAULT_IMAGE_COMPRESS_QUALITY,
            threshold_kb: DEFAULT_IMAGE_COMPRESS_THRESHOLD_KB,
        }
    }
}

/// will ****update（`None` = this not ）。
///
/// and `AutoContinuePatch` （ + if has 1 ）， by 1 ：
/// ** 1 PUT 1 **，「、parameter is old 」。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageCompressPatch {
    /// （`None` = this not ）
    pub enabled: Option<bool>,
    /// limit（`None` = this not ）。 use by 
    /// `IMAGE_COMPRESS_MIN_MAX_EDGE..=IMAGE_COMPRESS_MAX_MAX_EDGE` 。
    pub max_edge: Option<i64>,
    /// （`None` = this not ）。 use by 
    /// `IMAGE_COMPRESS_MIN_QUALITY..=IMAGE_COMPRESS_MAX_QUALITY` 。
    pub quality: Option<i64>,
    /// KB（`None` = this not ）。 use by 
    /// `IMAGE_COMPRESS_MIN_THRESHOLD_KB..=IMAGE_COMPRESS_MAX_THRESHOLD_KB` 。
    pub threshold_kb: Option<i64>,
}

// ─── by （ / will / failure / / ）─────────

/// by ：（`routingStrategy`）。
///
/// - `Order`：（） is 1 —— as ；
/// - `Rotate`：request to down 1 （ new will by ），usage；
/// - `Usage`：usage use （ no has by in has after ）。
///
/// can by 、 after ** not **（ use ：）。
pub const KEY_ROUTING_STRATEGY: &str = "routingStrategy";
/// will ： 1 will （`routingSticky`）。
///
/// - `Auto`：， to up not （ 5 minutesnone）；
/// - `Session`：（6 when will ）；
/// - `Turn`： in request not （ and has as 1 ，request not ）；
/// - `Off`：request。
pub const KEY_ROUTING_STICKY: &str = "routingSticky";
/// failure：failure 1 will （`routingFailoverBackoff`，default）
pub const KEY_ROUTING_FAILOVER_BACKOFF: &str = "routingFailoverBackoff";
/// （Sink）： has when 429 to ，**30 minutes after to **，
/// or before success 1 （`routingSink`，default）。
///
/// `route_state::SINK_TTL_MS` as ， in this 1 ： from before this is 
/// 「 1 」， that is （429 → to → not to →
/// no will success → not ； 11 after ）。
pub const KEY_ROUTING_SINK: &str = "routingSink";
/// （`routingBreakerEnabled`，default）
pub const KEY_ROUTING_BREAKER_ENABLED: &str = "routingBreakerEnabled";
/// ：failure after （`routingBreakerThreshold`）
pub const KEY_ROUTING_BREAKER_THRESHOLD: &str = "routingBreakerThreshold";
/// when （seconds）： after 1 （`routingBreakerTimeoutSeconds`）
pub const KEY_ROUTING_BREAKER_TIMEOUT: &str = "routingBreakerTimeoutSeconds";
/// ： after need success（`routingBreakerSuccessThreshold`）
pub const KEY_ROUTING_BREAKER_SUCCESS: &str = "routingBreakerSuccessThreshold";

pub const ROUTING_STRATEGY_ORDER: &str = "order";
pub const ROUTING_STRATEGY_ROTATE: &str = "rotate";
/// （`provider`）： by ， 1 use 。
///
/// and 「」 in ** **： this after （ by ），
/// has —— 「 use 、 not 1 」。
pub const ROUTING_STRATEGY_PROVIDER: &str = "provider";

pub const STICKY_OFF: &str = "off";
pub const STICKY_TURN: &str = "turn";
pub const STICKY_SESSION: &str = "session";
pub const STICKY_AUTO: &str = "auto";

/// ：1~20 
pub const ROUTING_BREAKER_MIN_THRESHOLD: i64 = 1;
pub const ROUTING_BREAKER_MAX_THRESHOLD: i64 = 20;
/// when ：10~600 seconds
pub const ROUTING_BREAKER_MIN_TIMEOUT: i64 = 10;
pub const ROUTING_BREAKER_MAX_TIMEOUT: i64 = 600;
/// ：1~10 
pub const ROUTING_BREAKER_MIN_SUCCESS: i64 = 1;
pub const ROUTING_BREAKER_MAX_SUCCESS: i64 = 10;

/// by （`RouteStrategy` Rust ；config `as_str`）
///
/// ：（ 1 allrequest）、（request）、
/// （ by 、 1 use ）。
/// 「（usage）」 already down （2026-10， by use need ）—— `from_id` 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteStrategy {
    Order,
    Rotate,
    /// （add， up no has ）： in 「」
    Provider,
}

impl RouteStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Order => ROUTING_STRATEGY_ORDER,
            Self::Rotate => ROUTING_STRATEGY_ROTATE,
            Self::Provider => ROUTING_STRATEGY_PROVIDER,
        }
    }

    /// config → 。**unknown 1 **（ and old as 1 ）。
    ///
    /// already down `"usage"`（） to **** not is default： that config is 
    /// use 「empty up 」， up is 「 」（rotate），
    /// 「 1 allrequest」 will and 。
    pub fn from_id(value: &str) -> Self {
        match value {
            ROUTING_STRATEGY_PROVIDER => Self::Provider,
            ROUTING_STRATEGY_ROTATE | "usage" => Self::Rotate,
            _ => Self::Order,
        }
    }
}

/// will （`StickyMode`）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StickyMode {
    Off,
    Turn,
    Session,
    Auto,
}

impl StickyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => STICKY_OFF,
            Self::Turn => STICKY_TURN,
            Self::Session => STICKY_SESSION,
            Self::Auto => STICKY_AUTO,
        }
    }

    pub fn from_id(value: &str) -> Self {
        match value {
            STICKY_SESSION => Self::Session,
            STICKY_TURN => Self::Turn,
            STICKY_OFF => Self::Off,
            _ => Self::Auto,
        }
    }

    /// when （ms）；`None` = not （Off / Turn）
    pub fn ttl_ms(&self) -> Option<i64> {
        match self {
            Self::Auto => Some(5 * 60 * 1000),
            Self::Session => Some(6 * 60 * 60 * 1000),
            Self::Off | Self::Turn => None,
        }
    }
}

/// by set（「 by 」）。request 1 
/// （`Copy`），set down 1 request —— and when / 1 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoutingSettings {
    pub strategy: RouteStrategy,
    pub sticky: StickyMode,
    pub failover_backoff: bool,
    pub sink: bool,
    pub breaker_enabled: bool,
    pub breaker_threshold: i64,
    pub breaker_timeout_seconds: i64,
    pub breaker_success_threshold: i64,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            strategy: RouteStrategy::Order,
            sticky: StickyMode::Auto,
            failover_backoff: true,
            sink: true,
            breaker_enabled: false,
            breaker_threshold: 5,
            breaker_timeout_seconds: 60,
            breaker_success_threshold: 2,
        }
    }
}

/// by set ****update（`None` = this not ）
#[derive(Clone, Copy, Debug, Default)]
pub struct RoutingPatch {
    pub strategy: Option<RouteStrategy>,
    pub sticky: Option<StickyMode>,
    pub failover_backoff: Option<bool>,
    pub sink: Option<bool>,
    pub breaker_enabled: Option<bool>,
    pub breaker_threshold: Option<i64>,
    pub breaker_timeout_seconds: Option<i64>,
    pub breaker_success_threshold: Option<i64>,
}

// ─── （autoContinue / autoContinueMaxRounds）─────────
//
// 、 and in `core::upstream::continuation`（、
// in that module header）， this 「 not 」 and 「」 。
//
// ── Why is ， not is 1 off/1/3/8 ──────────────
// 「 need not need 」 and 「」 is ： 1 （） and 
// ， in 「 need 」 this before down 。 1 will 「」 this **default**
// —— is config 1 （0 and off），
// will is when 。

/// 「 up to 1 、 」 （config.json ，
/// `core::upstream::continuation`）。
///
/// **default**， and `stripNewlineKeepalive` 1 to ， to not ： that 
/// is 「client to 」， is ** up ** ——
/// 1 before ， 1 not before 1 （ and 
/// `core::upstream::continuation` module header）。 can as + ，
/// default。
///
/// when is ， need （** not ** in before ， that is client 
/// ）； when use not ， down and before 1 。 1 in 
/// by and `/api/keepalive-strip` —— 「 in up 1 、 not for 」
/// not this need need new （ `api::auto_continue`）。
///
/// ：config no has this when `AGENT2API_AUTO_CONTINUE`
/// （`1` / `true` / `on` ， its empty， and 
/// `AGENT2API_STRIP_NEWLINE_KEEPALIVE` use 1 ，
/// `parse::env_auto_continue`）—— Docker from 1 ，
/// not 1 。
pub const KEY_AUTO_CONTINUE: &str = "autoContinue";

/// **limit**（config.json ）： 1 request「 1 」。
///
/// is ****： 1 （） is 「 this is not is 
/// up 1 」， 0.5 is ， in up 。limit is 
/// that ， not —— this is 1 
/// （ [`AUTO_CONTINUE_MAX_ROUNDS`]）， `core::upstream::continuation` module header。
///
/// `AGENT2API_AUTO_CONTINUE_MAX`： or 「 no 」，
/// and file 1 （**default not is **， by `parse` module header）。
pub const KEY_AUTO_CONTINUE_MAX_ROUNDS: &str = "autoContinueMaxRounds";

/// limitdefault：3 —— （ Trae2api-cn `continuation.py`）
/// default，「 1 1 」 ； and 
/// `DEFAULT_RETRY_COUNT` that 「」 is 1 to 。
pub const DEFAULT_AUTO_CONTINUE_MAX_ROUNDS: i64 = 3;

/// down ：1，** not 0**。
///
/// 1 not ， and 「」 is 1 ； this in config will 
/// as can 。 [`KEY_AUTO_CONTINUE`]， not need use 
/// （ and `retryCount` that ： that 0 is 「 not 、」 as ）。
pub const AUTO_CONTINUE_MIN_ROUNDS: i64 = 1;

/// limit：8。
///
/// 1 is 1 up （ before prompt）， 9 
/// ；「 to no 」 this in ，
/// this is not is 。
pub const AUTO_CONTINUE_MAX_ROUNDS: i64 = 8;

/// ****update（`None` = this not ）。
///
/// and `RoutingPatch` （ + if has 1 ）， in by is 
/// ** 1 PUT 1 **： 1 will 「 already 、limit is old 」 
/// ， this need 1 has 。
#[derive(Clone, Copy, Debug, Default)]
pub struct AutoContinuePatch {
    /// （`None` = this not ）
    pub enabled: Option<bool>,
    /// limit（`None` = this not ）。 use by 
    /// `AUTO_CONTINUE_MIN_ROUNDS..=AUTO_CONTINUE_MAX_ROUNDS` 。
    pub max_rounds: Option<i64>,
}
