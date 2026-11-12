//! Chat ** sanitize**： client down 
//! up can 。
//!
//! ── （ will ）──────────────────────
//! client will up use will ， after 
//! request —— up for ** after 1 **requestreturn 400， will 
//! 。 before already ：
//!
//! 1. `tool_calls[].function.name` as empty（ up use ，client
//! 1 "No such tool available" after in ）；
//! 2. tool and assistant `tool_calls` （Claude Desktop
//! skill when tool_result and in 1 user ，
//! `anthropic.rs` this when not for will this ）。
//!
//! ── Why in 1 ， not is in 1 ───────────
//! will to **any 1 ** up ： is chat ， by 
//! Responses / Anthropic。 1 Chat in sanitize 1 ，all、
//! all、all to 1 （ will by 1 ，
//! will add）。 up and [`super::strip_internal_fields`] is 1 
//! ： that is 「」 use ， this is 「」 that 1 。
//!
//! ── and WorkBuddy ───────────────────────────────
//! for and in `providers::workbuddy::normalize`（
//! workbuddy2api `tool_pairing.go`）。 1 then ，
//! in 1 to ； that 1 that 。
//!
//! ── ： can 1 up down ， will ──────────
//! is ****， not ：、、none for 。
//! and 1 （ its `tool_pairing.go` ）。
//!
//! ── ──────────────────────────────────────────────────
//! `mod.rs`： unwrap/expect/panic；， not IO。

use serde_json::{Map, Value};

/// use empty 。
///
/// and `responses.rs` / `anthropic.rs` ****empty when ——
/// 1 （） use 1 ， when 1 is 。
const UNKNOWN_TOOL_NAME: &str = "unknown";

/// sanitize （ use 1 ）。
///
/// and `workbuddy::normalize` `NormalizeReport` 1 to ：statistics** 
/// **。 this is （ 1 up down 、 use ）， 1 
/// use need can 「 for request」。
#[derive(Clone, Debug, Default)]
pub struct HistoryReport {
    /// empty use 
    pub renamed_tools: usize,
    /// is for （ in assistant and its to after ）
    pub repacked: bool,
    /// tool_call / tool 
    pub orphans_removed: usize,
}

impl HistoryReport {
    /// is has 
    pub fn notable(&self) -> bool {
        self.renamed_tools > 0 || self.repacked || self.orphans_removed > 0
    }

    /// 1 can in （ `logging::verbose`）
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.renamed_tools > 0 {
            parts.push(format!("空名工具调用补占位名 {} 个", self.renamed_tools));
        }
        if self.repacked {
            parts.push("tool 结果重排（配对断裂修复）".to_string());
        }
        if self.orphans_removed > 0 {
            parts.push(format!("剔除无法配对的 tool 条目 {} 个", self.orphans_removed));
        }
        parts.join("；")
    }
}

/// for Chat 1 sanitize（），return。
///
/// not can ：（ after ），（ to 
/// assistant after ）， after （ when 「 for 」 and will 1 
/// —— will in ）。
///
/// body not is for / no has messages when return（ body not in this 
/// error： up will to error）。
pub fn sanitize_history(body: &mut Value) -> HistoryReport {
    let mut report = HistoryReport::default();
    let Some(messages) = body.get_mut("messages").and_then(Value::as_array_mut) else {
        return report;
    };
    rename_empty_tool_names(messages, &mut report);
    repack_tool_results(messages, &mut report);
    cleanup_orphan_tools(messages, &mut report);
    report
}

/// 1 ： to empty use 。
///
/// Why not is 「 this use and its for 」： 1 —— for 
/// 、 not （ before ）， not will empty「none tool_calls
/// assistant」 this new 。delete then need 、 assistant empty 
/// ， 。
///
/// （`function` is for ）：`function` / for client use 
/// none from ， 1 will up to its ——
/// 、 up （ and `anthropic_outbound` for id 1 to ）。
fn rename_empty_tool_names(messages: &mut [Value], report: &mut HistoryReport) {
    for message in messages.iter_mut() {
        let Some(fields) = message.as_object_mut() else {
            continue;
        };
        if !is_role(fields, "assistant") {
            continue;
        }
        let Some(calls) = fields.get_mut("tool_calls").and_then(Value::as_array_mut) else {
            continue;
        };
        for call in calls.iter_mut() {
            let Some(function) = call.get_mut("function").and_then(Value::as_object_mut) else {
                continue;
            };
            let named = function
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .is_some_and(|name| !name.is_empty());
            if named {
                continue;
            }
            function.insert(
                "name".to_string(),
                Value::String(UNKNOWN_TOOL_NAME.to_string()),
            );
            report.renamed_tools += 1;
        }
    }
}

/// ： in `assistant.tool_calls` and its tool to after ，
/// 1 use in wire up **** and ** in assistant after **。
///
/// OpenAI need tool assistant， in for 
/// （ up 11148 / "tool calls and tool results do not match"） and will ；
/// Anthropic need `tool_result` in user before 。client not 
/// this then ：Codex image_resize_notice、Claude Desktop skill will 
/// 1 in 。
///
/// and `repackToolResultBlocks` has 1 （ its for ）：** not 
/// need 「assistant after 1 is 」**。 to 「assistant → 
/// → 」 break（ that 1 to ）， is this 1 not to ——
/// is Claude Desktop 。 this to after ：、
/// down 1 `assistant.tool_calls`、 or not tool as 。
///
/// in 「 in after 」 when ： 1 not to this is 
/// use ， not （ that to ）； for not 。
///
/// （、 has need need ）： request will ，
/// —— will as 1 （
/// in MB ） as 。
fn repack_tool_results(messages: &mut Vec<Value>, report: &mut HistoryReport) {
    if messages.len() < 3 {
        return;
    }
    let plan = plan_repacks(messages.as_slice());
    if plan.is_empty() {
        return;
    }
    let mut out: Vec<Value> = Vec::with_capacity(messages.len());
    let mut plan_index = 0usize;
    let mut index = 0usize;
    while index < messages.len() {
        if let Some(step) = plan.get(plan_index).filter(|step| step.start == index) {
            out.push(messages[step.start].clone());
            for &result in &step.results {
                out.push(messages[result].clone());
            }
            for &moved in &step.moved {
                out.push(messages[moved].clone());
            }
            index = step.end;
            plan_index += 1;
            continue;
        }
        out.push(messages[index].clone());
        index += 1;
    }
    *messages = out;
    report.repacked = true;
}

/// 1 （ down ； ）
struct Repack {
    /// （ `tool_calls` assistant） down 
    start: usize,
    /// down （ by for ）
    results: Vec<usize>,
    /// in and its down （ by for ， after ）
    moved: Vec<usize>,
    /// （ not ）： use after from this 
    end: usize,
}

/// need need （ then [`repack_tool_results`]）。
///
/// by down 、 not ： down 1 `tool_calls` assistant
/// （ down 1 ）， 1 down will 1 。
fn plan_repacks(messages: &[Value]) -> Vec<Repack> {
    let mut plan: Vec<Repack> = Vec::new();
    let mut index = 0usize;
    while index < messages.len() {
        let want = assistant_call_ids(&messages[index]);
        if want.is_empty() {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        let mut results: Vec<usize> = Vec::new();
        let mut moved: Vec<usize> = Vec::new();
        let mut needs = false;
        while index < messages.len() {
            let Some(fields) = messages[index].as_object() else {
                break;
            };
            let role = fields.get("role").and_then(Value::as_str).unwrap_or("");
            if role == "tool" {
                let id = fields
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !want.iter().any(|call| call == id) {
                    break; // 不属于本批：交还外层
                }
                results.push(index);
                if !moved.is_empty() {
                    needs = true; // 结果跟在插入物之后 = 确实需要重排
                }
                index += 1;
                if results.len() >= want.len() {
                    break; // 本批结果收齐，后面的消息与新的一轮无关
                }
                continue;
            }
            // down 1 assistant.tool_calls is new ， not can ：
            // 1 moved， that not to 。
            if is_role(fields, "assistant") && !assistant_call_ids(&messages[index]).is_empty() {
                break;
            }
            // when ， tool is 「 in in 」 ； 1 
            // not to when needs not will ，。
            moved.push(index);
            index += 1;
        }
        if needs {
            plan.push(Repack {
                start,
                results,
                moved,
                end: index,
            });
        }
    }
    plan
}

/// ：none for `tool_call` and tool （ by 1 keep for ）。
///
/// - `role:"tool"` `tool_call_id`（） and 
/// `assistant.tool_calls[].id`（ use ）；
/// - `assistant.tool_calls` by keep ： has for use ，empty then 
/// `tool_calls` ；
/// - `role:"tool"` in for use when ，delete。
///
/// **must use 1 keep **： if use （ tool_calls or 
/// delete）， will down 「none tool_calls assistant + tool」 this for ， up 
/// 400。empty id use / not up ， by 1 then 1 and —— use 
/// and WorkBuddy has （ that already in up ）。
fn cleanup_orphan_tools(messages: &mut Vec<Value>, report: &mut HistoryReport) {
    if messages.is_empty() {
        return;
    }
    let mut call_ids: Vec<String> = Vec::new();
    let mut result_ids: Vec<String> = Vec::new();
    for message in messages.iter() {
        let Some(fields) = message.as_object() else {
            continue;
        };
        match fields.get("role").and_then(Value::as_str).unwrap_or("") {
            "tool" => {
                if let Some(id) = non_empty_str(fields.get("tool_call_id")) {
                    result_ids.push(id);
                }
            }
            "assistant" => {
                if let Some(calls) = fields.get("tool_calls").and_then(Value::as_array) {
                    for call in calls {
                        if let Some(id) = call
                            .as_object()
                            .and_then(|call| non_empty_str(call.get("id")))
                        {
                            call_ids.push(id);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if call_ids.is_empty() && result_ids.is_empty() {
        return; // 无工具流量：零改动
    }
    // keep = use and id
    let keep: Vec<&String> = call_ids
        .iter()
        .filter(|id| result_ids.iter().any(|result| result == *id))
        .collect();
    // 1) use 
    for message in messages.iter_mut() {
        let Some(fields) = message.as_object_mut() else {
            continue;
        };
        if !is_role(fields, "assistant") {
            continue;
        }
        let Some(calls) = fields.get("tool_calls").and_then(Value::as_array) else {
            continue;
        };
        if calls.is_empty() {
            continue;
        }
        let kept: Vec<Value> = calls
            .iter()
            .filter(|call| {
                call.as_object()
                    .and_then(|call| non_empty_str(call.get("id")))
                    .map(|id| keep.iter().any(|kept| **kept == id))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if kept.len() == calls.len() {
            continue; // 整批齐全：零改动
        }
        report.orphans_removed += calls.len() - kept.len();
        if kept.is_empty() {
            fields.remove("tool_calls");
        } else {
            fields.insert("tool_calls".to_string(), Value::Array(kept));
        }
    }
    // 2) ： tool delete
    let before = messages.len();
    messages.retain(|message| {
        let Some(fields) = message.as_object() else {
            return true;
        };
        if fields.get("role").and_then(Value::as_str) != Some("tool") {
            return true;
        }
        non_empty_str(fields.get("tool_call_id"))
            .map(|id| keep.iter().any(|kept| **kept == id))
            .unwrap_or(false)
    });
    report.orphans_removed += before - messages.len();
}

/// 1 is is 
fn is_role(fields: &Map<String, Value>, role: &str) -> bool {
    fields.get("role").and_then(Value::as_str) == Some(role)
}

/// 1 empty（`tool_calls[].id` / `tool_call_id` ）
fn non_empty_str(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

/// assistant `tool_calls[].id` （ assistant / none use then empty vec）
fn assistant_call_ids(message: &Value) -> Vec<String> {
    message
        .as_object()
        .and_then(|fields| {
            if !is_role(fields, "assistant") {
                return None;
            }
            fields.get("tool_calls").and_then(Value::as_array)
        })
        .map(|calls| {
            calls
                .iter()
                .filter_map(|call| call.as_object().and_then(|call| non_empty_str(call.get("id"))))
                .collect()
        })
        .unwrap_or_default()
}
