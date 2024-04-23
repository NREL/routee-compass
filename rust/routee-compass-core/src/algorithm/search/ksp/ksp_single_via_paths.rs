use super::route_similarity_function::RouteSimilarityFunction;
use crate::{
    algorithm::search::{
        backtrack, direction::Direction, edge_traversal::EdgeTraversal,
        search_algorithm::SearchAlgorithm, search_algorithm_result::SearchAlgorithmResult,
        search_error::SearchError, search_instance::SearchInstance,
    },
    model::{road_network::vertex_id::VertexId, unit::cost::ReverseCost},
    util::priority_queue::InternalPriorityQueue,
};
use std::collections::HashMap;

/// generates a set of k-shortest paths using the single-via path algorithm.
pub fn run(
    source: VertexId,
    target: VertexId,
    k: usize,
    similarity: &RouteSimilarityFunction,
    si: &SearchInstance,
    underlying: &SearchAlgorithm,
) -> Result<SearchAlgorithmResult, SearchError> {
    // run forward and reverse search
    let SearchAlgorithmResult {
        trees: fwd_trees,
        routes: _,
        iterations: fwd_iterations,
    } = underlying.run_vertex_oriented(source, Some(target), &Direction::Forward, si)?;
    let SearchAlgorithmResult {
        trees: rev_trees,
        routes: _,
        iterations: rev_iterations,
    } = underlying.run_vertex_oriented(target, Some(source), &Direction::Reverse, si)?;
    if fwd_trees.len() != 1 {
        Err(SearchError::InternalSearchError(format!(
            "ksp solver fwd trees count should be exactly 1, found {}",
            fwd_trees.len()
        )))?;
    }
    if rev_trees.len() != 1 {
        Err(SearchError::InternalSearchError(format!(
            "ksp solver rev trees count should be exactly 1, found {}",
            rev_trees.len()
        )))?;
    }
    let fwd_tree = fwd_trees.first().ok_or_else(|| {
        SearchError::InternalSearchError(String::from("cannot retrieve fwd tree 0"))
    })?;
    let rev_tree = rev_trees.first().ok_or_else(|| {
        SearchError::InternalSearchError(String::from("cannot retrieve rev tree 0"))
    })?;

    // find intersection vertices
    let rev_vertices = rev_trees.iter().flatten().collect::<HashMap<_, _>>();
    let mut intersection_queue: InternalPriorityQueue<VertexId, ReverseCost> =
        InternalPriorityQueue::default();

    // valid intersection vertices should appear both as terminal vertices and lookup vertices in both trees
    // - being a "terminal vertex" places them at the shared meeting location, terminus of each tree somewhere
    // - being a "lookup vertex" means we can use them for backtracking forward and reverse paths
    for (vertex_id, fwd_branch) in fwd_tree {
        if let Some(rev_branch) = rev_vertices.get(&fwd_branch.terminal_vertex) {
            if rev_vertices.contains_key(&vertex_id) {
                let total_cost =
                    fwd_branch.edge_traversal.total_cost() + rev_branch.edge_traversal.total_cost();
                intersection_queue.push(*vertex_id, total_cost.into());
            }
        }
    }

    let tsp = backtrack::vertex_oriented_route(source, target, fwd_tree)?;
    let mut solution: Vec<Vec<EdgeTraversal>> = vec![tsp];
    let mut ksp_iterations: u64 = 0;
    loop {
        if solution.len() == k {
            break;
        }
        ksp_iterations += 1;
        match intersection_queue.pop() {
            None => break,
            Some((intersection_vertex_id, _)) => {
                let fwd_route =
                    backtrack::vertex_oriented_route(source, intersection_vertex_id, fwd_tree)?;
                let mut rev_route =
                    backtrack::vertex_oriented_route(target, intersection_vertex_id, rev_tree)?;
                rev_route.reverse();
                let this_route = fwd_route.into_iter().chain(rev_route).collect::<Vec<_>>();
                for solution_route in solution.iter() {
                    let similarity_value =
                        similarity.rank_similarity(&this_route, solution_route, si)?;
                    if !similarity.sufficiently_dissimilar(similarity_value) {
                        break;
                    }
                }
                solution.push(this_route);
            }
        }
    }

    // combine all data into this result
    let result = SearchAlgorithmResult {
        trees: vec![fwd_tree.clone(), rev_tree.clone()], // todo: figure out how to avoid this clone
        routes: solution,
        iterations: fwd_iterations + rev_iterations + ksp_iterations, // todo: figure out how to report individually
    };
    Ok(result)
}
