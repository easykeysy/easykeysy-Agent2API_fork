//! and for 。all for use 。

use std::collections::HashSet;

use serde_json::{json, Map, Value};

use crate::server::core::account_store::AccountStore;
use crate::server::core::capability;
use crate::server::core::key_scope::{self, KeyScope};
use crate::server::core::model_meta;
use crate::server::core::model_rules;
use crate::server::core::models::{list_item, list_response_from, model_id, suggest_from};

use super::routing::{builtin_target, entry_id_in_manifest};
use super::{active_manifests, aggregate_source, managed_manifests, manifest_for_provider, refresh_meta_for_provider};

/// 1 will for ： up id + ** already enabled** 。
///
/// ── `enabled` this 1 not can （ ）──────────────────
/// when ， will ， after in [`builtin_target`] 
/// 「」 1 ：`entry_id_in_manifest` by id not to will 
/// ** to for up 1 （`name`）**， in after by `default_enabled` 
/// —— that and none。 is 「、`/v1/models` in 」。
/// ：`space-bunny-alpha`（ and ） will in ，`mimo-v2.6-flash`
/// （ is `Mimo V2.6 Flash`， not in ） not will in —— 1 。
/// module header that 「all for use 」 is this must。
fn advertised_names(
    rules: &model_rules::ModelRules,
    provider: &str,
    manifest: &[Value],
) -> Vec<String> {
    let mut names: Vec<String> = manifest.iter().map(model_id).collect();
    names.extend(rules.mappings.iter()
        .filter(|mapping| mapping.enabled)
        .filter(|mapping| mapping.provider.as_deref().map_or(true, |owner| owner.eq_ignore_ascii_case(provider)))
        .map(|mapping| mapping.alias.clone()));
    names
}

/// in ， by for ； not can and up ID ，
/// otherwise will 1 up new in in 。
fn public_builtin_items(active: &[(String, Vec<Value>)]) -> Vec<(String, Value)> {
    let rules = model_rules::shared();
    let mut claimed = std::collections::HashMap::<String, usize>::new();
    let mut result: Vec<(String, Value)> = Vec::new();
    for (provider, manifest) in active {
        let names = advertised_names(&rules, provider, manifest);
        for name in names {
            let Some(wire) = builtin_target(&rules, provider, manifest, &name) else { continue };
            let Some(upstream) = manifest.iter().find(|item| model_id(item) == wire.model) else { continue };
            let mut item = list_item(upstream, provider);
            if let Some(index) = claimed.get(&name.to_lowercase()).copied() {
                let existing = &mut result[index].1;
                if item.get("supports_image_generation").and_then(Value::as_bool) == Some(true)
                    && existing.get("kind").and_then(Value::as_str) == Some("image")
                { existing["supports_image_generation"] = Value::Bool(true); }
                if item.get("supports_image_editing").and_then(Value::as_bool) == Some(true)
                    && existing.get("kind").and_then(Value::as_str) == Some("image")
                {
                    existing["supports_image_editing"] = Value::Bool(true);
                    existing["supports_images"] = Value::Bool(true);
                    existing["input_modalities"] = json!(["text", "image"]);
                }
                continue;
            }
            claimed.insert(name.to_lowercase(), result.len());
            if let Some(object) = item.as_object_mut() {
                object.insert("id".to_string(), Value::String(name.clone()));
                if !name.eq_ignore_ascii_case(&wire.model) {
                    object.insert("name".to_string(), Value::String(name));
                    object.insert("is_default".to_string(), Value::Bool(false));
                }
            }
            result.push((provider.to_string(), item));
        }
    }
    result
}

pub fn models_response(store: &AccountStore, scope: Option<&KeyScope>) -> Value {
    models_response_with_active(store, scope, active_manifests(store))
}

pub(super) fn models_response_with_active(
    store: &AccountStore,
    scope: Option<&KeyScope>,
    active: Vec<(String, Vec<Value>)>,
) -> Value {
    let active: Vec<_> = active.into_iter()
        .filter(|(provider, _)| key_scope::allows_provider(scope, provider))
        .collect();
    let (source, refreshed_at) = aggregate_source(&active);
    let mut data: Vec<_> = public_builtin_items(&active).into_iter()
        .map(|(_, item)| item)
        .filter(|item| key_scope::allows_model(scope, &model_id(item)))
        .collect();
    let builtin_count = data.len();
    super::super::custom::append_models_response(store, scope, &mut data);
    crate::server::core::routing_groups::append_public(store, scope, &mut data);
    let source = if data.len() > builtin_count { "aggregate" } else { source };
    list_response_from(data, source, refreshed_at)
}

/// 1 available no has when （ no ）， need 「 has 」 and 
/// Key ， 「 no has available」 that can error to ——
/// and before （ `pipeline::resolve_model` ）。
///
/// and runtime ID available， not can only no has old 。
pub fn has_available_providers(store: &AccountStore) -> bool {
    super::validation_snapshot(store).available
}

pub fn advertised_model_ids(store: &AccountStore) -> Vec<String> {
    super::validation_snapshot(store).models.clone()
}

pub fn advertised_manifest_contains(store: &AccountStore, model: &str) -> bool {
    super::validation_snapshot(store).models.iter().any(|id| id.eq_ignore_ascii_case(model))
}

pub fn suggest_advertised(store: &AccountStore, model: &str, limit: usize) -> Vec<String> {
    suggest_from(advertised_model_ids(store), model, limit)
}

pub fn session_models(store: &AccountStore) -> Vec<Value> {
    models_response(store, None).get("data").and_then(Value::as_array)
        .cloned().unwrap_or_default().into_iter().map(|mut item| {
            let provider = item.get("owned_by").and_then(Value::as_str).unwrap_or("").to_string();
            let is_default = item.get("is_default").cloned().unwrap_or(Value::Bool(false));
            if let Some(object) = item.as_object_mut() {
                object.insert("providerLabel".to_string(), Value::String(super::super::label_of(&provider)));
                object.insert("provider".to_string(), Value::String(provider));
                object.insert("isDefault".to_string(), is_default);
            }
            item
        }).collect()
}

pub fn models_by_provider(store: &AccountStore) -> Value {
    let mut map = Map::new();
    for (provider, manifest) in active_manifests(store) {
        let mut names: Vec<_> = public_builtin_items(&[(provider.clone(), manifest)]).into_iter()
            .map(|(_, item)| model_id(&item)).collect();
        names.sort_by_key(|name| name.to_lowercase());
        map.insert(provider, json!(names));
    }
    for (provider, items) in super::super::custom::catalog_providers(store) {
        let mut names: Vec<_> = items.iter().map(model_id).collect();
        names.sort_by_key(|name| name.to_lowercase());
        map.insert(provider, json!(names));
    }
    for group in crate::server::config::routing_profiles().groups.values() {
        for member in group.members.iter().filter(|member| member.included) {
            if let Some(names) = map.get_mut(&member.provider).and_then(Value::as_array_mut) {
                if !names.iter().any(|name| name.as_str().is_some_and(|name| name.eq_ignore_ascii_case(&group.model))) {
                    names.push(group.model.clone().into());
                }
            }
        }
    }
    Value::Object(map)
}

/// 1 （`models` 1 ）+ for default。
#[allow(clippy::too_many_arguments)]
fn push_managed_model(
    rules: &model_rules::ModelRules,
    models: &mut Vec<Value>,
    mappings: &mut Vec<Value>,
    attached: &mut HashSet<usize>,
    claimed: &mut HashSet<(String, String)>,
    provider: &str,
    source: &str,
    refreshed_at: i64,
    manifest: &[Value],
    item: Value,
) {
    let id = model_id(&item);
    if id.is_empty() {
        return;
    }
    if !claimed.insert((provider.to_string(), id.to_lowercase())) {
        return; // 同家同 id 的重复条目（见 `manage_view` 里 claimed 的说明）
    }
    let default = rules.binding(provider, &id, &id);
    let enabled = rules.default_enabled(provider, &id);
    mappings.push(json!({
        "alias": id, "target": id, "provider": provider, "enabled": enabled,
        "reasoning": default.and_then(|mapping| mapping.reasoning.clone()),
        "isDefault": true, "dangling": false, "carried": true,
    }));
    let mut aliases = Vec::new();
    for (index, mapping) in rules.mappings.iter().enumerate() {
        if !mapping.target.eq_ignore_ascii_case(&id)
            || !mapping.provider.as_deref().map_or(true, |owner| owner.eq_ignore_ascii_case(provider))
        { continue; }
        attached.insert(index);
        if mapping.alias.eq_ignore_ascii_case(&id)
            || aliases.iter().any(|alias: &String| alias.eq_ignore_ascii_case(&mapping.alias))
        { continue; }
        let Some(effective) = rules.binding(provider, &mapping.alias, &id) else { continue };
        aliases.push(mapping.alias.clone());
        mappings.push(json!({
            "alias": effective.alias, "target": id, "provider": provider,
            "enabled": rules.binding_enabled(provider, &effective.alias, &id, manifest), "reasoning": effective.reasoning,
            "isDefault": false, "dangling": false, "carried": true,
        }));
    }
    let source = if item.get("_manualModel").and_then(Value::as_bool) == Some(true) {
        "manual"
    } else { source };
    // from then ， not new getconfig。
    let cap_overrides: Vec<String> = rules
        .capabilities
        .iter()
        .find(|entry| entry.matches(provider, &id))
        .map(|entry| capability::overridden_keys(&entry.values))
        .unwrap_or_default();
    models.push(json!({
        "id": id, "name": item.get("name"), "credits": item.get("credits"),
        "isDefault": item.get("isDefault").and_then(Value::as_bool).unwrap_or(false),
        "provider": provider, "providerLabel": super::super::label_of(provider),
        "source": source, "enabled": enabled, "aliases": aliases,
        "capabilities": capability::effective(&item),
        "kind": if super::super::images::model_is_image(&item) { json!("image") }
            else { item.get("kind").cloned().unwrap_or(json!("chat")) },
        "output_modalities": item.get("output_modalities"),
        "supports_image_generation": super::super::images::provider_model_supports_generation(provider, &item),
        "supports_image_editing": super::super::images::provider_model_supports_editing(provider, &item),
        "capOverrides": cap_overrides,
        // can ****（`upstream` / `modelsdev` / `user`，
        // `core::model_meta` module header）。 「 up 」「
        // 」「 use 」 —— no has ， 1 
        // can ， can 。（ no has this ）= that 1 no has 
        // （）。
        "capSources": item.get("capSources"),
        // ** can **（， by `core::model_meta`
        // in 1 ： this > models.dev effort ）。
        // and up `mappings[].reasoning`（ use to **** ）
        // is ： this 「 this supports」， that 「 this 
        // use 1 」。（null）= this ， use 。
        "reasoningLevels": item.get("reasoningLevels"),
        "reasoningDefaultLevel": item.get("reasoningDefaultLevel"),
        "reasoningFixedLevel": item.get("reasoningFixedLevel"),
        "reasoningBindingSupported": item.get("reasoningBindingSupported"),
        "reasoningSource": item.get("reasoningSource"),
        // this 1 is when to （ms；0 = from success）。
        // and ** is `remote`**， when can this 
        // when （ `providers::catalog_cache` module header）。
        "refreshedAt": refreshed_at,
        // ** can **： this in client is will 。
        //
        // `"shown"` / `"hidden"` / `"undeclared"` ， by `models.rs`
        // in （ not ：Antigravity by 、Qoder 
        // up `enable`、Raccoon `visible`、CodeArts `display_enabled`）。
        // not new —— not then ，。
        //
        // （`null`）= this no has can （current is up all
        // provider）。 `—`，** not ** 「 can 」 or 「 not can 」：
        // 「 this no this 」 and 「 this not can 」 is 。
        "upstreamStatus": item.get("upstreamStatus"),
        "visibility": item.get("visibility").cloned().unwrap_or(Value::Null),
    }));
}

/// ID return 1 default； and default， 
/// 1 request。default not can delete， not this 1 its 。
///
/// ── can （`capabilities` / `capOverrides`）─────────────────
/// can （ 「 up down / 」 and 「 can 」）：
/// · `capabilities`：****（ + use ，，
/// `null` = —— and 「unsupported / 0」， use when need 
/// 「 not 」 and 「unsupported」）；
/// · `capOverrides`： use （ to 「 already 」；
/// empty = ）。
/// already by `manifest_for_provider` use ， this to is 。
/// `(provider, target)` can not can in up to 1 。
///
/// and [`manage_view`] `dangling` that path****： is 
/// [`managed_manifests`]（ has available ）， `provider` in that 、
/// old （provider ） to target 。
///
/// ── use ：（default） can not can ─────────────────────────
/// `POST /api/models/mappings/remove` use 1 `alias == target` default
/// can not can 。default， can 、 not can ；** already not in **
/// （ up down 、 that or no has available） when ， no has ，delete is 1 
/// —— 「 」 is to this （ [`manage_view`]
/// dangling ）。 this and use 1 「 has no has 」 ，
/// 「 as 、delete after 」 。
pub(crate) fn host_row_exists(store: &AccountStore, provider: Option<&str>, target: &str) -> bool {
    managed_manifests(store)
        .into_iter()
        .any(|(owner, manifest)| {
            provider.map_or(true, |provider| owner.eq_ignore_ascii_case(provider))
                && manifest.iter().any(|item| model_id(item).eq_ignore_ascii_case(target))
        })
}

pub fn manage_view(store: &AccountStore) -> Value {
    let rules = model_rules::shared();
    let mut models = Vec::new();
    let mut mappings = Vec::new();
    let mut providers = Vec::new();
    let mut attached = HashSet::new();
    // `(provider, id )` （ #177 1 ）。
    //
    // directory**** in （ Trae `parse_catalog` by 
    // `config_name`）， up when can can 、 has —— 
    // is 「 can check」， id will down that default 1 。
    // this 1 ： not directory， this 1 that 。
    let mut claimed: HashSet<(String, String)> = HashSet::new();
    // ** **（= has available ）。empty not in this 
    // when not —— up no has this 1 （ not will has ）， down 
    // will 1 use in this none from down （ need that ）。
    let mut managed_providers = HashSet::new();
    for (provider, mut manifest) in managed_manifests(store) {
        managed_providers.insert(provider.to_lowercase());
        manifest.sort_by_key(|item| !rules.default_enabled(&provider, &model_id(item)));
        let (remote, refreshed_at) = refresh_meta_for_provider(&provider);
        let source = if remote { "remote" } else { "builtin" };
        let count = manifest.iter().map(model_id).filter(|id| !id.is_empty())
            .map(|id| id.to_lowercase()).collect::<HashSet<_>>().len();
        providers.push(json!({
            "provider": provider,
            "providerLabel": super::super::label_of(&provider),
            "count": count,
            "source": if count == 0 { "none" } else { source },
            "refreshedAt": refreshed_at,
        }));
        for item in manifest.iter().cloned() {
            push_managed_model(
                &rules, &mut models, &mut mappings, &mut attached, &mut claimed,
                &provider, source, refreshed_at, &manifest, item,
            );
        }
    }
    for (index, mapping) in rules.mappings.iter().enumerate() {
        if attached.contains(&index) { continue; }
        // not in （ `managed_providers` ）。 old 
        // （provider ） not 1 ， not this —— is 「 for all」
        // ， no has 「 1 no 」 this 。
        if let Some(provider) = mapping.provider.as_deref() {
            if !managed_providers.contains(&provider.to_lowercase()) {
                continue;
            }
        }
        let carried = match mapping.provider.as_deref() {
            Some(provider) => entry_id_in_manifest(&manifest_for_provider(provider), &mapping.target).is_some(),
            None => !super::providers_for_model(&mapping.target).is_empty(),
        };
        let enabled = mapping.provider.as_deref().map_or(mapping.enabled, |provider| {
            rules.binding_enabled(provider, &mapping.alias, &mapping.target, &manifest_for_provider(provider))
        });
        mappings.push(json!({
            "alias": mapping.alias, "target": mapping.target, "provider": mapping.provider,
            "enabled": enabled, "reasoning": mapping.reasoning,
            "isDefault": mapping.alias.eq_ignore_ascii_case(&mapping.target),
            "dangling": true, "carried": carried,
        }));
    }
    json!({
        "models": models,
        "mappings": mappings,
        "providers": providers,
        "reasoningLevels": model_rules::REASONING_LEVELS,
        // models.dev current（`builtin` / `cache` / `network` + 
        // when + ）。 is 「 this / can is when 」 this ——
        // up that 1 、 new not new ， can by this 。
        "modelsDev": model_meta::status(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `from_raw` is config（ `modelRules` ）， 1 
    fn rules_with(mappings: Value) -> model_rules::ModelRules {
        let raw = json!({ "modelRules": { "mappings": mappings } });
        model_rules::ModelRules::from_raw(raw.as_object().expect("对象字面量"))
    }

    /// 1 and up （`space-bunny-alpha` is in that ）
    fn manifest() -> Vec<Value> {
        vec![json!({ "id": "stealth/space-bunny-alpha", "name": "space-bunny-alpha" })]
    }

    /// not can 。 this when ， will by 
    /// `builtin_target` 「」 —— is 「，
    /// `/v1/models` in 」（2026-10-03 ）。
    #[test]
    fn disabled_mapping_alias_is_not_advertised() {
        let rules = rules_with(json!([
            { "alias": "space-bunny-alpha", "target": "stealth/space-bunny-alpha",
              "provider": "cline-free", "enabled": false }
        ]));
        let names = advertised_names(&rules, "cline-free", &manifest());
        assert!(
            !names.iter().any(|name| name == "space-bunny-alpha"),
            "关掉的别名不该出现在候选名里: {names:?}"
        );
        // id not （）
        assert!(names.iter().any(|name| name == "stealth/space-bunny-alpha"));
    }

    /// old —— not can this can 1 no 
    #[test]
    fn enabled_mapping_alias_is_advertised() {
        let rules = rules_with(json!([
            { "alias": "space-bunny-alpha", "target": "stealth/space-bunny-alpha",
              "provider": "cline-free", "enabled": true }
        ]));
        let names = advertised_names(&rules, "cline-free", &manifest());
        assert!(names.iter().any(|name| name == "space-bunny-alpha"));
    }

    /// `enabled` = can up before = defaultenabled（ not as ）
    #[test]
    fn mapping_without_enabled_key_is_advertised() {
        let rules = rules_with(json!([
            { "alias": "space-bunny-alpha", "target": "stealth/space-bunny-alpha",
              "provider": "cline-free" }
        ]));
        let names = advertised_names(&rules, "cline-free", &manifest());
        assert!(names.iter().any(|name| name == "space-bunny-alpha"));
    }

    /// not this this 1 （provider in ）
    #[test]
    fn other_providers_alias_is_not_included() {
        let rules = rules_with(json!([
            { "alias": "x-alias", "target": "stealth/space-bunny-alpha",
              "provider": "workbuddy", "enabled": true }
        ]));
        let names = advertised_names(&rules, "cline-free", &manifest());
        assert!(!names.iter().any(|name| name == "x-alias"));
    }
}
