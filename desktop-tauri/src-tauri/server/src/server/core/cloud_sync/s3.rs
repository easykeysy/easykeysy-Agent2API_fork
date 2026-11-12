//! S3 ：PUT / GET / HEAD + AWS Signature V4 。
//!
//! ── Why SigV4 ────────────────────────────────────────
//! has （PUT / GET / HEAD）， `aws-sdk-s3` will 1 SDK and 
//! when ；「 not add」（ `server/Cargo.toml`）。
//! `hmac` / `sha2` already in （ALTCHA and ）， is 60 。
//!
//! ── URL ────────────────────────────────────────────────
//! · AWS （endpoint as empty or `amazonaws.com`）→ 
//!     `https://{bucket}.s3.{region}.amazonaws.com/{key}`
//! · （MinIO / R2 / OSS / COS / OBS）→ path-style
//!     `{scheme}://{host}/{bucket}/{key}`
//! R2 path-style（`https://{account}.r2.cloudflarestorage.com/{bucket}/{key}`），
//! and to S3 API 1 。
//!
//! ── need need ──────────────────────────────────────
//! and is `host` / `x-amz-date` / `x-amz-content-sha256`（+ optional 
//! `content-type`）。** not ** `content-length` ：reqwest will in after 1 
//! ， when not to ， for not up 。S3 ， need 
//! `SignedHeaders` and `Authorization` 1 can 。
//!
//! ── no has scheme when default https ────────────────────────────
//! and cc-switch `split_scheme_host` ： use `minio.local:9000` when by 
//! https ， `http://minio.local:9000` （ MinIO use after ）。

use std::time::Duration;

use reqwest::StatusCode;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TRANSFER_TIMEOUT_SECS: u64 = 300;

/// S3 （ after ）
pub(crate) struct S3Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket: String,
    /// ；empty = AWS 
    pub endpoint: String,
    /// User-Agent；empty = use reqwest default。** not and **， can 
    pub user_agent: String,
}

/// ── Why need can User-Agent ──────────────────────────────
/// S3 （ in ） to 「 use 」 up ， and in 
/// server by User-Agent ；UA not this use 401/403。 when 1 
/// is client UA that use （ `rclone/v1.67.0`）。UA not in SigV4
/// （module header「 need need 」）， not 。
fn client(credentials: &S3Credentials) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    // in use ：empty will 「 not UA」， not reqwest default
    let user_agent = credentials.user_agent.trim();
    if !user_agent.is_empty() {
        builder = builder.user_agent(user_agent.to_string());
    }
    builder
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {error}"))
}

fn is_aws_endpoint(endpoint: &str) -> bool {
    let trimmed = endpoint.trim();
    trimmed.is_empty() || trimmed.contains("amazonaws.com")
}

/// `(scheme, host[:port])`
fn split_scheme_host(endpoint: &str) -> (&str, &str) {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if let Some(rest) = trimmed.strip_prefix("http://") {
        ("http", rest)
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        ("https", rest)
    } else {
        ("https", trimmed)
    }
}

/// for URL（module header）
fn object_url(credentials: &S3Credentials, key: &str) -> String {
    let key = key.trim_start_matches('/');
    if is_aws_endpoint(&credentials.endpoint) {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            credentials.bucket, credentials.region, key
        )
    } else {
        let (scheme, host) = split_scheme_host(&credentials.endpoint);
        format!("{scheme}://{host}/{}/{}", credentials.bucket, key)
    }
}

/// URL（ use HEAD bucket）
fn bucket_url(credentials: &S3Credentials) -> String {
    if is_aws_endpoint(&credentials.endpoint) {
        format!(
            "https://{}.s3.{}.amazonaws.com/",
            credentials.bucket, credentials.region
        )
    } else {
        let (scheme, host) = split_scheme_host(&credentials.endpoint);
        format!("{scheme}://{host}/{}/", credentials.bucket)
    }
}

// ─── ─────────────────────────────────────────────────────

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC 接受任意长度密钥");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// AWS SigV4 （ RFC 3986 unreserved）
fn uri_encode(input: &str, encode_slash: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b'/' if !encode_slash => out.push('/'),
            _ => {
                use std::fmt::Write as _;
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

/// to 1 request，return need need up 。
///
/// return is `(header_name, header_value)` ， by use `.header()` up 。
/// not `HeaderMap`：reqwest builder is ， in 1 
/// `host`（reqwest will host， use is that ）。
fn sign(
    method: &str,
    url: &url::Url,
    body_hash: &str,
    content_type: Option<&str>,
    credentials: &S3Credentials,
    now: chrono::DateTime<chrono::Utc>,
) -> Vec<(String, String)> {
    let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();
    let datestamp = now.format("%Y%m%d").to_string();

    let host = match url.port() {
        Some(port) => format!("{}:{}", url.host_str().unwrap_or_default(), port),
        None => url.host_str().unwrap_or_default().to_string(),
    };

    // and ： by （SigV4 need ）
    let mut headers: Vec<(String, String)> = vec![
        ("host".to_string(), host),
        ("x-amz-content-sha256".to_string(), body_hash.to_string()),
        ("x-amz-date".to_string(), timestamp.clone()),
    ];
    if let Some(value) = content_type {
        headers.push(("content-type".to_string(), value.to_string()));
    }
    headers.sort_by(|a, b| a.0.cmp(&b.0));

    let canonical_headers: String = headers
        .iter()
        .map(|(name, value)| format!("{name}:{}\n", value.trim()))
        .collect();
    let signed_headers = headers
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(";");

    let canonical_uri = {
        let path = url.path();
        if path.is_empty() {
            "/"
        } else {
            path
        }
    };
    let mut query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect();
    query_pairs.sort();
    let canonical_query = query_pairs
        .iter()
        .map(|(key, value)| format!("{}={}", uri_encode(key, true), uri_encode(value, true)))
        .collect::<Vec<_>>()
        .join("&");

    let canonical_request = format!(
        "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{body_hash}"
    );

    let scope = format!("{datestamp}/{}/s3/aws4_request", credentials.region);
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{timestamp}\n{scope}\n{}",
        super::sha256_hex(canonical_request.as_bytes())
    );

    let k_date = hmac_sha256(
        format!("AWS4{}", credentials.secret_access_key).as_bytes(),
        datestamp.as_bytes(),
    );
    let k_region = hmac_sha256(&k_date, credentials.region.as_bytes());
    let k_service = hmac_sha256(&k_region, b"s3");
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hmac_sha256(&k_signing, string_to_sign.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
        credentials.access_key_id
    );

    let mut result = headers;
    result.push(("authorization".to_string(), authorization));
    result
}

/// request（ + need ）
fn signed_headers(
    method: &str,
    url: &str,
    body: &[u8],
    content_type: Option<&str>,
    credentials: &S3Credentials,
) -> Result<Vec<(String, String)>, String> {
    let parsed = url::Url::parse(url).map_err(|error| format!("S3 地址无效: {error}"))?;
    let body_hash = super::sha256_hex(body);
    Ok(sign(
        method,
        &parsed,
        &body_hash,
        content_type,
        credentials,
        chrono::Utc::now(),
    ))
}

fn redact_url(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(parsed) => {
            let mut out = format!("{}://", parsed.scheme());
            if let Some(host) = parsed.host_str() {
                out.push_str(host);
            }
            if let Some(port) = parsed.port() {
                out.push(':');
                out.push_str(&port.to_string());
            }
            out.push_str(parsed.path());
            out
        }
        Err(_) => raw.split('?').next().unwrap_or(raw).to_string(),
    }
}

fn transport_error(op: &str, error: &reqwest::Error) -> String {
    let reason = if error.is_timeout() {
        "请求超时"
    } else if error.is_connect() {
        "连接失败（端点 / 端口 / 证书，或需要代理）"
    } else if error.is_request() {
        "请求构造失败"
    } else {
        "网络请求失败"
    };
    format!("S3 {op}失败（{reason}）")
}

fn status_error(op: &str, status: StatusCode, url: &str) -> String {
    let safe_url = redact_url(url);
    let mut message = format!("S3 {op} 失败: {status} ({safe_url})");
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        message.push_str("。请检查 Access Key ID / Secret Access Key，以及该密钥是否有这个桶的读写权限。若确认无误仍被拒，部分服务（如中国科技云数据胶囊）会按 User-Agent 校验密钥绑定的客户端类型，可在下方填写「自定义 User-Agent」（如 rclone/v1.67.0）。");
    } else if status == StatusCode::NOT_FOUND {
        message.push_str("。请检查存储桶名称、区域（Region）与端点（Endpoint）。");
    } else if status == StatusCode::BAD_REQUEST {
        message.push_str("。请检查区域（Region）是否正确 —— 部分服务商（如 Cloudflare R2）要求填 auto。");
    }
    message
}

/// ： for HEAD。
pub(crate) async fn test_connection(credentials: &S3Credentials) -> Result<(), String> {
    let url = bucket_url(credentials);
    let headers = signed_headers("HEAD", &url, b"", None, credentials)?;
    let mut builder = client(credentials)?.head(&url).timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    let response = builder
        .send()
        .await
        .map_err(|error| transport_error("HEAD bucket", &error))?;
    if response.status().is_success() {
        return Ok(());
    }
    Err(status_error("HEAD bucket", response.status(), &url))
}

/// PUT 1 for 。
pub(crate) async fn put_object(
    credentials: &S3Credentials,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<(), String> {
    let url = object_url(credentials, key);
    let headers = signed_headers("PUT", &url, &bytes, Some(content_type), credentials)?;
    let mut builder = client(credentials)?
        .put(&url)
        .body(bytes)
        .timeout(Duration::from_secs(TRANSFER_TIMEOUT_SECS));
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    let response = builder
        .send()
        .await
        .map_err(|error| transport_error("PUT", &error))?;
    if response.status().is_success() {
        return Ok(());
    }
    Err(status_error("PUT", response.status(), &url))
}

/// GET 1 for ；404 return `None`。
pub(crate) async fn get_object(
    credentials: &S3Credentials,
    key: &str,
    max_bytes: usize,
) -> Result<Option<Vec<u8>>, String> {
    use futures::StreamExt as _;

    let url = object_url(credentials, key);
    let headers = signed_headers("GET", &url, b"", None, credentials)?;
    let mut builder = client(credentials)?
        .get(&url)
        .timeout(Duration::from_secs(TRANSFER_TIMEOUT_SECS));
    for (name, value) in headers {
        builder = builder.header(name, value);
    }
    let response = builder
        .send()
        .await
        .map_err(|error| transport_error("GET", &error))?;

    let status = response.status();
    if status == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(status_error("GET", status, &url));
    }
    if let Some(length) = response.content_length() {
        if length > max_bytes as u64 {
            return Err(format!(
                "远端对象超过上限（{length} 字节 > {max_bytes} 字节）: {}",
                redact_url(&url)
            ));
        }
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("读取 S3 响应失败: {error}"))?;
        if bytes.len().saturating_add(chunk.len()) > max_bytes {
            return Err(format!("远端对象超过上限（{max_bytes} 字节）"));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials() -> S3Credentials {
        S3Credentials {
            access_key_id: "AKIDEXAMPLE".to_string(),
            secret_access_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY".to_string(),
            region: "us-east-1".to_string(),
            bucket: "examplebucket".to_string(),
            endpoint: String::new(),
            user_agent: String::new(),
        }
    }

    #[test]
    fn aws_url_uses_virtual_host_style() {
        assert_eq!(
            object_url(&credentials(), "a/b.json"),
            "https://examplebucket.s3.us-east-1.amazonaws.com/a/b.json"
        );
    }

    #[test]
    fn custom_endpoint_uses_path_style() {
        let mut creds = credentials();
        creds.endpoint = "http://minio.local:9000".to_string();
        assert_eq!(
            object_url(&creds, "a/b.json"),
            "http://minio.local:9000/examplebucket/a/b.json"
        );
    }

    #[test]
    fn bare_endpoint_defaults_to_https() {
        let mut creds = credentials();
        creds.endpoint = "minio.local:9000".to_string();
        assert_eq!(
            object_url(&creds, "x.json"),
            "https://minio.local:9000/examplebucket/x.json"
        );
    }

    #[test]
    fn uri_encode_follows_sigv4_rules() {
        assert_eq!(uri_encode("a b/c", false), "a%20b/c");
        assert_eq!(uri_encode("a b/c", true), "a%20b%2Fc");
        assert_eq!(uri_encode("~_-.", true), "~_-.");
    }

    #[test]
    fn sign_produces_expected_authorization_shape() {
        let url = url::Url::parse("https://examplebucket.s3.us-east-1.amazonaws.com/a.json").unwrap();
        let headers = sign(
            "PUT",
            &url,
            &super::super::sha256_hex(b"{}"),
            Some("application/json"),
            &credentials(),
            chrono::Utc::now(),
        );
        let authorization = headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value.clone())
            .expect("authorization header");
        assert!(authorization.starts_with("AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE/"));
        assert!(authorization.contains("/us-east-1/s3/aws4_request"));
        assert!(authorization.contains("SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date"));
    }
}
