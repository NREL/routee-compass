use super::plugin::GridSearchPlugin;
use crate::{
    app::compass::CompassConfigurationError,
    plugin::input::{InputPlugin, InputPluginBuilder},
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
