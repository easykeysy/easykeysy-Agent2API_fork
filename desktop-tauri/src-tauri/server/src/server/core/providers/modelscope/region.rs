//! ModelScope（） ****（ this 1 ）。
//!
//! ── this Why in ──────────────────────────────────────
//! （ / directory / / to / ） already to 。
//! not 「 is 」—— that is up 。
//!
//! ** = provider** this is ****： 
//! `provider` 、、 up 、 to ， by 
//! `modelscope` / `modelscope-intl` 。 this ：
//! - provider id ↔ （`from_kind` / `from_provider_id` / `kind`）；
//! - and id before 。
//!
//! ── Why is provider not is 「 1 1 」───────────────
//! 「 up 1 」 after and already that ：
//! ** **， up in 1 、none by 。 
//! provider after has 、、 and 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::providers::{kind_id, ProviderKind};

/// ModelScope 。
///
/// = （ in before ）：`ALL` up after 。
/// ** in before **（ use 「 is that 」， and MonkeyCode / ZCode
/// 1 ）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Region {
    /// （provider id `modelscope`）
    #[default]
    Cn,
    /// （provider id `modelscope-intl`）
    Intl,
}

impl Region {
    /// （ use ）
    pub const ALL: [Region; 2] = [Region::Cn, Region::Intl];

    /// provider
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::Modelscope,
            Self::Intl => ProviderKind::ModelscopeIntl,
        }
    }

    /// provider id
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → ； not is ModelScope when None
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        match provider_id {
            "modelscope" => Some(Self::Cn),
            "modelscope-intl" => Some(Self::Intl),
            _ => None,
        }
    }

    /// provider kind → ； not is ModelScope when None
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        match kind {
            ProviderKind::Modelscope => Some(Self::Cn),
            ProviderKind::ModelscopeIntl => Some(Self::Intl),
            _ => None,
        }
    }

    /// （ up in `ModelScope` after that 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Self::Cn => "国内版",
            Self::Intl => "国际版",
        }
    }

    /// id ** before **（id use ）。
    ///
    /// before by and AutoClaw that 1 ： id empty not ，
    /// ** 1 in `HavanaId` can can **（，server id 
    /// then will ）。 has provider id will ，
    /// 「 and 」 not can can —— and is can 1 。
    ///
    /// no has ，** not ** id， before （Cline ）。
    pub fn account_id_prefix(self) -> &'static str {
        match self {
            Self::Cn => "modelscope-",
            Self::Intl => "modelscope-intl-",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Region;

    /// must and id for up —— for not up `adapter_for` will return
    /// error （ to 1 ）， this is not 。
    #[test]
    fn region_ids_match_the_expected_literals() {
        assert_eq!(Region::Cn.provider_id(), "modelscope");
        assert_eq!(Region::Intl.provider_id(), "modelscope-intl");
    }

    /// is ：id → region → id must to 
    #[test]
    fn region_lookup_round_trips() {
        for region in Region::ALL {
            assert_eq!(Region::from_provider_id(region.provider_id()), Some(region));
            assert_eq!(Region::from_kind(region.kind()), Some(region));
        }
        assert_eq!(Region::from_provider_id("loomy"), None);
    }

    /// id before must** not ** —— 1 in will 1 id，
    /// will up provider not 。
    ///
    /// Note this ** not **「 not as before 」：`modelscope-intl-` `modelscope-`
    /// （Cline ）。「 is not is ModelScope 」 1 
    /// `Region::from_provider_id`， not need use `starts_with` before —— that will 
    /// 。
    #[test]
    fn account_id_prefixes_are_distinct() {
        assert_ne!(Region::Cn.account_id_prefix(), Region::Intl.account_id_prefix());
    }
}
