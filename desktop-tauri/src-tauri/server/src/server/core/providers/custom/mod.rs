//! directory、 and 。

use serde_json::Value;

use crate::server::core::account_store::AccountStore;
use crate::server::core::custom_providers;
use crate::server::core::key_scope::{self, KeyScope};
use crate::server::core::models::{list_item, model_id};

/// ：（sub2api / NewAPI / DeepSeek / OpenCode Go / new /
/// / z.ai ） 。 by `balanceScheme` 
/// （）， by （`balanceQuery`）—— filemodule header。
pub mod balance;
/// up response（； and `balance` request，
/// for ）
pub mod balance_parse;
mod quota;
pub mod emulation;
pub mod forward;
/// new （OAuth2 + PKCE JWT），
pub mod sensenova_auth;

pub(crate) fn catalog_providers(store: &AccountStore) -> Vec<(String, Vec<Value>)> {
    let accounts = store.routing_snapshot();
    custom_providers::list().into_iter().filter_map(|provider| {
        let id = provider.get("id")?.as_str()?.to_string();
        if !accounts.available_providers.contains(&id) { return None; }
        let models = custom_providers::public_models(&provider);
        (!models.is_empty()).then_some((id, models))
    }).collect()
}

pub fn append_models_response(store: &AccountStore, scope: Option<&KeyScope>, data: &mut Vec<Value>) {
    let mut claimed: std::collections::HashSet<_> = data.iter()
        .map(|item| model_id(item).to_lowercase()).collect();
    for (provider, models) in catalog_providers(store) {
        if !key_scope::allows_provider(scope, &provider) { continue; }
        for model in models {
            let id = model_id(&model);
            if key_scope::allows_model(scope, &id) && claimed.insert(id.to_lowercase()) {
                data.push(list_item(&model, &provider));
            }
        }
    }
}
