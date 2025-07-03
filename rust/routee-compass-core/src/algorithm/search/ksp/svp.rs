use super::{ksp_query::KspQuery, ksp_termination_criteria::KspTerminationCriteria};
use crate::{
    algorithm::search::{
        a_star::bidirectional_ops, backtrack, direction::Direction, edge_traversal::EdgeTraversal,
        search_algorithm::SearchAlgorithm, search_algorithm_result::SearchAlgorithmResult,
        search_error::SearchError, search_instance::SearchInstance, util::RouteSimilarityFunction,
    },
    model::{network::vertex_id::VertexId, unit::ReverseCost},
    util::priority_queue::InternalPriorityQueue,
};
use itertools::Itertools;
use std::collections::HashMap;

/// generates a set of k-shortest paths using the single-via path algorithm.
pub fn run(
    query: &KspQuery,
    termination: &KspTerminationCriteria,
    similarity: &RouteSimilarityFunction,
    si: &SearchInstance,
    underlying: &SearchAlgorithm,
) -> Result<SearchAlgorithmResult, SearchError> {
    // run forward and reverse search
    let SearchAlgorithmResult {
        trees: fwd_trees,
        routes: _,
        iterations: fwd_iterations,
    } = underlying.run_vertex_oriented(
        query.source,
        Some(query.target),
        query.user_query,
        &Direction::Forward,
        si,
    )?;
    let SearchAlgorithmResult {
        trees: rev_trees,
        routes: _,
        iterations: rev_iterations,
    } = underlying.run_vertex_oriented(
        query.target,
        Some(query.source),
        query.user_query,
        &Direction::Reverse,
        si,
    )?;
    if fwd_trees.len() != 1 {
        Err(SearchError::InternalError(format!(
            "ksp solver fwd trees count should be exactly 1, found {}",
            fwd_trees.len()
        )))?;
    }
    if rev_trees.len() != 1 {
        Err(SearchError::InternalError(format!(
            "ksp solver rev trees count should be exactly 1, found {}",
            rev_trees.len()
        )))?;
    }
    let fwd_tree = fwd_trees
        .first()
        .ok_or_else(|| SearchError::InternalError(String::from("cannot retrieve fwd tree 0")))?;
    let rev_tree = rev_trees
        .first()
        .ok_or_else(|| SearchError::InternalError(String::from("cannot retrieve rev tree 0")))?;

    // find intersection vertices
    let rev_labels = rev_trees.iter().flatten().collect::<HashMap<_, _>>();
    let mut intersection_queue: InternalPriorityQueue<VertexId, ReverseCost> =
        InternalPriorityQueue::default();

    // valid intersection vertices should appear both as terminal vertices and lookup vertices in both trees
    // - being a "terminal vertex" places them at the shared meeting location, terminus of each tree somewhere
    // - being a "lookup vertex" means we can use them for backtracking forward and reverse paths
    for (label, fwd_branch) in fwd_tree {
        if let Some(rev_branch) = rev_labels.get(&fwd_branch.terminal_label) {
            if rev_labels.contains_key(&label) {
                let total_cost =
                    fwd_branch.edge_traversal.total_cost() + rev_branch.edge_traversal.total_cost();
                intersection_queue.push(label.vertex_id(), total_cost.into());
            }
        }
    }

    log::debug!("ksp intersection has {} vertices", intersection_queue.len());

    let tsp = backtrack::label_oriented_route(query.source, query.target, fwd_tree)?;
    let mut solution: Vec<Vec<EdgeTraversal>> = vec![tsp];
    let mut ksp_it: u64 = 0;
    loop {
        if termination.terminate_search(query.k, solution.len()) {
            log::debug!(
                "ksp:{} solution contains {} entries, quitting due to termination function {}",
                ksp_it,
                query.k,
                termination
            );
            break;
        }
        match intersection_queue.pop() {
            None => {
                log::debug!("ksp:{} queue is empty, quitting", ksp_it);
                break;
            }
            Some((intersection_vertex_id, _)) => {
                let mut accept_route = true;
                // create the i'th route by backtracking both trees and concatenating the result
                let fwd_route = backtrack::label_oriented_route(
                    query.source,
                    intersection_vertex_id,
                    fwd_tree,
                )?;
                let rev_route_backward = backtrack::label_oriented_route(
                    query.target,
                    intersection_vertex_id,
                    rev_tree,
                )?;
                let rev_route =
                    bidirectional_ops::reorient_reverse_route(&fwd_route, &rev_route_backward, si)?;
                let this_route = fwd_route.into_iter().chain(rev_route).collect::<Vec<_>>();

                // test loop
                if bidirectional_ops::route_contains_loop(&this_route, si)? {
                    log::debug!("ksp:{} contains loop", ksp_it);
                    accept_route = false;
                }

                // for test user-provided similarity threshold and absolute similarity
                for solution_route in solution.iter() {
                    let absolute_similarity = test_id_similarity(&this_route, solution_route);
                    let too_similar = similarity.clone().test_similarity(
                        &this_route.iter().collect_vec(),
                        &solution_route.iter().collect_vec(),
                        si,
                    )?;
                    if absolute_similarity || too_similar {
                        log::debug!("ksp:{} too similar", ksp_it);
                        accept_route = false;
                        break;
                    }
                }

                if accept_route {
                    log::debug!("ksp:{} alternative accepted", ksp_it);
                    solution.push(this_route);
                }
                ksp_it += 1;
            }
        }
    }

    log::debug!("ksp ran in {} iterations", ksp_it);

    let routes = solution.into_iter().take(query.k).collect_vec();

    // combine all data into this result
    let result = SearchAlgorithmResult {
        trees: vec![fwd_tree.clone(), rev_tree.clone()], // todo: figure out how to avoid this clone
        routes,
        iterations: fwd_iterations + rev_iterations + ksp_it, // todo: figure out how to report individually
    };
    Ok(result)
}

/// checks if these two routes have the same length and id sequence
fn test_id_similarity(a: &[EdgeTraversal], b: &[EdgeTraversal]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (edge_a, edge_b) in a.iter().zip(b) {
        if edge_a.edge_id != edge_b.edge_id {
            return false;
        }
    }
    true
}
