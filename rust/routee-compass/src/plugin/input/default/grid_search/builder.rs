use crate::{
    app::compass::config::{
        builders::InputPluginBuilder, compass_configuration_error::CompassConfigurationError,
    },
    plugin::input::input_plugin::InputPlugin,
};

use super::plugin::GridSearchPlugin;

pub struct GridSearchBuilder {}

impl InputPluginBuilder for GridSearchBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Box<dyn InputPlugin>, CompassConfigurationError> {
        Ok(Box::new(GridSearchPlugin {}))
    }
}
