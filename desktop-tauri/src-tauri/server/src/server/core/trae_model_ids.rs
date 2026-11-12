//! Trae ID 1 （2026-11）。
//!
//! ── Why need need this 1 ────────────────────────────────────────
//! old Trae directory `config_name` 1 `-solo`，Max 
//! 1 `-max`（`<id>-solo-max`）。 this after is ， not is up 。
//! in `config_name` and optional `<id>-max`；config（ then 、 by 、Key 、
//! 、 will 、、directory） old ，must 1 ，otherwise use will 
//! to 「 has 1 ID、config is 1 」。
//!
//! ── old requestavailable ──────────────────────────────────────────
//! ** **， not down in request。 old client can request
//! old client can request
//! `<id>-solo` / `<id>-solo-max`：， before 1 
//! ID、 after 1 new `<id>-max`（ `core::trae_model_ids::legacy_model_alias`）。
//!
//! ── Why 1 ──────────────────────────────────────────
//! directory and config old not in ， can ；「 use after 
//! 1 `-solo` 」 not can down 1 。 use config
//! `traeModelIdsMigrated` already （ and WorkBuddy 1 ）。
//!
//! ── 、 not ────────────────────────────────────
//! ： then ** up ID**（disabled / custom / capabilities / mappings.target
//! / seeded）、 by 、Key allowedModels、、lastRequestModel、
//! will model、Agent 、Trae and limit、directory。
//! not ：request / （ that is already ）， and ** use **（ has 
//! etc old ID when directory 1 ）。

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use crate::server::config;
use crate::server::core::account_store::AccountStore;
use crate::server::core::api_keys;
use crate::server::core::assistant_sessions;
use crate::server::core::model_rules::{self, CapabilityOverride, CustomModel, Mapping, RuleEntry};
use crate::server::core::plugin_host::rpc_cache;
use crate::server::core::providers::catalog_cache;
use crate::server::{access, logging};

const FLAG: &str = "traeModelIdsMigrated";
const TRAE: &str = "trae";
const AGENT_PROFILES: &str = "agentProfiles";

/// is already Trae old ID 。
pub(crate) fn migrated() -> bool {
    config::current()
        .raw()
        .get(FLAG)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

/// down 「 already 」（none has no has can need ， by module header）。
pub(crate) fn mark_migrated() -> bool {
    config::update_raw_field(FLAG, Value::Bool(true))
}

/// 。return `false` has no ， use ** not need **。
pub(crate) fn migrate(store: &AccountStore) -> bool {
    // ① directory「 old in 」 ， new ID。
    let mut mapping: HashMap<String, String> = HashMap::new();
    let mut cache_changed = migrate_cache("plugin:trae", &mut mapping);
    cache_changed += migrate_cache("trae", &mut mapping);

    // ② then / config old directory as ； no has 
    // after not ， up `-solo` 。
    let bases: HashSet<String> = mapping
        .values()
        .map(|base| base.to_ascii_lowercase())
        .collect();

    // ③ then / config / will / / 。
    let (rules_changed, rules_saved) = migrate_rules(&mapping);
    let config_saved = migrate_config(&mapping, &bases);
    let profiles_changed = migrate_agent_profiles(&mapping, &bases);
    let sessions_changed = assistant_sessions::migrate_model_ids(&mapping);
    let accounts_changed = store.migrate_trae_model_ids(&mapping);
    let memory_changed = rpc_cache::rename_models(TRAE, &mapping);

    let total = cache_changed
        + rules_changed
        + profiles_changed
        + sessions_changed
        + accounts_changed
        + memory_changed;
    if total > 0 {
        logging::log(
            "[Models]",
            &format!(
                "🧹 已清理 Trae 历史模型 ID 的 -solo / -solo-max 后缀（目录缓存 {cache_changed} 条、\
                 规则 {rules_changed} 条、助手方案 {profiles_changed} 条、会话 {sessions_changed} 条、\
                 账号限额 {accounts_changed} 个、运行中目录 {memory_changed} 条）"
            ),
        );
    }
    if !rules_saved || !config_saved {
        logging::log(
            "[Models]",
            "⚠️  Trae 历史模型 ID 清理未完全落盘：下次启动会重试（已改的部分是幂等的）",
        );
        return false;
    }
    true
}

/// directory old ID → new ID； when `id` 。
fn migrate_cache(scope: &str, mapping: &mut HashMap<String, String>) -> usize {
    let Some(cached) = catalog_cache::load(scope) else {
        return 0;
    };
    let mut models = cached.models.clone();
    let mut changed = 0usize;
    for entry in &mut models {
        rewrite_model_ids(entry, mapping, &mut changed);
    }
    if changed > 0 {
        catalog_cache::save(scope, &models, cached.fetched_at);
    }
    changed
}

fn rewrite_model_ids(
    value: &mut Value,
    mapping: &mut HashMap<String, String>,
    changed: &mut usize,
) {
    match value {
        Value::Array(items) => {
            for item in items {
                rewrite_model_ids(item, mapping, changed);
            }
        }
        Value::Object(object) => {
            if let Some(Value::String(id)) = object.get("id").cloned() {
                let was_max = strip_suffix(&id, "-solo-max").is_some();
                if let Some(target) = legacy_name(&id) {
                    mapping
                        .entry(id.trim().to_ascii_lowercase())
                        .or_insert_with(|| target.clone());
                    if !id.eq_ignore_ascii_case(&target) {
                        object.insert("id".to_string(), Value::String(target));
                        if was_max {
                            // old Max already is Max ， up 
                            // old `<id>-solo-max` request。
                            object.insert("traeMax".to_string(), Value::Bool(true));
                        }
                        *changed += 1;
                    }
                }
            }
            for (key, child) in object.iter_mut() {
                if key != "id" {
                    rewrite_model_ids(child, mapping, changed);
                }
            }
        }
        _ => {}
    }
}

fn migrate_rules(mapping: &HashMap<String, String>) -> (usize, bool) {
    let mut rules = model_rules::current();
    let mut changed = 0usize;
    let mapped = |name: &str| mapping.get(&name.trim().to_ascii_lowercase()).cloned();
    for entry in &mut rules.disabled {
        if is_trae(entry.provider.as_deref()) || entry.provider.is_none() {
            if let Some(base) = mapped(&entry.id) {
                entry.id = base;
                changed += 1;
            }
        }
    }
    for item in &mut rules.mappings {
        if let Some(base) = mapped(&item.alias) {
            item.alias = base;
            changed += 1;
        }
        if is_trae(item.provider.as_deref()) || item.provider.is_none() {
            if let Some(base) = mapped(&item.target) {
                item.target = base;
                changed += 1;
            }
        }
    }
    for item in &mut rules.custom {
        if item.provider.eq_ignore_ascii_case(TRAE) {
            if let Some(base) = mapped(&item.id) {
                item.id = base;
                changed += 1;
            }
        }
    }
    for item in &mut rules.capabilities {
        if item.provider.eq_ignore_ascii_case(TRAE) {
            if let Some(base) = mapped(&item.id) {
                item.id = base;
                changed += 1;
            }
        }
    }
    for key in &mut rules.seeded {
        if let Some((provider, id)) = key.split_once(':') {
            if provider.eq_ignore_ascii_case(TRAE) {
                if let Some(base) = mapped(id) {
                    *key = format!("{provider}:{base}");
                    changed += 1;
                }
            }
        }
    }
    changed += dedupe_disabled(&mut rules.disabled);
    changed += dedupe_mappings(&mut rules.mappings);
    changed += dedupe_custom(&mut rules.custom);
    changed += dedupe_capabilities(&mut rules.capabilities);
    changed += dedupe_seeded(&mut rules.seeded);
    if changed == 0 {
        return (0, true);
    }
    (changed, model_rules::save(&rules))
}

fn migrate_config(mapping: &HashMap<String, String>, bases: &HashSet<String>) -> bool {
    let mut changed = 0usize;
    let saved = config::update_raw_fields(|raw| {
        // by by config： is for 。
        if let Some(models) = raw
            .get_mut(config::KEY_ROUTING_PROFILES)
            .and_then(Value::as_object_mut)
            .and_then(|object| object.get_mut("models"))
            .and_then(Value::as_object_mut)
        {
            let keys: Vec<String> = models.keys().cloned().collect();
            for key in keys {
                let Some(base) = external_base(&key, mapping, bases) else {
                    continue;
                };
                if base.eq_ignore_ascii_case(&key) {
                    continue;
                }
                if let Some(value) = models.remove(&key) {
                    let base_key = base.to_ascii_lowercase();
                    if !models.contains_key(&base_key) {
                        models.insert(base_key, value);
                    }
                    changed += 1;
                }
            }
        }
        // Key （ Trae Key）。
        if let Some(items) = raw
            .get_mut(api_keys::KEY_API_KEYS)
            .and_then(Value::as_array_mut)
        {
            for item in items {
                let Some(object) = item.as_object_mut() else {
                    continue;
                };
                let (trae_allowed, trae_only) = match object.get("allowedProviders") {
                    None => (true, false),
                    Some(Value::Array(list)) if list.is_empty() => (true, false),
                    Some(Value::Array(list)) => (
                        list.iter().any(|value| {
                            value
                                .as_str()
                                .is_some_and(|id| id.eq_ignore_ascii_case(TRAE))
                        }),
                        list.iter().all(|value| {
                            value
                                .as_str()
                                .is_some_and(|id| id.eq_ignore_ascii_case(TRAE))
                        }),
                    ),
                    Some(_) => (false, false),
                };
                if !trae_allowed {
                    continue;
                }
                let Some(models) = object
                    .get_mut("allowedModels")
                    .and_then(Value::as_array_mut)
                else {
                    continue;
                };
                let before = models.clone();
                rewrite_names(models, mapping, bases, trae_only);
                if *models != before {
                    changed += 1;
                }
            }
        }
        // / 。
        if let Some(object) = raw
            .get_mut(config::KEY_ASSISTANT_CONFIG)
            .and_then(Value::as_object_mut)
        {
            for key in ["model", "titleModel"] {
                if let Some(text) = object.get(key).and_then(Value::as_str).map(str::to_string) {
                    if let Some(base) = external_base(&text, mapping, bases) {
                        object.insert(key.to_string(), Value::String(base));
                        changed += 1;
                    }
                }
            }
        }
        // 「」 default。
        if let Some(text) = raw
            .get("lastRequestModel")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            if let Some(base) = external_base(&text, mapping, bases) {
                raw.insert("lastRequestModel".to_string(), Value::String(base));
                changed += 1;
            }
        }
    });
    if changed > 0 && saved {
        config::reload();
    }
    saved
}

fn migrate_agent_profiles(mapping: &HashMap<String, String>, bases: &HashSet<String>) -> usize {
    let Some(text) = access::kv_get(AGENT_PROFILES) else {
        return 0;
    };
    let Ok(mut value) = serde_json::from_str::<Value>(&text) else {
        return 0;
    };
    let mut changed = 0usize;
    if let Some(clients) = value.as_object_mut() {
        for profiles in clients.values_mut() {
            let Some(items) = profiles.as_array_mut() else {
                continue;
            };
            for profile in items {
                let Some(object) = profile.as_object_mut() else {
                    continue;
                };
                rewrite_json_name(object, "model", mapping, bases, &mut changed);
                if let Some(models) = object.get_mut("models").and_then(Value::as_array_mut) {
                    for model in models {
                        if let Some(model) = model.as_object_mut() {
                            rewrite_json_name(model, "id", mapping, bases, &mut changed);
                        }
                    }
                }
                if let Some(claude) = object
                    .get_mut("claudeModels")
                    .and_then(Value::as_object_mut)
                {
                    for key in ["opus", "sonnet", "haiku", "subagent"] {
                        rewrite_json_name(claude, key, mapping, bases, &mut changed);
                    }
                }
            }
        }
    }
    if changed > 0 {
        access::kv_put(AGENT_PROFILES, &value.to_string());
    }
    changed
}

fn rewrite_json_name(
    object: &mut Map<String, Value>,
    key: &str,
    mapping: &HashMap<String, String>,
    bases: &HashSet<String>,
    changed: &mut usize,
) {
    let Some(text) = object.get(key).and_then(Value::as_str).map(str::to_string) else {
        return;
    };
    if let Some(base) = external_base(&text, mapping, bases) {
        object.insert(key.to_string(), Value::String(base));
        *changed += 1;
    }
}

fn rewrite_names(
    items: &mut Vec<Value>,
    mapping: &HashMap<String, String>,
    bases: &HashSet<String>,
    suffix_fallback: bool,
) {
    for item in items.iter_mut() {
        if let Some(text) = item.as_str() {
            if let Some(base) = external_base(text, mapping, bases).or_else(|| {
                suffix_fallback
                    .then(|| legacy_name(text))
                    .flatten()
            }) {
                *item = Value::String(base);
            }
        }
    }
    let mut seen = HashSet::new();
    items.retain(|item| {
        item.as_str()
            .map(|text| seen.insert(text.trim().to_ascii_lowercase()))
            .unwrap_or(true)
    });
}

/// for ： use directory to old ； when in in when 
/// by after （ use `my-solo` this ）。
fn external_base(
    name: &str,
    mapping: &HashMap<String, String>,
    bases: &HashSet<String>,
) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(base) = mapping.get(&trimmed.to_ascii_lowercase()) {
        return Some(base.clone());
    }
    // this is when ， old can can is （ up -solo ），
    // not can by after 。
    if bases.contains(&trimmed.to_ascii_lowercase()) {
        return None;
    }
    let target = legacy_name(trimmed)?;
    bases
        .contains(&target.to_ascii_lowercase())
        .then_some(target)
}

fn is_trae(provider: Option<&str>) -> bool {
    provider.is_some_and(|provider| provider.eq_ignore_ascii_case(TRAE))
}

/// old `<id>-solo` → ID； old `<id>-solo-max` → new `<id>-max`。
fn legacy_name(name: &str) -> Option<String> {
    if let Some(base) = strip_suffix(name, "-solo-max") {
        return Some(format!("{base}-max"));
    }
    strip_suffix(name, "-solo").map(str::to_string)
}

fn strip_suffix<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let name = name.trim();
    let cut = name.len().checked_sub(suffix.len())?;
    let base = name.get(..cut)?;
    let tail = name.get(cut..)?;
    (tail.eq_ignore_ascii_case(suffix) && !base.is_empty()).then_some(base)
}

fn dedupe_disabled(items: &mut Vec<RuleEntry>) -> usize {
    let mut seen = HashSet::new();
    let before = items.len();
    items.retain(|item| {
        seen.insert((
            item.provider.as_deref().unwrap_or("").to_ascii_lowercase(),
            item.id.to_ascii_lowercase(),
        ))
    });
    before - items.len()
}

fn dedupe_mappings(items: &mut Vec<Mapping>) -> usize {
    let mut seen = HashSet::new();
    let before = items.len();
    items.retain(|item| {
        seen.insert((
            item.alias.to_ascii_lowercase(),
            item.target.to_ascii_lowercase(),
            item.provider.as_deref().unwrap_or("").to_ascii_lowercase(),
            item.reasoning.as_deref().unwrap_or("").to_ascii_lowercase(),
            item.enabled,
        ))
    });
    before - items.len()
}

fn dedupe_custom(items: &mut Vec<CustomModel>) -> usize {
    let mut seen = HashSet::new();
    let before = items.len();
    items.retain(|item| {
        seen.insert((
            item.provider.to_ascii_lowercase(),
            item.id.to_ascii_lowercase(),
        ))
    });
    before - items.len()
}

fn dedupe_capabilities(items: &mut Vec<CapabilityOverride>) -> usize {
    let mut seen = HashSet::new();
    let before = items.len();
    items.retain(|item| {
        seen.insert((
            item.provider.to_ascii_lowercase(),
            item.id.to_ascii_lowercase(),
        ))
    });
    before - items.len()
}

fn dedupe_seeded(items: &mut Vec<String>) -> usize {
    let mut seen = HashSet::new();
    let before = items.len();
    items.retain(|item| seen.insert(item.to_ascii_lowercase()));
    before - items.len()
}

/// 1 ： old clientrequest `<id>-solo` when ID，`<id>-solo-max` 
/// new `<id>-max`。 Trae in ， 
/// `foo-solo` ；Max when to ， not request。
pub(crate) fn legacy_model_alias(model: &str) -> Option<String> {
    let target = legacy_name(model)?;
    // old is （ or use ） when 。
    if !crate::server::core::providers::catalog::providers_for_model(model).is_empty() {
        return None;
    }
    let manifest = crate::server::core::providers::catalog::manifest_for_provider(TRAE);
    if let Some(base) = strip_suffix(model, "-solo-max") {
        // `<id>-max` in Max up ； up 
        // not can old `<id>-solo-max` request use 。
        let ours = manifest.iter().any(|entry| {
            entry
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.eq_ignore_ascii_case(&target))
                && entry.get("traeMax").and_then(Value::as_bool) == Some(true)
        });
        if ours {
            return Some(target);
        }
        return crate::server::core::providers::catalog::entry_id_in_manifest(&manifest, base);
    }
    crate::server::core::providers::catalog::entry_id_in_manifest(&manifest, &target)
}
