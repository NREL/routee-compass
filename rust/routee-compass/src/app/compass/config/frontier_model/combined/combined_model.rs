use routee_compass_core::model::{
    frontier::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError},
    graph::Edge,
    state::state_model::StateModel,
    traversal::state::state_variable::StateVar,
};
use std::sync::Arc;

pub struct CombinedFrontierModel {
    pub inner_models: Vec<Arc<dyn FrontierModel>>,
}

impl FrontierModel for CombinedFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        state: &[StateVar],
        previous_edge: Option<&Edge>,
        state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for frontier_model in self.inner_models.iter() {
            if !frontier_model.valid_frontier(edge, state, previous_edge, state_model)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
