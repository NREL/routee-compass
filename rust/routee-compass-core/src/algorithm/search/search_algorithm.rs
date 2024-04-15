use super::a_star::a_star_algorithm;
use super::backtrack;
use super::search_algorithm_result::SearchAlgorithmResult;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
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
        src_id: VertexId,
        dst_id_opt: Option<VertexId>,
        search_instance: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                let search_result =
                    a_star_algorithm::run_a_star(src_id, dst_id_opt, search_instance)?;
                let routes = match dst_id_opt {
                    None => vec![],
                    Some(dst_id) => {
                        let route =
                            backtrack::vertex_oriented_route(src_id, dst_id, &search_result.tree)?;
                        vec![route]
                    }
                };
                Ok(SearchAlgorithmResult {
                    trees: vec![search_result.tree],
                    routes,
                    iterations: search_result.iterations,
                })
            }
            SearchAlgorithm::KspSingleVia { k, underlying } => todo!(),
        }
    }
    pub fn run_edge_oriented(
        &self,
        src_id: EdgeId,
        dst_id_opt: Option<EdgeId>,
        search_instance: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                let search_result = a_star_algorithm::run_a_star_edge_oriented(
                    src_id,
                    dst_id_opt,
                    search_instance,
                )?;
                let routes = match dst_id_opt {
                    None => vec![],
                    Some(dst_id) => {
                        let route = backtrack::edge_oriented_route(
                            src_id,
                            dst_id,
                            &search_result.tree,
                            search_instance.directed_graph.clone(),
                        )?;
                        vec![route]
                    }
                };
                Ok(SearchAlgorithmResult {
                    trees: vec![search_result.tree],
                    routes,
                    iterations: search_result.iterations,
                })
            }
            SearchAlgorithm::KspSingleVia { k, underlying } => todo!(),
        }
    }
}
