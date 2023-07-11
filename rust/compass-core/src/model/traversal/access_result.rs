use crate::model::cost::cost::Cost;
use crate::model::traversal::state::search_state::SearchState;

pub struct AccessResult {
    pub total_cost: Cost,
    pub cost_vector: Vec<Cost>,
    pub updated_state: SearchState,
}

impl AccessResult {
    pub fn no_cost(prev_state: &SearchState) -> AccessResult {
        return AccessResult {
            total_cost: Cost::ZERO,
            cost_vector: vec![],
            updated_state: prev_state.to_vec(),
        };
    }
}
