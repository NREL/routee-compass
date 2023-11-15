use crate::algorithm::search::direction::Direction;
use crate::model::property::edge::Edge;
use crate::model::property::vertex::Vertex;
use crate::model::road_network::graph_error::GraphError;
use crate::model::road_network::{edge_id::EdgeId, vertex_id::VertexId};
use std::collections::HashMap;
use std::path::Path;

use super::graph_loader::graph_from_files;

/// Road network topology represented as an adjacency list.
/// The `EdgeId` and `VertexId` values correspond to edge and
/// vertex indices in the `edges` and `vertices` vectors.
///
/// # Arguments
///
/// * `adj` - the forward-oriented adjacency list
/// * `rev` - the reverse-oriented adjacency list
/// * `edges` - for each `EdgeId`, the corresponding `Edge` record
/// * `vertices` - for each `VertexId`, the corresponding `Vertex` record
///
/// # Performance
///
/// Methods provided via the `Graph` type prefer avoiding copies.
/// Operations on a single entity should be _O(1)_. Most methods returning
/// collections will prefer chained iterators. A few will collect
/// into Vecs because of error handling or lifetimes, but those cases will only produce a
/// smaller subset of the source data.

#[derive(Debug)]
pub struct Graph {
    pub adj: Box<[HashMap<EdgeId, VertexId>]>,
    pub rev: Box<[HashMap<EdgeId, VertexId>]>,
    pub edges: Box<[Edge]>,
    pub vertices: Box<[Vertex]>,
}

impl Graph {
    /// Build a `Graph` from a pair of CSV files.
    /// You can also pass in the number of edges and vertices to avoid
    /// scanning the input files and potentially building vectors that
    /// have more memory than needed.
    ///
    /// # Arguments
    ///
    /// * `edge_list_csv` - path to the CSV file containing edge attributes
    /// * `vertex_list_csv` - path to the CSV file containing vertex attributes
    /// * `n_edges` - number of edges in the graph
    /// * `n_vertices` - number of vertices in the graph
    /// * `verbose` - whether to print progress information to the console
    ///
    /// # Returns
    ///
    /// A graph instance, or an error if an IO error occurred.
    ///
    pub fn from_files<P: AsRef<Path>>(
        edge_list_csv: &P,
        vertex_list_csv: &P,
        n_edges: Option<usize>,
        n_vertices: Option<usize>,
        verbose: Option<bool>,
    ) -> Result<Graph, GraphError> {
        graph_from_files(edge_list_csv, vertex_list_csv, n_edges, n_vertices, verbose)
    }
    /// number of edges in the Graph
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }

    /// number of vertices in the Graph
    pub fn n_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// helper function for creating a range of all edge ids in the graph.
    /// uses the knowledge that all ids are unique and consecutive integers
    /// beginning at zero.
    pub fn edge_ids(&self) -> Box<dyn Iterator<Item = EdgeId>> {
        let range = (0..self.n_edges()).map(EdgeId);
        Box::new(range)
    }

    /// helper function for creating a range of all vertex ids in the graph.
    /// uses the knowledge that all ids are unique and consecutive integers
    /// beginning at zero.
    pub fn vertex_ids(&self) -> Box<dyn Iterator<Item = VertexId>> {
        let range = (0..self.n_vertices()).map(VertexId);
        Box::new(range)
    }

    /// retrieve an `Edge` record from the graph
    ///
    /// # Arguments
    ///
    /// * `edge_id` - the `EdgeId` for the `Edge` that we want to retrieve
    ///
    /// # Returns
    ///
    /// The associated `Edge` or an error if the id is missing
    pub fn get_edge(&self, edge_id: EdgeId) -> Result<&Edge, GraphError> {
        match self.edges.get(edge_id.0) {
            None => Err(GraphError::EdgeAttributeNotFound { edge_id }),
            Some(edge) => Ok(edge),
        }
    }

    /// retrieve a `Vertex` record from the graph
    ///
    /// # Arguments
    ///
    /// * `vertex_id` - the `VertexId` for the `Vertex` that we want to retrieve
    ///
    /// # Returns
    ///
    /// The associated `Vertex` or an error if the id is missing
    pub fn get_vertex(&self, vertex_id: VertexId) -> Result<&Vertex, GraphError> {
        match self.vertices.get(vertex_id.0) {
            None => Err(GraphError::VertexAttributeNotFound { vertex_id }),
            Some(vertex) => Ok(vertex),
        }
    }

    /// retrieve a list of `EdgeId`s for edges that depart from the given `VertexId`
    ///
    /// # Arguments
    ///
    /// * `src` - the `VertexId` for the source vertex of edges
    ///
    /// # Returns
    ///
    /// A list of `EdgeIds` for outbound edges that leave this `VertexId`, or an error
    /// if the vertex is missing from the Graph adjacency matrix.
    pub fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.adj.get(src.0) {
            None => Err(GraphError::VertexWithoutOutEdges { vertex_id: src }),
            Some(out_map) => {
                let edge_ids = out_map.keys().cloned().collect();
                Ok(edge_ids)
            }
        }
    }

    /// retrieve a list of `EdgeId`s for edges that arrive at the given `VertexId`
    ///
    /// # Arguments
    ///
    /// * `dst` - the `VertexId` for the destination vertex of edges
    ///
    /// # Returns
    ///
    /// A list of `EdgeIds` for inbound edges that arrive at this `VertexId`, or an error
    /// if the vertex is missing from the Graph adjacency matrix.
    pub fn in_edges(&self, dst: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.rev.get(dst.0) {
            None => Err(GraphError::VertexWithoutInEdges { vertex_id: dst }),
            Some(in_map) => {
                let edge_ids = in_map.keys().cloned().collect();
                Ok(edge_ids)
            }
        }
    }

    /// retrieve the source vertex id of an edge
    ///
    /// # Arguments
    ///
    /// * `edge_id` - the edge to find a source vertex id
    ///
    /// # Returns
    ///
    /// The source `VertexId` of an `Edge` or an error if the edge is missing
    pub fn src_vertex_id(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.get_edge(edge_id).map(|e| e.src_vertex_id)
    }

    /// retrieve the destination vertex id of an edge
    ///
    /// # Arguments
    ///
    /// * `edge_id` - the edge to find a destination vertex id
    ///
    /// # Returns
    ///
    /// The destination `VertexId` of an `Edge` or an error if the edge is missing
    pub fn dst_vertex_id(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.get_edge(edge_id).map(|e| e.dst_vertex_id)
    }

    /// helper function to give incident edges to a vertex based on a
    /// traversal direction.
    ///
    /// # Arguments
    ///
    /// * `vertex_id` - vertex to find edges which connect to it
    /// * `direction` - whether to find out edges (Forward) or in edges (Reverse)
    ///
    /// # Returns
    ///
    /// The incident `EdgeId`s or an error if the vertex is not connected.
    pub fn incident_edges(
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
    ///
    /// # Arguments
    ///
    /// * `edge_id` - edge to find the vertex which connects to it
    /// * `direction` - whether to find the destination (Forward) or source (Reverse) vertex
    ///
    /// # Returns
    ///
    /// The incident `VertexId` of an edge or an error if the edge is missing
    pub fn incident_vertex(
        &self,
        edge_id: EdgeId,
        direction: Direction,
    ) -> Result<VertexId, GraphError> {
        match direction {
            Direction::Forward => self.dst_vertex_id(edge_id),
            Direction::Reverse => self.src_vertex_id(edge_id),
        }
    }

    /// retrieve the triplet of `Vertex` -> `Edge` -> `Vertex` for some `EdgeId`
    ///
    /// # Arguments
    ///
    /// * `edge_id` - the id of the edge to collect attributes for
    ///
    /// # returns
    ///
    /// The triplet of attributes surrounding one `Edge` or an error if
    /// any id is invalid.
    pub fn edge_triplet_attrs(
        &self,
        edge_id: EdgeId,
    ) -> Result<(&Vertex, &Edge, &Vertex), GraphError> {
        let edge = self.get_edge(edge_id)?;
        let src = self.get_vertex(edge.src_vertex_id)?;
        let dst = self.get_vertex(edge.dst_vertex_id)?;

        Ok((src, edge, dst))
    }

    /// creates `VertexId` -> `EdgeId` -> `VertexId` triplets based on
    /// a `VertexId` and a traversal `Direction`.
    ///
    /// Regardless of the direction chosen, the source `VertexId` appears first and the
    /// terminal `VertexId` appears in the third slot.
    ///
    /// # Arguments
    ///
    /// * `vertex_id` - id of the vertex to lookup incident triplets
    /// * `direction` - direction to traverse to/from the `vertex_id`
    ///
    /// # Returns
    ///
    /// For each edge connected to this `vertex_id` traversable via the provided `direction`,
    /// a triplet of the `EdgeId` and it's connecting `VertexId`s.
    pub fn incident_triplets(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<(VertexId, EdgeId, VertexId)>, GraphError> {
        self.incident_edges(vertex_id, direction)?
            .into_iter()
            .map(|edge_id| {
                let terminal_vid = self.incident_vertex(edge_id, direction)?;
                Ok((vertex_id, edge_id, terminal_vid))
            })
            .collect()
    }

    /// creates `Vertex` -> `Edge` -> `Vertex` triplets based on
    /// a `Vertex` and a traversal `Direction`.
    ///
    /// Regardless of the direction chosen, the source `Vertex` appears first and the
    /// terminal `Vertex` appears in the third slot.
    ///
    /// # Arguments
    ///
    /// * `vertex_id` - id of the vertex to lookup incident triplets
    /// * `direction` - direction to traverse to/from the `vertex_id`
    ///
    /// # Returns
    ///
    /// For each edge connected to this `vertex_id` traversable via the provided `direction`,
    /// a triplet of the `Edge` and it's connecting `Vertex`s.
    pub fn incident_triplet_attributes(
        &self,
        vertex_id: VertexId,
        direction: Direction,
    ) -> Result<Vec<(&Vertex, &Edge, &Vertex)>, GraphError> {
        self.incident_triplets(vertex_id, direction)?
            .iter()
            .map(|(src_id, edge_id, dst_id)| {
                let src = self.get_vertex(*src_id)?;
                let edge = self.get_edge(*edge_id)?;
                let dst = self.get_vertex(*dst_id)?;
                Ok((src, edge, dst))
            })
            .collect()
    }
}
