use crate::model::road_network::graph::Graph;
use crate::model::road_network::{graph_error::GraphError, vertex_id::VertexId};
use std::collections::HashSet;

/// Conducts a depth-first search (DFS) on a directed graph.
///
/// This function takes a directed graph, a vertex ID, a mutable HashSet to keep track of visited vertices,
/// and a mutable Vec as a container to store the order in which vertices are visited.
/// The function returns an empty Result on successful completion, and a GraphError if something went wrong.
///
/// # Arguments
///
/// * `graph` - A directed graph to perform DFS on.
/// * `vertex` - A VertexId to start the DFS from.
/// * `visited` - A mutable reference to a HashSet storing visited VertexIds.
/// * `stack` - A mutable reference to a stack storing VertexIds in the order of DFS traversal.
///
/// # Errors
///
/// Returns an error if the `graph` has an issue like a non-existing vertex.
///
pub fn depth_first_search(
    graph: &Graph,
    vertex: VertexId,
    visited: &mut HashSet<VertexId>,
    stack: &mut Vec<VertexId>,
) -> Result<(), GraphError> {
    if visited.contains(&vertex) {
        return Ok(());
    }

    visited.insert(vertex);

    let edges = graph.out_edges(vertex)?;
    for edge in edges {
        let dst = graph.dst_vertex_id(edge)?;
        depth_first_search(graph, dst, visited, stack)?;
    }

    stack.push(vertex);

    Ok(())
}

/// Conducts a reverse depth-first search (DFS) on a directed graph.
///
/// This function takes a directed graph, a vertex ID, a mutable HashSet to keep track of visited vertices,
/// and a mutable Vec as a container to store the order in which vertices are visited. The function returns an empty Result on successful
/// completion, and a GraphError if something went wrong.
///
/// # Arguments
///
/// * `graph` - A directed graph to perform DFS on.
/// * `vertex` - A VertexId to start the DFS from.
/// * `visited` - A mutable reference to a HashSet storing visited VertexIds.
/// * `stack` - A mutable reference to a stack storing VertexIds in the order of reverse DFS traversal.
///
/// # Errors
///
/// Returns an error if the `graph` has an issue like a non-existing vertex.
///
pub fn reverse_depth_first_search(
    graph: &Graph,
    vertex: VertexId,
    visited: &mut HashSet<VertexId>,
    stack: &mut Vec<VertexId>,
) -> Result<(), GraphError> {
    if visited.contains(&vertex) {
        return Ok(());
    }

    visited.insert(vertex);

    let edges = graph.in_edges(vertex)?;
    for edge in edges {
        let src = graph.src_vertex_id(edge)?;
        reverse_depth_first_search(graph, src, visited, stack)?;
    }

    stack.push(vertex);

    Ok(())
}

/// Finds all strongly connected components in a directed graph.
///
/// This function takes a directed graph and returns a Vec of Vecs of VertexIds, where each Vec of VertexIds is a strongly connected component.
///
/// # Arguments
///
/// * `graph` - A directed graph to find strongly connected components in.
///
/// # Errors
///
/// Returns an error if the `graph` has an issue like a non-existing vertex.
///
pub fn all_strongly_connected_componenets(graph: &Graph) -> Result<Vec<Vec<VertexId>>, GraphError> {
    let mut visited: HashSet<VertexId> = HashSet::new();
    let mut container: Vec<VertexId> = Vec::new();

    let mut result: Vec<Vec<VertexId>> = Vec::new();

    for vertex_id in graph.vertex_ids() {
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

/// Finds the largest strongly connected component in a directed graph.
///
/// This function takes a directed graph and returns a Vec of VertexIds, where each VertexId is a vertex in the largest strongly connected component.
///
/// # Arguments
///
/// * `graph` - A directed graph to find the largest strongly connected component in.
///
/// # Errors
///
/// Returns an error if the `graph` has an issue like a non-existing vertex.
///
pub fn largest_strongly_connected_component(graph: &Graph) -> Result<Vec<VertexId>, GraphError> {
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
    use super::*;
    use crate::model::property::{edge::Edge, vertex::Vertex};
    use std::collections::HashMap;

    fn build_mock_graph() -> Graph {
        let vertices = vec![
            Vertex::new(0, 0.0, 0.0),
            Vertex::new(1, 1.0, 1.0),
            Vertex::new(2, 2.0, 2.0),
            Vertex::new(3, 3.0, 3.0),
            Vertex::new(4, 4.0, 4.0),
        ];

        let edges = vec![
            Edge::new(0, 0, 1, 10.0),
            Edge::new(1, 1, 0, 10.0),
            Edge::new(2, 1, 2, 10.0),
            Edge::new(3, 2, 1, 10.0),
            Edge::new(4, 2, 3, 10.0),
            Edge::new(5, 3, 2, 10.0),
            Edge::new(6, 3, 0, 10.0),
            Edge::new(7, 0, 3, 10.0),
            Edge::new(8, 0, 2, 10.0),
            Edge::new(9, 1, 3, 10.0),
            Edge::new(10, 2, 0, 10.0),
            Edge::new(11, 3, 1, 10.0),
            Edge::new(12, 4, 4, 10.0),
        ];

        // Create the adjacency and reverse adjacency lists.
        let mut adj = vec![HashMap::new(); vertices.len()];
        let mut rev = vec![HashMap::new(); vertices.len()];

        for edge in &edges {
            adj[edge.src_vertex_id.0].insert(edge.edge_id, edge.dst_vertex_id);
            rev[edge.dst_vertex_id.0].insert(edge.edge_id, edge.src_vertex_id);
        }

        // Construct the Graph instance.

        Graph {
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
            vertices: vertices.into_boxed_slice(),
        }
    }

    #[test]
    fn test_largest_strongly_connected_component() {
        let graph = build_mock_graph();
        let component = largest_strongly_connected_component(&graph).unwrap();
        assert_eq!(component.len(), 4);
        assert!(component.contains(&VertexId(0)));
        assert!(component.contains(&VertexId(1)));
        assert!(component.contains(&VertexId(2)));
        assert!(component.contains(&VertexId(3)));
    }

    #[test]
    fn test_all_strongly_connected_components() {
        let graph = build_mock_graph();
        let mut components = all_strongly_connected_componenets(&graph).unwrap();
        components.sort_by_key(|c| c.len());
        assert_eq!(components.len(), 2);
        assert_eq!(components[1].len(), 4);
        assert_eq!(components[0].len(), 1);
    }
}
