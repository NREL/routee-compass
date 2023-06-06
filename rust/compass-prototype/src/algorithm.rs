use std::collections::{BinaryHeap, HashMap};
use std::{cmp::Reverse, collections::HashSet};

use crate::graph::{Graph, Link, NodeId};
use crate::powertrain::VehicleParameters;

use pyo3::prelude::*;

pub fn build_restriction_function(
    vehicle_parameters: Option<VehicleParameters>,
) -> impl Fn(&Link) -> bool {
    move |link: &Link| {
        if let Some(vehicle) = &vehicle_parameters {
            // NOTE: not currently using weight limit
            // let over_weight_limit = match link.weight_limit_lbs {
            //     Some(limit) => vehicle.weight_lbs > limit,
            //     None => false,
            // };
            let over_height_limit = match link.height_limit_inches {
                Some(limit) => vehicle.height_inches > limit,
                None => false,
            };
            let over_width_limit = match link.width_limit_inches {
                Some(limit) => vehicle.width_inches > limit,
                None => false,
            };
            let over_length_limit = match link.length_limit_inches {
                Some(limit) => vehicle.length_inches > limit,
                None => false,
            };
            over_height_limit || over_width_limit || over_length_limit
        } else {
            false
        }
    }
}

pub fn dijkstra_shortest_path(
    graph: &Graph,
    start: &NodeId,
    end: &NodeId,
    cost_function: impl Fn(&Link) -> u32,
    restriction_function: impl Fn(&Link) -> bool,
) -> Option<(u32, Vec<NodeId>)> {
    let mut visited = HashSet::new();
    let mut min_heap = BinaryHeap::new();
    let mut parents: HashMap<NodeId, NodeId> = HashMap::new();
    let mut distances: HashMap<NodeId, u32> = HashMap::new();

    min_heap.push((Reverse(0), start.clone()));
    distances.insert(start.clone(), 0);

    while let Some((Reverse(cost), current)) = min_heap.pop() {
        if visited.contains(&current) {
            continue;
        }

        visited.insert(current.clone());

        if &current == end {
            let mut path = Vec::new();
            let mut current_node = end;

            while current_node != start {
                path.push(current_node.clone());
                current_node = parents.get(current_node).unwrap();
            }

            path.push(start.clone());
            path.reverse();
            return Some((cost, path));
        }

        if let Some(links) = graph.neighbors(&current) {
            for link in links {
                // Skip if the link is restricted
                if restriction_function(link) {
                    continue;
                }
                let neighbor = if current == link.start_node {
                    &link.end_node
                } else {
                    &link.start_node
                };

                let neighbor_cost = cost_function(link);
                let new_cost = cost + neighbor_cost;

                if !distances.contains_key(neighbor) || new_cost < *distances.get(neighbor).unwrap()
                {
                    distances.insert(neighbor.clone(), new_cost);
                    parents.insert(neighbor.clone(), current.clone());
                    min_heap.push((Reverse(new_cost), neighbor.clone()));
                }
            }
        }
    }

    None
}
pub fn dfs(graph: &Graph, node: &NodeId, visited: &mut HashSet<NodeId>, stack: &mut Vec<NodeId>) {
    visited.insert(node.clone());
    if let Some(links) = graph.adjacency_list.get(node) {
        for link in links {
            if !visited.contains(&link.end_node) {
                dfs(graph, &link.end_node, visited, stack);
            }
        }
    }
    stack.push(node.clone());
}

pub fn dfs_transpose(
    graph: &Graph,
    node: &NodeId,
    visited: &mut HashSet<NodeId>,
    scc: &mut HashSet<NodeId>,
) {
    visited.insert(node.clone());
    scc.insert(node.clone());
    if let Some(links) = graph.adjacency_list.get(node) {
        for link in links {
            if !visited.contains(&link.end_node) {
                dfs_transpose(graph, &link.end_node, visited, scc);
            }
        }
    }
}

#[pyfunction]
pub fn extract_largest_scc(graph: &Graph) -> Graph {
    let mut stack = Vec::new();
    let mut visited = HashSet::new();

    for node in graph.adjacency_list.keys() {
        if !visited.contains(node) {
            dfs(graph, node, &mut visited, &mut stack);
        }
    }

    let transpose = graph.get_transpose();
    visited.clear();

    let mut largest_scc = HashSet::new();
    while let Some(node) = stack.pop() {
        if !visited.contains(&node) {
            let mut scc = HashSet::new();
            dfs_transpose(&transpose, &node, &mut visited, &mut scc);
            if scc.len() > largest_scc.len() {
                largest_scc = scc;
            }
        }
    }

    let mut largest_scc_graph = Graph::new();
    for node_id in &largest_scc {
        let node = graph.nodes.get(node_id).unwrap().clone();
        largest_scc_graph.add_node(node);
        if let Some(links) = graph.adjacency_list.get(node_id) {
            for link in links {
                if largest_scc.contains(&link.end_node) {
                    largest_scc_graph.add_link(link.clone());
                }
            }
        }
    }

    largest_scc_graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prototype::graph::{Link, Node};

    fn dummy_link_from_nodes(a: NodeId, b: NodeId) -> Link {
        Link::new(
            a,
            b,
            10,
            10,
            10,
            1,
            [None, None, None, None, None, None, None],
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_largest_scc() {
        let mut graph = Graph::new();

        // build 10 nodes
        let mut nodes = Vec::new();
        for i in 0..10 {
            nodes.push(i);
            graph.add_node(Node::new(i, 0, 0))
        }

        // build a graph with two sccs, one with more 10 links and the other with 6
        for i in 1..6 {
            graph.add_link(dummy_link_from_nodes(nodes[i - 1], nodes[i]));
            graph.add_link(dummy_link_from_nodes(nodes[i], nodes[i - 1]));
        }

        for i in 7..10 {
            graph.add_link(dummy_link_from_nodes(nodes[i - 1], nodes[i]));
            graph.add_link(dummy_link_from_nodes(nodes[i], nodes[i - 1]));
        }

        let scc = extract_largest_scc(&graph);
        assert_eq!(scc.number_of_links(), 10);
    }

    #[test]
    fn test_two_node_scc() {
        let mut g = Graph {
            nodes: HashMap::new(),
            adjacency_list: HashMap::new(),
        };
        let node_a = Node { id: 1, x: 0, y: 0 };
        let node_b = Node { id: 2, x: 1, y: 1 };
        g.add_node(node_a.clone());
        g.add_node(node_b.clone());
        g.add_link(dummy_link_from_nodes(node_a.id, node_b.id));
        g.add_link(dummy_link_from_nodes(node_b.id, node_a.id));
        let scc = extract_largest_scc(&g);
        assert_eq!(scc.nodes.len(), 2); // Two nodes strongly connected
        assert_eq!(scc.adjacency_list.len(), 2);
    }
}
