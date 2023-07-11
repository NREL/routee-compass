use super::cost_function::{EdgeEdgeCostFunction, TerminateSearchFunction, ValidFrontierFunction};
use crate::model::traversal::state::state_variable::StateVar;

pub struct EdgeEdgeCostFunctionConfig<'a> {
    pub cost_fn: &'a EdgeEdgeCostFunction,
    pub init_state: &'a Vec<StateVar>,
    pub valid_fn: Option<&'a ValidFrontierFunction>,
    pub terminate_fn: Option<&'a TerminateSearchFunction>,
}

impl<'a> EdgeEdgeCostFunctionConfig<'a> {
    pub fn new(
        cost_fn: &'a EdgeEdgeCostFunction,
        init_state: &'a Vec<StateVar>,
    ) -> EdgeEdgeCostFunctionConfig<'a> {
        return EdgeEdgeCostFunctionConfig {
            cost_fn,
            init_state,
            valid_fn: None,
            terminate_fn: None,
        };
    }
}
