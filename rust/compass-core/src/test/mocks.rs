use std::collections::HashMap;

use crate::model::units::{Length, Ratio, Velocity};
use geo::coord;
use uom::si;

use crate::model::{
    graph::{
        directed_graph::DirectedGraph, edge_id::EdgeId, graph_error::GraphError,
        vertex_id::VertexId,
    },
    property::{edge::Edge, road_class::RoadClass, vertex::Vertex},
};

#[cfg(test)]
pub struct TestDG {
    adj: HashMap<VertexId, HashMap<EdgeId, VertexId>>,
    rev: HashMap<VertexId, HashMap<EdgeId, VertexId>>,
    edges: HashMap<EdgeId, Edge>,
}

#[cfg(test)]
impl Default for TestDG {
    fn default() -> Self {
        TestDG {
            adj: HashMap::new(),
            rev: HashMap::new(),
            edges: HashMap::new(),
        }
    }
}
#[cfg(test)]
impl DirectedGraph for TestDG {
    fn all_edge_ids(&self) -> Vec<EdgeId> {
        self.edges.keys().cloned().collect()
    }
    fn all_vertex_ids(&self) -> Vec<VertexId> {
        self.adj.keys().cloned().collect()
    }
    fn all_edges(&self) -> Vec<Edge> {
        self.edges.values().cloned().collect()
    }
    fn all_vertices(&self) -> Vec<Vertex> {
        self.adj
            .keys()
            .map(|v| Vertex {
                vertex_id: *v,
                coordinate: coord! {x: 0.0, y: 0.0},
            })
            .collect()
    }
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError> {
        match self.edges.get(&edge_id) {
            None => Err(GraphError::EdgeAttributeNotFound { edge_id }),
            Some(edge) => Ok(*edge),
        }
    }
    fn vertex_attr(&self, _vertex_id: VertexId) -> Result<Vertex, GraphError> {
        Ok(Vertex {
            vertex_id: VertexId(0),
            coordinate: coord! {x: 0.0, y: 0.0},
        })
    }
    fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.adj.get(&src) {
            None => Err(GraphError::VertexWithoutOutEdges { vertex_id: src }),
            Some(out_map) => {
                let edges = out_map.keys().cloned().collect();
                Ok(edges)
            }
        }
    }
    fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.rev.get(&src) {
            None => Err(GraphError::VertexWithoutInEdges { vertex_id: src }),
            Some(out_map) => {
                let edges = out_map.keys().cloned().collect();
                Ok(edges)
            }
        }
    }
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e| e.src_vertex_id)
    }
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e: Edge| e.dst_vertex_id)
    }
}

#[cfg(test)]
impl TestDG {
    pub fn new(
        adj: HashMap<VertexId, HashMap<EdgeId, VertexId>>,
        lengths: HashMap<EdgeId, Length>,
    ) -> Result<TestDG, GraphError> {
        let mut edges: HashMap<EdgeId, Edge> = HashMap::new();
        for (src, out_edges) in &adj {
            for (edge_id, dst) in out_edges {
                let length = lengths
                    .get(edge_id)
                    .ok_or(GraphError::EdgeIdNotFound { edge_id: *edge_id })?;
                let edge = Edge {
                    edge_id: edge_id.clone(),
                    src_vertex_id: src.clone(),
                    dst_vertex_id: dst.clone(),
                    road_class: RoadClass(0),
                    distance: length.clone(),
                    grade: Ratio::new::<si::ratio::per_mille>(0.0),
                };
                edges.insert(edge_id.clone(), edge);
            }
        }
        let mut rev: HashMap<VertexId, HashMap<EdgeId, VertexId>> = HashMap::new();
        for (src, out_edges) in &adj {
            for (edge_id, dst) in out_edges {
                if rev.contains_key(dst) {
                    rev.get_mut(dst)
                        .unwrap()
                        .insert(edge_id.clone(), src.clone());
                } else {
                    let mut new_map: HashMap<EdgeId, VertexId> = HashMap::new();
                    new_map.insert(edge_id.clone(), src.clone());
                    rev.insert(dst.clone(), new_map);
                }
            }
        }

        Ok(TestDG { adj, rev, edges })
    }
}
