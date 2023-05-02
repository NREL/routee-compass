use std::collections::{BinaryHeap, HashMap};
use std::{cmp::Reverse, collections::HashSet};

use crate::prototype::graph::{Graph, Link, Node};
use crate::prototype::powertrain::VehicleParameters;

use pyo3::prelude::*;

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
    start: &Node,
    end: &Node,
    cost_function: impl Fn(&Link) -> u32,
    restriction_function: impl Fn(&Link) -> bool,
) -> Option<(u32, Vec<Node>)> {
    let mut visited = HashSet::new();
    let mut min_heap = BinaryHeap::new();
    let mut parents: HashMap<Node, Node> = HashMap::new();
    let mut distances: HashMap<Node, u32> = HashMap::new();

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

pub fn dfs(graph: &Graph, node: &Node, visited: &mut HashSet<Node>, stack: &mut Vec<Node>) {
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
    node: &Node,
    visited: &mut HashSet<Node>,
    scc: &mut HashSet<Node>,
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
pub fn largest_scc(graph: &Graph) -> Graph {
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
    for node in &largest_scc {
        if let Some(links) = graph.adjacency_list.get(node) {
            for link in links {
                if largest_scc.contains(&link.end_node) {
                    largest_scc_graph.add_link(link.clone());
                }
            }
        }
    }

    largest_scc_graph
}
