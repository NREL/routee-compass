use crate::model::{
    access::access_model_error::AccessModelError,
    cost::cost_error::CostError,
    frontier::frontier_model_error::FrontierModelError,
    graph::{edge_id::EdgeId, graph_error::GraphError, vertex_id::VertexId},
    state::state_error::StateError,
    termination::termination_model_error::TerminationModelError,
    traversal::traversal_model_error::TraversalModelError,
};

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("distance heuristic can only be provided when there is a target")]
    DistanceHeuristicWithNoTarget,
    #[error(transparent)]
    StateError(#[from] StateError),
    #[error(transparent)]
    GraphError(#[from] GraphError),
    #[error(transparent)]
    TerminationModelError(#[from] TerminationModelError),
    #[error(transparent)]
    TraversalModelFailure(#[from] TraversalModelError),
    #[error(transparent)]
    AccessModelFailure(#[from] AccessModelError),
    #[error(transparent)]
    FrontierModelFailure(#[from] FrontierModelError),
    #[error(transparent)]
    CostError(#[from] CostError),
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
    #[error("failure building search algorithm: {0}")]
    BuildError(String),
    #[error("internal error due to search logic: {0}")]
    InternalSearchError(String),
}
