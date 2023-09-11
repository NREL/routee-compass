use crate::model::{
    frontier::frontier_model_error::FrontierModelError,
    graphv2::graph_error::GraphError,
    graphv2::{edge_id::EdgeId, vertex_id::VertexId},
    termination::termination_model_error::TerminationModelError,
    traversal::traversal_model_error::TraversalModelError,
};

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("distance heuristic can only be provided when there is a target")]
    DistanceHeuristicWithNoTarget,
    #[error(transparent)]
    GraphError(#[from] GraphError),
    #[error(transparent)]
    TerminationModelError(#[from] TerminationModelError),
    #[error("failure applying the traversal model in search")]
    TraversalModelFailure(#[from] TraversalModelError),
    #[error("failure applying the frontier model in search")]
    FrontierModelFailure(#[from] FrontierModelError),
    #[error("loop in search result revisits edge {0}")]
    LoopInSearchResult(EdgeId),
    #[error("query terminated due to {0}")]
    QueryTerminated(String),
    #[error("no path exists between vertices {0} and {1}")]
    NoPathExists(VertexId, VertexId),
    #[error("search tree is missing linked vertex {0}")]
    VertexMissingFromSearchTree(VertexId),
    #[error("error accessing shared read-only dataset: {0}")]
    ReadOnlyPoisonError(String),
    #[error("internal error due to search logic: {0}")]
    InternalSearchError(String),
}
