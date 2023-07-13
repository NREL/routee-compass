use std::collections::HashSet;

use crate::model::graph::directed_graph::DirectedGraph;
use crate::model::graph::{graph_error::GraphError, vertex_id::VertexId};

fn depth_first_search(
    graph: &impl DirectedGraph,
    vertex: VertexId,
    visited: &mut HashSet<VertexId>,
    container: &mut Vec<VertexId>,
) -> Result<(), GraphError> {
    if visited.contains(&vertex) {
        return Ok(());
    }

    visited.insert(vertex);

    let edges = graph.out_edges(vertex)?;
    for edge in edges {
        let dst = graph.dst_vertex(edge)?;
        depth_first_search(graph, dst, visited, container)?;
    }

    container.push(vertex);

    Ok(())
}

fn reverse_depth_first_search(
    graph: &impl DirectedGraph,
    vertex: VertexId,
    visited: &mut HashSet<VertexId>,
    container: &mut Vec<VertexId>,
) -> Result<(), GraphError> {
    if visited.contains(&vertex) {
        return Ok(());
    }

    visited.insert(vertex);

    let edges = graph.in_edges(vertex)?;
    for edge in edges {
        let src = graph.src_vertex(edge)?;
        reverse_depth_first_search(graph, src, visited, container)?;
    }

    container.push(vertex);

    Ok(())
}

fn all_strongly_connected_componenets(
    graph: &impl DirectedGraph,
) -> Result<Vec<Vec<VertexId>>, GraphError> {
    let mut visited: HashSet<VertexId> = HashSet::new();
    let mut container: Vec<VertexId> = Vec::new();

    let mut result: Vec<Vec<VertexId>> = Vec::new();

    for vertex_id in graph.all_vertex_ids() {
        depth_first_search(graph, vertex_id, &mut visited, &mut container)?;
    }

    visited.clear();

    while let Some(vertex_id) = container.pop() {
        if visited.contains(&vertex_id) {
            continue;
        }

        let mut component: Vec<VertexId> = Vec::new();
        reverse_depth_first_search(graph, vertex_id, &mut visited, &mut component)?;
        result.push(component);
    }

    Ok(result)
}

fn largest_strongly_connected_component(
    graph: &impl DirectedGraph,
) -> Result<Vec<VertexId>, GraphError> {
    let components = all_strongly_connected_componenets(graph)?;

    let mut largest_component: Vec<VertexId> = Vec::new();

    for component in components {
        if component.len() > largest_component.len() {
            largest_component = component;
        }
    }

    Ok(largest_component)
}

#[cfg(test)]
mod tests {

}