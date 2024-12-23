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

impl FrontierModel for NoRestriction {
    fn valid_frontier(
        &self,
        _edge: &crate::model::network::Edge,
        _state: &[crate::model::traversal::state::state_variable::StateVar],
        _tree: &std::collections::HashMap<
            crate::model::network::VertexId,
            crate::algorithm::search::search_tree_branch::SearchTreeBranch,
        >,
        _direction: &crate::algorithm::search::direction::Direction,
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
