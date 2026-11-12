//! Cline then ：default + + before 。
//!
//! ── Why 1 file ──────────────────────────────────────
//! this **Cline / **， and workbuddy / that 
//! （ in `mod.rs`） no has ； and its in 「」 is 1 ，
//! and in in 1 file， not 1 can 。
//! 1 is `mod.rs` to 「 then 」 （：
//! file not 800 ）。
//!
//! ── and `mod.rs` ──────────────────────────────────────
//! this use to is `mod.rs` （`ModelRules` 、`Mapping` /
//! `RuleEntry` 、`save` / `alias_valid` etc ）， not new 。
//! ** not is ** then ， is that Cline use 。

use super::*;

use crate::server::core::providers::cline::models::{friendly_alias, Pool};

/// Cline **default**（ by use ， 1 ）。
///
/// ── Why need （ not is in when before ）─────────────────────
/// Cline up id `cline-free/deepseek-v4.1-flash`（） and 
/// `cline-pass/glm-5.3`（），** before is **， will 404。
/// use id for can use ，client `cline-free/deepseek-v4.1-flash`
/// 、 and `deepseek-v4.1-flash` 。 to before 
/// 1 `deepseek-v4.1-flash → cline-free/deepseek-v4.1-flash` ，
/// can use 。
///
/// ── id （）───────────────────────
/// has not before （`z-ai/glm-5.3-flash`、
/// `poolside/laguna-s-2.1:free`）， this has can 。 by 
/// [`friendly_alias`] by to ： id ** before **（ that is ，
/// and before ），`recommended` `openai/gpt-6-astra` then not 
/// （ that before is ）。 up id（`glm-5.3-flash` is CatPaw
/// ）** not **： 1 for by is by 。
///
/// ── （ and ）─────────────────────────────
/// 1. ** etc **： `(provider, id)` `seeded`， use after this 
/// not will down 1 new ；
/// 2. ** not **：alias already use → ；
/// 3. ** not and up id **：alias and **** up id → 。
/// ** will **：`deepseek-v4.1-flash` has ，** in is 
/// provider** —— `has_alias` is 「 use not 」， that 
/// to ， 1 seeded、 not 。 no to that by 
/// `EXTRA_ALIASES` up （ that ， provider alias）。
///
/// ── `provider` parameter（ after add）────────────────────────────
/// `"cline-free"` / `"cline-pass"`（= `cline::models::Pool::provider_id()`）。
/// ** use need id **（`cline::models::ids_of(pool)`）—— 1 
/// id will this seeded not ， down 1 that when will up ，
/// 1 。
///
/// ── Why「seeded has no 」 need ────────────────
/// and `seed_default_enabled` 1 by ： not down will 1 ，
/// can can —— use will 。
/// is 「 for 」 ， that must。
///
/// return to need ； no has new when return None（ not ）。
pub fn seed_cline_defaults(provider: &str, ids: &[String]) -> Option<String> {
    let added = super::mutations::mutate(|rules| {
        let mut added = Vec::new();
        let pool = Pool::from_provider_id(provider).unwrap_or(Pool::Pass);
        for id in ids {
            let id = id.trim();
            if id.is_empty() { continue; }
            seed_extra_aliases(rules, provider, id, ids, &mut added);
            if rules.is_seeded(provider, id) { continue; }
            rules.seeded.push(format!("{provider}:{id}"));
            let Some(alias) = friendly_alias(pool, id) else { continue };
            if !alias_valid(alias) || ids.iter().any(|id| id.eq_ignore_ascii_case(alias)) { continue; }
            if rules.mappings.iter().any(|mapping| mapping.alias.eq_ignore_ascii_case(alias)
                && mapping.provider.as_deref().map_or(true, |owner| owner.eq_ignore_ascii_case(provider)))
            { continue; }
            rules.mappings.push(Mapping {
                alias: alias.to_string(), target: id.to_string(), provider: Some(provider.to_string()),
                reasoning: None, enabled: true,
            });
            added.push(format!("{alias} → {id}"));
        }
        Ok(added)
    }).ok()?;
    if added.is_empty() { return None; }
    Some(format!("🧩 Cline 模型默认映射: [{}]", added.join(", ")))
}

// ─── Cline as provider ──────────────────

/// before Cline provider id（** in **， already none this id）
const LEGACY_CLINE_ID: &str = "cline";

/// config** old Cline this 1 ** then to after 。
///
/// ── Whymust（ this is 1 ）──────────────
/// then is **(provider, id)**，provider id 1 ， old not up 。
/// not after not is 「 1 set」， is ****：
/// - `disabled` `provider: "cline"` → use use **
/// enabled**， and not to that （ that 1 already not up ）；
/// - `seeded` `cline:<id>` → as 「 this from no 」， is 
/// 1 。 use will **** —— this is 
/// `seeded` in need ， 1 。
///
/// ── （ and file its ）──────────────
/// 1 use `cline::models::pool_of(id)` by ** id before **：
/// - id is up id（ is id），；
/// - up ；provider old ** in target 
/// before when provider** —— `cline-free/…` / `cline-pass/…` this id is 
/// Cline has ， up provider is ；none before id（ `glm-5.3-flash`）
/// can can when ， to 1 will is ** as **， not ；
/// - `seeded` id can `#alias:<>` after （），
/// `#` before that 。
///
/// etc ： after 1 not will in 1 （`cline:` and `cline-free:` /
/// `cline-pass:` in etc down not —— `cline-free:` 6 is `-`，
/// not will `starts_with("cline:")`）。 can in when use 。
///
/// no has when return None，** not **（ and 1 ：path up 
/// not must as ）。
pub fn migrate_cline_split() -> Option<String> {
    let mut rules = current();
    let mut touched = false;
    let mut summary: Vec<String> = Vec::new();

    // ① ：provider （`hidden` already 「delete/」，
    // `disabled` —— old config hidden in ，
    // mod.rs 「 1 」）
    let mut count = 0usize;
    for entry in rules.disabled.iter_mut() {
        if entry.provider.as_deref() != Some(LEGACY_CLINE_ID) {
            continue;
        }
        let pool = crate::server::core::providers::cline::models::pool_of(&entry.id);
        entry.provider = Some(pool.provider_id().to_string());
        count += 1;
    }
    if count > 0 {
        touched = true;
        summary.push(format!("禁用 {count} 条"));
    }

    // ② ： old provider ；provider target is Cline before up 
    let mut renamed = 0usize;
    let mut tagged = 0usize;
    for mapping in rules.mappings.iter_mut() {
        match mapping.provider.as_deref() {
            Some(LEGACY_CLINE_ID) => {
                let pool = cline_pool_of(&mapping.target).provider_id();
                mapping.provider = Some(pool.to_string());
                renamed += 1;
            }
            // old ： in target ** before ** when （）
            None if is_cline_channel(&mapping.target) => {
                let pool = cline_pool_of(&mapping.target).provider_id();
                mapping.provider = Some(pool.to_string());
                tagged += 1;
            }
            _ => {}
        }
    }
    if renamed > 0 || tagged > 0 {
        touched = true;
        summary.push(format!("映射 {renamed} 条改名 / {tagged} 条补归属"));
    }

    // ③ ：`cline:<id>[#alias:<>]` → `< provider id>:< 1 >`
    let mut reseeded = 0usize;
    for key in rules.seeded.iter_mut() {
        let Some(rest) = key.strip_prefix("cline:") else {
            continue;
        };
        // id and `#alias:` after ， use id 
        let id_part = rest.split('#').next().unwrap_or(rest);
        let pool = cline_pool_of(id_part).provider_id();
        *key = format!("{pool}:{rest}");
        reseeded += 1;
    }
    if reseeded > 0 {
        touched = true;
        summary.push(format!("种子标记 {reseeded} 条"));
    }

    if !touched {
        return None;
    }
    save(&rules);
    Some(format!(
        "🔀 Cline 已拆为 Cline Free / Cline Pass 两家，存量规则已迁移（{}）",
        summary.join("；")
    ))
}

/// id → （** use ** ）。
///
/// has 1 （`cline::models::pool_of`， by id before ）， this 
/// is that path 1 —— need ，path will no 
/// 。
fn cline_pool_of(model_id: &str) -> crate::server::core::providers::cline::models::Pool {
    crate::server::core::providers::cline::models::pool_of(model_id)
}

/// this id is not is Cline **** id（ `cline-free/` / `cline-pass/` before ）。
///
/// use 「Cline has id」 and 「 can can 」：
/// before can old to 1 ， after not can （ `migrate_cline_split`）。
fn is_cline_channel(model_id: &str) -> bool {
    ["cline-free/", "cline-pass/"]
        .iter()
        .any(|prefix| model_id.starts_with(prefix))
}

