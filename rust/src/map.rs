use rstar::RTree;

use pyo3::{prelude::*, types::PyType};
use anyhow::Result;

use crate::{
    algorithm::dijkstra_shortest_path,
    graph::{Graph, Node},
};

#[pyclass]
pub struct RustMap {
    pub graph: Graph,
    pub rtree: RTree<Node>,
}

#[pymethods]
impl RustMap {
    #[new]
    pub fn new(graph: Graph) -> Self {
        let rtree = RTree::bulk_load(graph.get_nodes().clone());
        RustMap { graph, rtree }
    }

    pub fn to_file(&self, path: &str) -> Result<()> {
        self.graph.to_file(path)
    }

    #[classmethod]
    pub fn from_file(_: &PyType, path: &str) -> Result<Self> {
        let graph = Graph::from_file(path)?;
        Ok(RustMap::new(graph))
    }

    pub fn get_closest_node(&self, point: [isize; 2]) -> Option<Node> {
        self.rtree.nearest_neighbor(&point).cloned()
    }

    pub fn shortest_path(&self, start: [isize; 2], end: [isize; 2]) -> Option<Vec<Node>> {
        let start_node = self.get_closest_node(start)?;
        let end_node = self.get_closest_node(end)?;
        dijkstra_shortest_path(&self.graph, &start_node, &end_node, |link| link.time)
            .map(|(_, path)| path)
    }
}
