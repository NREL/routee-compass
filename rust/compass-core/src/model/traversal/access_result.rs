use crate::model::cost::cost::Cost;
use crate::model::traversal::state::traversal_state::TraversalState;

pub enum AccessCost {
    NoCost,
    Cost(Cost),
}
pub struct AccessResult {
    pub cost: AccessCost,
    pub cost_vector: Option<Vec<Cost>>,
    pub updated_state: Option<TraversalState>,
}

impl AccessResult {
    pub fn no_cost() -> AccessResult {
        return AccessResult {
            cost: AccessCost::NoCost,
            cost_vector: None,
            updated_state: None,
        };
    }
}
