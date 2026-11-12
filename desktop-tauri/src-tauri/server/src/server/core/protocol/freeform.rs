//! Responses custom（freeform） ↔ Chat function 。
//!
//! ── Why need need this 1 ────────────────────────────────────────
//! Responses has 1 ** no has JSON parameter**：
//!
//! ```json
//! {"type":"custom","name":"exec",
//!  "format":{"type":"grammar","syntax":"lark","definition":"…"}}
//! ```
//!
//! for not is `arguments` JSON， is 1 ** by **—— in 
//! `custom_tool_call` `input` 。Codex `exec` / `apply_patch`
//! is this 。
//!
//! up is Chat Completions ：「function + JSON schema parameter」。
//! custom no has （ up not ），： not to 
//! ， is ** use **，client as 1 no ，
//! up not will ， HTTP 200、none、none —— 。
//!
//! this to ：
//! - ：custom → `input` parameter function ， 「 this is by 
//! 」 and grammar description（[`downgrade_custom_tool`]）；
//! - ： up `arguments` → Responses `input`（[`unwrap_freeform_input`]），
//! and to （[`wrap_freeform_input`]），。
//!
//! need not need `custom_tool_call` by use ， need need 「
//! is custom 」—— that is [`custom_tool_names`] 。

use serde_json::{json, Value};

use super::{json_text, string_field};

/// after by 。
///
/// Chat function must has JSON schema parameter， by no has parameter，
/// is 1 1 —— to use this for 。
pub const FREEFORM_FIELD: &str = "input";

/// description by 。
///
/// not this ， will by function JSON parameter，or use 
/// markdown —— will to not can 。
const FREEFORM_HINT: &str = "\n\n[自由文本工具] 该工具没有 JSON 参数表：唯一参数 \
`input` 是要原样交给工具执行的完整文本（源码、补丁或命令）。请直接把该文本放进 \
input 字段，不要转义成字符串字面量、不要加 markdown 代码围栏、不要附加任何解释。";

/// is is custom（freeform）
pub fn is_custom_tool(tool: &Value) -> bool {
    string_field(tool, "type").eq_ignore_ascii_case("custom")
}

/// custom → Chat function （）。
///
/// when return `None`（ no has function up not ）。
pub fn downgrade_custom_tool(tool: &Value) -> Option<Value> {
    let name = string_field(tool, "name");
    if name.is_empty() {
        return None;
    }
    let mut description = string_field(tool, "description");
    description.push_str(FREEFORM_HINT);
    if let Some(grammar) = grammar_hint(tool) {
        description.push_str(&grammar);
    }
    Some(json!({
        "type": "function",
        "function": {
            "name": name,
            "description": description,
            "parameters": {
                "type": "object",
                "properties": {
                    FREEFORM_FIELD: {
                        "type": "string",
                        "description": "该工具的完整自由文本输入，原样交给工具执行。",
                    },
                },
                "required": [FREEFORM_FIELD],
            },
        },
    }))
}

/// grammar（`format: {type:"grammar", syntax, definition}`）→ 。
///
/// grammar is by 「parameter」： not ， input 
/// not 。 description， not —— no has 。
fn grammar_hint(tool: &Value) -> Option<String> {
    let format = tool.get("format")?;
    let definition = string_field(format, "definition");
    if definition.is_empty() {
        return None;
    }
    let syntax = string_field(format, "syntax");
    let syntax = if syntax.is_empty() { "lark" } else { &syntax };
    Some(format!(
        "\n\n[输入语法] input 必须符合以下 {syntax} 文法：\n{definition}"
    ))
}

/// up `arguments` → Responses `input`（）。
///
/// is when `{"input": "…"}`。 not 1 ： can can to ，
/// can can 。 —— this etc use ，
/// 1 。
pub fn unwrap_freeform_input(arguments: &str) -> String {
    let trimmed = arguments.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    match serde_json::from_str::<Value>(trimmed) {
        Ok(Value::Object(fields)) => match fields.get(FREEFORM_FIELD) {
            Some(Value::String(text)) => text.clone(),
            Some(other) => json_text(other),
            // empty for （`{}`） is 「 this use no has 」， JSON to 
            None if fields.is_empty() => String::new(),
            // has no input： by ， not 
            None => trimmed.to_string(),
        },
        // JSON （`"…"`）： and ， 
        Ok(Value::String(text)) => text,
        // （ no JSON）： is need ， use 
        _ => trimmed.to_string(),
    }
}

/// Responses `input` → Chat `arguments` JSON（ when use ）。
///
/// and for ： `custom_tool_call` need function use 
/// in to up messages ，otherwise to before after not 1 。
pub fn wrap_freeform_input(input: &str) -> String {
    json!({ FREEFORM_FIELD: input }).to_string()
}
