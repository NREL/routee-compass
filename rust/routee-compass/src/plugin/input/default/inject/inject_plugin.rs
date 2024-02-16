use crate::plugin::{input::input_plugin::InputPlugin, plugin_error::PluginError};

pub struct InjectInputPlugin {
    key: String,
    value: serde_json::Value,
}

impl InjectInputPlugin {
    pub fn new(key: String, value: serde_json::Value) -> InjectInputPlugin {
        InjectInputPlugin { key, value }
    }
}

impl InputPlugin for InjectInputPlugin {
    fn process(&self, input: &mut serde_json::Value) -> Result<(), PluginError> {
        input[self.key.clone()] = self.value.clone();
        Ok(())
    }
}
