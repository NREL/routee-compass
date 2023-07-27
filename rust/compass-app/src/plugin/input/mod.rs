use super::plugin_error::PluginError;

pub mod query;
pub mod rtree;

type InputPlugin = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, PluginError>>;
