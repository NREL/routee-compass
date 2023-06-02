use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::{
        cost::cost::Cost,
        property::{edge::Edge, vertex::Vertex},
        traversal::{traversal_error::TraversalError, traversal_model::TraversalModel},
        units::seconds::Seconds,
    },
};

struct TravelTimeTraversal;

impl TraversalModel for TravelTimeTraversal {
    type State = Seconds;

    fn initial_state(&self) -> Result<Self::State, TraversalError> {
        Ok(Seconds(0))
    }

    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError> {
        let trave_time_seconds = edge.free_flow_travel_time_seconds();

        Ok((Cost(trave_time_seconds.0), *state + trave_time_seconds))
    }

    fn access_cost(
        &self,
        _v1: &Vertex,
        _src: &Edge,
        _v2: &Vertex,
        _dst: &Edge,
        _v3: &Vertex,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError> {
        Ok((Cost::ZERO, state.clone()))
    }

    fn valid_frontier(&self, _frontier: &EdgeFrontier<Self::State>) -> Result<bool, TraversalError> {
        Ok(true)
    }

    fn terminate_search(
        &self,
        _frontier: &EdgeFrontier<Self::State>,
    ) -> Result<bool, TraversalError> {
        Ok(false)
    }
}
