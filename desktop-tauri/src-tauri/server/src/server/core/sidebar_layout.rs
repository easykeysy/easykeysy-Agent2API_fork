//! （ / / ） 。
//!
//! ── Why from localStorage ────────────────────────────
//! is before （ and / / ）， localStorage。
//! that before is 「 this ， not need need 」——
//! has （ `tauri://localhost` and 
//! `http://127.0.0.1:<port>`），** origin not 、localStorage **，
//! is in WebUI in client old ， use to is 「set no 」。
//!
//! after is ** 1 **， 1 ；localStorage 
//! （ `ui/sidebar-layout.js` module header —— that Why not can 
//! ）。
//!
//! ── Why after not ────────────────────────────────
//! （ in 、 not can 、 need has 1 、
//! new ） by before `ui/sidebar-layout.js` `MENUS` ——
//! that ，** is has **
//! （`plugin:<pack>:<page>`， `core::plugin_host`）。 after 1 then 
//! will 1 ： before 1 ， after 。
//!
//! ** not JSON **： need is for ，
//! ，（`normalizeLayout`） in before 。 this and 
//! `agentProfiles` to not （ that after `Profile` ），Reason is 
//! by **** has ， not is by has 。
//!
//! ── ────────────────────────────────────────────────
//! `kv` 1 ， JSON （ and `proxySettings` / `agentProfiles`
//! ）。 not add， not schema 。 in 
//! `db::schema::RESERVED_KV_KEYS` —— will use 1 API Key 
//! 「config already 」。

use serde_json::Value;

use crate::server::access;

/// `kv` （， `db::schema::RESERVED_KV_KEYS`）
pub const KV_KEY: &str = "sidebarLayout";

/// 。`None` = no （ before default）。
///
/// failurereturn `None`： that not when ， before default 1 ——
/// will `normalizeLayout` unknown， 1 。
///
/// empty for by 「 no 」：`save(None)`（default） is `{}`，
/// if in this return `Some({})`， will `{layout: {}}` not is 
/// `{layout: null}` —— before can （`normalizeLayout({})` to 
/// `defaultLayout()`）， this has 1 。
pub fn load() -> Option<Value> {
    let text = access::kv_get(KV_KEY)?;
    let value = serde_json::from_str::<Value>(&text).ok()?;
    // not is for = not is （ / / null）—— no 
    let object = value.as_object()?;
    (!object.is_empty()).then_some(value)
}

/// 。`None` = （default）。
///
/// and `core::agents::save_client` 1 to ：（ is ，
/// will 1 update）。success —— not 
/// `config::update`， that path not to 。
pub fn save(layout: Option<&Value>) -> Result<(), String> {
    match layout {
        Some(value) => {
            if !value.is_object() {
                return Err("侧边栏布局必须是 JSON 对象".to_string());
            }
            let text = serde_json::to_string(value)
                .map_err(|error| format!("布局序列化失败: {error}"))?;
            access::kv_put(KV_KEY, &text);
        }
        // ：empty for not is —— `kv_put` no has delete，empty for 
        // `normalizeLayout` to default（`groups` as empty → `defaultLayout()`），
        // and 「 no 」 1 ， and not as new 1 DELETE path。
        None => access::kv_put(KV_KEY, "{}"),
    }
    // `sidebarLayout` in （ `cloud_sync::snapshot` 
    // `SYNCED_RESERVED_KEYS`）， not `config::update`，。
    crate::server::core::cloud_sync::auto_sync::notify_change();
    Ok(())
}
