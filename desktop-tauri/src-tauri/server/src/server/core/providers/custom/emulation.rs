//! **OpenCode client**（ up `clientEmulation: "opencode"`）。
//!
//! ── Why need need （ up ，2026-10-14 for opencode v2.0.27 ）──
//! `https://opencode.ai/inference/v1` ****（`*-free` etc ） in when 
//! will 403：
//! ```text
//! {"type":"error","error":{"type":"FreeTierError","message":"OpenCode's free
//!  tier can only be used from within OpenCode. Download it at
//!  https://opencode.ai and keep building."}}
//! ```
//! this use is **client request**（`opencode serve` 1 ，
//! `BUN_CONFIG_VERBOSE_FETCH=curl` request） for ，
//! 。current this need when ：
//! 1. is **** `public`（`Authorization: Bearer public`）——
//! ， not need need 、 not need need ；
//! 2. ** will when in and **：`x-opencode-session`、
//!      `x-opencode-session-id`、`x-session-affinity`、`x-session-id`。
//! `x-opencode-session-id` 403；
//! 3. `x-opencode-request` and `x-opencode-project` ** 1 not can ** —— 
//! up （ `msg_…`、 sha1） 403， v2 client this 
//! not ；
//! 4. request is 「 can 」：`stream: true`， and `tools` when 1 
//! shell （`bash` or `shell`） and 1 `read`（ need ，
//! parameter and not ； [`SHELL_TOOL_ALIASES`]）。
//!
//! will id ** and **（`ses_<12 hex><14 Base62>`， 12 
//! ≥ [`LOW12_FLOOR`]），** not is in server** —— 
//! （[`session_id`]， 12 [`LOW12_TAG`]）， not need need or 
//! client。：`/zen/v1` already （ 1 403），current is 
//! `/inference/v1`（ baseUrl already ， `custom_providers`
//! ）。
//!
//! this 。** is has as opt-in**： in 
//! `clientEmulation: "opencode"` when （「OpenCode Zen」 will up 
//! ）， and its path not 。
//!
//! ── has ：OpenCode **Go**（`/zen/go/v1`）─────────────
//! Go is ， need and ** not **： need client** 1 
//! will **（ up `docs/go/#where-can-i-use-it` by need ：「Send a
//! stable session ID in `x-opencode-session` for each conversation」），
//! not need ， request（ #44300/#44382）。
//! for Go 1 —— [`apply_go_headers`]： will and UA，
//! and request 1 not 。 by `providers::custom::forward` by 
//! ****（ `custom_providers::is_opencode_go_endpoint`），
//! and Go ** not `clientEmulation` ** —— that need is 
//! need ， not is 「」， no has can 。
//!
//! ── and （ use before need ）─────────────────────────
//! · will ** use request**（、 `stream: true`）——
//! and 「」 default， in when ；
//! · is up ， not is to use ：client** no has ** when 
//! 1 and down `tool_choice: "none"`， 。client when 
//! its `tool_choice`（ will use client）， when and 
//! and in —— up need is this ；
//! · up when can can this （2026-09、2026-10-09、2026-10-13、2026-10-14
//! ）， when as 403 `FreeTierError`。 to this 403， is 
//! ** is not is `/inference/v1` → will not → has no has 
//! `x-opencode-request` / `x-opencode-project` → will id 12 →
//! request `stream` and for **， after up then （ is 「 
//! for 、 403」）。
//! this is **client request** for 
//! （`BUN_CONFIG_VERBOSE_FETCH=curl opencode serve`）， ——
//! that in `/zen/v1` up ， will 1 。
//!
//! ── and its ────────────────────────────────────────
//! ****：request（[`ensure_agent_shape`]）、 will id and 
//! is （[`session_id`] / [`is_acceptable_session_id`]）、
//! （[`apply_headers`]）。 not 、 to by `providers::custom::forward` ；
//! and in `core::custom_providers`。

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// up （ and `custom_providers::CLIENT_EMULATION_OPENCODE` ；
/// in that is as and in 1 ， this use ）。
pub use crate::server::core::custom_providers::CLIENT_EMULATION_OPENCODE;

/// ： up （none need 、none）。
pub const ANONYMOUS_KEY: &str = "public";

/// CLI User-Agent（ for can ）
const USER_AGENT: &str = "opencode/1.18.31 (windows amd64; node22)";

/// need （， has ）： 1 shell 
/// + 1 `read`，。
const STUB_TOOLS: [&str; 2] = ["bash", "read"];

/// 「shell 」 etc —— up `bash` or `shell` 1 （ v2 client
/// is `shell`）。 1 use [`STUB_TOOLS`] `bash`。
const SHELL_TOOL_ALIASES: [&str; 2] = ["bash", "shell"];

/// will Base62 （ and for 1 ）
const BASE62: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// will ：** user ** content 。
///
/// Why use not is ： for not ， can 1 for 
/// 1 to 1 will —— up by will and ， id etc 
/// 。 not to （ no has user / content as empty） when to empty， use 
/// will （ for ，none and need ）。
pub fn conversation_seed(body: &Value) -> String {
    let Some(messages) = body.get("messages").and_then(Value::as_array) else {
        return String::new();
    };
    for message in messages {
        if message.get("role").and_then(Value::as_str) != Some("user") {
            continue;
        }
        let Some(content) = message.get("content") else { continue };
        if content.is_null() {
            continue;
        }
        // ；（） and its —— and for 
        // （`json.Marshal(m["content"])`） 1 ：** not **，
        // need 1 for 1 can 
        let seed = match content {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        };
        if !seed.is_empty() {
            return seed;
        }
    }
    String::new()
}

/// any → up need `ses_<12 hex><14 Base62>` will id。
///
/// （ and ， [`is_acceptable_session_id`]） when **return** ——
/// use and ； not new 。
///
/// when 12 as [`LOW12_TAG`]（client ）， its 36 and 
/// 14 `SHA-256()`：** 1 for to 1 id**， up 
/// by will and not （ this is user 
/// Reason， [`conversation_seed`]）。
pub fn session_id(seed: &str) -> String {
    if is_acceptable_session_id(seed) {
        return seed.to_string();
    }
    let seed = if seed.is_empty() {
        // empty（ no has can for ） use ： will ，none and need ，
        // must —— otherwise up 1 403
        random_seed()
    } else {
        seed.to_string()
    };
    let digest = Sha256::digest(format!("ses\u{0}{seed}").as_bytes());
    // 12 by client（ by LOW12_TAG ）： use before 
    // 6 ， 12 is ， has 64/4096 in 。
    let mut bits = 0u64;
    for byte in &digest[..6] {
        bits = (bits << 8) | u64::from(*byte);
    }
    let bits = (bits & !0xFFF) | u64::from(LOW12_TAG);
    format!("ses_{bits:012x}{}", base62_fixed(&digest[6..16], 14))
}

/// `ses_` + 12 hex + 14 Base62（ up 2026-09-16 for its 1 403）
fn valid_session_id(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("ses_") else { return false };
    if rest.len() != 12 + 14 {
        return false;
    }
    let (hex, tail) = rest.split_at(12);
    hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && tail.bytes().all(|byte| BASE62.contains(&byte))
}

/// will id 12 ** down **。
///
/// 2026-10-13 for `POST /zen/v1/chat/completions` ： 12 
/// `0xFBF`（4031） 、`0xFC0`（4032）。 up ** this 12 ** ——
/// `ses_0000000fffff`（）、`ses_ecbcb548a000`（） ，
/// can and id none； up 23 id 12 is `0xFFE`。
/// 5 、6 id 1 ， in 146 seconds no has —— this 1 ** not is **
/// when new ， is 。
///
/// 2026-10-14 `/inference/v1` after ， down ****（ 12 
/// `0x100` 、`0xFFE` ）， no has this 。
const LOW12_FLOOR: u16 = 0xFC0;

/// will id 12 ****：client as this ， in in 。
///
/// use [`LOW12_FLOOR`]、 use ，： is 「 up when 
/// 」， is 「client」， will up if ， 1 。
const LOW12_TAG: u16 = 0xFFE;

/// this can not can ** will **：（[`valid_session_id`]） and 
/// （ 12 ≥ [`LOW12_FLOOR`]）。
///
/// is ： is 2026-09 、 is 2026-10 ，
/// 1 will 「 this id Why 」 will none from 。
pub fn is_acceptable_session_id(text: &str) -> bool {
    if !valid_session_id(text) {
        return false;
    }
    // `ses_` after 12 is hex（valid_session_id already is ASCII）， 3 
    // 12 。 use get not is ： is not panic。
    let Some(hex) = text.get(4..16) else { return false };
    let Some(low) = hex.get(9..12).and_then(|tail| u16::from_str_radix(tail, 16).ok()) else {
        return false;
    };
    low >= LOW12_FLOOR
}

/// （`getrandom` failure when when —— this is will id ，
/// not is ， after only is 「 1 for and not 」， not this requestfailure）
fn random_seed() -> String {
    let mut bytes = [0u8; 16];
    if getrandom::getrandom(&mut bytes).is_ok() {
        return bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    }
    format!("fallback-{}", crate::server::logging::now_ms())
}

/// Base62（ to `width` ； not then from ）
fn base62_fixed(bytes: &[u8], width: usize) -> String {
    // 1 128 ：「 256 」 and use 62 。
    // use u32 （ not ）—— 14 Base62 83 bit，
    // 16 ， and 。
    let mut digits = vec![0u32; width];
    for byte in bytes {
        let mut carry = u32::from(*byte);
        for digit in digits.iter_mut().rev() {
            let value = *digit * 256 + carry;
            *digit = value % 62;
            carry = value / 62;
        }
        // （ width Base62 can ）：
        // up and ， not 1 
    }
    digits
        .iter()
        .map(|digit| BASE62[(*digit as usize).min(61)] as char)
        .collect()
}

/// ： v2 client will to `headers` up 。
///
/// `headers` use default（`Authorization` / `Content-Type`），
/// —— is ** key **： key use （must），
/// no use （）。 this and `ProviderQuirks::apply_to` and 
/// ，default、， not can 。
///
/// ── is **** ， 1 1 not ────────────────
/// · ** will must when in and **（ `x-opencode-session-id` → 403）；
/// · ** not ** `x-opencode-request` / `x-opencode-project` —— v2 client
/// this not ， up 403。 old 
/// `req_…` / `prj_…` is 2026-10 this 403 1 ，。
pub fn apply_headers(headers: &mut Vec<(String, String)>, api_key: &str, session: &str) {
    let credential = if api_key.trim().is_empty() {
        ANONYMOUS_KEY
    } else {
        api_key.trim()
    };
    let pairs = [
        ("Authorization".to_string(), format!("Bearer {credential}")),
        // CLI ：「 not client」， will 403
        ("User-Agent".to_string(), USER_AGENT.to_string()),
        ("x-opencode-client".to_string(), "cli".to_string()),
        // will （ 1 for 1 id， up by and ）
        ("x-opencode-session".to_string(), session.to_string()),
        ("x-opencode-session-id".to_string(), session.to_string()),
        ("x-session-affinity".to_string(), session.to_string()),
        ("X-Session-Id".to_string(), session.to_string()),
    ];
    merge(headers, pairs);
}

/// **Go **（`/zen/go/v1`） will 。 in Go up 1 not need need ，
/// Go this 1 （ up is ）。
pub const GO_SESSION_HEADER: &str = "x-opencode-session";

/// Go User-Agent：Go need client「」（`my-coding-agent/1.0`
/// this ），** not need use HTTP ** —— default `undici` is 
/// that ， this pathmust。 crate ，update。
fn go_user_agent() -> String {
    format!("agent2api/{}", env!("CARGO_PKG_VERSION"))
}

/// **Go **（`/zen/go/v1`） ： will and UA。
///
/// and is ：Go need is 「 for 1 will 」 this ** 
/// by need **（ 400 MissingSessionID， this check in before 、 and not 
/// ）。：
/// · ** not ** —— Go use API Key， this 1 Authorization not ；
/// · ** not request** —— Go （ #44300/#44382），
/// `ensure_agent_shape` that for Go 1 not ；
/// · will id [`session_id`] （ 1 for 1 id），
/// Go by and and 。
pub fn apply_go_headers(headers: &mut Vec<(String, String)>, session: &str) {
    merge(headers, [
        ("User-Agent".to_string(), go_user_agent()),
        (GO_SESSION_HEADER.to_string(), session.to_string()),
    ]);
}

/// 1 for by and （ASCII not ；， its ）
fn merge(
    headers: &mut Vec<(String, String)>,
    pairs: impl IntoIterator<Item = (String, String)>,
) {
    for (key, value) in pairs {
        match headers.iter_mut().find(|(name, _)| name.eq_ignore_ascii_case(&key)) {
            Some(slot) => slot.1 = value,
            None => headers.push((key, value)),
        }
    }
}

/// request「 can 」：`stream: true` + `tools` must when 1 
/// shell and 1 `read`（module header 4 ）。
///
/// return is request（ to use ）。：
/// · `stream` true（ up ； down by 
/// —— this and for **all** up has 1 ， not is this new as ）；
/// · ** ，client 1 **（ not parameter）。
/// Note is 「」 not is 「 no has 」：client when ， if not 
/// shell / `read` this ， not —— this request 1 403；
/// · shell ** etc **：client `bash` or `shell`， not 
/// （ up ， [`SHELL_TOOL_ALIASES`]）；
/// · client** no has tools** when `tool_choice: "none"`： as ，
/// use 。client when not `tool_choice` —— that will 
/// can use use 。
pub fn ensure_agent_shape(body: &mut Value) -> bool {
    let mut changed = false;
    let Some(object) = body.as_object_mut() else { return false };
    if object.get("stream").and_then(Value::as_bool) != Some(true) {
        object.insert("stream".to_string(), Value::Bool(true));
        changed = true;
    }
    let had_tools = object
        .get("tools")
        .and_then(Value::as_array)
        .is_some_and(|tools| !tools.is_empty());
    let mut tools: Vec<Value> = object
        .get("tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let declared = tool_names(&tools);
    for name in STUB_TOOLS {
        // shell use etc ：`bash` this need client `bash` or `shell` 
        let satisfied = if name == "bash" {
            declared.iter().any(|item| SHELL_TOOL_ALIASES.contains(&item.as_str()))
        } else {
            declared.iter().any(|item| item.as_str() == name)
        };
        if satisfied {
            continue;
        }
        tools.push(stub_tool(name));
        changed = true;
    }
    object.insert("tools".to_string(), Value::Array(tools));
    if !had_tools {
        object.insert("tool_choice".to_string(), Value::String("none".to_string()));
    }
    changed
}

/// request `tools` already function （chat ）。 not 1 ——
/// up is （`Bash` this not ， use 1 can ）。
fn tool_names(tools: &[Value]) -> Vec<String> {
    tools
        .iter()
        .filter_map(|tool| {
            tool.get("function")
                .and_then(|function| function.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

/// 1 （parameter and not and up ， to can is as when 
/// 1 is ）
fn stub_tool(name: &str) -> Value {
    let property = if name == "bash" { "command" } else { "filePath" };
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": "(internal placeholder — do not call)",
            "parameters": {
                "type": "object",
                "properties": { property: { "type": "string" } },
            },
        },
    })
}
