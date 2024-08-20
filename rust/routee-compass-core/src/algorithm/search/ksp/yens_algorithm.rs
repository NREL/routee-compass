use std::collections::HashSet;

use itertools::Itertools;

use super::ksp_termination_criteria::KspTerminationCriteria;
use crate::{
    algorithm::search::{
        edge_traversal::EdgeTraversal, search_algorithm::SearchAlgorithm,
        search_algorithm_result::SearchAlgorithmResult, search_error::SearchError,
        search_instance::SearchInstance, util::route_similarity_function::RouteSimilarityFunction,
    },
    model::road_network::{edge_id::EdgeId, vertex_id::VertexId},
};

pub fn run(
    source: VertexId,
    target: VertexId,
    k: usize,
    termination: &KspTerminationCriteria,
    similarity: &RouteSimilarityFunction,
    si: &SearchInstance,
    underlying: &SearchAlgorithm,
) -> Result<SearchAlgorithmResult, SearchError> {
    // base case: we always have the true-shortest path
    let shortest = underlying.run_vertex_oriented(
        source,
        Some(target),
        &crate::algorithm::search::direction::Direction::Forward,
        si,
    )?;
    if shortest.routes.is_empty() {
        return Ok(SearchAlgorithmResult::default());
    }
    let mut accepted: Vec<SearchAlgorithmResult> = vec![shortest];

    while accepted.len() < k {
        let last_result = accepted
            .last()
            .ok_or(SearchError::InternalSearchError(String::from(
                "at least one route should be in routes",
            )))?;
        let prev_accepted_path = get_first_route(last_result)?;

        // step through each index along the most recently-accepted path
        for spur_idx in 0..prev_accepted_path.len() - 2 {
            let spur_len: usize = spur_idx + 1;
            let mut cut_edges: HashSet<EdgeId> = HashSet::new();
            let root_path = prev_accepted_path.iter().take(spur_len).collect_vec();

            // cut frontier edges based on previous paths with matching root path
            for accepted_result in accepted.iter() {
                let accepted_route = get_first_route(accepted_result)?;
                let accepted_path_root = accepted_route.iter().take(spur_len).collect_vec();
                if same_path(&root_path, &accepted_path_root) {
                    if let Some(cut_edge) = accepted_route.get(spur_idx + 1) {
                        cut_edges.insert(cut_edge.edge_id);
                    }
                }
            }

            todo!("we need a generic cost model to inject infinte-cost links!")
        }
    }

    todo!()
}

fn get_first_route(res: &SearchAlgorithmResult) -> Result<&Vec<EdgeTraversal>, SearchError> {
    res.routes
        .first()
        .ok_or(SearchError::InternalSearchError(String::from(
            "no empty results should be stored in routes",
        )))
}

fn same_path(a: &[&EdgeTraversal], b: &[&EdgeTraversal]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (a_edge, b_edge) in a.iter().zip(b) {
        if a_edge.edge_id == b_edge.edge_id {
            return false;
        }
    }
    true
}
