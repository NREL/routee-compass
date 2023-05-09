use std::cmp;
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
struct Tarjan {
    index: u32,
    indexes: HashMap<NodeId, u32>,
    lowlinks: HashMap<NodeId, u32>,
    on_stack: HashSet<NodeId>,
    stack: Vec<NodeId>,
    scc: Vec<Vec<NodeId>>,
}

impl Tarjan {
    fn new() -> Self {
        Self {
            index: 0,
            indexes: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            stack: Vec::new(),
            scc: Vec::new(),
        }
    }

    fn strongconnect(&mut self, graph: &Graph, v: NodeId) {
        self.indexes.insert(v, self.index);
        self.lowlinks.insert(v, self.index);
        self.index += 1;
        self.stack.push(v);
        self.on_stack.insert(v);

        if let Some(neighbors) = graph.adjacency_list.get(&v) {
            for link in neighbors {
                let w = link.end_node;
                if !self.indexes.contains_key(&w) {
                    self.strongconnect(graph, w);
                    let v_lowlink = *self.lowlinks.get(&v).unwrap();
                    let w_lowlink = *self.lowlinks.get(&w).unwrap();
                    self.lowlinks.insert(v, cmp::min(v_lowlink, w_lowlink));
                } else if self.on_stack.contains(&w) {
                    let v_lowlink = *self.lowlinks.get(&v).unwrap();
                    let w_index = *self.indexes.get(&w).unwrap();
                    self.lowlinks.insert(v, cmp::min(v_lowlink, w_index));
                }
            }
        }

        if self.lowlinks[&v] == self.indexes[&v] {
            let mut component = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack.remove(&w);
                component.push(w);
                if w == v {
                    break;
                }
            }
            self.scc.push(component);
        }
    }

    fn run(&mut self, graph: &Graph) {
        for node in graph.nodes.keys() {
            if !self.indexes.contains_key(node) {
                self.strongconnect(graph, *node);
            }
        }
    }

    fn largest_scc(&self) -> Option<&Vec<NodeId>> {
        self.scc.iter().max_by_key(|scc| scc.len())
    }
}

#[pyfunction]
pub fn extract_largest_scc(graph: &Graph) -> Option<Graph> {
    let mut tarjan = Tarjan::new();
    tarjan.run(graph);
    let largest_scc = tarjan.largest_scc()?;
    let mut new_graph = Graph {
        nodes: HashMap::new(),
        adjacency_list: HashMap::new(),
    };
    for &node_id in largest_scc {
        if let Some(node) = graph.nodes.get(&node_id) {
            new_graph.nodes.insert(node_id, node.clone());
            if let Some(links) = graph.adjacency_list.get(&node_id) {
                let mut new_links = Vec::new();
                for link in links {
                    if largest_scc.contains(&link.start_node)
                        && largest_scc.contains(&link.end_node)
                    {
                        new_links.push(link.clone());
                    }
                }
                new_graph.adjacency_list.insert(node_id, new_links);
            }
        }
    }
    Some(new_graph)
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
        assert!(extract_largest_scc(&g).is_none());
    }

    #[test]
    fn test_single_node_graph() {
        let mut g = Graph {
            nodes: HashMap::new(),
            adjacency_list: HashMap::new(),
        };
        g.nodes.insert(1, Node { id: 1, x: 0, y: 0 });
        let scc = extract_largest_scc(&g).unwrap();
        assert_eq!(scc.nodes.len(), 1);
        assert_eq!(scc.adjacency_list.len(), 0);
    }

    #[test]
    fn test_disconnected_graph() {
        let mut g = Graph {
            nodes: HashMap::new(),
            adjacency_list: HashMap::new(),
        };
        g.add_node(Node { id: 1, x: 0, y: 0 });
        g.add_node(Node { id: 2, x: 1, y: 1 });
        let scc = extract_largest_scc(&g).unwrap();
        assert_eq!(scc.nodes.len(), 1); // Only one node per SCC in a disconnected graph
        assert_eq!(scc.adjacency_list.len(), 0);
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
