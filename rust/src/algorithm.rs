use pyo3::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::{cmp::Reverse, collections::HashSet};

use crate::graph::{Graph, Link, Node};

pub type CostFunction = fn(&Link) -> u32;

#[pyfunction]
pub fn py_time_shortest_path(graph: &Graph, start: &Node, end: &Node) -> Option<(u32, Vec<Node>)> {
    let cost_function_time = |link: &Link| link.time;
    dijkstra_shortest_path(graph, start, end, cost_function_time)
}
pub fn dijkstra_shortest_path(
    graph: &Graph,
    start: &Node,
    end: &Node,
    cost_function: CostFunction,
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
