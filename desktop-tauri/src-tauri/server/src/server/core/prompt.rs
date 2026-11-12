//! has ：`passthrough` / `custom` / `append` 。
//!
//! workbuddy2api `internal/prompt/prompt.go`（、、
//! for ）， is —— **、 not **：
//!
//! ```text
//! ① （）： before use has **/** system ，
//! from 「system 」 —— client no will 。
//! ② （core::sanitize）：/request ，
//! user / assistant / tool_calls / reasoning_content this ** use **，
//! and 1 in （ `upstream::payload`）。
//! ```
//!
//! ── Why need has （ not is ）─────────────────────
//! up by ****（），「 1 」 can 
//! —— that is in 。 is **** （Claude Code / Codex system
//! has ）， can in that ； 1 client can can 
//! new 1 。`custom` from up ：client system that ，
//! up to system is 、 can 1 。
//!
//! ── （ and 1 ）───────────────────────
//! - `passthrough`（**default**）： not system，client —— and before 
//! as 。 when 1 is ；
//! - `custom`：delete messages **all** `system` / `developer` ， in 
//! 1 has （client，「 need use 」
//! use ）；
//! - `append`： in **** `system` / `developer` after 1 
//! has ， has not （client and and use ）。
//!
//! ── （`core::degrade`）used for passthrough / append ─────
//! `custom` down system already by ， not is system 
//! （ is use ），** not path**（`degradable`）。
//! `append` in as （`rewrite` + in ）： is 
//! ， is 1 。
//!
//! ── ─────────────────────────────────────
//! and from （configfile / file / default） by `config` ，
//! when by ；none、none、none IO，「 to body and ，
//! that this 」。

use serde_json::{json, Value};

/// ： when use ** in **（ for 
/// `prompt.Degraded`）。
///
/// ： need is 「 not will 」， when not 
/// use —— not is for ， is ****。
pub const DEGRADED_PROMPT: &str = "You are a helpful assistant. Answer in the user's language, \
follow the user's instructions, and keep replies direct and concise.";

/// default：`custom` / `append` **file** when use 。
///
/// and 1 「 has 1 can use 」， use as 1 down 
/// `custom` file。 is ** ** use 
/// （ that is 300 ， and none， not ）：
/// need is 「 not will up 」， use 
/// path `promptFile`（ to UTF-8 file can ）。
pub const BUILT_IN_PROMPT: &str = "\
You are a helpful coding assistant working inside the user's local development environment.

- Answer in the user's language unless asked otherwise.
- Be direct and concise: lead with the answer, then the details that matter.
- Use the tools you are given to read and change files, and never claim to have run \
something you did not actually run.
- Prefer the smallest change that solves the problem, and match the style of the surrounding code.
- When something is uncertain, say so and check instead of guessing.
- Do not run destructive commands (deleting data, resetting repositories, force-pushing) \
without explicit confirmation from the user.";

/// （configfile is ， `config::KEY_PROMPT_MODE`）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptMode {
    /// client system（default； and before as ）
    #[default]
    Passthrough,
    /// use has client system / developer 
    Custom,
    /// in system / developer after 1 has 
    Append,
}

impl PromptMode {
    /// （ down and 400 use 1 ， 1 ）
    pub const ALL: [PromptMode; 3] = [
        PromptMode::Passthrough,
        PromptMode::Custom,
        PromptMode::Append,
    ];

    /// config and response 
    pub fn as_str(self) -> &'static str {
        match self {
            PromptMode::Passthrough => "passthrough",
            PromptMode::Custom => "custom",
            PromptMode::Append => "append",
        }
    }

    /// config： not + empty（ for Go 
    /// `strings.ToLower(strings.TrimSpace(...))`）。empty by **default**，
    /// and 「 passthrough」 1 ； to `None`（ by use 
    /// is —— file、 400）。
    pub fn parse(text: &str) -> Option<Self> {
        let normalized = text.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Some(PromptMode::Passthrough);
        }
        PromptMode::ALL
            .into_iter()
            .find(|mode| mode.as_str() == normalized)
    }

    /// and use in 
    pub fn label(self) -> &'static str {
        match self {
            PromptMode::Passthrough => "透传客户端 system",
            PromptMode::Custom => "替换为网关提示词",
            PromptMode::Append => "追加网关提示词",
        }
    }

    /// this is will in when use in 。
    ///
    /// `custom` as ：system already by ， not to system 
    /// （module header「used for passthrough / append」）。
    pub fn degradable(self) -> bool {
        self != PromptMode::Custom
    }
}

/// （ and need can 「 this use to is 1 」）。
///
/// is this ****： that > file > default
/// （ `config::resolve_choice`）。 has use ：、
/// already has file、 not use 。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptSource {
    /// no has （`passthrough` ）
    None,
    /// default（`custom` / `append` and no 、 no file）
    BuiltIn,
    /// use file
    File,
    /// use in set**** （）
    Inline,
}

impl PromptSource {
    pub fn as_str(self) -> &'static str {
        match self {
            PromptSource::None => "none",
            PromptSource::BuiltIn => "builtin",
            PromptSource::File => "file",
            PromptSource::Inline => "inline",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PromptSource::None => "不透传（passthrough）",
            PromptSource::BuiltIn => "内置默认提示词",
            PromptSource::File => "提示词文件",
            PromptSource::Inline => "界面里编辑的提示词",
        }
    }
}

/// ****： / / 。
///
/// ── Why is not is 1 ─────────────────────────────────
/// up by ****（2026-09-28 ： 200、 and 1 
/// 405/3012， `providers::zcode::OFFICIAL_PROMPT_NOTE`），「 this 
/// 」 in up is ****： 1 will use as in 1 ，
/// that up not 。
///
/// 1 ****（`zcode::plan` from `zcode_system.json` ，
/// `{cwd}` this ），** use **（config that ）——
/// 1 ， when and （[`Self::or`]）， not 1 in 。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GatewayBlocks {
    /// 1 ：（ `You are ZCode, an interactive coding agent`）
    pub identity: String,
    /// ：（ use 、 that 1 ）
    pub stable: String,
    /// ：（ / up down + Environment）
    pub dynamic: String,
}

impl GatewayBlocks {
    /// ： is config and ， in 1 ， use 
    pub const FIELDS: [&'static str; 3] = ["identity", "stable", "dynamic"];

    /// 1 （ [`Self::FIELDS`]； not to empty， and 「 this no 」）
    pub fn get(&self, field: &str) -> &str {
        match field {
            "identity" => &self.identity,
            "stable" => &self.stable,
            "dynamic" => &self.dynamic,
            _ => "",
        }
    }

    /// 1 （ [`Self::FIELDS`]）
    pub fn set(&mut self, field: &str, value: String) {
        match field {
            "identity" => self.identity = value,
            "stable" => self.stable = value,
            "dynamic" => self.dynamic = value,
            _ => {}
        }
    }

    /// empty（= no has ，config not this this 1 empty）
    pub fn is_empty(&self) -> bool {
        Self::FIELDS.iter().all(|field| self.get(field).trim().is_empty())
    }

    /// and ： this 1 is empty use `fallback` that 1 。
    ///
    /// 「 use 1 」 is （e.g. ）， and must is 
    /// —— will use as 1 that 1 。
    pub fn or(&self, fallback: &GatewayBlocks) -> GatewayBlocks {
        let pick = |field: &str| {
            let mine = self.get(field);
            if mine.trim().is_empty() {
                fallback.get(field).to_string()
            } else {
                mine.to_string()
            }
        };
        GatewayBlocks {
            identity: pick("identity"),
            stable: pick("stable"),
            dynamic: pick("dynamic"),
        }
    }
}

/// 1 request （default + ** use **）。
///
/// use not is ： by `upstream::forward` from config 1 ，
/// `ProviderContext` to —— can can has ，request 1 
/// （`config::current()` already config， that in 
/// ctx ， this to ）。
///
/// ── Why by ──────────────────────────────────────────
/// 「 use 」 is ** by up ** not （ `config::KEY_PROMPT_PROVIDERS`），
/// （ / / ） is 1 （[`PromptChoice::apply`]）——
/// this 「 1 」 ， 1 not 。`provider_id` in 
/// `payload::send_body` has ， that is 「 1 will before 」 when 。
#[derive(Clone, Copy, Debug)]
pub struct PromptPlan<'a> {
    /// config use that 1 （ for `promptMode` / `promptFile`）
    pub default: PromptChoice<'a>,
    /// （config；empty = all use default）
    providers: &'a std::collections::BTreeMap<String, crate::server::config::ProviderPrompt>,
}

impl<'a> PromptPlan<'a> {
    /// 1 request （** 1 **：`config::RuntimeConfig::prompt_plan`）。
    ///
    /// not for ： has 「 by id 1 」 this 1 use （
    /// [`Self::for_provider`]）， use to will has 1 。
    ///
    /// 「」 ** not in this **： by `config::gateway_prompt_enabled`
    /// （ that is 1 path up ， that ）。 1 ，
    /// not 「 = 」 default， not will has 1 。
    pub fn new(
        default: PromptChoice<'a>,
        providers: &'a std::collections::BTreeMap<String, crate::server::config::ProviderPrompt>,
    ) -> Self {
        Self { default, providers }
    }

    /// this 1 need use that 1 （ no `default`）。
    ///
    /// return** use ** not is ：， 1 etc 
    /// 「 use 」 this 。
    pub fn for_provider(&self, provider_id: &str) -> PromptChoice<'_> {
        match self.providers.get(provider_id) {
            Some(entry) => PromptChoice {
                mode: entry.mode,
                text: &entry.text,
                source: entry.source,
            },
            None => self.default,
        }
    }
}

/// 1 （ + use ）。
///
/// default and 1 this after ，（[`Self::apply`]、）
/// has 1 ——「 this 、 1 that 」 this from up not 。
#[derive(Clone, Copy, Debug)]
pub struct PromptChoice<'a> {
    pub mode: PromptMode,
    /// `custom` / `append` need use （`passthrough` down as empty）
    pub text: &'a str,
    /// this （default / file / none）—— and set need can 
    /// 「 this use to is 1 」
    pub source: PromptSource,
}

impl<'a> PromptChoice<'a> {
    /// this use ： can use in ，otherwise use 
    pub fn text_for(&self, degraded: bool) -> &str {
        if degraded && self.mode.degradable() {
            DEGRADED_PROMPT
        } else {
            self.text
        }
    }

    /// before to body up ：`Some()` = ，`None` = ** not **
    /// （`passthrough` and —— defaultpath， use use client）。
    ///
    /// `degraded` is **request** is already （request when already ， or 
    /// request after by ， `upstream::provider_loop`）。
    pub fn apply(&self, body: &Value, degraded: bool) -> Option<Value> {
        let text = self.text_for(degraded);
        if text.is_empty() {
            return None;
        }
        match self.mode {
            // and ： no has need （ up text as empty already return，
            // this 1 ， up 「passthrough is not 」）
            PromptMode::Passthrough => {
                if degraded {
                    rewrite(body, text)
                } else {
                    None
                }
            }
            // ：append as （module header）
            PromptMode::Append if !degraded => append(body, text),
            PromptMode::Custom | PromptMode::Append => rewrite(body, text),
        }
    }
}

/// ：delete messages **all** `system` / `developer` ， 1 
/// （ for Go `prompt.Rewrite`）。
///
/// none `messages` （ or not is ）→ 1 `messages: [ system]`，
/// its —— and ： path up 「 not failure」，
/// when need 1 can body。
///
/// and Go 1 ：Go in JSON failure when return body； to already is 
/// `Value`， no has this 1 。
pub fn rewrite(body: &Value, text: &str) -> Option<Value> {
    if text.is_empty() {
        return None;
    }
    let mut next = body.clone();
    let object = next.as_object_mut()?;
    let gateway = gateway_message(text);
    match object.get_mut("messages").and_then(Value::as_array_mut) {
        Some(messages) => {
            // ， system/developer —— and 
            // in 1 （ is this ）
            let mut kept: Vec<Value> = Vec::with_capacity(messages.len() + 1);
            kept.push(gateway);
            kept.extend(
                messages
                    .iter()
                    .filter(|message| !is_system_role(message))
                    .cloned(),
            );
            *messages = kept;
        }
        None => {
            object.insert("messages".to_string(), Value::Array(vec![gateway]));
        }
    }
    Some(next)
}

/// ： in **** `system` / `developer` after 1 ，
/// has not （ for Go `prompt.Append`）。
///
/// 「」 and [`rewrite`] delete** 1 **： from `messages[0]`
/// to after need is system/developer ， to 1 system/developer
/// （ for 、none role ）。
pub fn append(body: &Value, text: &str) -> Option<Value> {
    if text.is_empty() {
        return None;
    }
    let mut next = body.clone();
    let object = next.as_object_mut()?;
    let gateway = gateway_message(text);
    match object.get_mut("messages").and_then(Value::as_array_mut) {
        Some(messages) => {
            let insert_at = messages
                .iter()
                .take_while(|message| is_system_role(message))
                .count();
            messages.insert(insert_at, gateway);
        }
        None => {
            object.insert("messages".to_string(), Value::Array(vec![gateway]));
        }
    }
    Some(next)
}

/// 。
///
/// use `system` not is `developer`： up role no has `developer`
/// （ `providers::workbuddy::normalize` 1 ）， `developer` etc 
/// 1 1 + 。 is 1 by 。
fn gateway_message(text: &str) -> Value {
    json!({ "role": "system", "content": text })
}

/// this is system ：**** etc `system` or `developer`。
///
/// （ not is before /）： its （user / assistant / tool / anyunknown）
/// 1 not —— this is 「 has not 」 ， 1 will use 。
fn is_system_role(message: &Value) -> bool {
    message
        .get("role")
        .and_then(Value::as_str)
        .map(|role| role == "system" || role == "developer")
        .unwrap_or(false)
}
