//! DDL and schema （`PRAGMA user_version` ）。
//!
//! ── Why not use rusqlite_migration ─────────────
//! use is 「 1 file + and + + 」。
//! ** 1 6 、 not **（ old not 、 new can empty `data` JSON ），
//! is 「 → DDL → 」。 will ：
//! 1 new 、 1 （ and `user_version` 1 ， 1 ）、
//! and 「 etc 」 this need need can 。
//! can 、 when can for file， 1 to 。
//!
//! ── Why DDL in in file ─────────────────────────────
//! is ** store **：（ store ） and （API）
//! not file、 not ， must has 1 1 。 in 
//! store `CREATE TABLE` ， 1 will 「」 file
//! （ 1 ，A has this 、B no has ）。 in after 「」
//! use this 1 file。
//!
//! ── ────────────────────────────────────────────
//! `user_version`（default 0 = new ）， for to [`SCHEMA_VERSION`] 
//! by use [`apply_version`]， after `user_version` 
//! [`SCHEMA_VERSION`]。** 1 is 1 **（DDL and 1 
//! ）： in failure when already that 、 after 1 ， down from 
//! —— not will 「 1 already new 」 。
//!
//! ── Whyall IF NOT EXISTS ────────────────────────
//! is ** 1 ** ， in 「 already in 、 is 0」 
//! ： use file、or from old before 。DDL etc after this 
//! can （ up 、 has ）， not is in when 1 
//! 「table already exists」 use in 。
//!
//! ── kv （ after by ）──────────────
//! `kv` is 「 not 」 ，：
//! - **config（ `config.json`） use **， 1 not ：
//!     `apiKeys`、`modelRules`、`logRetentionDays`、`debugMode`……
//! Why not is snake_case： this when is **HTTP API 
//! **（`GET /api/config` response、 before `config.X` ）， will 
//! before 、、 then when ； and JSON 
//! 1 1 for ， when can configfile for 。
//! and configfile 1 ：** is that JSON**（ `"30"`、
//! for `{...}`、 `[...]`）， `config.json` → `kv` 
//! is 「 」， not need need as 。
//! - ** its use **（， and before ）： [`RESERVED_KV_KEYS`]。
//! Note `desktopSettings` is ****set for not is ——
//! set is `settings.rs` （ before 1 `PUT` ），
//! will 「 1 」update， and 。
//!
//! ── Why not can （T7 this is 1 ）──────────
//! `config.json` `kv` after ，「config」 not by ** **
//! （ before config 1 file）， is by is in [`RESERVED_KV_KEYS`] 
//! 。`config::save_raw` 「delete already not in 」 need by this excluding ——
//! config if and ， 1 config will that ****
//! （e.g.config `desktopSettings`， use 1 API Key set）。
//! currentconfig（`apiKeys` / `modelRules` / `logRetentionDays` / `debugMode` /
//! `sanitizeBlacklistFingerprints` / `promptMode` / `promptFile` /
//! `scheduledTasks` / `autoCheckin` / `locale` / `lastRequestModel` /
//! `providerRoute` / `updateProxy` / `updateBeta` / `*Dir` / `*RetentionDays` /
//! `retry*` / `scheduledTasks.*` / `*Imported` /
//! old `apiKey`） and **none**， for （ `update_raw_field` /
//! `raw.insert` / `KEY_*` vs down `RESERVED_KV_KEYS`）。
//! after when **must** for 1 ： is use config or 
//! ，（ 1 in file， 1 in use ）。

use rusqlite::Connection;

/// `kv` ** not config** （module header「 not can 」 
/// is and config ）。
///
/// Why need has 1 in ：`config::save_raw` must「 not 」，
/// can in config when 「config already 」 not 。 no has this 
/// ， that delete can 「all not in new config 」—— set、
/// use 、 down 1 id will in use 1 API Key when 。
///
/// is ****： use （`logs_store::sql::NEXT_ID_KEY`）
/// is when use ，
/// 「config need excluding」 this 1 。 if not 1 ， is config
/// —— add when ** need **（ use + ），
/// [`is_reserved`] is 1 。
///
/// can can has ** no has use ** （ `desensitize`）： that is 
/// 「 can already 、」 ， 1 is config。
pub const RESERVED_KV_KEYS: &[&str] = &[
    // use （core::account_store::sql）
    // 「Agent」 use config：{ clientId: [Profile] } JSON 1 
    // （core::agents）。， and desktopSettings 1 。
    "agentProfiles",
    "priorityScope",
    // （core::sidebar_layout）：「 / / 」JSON 1 。
    // from before localStorage， and is origin、localStorage
    // —— is 「 in WebUI ，client old 」。 after 1 。
    // 「 its 」：config not can （otherwise use 1 API Key
    // default）。
    "sidebarLayout",
    // down 1 id （logs_store::sql）
    "logsNextId",
    // set JSON（ settings）—— 1 not is ：
    // set is （ before 1 PUT ）， will 1 
    // update， and 。
    "desktopSettings",
    // ****： old 「」 （ / / use / use ）。
    //
    // that can already then （`core::sanitize`）delete，
    // this ** not has **。 in this is has ：config
    // not will this —— use that （ can 
    // `sqlite3` ）， not is as 1 API Key 。 need when 
    // this 1 can 。
    "desensitize",
    // （server::access）：{username, hash} JSON 1 ——
    // in up （ when ），「 its 
    // 」 1 ： not is config，config not can 。
    "panelAdmin",
    // new （server::access）： 1 （ is ：
    // 1 ），/ is 。 new in sha256。
    "panelTokens",
    // access will （server::access）： and `panelTokens` 1 、
    // sha256。 by is 「 not this all already invalid」—— before 
    // in ， after in cookie 401， 401 not cookie，
    // as 「 1 for not up 」。
    "panelAccessSessions",
    // ALTCHA HMAC （server::altcha）： challenge
    // when and ， HMAC use 、 from not 。「 its 」，
    // config not can 。
    "panelAltchaSecret",
    // config.json （db::migrate::config）。Whyconfig this 1 
    // ** need need ** its not need need ：config is ， no 「config
    // no has 」（ by and db::migrate::config module header）。
    "configMigrated",
    // request / （request_stats::legacy）。
    //
    // this use ， `logs` / `accounts` / `debug_traffic` not use ，
    // Reason is ** that has available **（ / uuid / request id），
    // 「 this already 」 can by （`INSERT OR IGNORE`）；
    // `requests` is `row_id`（ and none）、`id` in old can can is 
    // empty，`request_daily` `date` will UPSERT —— not 
    // 「 this already 」 。
    // `request_stats::legacy::MARKER_REQUESTS` 。
    //
    // in this is **must** ：`config::save_raw` config when need excluding
    // 「 not 」 ， will use 1 API Key this 
    // —— down 「」 will 1 。
    "requestsMigrated",
    "dailyMigrated",
    // usage （request_stats::aggregate 
    // `MODEL_UPSTREAM_MARKER`）—— #139 is this ：
    // in `remap_model_days` ****（ and 1 ， has ：
    // in after no 、 down ）， `config::init` 
    // in that before —— not ， use any 1 config when `save_raw` will 
    // 「config already 」， down 1 
    // （ when is etc 、； down ）。
    "modelUsageUpstreamRemapped",
    // （core::providers::catalog_cache）：
    // 「 up success to 」 1 （ 1 by that module header）。
    // 「 its 」—— not is config，config not can 。
    "modelCatalogCache",
    // after and （core::task_state）：「 
    // up / up success / down / failure / 」 1 。
    // before 「 in 、 from 」 —— is not 
    // GitHub check、、 new 、 1 up request。
    // 「 its 」：config not can （otherwise、）。
    "backgroundTaskState",
    // 1 and ，only， not and config or 。
    "backgroundTaskStateRecovery",
    // old 「 when 」 （ already ）： in is 
    // —— `core::usage_records` when account_usage_records
    // after delete； if that failure， this not config 1 。
    // success after this not ， in is 。
    "usageQuerySnapshot",
    // （core::proxy_pool）： `{items: [...]}` 1 ——
    // 「」 ， can by id use 。「 its 」：
    // not is config（config not can ，otherwise use will 1 API Key
    // empty）。 not 、， by that module header。
    "proxyPool",
    // and old ，config not can this 。
    "proxySettings",
    // can already down ； old ， not configdelete。
    "subscriptions",
    // models.dev new （core::model_meta::snapshot）：
    // 「 up success to 」 1 （file and ，
    // `scripts/sync-modelsdev.mjs`）。** not is **config —— config not can ，
    // otherwise use 1 API Key when that old 。
    // and up `modelCatalogCache` is ： that up ****，
    // this ****（ can / ）， new and not 。
    "modelsDevCatalog",
    // config and （core::cloud_sync）：
    // `{provider, webdav:{…}, s3:{…}, syncPassword(), enabled, autoSync,
    // remoteRoot, profile, status:{lastSyncAt, lastSnapshotId, …}}` 1 。
    //
    // Why is not is config：****（WebDAV / S3 Secret
    // Access Key ） and ，「 its 」 1 ；
    // is ** not can up ** —— otherwise etc to ，
    // when will old 。 `cloud_sync::snapshot` 
    // excluding， not 1 「 this 」 。
    "cloudSync",
];

/// this is 「 its 」（ not config）。
///
/// 1 ：`config::save_raw` use not and 「delete already not in 」。
/// not is use `.contains()`： will if before ，
/// 1 can （ and `db::migrate` 1 `sql::has_key` 1 ）。
pub fn is_reserved(key: &str) -> bool {
    RESERVED_KV_KEYS.contains(&key)
}

/// current schema 。 / when +1， and in [`apply_version`] 1 。
///
/// Why from 1 not is 0 ：0 is new ， use 「 already is new 」 will 
/// new 。1 after 「 no has 」 and 「schema 1」 is 。
///
/// ── 2：requests JSON ──────────────────────
/// [`V2_SCHEMA`]。 new will by 1 → 2 （v1 when ** not ** this ，
/// by v2 ALTER up ），「 new 」 and 「 from v1 up 」 1 ——
/// this is Why v1 DDL ** not ** this ： that will path
/// （ new has 、 ALTER has ， ， down 1 ）。
///
/// ── 3：request_raw （， for can ）───────
/// [`V3_SCHEMA`]。 and v2 ： this is ** new ** not is ，
/// `CREATE TABLE IF NOT EXISTS`（ not need need ALTER， not need need 「v1 not 、v2 」
/// that ）。（user_version=2） and new will v3 —— this 1 
/// ， new in v1 ** not **（ has v3 1 ， and v2 that 
/// 「 not need v1」 ）。
///
/// ── 4： v3 ** etc 1 **（）──────
/// [`V4_SCHEMA`]。 this not is 1 ， is for 「 3、 not in 」
/// this ： has `user_version=3` no has `request_raw`（
/// in this down DDL，）。 after is that 
/// `DELETE FROM request_raw` 「no such table」→ ** ** → 
/// in not （ up as 「 has in 、 no has 」）。
/// v3 DDL is `CREATE TABLE IF NOT EXISTS`， 1 ：
/// in ， not in up ， 1 。
///
/// ── 5：requests etc ──────────────────────
/// [`V5_SCHEMA`]。 and v2 1 （ has ，ALTER ）：
/// v1 DDL not ，path（ new 1→5 / from any up ）
/// to 1 。
///
/// ── 6：requests and when ───────────────────
/// [`V6_SCHEMA`]。 is has （ALTER ）。 this in ** in **
/// has ， when 1 empty（ `request_stats::sql` ）——
/// 「 has no has 」 is 「 this 1 in 」 ， and status=0 。
///
/// ── 7：requests 「」 1 ─────────────────────
/// [`V7_SCHEMA`]。 is has （ ALTER）。 is 
/// ， request will request —— this 1 and ，
/// is can excluding（ is ， will ），
/// request that old can to 、 and 「」。
/// ── 8：account_usage_records （ ）─────
/// [`V8_SCHEMA`]。 and v3 1 （ new ，`CREATE TABLE IF NOT EXISTS`，
/// and new will this 1 ）。 old kv `usageQuerySnapshot`
/// as 「 1 」 ：「 when 」
/// after by to ， 1 1 can update、 
/// （kv not this ）。 old by `core::usage_records`
/// when ， after not will empty。
///
/// ── 11：checkin_records （ to ）───────
/// [`V11_SCHEMA`]。 and v3 / v8 1 （ new ，`CREATE TABLE IF NOT EXISTS`）。
/// is to in 「 to 」 ： 1 to ， 1 
/// （success / already / failure and its error / ）， up before 「 has 
/// config.json 1 lastResult」 not 。 is 
/// `core::checkin_records`（ 1 ：`core::billing::checkin::run_checkin`）。
///
/// ── 12：requests 「client」 1 ─────────────────────
/// [`V12_SCHEMA`]。 and v5 / v10 1 （ has ， ALTER）。
/// request 「client」： handler from down `User-Agent` by 
/// `core::client` then client（`Codex` / `ZCode` / `Claude Code`
/// …），。 not is UA：UA ，
/// not 「 is 」。 old DEFAULT '' = no has （ that when no ），
/// —— not to old 。
///
/// ── 13：empty `request_raw` （ 1 empty）─────
/// for down from as **only when **
/// （ for OmniProxy ： not not ）， before 
/// none down that in empty —— for use not has （ new down 
/// not will has new when ），empty。
///
/// Note this 1 is **** not is （ 1 ）。 in schema
/// not is kv ：「 1 」， is has etc 
/// （`user_version` and ）， use is add —— that 
/// kv will 「 this no has 」 。
///
/// not `*_SCHEMA` in `apply_version` v13 ：
/// need ** **，`migrate` is VACUUM（ empty
/// to file —— DELETE not will file，「empty」 is this 1 
/// ）， `migrate` that 。
///
/// ── 14： `data` as （`core::crypto` `enc1:` ）──
/// this 1 ** no has DDL**， not （`cleaned_raw_rows` 0）：
/// is 1 ** **。`accounts.data` from JSON AES-256-GCM
/// ， this no has need 、 no has need —— is ， by 
/// `core::account_store::store_admin::encrypt_legacy_accounts`（ in 
/// `account_bootstrap::run` ） ； in 
/// （`{config_dir}/account.key`， `AGENT2API_ACCOUNT_KEY`，
/// `core::crypto` module header）。 `apply_version` v14 is empty。
///
/// **Why need 1 **（ not is that ）： as 「 」
/// ， `user_version` is 1 。 to 14 ， its `data` 
/// ； this ** in old that 1 has use ** —— `migrate` new 「
/// 」（ [`migrate`]） in up ： old 
/// （`SCHEMA_VERSION <= 13`） to v14 will failure， not is 、
/// in when 「 not 」「 no has 」DELETE （「 1 =
/// 」）。 for ，v14 is 1 empty。
///
/// ── 15：requests （`status` and ）─────
/// [`V15_SCHEMA`]。 1 ：、 already 
/// MB up ， path in and 。
///
///   · `idx_requests_status`：`SELECT COUNT(*) FROM requests WHERE status = 0`
/// in ** no has available** when for `SCAN`（ 3 50 ms ，
/// in has ）。 this not —— is before 「 in 」
/// （ 5 seconds 1 ， and current in none， `ui/app.js`
/// `syncRunningRequests`） and request 1 。 after 
/// by status ， not 。
/// · `idx_requests_summary`：`/api/stats/summary` 24 when 
/// （`request_stats::sql::select_summary_rows`） need need 6 ，
/// must —— KB `route` / `attempt_details`，
/// is after 。 this this **
/// **（ and that SELECT 1 ， 1 need for 1 ）。
///
/// is `CREATE INDEX IF NOT EXISTS`（ and its etc 1 ）： in 
/// up need 1 ， and 、 1 ；`requests` has when 
/// in ms， not will 。** not need this [`V1_SCHEMA`] **：
/// new by 1→16 ，v15 that 1 will ， v1 1 etc 1 
/// has —— will 1 1 ， is `V4_SCHEMA` that 。
///
/// ── v16：AI for will （[`V16_SCHEMA`]）──────────────────
/// from 「 1 for 」「 has will 」： 1 will 、 1 ，
/// can 。** JSON**（ not is 1 ） 
/// by and `kv` 1 to ： is 「 1 will 」， will 
/// N + 1 ， no has 1 need need 「 by 」。
///
/// ── 22：directory new failure（[`reset_model_refresh_cooldown`]）
/// this is 1 ****（none DDL）： old 「 1 failure」failure，
/// is 1 401 can by to 24 when up ， up 「 in 」。
/// already `round_succeeded`， `failures` / `retryAt` in this 
/// 1 ，otherwise after need etc old to —— etc no 。
///
/// ── 23：requests 「」 1 （[`V23_SCHEMA`]）─────────
/// has （ already WorkBuddy `usage.credit`） will by request up ，
/// requestadd「」。 and token not ： this 1 ** can empty REAL** ——
/// `NULL` = this 1 no up （ old / not this ），0 = ； use 0 will 
/// 「 no to 」「 0 」， 1 。
pub const SCHEMA_VERSION: i64 = 24;

/// 1 and ： before all JSON / JSONL file for 。
///
/// Why 1 batch not is 1 ：「 1
/// schema」 this 1 ， will 「v1 to 」 need need 
/// can 。 use （ down by ）， use 
/// `execute_batch` 1 。
const V1_SCHEMA: &str = "
-- ── kv：通用键值表 ────────────────────────────────────────────
-- 存什么：网关配置的每个顶层键、桌面设置整份、各存储的元数据、零散状态。
-- 为什么要有它：这些数据的共同点是「没有固定结构、读写都是整份进出」——
-- 为它们各建一张表，表数量会随配置项增长；塞进一个 TEXT 列则失去按 key 精确
-- 读写的能力（那是它们唯一需要的查询能力）。一列值 + 主键键名正好。
-- 为什么值是 TEXT 而不是 BLOB / JSONB：SQLite 没有原生 JSON 类型，JSON 就是
-- 文本；存 TEXT 时 `json_extract` 等 JSON 函数可以直接用来做条件查询，
-- 而 BLOB 会强制每次比较都做编码转换。值里存什么形态由写入方决定（见模块头
-- 的键命名规范），本表不做校验。
CREATE TABLE IF NOT EXISTS kv (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- ── accounts：账号表（原 accounts.json）────────────────────────
-- 存什么：一条账号记录一行。`data` 列存**整条账号记录的 JSON 原文**。
-- 为什么保留 JSON 原文而不是把每个字段都拆成列：「用户升级绝不能丢账号里
-- 任何字段」是本项目的硬约束（含未知字段、用户手工加的备注、旧版本遗留的
-- currentAccountId），而账号里有 `rateLimits` 这类嵌套结构、各家 provider
-- 的字段集还在演进。拆列意味着每加一个字段都要一次 schema 迁移 + 一次数据
-- 重写，且**漏掉一个字段就等于升级时把那列的数据静默丢掉** —— 这是
-- `core::account_store` 模块头第 1 条不变量，JSON 原文是唯一能自然满足它的形态。
-- 那一列的例外：需要**排序 / 筛选 / 校验唯一性**的字段额外提取成列，
-- 它们的值从 JSON 里复制一份（权威副本仍是 `data`，提取列是查询用的投影）。
--   id       主键，账号唯一标识（与 JSON 里的 id 同值）
--   provider 提供商 id（workbuddy / raccoon / cline-free…），按家分组查
--   priority 优先级数值，越小越先用（全局唯一，见账号存储不变量 2）
--   enabled  是否启用，0/1。SQLite 没有布尔类型，按惯例用 INTEGER
--   added_at 加入时间（毫秒 Unix 时间戳），同优先级时的稳定排序依据
--   data     整条记录的 JSON 原文（**权威副本**）
-- 「已启用且有凭证的第一个账号」这个派生（pick_current）走下面的复合索引：
-- 先按 provider 分片、再按 priority 升序，added_at 保证同优先级顺序稳定。
CREATE TABLE IF NOT EXISTS accounts (
  id       TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  priority INTEGER NOT NULL,
  enabled  INTEGER NOT NULL DEFAULT 1,
  added_at INTEGER NOT NULL DEFAULT 0,
  data     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_accounts_order ON accounts(provider, priority, added_at);

-- ── logs：事件日志（原 logs.jsonl）─────────────────────────────
-- 存什么：日志页展示的运行日志，一行一条。
-- 为什么 id 是 INTEGER PRIMARY KEY 而不另建自增列：SQLite 里
-- 「INTEGER PRIMARY KEY」这一列**就是** rowid 的别名，天然自增、天然唯一，
-- 且是物理存储顺序（按 id 查是 O(log n) 的 B 树查找）。这正是原来内存里
-- 那条「自增 id」的等价物，前端按 id 增量拉取（`since_id`）因此直接可用。
-- 为什么 data 可空：结构化事件的附加字段（429 切换的 from/to 等）只有部分
-- 日志有；空串与「没有 data」语义不同，所以用 NULL 而不是默认 '{}'。
-- 为什么 ts 存 INTEGER 毫秒：原文件里就是 `Date.now()` 的毫秒数，且范围
-- 与负数场景都在 i64 内，不需要 TEXT 的日期格式（排序即数值排序才是重点）。
-- 三个索引（ts / level / category）对应日志页固定的三种过滤维度：
-- 按时间倒序翻页、按「级别及以上」筛选、按分类筛选。刻意**不做**复合索引：
-- 三个维度是独立下拉、组合方式多，复合索引只能覆盖其中一种组合，反而不如
-- 让 SQLite 自己挑（数据量级是几百到几万行，单列索引足够）。
CREATE TABLE IF NOT EXISTS logs (
  id       INTEGER PRIMARY KEY,
  ts       INTEGER NOT NULL,
  level    TEXT NOT NULL,
  category TEXT NOT NULL,
  message  TEXT NOT NULL,
  data     TEXT
);
CREATE INDEX IF NOT EXISTS idx_logs_ts ON logs(ts);
CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
CREATE INDEX IF NOT EXISTS idx_logs_category ON logs(category);

-- ── requests：请求明细（原 requests.jsonl）─────────────────────
-- 存什么：每次对话转发的记账明细（模型、账号、状态、耗时、token 用量）。
-- 为什么是 AUTOINCREMENT 的 row_id 而不是让 id 当主键：
--   row_id 是**自增物理行号**（内部用，保证稳定顺序、可作游标）；
--   id 是请求的**关联键**（与调试报文 debug_traffic.id 同值，靠它把明细与
--   原始报文对上），旧数据里可能是空串，**且不保证唯一**（重试、去重排队
--   的若干路径会产生同 id 的记账）。两个字段职责不同，混用会让「空串 id
--   被当成主键」直接撞唯一约束而丢掉整条记录。
-- 为什么显式写 AUTOINCREMENT：不加它时 rowid 会被复用（删掉最大行后新行
-- 会拿到同一个号），而这里 row_id 会被当作翻页游标，复用会让「翻页漏行 /
-- 重行」。代价是多一张 sqlite_sequence 表，可忽略。
-- 为什么 token 四列各自 DEFAULT 0 而不是可空：原 JSON 里缺失就是 0
-- （前端直接做加法与求和），可空会让每个读点都要处理 NULL；这四个数的
-- 「没有」与「0」在语义上也没有区别。
-- 其余 DEFAULT 同理：都是「旧数据没有这个字段」时的缺省值，含义与文件版
-- 的读取兜底（`Number(x) || 0`）逐字对应 —— 导入旧数据时不必预先补齐每个
-- 字段，缺的列走 DEFAULT 即可。
CREATE TABLE IF NOT EXISTS requests (
  row_id            INTEGER PRIMARY KEY AUTOINCREMENT,
  id                TEXT NOT NULL DEFAULT '',
  ts                INTEGER NOT NULL,
  model             TEXT NOT NULL,
  account_id        TEXT NOT NULL DEFAULT '',
  account_name      TEXT NOT NULL DEFAULT '',
  status            INTEGER NOT NULL DEFAULT 0,
  duration_ms       INTEGER NOT NULL DEFAULT 0,
  first_response_ms INTEGER,
  attempts          INTEGER NOT NULL DEFAULT 1,
  error             TEXT,
  prompt_tokens     INTEGER NOT NULL DEFAULT 0,
  completion_tokens INTEGER NOT NULL DEFAULT 0,
  total_tokens      INTEGER NOT NULL DEFAULT 0,
  cache_read_tokens INTEGER NOT NULL DEFAULT 0,
  provider          TEXT NOT NULL DEFAULT '',
  client_model      TEXT NOT NULL DEFAULT '',
  upstream_model    TEXT NOT NULL DEFAULT ''
);
-- ts：报表按区间聚合（近 24 小时 / 7 天 / 30 天）与明细按时间倒序翻页。
-- id：由请求 id 反查明细（与调试报文对照、按 id 看一次请求的完整记录）。
CREATE INDEX IF NOT EXISTS idx_requests_ts ON requests(ts);
CREATE INDEX IF NOT EXISTS idx_requests_id ON requests(id);

-- ── request_daily：按天聚合（原 request-daily.jsonl）──────────
-- 存什么：一天一行（date 是**本地时区**的 `YYYY-MM-DD`，与文件名版逐字相同；
-- 时区处理在 `request_stats::clock`，本表只存字符串、不做时区推断）。
-- 为什么 date 直接当主键：一天一行是这张表的全部语义，主键即唯一约束，
-- 天然让「同一天重复记账」变成 UPSERT 而不是插入两行。
-- 为什么三个 *Stats 列是 JSON 数组文本：它们是「当天按模型 / 提供商 / 账号
-- 的累计」，条目数随用户配置变化（模型目录是远程刷新的）、且**整体读写**
-- （报表要整份拿出来算），拆表会让「取一天的三个维度」变成三次 join。
-- 与文件版的行内嵌数组一一对应，导入时原样搬过来即可。
-- 默认 `'[]'` 而不是 NULL：空数组与「这一列没有数据」在这张表里是同一件事，
-- 统一成 `'[]'` 让读取方少一个分支。
-- requests/successful/tokens 这几列是「当天总量」，保留成独立列而不是也从
-- JSON 里算：热力图与汇总行只读它们，拆出来才走得上主键索引。
CREATE TABLE IF NOT EXISTS request_daily (
  date               TEXT PRIMARY KEY,
  requests           INTEGER NOT NULL DEFAULT 0,
  successful         INTEGER NOT NULL DEFAULT 0,
  tokens             INTEGER NOT NULL DEFAULT 0,
  cache_hit_tokens   INTEGER NOT NULL DEFAULT 0,
  cache_input_tokens INTEGER NOT NULL DEFAULT 0,
  model_tokens       TEXT NOT NULL DEFAULT '[]',
  provider_stats     TEXT NOT NULL DEFAULT '[]',
  account_stats      TEXT NOT NULL DEFAULT '[]'
);

-- ── debug_traffic：调试报文（原 debug-traffic.jsonl）──────────
-- 存什么：调试模式下的**上游侧**原始请求/响应（头部已脱敏，脱敏不可关闭）。
-- 为什么 id 是主键：它与 requests.id 同值、是跨表关联键；一条请求只有一份
-- 报文，所以天然唯一（与 requests 不同 —— 那边同 id 可以有多行记账）。
-- 为什么 request/response 的 body 列可空、而 header 列 NOT NULL 带默认值：
-- 请求侧一定存在（我们至少发出去过），响应侧可能因为上游断连而没有；
-- 而头部哪怕没有也应该是 `'{}'`（前端按对象读）。
-- 为什么体用 TEXT 而不用 BLOB：里面是 SSE 文本 / JSON 文本，存 TEXT 才能在
-- `sqlite3` 命令行里直接看（排障场景这是主要用法）。
-- truncated 保留历史报文的截断标记；新采集的正文不截断。
-- size 是完整报文落库后的文本字节合计，不含 SQLite 页与行开销。
-- 索引只有 ts：调试报文按时间倒序翻页是唯一的批量读法，其余都按主键 id 取。
CREATE TABLE IF NOT EXISTS debug_traffic (
  id               TEXT PRIMARY KEY,
  ts               INTEGER NOT NULL,
  url              TEXT NOT NULL DEFAULT '',
  provider         TEXT NOT NULL DEFAULT '',
  request_headers  TEXT NOT NULL DEFAULT '{}',
  request_body     TEXT NOT NULL DEFAULT '',
  status           INTEGER,
  response_headers TEXT,
  response_body    TEXT,
  truncated        INTEGER NOT NULL DEFAULT 0,
  size             INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_debug_traffic_ts ON debug_traffic(ts);
";

/// 2： to `requests` JSON （ + in ）。
///
/// ── this ──────────────────────────────────────────
/// request 「」 can 1 「」（）， as 
/// 「 1 、Why」 and 「 this in 」 in no has ——
/// before in in ， after statistics。
/// up after ， that has can （ `ui/requests-panel.js` ）。
///
/// ── Why is 「 can empty JSON 」 not is ─────────────────────
/// is ** 1 **， is 、 no has by 
/// need （ need in `sqlite3` for JSON use `json_each`）。 and `request_daily`
/// `*Stats` 1 —— by in `request_stats::record::RequestEntry
/// ::attempt_details` 。
///
/// ── defaultWhy is `'[]'` not is NULL ─────────────────────────
/// and `request_daily` that 1 ：empty and 「 this 1 no has 」 is 1 ，
/// 1 `'[]'` 1 。（`sql::encode_json_list`） 
/// failure `'[]'`， 1 。
///
/// ── Why use ALTER not ────────────────────────────────
/// its and 1 not ， need 「 new → → old → 」，
/// `requests` is （ use can ）。
/// ALTER ADD COLUMN is SQLite （ when 、 not ）， can ignore。
///
/// ── Why this not can IF NOT EXISTS ───────────────────────────
/// SQLite `ALTER TABLE ... ADD COLUMN` **unsupported** `IF NOT EXISTS`（ this is and 
/// fileall CREATE ）。 etc by **** ——
/// 1 1 ， `user_version` in 1 （
/// `migrate` ）。 this when not need 「 1 IF NOT EXISTS」：
/// will ， not is empty。
///
/// ： new will v1（** not ** this requests ），
/// is path（ new / ） to 1 。 not need as 「 1 ALTER」
/// this v1 DDL —— that after and v1 DDL for not up 。
const V2_SCHEMA: &str = "
-- ── requests 补两列（schema v2）───────────────────────────────
-- attempt_details：每次上游尝试的明细 `[{provider,status,error}]`，
--   按发生顺序。条数 <= attempts（采集侧一一对应），另受
--   `core::upstream::usage::MAX_ATTEMPT_DETAILS`（24）的体积闸约束。
-- sensitive_hits：本次请求命中的脱敏规则 `[{word,count}]`，按次数降序。
--   来源是脱敏模块算出的命中明细（改造前是「命中的敏感词」，现在是
--   「命中的硬编码规则标签」，见 `core::sanitize`；列名与结构未动，
--   前端请求日志的「敏」标签因此照常工作）。
-- 两列都 NOT NULL DEFAULT '[]'：见上面「默认值为什么是 '[]'」。
-- 「DEFAULT 后面的值必须是常量」——SQLite 对带非常量默认值的 ADD COLUMN 会
-- 直接报错，`'[]'` 是字面量，满足这条。
ALTER TABLE requests ADD COLUMN attempt_details TEXT NOT NULL DEFAULT '[]';
ALTER TABLE requests ADD COLUMN sensitive_hits TEXT NOT NULL DEFAULT '[]';
";

/// 3：`request_raw` —— request/response ** down **（ for can ）。
///
/// ── and debug_traffic is ──────────────────────────
/// `debug_traffic` after up ； down ，
/// request 「 for 」 use 。 by 1 request id ，。
///
/// ── ： and 1 （v13 ）───────────────────
/// ** in when **（ for OmniProxy ： not not 
/// ）—— when not ，path。v13 before 
/// is none ， that already by v13 1 empty（ [`SCHEMA_VERSION`]）。
///
/// ── Why not is to requests ────────────────────
/// is ， not can 。
/// after in use when 。
///
/// ── Why id ────────────────────────────────────
/// 1 request has 1 「 down 」 ， 1 for 1 （ and requests id
/// not —— that id can ， not ）。 1 ，
/// 「 id 」 is UPSERT ， not need need 。
///
/// ── Why use TEXT not use BLOB ─────────────────────────────
/// and debug_traffic 1 to ： is JSON / SSE ， TEXT can in 
/// `sqlite3` （ need use ）。
///
/// ── Why no has （ ts not ）────────────────────────────
/// has ： by id （）、 when by 
/// 「`id IN (SELECT id FROM requests …)`」（）。 1 is 
/// use empty（`DELETE FROM request_raw`）， in 
/// —— by that ， not will to need need as 。
const V3_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS request_raw (
  id            TEXT PRIMARY KEY,
  ts            INTEGER NOT NULL,
  request_body  TEXT NOT NULL DEFAULT '',
  response_body TEXT NOT NULL DEFAULT '',
  size          INTEGER NOT NULL DEFAULT 0
);
";

/// 4：v3 etc （， [`SCHEMA_VERSION`]）。
///
/// use [`V3_SCHEMA`] not is 1 ： if 1 ， will request_raw
/// when 1 ，「 has 1 」 is v2 that 。
/// `CREATE TABLE IF NOT EXISTS` for （ already in ） is none。
const V4_SCHEMA: &str = V3_SCHEMA;

/// 5：`requests` （schema v5）—— etc 。
///
/// ── ──────────────────────────────────────────────────
/// - `client_reasoning`：** down request**client etc 
/// （ and 1 then `core::model_rules::reasoning::read_client_level`）。
/// empty = client no （ or this no has old ）。
/// - `upstream_reasoning`：** up request** etc （ 
/// or client and etc ； in 
/// `core::upstream::payload::send_body`， and `upstream_model` when ）。
/// empty = no has etc （client no and no 、 not etc 、
/// 「」、 or 1 no ）。
///
/// NOT NULL DEFAULT ''： and `client_model` / `upstream_model` 1 
/// 「 in 、empty is no has 」 ， before not 「」。
/// ALTER by （Why not v1 DDL、Why not can 
/// IF NOT EXISTS、 etc ） and [`V2_SCHEMA`] ， not 。
const V5_SCHEMA: &str = "
-- ── requests 补两列（schema v5）──────────────────────────────
ALTER TABLE requests ADD COLUMN client_reasoning TEXT NOT NULL DEFAULT '';
ALTER TABLE requests ADD COLUMN upstream_reasoning TEXT NOT NULL DEFAULT '';
";

/// 6：`requests` （schema v6）—— in request **** and ** when **。
///
/// ── ──────────────────────────────────────────────────
/// - `phase`： this requestcurrent ， is `core::upstream::usage::
/// LogPhase` （`connecting` in / `waiting` etc response /
/// `streaming` response in / `retrying` in / `queued` in ）；empty = not in （、
/// old 、 and before failure from in ）。
/// - `phase_started_at`：**current** when （ms when ， and `ts`
/// 1 ）。 when （request） by 。
///
/// ── Why「 in 」───────────────────────────────
/// is in （「response in 153seconds」 this ），
/// 1 not —— （`update_running_request` /
/// `finish_stale_running` / `finish_running_request`） 1 
/// empty / NULL： not ， 1 already failure will 「 up to 」，
/// that not is 、 no has 。
///
/// ── Why not OmniProxy that 、 when ────────────
/// that when is `phase_started_at`（SQL `julianday('now')` ），
/// this 1 ：`phase_started_at` 、`phaseElapsedMs` in API when 
/// （ `request_stats::report::entry_json`）。 before to is 1 
/// and server when ， not when 。
///
/// NOT NULL / can empty ：`phase` `client_model` that 1 「 in 、
/// empty is no has 」 （ before 1 「」）；`phase_started_at` and 
/// `first_response_ms` 1 is ** can can no has **（ not in ）， can empty，
/// use NULL not is 0 「 no has this when 」。
/// ALTER by （Why not v1 DDL、Why not can 
/// IF NOT EXISTS、 etc ） and [`V2_SCHEMA`] ， not 。
const V6_SCHEMA: &str = "
-- ── requests 补两列（schema v6）──────────────────────────────
ALTER TABLE requests ADD COLUMN phase TEXT NOT NULL DEFAULT '';
ALTER TABLE requests ADD COLUMN phase_started_at INTEGER;
";

/// 7：`requests` 1 （schema v7）—— this request is not is **** 。
///
/// ── ──────────────────────────────────────────────────
/// `is_test = 1`： this 「」（`api::model_test`）；
/// `0`（default， old ）：。
///
/// ── Why need has （ not is 「 / 」）────────────────
/// request and request** is 1 **（ this is ： and 
/// ）， in 1 1 —— 、、、usage。
/// 1 can is 「 this is 」， that has 。 by 
/// （`request_stats` `fold_into_daily` and `rebuild_day`），
/// request then and 。
///
/// ALTER by （Why not v1 DDL、Why not can 
/// IF NOT EXISTS、 etc ） and [`V2_SCHEMA`] ， not 。
const V7_SCHEMA: &str = "
-- ── requests 补一列（schema v7）──────────────────────────────
ALTER TABLE requests ADD COLUMN is_test INTEGER NOT NULL DEFAULT 0;
";

/// 8：account_usage_records —— 1 （ 1 1 ，
/// id， is UPSERT）。
///
/// ：`usage` is return ** 1 JSON **（ and `/api/accounts/usage`
/// `usage` 1 ， before by ），`error` / `code` is failure 
/// Reason and （ and ）。`remaining` is from usage ****
/// （ 1 = `available`，workbuddy has = `totalLeft`）： 
/// 「 not 」request need 1 ， from JSON ， 1 can ；
/// `unlimited` 1 ，workbuddy ∞ not and 。
///
/// ── Why `remaining` in success when ────────────────────────
/// failure up success ： not 「 this failure」（
/// 「failure → → 402」 ）， need need success and to up 。
/// failure in `error` / `code` ， not 。
///
/// Why when is INTEGER ms not is DATETIME：（ to = up +
/// ） need is can ms， and `logging::now_ms` 1 ， use 。
const V8_SCHEMA: &str = "
-- ── account_usage_records：每账号的余额查询记录（schema v8）──
CREATE TABLE IF NOT EXISTS account_usage_records (
  account_id      TEXT PRIMARY KEY,
  usage           TEXT,
  error           TEXT,
  code            TEXT,
  remaining       REAL,
  unlimited       INTEGER NOT NULL DEFAULT 0 CHECK (unlimited IN (0, 1)),
  last_success_at INTEGER NOT NULL DEFAULT 0,
  last_attempt_at INTEGER NOT NULL DEFAULT 0,
  updated_at      INTEGER NOT NULL DEFAULT 0
);
";

/// 9： to `requests` 「 by + when 」 。
///
/// ── ──────────────────────────────────────────────
/// limit **Token then **： then is 「 this in current
/// （ for when ，30 minutes ~ 24 when ） 1 Token」——
/// 1 `SELECT account_id, SUM(total_tokens) FROM requests
/// WHERE account_id = ? AND ts >= ?` 。 no has this can 
/// `idx_requests_ts` to up ，limit new （ 10 seconds 1 ）
/// will request 1 。
///
/// Why is `(account_id, ts)` not is `account_id`：
/// （`ts >= `） need in ** 1 **，
/// 1 has ； account_id to after ts。
///
/// use ：`core::limiter` new （ by distinct 
/// ） and when （ use ）。
const V9_SCHEMA: &str = "
-- ── requests 补按账号聚合的索引（schema v9）──────────────────
CREATE INDEX IF NOT EXISTS idx_requests_account_ts ON requests(account_id, ts);
";

/// 10：requests 「」 1 。
///
/// JSON （`{strategy, sticky, stickyHit, picked, ordered[],
/// blocked[]}`， already to has ）。DEFAULT NULL： old and 「 no to 
/// 」 request is 「 no has 」—— for not 1 ， and 
/// `attempt_details` `'[]'` not （ that 「empty」 is has ， this NULL
/// is 「 no this 」， not 1 empty）。
const V10_SCHEMA: &str = "
-- ── requests 补选路决策快照列（schema v10）───────────────────
ALTER TABLE requests ADD COLUMN route TEXT;
";

/// 11：checkin_records —— to ****。
///
/// ： 1 to （ when / / / /
/// ）， 1 。 ：
/// - `ts`： when （ms，`logging::now_ms` ； and ）；
/// - `date`： when `YYYY-MM-DD`（ up ，`core::beijing` ——
/// 「 to 」 by ， when none， and 1 ）；
/// - `source`： `schedule / makeup / startup / manual / single /
/// activity`（ `core::checkin_records` module header）， as 「」；
/// - `reason`： to that （「 to （08:00）」「 to 」…），；
/// - `account_id` / `account_name` / `provider`：（ has ，
/// ；delete after can ）；
/// - `outcome`：`success / already / pending / failed`（ use to 
/// 1 for ， 1 ， `core::checkin_records::classify`）；
/// - `ok`：`success|already → 1` （「 already 」 SQL ，
/// `(outcome='success' OR outcome='already')`）；
/// - `message`：claim.msg （「 +100」 etc ** in this **）；
/// - `error`：error（failure Reason）；
/// - `kept_active`：success（ for `pokeSucceeded` or 
/// will `webConversation.succeeded` 1 ， `checkin_records::kept_active_of`）。
///
/// Why row_id use AUTOINCREMENT： and `requests` 1 by —— ，
/// use rowid will 「 / 」。
///
/// （ts / outcome / provider）， for ，
/// not （ and logs 1 ：、， SQLite ）。
const V11_SCHEMA: &str = "
-- ── checkin_records：签到记录的每账号明细台账（schema v11）────
CREATE TABLE IF NOT EXISTS checkin_records (
  row_id       INTEGER PRIMARY KEY AUTOINCREMENT,
  ts           INTEGER NOT NULL,
  date         TEXT NOT NULL DEFAULT '',
  source       TEXT NOT NULL DEFAULT '',
  reason       TEXT NOT NULL DEFAULT '',
  account_id   TEXT NOT NULL DEFAULT '',
  account_name TEXT NOT NULL DEFAULT '',
  provider     TEXT NOT NULL DEFAULT '',
  outcome      TEXT NOT NULL DEFAULT '',
  ok           INTEGER NOT NULL DEFAULT 0,
  message      TEXT,
  error        TEXT,
  kept_active  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_checkin_records_ts ON checkin_records(ts);
CREATE INDEX IF NOT EXISTS idx_checkin_records_outcome ON checkin_records(outcome);
CREATE INDEX IF NOT EXISTS idx_checkin_records_provider ON checkin_records(provider);
";

/// 12：requests 「client」 1 （schema v12）。
///
/// and v5 / v7 / v10 1 （ has ， ALTER， etc ）：
/// `client` from down `User-Agent` client（`core::client`），
/// request 「client」。DEFAULT ''： old and 「request no UA」 path is 
/// empty —— in （）， not 。
const V12_SCHEMA: &str = "
-- ── requests 补来源客户端列（schema v12）─────────────────────
ALTER TABLE requests ADD COLUMN client TEXT NOT NULL DEFAULT '';
";
// v13 no has for `*_SCHEMA` ： has 1 DELETE， and in `apply_version`
// use `execute` （ need ）， in that —— 
// `SCHEMA_VERSION` v13 。

/// 15：requests （schema v15）。
///
/// and v9 1 （ has ，`CREATE INDEX IF NOT EXISTS`， etc ）。
/// and [`SCHEMA_VERSION`] v15 。
///
/// `idx_requests_summary` = `sql::SUMMARY_COLUMNS` ** before 
/// `ts, is_test`**：`ts` in before can `WHERE ts >= ? AND ts <= ?` ，
/// `is_test` its after is as in （ not need ）。
/// its 5 （`model` / `upstream_model` / `total_tokens` / `cache_read_tokens`
/// / `prompt_tokens`） is need —— 1 need ， 1 
/// empty and 。
const V15_SCHEMA: &str = "
-- ── requests 补 status 索引（schema v15）─────────────────────
CREATE INDEX IF NOT EXISTS idx_requests_status ON requests(status);
-- ── requests 补报表窄列覆盖索引（schema v15）─────────────────
CREATE INDEX IF NOT EXISTS idx_requests_summary ON requests(
  ts, is_test, model, upstream_model, total_tokens, cache_read_tokens, prompt_tokens
);
";

/// 16：AI for will （schema v16）。
///
/// ── ──────────────────────────────────────────────────
/// 1 will 1 。 ：
/// - `id`： will id（ before ，`crypto.randomUUID()` ）。 before 
/// not is ： new will when before need use this id ， etc 1 to 
/// will 「 new → 1 」 1 must etc in ；
/// - `title`： 。 1 use before if （ before up ），
/// use no 1 is —— after not 「」 that 1 up use ；
/// - `messages`：** JSON **（`[{role, content, toolCalls?,
/// toolCallId?}, …]`， and to up wire ）。 not is by 
/// [`SCHEMA_VERSION`] v16 ；
/// - `model` / `reasoning`： this will after use and etc 。** will ** not is 
/// 1 ： use is 「 this for from in 」， not is 
/// ； 1 will when before from this ；
/// - `created_at` / `updated_at`：ms when 。 by `updated_at` 
/// （ in up ），`updated_at` new 。
///
/// ── ────────────────────────────────────────────────────
/// has 1 `updated_at` ： is 「 by update」，
/// no has by / need （ will is ， before 1 can ）。
const V16_SCHEMA: &str = "
-- ── assistant_sessions：AI 助手的对话会话（schema v16）───────
CREATE TABLE IF NOT EXISTS assistant_sessions (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL DEFAULT '',
  messages    TEXT NOT NULL DEFAULT '[]',
  model       TEXT,
  reasoning   TEXT,
  created_at  INTEGER NOT NULL DEFAULT 0,
  updated_at  INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_assistant_sessions_updated ON assistant_sessions(updated_at DESC);
";

const V17_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS account_limiter_events (
  row_id INTEGER PRIMARY KEY AUTOINCREMENT,
  id TEXT NOT NULL UNIQUE,
  account_id TEXT NOT NULL,
  model TEXT NOT NULL DEFAULT '',
  channel TEXT NOT NULL DEFAULT '',
  ts INTEGER NOT NULL,
  tokens INTEGER NOT NULL DEFAULT 0,
  requests INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_limiter_events_account_time ON account_limiter_events(account_id, ts, channel);
INSERT OR IGNORE INTO account_limiter_events (id, account_id, model, ts, tokens, requests)
SELECT 'legacy:' || row_id, account_id,
       CASE WHEN upstream_model != '' THEN upstream_model ELSE model END,
       ts, MAX(total_tokens, 0), 1
FROM requests WHERE account_id != '' AND status != 0 AND attempts > 0;
";

/// 23：requests 「」 1 （schema v23）。
///
/// and v5 / v7 / v10 / v12 1 （ has ， ALTER， etc ）。
/// Why is **REAL and can empty**， not is token that `INTEGER NOT NULL DEFAULT 0`：
/// · up `credit` is （WorkBuddy 0.03～0.31）， will ；
/// · `NULL` 「 this 1 no up / old no has 」，0 「 up 0」。 for before 
/// 「—」、 for after 「0」—— and 0 will 「 no to 」
/// 「 this request」， is /usage 1 that 。
const V23_SCHEMA: &str = "
-- ── requests 补积分消耗列（schema v23）──────────────────────
ALTER TABLE requests ADD COLUMN credit REAL;
";

/// to [`SCHEMA_VERSION`]（ etc ： already is new when not ）。
///
/// ** when return `Err` **（v14 has 1 has ，
/// that ）： after `accounts.data` old 
/// 、 「 no has 」， etc 「 1 =
/// 」。**** then ；** etc ** then empty。
///
/// return `rusqlite::Result` not is error： use `Db::open`
/// is 1 ， error 1 can （`format!`）； this need need 
/// 「failure / DDL failure / failure」， rusqlite 
/// error（ SQLite error and ） is 1 。 that 「 new 」
/// by use `ToSqlConversionFailure` （ `Display` is ）。
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current == SCHEMA_VERSION {
        // already is new ： not （ new after 、 already this ）。
        return Ok(());
    }
    if current > SCHEMA_VERSION {
        // ── new ：****（ has 1 has ）─────────
        //
        // before this is 「 not 、 not 」： by is 「 use from new to 
        // old ， will old not ， is 1 」。 that by in 
        // `accounts.data` is —— old not new new / new ，
        // that ， is 1 new can 。
        //
        // ** not **：`accounts.data` is `enc1:` （v14 ）， old 
        // （`SCHEMA_VERSION <= 13`） `decode_data` JSON —— is ：
        // 1. old 1 will failure， **** → up 
        // ；
        // 2. is path： old `save_state` by id ，「 has 、
        // no has 」 will `DELETE FROM accounts WHERE id = ?` ——
        // not that **** this 。 is 「 1 and 
        // any 1 」= after ****。
        // etc 「 1 = 」， is not can use 
        // —— this failure： in Reason， use use new 。
        //
        // this not is 「 schema」（ that need need 、 not can ， from no ， not 
        // will ）： this is ** in before **，file 1 not 。
        //
        // use this error：`ToSqlConversionFailure` `Display` is that 
        // （ has ， `db::migrate::logs::assign_log_id` ），
        // is `Db::open` `format!("/ schema failure: {error}")` will this 
        // to 。
        return Err(rusqlite::Error::ToSqlConversionFailure(
            format!(
                "库 schema 版本（{current}）高于当前程序（{SCHEMA_VERSION}）：这通常是回退到了旧版本。\
                 请使用更新的程序，或从备份恢复——用旧程序打开会把加密后的账号静默删除。"
            )
            .into(),
        ));
    }
    // v13 （0 = this 1 no ）： after is 
    // VACUUM（ that ）。 is as VACUUM not can in 。
    let mut cleaned_raw_rows = 0usize;
    for version in (current + 1)..=SCHEMA_VERSION {
        // 1 ：DDL and must 1 or 1 not ，
        // otherwise in after will 「 already 、 is old 」（ down DDL，
        // IF NOT EXISTS can ） and 「 already new 、 no 」（** not **）
        // ， after ， can 「 1 」 。
        //
        // use `unchecked_transaction()` not is `transaction()`： after need 
        // `&mut Connection`， is `&Connection`（ in `Db::open`
        // use ， that when `Db` no ， not to `with_mut`）。「 1 up 
        // not can has 」 this by use —— `Db::open` has 1 that ，
        // not will has 。
        //
        // Note `tx` not need need `mut`：`pragma_update` / `commit` `&self`，
        // by SQLite （ 1 up not and ），
        // not is Rust use 。
        let tx = conn.unchecked_transaction()?;
        apply_version(&tx, version, &mut cleaned_raw_rows)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    if cleaned_raw_rows > 0 {
        // ── after VACUUM 1 （v13 「empty」 in this ）──
        // `DELETE` is empty，file** not will ** —— for 「 after up 
        // to down 」 use ， has VACUUM（file） empty to 
        // file。 is `cleaned_raw_rows > 0`： new （empty） and 
        // already 0 、 not this ； has 「 has 」 
        // that will 1 （ MB seconds， 1 ）。
        //
        // VACUUM not can in ， in （ already ）；
        // failure** not **： already ，empty is as —— use 
        // after can in request 「」（ that has after and 
        // ， `RequestStats::compact`）。
        //
        // VACUUM before after 1 checkpoint（ and `RequestStats::compact` ， by 
        // that ）：WAL down VACUUM will `-wal`，
        // that ，empty is from to WAL、total use not （issue #297）。
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        let _ = conn.execute_batch("VACUUM;");
        let _ = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
    }
    Ok(())
}

/// DDL / ， and 「 」 to `migrate`
/// （ has v13 will ， its 0）。**add when in this 1 **，
/// and in up this /。
///
/// `cleaned_raw_rows` not is return： is 1 DDL、
/// no has need up ， if `Result<usize>` then 1 `Ok(0)`
/// （），「 has that 1 」。
///
/// `_` return Ok not is unreachable!： is （file can can 
/// update ， old to ）， [`SCHEMA_VERSION`] not this panic。
/// `migrate` not can can to this （ up is SCHEMA_VERSION），
/// this is 「 use version」 。
fn apply_version(
    conn: &Connection,
    version: i64,
    cleaned_raw_rows: &mut usize,
) -> rusqlite::Result<()> {
    match version {
        1 => conn.execute_batch(V1_SCHEMA),
        // v2：requests JSON （ / in ）。
        // and v1 has 1 ： this is ALTER not is CREATE，** no has **
        // IF NOT EXISTS available（ V2_SCHEMA ）， etc by 。
        2 => conn.execute_batch(V2_SCHEMA),
        // v3：request_raw （ new use CREATE IF NOT EXISTS， V3_SCHEMA）
        3 => conn.execute_batch(V3_SCHEMA),
        // v4： v3 （「 3、 not in 」 ， SCHEMA_VERSION）
        4 => conn.execute_batch(V4_SCHEMA),
        // v5：requests etc （ down / up ， V5_SCHEMA）
        5 => conn.execute_batch(V5_SCHEMA),
        // v6：requests and when （ in request ， V6_SCHEMA）
        6 => conn.execute_batch(V6_SCHEMA),
        // v7：requests 「」 1 （ ， V7_SCHEMA）
        7 => conn.execute_batch(V7_SCHEMA),
        // v8：account_usage_records （ old kv ， V8_SCHEMA）
        8 => conn.execute_batch(V8_SCHEMA),
        // v9：requests by （limit Token ， V9_SCHEMA）
        9 => conn.execute_batch(V9_SCHEMA),
        // v10：requests 「」 1 （request by ， V10_SCHEMA）
        10 => conn.execute_batch(V10_SCHEMA),
        // v11：checkin_records to （ to ， V11_SCHEMA）
        11 => conn.execute_batch(V11_SCHEMA),
        // v12：requests 「client」 1 （request client， V12_SCHEMA）
        12 => conn.execute_batch(V12_SCHEMA),
        // v13：empty for （ 1 empty， SCHEMA_VERSION 
        // v13 ）。 v3 ： has 「 not 、request_raw 
        // 」 （v4 is as ）—— for that DELETE will 
        // 「no such table」 and ，。
        // use `execute` not is `execute_batch`： need ，`migrate`
        // is VACUUM（ DELETE not will file）。
        13 => {
            conn.execute_batch(V3_SCHEMA)?;
            *cleaned_raw_rows = conn.execute("DELETE FROM request_raw", [])?;
            Ok(())
        }
        // v14： `data` as （`enc1:` ）。
        // ** this 1 no has DDL**， is empty： is （ by 
        // `account_store::store_admin::encrypt_legacy_accounts` 
        // ）， no has need 、 no has need 、 no has need （`cleaned_raw_rows`
        // 0， not VACUUM）。 use in ** old that 1 **： old 
        // to v14 when failure（ `migrate` ）， not is 
        // after 。 [`SCHEMA_VERSION`] v14 。
        14 => Ok(()),
        // v15：requests （status + ，
        // V15_SCHEMA and SCHEMA_VERSION v15 ）。
        15 => conn.execute_batch(V15_SCHEMA),
        // v16：assistant_sessions for will （ V16_SCHEMA and 
        // SCHEMA_VERSION v16 ）。 new use CREATE IF NOT EXISTS， etc 。
        16 => conn.execute_batch(V16_SCHEMA),
        17 => conn.execute_batch(V17_SCHEMA),
        18 => conn.execute_batch(include_str!("schema_source_stats.sql")),
        19 => conn.execute_batch(include_str!("schema_assistant_images.sql")),
        20 => conn.execute_batch(include_str!("schema_debug_raw.sql")),
        21 => conn.execute_batch(include_str!("schema_assistant_title.sql")),
        // v22：「directory new 」 in old down failure（ V22 ）
        22 => reset_model_refresh_cooldown(conn),
        // v23：requests 「」 1 （request ， V23_SCHEMA）
        23 => conn.execute_batch(V23_SCHEMA),
        24 => conn.execute_batch(include_str!("schema_client_identification.sql")),
        _ => Ok(()),
    }
}

/// 22： 「directory new 」 that **failure**（，none DDL）。
///
/// ── is ──────────────────────────────────────────────
/// old is `failed == 0`（`core::scheduled_tasks::run_backend` 
/// `TASK_MODEL_REFRESH` ）：** 1 ** new failure failure。
/// failure is —— 1 up （e.g. CatPaw 401）
/// 1 failure 1 ，`failures` is by `1h × 2^(n-1)` 1 to 24 when limit，
/// its can 1 in old up 。 up as 「 in ，N when after 」，
/// is new can 。
///
/// already and 1 （`round_succeeded`： has success not failure），
/// ** that failure and `retryAt` in **： not in this 1 ， use after 
/// need etc to old to （ can to when ）， etc not 。
///
/// ── Why new this 1 ─────────────────────────────────
/// no has this ， is failure（ / up ），
/// 1 and etc use before 、 1 up 。
///
/// ── Why in this not is 1 ─────────────────────
/// `user_version` is 「 1 」 （ [`migrate`]），
/// use add； kv 1 need `RESERVED_KV_KEYS`，
/// otherwise 1 config will 「 already delete 」、 down 1 。
///
/// failure not ： is `kv.backgroundTaskState` 1 JSON， not to / not 
/// 「 no has need 」， not 。
fn reset_model_refresh_cooldown(conn: &Connection) -> rusqlite::Result<()> {
    use rusqlite::OptionalExtension;
    use serde_json::{json, Map, Value};

    let text: Option<String> = conn
        .query_row("SELECT value FROM kv WHERE key = 'backgroundTaskState'", [], |row| row.get(0))
        .optional()?;
    let Some(text) = text else { return Ok(()) };
    let Ok(mut states) = serde_json::from_str::<Map<String, Value>>(&text) else {
        return Ok(());
    };
    // is id （`scheduled_tasks::state_key`： new not is update
    // check that before ）。 `modelRefresh:<provider>` not ——
    // is failure， and this none。
    let Some(state) = states.get_mut("modelRefresh").and_then(Value::as_object_mut) else {
        return Ok(());
    };
    let had_cooldown = state.get("retryAt").and_then(Value::as_i64).unwrap_or(0) > 0
        || state.get("failures").and_then(Value::as_u64).unwrap_or(0) > 0;
    if !had_cooldown {
        return Ok(());
    }
    state.insert("retryAt".to_string(), json!(0));
    state.insert("failures".to_string(), json!(0));
    let encoded = serde_json::to_string(&states)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
    conn.execute(
        "UPDATE kv SET value = ?1 WHERE key = 'backgroundTaskState'",
        rusqlite::params![encoded],
    )?;
    Ok(())
}
