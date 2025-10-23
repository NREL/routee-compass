use std::collections::HashSet;

use super::{Edge, EdgeId, EdgeList, NetworkError, Vertex, VertexId};
use crate::algorithm::search::Direction;
use crate::model::network::EdgeListId;
use crate::model::network::GraphConfig;
use crate::util::fs::read_utils;
use indexmap::IndexMap;
use itertools::Itertools;
use kdam::Bar;

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
    pub vertices: Box<[Vertex]>,
    pub edge_lists: Vec<EdgeList>,
    pub adj: DenseAdjacencyList,
    pub rev: DenseAdjacencyList,
}

/// a graph adjacency list with an entry (possibly empty) for each VertexId in the Graph.
pub type DenseAdjacencyList = Box<[IndexMap<(EdgeListId, EdgeId), VertexId>]>;

impl TryFrom<&GraphConfig> for Graph {
    type Error = NetworkError;

    /// create a graph from a JSON argument. it should be an object that contains
    /// two keys, one for each file path.
    fn try_from(config: &GraphConfig) -> Result<Self, Self::Error> {
        let vertices: Box<[Vertex]> = read_utils::from_csv(
            &config.vertex_list_input_file,
            true,
            Some(Bar::builder().desc(format!("graph vertices: {}", config.vertex_list_input_file))),
            None,
        )
        .map_err(|e| NetworkError::CsvError { source: e })?;

        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> = vec![IndexMap::new(); vertices.len()];
        let mut rev: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> = vec![IndexMap::new(); vertices.len()];

        // this callback is invoked when reading each line of the edge list input file and
        // inserts the adjacency information of the edge (src)-[edge]->(dst).

        let edge_lists = config
            .edge_list
            .iter()
            .enumerate()
            .map(|(idx, c)| EdgeList::new(&c.input_file, EdgeListId(idx)))
            .collect::<Result<Vec<_>, _>>()?;

        let mut missing_vertices: HashSet<VertexId> = HashSet::new();
        for edge_list in edge_lists.iter() {
            for edge in edge_list.edges() {
                
                match adj.get_mut(edge.src_vertex_id.0) {
                    None => {
                        missing_vertices.insert(edge.src_vertex_id);
                    }
                    Some(out_links) => {
                        out_links.insert((edge.edge_list_id, edge.edge_id), edge.dst_vertex_id);
                    }
                }
                match rev.get_mut(edge.dst_vertex_id.0) {
                    None => {
                        missing_vertices.insert(edge.dst_vertex_id);
                    }
                    Some(in_links) => {
                        in_links.insert((edge.edge_list_id, edge.edge_id), edge.src_vertex_id);
                    }
                }
            }
        }

        let graph = Graph {
            edge_lists,
            vertices,
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice()
        };

        Ok(graph)
    }
}

impl Graph {
    pub fn get_edge_list(&self, edge_list_id: &EdgeListId) -> Result<&EdgeList, NetworkError> {
        self.edge_lists
            .get(edge_list_id.0)
            .ok_or(NetworkError::EdgeListNotFound(*edge_list_id))
    }

    pub fn n_edge_lists(&self) -> usize {
        self.edge_lists.len()
    }

    /// number of edges in the Graph, not to be conflated with the list of edge ids
    pub fn n_edges(&self) -> usize {
        self.edge_lists.iter().map(|el| el.len()).sum::<usize>()
    }

    /// number of vertices in the Graph
    pub fn n_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// helper function for creating a range of all edge ids in the graph.
    /// uses the knowledge that all ids are unique and consecutive integers
    /// beginning at zero.
    pub fn edge_ids(
        &self,
        edge_list_id: &EdgeListId,
    ) -> Result<Box<dyn Iterator<Item = EdgeId>>, NetworkError> {
        self.get_edge_list(edge_list_id).map(|e| {
            let iter: Box<dyn Iterator<Item = EdgeId>> = Box::new((0..e.len()).map(EdgeId));
            iter
        })
    }

    /// helper function for creating a range of all vertex ids in the graph.
    /// uses the knowledge that all ids are unique and consecutive integers
    /// beginning at zero.
    pub fn vertex_ids(&self) -> Box<dyn Iterator<Item = VertexId>> {
        let range = (0..self.n_vertices()).map(VertexId);
        Box::new(range)
    }

    /// iterates through all edges in the graph
    pub fn edges<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Edge> + 'a> {
        Box::new(self.edge_lists.iter().flat_map(|el| el.edges()))
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
    pub fn get_edge(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<&Edge, NetworkError> {
        match self.edge_lists.get(edge_list_id.0) {
            None => Err(NetworkError::InternalError(format!(
                "EdgeListId not found: {edge_list_id}"
            ))),
            Some(edge_list) => match edge_list.get(edge_id) {
                None => Err(NetworkError::EdgeNotFound(*edge_id)),
                Some(edge) => Ok(edge),
            },
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
    pub fn get_vertex(&self, vertex_id: &VertexId) -> Result<&Vertex, NetworkError> {
        match self.vertices.get(vertex_id.0) {
            None => Err(NetworkError::VertexNotFound(*vertex_id)),
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
    pub fn out_edges(&self, src: &VertexId) -> Vec<(EdgeListId, EdgeId)> {
        self.out_edges_iter(src).cloned().collect_vec()
    }

    /// builds an iterator
    pub fn out_edges_iter<'a>(
        &'a self,
        src: &'a VertexId,
    ) -> Box<dyn Iterator<Item = &'a (EdgeListId, EdgeId)> + 'a> {
        match self.adj.get(src.0) {
            Some(out_map) => Box::new(out_map.keys()),
            None => Box::new(std::iter::empty()),
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
    pub fn in_edges(&self, dst: &VertexId) -> Vec<(EdgeListId, EdgeId)> {
        self.in_edges_iter(dst).cloned().collect_vec()
    }

    pub fn in_edges_iter<'a>(
        &'a self,
        src: &'a VertexId,
    ) -> Box<dyn Iterator<Item = &'a (EdgeListId, EdgeId)> + 'a> {
        match self.rev.get(src.0) {
            Some(in_map) => Box::new(in_map.keys()),
            None => Box::new(std::iter::empty()),
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
    pub fn src_vertex_id(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, NetworkError> {
        self.get_edge(edge_list_id, edge_id)
            .map(|e| e.src_vertex_id)
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
    pub fn dst_vertex_id(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<VertexId, NetworkError> {
        self.get_edge(edge_list_id, edge_id)
            .map(|e| e.dst_vertex_id)
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
        vertex_id: &VertexId,
        direction: &Direction,
    ) -> Vec<(EdgeListId, EdgeId)> {
        match direction {
            Direction::Forward => self.out_edges(vertex_id),
            Direction::Reverse => self.in_edges(vertex_id),
        }
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
    pub fn incident_edges_iter<'a>(
        &'a self,
        vertex_id: &'a VertexId,
        direction: &Direction,
    ) -> Box<dyn Iterator<Item = &'a (EdgeListId, EdgeId)> + 'a> {
        match direction {
            Direction::Forward => self.out_edges_iter(vertex_id),
            Direction::Reverse => self.in_edges_iter(vertex_id),
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
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
        direction: &Direction,
    ) -> Result<VertexId, NetworkError> {
        match direction {
            Direction::Forward => self.dst_vertex_id(edge_list_id, edge_id),
            Direction::Reverse => self.src_vertex_id(edge_list_id, edge_id),
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
    pub fn edge_triplet(
        &self,
        edge_list_id: &EdgeListId,
        edge_id: &EdgeId,
    ) -> Result<(&Vertex, &Edge, &Vertex), NetworkError> {
        let edge = self.get_edge(edge_list_id, edge_id)?;
        let src = self.get_vertex(&edge.src_vertex_id)?;
        let dst = self.get_vertex(&edge.dst_vertex_id)?;

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
    pub fn incident_triplet_ids(
        &self,
        vertex_id: &VertexId,
        direction: &Direction,
    ) -> Result<Vec<(VertexId, EdgeListId, EdgeId, VertexId)>, NetworkError> {
        self.incident_edges_iter(vertex_id, direction)
            .map(|(edge_list_id, edge_id)| {
                let terminal_vid = self.incident_vertex(&edge_list_id, &edge_id, direction)?;
                Ok((*vertex_id, *edge_list_id, *edge_id, terminal_vid))
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
        vertex_id: &VertexId,
        direction: &Direction,
    ) -> Result<Vec<(&Vertex, &Edge, &Vertex)>, NetworkError> {
        self.incident_triplet_ids(vertex_id, direction)?
            .iter()
            .map(|(src_id, edge_list_id, edge_id, dst_id)| {
                let src = self.get_vertex(src_id)?;
                let edge = self.get_edge(edge_list_id, edge_id)?;
                let dst = self.get_vertex(dst_id)?;
                Ok((src, edge, dst))
            })
            .collect()
    }
}
