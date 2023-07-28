use super::plugin_error::PluginError;

pub mod geometry;
pub mod result;

type OutputPlugin = Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, PluginError>>;