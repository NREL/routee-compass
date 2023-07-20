use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::traversal::function::function::EdgeCostFunction;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};

use uom::si;

/// implements a free-flow traversal cost function.
/// for each edge, we compute the travel time from distance and free flow speed.
/// this is added to the state vector and returned directly as the cost as well.
pub fn free_flow_cost_function() -> EdgeCostFunction {
    let ffcf = Box::new(
        move |_src: &Vertex, edge: &Edge, _dst: &Vertex, state: &StateVector| {
            let time = edge.distance / edge.free_flow_speed;
            let seconds: f64 = time.get::<si::time::second>().into();
            let mut s = state.to_vec();
            s[0] = s[0] + StateVar(seconds);
            Ok((Cost::from_f64(seconds), s))
        },
    );
    return ffcf;
}

/// starting state for a free flow search
pub fn initial_free_flow_state() -> StateVector {
    vec![StateVar(0.0)]
}
