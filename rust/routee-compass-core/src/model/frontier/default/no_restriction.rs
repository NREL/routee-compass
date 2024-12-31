use crate::model::{
    frontier::{frontier_model_error::FrontierModelError, FrontierModel, FrontierModelService},
    state::StateModel,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct NoRestriction {}

impl FrontierModel for NoRestriction {
    fn valid_frontier(
        &self,
        _edge: &crate::model::network::Edge,
        _state: &[crate::model::state::StateVariable],
        _tree: &std::collections::HashMap<
            crate::model::network::VertexId,
            crate::algorithm::search::SearchTreeBranch,
        >,
        _direction: &crate::algorithm::search::Direction,
        _state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        Ok(true)
    }

    fn valid_edge(&self, _edge: &crate::model::network::Edge) -> Result<bool, FrontierModelError> {
        Ok(true)
    }
}

impl FrontierModelService for NoRestriction {
    fn build(
        &self,
        _query: &serde_json::Value,
        _state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError> {
        Ok(Arc::new(self.clone()))
    }
}
