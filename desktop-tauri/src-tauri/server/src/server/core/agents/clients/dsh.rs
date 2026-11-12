//! DeepSeek Harness（`dsh`） 。
//!
//! in use set `~/.dsh/settings.yaml` `llm-pi-ai.providers.< by >` down ； not in 
//! this —— DSH set use （`apiKeyEnv` is ）， in directory 
//! `.credentials.yaml` `refs` down ， need when file。 is DSH when by need 
//! use ， not need need ， new already can to new 。
//!
//! and is DSH 0.1.2-rc.1 `@deepseek-ai/dsh-llm-pi-ai`（ already in ， 
//! ） and `dsh-credentials-local`：
//!
//! - by must to `api` / `baseURL` / `models`，available `api` has 
//!   `openai-completions`、`openai-responses`、`anthropic-messages`；
//! - `id` / `name` / `contextWindow` / `maxTokens` / `input[]` / `reasoningEfforts`，
//! `input` has `text` and `image`； has 
//!   `off` / `minimal` / `low` / `medium` / `high` / `xhigh` / `max`；
//! - none will DSH in when **** `llm-pi-ai` empty， this ，
//! directory for not up ， not ；
//! - `.credentials.yaml` must is `version: 1` and has `version` / `refs` / `records`，
//! otherwise DSH will ，；
//! - set is `< by 、 down >_API_KEY`， this use 1 。

use super::super::{
    files::{self, Change},
    validate, yaml_editing, Model, Profile,
};
use super::{native_entry, native_id, remove, set};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const NAMESPACE: &str = "llm-pi-ai";
const DEFAULT_MODEL: &str = "agent-default-model";
/// DSH ，set 。
const LEVELS: [&str; 7] = ["off", "minimal", "low", "medium", "high", "xhigh", "max"];

fn api_of(protocol: &str) -> Option<&'static str> {
    match protocol {
        "openai_chat" => Some("openai-completions"),
        "responses" => Some("openai-responses"),
        "anthropic" => Some("anthropic-messages"),
        _ => None,
    }
}

fn protocol_of(api: &str) -> &'static str {
    match api {
        "openai-responses" => "responses",
        "anthropic-messages" => "anthropic",
        "openai-completions" => "openai_chat",
        _ => "",
    }
}

/// by is DSH set ， 1 。
fn route_of(source: &str, profile: &Profile) -> String {
    if !source.trim().is_empty() {
        return source.trim().to_string();
    }
    let raw = native_id(profile).to_lowercase();
    let mut route = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            route.push(ch);
        } else if !route.ends_with('-') {
            route.push('-');
        }
    }
    let route = route.trim_matches('-').to_string();
    if route.is_empty() {
        "agent2api".into()
    } else {
        route
    }
}

/// use ： and DSH set 1 （`< by >_API_KEY`）。
fn credential_ref(route: &str) -> String {
    let mut name = String::new();
    for ch in route.chars() {
        if ch.is_ascii_alphanumeric() {
            name.push(ch.to_ascii_uppercase());
        } else if !name.ends_with('_') {
            name.push('_');
        }
    }
    format!("{}_API_KEY", name.trim_matches('_'))
}

/// file DSH ：`$DSH_HOME/.credentials.yaml`， no when in setfile。
pub fn credential_path(path: &Path) -> PathBuf {
    match std::env::var_os("DSH_HOME").filter(|value| !value.is_empty()) {
        Some(home) => PathBuf::from(home).join(".credentials.yaml"),
        None => path.with_file_name(".credentials.yaml"),
    }
}

fn references(path: &Path) -> Result<Map<String, Value>, String> {
    let file = credential_path(path);
    let Some(bytes) = files::read(&file)? else {
        return Ok(Map::new());
    };
    let text = files::text(Some(&bytes))?;
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    // ：file not for when by 「 no has available」， will 。
    let Ok(value) = yaml_editing::parse(&text) else {
        return Ok(Map::new());
    };
    Ok(value
        .get("refs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

pub fn read(path: &Path, value: &Value) -> Result<Vec<Profile>, String> {
    let Some(providers) = value
        .pointer(&format!("/{NAMESPACE}/providers"))
        .and_then(Value::as_object)
    else {
        return Ok(Vec::new());
    };
    let refs = references(path)?;
    let selection = value.get(DEFAULT_MODEL);
    let active = selection
        .and_then(|value| value.get("provider"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut profiles = Vec::new();
    for (route, entry) in providers {
        let Some(fields) = entry.as_object() else {
            continue;
        };
        let api = fields.get("api").and_then(Value::as_str).unwrap_or("");
        let display = fields
            .get("displayName")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(route);
        let mut profile = Profile::new(
            route,
            display,
            protocol_of(api),
            fields.get("baseURL").and_then(Value::as_str).unwrap_or(""),
        );
        profile.env_key = fields
            .get("apiKeyEnv")
            .and_then(Value::as_str)
            .unwrap_or("")
            .into();
        profile.api_key = refs
            .get(&profile.env_key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .into();
        if active == route {
            profile.model = selection
                .and_then(|value| value.get("model"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .into();
            profile.effort = selection
                .and_then(|value| value.get("reasoningEffort"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .into();
        }
        profile.enabled = active == route;
        profile.default = profile.enabled;
        profile.native.entry = entry.clone();
        profile.native.source_id = route.clone();
        profile.native.settings = selection.cloned().unwrap_or(Value::Null);
        profile.models = read_models(fields.get("models"));
        profiles.push(profile);
    }
    Ok(profiles)
}

fn read_models(value: Option<&Value>) -> Vec<Model> {
    let Some(items) = value.and_then(Value::as_array) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|entry| {
            let id = entry.get("id").and_then(Value::as_str)?;
            let efforts = entry.get("reasoningEfforts");
            Some(Model {
                id: id.into(),
                name: entry
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
                context_window: entry.get("contextWindow").and_then(Value::as_u64),
                max_output_tokens: entry.get("maxTokens").and_then(Value::as_u64),
                supports_tools: None,
                supports_images: entry
                    .get("input")
                    .and_then(Value::as_array)
                    .map(|list| list.iter().any(|value| value.as_str() == Some("image"))),
                supports_reasoning: match efforts {
                    Some(Value::Bool(false)) => Some(false),
                    Some(Value::Object(map)) if !map.is_empty() => Some(true),
                    _ => None,
                },
                reasoning_levels: efforts
                    .and_then(Value::as_object)
                    .map(|map| map.keys().cloned().collect())
                    .unwrap_or_default(),
                effort: String::new(),
            })
        })
        .collect()
}

/// directory to DSH ； for not up ， not 。
fn efforts(levels: &[String]) -> Map<String, Value> {
    let mut offered: Vec<(&'static str, Option<&'static str>)> = Vec::new();
    for level in levels {
        let mapped = match level.trim().to_ascii_lowercase().as_str() {
            "off" | "none" | "disabled" => Some(("off", None)),
            "minimal" => Some(("minimal", Some("minimal"))),
            "low" => Some(("low", Some("low"))),
            "medium" | "mid" => Some(("medium", Some("medium"))),
            "high" => Some(("high", Some("high"))),
            "xhigh" => Some(("xhigh", Some("xhigh"))),
            "max" => Some(("max", Some("max"))),
            _ => None,
        };
        if let Some(entry) = mapped {
            if !offered.iter().any(|(key, _)| *key == entry.0) {
                offered.push(entry);
            }
        }
    }
    let mut ordered = Map::new();
    for key in LEVELS {
        if let Some((_, wire)) = offered.iter().find(|(name, _)| *name == key) {
            ordered.insert(key.into(), wire.map_or(Value::Null, |wire| json!(wire)));
        }
    }
    ordered
}

/// ： use file unknown（`compat` ）， as 。
fn model_entry(model: &Model, existing: Option<&Value>) -> Result<Value, String> {
    if model.id.trim().is_empty() {
        return Err("模型 ID 不能为空".into());
    }
    let id = model.id.trim().to_string();
    let mut entry = existing
        .filter(|value| value.is_object())
        .cloned()
        .unwrap_or_else(|| json!({}));
    set(&mut entry, &["id"], json!(id))?;
    if !model.name.trim().is_empty() && model.name.trim() != id {
        set(&mut entry, &["name"], json!(model.name.trim()))?;
    } else {
        remove(&mut entry, &["name"]);
    }
    match model.context_window {
        Some(value) => set(&mut entry, &["contextWindow"], json!(value))?,
        None => remove(&mut entry, &["contextWindow"]),
    }
    match model.max_output_tokens {
        Some(value) => set(&mut entry, &["maxTokens"], json!(value))?,
        None => remove(&mut entry, &["maxTokens"]),
    }
    // ：DSH will as in will in up ， is before 。
    set(
        &mut entry,
        &["input"],
        json!(if model.supports_images == Some(true) {
            vec!["text", "image"]
        } else {
            vec!["text"]
        }),
    )?;
    if model.supports_reasoning == Some(false) {
        set(&mut entry, &["reasoningEfforts"], json!(false))?;
    } else {
        let offered = efforts(&model.reasoning_levels);
        if offered.is_empty() {
            remove(&mut entry, &["reasoningEfforts"]);
        } else {
            set(&mut entry, &["reasoningEfforts"], Value::Object(offered))?;
        }
    }
    Ok(entry)
}

pub fn write(
    path: &Path,
    value: &mut Value,
    profile: &Profile,
    operation: &str,
    warnings: &mut Vec<String>,
) -> Result<Vec<Change>, String> {
    let api = api_of(&profile.protocol)
        .ok_or("DSH 只支持 OpenAI Chat Completions、OpenAI Responses 与 Anthropic Messages")?;
    let base = profile.base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("请填写 DSH 的接口基址".into());
    }
    let route = route_of(&profile.native.source_id, profile);
    let existing = value
        .get(NAMESPACE)
        .and_then(|value| value.get("providers"))
        .and_then(|value| value.get(route.as_str()));
    let mut entry = native_entry(existing, &profile.native.entry)?;
    set(
        &mut entry,
        &["displayName"],
        json!(if profile.name.trim().is_empty() {
            "Agent2API"
        } else {
            profile.name.trim()
        }),
    )?;
    set(&mut entry, &["api"], json!(api))?;
    set(&mut entry, &["baseURL"], json!(base))?;
    let previous = entry
        .get("models")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let models = profile
        .models
        .iter()
        .map(|model| {
            let existing = previous.iter().find(|item| {
                item.get("id").and_then(Value::as_str) == Some(model.id.trim())
            });
            model_entry(model, existing)
        })
        .collect::<Result<Vec<_>, _>>()?;
    set(&mut entry, &["models"], json!(models))?;
    let existing_ref = entry
        .get("apiKeyEnv")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let mut pending = None;
    match validate::credential(profile)?.as_deref() {
        Some(key) if !key.is_empty() => {
            // use use ， new use ， when 。
            let reference = if profile.auth_mode == "preserve" && !existing_ref.is_empty() {
                existing_ref.clone()
            } else {
                credential_ref(&route)
            };
            set(&mut entry, &["apiKeyEnv"], json!(reference))?;
            pending = Some((reference, key.to_string()));
        }
        Some(_) => remove(&mut entry, &["apiKeyEnv"]),
        None if profile.auth_mode == "env" => {
            set(&mut entry, &["apiKeyEnv"], json!(profile.env_key))?;
        }
        None => {}
    }
    set(value, &["llm-pi-ai", "providers", &route], entry)?;
    let selected = value
        .get(DEFAULT_MODEL)
        .and_then(|value| value.get("provider"))
        .and_then(Value::as_str)
        == Some(route.as_str());
    if (operation == "default" || selected) && !profile.model.trim().is_empty() {
        set(value, &[DEFAULT_MODEL, "provider"], json!(route))?;
        set(value, &[DEFAULT_MODEL, "model"], json!(profile.model.trim()))?;
        // directory 「」 off / none ，DSH has off。
        let effort = match profile.effort.trim() {
            "" | "default" => None,
            "none" => Some("off"),
            other => Some(other),
        };
        match effort {
            Some(effort) => set(value, &[DEFAULT_MODEL, "reasoningEffort"], json!(effort))?,
            None => remove(value, &[DEFAULT_MODEL, "reasoningEffort"]),
        }
    }
    let mut extra = Vec::new();
    if let Some((reference, key)) = pending {
        extra.push(credential_change(path, &reference, Some(&key))?);
        warnings.push(
            "同时修改 settings.yaml 与 .credentials.yaml（密钥只以引用名出现在设置里）。".into(),
        );
    }
    Ok(extra)
}

fn credential_change(path: &Path, reference: &str, key: Option<&str>) -> Result<Change, String> {
    let file = credential_path(path);
    let mut change = Change::load(file)?;
    let original = files::text(change.before.as_deref())?;
    let mut store = if original.trim().is_empty() {
        json!({ "version": 1 })
    } else {
        yaml_editing::parse(&original)?
    };
    if store.as_object().is_some_and(Map::is_empty) {
        store = json!({ "version": 1 });
    }
    supported(&store, &change.path)?;
    if store.get("refs").is_none_or(Value::is_null) {
        set(&mut store, &["refs"], json!({}))?;
    }
    match key {
        Some(key) => set(&mut store, &["refs", reference], json!(key))?,
        None => remove(&mut store, &["refs", reference]),
    }
    change.after = Some(yaml_editing::render(&original, &store)?.into_bytes());
    Ok(change)
}

/// DSH for file is need ： `version`（ old ） or unknown when will 
/// ， this ， 。
fn supported(store: &Value, path: &Path) -> Result<(), String> {
    match store.get("version").and_then(Value::as_u64) {
        Some(1) => {}
        Some(version) => {
            return Err(format!(
                "{} 的 version 是 {version}，本页只认 DSH 的 version 1 布局",
                path.display()
            ))
        }
        None => {
            return Err(format!(
                "{} 不是 version 1 布局：先启动一次 DSH 让它自行迁移，再回到本页接入",
                path.display()
            ))
        }
    }
    for key in store.as_object().map(Map::keys).into_iter().flatten() {
        if !matches!(key.as_str(), "version" | "refs" | "records") {
            return Err(format!(
                "{} 含有未知顶层字段 {key}，DSH 自身也会拒绝读取，请先处理",
                path.display()
            ));
        }
    }
    Ok(())
}

pub fn delete(path: &Path, value: &mut Value, profile: &Profile) -> Result<Vec<Change>, String> {
    let route = route_of(&profile.native.source_id, profile);
    remove(value, &[NAMESPACE, "providers", &route]);
    let selected = value
        .get(DEFAULT_MODEL)
        .and_then(|value| value.get("provider"))
        .and_then(Value::as_str)
        == Some(route.as_str());
    if selected {
        // default to when DSH will not ； this to default。
        remove(value, &[DEFAULT_MODEL]);
    }
    let mut extra = Vec::new();
    let file = credential_path(path);
    if Change::load(file.clone())?.before.is_some() {
        extra.push(credential_change(path, &credential_ref(&route), None)?);
    }
    Ok(extra)
}
