use crate::plugin::plugin_error::PluginError;

pub trait InputPlugin: Send + Sync {
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError>;
}
