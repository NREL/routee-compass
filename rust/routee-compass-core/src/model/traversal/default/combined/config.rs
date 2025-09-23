use serde::{Deserialize, Serialize};
use serde_json::Value;

/// configures a traversal model that combines many different traversal models.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CombinedTraversalConfig {
    /// an array of model configurations, each its own argument to a TraversalModelBuilder::build() invocation
    pub models: Vec<Value>,
    /// if true, unaccounted for features will not trigger an error. see combined_ops for more details.
    /// if ignore_missing is None, the default will be false ("do not ignore missing").
    pub ignore_missing: Option<bool>,
}
