//! WorkBuddy ****（ / is provider）。
//!
//! ── already to ─────────────────────────────────
//! / client / 、request 1 （ / tool_choice /
//! before / tool for ）、token 、`/v3/config` directory new 、daily
//! 、（ / / ）、 —— is 
//! 「client up 」 ， by need not in 。
//!
//! this ****： ↔ provider id ↔ `ProviderKind` 、、
//! ， and id 1 （`cn` / `intl` is ****：
//! `edition` is this ）。
//!
//! ── Why is provider ───────────────────────────
//! and Cline 、AutoClaw / Accio / ZCode 1 ： 
//! 「 up `edition` 」 will directory has 1 （ ，issue
//! #74）、empty has 1 （none，issue #89）、 not 
//! （request can can to 1 up ）。 by after ，directory、、
//! new 、 then 、、、 by provider id 。
//!
//! ── provider id and ──────────────────────────────────
//! **must** `"workbuddy"`： is ， will that 
//! after 「unknown provider」。 when is `DEFAULT_PROVIDER_ID`
//! （ up when ）。
//! use new id `"workbuddy-intl"`， by 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::endpoints::{resolve_edition, EditionInfo};
use crate::server::core::providers::{kind_id, ProviderKind};

/// WorkBuddy 。
///
/// = （ in before ）：`ALL` up after 。
/// in before is as is 。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Region {
    /// （provider id is `workbuddy`）
    #[default]
    Cn,
    /// （provider id is `workbuddy-intl`）
    Intl,
}

impl Region {
    /// （： in before ）
    pub const ALL: [Region; 2] = [Region::Cn, Region::Intl];

    /// for provider kind（ → ** 1 **）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::WorkBuddy,
            Self::Intl => ProviderKind::WorkBuddyIntl,
        }
    }

    /// provider id（`"workbuddy"` / `"workbuddy-intl"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （`workbuddy` id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.provider_id() == provider_id)
    }

    /// this kind is not is workbuddy （）—— in this 1 
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        Self::ALL.into_iter().find(|region| region.kind() == kind)
    }

    /// （ up in `WorkBuddy` after that 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Self::Cn => "国内版",
            Self::Intl => "国际版",
        }
    }

    /// （ + ）。
    ///
    /// has this —— / before / platform / client in ，
    /// not has （ `core::endpoints` module header）。
    pub fn edition(self) -> &'static EditionInfo {
        resolve_edition(Some(self.id()))
    }

    /// id（`"cn"` / `"intl"`；，）
    pub fn id(self) -> &'static str {
        match self {
            Self::Cn => "cn",
            Self::Intl => "intl",
        }
    }

    /// ****（`catalog_cache` ）。
    ///
    /// 1 1 is need ： before use 1 ，
    /// （issue #74）。
    pub fn catalog_scope(self) -> &'static str {
        match self {
            Self::Cn => crate::server::core::providers::catalog_cache::SCOPE_WORKBUDDY,
            Self::Intl => crate::server::core::providers::catalog_cache::SCOPE_WORKBUDDY_INTL,
        }
    }

    /// id 1 （ `cn` / `intl` and ，unknown）
    /// —— in `resolve_edition` ， not need 1 。
    ///
    /// 1 after **id ** not is 「return use is not is 1 」：
    /// `resolve_edition` return `&'static` `const` ， use is 
    /// in 1 ** not is **（ will when for when ）。
    pub fn from_edition_id(value: Option<&str>) -> Self {
        let id = resolve_edition(value).id;
        Self::ALL
            .into_iter()
            .find(|region| region.id() == id)
            .unwrap_or(Self::Cn)
    }
}

/// this provider id is not is **workbuddy **（ / ）。
///
/// in this 1 。** when this use **： use 1 schema
/// （`uid` + `edition` / `endpoint` / `prefixPath` / `platform` and 
/// accessToken/refreshToken ）， is 「 by schema 」 
/// （ 1 、、）must 1 ——
/// `provider == kind_id(ProviderKind::WorkBuddy)` will 
/// 「」 that 1 （ when empty、 not ）。
///
/// ** when not this use **： need need （directory、、、
/// then 、）—— that use `Region` or provider id。
pub fn is_workbuddy_family(provider_id: &str) -> bool {
    Region::from_provider_id(provider_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ↔ provider id ↔ kind must（）。
    #[test]
    fn region_and_provider_id_are_bijective() {
        for region in Region::ALL {
            assert_eq!(Some(region), Region::from_provider_id(region.provider_id()));
            assert_eq!(Some(region), Region::from_kind(region.kind()));
        }
        assert_eq!(None, Region::from_provider_id("zcode"));
    }

    /// id 1 must： `edition` is `cn` / `intl`，
    /// unknown（） —— that is 。
    #[test]
    fn edition_ids_normalize_stably() {
        assert_eq!(Region::Cn, Region::from_edition_id(Some("cn")));
        assert_eq!(Region::Intl, Region::from_edition_id(Some("intl")));
        assert_eq!(Region::Cn, Region::from_edition_id(None));
        assert_eq!(Region::Cn, Region::from_edition_id(Some("mars")));
    }
}
