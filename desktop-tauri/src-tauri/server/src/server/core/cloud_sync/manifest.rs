//! manifest：「 this 1 is 」 and 。
//!
//! ── Why artifact、 after manifest ─────────────────────
//! manifest is 「 this up already 」 1 。、 ， in 
//! will down 1 to manifest —— down 1 down size / sha256 when failure，
//! use to is 「」 not is 「 up no 」。（、manifest
//! after ） when ， will to ** up 1 **，。
//! and cc-switch `webdav_sync::upload` 1 。
//!
//! ── Why format / version need ─────────────────────
//! `format` 「 this is not is can 」（directory can can use ）；`version`
//! 「 is 」。 1 ， will not 「 old 」 and 
//! 「 file」—— before can ， after can 「 this not is directory」。
//!
//! ── and schema ──────────────────────────────────
//! `schemaVersion` is ****（`db::schema::SCHEMA_VERSION`）。 →
//! （ old for / ）； → ， use when by 「 and current
//! 」， default。 this and 1 to （ not ）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::server::db::schema::SCHEMA_VERSION;

/// manifest 。 this etc all already 「 not 」，
/// in not when 。
pub(crate) const PROTOCOL_FORMAT: &str = "agent2api-cloud-sync";
/// （directory / ）。 and [`SCHEMA_VERSION`] is ：
/// this 「」， that 「」。
pub(crate) const PROTOCOL_VERSION: u32 = 1;

/// directory down directory：`{remoteRoot}/v1/{profile}/`。
pub(crate) fn protocol_dir() -> String {
    format!("v{PROTOCOL_VERSION}")
}

/// manifest file
pub(crate) const REMOTE_MANIFEST: &str = "manifest.json";
/// file
pub(crate) const REMOTE_SNAPSHOT: &str = "snapshot.json";
/// file
pub(crate) const REMOTE_KEY_FILE: &str = "account.key.enc";

/// manifest limit。，1MB already ； that file
/// not is manifest（or ）， not this 。
pub(crate) const MAX_MANIFEST_BYTES: usize = 1024 * 1024;

/// artifact limit。 / config / to / usage，
/// 256MB ；， 。
pub(crate) const MAX_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;

/// 1 artifact （sha256 + ）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactMeta {
    pub sha256: String,
    pub size: u64,
}

/// manifest。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncManifest {
    pub format: String,
    pub version: u32,
    /// this （used for and ， not and ）
    #[serde(default)]
    pub app_version: String,
    /// schema （ use ，module header）
    pub schema_version: i64,
    /// this （ down 「 this is 」）
    #[serde(default)]
    pub device_name: String,
    /// when （RFC3339）
    #[serde(default)]
    pub created_at: String,
    /// ：artifact need ， use 「 is not is 1 」
    pub snapshot_id: String,
    /// ；`None` = up when no has `account.key`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_fingerprint: Option<String>,
    /// file → （** this up ** file）
    pub artifacts: BTreeMap<String, ArtifactMeta>,
}

impl SyncManifest {
    /// this manifest is not is can 。
    pub(crate) fn is_ours(&self) -> bool {
        self.format == PROTOCOL_FORMAT
    }
}

/// ： need can down ， need to ** can ** by 。
pub(crate) fn validate_compat(manifest: &SyncManifest) -> Result<(), String> {
    if !manifest.is_ours() {
        return Err(format!(
            "远端 manifest 不是本功能生成的（format = {}）。请确认「远程根目录 / 档案」填的是云同步专用的位置。",
            manifest.format
        ));
    }
    if manifest.version != PROTOCOL_VERSION {
        return Err(format!(
            "远端同步数据来自不兼容的协议版本 v{}（本机 v{PROTOCOL_VERSION}），请升级应用后重试。",
            manifest.version
        ));
    }
    if manifest.schema_version > SCHEMA_VERSION {
        return Err(format!(
            "远端数据来自更新版本的应用（库版本 {} > 当前 {SCHEMA_VERSION}），请先升级本应用再下载。",
            manifest.schema_version
        ));
    }
    Ok(())
}

/// from artifact 1 id。
///
/// `BTreeMap` by ， 1 to 1 id —— 
/// 「 this is not is 」。
pub(crate) fn snapshot_id_of(artifacts: &BTreeMap<String, ArtifactMeta>) -> String {
    let parts: Vec<String> = artifacts
        .iter()
        .map(|(name, meta)| format!("{name}:{}", meta.sha256))
        .collect();
    crate::server::core::cloud_sync::sha256_hex(parts.join("|").as_bytes())
}

/// down and manifest 1 。
///
/// ： not when can to （ / ）， not 1 
/// MB sha256。
pub(crate) fn verify_artifact(
    bytes: &[u8],
    name: &str,
    meta: &ArtifactMeta,
) -> Result<(), String> {
    if bytes.len() as u64 != meta.size {
        return Err(format!(
            "远端 {name} 大小不符（应为 {} 字节，实际 {} 字节），文件可能被截断，请重新上传。",
            meta.size,
            bytes.len()
        ));
    }
    let actual = crate::server::core::cloud_sync::sha256_hex(bytes);
    if actual != meta.sha256 {
        return Err(format!(
            "远端 {name} 校验失败（sha256 不一致），文件可能已损坏，请重新上传。"
        ));
    }
    Ok(())
}

/// use directory / path。
///
/// empty and `.` / `..`：path by use ，`..` will directory
/// （WebDAV up etc use directoryfile）。 this and cc-switch 
/// `path_segments` 1 ， is `..`。
pub(crate) fn sanitize_segments(raw: &str) -> Result<Vec<String>, String> {
    let mut segments = Vec::new();
    for part in raw.trim().trim_matches('/').split('/') {
        let part = part.trim();
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err("远程路径不能包含 .. 段".to_string());
        }
        if part.contains('\\') {
            return Err("远程路径请用 / 分隔".to_string());
        }
        segments.push(part.to_string());
    }
    Ok(segments)
}

/// directory path：`{remoteRoot…}/v1/{profile…}`。
pub(crate) fn remote_dir_segments(remote_root: &str, profile: &str) -> Result<Vec<String>, String> {
    let mut segments = sanitize_segments(remote_root)?;
    segments.push(protocol_dir());
    segments.extend(sanitize_segments(profile)?);
    Ok(segments)
}

/// directory （`/root/v1/default`）
pub(crate) fn remote_dir_display(remote_root: &str, profile: &str) -> String {
    let segments = remote_dir_segments(remote_root, profile).unwrap_or_default();
    if segments.is_empty() {
        return "/".to_string();
    }
    format!("/{}", segments.join("/"))
}

/// ：Windows COMPUTERNAME / Unix HOSTNAME， after hostname 。
///
/// used for manifest ， not to to `Unknown Device` —— not this as not to 
/// up failure。
pub(crate) fn detect_device_name() -> String {
    for key in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(value) = std::env::var(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.chars().take(64).collect();
            }
        }
    }
    "Unknown Device".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_segments_rejects_parent_dir() {
        assert!(sanitize_segments("a/../b").is_err());
        assert_eq!(
            sanitize_segments("/a//b/").unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn remote_dir_segments_appends_protocol_and_profile() {
        let segments = remote_dir_segments("agent2api-sync", "default").unwrap();
        assert_eq!(
            segments,
            vec![
                "agent2api-sync".to_string(),
                format!("v{PROTOCOL_VERSION}"),
                "default".to_string()
            ]
        );
    }

    #[test]
    fn validate_compat_rejects_newer_schema() {
        let manifest = SyncManifest {
            format: PROTOCOL_FORMAT.to_string(),
            version: PROTOCOL_VERSION,
            app_version: "3.1.0".to_string(),
            schema_version: SCHEMA_VERSION + 1,
            device_name: "x".to_string(),
            created_at: "2026-10-19T00:00:00Z".to_string(),
            snapshot_id: "id".to_string(),
            key_fingerprint: None,
            artifacts: BTreeMap::new(),
        };
        assert!(validate_compat(&manifest).is_err());
    }

    #[test]
    fn snapshot_id_is_stable_and_order_independent() {
        let mut a = BTreeMap::new();
        a.insert(
            "snapshot.json".to_string(),
            ArtifactMeta { sha256: "aa".to_string(), size: 1 },
        );
        let mut b = BTreeMap::new();
        b.insert(
            "snapshot.json".to_string(),
            ArtifactMeta { sha256: "aa".to_string(), size: 1 },
        );
        assert_eq!(snapshot_id_of(&a), snapshot_id_of(&b));
    }
}
