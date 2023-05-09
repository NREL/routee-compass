use std::collections::{BinaryHeap, HashMap};
use std::{cmp::Reverse, collections::HashSet};

use crate::graph::{Graph, Link, NodeId};
use crate::powertrain::VehicleParameters;

use pathfinding::prelude::strongly_connected_components;
use pyo3::prelude::*;

use anyhow::{anyhow, Result};

pub fn build_restriction_function(
    vehicle_parameters: Option<VehicleParameters>,
) -> impl Fn(&Link) -> bool {
    move |link: &Link| {
        if let Some(vehicle) = &vehicle_parameters {
            let over_weight_limit = match link.weight_limit_lbs {
                Some(limit) => vehicle.weight_lbs > limit,
                None => false,
            };
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
            over_height_limit || over_weight_limit || over_width_limit || over_length_limit
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
#[pyfunction]
pub fn extract_largest_scc(graph: &Graph) -> Result<Graph> {
    let node_ids = graph.nodes.keys().cloned().collect::<Vec<_>>();
    let successors = |node_id: &NodeId| {
        if let Some(links) = graph.adjacency_list.get(node_id) {
            links.iter().map(|link| link.end_node).collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };
    let all_sccs = strongly_connected_components(&node_ids, successors);
    let largest_scc = all_sccs
        .into_iter()
        .max_by_key(|scc| scc.len())
        .ok_or(anyhow!("No SCCs found"))?;

    let mut new_graph = Graph::new();
    for node_id in largest_scc {
        let node = graph
            .nodes
            .get(&node_id)
            .ok_or(anyhow!("Node {} not found in graph", node_id))?;
        new_graph.add_node(node.clone());
        let links = graph
            .adjacency_list
            .get(&node_id)
            .ok_or(anyhow!("Node {} not found in graph", node_id))?;
        new_graph.add_links_bulk(links.clone());
    }
    Ok(new_graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{Link, Node};

    fn dummy_link_from_nodes(a: NodeId, b: NodeId) -> Link {
        Link::new(
            a,
            b,
            10,
            10,
            10,
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

        let scc = extract_largest_scc(&graph).unwrap();
        assert_eq!(scc.number_of_links(), 10);
    }

    #[test]
    fn test_empty_graph() {
        let g = Graph {
            nodes: HashMap::new(),
            adjacency_list: HashMap::new(),
        };
        // make sure the function returns an error
        assert!(extract_largest_scc(&g).is_err());
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
        let scc = extract_largest_scc(&g).unwrap();
        assert_eq!(scc.nodes.len(), 2); // Two nodes strongly connected
        assert_eq!(scc.adjacency_list.len(), 2);
    }
}
