use super::plugin::GridSearchPlugin;
use routee_compass_core::config::CompassConfigurationError;
use crate::plugin::input::{
    InputPlugin, InputPluginBuilder,
};
use std::sync::Arc;

pub struct GridSearchBuilder {}

impl InputPluginBuilder for GridSearchBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn InputPlugin>, CompassConfigurationError> {
        Ok(Arc::new(GridSearchPlugin {}))
    }
}
