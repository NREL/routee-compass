use super::backtrack;
use super::edge_traversal::EdgeTraversal;
use super::ksp::KspQuery;
use super::ksp::KspTerminationCriteria;
use super::ksp::{svp, yens};
use super::search_algorithm_result::SearchAlgorithmResult;
use super::search_error::SearchError;
use super::search_instance::SearchInstance;
use super::search_tree_branch::SearchTreeBranch;
use super::util::RouteSimilarityFunction;
use super::{a_star, direction::Direction};
use crate::model::network::{edge_id::EdgeId, vertex_id::VertexId};
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
    Yens {
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
        query: &serde_json::Value,
        direction: &Direction,
        si: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::Dijkstra => SearchAlgorithm::AStarAlgorithm {
                weight_factor: Some(Cost::ZERO),
            }
            .run_vertex_oriented(src_id, dst_id_opt, query, direction, si),
            SearchAlgorithm::AStarAlgorithm { weight_factor } => {
                let w_val = match query.get("weight_factor") {
                    Some(w_json) => w_json
                        .as_f64()
                        .ok_or(SearchError::BuildError(format!(
                            "weight_factor must be a float, found {}",
                            w_json
                        )))
                        .map(|f| Some(Cost::new(f))),
                    None => Ok(*weight_factor),
                }?;
                let search_result =
                    a_star::run_vertex_oriented(src_id, dst_id_opt, direction, w_val, si)?;
                let routes = match dst_id_opt {
                    None => vec![],
                    Some(dst_id) => {
                        let route =
                            backtrack::label_oriented_route(src_id, dst_id, &search_result.tree)?;
                        vec![route]
                    }
                };
                Ok(SearchAlgorithmResult {
                    trees: vec![search_result.tree],
                    routes,
                    iterations: search_result.iterations,
                })
            }
            SearchAlgorithm::Yens {
                k,
                underlying,
                similarity,
                termination,
            } => {
                let dst_id = dst_id_opt.ok_or_else(|| {
                    SearchError::BuildError(String::from(
                        "attempting to run KSP algorithm without destination",
                    ))
                })?;
                let sim_fn = similarity.as_ref().cloned().unwrap_or_default();
                let term_fn = termination.as_ref().cloned().unwrap_or_default();
                let ksp_query = KspQuery::new(src_id, dst_id, query, *k)?;
                yens::run(&ksp_query, &term_fn, &sim_fn, si, underlying)
            }
            SearchAlgorithm::KspSingleVia {
                k,
                underlying,
                similarity,
                termination,
            } => {
                let dst_id = dst_id_opt.ok_or_else(|| {
                    SearchError::BuildError(String::from(
                        "attempting to run KSP algorithm without destination",
                    ))
                })?;
                let sim_fn = similarity.as_ref().cloned().unwrap_or_default();
                let term_fn = termination.as_ref().cloned().unwrap_or_default();
                let ksp_query = KspQuery::new(src_id, dst_id, query, *k)?;
                svp::run(&ksp_query, &term_fn, &sim_fn, si, underlying)
            }
        }
    }
    pub fn run_edge_oriented(
        &self,
        src_id: EdgeId,
        dst_id_opt: Option<EdgeId>,
        query: &serde_json::Value,
        direction: &Direction,
        search_instance: &SearchInstance,
    ) -> Result<SearchAlgorithmResult, SearchError> {
        match self {
            SearchAlgorithm::Dijkstra => SearchAlgorithm::AStarAlgorithm {
                weight_factor: Some(Cost::ZERO),
            }
            .run_edge_oriented(src_id, dst_id_opt, query, direction, search_instance),
            SearchAlgorithm::AStarAlgorithm { weight_factor } => {
                let search_result = a_star::run_edge_oriented(
                    src_id,
                    dst_id_opt,
                    direction,
                    *weight_factor,
                    search_instance,
                )?;
                let routes = match dst_id_opt {
                    None => vec![],
                    Some(dst_id) => {
                        let route = backtrack::label_edge_oriented_route(
                            src_id,
                            dst_id,
                            &search_result.tree,
                            search_instance.graph.clone(),
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
            } => run_edge_oriented(src_id, dst_id_opt, query, direction, self, search_instance),
            SearchAlgorithm::Yens {
                k: _,
                underlying: _,
                similarity: _,
                termination: _,
            } => run_edge_oriented(src_id, dst_id_opt, query, direction, self, search_instance),
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
    query: &serde_json::Value,
    direction: &Direction,
    alg: &SearchAlgorithm,
    si: &SearchInstance,
) -> Result<SearchAlgorithmResult, SearchError> {
    // 1. guard against edge conditions (src==dst, src.dst_v == dst.src_v)
    let initial_state = si.state_model.initial_state()?;
    let e1_src = si.graph.src_vertex_id(&source)?;
    let e1_label = si
        .label_model
        .label_from_state(e1_src, &initial_state, &si.state_model)?;
    let e1_dst = si.graph.dst_vertex_id(&source)?;
    let src_et = EdgeTraversal {
        edge_id: source,
        access_cost: Cost::ZERO,
        traversal_cost: Cost::ZERO,
        result_state: si.state_model.initial_state()?,
    };

    match target {
        None => {
            let src_branch = SearchTreeBranch {
                terminal_label: e1_label,
                edge_traversal: src_et.clone(),
            };
            let SearchAlgorithmResult {
                mut trees,
                mut routes,
                iterations,
            } = alg.run_vertex_oriented(e1_dst, None, query, direction, si)?;

            let dst_label =
                si.label_model
                    .label_from_state(e1_dst, &initial_state, &si.state_model)?;

            for tree in trees.iter_mut() {
                if !tree.contains_key(&dst_label) {
                    tree.extend([(dst_label.clone(), src_branch.clone())]);
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
            let e2_src = si.graph.src_vertex_id(&target_edge)?;
            let e2_dst = si.graph.dst_vertex_id(&target_edge)?;

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

                // Create labels for the vertices using appropriate states
                let dst_label = si.label_model.label_from_state(
                    e2_dst,
                    &dst_et.result_state,
                    &si.state_model,
                )?;
                let src_label = si.label_model.label_from_state(
                    e1_dst,
                    &src_et.result_state,
                    &si.state_model,
                )?;

                let src_traversal = SearchTreeBranch {
                    terminal_label: src_label.clone(),
                    edge_traversal: dst_et.clone(),
                };
                let dst_traversal = SearchTreeBranch {
                    terminal_label: dst_label.clone(),
                    edge_traversal: src_et.clone(),
                };

                let tree = HashMap::from([(dst_label, src_traversal), (src_label, dst_traversal)]);
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
                } = alg.run_vertex_oriented(e1_dst, Some(e2_src), query, direction, si)?;

                if trees.is_empty() {
                    return Err(SearchError::NoPathExistsBetweenVertices(e1_dst, e2_src, 0));
                }

                // it is possible that the search already found these vertices. one major edge
                // case is when the trip starts with a u-turn.
                for route in routes.iter_mut() {
                    let final_state = route.last().ok_or_else(|| {
                        SearchError::InternalError(String::from("found empty result route"))
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
