use crate::model::traversal::Vec<StateVar>;
use crate::model::unit::Cost;

pub struct AccessResult {
    pub cost: Cost,
    pub updated_state: Option<Vec<StateVar>>,
}

impl AccessResult {
    pub fn no_cost() -> AccessResult {
        AccessResult {
            cost: Cost::ZERO,
            updated_state: None,
        }
    }
}
