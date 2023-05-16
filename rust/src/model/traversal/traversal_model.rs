use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::{cost::cost::Cost, property::edge::Edge},
};

use super::traversal_error::TraversalError;

pub trait TraversalModel: Sync + Send {
    type State: Sync + Send + Eq + Copy + Clone;

    fn initial_state(&self) -> Result<Self::State, TraversalError>;

    fn traversal_cost(
        &self,
        e: &Edge,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError>;

    fn access_cost(
        &self,
        src: &Edge,
        dst: &Edge,
        state: &Self::State,
    ) -> Result<(Cost, Self::State), TraversalError>;

    // fn update(&self, s: Self::State, c: Cost) -> Result<Self::State, TraversalError>;

    fn valid_frontier(&self, frontier: &EdgeFrontier<Self::State>) -> Result<bool, TraversalError>;

    fn terminate_search(
        &self,
        frontier: &EdgeFrontier<Self::State>,
    ) -> Result<bool, TraversalError>;
}
