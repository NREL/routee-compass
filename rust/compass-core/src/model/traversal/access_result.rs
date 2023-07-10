use super::state::state_variable::StateVar;
use crate::model::cost::cost::Cost;

pub struct AccessResult {
    pub total_cost: Cost,
    pub cost_vector: Vec<Cost>,
    pub updated_state: Vec<Vec<StateVar>>,
}
