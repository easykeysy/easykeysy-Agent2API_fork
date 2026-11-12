//! （edition）：`cn` / `intl`。
//!
//! ── this has ──────────────────────────────────────────
//! up （`auth` / `llm` / `billing` / `activity` / `config` /
//! `cloudAgent` / `sidecar` ）、client and CLI 、UA 、
//! 、 up 、`Context`（baseUrl / prefix / platform ）——
//! that is 「client up 」 ， by need already 
//! to （`agent2api-providers`）， not has 。
//!
//! ── in ────────────────────────────────────────────
//! ** will that **：`edition` （`cn` / `intl`） and 
//! 。 `edition` is this ， 1 must
//! （`international` / `global` / `ai` …），otherwise after will 
//! unknown、error 。
//!
//! / before / platform / client** not in this **： by already has 
//! ， no has is empty —— not need need this 。

use std::path::PathBuf;

/// default id（ ； is 「 no edition」 when ）
pub const DEFAULT_EDITION: &str = "cn";

/// 1 ****（ + ）。
///
/// has ： 1 will 「 up this 」。
#[derive(Clone, Copy, Debug)]
pub struct EditionInfo {
    /// （`cn` / `intl`）—— `edition` is 
    pub id: &'static str,
    /// （ up in after that 1 ）
    pub label: &'static str,
}

/// 
const EDITION_CN: EditionInfo = EditionInfo {
    id: "cn",
    label: "国内版",
};

/// 
const EDITION_INTL: EditionInfo = EditionInfo {
    id: "intl",
    label: "国际版",
};

/// ， and up after 1 （cn → intl）
pub const EDITIONS: &[EditionInfo] = &[EDITION_CN, EDITION_INTL];

/// id： `cn` / `intl` and if ，unknown。
///
/// is ****（、file、 old config this ），
/// not is up —— 。
pub fn resolve_edition(value: Option<&str>) -> &'static EditionInfo {
    let id = value.unwrap_or("").trim().to_lowercase();
    if id.is_empty() || id == "cn" {
        return &EDITION_CN;
    }
    if id == "intl" {
        return &EDITION_INTL;
    }
    if matches!(
        id.as_str(),
        "international" | "global" | "ai" | "workbuddy-ai" | "workbuddyai" | "en"
    ) {
        return &EDITION_INTL;
    }
    if matches!(
        id.as_str(),
        "china" | "mainland" | "workbuddy" | "zh" | "cn-intl"
    ) {
        return &EDITION_CN;
    }
    &EDITION_CN
}

/// old file（onlyused for to ， account_store）
pub fn legacy_auth_file() -> PathBuf {
    crate::server::config::config_dir().join("auth.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// id 1 must： `edition` is `cn` / `intl`，
    /// unknown（） —— that is 。
    #[test]
    fn edition_ids_normalize_stably() {
        assert_eq!("cn", resolve_edition(Some("cn")).id);
        assert_eq!("intl", resolve_edition(Some("intl")).id);
        assert_eq!("cn", resolve_edition(None).id);
        assert_eq!("cn", resolve_edition(Some("mars")).id);
        // （ old and old config）
        assert_eq!("intl", resolve_edition(Some("International")).id);
        assert_eq!("intl", resolve_edition(Some("workbuddy-ai")).id);
        assert_eq!("cn", resolve_edition(Some("workbuddy")).id);
    }
}
