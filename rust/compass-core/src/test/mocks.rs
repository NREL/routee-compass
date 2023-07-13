use std::collections::HashMap;

use crate::model::{
    graph::{
        directed_graph::DirectedGraph, edge_id::EdgeId, graph_error::GraphError,
        vertex_id::VertexId,
    },
    property::{edge::Edge, vertex::Vertex, road_class::RoadClass},
    units::{ordinate::Ordinate, cm_per_second::CmPerSecond, centimeters::Centimeters, millis::Millis},
};

pub struct TestDG<'a> {
    adj: &'a HashMap<VertexId, HashMap<EdgeId, VertexId>>,
    edges: HashMap<EdgeId, Edge>,
}
impl DirectedGraph for TestDG<'_> {
    fn all_edge_ids(&self) -> Vec<EdgeId> {
        self.edges.keys().cloned().collect()
    }
    fn all_vertex_ids(&self) -> Vec<VertexId> {
        self.adj.keys().cloned().collect()
    }
    fn all_edges(&self) -> Vec<Edge> {
        self.edges.values().cloned().collect()
    }
    fn all_verticies(&self) -> Vec<Vertex> {
        self.adj
            .keys()
            .map(|v| Vertex {
                vertex_id: *v,
                x: Ordinate(0.0),
                y: Ordinate(0.0),
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
            x: Ordinate(0.0),
            y: Ordinate(0.0),
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
    fn in_edges(&self, _src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        Err(GraphError::TestError) // not used
    }
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e| e.src_vertex_id)
    }
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e: Edge| e.dst_vertex_id)
    }
}

impl<'a> TestDG<'a> {
    pub fn new(
        adj: &'a HashMap<VertexId, HashMap<EdgeId, VertexId>>,
        edges_cps: HashMap<EdgeId, CmPerSecond>,
    ) -> Result<TestDG<'a>, GraphError> {
        let mut edges: HashMap<EdgeId, Edge> = HashMap::new();
        for (src, out_edges) in adj {
            for (edge_id, dst) in out_edges {
                let cps = edges_cps.get(&edge_id).ok_or(GraphError::EdgeIdNotFound {
                    edge_id: edge_id.clone(),
                })?;
                let edge = Edge {
                    edge_id: edge_id.clone(),
                    src_vertex_id: src.clone(),
                    dst_vertex_id: dst.clone(),
                    road_class: RoadClass(0),
                    free_flow_speed_cps: cps.clone(),
                    distance_centimeters: Centimeters(100),
                    grade_millis: Millis(0),
                };
                edges.insert(edge_id.clone(), edge);
            }
        }

        Ok(TestDG { adj, edges })
    }
}
