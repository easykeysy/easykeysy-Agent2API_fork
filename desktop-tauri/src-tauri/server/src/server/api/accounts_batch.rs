//! ；config use apply_patch。

use axum::body::Bytes;
use axum::response::Response;
use serde_json::{json, Map, Value};

use super::accounts::{parse_json_body, store_error};
use crate::server::errors::management_error;
use crate::server::http::ok_json;
use crate::server::ServerState;

pub async fn batch_accounts(state: &ServerState, body: &Bytes) -> Response {
    let payload = match parse_json_body(&body, "请求内容不是有效 JSON") {
        Ok(value) => value,
        Err(response) => return response,
    };
    // ids + empty， Node 
    let mut ids: Vec<String> = Vec::new();
    if let Some(items) = payload.get("ids").and_then(Value::as_array) {
        for item in items {
            if let Some(text) = item.as_str() {
                if !text.is_empty() && !ids.iter().any(|existing| existing == text) {
                    ids.push(text.to_string());
                }
            }
        }
    }
    if ids.is_empty() {
        return management_error(400, "缺少要操作的账号 id");
    }
    let action = payload
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    let result = match action.as_str() {
        "remove" => state.store().batch_remove(&ids),
        "enable" | "disable" => {
            let enabled = action == "enable";
            state.store().batch_update(&ids, &json!({ "enabled": enabled }))
        }
        "proxy" => {
            // proxy=null（ as ）； is 
            let Some(target) = payload.get("proxy") else {
                return management_error(400, "批量修改代理时缺少 proxy 字段");
            };
            state.store().batch_update(&ids, &json!({ "proxy": target }))
        }
        "concurrency" => {
            // and limit（0 = not ）； is ， by store 1 
            // （、 999， and set 1 ）
            let Some(target) = payload.get("maxConcurrent") else {
                return management_error(400, "批量设置并发上限时缺少 maxConcurrent 字段");
            };
            state
                .store()
                .batch_update(&ids, &json!({ "maxConcurrent": target }))
        }
        "usageQuery" => {
            let Some(target) = payload.get("usageQuery") else {
                return management_error(400, "批量查询设置缺少 usageQuery 字段");
            };
            state.store().batch_update(&ids, &json!({ "usageQuery": target }))
        }
        "limiters" => {
            let Some(mode) = payload.get("limiterMode") else {
                return management_error(400, "批量限制器操作缺少 limiterMode 字段");
            };
            let mut patch = Map::new();
            patch.insert("limiterMode".to_string(), mode.clone());
            if let Some(groups) = payload.get("limiterGroups") {
                patch.insert("limiterGroups".to_string(), groups.clone());
            }
            if let Some(rules) = payload.get("limiters") {
                patch.insert("limiters".to_string(), rules.clone());
            }
            state.store().batch_update(&ids, &Value::Object(patch))
        }
        other => {
            return management_error(
                400,
                format!(
                    "不支持的批量操作: {}",
                    if other.is_empty() { "(空)" } else { other }
                ),
            )
        }
    };

    match result {
        Ok(value) => {
            // response up action， and Node `{ action, ...result, list }` 1 
            let mut merged = Map::new();
            merged.insert("action".to_string(), Value::String(action));
            if let Some(object) = value.as_object() {
                for (key, item) in object {
                    merged.insert(key.clone(), item.clone());
                }
            }
            ok_json(Value::Object(merged))
        }
        Err(error) => store_error(error),
    }
}
