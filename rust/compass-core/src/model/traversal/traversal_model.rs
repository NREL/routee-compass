use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::{
        cost::cost::Cost,
        property::{edge::Edge, vertex::Vertex},
    },
};

use super::traversal_error::TraversalError;

pub trait TraversalModel: Sync + Send {
    type State: Sync + Send + Clone;

    fn initial_state(&self) -> Result<Self::State, TraversalError>;

    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError>;

    fn access_cost(
        &self,
        v1: &Vertex,
        src: &Edge,
        v2: &Vertex,
        dst: &Edge,
        v3: &Vertex,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError>;

    fn valid_frontier(&self, frontier: &EdgeFrontier<Self::State>) -> Result<bool, TraversalError>;

    fn terminate_search(
        &self,
        frontier: &EdgeFrontier<Self::State>,
    ) -> Result<bool, TraversalError>;
}
