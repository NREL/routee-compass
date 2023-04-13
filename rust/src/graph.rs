use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct Restriction {
    pub weight_limit_lbs: u32,
    pub height_limit_feet: u8,
    pub width_limit_feet: u8,
    pub length_limit_feet: u8,
}

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node {
    pub id: u32,
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(id: u32) -> Self {
        Node { id }
    }
}

#[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
pub struct Link {
    pub start_node: Node,
    pub end_node: Node,
    pub road_class: u8,
    pub time: u32,
    pub distance: u32,
    pub grade: i16,
    pub restriction: Option<Restriction>,
}

#[pymethods]
impl Link {
    #[new]
    pub fn new(
        start_node: Node,
        end_node: Node,
        road_class: u8,
        time: u32,
        distance: u32,
        grade: i16,
        restriction: Option<Restriction>,
    ) -> Self {
        Link {
            start_node,
            end_node,
            road_class,
            time,
            distance,
            grade,
            restriction,
        }
    }
}

#[pyclass]
pub struct Graph {
    #[pyo3(get)]
    adjacency_list: HashMap<Node, HashSet<Link>>,
}

impl Graph {
    pub fn neighbors(&self, node: &Node) -> Option<&HashSet<Link>> {
        self.adjacency_list.get(node)
    }
}

#[pymethods]
impl Graph {
    #[new]
    pub fn new() -> Self {
        Graph {
            adjacency_list: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, link: Link) {
        self.adjacency_list
            .entry(link.start_node)
            .or_insert_with(HashSet::new)
            .insert(link);
    }
}
