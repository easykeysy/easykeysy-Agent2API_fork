//! Accio **** and （ this 1 ）。
//!
//! ── this Why in ──────────────────────────────────────
//! （ / directory / / / ） already to 。 not 
//! 「Accio up 、」—— that is to 。
//!
//! ** = provider** this is ****： 
//! `provider` 、、 up 、 to ， by 
//! `accio` / `accio-cn` 。 this ：
//! - provider id ↔ （`from_kind` / `from_provider_id` / `kind`）；
//! - `edition` （`intl` / `cn`） and 。
//!
//! up 、path、`appKey`、 this ** 1 not ** —— in 。
//!
//! ── Why is 1 etc not is 「 1 up 1 」────────────────
//! and `autoclaw::region` / Cline 1 ： up ，
//! and not 「 」， will 。

use crate::server::core::providers::{kind_id, ProviderKind};

/// Accio 。** = provider**。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Region {
    /// 
    Global,
    /// 
    Cn,
}

impl Region {
    /// （、 use ）
    pub const ALL: [Region; 2] = [Region::Global, Region::Cn];

    /// provider
    pub const fn kind(self) -> ProviderKind {
        match self {
            Self::Global => ProviderKind::Accio,
            Self::Cn => ProviderKind::AccioCn,
        }
    }

    /// provider id（`"accio"` / `"accio-cn"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → ； not is Accio when None
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        match provider_id {
            "accio" => Some(Self::Global),
            "accio-cn" => Some(Self::Cn),
            _ => None,
        }
    }

    /// provider kind → ； not is Accio when None
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        match kind {
            ProviderKind::Accio => Some(Self::Global),
            ProviderKind::AccioCn => Some(Self::Cn),
            _ => None,
        }
    }

    /// `edition` （ and its 1 ：`intl` / `cn`）
    pub const fn edition(self) -> &'static str {
        match self {
            Self::Global => "intl",
            Self::Cn => "cn",
        }
    }

    /// and 
    pub const fn label(self) -> &'static str {
        match self {
            Self::Global => "国际版",
            Self::Cn => "国内版",
        }
    }
}
