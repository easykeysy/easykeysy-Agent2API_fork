//! directory： 、 for and 。
//!
//! ID is default， is 。；default
//! not will use to up ， not will 。
//! in file， in routing.rs， for and in view.rs。

mod routing;
mod snapshot;
mod view;

pub(crate) use snapshot::{config_changed, generation, invalidate, validation_snapshot, ValidationCache};

use serde_json::Value;

use crate::server::core::account_store::AccountStore;
use crate::server::core::model_meta;
use crate::server::core::model_rules;
use crate::server::core::models::model_id;
use crate::server::core::providers::adapter::adapter_by_id;
use crate::server::core::providers::{kind_from_id, kind_id, ProviderKind, PROVIDERS};

pub(crate) use routing::entry_id_in_manifest;
pub use routing::{
    default_model_catalog, default_model_usable, forwarding_providers, model_blocked_everywhere,
    providers_for_model, wire_target_for_provider, WireTarget,
};
pub use view::{
    advertised_manifest_contains, advertised_model_ids, has_available_providers, manage_view,
    models_by_provider, models_response, session_models, suggest_advertised,
};
/// 「 this in up has no has can 」——delete when （）。
/// 1 use ， not for `pub use`。
pub(crate) use view::host_row_exists;

pub(crate) fn upstream_manifests() -> Vec<(String, Vec<Value>)> {
    all_provider_ids().into_iter().map(|provider| {
        let models = adapter_by_id(&provider).map(|adapter| adapter.list_models()).unwrap_or_default();
        (provider, models)
    }).collect()
}

/// can not use ； ID after ， need to this up 。
/// use 、models.dev and can in this 1 use ， and use 。
pub(crate) fn manifest_for_provider(provider: &str) -> Vec<Value> {
    let Some(adapter) = adapter_by_id(provider) else {
        return Vec::new();
    };
    let mut models = adapter.list_models();
    for item in model_rules::custom_models_for(provider) {
        let id = model_id(&item);
        if !id.is_empty()
            && !models.iter().any(|existing| model_id(existing).eq_ignore_ascii_case(&id))
        {
            let mut item = item;
            item["_manualModel"] = Value::Bool(true);
            models.push(item);
        }
    }
    for item in models.iter_mut() {
        model_meta::apply(item);
    }
    model_rules::apply_capability_overrides(provider, &mut models);
    models
}

/// use by kind ， already 。
pub(crate) fn manifest_for(kind: ProviderKind) -> Vec<Value> {
    manifest_for_provider(kind_id(kind))
}

/// old directory already ， not has by kind new 。
pub(crate) fn refresh_meta(_kind: ProviderKind) -> (bool, i64) {
    (false, 0)
}

pub(crate) fn refresh_meta_for_provider(provider: &str) -> (bool, i64) {
    if crate::server::core::plugin_host::serves_provider(provider) {
        let meta = crate::server::core::plugin_host::model_catalog_meta(provider);
        return (meta.remote, meta.refreshed_at);
    }
    kind_from_id(provider).map(refresh_meta).unwrap_or_default()
}

fn all_provider_ids() -> Vec<String> {
    let mut ids = super::executable_provider_ids();
    for provider in PROVIDERS {
        if !ids.iter().any(|id| id.eq_ignore_ascii_case(provider.id)) {
            ids.push(provider.id.to_string());
        }
    }
    ids
}

pub fn provider_available(store: &AccountStore, provider: &str) -> bool {
    store.routing_snapshot().available_providers.contains(provider)
}

/// need need ； for in up use 。
pub(crate) fn managed_manifests(store: &AccountStore) -> Vec<(String, Vec<Value>)> {
    let accounts = store.routing_snapshot();
    all_provider_ids()
        .into_iter()
        .filter(|provider| accounts.available_providers.contains(provider))
        .map(|provider| {
            let manifest = manifest_for_provider(&provider);
            (provider, manifest)
        })
        .collect()
}

pub fn active_manifests(store: &AccountStore) -> Vec<(String, Vec<Value>)> {
    managed_manifests(store).into_iter().filter_map(|(provider, manifest)| {
        let manifest = adapter_by_id(&provider)
            .map(|adapter| adapter.advertise_models(store, manifest))
            .unwrap_or_default();
        (!manifest.is_empty()).then_some((provider, manifest))
    }).collect()
}

fn aggregate_source(active: &[(String, Vec<Value>)]) -> (&'static str, i64) {
    match active {
        [(provider, _)] => {
            let (remote, refreshed_at) = refresh_meta_for_provider(provider);
            (if remote { "remote" } else { "builtin" }, refreshed_at)
        }
        [] => ("none", 0),
        _ => ("aggregate", active.first().map(|(provider, _)| refresh_meta_for_provider(provider).1).unwrap_or(0)),
    }
}
