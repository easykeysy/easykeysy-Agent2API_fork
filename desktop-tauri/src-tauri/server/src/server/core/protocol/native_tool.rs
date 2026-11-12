//! （ up server） and by 。
//!
//! ── is 「」────────────────────────────────────
//! and function （**client**： to parameter，client ） not ，
//! by ** up **， is has ：
//!
//! ```json
//! Responses：{"type":"web_search"} / {"type":"tool_search"} / {"type":"file_search"}
//! Anthropic：{"type":"web_search_20250305","name":"web_search","max_uses":5}
//! ```
//!
//! no has `function` ，parameter not in client。 is 
//! Chat， no has this for —— not is issue #61 / #55 ：
//! （Responses path） or 1 **empty function**
//! （Anthropic path：`name` in 、`type` no —— up not will ， to 
//! schema is empty ），client is 「 none use 」。
//!
//! ── ： + by ─────────────────────────
//! not not ， is ****（[`carry`]： ，
//! 1 [`FIELD_NATIVE_TOOL`] ）。 when by ****：
//!
//! · and 1 （ anthropic / responses up ）→ [`restore`]
//! 。DeepSeek Anthropic `web_search_20250305` /
//! `web_search_20260209` ， is （
//! 9Router as `claudeSupportedToolTypes`）；
//! · （client after ， `type:"function"` ）
//! → 1 ： + 。 not after not 1 ：CatPaw 400、
//! Qoder / Trae 、 its to 1 not up ；
//! · （ and not ）→ + 。** not **：
//! `web_search` in Chat（ `web_search.enable` that ） and Responses 
//! not ，`web_search_20250305` is up —— 1 
//! （ up ， and ）。
//!
//! 1 [`Downgrade::describe`]：must can 「、Why」——
//! that is #55 that 1 。
//!
//! ── and ─────────────────────────────────
//! `_wb_` before （ [`super`] ）：chat （、
//! chat ） before by [`super::strip_internal_fields`] 1 ，
//! not will to up ； in ，「 this is from 1 」。

use serde_json::{json, Value};

/// （ for up ， = ）。
pub const FIELD_NATIVE_TOOL: &str = "_wb_native_tool";

/// ：chat （client chat ， 
/// `{"type":"web_search","web_search":{"enable":true}}`）。
///
/// this 1 ** not **（chat is ， can ）—— use 
/// 「 is chat」 when for ， [`strip_cross_protocol`]。
pub const ORIGIN_CHAT: &str = "chat";
/// ：Responses（`web_search` / `tool_search` etc ）。
pub const ORIGIN_RESPONSES: &str = "responses";
/// ：Anthropic（`web_search_20250305` etc server）。
pub const ORIGIN_ANTHROPIC: &str = "anthropic";

/// 1 （ use ， [`Downgrade::describe`]）。
pub struct Downgrade {
    /// （，used for and ）
    pub tools: Vec<Value>,
    /// 1 and `tool_choice` and Reason（ no has is None）
    pub choice: Option<String>,
}

impl Downgrade {
    /// 。`provider` Some when up `provider=` （ use ；
    /// in use ， not to provider id）；`reason` is this 1 
    /// （e.g.「 up as anthropic 」）。
    pub fn describe(&self, provider: Option<&str>, reason: Option<&str>) -> String {
        let mut text = String::new();
        if !self.tools.is_empty() {
            let labels: Vec<String> = self
                .tools
                .iter()
                .map(super::tool_plan::tool_kind_label)
                .collect();
            text.push_str(&format!(
                "⚠️ 剔除原生（服务端执行）工具声明 {} 条：{}",
                self.tools.len(),
                labels.join(", ")
            ));
            if let Some(reason) = reason {
                text.push_str(&format!("（{reason}）"));
            }
        }
        if let Some(choice) = &self.choice {
            if text.is_empty() {
                text.push_str(&format!("⚠️ 撤掉 tool_choice（{choice}）"));
            } else {
                text.push_str(&format!("；连带撤掉 tool_choice（{choice}）"));
            }
        }
        if let Some(provider) = provider {
            text.push_str(&format!("；provider={provider}"));
        }
        text
    }
}

/// 1 ：****，。
///
/// Responses （`tools: ["web_search"]` etc 
/// `{"type":"web_search"}`）： not ， 1 for 。
pub fn carry(tool: &Value, origin: &str) -> Value {
    let mut entry = match tool {
        Value::String(name) => json!({ "type": name }),
        other => other.clone(),
    };
    if let Some(object) = entry.as_object_mut() {
        object.insert(
            FIELD_NATIVE_TOOL.to_string(),
            Value::String(origin.to_string()),
        );
    }
    entry
}

/// （ not is then None）
pub fn origin_of(tool: &Value) -> Option<&str> {
    tool.get(FIELD_NATIVE_TOOL)
        .and_then(Value::as_str)
        .filter(|origin| !origin.trim().is_empty())
}

/// is ：**** ， or `type` in and not is `function` 。
///
/// no ：chat request is ，client 
/// chat （ that ） not will [`carry`]。
pub fn is_native(tool: &Value) -> bool {
    if origin_of(tool).is_some() {
        return true;
    }
    match tool {
        // can can is （function must to parameter， not ）
        Value::String(_) => true,
        _ => {
            let kind = super::string_field(tool, "type");
            !kind.is_empty() && !kind.eq_ignore_ascii_case("function")
        }
    }
}

/// Anthropic ：`type:"custom"` is **client** 
/// （ new spec）， not is server —— server `type` is 
/// `web_search_20250305` this （ `super::anthropic` ）。
pub fn is_native_anthropic(tool: &Value) -> bool {
    let kind = super::string_field(tool, "type");
    !kind.is_empty()
        && !kind.eq_ignore_ascii_case("function")
        && !kind.eq_ignore_ascii_case("custom")
}

/// 「 to up 」 ：， its 。
///
/// in and [`origin_of`] 1 when use —— no has can 
/// （module header「 not 」 that 1 ）。
pub fn restore(tool: &Value) -> Value {
    let mut entry = tool.clone();
    // has ： after can （`_wb_` before 1 ）
    super::strip_on_object(&mut entry);
    entry
}

/// （`name` ； no has name `type`；
/// ）。`tool_choice` use 。
pub fn name_of(tool: &Value) -> String {
    let name = super::string_field(tool, "name");
    if !name.is_empty() {
        return name;
    }
    match tool {
        Value::String(text) => text.trim().to_string(),
        other => super::string_field(other, "type"),
    }
}

/// 1 chat `tool_choice` to （`auto` / `none` / `required`
/// this not ）。
pub fn choice_target(choice: &Value) -> Option<String> {
    match choice {
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty()
                || matches!(
                    text.to_ascii_lowercase().as_str(),
                    "auto" | "none" | "required"
                )
            {
                None
            } else {
                Some(text.to_string())
            }
        }
        _ => {
            let nested = choice
                .pointer("/function/name")
                .map(super::string_value)
                .unwrap_or_default();
            if !nested.trim().is_empty() {
                return Some(nested);
            }
            let flat = super::string_field(choice, "name");
            if flat.is_empty() { None } else { Some(flat) }
        }
    }
}

/// `tool_choice` is is （chat no has 「 use 」 ）。
///
/// excluding `auto` / `none` / `required` for ： that is chat 
/// （client use ，WorkBuddy 1 has ）。
pub fn is_native_choice(choice: &Value) -> bool {
    if choice.is_string() {
        return false;
    }
    let kind = super::string_field(choice, "type").to_ascii_lowercase();
    !kind.is_empty()
        && !matches!(kind.as_str(), "function" | "auto" | "none" | "required")
}

/// `tool_choice` is and ： is then return 1 Reason。
///
/// ： to ； or in 。must ——
/// 1 to not in `tool_choice`， up will by 「 not in 」 
/// 400（CatPaw `select_tools` is this ）。
pub fn choice_conflict(choice: &Value, dropped: &[Value]) -> Option<String> {
    if is_native_choice(choice) {
        let kind = super::string_field(choice, "type");
        return Some(format!("它本身指向原生工具（type={kind}）"));
    }
    let name = choice_target(choice)?;
    dropped
        .iter()
        .any(|tool| name_of(tool) == name)
        .then(|| format!("它点名的「{name}」是本次被剔除的原生工具"))
}

/// ：****（ `type:"function"` ）。
///
/// is [`is_native`] —— （`type:"function"`， and ）
/// 1 。 this ** not can ** use ： that will client 1 and ， to 
/// 1 no has tools request， can use （2026-10-01 that 「
/// not 」 is this ）。
///
/// no has can （request）return None； has then to `tool_choice`。
pub fn downgrade(body: &Value) -> Option<(Value, Downgrade)> {
    strip_with(body, is_native)
}

/// chat ：「 ≠ 」 ****。
///
/// none 1 not —— that is chat up chat ，
/// is （ up not by use config）。
pub fn strip_cross_protocol(body: &Value, target: &str) -> Option<(Value, Downgrade)> {
    strip_with(body, |tool| match origin_of(tool) {
        Some(origin) => !origin.eq_ignore_ascii_case(target),
        None => false,
    })
}

/// and `tool_choice` （ `should_drop` ）。
fn strip_with<F: Fn(&Value) -> bool>(body: &Value, should_drop: F) -> Option<(Value, Downgrade)> {
    let mut next = body.clone();
    let mut dropped: Vec<Value> = Vec::new();
    if let Some(tools) = body.get("tools").and_then(Value::as_array) {
        let mut kept: Vec<Value> = Vec::with_capacity(tools.len());
        for tool in tools {
            if should_drop(tool) {
                dropped.push(tool.clone());
            } else {
                kept.push(tool.clone());
            }
        }
        if !dropped.is_empty() {
            if let Some(object) = next.as_object_mut() {
                if kept.is_empty() {
                    object.remove("tools");
                } else {
                    object.insert("tools".to_string(), Value::Array(kept));
                }
            }
        }
    }
    let mut choice: Option<String> = None;
    if let Some(current) = next.get("tool_choice").cloned() {
        if let Some(reason) = choice_conflict(&current, &dropped) {
            if let Some(object) = next.as_object_mut() {
                object.remove("tool_choice");
            }
            choice = Some(reason);
        }
    }
    if dropped.is_empty() && choice.is_none() {
        return None;
    }
    Some((next, Downgrade { tools: dropped, choice }))
}
