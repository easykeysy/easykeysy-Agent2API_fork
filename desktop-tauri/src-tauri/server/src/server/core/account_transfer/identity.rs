//! 、provider 、 use 、 1 id and 。
//! and new use account_store::account_identity，provider is use 。
//! CodeArts 、Trae and 、 not can ID or 。

use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::server::core::account_store::priority::{
    normalize_priority_value, DEFAULT_PRIORITY, MAX_PRIORITY, MIN_PRIORITY,
};
use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::providers::{
    is_known_provider_id, kind_id, ProviderKind, DEFAULT_PROVIDER_ID,
};

/// 「 this 」 already （ 1 empty has ）。
///
/// not by provider ， is 
/// 。 old `accessToken` / `refreshToken`， is 
/// OfficeAce（`modelAppKey` + `modelAppSecret`）、（`apiKey`）、
/// ZCode（`jwt`） this will 「」—— 、 not 。
///
/// add 1 when if in this （e.g. `xxxToken`），** need use ，
/// need in this 1 **； is 「、failure」， this need need 
/// in 1 1 ， not 。
pub(super) const CREDENTIAL_KEYS: &[&str] = &[
    "accessToken",
    "refreshToken",
    "apiKey",
    "modelAppKey",
    "modelAppSecret",
    "secretAccessKey",
    "securityToken",
    "jwt",
    "accessKeyId",
    "secretKey",
];

/// when id（ by not token， when clientfile）。///
/// ****： provider use this id —— use 
/// ， id after importDesktop nonecreate/ new 。
///
/// AutoClaw （ / 1 id）： is 
/// （ is 1 `auth.json`， not provider）， id need 
/// —— that ， can ， after as 
/// 「 not token」 `available: false`。
pub(super) const RESERVED_DESKTOP_IDS: [&str; 4] = [
    "raccoon-desktop",
    // CatPaw already to ， `desktop-auth` this id is ****
    // （ in ）—— when not can 1 。
    "desktop-auth",
    crate::server::core::account_store::autoclaw_accounts::DESKTOP_ACCOUNT_ID,
    crate::server::core::account_store::autoclaw_accounts::INTL_DESKTOP_ACCOUNT_ID,
];

pub(super) fn is_reserved_desktop_id(id: &str) -> bool {
    if RESERVED_DESKTOP_IDS.iter().any(|known| *known == id) {
        return true;
    }
    // Cline id（`cline-free-desktop` / `cline-pass-desktop`）。
    // ， 「 Cline id not desktop 」 will 
    // this id， 「」 not when 。
    // after in this not is use ： not has Cline ，
    // this id is ****（ in ），must in 。
    [
        kind_id(ProviderKind::ClineFree),
        kind_id(ProviderKind::ClinePass),
    ]
    .iter()
    .any(|provider| *id == format!("{provider}-desktop"))
}

/// `desktop: true` = when use ， not is can 。
pub(super) fn is_desktop_item(item: &Map<String, Value>) -> bool {
    let provider = item.get("provider").and_then(Value::as_str).unwrap_or(DEFAULT_PROVIDER_ID);
    crate::server::core::account_store::account_identity::reference(provider, item)
}

/// provider。
///
/// / null / empty → as WorkBuddy（ old file no has provider）；
/// empty not is already id → Err（ not unknown id WorkBuddy WorkBuddy ）。
///
/// ── before old need by （2026-10）──────────────────────
/// WorkBuddy before has 1 ， in file this ：
/// `{"provider": "workbuddy", "edition": "intl", ...}`。 that provider 
/// is 「 old 1 that 1 」， not is 「 this 」——
/// will （ is ， will 
/// ）， this by `edition` 。（ edition ）
/// is `workbuddy`， as not ； `workbuddy-intl` 
/// new not （ that already is after ）。
///
/// **：`custom-` before **（）。 is 、 not 
/// （ `custom_providers` module header）， by will —— that is 
/// 「、 not 」 。 this before ， in by use in and 
/// file after （`custom_providers::get`）。
pub(super) fn resolve_provider(item: &Map<String, Value>) -> Result<String, String> {
    match item.get("provider") {
        None | Some(Value::Null) => Ok(DEFAULT_PROVIDER_ID.to_string()),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(DEFAULT_PROVIDER_ID.to_string());
            }
            if !trimmed.starts_with(crate::server::core::custom_providers::ID_PREFIX)
                && !is_known_provider_id(trimmed)
            {
                return Err(format!(
                    "未知的提供商 id「{trimmed}」：注册表不认识，不能当作 WorkBuddy 导入"
                ));
            }
            if trimmed == DEFAULT_PROVIDER_ID && !item.get("pluginSecret").is_some_and(|value| !value.is_null()) {
                let region = crate::server::core::providers::workbuddy::Region::from_edition_id(
                    item.get("edition").and_then(Value::as_str),
                );
                return Ok(region.provider_id().to_string());
            }
            if crate::server::core::account_store::is_qoder_family(trimmed) {
                let record = StoredAccount::from_map(item.clone());
                if let Some(region) = crate::server::core::account_store::qoder_accounts::migrated_region(&record) {
                    return Ok(region.provider_id().to_string());
                }
            }
            Ok(trimmed.to_string())
        }
        Some(_) => Err("provider 字段必须是字符串".to_string()),
    }
}


/// ；none when Err（ this failure， not ）。
pub(super) fn identity_of_item(provider: &str, item: &Map<String, Value>) -> Result<String, String> {
    if is_desktop_item(item) {
        return crate::server::core::account_store::account_identity::text(item, &["id"])
            .ok_or_else(|| "桌面端账号缺少独立 id".to_string());
    }
    crate::server::core::account_store::account_identity::for_import(provider, item)
}

/// （ and `identity_of_item` 1 ）； when None。
#[cfg(test)]
pub(super) fn identity_of_record(provider: &str, record: &StoredAccount) -> Option<String> {
    identity_of_item(provider, record.fields()).ok()
}

/// 1 ** 1 ** id： use ， use when 
/// `<id>-2`、`<id>-3`…， to when after ； in not （ not can can ）return None。
///
/// Why not can use `user-<uid>` this ： can can already use 
/// （id empty is ）， id will 1 。
pub(super) fn allocate_unique_id(preferred: &str, taken: &HashSet<String>) -> Option<String> {
    if !taken.contains(preferred) {
        return Some(preferred.to_string());
    }
    for suffix in 2..=9999 {
        let candidate = format!("{preferred}-{suffix}");
        if !taken.contains(&candidate) {
            return Some(candidate);
        }
    }
    let base = format!("{preferred}-{}", crate::server::logging::now_ms());
    if !taken.contains(&base) {
        return Some(base);
    }
    for suffix in 2..=9999 {
        let candidate = format!("{base}-{suffix}");
        if !taken.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// in empty（ in has after ）。
///
/// 1 use is ****（all use 1 ，
/// `priority.rs` module header）， use must all —— 
/// provider will and 。 when return None，** use must**：
/// default will （`next_free_priority` path is that ，
/// use not can ）。
pub(super) fn allocate_priority(used: &[i64]) -> Option<i64> {
    let normalized: Vec<i64> = used
        .iter()
        .map(|value| normalize_priority_value(*value))
        .collect();
    let max = normalized
        .iter()
        .copied()
        .max()
        .unwrap_or(DEFAULT_PRIORITY - 1);
    let mut candidate = max.saturating_add(1);
    while candidate <= MAX_PRIORITY && normalized.contains(&candidate) {
        candidate += 1;
    }
    if candidate <= MAX_PRIORITY {
        return Some(candidate);
    }
    (MIN_PRIORITY..=MAX_PRIORITY).find(|value| !normalized.contains(value))
}

#[cfg(test)]
mod tests {
    //! this ****（/ is 1 ）， not 
    //! —— after need when and file，`api::accounts` that already has 。
    use serde_json::json;

    use super::*;

    fn item(fields: serde_json::Value) -> Map<String, Value> {
        fields.as_object().expect("对象").clone()
    }

    #[test]
    fn trae_identity_is_variant_plus_uid_and_never_falls_through_to_userid() {
        // that "、 not "： by `userId` ，
        // Trae has `uid`（ and workbuddy / catpaw ）。 
        // is when 「 userId（none trae ）」failure。
        assert_eq!(
            "solo:51029416092912",
            identity_of_item(
                "trae",
                &item(json!({"uid": "51029416092912", "variant": "solo", "userId": ""}))
            )
            .expect("应识别出身份")
        );
        assert_eq!(
            "cn:51029416092912",
            identity_of_item("trae", &item(json!({"uid": "51029416092912", "variant": "cn"}))).expect("应识别出身份"),
            "两个谱系是同一个人也是两条记录（与落账号一侧的判重口径一致）"
        );
        // variant when **must** and `identity_of_record` 1 ，
        // otherwise 1 in and ， → 。
        assert_eq!(
            "solo:777",
            identity_of_item("trae", &item(json!({"uid": "777"}))).expect("缺 variant 按 solo"),
        );
        assert!(identity_of_item("trae", &item(json!({"uid": "  ", "userId": "51029416092912"}))).is_err(), "空 uid 不能拿 userId 凑");
    }

    /// **add provider not file**： in use up can 。
    ///
    /// is 「 1 need 」 this —— old 
    /// OfficeAce（ has `modelAppKey`） to `userId` ，「 userId」
    /// failure，、 not 。
    #[test]
    fn generic_chain_covers_providers_without_a_branch() {
        // OfficeAce： has modelAppKey（+ modelAppSecret）， no has uid/userId
        assert_eq!(
            "app-key-1",
            identity_of_item(
                "officeace",
                &item(json!({"modelAppKey": "app-key-1", "modelAppSecret": "s", "id": "officeace-x"}))
            )
            .expect("通用链应认出 modelAppKey")
        );
        // ：apiKey （ and 「 key and 」 path）
        assert_eq!(
            "sk-abc",
            identity_of_item(
                "custom-openai",
                &item(json!({"apiKey": "sk-abc", "id": "custom-openai-1"}))
            )
            .expect("apiKey 即身份")
        );
        // Command Code：key in accessToken
        assert_eq!(
            "user_abc",
            identity_of_item("commandcode", &item(json!({"accessToken": "user_abc"})))
                .expect("accessToken 即身份")
        );
        // when to id（， 1 ，「 id 」 ）
        assert_eq!(
            "atomcode-1",
            identity_of_item("atomcode", &item(json!({"id": "atomcode-1"}))).expect("退到 id")
        );
        // no has = none（ this failure， not ）
        assert!(identity_of_item("officeace", &item(json!({"name": "无身份"}))).is_err());
    }

    /// **must** 1 ： this is can not can before 。
    ///
    /// old 1 then in `identity_of_item` and `identity_of_record` 
    /// 1 ， 1 will 1 not up （）。 in 
    /// use 1 ， this 「 1 → 1 」。
    #[test]
    fn item_and_record_agree_on_every_provider() {
        for provider in [
            "workbuddy", "workbuddy-intl", "raccoon", "catpaw", "autoclaw", "autoclaw-intl",
            "qoder", "qoder-intl", "cline-free", "cline-pass", "codearts", "trae", "loomy",
            "kuku", "commandcode", "antigravity", "officeace", "orcaterm", "chatglm",
            "modelscope", "atomcode", "lobsterai", "zcode",
        ] {
            let fields = item(json!({
                "id": "acc-1",
                "provider": provider,
                "uid": "u-1",
                "userId": "u-1",
                "loginName": "name-1",
                "account": "acc-1@example.com",
                "apiKey": "key-1",
                "accessToken": "token-1",
                "modelAppKey": "app-1",
                "variant": "solo",
                "mode": "cn",
            }));
            let from_item = identity_of_item(provider, &fields);
            let record = StoredAccount::from_map(fields);
            let from_record = identity_of_record(provider, &record);
            assert_eq!(
                from_item.ok(),
                from_record,
                "provider={provider}：导入条目与本机记录算出的身份必须一致"
            );
        }
    }

    /// （ use not can ）。
    #[test]
    fn special_providers_keep_their_multi_segment_identity() {
        // CodeArts：domain + userId （ by userId will down and 1 ）
        assert_eq!(
            "dom-a\u{0}u-1",
            identity_of_item("codearts", &item(json!({"domainId": "dom-a", "userId": "u-1"})))
                .expect("两段身份")
        );
        // Qoder： + userId 
        assert_eq!(
            "cn:u-1",
            identity_of_item("qoder", &item(json!({"userId": "u-1", "mode": "cn"})))
                .expect("两段身份")
        );
        // CatPaw：uid （ use userId in uid before ， will ）
        assert_eq!(
            "uid-1",
            identity_of_item("catpaw", &item(json!({"uid": "uid-1", "userId": "alias-1"})))
                .expect("uid 优先")
        );
    }
}
