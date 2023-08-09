use super::access_result::AccessResult;
use super::traversal_model_error::TraversalModelError;
use crate::model::traversal::state::search_state::SearchState;
use crate::model::traversal::traversal_result::TraversalResult;
use crate::{
    algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier,
    model::property::{edge::Edge, vertex::Vertex},
};

pub trait TraversalModel: Send + Sync {
    fn initial_state(&self) -> SearchState;
    fn traversal_cost(
        &self,
        src: &Vertex,
        edge: &Edge,
        dst: &Vertex,
        state: &SearchState,
    ) -> Result<TraversalResult, TraversalModelError>;
    fn access_cost(
        &self,
        v1: &Vertex,
        src: &Edge,
        v2: &Vertex,
        dst: &Edge,
        v3: &Vertex,
        state: &SearchState,
    ) -> Result<AccessResult, TraversalModelError> {
        Ok(AccessResult::no_cost(state))
    }
    fn valid_frontier(&self, frontier: &EdgeFrontier) -> Result<bool, TraversalModelError> {
        Ok(true)
    }
    fn terminate_search(&self, frontier: &EdgeFrontier) -> Result<bool, TraversalModelError> {
        Ok(false)
    }
}
