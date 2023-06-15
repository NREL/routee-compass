use std::sync::PoisonError;

use crate::model::{
    cost::cost_error::CostError,
    graph::{graph_error::GraphError, vertex_id::VertexId},
    traversal::traversal_error::TraversalError,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum SearchError {
    #[error("distance heuristic can only be provided when there is a target")]
    DistanceHeuristicWithNoTarget,
    #[error("expected graph objects were not present")]
    GraphCorrectnessFailure(#[from] GraphError),
    #[error("failure applying the traversal model in search")]
    TraversalModelFailure(#[from] TraversalError),
    #[error("failure calculating cost")]
    CostCalculationError(#[from] CostError),
    #[error("search tree is missing linked vertex {0}")]
    VertexMissingFromSearchTree(VertexId),
    #[error("error accessing shared read-only dataset: {0}")]
    ReadOnlyPoisonError(String),
    #[error("internal error due to search logic: {0}")]
    InternalSearchError(String),
}
