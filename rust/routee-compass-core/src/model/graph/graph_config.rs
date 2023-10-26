use crate::model::graph::graph_error::GraphError;
use crate::util::fs::fs_utils::line_count;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct GraphConfig {
    pub edge_list_csv: String,
    pub vertex_list_csv: String,
    pub n_edges: Option<usize>,
    pub n_vertices: Option<usize>,
    pub verbose: bool,
}

impl GraphConfig {
    pub fn read_file_sizes(&self) -> Result<(usize, usize), GraphError> {
        let n_edges = self
            .get_n_edges()
            .map_err(|e| GraphError::IOError { source: e })?;
        let n_vertices = self
            .get_n_vertices()
            .map_err(|e| GraphError::IOError { source: e })?;
        if n_edges < 1 {
            return Err(GraphError::EmptyFileSource {
                filename: self.edge_list_csv.clone(),
            });
        }
        if n_vertices < 1 {
            return Err(GraphError::EmptyFileSource {
                filename: self.vertex_list_csv.clone(),
            });
        }
        Ok((n_edges, n_vertices))
    }

    pub fn get_n_edges(&self) -> std::io::Result<usize> {
        match self.n_edges {
            Some(n) => Ok(n),
            None => {
                warn!("edge list size not provided, scanning input to determine size");
                let is_gzip = self.edge_list_csv.ends_with(".gz");
                let n = line_count(self.edge_list_csv.clone(), is_gzip)?;
                Ok(n - 1) // drop count of header line
            }
        }
    }

    pub fn get_n_vertices(&self) -> std::io::Result<usize> {
        match self.n_vertices {
            Some(n) => Ok(n),
            None => {
                warn!("vertex list size not provided, scanning input to determine size");
                let is_gzip = self.vertex_list_csv.ends_with(".gz");
                let n = line_count(self.vertex_list_csv.clone(), is_gzip)?;
                Ok(n - 1) // drop count of header line
            }
        }
    }
}
