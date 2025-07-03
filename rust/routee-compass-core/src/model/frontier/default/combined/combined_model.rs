use crate::model::{
    frontier::{FrontierModel, FrontierModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::sync::Arc;

pub struct CombinedFrontierModel {
    pub inner_models: Vec<Arc<dyn FrontierModel>>,
}

impl FrontierModel for CombinedFrontierModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        previous_edge: Option<&Edge>,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<bool, FrontierModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for frontier_model in self.inner_models.iter() {
            if !frontier_model.valid_frontier(edge, previous_edge, state, state_model)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FrontierModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for frontier_model in self.inner_models.iter() {
            if !frontier_model.valid_edge(edge)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
