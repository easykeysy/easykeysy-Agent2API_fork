//! ZCode ****（ / is provider）。
//!
//! ── already to ─────────────────────────────────
//! 、、Anthropic 、、、
//! 、 —— is 「 up 」 ， 1 to 
//! `agent2api-providers`（ not ）。
//!
//! this ****： ↔ provider id ↔ `ProviderKind` ， and 
//! need use （`label` / `account_id_prefix`）。
//!
//! ── Why is provider not is 「 1 1 」──────────
//! and Cline 、AutoClaw / Accio / Qoder 1 ：
//! 「 1 provider up `region` 」 will ** **， up in 
//! 1 、「 」 not ，none by 。
//! by provider after has 、、 and 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::providers::{kind_id, ProviderKind};

/// ZCode 。
///
/// = （ in before ）：`ALL` up after 
/// （ down is that 1 ， and AutoClaw by 1 ）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Region {
    /// （；provider id is `zcode`）
    #[default]
    Cn,
    /// （Z.AI；provider id is `zcode-intl`）
    Intl,
}

impl Region {
    /// （： in before ）
    pub const ALL: [Region; 2] = [Region::Cn, Region::Intl];

    /// for provider kind（ → ** 1 **）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::Zcode,
            Self::Intl => ProviderKind::ZcodeIntl,
        }
    }

    /// provider id（`"zcode"` / `"zcode-intl"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （`zcode` id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.provider_id() == provider_id)
    }

    /// this kind is not is ZCode （）—— in this 1 
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        Self::ALL.into_iter().find(|region| region.kind() == kind)
    }

    /// （ up in `ZCode` after that 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Self::Cn => "国内版",
            Self::Intl => "国际版",
        }
    }

    /// id ** before **（id use ）。
    ///
    /// before ： before can not —— 1 in /
    /// userId can can （ and AutoClaw 1 ），
    /// id will ，「 and 」 is 
    /// provider 1 。
    ///
    /// id when is 1 before （ that is 1 ）。
    pub fn account_id_prefix(self) -> &'static str {
        match self {
            Self::Cn => "zcode-user-",
            Self::Intl => "zcode-intl-user-",
        }
    }
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
        assert_eq!(None, Region::from_provider_id("workbuddy"));
        assert_eq!(None, Region::from_provider_id("zcode-intl-2"));
    }

    /// id before must not （otherwise will id）。
    #[test]
    fn account_id_prefixes_never_collide() {
        let cn = Region::Cn.account_id_prefix();
        let intl = Region::Intl.account_id_prefix();
        assert_ne!(cn, intl);
        // before not can ： 1 to before will 「 by before 」
        assert!(!intl.starts_with(cn) && !cn.starts_with(intl));
    }
}
