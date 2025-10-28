use std::{fmt::Display, path::Path};

use kdam::tqdm;

use crate::{
    model::network::{Edge, EdgeConfig, EdgeId, EdgeListId, NetworkError},
    util::fs::read_utils,
};

/// An adjacency list covering some list of edges drawn over the Graph vertex list.
#[derive(Clone, Debug)]
pub struct EdgeList(pub Box<[Edge]>);

impl EdgeList {
    /// builds a new edge list on top of the vertex list of a graph, from some CSV file
    /// containing the edge adjancencies.
    pub fn new<P: AsRef<Path> + Display>(
        edge_list_input_file: &P,
        edge_list_id: EdgeListId,
    ) -> Result<EdgeList, NetworkError> {
        // read each row as an [`EdgeConfig`] and then assign the [`EdgeListId`] to finalize it as a [`Edge`].
        let edge_config_iter = tqdm!(
            read_utils::iterator_from_csv(edge_list_input_file, true, None)?,
            desc = format!("graph edge list {}: {}", edge_list_id, edge_list_input_file)
        );
        let edges = edge_config_iter
            .map(|r| r.map(|edge_config: EdgeConfig| edge_config.assign_edge_list(&edge_list_id)))
            .collect::<Result<Vec<Edge>, csv::Error>>()?
            .into_boxed_slice();
        eprintln!();

        let edge_list = EdgeList(edges);
        Ok(edge_list)
    }

    /// number of edges in the Graph
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the edge list contains no edges.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn edges<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Edge> + 'a> {
        Box::new(self.0.iter())
    }

    pub fn get(&self, edge_id: &EdgeId) -> Option<&Edge> {
        self.0.get(edge_id.0)
    }
}
