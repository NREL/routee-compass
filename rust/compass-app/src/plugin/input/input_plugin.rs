use crate::plugin::plugin_error::PluginError;

pub trait InputPlugin: Send + Sync {
    fn proccess(&self, input: &serde_json::Value) -> Result<serde_json::Value, PluginError>;
}
