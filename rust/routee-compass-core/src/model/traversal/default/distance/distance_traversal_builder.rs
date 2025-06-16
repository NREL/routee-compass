use super::DistanceTraversalService;
use crate::model::traversal::TraversalModelBuilder;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct DistanceTraversalBuilder {}

impl TraversalModelBuilder for DistanceTraversalBuilder {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let m: Arc<dyn TraversalModelService> = Arc::new(DistanceTraversalService {});
        Ok(m)
    }
}
