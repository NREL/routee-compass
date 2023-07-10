use super::cost_function::{EdgeCostFunction, TerminateSearchFunction, ValidFrontierFunction};
use crate::model::traversal::state::state_variable::StateVar;

pub struct EdgeCostFunctionConfig<'a> {
    pub cost_fn: &'a EdgeCostFunction,
    pub init_state: &'a Vec<StateVar>,
    pub valid_fn: Option<&'a ValidFrontierFunction>,
    pub terminate_fn: Option<&'a TerminateSearchFunction>,
}
