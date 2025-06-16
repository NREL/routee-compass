use super::ElevationTraversalModel;
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct ElevationTraversalBuilder {}

impl TraversalModelBuilder for ElevationTraversalBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let service = Arc::new(ElevationTraversalModel {});
        Ok(service)
    }
}
