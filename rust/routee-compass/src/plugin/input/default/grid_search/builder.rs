use std::sync::Arc;

use crate::{
    app::compass::config::{
        builder::input_plugin_builder::InputPluginBuilder,
        compass_configuration_error::CompassConfigurationError,
    },
    plugin::input::input_plugin::InputPlugin,
};

use super::plugin::GridSearchPlugin;

pub struct GridSearchBuilder {}

impl InputPluginBuilder for GridSearchBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(GridSearchPlugin {}))
    }
}
