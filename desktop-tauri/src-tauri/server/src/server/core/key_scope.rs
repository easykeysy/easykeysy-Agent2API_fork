//! 「 this request in is Key」 and **available / available**limit
//! （R9， OmniProxy `allowed_provider_ids` / `allowed_models`）。
//!
//! ── Why 1 （ not is api_keys.rs）────────────────
//! by ：
//! 1. `api_keys.rs` is **config **（config.json `apiKeys` ：、
//! 、）， not axum； is **request use ** 
//! （request、 handler and ）， need axum `Request`。
//! in 1 will that file when ；
//! 2. limit **path**（ in → handler → ）
//!      `http.rs` / `api/*` / `core::upstream` ， in 1 has up 
//! 「 」。
//!
//! ── in 「 in that Key」 down （ ）──────
//! use **axum request**（`http::Request::extensions_mut().insert(...)`）：
//! - 、none； no has ， not 。
//! - ** not request and request**： by 「client body sha256」
//! （`api::chat` / `api::protocol`）， body will ；
//! request then will when to up （ will 
//! `client_headers`， `ProviderAdapter::build_chat_request`）。
//! - and has 1 ：axum `Extension<T>` is 「request use 」 
//! ， `ServerState` `State` is as 1 ，「 in 
//! Key」 is **request** ， is Extension use 。
//!
//! ── for and ────────────────────────────────────
//! Environment variable key no has ， already request can no has `KeyScope`。
//! `None` unlimited， not already ；invalid、disabled or none Key
//! must by ， not can as not limit use 。

use std::collections::HashSet;

use axum::extract::Request;

use crate::server::core::api_keys::ApiKeyEntry;

/// request in Key limit（** already 1 **：empty = unlimited）。
///
/// `HashSet`（） not is `Vec<String>`： in ** 1 
/// ** and ** 1 ** up ， `eq_ignore_ascii_case` is O(n)
/// ； 1 after is 1 。 not and 
/// （、provider id for ignore， `model_rules` module header）。
#[derive(Clone, Debug, Default)]
pub struct KeyScope {
    /// provider id（），empty = unlimited
    allowed_providers: HashSet<String>,
    /// for （），empty = unlimited
    allowed_models: HashSet<String>,
}

impl KeyScope {
    /// by 1 Key （`allowed*` as empty = this unlimited）
    pub fn from_entry(entry: &ApiKeyEntry) -> Self {
        Self {
            allowed_providers: normalize(&entry.allowed_providers),
            allowed_models: normalize(&entry.allowed_models),
        }
    }

    /// ** has this 1 **（ use ： need in ** this 1 ** up ，
    /// not is 「 in 1 」—— 1 for in 1 ，
    /// not 「 this 1 」 is ，none）。
    ///
    /// unlimited： by request（ that is this 1 default），
    /// 1 will 1 can can for not up 。
    pub fn provider_only(provider: &str) -> Self {
        let mut allowed_providers = HashSet::new();
        let id = normalize_one(provider);
        if !id.is_empty() {
            allowed_providers.insert(id);
        }
        Self {
            allowed_providers,
            allowed_models: HashSet::new(),
        }
    }

    pub fn for_test_provider(&self, provider: &str) -> Option<Self> {
        if !self.allows_provider(provider) {
            return None;
        }
        let mut scope = Self::provider_only(provider);
        scope.allowed_models = self.allowed_models.clone();
        Some(scope)
    }

    /// is limit（ up 「 has no has 」 ，used for）
    pub fn restricts_providers(&self) -> bool {
        !self.allowed_providers.is_empty()
    }

    /// is limit
    pub fn restricts_models(&self) -> bool {
        !self.allowed_models.is_empty()
    }

    /// request `model`（** for **， alias）？
    ///
    /// for ignore and empty： will from HTTP request，
    /// client 1 empty not this 「 not in 」。
    pub fn allows_model(&self, model: &str) -> bool {
        self.allowed_models.is_empty() || self.allowed_models.contains(&normalize_one(model))
    }

    /// by to `provider`（provider id）？
    pub fn allows_provider(&self, provider: &str) -> bool {
        self.allowed_providers.is_empty() || self.allowed_providers.contains(&normalize_one(provider))
    }

    /// provider id（、；limit = empty）。
    ///
    /// before 1 is ****（`provider_loop` 
    /// `ignore_model_gate`）： in ， ——
    /// can can no enabled（、 not enabled）， by by 
    /// not used for this 1 。
    pub fn allowed_provider_ids(&self) -> Vec<String> {
        sorted(&self.allowed_providers)
    }

    /// can need （ and error use ；empty = `unlimited`）
    pub fn describe(&self) -> String {
        let providers = if self.restricts_providers() {
            sorted(&self.allowed_providers).join("、")
        } else {
            "unlimited".to_string()
        };
        let models = if self.restricts_models() {
            format!("{} ", self.allowed_models.len())
        } else {
            "unlimited".to_string()
        };
        format!(" {providers}； {models}")
    }
}

/// 1 1 （empty、empty）
fn normalize(list: &[String]) -> HashSet<String> {
    list.iter()
        .map(|item| normalize_one(item))
        .filter(|item| !item.is_empty())
        .collect()
}

/// 1 ：empty + 
fn normalize_one(value: &str) -> String {
    value.trim().to_lowercase()
}

/// after （ need ，HashSet not ）
fn sorted(set: &HashSet<String>) -> Vec<String> {
    let mut out: Vec<String> = set.iter().cloned().collect();
    out.sort();
    out
}

/// in Key request（** has in **）。
///
/// not already has ： in 1 ， will if has 1 in ，
/// 「 1 in Key 」「 after before 」。
pub fn attach(request: &mut Request, scope: KeyScope) {
    if request.extensions().get::<KeyScope>().is_none() {
        request.extensions_mut().insert(scope);
    }
}

/// ；`None` = unlimited。 use must。
pub fn allows_model(scope: Option<&KeyScope>, model: &str) -> bool {
    scope.map_or(true, |scope| scope.allows_model(model))
}

/// up ，
pub fn allows_provider(scope: Option<&KeyScope>, provider: &str) -> bool {
    scope.map_or(true, |scope| scope.allows_provider(provider))
}
