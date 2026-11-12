//! for to up 。 by and use ， in 1 after 1 then 。

use serde_json::Value;

use crate::server::core::{custom_providers, model_rules};
use crate::server::core::models::model_id;
use crate::server::core::providers::adapter::adapter_by_id;

use super::{all_provider_ids, manifest_for_provider};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WireTarget {
    pub model: String,
    pub reasoning: Option<String>,
}

pub(crate) fn entry_id_in_manifest(manifest: &[Value], requested: &str) -> Option<String> {
    let requested = requested.trim();
    if requested.is_empty() {
        return None;
    }
    // ，must ID；return 。
    manifest.iter().find(|entry| model_id(entry).eq_ignore_ascii_case(requested))
        .or_else(|| manifest.iter().find(|entry| {
            entry.get("name")
                .map(crate::server::core::models::shape_value_text)
                .is_some_and(|name| name.eq_ignore_ascii_case(requested))
        }))
        .map(model_id)
}

pub(super) fn builtin_target(
    rules: &model_rules::ModelRules,
    provider: &str,
    manifest: &[Value],
    requested: &str,
) -> Option<WireTarget> {
    let requested = requested.trim();
    let exact = manifest.iter()
        .find(|entry| model_id(entry).eq_ignore_ascii_case(requested))
        .map(model_id);
    let mappings: Vec<_> = rules.mappings.iter()
        .filter(|mapping| mapping.alias.eq_ignore_ascii_case(requested))
        .filter(|mapping| mapping.provider.as_deref()
            .map_or(true, |id| id.eq_ignore_ascii_case(provider)))
        .collect();
    let default = exact.or_else(|| {
        if mappings.is_empty() { entry_id_in_manifest(manifest, requested) } else { None }
    });
    if let Some(id) = default {
        if rules.default_enabled(provider, &id) {
            return Some(WireTarget {
                reasoning: rules.binding(provider, &id, &id).and_then(|m| m.reasoning.clone()),
                model: id,
            });
        }
    }
    let mapping = rules.selected_mapping(provider, requested, manifest)?;
    entry_id_in_manifest(manifest, &mapping.target).map(|model| WireTarget {
        model,
        reasoning: mapping.reasoning.clone(),
    })
}

/// can not use ， API old then 。
/// ，must can to this up 。
pub fn providers_for_model(model: &str) -> Vec<String> {
    let model = model.trim();
    if model.is_empty() {
        return Vec::new();
    }
    let rules = model_rules::shared();
    let manifests: Vec<_> = all_provider_ids().into_iter()
        .map(|provider| {
            let manifest = manifest_for_provider(&provider);
            (provider, manifest)
        })
        .collect();
    let exact = manifests.iter().any(|(_, items)| {
        items.iter().any(|item| model_id(item).eq_ignore_ascii_case(model))
    });
    let mut result: Vec<_> = manifests.into_iter().filter(|(provider, items)| {
        let declared = if exact {
            items.iter().any(|item| model_id(item).eq_ignore_ascii_case(model))
        } else {
            entry_id_in_manifest(items, model).is_some()
        };
        let mapped = rules.mappings.iter().any(|mapping| {
            mapping.alias.eq_ignore_ascii_case(model)
                && mapping.provider.as_deref().map_or(true, |owner| owner.eq_ignore_ascii_case(provider))
                && entry_id_in_manifest(items, &mapping.target).is_some()
        });
        declared || mapped
    }).map(|(provider, _)| provider).collect();
    result.extend(custom_providers::list().into_iter().filter_map(|provider| {
        let models = provider.get("models")?.as_array()?;
        let declared = models.iter().any(|entry| model_id(entry).eq_ignore_ascii_case(model));
        let mapped = provider
            .get("mappings")
            .and_then(Value::as_array)
            .is_some_and(|mappings| {
                mappings.iter().any(|mapping| {
                    mapping
                        .get("alias")
                        .and_then(Value::as_str)
                        .is_some_and(|alias| alias.eq_ignore_ascii_case(model))
                })
            });
        (declared || mapped)
            .then(|| provider.get("id").and_then(Value::as_str).unwrap_or("").to_string())
    }));
    result
}

/// return has ， not ID to default。
pub fn forwarding_providers(model: &str) -> Vec<String> {
    let rules = model_rules::shared();
    let mut result: Vec<_> = all_provider_ids().into_iter().filter(|provider| {
        builtin_target(&rules, provider, &manifest_for_provider(provider), model).is_some()
    }).collect();
    result.extend(custom_providers::carriers_of_model(model));
    result
}

/// 「 this is not is 」—— in no has when use 「 」
/// and 「 no has this 」 404 （`pipeline::resolve_model` and 
/// `provider_loop::forward_with_providers` 1 ）。
///
/// ── Why `providers_for_model`（ already ）──────
/// this 「 need ** in ** 」： 
/// `has_alias(model)` and mappings alias 。 in 「 up 
/// down 、 already not in 」 when will ** in ** —— in 
/// config， to up already no ， not ， not this 「 already in in 」：
/// that will use 1 ** already not in ** 。
/// 「 in 、 in 、 is 」 already 
/// [`providers_for_model`] —— that ** not `enabled`**（ ），
/// need has empty， this not will ； down empty
/// 「 not in 」， and 1 。
pub fn model_blocked_everywhere(model: &str) -> bool {
    if !forwarding_providers(model).is_empty() {
        return false;
    }
    !providers_for_model(model).is_empty()
}

pub fn wire_target_for_provider(model: &str, provider_id: &str, _account: Option<&Value>) -> WireTarget {
    let requested = model.trim();
    // in 1 ， before after its etc 。
    let resolved = if custom_providers::is_custom_provider_id(provider_id) {
        None
    } else {
        builtin_target(&model_rules::shared(), provider_id, &manifest_for_provider(provider_id), requested)
    };
    resolved.unwrap_or_else(|| WireTarget { model: requested.to_string(), reasoning: None })
}

pub fn default_model_catalog() -> Vec<Value> {
    let rules = model_rules::shared();
    let mut result = Vec::new();
    for provider in all_provider_ids().into_iter().filter(|provider| {
        adapter_by_id(provider).is_some_and(|adapter| adapter.supports_default_model())
    }) {
        for entry in manifest_for_provider(&provider) {
            let id = model_id(&entry);
            if !id.is_empty() && !super::super::images::model_is_image(&entry)
                && rules.default_enabled(&provider, &id)
                && !result.iter().any(|item| model_id(item).eq_ignore_ascii_case(&id))
            {
                result.push(entry);
            }
        }
    }
    result
}

pub fn default_model_usable(model: &str) -> bool {
    entry_id_in_manifest(&default_model_catalog(), model).is_some()
}
