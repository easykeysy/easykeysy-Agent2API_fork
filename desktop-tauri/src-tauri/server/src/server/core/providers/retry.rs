//! optional failure； old returnunknown when use 。

use serde::{Deserialize, Serialize};

pub const RETRY_DECISION_METHOD: &str = "retry.decision";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RetryDecision {
    #[default]
    Default,
    Retry { delay_ms: u64, reason: String },
    Stop { reason: String },
}
