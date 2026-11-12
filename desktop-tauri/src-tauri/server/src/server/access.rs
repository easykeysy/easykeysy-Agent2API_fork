//! ：、 will 、failure and /v1 fail-closed。
//!
//! ── （ OmniProxy + ）────────────────
//! · access token：****（2 when ），HttpOnly cookie（path=/），
//! —— ，；
//! · refresh token：****（30 ），HttpOnly cookie（path=/api/panel，
//! ）， **sha256 **； to 
//! `/api/panel/refresh` new access + new refresh（ 1 will ，
//! old rotated）。access in after no —— 
//! refresh new ， use none。
//! · ： by will ；** already old refresh as 
//! **（）， will ， need need new 。
//!
//! ── ─────────────────────────────────────────
//! · （）： new when 「create」，
//! `kv` `panelAdmin` —— not need need in config；
//! · （`AGENT2API_ADMIN_USER` + ，，
//! none / IaC use ）： `AGENT2API_ADMIN_PASSWORD`（，
//! bcrypt ） or `AGENT2API_ADMIN_PASSWORD_HASH`
//! （ already is bcrypt ，）。 when ——
//! ** has **， in config。
//!
//! ── when enabled ────────────────────────────────────────
//! 注册过（或环境变量预置了）管理员即启用：`/api/*` need need will or API Key。
//!
//! **default not **，set「」 will 
//! （ `desktop-tauri/src/commands.rs` `panel_register`）—— 1 
//! must has 。 is ** to ** ： after not will ，
//! 1 。
//! will by use ， and none。
//! need ； will not can use Key。

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;

use crate::server::db::Db;
use crate::server::logging;

pub(crate) mod diagnostics;
pub mod local_session;

/// will cookie ：** 1 **（`agent2api-panel=<access>.<refresh>`）。
///
/// ── Why 1 ─────────────────────────────────────────
/// / new **** `Set-Cookie`（access in `Path=/`、refresh in 
/// `Path=/api/panel`）。 in （fnOS docker that 1 ） will response
/// by 1 ， is cookie —— up is 
/// `path|domain|max-age|mode|session|version|expires` this 1 ， new will no 
/// by `/` 。 after is in need ： 1 is 1 
/// for not up 63 。
/// ** 1 Set-Cookie no has can for **， not will ，cookie ；
/// not localStorage（ that `=none` is ）。
///
/// ── （ has ）───────────────────────────────────────
/// refresh Path from `/api/panel` to `/`： this 1 when 。
/// this and `/v1/*`， and access in `/`， 
/// is 「 down can path」； to is 「 in not 」。
/// refresh（30 ）：access by server will （`AccessRecord` 
/// ），cookie is —— after 、serverinvalid、 refresh，
/// and 1 。
pub const ACCESS_COOKIE: &str = "agent2api-panel";
/// 1 cookie 。 is `random_hex`， not ，
/// `split_once('.')` is ， not need need 。
const TOKEN_SEP: char = '.';
/// will cookie（90 ）： `/api/panel/cookie-probe`
/// 「 this cookie can not can 」。 in down / not ，failure when 
/// before need response（ api::panel::cookie_probe and panel::issue_response）。
/// not will ，server for 、 not will 。
///
/// ** 1 **： need is response that `Set-Cookie` ，response in 
/// 1 —— must and for 1 （ is as ；
/// 1 after will 「」）。
pub const PROBE_COOKIE: &str = "agent2api-panel-probe";
/// **** old refresh cookie （ up 1 、`Path=/api/panel`）。
///
/// 、 not ： 1 after / new not ， this is as ** that 1 
/// not all in **。 that 1 access in （empty），
/// access to new kv （`panelAccessSessions`）， after that 
/// access invalid —— 1 can will is `panelTokens` that refresh，
/// in this old cookie （refresh no ， old old valid）。
/// not etc ** 1 **。
///
/// and `split_session_cookie` for access is 1 by ： that not will 
/// will none， this not will refresh 1 。
///
/// in will cookie after ： in when new （ old is up 1 ，
/// in 「 already 」 will 1 ）。** down **。
const LEGACY_REFRESH_COOKIE: &str = "agent2api-panel-rt";

/// access token valid（）
const ACCESS_TTL: Duration = Duration::from_secs(2 * 3600);
/// refresh token valid（）
const REFRESH_TTL: Duration = Duration::from_secs(30 * 24 * 3600);
/// and ： 1 refresh after this when ， 1 up 
/// by 「 1 and 」（ `rotate_session` ），
/// old 、。 1 minutes when up access 
/// （client and 1 up down and ）。
const ROTATION_GRACE_MS: i64 = 60_000;
/// failure： 1 failure 5 ， 5 minutes（ by IP not is ——
/// will 「」 1 ，Tinyauth that CVE ）
const LOCKOUT_THRESHOLD: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(300);

const KV_ADMIN_KEY: &str = "panelAdmin";
const KV_TOKENS_KEY: &str = "panelTokens";
/// access will （ sha256， `AccessRecord`）。⚠️ addmust when 
/// `db::schema::RESERVED_KV_KEYS`，otherwise use 1 config 。
const KV_ACCESS_KEY: &str = "panelAccessSessions";

// ── （ when ；kv ）────────────────────

static STORE_DB: OnceLock<Option<Db>> = OnceLock::new();

/// to （`ServerState::bootstrap` after use 1 ）。
/// `None` = unavailable： and new can 
/// （ can ， will in ）。
pub fn attach_db(db: Option<Db>) {
    STORE_DB.set(db).ok();
}

fn db() -> Option<&'static Db> {
    STORE_DB.get().and_then(|slot| slot.as_ref())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn sha256_hex(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ── （ is 1 ； in ）────────────────
//
// and sub2api / new-api 1 ：`AGENT2API_ADMIN_USER` + in ** in no has 
// ** when as `kv.panelAdmin`， after all —— 
// ， old not will 「」。
//
// old is 「 1 」（`verify_login` env、`sync_env_admin_to_store`
// ）。 after is env in not ： new 
// ， env that for 。 after this 1 。
static ENV_ADMIN: OnceLock<Option<(String, String)>> = OnceLock::new();

/// （onlyused for，** not and **）。
fn env_admin() -> Option<&'static (String, String)> {
    // OnceLock<Option<(String, String)>>： use when ， after return use 。
    // has ：HASH（ already is bcrypt ）；PASSWORD（） then 
    // bcrypt::hash 1 —— and from has 。
    ENV_ADMIN.get_or_init(|| {
        let user = std::env::var("AGENT2API_ADMIN_USER")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let from_hash = std::env::var("AGENT2API_ADMIN_PASSWORD_HASH")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            // `htpasswd -nBC` （"user:$2y$…"）： after is 
            // bcrypt —— 
            // can use ， not before 
            .map(|value| match value.split_once(':') {
                Some((_, hash)) if hash.starts_with("$2") => hash.to_string(),
                _ => value,
            });
        let hash = match from_hash {
            Some(hash) => Some(hash),
            None => std::env::var("AGENT2API_ADMIN_PASSWORD")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .and_then(|plain| match bcrypt::hash(plain, 10) {
                    Ok(hash) => Some(hash),
                    // bcrypt （ 72 ）：，
                    // as already success
                    Err(error) => {
                        eprintln!("❌ AGENT2API_ADMIN_PASSWORD none: {error}");
                        None
                    }
                }),
        };
        match (user, hash) {
            (Some(user), Some(hash)) => Some((user, hash)),
            _ => None,
        }
    })
    .as_ref()
}

/// （`kv` `panelAdmin`：{username, hash}）。
fn store_admin() -> Option<(String, String)> {
    let db = db()?;
    db.with(|conn| {
        conn.query_row(
            "SELECT value FROM kv WHERE key = ?1",
            rusqlite::params![KV_ADMIN_KEY],
            |row| row.get::<_, String>(0),
        )
        .ok()
    })
    .flatten()
    .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
    .and_then(|object| {
        let username = object.get("username")?.as_str()?.to_string();
        let hash = object.get("hash")?.as_str()?.to_string();
        Some((username, hash))
    })
}

/// is enabled（ or ）。
pub fn panel_auth_enabled() -> bool {
    env_admin().is_some() || store_admin().is_some()
}

/// is already in （「」 is 「」）。
pub fn admin_registered() -> bool {
    panel_auth_enabled()
}

/// → bcrypt （ 10， and env 、htpasswd -nBC ）。
///
/// parameter in this 1 ：（`api::panel`） and 
/// （ HTTP path） need 1 ，parameter will 
/// not 。
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, 10).map_err(|error| format!("failure: {error}"))
}

/// （ in none when success —— etc ：
/// down has 1 ， after to will to 「 already 」）。
///
/// return Ok(false) = already has （ or ），。
pub fn setup_admin(username: &str, password_hash: &str) -> Result<bool, String> {
    if panel_auth_enabled() {
        return Ok(false);
    }
    let Some(db) = db() else {
        return Err("unavailable，none".to_string());
    };
    let payload = serde_json::json!({ "username": username, "hash": password_hash });
    // ON CONFLICT DO **NOTHING**（ not is DO UPDATE）： is **** ——
    // request when up panel_auth_enabled() check when ，DO UPDATE will 
    // after （ empty）。DO NOTHING down 
    // has 1 ， after to to changed=0 → Ok(false) → 409。
    // （env `sync_env_admin_to_store`， and none。）
    let changed = db
        .with_mut(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO NOTHING",
                rusqlite::params![KV_ADMIN_KEY, payload.to_string()],
            )
        })
        .and_then(|result| result.ok())
        .ok_or_else(|| "failure".to_string())?;
    Ok(changed > 0)
}

/// current use （return None）。
pub fn admin_username() -> Option<String> {
    store_admin().map(|(username, _)| username)
}

/// ： old → new → **current will all will **。
///
/// its will by and 1 ：「 old can can 」，
/// will ， etc will can use 30 （refresh ）。
/// current this will —— use in up ， down is 
/// （ as failure）。
///
/// return `Ok(true)` = already ；`Ok(false)` = old not for ；`Err` = /failure。
pub fn change_password(
    headers: &HeaderMap,
    old_password: &str,
    new_password_hash: &str,
) -> Result<bool, String> {
    let Some((username, _)) = store_admin() else {
        return Err("".to_string());
    };
    if !verify_login(&username, old_password) {
        return Ok(false);
    }
    let Some(db) = db() else {
        return Err("unavailable，none new ".to_string());
    };
    let payload = serde_json::json!({ "username": username, "hash": new_password_hash });
    db.with_mut(|conn| {
        conn.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![KV_ADMIN_KEY, payload.to_string()],
        )
    })
    .and_then(|result| result.ok())
    .filter(|changed| *changed > 0)
    .ok_or_else(|| " new failure".to_string())?;

    revoke_other_sessions(headers);
    Ok(true)
}

/// **request this ** all will （ use ）。
///
/// 「current will 」 by request access ： in that access session
/// id， its 。 not to （request refresh、 or access ） when 
/// as 「」—— can 1 ， not 1 not will 。
fn revoke_other_sessions(headers: &HeaderMap) {
    let current_session = {
        let table = match access_tokens().lock() {
            Ok(table) => table,
            Err(poisoned) => poisoned.into_inner(),
        };
        access_candidates(headers)
            .iter()
            .find_map(|token| table.get(&sha256_hex(token)).map(|(session, _)| session.clone()))
    };

    {
        let mut table = match refresh_tokens().lock() {
            Ok(table) => table,
            Err(poisoned) => poisoned.into_inner(),
        };
        match &current_session {
            Some(session) => table.retain(|record| record.session == *session),
            None => table.clear(),
        }
        persist_refresh_tokens(&mut table);
    }
    {
        let mut table = match access_tokens().lock() {
            Ok(table) => table,
            Err(poisoned) => poisoned.into_inner(),
        };
        match &current_session {
            Some(session) => table.retain(|_, (session_of, _)| session_of == session),
            None => table.clear(),
        }
        persist_access_tokens(&mut table);
    }
}

/// ****（`kv` `panelAdmin`）。
///
/// in ** no has ** when （`ON CONFLICT DO NOTHING`）：
/// · → use env ， after can use ；
/// · after → already has ，env ignore —— use in 
/// not will old ；
/// · → in 「set → 」， or `panelAdmin` new 。
///
/// env_admin() already bcrypt ， is —— 
/// not will 。 no env then not ，path。
/// when （`attach_db` after ） use 1 。
pub fn sync_env_admin_to_store() {
    let Some((user, hash)) = env_admin() else {
        return;
    };
    let Some(db) = db() else {
        return;
    };
    let payload = serde_json::json!({ "username": user, "hash": hash });
    let changed = db
        .with_mut(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO NOTHING",
                rusqlite::params![KV_ADMIN_KEY, payload.to_string()],
            )
        })
        .and_then(|result| result.ok())
        .unwrap_or(0);
    if changed > 0 {
        logging::log(
            "[Security]",
            "✅ already use （ after as ， use set）",
        );
    }
}

/// 。bcrypt `htpasswd -nBC` 。
fn verify_credentials(username: &str, password: &str, expected: &(String, String)) -> bool {    let user_ok = constant_time_eq(username.as_bytes(), expected.0.as_bytes());
    let pass_ok = bcrypt::verify(password, &expected.1).unwrap_or(false);
    // return，「 use for not for 」 when 
    user_ok && pass_ok
}

/// by （**** —— only in when and ，
/// by `sync_env_admin_to_store`）。
pub fn verify_login(username: &str, password: &str) -> bool {
    store_admin().is_some_and(|expected| verify_credentials(username, password, &expected))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

// ── access token（， sha256）──────────────────────

/// access token 1 。** and sha256(token) as ** ——
/// is 、 in ， is 「」 etc 「 all will invalid」：
/// that cookie in 、 in when ，server not 
/// （ 1 ：` will invalid` 51 is 1 already for not up ，
/// when and 12:26 that after ）。
///
/// in `panelTokens` that already ：， when 。
/// not —— that （refresh ）， not to 
/// not to old 、 not to new this 。
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AccessRecord {
    /// sha256(token)
    hash: String,
    /// will id： 1 access ， by 
    session: String,
    /// when （ms， —— `Instant` not ）
    expires_at: i64,
}

type AccessTable = HashMap<String, (String, i64)>;

static ACCESS_TOKENS: OnceLock<Mutex<AccessTable>> = OnceLock::new();

fn access_tokens() -> &'static Mutex<AccessTable> {
    ACCESS_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// hex （ will / altcha salt etc use ）。
pub fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    getrandom::getrandom(&mut buf).expect("unavailable");
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

/// 1 kv （altcha etc use ； not in return None）。
pub fn kv_get(key: &str) -> Option<String> {
    let db = db()?;
    db.with(|conn| {
        conn.query_row(
            "SELECT value FROM kv WHERE key = ?1",
            rusqlite::params![key],
            |row| row.get::<_, String>(0),
        )
        .ok()
    })
    .flatten()
}

/// 1 kv （upsert）。 use 「RESERVED_KV_KEYS 
/// 」—— config by that excluding， this will 1 config。
pub fn kv_put(key: &str, value: &str) {
    let Some(db) = db() else { return };
    let _ = db.with_mut(|conn| {
        conn.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )
    });
}

// ── refresh token（， sha256）─────────────────────

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct RefreshRecord {
    /// sha256(token)
    hash: String,
    /// will id： 1 ， by 
    session: String,
    /// when （ms）
    expires_at: i64,
    /// already （ old not can ； = ，）
    rotated: bool,
    /// when （ms；0 = from ）。 and need use 
    /// （ `rotate_session`）。`#[serde(default)]`： before no has 
    /// this ， by 「 from 」 —— old 1 already in 
    /// ， and old 1 。
    #[serde(default)]
    rotated_at: i64,
}

static REFRESH_TOKENS: OnceLock<Mutex<Vec<RefreshRecord>>> = OnceLock::new();

fn refresh_tokens() -> &'static Mutex<Vec<RefreshRecord>> {
    REFRESH_TOKENS.get_or_init(|| Mutex::new(Vec::new()))
}

/// after empty（`db::backup::import_from_file` use ）。
///
/// `panelTokens` ， cookie for not up 
/// new ； old access will if ， use will 「 already 」 use to 
/// 401。emptyall new —— as and new 1 。
pub fn clear_access_tokens() {
    let mut table = match access_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table.clear();
    // in ， will 1 " down 1 " —— 1 empty
    persist_access_tokens(&mut table);
}

/// when access will （ and `load_refresh_tokens` when use ）。
pub fn load_access_tokens() {
    let Some(loaded) = kv_get(KV_ACCESS_KEY) else { return };
    let Ok(records) = serde_json::from_str::<Vec<AccessRecord>>(&loaded) else { return };
    let table = access_table_from(records, now_ms());
    match access_tokens().lock() {
        Ok(mut current) => *current = table,
        Err(poisoned) => *poisoned.into_inner() = table,
    }
}

/// → ： 1 （ can can is before that ）。
fn access_table_from(records: Vec<AccessRecord>, now: i64) -> AccessTable {
    records
        .into_iter()
        .filter(|record| record.expires_at > now)
        .map(|record| (record.hash, (record.session, record.expires_at)))
        .collect()
}

/// → （）。****： and 
/// is 1 kv， etc availablefile。
fn access_records(table: &AccessTable, now: i64) -> Vec<AccessRecord> {
    table
        .iter()
        .filter(|(_, (_, expiry))| *expiry > now)
        .map(|(hash, (session, expires_at))| AccessRecord {
            hash: hash.clone(),
            session: session.clone(),
            expires_at: *expires_at,
        })
        .collect()
}

fn persist_access_tokens(table: &mut AccessTable) {
    let records = access_records(table, now_ms());
    let payload = serde_json::to_string(&records).unwrap_or_else(|_| "[]".to_string());
    kv_put(KV_ACCESS_KEY, &payload);
}

/// when new （`bootstrap` after use 1 ；
/// after by `db::backup::import_from_file` 1 ）。
pub fn load_refresh_tokens() {
    let Some(loaded) = db().and_then(|db| {
        db.with(|conn| {
            conn.query_row(
                "SELECT value FROM kv WHERE key = ?1",
                rusqlite::params![KV_TOKENS_KEY],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .flatten()
    }) else {
        return;
    };
    let Ok(records) = serde_json::from_str::<Vec<RefreshRecord>>(&loaded) else {
        return;
    };
    match refresh_tokens().lock() {
        Ok(mut table) => *table = records,
        Err(poisoned) => *poisoned.into_inner() = records,
    }
}

fn persist_refresh_tokens(table: &mut Vec<RefreshRecord>) {
    // （： 1 + if ）
    let now = now_ms();
    table.retain(|record| record.expires_at > now);
    if let Some(db) = db() {
        let payload = serde_json::to_string(&*table).unwrap_or_else(|_| "[]".to_string());
        let _ = db.with_mut(|conn| {
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                rusqlite::params![KV_TOKENS_KEY, payload],
            )
        });
    }
}

// ── will / / / ─────────────────────────────

/// 1 ：** 1 ** Set-Cookie to handler down （ 1 ，
/// `ACCESS_COOKIE` ）； when response， cookie not 
/// localStorage after request（ `session_valid` ）。
pub struct IssuedSession {
    pub session_cookie: String,
    pub access_token: String,
    pub refresh_token: String,
    /// this access **remaining**（seconds，）。
    ///
    /// to client**** use ： before has 「 etc 401 」 this 1 ， is access
    /// 1 need 1 401 —— that （，
    /// 2.5 seconds 1 ） 1 up ， and 2 when 1 。client
    /// to remaining can in to before ， this 401 1 。
    ///
    /// client when is ****， not is ：server by will valid
    /// （`session_valid`），、 when 、client not will will 「401 → 」
    /// path， not will 。
    pub expires_in: i64,
}

impl IssuedSession {
    /// access remaining（seconds）。 and [`IssuedSession::issue`]，
    /// etc `ACCESS_TTL`； is as 「 from 」 has 1 ——
    /// will if ， this 1 can 。
    pub fn access_ttl_secs() -> i64 {
        ACCESS_TTL.as_secs() as i64
    }
}

/// 1 use `Set-Cookie` ：`agent2api-panel=<access>.<refresh>`。
///
/// Path as `/`、`HttpOnly`、`SameSite=Lax`；`Max-Age` refresh 
/// （access by server，cookie is —— access when 
/// will 1 ）。
fn issue_cookie(access_token: &str, refresh_token: &str) -> String {
    format!(
        "{ACCESS_COOKIE}={access_token}{TOKEN_SEP}{refresh_token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        REFRESH_TTL.as_secs()
    )
}

/// 1 use `Set-Cookie`（ Path + `Max-Age=0`）。
///
/// Path must and when **** —— cookie need and Path when in 。
/// from before this `Path=/`， refresh cookie in `Path=/api/panel` down ，
/// is from no ： 1 old refresh，
/// `/api/panel/*` ； that 1 not in ，`rotate_session`
/// can by 「 already 」 in this old ⇒ 、 will ⇒ use 
/// 。 this is 「 cookie Why」 1 。
/// in has 1 will cookie、 has 1 Path， this " 1 not " not in 。
pub fn clear_cookie() -> String {
    format!("{ACCESS_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

/// from 1 will cookie (access, refresh)。
///
/// must**empty**，otherwise not ：`a.` / `.b` / `.` this is 
/// in response after down ， will 1 not in 
/// （ need —— empty in `cookie_values` that 1 already ， no has ）。
///
/// no has （ before that down access） access ： need in server
/// （ that use ， is this 1 no has cookie refresh， in ），
/// need already not （ down 1 401 will `clear_cookie` ）。 not as 1 。
fn split_session_cookie(value: &str) -> (Option<&str>, Option<&str>) {
    if let Some((access, refresh)) = value.split_once(TOKEN_SEP) {
        return if access.is_empty() || refresh.is_empty() {
            (None, None)
        } else {
            (Some(access), Some(refresh))
        };
    }
    if value.is_empty() { (None, None) } else { (Some(value), None) }
}

impl IssuedSession {
    fn issue(session: String) -> Self {
        let access_token = random_hex(32);
        let refresh_token = random_hex(48);
        let expiry = now_ms() + ACCESS_TTL.as_millis() as i64;
        match access_tokens().lock() {
            Ok(mut table) => {
                table.insert(sha256_hex(&access_token), (session.clone(), expiry));
                persist_access_tokens(&mut table);
            }
            Err(poisoned) => {
                let mut table = poisoned.into_inner();
                table.insert(sha256_hex(&access_token), (session.clone(), expiry));
                persist_access_tokens(&mut table);
            }
        }
        match refresh_tokens().lock() {
            Ok(mut table) => {
                table.push(RefreshRecord {
                    hash: sha256_hex(&refresh_token),
                    session,
                    expires_at: now_ms() + REFRESH_TTL.as_millis() as i64,
                    rotated: false,
                    rotated_at: 0,
                });
                persist_refresh_tokens(&mut table);
            }
            Err(poisoned) => {
                let mut table = poisoned.into_inner();
                table.push(RefreshRecord {
                    hash: sha256_hex(&refresh_token),
                    session,
                    expires_at: now_ms() + REFRESH_TTL.as_millis() as i64,
                    rotated: false,
                    rotated_at: 0,
                });
                persist_refresh_tokens(&mut table);
            }
        }
        Self {
            session_cookie: issue_cookie(&access_token, &refresh_token),
            access_token,
            refresh_token,
            expires_in: Self::access_ttl_secs(),
        }
    }

    /// ： 1 new will 。
    pub fn new_session() -> Self {
        Self::issue(random_hex(16))
    }
}

/// access **request**：`x-panel-token` → `Authorization: Bearer`。
///
/// in by ： not total in —— from fnOS docker 1 
/// when ，request in ，HttpOnly cookie （ down → ） can can 。
/// before （web_shim / login.html） response localStorage， after 
/// request；server**cookie in before 、 in after **（ `access_candidates`），
/// as not 。
/// JS can （localStorage）， and HttpOnly cookie to 
/// XSS —— this is down available， is can 。
fn access_header_tokens(headers: &HeaderMap) -> Vec<String> {
    named_header_tokens(headers, "x-panel-token")
        .into_iter()
        .chain(bearer_tokens_of(headers))
        .collect()
}

/// 1 （ can can ）request allempty。
///
/// is need ： and in （ response 1 that ） will 
/// this **request**， is 1 `Cookie` can can `agent2api-panel=`
/// —— 1 valid 1 old 。 1 etc ""。
fn named_header_tokens(headers: &HeaderMap, name: &str) -> Vec<String> {
    let name = axum::http::HeaderName::from_bytes(name.as_bytes());
    let Ok(name) = name else { return Vec::new() };
    headers
        .get_all(name)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn bearer_tokens_of(headers: &HeaderMap) -> Vec<String> {
    named_header_tokens(headers, "authorization")
        .into_iter()
        .filter_map(|value| {
            let token = value.strip_prefix("Bearer ")?.trim();
            if token.is_empty() {
                return None;
            }
            Some(token.to_string())
        })
        .collect()
}

/// access ****， by 「 cookie → `x-panel-token` → Bearer」
/// （cookie in before ， and from before 1 ）。
///
/// ── Why not is 「cookie 、 in 」────────────────────
/// （2026-09-29，fnOS docker :5666 in ）：success new will after ，
/// down 1 `/api/session` invalid， for not up —— is 
/// ** up 1 ** that （`7a951233`）， （`84f8d751`） no up 。 is in 
/// response `Set-Cookie` by 1 ：`agent2api-panel= new , Path=/, HttpOnly, …`
/// `Path` from 1 cookie（ that `path|domain|max-age|
/// expires|session|version` is ）， new will no by `/` down ， is 
/// that down old 。
///
/// when ** 1 ** can this will is request， in 「response」
/// down in （ `api::panel::issue_response`）。 this must and ： 
/// in → client to body and → server** not ** that old 
/// cookie 。 1 is 「 cookie 」 ， or 「
/// not to 」。
///
/// up that **** after from ：/ new as ** 1 ** will cookie
/// （access and refresh 1 ， `ACCESS_COOKIE`）， 1 Set-Cookie no has can 
/// for ； to 1 —— need and for 1 ， in need is 
/// 1 。「 cookie can can has 」 this old ： that before down not will 
/// ， is 。
///
/// cookie in before ： in ， use to cookie ，localStorage
/// that already old not this （ not ， not —— is 
/// 「 has no has 1 valid」， not is 「 1 is 」）。
fn access_candidates(headers: &HeaderMap) -> Vec<String> {
    dedup(
        session_cookie_halves(headers)
            .into_iter()
            .filter_map(|half| half.0)
            .chain(access_header_tokens(headers))
            .collect(),
    )
}

/// refresh ****， and `access_candidates` 
/// （ will cookie 1 → old refresh cookie → `x-panel-refresh`
/// → Bearer）。
///
/// and access use 1 「Why not 」 by ：request will that old 
/// refresh ， is —— up ，，
/// use （ this path before in "expired"， no is ）。
///
/// old before that 1 is **** use （ `LEGACY_REFRESH_COOKIE`）： in will cookie
/// after ， new in when old not will 。
pub fn refresh_candidates(headers: &HeaderMap) -> Vec<String> {
    dedup(
        session_cookie_halves(headers)
            .into_iter()
            .filter_map(|half| half.1)
            .chain(cookie_values(headers, LEGACY_REFRESH_COOKIE))
            .chain(named_header_tokens(headers, "x-panel-refresh"))
            .chain(bearer_tokens_of(headers))
            .collect(),
    )
}

/// ** 1 ** will cookie (access, refresh)。
///
/// （ not is 1 ） is must ： and in will in down down ， its in 1 
/// is new will 、 1 is ； 1 etc 。
fn session_cookie_halves(headers: &HeaderMap) -> Vec<(Option<String>, Option<String>)> {
    cookie_values(headers, ACCESS_COOKIE)
        .iter()
        .map(|value| {
            let (access, refresh) = split_session_cookie(value);
            (access.map(str::to_string), refresh.map(str::to_string))
        })
        .collect()
}

fn dedup(values: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

/// by use will 。
pub fn local_session_valid(headers: &HeaderMap) -> bool {
    let tokens = access_candidates(headers);
    local_session::valid_any(tokens.iter().map(String::as_str))
}

/// requestvalid will or access token when 。
///
/// from before this 1 （cookie has cookie）， is " has 1 cookie"
/// and " no "（ is 401）， not —— 
/// `access_candidates` in 。
pub fn session_valid(headers: &HeaderMap) -> bool {
    let tokens = access_candidates(headers);
    if tokens.is_empty() {
        return false;
    }
    if local_session::valid_any(tokens.iter().map(String::as_str)) {
        return true;
    }
    let now = now_ms();
    let mut table = match access_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table.retain(|_, (_, expiry)| *expiry > now);
    tokens
        .iter()
        .any(|token| table.contains_key(&sha256_hex(token)))
}

/// request is ****（ will cookie 1 、`x-panel-token`、
/// `x-panel-refresh`、`Authorization: Bearer`、`x-api-key`）。
///
/// ── Why need need this ──────────────────────────────────────
/// 「 will invalid」 that 1 for all 401 1 ， its in not ：
/// · ** no ** —— 1 、 `/api/session`
/// 。 this is ， not is ； 1 will and 
/// `/api/*`， by error etc 1 ， will 
/// error、 500 。
/// · ** for not up ** —— cookie、、Key 。 this is need 。
///
/// 「 has no has 」， not for not for —— for not for is that in 。
///
/// cookie（`PROBE_COOKIE`）** not **： not will ， before will 
/// down ， will 「」「」。
pub fn presents_credentials(headers: &HeaderMap) -> bool {
    !access_candidates(headers).is_empty()
        || !refresh_candidates(headers).is_empty()
        || headers.contains_key("x-api-key")
}

/// request can to access （ use ， —— 「 cookie
/// up 」 when need can 1 is ， not ）。
pub fn access_fingerprints(headers: &HeaderMap) -> Vec<String> {
    access_candidates(headers)
        .into_iter()
        .take(2)
        .map(|token| token_fingerprint(&token))
        .collect()
}

/// use request refresh new 1 （ 1 will ）。
///
/// by ，** 1 **（、） is current will —— old cookie
/// and old not will 。return `None` = no has 1 can use 。
///
/// **** old ， in 「 no has 1 is 」 when ： when if 
/// in 1 already ，（ up new in ）—— will 
/// ， new 。 need ： when new old is ****
/// （ and in is this ）， " to old that " will 。
///
/// already has 1 ****（`ROTATION_GRACE_MS`）： 
/// up ， by 「 1 and 」 not is —— access that 1 ，
/// / up down will ** 1 ** no refresh when ，
/// to success、 after to will up 「 already 」； this is ， not is 。client 
/// and 1 up down and ， can in server
/// （ `rotate_session` ）。 old 。
pub fn rotate_session(headers: &HeaderMap) -> Option<IssuedSession> {
    let candidates = refresh_candidates(headers);
    if candidates.is_empty() {
        return None;
    }
    let hashes: Vec<String> = candidates.iter().map(|token| sha256_hex(token)).collect();
    let now = now_ms();
    let mut table = match refresh_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table.retain(|record| record.expires_at > now);
    let live = hashes.iter().find_map(|hash| {
        table
            .iter()
            .position(|record| &record.hash == hash && !record.rotated)
    });
    if let Some(index) = live {
        let session = table[index].session.clone();
        table[index].rotated = true;
        table[index].rotated_at = now;
        drop(table);
        return Some(IssuedSession::issue(session));
    }
    // 1 no has ： by 「 already 」， will 
    // （refresh + this all access）。
    let replay = hashes.iter().find_map(|hash| {
        table
            .iter()
            .position(|record| &record.hash == hash && record.rotated)
    });
    // already no in → invalid（ / will / ）：
    // by 「 new failure」， not 。
    let index = replay?;
    let session = table[index].session.clone();
    // ── and ─────────────────────────────────────
    // this refresh in 、 and up has → ： from up 
    // 1 new to this after 。** not ** that —— 
    // to that ， etc 「 to new refresh」。
    // （ or up already none，e.g. / already ） old 。
    // ： in 「 use after 1 minutes」 can new —— has 
    // 1 minutes， and in that before valid， no has 。
    if now - table[index].rotated_at <= ROTATION_GRACE_MS {
        if let Some(live) = table
            .iter()
            .find(|record| record.session == session && !record.rotated)
        {
            let session = live.session.clone();
            drop(table);
            return Some(IssuedSession::issue(session));
        }
    }
    table.retain(|record| record.session != session);
    persist_refresh_tokens(&mut table);
    drop(table);
    revoke_access_of_session(&session);
    None
}

/// request is 1 **** refresh（、）。： not 、
/// not 、 not will 。
///
/// in by is 401 must， this in request up 1 1 ：
///
/// - **access 、refresh **： this is before **** in 。 that will cookie
/// must in —— client to 401 down 1 is `/api/panel/refresh`
/// new 。 when etc 30 2 when 1 
/// （access is `ACCESS_TTL`， 1 cookie 30 refresh）。
/// - ** will already no **： this is this cookie —— not that 
/// new 1 invalid（： 1 for not up 
/// 51 ）。
///
/// and `rotate_session` 「」 1 ： need has 1 in and 
/// ，，cookie not can 。
pub fn has_live_refresh(headers: &HeaderMap) -> bool {
    let hashes: Vec<String> = refresh_candidates(headers)
        .iter()
        .map(|token| sha256_hex(token))
        .collect();
    if hashes.is_empty() {
        return false;
    }
    let now = now_ms();
    let mut table = match refresh_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table.retain(|record| record.expires_at > now);
    hashes.iter().any(|hash| {
        table
            .iter()
            .any(|record| &record.hash == hash && !record.rotated)
    })
}

/// ： by refresh to will ， + request can to access。
pub fn revoke_session(headers: &HeaderMap) {
    let hashes: Vec<String> = refresh_candidates(headers)
        .iter()
        .map(|token| sha256_hex(token))
        .collect();
    let presented_access = access_candidates(headers);
    local_session::revoke_any(&presented_access);
    let target_session;
    {
        let mut table = match refresh_tokens().lock() {
            Ok(table) => table,
            Err(poisoned) => poisoned.into_inner(),
        };
        target_session = hashes
            .iter()
            .find_map(|hash| {
                table
                    .iter()
                    .find(|record| &record.hash == hash)
                    .map(|record| record.session.clone())
            });
        if let Some(session) = &target_session {
            table.retain(|record| record.session != *session);
            persist_refresh_tokens(&mut table);
        }
    }
    // access 1 and ： token by （ 1 ，including old that ——
    // when no old ）， can will by 
    let mut table = match access_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    for token in presented_access {
        table.remove(&sha256_hex(&token));
    }
    if let Some(session) = &target_session {
        table.retain(|_, (session_of, _)| session_of != session);
    }
    persist_access_tokens(&mut table);
}

/// will all access token （ use ）。
fn revoke_access_of_session(session: &str) {
    let mut table = match access_tokens().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table.retain(|_, (session_of, _)| session_of != session);
    persist_access_tokens(&mut table);
}

/// `Cookie` ****（ by up ）。
///
/// this ** not **「 1 」 that ：（ `access_candidates`） 
/// is 「 cookie has up when 1 」。 1 `Option<String>` 
/// etc to down 1 1 —— need in use `.first()`， need 
/// to `session_valid` / `rotate_session` that 1 。
///
/// is ** will ** ， not is ： in 、 or 1 not /path
/// 1 （cookie by not ，fnOS :5666 in and :3065 use 1 ）。
pub fn cookie_values(headers: &HeaderMap, name: &str) -> Vec<String> {
    let Some(header) = headers.get(axum::http::header::COOKIE).and_then(|v| v.to_str().ok()) else {
        return Vec::new();
    };
    cookie_values_from_str(header, name)
}

/// `Cookie` ****（，** to 、 not **）。
///
/// Why need this ：「success 」 this ， to 「cookie has / none」
/// is not —— in （fnOS docker this ） is `Set-Cookie`
/// and or 1 ， is is **** cookie or ** up 1 ** will 
/// cookie： is "cookie= has "， for not up this 1 will 。 can 1 
/// （ no to / to not is this / in ）， is ， etc 
/// to up ， this 。
pub fn cookie_names(headers: &HeaderMap) -> Vec<String> {
    let Some(header) = headers.get(axum::http::header::COOKIE).and_then(|v| v.to_str().ok()) else {
        return Vec::new();
    };
    let mut names: Vec<String> = header
        .split(';')
        .filter_map(|part| part.split('=').next().map(str::trim))
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();
    names.sort();
    names.dedup();
    names
}

/// ****（sha256 before 8 ）—— as for up ， not 。
///
/// not can （ that etc SQLite and file）。
/// 「success、 down 1 request invalid」 when must can 1 ： that 
/// access cookie is ** this ** is ** up 1 ** —— in "cookie= has "
/// 1 1 。8 （4×10⁸ empty） after ， not 。
pub fn token_fingerprint(value: &str) -> String {
    sha256_hex(value).chars().take(8).collect()
}

/// down access （ and after request for up use ）。
pub fn access_fingerprint(session: &IssuedSession) -> String {
    token_fingerprint(&session.access_token)
}

fn cookie_values_from_str(header: &str, name: &str) -> Vec<String> {
    header
        .split(';')
        .filter_map(|part| {
            let part = part.trim();
            let value = part.strip_prefix(name)?.strip_prefix('=')?;
            Some(value.trim().to_string())
        })
        // ** has ** that ： in Set-Cookie when `name=`（empty）
        // in ， not is ， will 1 empty 。
        .filter(|value: &String| !value.is_empty())
        .collect()
}

// ── failure（ by IP）────────────────────────────────

struct Attempt {
    failures: u32,
    locked_until: Option<Instant>,
}

static ATTEMPTS: OnceLock<Mutex<HashMap<IpAddr, Attempt>>> = OnceLock::new();

fn attempts() -> &'static Mutex<HashMap<IpAddr, Attempt>> {
    ATTEMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// this current is in 。
pub fn login_locked(source: IpAddr) -> bool {
    let table = match attempts().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    table
        .get(&source)
        .and_then(|attempt| attempt.locked_until)
        .is_some_and(|until| Instant::now() < until)
}

/// failure：； to 1 when 。
pub fn record_login_failure(source: IpAddr) {
    let mut table = match attempts().lock() {
        Ok(table) => table,
        Err(poisoned) => poisoned.into_inner(),
    };
    let attempt = table
        .entry(source)
        .or_insert(Attempt { failures: 0, locked_until: None });
    attempt.failures += 1;
    if attempt.failures >= LOCKOUT_THRESHOLD {
        attempt.locked_until = Some(Instant::now() + LOCKOUT_DURATION);
        attempt.failures = 0;
    }
}

/// success：empty this failure。
pub fn clear_login_failures(source: IpAddr) {
    match attempts().lock() {
        Ok(mut table) => {
            table.remove(&source);
        }
        Err(poisoned) => {
            poisoned.into_inner().remove(&source);
        }
    };
}

// ── /v1 fail-closed（headless Key when ， bin）────

static V1_FAIL_CLOSED: AtomicBool = AtomicBool::new(false);

/// headless ： when `/api/*`（/ 
/// ） 1 —— otherwise before to 200 will ， 「
/// must」 can 1 。 not set（ not ）。
static PANEL_GATE: AtomicBool = AtomicBool::new(false);

/// headless is 「 /api/* 」 down 。
pub fn panel_gate() -> bool {
    PANEL_GATE.load(Ordering::Relaxed)
}

/// headless when （enabled after ， is as 
/// ：， not ）。
pub fn set_panel_gate(on: bool) {
    PANEL_GATE.store(on, Ordering::Relaxed);
}

/// `/v1/*` is fail-closed（not configured Key headless ）。
pub fn v1_fail_closed() -> bool {
    V1_FAIL_CLOSED.load(Ordering::Relaxed)
}

/// headless when by 「 has no has Key」 is fail-closed。
pub fn set_v1_fail_closed(on: bool) {
    V1_FAIL_CLOSED.store(on, Ordering::Relaxed);
}

// Note: `panelAdmin` / `panelTokens` / `panelAccessSessions` kv already 
// `db::schema::RESERVED_KV_KEYS`（configexcluding）—— add when must
// ，otherwise use 1 config will and 。

#[cfg(test)]
mod credential_candidate_tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderName, HeaderValue};

    fn header_name(value: &str) -> HeaderName {
        HeaderName::from_bytes(value.as_bytes()).expect(" must is token")
    }

    /// up request：`cookie` （empty = not Cookie ）+ any 
    /// `(, )`（ can ， append， as is 「 1 up 」
    /// this in ）。
    fn request(cookie: &str, pairs: &[(&str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if !cookie.is_empty() {
            headers.insert(
                axum::http::header::COOKIE,
                HeaderValue::from_str(cookie).unwrap(),
            );
        }
        for (name, value) in pairs {
            headers.append(header_name(name), HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    /// will cookie up （ 1 ， and `issue_cookie` ）。
    fn session_cookie(session: &IssuedSession) -> String {
        format!(
            "{ACCESS_COOKIE}={}.{}",
            session.access_token, session.refresh_token
        )
    }

    /// refresh will cookie（access that 1 is ）——
    /// use can refresh not access 。
    fn refresh_only_cookie(token: &str) -> String {
        format!("{ACCESS_COOKIE}=not-a-live-access.{token}")
    }

    /// ： cookie → `x-panel-refresh` → Bearer，。
    ///
    /// cookie in before （ and from before ）； in 「 1 invalid when 
    /// not 」。cookie is ** 1 will cookie **（refresh）。
    #[test]
    fn candidates_are_cookie_values_first_then_headers_and_deduplicated() {
        let headers = request(
            "agent2api-panel=acc-a.rt-a; x=1; agent2api-panel=acc-b.rt-b",
            &[("x-panel-refresh", "from-header"), ("authorization", "Bearer from-bearer")],
        );
        assert_eq!(
            vec![
                "rt-a".to_string(),
                "rt-b".to_string(),
                "from-header".to_string(),
                "from-bearer".to_string(),
            ],
            refresh_candidates(&headers)
        );
        // 1 cookie 1 access 、 refresh ， not 
        let halves = request("agent2api-panel=acc-a.rt-a", &[]);
        assert_eq!(vec!["acc-a".to_string()], access_candidates(&halves));

        // 1 in cookie in ： 1 
        let once = request("agent2api-panel=acc.same", &[("x-panel-refresh", "same")]);
        assert_eq!(vec!["same".to_string()], refresh_candidates(&once));

        // empty not （`name=` is Set-Cookie down ， not is ）
        let blanks = request("agent2api-panel=;", &[("x-panel-refresh", "   ")]);
        assert!(refresh_candidates(&blanks).is_empty());
        // for is （ in response after is this ）： not 
        for broken in ["agent2api-panel=.", "agent2api-panel=acc.", "agent2api-panel=.rt"] {
            let headers = request(broken, &[]);
            assert!(access_candidates(&headers).is_empty(), "{broken} not this access ");
            assert!(refresh_candidates(&headers).is_empty(), "{broken} not this refresh ");
        }
    }

    /// access 1 （cookie → x-panel-token → Bearer）。
    ///
    /// before " has 1 " is ** has **： before that down access
    /// need can （`split_session_cookie` that ），otherwise will 
    /// will none 1 。 1 after down 1 use 。
    #[test]
    fn access_candidates_follow_the_same_order() {
        let headers = request(
            "agent2api-panel=cookie-one; agent2api-panel=cookie-two",
            &[("x-panel-token", "header-one"), ("authorization", "Bearer bearer-one")],
        );
        assert_eq!(
            vec![
                "cookie-one".to_string(),
                "cookie-two".to_string(),
                "header-one".to_string(),
                "bearer-one".to_string(),
            ],
            access_candidates(&headers)
        );
    }

    /// 1 ： 1 is access ， and this cookie when refresh。
    #[test]
    fn one_cookie_feeds_both_sides() {
        let session = IssuedSession::new_session();
        let text = session_cookie(&session);
        // and ： will cookie must is `issue_cookie` that 
        assert_eq!(text, session.session_cookie.split(';').next().unwrap().to_string());
        assert_eq!(vec![session.access_token.clone()], access_candidates(&request(&text, &[])));
        assert_eq!(vec![session.refresh_token.clone()], refresh_candidates(&request(&text, &[])));
        assert!(session_valid(&request(&text, &[])), " 1 cookie this will ");
    }

    /// ： up 1 that refresh cookie（`agent2api-panel-rt`） when ，
    /// must refresh —— otherwise that 1 access is new （empty），
    /// 1 can will has ， not etc ** 1 **。
    ///
    /// ： can （ old not ）， and ** not **（ will cookie
    /// in when new in before ）。
    #[test]
    fn a_legacy_refresh_cookie_still_feeds_the_rotation() {
        let session = IssuedSession::new_session();
        // has old ： need 
        let legacy_only = request(
            &format!("{LEGACY_REFRESH_COOKIE}={}", session.refresh_token),
            &[],
        );
        assert_eq!(
            vec![session.refresh_token.clone()],
            refresh_candidates(&legacy_only),
            " old refresh must（ not down ）"
        );
        assert!(
            rotate_session(&legacy_only).is_some(),
            " old cookie can to new "
        );

        // old + new when in ： new in before ， old not 
        let both = request(
            &format!(
                "{}=acc.{rt}; {LEGACY_REFRESH_COOKIE}=legacy-rt",
                ACCESS_COOKIE,
                rt = session.refresh_token
            ),
            &[],
        );
        assert_eq!(
            vec![session.refresh_token.clone(), "legacy-rt".to_string()],
            refresh_candidates(&both),
            " will cookie refresh need in old before "
        );
    }

    /// for （2026-09-29，fnOS :5666 in ）： 1 ****
    /// cookie、 to —— old in cookie that 1 ， is 
    /// 「success → down 1 request 401 → 」。 in 1 need 。
    #[test]
    fn a_stale_cookie_cannot_shadow_the_header_token() {
        let session = IssuedSession::new_session();
        let stale = random_hex(32);

        let with_header = request(
            &format!("{ACCESS_COOKIE}={stale}"),
            &[("x-panel-token", &session.access_token)],
        );
        assert!(
            session_valid(&with_header),
            " cookie not valid in "
        );

        // 1 will 、 1 cookie， is ** not that ** → invalid。
        // this 1 is up 1 ：，「 up true」 can can is 
        // 「」 can can is 「 none」。
        let cookie_only = request(&format!("{ACCESS_COOKIE}={stale}"), &[]);
        assert!(
            !session_valid(&cookie_only),
            " cookie when mustinvalid（otherwise up 1 no ）"
        );
    }

    /// cookie up （ not Path 1 up ） when ， 1 
    /// will —— valid that in after 1 。
    #[test]
    fn every_repeated_cookie_value_gets_tried() {
        let session = IssuedSession::new_session();
        let stale = random_hex(32);

        let stale_first = request(
            &format!("{ACCESS_COOKIE}={stale}; {ACCESS_COOKIE}={}", session.access_token),
            &[],
        );
        assert!(session_valid(&stale_first), " cookie must to ");

        let live_first = request(
            &format!("{ACCESS_COOKIE}={}; {ACCESS_COOKIE}={stale}", session.access_token),
            &[],
        );
        assert!(session_valid(&live_first));

        // is → invalid（ up ， to up 1 for ）
        let both_stale = request(
            &format!("{ACCESS_COOKIE}={stale}; {ACCESS_COOKIE}={}", random_hex(32)),
            &[],
        );
        assert!(!session_valid(&both_stale));
    }

    /// Bearer need ：request in 。
    #[test]
    fn the_bearer_channel_is_a_candidate_too() {
        let session = IssuedSession::new_session();
        let headers = request(
            &format!("{ACCESS_COOKIE}={}", random_hex(32)),
            &[("authorization", &format!("Bearer {}", session.access_token))],
        );
        assert!(session_valid(&headers));
    }

    /// ：cookie that ** already ** refresh in before when ， need to that ，
    /// not 。
    #[test]
    fn rotation_skips_a_rotated_stale_refresh_to_the_live_one() {
        let first = IssuedSession::new_session();
        let second = rotate_session(&request(&refresh_only_cookie(&first.refresh_token), &[]))
            .expect("success");
        assert!(
            session_valid(&request(&session_cookie(&second), &[])),
            " access will （otherwise after in empty）"
        );

        // in first.refresh is 「 already 」，second.refresh is 。
        // if first that （） and in its before ， old will in 、
        // 、 → return None → use 。
        let mixed = request(
            &refresh_only_cookie(&first.refresh_token),
            &[("x-panel-refresh", &second.refresh_token)],
        );
        let third = rotate_session(&mixed).expect(" old that not this ");

        // ： new access valid、 new refresh can 1 
        assert!(session_valid(&request(&session_cookie(&third), &[])));
        assert!(rotate_session(&request(&session_cookie(&third), &[])).is_some());
    }

    /// no ：**** already old （ no has 1 
    /// in ）， is will ； 1 as then is and （
    /// when up access ， `rotate_session` ）。
    #[test]
    fn a_replayed_refresh_alone_still_kills_its_chain() {
        let first = IssuedSession::new_session();
        let second = rotate_session(&request(&refresh_only_cookie(&first.refresh_token), &[]))
            .expect("success");

        // old up = and ：， and not 
        let concurrent = rotate_session(&request(&refresh_only_cookie(&first.refresh_token), &[]))
            .expect(" refresh this by and new ");
        assert!(
            session_valid(&request(&session_cookie(&concurrent), &[])),
            " new access need valid"
        );
        assert!(
            rotate_session(&request(&session_cookie(&second), &[])).is_some(),
            " not up has refresh"
        );

        // already to → 1 old up is ：
        {
            let mut table = refresh_tokens().lock().unwrap();
            for record in table.iter_mut() {
                if record.rotated {
                    record.rotated_at = now_ms() - ROTATION_GRACE_MS - 1;
                }
            }
        }
        let replayed = rotate_session(&request(&refresh_only_cookie(&first.refresh_token), &[]));
        assert!(replayed.is_none(), " already must");

        // ****（）—— this 1 is 「」 
        assert!(
            rotate_session(&request(&session_cookie(&second), &[])).is_none(),
            " will "
        );
        assert!(
            !session_valid(&request(&session_cookie(&second), &[])),
            " this access need 1 and "
        );
    }

    /// ： 1 in ， and ** 1 ** up access by 。
    #[test]
    fn logout_revokes_through_the_stale_cookie_and_the_header_together() {
        let session = IssuedSession::new_session();
        let foreign_access = random_hex(32);
        let headers = request(
            &session_cookie(&session),
            &[("x-panel-token", &session.access_token)],
        );
        assert!(session_valid(&headers));
        revoke_session(&headers);

        // no ：refresh not new ，access not valid
        assert!(rotate_session(&request(&session_cookie(&session), &[])).is_none());
        assert!(!session_valid(&request(
            &format!("{ACCESS_COOKIE}={}", session.access_token),
            &[]
        )));
        // for ： no 1 access not （ is ， not is ）
        let other = IssuedSession::new_session();
        assert!(session_valid(&request(
            &format!("{ACCESS_COOKIE}={}", other.access_token),
            &[]
        )));
        assert!(!session_valid(&request(
            &format!("{ACCESS_COOKIE}={foreign_access}"),
            &[]
        )));
    }
}

#[cfg(test)]
mod session_lifecycle_tests {
    //! will ：****（ not down ） and ****
    //! （ cookie not will 401）。
    //!
    //! is for ：access in ， 1 
    //! all already invalid， 401 not cookie ⇒ 1 for not up 
    //! 51 。「 old 」 。

    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    /// up Cookie （ is will cookie ）。
    fn cookie_header(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            HeaderValue::from_str(&format!("{ACCESS_COOKIE}={value}")).unwrap(),
        );
        headers
    }

    /// 1 will cookie （ and `issue_cookie` ）—— use by up 。
    fn cookie_value(session: &IssuedSession) -> String {
        format!("{}.{}", session.access_token, session.refresh_token)
    }

    /// 1 Set-Cookie Path （ no has None）—— and for 。
    fn path_of(cookie: &str) -> Option<&str> {
        cookie
            .split(';')
            .map(str::trim)
            .find_map(|part| part.strip_prefix("Path=").map(str::trim))
    }

    /// will after valid（= not down ）。
    ///
    /// this is ** 1 **： from 「 as 」「sha256 as 」 when ， need 
    /// and has 1 ， this （ `session_valid` not ——
    /// is not to ， and 「 」 1 1 ）。
    #[test]
    fn an_issued_session_survives_a_reload() {
        let issued = IssuedSession::new_session();
        let table = access_tokens().lock().unwrap().clone();
        let dumped = serde_json::to_string(&access_records(&table, now_ms())).unwrap();
        let reloaded =
            access_table_from(serde_json::from_str::<Vec<AccessRecord>>(&dumped).unwrap(), now_ms());
        assert!(
            reloaded.contains_key(&sha256_hex(&issued.access_token)),
            " must by sha256 this "
        );
        // that after ， old as （ this not ， and and use ）
        let hash = sha256_hex(&issued.access_token);
        assert!(reloaded.get(&hash).is_some_and(|(_, expiry)| *expiry > now_ms()));
        assert!(session_valid(&cookie_header(&cookie_value(&issued))));
    }

    /// that is before ， not can （， not is `Instant`）。
    #[test]
    fn expired_records_do_not_come_back() {
        let expired = vec![AccessRecord {
            hash: sha256_hex("a-dead-token"),
            session: "s".to_string(),
            expires_at: now_ms() - 1,
        }];
        assert!(access_table_from(expired, now_ms()).is_empty(), " not this ");
    }

    /// has ：/ 1 kv ， etc available
    /// file。
    #[test]
    fn stored_records_never_carry_a_raw_token() {
        let issued = IssuedSession::new_session();
        let table = access_tokens().lock().unwrap().clone();
        let dumped = serde_json::to_string(&access_records(&table, now_ms())).unwrap();
        assert!(dumped.contains(&sha256_hex(&issued.access_token)));
        assert!(
            !dumped.contains(&issued.access_token),
            " not can （ that file etc file）"
        );
    }

    /// ** 1 ** `Set-Cookie` down will ， in 1 up 。
    ///
    /// is （ and in ）：
    /// · has ** 1 ** cookie —— `Set-Cookie` will in by 1 ，
    /// as `path|domain|max-age|expires` cookie， new will not 
    /// （ in need ， 1 ： 1 63 ）；
    /// · Path and 1 —— cookie need and Path when in ，
    /// from before refresh in `/api/panel` `/`， that from no ；
    /// · 1 in （access and refresh），otherwise not to 。
    /// old this （ Set-Cookie、refresh Path not etc ）。
    #[test]
    fn one_cookie_carries_the_whole_session_and_clearing_targets_it() {
        let issued = IssuedSession::new_session();
        // 1 ： has 1 ， not cookie
        let (pair, attributes) = issued.session_cookie.split_once("; ").expect(" Path ");
        assert!(
            pair.contains('.'),
            " will cookie must is access.refresh ， {pair}"
        );
        assert!(
            pair.starts_with(&format!("{ACCESS_COOKIE}=")),
            "cookie need is that 1 ， {pair}"
        );
        assert!(
            !issued.session_cookie.contains(','),
            " 1 Set-Cookie not this has —— is and in "
        );
        assert!(attributes.contains("Path=/"));
        assert!(attributes.contains("HttpOnly"));
        assert!(attributes.contains("SameSite=Lax"));
        assert!(
            attributes.contains(&format!("Max-Age={}", REFRESH_TTL.as_secs())),
            " refresh：access by server will ，cookie is "
        );
        assert_eq!(
            path_of(&clear_cookie()),
            path_of(&issued.session_cookie),
            " Path must and 1 "
        );
        assert_eq!(path_of(&issued.session_cookie), Some("/"));
        assert!(clear_cookie().contains(&format!("{ACCESS_COOKIE}=;")));
        assert!(clear_cookie().contains("Max-Age=0"));
        // ： 1 access，
        assert!(session_valid(&cookie_header(&cookie_value(&issued))));
        assert!(rotate_session(&cookie_header(&cookie_value(&issued))).is_some());
    }
}
