//! 。 default or 。
//! provider onlyused for， by 。

use crate::server::core::providers::catalog::{forwarding_providers, providers_for_model};
use crate::server::core::providers::DEFAULT_PROVIDER_ID;

pub fn route_for_forward(model: &str) -> Vec<String> {
    // has when old default up as 。
    if model.trim().is_empty() {
        return crate::server::core::routing_scope::providers_for(model, &[DEFAULT_PROVIDER_ID.to_string()]);
    }
    let providers: Vec<_> = forwarding_providers(model).into_iter()
        .filter(|provider| !super::images::provider_model_is_image(provider, model))
        .collect();
    if !providers.is_empty() {
        return crate::server::core::routing_scope::providers_for(model, &providers);
    }
    // has 、 no has → ， not default（otherwise
    // 「」 will 1 to workbuddy its success）。
    if !providers_for_model(model).is_empty() {
        return Vec::new();
    }
    // unknown ： use has ， to default up （resolve_model in has available
    // when already 1 「 not in 」， to this is no ）。
    crate::server::core::routing_scope::providers_for(model, &[DEFAULT_PROVIDER_ID.to_string()])
}
