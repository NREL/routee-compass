use crate::model::{graph::graph_error::GraphError, traversal::traversal_error::TraversalError};

#[derive(thiserror::Error, Debug, Clone)]
pub enum SearchError {
    #[error("distance heuristic can only be provided when there is a target")]
    DistanceHeuristicWithNoTarget,
    #[error("expected graph objects were not present")]
    GraphCorrectnessFailure(#[from] GraphError),
    #[error("failure applying the traversal model in search")]
    TraversalModelFailure(#[from] TraversalError),
}
