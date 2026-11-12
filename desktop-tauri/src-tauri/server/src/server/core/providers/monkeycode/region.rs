//! MonkeyCode ****（ / ）—— and need need that 。
//!
//! ── this fileWhy ──────────────────────────────────
//! 、directory、 to ****
//! （`agent2api-providers` `monkeycode/`）， not —— that is 
//! 1 。、cookie 、 this up 。
//!
//! this need need 「」 this ， as is ****：
//! - provider id（`monkeycode` / `monkeycode-intl`）；
//! - id before ， 1 in not will id；
//! - up need 「 / 」。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::providers::{kind_id, ProviderKind};

/// MonkeyCode 。
///
/// = （ in before ）：`ALL` and up 
/// after 。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Region {
    /// （provider id `monkeycode`）
    #[default]
    Cn,
    /// （provider id `monkeycode-intl`）
    Intl,
}

impl Region {
    /// （： in before ）
    pub const ALL: [Region; 2] = [Region::Cn, Region::Intl];

    /// for provider kind（ → ** 1 **）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::MonkeyCode,
            Self::Intl => ProviderKind::MonkeyCodeIntl,
        }
    }

    /// provider id（`"monkeycode"` / `"monkeycode-intl"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （`monkeycode` id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.provider_id() == provider_id)
    }

    /// this kind is not is MonkeyCode （）—— in this 1 
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        Self::ALL.into_iter().find(|region| region.kind() == kind)
    }

    /// （`"cn"` / `"intl"`，used for）
    pub fn id(self) -> &'static str {
        match self {
            Self::Cn => "cn",
            Self::Intl => "intl",
        }
    }

    /// after （ up in `MonkeyCode` after that 1 ）
    pub fn label(self) -> &'static str {
        match self {
            Self::Cn => "国内版",
            Self::Intl => "国际版",
        }
    }

    /// id ** before **（id use ）。
    ///
    /// provider id already ， id in ** 1 **：
    /// 1 in 、server id then 1 will 。 up before after 
    /// not ， 1 can and 。
    pub fn account_id_prefix(self) -> &'static str {
        match self {
            Self::Cn => "monkeycode-",
            Self::Intl => "monkeycode-intl-",
        }
    }
}
