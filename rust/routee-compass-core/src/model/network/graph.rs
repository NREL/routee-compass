use super::{Edge, EdgeId, EdgeList, NetworkError, Vertex, VertexId};
use crate::algorithm::search::Direction;
use crate::model::network::EdgeListId;
use crate::model::network::GraphConfig;
use crate::util::fs::read_utils;
use indexmap::IndexMap;
use itertools::Itertools;
use kdam::tqdm;
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

        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(); vertices.len()];
        let mut rev: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(); vertices.len()];

        // this callback is invoked when reading each line of the edge list input file and
        // inserts the adjacency information of the edge (src)-[edge]->(dst).

        let edge_lists = config
            .edge_list
            .iter()
            .enumerate()
            .map(|(idx, c)| EdgeList::new(&c.input_file, EdgeListId(idx)))
            .collect::<Result<Vec<_>, _>>()?;

        let total_edges = edge_lists.iter().map(|el| el.len()).sum::<usize>();
        log::info!(
            "loaded {} edge lists with a total of {} edges",
            edge_lists.len(),
            total_edges
        );

        let build_adjacencies_iter = tqdm!(
            edge_lists.iter().flat_map(|el| el.edges()),
            desc = "building adjacencies",
            total = total_edges
        );
        let mut bad_refs: Vec<String> = vec![];
        for edge in build_adjacencies_iter {
            if let Err(e) = append_to_adjacency(edge, &mut adj, true) {
                bad_refs.push(e);
            }
            if let Err(e) = append_to_adjacency(edge, &mut rev, false) {
                bad_refs.push(e);
            }
        }

        if !bad_refs.is_empty() {
            let msg = format!("[{}]", bad_refs.iter().take(5).join("\n  "));
            return Err(NetworkError::DatasetError(format!(
                "invalid edge lists for vertex set. (up to) first five errors:\n  {msg}"
            )));
        }

        let graph = Graph {
            edge_lists,
            vertices,
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice(),
        };

        Ok(graph)
    }
}

impl Graph {
    /// access a specific EdgeList by its id
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
                let terminal_vid = self.incident_vertex(edge_list_id, edge_id, direction)?;
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

/// Appends an edge to an adjacency list.
///
/// # Arguments
///
/// * `edge` - The edge to append to the adjacency list
/// * `adj` - The adjacency list to modify
/// * `forward` - If `true`, appends using the source vertex id (forward-oriented adjacency). If `false`, uses the destination vertex id (reverse-oriented adjacency).
///
/// # Returns
///
/// Returns `Ok(())` if the edge was successfully appended, or an error message if the
/// required vertex is not found in the adjacency list.
fn append_to_adjacency(
    edge: &Edge,
    adj: &mut [IndexMap<(EdgeListId, EdgeId), VertexId>],
    forward: bool,
) -> Result<(), String> {
    let vertex_idx = if forward {
        edge.src_vertex_id.0
    } else {
        edge.dst_vertex_id.0
    };
    match adj.get_mut(vertex_idx) {
        None => {
            let direction = if forward { "forward" } else { "reverse" };
            Err(format!(
                "vertex {} not found in {} adjacencies for edge list, edge: {}, {}",
                vertex_idx, direction, edge.edge_list_id.0, edge.edge_id.0
            ))
        }
        Some(out_links) => {
            let target_vertex = if forward {
                edge.dst_vertex_id
            } else {
                edge.src_vertex_id
            };
            out_links.insert((edge.edge_list_id, edge.edge_id), target_vertex);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::{f64::Length, length::meter};

    fn create_test_edge(
        edge_list_id: usize,
        edge_id: usize,
        src_vertex_id: usize,
        dst_vertex_id: usize,
    ) -> Edge {
        Edge::new(
            edge_list_id,
            edge_id,
            src_vertex_id,
            dst_vertex_id,
            Length::new::<meter>(1.0),
        )
    }

    #[test]
    fn test_append_to_adjacency_forward_success() {
        // Create an adjacency list with 3 vertices
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new(), IndexMap::new()];

        // Create an edge from vertex 0 to vertex 2
        let edge = create_test_edge(0, 0, 0, 2);

        // Test forward adjacency (should use src_vertex_id = 0 as index)
        let result = append_to_adjacency(&edge, &mut adj, true);

        assert!(result.is_ok());

        // Check that the edge was added to the correct adjacency list entry
        let expected_key = (EdgeListId(0), EdgeId(0));
        assert!(adj[0].contains_key(&expected_key));
        assert_eq!(adj[0][&expected_key], VertexId(2)); // Target should be dst_vertex_id

        // Other adjacency lists should remain empty
        assert!(adj[1].is_empty());
        assert!(adj[2].is_empty());
    }

    #[test]
    fn test_append_to_adjacency_reverse_success() {
        // Create an adjacency list with 3 vertices
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new(), IndexMap::new()];

        // Create an edge from vertex 0 to vertex 2
        let edge = create_test_edge(0, 0, 0, 2);

        // Test reverse adjacency (should use dst_vertex_id = 2 as index)
        let result = append_to_adjacency(&edge, &mut adj, false);

        assert!(result.is_ok());

        // Check that the edge was added to the correct adjacency list entry
        let expected_key = (EdgeListId(0), EdgeId(0));
        assert!(adj[2].contains_key(&expected_key));
        assert_eq!(adj[2][&expected_key], VertexId(0)); // Target should be src_vertex_id

        // Other adjacency lists should remain empty
        assert!(adj[0].is_empty());
        assert!(adj[1].is_empty());
    }

    #[test]
    fn test_append_to_adjacency_forward_invalid_vertex() {
        // Create an adjacency list with only 2 vertices (indices 0 and 1)
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new()];

        // Create an edge with src_vertex_id = 3 (out of bounds)
        let edge = create_test_edge(0, 5, 3, 1);

        // Test forward adjacency - should fail because vertex 3 doesn't exist
        let result = append_to_adjacency(&edge, &mut adj, true);

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("vertex 3 not found in forward adjacencies"));
        assert!(error_msg.contains("edge list, edge: 0, 5"));
    }

    #[test]
    fn test_append_to_adjacency_reverse_invalid_vertex() {
        // Create an adjacency list with only 2 vertices (indices 0 and 1)
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new()];

        // Create an edge with dst_vertex_id = 5 (out of bounds)
        let edge = create_test_edge(1, 10, 0, 5);

        // Test reverse adjacency - should fail because vertex 5 doesn't exist
        let result = append_to_adjacency(&edge, &mut adj, false);

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("vertex 5 not found in reverse adjacencies"));
        assert!(error_msg.contains("edge list, edge: 1, 10"));
    }

    #[test]
    fn test_append_to_adjacency_multiple_edges_same_vertex() {
        // Create an adjacency list with 3 vertices
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new(), IndexMap::new()];

        // Create multiple edges from vertex 0
        let edge1 = create_test_edge(0, 0, 0, 1);
        let edge2 = create_test_edge(0, 1, 0, 2);
        let edge3 = create_test_edge(1, 0, 0, 2); // Different edge list

        // Add all edges in forward direction
        assert!(append_to_adjacency(&edge1, &mut adj, true).is_ok());
        assert!(append_to_adjacency(&edge2, &mut adj, true).is_ok());
        assert!(append_to_adjacency(&edge3, &mut adj, true).is_ok());

        // Check that all edges were added to vertex 0's adjacency list
        assert_eq!(adj[0].len(), 3);

        // Verify each edge maps to the correct target vertex
        assert_eq!(adj[0][&(EdgeListId(0), EdgeId(0))], VertexId(1));
        assert_eq!(adj[0][&(EdgeListId(0), EdgeId(1))], VertexId(2));
        assert_eq!(adj[0][&(EdgeListId(1), EdgeId(0))], VertexId(2));
    }

    #[test]
    fn test_append_to_adjacency_edge_overwrite() {
        // Create an adjacency list with 3 vertices
        let mut adj: Vec<IndexMap<(EdgeListId, EdgeId), VertexId>> =
            vec![IndexMap::new(), IndexMap::new(), IndexMap::new()];

        // Create two edges with the same EdgeListId and EdgeId but different targets
        let edge1 = create_test_edge(0, 0, 0, 1);
        let edge2 = create_test_edge(0, 0, 0, 2); // Same edge list and edge id

        // Add first edge
        assert!(append_to_adjacency(&edge1, &mut adj, true).is_ok());
        assert_eq!(adj[0][&(EdgeListId(0), EdgeId(0))], VertexId(1));

        // Add second edge - should overwrite the first one
        assert!(append_to_adjacency(&edge2, &mut adj, true).is_ok());
        assert_eq!(adj[0].len(), 1); // Still only one entry
        assert_eq!(adj[0][&(EdgeListId(0), EdgeId(0))], VertexId(2)); // Updated target
    }
}
