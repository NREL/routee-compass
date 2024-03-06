use crate::model::{
    frontier::{
        frontier_model::FrontierModel, frontier_model_error::FrontierModelError,
        frontier_model_service::FrontierModelService,
    },
    state::state_model::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct NoRestriction {}

impl FrontierModel for NoRestriction {}

impl FrontierModelService for NoRestriction {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        Ok(Arc::new(self.clone()))
    }
}
