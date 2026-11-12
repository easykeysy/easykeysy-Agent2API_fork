//! ModelScope（） ****（ this 1 ）。
//!
//! ── this fileWhy in ──────────────────────────────────────
//! already to （ / / directory / / to ）。
//! not this ， not has path。
//!
//! ** ** has ： up `userId`（HavanaId）
//! and `accessToken`（`ms-` ） this （ no has `pluginSecret`），
//! need by this —— its is **must**。
//!
//! ── Whymust in this ──────────────────────────────────
//! is 「 JSON `accessToken`」。 this 
//! HTTP response and to ， 1 etc to all can to 
//! 。 new `to_plugin_public_account`（ in `pluginSecret` ，
//! that not will ）。
//!
//! ── ──────────────────────────────────────────────────
//! release is `panic=abort`：file unwrap/expect/panic。

use serde_json::Value;

use super::state::StoredAccount;
use super::store::AccountStore;

impl AccountStore {
    /// ModelScope ****（ HTTP response）：， 4 。
    ///
    /// no has `edition` ： by provider id （ provider、 ），
    /// not need need up 1 。
    pub fn to_modelscope_public_account(&self, record: &StoredAccount) -> Value {
        let mut value = record.to_value();
        if let Some(object) = value.as_object_mut() {
            object.remove("accessToken");
            object.insert("available".to_string(), Value::Bool(record.enabled()));
            object.insert("hasToken".to_string(), Value::Bool(true));
        }
        value
    }
}
