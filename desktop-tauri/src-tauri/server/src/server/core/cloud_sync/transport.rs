//! ： 「 1 file / 1 file」 from 「WebDAV is S3」。
//!
//! （`super::mod` up / down / ） [`Transport`] ，
//! （e.g. SMB / FTP） need 1 trait， not 。
//! and cc-switch `sync_protocol` and `webdav` / `s3` by 1 。

use super::manifest::{self, ArtifactMeta, MAX_ARTIFACT_BYTES, MAX_MANIFEST_BYTES};
use super::{s3, settings, webdav};

/// 1 need use to 
pub(crate) enum Transport {
    WebDav {
        base_url: String,
        auth: webdav::WebDavAuth,
        /// directory（`remoteRoot/v1/profile`）， need 
        segments: Vec<String>,
        /// User-Agent；empty = default。 by client
        user_agent: String,
    },
    S3 {
        credentials: s3::S3Credentials,
        /// directory（ for before ）：`remoteRoot/v1/profile`
        prefix: String,
    },
}

impl Transport {
    /// from set（ in this ， in ）
    pub(crate) fn from_settings(settings: &settings::CloudSyncSettings) -> Result<Self, String> {
        let segments = manifest::remote_dir_segments(&settings.remote_root, &settings.profile)?;
        match settings.provider.as_str() {
            settings::PROVIDER_S3 => Ok(Transport::S3 {
                credentials: s3::S3Credentials {
                    access_key_id: settings.s3.access_key_id.clone(),
                    secret_access_key: settings.s3_secret_plain()?,
                    region: settings.s3.region.clone(),
                    bucket: settings.s3.bucket.clone(),
                    endpoint: settings.s3.endpoint.clone(),
                    user_agent: settings.s3.user_agent.clone(),
                },
                prefix: segments.join("/"),
            }),
            _ => Ok(Transport::WebDav {
                base_url: settings.webdav.base_url.clone(),
                auth: webdav::auth_from_credentials(
                    &settings.webdav.username,
                    &settings.webdav_password_plain()?,
                ),
                segments,
                user_agent: settings.webdav.user_agent.clone(),
            }),
        }
    }

    /// directory path
    pub(crate) fn display_path(&self) -> String {
        match self {
            Transport::WebDav { segments, .. } => format!("/{}", segments.join("/")),
            Transport::S3 { credentials, prefix } => {
                format!("{}/{prefix}", credentials.bucket)
            }
        }
    }

    /// （ directory， 1 up no ）
    pub(crate) async fn test_connection(&self) -> Result<(), String> {
        match self {
            Transport::WebDav {
                base_url,
                auth,
                segments,
                user_agent,
            } => {
                webdav::test_connection(base_url, auth, user_agent).await?;
                webdav::ensure_remote_directories(base_url, segments, auth, user_agent).await
            }
            Transport::S3 { credentials, .. } => s3::test_connection(credentials).await,
        }
    }

    /// 1 file（ manifest after ， `manifest` module header）
    pub(crate) async fn put(
        &self,
        name: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        match self {
            Transport::WebDav {
                base_url,
                auth,
                segments,
                user_agent,
            } => {
                let mut file_segments = segments.clone();
                file_segments.extend(manifest::sanitize_segments(name)?);
                let url = webdav::build_remote_url(base_url, &file_segments)?;
                webdav::put_bytes(&url, auth, bytes, content_type, user_agent).await
            }
            Transport::S3 { credentials, prefix } => {
                let key = format!("{prefix}/{}", name.trim_start_matches('/'));
                s3::put_object(credentials, &key, bytes, content_type).await
            }
        }
    }

    /// 1 file； not in return `None`（「 no has 」 is ）
    pub(crate) async fn get(
        &self,
        name: &str,
        max_bytes: usize,
    ) -> Result<Option<Vec<u8>>, String> {
        match self {
            Transport::WebDav {
                base_url,
                auth,
                segments,
                user_agent,
            } => {
                let mut file_segments = segments.clone();
                file_segments.extend(manifest::sanitize_segments(name)?);
                let url = webdav::build_remote_url(base_url, &file_segments)?;
                webdav::get_bytes(&url, auth, max_bytes, user_agent).await
            }
            Transport::S3 { credentials, prefix } => {
                let key = format!("{prefix}/{}", name.trim_start_matches('/'));
                s3::get_object(credentials, &key, max_bytes).await
            }
        }
    }

    /// manifest（limit）
    pub(crate) async fn get_manifest(
        &self,
    ) -> Result<Option<(Vec<u8>, manifest::SyncManifest)>, String> {
        let Some(bytes) = self.get(manifest::REMOTE_MANIFEST, MAX_MANIFEST_BYTES).await? else {
            return Ok(None);
        };
        let parsed: manifest::SyncManifest = serde_json::from_slice(&bytes).map_err(|error| {
            format!("远端 manifest.json 不是合法 JSON（可能不是本功能生成的文件）: {error}")
        })?;
        Ok(Some((bytes, parsed)))
    }

    /// 1 artifact and by manifest （size + sha256）
    pub(crate) async fn get_artifact(
        &self,
        name: &str,
        artifacts: &std::collections::BTreeMap<String, ArtifactMeta>,
    ) -> Result<Vec<u8>, String> {
        let meta = artifacts.get(name).ok_or_else(|| {
            format!("远端 manifest 里没有 {name}（这份快照不完整，请重新上传）")
        })?;
        if meta.size > MAX_ARTIFACT_BYTES {
            return Err(format!(
                "远端 {name} 超过上限（{} 字节 > {MAX_ARTIFACT_BYTES} 字节）",
                meta.size
            ));
        }
        let bytes = self
            .get(name, MAX_ARTIFACT_BYTES as usize)
            .await?
            .ok_or_else(|| format!("远端缺少 {name}（manifest 说有、实际读不到）"))?;
        manifest::verify_artifact(&bytes, name, meta)?;
        Ok(bytes)
    }
}
