use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;

use pyo3::prelude::*;

use anyhow::Result;
use bincode;
use pyo3::types::PyType;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Restriction {
    pub weight_limit_lbs: u32,
    pub height_limit_feet: u8,
    pub width_limit_feet: u8,
    pub length_limit_feet: u8,
}

#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Node {
    #[pyo3(get)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Serialize, Deserialize)]
pub struct Link {
    #[pyo3(get)]
    pub start_node: Node,
    #[pyo3(get)]
    pub end_node: Node,
    #[pyo3(get)]
    pub road_class: u8,
    #[pyo3(get)]
    pub time: u32,
    #[pyo3(get)]
    pub distance: u32,
    #[pyo3(get)]
    pub grade: i16,
    #[pyo3(get)]
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
#[derive(Serialize, Deserialize)]
pub struct Graph {
    #[pyo3(get)]
    adjacency_list: HashMap<Node, HashSet<Link>>,
}

impl Graph {
    pub fn neighbors(&self, node: &Node) -> Option<&HashSet<Link>> {
        self.adjacency_list.get(node)
    }
    pub fn to_binary(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
    pub fn from_binary(binary: &[u8]) -> Self {
        bincode::deserialize(binary).unwrap()
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

    pub fn to_file(&self, filename: &str) -> Result<()> {
        let path = PathBuf::from(filename);
        let mut file = std::fs::File::create(path)?;
        bincode::serialize_into(&mut file, &self)?;
        Ok(())
    }

    #[classmethod]
    pub fn from_file(_: &PyType, filename: &str) -> Result<Self> {
        let path = PathBuf::from(filename);
        let file = std::fs::File::open(path)?;
        let graph = bincode::deserialize_from(file)?;
        Ok(graph)
    }

    pub fn add_edge(&mut self, link: Link) {
        self.adjacency_list
            .entry(link.start_node)
            .or_insert_with(HashSet::new)
            .insert(link);
    }
}
