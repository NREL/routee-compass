use crate::model::cost::Cost;
use crate::model::traversal::state::traversal_state::TraversalState;

/// The state of a search after completing an edge traversal, along
/// with the cost of traversing that edge.
pub struct TraversalResult {
    pub total_cost: Cost,
    pub updated_state: TraversalState,
}
