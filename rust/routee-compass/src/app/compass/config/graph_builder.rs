use std::path::PathBuf;

use log::warn;
use routee_compass_core::{
    model::{
        graph::{
            edge_loader::{EdgeLoader, EdgeLoaderConfig},
            graph::Graph,
            graph_error::GraphError,
            vertex_loader::VertexLoaderConfig,
        },
        property::vertex::Vertex,
    },
    util::fs::fs_utils::line_count,
};

use crate::app::compass::compass_configuration_field::CompassConfigurationField;

use super::{
    compass_configuration_error::CompassConfigurationError,
    config_json_extension::ConfigJsonExtensions,
};

pub struct DefaultGraphBuilder {}

impl DefaultGraphBuilder {
    /// tries to build a Graph from a JSON object.
    ///
    /// for both edge and vertex lists, we assume all ids can be used as indices
    /// to an array data structure. to find the size of each array, we pass once
    /// through each file to count the number of rows (minus header) of the CSV.
    /// then we can build a Vec *once* and insert rows as we decode them without
    /// a sort.
    ///
    /// # Arguments
    ///
    /// * `params` - configuration JSON object for building a `Graph` instance
    ///
    /// # Returns
    ///
    /// A graph instance, or an error if an IO error occurred.
    pub fn build(params: &serde_json::Value) -> Result<Graph, CompassConfigurationError> {
        let graph_key = CompassConfigurationField::Graph.to_string();
        let edge_list_csv =
            params.get_config_path(String::from("edge_list_csv"), graph_key.clone())?;
        let vertex_list_csv =
            params.get_config_path(String::from("vertex_list_csv"), graph_key.clone())?;
        let maybe_n_edges =
            params.get_config_serde_optional(String::from("n_edges"), graph_key.clone())?;
        let maybe_n_vertices =
            params.get_config_serde_optional(String::from("n_vertices"), graph_key.clone())?;
        let verbose: Option<bool> =
            params.get_config_serde_optional(String::from("verbose"), graph_key.clone())?;

        let n_edges = match maybe_n_edges {
            Some(n) => n,
            None => get_n_edges(edge_list_csv.clone())?,
        };

        let n_vertices = match maybe_n_vertices {
            Some(n) => n,
            None => get_n_vertices(vertex_list_csv.clone())?,
        };

        let e_conf = EdgeLoaderConfig {
            edge_list_csv,
            n_edges,
            n_vertices,
        };

        let e_result = EdgeLoader::try_from(e_conf)?;

        let v_conf = VertexLoaderConfig {
            vertex_list_csv,
            n_vertices,
        };

        let vertices: Vec<Vertex> = v_conf.try_into()?;

        let graph = Graph {
            adj: e_result.adj,
            rev: e_result.rev,
            edges: e_result.edges,
            vertices,
        };

        Ok(graph)
    }
}

fn get_n_edges(edge_list_csv: PathBuf) -> Result<usize, GraphError> {
    warn!("edge list size not provided, scanning input to determine size");
    // check if the extension is .gz
    let is_gzip = edge_list_csv
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(edge_list_csv.clone(), is_gzip)?;
    if n < 1 {
        return Err(GraphError::EmptyFileSource {
            filename: edge_list_csv.clone(),
        });
    }
    Ok(n - 1) // drop count of header line
}

fn get_n_vertices(vertex_list_csv: PathBuf) -> Result<usize, GraphError> {
    warn!("vertex list size not provided, scanning input to determine size");
    let is_gzip = vertex_list_csv 
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(vertex_list_csv.clone(), is_gzip)?;
    if n < 1 {
        return Err(GraphError::EmptyFileSource {
            filename: vertex_list_csv.clone(),
        });
    }
    Ok(n - 1) // drop count of header line
}
