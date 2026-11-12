//! ： Responses Chat can 1 。
//!
//! ── Why need need ──────────────────────────────────────────
//! Chat `tools` is 1 ， is `{type:"function", function:{…}}`；
//! Responses 「 / 」：
//! · `type:"custom"`： by ， no has JSON parameter（ `freeform` ）；
//! · `type:"namespace"`：empty， 1 in 1 down 。Codex 
//! Responses Lite path use `functions` / `clock` / `collaboration` /
//! `mcp__*` etc 。
//!
//! namespace up not ，： not to ，
//! can use （2026-09 that 「Codex not 」 ）。
//!
//! ── and ──────────────────────────────────────────────
//! by `namespace + name` —— this is Codex 
//! （ MCP empty `mcp__server__` ， is `mcp__server__status`
//! this ）。 when ` → (namespace, )` [`ToolPlan::namespaced`]，
//! use `namespace` ：Codex by by 
//! `{name, namespace}` ， `namespace` will `unsupported call`。
//!
//! when by Codex 1 then 1 （ already namespace use ，otherwise before ），
//! this 「 → 」 is etc ， not will 。
//!
//! ── defer_loading ──────────────────────────────────
//! Responses namespace `defer_loading: true`， is 「 not to 
//! ， etc use `tool_search` to 」。 up Chat no has tool_search，
//! is this **ignore defer_loading、 down **： can 1 up down ，
//! not to is unavailable —— its 。

use serde_json::{json, Map, Value};

use super::{freeform, is_truthy, string_field, string_value};
use crate::server::logging;

/// `additional_tools` type 。
///
/// Codex Responses Lite path use ： `tools` null，
/// in `input[]` this 1 （ OpenAI 「Add tools at a specific
/// point in the input」）。 up is Chat ， no has 「 for in 」
/// ， 1 request。
const ADDITIONAL_TOOLS_ITEM: &str = "additional_tools";

/// empty type 
const NAMESPACE_TOOL: &str = "namespace";

/// 1 request after 。
#[derive(Default)]
pub struct ToolPlan {
    /// can to `tool_to_chat` （namespace already before ）
    pub declarations: Vec<Value>,
    /// → (namespace, )： `namespace` use 
    pub namespaced: std::collections::BTreeMap<String, (String, String)>,
    /// is custom（ by ） ****： item 
    pub custom_names: std::collections::BTreeSet<String>,
}

impl ToolPlan {
    /// this is is custom 
    pub fn is_custom(&self, name: &str) -> bool {
        self.custom_names.contains(name)
    }
}

/// request and 。
///
/// and ： `tools`（）、`input[]` `additional_tools`
/// （Codex Lite ）， and namespace 。 in before ，
/// is request， up in 。
pub fn plan_tools(request: &Value) -> ToolPlan {
    let mut plan = ToolPlan::default();
    if let Some(tools) = request.get("tools").and_then(Value::as_array) {
        for tool in tools {
            push_tool(&mut plan, tool);
        }
    }
    let Some(items) = request.get("input").and_then(Value::as_array) else {
        return plan;
    };
    for item in items {
        if !string_field(item, "type").eq_ignore_ascii_case(ADDITIONAL_TOOLS_ITEM) {
            continue;
        }
        // is `tools: []`。 Lite path in not Codex 
        // to （`tool_names: [...]`） —— that no has can 
        // ， can ， 1 schema will by errorparameter use 
        if let Some(tools) = item.get("tools").and_then(Value::as_array) {
            for tool in tools {
                push_tool(&mut plan, tool);
            }
        } else if let Some(names) = item.get("tool_names").and_then(Value::as_array) {
            let listed: Vec<String> = names.iter().map(|name| string_value(name)).collect();
            logging::log(
                "[Responses]",
                &format!(
                    "⚠️ additional_tools 只给了工具名、没有定义，无法转发：{}",
                    listed.join(", ")
                ),
            );
        }
    }
    plan
}

/// ：namespace ， its down 。
fn push_tool(plan: &mut ToolPlan, tool: &Value) {
    if string_field(tool, "type").eq_ignore_ascii_case(NAMESPACE_TOOL) {
        expand_namespace(plan, tool);
        return;
    }
    record_custom(plan, tool);
    plan.declarations.push(tool.clone());
}

/// namespace → 1 ，empty before 。
fn expand_namespace(plan: &mut ToolPlan, namespace: &Value) {
    let ns_name = string_field(namespace, "name");
    let ns_desc = string_field(namespace, "description");
    let Some(children) = namespace.get("tools").and_then(Value::as_array) else {
        // no has namespace is empty（Codex use ）， no can 
        return;
    };
    for child in children {
        // namespace： no this ， not ，
        if string_field(child, "type").eq_ignore_ascii_case(NAMESPACE_TOOL) {
            logging::log(
                "[Responses]",
                &format!("⚠️ 嵌套 namespace 无法展开，已跳过：{}", string_field(child, "name")),
            );
            continue;
        }
        let child_name = string_field(child, "name");
        if child_name.is_empty() {
            continue;
        }
        // module header（ and Codex 1 then 1 ）
        let flat = if ns_name.is_empty() {
            child_name.clone()
        } else {
            format!("{ns_name}{child_name}")
        };
        let mut decl = child.clone();
        if let Some(fields) = decl.as_object_mut() {
            fields.insert("name".to_string(), Value::String(flat.clone()));
            // empty ： to ， not to 
            // none from this is 
            if !ns_desc.is_empty() {
                append_description(fields, &format!("\n\n[所属分组 `{ns_name}`] {ns_desc}"));
            }
            // defer_loading up no has for ， unknown
            fields.remove("defer_loading");
        }
        record_custom(plan, &decl);
        if !ns_name.is_empty() {
            plan.namespaced
                .insert(flat, (ns_name.clone(), child_name));
        }
        plan.declarations.push(decl);
    }
}

/// down this is not is custom（ by ）
fn record_custom(plan: &mut ToolPlan, tool: &Value) {
    if !freeform::is_custom_tool(tool) {
        return;
    }
    let name = string_field(tool, "name");
    if !name.is_empty() {
        plan.custom_names.insert(name);
    }
}

/// description 1 （ no has description new ）
fn append_description(fields: &mut Map<String, Value>, extra: &str) {
    let existing = string_field(&Value::Object(fields.clone()), "description");
    let merged = format!("{existing}{extra}");
    fields.insert("description".to_string(), Value::String(merged));
}

/// use 「」。
///
/// client ，`function_call` `name` is ****、empty in 
/// `namespace` ； to up Chat must is after ，otherwise to 
/// before after not 1 。 1 then and Codex 1 ： already before use （ etc ）。
pub fn flatten_call_name(item: &Value) -> String {
    let name = string_field(item, "name");
    let namespace = string_field(item, "namespace");
    if namespace.is_empty() || name.is_empty() || name.starts_with(&namespace) {
        return name;
    }
    format!("{namespace}{name}")
}

/// use → `namespace` Responses item。
///
/// `plan` has this when up `namespace`：Codex by 
/// `{name, namespace}` ， `unsupported call`。
pub fn restore_namespace(mut item: Value, flat_name: &str, plan: &ToolPlan) -> Value {
    let Some((namespace, original)) = plan.namespaced.get(flat_name) else {
        return item;
    };
    if let Some(fields) = item.as_object_mut() {
        fields.insert("namespace".to_string(), Value::String(namespace.clone()));
        fields.insert("name".to_string(), Value::String(original.clone()));
    }
    item
}

/// in can （`function:bash` / `web_search` / …）。
///
/// when must down ：「 N 」， none from is not is 
/// can —— 2026-09 that Codex ， is use ，
/// if when has this ， 1 can 。
pub fn tool_kind_label(tool: &Value) -> String {
    // （`tools: ["web_search"]`）： is 
    if let Some(text) = tool.as_str() {
        let text = text.trim();
        return if text.is_empty() {
            "未命名工具".to_string()
        } else {
            text.to_string()
        };
    }
    let kind = string_field(tool, "type");
    // can ： `name`， and Chat 
    // （`{"type":"function","function":{"name":…}}`） `function.name`。
    // ， OpenAI in 「function」，
    // etc no —— 2026-10-01 「 」 when this 。
    let name = {
        let flat = string_field(tool, "name");
        if !flat.is_empty() {
            flat
        } else {
            tool.get("function")
                .map(|function| string_field(function, "name"))
                .unwrap_or_default()
        }
    };
    if name.is_empty() {
        if kind.is_empty() { "未命名工具".to_string() } else { kind }
    } else if kind.is_empty() || kind.eq_ignore_ascii_case("function") {
        name
    } else {
        format!("{kind}:{name}")
    }
}

/// is is 「 already 」Chat （client use when ）
pub fn is_nested_function(tool: &Value) -> bool {
    string_field(tool, "type").eq_ignore_ascii_case("function") && tool.get("function").is_some()
}

/// function → Chat （`{type, name, parameters}` →
/// `{type:"function", function:{name, parameters}}`）。 when return `None`。
pub fn nested_from_flat(tool: &Value) -> Option<Value> {
    let name = string_field(tool, "name");
    if name.is_empty() {
        return None;
    }
    let parameters = tool
        .get("parameters")
        .filter(|value| is_truthy(value))
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object", "properties": {} }));
    let mut function = Map::new();
    function.insert("name".to_string(), Value::String(name));
    if let Some(description) = tool.get("description").filter(|value| is_truthy(value)) {
        function.insert("description".to_string(), description.clone());
    }
    function.insert("parameters".to_string(), parameters);
    // strict is Responses ，Chat function （ up supports）
    if let Some(strict) = tool.get("strict") {
        function.insert("strict".to_string(), strict.clone());
    }
    Some(json!({ "type": "function", "function": Value::Object(function) }))
}
