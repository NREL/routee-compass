use std::collections::HashMap;

use crate::model::{
    graph::{
        directed_graph::DirectedGraph, edge_id::EdgeId, graph_error::GraphError,
        vertex_id::VertexId,
    },
    property::{edge::Edge, vertex::Vertex},
};

struct DefaultDirectedGraph {
    pub vertices: HashMap<VertexId, Vertex>,
    pub edges: HashMap<EdgeId, Edge>,
    pub adjacency_list: HashMap<VertexId, HashMap<EdgeId, VertexId>>,
}

impl DirectedGraph for DefaultDirectedGraph {
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError> {
        match self.edges.get(&edge_id) {
            Some(edge) => Ok(*edge),
            None => Err(GraphError::EdgeIdNotFound { edge_id }),
        }
    }
    fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, GraphError> {
        match self.vertices.get(&vertex_id) {
            Some(vertex) => Ok(*vertex),
            None => Err(GraphError::VertexIdNotFound { vertex_id }),
        }
    }
    fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.adjacency_list.get(&src) {
            Some(edges) => Ok(edges.keys().map(|edge_id| *edge_id).collect()),
            None => Err(GraphError::VertexIdNotFound { vertex_id: src }),
        }
    }
    fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        todo!()
    }
    fn src_vertex(
        &self,
        edge_id: crate::model::graph::edge_id::EdgeId,
    ) -> Result<VertexId, crate::model::graph::graph_error::GraphError> {
        self.edge_attr(edge_id).map(|edge| edge.start_vertex)
    }
    fn dst_vertex(
        &self,
        edge_id: crate::model::graph::edge_id::EdgeId,
    ) -> Result<VertexId, crate::model::graph::graph_error::GraphError> {
        self.edge_attr(edge_id).map(|edge| edge.end_vertex)
    }
}
