use geo_types::Coord;
use routee_compass_core::model::{property::vertex::Vertex, road_network::graph::Graph};
use rstar::RTree;

use super::vertex_r_tree_object::VertexRTreeObject;

pub struct VertexRTree {
    rtree: RTree<VertexRTreeObject>,
}

impl VertexRTree {
    pub fn new(vertices: Vec<Vertex>) -> Self {
        let rtree_vertices: Vec<VertexRTreeObject> =
            vertices.into_iter().map(VertexRTreeObject::new).collect();
        let rtree = RTree::bulk_load(rtree_vertices);
        Self { rtree }
    }

    pub fn from_directed_graph(graph: &Graph) -> Self {
        let vertices = graph.vertices.to_vec();
        Self::new(vertices)
    }

    pub fn nearest_vertex(&self, point: Coord<f64>) -> Option<&Vertex> {
        match self.rtree.nearest_neighbor(&point) {
            Some(rtree_vertex) => Some(&rtree_vertex.vertex),
            None => None,
        }
    }

    pub fn nearest_vertices(&self, point: Coord<f64>, n: usize) -> Vec<&Vertex> {
        self.rtree
            .nearest_neighbor_iter(&point)
            .take(n)
            .map(|rtv| &rtv.vertex)
            .collect()
    }
}
