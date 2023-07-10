use super::cost_function::EdgeCostFunction;
use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::traversal::cost_function::edge_cost_function_config::EdgeCostFunctionConfig;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::traversal::traversal_error::TraversalError;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};

/// implements a free-flow traversal cost function.
/// for each edge, we compute the travel time from distance and free flow speed.
/// this is added to the state vector and returned directly as the cost as well.
pub fn free_flow_cost_function(
    src: &Vertex,
    edge: &Edge,
    dst: &Vertex,
    state: &StateVector,
) -> Result<(Cost, StateVector), TraversalError> {
    let c = edge
        .distance_centimeters
        .travel_time_millis(&edge.free_flow_speed_cps)
        .0;
    let mut s = state.to_vec();
    s[0] = s[0] + StateVar(c as f64);
    Ok((Cost(c), s))
}

// pub FREE_FLOW_COST_FUNCTION: EdgeCostFunction = Box::new(free_flow_cost_function);
pub struct FreeFlowCostFunction;

// pub const INITIAL_STATE: StateVector = vec![StateVar(0.0)];

// pub static FREE_FLOW_COST_CONFIG: EdgeCostFunctionConfig = {
//     let ffcf = move |src: &Vertex, edge: &Edge, dst: &Vertex, state: &StateVector| {
//         let c = edge
//             .distance_centimeters
//             .travel_time_millis(&edge.free_flow_speed_cps)
//             .0;
//         let mut s = state.to_vec();
//         s[0] = s[0] + StateVar(c as f64);
//         Ok((Cost(c), s))
//     };
//     let ffcf: EdgeCostFunction = Box::new(ffcf);
//     EdgeCostFunctionConfig {
//         cost_fn: ffcf,
//         init_state: vec![StateVar(0.0)],
//         valid_fn: None,
//         terminate_fn: None,
//     }
// };

// impl<'a> FreeFlowCostFunction {
//     pub fn get_config() -> EdgeCostFunctionConfig<'a> {
//         // let ffcf = |src: &Vertex, edge: &Edge, dst: &Vertex, state: &StateVector| {
//         //     let c = edge
//         //         .distance_centimeters
//         //         .travel_time_millis(&edge.free_flow_speed_cps)
//         //         .0;
//         //     let mut s = state.to_vec();
//         //     s[0] = s[0] + StateVar(c as f64);
//         //     Ok((Cost(c), s))
//         // };
//         let ffcf: EdgeCostFunction = Box::new(free_flow_cost_function);
//         let result: EdgeCostFunctionConfig<'a> = EdgeCostFunctionConfig {
//             cost_fn: &ffcf,
//             init_state: &vec![StateVar(0.0)],
//             valid_fn: None,
//             terminate_fn: None,
//         };
//         result
//     }
// }
