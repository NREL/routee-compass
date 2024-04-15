use serde::{Deserialize, Serialize};

use super::a_star::a_star_algorithm;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use super::search_result::SearchResult;
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchAlgorithm {
    #[serde(rename = "a*")]
    AStarAlgorithm,
    KspSingleVia {
        k: usize,
        underlying: Box<SearchAlgorithm>,
    },
}

impl SearchAlgorithm {
    pub fn run_vertex_oriented(
        &self,
        origin: VertexId,
        destination: Option<VertexId>,
        search_instance: &SearchInstance,
    ) -> Result<Vec<SearchResult>, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                a_star_algorithm::run_a_star(origin, destination, search_instance).map(|r| vec![r])
            }
            SearchAlgorithm::KspSingleVia { k, underlying } => todo!(),
        }
    }
    pub fn run_edge_oriented(
        &self,
        origin: EdgeId,
        destination: Option<EdgeId>,
        search_instance: &SearchInstance,
    ) -> Result<Vec<SearchResult>, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                a_star_algorithm::run_a_star_edge_oriented(origin, destination, search_instance)
                    .map(|r| vec![r])
            }
            SearchAlgorithm::KspSingleVia { k, underlying } => todo!(),
        }
    }
}
