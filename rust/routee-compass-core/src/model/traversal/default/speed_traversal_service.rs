use super::speed_traversal_model::SpeedTraversalModel;
use crate::model::traversal::{
    traversal_model::TraversalModel, traversal_model_error::TraversalModelError,
    traversal_model_service::TraversalModelService,
};
use std::sync::Arc;

pub struct SpeedLookupService {
    pub m: Arc<SpeedTraversalModel>,
}

impl TraversalModelService for SpeedLookupService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(self.m.clone())
    }
}
