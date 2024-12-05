use super::{
    edge_loader::{EdgeLoader, EdgeLoaderConfig},
    vertex_loader::VertexLoaderConfig,
    Graph, NetworkError, Vertex,
};
use crate::util::fs::fs_utils::line_count;
use log::warn;
use std::path::Path;

pub fn graph_from_files<P: AsRef<Path>>(
    edge_list_csv: &P,
    vertex_list_csv: &P,
    n_edges: Option<usize>,
    n_vertices: Option<usize>,
    verbose: Option<bool>,
) -> Result<Graph, NetworkError> {
    let verbose = verbose.unwrap_or(false);
    let n_edges = match n_edges {
        Some(n) => n,
        None => {
            if verbose {
                warn!("edge list size not provided, scanning input to determine size");
            }
            get_n_edges(&edge_list_csv)?
        }
    };

    let n_vertices = match n_vertices {
        Some(n) => n,
        None => {
            if verbose {
                warn!("vertex list size not provided, scanning input to determine size");
            }
            get_n_vertices(&vertex_list_csv)?
        }
    };
    let e_conf = EdgeLoaderConfig {
        edge_list_csv: edge_list_csv.as_ref().to_path_buf(),
        n_edges,
        n_vertices,
    };

    let e_result = EdgeLoader::try_from(e_conf)?;

    let v_conf = VertexLoaderConfig {
        vertex_list_csv: vertex_list_csv.as_ref().to_path_buf(),
        n_vertices,
    };

    let vertices: Box<[Vertex]> = v_conf.try_into()?;

    let graph = Graph {
        adj: e_result.adj,
        rev: e_result.rev,
        edges: e_result.edges,
        vertices,
    };

    Ok(graph)
}

fn get_n_edges<P: AsRef<Path>>(edge_list_csv: &P) -> Result<usize, NetworkError> {
    // check if the extension is .gz
    let is_gzip = edge_list_csv
        .as_ref()
        .to_path_buf()
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(edge_list_csv, is_gzip)?;
    if n < 1 {
        let msg = match edge_list_csv.as_ref().to_str() {
            Some(path_str) => format!("edge file {} is empty", path_str),
            None => String::from("edge file is empty"),
        };
        return Err(NetworkError::DatasetError(msg));
    }
    Ok(n - 1) // drop count of header line
}

fn get_n_vertices<P: AsRef<Path>>(vertex_list_csv: &P) -> Result<usize, NetworkError> {
    let is_gzip = vertex_list_csv
        .as_ref()
        .to_path_buf()
        .extension()
        .map(|ext| ext.to_str() == Some("gz"))
        .unwrap_or(false);
    let n = line_count(vertex_list_csv, is_gzip)?;
    if n < 1 {
        let msg = match vertex_list_csv.as_ref().to_str() {
            Some(path_str) => format!("vertex file {} is empty", path_str),
            None => String::from("vertex file is empty"),
        };
        return Err(NetworkError::DatasetError(msg));
    }
    Ok(n - 1) // drop count of header line
}
