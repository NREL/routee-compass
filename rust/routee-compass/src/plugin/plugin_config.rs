use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginConfig {
    pub input_plugins: Vec<Value>,
    pub output_plugins: Vec<Value>,
}
