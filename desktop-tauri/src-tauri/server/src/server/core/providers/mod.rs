//! （provider）：Agent2API up ** and **。
//!
//! ── Why need has this ──────────────────────────────────────
//! before has WorkBuddy 1 up ，provider is ： is 
//! workbuddy 、 in `endpoints.rs`、 in `auth.rs`。
//! （ [`PROVIDERS`]， and ， 
//! `mod.rs` module header） after ，
//! 「 this 」「 this 1 」 need need 1 
//! 1 —— is 。
//!
//! `provider` is **provider id **
//! （`"workbuddy"` / `"raccoon"` / `"catpaw"` / `"autoclaw"` / `"qoder"` /
//! `"cline-free"` / `"cline-pass"`）： need accounts.json、 need in HTTP response、
//! need before use ，
//! ** is **， not can 。
//! id ↔ `ProviderKind` ↔ `ProviderMeta` ， this 
//! to account_store / api 1 （ 1 not will ， will ）。
//!
//! ── file and its （W4a provider）─────────────
//! file has ** and **：、、 。
//! §4.2 `ProviderAdapter` trait and in `adapter.rs`，
//! in `workbuddy.rs` / `raccoon/` / `catpaw/` / `autoclaw/` /
//! `qoder/`。 use （`pending.rs`，W6 delete） already up 
//! —— in `adapter_for` match is ， new kind when will 
//! to ，「 provider 」 in ， not need need 
//! 。
//!
//! `ProviderKind` / `PROVIDERS` old and ；disabled 
//! ID by `availability` ， not or 。
//! and directory `is_executable_provider` / `executable_provider_ids`，
//! 1 `adapter::adapter_by_id`， not unknown runtime id WorkBuddy。
//!
//! ── Why use not is HashMap ─────────────────────
//! provider is **** （ not is ）， has （ [`PROVIDERS`]）；
//! use `&'static [ProviderMeta]` can `meta()` return `&'static` use 
//! （ no has 、 no has ）， and —— before to `providers`
//! ， for and 。
//!
//! ── ─────────────────────────────────────────────────
//! file，`core::providers` down ：
//!
//! ```text
//! adapter.rs ProviderAdapter （ provider）
//! missing_plugin.rs no when （ 503 + ）
//! catalog.rs directory（ and /v1/models 1 ）
//! router.rs by （ + by → ）
//! catalog_*.rs directory new / （ use ）
//! <provider>/ ****： ↔ provider id ↔ 、
//! ```
//!
//! ** and not in this ** —— 
//! （`agent2api-providers`， not ）。 this is ： not 
//! has 「client up 」 ，、、
//! use config and 。
//!
//! `<provider>/` directory down is ****（review when down ）：
//! - up / path / client / UA or ；
//! - request、、SSE ；
//! - file（client in ）、、/。
//!
//! file「 and 」 this 1 ， not not 。

pub mod adapter;
pub mod availability;
pub mod accio;
pub mod autoclaw;
pub mod catalog;
/// ****（ in after by ，module header）。
/// not ： is use ， by scope JSON。
pub mod catalog_cache;
mod catalog_legacy;
pub mod catalog_refresh;
pub mod cline;
pub mod content_block;
/// （ use baseUrl + apiKey） ****： not is ，
/// not `ProviderKind` / `PROVIDERS`（ `custom_providers` module header）。
/// for is use OpenAI path —— this ** not for up **，
/// use ， no has 。
pub mod custom;
pub mod images;
pub mod loomy;
/// MonkeyCode ：`monkeycode/` ****
/// （`monkeycode` / `monkeycode-intl` is provider） and 。
/// and in 。
pub mod monkeycode;
/// ModelScope（） ：`modelscope/region.rs` ****
/// （`modelscope` / `modelscope-intl` is provider）。、 to and 
/// in 。
pub mod modelscope;
pub mod onboarding_memory;
/// ****： 1 in 「 no not 」 when ，
/// （current 1 use is ZCode ， and module header）。
/// not ： is use ， by provider id 。
pub mod readiness;
pub mod policy;
/// **** when （ + ）。
///
/// 「client up 」 to 
/// （ not ）， this no has —— this is 
/// 「」 when that path。file 。
pub mod missing_plugin;
pub mod qoder;
pub mod refresh_flight;
pub mod router;
pub mod retry;
pub mod workbuddy;
pub mod zcode;

use serde_json::{json, Value};

/// 。
///
/// `Copy` is ： is ，、 not this has all。
/// `Hash` + `Eq` `HashMap<ProviderKind, _>` this by provider use 
/// （ after 、 by will use to ）。
///
/// = （ before providers need 、 when by ），
/// new in **** and `PROVIDERS`。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ProviderKind {
    /// WorkBuddy ****（provider id `workbuddy`）
    WorkBuddy,
    /// WorkBuddy ****（`workbuddy-intl`）
    ///
    /// ── Why is provider ──────────────────────────
    /// 「 up `edition` 」 after is （
    /// `workbuddy::region` module header）：directory has 1 （ ，
    /// issue #74）、 then has 1 `(provider, id)` empty（none
    /// ，issue #89）、 not （request can can to 1 up ）。
    /// by after ，directory、、 new 、 then 、、、
    /// by provider id 。
    ///
    /// `workbuddy`： is （ will after 
    /// 「unknown provider」）。
    WorkBuddyIntl,
    /// （`raccoon`）
    Raccoon,
    /// CatPaw（）；`catpaw`
    CatPaw,
    /// AutoClaw（ autoglm）****（`autoclaw`）
    ///
    /// **provider id `autoclaw` not **： is `accounts.json`
    /// 。
    AutoClaw,
    /// AutoClaw ****（`autoclaw-intl`）： 1 、 not 
    AutoClawIntl,
    /// Qoder（）** in **（`qoder`）
    Qoder,
    /// Qoder ****（`qoder-intl`）
    QoderIntl,
    /// CodeArts（ as ）；`codearts`
    CodeArts,
    /// Cline ****（`cline-free`）
    ///
    /// ── Why is provider ─────────────────────────
    /// up is 1 up ， ： has 
    /// 、、 then and 。 after can ， then can by 
    /// （ 1 can when to ， by when 1 ）。
    ClineFree,
    /// Cline ****（`cline-pass`）
    ClinePass,
    /// Accio ****（`accio`）
    Accio,
    /// Accio ****（`accio-cn`）
    AccioCn,
    /// ZCode ****（`zcode`）
    Zcode,
    /// ZCode ****（`zcode-intl`）
    ZcodeIntl,
    /// Trae（`trae`）
    ///
    /// ── Why has 1 ────────────────────────────────────────
    /// SOLO and is ** 、 Origin**（ not is ），
    /// not is 1 provider 。 will when kind。
    Trae,
    /// Loomy（）；`loomy`
    Loomy,
    /// KukuAI（「 AI / GenFlowPro」）；`kuku`
    Kuku,
    /// MonkeyCode ****（`monkeycode`）
    MonkeyCode,
    /// MonkeyCode ****（`monkeycode-intl`）
    MonkeyCodeIntl,
    /// Command Code（`commandcode`）
    CommandCode,
    /// Antigravity（Google AI IDE）；`antigravity`
    ///
    /// ── Why is ** 1 **（ no has ）─────────────────────────
    /// no has region parameter、 1 （`sandbox`/`daily`/`prod` is ****
    /// not is ）， not need need ， not need need `region.rs`。
    Antigravity,
    /// OfficeAce（ as / OfficeClaw / jiuwenclaw）；`officeace`
    OfficeAce,
    /// OrcaTerm（）；`orcaterm`
    OrcaTerm,
    /// （`chatglm.cn`）；`chatglm`
    Chatglm,
    /// ModelScope（）****（`modelscope`）
    Modelscope,
    /// ModelScope ****（`modelscope-intl`）
    ModelscopeIntl,
    /// AtomCode（AtomGit / GitCode）；`atomcode`
    Atomcode,
    /// LobsterAI（ has 「 has 」）；`lobsterai`
    LobsterAi,
}

/// 1 。
///
/// is `&'static str`：file ， not need need String and all。
///
/// this ** no has ** by ： use 1 by （ 1 ），
/// provider is ， not has 1 。
pub struct ProviderMeta {
    /// provider id： accounts.json、 HTTP response、 before use 
    pub id: &'static str,
    /// （ before 、 use in /）
    pub label: &'static str,
}

/// （， before `providers` need by ）。
///
/// add provider when ** this ** + 1 `ProviderKind` ： 
/// default（`DEFAULT_PROVIDER_ID`）、id 、 provider 
/// （`kind_from_id` unknown） from —— use not 1 id 。
///
/// used for****（providers need 、directory and when ）
/// and old （ by and when ， as old default by ）。
pub const PROVIDERS: &[ProviderMeta] = &[
    // = ，directory and when 。
    // for / ****（ 1 ， in 
    // will 「」 1 ）， in before （ and 1 ）。
    ProviderMeta { id: "workbuddy", label: "WorkBuddy 国内版" },
    ProviderMeta { id: "workbuddy-intl", label: "WorkBuddy 国际版" },
    ProviderMeta { id: "raccoon", label: "小浣熊" },
    ProviderMeta { id: "catpaw", label: "CatPaw" },
    ProviderMeta { id: "autoclaw", label: "AutoClaw 国内版" },
    ProviderMeta { id: "autoclaw-intl", label: "AutoClaw 国际版" },
    ProviderMeta { id: "qoder", label: "Qoder 中国版" },
    ProviderMeta { id: "qoder-intl", label: "Qoder 国际版" },
    ProviderMeta { id: "cline-free", label: "Cline Free" },
    ProviderMeta { id: "cline-pass", label: "Cline Pass" },
    ProviderMeta { id: "accio", label: "Accio" },
    ProviderMeta { id: "accio-cn", label: "Accio 国内版" },
    ProviderMeta { id: "zcode", label: "ZCode 国内版" },
    ProviderMeta { id: "zcode-intl", label: "ZCode 国际版" },
    ProviderMeta { id: "codearts", label: "CodeArts" },
    ProviderMeta { id: "trae", label: "Trae" },
    ProviderMeta { id: "loomy", label: "Loomy" },
    ProviderMeta { id: "kuku", label: "KukuAI" },
    ProviderMeta { id: "monkeycode", label: "MonkeyCode" },
    ProviderMeta { id: "monkeycode-intl", label: "MonkeyCode 国际版" },
    ProviderMeta { id: "commandcode", label: "Command Code" },
    ProviderMeta { id: "antigravity", label: "Antigravity" },
    // down in ： and before ，
    // directory and when —— 「 when already has 」。
    ProviderMeta { id: "officeace", label: "OfficeAce 果办" },
    ProviderMeta { id: "orcaterm", label: "OrcaTerm" },
    ProviderMeta { id: "chatglm", label: "智谱清言" },
    ProviderMeta { id: "modelscope", label: "ModelScope 国内版" },
    ProviderMeta { id: "modelscope-intl", label: "ModelScope 国际版" },
    ProviderMeta { id: "atomcode", label: "AtomCode" },
    ProviderMeta { id: "lobsterai", label: "LobsterAI" },
];

/// provider id in down （unknown id → None）。
/// use old default by （10/20/30/40 and down ）。
pub fn provider_index(id: &str) -> Option<usize> {
    PROVIDERS.iter().position(|meta| meta.id == id)
}

/// provider id： when no has `provider` （ or as empty）
/// 1 —— workbuddy up when 。
///
/// not is ：、`add_account` 、 use 
/// must is 1 。
///
/// **must and `PROVIDERS[0].id` 1 **（ is 1 ）：
/// default as workbuddy is （ not is 「 1 」）， not 
/// 「」—— that will has in 1 new provider will 
/// 。 this + none not to （`const` 
/// in down up not ）， by up that and 。
pub const DEFAULT_PROVIDER_ID: &str = "workbuddy";

/// provider id → `ProviderKind`；unknown id return `None`。
///
/// ****unknown（ not in this 1 id ）， after by id 。
/// will provider when + 1 match ， not will not 1 。
///
/// ── Why not is `_ => WorkBuddy` （W4a need ）──────
/// has workbuddy + raccoon， is 「raccoon 1 
/// workbuddy」 。 catpaw / autoclaw after that ** will new provider
/// WorkBuddy**：`/api/accounts` `{"provider":"catpaw"}` will 
/// workbuddy workbuddy ，`providerRoute` will —— ，
/// up not 。 in 「**unknown id → None**」： not is not ，
/// by use by （ 400、、）。
///
/// ── （ can ， will ）────
/// `&str` match need need （Rust none for check），：
/// 1. ** in before **：id not in `PROVIDERS` None。 this is 
/// ，`PROVIDERS` is 1 need need ；
/// 2. ** for 「 has 、 this 」** —— that none in 
/// （ this is need ）， use `debug_assert!` in ，
/// release return None（safe side： new provider as 「unknown」 not is 
/// 「 」）。 will `ProviderKind` when ，`kind_id` 
/// match will ， this and this 1 update。
///
/// use for `None` （W4a already ，file ）：
/// - **path**（ Key available、
/// `api::config_api::parse_provider_route`）→ 400「unknown 」，；
/// - **path**（`config.rs` 、`catalog.rs` `all_kinds`）
/// → `filter_map` this （default / not ）， not panic；
/// - **path**（`request_stats::report::provider_label`）→ id；
/// - **path**（`api::accounts::add_account`）→ workbuddy 
/// （client not provider has ）， already in 
/// that match 400（ not is new ， is ）；
/// - `router::route_for_forward` default provider not to → empty（ use 
/// 「 no has available 」）。 is 「 or 」「 」。
pub fn kind_from_id(id: &str) -> Option<ProviderKind> {
    if !PROVIDERS.iter().any(|meta| meta.id == id) {
        return None;
    }
    match id {
        "workbuddy" => Some(ProviderKind::WorkBuddy),
        "workbuddy-intl" => Some(ProviderKind::WorkBuddyIntl),
        "raccoon" => Some(ProviderKind::Raccoon),
        "catpaw" => Some(ProviderKind::CatPaw),
        "autoclaw" => Some(ProviderKind::AutoClaw),
        "autoclaw-intl" => Some(ProviderKind::AutoClawIntl),
        "qoder" => Some(ProviderKind::Qoder),
        "qoder-intl" => Some(ProviderKind::QoderIntl),
        "cline-free" => Some(ProviderKind::ClineFree),
        "cline-pass" => Some(ProviderKind::ClinePass),
        "accio" => Some(ProviderKind::Accio),
        "accio-cn" => Some(ProviderKind::AccioCn),
        "zcode" => Some(ProviderKind::Zcode),
        "zcode-intl" => Some(ProviderKind::ZcodeIntl),
        "codearts" => Some(ProviderKind::CodeArts),
        "trae" => Some(ProviderKind::Trae),
        "loomy" => Some(ProviderKind::Loomy),
        "kuku" => Some(ProviderKind::Kuku),
        "monkeycode" => Some(ProviderKind::MonkeyCode),
        "monkeycode-intl" => Some(ProviderKind::MonkeyCodeIntl),
        "commandcode" => Some(ProviderKind::CommandCode),
        "antigravity" => Some(ProviderKind::Antigravity),
        "officeace" => Some(ProviderKind::OfficeAce),
        "orcaterm" => Some(ProviderKind::OrcaTerm),
        "chatglm" => Some(ProviderKind::Chatglm),
        "modelscope" => Some(ProviderKind::Modelscope),
        "modelscope-intl" => Some(ProviderKind::ModelscopeIntl),
        "atomcode" => Some(ProviderKind::Atomcode),
        "lobsterai" => Some(ProviderKind::LobsterAi),
        // to this = up already 、 this match no has for ：
        // can can is has to `PROVIDERS` this 。；
        // release return None（ up ： can as 「unknown」， not can ）。
        other => {
            debug_assert!(false, "PROVIDERS 里的 id `{other}` 缺少 kind_from_id 分支");
            None
        }
    }
}

/// `ProviderKind` → provider id（`&'static str`， that ）。
///
/// `const fn`（W4a ）： etc need provider id ****
/// （e.g. `account_store::RACCOON_PROVIDER_ID`）， has can in const up down ，
/// is that from ， not is 1 。const fn match、
/// not —— and use 。
pub const fn kind_id(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::WorkBuddy => "workbuddy",
        ProviderKind::WorkBuddyIntl => "workbuddy-intl",
        ProviderKind::Raccoon => "raccoon",
        ProviderKind::CatPaw => "catpaw",
        ProviderKind::AutoClaw => "autoclaw",
        ProviderKind::AutoClawIntl => "autoclaw-intl",
        ProviderKind::Qoder => "qoder",
        ProviderKind::QoderIntl => "qoder-intl",
        ProviderKind::ClineFree => "cline-free",
        ProviderKind::ClinePass => "cline-pass",
        ProviderKind::Accio => "accio",
        ProviderKind::AccioCn => "accio-cn",
        ProviderKind::Zcode => "zcode",
        ProviderKind::ZcodeIntl => "zcode-intl",
        ProviderKind::CodeArts => "codearts",
        ProviderKind::Trae => "trae",
        ProviderKind::Loomy => "loomy",
        ProviderKind::Kuku => "kuku",
        ProviderKind::MonkeyCode => "monkeycode",
        ProviderKind::MonkeyCodeIntl => "monkeycode-intl",
        ProviderKind::CommandCode => "commandcode",
        ProviderKind::Antigravity => "antigravity",
        ProviderKind::OfficeAce => "officeace",
        ProviderKind::OrcaTerm => "orcaterm",
        ProviderKind::Chatglm => "chatglm",
        ProviderKind::Modelscope => "modelscope",
        ProviderKind::ModelscopeIntl => "modelscope-intl",
        ProviderKind::Atomcode => "atomcode",
        ProviderKind::LobsterAi => "lobsterai",
    }
}

/// this id is already （ old 、disabled and and ）。
///
/// ID can ，； is available
/// must `is_executable_provider`。
pub fn is_known_provider_id(id: &str) -> bool {
    kind_from_id(id).is_some() || availability::is_suspended(id) || is_executable_provider(id)
}

/// use FULL-auth ；disabled not and error。
pub(crate) fn is_scoped_runtime_provider(id: &str) -> bool {
    availability::is_runtime_identity(id)
}

/// has provider（ not old MissingPlugin ）。
/// `custom-` has and path， by 。
pub fn is_executable_provider(id: &str) -> bool {
    !availability::is_suspended(id) && crate::server::core::plugin_host::serves_provider(id)
}

/// ID；， not disabled。
pub fn executable_provider_ids() -> Vec<String> {
    crate::server::core::plugin_host::installed_ids()
}

/// provider id → to ； id 。
///
/// ── Whyreturn `String` not is `&str`（ has that ）────
/// （`custom-` before ， `custom_providers`） is **
/// **（ in kv config）， not to `&'static`。 to `&str` has ：
/// leak （ not ， use ） or in （ and config
/// 1 ）。return `String` use 1 —— use is and 
/// HTTP response path， this 「 not 、 not 」 is 。
///
/// ── ──────────────────────────────────────────────
/// 1. `custom-` before → `custom_providers::label_of`（ use to ），
/// not to （ already delete / id） id；
/// 2. its ： return `label`， 。
///
/// id not is 「unknown」 by not ： use up id （
/// 、`owned_by`、）， id when ， 1 
/// 「unknown」 can 。 before this （ `ui/providers.js`）。
///
/// this is 「id → to 」 ** 1 ** —— not need 
/// `match id { "catpaw" => "CatPaw", ... }`： that 1 not will ，
/// will up 1 。
pub fn label_of(id: &str) -> String {
    if id.starts_with(crate::server::core::custom_providers::ID_PREFIX) {
        if let Some(label) = crate::server::core::custom_providers::label_of(id) {
            return label;
        }
        return id.to_string();
    }
    match kind_from_id(id) {
        // ： that is has （ after not this ）。
        Some(kind) => meta(kind).label.to_string(),
        // runtime id not kind ：， after ID。
        None => crate::server::core::plugin_host::lookup(id)
            .map(|adapter| adapter.label().to_string())
            .unwrap_or_else(|| id.to_string()),
    }
}

// ── 「 this has 1 」 already delete（2026-10 ）──────────
// in this by kind to `zcode::OFFICIAL_PROMPT_NOTE` /
// `official_prompt_approx_chars` / `official_blocks_template`。ZCode 
// to after ， that （ up by + ） and 
// up ，。
//
// in by `api::prompt` `prompt.info` —— not in 
// （ has ， in ）。

/// `ProviderKind` → 。
///
/// return `&'static`： is ， use none need or 。
/// use `kind_id` ， enum and not will 1 label/priority。
pub fn meta(kind: ProviderKind) -> &'static ProviderMeta {
    let id = kind_id(kind);
    PROVIDERS
        .iter()
        .find(|meta| meta.id == id)
        // has （`kind_id` return is id）。 not 1 
        // （ has enum ） when to 1 not is panic ——
        // release is panic=abort， panic will use 。
        .unwrap_or(&PROVIDERS[0])
}

/// is this （ has and ）。
///
/// ， not or 。
/// by by `custom-` 。
pub fn serves_or_has_accounts<F>(id: &str, count: F) -> bool
where
    F: Fn(&str) -> usize,
{
    let _ = count;
    is_executable_provider(id)
}

/// need ： old ， runtime ID。
///
/// `{id, label, count, category, source}`； 
/// category / `plugin`。 not 。
/// `account_providers` parameter has 。
pub fn summary_json<F>(count: F, account_providers: &[String]) -> Vec<Value>
where
    F: Fn(&str) -> usize,
{
    let mut list: Vec<Value> = PROVIDERS
        .iter()
        .filter_map(|meta| kind_from_id(meta.id))
        .map(|kind| meta(kind))
        .filter(|detail| serves_or_has_accounts(detail.id, &count))
        .map(|detail| {
            let plugin = crate::server::core::plugin_host::serves_provider(detail.id);
            json!({
                "id": detail.id,
                "label": detail.label,
                "count": count(detail.id),
                "category": if plugin {
                    json!(crate::server::core::plugin_host::provider_category(detail.id))
                } else { json!("agent") },
                "source": if plugin { "plugin" } else { "builtin" },
            })
        })
        .collect();
    let known = |list: &Vec<Value>, id: &str| {
        list.iter()
            .any(|item| item.get("id").and_then(Value::as_str) == Some(id))
    };
    // 2 ：current （label ）
    for id in crate::server::core::plugin_host::installed_ids() {
        if known(&list, &id) {
            continue;
        }
        let Some(adapter) = crate::server::core::plugin_host::lookup(&id) else {
            continue;
        };
        list.push(json!({
            "id": id,
            "label": adapter.label(),
            "count": count(&id),
            "category": crate::server::core::plugin_host::provider_category(&id),
            "source": "plugin",
        }));
    }
    // not ， in etc 。
    let _ = account_providers;
    list
}
