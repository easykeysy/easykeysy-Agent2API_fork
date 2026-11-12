//! / （ for src/workbuddy-account-transfer.mjs）。
//!
//! this is ****： not 、 not ，// by store 
//! （`AccountStore` load_locked / save_locked / with_lock）。
//!
//! accessToken / refreshToken： after use ，
//! not 1 。 rateLimits is when 、 down is 
//! ， no has ， 1 not 。
//!
//! ── then （）────────────────────────────────
//! 1. **must provider**：identity as 「provider + 」（WorkBuddy
//! uid / CatPaw uid|loginName / raccoon、AutoClaw userId / Qoder +userId /
//! Cline account / apiKey）。 not and ， not 
//! provider —— id and uid empty， by uid will 
//! （ 1 ）。
//! 2. file**** provider must is already id or `custom-` before 
//! （ after need in ， down ）； provider（ old file）
//! by WorkBuddy ， not in provider。
//! 3. **id and in 1 1 for **：update by in after id（file
//! id is add when ， can can ）；add when if id already 
//! provider 1 use then this failure（ not 、 not ）， 
//! use then as id ， 1 new id。
//! 4. **priority and addedAt 1 **： by ， 1 
//! file not this 。 new in ****
//! （ 1 use is use 1 ， and path 
//! `next_free_priority` ；failure， not default）。
//! 5. ** when **（`desktop: true` or id） not ： 
//! by not file， use if empty 1 「 has none」 will 
//! to 401。 this skipped and already has importDesktop 。
//!
//! ── （`custom-` before ）───────────────────────────
//! and is ****： `provider` to （）。
//! v2 file 1 `customProviders`（， not —— apiKey in 
//! ）， when ** and 、**， not empty。
//! is apiKey（ can empty， and path）、 is apiKey（ then id）；
//! file has and after not in （ up has 「file 」），
//! for failure and Reason。
//!
//! 1 failed / skipped and ， not 1 。
//! return `{ total, added, updated, skipped, failed, errors, customProviders }`。
//!
//! ：identity.rs and id ；normalize.rs 1 。

mod export;
mod identity;
mod normalize;

pub use export::{export_account_files, ExportSelection, ACCOUNT_FILE_FORMAT};

use serde_json::{json, Map, Value};

use crate::server::core::account_store::state::StoredAccount;
use crate::server::core::account_store::store::{AccountStore, AccountStoreError};
use crate::server::core::account_store::MAX_TOKEN_LENGTH;
use crate::server::core::providers::kind_id;
use crate::server::core::providers::ProviderKind;

use identity::{
    allocate_priority, allocate_unique_id, identity_of_item,
    is_desktop_item, is_reserved_desktop_id, resolve_provider, CREDENTIAL_KEYS,
};
use normalize::normalize_imported;

/// file（ after ）。
///
/// v2：add `customProviders` （）。v1 file no has this 1 ，
/// by 「 no has can and 」 —— in v1 file not ，
/// not in 。
pub const EXPORT_VERSION: i64 = 2;

/// 
enum ImportOutcome {
    Updated(String),
    Added(String),
    Skipped(String),
}

/// statistics。
///
/// （ use etc ） and failure（）： `failed` 
/// failure，`errors` failure、（ `skipped: true` ），
/// is before 「failure」 not will Reasonfailure。
#[derive(Default)]
struct ImportStats {
    added: usize,
    updated: usize,
    skipped: usize,
    failures: Vec<Value>,
    skipped_items: Vec<Value>,
    imported_ids: Vec<String>,
}

/// → ：， rateLimits（ when 
/// ） and down 。
fn to_export_record(record: &StoredAccount) -> Value {
    let mut item = Map::new();
    for (key, value) in record.fields() {
        if key.starts_with('_') || key == "rateLimits" {
            continue;
        }
        // `value === undefined ? null : value`：JSON not in undefined，
        // this need （ is ）
        item.insert(key.clone(), value.clone());
    }
    // 1 ：token and to ， and use for 
    let access_token = record.access_token();
    let refresh_token = record.refresh_token();
    item.insert("accessToken".to_string(), Value::String(access_token));
    item.insert("refreshToken".to_string(), Value::String(refresh_token));
    item.insert("proxy".to_string(), record.proxy());
    // provider is 1 ：， and as 
    // （ by provider() WorkBuddy）
    item.insert("provider".to_string(), Value::String(record.provider()));
    Value::Object(item)
}

/// （ after use ）， and up 。
///
/// in ****：`custom_providers::list()` config ，
/// not （ and 「 and 、 after 」 1 ）。
/// not —— apiKey is ， already 。
pub fn export_accounts(store: &AccountStore) -> Value {
    let accounts = store.with_lock(|guard| {
        let state = store.load_locked(guard);
        state.accounts.iter().map(to_export_record).collect::<Vec<_>>()
    });
    json!({
        "version": EXPORT_VERSION,
        "exportedAt": crate::server::logging::now_ms(),
        "customProviders": crate::server::core::custom_providers::list(),
        "accounts": accounts,
    })
}

/// （file ），merge file。
///
/// return `{ total, added, updated, skipped, failed, errors, customProviders }`。
pub fn import_accounts(store: &AccountStore, payload: &Value) -> Result<Value, AccountStoreError> {
    let Some(root) = payload.as_object() else {
        return Err(AccountStoreError::new("导入内容必须是 JSON 对象", 400));
    };
    let Some(items) = root.get("accounts").and_then(Value::as_array) else {
        return Err(AccountStoreError::new("缺少 accounts 数组", 400));
    };
    if items.is_empty() {
        return Err(AccountStoreError::new("accounts 为空，没有可导入的账号", 400));
    }

    // （ not ）， after and errors to before 
    let mut definition_warnings: Vec<Value> = Vec::new();
    // ── and ──
    // provider to this （）； and and config、
    // not —— in with_lock before ， not 。 this not is 
    // by 400（： not this ）。
    let (providers_added, providers_updated) = match root.get("customProviders") {
        Some(Value::Array(definitions)) => {
            let report = crate::server::core::custom_providers::transfer::merge_imported(
                definitions,
            )
            .map_err(|message| AccountStoreError::new(message, 500))?;
            definition_warnings.extend(report.warnings.into_iter().map(|warning| {
                json!({
                    "id": warning.id,
                    "message": warning.message,
                    "customProvider": true,
                })
            }));
            (report.added, report.updated)
        }
        Some(_) => {
            return Err(AccountStoreError::new("customProviders 必须是数组", 400));
        }
        None => (0, 0),
    };

    let (result, catpaw_invalidations) = store.with_lock(|guard| {
        let mut state = store.load_locked(guard);
        let mut stats = ImportStats::default();
        let mut taken_ids: Vec<String> = state
            .accounts
            .iter()
            .map(|item| item.id().to_string())
            .collect();
        // need need will CatPaw （， and after use ）
        let mut catpaw_invalidations: Vec<String> = Vec::new();

        for item in items {
            let outcome = import_one_item(
                &mut state,
                item,
                &mut taken_ids,
                &mut catpaw_invalidations,
            );
            match outcome {
                Ok(ImportOutcome::Updated(id)) => {
                    stats.updated += 1;
                    stats.imported_ids.push(id);
                }
                Ok(ImportOutcome::Added(id)) => {
                    stats.added += 1;
                    stats.imported_ids.push(id);
                }
                Ok(ImportOutcome::Skipped(reason)) => {
                    let id = error_id_of(item);
                    stats.skipped += 1;
                    stats
                        .skipped_items
                        .push(json!({ "id": id, "message": reason, "skipped": true }));
                }
                Err(message) => {
                    let id = error_id_of(item);
                    stats.failures.push(json!({ "id": id, "message": message }));
                }
            }
        }

        crate::server::core::account_store::codearts_accounts::migrate_limiters(&mut state.accounts);
        store.save_locked(&state, guard)?;
        let failed = stats.failures.len();
        // errors ：failure → → （ skipped 、
        // customProvider ， before can ）
        let mut errors = stats.failures.clone();
        errors.extend(stats.skipped_items.iter().cloned());
        errors.extend(definition_warnings.iter().cloned());
        // by **** to ， not ：`failure {failed} ` is none ，
        // is 「failure 0 」 this success 「failure」， will 
        // infer_level error —— statistics error， etc 1 
        // 。（ and scheduled_tasks.rs total 1 。）
        crate::server::logging::log_with_level(
            "[Accounts]",
            &format!(
                "📥 账号导入完成: 共 {} 条，新增 {} 个，更新 {} 个，跳过 {} 条，失败 {failed} 条\
                 （自定义提供商定义：新增 {} 家、更新 {} 家）",
                items.len(),
                stats.added,
                stats.updated,
                stats.skipped,
                providers_added,
                providers_updated,
            ),
            if failed > 0 { "error" } else { "info" },
        );
        let result = json!({
            "total": items.len(),
            "added": stats.added,
            "updated": stats.updated,
            "skipped": stats.skipped,
            "failed": failed,
            "errors": errors,
            "importedIds": stats.imported_ids,
            "customProviders": {
                "added": providers_added,
                "updated": providers_updated,
            },
        });
        Ok::<_, AccountStoreError>((result, catpaw_invalidations))
    })?;

    // already 、 already ： CatPaw will （ not ，
    // and store_crud remove_account / update_account 1 ）
    for id in catpaw_invalidations {
        store.invalidate_catpaw_sessions(&id, kind_id(ProviderKind::CatPaw));
    }
    Ok(result)
}

/// failure use ： id， its uid / userId
fn error_id_of(item: &Value) -> String {
    let text = |key: &str| {
        item.get(key)
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string()
    };
    for key in ["id", "uid", "userId", "loginName"] {
        let value = text(key);
        if !value.is_empty() {
            return value;
        }
    }
    String::new()
}

/// 。
///
/// return `Ok(Updated/Added/Skipped)` or `Err(Reason)`（ this failure，）。
fn import_one_item(
    state: &mut crate::server::core::account_store::state::AccountState,
    item: &Value,
    taken_ids: &mut Vec<String>,
    catpaw_invalidations: &mut Vec<String>,
) -> Result<ImportOutcome, String> {
    let Some(object) = item.as_object() else {
        return Err("账号记录必须是 JSON 对象".to_string());
    };
    let item_id = object
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if item_id.is_empty() {
        return Err("缺少 id".to_string());
    }

    // ── when use ： not （file 5 ）──
    // this in provider before ： use not is can ，
    // none provider is ， by 「」 not is 「failure」。
    if is_desktop_item(object) || is_reserved_desktop_id(&item_id) {
        return Ok(ImportOutcome::Skipped(
            "桌面端实时登录态账号不随导出文件迁移（凭证在本机客户端存储，不落账号文件）；\
             请在目标机器用「导入桌面端登录态」重新读取"
                .to_string(),
        ));
    }

    // ── provider ： provider must is already id or already in ──
    let provider = resolve_provider(object)?;

    // ** in **（ not ， can in this ）：
    // and in before already ，file to this in ；
    // not to = file no has this 、 no —— is 
    // （、failure），failure。
    if provider.starts_with(crate::server::core::custom_providers::ID_PREFIX)
        && crate::server::core::custom_providers::get(&provider).is_none()
    {
        return Err(format!(
            "自定义提供商「{provider}」不存在：导出文件里没有它的定义，本机也没有创建过"
        ));
    }

    // ── ： need has 1 available （ has use not can ）──
    //
    // ── Why not can accessToken / refreshToken ─────────────────
    // not ： is `apiKey`（ and can empty —— and path，
    // up not need need key）、OfficeAce is `modelAppKey` + `modelAppSecret` that 1 for 、
    // ZCode `jwt`、Command Code key in `accessToken`。 old 
    // before ， is OfficeAce **、 not **（
    // 「 accessToken and refreshToken」）—— 1 need 1 ， is 
    // this need 。
    //
    // in by 「 has no has **** 1 already 」： 1 
    // （`CREDENTIAL_KEYS`），add 1 when need in this up not use this 。
    let is_custom_provider =
        provider.starts_with(crate::server::core::custom_providers::ID_PREFIX);
    let access_token = object
        .get("accessToken")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let refresh_token = object
        .get("refreshToken")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let plugin_secret = object.get("pluginSecret").filter(|value| !value.is_null());
    let has_any_credential = plugin_secret.is_some()
        || CREDENTIAL_KEYS.iter().any(|key| {
            object
                .get(*key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        });
    // apiKey can empty（`noAuth` ）， not need 
    if !is_custom_provider && !has_any_credential {
        return Err("缺少凭证（accessToken / refreshToken / apiKey / modelAppKey 等均为空）"
            .to_string());
    }
    if access_token.chars().count() > MAX_TOKEN_LENGTH
        || refresh_token.chars().count() > MAX_TOKEN_LENGTH
    {
        return Err("token 过长".to_string());
    }

    // ── ：provider + （ not ， identity.rs）──
    identity_of_item(&provider, object)?;

    // id up when when ： id not can 
    // （ in client， will when 1 ）
    if state
        .accounts
        .iter()
        .any(|record| record.id() == item_id && is_desktop_item(record.fields()))
    {
        return Ok(ImportOutcome::Skipped(format!(
            "账号 id「{item_id}」在本机是桌面端实时登录态（凭证在客户端存储），\
             通用导入不改写它；如需刷新请在目标机器用「导入桌面端登录态」"
        )));
    }

    // provider （ provider use ： uid not will ）。
    // not and ： and can is 1 ，
    // and will 「 when 」。
    let matched = state.accounts.iter().position(|record| {
        !is_desktop_item(record.fields())
            && record.provider() == provider
            && crate::server::core::account_store::account_identity::same(&provider, record.fields(), object)
    }).or_else(|| state.accounts.iter().position(|record| {
        !is_desktop_item(record.fields())
            && record.provider() == provider
            && record.id() == item_id
            && !crate::server::core::account_store::account_identity::conflicts(&provider, record.fields(), object)
    }));
    if let Some(index) = matched {
        // provider 「id ↔ 」must 1 ： id if already to provider 
        // 1 ，file and —— this failure， not not 
        if state.accounts[index].id() != item_id {
            let points_elsewhere = state.accounts.iter().any(|record| {
                record.id() == item_id
                    && record.provider() == provider
                    && crate::server::core::account_store::account_identity::conflicts(&provider, record.fields(), object)
            });
            if points_elsewhere {
                return Err("导入的账号 ID 在本提供商内指向另一条业务身份，未覆盖原账号".to_string());
            }
        }
        // provider + → and to ，** id**：
        // file id is add when （ can can ），
        // by id will 「id 1 」 and etc 。
        let before = state.accounts[index].clone();
        let normalized = normalize_imported(object, &provider, Some(&before))?;
        {
            let target = state.accounts[index].fields_mut();
            for (key, value) in normalized {
                target.insert(key, value);
            }
        }
        // WorkBuddy not this ： old 。
        // 「 is not is WorkBuddy 」，** not ** —— add 1 provider
        // when not need need 1 （ this is old ）。
        if !crate::server::core::providers::workbuddy::is_workbuddy_family(&provider) {
            for key in ["edition", "prefixPath", "endpoint", "platform"] {
                state.accounts[index].remove(key);
            }
        }
        state.accounts[index].set("updatedAt", Value::from(crate::server::logging::now_ms()));
        if provider == kind_id(ProviderKind::CatPaw) {
            // update： this CatPaw will must in 
            catpaw_invalidations.push(state.accounts[index].id().to_string());
        }
        return Ok(ImportOutcome::Updated(state.accounts[index].id().to_string()));
    }

    // ── add ──
    // to this provider no has ： if this id provider 
    // use ， is file —— failure not is ， not 
    if let Some(existing) = state.accounts.iter().find(|record| record.id() == item_id) {
        if existing.provider() == provider {
            return Err("导入的账号 ID 在本提供商内已属于另一条业务身份，未覆盖原账号".to_string());
        }
    }
    // id empty is **** （ use 1 ）： use when ，
    // 1 1 id（ not ， not can can ）
    let taken: std::collections::HashSet<String> = taken_ids.iter().cloned().collect();
    let Some(new_id) = allocate_unique_id(&item_id, &taken) else {
        return Err("导入账号 ID 冲突，无法分配独立 ID，未覆盖原账号".to_string());
    };

    let normalized = normalize_imported(object, &provider, None)?;
    let now = crate::server::logging::now_ms();
    let mut record = Map::new();
    record.insert("id".to_string(), Value::String(new_id.clone()));
    // provider and （uid / loginName / userId） already by normalize 
    for (key, value) in normalized {
        record.insert(key, value);
    }
    // priority in ****： 1 use is use 1 
    // （ `priority.rs` module header），must all ——
    // provider will and （）。
    // failure（ not not ， not default）
    let used: Vec<i64> = state.accounts.iter().map(StoredAccount::priority).collect();
    let Some(priority) = allocate_priority(&used) else {
        return Err("优先级号段（0-9999）已用满，无法为新账号分配优先级".to_string());
    };
    record.insert("priority".to_string(), Value::from(priority));
    record.insert("addedAt".to_string(), Value::from(now));
    record.insert("updatedAt".to_string(), Value::from(now));
    // no has ： when 
    record.remove("desktop");
    let mut saved = StoredAccount::from_map(record);
    if saved.name().is_empty() {
        let fallback = {
            let nickname = saved.nickname();
            if nickname.is_empty() {
                format!("账号 {}", state.accounts.len() + 1)
            } else {
                nickname
            }
        };
        saved.set("name", Value::String(fallback));
    }
    state.accounts.push(saved);
    // id use ： after not will to 1 id
    taken_ids.push(new_id.clone());
    Ok(ImportOutcome::Added(new_id))
}
