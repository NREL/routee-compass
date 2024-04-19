use super::route_similarity_function::RouteSimilarityFunction;
use crate::{
    algorithm::search::{
        direction::Direction, edge_traversal::EdgeTraversal, search_algorithm::SearchAlgorithm,
        search_algorithm_result::SearchAlgorithmResult, search_error::SearchError,
        search_instance::SearchInstance,
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
    let fwd_result =
        underlying.run_vertex_oriented(source, Some(target), &Direction::Forward, si)?;
    let rev_result =
        underlying.run_vertex_oriented(source, Some(target), &Direction::Reverse, si)?;

    // find intersection vertices
    let fwd_vertices = fwd_result.trees.iter().flatten().collect::<HashMap<_, _>>();
    let mut intersection_queue: InternalPriorityQueue<VertexId, ReverseCost> =
        InternalPriorityQueue::default();
    for tree in rev_result.trees {
        for (vertex_id, rev_branch) in tree {
            if let Some(fwd_branch) = fwd_vertices.get(&vertex_id) {
                let total_cost =
                    fwd_branch.edge_traversal.total_cost() + rev_branch.edge_traversal.total_cost();
                intersection_queue.push(vertex_id, total_cost.into());
            }
        }
    }

    // todo: implement SimilarityFunction
    let mut solution: Vec<Vec<EdgeTraversal>> = vec![];
    loop {
        if solution.len() == k {
            break;
        }
        match intersection_queue.pop() {
            None => break,
            Some((intersection_vertex_id, _)) => {
                // - backtrack both routes
                // - compare with similarity to each solution route
                // similarity.rank_similarity(a, b)
                // - append when sufficiently dis-similar
            }
        }
    }
    // combine all data into this result
    let result = SearchAlgorithmResult {
        trees: todo!(),
        routes: todo!(),
        iterations: todo!(),
    };
    return Ok(result);
}
