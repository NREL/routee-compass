use std::sync::Arc;

use crate::{
    app::compass::model::{builders::InputPluginBuilder, CompassConfigurationError},
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
