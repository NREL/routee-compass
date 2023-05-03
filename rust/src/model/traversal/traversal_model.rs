use crate::{
    algorithm::search::min_search_tree::frontier::Frontier,
    model::{cost::cost::Cost, graph::edge_id::EdgeId, property::edge::Edge},
};

use super::traversal_error::TraversalError;

pub trait TraversalModel {
    type State;

    fn initial_state(&self, e: Edge) -> Result<Self::State, TraversalError>;

    fn traversal_cost(&self, e: Edge) -> Result<Cost, TraversalError>;

    fn access_cost(&self, src: Edge, dst: Edge) -> Result<Cost, TraversalError>;

    fn update(&self, s: Self::State, c: Cost) -> Result<Self::State, TraversalError>;

    // todo: provide some acceptably-scoped termination function
    // fn terminate(
    //     &self,
    //     edge_id: EdgeId,
    //     frontier: Frontier<Self::State>,
    // ) -> Result<bool, TraversalError>;
}
