use super::DistanceTraversalService;
use crate::config::CompassConfigurationField;
use crate::model::traversal::TraversalModelBuilder;
use crate::model::traversal::TraversalModelError;
use crate::model::traversal::TraversalModelService;
use std::sync::Arc;

pub struct DistanceTraversalBuilder {}

impl TraversalModelBuilder for DistanceTraversalBuilder {
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModelService>, TraversalModelError> {
        let traversal_key = CompassConfigurationField::Traversal.to_string();
        let m: Arc<dyn TraversalModelService> = Arc::new(DistanceTraversalService {});
        Ok(m)
    }
}
