use crate::model::traversal::state::traversal_state::TraversalState;
use crate::model::unit::Cost;

pub struct AccessResult {
    pub cost: Cost,
    pub updated_state: Option<TraversalState>,
}

impl AccessResult {
    pub fn no_cost() -> AccessResult {
        AccessResult {
            cost: Cost::ZERO,
            updated_state: None,
        }
    }
}
