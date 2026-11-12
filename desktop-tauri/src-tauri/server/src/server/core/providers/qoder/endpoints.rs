//! Qoder ****（ in / is provider）。
//!
//! ── already to ─────────────────────────────────
//! COSY 、request、SSE 、、、、 to 、
//! （UMID ）、directory and in `agent2api-providers` （ not ）。
//! this ****： ↔ provider id ↔ `ProviderKind` ， and 
//! need use （`id` / `edition` / `label`）。
//!
//! ── provider （2026-10 ）────────────────────────
//! and AutoClaw / Accio / ZCode ： is ** provider**（`qoder` in 
//! / `qoder-intl` ）， → in impl， not need 
//! `"qoder-intl"` 。 in `qoder`（ `accounts` 
//! ，）。
//!
//! ── up Why ────────────────────────────────────
//! 、openapi、 is 「 up 」 ， 1 to 
//! 。 will 1 no 、 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::Value;

use crate::server::core::providers::{kind_id, ProviderKind};
use crate::server::errors::GatewayError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Global,
    Cn,
}

impl Region {
    /// （： in in before —— already has 1 ，）。
    pub const ALL: [Region; 2] = [Region::Cn, Region::Global];

    /// for provider kind（ → ** 1 **）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Cn => ProviderKind::Qoder,
            Self::Global => ProviderKind::QoderIntl,
        }
    }

    /// provider id（`"qoder"` / `"qoder-intl"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （`qoder` id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|region| region.provider_id() == provider_id)
    }

    /// this kind is not is Qoder （）—— in this 1 
    pub fn from_kind(kind: ProviderKind) -> Option<Self> {
        Self::ALL.into_iter().find(|region| region.kind() == kind)
    }

    /// （`global` / `intl` / `cn`，empty by ）。
    ///
    /// as ** old **： / file `mode` 。 new 
    /// by provider id 。
    pub fn parse(value: &str) -> Result<Self, GatewayError> {
        match value.trim() {
            "" | "global" | "intl" => Ok(Self::Global),
            "cn" => Ok(Self::Cn),
            _ => Err(GatewayError::with_status(
                400,
                "Qoder 地区必须为 global（国际版）或 cn（中国版）",
            )),
        }
    }

    /// from `mode` / `edition` （， and old 1 ）。
    pub fn from_payload(payload: &Value) -> Result<Self, GatewayError> {
        let mode = payload.get("mode").or_else(|| payload.get("edition"));
        match mode {
            None | Some(Value::Null) => Ok(Self::Global),
            Some(Value::String(value)) => Self::parse(value),
            _ => Err(GatewayError::with_status(400, "Qoder 地区必须是字符串")),
        }
    }

    /// （ / / before ）。
    pub fn id(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Cn => "cn",
        }
    }

    /// （ `edition` ， before and ）。
    pub fn edition(self) -> &'static str {
        match self {
            Self::Global => "intl",
            Self::Cn => "cn",
        }
    }

    /// （`editionLabel` and use ）。
    pub fn label(self) -> &'static str {
        match self {
            Self::Global => "国际版",
            Self::Cn => "中国版",
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
    }

    /// old `mode` / `edition` ： by ，
    /// `cn` is in ， its 400。
    #[test]
    fn legacy_region_values_parse_stably() {
        assert_eq!(Region::Global, Region::parse("").unwrap_or(Region::Cn));
        assert_eq!(Region::Global, Region::parse("global").unwrap_or(Region::Cn));
        assert_eq!(Region::Global, Region::parse("intl").unwrap_or(Region::Cn));
        assert_eq!(Region::Cn, Region::parse("cn").unwrap_or(Region::Global));
        assert!(Region::parse("mars").is_err());
        // edition （ before is `intl` / `cn`）
        assert_eq!(
            Region::Global,
            Region::from_payload(&serde_json::json!({ "edition": "intl" }))
                .unwrap_or(Region::Cn)
        );
        assert_eq!(
            Region::Cn,
            Region::from_payload(&serde_json::json!({ "mode": "cn" })).unwrap_or(Region::Global)
        );
    }
}
