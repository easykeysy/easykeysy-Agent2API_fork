//! in this ：** FULL-auth **（scoped runtime） and 
//! ** has no has **（suspended）。 up is 1 ，
//! after must —— is scoped runtime， already has 。

pub const BUILTIN_SUBSCRIPTION_IDS: [&str; 5] = [
    "claude-subscription",
    "codex-subscription",
    "chatgpt-subscription",
    "copilot-subscription",
    "gemini-subscription",
];

/// **scoped runtime**（ use FULL-auth ） 。
///
/// is ****， not is **available**： this to 、
/// error（`plugin_host/strict_errors.rs`）、directory new need 1 
/// （`plugin_host/rpc.rs`）。 can can in ， can can 
/// disabled —— 「 has no has 」 [`is_suspended`]。
const SCOPED_RUNTIME_IDS: [&str; 13] = [
    "cursor",
    "devin",
    "factory",
    "zed",
    "grok",
    "minimax-code",
    "minimax-code-global",
    "qwenwork",
    // （`qwenwork-intl`） and 、，
    // scoped runtime —— error will not 1 。
    "qwenwork-intl",
    "qwenwork-enterprise",
    "kimi-work",
    "opendesign",
    "nous-portal",
];

/// before ** no has ** （ and ， not and use ）。
///
/// ── Why not in ──────────────────────────────
/// already by 「」（Rust， Agent2Providers `qwenwork`）。
/// in disabled will `is_executable_provider` as 
/// （`!is_suspended(id) && serves_provider(id)`）， is not 
/// —— is 「 up ，request no has up 」， not 。
///
/// ── ****Why in disabled（2026-11-03）────────────
/// and 「 no has 」 not ： for / is ， is ****—— use 
/// not up ， will failure。 up for 
/// （`https://gateway.qwenwork.ai/oauth/callback`）， 1 ；
/// already disabled（ for request 400）， not to 。
/// from can ：`is_suspended` will 、、directory new ，
/// in [`SCOPED_RUNTIME_IDS`] —— already in old can and 。
/// up ， from this can （ not use ）。
///
/// `qwenwork-enterprise`（）disabled： need provider
/// （ ）。
///
/// ── Kiro Why in 2026-11 this ─────────────────────
/// already by 「」（Rust， Agent2Providers `kiro`）。
/// in [`SUSPENDED_IDS`] will `is_executable_provider` as ， and 
/// 1 ：、 up ， up not to this 。
/// when from [`SCOPED_RUNTIME_IDS`] ：Kiro 、error and directory by 
/// ， not need need and 「 1 」。
const SUSPENDED_IDS: [&str; 17] = [
    "cursor",
    "devin",
    "factory",
    "zed",
    "grok",
    "minimax-code",
    "minimax-code-global",
    "qwenwork-intl",
    "qwenwork-enterprise",
    "kimi-work",
    "opendesign",
    "nous-portal",
    "claude-subscription",
    "codex-subscription",
    "chatgpt-subscription",
    "copilot-subscription",
    "gemini-subscription",
];

/// is not is 「」 that 1 （ / / ）。
///
/// error need for this id use 1 （`strict_error_codes` 
/// `qwen_number`）， has this 1 ， in 1 id 。
pub(crate) fn is_qwenwork_family(id: &str) -> bool {
    matches!(id, "qwenwork" | "qwenwork-intl" | "qwenwork-enterprise")
}

pub(crate) fn is_runtime_identity(id: &str) -> bool {
    BUILTIN_SUBSCRIPTION_IDS.contains(&id) || SCOPED_RUNTIME_IDS.contains(&id)
}

/// this has no has 。
pub fn is_suspended(id: &str) -> bool {
    SUSPENDED_IDS.contains(&id)
}
