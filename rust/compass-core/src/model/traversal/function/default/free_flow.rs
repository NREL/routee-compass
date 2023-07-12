use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::traversal::function::function::EdgeCostFunction;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};

/// implements a free-flow traversal cost function.
/// for each edge, we compute the travel time from distance and free flow speed.
/// this is added to the state vector and returned directly as the cost as well.
pub fn free_flow_cost_function() -> EdgeCostFunction {
    let ffcf = Box::new(
        move |_src: &Vertex, edge: &Edge, _dst: &Vertex, state: &StateVector| {
            let c = edge
                .distance_centimeters
                .travel_time_millis(&edge.free_flow_speed_cps)
                .0;
            let mut s = state.to_vec();
            s[0] = s[0] + StateVar(c as f64);
            Ok((Cost(c), s))
        },
    );
    return ffcf;
}

/// starting state for a free flow search
pub fn initial_free_flow_state() -> StateVector {
    vec![StateVar(0.0)]
}
