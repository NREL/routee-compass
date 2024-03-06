use super::{
    speed_traversal_engine::SpeedTraversalEngine, speed_traversal_model::SpeedTraversalModel,
};
use crate::model::{
    state::state_model::StateModel,
    traversal::{
        traversal_model::TraversalModel, traversal_model_error::TraversalModelError,
        traversal_model_service::TraversalModelService,
    },
};
use std::sync::Arc;

pub struct SpeedLookupService {
    pub e: Arc<SpeedTraversalEngine>,
}

impl TraversalModelService for SpeedLookupService {
    fn build(
        &self,
        _parameters: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError> {
        Ok(Arc::new(SpeedTraversalModel::new(
            self.e.clone(),
            state_model.clone(),
        )))
    }
}
