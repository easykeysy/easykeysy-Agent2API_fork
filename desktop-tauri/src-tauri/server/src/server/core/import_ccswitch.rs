//! 「 from its 」 1 ：**cc-switch** 。
//!
//! ── cc-switch is 、Why can file ──────────────────
//! cc-switch is （Tauri + SQLite）， Claude Code / Codex /
//! Gemini CLI etc client config， in `~/.cc-switch/cc-switch.db`。
//! no has for ，path、 is Rust —— 
//! 1 SQLite file can ， use config。（cc-switch
//! `src-tauri/src/provider.rs` `resolve_usage_credentials`） to 
//! app_type from `settings_config` JSON base_url / api_key then ，
//! this for （claude / codex ； its by ）。
//!
//! ── ──────────────────────────────────────────
//! · `importable = true` —— claude / codex and base_url and api_key 、
//! no has ： before can ，「」 by this create
//! + （ use `POST /api/custom-providers`， not in ）。
//! · （`reason` = already in ）—— and already and 
//! （：）。
//! · its —— cc-switch its app_type（gemini / grokbuild / opencode /
//! openclaw / hermes / pi / claude-desktop） and or ：
//! `importable = false` + Reason， before 「unsupported」， use 
//! Why， not is 。
//!
//! ── api_key Why can in response ────────────────────────
//! this is ：request（ WebView） and （） in 1 
//! up ，Key not ； need 。 in 
//! protected（） after ， and 「client」 1 。
//!
//! ── ──────────────────────────────────────────────────
//! not panic（release is panic=abort）：SQLite not 、 is JSON、
//! TOML failure 1 by 「 this not can 」， not failure。
//! （`SQLITE_OPEN_READ_ONLY`）：cc-switch can can in ， not can 
//! for （ WAL file not this by ）。

use std::path::PathBuf;

use rusqlite::OpenFlags;
use serde_json::{json, Map, Value};

use crate::server::core::custom_providers;

/// can app_type（SQLite `providers.app_type` ， and cc-switch
/// `AppType::as_str()` 1 ）。claude-desktop and claude is ** 1 ** config
/// （ is Anthropic env map，cc-switch is 1 ），
/// 1 and can 。
const SUPPORTED_TYPES: [&str; 3] = ["claude", "codex", "claude-desktop"];

/// its app_type → up （ 「unsupported」 use ）。
fn unsupported_label(app_type: &str) -> &'static str {
    match app_type {
        "gemini" => "Gemini CLI",
        "grokbuild" => "Grok Build",
        "opencode" => "OpenCode",
        "openclaw" => "OpenClaw",
        "hermes" => "Hermes",
        "pi" => "Pi",
        _ => "该客户端",
    }
}

/// use directory（Windows USERPROFILE， and / 1 ，
/// not dirs crate）。
fn home_dir() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// cc-switch path（`~/.cc-switch/cc-switch.db`）。
///
/// not v3.10.3 `HOME` ： that is 1 ，
/// default not to 「 to 」， 1 old —— etc 
/// 1 use config。
fn cc_switch_db_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".cc-switch").join("cc-switch.db"))
}

/// JSON （ / → empty）
fn string_field(value: Option<&Value>, key: &str) -> String {
    value
        .and_then(|item| item.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

/// 1 empty（ and cc-switch `first_non_empty` 1 ：
/// preset will empty，must「empty」 not is 「」）
fn first_non_empty(env: Option<&Value>, keys: &[&str]) -> String {
    let Some(env) = env else { return String::new() };
    for key in keys {
        let text = string_field(Some(env), key);
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

/// Codex `config` is **TOML **。 provider base_url：
/// `model_provider`（）→ `model_providers.<>.base_url`，
/// `base_url`。 then cc-switch `codex_config::extract_codex_base_url`
/// —— `[model_providers.*]` not （ not this ）。
///
/// `toml::from_str::<toml::Table>`（serde ****）。
/// not can use `toml::Value::from_str`：toml 0.9 is 「 」，
/// for （ `key = value`） 1 —— ：Codex 
/// base_url as empty，「not configured」。
fn extract_codex_base_url(config_text: &str) -> String {
    let Ok(doc) = toml::from_str::<toml::Table>(config_text) else {
        return String::new();
    };
    if let Some(active) = doc.get("model_provider").and_then(|v| v.as_str()) {
        if let Some(url) = doc
            .get("model_providers")
            .and_then(|providers| providers.get(active))
            .and_then(|provider| provider.get("base_url"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|text| !text.is_empty())
        {
            return url.to_string();
        }
    }
    doc.get("base_url")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

/// Codex API Key：`auth.OPENAI_API_KEY` ， config TOML 
/// provider（ or ） `experimental_bearer_token`（cc-switch ）。
fn extract_codex_api_key(settings: &Value, config_text: &str) -> String {
    let from_auth = string_field(settings.get("auth"), "OPENAI_API_KEY");
    if !from_auth.is_empty() {
        return from_auth;
    }
    let Ok(doc) = toml::from_str::<toml::Table>(config_text) else {
        return String::new();
    };
    if let Some(active) = doc.get("model_provider").and_then(|v| v.as_str()) {
        let token = doc
            .get("model_providers")
            .and_then(|providers| providers.get(active))
            .and_then(|provider| provider.get("experimental_bearer_token"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if !token.is_empty() {
            return token.to_string();
        }
    }
    doc.get("experimental_bearer_token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

/// Codex default（config.toml `model`）： when as ****
/// to new —— is by before ， not need 
/// can use （ before runImport ）。
fn extract_codex_model(config_text: &str) -> String {
    let Ok(doc) = toml::from_str::<toml::Table>(config_text) else {
        return String::new();
    };
    doc.get("model")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or("")
        .to_string()
}

/// Claude Code `[1M]` after is ** can **（ 100 up down ）， up 
/// 1 not ， before must —— cc-switch this （`proxy/model_mapper.rs`
/// `strip_one_m_suffix_for_upstream`，：「 up API not this 
/// can ， before need need 」）。
///
/// return `( up , need need client)`：client in **** when empty
/// —— client（Claude Code） is that ， no has alias → up 
/// this ， will as 「 not in 」 （
/// `custom_providers::bindings::resolve`）。
fn split_one_m_marker(model: &str) -> (String, String) {
    /// and cc-switch `ONE_M_CONTEXT_MARKER` 1 （ not ）
    const MARKER: &str = "[1M]";
    let trimmed = model.trim();
    if trimmed.len() > MARKER.len()
        && trimmed.to_ascii_lowercase().ends_with(&MARKER.to_ascii_lowercase())
    {
        let clean = trimmed[..trimmed.len() - MARKER.len()].trim_end().to_string();
        if !clean.is_empty() {
            return (clean, trimmed.to_string());
        }
    }
    (trimmed.to_string(), String::new())
}

/// 1 claude need ：`(, )`。
///
/// ，：
/// 1. `ANTHROPIC_MODEL` —— ， use in use that ；
/// 2. `ANTHROPIC_DEFAULT_*_MODEL` —— Claude Code ，
/// down is up （ has as empty、 has ）。
/// `*_MODEL_NAME` 1 not ： that is Claude Code ****
/// （cc-switch ：「otherwise Claude Code will up 1 」），
/// not is to up id。
///
/// limit 6 ： this is 1 「 can use 」 ， not is up directory（ has 
/// 「get」）。 as empty is （ this no ）。
fn collect_claude_models(env: Option<&Value>) -> (Vec<String>, Vec<Value>) {
    /// limit（ 3~4 ，6 and not ）
    const MAX_MODELS: usize = 6;
    let mut models: Vec<String> = Vec::new();
    let mut mappings: Vec<Value> = Vec::new();
    let mut push = |raw: &str| {
        let (clean, alias) = split_one_m_marker(raw);
        if clean.is_empty() {
            return;
        }
        if !models.iter().any(|known| known.eq_ignore_ascii_case(&clean)) {
            if models.len() >= MAX_MODELS {
                return;
            }
            models.push(clean.clone());
        }
        if !alias.is_empty() {
            let known = mappings.iter().any(|item| {
                item.get("alias").and_then(Value::as_str)
                    .is_some_and(|text| text.eq_ignore_ascii_case(&alias))
            });
            if !known && mappings.len() < MAX_MODELS {
                mappings.push(json!({ "alias": alias, "target": clean }));
            }
        }
    };
    push(&string_field(env, "ANTHROPIC_MODEL"));
    if let Some(env) = env.and_then(Value::as_object) {
        for (key, value) in env {
            if !key.starts_with("ANTHROPIC_DEFAULT_") || !key.ends_with("_MODEL") {
                continue;
            }
            if let Some(text) = value.as_str() {
                push(text);
            }
        }
    }
    (models, mappings)
}

/// 1 cc-switch → （ can and Reason）。
fn scan_entry(
    id: &str,
    app_type: &str,
    name: &str,
    notes: &str,
    settings: &Value,
    local_names: &std::collections::HashSet<String>,
) -> Value {
    let mut base_url = String::new();
    let mut api_key = String::new();
    let mut protocol = String::new();
    // and （claude from env ；codex has model）
    let mut models: Vec<String> = Vec::new();
    let mut model_mappings: Vec<Value> = Vec::new();

    match app_type {
        // Anthropic ：Claude Code and Claude Desktop config 1 
        // （env BASE_URL + Key， cc-switch ）
        "claude" | "claude-desktop" => {
            let env = settings.get("env");
            base_url = string_field(env, "ANTHROPIC_BASE_URL");
            api_key = first_non_empty(
                env,
                &[
                    "ANTHROPIC_AUTH_TOKEN",
                    "ANTHROPIC_API_KEY",
                    "OPENROUTER_API_KEY",
                    "GOOGLE_API_KEY",
                ],
            );
            protocol = custom_providers::PROTOCOL_ANTHROPIC.to_string();
            (models, model_mappings) = collect_claude_models(env);
        }
        // Codex ：key in auth / TOML，base_url in TOML 
        "codex" => {
            let config_text = string_field(Some(settings), "config");
            base_url = extract_codex_base_url(&config_text);
            api_key = extract_codex_api_key(settings, &config_text);
            protocol = custom_providers::PROTOCOL_RESPONSES.to_string();
            let model = extract_codex_model(&config_text);
            if !model.is_empty() {
                models.push(model);
            }
        }
        _ => {}
    }

    // not can Reason（ by ： → → → ）
    let reason = if !SUPPORTED_TYPES.contains(&app_type) {
        format!("cc-switch 的 {} 配置，暂不支持导入", unsupported_label(app_type))
    } else if base_url.is_empty() {
        if app_type.starts_with("claude") {
            "官方直连配置，没有可导入的自定义端点".to_string()
        } else {
            "未配置自定义端点".to_string()
        }
    } else if api_key.is_empty() {
        "缺少 API Key".to_string()
    } else if local_names.contains(name) {
        "本地已存在同名提供商，跳过".to_string()
    } else {
        String::new()
    };
    let importable = reason.is_empty();

    json!({
        "id": id,
        "appType": app_type,
        "name": name,
        "notes": notes,
        // for can to （ not need need ， 1 ）
        "baseUrl": if importable { base_url.as_str() } else { "" },
        "apiKey": if importable { api_key.as_str() } else { "" },
        "protocol": protocol,
        // and （ is by before ， before runImport）。
        // `models` is up ；`modelMappings` is 「 
        // client → up 」，client can by to 。 can can as empty。
        "models": if importable { models } else { Vec::new() },
        "modelMappings": if importable { model_mappings } else { Vec::new() },
        "importable": importable,
        "reason": reason,
    })
}

/// cc-switch ，return API response（ not failure：error in response）。
pub fn scan() -> Value {
    let Some(path) = cc_switch_db_path() else {
        return json!({
            "available": false,
            "reason": "无法定位用户主目录，找不到 cc-switch 的数据文件",
        });
    };
    let path_text = path.to_string_lossy().to_string();
    if !path.exists() {
        return json!({
            "available": false,
            "path": path_text,
            "reason": "未找到 cc-switch 的数据库（~/.cc-switch/cc-switch.db）——请确认本机安装了 cc-switch，并在里面添加过供应商",
        });
    }

    let conn = match rusqlite::Connection::open_with_flags(
        &path,
        // + not create：cc-switch can can in ， for 
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(conn) => conn,
        Err(error) => {
            return json!({
                "available": false,
                "path": path_text,
                "reason": format!("无法读取 cc-switch 数据库（可能正被占用）: {error}"),
            });
        }
    };

    // already （）。list() by up 
    // ， can ignore（ is ）。
    let local_names: std::collections::HashSet<String> = custom_providers
        ::list()
        .iter()
        .filter_map(|item| item.get("name").and_then(Value::as_str).map(str::to_string))
        .collect();

    let mut stmt = match conn.prepare(
        "SELECT id, app_type, name, settings_config, COALESCE(notes, '') FROM providers",
    ) {
        Ok(stmt) => stmt,
        Err(error) => {
            return json!({
                "available": false,
                "path": path_text,
                "reason": format!("cc-switch 数据库结构与预期不符（{error}）：可能是不受支持的旧版本"),
            });
        }
    };

    let mut rows: Vec<(String, String, String, String, String)> = Vec::new();
    let mapped = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0).unwrap_or_default(),
            row.get::<_, String>(1).unwrap_or_default(),
            row.get::<_, String>(2).unwrap_or_default(),
            row.get::<_, String>(3).unwrap_or_default(),
            row.get::<_, String>(4).unwrap_or_default(),
        ))
    });
    let Ok(mapped) = mapped else {
        return json!({
            "available": false,
            "path": path_text,
            "reason": "读取 cc-switch 数据失败：数据库可能正被写入，稍后重试",
        });
    };
    for row in mapped.flatten() {
        rows.push(row);
    }

    let mut entries: Vec<Value> = Vec::new();
    for (id, app_type, name, settings_text, notes) in &rows {
        // name no has ，（ and has to 1 ）
        if name.trim().is_empty() {
            continue;
        }
        let settings: Value = serde_json::from_str(settings_text).unwrap_or(Value::Null);
        let settings = if settings.is_object() {
            settings
        } else {
            Value::Object(Map::new())
        };
        entries.push(scan_entry(id, app_type, name.trim(), notes.trim(), &settings, &local_names));
    }

    // ： can in before （claude → codex → ），unsupported / in after by 
    // app_type —— use need to ， Reason in can 。
    let rank = |entry: &Value| -> (usize, String, String) {
        let importable = entry.get("importable").and_then(Value::as_bool) == Some(true);
        (
            if importable { 0 } else { 1 },
            string_field(Some(entry), "appType"),
            string_field(Some(entry), "name").to_lowercase(),
        )
    };
    entries.sort_by(|a, b| rank(a).cmp(&rank(b)));

    json!({
        "available": true,
        "path": path_text,
        "total": entries.len(),
        "providers": entries,
    })
}
