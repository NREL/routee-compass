use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    model::road_network::{edge_id::EdgeId, graph::Graph, vertex_id::VertexId},
    util::read_only_lock::ExecutorReadOnlyLock,
};

use super::{
    edge_traversal::EdgeTraversal, search_error::SearchError, search_tree_branch::SearchTreeBranch,
};

/// reconstructs a path from a minimum shortest path tree for some source and target vertex
/// directionality travels up from target to source, toward root of the tree, in both the forward
/// and reverse cases.
pub fn vertex_oriented_route(
    source_id: VertexId,
    target_id: VertexId,
    solution: &HashMap<VertexId, SearchTreeBranch>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let mut result: Vec<EdgeTraversal> = vec![];
    let mut visited: HashSet<EdgeId> = HashSet::new();
    let mut this_vertex = target_id;
    loop {
        if this_vertex == source_id {
            break;
        }
        let traversal = solution
            .get(&this_vertex)
            .ok_or_else(|| SearchError::VertexMissingFromSearchTree(this_vertex))?;
        let first_visit = visited.insert(traversal.edge_traversal.edge_id);
        if !first_visit {
            return Err(SearchError::LoopInSearchResult(
                traversal.edge_traversal.edge_id,
            ));
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
    graph: Arc<ExecutorReadOnlyLock<Graph>>,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    let g_inner = graph
        .read()
        .map_err(|e| SearchError::ReadOnlyPoisonError(e.to_string()))?;
    let o_v = g_inner
        .src_vertex_id(source_id)
        .map_err(SearchError::GraphError)?;
    let d_v = g_inner
        .dst_vertex_id(target_id)
        .map_err(SearchError::GraphError)?;
    vertex_oriented_route(o_v, d_v, solution)
}
