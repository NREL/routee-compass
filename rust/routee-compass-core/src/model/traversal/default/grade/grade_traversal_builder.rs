use super::{GradeConfiguration, GradeTraversalEngine, GradeTraversalService};
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct GradeTraversalBuilder {}

impl TraversalModelBuilder for GradeTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let config: GradeConfiguration =
            serde_json::from_value(parameters.clone()).map_err(|e| {
                TraversalModelError::BuildError(format!("failed to read grade configuration: {e}"))
            })?;
        let engine = Arc::new(GradeTraversalEngine::new(&config)?);
        let service = Arc::new(GradeTraversalService::new(engine));
        Ok(service)
    }
}
