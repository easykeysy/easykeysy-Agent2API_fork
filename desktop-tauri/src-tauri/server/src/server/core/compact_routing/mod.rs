//! clientrequest。currentrequest ， not 。

use std::borrow::Cow;
use serde_json::Value;

pub(crate) mod parameters;

pub const ROUTE_FIELD: &str = "_wb_compact_route";

// ：ZCode core/compact/prompt.ts。 use only in CLI ，HTTP no has use or use 。
const ZCODE_PREAMBLE: &str = "CRITICAL: Respond with TEXT ONLY. Do NOT call any tools.\n\n\
- Do NOT use Read, Bash, Grep, Glob, Edit, Write, or ANY other tool.\n\
- You already have all the context you need in the conversation above.\n\
- Tool calls will be REJECTED and will waste your only turn — you will fail the task.\n\
- Your entire response must be plain text: an <analysis> block followed by a <summary> block.\n\n";
const ZCODE_TASK: &str = "Your task is to create a detailed summary of the conversation so far, paying close attention to the user's explicit requests and your previous actions.";
const ZCODE_TRAILER: &str = "\n\nREMINDER: Do NOT call any tools. Respond with plain text only — an <analysis> block followed by a <summary> block. Tool calls will be rejected and you will fail the task.";
const ZCODE_STRUCTURE: &[&str] = &[
    "This summary should be thorough in capturing technical details, code patterns, and architectural decisions",
    "Before providing your final summary, wrap your analysis in <analysis> tags",
    "Your summary should include the following sections:",
    "1. Primary Request and Intent:",
    "6. All user messages:",
    "9. Optional Next Step:",
    "Here's an example of how your output should be structured:",
    "</summary>\n</example>",
    "Please provide your summary based on the conversation so far, following this structure",
];

fn template_matches(text: &str) -> bool {
    let normalized = if text.contains('\r') { Cow::Owned(text.replace("\r\n", "\n")) } else { Cow::Borrowed(text) };
    let Some(rest) = normalized.trim().strip_prefix(ZCODE_PREAMBLE) else { return false; };
    if !rest.starts_with(ZCODE_TASK) || !rest.ends_with(ZCODE_TRAILER) { return false; }
    // must； use Additional Instructions ， not and 。
    let mut remaining = rest;
    for marker in ZCODE_STRUCTURE {
        let Some((_, tail)) = remaining.split_once(marker) else { return false; };
        remaining = tail;
    }
    true
}

#[derive(Clone, Copy)]
pub enum RequestFormat { Chat, Responses, Anthropic }

pub fn client_for_request(client: &str, body: &Value, format: RequestFormat) -> Option<&'static str> {
    if client != "ZCode" { return None; }
    let settings = crate::server::config::compact_routes::current();
    if settings.settings.as_ref().is_ok_and(|config| !config.enabled || config.rules.is_empty()) { return None; }
    let messages = body.get(match format { RequestFormat::Responses => "input", _ => "messages" })?.as_array()?;
    let last = messages.last()?;
    if last.get("role")?.as_str()? != "user" { return None; }
    // before has 1 response； not is already will 。
    if !messages[..messages.len().saturating_sub(1)].iter().any(|message|
        message.get("role").and_then(Value::as_str) == Some("assistant")
            || matches!(format, RequestFormat::Responses) && message.get("type").and_then(Value::as_str) == Some("function_call")) { return None; }
    let content = last.get("content")?;
    let matches = if let Some(text) = content.as_str() {
        template_matches(text)
    } else {
        // must in before ：Anthropic user will and as ， Chat after will 1 。
        let part = content.as_array()?.last()?;
        let text_type = match format { RequestFormat::Responses => "input_text", _ => "text" };
        if part.get("type").and_then(Value::as_str) != Some(text_type) { return None; }
        part.get("text").and_then(Value::as_str).is_some_and(template_matches)
    };
    matches.then_some(crate::server::config::compact_routes::CLIENT_ZCODE)
}

pub fn mark_routed(body: &mut Value, source: &str, target: &str, client: &str) {
    if let Some(object) = body.as_object_mut() {
        object.insert(ROUTE_FIELD.into(), serde_json::json!({ "source": source, "target": target, "client": client }));
    }
}

pub fn take_route(body: &mut Value) -> Option<Value> {
    body.as_object_mut()?.remove(ROUTE_FIELD)
}
