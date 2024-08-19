use super::backtrack;
use super::edge_traversal::EdgeTraversal;
use super::ksp::ksp_single_via_paths;
use super::ksp::ksp_termination_criteria::KspTerminationCriteria;
use super::ksp::route_similarity_function::RouteSimilarityFunction;
use super::search_algorithm_result::SearchAlgorithmResult;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use super::search_tree_branch::SearchTreeBranch;
use super::{a_star::a_star_algorithm, direction::Direction};
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};

use crate::model::unit::Cost;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SearchAlgorithm {
    Dijkstra,
    #[serde(rename = "a*")]
    AStarAlgorithm {
        weight_factor: Option<Cost>,
    },
    KspSingleVia {
        k: usize,
        underlying: Box<SearchAlgorithm>,
        similarity: Option<RouteSimilarityFunction>,
        termination: Option<KspTerminationCriteria>,
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
            SearchAlgorithm::Dijkstra => SearchAlgorithm::AStarAlgorithm {
                weight_factor: Some(Cost::ZERO),
            }
            .run_vertex_oriented(src_id, dst_id_opt, direction, si),
            SearchAlgorithm::AStarAlgorithm { weight_factor } => {
                let search_result = a_star_algorithm::run_a_star(
                    src_id,
                    dst_id_opt,
                    direction,
                    *weight_factor,
                    si,
                )?;
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
                termination,
            } => match dst_id_opt {
                Some(dst_id) => {
                    let sim_fn = similarity.as_ref().cloned().unwrap_or_default();
                    let term_fn = termination.as_ref().cloned().unwrap_or_default();
                    ksp_single_via_paths::run(src_id, dst_id, *k, &term_fn, &sim_fn, si, underlying)
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
            SearchAlgorithm::Dijkstra => SearchAlgorithm::AStarAlgorithm {
                weight_factor: Some(Cost::ZERO),
            }
            .run_edge_oriented(src_id, dst_id_opt, direction, search_instance),
            SearchAlgorithm::AStarAlgorithm { weight_factor } => {
                let search_result = a_star_algorithm::run_a_star_edge_oriented(
                    src_id,
                    dst_id_opt,
                    direction,
                    *weight_factor,
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
                termination: _,
            } => run_edge_oriented(src_id, dst_id_opt, direction, self, search_instance),
        }
    }
}

// convenience method when origin and destination are specified using
/// edge ids instead of vertex ids. invokes a vertex-oriented search
/// from the out-vertex of the source edge to the in-vertex of the
/// target edge. composes the result with the source and target.
///
/// not tested.
pub fn run_edge_oriented(
    source: EdgeId,
    target: Option<EdgeId>,
    direction: &Direction,
    alg: &SearchAlgorithm,
    si: &SearchInstance,
) -> Result<SearchAlgorithmResult, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let e1_src = si.directed_graph.src_vertex_id(source)?;
    let e1_dst = si.directed_graph.dst_vertex_id(source)?;
    let src_et = EdgeTraversal {
        edge_id: source,
        access_cost: Cost::ZERO,
        traversal_cost: Cost::ZERO,
        result_state: si.state_model.initial_state()?,
    };

    match target {
        None => {
            let src_branch = SearchTreeBranch {
                terminal_vertex: e1_src,
                edge_traversal: src_et.clone(),
            };
            let SearchAlgorithmResult {
                mut trees,
                mut routes,
                iterations,
            } = alg.run_vertex_oriented(e1_dst, None, direction, si)?;
            for tree in trees.iter_mut() {
                if !tree.contains_key(&e1_dst) {
                    tree.extend([(e1_dst, src_branch.clone())]);
                }
            }
            for route in routes.iter_mut() {
                route.insert(0, src_et.clone());
            }
            let updated = SearchAlgorithmResult {
                trees,
                routes,
                iterations: iterations + 1,
            };
            Ok(updated)
        }
        Some(target_edge) => {
            let e2_src = si.directed_graph.src_vertex_id(target_edge)?;
            let e2_dst = si.directed_graph.dst_vertex_id(target_edge)?;

            if source == target_edge {
                Ok(SearchAlgorithmResult::default())
            } else if e1_dst == e2_src {
                // route is simply source -> target
                let init_state = si.state_model.initial_state()?;
                let src_et = EdgeTraversal::forward_traversal(source, None, &init_state, si)?;
                let dst_et = EdgeTraversal::forward_traversal(
                    target_edge,
                    Some(source),
                    &src_et.result_state,
                    si,
                )?;
                let src_traversal = SearchTreeBranch {
                    terminal_vertex: e2_src,
                    edge_traversal: dst_et.clone(),
                };
                let dst_traversal = SearchTreeBranch {
                    terminal_vertex: e1_src,
                    edge_traversal: src_et.clone(),
                };
                let tree = HashMap::from([(e2_dst, src_traversal), (e1_dst, dst_traversal)]);
                let route = vec![src_et, dst_et];
                let result = SearchAlgorithmResult {
                    trees: vec![tree],
                    routes: vec![route],
                    iterations: 1,
                };
                return Ok(result);
            } else {
                // run a search and append source/target edges to result
                let SearchAlgorithmResult {
                    trees,
                    mut routes,
                    iterations,
                } = alg.run_vertex_oriented(e1_dst, Some(e2_src), direction, si)?;

                if trees.is_empty() {
                    return Err(SearchError::NoPathExists(e1_dst, e2_src));
                }

                // it is possible that the search already found these vertices. one major edge
                // case is when the trip starts with a u-turn.
                for route in routes.iter_mut() {
                    let final_state = route.last().ok_or_else(|| {
                        SearchError::InternalSearchError(String::from("found empty result route"))
                    })?;

                    let dst_et = EdgeTraversal {
                        edge_id: target_edge,
                        access_cost: Cost::ZERO,
                        traversal_cost: Cost::ZERO,
                        result_state: final_state.result_state.to_vec(),
                    };
                    route.insert(0, src_et.clone());
                    route.push(dst_et.clone());
                }

                let result = SearchAlgorithmResult {
                    trees,
                    routes,
                    iterations: iterations + 2,
                };
                Ok(result)
            }
        }
    }
}
