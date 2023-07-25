use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::traversal::function::function::EdgeCostFunction;
use crate::model::traversal::state::search_state::StateVector;
use crate::model::{cost::cost::Cost, traversal::state::state_variable::StateVar};

/// implements a free-flow traversal cost function.
/// for each edge, we compute the travel time from distance and free flow speed.
/// this is added to the state vector and returned directly as the cost as well.
pub fn distance_cost_function() -> EdgeCostFunction {
    let dcf = Box::new(
        move |_src: &Vertex, edge: &Edge, _dst: &Vertex, state: &StateVector| {
            let c: ordered_float::OrderedFloat<f64> =
                ordered_float::OrderedFloat(edge.distance.value);
            let mut s = state.to_vec();
            s[0] = s[0] + StateVar(*c);
            Ok((Cost(c), s))
        },
    );
    return dcf;
}

/// starting state for a free flow search
pub fn initial_distance_state() -> StateVector {
    vec![StateVar(0.0)]
}
