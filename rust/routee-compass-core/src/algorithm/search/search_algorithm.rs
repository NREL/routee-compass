use super::backtrack;
use super::ksp::ksp_single_via_paths;
use super::ksp::route_similarity_function::RouteSimilarityFunction;
use super::search_algorithm_result::SearchAlgorithmResult;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use super::{a_star::a_star_algorithm, direction::Direction};
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
        similarity: RouteSimilarityFunction,
    },
}

impl SearchAlgorithm {
    pub fn run_vertex_oriented(
        &self,
        src_id: VertexId,
        dst_id_opt: Option<VertexId>,
        direction: &Direction,
        si: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                let search_result =
                    a_star_algorithm::run_a_star(src_id, dst_id_opt, direction, si)?;
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
            SearchAlgorithm::KspSingleVia {
                k,
                underlying,
                similarity,
            } => match dst_id_opt {
                Some(dst_id) => {
                    ksp_single_via_paths::run(src_id, dst_id, *k, similarity, si, underlying)
                }
                None => Err(SearchError::BuildError(String::from(
                    "request has source but no destination which is invalid for k-shortest paths",
                ))),
            },
        }
    }
    pub fn run_edge_oriented(
        &self,
        src_id: EdgeId,
        dst_id_opt: Option<EdgeId>,
        direction: &Direction,
        search_instance: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::AStarAlgorithm => {
                let search_result = a_star_algorithm::run_a_star_edge_oriented(
                    src_id,
                    dst_id_opt,
                    direction,
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
            SearchAlgorithm::KspSingleVia {
                k: _,
                underlying: _,
                similarity: _,
            } => todo!(),
        }
    }
}
