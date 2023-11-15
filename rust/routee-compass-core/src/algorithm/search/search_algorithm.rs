use super::a_star::a_star_algorithm;
use super::search_error::SearchError;
use super::MinSearchTree;
use crate::algorithm::search::search_algorithm_type::SearchAlgorithmType;
use crate::model::frontier::frontier_model::FrontierModel;
use crate::model::road_network::graph::Graph;
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};
use crate::model::termination::termination_model::TerminationModel;
use crate::model::traversal::traversal_model::TraversalModel;
use crate::util::read_only_lock::ExecutorReadOnlyLock;
use std::sync::Arc;

pub enum SearchAlgorithm {
    AStarAlgorithm,
}

impl TryFrom<&serde_json::Value> for SearchAlgorithm {
    type Error = SearchError;

    fn try_from(config: &serde_json::Value) -> Result<Self, Self::Error> {
        let alg_type: SearchAlgorithmType = config.try_into()?;
        match alg_type {
            SearchAlgorithmType::AStar => Ok(SearchAlgorithm::AStarAlgorithm),
        }
    }
}

impl SearchAlgorithm {
    pub fn run_vertex_oriented(
        &self,
        origin: VertexId,
        destination: Option<VertexId>,
        graph: Arc<ExecutorReadOnlyLock<Graph>>,
        traversal_model: Arc<dyn TraversalModel>,
        frontier_model: Arc<ExecutorReadOnlyLock<Box<dyn FrontierModel>>>,
        termination_model: Arc<ExecutorReadOnlyLock<TerminationModel>>,
    ) -> Result<MinSearchTree, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => a_star_algorithm::run_a_star(
                origin,
                destination,
                graph,
                traversal_model,
                frontier_model,
                termination_model,
            ),
        }
    }
    pub fn run_edge_oriented(
        &self,
        origin: EdgeId,
        destination: Option<EdgeId>,
        graph: Arc<ExecutorReadOnlyLock<Graph>>,
        traversal_model: Arc<dyn TraversalModel>,
        frontier_model: Arc<ExecutorReadOnlyLock<Box<dyn FrontierModel>>>,
        termination_model: Arc<ExecutorReadOnlyLock<TerminationModel>>,
    ) -> Result<MinSearchTree, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => a_star_algorithm::run_a_star_edge_oriented(
                origin,
                destination,
                graph,
                traversal_model,
                frontier_model,
                termination_model,
            ),
        }
    }
}
