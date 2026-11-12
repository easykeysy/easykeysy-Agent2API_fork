//! **set and **（kv `cloudSync`） and config。
//!
//! ── Why not `config` ───────────────────────────
//! and ，「 its 」； is **
//! not can up **（otherwise will to ， when will old ）。
//! `RESERVED_KV_KEYS` after ， excluding —— this 
//! 「 」， not is 1 。
//!
//! ── / Secret ────────────────────────────────────
//! `crypto::seal`（`SecretScope::CloudSync`） after ， in 
//! 「 that 1 」 and 「request that 1 」 in 。set when 
//! `hasPassword` / `hasSecretKey`， not —— and `core::update::token` 
//! `status_json` 1 to 。
//!
//! ── Why is ────────────────────────────────────
//! is ****（`account.key`）。empty etc 
//! can to up ；all is 
//! use that ， after and 。

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::server::core::crypto::{self, SecretScope};
use crate::server::db::Db;

use super::manifest;

/// `kv` （， `db::schema::RESERVED_KV_KEYS`）
pub(crate) const KV_KEY: &str = "cloudSync";

/// 
pub(crate) const PROVIDER_WEBDAV: &str = "webdav";
pub(crate) const PROVIDER_S3: &str = "s3";

/// defaultdirectory。 and cc-switch `cc-switch-sync` 1 ： to 1 ** not will 
/// use ** ， use not not will up directory down its use file。
pub(crate) const DEFAULT_REMOTE_ROOT: &str = "agent2api-sync";
/// default（config when ）
pub(crate) const DEFAULT_PROFILE: &str = "default";

/// 。8 is ： for ，
/// will 「 + 」 can 。
pub(crate) const MIN_PASSWORD_LEN: usize = 8;

/// WebDAV set（ up that 1 ）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WebDavConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub username: String,
    /// `enc1:` （module header）。empty = set
    #[serde(default)]
    pub password: String,
    /// User-Agent。empty = use default。（ in ）
    /// by User-Agent client， need use 
    #[serde(default)]
    pub user_agent: String,
}

/// S3 set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct S3Config {
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub bucket: String,
    #[serde(default)]
    pub access_key_id: String,
    /// `enc1:` 。empty = set
    #[serde(default)]
    pub secret_access_key: String,
    /// （`minio.example.com:9000`）；empty = AWS 
    #[serde(default)]
    pub endpoint: String,
    /// User-Agent。empty = use default。（ in ）
    /// by User-Agent client， need use 
    #[serde(default)]
    pub user_agent: String,
}

/// （server，）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncStatus {
    /// up success when （ms）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,
    /// up success ：`upload` / `download`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
    /// up success id
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_snapshot_id: Option<String>,
    /// up up when （ use 「 no 」）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_local_hash: Option<String>,
    /// up up when （must）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_key_fingerprint: Option<String>,
    /// up failure Reason
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// failure：`manual` / `auto`。 is （failure
    /// use no ，must；failure has toast）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_source: Option<String>,
}

/// set
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CloudSyncSettings {
    /// current ：`webdav` / `s3`。config when ， not already 
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub webdav: WebDavConfig,
    #[serde(default)]
    pub s3: S3Config,
    /// `enc1:` 。empty = set
    #[serde(default)]
    pub sync_password: String,
    /// total。 when up / down will （ and cc-switch `enabled` ）
    #[serde(default)]
    pub enabled: bool,
    /// after up 
    #[serde(default)]
    pub auto_sync: bool,
    #[serde(default = "default_remote_root")]
    pub remote_root: String,
    #[serde(default = "default_profile")]
    pub profile: String,
    #[serde(default)]
    pub status: SyncStatus,
}

fn default_provider() -> String {
    PROVIDER_WEBDAV.to_string()
}

fn default_remote_root() -> String {
    DEFAULT_REMOTE_ROOT.to_string()
}

fn default_profile() -> String {
    DEFAULT_PROFILE.to_string()
}

impl Default for CloudSyncSettings {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            webdav: WebDavConfig::default(),
            s3: S3Config::default(),
            sync_password: String::new(),
            enabled: false,
            auto_sync: false,
            remote_root: default_remote_root(),
            profile: default_profile(),
            status: SyncStatus::default(),
        }
    }
}

impl CloudSyncSettings {
    /// empty and default（ when use ）
    pub(crate) fn normalize(&mut self) {
        if self.provider != PROVIDER_S3 {
            self.provider = PROVIDER_WEBDAV.to_string();
        }
        self.webdav.base_url = self.webdav.base_url.trim().to_string();
        self.webdav.username = self.webdav.username.trim().to_string();
        self.webdav.user_agent = self.webdav.user_agent.trim().to_string();
        self.s3.region = self.s3.region.trim().to_string();
        self.s3.bucket = self.s3.bucket.trim().to_string();
        self.s3.access_key_id = self.s3.access_key_id.trim().to_string();
        self.s3.endpoint = self.s3.endpoint.trim().to_string();
        self.s3.user_agent = self.s3.user_agent.trim().to_string();
        self.remote_root = self.remote_root.trim().to_string();
        self.profile = self.profile.trim().to_string();
        if self.remote_root.is_empty() {
            self.remote_root = default_remote_root();
        }
        if self.profile.is_empty() {
            self.profile = default_profile();
        }
    }

    /// before ：**current** 。
    ///
    /// 1 config can can 1 （ use ）， not this ——
    /// not will use to ， as `provider` has 1 。
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.sync_password.is_empty() {
            return Err("请设置同步密码（用于加密上云的账号密钥）".to_string());
        }
        self.validate_remote()
    }

    /// 「 can up 」 need （ not ）。
    ///
    /// ── Why down / not can use ────────────────────
    /// new 1 down when ，** no has **（ not in ，
    /// ）。 will 「」 and 「 down 」 in use before 
    /// ， that 1 （`import-key`） need need up file ——
    /// is 。 this path need / 。
    pub(crate) fn validate_remote(&self) -> Result<(), String> {
        manifest::remote_dir_segments(&self.remote_root, &self.profile)?;
        match self.provider.as_str() {
            PROVIDER_S3 => {
                if self.s3.bucket.is_empty() {
                    return Err("S3 存储桶不能为空".to_string());
                }
                if self.s3.region.is_empty() {
                    return Err("S3 区域不能为空（Cloudflare R2 填 auto）".to_string());
                }
                if self.s3.access_key_id.is_empty() {
                    return Err("S3 Access Key ID 不能为空".to_string());
                }
                if self.s3.secret_access_key.is_empty() {
                    return Err("S3 Secret Access Key 不能为空".to_string());
                }
            }
            _ => {
                if self.webdav.base_url.is_empty() {
                    return Err("WebDAV 地址不能为空".to_string());
                }
                if self.webdav.username.is_empty() {
                    return Err("WebDAV 用户名不能为空".to_string());
                }
            }
        }
        Ok(())
    }

    /// ： / Secret to 「 has no has 」， not 。
    pub(crate) fn public_json(&self) -> Value {
        json!({
            "provider": self.provider,
            "webdav": {
                "baseUrl": self.webdav.base_url,
                "username": self.webdav.username,
                "hasPassword": !self.webdav.password.is_empty(),
                "userAgent": self.webdav.user_agent,
            },
            "s3": {
                "region": self.s3.region,
                "bucket": self.s3.bucket,
                "accessKeyId": self.s3.access_key_id,
                "endpoint": self.s3.endpoint,
                "hasSecretKey": !self.s3.secret_access_key.is_empty(),
                "userAgent": self.s3.user_agent,
            },
            "hasSyncPassword": !self.sync_password.is_empty(),
            "enabled": self.enabled,
            "autoSync": self.auto_sync,
            "remoteRoot": self.remote_root,
            "profile": self.profile,
            "status": self.status,
            "remotePath": manifest::remote_dir_display(&self.remote_root, &self.profile),
            "keyFingerprint": crypto::key_fingerprint(SecretScope::Accounts),
        })
    }

    /// （set → `None`）
    pub(crate) fn sync_password_plain(&self) -> Result<Option<String>, String> {
        if self.sync_password.is_empty() {
            return Ok(None);
        }
        let bytes = crypto::open(&self.sync_password, SecretScope::CloudSync)?;
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| "同步密码不是合法 UTF-8 文本".to_string())
    }

    /// WebDAV （set → empty）
    pub(crate) fn webdav_password_plain(&self) -> Result<String, String> {
        if self.webdav.password.is_empty() {
            return Ok(String::new());
        }
        let bytes = crypto::open(&self.webdav.password, SecretScope::CloudSync)?;
        String::from_utf8(bytes).map_err(|_| "WebDAV 密码不是合法 UTF-8 文本".to_string())
    }

    /// S3 Secret （set → empty）
    pub(crate) fn s3_secret_plain(&self) -> Result<String, String> {
        if self.s3.secret_access_key.is_empty() {
            return Ok(String::new());
        }
        let bytes = crypto::open(&self.s3.secret_access_key, SecretScope::CloudSync)?;
        String::from_utf8(bytes).map_err(|_| "S3 Secret Access Key 不是合法 UTF-8 文本".to_string())
    }
}

// ─── ─────────────────────────────────────────────

static DB: OnceLock<Option<Db>> = OnceLock::new();

/// （`bootstrap` use 1 ， and its kv ）
pub(crate) fn install(db: Option<Db>) {
    let _ = DB.set(db);
}

fn database() -> Option<&'static Db> {
    DB.get().and_then(Option::as_ref)
}

// ─── ─────────────────────────────────────────────────────

/// set。 not in / failure / unavailable 1 default ——
/// and `settings::load` to 1 （ not = no ）。
pub(crate) fn load() -> CloudSyncSettings {
    let Some(db) = database() else {
        return CloudSyncSettings::default();
    };
    let text = db
        .with(|conn| {
            conn.query_row(
                "SELECT value FROM kv WHERE key = ?1",
                rusqlite::params![KV_KEY],
                |row| row.get::<_, String>(0),
            )
            .ok()
        })
        .flatten();
    let Some(text) = text else {
        return CloudSyncSettings::default();
    };
    let mut settings: CloudSyncSettings = serde_json::from_str(&text).unwrap_or_default();
    // / emptypath need ，otherwisepath will `/v1/default`
    settings.normalize();
    settings
}

/// （）。return， use in 。
pub(crate) fn save(settings: &CloudSyncSettings) -> Result<(), String> {
    let db = database().ok_or_else(|| "数据库不可用，云同步设置未保存".to_string())?;
    let text = serde_json::to_string(settings)
        .map_err(|error| format!("云同步设置序列化失败: {error}"))?;
    db.with(|conn| {
        conn.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![KV_KEY, text],
        )
        .map(|_| ())
        .map_err(|error| format!("保存云同步设置失败: {error}"))
    })
    .ok_or_else(|| "数据库不可用，云同步设置未保存".to_string())?
}

/// update（success after / failure after use ）。--。
pub(crate) fn update_status(patch: impl FnOnce(&mut SyncStatus)) {
    let mut settings = load();
    patch(&mut settings.status);
    if let Err(error) = save(&settings) {
        // not not this （ and cc-switch best-effort ），
        // must： down 1 set not to 「 up when 」 when need Reason。
        crate::server::logging::log("[CloudSync]", &format!("⚠️ 同步状态写入失败: {error}"));
    }
}

/// request： → 。
///
/// / Secret ** use **：request `hasPassword` as and `password` as empty
/// = use no this 1 ， that ；`password` empty = new （ new ）。
/// this 1 and cc-switch `passwordTouched` 1 ， to ——
/// that is before need not need empty， this is after by 「 has no has new 」， before 1 。
pub(crate) struct SaveInput {
    pub provider: String,
    pub webdav_base_url: String,
    pub webdav_username: String,
    pub webdav_password: String,
    pub webdav_user_agent: String,
    pub s3_region: String,
    pub s3_bucket: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_endpoint: String,
    pub s3_user_agent: String,
    pub sync_password: String,
    pub enabled: bool,
    pub auto_sync: bool,
    pub remote_root: String,
    pub profile: String,
}

/// use 1 request，return after set。
pub(crate) fn apply_save(input: SaveInput) -> Result<CloudSyncSettings, String> {
    let mut settings = load();
    settings.provider = input.provider;
    settings.webdav.base_url = input.webdav_base_url;
    settings.webdav.username = input.webdav_username;
    settings.webdav.user_agent = input.webdav_user_agent;
    // empty = use old ；empty = new （ after ）
    if !input.webdav_password.is_empty() {
        settings.webdav.password = crypto::seal(
            input.webdav_password.as_bytes(),
            SecretScope::CloudSync,
        )?;
    }
    settings.s3.region = input.s3_region;
    settings.s3.bucket = input.s3_bucket;
    settings.s3.access_key_id = input.s3_access_key_id;
    if !input.s3_secret_access_key.is_empty() {
        settings.s3.secret_access_key = crypto::seal(
            input.s3_secret_access_key.as_bytes(),
            SecretScope::CloudSync,
        )?;
    }
    settings.s3.endpoint = input.s3_endpoint;
    settings.s3.user_agent = input.s3_user_agent;
    if !input.sync_password.is_empty() {
        if input.sync_password.chars().count() < MIN_PASSWORD_LEN {
            return Err(format!("同步密码至少需要 {MIN_PASSWORD_LEN} 位"));
        }
        settings.sync_password =
            crypto::seal(input.sync_password.as_bytes(), SecretScope::CloudSync)?;
    }
    settings.enabled = input.enabled;
    settings.auto_sync = input.auto_sync && input.enabled;
    settings.remote_root = input.remote_root;
    settings.profile = input.profile;
    settings.normalize();
    settings.validate()?;
    save(&settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_are_stable() {
        let settings = CloudSyncSettings::default();
        assert_eq!(settings.remote_root, "agent2api-sync");
        assert_eq!(settings.profile, "default");
        assert_eq!(settings.provider, PROVIDER_WEBDAV);
    }

    #[test]
    fn normalize_restores_blank_paths() {
        let mut settings = CloudSyncSettings {
            remote_root: "  ".to_string(),
            profile: "".to_string(),
            ..CloudSyncSettings::default()
        };
        settings.normalize();
        assert_eq!(settings.remote_root, DEFAULT_REMOTE_ROOT);
        assert_eq!(settings.profile, DEFAULT_PROFILE);
    }

    #[test]
    fn validate_rejects_blank_sync_password() {
        let mut settings = CloudSyncSettings {
            sync_password: String::new(),
            webdav: WebDavConfig {
                base_url: "https://dav.example.com/dav".to_string(),
                username: "u".to_string(),
                password: "x".to_string(),
                user_agent: String::new(),
            },
            ..CloudSyncSettings::default()
        };
        settings.normalize();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn validate_only_checks_active_provider() {
        let mut settings = CloudSyncSettings {
            provider: PROVIDER_WEBDAV.to_string(),
            sync_password: "sealed".to_string(),
            webdav: WebDavConfig {
                base_url: "https://dav.example.com/dav".to_string(),
                username: "u".to_string(),
                password: "x".to_string(),
                user_agent: String::new(),
            },
            ..CloudSyncSettings::default()
        };
        settings.normalize();
        // S3 that empty not this WebDAV 
        assert!(settings.validate().is_ok());
    }
}
