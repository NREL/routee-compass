use crate::algorithm::search::min_search_tree::direction::Direction;
use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;

use super::edge_id::EdgeId;
use super::graph_error::GraphError;
use super::vertex_id::VertexId;

pub trait DirectedGraph {
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError>;
    fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, GraphError>;
    fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError>;
    fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError>;
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError>;
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError>;

    /// helper function to give incident edges to a vertex based on a
    /// traversal direction.
    fn incident_edges(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<EdgeId>, GraphError> {
        match direction {
            Direction::Forward => self.out_edges(vertex_id),
            Direction::Reverse => self.in_edges(vertex_id),
        }
    }

    /// helper function to give the incident vertex to an edge based on a
    /// traversal direction.
    fn incident_vertex(
        &self,
        edge_id: EdgeId,
        direction: Direction,
    ) -> Result<VertexId, GraphError> {
        match direction {
            Direction::Forward => self.dst_vertex(edge_id),
            Direction::Reverse => self.src_vertex(edge_id),
        }
    }

    /// helper function to create VertexId EdgeId VertexId triplets based on
    /// a traversal direction, where the vertex_id function argument appears in
    /// the first slot and the terminal vertex id appears in the final slot
    /// of each result triplet.
    fn incident_triplets(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<(VertexId, EdgeId, VertexId)>, GraphError> {
        let edge_ids = self.incident_edges(vertex_id, direction)?;
        let mut result: Vec<(VertexId, EdgeId, VertexId)> = Vec::new();
        for edge_id in edge_ids {
            let terminal_vid = self.incident_vertex(edge_id, direction)?;
            result.push((vertex_id, edge_id, terminal_vid));
        }
        Ok(result)
    }

    fn incident_triplet_attributes(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<(Vertex, Edge, Vertex)>, GraphError> {
        let triplets = self.incident_triplets(vertex_id, direction)?;
        let mut result: Vec<(Vertex, Edge, Vertex)> = Vec::new();
        for (src_id, edge_id, dst_id) in triplets {
            let src = self.vertex_attr(src_id)?;
            let edge = self.edge_attr(edge_id)?;
            let dst = self.vertex_attr(dst_id)?;
            result.push((src, edge, dst));
        }
        Ok(result)
    }
}
