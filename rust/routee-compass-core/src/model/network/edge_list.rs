use std::{collections::HashSet, fmt::Display, path::Path};

use indexmap::IndexMap;
use kdam::tqdm;

use crate::{
    model::network::{Edge, EdgeConfig, EdgeId, EdgeListId, NetworkError, VertexId},
    util::fs::read_utils,
};

/// An adjacency list covering some list of edges drawn over the Graph vertex list.
#[derive(Clone, Debug)]
pub struct EdgeList {
    pub adj: Box<[IndexMap<EdgeId, VertexId>]>,
    pub rev: Box<[IndexMap<EdgeId, VertexId>]>,
    pub edges: Box<[Edge]>,
}

impl EdgeList {
    /// builds a new edge list on top of the vertex list of a graph, from some CSV file
    /// containing the edge adjancencies.
    pub fn new<P: AsRef<Path> + Display>(
        edge_list_input_file: &P,
        edge_list_id: EdgeListId,
        n_vertices: usize,
    ) -> Result<EdgeList, NetworkError> {
        let mut adj: Vec<IndexMap<EdgeId, VertexId>> =
            vec![IndexMap::new(); n_vertices];
        let mut rev: Vec<IndexMap<EdgeId, VertexId>> =
            vec![IndexMap::new(); n_vertices];
        let mut missing_vertices: HashSet<VertexId> = HashSet::new();

        // this callback is invoked when reading each line of the edge list input file and
        // inserts the adjacency information of the edge (src)-[edge]->(dst).
        let cb = Box::new(|edge: &EdgeConfig| {
            // the Edge provides us with all id information to build our adjacency lists as well
            match adj.get_mut(edge.src_vertex_id.0) {
                None => {
                    missing_vertices.insert(edge.src_vertex_id);
                }
                Some(out_links) => {
                    out_links.insert(edge.edge_id, edge.dst_vertex_id);
                }
            }
            match rev.get_mut(edge.dst_vertex_id.0) {
                None => {
                    missing_vertices.insert(edge.dst_vertex_id);
                }
                Some(in_links) => {
                    in_links.insert(edge.edge_id, edge.src_vertex_id);
                }
            }
        });

        // read each row as an [`EdgeConfig`] and then assign the [`EdgeListId`] to finalize it as a [`Edge`].
        let edge_config_iter = tqdm!(
            read_utils::iterator_from_csv(edge_list_input_file, true, Some(cb))?,
            desc = format!("graph edge list {}: {}", edge_list_id, edge_list_input_file)
        );
        let edges = edge_config_iter
            .map(|r| r.map(|edge_config| edge_config.assign_edge_list(&edge_list_id)))
            .collect::<Result<Vec<Edge>, csv::Error>>()?
            .into_boxed_slice();

        let edge_list = EdgeList {
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice(),
            edges,
        };
        Ok(edge_list)
    }

    /// number of edges in the Graph
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }
}
