use super::TimeTraversalModel;
use crate::model::traversal::{TraversalModelBuilder, TraversalModelError, TraversalModelService};
use std::sync::Arc;

pub struct TimeTraversalBuilder {}

impl TraversalModelBuilder for TimeTraversalBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let service = Arc::new(TimeTraversalModel {});
        Ok(service)
    }
}
