use super::{ElevationConfiguration, ElevationTraversalModel};
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct ElevationTraversalBuilder {}

impl TraversalModelBuilder for ElevationTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let config: ElevationConfiguration =
            serde_json::from_value(parameters.clone()).map_err(|e| {
                TraversalModelError::BuildError(format!(
                    "failed to build time traversal model from config: {}",
                    e
                ))
            })?;
        let service = Arc::new(ElevationTraversalModel::new(&config));
        Ok(service)
    }
}
