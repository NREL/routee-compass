use crate::model::cost::cost::Cost;
use crate::model::traversal::state::traversal_state::TraversalState;

pub struct TraversalResult {
    pub total_cost: Cost,
    pub updated_state: TraversalState,
}
