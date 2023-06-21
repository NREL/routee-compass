use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::{
        cost::cost::Cost,
        property::{edge::Edge, vertex::Vertex},
    },
};

use super::{traversal_error::TraversalError, traversal_model::TraversalModel};

pub struct FreeFlowTraversalModel;
impl TraversalModel for FreeFlowTraversalModel {
    type State = i64;
    fn initial_state(&self) -> Result<Self::State, TraversalError> {
        Ok(0)
    }

    fn traversal_cost(
        &self,
        _src: &Vertex,
        edge: &Edge,
        _dst: &Vertex,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError> {
        let c = edge
            .distance_centimeters
            .travel_time_millis(&edge.free_flow_speed_cps)
            .0;
        Ok((Cost(c), state + c))
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

    fn valid_frontier(&self, frontier: &EdgeFrontier<Self::State>) -> Result<bool, TraversalError> {
        Ok(true)
    }

    fn terminate_search(
        &self,
        frontier: &EdgeFrontier<Self::State>,
    ) -> Result<bool, TraversalError> {
        Ok(false)
    }
}
