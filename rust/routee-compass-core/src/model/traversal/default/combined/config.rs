use serde::{Deserialize, Serialize};
use serde_json::Value;

/// configures a traversal model that combines many different traversal models.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CombinedTraversalConfig {
    pub models: Vec<Value>,
    pub ignore_missing: bool
}