use super::a_star::a_star_algorithm;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use super::search_result::SearchResult;
use crate::algorithm::search::search_algorithm_type::SearchAlgorithmType;
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};

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
        search_instance: &SearchInstance,
    ) -> Result<SearchResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                a_star_algorithm::run_a_star(origin, destination, search_instance)
            }
        }
    }
    pub fn run_edge_oriented(
        &self,
        origin: EdgeId,
        destination: Option<EdgeId>,
        search_instance: &SearchInstance,
    ) -> Result<SearchResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                a_star_algorithm::run_a_star_edge_oriented(origin, destination, search_instance)
            }
        }
    }
}
