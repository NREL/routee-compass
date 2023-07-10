use super::cost_function::EdgeCostFunction;
use crate::model::traversal::cost_function::edge_cost_function_config::EdgeCostFunctionConfig;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};

/// implements a free-flow traversal cost function.
/// for each edge, we compute the travel time from distance and free flow speed.
/// this is added to the state vector and returned directly as the cost as well.
pub const FREE_FLOW_COST_FUNCTION: EdgeCostFunction = Box::new(|_, edge, _, state| {
    let c = edge
        .distance_centimeters
        .travel_time_millis(&edge.free_flow_speed_cps)
        .0;
    let mut s = state.to_vec();
    s[0] = s[0] + StateVar(c as f64);
    Ok((Cost(c), s))
});

pub const INITIAL_STATE: StateVector = vec![StateVar(0.0)];

/// helper configuration for installing free flow traversal models
pub const FREE_FLOW_COST_CONFIG: EdgeCostFunctionConfig = EdgeCostFunctionConfig {
    cost_fn: &FREE_FLOW_COST_FUNCTION,
    init_state: &INITIAL_STATE,
    valid_fn: None,
    terminate_fn: None,
};
