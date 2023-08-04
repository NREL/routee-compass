use crate::model::traversal::function::cost_function_error::CostFunctionError;
use crate::model::{
    graph::{edge_id::EdgeId, graph_error::GraphError, vertex_id::VertexId},
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
    CostFunctionError(#[from] CostFunctionError),
    #[error("loop in search result revisits edge {0}")]
    LoopInSearchResult(EdgeId),
    #[error("no path exists between vertices {0} and {1}")]
    NoPathExists(VertexId, VertexId),
    #[error("search tree is missing linked vertex {0}")]
    VertexMissingFromSearchTree(VertexId),
    #[error("error accessing shared read-only dataset: {0}")]
    ReadOnlyPoisonError(String),
    #[error("internal error due to search logic: {0}")]
    InternalSearchError(String),
}
