use crate::model::cost::cost::Cost;
use crate::model::traversal::state::search_state::SearchState;

pub struct TraversalResult {
    pub total_cost: Cost,
    pub updated_state: SearchState,
}
