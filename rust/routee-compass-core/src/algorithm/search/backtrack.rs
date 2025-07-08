use super::{
    edge_traversal::EdgeTraversal, search_error::SearchError, search_tree_branch::SearchTreeBranch,
};
use crate::model::{
    label::Label,
    network::{edge_id::EdgeId, graph::Graph, vertex_id::VertexId},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// reconstructs a path from a minimum shortest path tree using labels for some source and target vertex
/// directionality travels up from target to source, toward root of the tree, in both the forward
/// and reverse cases.
pub fn label_oriented_route(
    source_id: VertexId,
    target_id: VertexId,
    solution: &HashMap<Label, SearchTreeBranch>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    if solution.is_empty() {
        return Ok(vec![]);
    }

    log::debug!(
        "label_oriented_route: source_id: {}, target_id: {}, solution size: {}",
        source_id,
        target_id,
        solution.len()
    );

    let mut result: Vec<EdgeTraversal> = vec![];
    let mut visited: HashSet<VertexId> = HashSet::new();

    // Find the target label in the solution - there should be exactly one optimal path to target
    let (mut current_label, _) = solution
        .iter()
        .find(|(label, _)| label.vertex_id() == target_id)
        .ok_or_else(|| {
            SearchError::InternalError(format!(
                "target vertex {} not found in solution tree with {} branches",
                target_id,
                solution.len()
            ))
        })?;

    loop {
        let current_vertex = current_label.vertex_id();
        if current_vertex == source_id {
            break;
        }

        let first_visit = visited.insert(current_vertex);
        if !first_visit {
            return Err(SearchError::InternalError(format!(
                "loop in search result, vertex {:?} visited more than once",
                current_vertex
            )));
        }

        let branch = solution.get(current_label).ok_or_else(|| {
            SearchError::InternalError(format!(
                "label {:?} not found in solution tree during backtrack",
                current_label
            ))
        })?;

        result.push(branch.edge_traversal.clone());
        current_label = &branch.terminal_label;
    }

    let reversed = result.into_iter().rev().collect();
    Ok(reversed)
}

/// edge-oriented backtrack method using labels
pub fn label_edge_oriented_route(
    source_id: EdgeId,
    target_id: EdgeId,
    solution: &HashMap<Label, SearchTreeBranch>,
    graph: Arc<Graph>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let o_v = graph.dst_vertex_id(&source_id)?;
    let d_v = graph.src_vertex_id(&target_id)?;
    label_oriented_route(o_v, d_v, solution)
}
