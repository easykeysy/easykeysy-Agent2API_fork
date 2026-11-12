//! ： to WebDAV / S3 ， or from 。
//!
//! ```text
//! GET /api/cloud-sync set + 
//! PUT /api/cloud-sync set
//! POST /api/cloud-sync/test （directory）
//! GET /api/cloud-sync/remote （ use ）
//! POST /api/cloud-sync/upload up 
//! POST /api/cloud-sync/download down and use 
//! POST /api/cloud-sync/import-key use and 
//! ```
//!
//! ── Why this file ────────────────────────────────────
//! `settings` set and （kv `cloudSync`）
//! `manifest` 「 this 1 is 」 and 
//! `snapshot` / （ 1 ）
//! `keyfile` → 
//! `webdav` / `s3` 
//! `transport` （ not ）
//! `auto_sync` + 
//!
//! ── Why 1 ────────────────────────────────────
//! up and down will 1 ， and after ——
//! use down by can can to 「 is 、 is old 」 this 
//! 。 and cc-switch `sync_mutex` 1 to ：all use 1 ，。
//! is `tokio::sync::Mutex`（ await has ）， not can std 。
//!
//! ── down will ────────────────────────────────────
//! and 1 （ `db::backup` module header），：
//! · `config::reload()` —— configmust，otherwise is old config；
//! · `usage_records::reload_facts()` —— is ；
//! · `crypto::reset_cache(Accounts)` —— **onlyfile **（ `keyfile`）；
//! · 、set、** not in **， not need need 。

pub(crate) mod auto_sync;
pub(crate) mod keyfile;
pub(crate) mod manifest;
pub(crate) mod s3;
pub(crate) mod settings;
pub(crate) mod snapshot;
pub(crate) mod transport;
pub(crate) mod webdav;

use std::sync::OnceLock;

use serde_json::{json, Value};

use crate::server::core::crypto::{self, SecretScope};
use crate::server::db::Db;
use crate::server::logging;

use manifest::{MAX_ARTIFACT_BYTES, REMOTE_KEY_FILE, REMOTE_MANIFEST, REMOTE_SNAPSHOT};
use transport::Transport;

/// （`bootstrap` use 1 ；set in `settings` ）。
pub(crate) fn install(db: Option<Db>) {
    settings::install(db);
}

/// （ up / down / use ）
pub(crate) fn sync_lock() -> &'static tokio::sync::Mutex<()> {
    static LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
}

/// sha256 → （manifest / / use 1 ）
pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    let digest = Sha256::digest(bytes);
    let mut text = String::with_capacity(64);
    for byte in digest.iter() {
        use std::fmt::Write as _;
        let _ = write!(text, "{byte:02x}");
    }
    text
}

// ─── set ─────────────────────────────────────────────────────

/// GET /api/cloud-sync 
pub(crate) fn settings_json() -> Value {
    settings::load().public_json()
}

/// set。`input` empty = use old （ `settings::apply_save`）。
pub(crate) fn save_settings(input: settings::SaveInput) -> Result<Value, String> {
    let saved = settings::apply_save(input)?;
    logging::log(
        "[CloudSync]",
        &format!(
            "云同步设置已保存：{}，远端 {}{}",
            if saved.provider == settings::PROVIDER_S3 {
                "S3 兼容存储"
            } else {
                "WebDAV"
            },
            manifest::remote_dir_display(&saved.remote_root, &saved.profile),
            if saved.enabled { "（已启用）" } else { "（未启用）" }
        ),
    );
    Ok(saved.public_json())
}

/// set and need 「 already enabled」。total —— and cc-switch ：
/// 「 this not and 」， not this as use 1 down by 。
///
/// （`validate_remote`）： in ** down / **path up as empty，
/// by `settings::validate_remote` （ new 1 down when no has ）。
/// need need is up （ `upload_locked`）， that 。
fn require_enabled() -> Result<settings::CloudSyncSettings, String> {
    let settings = settings::load();
    if !settings.enabled {
        return Err("云同步未启用，请先在设置里打开开关".to_string());
    }
    settings.validate_remote()?;
    Ok(settings)
}

// ─── ─────────────────────────────────────────────────

/// ：** not **total（ no is config），
/// need current 。
pub(crate) async fn test_connection(input: settings::SaveInput) -> Result<Value, String> {
    // use 「current already + this 」 1 when set：
    // use no when ， this is new 。
    let mut probe = settings::load();
    probe.provider = input.provider;
    probe.webdav.base_url = input.webdav_base_url;
    probe.webdav.username = input.webdav_username;
    probe.webdav.user_agent = input.webdav_user_agent;
    if !input.webdav_password.is_empty() {
        probe.webdav.password = crypto::seal(
            input.webdav_password.as_bytes(),
            SecretScope::CloudSync,
        )?;
    }
    probe.s3.region = input.s3_region;
    probe.s3.bucket = input.s3_bucket;
    probe.s3.access_key_id = input.s3_access_key_id;
    if !input.s3_secret_access_key.is_empty() {
        probe.s3.secret_access_key = crypto::seal(
            input.s3_secret_access_key.as_bytes(),
            SecretScope::CloudSync,
        )?;
    }
    probe.s3.endpoint = input.s3_endpoint;
    probe.s3.user_agent = input.s3_user_agent;
    probe.remote_root = input.remote_root;
    probe.profile = input.profile;
    probe.normalize();

    // current ： when 1 ，「set」
    // this 1 （ not need need ， that used for）。
    let mut validate_target = probe.clone();
    if validate_target.sync_password.is_empty() {
        validate_target.sync_password = "placeholder".to_string();
    }
    validate_target.validate()?;

    let transport = Transport::from_settings(&probe)?;
    transport.test_connection().await?;
    Ok(json!({
        "success": true,
        "remotePath": transport.display_path(),
    }))
}

// ─── ─────────────────────────────────────────────────

/// need （ not ）
pub(crate) async fn remote_info() -> Result<Value, String> {
    let settings = settings::load();
    // need / （ new no has ， validate_remote）
    settings.validate_remote()?;
    let transport = Transport::from_settings(&settings)?;
    let Some((bytes, manifest)) = transport.get_manifest().await? else {
        return Ok(json!({
            "empty": true,
            "remotePath": transport.display_path(),
        }));
    };
    let compatible = manifest::validate_compat(&manifest).is_ok();
    let incompatible_reason = manifest::validate_compat(&manifest).err();
    let local_fingerprint = crypto::key_fingerprint(SecretScope::Accounts);
    Ok(json!({
        "empty": false,
        "remotePath": transport.display_path(),
        "manifestBytes": bytes.len(),
        "format": manifest.format,
        "version": manifest.version,
        "appVersion": manifest.app_version,
        "schemaVersion": manifest.schema_version,
        "deviceName": manifest.device_name,
        "createdAt": manifest.created_at,
        "snapshotId": manifest.snapshot_id,
        "artifacts": manifest.artifacts.keys().collect::<Vec<_>>(),
        "compatible": compatible,
        "incompatibleReason": incompatible_reason,
        "keyFingerprint": manifest.key_fingerprint,
        "localKeyFingerprint": local_fingerprint,
        // 、 and and not 1 （ or no has ）→
        // down before must
        "keyRequired": manifest.key_fingerprint.is_some()
            && manifest.key_fingerprint != local_fingerprint,
        "hasKeyArtifact": manifest.artifacts.contains_key(REMOTE_KEY_FILE),
    }))
}

// ─── up ─────────────────────────────────────────────────────

/// up ： → → artifact → after manifest。
pub(crate) async fn upload(db: &Db) -> Result<Value, String> {
    let _guard = sync_lock().lock().await;
    let settings = require_enabled()?;
    upload_locked(db, &settings).await
}

/// up （ use already ）。 this ，。
pub(crate) async fn upload_locked(
    db: &Db,
    settings: &settings::CloudSyncSettings,
) -> Result<Value, String> {
    let transport = Transport::from_settings(settings)?;
    transport.test_connection().await?;

    let snapshot = snapshot::build(db)?;
    let snapshot_bytes = serde_json::to_vec(&snapshot)
        .map_err(|error| format!("序列化同步快照失败: {error}"))?;
    if snapshot_bytes.len() as u64 > MAX_ARTIFACT_BYTES {
        return Err(format!(
            "本机业务数据超过同步上限（{} 字节 > {} 字节），请先清理签到 / 余额记录",
            snapshot_bytes.len(),
            MAX_ARTIFACT_BYTES
        ));
    }

    // ： no has （ from ） when not 
    let key_bytes = crypto::existing_key_bytes(SecretScope::Accounts);
    let password = settings.sync_password_plain()?.ok_or_else(|| {
        "未设置同步密码，无法加密上云的账号密钥".to_string()
    })?;
    let key_envelope = match key_bytes.as_ref() {
        Some(bytes) => Some(keyfile::seal_key(bytes, &password)?),
        None => None,
    };
    let key_fingerprint = key_bytes
        .as_ref()
        .map(|bytes| crypto::fingerprint_of(bytes));

    let manifest = snapshot::build_manifest(
        &snapshot_bytes,
        key_envelope.as_deref(),
        key_fingerprint.clone(),
    );
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("序列化 manifest 失败: {error}"))?;

    // ：artifact 、manifest after （ manifest module header）
    transport
        .put(REMOTE_SNAPSHOT, snapshot_bytes.clone(), "application/json")
        .await?;
    if let Some(envelope) = key_envelope {
        transport
            .put(REMOTE_KEY_FILE, envelope, "application/json")
            .await?;
    }
    transport
        .put(REMOTE_MANIFEST, manifest_bytes, "application/json")
        .await?;

    let content_hash = snapshot.content_hash();
    let snapshot_id = manifest.snapshot_id.clone();
    settings::update_status(|status| {
        status.last_sync_at = Some(logging::now_ms());
        status.last_action = Some("upload".to_string());
        status.last_snapshot_id = Some(snapshot_id.clone());
        status.last_local_hash = Some(content_hash.clone());
        status.last_key_fingerprint = key_fingerprint.clone();
        status.last_error = None;
        status.last_error_source = None;
    });
    logging::log(
        "[CloudSync]",
        &format!(
            "☁️ 云同步上传完成：{}（{} 字节，账号 {} 条）",
            transport.display_path(),
            snapshot_bytes.len(),
            snapshot.tables.get("accounts").map(Vec::len).unwrap_or(0)
        ),
    );

    Ok(json!({
        "status": "uploaded",
        "remotePath": transport.display_path(),
        "bytes": snapshot_bytes.len(),
        "snapshotId": snapshot_id,
        "rows": snapshot.row_counts(),
    }))
}

// ─── down ─────────────────────────────────────────────────────

/// down ： manifest → → → → use → 。
pub(crate) async fn download(db: &Db, config_dir: &std::path::Path) -> Result<Value, String> {
    let _guard = sync_lock().lock().await;
    let settings = require_enabled()?;
    download_locked(db, config_dir, &settings).await
}

/// down （ use already ）。
///
/// down use `auto_sync` 「 use + 」 this 1 ： will 
/// ， if not ， will down 1 （ 1 ， and 
/// if is ， this 1 「 down 」「 」）。
pub(crate) async fn download_locked(
    db: &Db,
    config_dir: &std::path::Path,
    settings: &settings::CloudSyncSettings,
) -> Result<Value, String> {
    let transport = Transport::from_settings(settings)?;
    let Some((manifest_bytes, manifest)) = transport.get_manifest().await? else {
        return Err("远端还没有可下载的同步数据".to_string());
    };
    manifest::validate_compat(&manifest)?;

    // check：、 and and not 1 when must import-key。
    // this ** not ** use —— that is 「 up when use 」，
    // this can can 1 （ use not ）。 use ，failure
    // （「 not 」 not is 「 not 」）。
    let local_fingerprint = crypto::key_fingerprint(SecretScope::Accounts);
    if manifest.key_fingerprint.is_some() && manifest.key_fingerprint != local_fingerprint {
        return Err(KEY_REQUIRED.to_string());
    }

    let snapshot_bytes = transport
        .get_artifact(REMOTE_SNAPSHOT, &manifest.artifacts)
        .await?;
    let snapshot: snapshot::Snapshot = serde_json::from_slice(&snapshot_bytes)
        .map_err(|error| format!("远端 snapshot.json 解析失败: {error}"))?;

    // ： use **current** 1 to backups/
    let safety = {
        let current = snapshot::build(db)?;
        snapshot::safety_backup(&current, config_dir)?
    };

    let applied = {
        let _suppress = auto_sync::SuppressionGuard::new();
        let applied = snapshot::apply(db, &snapshot)?;
        // config： down is kv， not is old config
        crate::server::config::reload();
        // ：（ can can 1 ）
        crate::server::core::usage_records::reload_facts();
        crate::server::core::limiter::sources::reload();
        crate::server::core::limiter::counter::invalidate();
        applied
    };

    let manifest_hash = sha256_hex(&manifest_bytes);
    let snapshot_id = manifest.snapshot_id.clone();
    let device = manifest.device_name.clone();
    let created_at = manifest.created_at.clone();
    settings::update_status(|status| {
        status.last_sync_at = Some(logging::now_ms());
        status.last_action = Some("download".to_string());
        status.last_snapshot_id = Some(snapshot_id.clone());
        status.last_local_hash = Some(manifest_hash.clone());
        status.last_key_fingerprint = manifest.key_fingerprint.clone();
        status.last_error = None;
        status.last_error_source = None;
    });
    logging::log(
        "[CloudSync]",
        &format!(
            "☁️ 云同步下载完成：来自 {}（{}），替换前数据已备份到 backups/{}",
            if device.is_empty() { "未知设备" } else { &device },
            if created_at.is_empty() { "未知时间" } else { &created_at },
            safety
        ),
    );

    Ok(json!({
        "status": "downloaded",
        "remotePath": transport.display_path(),
        "deviceName": device,
        "createdAt": created_at,
        "snapshotId": snapshot_id,
        "applied": applied,
        "safetyBackup": safety,
    }))
}

/// down need need when return **error**。
///
/// use not is error： for is `Result<_, String>`
/// （ and its API 1 ）， need by 「」。 when is 
/// error， use down when can 。
pub(crate) const KEY_REQUIRED: &str = "远端快照使用了另一把账号加密密钥，请先输入同步密码导入密钥";

/// use and to 。
pub(crate) async fn import_key(password: &str) -> Result<Value, String> {
    if password.is_empty() {
        return Err("同步密码不能为空".to_string());
    }
    let _guard = sync_lock().lock().await;
    let settings = settings::load();
    // ： this 1 use is 「 new no has 」 when 
    settings.validate_remote()?;
    let transport = Transport::from_settings(&settings)?;
    let Some((_, manifest)) = transport.get_manifest().await? else {
        return Err("远端还没有可下载的同步数据".to_string());
    };
    if !manifest.artifacts.contains_key(REMOTE_KEY_FILE) {
        return Err("远端这份快照没有携带账号密钥（上传时本机还没有保存过账号）".to_string());
    }
    let envelope = transport
        .get_artifact(REMOTE_KEY_FILE, &manifest.artifacts)
        .await?;
    let key = keyfile::open_key(&envelope, password)?;
    let fingerprint = crypto::fingerprint_of(&key);
    if manifest.key_fingerprint.as_deref() != Some(fingerprint.as_str()) {
        return Err(
            "解出的密钥与远端 manifest 记录的指纹不一致，文件可能已损坏，请重新上传".to_string(),
        );
    }
    let backup = keyfile::install_key(&key)?;
    logging::log(
        "[CloudSync]",
        &format!(
            "🔑 已从云同步导入账号密钥（指纹 {fingerprint}{}）",
            backup
                .as_ref()
                .map(|name| format!("，原密钥已备份为 {name}"))
                .unwrap_or_default()
        ),
    );
    Ok(json!({
        "status": "imported",
        "keyFingerprint": fingerprint,
        "backup": backup,
    }))
}

/// 1 failure（ / ）。 + 。
///
/// use `warn` not is `error`：failure can （、），
/// error will error， 。
pub(crate) fn record_failure(error: &str, source: &str) {
    settings::update_status(|status| {
        status.last_error = Some(error.to_string());
        status.last_error_source = Some(source.to_string());
    });
    logging::log_with_level(
        "[CloudSync]",
        &format!(
            "⚠️ 云同步{}失败：{error}",
            if source == "auto" { "（自动）" } else { "" }
        ),
        "warn",
    );
}
