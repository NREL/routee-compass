use std::collections::HashSet;

use crate::model::graph::directed_graph::DirectedGraph;
use crate::model::graph::{graph_error::GraphError, vertex_id::VertexId};

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
    graph: &impl DirectedGraph,
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
        let dst = graph.dst_vertex(edge)?;
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
fn reverse_depth_first_search(
    graph: &impl DirectedGraph,
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
        let src = graph.src_vertex(edge)?;
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
    use std::collections::HashMap;

    use crate::model::units::Length;
    use uom::si::length::centimeter;

    use super::*;
    use crate::{model::graph::edge_id::EdgeId, test::mocks::TestDG};

    fn build_mock_graph() -> impl DirectedGraph {
        // A test graph with 2 strongly connected components
        let adj = HashMap::from([
            (
                VertexId(0),
                HashMap::from([
                    (EdgeId(0), VertexId(1)),
                    (EdgeId(7), VertexId(3)),
                    (EdgeId(8), VertexId(2)),
                ]),
            ),
            (
                VertexId(1),
                HashMap::from([
                    (EdgeId(1), VertexId(0)),
                    (EdgeId(2), VertexId(2)),
                    (EdgeId(9), VertexId(3)),
                ]),
            ),
            (
                VertexId(2),
                HashMap::from([
                    (EdgeId(3), VertexId(1)),
                    (EdgeId(4), VertexId(3)),
                    (EdgeId(10), VertexId(0)),
                ]),
            ),
            (
                VertexId(3),
                HashMap::from([
                    (EdgeId(5), VertexId(2)),
                    (EdgeId(6), VertexId(0)),
                    (EdgeId(11), VertexId(1)),
                ]),
            ),
            (
                VertexId(4),
                HashMap::from([(EdgeId(12), VertexId(4))]), // self-loop for the disjoint node
            ),
        ]);

        let lengths = HashMap::from([
            (EdgeId(0), Length::new::<centimeter>(10.0)),
            (EdgeId(1), Length::new::<centimeter>(10.0)),
            (EdgeId(2), Length::new::<centimeter>(10.0)),
            (EdgeId(3), Length::new::<centimeter>(10.0)),
            (EdgeId(4), Length::new::<centimeter>(10.0)),
            (EdgeId(5), Length::new::<centimeter>(10.0)),
            (EdgeId(6), Length::new::<centimeter>(10.0)),
            (EdgeId(7), Length::new::<centimeter>(10.0)),
            (EdgeId(8), Length::new::<centimeter>(10.0)),
            (EdgeId(9), Length::new::<centimeter>(10.0)),
            (EdgeId(10), Length::new::<centimeter>(10.0)),
            (EdgeId(11), Length::new::<centimeter>(10.0)),
            (EdgeId(12), Length::new::<centimeter>(10.0)),
        ]);

        let graph = TestDG::new(adj, lengths).unwrap();
        graph
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
