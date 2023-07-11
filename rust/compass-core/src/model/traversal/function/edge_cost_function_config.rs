use super::function::{EdgeCostFunction, TerminateSearchFunction, ValidFrontierFunction};
use crate::model::traversal::state::state_variable::StateVar;

pub struct EdgeCostFunctionConfig<'a> {
    pub cost_fn: &'a EdgeCostFunction,
    pub init_state: &'a Vec<StateVar>,
    pub valid_fn: Option<&'a ValidFrontierFunction>,
    pub terminate_fn: Option<&'a TerminateSearchFunction>,
}

impl<'a> EdgeCostFunctionConfig<'a> {
    pub fn new(
        cost_fn: &'a EdgeCostFunction,
        init_state: &'a Vec<StateVar>,
    ) -> EdgeCostFunctionConfig<'a> {
        return EdgeCostFunctionConfig {
            cost_fn,
            init_state,
            valid_fn: None,
            terminate_fn: None,
        };
    }
}
