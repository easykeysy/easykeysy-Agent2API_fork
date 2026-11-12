//! **** and （file `customProviders` ）。
//!
//! ── Why need up ──────────────────────────
//! `provider` is 「 to 」（`custom-…` id）。
//! not ， after that will to 1 not in ：、
//! failure。 v2 file 1 `customProviders`（ not 
//! —— apiKey is ， in ）， when and this 1 、
//! ， not will empty。
//!
//! ── and （ and 1 to ）────────────────────────────
//! - by id ： in → use fileupdate（ `createdAt` —— 
//! and 「」，file when 1 ）；
//! in → ，`createdAt` current when （ `addedAt` ）。
//! - **id not new **：， id down 。
//! 12 hex can ignore， not not id 。
//! - file id ： to （ and `validate_model_entries` to 
//! 1 ）， after to 1 。
//!
//! 1 and ， not ；failurefailure
//! （ and `create` / `update` 1 —— new no has is failure）。
//!
//! file in `custom_providers` down not is `account_transfer` down ：
//! （ 、models/mappings 1 ） is has ，
//! and use `item_of` / `model_entries_of` / `mapping_entries_of`，
//! in not that to 。

use std::collections::HashSet;

use serde_json::{Map, Value};

// has and ： can （Rust then ）， need 
use super::{item_of, normalize_base_url, read_items, write_items, ID_PREFIX};
use crate::server::logging;

/// 1 （： and `errors` ）
pub(crate) struct DefinitionWarning {
    pub id: String,
    pub message: String,
}

/// and ：add / update， and 
pub(crate) struct MergeReport {
    pub added: usize,
    pub updated: usize,
    pub warnings: Vec<DefinitionWarning>,
}

/// file and config and 。
///
/// `items` is file `customProviders` down ；return when ** already **。
/// failure（ not success）return Err， use mustfailure。
pub(crate) fn merge_imported(items: &[Value]) -> Result<MergeReport, String> {
    let mut current = read_items();
    let mut report = MergeReport { added: 0, updated: 0, warnings: Vec::new() };
    let mut seen: HashSet<String> = HashSet::new();

    for item in items {
        let Some(object) = item.as_object() else {
            report.warnings.push(DefinitionWarning {
                id: String::new(),
                message: "自定义提供商定义必须是 JSON 对象，该条已跳过".to_string(),
            });
            continue;
        };
        let id = object
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        // before and `is_custom_provider_id` 1 1 ： not `custom-` before 
        // id 1 not （file not will has ， this is ）
        if id.is_empty() || !id.starts_with(ID_PREFIX) {
            report.warnings.push(DefinitionWarning {
                id,
                message: "自定义提供商定义缺少合法 id（custom- 前缀），该条已跳过".to_string(),
            });
            continue;
        }
        if !seen.insert(id.clone()) {
            report.warnings.push(DefinitionWarning {
                id,
                message: "导出文件里重复出现的自定义提供商定义，仅采纳第一条".to_string(),
            });
            continue;
        }
        let normalized = match normalize_item(object) {
            Ok(value) => value,
            Err(message) => {
                report.warnings.push(DefinitionWarning { id, message });
                continue;
            }
        };
        match current
            .iter()
            .position(|entry| entry.get("id").and_then(Value::as_str) == Some(id.as_str()))
        {
            Some(index) => {
                // update： createdAt （，module header）， its file
                let mut merged = normalized;
                if let Some(local) = current[index].get("createdAt").and_then(Value::as_i64) {
                    if local > 0 {
                        merged["createdAt"] = Value::from(local);
                    }
                }
                current[index] = merged;
                report.updated += 1;
            }
            None => {
                // add：createdAt current when （ addedAt ）
                let mut entry = normalized;
                entry["createdAt"] = Value::from(logging::now_ms());
                current.push(entry);
                report.added += 1;
            }
        }
    }

    if !write_items(&current) {
        return Err("保存失败：自定义提供商定义写入未成功（请检查磁盘空间与配置目录权限）".to_string());
    }
    Ok(report)
}

/// 1 → （ use 1 ， up has ）。
///
/// `item_of` default and models/mappings 1 （、）；
/// this 「」：baseUrl must is URL（create ，
/// is —— empty failure， is ）。
/// not ：`item_of` will id， and path 1 to 。
fn normalize_item(object: &Map<String, Value>) -> Result<Value, String> {
    super::presets::validate(object.get("presetKey"))?;
    let mut item = item_of(object);
    let base_url = item
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let normalized = normalize_base_url(&base_url)?;
    item["baseUrl"] = Value::String(normalized);
    Ok(item)
}
