//! ** **： up this 1 to 1 ， 。
//!
//! ── Why need need ────────────────────────────────────────────
//! up turn will ** in **： use （`finish_reason: length` 1 ）、 
//! 、or in 、 1 。client to is 
//! ****， no 「 this 」 and 「 」—— before 
//! is 1 no has ， use as 。
//!
//! and （ Trae2api-cn `src/continuation.py`） 1 ：
//! 「 this 1 」， ** already that ** assistant to up ，
//! 1 「」， 1 —— new ** in 1 clientresponse after **，
//! not is client to 。
//!
//! ── ：， by for ────────────────────
//! [`looks_truncated`] is ** 1 ** 1 ，
//! （ chat ， [`TurnAccumulator`] from down 1 
//! ）。「」 is this ： 1 1 ，
//! will 「、 not 」 this not 。
//! 1. ** not **（， in length before ）： use in client，
//! tool_calls 1 etc 1 use ；
//! 2. `finish_reason` in length （8 ， [`LENGTH_FINISH_ALIASES`]）；
//! 3. as empty** has **： in ， this 1 （ and empty → to 
//! has empty， not ）；
//! 4. ``` as ****（， is ）；
//! 5. 100 （ to 「」 +2 ， for ）；
//! 6. 100 rstrip after in （`xxx,` / `xxx|` / `xxx...` / `xxx:` etc 
//! 「 to 1 」 ）， or 「 + and not 」。
//!
//! ── （）────────────────────────────────────────
//! - limit：[`config::auto_continue_max_rounds`]（default 3）；
//! - ****： new and up 1 0.5 ——
//! this is 「 in 1 1 」 （）。
//!
//! ── （must）────────────────────────────────────────
//! - ** = up **： 1 prompt_tokens before ，
//! is ** and **（ not is ）。 and [`sum_tokens`]， when by 
//! 1 `report_usage` up （ and to total）。
//! client** to ** usage ： when ， not ， not total。
//! ** before ** to usage when ；** after **（ and ） that 
//! and 1 —— when ， not when 。OpenAI 
//! （`stream_options.include_usage`） is usage in finish ** after **，
//! up that ，client can to ** after 1 ** usage —— already not ，
//! not ： 1 up no usage， not 。
//! - ** 1 for will 1 **： is 1 （ 1 
//! `UpstreamService::forward`），** will new 1 ** —— can can to 1 
//! 、 1 （ need this Key ）。 for need 
//! ， this is ： this etc 「 1 clientrequest in 
//! up up 」。
//! - will after 1 ； is 「 after 1 as 」—— to when ，
//! request that 1 to after that 1 （ and 1 ）。
//! - path ****（ null `finish_reason` that 1 and `[DONE]`） not ，
//! to 「」 or 「 down 1 」。 is ** when ** down ，
//! has this 1 to EOF（ to ）， not is 。
//!
//! ── not client parameter（including max_tokens）─────────────────────
//! is **request + **，`max_tokens` / `temperature` /
//! `tools` etc 1 。 up as 「 to 」 when ， is 1 
//! （ that is clientparameter，client down 1 to limit not is ），
//! and 「 」 is 。 need 、。
//!
//! ── default ─────────────────────────────────────────────────
//! is `autoContinue`（ `config::auto_continue`）， when use ** not **，
//! down and before 1 。 by need ： will 1 up quota，
//! and is 「client to 」 this can as ， not is 。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap / expect / panic。

use std::io;
use std::sync::Arc;

use axum::http::HeaderMap;
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::Stream;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::server::config;
use crate::server::core::key_scope::KeyScope;
use crate::server::core::upstream::{ForwardOutcome, ForwardRequest, UpstreamService, usage};
use crate::server::core::upstream::refusal;
use crate::server::logging;

/// ： up 「 use 」 （ 
/// `_LENGTH_FINISH_REASONS` for ）。 1 etc that 1 ，
/// can no ， not need 。
const LENGTH_FINISH_ALIASES: &[&str] = &[
    "length",
    "max_tokens",
    "max_output_tokens",
    "max_token",
    "token_limit",
    "model_context_window_exceeded",
    "context_window_exceeded",
    "context_length_exceeded",
];

/// ****（rstrip after ）。
///
/// this is 8 then ， can etc ， not then ：
/// - `,$`、`[|｜]$`、`\\$`、`//$`、`#$`、`[-–—]$`、`[:：]$` is 「 」；
/// - `//` is 「 `/` 」 （ when is ）， and ；
/// - `\.\.\.$`（） by `ends_with("...")` —— ** not **，
/// `.` is ， this is and 1 by 。
const DANGLING_ENDING_CHARS: &[char] = &[
    ',', '|', '｜', '\\', '/', '#', '-', '–', '—', ':', '：',
];

/// （ `_SENTENCE_END_RE` for ）。
///
/// use has 1 ：「 + 」 that ，** not ** this when 
/// as no 。 （`”` `’` `」`） is has ： in 
/// ， will 「」。
const SENTENCE_ENDING_CHARS: &[char] = &[
    '.', '!', '?', '。', '！', '？', ']', ')', '}', '`', '"', '\'', '”', '’', '』', '」',
];

/// for （ `body[-100:]`）。
const TAIL_SCAN_CHARS: usize = 100;

/// 「」 limit（ `_FRAGMENT_TEXT_LIMIT`）。
const FRAGMENT_TEXT_MAX_CHARS: usize = 40;

/// 「」 down （ `_FRAGMENT_REASONING_MIN`）。
const FRAGMENT_REASONING_MIN_CHARS: usize = 200;

/// and （ `_FRAGMENT_REASONING_RATIO`： 4 ）。
///
/// by ， this 1 **current not **：limit 40 ⇒ 
/// 160， down already is 200。 not is ， is is this 
/// 1 （ 1 will new ）； need ， 40 and 200
/// 1 ， this 1 。
const FRAGMENT_REASONING_RATIO: usize = 4;

/// （ `_DEFAULT_SIMILARITY_THRESHOLD`）。
///
/// is ****： 0.5 when 1 （ and 1 ）， as 
/// 0.5 is 「 1 not 」 not is 。
const SIMILARITY_STOP: f64 = 0.5;

/// 1 「 after 」（ `stream()` held ）。
///
/// this in by is （ has up in `finish_reason` after usage ，
/// if will usage in finish before —— that is ）。
/// ** not can by up **： after 1 up （ or ） will 
/// none。1 MiB is ：（ 1 usage、） 1/1000 use not to ，
/// up already not is 「」 is 「」。
const MAX_HELD_BYTES: usize = 1024 * 1024;

/// （ `CONTINUE_INSTRUCTION` ）。
///
/// this will up request， is up （「 not need already 」 is 
/// 1 ， 1 is [`SIMILARITY_STOP`]）。
pub const CONTINUE_INSTRUCTION: &str =
    "请继续输出，从你中断的地方继续，不要重复已经写过的内容，不要重新开始。";

/// 「 has 、 as empty」 when use （ `REASONING_ONLY_INSTRUCTION`）。
///
/// ：， and this is use 。
pub const REASONING_ONLY_INSTRUCTION: &str = "Your previous response contained only \
    thinking/reasoning with no actual output. Provide the actual response now - either \
    text content or a tool call.";

/// 1 up ****： 。
///
/// （ chat 、 down ） this 1 ， to 
/// [`looks_truncated`] —— when ， when use 1 。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TurnParts {
    /// can （`message.content` / `delta.content` ）
    pub text: String,
    /// （`reasoning_content` and `reasoning` ， 1 ）
    pub reasoning: String,
    /// use 
    pub has_tool_calls: bool,
    /// `finish_reason`（None = up no to ，e.g. officeace that ）
    pub finish: Option<String>,
}

/// `finish_reason` is 「 use 」 1 （ not ， and 
/// `refusal::is_refusal` 1 ）。
pub fn is_length_finish(finish: &str) -> bool {
    LENGTH_FINISH_ALIASES
        .iter()
        .any(|alias| alias.eq_ignore_ascii_case(finish))
}

/// this 1 not ** to 1 **（module header， and for ）。
pub fn looks_truncated(parts: &TurnParts) -> bool {
    // 1. not （ in length before ， by module header 1 ）
    if parts.has_tool_calls {
        return false;
    }
    // 2. up 「 use 」
    if parts
        .finish
        .as_deref()
        .is_some_and(|finish| is_length_finish(finish))
    {
        return true;
    }
    let body = parts.text.as_str();
    if body.trim().is_empty() {
        // 3. as empty： has 1 ；empty to has empty
        return !parts.reasoning.trim().is_empty();
    }
    // 4. （）
    if unclosed_code_fence(body) {
        return true;
    }
    let tail = last_chars(body, TAIL_SCAN_CHARS);
    // 5. 
    if unbalanced_brackets(&tail) {
        return true;
    }
    // 6a. 
    if dangling_tail(&tail) {
        return true;
    }
    // 6b. + and not 
    stopped_after_thinking(body, &parts.reasoning)
}

/// has 、 no has can （ 1 use ，）。
pub fn looks_reasoning_only(parts: &TurnParts) -> bool {
    !parts.has_tool_calls && parts.text.trim().is_empty() && !parts.reasoning.trim().is_empty()
}

/// ****（ `similarity` for ）： ，
/// 1 up 。
///
/// not is 、 not is before ： this is as and for 「 1 
/// 」， this is 。 etc ，「」。
pub fn similarity(left: &str, right: &str) -> f64 {
    let overlap = std::cmp::min(left.chars().count(), right.chars().count());
    if overlap == 0 {
        return 0.0;
    }
    let same = left
        .chars()
        .zip(right.chars())
        .filter(|(a, b)| a == b)
        .count();
    same as f64 / overlap as f64
}

/// this 1 is not is up 1 ****（， [`SIMILARITY_STOP`]）。
pub fn is_repeat(previous: &str, next: &str) -> bool {
    similarity(previous, next) > SIMILARITY_STOP
}

/// request「 this 1 」 ： already assistant ，
/// 1 user （ `continued_messages` ）。
///
/// return false = this body no has `messages` （ not ， use not ）。
/// use is ****（ not trim）： use trim after ， up need 
/// is up that ，otherwise「 from in 」 for not up 。
pub fn append_continuation(body: &mut Value, partial: &str, reasoning_only: bool) -> bool {
    let Some(messages) = body.get_mut("messages").and_then(Value::as_array_mut) else {
        return false;
    };
    if !partial.trim().is_empty() {
        messages.push(json!({"role": "assistant", "content": partial}));
    }
    let instruction = if reasoning_only {
        REASONING_ONLY_INSTRUCTION
    } else {
        CONTINUE_INSTRUCTION
    };
    messages.push(json!({"role": "user", "content": instruction}));
    true
}

/// from chat （`choices[0].message` that 1 ）。
///
/// and `aggregate::CompletionAccumulator` 1 ：`content` is 
/// （ up to null when by empty）、`reasoning_content` can can has 、`tool_calls` can can has 、
/// `finish_reason` in （ when "stop"）。
pub fn parts_of_completion(body: &Value) -> Option<TurnParts> {
    let choice = body.get("choices").and_then(Value::as_array)?.first()?;
    let message = choice.get("message")?;
    let mut parts = TurnParts {
        text: message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        reasoning: message
            .get("reasoning_content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        has_tool_calls: message
            .get("tool_calls")
            .and_then(Value::as_array)
            .is_some_and(|calls| !calls.is_empty()),
        finish: choice
            .get("finish_reason")
            .and_then(Value::as_str)
            .filter(|reason| !reason.is_empty())
            .map(str::to_string),
    };
    // in `reasoning`（ ， aggregate.rs 
    // reasoning ）； in before when after ， 1 。
    if parts.reasoning.is_empty() {
        parts.reasoning = message
            .get("reasoning")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
    }
    Some(parts)
}

/// ** can ** 1 （ use `join("")`： up no has ，
/// in empty or will markdown and ）。
pub fn join_text(rounds: &[String]) -> String {
    rounds.concat()
}

/// 「 to before as 、client will to 」。
///
/// ── Why not is that 1 （ ）────────────
/// （ ```、、） is 「**** no has 」，
/// is **** 1 ： up 1 that ``` is this 1 
/// 。 by that will 「 already 」「」——
/// 2026-10-14 up to this ：fence that ，client to is 
/// `…return 42\n```\n up return 42。` ** after 1 1 **。
/// from not is 「」， is in up 。
///
/// ── Why `finish` and `has_tool_calls` ──────────────
/// this is ** **：`finish_reason: length` is this 1 ，
/// use is this 1 （ before 1 no has not this 1 no has —— or will 
/// current this 1 ，must 1 not ）。
fn verdict_of(
    accumulated_text: &str,
    accumulated_reasoning: &str,
    any_tool_calls: bool,
    current: &TurnParts,
) -> TurnParts {
    TurnParts {
        text: accumulated_text.to_string(),
        reasoning: accumulated_reasoning.to_string(),
        has_tool_calls: any_tool_calls || current.has_tool_calls,
        finish: current.finish.clone(),
    }
}

/// usage and （module header「」）。
///
/// token and ** and **： 1 is 1 up ，
/// N prompt before N-1 （ up is this ），
/// `cache_read` by （ in 1 ），`credit` is 。
/// and after ，request and to is 「 this clientrequest」。
///
/// `credit` is `Option`： need has 1 up （ no up by 0 and and ），
/// no up None（= this 1 not this ）。
///
/// and `RequestTelemetry::report_usage` 「 after 1 as 」** not **： that is 
/// 1 up （ after to ）， this is **** ——
/// by in when 1 `report_usage`， and after 1 up up 。
///
/// and [`usage::UsageTokens`]（ not is usage JSON ） is has ：
/// and 「 1 」 has 1 （`usage::extract_usage`）， this 1 etc 
/// 。
pub fn sum_tokens(rounds: &[usage::UsageTokens]) -> usage::UsageTokens {
    rounds.iter().fold(usage::UsageTokens::default(), |mut acc, round| {
        acc.prompt += round.prompt;
        acc.completion += round.completion;
        acc.total += round.total;
        acc.cache_read += round.cache_read;
        acc.credit = match (acc.credit, round.credit) {
            (None, None) => None,
            (left, right) => Some(left.unwrap_or(0.0) + right.unwrap_or(0.0)),
        };
        acc
    })
}

/// and after token → 1 OpenAI usage for 。
///
/// use ： to `report_usage`（ this ）， and stitched
/// `usage` 。 after **client to **（ up can can to ），
/// this is and after 1 ： no has 1 usage can total。
/// in （≥2 ） when this ，response 1 not 。
pub fn usage_json(tokens: &usage::UsageTokens) -> Value {
    let mut value = json!({
        "prompt_tokens": tokens.prompt,
        "completion_tokens": tokens.completion,
        "total_tokens": tokens.total,
        "prompt_tokens_details": { "cached_tokens": tokens.cache_read },
    });
    // in has when ： `"credit": null` will down as this 1 
    // 「 1 no has 」， and null in `extract_usage` etc 。
    if let (Some(credit), Some(object)) = (tokens.credit, value.as_object_mut()) {
        object.insert("credit".to_string(), json!(credit));
    }
    value
}

/// from 1 chat token （ / for by 0， and 1 ）。
pub fn tokens_of_completion(body: &Value) -> usage::UsageTokens {
    body.get("usage")
        .and_then(usage::extract_usage)
        .unwrap_or_default()
}

/// ``` as （ `_unclosed_code_fence`）。
fn unclosed_code_fence(text: &str) -> bool {
    text.matches("```").count() % 2 == 1
}

/// `n` ****（ not is ： in by will ）。
fn last_chars(text: &str, n: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let start = chars.len().saturating_sub(n);
    chars[start..].iter().collect()
}

/// is （ `_unbalanced_brackets`， need ）。
///
/// by ****（opens > closes + 2）： to and ；
/// ** for **： 1 （`{a: [1, 2}` this ） is 。
///
/// ── and ** not ** 1 （ will ）────────────────
/// to 「 for not up 」 or 「empty when 」return
/// truncated。 that is docstring ：「 is 
/// none 」， when 「 」。 is 
/// ** 100 ** this —— 1 ， can can in 
/// `{` and `}` ， is that `}` in 「 」，
/// 1 、 1 quota and after 1 。
/// （client to ）（ to ）， this 
/// **「」**： for not up by ， not 、 not 。
/// that `{a: [1, 2}` to （`}` for not up `[` ，
/// to when `{` and `[` ）。
fn unbalanced_brackets(tail: &str) -> bool {
    let opens = tail.chars().filter(|c| matches!(c, '[' | '{' | '(')).count();
    let closes = tail.chars().filter(|c| matches!(c, ']' | '}' | ')')).count();
    if opens > closes + 2 {
        return true;
    }
    let mut stack: Vec<char> = Vec::new();
    for c in tail.chars() {
        match c {
            '[' | '{' | '(' => stack.push(c),
            ')' | ']' | '}' => {
                // for success； for not up （empty） by ， by 
                if matching_opener(c).is_some_and(|opener| stack.last() == Some(&opener)) {
                    stack.pop();
                }
            }
            _ => {}
        }
    }
    !stack.is_empty()
}

/// for （ not is when None）。
fn matching_opener(closer: char) -> Option<char> {
    match closer {
        ')' => Some('('),
        ']' => Some('['),
        '}' => Some('{'),
        _ => None,
    }
}

/// rstrip after is in （ [`DANGLING_ENDING_CHARS`]）。
fn dangling_tail(tail: &str) -> bool {
    let trimmed = tail.trim_end();
    if trimmed.ends_with("...") {
        return true;
    }
    trimmed
        .chars()
        .last()
        .is_some_and(|c| DANGLING_ENDING_CHARS.contains(&c))
}

/// 「 + and no 」（ `_stopped_after_thinking`）。
///
/// ： in ， 1 。 that 
/// ， use not 「 its no 」。 1 。
/// by [`is_repeat`] （）。
fn stopped_after_thinking(text: &str, reasoning: &str) -> bool {
    let visible = text.trim();
    let thinking = reasoning.trim();
    let visible_chars = visible.chars().count();
    if visible_chars == 0 || visible_chars > FRAGMENT_TEXT_MAX_CHARS {
        return false;
    }
    let thinking_chars = thinking.chars().count();
    if thinking_chars < FRAGMENT_REASONING_MIN_CHARS {
        return false;
    }
    if thinking_chars < visible_chars * FRAGMENT_REASONING_RATIO {
        return false;
    }
    !ends_with_sentence_mark(visible)
}

/// is （`visible` already trim， can ， not empty）。
fn ends_with_sentence_mark(visible: &str) -> bool {
    visible
        .chars()
        .last()
        .is_some_and(|c| SENTENCE_ENDING_CHARS.contains(&c))
}

/// path ： from ** down ** [`TurnParts`]。
///
/// ── Why down not is up ────────────────────────
/// already `ReasoningCoalescer`： 、 already 
/// （`sse.rs` strip ）。 by up 1 will has after ：
/// （ 6a will `\n` ）、。
///
/// ── Why not O(1) ────────────────────────────
/// 4 need **** 、 need **** ， etc 
/// as this 1 —— to 「」 。 is 「 1 
/// 」， and `RecordingStream` already in down ， not is add 。
#[derive(Clone, Debug, Default)]
pub struct TurnAccumulator {
    parts: TurnParts,
    /// is already to （ null `finish_reason` that 1 ）
    saw_finish_frame: bool,
    /// is already to `[DONE]`
    saw_done: bool,
    /// is error（`{"error": …}`）
    saw_error: bool,
    /// to after 1 usage（ up can can in ， after 1 = this ，
    /// and `RequestTelemetry::report_usage` 1 ； by [`sum_tokens`] and ）
    usage: Option<Value>,
}

impl TurnAccumulator {
    /// 1 （`ForwardStream` to down that 1 ： already `data: …\n\n` ）。
    ///
    /// return this 1 ** is not is **（ / `[DONE]` / error）—— use 
    /// 「 not 」 is 「 down 」。failure by （ not to 
    /// no ， and `refusal` fail-open 1 to ）。
    pub fn push_frame(&mut self, frame: &Bytes) -> bool {
        let text = String::from_utf8_lossy(frame);
        // ：`[DONE]`（ and refusal use 1 ， frame_is_done ）
        if frame_is_done(&text) {
            self.saw_done = true;
            return true;
        }
        let Some(value) = first_json_payload(&text) else {
            return false;
        };
        // usage 1 （， not and ）；usage in `sse.rs`
        if let Some(usage_object) = value.get("usage").filter(|usage| usage.is_object()) {
            self.usage = Some(usage_object.clone());
        }
        if value.get("error").is_some() {
            self.saw_error = true;
            return true;
        }
        let mut terminal = false;
        // usage-only （choices or as empty） not and not ：usage in 
        // `sse.rs` （`telemetry.report_usage`）， not 
        let Some(choices) = value.get("choices").and_then(Value::as_array) else {
            return false;
        };
        for choice in choices {
            if let Some(reason) = choice
                .get("finish_reason")
                .and_then(Value::as_str)
                .filter(|reason| !reason.is_empty())
            {
                self.parts.finish = Some(reason.to_string());
                self.saw_finish_frame = true;
                terminal = true;
            }
            let Some(delta) = choice.get("delta") else {
                continue;
            };
            if let Some(content) = delta.get("content").and_then(Value::as_str) {
                self.parts.text.push_str(content);
            }
            for key in ["reasoning_content", "reasoning"] {
                if let Some(thinking) = delta.get(key).and_then(Value::as_str) {
                    self.parts.reasoning.push_str(thinking);
                }
            }
            if delta
                .get("tool_calls")
                .and_then(Value::as_array)
                .is_some_and(|calls| !calls.is_empty())
            {
                self.parts.has_tool_calls = true;
            }
        }
        terminal
    }

    /// 。
    pub fn parts(&self) -> &TurnParts {
        &self.parts
    }

    /// is already to （`[DONE]` or error）—— 「」 1 。
    pub fn saw_done(&self) -> bool {
        self.saw_done
    }

    /// is error。
    pub fn saw_error(&self) -> bool {
        self.saw_error
    }

    /// token （ no has usage when 0 —— and 1 ： can not not ）。
    pub fn tokens(&self) -> usage::UsageTokens {
        self.usage
            .as_ref()
            .and_then(usage::extract_usage)
            .unwrap_or_default()
    }
}

/// this 1 is not is `[DONE]` 。
///
/// ** use `refusal::frame_payloads`**：「 finish_reason」「」
/// and 「」 is 1 path，must 1 —— otherwise will in 
/// 1 1 、 1 （ that will as 「 1 」）。
fn frame_is_done(text: &str) -> bool {
    text.lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(str::trim)
        .any(|payload| payload == "[DONE]")
}

/// 1 1 can JSON for `data:` 。
fn first_json_payload(text: &str) -> Option<Value> {
    refusal::frame_payloads(text).find_map(|payload| serde_json::from_str::<Value>(payload).ok())
}

/// down that （ and `ForwardOutcome::Stream` 1 ）。
type Downstream = Box<dyn Stream<Item = Result<Bytes, io::Error>> + Send + Unpin>;

/// 1 clientrequest need use **** up down 。
///
/// 1 is owned： is 「client that in when 1 up 」，
/// await and spawn not use （ `ProviderContext` is use ，
/// that is ** 1 ** ， is not can use that ）。
pub struct Continuation {
    /// （`UpstreamService` is Clone ， Arc by and ）
    pub service: UpstreamService,
    /// client chat （`messages` in ）； in up 
    pub base_body: Value,
    /// client this 1 need is is （ use 1 ）
    pub stream: bool,
    pub client_headers: HeaderMap,
    pub telemetry: Arc<usage::RequestTelemetry>,
    pub allowed_providers: Option<KeyScope>,
    /// totallimit（****）：`2` = 1 
    pub max_rounds: usize,
}

/// ，「 this request need not need 」。
///
/// `None` when use ** not ** —— down and before 1 。 this not is 
/// 「 etc ， 1 none」： 1 need 1 channel and 1 JSON
/// ， need ， 0 。
///
/// ** in this （+1）， in use **：config 
/// `autoContinueMaxRounds` is ****（`AUTO_CONTINUE_MIN_ROUNDS` is 
/// 「 1 request『 1 』」，「 to 1 」 is this ），
/// [`Continuation::max_rounds`] is ** total**。 3 = 3 （total 4 ），
/// 1 = 1 。
///
/// from before this is `(max >= 2).then_some(max)`， 1 ： 3 
/// 2 ， 1（PUT ）return `None` —— config for 
/// that （「 1 not ， and is 1 」，
/// `AUTO_CONTINUE_MIN_ROUNDS`）。
pub fn plan() -> Option<usize> {
    if !config::auto_continue() {
        return None;
    }
    Some(total_rounds_of(config::auto_continue_max_rounds()))
}

/// config（）→ [`Continuation::max_rounds`]（ total）。
///
/// is as can ：`plan()` is config， and use config
/// will （ already has ），、configconfig。
fn total_rounds_of(continuations: i64) -> usize {
    continuations.clamp(1, 1_000) as usize + 1
}

#[cfg(test)]
mod round_budget_tests {
    //! → total 。 is as this ** 1 **： from before `plan()`
    //! 1 （`(max >= 2).then_some(max)`）， is 3 2 、
    //! 1（PUT ）return None can —— config
    //! 「， not need use 」（ `AUTO_CONTINUE_MIN_ROUNDS`）。

    use super::*;

    #[test]
    fn the_configured_continuation_count_becomes_rounds_including_the_first() {
        assert_eq!(total_rounds_of(1), 2, "设 1 = 续一轮（不是关掉功能）");
        assert_eq!(total_rounds_of(3), 4, "设 3 = 续三轮（默认值，别少一轮）");
        assert_eq!(total_rounds_of(8), 9, "上限 8 = 续八轮");
        // （config already 1..=8， this is ： that ）
        assert_eq!(total_rounds_of(0), 2);
        assert_eq!(total_rounds_of(-5), 2);
    }
}

impl Continuation {
    /// ： chat 。
    ///
    /// 1 no when **return** that 1 （ `usage` not ）—— 「 no 」
    /// is request ， that path must and before ，otherwise「
    /// as 」 no 。
    pub async fn complete(&self, first: Value) -> Value {
        let mut body = first;
        let mut texts: Vec<String> = Vec::new();
        let mut reasonings: Vec<String> = Vec::new();
        let mut tokens: Vec<usage::UsageTokens> = Vec::new();
        // （ 3 messages need 1、2 ，
        // not is after that 1 —— otherwise will 1 ）
        let mut working = self.base_body.clone();
        let mut rounds = 1usize;
        loop {
            // not `choices[0].message` not not （fail-open， refusal）
            let Some(parts) = parts_of_completion(&body) else {
                break;
            };
            let repeat = texts
                .last()
                .is_some_and(|previous| is_repeat(previous, &parts.text));
            // 「 already + 」 （ and 1 ， [`verdict_of`]）。
            // `any_tool_calls` this etc that ： before 1 use will 
            // 「 not 」 and break， not to this 1 
            let verdict = verdict_of(
                &(join_text(&texts) + &parts.text),
                &(join_text(&reasonings) + &parts.reasoning),
                parts.has_tool_calls,
                &parts,
            );
            // that 1 ** in **：path that when ，
            // not ； this if ， 1 will 「stream:false not to 、
            // stream:true to 」—— must to 1 。
            // use is 「 not 」， not is 「 」。
            texts.push(parts.text.clone());
            reasonings.push(parts.reasoning.clone());
            tokens.push(tokens_of_completion(&body));
            if repeat || !looks_truncated(&verdict) || rounds >= self.max_rounds || self.telemetry.is_cancelled() {
                break;
            }
            // no has messages not （， to this by not ）
            if !append_continuation(&mut working, &parts.text, looks_reasoning_only(&verdict)) {
                break;
            }
            rounds += 1;
            match self.forward_round(working.clone()).await {
                Ok(ForwardOutcome::Completion { body: next }) => {
                    // and 1 、 1 （「 as 」limit，
                    // `stream` that ）
                    self.telemetry.note_attempt_notice("自动续写轮（上一轮疑似被截断）");
                    body = next;
                }
                // not is / failure：`body` up 1 that ，
                // client to and before 1 （ not ）
                _ => break,
            }
        }
        if texts.len() < 2 {
            return body;
        }
        // usage and ：（ report_usage is 「 after 1 as 」，
        // this 「 after 1 」 and ），client to that 
        let merged = sum_tokens(&tokens);
        self.telemetry.report_usage(&usage_json(&merged));
        if let Some(usage_object) = body.as_object_mut() {
            usage_object.insert("usage".to_string(), usage_json(&merged));
        }
        let Some(choice) = body
            .get_mut("choices")
            .and_then(Value::as_array_mut)
            .and_then(|choices| choices.first_mut())
            .and_then(|choice| choice.as_object_mut())
        else {
            return body;
        };
        let Some(message) = choice.get_mut("message").and_then(Value::as_object_mut) else {
            return body;
        };
        message.insert("content".to_string(), json!(join_text(&texts)));
        let merged_reasoning = join_text(&reasonings);
        if !merged_reasoning.is_empty() {
            message.insert("reasoning_content".to_string(), json!(merged_reasoning));
        }
        body
    }

    /// ： ** 1 ** down 。
    ///
    /// and has 「 when 」 1 （ after mpsc，pathreturn
    /// `ReceiverStream`， `providers::kuku::chat` etc 1 ）—— in 
    /// `poll_next` poll 1 future has `cancellation::CancellableStream` 1 ，
    /// not need need in up 。
    ///
    /// down when ：** when **； has （ null `finish_reason` 、
    /// `[DONE]`、error） not ， etc EOF after 「」（ not ） or 「」
    /// （ down 1 ）。 no when ， and and before ，
    /// has after 1 to when to 1 （EOF in after ）。
    pub fn stream(&self, first: Downstream) -> Downstream {
        let service = self.service.clone();
        let telemetry = self.telemetry.clone();
        let client_headers = self.client_headers.clone();
        let allowed_providers = self.allowed_providers.clone();
        let mode = self.stream;
        let max_rounds = self.max_rounds;
        let mut working = self.base_body.clone();
        let (sender, receiver) = mpsc::channel::<Result<Bytes, io::Error>>(64);
        crate::spawn_task(async move {
            let mut current = first;
            let mut texts: Vec<String> = Vec::new();
            let mut tokens: Vec<usage::UsageTokens> = Vec::new();
            // client already to （， by [`verdict_of`]）
            let mut joined = String::new();
            let mut thinking = String::new();
            let mut any_tool_calls = false;
            let mut rounds = 1usize;
            loop {
                let mut accumulator = TurnAccumulator::default();
                let mut held: Vec<Bytes> = Vec::new();
                let mut held_bytes = 0usize;
                let mut held_full = false;
                let mut client_gone = false;
                while let Some(item) = current.next().await {
                    match item {
                        Ok(frame) => {
                            // 1 ，** after all 1 and **： has up in 
                            // `finish_reason` after 1 usage， if by 「」
                            // ，client will to usage to finish —— that is 
                            // ， 1 。
                            if accumulator.push_frame(&frame) || !held.is_empty() {
                                // **must has limit**： up in after 1 
                                // （ or up ） will this Vec none —— 
                                // not this to up 1 。 after not ，
                                // ** not **「 already 」 （ ，
                                // and path held is 1 ）， not 
                                // 「 need not need 」 （ is ， not is held）。
                                if held_bytes + frame.len() > MAX_HELD_BYTES {
                                    if !held_full {
                                        held_full = true;
                                        logging::verbose(
                                            "Continuation",
                                            "终止帧之后的补帧超过上限，后续帧丢弃（本轮不再累积）",
                                        );
                                    }
                                } else {
                                    held_bytes += frame.len();
                                    held.push(frame);
                                }
                            } else if sender.send(Ok(frame)).await.is_err() {
                                client_gone = true;
                                break;
                            }
                        }
                        Err(error) => {
                            // `ForwardStream` not will to Err（ in poll_next error
                            // error + [DONE]）； to Err not is —— after ，
                            // in this 1 
                            let _ = sender.send(Err(error)).await;
                            return;
                        }
                    }
                }
                if client_gone {
                    // client： 1 no not 1 up quota
                    return;
                }
                let parts = accumulator.parts().clone();
                tokens.push(accumulator.tokens());
                let repeat = texts
                    .last()
                    .is_some_and(|previous| is_repeat(previous, &parts.text));
                // in ** and before **：verdict is 「 before + 」 ，
                // is client will to 。
                // ⚠️ must up `parts.text` ：「 already that 」 in 1 is empty，
                // is 1 not 、 1 not will （2026-10-14 up to this ）
                let verdict = verdict_of(
                    &(joined.clone() + &parts.text),
                    &(thinking.clone() + &parts.reasoning),
                    any_tool_calls,
                    &parts,
                );
                joined.push_str(&parts.text);
                thinking.push_str(&parts.reasoning);
                any_tool_calls |= parts.has_tool_calls;
                texts.push(parts.text.clone());
                let want_more = rounds < max_rounds
                    && !accumulator.saw_error()
                    && !repeat
                    && looks_truncated(&verdict)
                    && !telemetry.is_cancelled()
                    // client in this 1 EOF to 「」。 up that 
                    // `sender.send` failure is 1 ** already ** ， this 
                    // no has —— after will 1 up （，
                    // ）。`is_closed` is ， 1 。
                    && !sender.is_closed()
                    // ** 1 no **（ and empty）： no has can 。
                    // is ****， up 1 in 、，
                    // for empty not （similarity("", x) 0）—— not will 
                    // 1 to `max_rounds`， 1 。
                    // Note「empty」：empty has is 3 need 。
                    && !(parts.text.trim().is_empty() && parts.reasoning.trim().is_empty());
                if want_more
                    && append_continuation(&mut working, &parts.text, looks_reasoning_only(&verdict))
                {
                    rounds += 1;
                    match forward_round_with(
                        &service,
                        mode,
                        &client_headers,
                        &telemetry,
                        allowed_providers.clone(),
                        working.clone(),
                    )
                    .await
                    {
                        Ok(ForwardOutcome::Stream { stream: next, .. }) => {
                            // in ** this 1 ** up （ already is after 1 ；
                            // if this 1 ， in after that ——
                            // not is ： ， 1 ）
                            //
                            // ⚠️ this 1 ** can can not **：`note_attempt_notice` is 
                            // 「 as 」， that 1 if already 「unavailable、
                            // 」， to 。 not to not etc no ——
                            // is `attempts` and （ `TelemetrySnapshot::attempts`）。
                            telemetry.note_attempt_notice("自动续写轮（上一轮疑似被截断）");
                            // need ： up path 
                            drop(held);
                            current = next;
                            continue;
                        }
                        Ok(ForwardOutcome::Completion { .. }) => {
                            logging::verbose("Continuation", "续写轮回的是非流式体，停止续写");
                        }
                        Err(error) => {
                            // not note_error：client will to up 1 ， this request
                            // in and before 1 （failure not this successfailure）
                            logging::verbose(
                                "Continuation",
                                &format!("续写轮转发失败，按本轮结束收尾: {error}"),
                            );
                        }
                    }
                }
                for frame in held {
                    if sender.send(Ok(frame)).await.is_err() {
                        break;
                    }
                }
                break;
            }
            if texts.len() > 1 {
                telemetry.report_usage(&usage_json(&sum_tokens(&tokens)));
            }
        });
        Box::new(ReceiverStream::new(receiver))
    }

    /// 1 request（ and has `body` and `dedupe_key`）。
    async fn forward_round(&self, body: Value) -> Result<ForwardOutcome, crate::server::errors::GatewayError> {
        forward_round_with(
            &self.service,
            self.stream,
            &self.client_headers,
            &self.telemetry,
            self.allowed_providers.clone(),
            body,
        )
        .await
    }
}

/// and 1 request。
///
/// `dedupe_key` **empty** is must ， not is ： that `InFlightGuard` in 
/// to client up （ not drop not ）， if use 1 key will etc 
/// —— in etc ， etc 45 seconds（`mod.rs` 
/// `wait_for_in_flight` / `INFLIGHT_WAIT_MS`）。empty in 。
///
/// by is as need use ：`complete` in `self` up ，
/// `stream` in spawn owned up （ that not to `self`）。
async fn forward_round_with(
    service: &UpstreamService,
    stream: bool,
    client_headers: &HeaderMap,
    telemetry: &Arc<usage::RequestTelemetry>,
    allowed_providers: Option<KeyScope>,
    mut body: Value,
) -> Result<ForwardOutcome, crate::server::errors::GatewayError> {
    if body.get("_wb_route_group_id").and_then(Value::as_str).is_some() {
        let snapshot = telemetry.snapshot();
        let candidate = snapshot.route.as_ref().and_then(|route| route.get("pickedCandidate")).and_then(Value::as_str).unwrap_or("");
        if candidate.is_empty() || snapshot.account_id.is_empty() || snapshot.upstream_model.is_empty() {
            return Err(crate::server::errors::GatewayError::with_status(503, "路由组自动续写缺少原模型与账号，已停止续写"));
        }
        body["_wb_route_continue"] = json!({ "candidateId": candidate, "accountId": snapshot.account_id, "model": snapshot.upstream_model });
    }
    service
        .forward(ForwardRequest {
            body,
            stream,
            dedupe_key: String::new(),
            client_headers: client_headers.clone(),
            telemetry: telemetry.clone(),
            allowed_providers,
            // and ： not 、 not 、 not 
            pinned_account: None,
            ignore_model_gate: false,
            proxy_override: None,
        })
        .await
}

#[cfg(test)]
mod truncation_criteria {
    //! and （module header for ）。
    //!
    //! 1 ** for **： is 「 already 1 」——
    //! client will to ， 1 up quota。 is to （），
    //! this use is 「 not 」。

    use serde_json::json;

    use super::*;

    fn parts(text: &str, reasoning: &str, finish: Option<&str>) -> TurnParts {
        TurnParts {
            text: text.to_string(),
            reasoning: reasoning.to_string(),
            has_tool_calls: false,
            finish: finish.map(str::to_string),
        }
    }

    /// ：「 its 」 ， 1 not 。
    #[test]
    fn complete_answers_are_not_flagged() {
        for text in [
            "答案是 42。",
            "Totals: 3 items, 2 shipped, 1 returned.",
            "```rust\nfn main() {}\n```",
            "配置在 `~/.qoder-cn/settings.json`，改完重启即可",
            "- 第一步：打开面板\n- 第二步：点「刷新模型」",
        ] {
            assert!(
                !looks_truncated(&parts(text, "", Some("stop"))),
                "正常答复被判成截断（会重复输出并白花一轮配额）：{text:?}"
            );
        }
    }

    /// 1： not ，**** finish_reason use 。
    ///
    /// for in 1 use ： 、 no has tool_calls when must ——
    /// otherwise「」 this its in 1 。
    #[test]
    fn tool_call_turns_are_never_continued() {
        let mut with_calls = parts("我把文件读完之后", "", Some("length"));
        with_calls.has_tool_calls = true;
        assert!(!looks_truncated(&with_calls), "工具轮的执行权在客户端，不许续写");
        assert!(
            looks_truncated(&parts("我把文件读完之后", "", Some("length"))),
            "同样的正文没有 tool_calls 时应当点亮（否则上面那条断言是空跑）"
        );
    }

    /// 2：length 1 8 need ， and not 。
    ///
    /// 1 = that 1 ； up to is not is can 。
    #[test]
    fn every_length_alias_counts_as_truncation() {
        for alias in LENGTH_FINISH_ALIASES {
            assert!(
                is_length_finish(alias)
                    && looks_truncated(&parts("就这样", "", Some(alias))),
                "length 别名没认出来: {alias}"
            );
        }
        assert!(is_length_finish("LENGTH"), "大小写不敏感");
        assert!(!is_length_finish("tool_calls"));
        assert!(!is_length_finish("content_filter"), "拒答归 refusal 管，不是截断");
    }

    /// 3：empty + has need ；empty** not **（ that is has empty ）。
    #[test]
    fn empty_visible_text_depends_on_the_reasoning_channel() {
        assert!(looks_truncated(&parts("", "先分析题目，再考虑边界情况。", None)));
        assert!(!looks_truncated(&parts("   ", "", None)), "两个通道都空：不归本模块管");
        // empty + has + finish is content_filter：****。
        // this not is ， is ： in by `refusal` down （ is 
        // up finish ）， can to is that —— 
        // not this finish ，otherwise for 1 has 。
        assert!(looks_truncated(&parts("", "思考", Some("content_filter"))));
    }

    /// 4： as 。
    #[test]
    fn an_unclosed_code_fence_is_hard_evidence() {
        assert!(looks_truncated(&parts("看这段：\n```python\ndef f():\n    return 1", "", None)));
        assert!(
            !looks_truncated(&parts("看这段：\n```python\ndef f():\n    return 1\n```", "", None)),
            "闭合的围栏不该点亮"
        );
    }

    /// 5： —— 「」， by 。
    ///
    /// and is ** not for **：`{a: [1, 2}` must to （
    /// ）， in 1 for in that `}` must** not ** to 
    /// （ will to ， `unbalanced_brackets` ）。
    #[test]
    fn bracket_checks_end_with_an_open_group_not_with_a_stray_closer() {
        assert!(!looks_truncated(&parts("先跑 (npm test) 再提交", "", None)), "配对完整：写完了");
        assert!(looks_truncated(&parts("返回 {a: [1, 2}", "", None)), "仍未闭合必须抓到");
        assert!(looks_truncated(&parts("调用 compute(a, b", "", None)));
        // ： is 「 1 in 」， is not for 
        assert!(
            !looks_truncated(&parts("这里有个坑) 以及另一个)", "", None)),
            "窗口切出来的落单右括号不该点亮"
        );
        // （ 1 no ）
        assert!(looks_truncated(&parts("嵌套 ( ( ( 还没关", "", None)));
    }

    /// 6a：；** not **（）。
    #[test]
    fn dangling_tails_are_flagged_but_a_period_is_not() {
        for text in [
            "总价包括：",
            "第一行 |",
            "等等...",
            "路径是 src\\",
            "注释 #",
            "下一项 -",
            "结果是 :",
            "表格单元 ｜",
        ] {
            assert!(looks_truncated(&parts(text, "", None)), "悬挂尾部没抓到: {text:?}");
        }
        assert!(
            !looks_truncated(&parts("结果是 42.", "", None)),
            "单个句点是正常收尾，不许和省略号混起来"
        );
    }

    /// 6b： + and no 。
    ///
    /// has ：（limit）/ not / already 。
    /// that in has down not （Reason `FRAGMENT_REASONING_RATIO` ），
    /// this not as —— 1 not will 「 has 」。
    #[test]
    fn a_long_think_with_a_stopped_fragment_needs_one_more_round() {
        let thinking = "分".to_string().repeat(FRAGMENT_REASONING_MIN_CHARS);
        assert!(looks_truncated(&parts("好的", &thinking, None)), "整轮烧在思考里");
        assert!(
            !looks_truncated(&parts(&"好的，".repeat(20), &thinking, None)),
            "正文超过上限就不该走这条判据"
        );
        assert!(
            !looks_truncated(&parts("好的", "短思考", None)),
            "思考不够长时，短句可能就是答案"
        );
        assert!(
            !looks_truncated(&parts("收到。", &thinking, None)),
            "正文以句末符号收尾 = 说完了"
        );
    }

    /// must「 after 」， not is that 1 —— up to 1 。
    ///
    /// ： 1 1 no ```（ this ）， up and 。
    /// by that ， is 「」⇒ ，client to 
    /// up 1 。 this use to ：
    /// ** 1 、 must**。
    #[test]
    fn the_verdict_reads_the_stitched_answer_not_the_last_round() {
        let first = parts("示例代码：\n```python\ndef solve():\n    return 42\n", "", Some("stop"));
        assert!(looks_truncated(&first), "第一轮留着没闭合的围栏：该续");
        let second = parts("```\n上面的函数返回 42。", "", Some("stop"));
        assert!(
            looks_truncated(&second),
            "本轮单独看仍是半截（这正是旧判据多点一次火的地方）"
        );
        let stitched = verdict_of(
            &(first.text.clone() + &second.text),
            "",
            false,
            &second,
        );
        assert!(
            !looks_truncated(&stitched),
            "拼接后那道围栏已经闭合：整段答案是写完的，不许再打一轮"
        );
        // （ before no has already ）must —— this is 「」
        // must****： will empty， 1 not 
        let opening = verdict_of(&first.text.clone(), "", false, &first);
        assert!(
            looks_truncated(&opening),
            "首轮的判据里必须带上本轮自己那段文本"
        );
    }

    /// ： before no has use ， not no has 。
    #[test]
    fn a_tool_call_in_the_current_round_stops_the_chain() {
        let mut current = parts("接着写", "", Some("stop"));
        current.has_tool_calls = true;
        let verdict = verdict_of("前面已经写好的整段答案", "", false, &current);
        assert!(verdict.has_tool_calls, "本轮的 tool_calls 必须并进整体判据");
        assert!(!looks_truncated(&verdict));
    }

    /// ：、 0.5 not （ ）。
    #[test]
    fn the_repeat_gate_is_strictly_above_half() {
        assert!((similarity("abcd", "abcd") - 1.0).abs() < f64::EPSILON);
        assert_eq!(0.0, similarity("", "abcd"), "空串不做分母");
        assert!(is_repeat("接着写", "接着写下去"));
        // = 0.5 ⇒ not as 
        assert!(!is_repeat("abcd", "abxy"));
        assert_eq!(0.5, similarity("abcd", "abxy"));
    }

    /// ： assistant + 1 user （ and need for ）。
    #[test]
    fn continuation_appends_the_partial_then_the_instruction() {
        let mut body = json!({"messages": [{"role": "user", "content": "写一首诗"}]});
        assert!(append_continuation(&mut body, "床前明月光，", false));
        let messages = body["messages"].as_array().cloned().unwrap_or_default();
        assert_eq!(3, messages.len());
        assert_eq!("assistant", messages[1]["role"]);
        assert_eq!("床前明月光，", messages[1]["content"], "回放用原文，不许 trim");
        assert_eq!("user", messages[2]["role"]);
        assert_eq!(CONTINUE_INSTRUCTION, messages[2]["content"]);
    }

    /// as empty when not empty assistant ，（ use that ）。
    #[test]
    fn an_empty_partial_does_not_create_an_empty_assistant_turn() {
        let mut body = json!({"messages": [{"role": "user", "content": "继续"}]});
        assert!(append_continuation(&mut body, "   ", true));
        let messages = body["messages"].as_array().cloned().unwrap_or_default();
        assert_eq!(2, messages.len(), "空正文不该占一条 assistant");
        assert_eq!(REASONING_ONLY_INSTRUCTION, messages[1]["content"]);
        // no has messages ： not ，return false and not 
        let mut broken = json!({"prompt": "裸 prompt"});
        assert!(!append_continuation(&mut broken, "半截", false));
        assert_eq!(json!({"prompt": "裸 prompt"}), broken);
    }

    /// need to （ 、 use 、）。
    #[test]
    fn completion_parts_read_the_aggregated_shape() {
        let body = json!({
            "choices": [{
                "message": {"role": "assistant", "content": "半截", "reasoning_content": "想了很久"},
                "finish_reason": "length",
            }],
        });
        let parts = parts_of_completion(&body).expect("聚合体的形状必须读得出");
        assert_eq!("半截", parts.text);
        assert_eq!("想了很久", parts.reasoning);
        assert_eq!(Some("length".to_string()), parts.finish);
        assert!(!parts.has_tool_calls);
        // `reasoning` this has up in use （ ）
        let alt = json!({"choices": [{"message": {"content": "", "reasoning": "思考"}}]});
        assert_eq!(
            "思考",
            parts_of_completion(&alt).unwrap().reasoning,
            "reasoning_content 缺席时要退回 reasoning"
        );
        let with_calls = json!({"choices": [{"message": {"tool_calls": [{"id": "1"}]}}]});
        assert!(parts_of_completion(&with_calls).unwrap().has_tool_calls);
        assert!(parts_of_completion(&json!({"choices": []})).is_none(), "没有 choice 就判不了");
    }

    fn frame(payload: Value) -> Bytes {
        Bytes::from(format!("data: {}\n\n", payload))
    }

    /// ：///。
    #[test]
    fn the_stream_accumulator_reads_downstream_frames() {
        let mut acc = TurnAccumulator::default();
        assert!(!acc.push_frame(&frame(json!({"choices": [{"delta": {"content": "前半"}}]}))));
        assert!(!acc.push_frame(&frame(json!({"choices": [{"delta": {"content": "，后半"}}]}))));
        assert!(!acc.push_frame(&frame(json!({"choices": [{"delta": {"reasoning_content": "思考"}}]}))));
        assert!(!acc.push_frame(&frame(json!({"choices": [{"delta": {"tool_calls": [{"index": 0}]}}]}))));
        assert_eq!("前半，后半", acc.parts().text, "逐帧拼接");
        assert_eq!("思考", acc.parts().reasoning);
        assert!(acc.parts().has_tool_calls);
        assert!(!acc.saw_error());
        // ：return true（ use not ）
        assert!(acc.push_frame(&frame(json!({"choices": [{"delta": {}, "finish_reason": "length"}]}))));
        assert_eq!(Some("length".to_string()), acc.parts().finish);
        assert!(acc.push_frame(&Bytes::from_static(b"data: [DONE]\n\n")));
        assert!(acc.saw_done());
    }

    /// 「 not is 」 ：error need 、 and JSON not not 。
    #[test]
    fn error_comment_and_unparsable_frames_are_classified() {
        let mut acc = TurnAccumulator::default();
        assert!(acc.push_frame(&frame(json!({"error": {"message": "上游断了", "type": "proxy_error"}}))));
        assert!(acc.saw_error(), "错误帧必须让调用侧别再续写");
        assert!(!acc.saw_done());

        let mut noisy = TurnAccumulator::default();
        assert!(!noisy.push_frame(&Bytes::from_static(b": keepalive\n\n")));
        assert!(!noisy.push_frame(&Bytes::from_static(b"data: not json\n\n")));
        assert!(!noisy.push_frame(&frame(json!({"choices": [{"delta": {"content": "A"}}], "x": 1}))));
        assert_eq!("A", noisy.parts().text, "坏帧只被跳过，不许把已攒的正文弄脏");
    }

    /// usage： after 1 ， and ， and and can `extract_usage` 。
    ///
    /// after is ****： and is ， by ；
    /// for not up when will 0（「 this request no 」）。
    #[test]
    fn usage_is_last_wins_per_round_and_summed_across_rounds() {
        let mut acc = TurnAccumulator::default();
        acc.push_frame(&frame(json!({"choices": [], "usage": {"prompt_tokens": 10, "completion_tokens": 2}})));
        acc.push_frame(&frame(json!({"choices": [], "usage": {"prompt_tokens": 10, "completion_tokens": 9, "total_tokens": 19}})));
        let round = acc.tokens();
        assert_eq!(9, round.completion, "同一轮里后到的 usage 是最终值");

        let rounds = [
            usage::UsageTokens { prompt: 10, completion: 9, total: 19, cache_read: 4, credit: Some(0.25) },
            usage::UsageTokens { prompt: 20, completion: 5, total: 25, cache_read: 0, credit: Some(0.5) },
        ];
        let merged = sum_tokens(&rounds);
        assert_eq!(30, merged.prompt, "每轮的 prompt 都含全部历史，真实成本按轮求和");
        assert_eq!(14, merged.completion);
        assert_eq!(44, merged.total);
        assert_eq!(4, merged.cache_read);
        assert_eq!(Some(0.75), merged.credit, "积分也是逐轮扣的，跨轮按和计");
        let json = usage_json(&merged);
        assert_eq!(merged, usage::extract_usage(&json).unwrap(), "合并值必须能被提取侧读回来");
    }

    /// not ： up no has ， will and markdown。
    #[test]
    fn rounds_are_joined_without_separators() {
        assert_eq!("abc", join_text(&["ab".to_string(), "c".to_string()]));
        assert_eq!("x", join_text(&["x".to_string()]));
        assert_eq!("", join_text(&[]));
    }
}
