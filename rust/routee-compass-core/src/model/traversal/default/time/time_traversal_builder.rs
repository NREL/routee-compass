use super::{time_configuration::TimeConfiguration, TimeTraversalModel};
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct TimeTraversalBuilder {}

impl TraversalModelBuilder for TimeTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let config: TimeConfiguration =
            serde_json::from_value(parameters.clone()).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failed to build time traversal model from config: {}",
                    e
                ))
            })?;
        let service = Arc::new(TimeTraversalModel::from(&config));
        Ok(service)
    }
}
