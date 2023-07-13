use geo::Coord;
use rstar::RTree;

use crate::model::{graph::directed_graph::DirectedGraph, property::vertex::Vertex};

pub struct VertexRTree {
    rtree: RTree<Vertex>,
}

impl VertexRTree {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        let rtree = RTree::bulk_load(vertices);
        Self { rtree }
    }

    pub fn from_directed_graph(graph: &impl DirectedGraph) -> Self {
        Self::new(graph.all_vertices())
    }

    pub fn nearest_vertex(&self, point: Coord<f64>) -> Option<&Vertex> {
        self.rtree.nearest_neighbor(&point)
    }

    pub fn nearest_vertices(&self, point: Coord<f64>, n: usize) -> Vec<&Vertex> {
        self.rtree.nearest_neighbor_iter(&point).take(n).collect()
    }
}
