//! AutoClaw ****： provider id and 。
//!
//! ── this Why this ────────────────────────────────────
//! AutoClaw （、、 new 、directory、、 to 、、****） already 
//! to 。「 this is 1 」—— is ** and 
//! **， not is up ：
//!
//! ```text
//! autoclaw （id not ： accounts.json is ）
//! autoclaw-intl 
//! ```
//!
//! / / client / / in ——
//! not has 「client」 （ this is ）。
//!
//! ── provider id Why** not ** ────────────────────
//! `autoclaw` this id already in use accounts.json （`provider` 
//! is ）。 `autoclaw-cn` will in after 「unknown
//! provider」——`kind_from_id` return None， from 。
//! ****，id `autoclaw` not ； `autoclaw-intl`。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::providers::{kind_id, ProviderKind};

/// AutoClaw 。
///
/// = （ in before ）：`ALL` up after 。
/// ** in before ** is as is already has that 1 （）。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Region {
    /// （ already has that 1 ，provider id is `autoclaw`）
    #[default]
    Cn,
    /// （provider id is `autoclaw-intl`）
    Intl,
}

impl Region {
    /// （： in before ）
    pub const ALL: [Region; 2] = [Region::Cn, Region::Intl];

    /// for provider kind（ → ** 1 **）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::AutoClaw,
            Self::Intl => ProviderKind::AutoClawIntl,
        }
    }

    /// provider id（`"autoclaw"` / `"autoclaw-intl"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （`autoclaw` id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.provider_id() == provider_id)
    }

    /// this kind is not is AutoClaw （）—— in this 1 
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        Self::ALL.into_iter().find(|region| region.kind() == kind)
    }

    /// （ up in `AutoClaw` after that 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Self::Cn => "国内版",
            Self::Intl => "国际版",
        }
    }
}
