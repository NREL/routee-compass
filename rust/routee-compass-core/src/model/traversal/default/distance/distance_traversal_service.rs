use super::DistanceTraversalModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct DistanceTraversalService {}

impl TraversalModelService for DistanceTraversalService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        let m: Arc<dyn TraversalModel> = Arc::new(DistanceTraversalModel {});
        Ok(m)
    }
}
