use crate::model::{
    filter::{FilterModel, FilterModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::sync::Arc;

pub struct CombinedFilterModel {
    pub inner_models: Vec<Arc<dyn FilterModel>>,
}

impl FilterModel for CombinedFilterModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        previous_edge: Option<&Edge>,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<bool, FilterModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for filter_model in self.inner_models.iter() {
            if !filter_model.valid_frontier(edge, previous_edge, state, state_model)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, FilterModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for filter_model in self.inner_models.iter() {
            if !filter_model.valid_edge(edge)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
