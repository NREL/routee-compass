use super::function::{EdgeCostFunction, TerminateSearchFunction, ValidFrontierFunction};
use crate::model::traversal::state::state_variable::StateVar;

pub struct EdgeCostFunctionConfig<'a> {
    pub cost_fn: &'a EdgeCostFunction,
    pub valid_fn: &'a Option<ValidFrontierFunction>,
    pub terminate_fn: &'a Option<TerminateSearchFunction>,
    pub init_state: &'a Vec<StateVar>,
}
