use crate::model::{
    constraint::{ConstraintModel, ConstraintModelError},
    network::Edge,
    state::{StateModel, StateVariable},
};
use std::sync::Arc;

pub struct CombinedConstraintModel {
    pub inner_models: Vec<Arc<dyn ConstraintModel>>,
}

impl ConstraintModel for CombinedConstraintModel {
    fn valid_frontier(
        &self,
        edge: &Edge,
        previous_edge: Option<&Edge>,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<bool, ConstraintModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for constraint_model in self.inner_models.iter() {
            if !constraint_model.valid_frontier(edge, previous_edge, state, state_model)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn valid_edge(&self, edge: &Edge) -> Result<bool, ConstraintModelError> {
        // If any of the inner models return an invalid frontier, it invalidates the whole set and we
        // return an early false. We only return true if all the frontiers are valid.
        for constraint_model in self.inner_models.iter() {
            if !constraint_model.valid_edge(edge)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
