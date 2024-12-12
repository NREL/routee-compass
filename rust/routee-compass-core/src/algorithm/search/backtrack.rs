use super::{
    edge_traversal::EdgeTraversal, search_error::SearchError, search_tree_branch::SearchTreeBranch,
};
use crate::model::network::{edge_id::EdgeId, graph::Graph, vertex_id::VertexId};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// reconstructs a path from a minimum shortest path tree for some source and target vertex
/// directionality travels up from target to source, toward root of the tree, in both the forward
/// and reverse cases.
pub fn vertex_oriented_route(
    source_id: VertexId,
    target_id: VertexId,
    solution: &HashMap<VertexId, SearchTreeBranch>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    if solution.is_empty() {
        return Ok(vec![]);
    }

    let mut result: Vec<EdgeTraversal> = vec![];
    let mut visited: HashSet<EdgeId> = HashSet::new();
    let mut this_vertex = target_id;
    loop {
        if this_vertex == source_id {
            break;
        }
        let traversal = solution.get(&this_vertex).ok_or_else(|| {
            SearchError::InternalError(format!(
                "resulting tree with {} branches missing vertex {} expected via backtrack",
                solution.len(),
                this_vertex
            ))
        })?;
        let first_visit = visited.insert(traversal.edge_traversal.edge_id);
        if !first_visit {
            return Err(SearchError::InternalError(format!(
                "loop in search result, edge {} visited more than once",
                traversal.edge_traversal.edge_id
            )));
        }
        result.push(traversal.edge_traversal.clone());
        this_vertex = traversal.terminal_vertex;
    }
    let reversed = result.into_iter().rev().collect();
    Ok(reversed)
}

/// edge-oriented backtrack method
pub fn edge_oriented_route(
    source_id: EdgeId,
    target_id: EdgeId,
    solution: &HashMap<VertexId, SearchTreeBranch>,
    graph: Arc<Graph>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let o_v = graph.src_vertex_id(&source_id)?;
    let d_v = graph.dst_vertex_id(&target_id)?;
    vertex_oriented_route(o_v, d_v, solution)
}
