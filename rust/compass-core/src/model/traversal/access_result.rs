use crate::model::cost::cost::Cost;
use crate::model::traversal::state::traversal_state::TraversalState;

pub struct AccessResult {
    pub total_cost: Cost,
    pub cost_vector: Vec<Cost>,
    pub updated_state: TraversalState,
}

impl AccessResult {
    pub fn no_cost(prev_state: &TraversalState) -> AccessResult {
        return AccessResult {
            total_cost: Cost::ZERO,
            cost_vector: vec![],
            updated_state: prev_state.to_vec(),
        };
    }
}
