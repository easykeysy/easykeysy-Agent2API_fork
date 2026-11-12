//! WorkBuddy then ：defaultenabled + ****（2026-10）。
//!
//! ── Why 1 file ──────────────────────────────────────
//! and `model_rules/cline.rs` 1 by ： this 1 then （ + provider
//! ） 1 ， `mod.rs` already file
//! （1349 ）。 after `mod.rs` 「 and 1 none」 then ， 
//! has 1 file。`pub use` use 
//! `model_rules::seed_workbuddy_defaults` / `model_rules::migrate_workbuddy_split`。
//!
//! ── file （ before ）──────────────────────────
//! 1. ** then by `(provider, id)` **， WorkBuddy after is provider
//! （`workbuddy` / `workbuddy-intl`）—— 「」must by 1 
//! （ `seed_workbuddy_defaults` `provider` parameter）；
//! 2. ** then has `workbuddy` 1 **（ before not in ）， need 
//! **** 1 to —— not is ， as is that id。
//! （Cline that ） in this will then 。
//!
//! not ： etc 、none when ** not **（path up not 
//! must as ），return `Option<String>` as to use 1 。

use super::{current, save};

/// WorkBuddy **defaultenabled**： already disabled（ as by can ），
/// and 。
///
/// ── Whydisabled（2026-10-17， and Qoder 1 ）───────────────────
/// this 3 defaultenabled， its 1 default use 。 before is 「
/// not this ， 」。 in that before ** 
/// **：`/v3/config` `agents[cli].models` is client picker
/// （ 18 ， `data.models` has 60 ）， by `visibility` after ，
/// 「 this 」 by up 。
///
/// in 1 will has ： that ** can ** （`glm-5.3`
/// / `kimi-k3-1` / `minimax-m3` …） in 「 can 」、 is ——
/// use to is 。 and Qoder that is 1 。
///
/// this （ not is ） has use ： 1 is need by 
/// 「 」； is will if need ， this has 。
pub const WORKBUDDY_DEFAULT_ENABLED: &[&str] = &["hy3", "hy4-preview-f", "deepseek-v4.1-flash"];

/// WorkBuddy **default then **： already disabled， not 。
///
/// ── Why not `seed_default_enabled`（2026-10-17）─────────────
/// and [`super::seed_qoder_defaults`] ： this by can 
/// （[`super::seed_hidden_disabled`]， use in `providers::adapter`），
/// 1 will ** can **。
///
/// is as not use （`core::models` `apply_remote` and 
/// `providers::adapter::seed_current_workbuddy_defaults` in ）。
///
/// ⚠️ use up already has old `disabled` ，
/// [`super::migrate_workbuddy_default_enabled`] 1 。
pub fn seed_workbuddy_defaults(_provider: &str, _ids: &[String]) -> Option<String> {
    None
}

/// before after provider id（ use ； 1 `workbuddy::Region`）
const LEGACY_PROVIDER: &str = "workbuddy";

/// WorkBuddy ** then **（2026-10）。
///
/// ── 、Why is 「」─────────────────────────────────
/// before 「WorkBuddy then 」 this 1 when and （ has 1 
/// provider， has 1 then ）。 after is `workbuddy`， is new id，
/// is that then ** use 1 ** —— not ：
/// - use enabled in to 「default use 」（ will ），
/// is ****，client 400 `model_not_found`；
/// - use in down （ provider ）；
/// - 「 this has this 」 in 。
///
/// `workbuddy` down to ：`disabled`（）、
/// `mappings`（ / / etc ）、`custom`（ ）、
/// `capabilities`（ can ）、`seeded`（）。
///
/// ── Why「」 not is 「」（ and Cline that ）───────────
/// `migrate_cline_split` old `cline:` **** to ： that when old id ，
/// then this 。 `workbuddy` in （ is ），
/// will then 、 down 1 no has then empty。
///
/// ── ** 1 **（ use ， this ）──────────────
/// no has Cline that ： after use in 1 
/// （「 this in use 」） will down 1 again to ，
/// use 。 by `account_bootstrap` and 
/// ** use 1 1 **（`config::workbuddy_split_migrated`） 1 。
///
/// new then （ use in after to ） 1 not 。
pub fn migrate_workbuddy_split() -> (SplitMigrationOutcome, Option<String>) {
    let intl = crate::server::core::providers::workbuddy::Region::Intl.provider_id();
    let mut rules = current();
    let mut summary: Vec<String> = Vec::new();

    // 「 has id」= **** ∪ ****。
    //
    // Why need ： is 「 this 」 ， can can has 
    // no has id —— config id （`is_seeded` for workbuddy / raccoon
    // old ） will 1 not up ， use for 
    // （e.g. `hy3`）** is ** ，must 1 。
    // by will this id ， is 「 hy3 、」。
    let mut known_ids: Vec<String> = rules
        .seeded
        .iter()
        .filter_map(|key| key.strip_prefix("workbuddy:"))
        .map(|rest| rest.split('#').next().unwrap_or(rest).to_string())
        .collect();
    for entry in rules.disabled.iter() {
        if entry.provider.as_deref() != Some(LEGACY_PROVIDER) {
            continue;
        }
        if !known_ids.iter().any(|id| id.eq_ignore_ascii_case(&entry.id)) {
            known_ids.push(entry.id.clone());
        }
    }

    // ① ： ** etc **（ before as ）。
    //
    // ── Why is 「」 not is 「 1 」──────────
    // up ， can can ** already **（`restore_cached_catalogs`
    // in `account_bootstrap` before ）： will by default use 。
    // if this 「 no has 」 ， that 「 already use 」 id will 
    // 「 already in 」 —— use enabled in in use ，
    // is ，client 400 `model_not_found`。
    // as ： use ， use ； no use ， 
    // use ****（ default）。
    //
    // **** ：provider as None for 
    // （ that down `cn_disabled` as ， not to delete）， not can 。
    let mut changed = 0usize;
    for id in &known_ids {
        let cn_disabled = rules
            .disabled
            .iter()
            .any(|entry| entry.matches(LEGACY_PROVIDER, id));
        let intl_disabled = rules
            .disabled
            .iter()
            .any(|entry| entry.matches(intl, id));
        if cn_disabled && !intl_disabled {
            rules.disabled.push(super::RuleEntry::new(Some(intl), id));
            changed += 1;
        } else if !cn_disabled && intl_disabled {
            rules.disabled.retain(|entry| {
                !(entry
                    .provider
                    .as_deref()
                    .is_some_and(|owner| owner.eq_ignore_ascii_case(intl))
                    && entry.id.eq_ignore_ascii_case(id))
            });
            changed += 1;
        }
    }
    if changed > 0 {
        summary.push(format!("启停同步 {changed} 条"));
    }

    // ② / （`(alias, target, provider)` 1 ）。
    // not ， this 「 no has 」 is 。
    let mut added = 0usize;
    let copies: Vec<super::Mapping> = rules
        .mappings
        .iter()
        .filter(|mapping| mapping.provider.as_deref() == Some(LEGACY_PROVIDER))
        .filter(|mapping| {
            !rules.mappings.iter().any(|existing| {
                existing.alias.eq_ignore_ascii_case(&mapping.alias)
                    && existing.target.eq_ignore_ascii_case(&mapping.target)
                    && existing
                        .provider
                        .as_deref()
                        .is_some_and(|owner| owner.eq_ignore_ascii_case(intl))
            })
        })
        .cloned()
        .collect();
    for mut mapping in copies {
        mapping.provider = Some(intl.to_string());
        rules.mappings.push(mapping);
        added += 1;
    }
    if added > 0 {
        summary.push(format!("映射 {added} 条"));
    }

    // ③ （`(provider, id)`）
    let mut added = 0usize;
    let copies: Vec<super::CustomModel> = rules
        .custom
        .iter()
        .filter(|item| item.provider.eq_ignore_ascii_case(LEGACY_PROVIDER))
        .filter(|item| !rules.custom.iter().any(|existing| existing.matches(intl, &item.id)))
        .cloned()
        .collect();
    for mut item in copies {
        item.provider = intl.to_string();
        rules.custom.push(item);
        added += 1;
    }
    if added > 0 {
        summary.push(format!("自定义模型 {added} 条"));
    }

    // ④ can （`(provider, id)`）
    let mut added = 0usize;
    let copies: Vec<super::CapabilityOverride> = rules
        .capabilities
        .iter()
        .filter(|item| item.provider.eq_ignore_ascii_case(LEGACY_PROVIDER))
        .filter(|item| !rules.capabilities.iter().any(|existing| existing.matches(intl, &item.id)))
        .cloned()
        .collect();
    for mut item in copies {
        item.provider = intl.to_string();
        rules.capabilities.push(item);
        added += 1;
    }
    if added > 0 {
        summary.push(format!("能力位 {added} 条"));
    }

    // ⑤ ：`workbuddy:<id>[#alias:<>]` → 1 `workbuddy-intl:< 1 >`。
    // ， down 1 will 「enabled」 by 
    // （）。 is 「」， and none。
    let mut added = 0usize;
    let prefix = format!("{LEGACY_PROVIDER}:");
    let intl_prefix = format!("{intl}:");
    let copies: Vec<String> = rules
        .seeded
        .iter()
        .filter_map(|key| key.strip_prefix(&prefix))
        .map(|rest| format!("{intl_prefix}{rest}"))
        .filter(|key| !rules.seeded.iter().any(|existing| existing.eq_ignore_ascii_case(key)))
        .collect();
    for key in copies {
        rules.seeded.push(key);
        added += 1;
    }
    if added > 0 {
        summary.push(format!("种子标记 {added} 条"));
    }

    if summary.is_empty() {
        return (SplitMigrationOutcome::Nothing, None);
    }
    if !save(&rules) {
        // then no ： use ** not ** 1 ， down 1 ——
        // 「、 then no 」（ after will use in 
        // ）。
        return (
            SplitMigrationOutcome::Failed,
            Some(format!(
                "⚠️  WorkBuddy 拆家的模型规则同步未能落盘（{}）：下次启动会重试",
                summary.join("；")
            )),
        );
    }
    (
        SplitMigrationOutcome::Done,
        Some(format!(
            "🔀 WorkBuddy 已拆为国内版 / 国际版两家，存量模型规则已同步（{}）\
             —— 两家的启用、别名与能力位保持一致",
            summary.join("；")
        )),
    )
}

/// 1 then （ use need not need down 1 ）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SplitMigrationOutcome {
    /// no has can （ new / no then ）—— can 
    Nothing,
    /// already and success —— can 
    Done,
    /// has need ** no ** —— ** not need **， down 
    Failed,
}

impl SplitMigrationOutcome {
    /// this is down 「 already 」
    pub fn allows_marking_done(self) -> bool {
        !matches!(self, Self::Failed)
    }
}
