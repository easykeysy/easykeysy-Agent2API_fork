//! WebDAV ：PROPFIND / MKCOL / PUT / GET。
//!
//! 「 1 URL up / 」 this 1 ， not manifest、 not 
//! 。 in `super::mod`， by and cc-switch 
//! `services/webdav.rs` / `services/webdav_sync.rs` ： can （URL 、
//! ）， then not HTTP 。
//!
//! ── ────────────────────────────────────────────────
//! use `egress::client_for(None)`（） is ？****
//! （`client_for_system_proxy`）： use up can Nextcloud / ，
//! in （ can 、 not is ）。 this and 「
//! 」 1 to —— no has 「」 this ， not this 
//! 「 is 」。
//!
//! ── when ────────────────────────────────────────────────────
//! Client， not use `egress` Client： after read_timeout 
//! 「request when 」set（ to SSE， can can use 600 seconds）， need is 
//! 「 30 seconds、 300 seconds」 this 1 。 1 not 
//! Client （ 1 ） 1 。

mod download;

use std::sync::OnceLock;
use std::time::Duration;

use reqwest::{Method, StatusCode, Url};

/// request（PROPFIND / MKCOL） when 
const DEFAULT_TIMEOUT_SECS: u64 = 30;
/// file（PUT / GET ） when 。request total —— 256MB limit down ，
/// 300 seconds 0.85MB/s down ， use up 。
const TRANSFER_TIMEOUT_SECS: u64 = 300;

/// Basic Auth `( use , )`
pub(crate) type WebDavAuth = Option<(String, Option<String>)>;

fn method_propfind() -> Method {
    Method::from_bytes(b"PROPFIND").expect("PROPFIND 是合法 HTTP 方法")
}

fn method_mkcol() -> Method {
    Method::from_bytes(b"MKCOL").expect("MKCOL 是合法 HTTP 方法")
}

/// ── User-Agent ───────────────────────────────────────
/// WebDAV （ in ） to 「 use 」 up ， and by 
/// User-Agent ；UA not this use 。`user_agent` as empty = not 。
fn client(user_agent: &str) -> Result<reqwest::Client, String> {
    client_with_policy(reqwest::redirect::Policy::limited(10), user_agent)
}

fn client_with_policy(
    policy: reqwest::redirect::Policy,
    user_agent: &str,
) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .redirect(policy);
    // in use ：empty will 「 not UA」， not default
    let user_agent = user_agent.trim();
    if !user_agent.is_empty() {
        builder = builder.user_agent(user_agent.to_string());
    }
    // not total when （.timeout）， by request ：PROPFIND 30 seconds、 300 seconds
    builder
        .build()
        .map_err(|error| format!("创建 HTTP 客户端失败: {}", redact_detail(&error.to_string())))
}

/// and WebDAV （ http / https）。
pub(crate) fn parse_base_url(raw: &str) -> Result<Url, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("WebDAV 地址不能为空".to_string());
    }
    let url = Url::parse(trimmed).map_err(|error| format!("WebDAV 地址无效: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        other => Err(format!("WebDAV 只支持 http/https 地址（当前是 {other}）")),
    }
}

/// and path URL。 （empty and in ）。
pub(crate) fn build_remote_url(base_url: &str, segments: &[String]) -> Result<String, String> {
    let mut url = parse_base_url(base_url)?;
    {
        let mut path = url
            .path_segments_mut()
            .map_err(|_| "WebDAV 地址格式不支持追加路径".to_string())?;
        path.pop_if_empty();
        for segment in segments {
            path.push(segment);
        }
    }
    Ok(url.to_string())
}

/// use / → Basic Auth（ use as empty = not ， WebDAV）
pub(crate) fn auth_from_credentials(username: &str, password: &str) -> WebDavAuth {
    let user = username.trim();
    if user.is_empty() {
        return None;
    }
    Some((user.to_string(), Some(password.to_string())))
}

fn apply_auth(
    builder: reqwest::RequestBuilder,
    auth: &WebDavAuth,
) -> reqwest::RequestBuilder {
    match auth {
        Some((user, password)) => builder.basic_auth(user, password.as_deref()),
        None => builder,
    }
}

/// error** not can and **：URL can can `user:pass@`， can can 
/// parameter。 scheme + host + port + path。
pub(crate) fn redact_url(raw: &str) -> String {
    match Url::parse(raw) {
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
        Err(_) => "（无效地址）".to_string(),
    }
}

fn is_jianguoyun(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_lowercase()))
        .map(|host| host.contains("jianguoyun.com") || host.contains("nutstore"))
        .unwrap_or(false)
}

fn status_error(op: &str, status: StatusCode, url: &str) -> String {
    let safe_url = redact_url(url);
    let mut message = format!("WebDAV {op} 失败: {status} ({safe_url})");
    let jgy = is_jianguoyun(url);
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        if jgy {
            message.push_str(
                "。坚果云请使用「第三方应用密码」，并确认地址指向 /dav/ 下的目录。",
            );
        } else {
            message.push_str("。请检查 WebDAV 用户名、密码与目录读写权限。");
        }
    } else if jgy && (status == StatusCode::NOT_FOUND || status.is_redirection()) {
        message.push_str("。坚果云常见原因：地址不在 /dav/ 可写目录下。");
    } else if op == "MKCOL" && status == StatusCode::CONFLICT {
        if jgy {
            message.push_str(
                "。坚果云不允许自动创建顶层文件夹，请先在网页端手动创建后再试。",
            );
        } else {
            message.push_str("。请确认上级目录存在。");
        }
    } else if status == StatusCode::NOT_FOUND {
        message.push_str("。请确认 WebDAV 地址与远程根目录是否正确。");
    }
    message
}

fn redact_detail(text: &str) -> String {
    static URLS: OnceLock<Option<regex::Regex>> = OnceLock::new();
    let Some(urls) = URLS.get_or_init(|| regex::Regex::new(r#"(?i)\b[a-z][a-z0-9+.-]*://[^\s<>\"']+"#).ok()) else {
        return "底层错误详情不可用".to_string();
    };
    urls.replace_all(text, |found: &regex::Captures<'_>| redact_url(&found[0])).into_owned()
}

fn error_detail(error: &reqwest::Error) -> String {
    let mut parts = Vec::new();
    let mut source = std::error::Error::source(error);
    for _ in 0..8 {
        let Some(current) = source else { break };
        let text = redact_detail(&current.to_string());
        if !text.is_empty() && !parts.contains(&text) {
            parts.push(text);
        }
        source = current.source();
    }
    parts.join("；")
}

fn transport_error(op: &str, url: &str, error: &reqwest::Error) -> String {
    let detail = error_detail(error);
    let lower = detail.to_ascii_lowercase();
    let reason = if error.is_timeout() {
        "请求超时"
    } else if error.is_redirect() {
        "重定向失败"
    } else if error.is_builder() {
        "请求构造失败"
    } else if error.is_connect() && (lower.contains("certificate") || lower.contains("cert") || lower.contains("tls")) {
        "TLS / 证书校验失败"
    } else if error.is_connect() && (lower.contains("dns") || lower.contains("resolve") || lower.contains("lookup")) {
        "域名解析失败"
    } else if error.is_connect() {
        "连接失败（目标地址 / 端口或代理）"
    } else if error.is_body() {
        "响应传输失败"
    } else {
        "网络请求失败"
    };
    let mut message = format!("WebDAV {op}失败（{reason}）: {}", redact_url(url));
    if !detail.is_empty() {
        message.push_str(&format!("。底层原因：{detail}"));
    }
    message
}

/// ： for `PROPFIND Depth: 0`。
///
/// use PROPFIND not is HEAD： WebDAV for directory HEAD return 405，
/// PROPFIND is WebDAV 「， this has 」， can when 「 for 」 and 
/// 「 this is WebDAV not is 」。
pub(crate) async fn test_connection(
    base_url: &str,
    auth: &WebDavAuth,
    user_agent: &str,
) -> Result<(), String> {
    let url = parse_base_url(base_url)?;
    let response = apply_auth(
        client(user_agent)?
            .request(method_propfind(), url)
            .header("Depth", "0")
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
        auth,
    )
    .send()
    .await
    .map_err(|error| transport_error("连接", base_url, &error))?;

    if response.status().is_success() || response.status() == StatusCode::MULTI_STATUS {
        return Ok(());
    }
    Err(status_error("PROPFIND", response.status(), base_url))
}

/// createdirectory（`MKCOL` must 1 1 ）。
///
/// ── create + ────────────────────────────────────
/// MKCOL 「 PROPFIND not 」 1 。 already in when server
/// 405（METHOD_NOT_ALLOWED） or 409（CONFLICT）， this can can is 「 already in 」
/// can can is 「directory not in 」， 1 PROPFIND ； in success。
pub(crate) async fn ensure_remote_directories(
    base_url: &str,
    segments: &[String],
    auth: &WebDavAuth,
    user_agent: &str,
) -> Result<(), String> {
    if segments.is_empty() {
        return Ok(());
    }
    let http = client(user_agent)?;
    for depth in 1..=segments.len() {
        let url = build_remote_url(base_url, &segments[..depth])?;
        let dir_url = if url.ends_with('/') { url } else { format!("{url}/") };
        let response = apply_auth(
            http.request(method_mkcol(), &dir_url)
                .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
            auth,
        )
        .send()
        .await
        .map_err(|error| transport_error("MKCOL", &dir_url, &error))?;

        let status = response.status();
        if status == StatusCode::CREATED || status.is_success() {
            continue;
        }
        if status == StatusCode::METHOD_NOT_ALLOWED
            || status == StatusCode::CONFLICT
            || status.is_redirection()
        {
            if propfind_exists(&http, &dir_url, auth).await? {
                continue;
            }
        }
        return Err(status_error("MKCOL", status, &dir_url));
    }
    Ok(())
}

async fn propfind_exists(
    http: &reqwest::Client,
    url: &str,
    auth: &WebDavAuth,
) -> Result<bool, String> {
    let response = apply_auth(
        http.request(method_propfind(), url)
            .header("Depth", "0")
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
        auth,
    )
    .send()
    .await;
    match response {
        Ok(response) => Ok(response.status().is_success() || response.status() == StatusCode::MULTI_STATUS),
        // failure by 「 not in 」： use will to MKCOL error（ this 
        // error「directory not in 」）
        Err(_) => Ok(false),
    }
}

/// PUT 1 file。
pub(crate) async fn put_bytes(
    url: &str,
    auth: &WebDavAuth,
    bytes: Vec<u8>,
    content_type: &str,
    user_agent: &str,
) -> Result<(), String> {
    let response = apply_auth(
        client(user_agent)?
            .put(url)
            .header("Content-Type", content_type)
            .body(bytes)
            .timeout(Duration::from_secs(TRANSFER_TIMEOUT_SECS)),
        auth,
    )
    .send()
    .await
    .map_err(|error| transport_error("PUT", url, &error))?;
    if response.status().is_success() {
        return Ok(());
    }
    Err(status_error("PUT", response.status(), url))
}

/// GET 1 file；404 return `None`（「 no has this 」 is ， not is error）。
///
/// `max_bytes` is **limit**： `Content-Length`， in when ——
/// not （server can not ， or 1 ），。
pub(crate) async fn get_bytes(
    url: &str,
    auth: &WebDavAuth,
    max_bytes: usize,
    user_agent: &str,
) -> Result<Option<Vec<u8>>, String> {
    download::get_bytes(url, auth, max_bytes, user_agent).await
}

/// delete 1 file（current in 「 up failure after 」 use ； to will 
/// 「empty」 by ）。404 as success。
#[allow(dead_code)]
pub(crate) async fn delete(url: &str, auth: &WebDavAuth, user_agent: &str) -> Result<(), String> {
    let response = apply_auth(
        client(user_agent)?
            .delete(url)
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
        auth,
    )
    .send()
    .await
    .map_err(|error| transport_error("DELETE", url, &error))?;
    if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
        return Ok(());
    }
    Err(status_error("DELETE", response.status(), url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_remote_url_encodes_segments() {
        let url = build_remote_url(
            "https://dav.example.com/remote.php/dav/files/demo/",
            &["agent2api sync".to_string(), "v1".to_string(), "default".to_string()],
        )
        .unwrap();
        assert_eq!(
            url,
            "https://dav.example.com/remote.php/dav/files/demo/agent2api%20sync/v1/default"
        );
        assert!(!url.contains("//agent2api"), "不应出现双斜杠");
    }

    #[test]
    fn redact_url_strips_credentials_and_query() {
        let redacted = redact_url("https://alice:secret@example.com:8443/dav?token=abc");
        assert_eq!(redacted, "https://example.com:8443/dav");
        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("abc"));
    }

    #[test]
    fn parse_base_url_rejects_other_schemes() {
        assert!(parse_base_url("ftp://example.com/dav").is_err());
        assert!(parse_base_url("  ").is_err());
        assert!(parse_base_url("https://dav.jianguoyun.com/dav/").is_ok());
    }

    #[test]
    fn auth_from_credentials_rejects_blank_username() {
        assert!(auth_from_credentials("  ", "pass").is_none());
        assert_eq!(
            auth_from_credentials(" user ", "pass"),
            Some(("user".to_string(), Some("pass".to_string())))
        );
    }

    #[test]
    fn jianguoyun_detection() {
        assert!(is_jianguoyun("https://dav.jianguoyun.com/dav"));
        assert!(!is_jianguoyun("https://nextcloud.example.com/dav"));
    }
}
