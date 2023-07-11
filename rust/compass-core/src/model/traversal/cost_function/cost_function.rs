use crate::algorithm::search::min_search_tree::dijkstra::edge_frontier::EdgeFrontier;
use crate::model::cost::cost::Cost;
use crate::model::property::{edge::Edge, vertex::Vertex};
use crate::model::traversal::state::state_variable::StateVar;
use crate::model::traversal::traversal_error::TraversalError;

/// the cost for traversing an edge
pub type EdgeCostFunction = Box<
    dyn Fn(&Vertex, &Edge, &Vertex, &Vec<StateVar>) -> Result<(Cost, Vec<StateVar>), TraversalError>
        + Sync
        + 'static,
>;

/// the cost for accessing the 2nd edge argument which can
/// be subject to the 1st edge argument
pub type EdgeEdgeCostFunction = Box<
    dyn Fn(
            &Vertex,
            &Edge,
            &Vertex,
            &Edge,
            &Vertex,
            &Vec<StateVar>,
        ) -> Result<(Cost, Vec<StateVar>), TraversalError>
        + Sync
        + 'static,
>;

/// returns true if the frontier is valid to use in a search
pub type ValidFrontierFunction =
    Box<dyn Fn(&EdgeFrontier<Vec<Vec<StateVar>>>) -> Result<bool, TraversalError> + Sync + 'static>;

/// returns true if we want to terminate the search upon reaching this frontier
pub type TerminateSearchFunction =
    Box<dyn Fn(&EdgeFrontier<Vec<Vec<StateVar>>>) -> Result<bool, TraversalError> + Sync + 'static>;
