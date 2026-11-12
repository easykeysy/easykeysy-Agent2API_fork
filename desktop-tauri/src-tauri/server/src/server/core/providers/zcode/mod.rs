//! ZCode（ / Z.AI client） ****： / provider。
//!
//! ── already to ─────────────────────────────────
//! CLI 、、（Anthropic + 
//! ）、 when 、、 and 、、
//! 、GLM-5.x 、 and in 
//! `agent2api-providers` （ not ）。
//!
//! this ：
//!
//! ```text
//! region.rs ↔ provider id ↔ ProviderKind 、
//! `account_store::zcode_accounts`（ ）
//! `zcodePlan` （ and ， not up ）
//! ```
//!
//! ── Why「」 this in ───────────────────────
//! `zcodePlan` is ** up 1 **， down 、 、 after 
//! by 。「 use 」， not 「 up 」 ——
//! URL、、 in 。 this is ****
//! （`coding-plan` / `start-plan`） and ****， and 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic。

pub mod region;

/// up 「 use 」（`accounts.json` ； 1 ）。
///
/// Why `zcode` before not is `plan`： is ** use ** 1 
/// （`StoredAccount` ），`plan` this in （Cline 、
/// Trae planType） has ，； before after before 、、
/// is 1 ， not in 。
pub const PLAN_FIELD: &str = "zcodePlan";

/// （default）： use to 。
pub const PLAN_CODING: &str = "coding-plan";

/// （）： 1 ， use JWT 。
///
/// this 、、， in —— 
/// 「 this 1 」。
pub const PLAN_START: &str = "start-plan";

/// 1 / to ： up ， its （empty）return None。
///
/// and `planId` 1 ** not **： not 1 ，
/// not is to default —— that will use as ， in use old 。
pub fn normalize_plan(raw: &str) -> Option<&'static str> {
    match raw.trim() {
        PLAN_CODING => Some(PLAN_CODING),
        PLAN_START => Some(PLAN_START),
        _ => None,
    }
}

/// （****） use 。
///
/// / not = [`PLAN_CODING`]： this is default， is 「 after can use 」
/// that （default as and before ）。
pub fn plan_of(account: &serde_json::Value) -> &'static str {
    account
        .get(PLAN_FIELD)
        .and_then(serde_json::Value::as_str)
        .and_then(normalize_plan)
        .unwrap_or(PLAN_CODING)
}

/// ： down and after use this 1 
/// （ 1 ， use will in to 「」、 in to 
/// 「Coding Plan」， for not up is 1 set）。
///
/// Start Plan this 「****」 not is 「」： that 
/// when ， has ** new use ** —— 「」
/// will need need this use as and none。
pub fn plan_label(plan: &str) -> &'static str {
    match plan {
        PLAN_START => "套餐池（Start Plan）",
        _ => "编码套餐（Coding Plan）",
    }
}

// ── 「 use config」 （`settings.get` `key`）──────────
// ZCode （ + ） to ，「 not 」
// 「」 is ** use config**（set that 1 ）。 this 
// 1 config， in `plugin_host::bootstrap` （ this ，
// not will `plugin.<provider>.<key>`）。

/// 「」 （ = ， and default 1 ）。
pub const SETTING_PROMPT_ENABLED: &str = "gatewayPrompt";

/// 「」 （`{identity, stable, dynamic}`， ）。
pub const SETTING_PROMPT_TEXT: &str = "gatewayPromptText";
