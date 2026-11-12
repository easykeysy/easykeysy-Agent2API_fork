//! Antigravity （`thoughtSignature`） **server**。
//!
//! ── Why need need （issue #291）──────────────────────────────
//! up `functionCall` ** need **： not 400（
//! 「Function call is missing a thought_signature in functionCall parts」）；
//! 1 400（「Corrupted thought signature」—— ，
//! 「default」 not ）。
//! ── use ────────────────────────────────────────────────
//! response and in use 1 。 by will + use id
//! ， not client use call_1 when ； no has when 
//! 。client not 。
//!
//! in `tool_calls[].extra_content.google.thought_signature`
//! （Google OpenAI ）response —— that is ，
//! OpenAI client down 1 will ， is 「 → client
//! 」 400；client no ， by 。
//!
//! is **server**、 not when （Manager by call id、
//! ；9router by call id / will ）。 before not request，
//! that in 「 up not 」 before up ； before （ up 400）， is 
//! up that 。：
//! - ****（response）：`antigravity_stream` `tool_call_frames` —— and 
//! client use that 1 ， 1 ；
//! - ****（request）：`antigravity_outbound` —— client 
//! （ that is 1 ）， is client no when 。
//!
//! ── ──────────────────────────────────────────────────
//! 1. `sessionId + call_id → signature`（）： up to functionCall id，OpenAI 
//! clientmust use （`tool_calls[].id` / `tool_call_id` need for ），
//! in up —— client in this 1 ；
//! 2. `sessionId + → signature`（）： will / id，
//! that when by will 。sessionId is will （
//! `antigravity_outbound::session_id`， from 1 ， 
//! ）。 if to 「 this 1 use 」「 will 」， this 1 can can 
//! up —— that 400（ and no has when ）， not will 。
//!
//! ── and ：（two-generation） has ──────────
//! ；（[`CAPACITY`] ） as before 、empty；
//! before ，** in before not **（ need can use ，path
//! —— not ， in down 1 ）。 2×CAPACITY 
//! （ 1 1KB + ， up ~4MB）。 not `lru` ：「
//! 」， as 1 crate not ， for 
//! 「 for 」 this 。 in ：empty ——
//! before old will not ， that will down 1 can can 400， to 
//! to new tool_call（ as ， 1 limit）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`： unwrap/expect/panic； by 「 in / ignore
//! 」 —— this can ， not 1 requestfailure。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// （ 2×CAPACITY ，module header）
const CAPACITY: usize = 2048;

/// 「」 ： `current`， as `previous`
#[derive(Default)]
struct Generations {
    current: HashMap<String, String>,
    previous: HashMap<String, String>,
}

static CACHE: OnceLock<Mutex<Generations>> = OnceLock::new();

/// （ by 「 use 」： this can ，module header）
fn cache() -> &'static Mutex<Generations> {
    CACHE.get_or_init(|| Mutex::new(Generations::default()))
}

/// before ：call id （ and will ， before not ）
const CALL_PREFIX: &str = "c|";
/// before ： will + 
const SESSION_PREFIX: &str = "s|";

/// 1 「 will + call id → 」（response：tool_call down when use ）。
pub fn remember_call(session: &str, call_id: &str, signature: &str) {
    if session.is_empty() || call_id.is_empty() || signature.is_empty() {
        return;
    }
    insert(format!("{CALL_PREFIX}{session}|{call_id}"), signature.to_string());
}

/// 1 「 will + → 」（response； will not to when not ，
/// empty will will —— that not is 1 ）。
pub fn remember_for_session(session: &str, name: &str, signature: &str) {
    if session.is_empty() || name.is_empty() || signature.is_empty() {
        return;
    }
    insert(format!("{SESSION_PREFIX}{session}|{name}"), signature.to_string());
}

/// by will and call id （request ）。
pub fn lookup_call(session: &str, call_id: &str) -> Option<String> {
    if session.is_empty() || call_id.is_empty() {
        return None;
    }
    lookup(&format!("{CALL_PREFIX}{session}|{call_id}"))
}

/// by 「 will + 」（request ）。
pub fn lookup_for_session(session: &str, name: &str) -> Option<String> {
    if session.is_empty() || name.is_empty() {
        return None;
    }
    lookup(&format!("{SESSION_PREFIX}{session}|{name}"))
}

/// ： and is new when （ old if ， before has 1 can ）
fn insert(key: String, value: String) {
    if let Ok(mut guard) = cache().lock() {
        if guard.current.len() >= CAPACITY && !guard.current.contains_key(&key) {
            guard.previous = std::mem::take(&mut guard.current);
        }
        guard.current.insert(key, value);
    }
    // in （ panic ）→ this ： can ， not 
}

/// ： after before ； in 
fn lookup(key: &str) -> Option<String> {
    let guard = cache().lock().ok()?;
    guard
        .current
        .get(key)
        .or_else(|| guard.previous.get(key))
        .cloned()
}
