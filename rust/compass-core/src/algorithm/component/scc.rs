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
    use std::collections::HashMap;

    use super::*;
    use crate::{
        model::{graph::edge_id::EdgeId, units::cm_per_second::CmPerSecond},
        test::mocks::TestDG,
    };

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
        let speeds = HashMap::from([
            (EdgeId(0), CmPerSecond(10)),
            (EdgeId(1), CmPerSecond(10)),
            (EdgeId(2), CmPerSecond(10)),
            (EdgeId(3), CmPerSecond(10)),
            (EdgeId(4), CmPerSecond(10)),
            (EdgeId(5), CmPerSecond(10)),
            (EdgeId(6), CmPerSecond(10)),
            (EdgeId(7), CmPerSecond(10)),
            (EdgeId(8), CmPerSecond(10)),
            (EdgeId(9), CmPerSecond(10)),
            (EdgeId(10), CmPerSecond(10)),
            (EdgeId(11), CmPerSecond(10)),
            (EdgeId(12), CmPerSecond(10)),
        ]);

        let graph = TestDG::new(adj, speeds).unwrap();
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
