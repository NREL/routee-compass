use super::function::{EdgeEdgeCostFunction, TerminateSearchFunction, ValidFrontierFunction};
use crate::model::traversal::state::state_variable::StateVar;

pub struct EdgeEdgeCostFunctionConfig<'a> {
    pub cost_fn: &'a EdgeEdgeCostFunction,
    pub valid_fn: &'a Option<ValidFrontierFunction>,
    pub terminate_fn: &'a Option<TerminateSearchFunction>,
    pub init_state: &'a Vec<StateVar>,
}
