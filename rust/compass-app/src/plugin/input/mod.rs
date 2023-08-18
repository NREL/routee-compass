use super::plugin_error::PluginError;

pub mod input_field;
pub mod input_json_extensions;
pub mod input_plugin_ops;
pub mod rtree;

pub type InputPlugin = Box<dyn Fn(&serde_json::Value) -> Result<serde_json::Value, PluginError>>;
