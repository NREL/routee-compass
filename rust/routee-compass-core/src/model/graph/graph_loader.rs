use std::path::PathBuf;

use log::warn;

use crate::{model::property::vertex::Vertex, util::fs::fs_utils::line_count};

use super::{
    edge_loader::{EdgeLoader, EdgeLoaderConfig},
    graph::Graph,
    graph_error::GraphError,
    vertex_loader::VertexLoaderConfig,
};

pub fn graph_from_files(
    edge_list_csv: PathBuf,
    vertex_list_csv: PathBuf,
    n_edges: Option<usize>,
    n_vertices: Option<usize>,
    verbose: Option<bool>,
) -> Result<Graph, GraphError> {
    let verbose = verbose.unwrap_or(false);
    let n_edges = match n_edges {
        Some(n) => n,
        None => {
            if verbose {
                warn!("edge list size not provided, scanning input to determine size");
            }
            get_n_edges(edge_list_csv.clone())?
        }
    };

    let n_vertices = match n_vertices {
        Some(n) => n,
        None => {
            if verbose {
                warn!("vertex list size not provided, scanning input to determine size");
            }
            get_n_vertices(vertex_list_csv.clone())?
        }
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

fn get_n_edges(edge_list_csv: PathBuf) -> Result<usize, GraphError> {
    // check if the extension is .gz
    let is_gzip = edge_list_csv
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(edge_list_csv.clone(), is_gzip)?;
    if n < 1 {
        return Err(GraphError::EmptyFileSource {
            filename: edge_list_csv,
        });
    }
    Ok(n - 1) // drop count of header line
}

fn get_n_vertices(vertex_list_csv: PathBuf) -> Result<usize, GraphError> {
    let is_gzip = vertex_list_csv
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(vertex_list_csv.clone(), is_gzip)?;
    if n < 1 {
        return Err(GraphError::EmptyFileSource {
            filename: vertex_list_csv,
        });
    }
    Ok(n - 1) // drop count of header line
}
