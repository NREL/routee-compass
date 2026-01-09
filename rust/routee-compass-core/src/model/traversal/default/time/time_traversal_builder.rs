use super::TimeTraversalModel;
use crate::model::traversal::{
    default::time::TimeTraversalConfig, TraversalModelBuilder, TraversalModelError,
    TraversalModelService,
};
use std::sync::Arc;

pub struct TimeTraversalBuilder {}

impl TraversalModelBuilder for TimeTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let config: TimeTraversalConfig =
            serde_json::from_value(parameters.clone()).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failed to read time traversal model configuration: {e}"
                ))
            })?;
        // Note: indices are resolved during the build() call on the service
        let model = TimeTraversalModel::new(
            config, 0,    // dummy value, will be set in build()
            0,    // dummy value, will be set in build()
            0,    // dummy value, will be set in build()
            None, // dummy value, will be set in build()
        );
        let service = Arc::new(model);
        Ok(service)
    }
}
