use serde_json::json;

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
    fn process(&self, input: &serde_json::Value) -> Result<Vec<serde_json::Value>, PluginError> {
        let mut updated_obj = input.clone();
        let updated = updated_obj.as_object_mut().ok_or_else(|| {
            PluginError::InternalError(format!(
                "expected input JSON to be an object, found {}",
                input
            ))
        })?;
        updated.insert(self.key.clone(), self.value.clone());
        Ok(vec![json!(updated)])
    }
}
