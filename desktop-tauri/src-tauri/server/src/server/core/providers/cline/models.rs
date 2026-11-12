//! Cline（ `api.cline.bot`） ****： = provider。
//!
//! ── already to ─────────────────────────────────
//! （`workos:` before ）、、、directory、、、
//! in `agent2api-providers` （ not ）。 this 
//! **must in /config** ：
//!
//! ```text
//! 1. ↔ provider id ↔ ProviderKind （`Pool`）—— is ，
//! `provider` is this id；
//! 2. up id before （`cline-free/` / `cline-pass/`）—— is **
//! **， then （alias → target） and by need use 「 this 
//! 」， then is use config、 in ；
//! 3. `friendly_alias` before then —— default（`model_rules::cline`）
//! need use 「 this this has 」。
//! ```
//!
//! ── Why is provider ─────────────────────────
//! `cline-free` and `cline-pass` in up is ** 1 up **，
//! ** **： has 、、 then and 。
//!
//! is 「 1 1 」（ `pool` ， by 「
//! has 」 1 ）。 that has not ：
//! - up 1 Cline ，****；
//! - then is 「(provider, id)」， use 1 provider，
//! 1 will 1 ；
//! - （`deepseek-v4.1-flash` has ） need 「 has 
//! 」， then 。
//!
//! after ** has **： `list` this ，
//! and none。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use crate::server::core::providers::{kind_from_id, kind_id, ProviderKind};

/// before （ up id 1 ）
pub const FREE_PREFIX: &str = "cline-free/";

/// before （ up id 1 ）
pub const PASS_PREFIX: &str = "cline-pass/";

/// before （** already not **： 403）
pub const CLOUD_PREFIX: &str = "cline-cloud/";

/// 。** 1 = 1 **， this when is 「 this is 」 
/// —— and provider id / `ProviderKind` in this ， not need 
/// `"cline-free"` this 。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pool {
    /// （`cline-free/...` and none before ）
    Free,
    /// ClinePass （`cline-pass/...`）
    Pass,
}

impl Pool {
    /// （：Free in before ）。
    pub const ALL: [Pool; 2] = [Pool::Free, Pool::Pass];

    /// `pool` / id （ `cline-free`、`free`、`Free`）。
    ///
    /// ── Why is `pass`（ ）──────────────────
    /// this in as ** old **： `pool: "free"` /
    /// `"pass"` （ before ）， when by 1 。 `pass`
    /// use old —— old or `pass`，must to 
    /// **** ，otherwise use 1 。
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "free" | "cline-free" => Self::Free,
            _ => Self::Pass,
        }
    }

    /// this in is provider（** 1 ** → ）
    pub fn kind(self) -> ProviderKind {
        match self {
            Self::Free => ProviderKind::ClineFree,
            Self::Pass => ProviderKind::ClinePass,
        }
    }

    /// this provider id（`"cline-free"` / `"cline-pass"`）
    pub fn provider_id(self) -> &'static str {
        kind_id(self.kind())
    }

    /// provider id → （ id return None）
    pub fn from_provider_id(provider_id: &str) -> Option<Self> {
        kind_from_id(provider_id).and_then(|kind| match kind {
            ProviderKind::ClineFree => Some(Self::Free),
            ProviderKind::ClinePass => Some(Self::Pass),
            _ => None,
        })
    }

    /// up id ** before **（`cline-free/` / `cline-pass/`）。
    ///
    /// this before must up （ is ）， is 「 」
    /// （ `catalog::wire_target_for_provider`）。
    pub fn target_prefix(self) -> &'static str {
        match self {
            Self::Free => FREE_PREFIX,
            Self::Pass => PASS_PREFIX,
        }
    }
}

/// up id → （** by id before **）。
///
/// ── in use ， ────────────────────────────────────
/// directory by ** by response **（ has not `cline-free/`
/// before id， by before will ）。 this use has ：
/// - **** （`model_rules::cline::migrate_cline_split`）：
/// `pool` is by this ，must to ；
/// - then old target not before when ；
/// - and 。
///
/// none before id `Pass`： before no when none from ， pass is ****。
pub fn pool_of(model_id: &str) -> Pool {
    if model_id.starts_with(FREE_PREFIX) {
        return Pool::Free;
    }
    Pool::Pass
}

/// up id → for down ****（ this this has before ）； no has can return None。
///
/// ── by ， not is id ────────────────────────────────────
/// ** not before ** （`z-ai/glm-5.3-flash`、
/// `poolside/laguna-s-2.1:free`）： by up to ，id up has 
/// before 。 this this has 1 can ， is ：
/// - before （`cline-free/` / `cline-pass/` / `cline-cloud/`）→ before ；
/// - **** not before 、 before （`z-ai/…`）→ before ；
/// - its （ none before 、`recommended` `openai/gpt-6-astra` this ）
/// → `None`。 that before is ****（`openai/gpt-6-astra` and 
/// `anthropic/claude-opus-5` is not ）， will up not in 。
///
/// ── （）─────────────────────────────────────────
/// can can and id （`glm-5.3-flash` when is CatPaw 
/// id）。 in by down is **** ： 1 alias 1 、
/// 1 （ `model_rules` module header）， not 。
pub fn friendly_alias(pool: Pool, model_id: &str) -> Option<&str> {
    let id = model_id.trim();
    if id.is_empty() {
        return None;
    }
    // before ：`cline-free/deepseek-v4.1-flash` → `deepseek-v4.1-flash`
    for prefix in [FREE_PREFIX, PASS_PREFIX, CLOUD_PREFIX] {
        if let Some(rest) = id.strip_prefix(prefix) {
            return Some(rest);
        }
    }
    // id：`z-ai/glm-5.3-flash` → `glm-5.3-flash`。
    // no has `/` id（ up to ）none need 。
    if pool == Pool::Free {
        let (_, rest) = id.split_once('/')?;
        // after `:free`（`poolside/laguna-s-2.1:free` → `laguna-s-2.1`）——
        // Cline 1 is this （ `:free` and ` (free)` after ）。
        // after will to 1 client 。
        let rest = rest.strip_suffix(":free").unwrap_or(rest);
        return (!rest.is_empty()).then_some(rest);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ↔ provider id ↔ kind must（）。
    #[test]
    fn pool_and_provider_id_are_bijective() {
        for pool in Pool::ALL {
            assert_eq!(Some(pool), Pool::from_provider_id(pool.provider_id()));
            assert_eq!(
                Some(pool.kind()),
                kind_from_id(pool.provider_id()),
                "kind 与 provider id 必须互查一致"
            );
        }
        assert_eq!(None, Pool::from_provider_id("cline"));
        assert_eq!(None, Pool::from_provider_id("workbuddy"));
    }

    /// old `pool` （`free` / `pass` / / ）：
    /// and 1 `pass`，must to 。
    #[test]
    fn legacy_pool_values_parse_stably() {
        assert_eq!(Pool::Free, Pool::parse("free"));
        assert_eq!(Pool::Free, Pool::parse("cline-free"));
        assert_eq!(Pool::Pass, Pool::parse("pass"));
        assert_eq!(Pool::Pass, Pool::parse("cline-pass"));
        assert_eq!(Pool::Pass, Pool::parse(""));
        assert_eq!(Pool::Pass, Pool::parse("nonsense"));
    }

    /// then ： before before ； id before and `:free`；
    /// id and recommended id not （ that is ）。
    #[test]
    fn friendly_alias_strips_channels_not_identities() {
        assert_eq!(
            Some("deepseek-v4.1-flash"),
            friendly_alias(Pool::Free, "cline-free/deepseek-v4.1-flash")
        );
        assert_eq!(
            Some("glm-5.3"),
            friendly_alias(Pool::Pass, "cline-pass/glm-5.3")
        );
        assert_eq!(
            Some("glm-5.3-flash"),
            friendly_alias(Pool::Free, "z-ai/glm-5.3-flash")
        );
        assert_eq!(
            Some("laguna-s-2.1"),
            friendly_alias(Pool::Free, "poolside/laguna-s-2.1:free")
        );
        // id is ， not 
        assert_eq!(None, friendly_alias(Pool::Pass, "openai/gpt-6-astra"));
        // up to （none before none）none need 
        assert_eq!(None, friendly_alias(Pool::Free, "glm-5.3-flash"));
    }

    /// before ： has `cline-free/` ， its （ id）。
    #[test]
    fn pool_of_only_recognizes_the_free_channel_prefix() {
        assert_eq!(Pool::Free, pool_of("cline-free/deepseek-v4.1-flash"));
        assert_eq!(Pool::Pass, pool_of("cline-pass/glm-5.3"));
        assert_eq!(Pool::Pass, pool_of("z-ai/glm-5.3-flash"));
        assert_eq!(Pool::Pass, pool_of("glm-5.3-flash"));
    }
}
