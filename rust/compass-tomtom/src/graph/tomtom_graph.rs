use std::collections::HashMap;

use compass_core::model::{
    graph::{
        directed_graph::DirectedGraph, edge_id::EdgeId, graph_error::GraphError,
        vertex_id::VertexId,
    },
    property::{edge::Edge, vertex::Vertex},
};

use crate::graph::{
    tomtom_edge_list::{TomTomEdgeList, TomTomEdgeListConfig},
    tomtom_vertex_list::{read_vertex_list, TomTomVertexListConfig},
};

use super::tomtom_graph_config::TomTomGraphConfig;
use super::tomtom_graph_error::TomTomGraphError;
use log::info;
pub struct TomTomGraph {
    pub adj: Vec<HashMap<EdgeId, VertexId>>,
    pub rev: Vec<HashMap<EdgeId, VertexId>>,
    pub edges: Vec<Edge>,
    pub vertices: Vec<Vertex>,
}

impl DirectedGraph for TomTomGraph {
    fn all_edge_ids(&self) -> Vec<EdgeId> {
        self.edges.iter().map(|edge| edge.edge_id).collect()
    }
    fn all_edges(&self) -> Vec<Edge> {
        self.edges.iter().cloned().collect()
    }
    fn all_vertex_ids(&self) -> Vec<VertexId> {
        self.vertices
            .iter()
            .map(|vertex| vertex.vertex_id)
            .collect()
    }
    fn all_verticies(&self) -> Vec<Vertex> {
        self.vertices.iter().cloned().collect()
    }
    fn edge_attr(&self, edge_id: EdgeId) -> Result<Edge, GraphError> {
        match self.edges.get(edge_id.0 as usize) {
            None => Err(GraphError::EdgeAttributeNotFound { edge_id }),
            Some(edge) => Ok(*edge),
        }
    }
    fn vertex_attr(&self, vertex_id: VertexId) -> Result<Vertex, GraphError> {
        match self.vertices.get(vertex_id.0 as usize) {
            None => Err(GraphError::VertexAttributeNotFound { vertex_id }),
            Some(vertex) => Ok(*vertex),
        }
    }
    fn out_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.adj.get(src.0 as usize) {
            None => Err(GraphError::VertexWithoutOutEdges { vertex_id: src }),
            Some(out_map) => {
                let edge_ids = out_map.keys().cloned().collect();
                Ok(edge_ids)
            }
        }
    }
    fn in_edges(&self, src: VertexId) -> Result<Vec<EdgeId>, GraphError> {
        match self.rev.get(src.0 as usize) {
            None => Err(GraphError::VertexWithoutInEdges { vertex_id: src }),
            Some(in_map) => {
                let edge_ids = in_map.keys().cloned().collect();
                Ok(edge_ids)
            }
        }
    }
    fn src_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e| e.src_vertex_id)
    }
    fn dst_vertex(&self, edge_id: EdgeId) -> Result<VertexId, GraphError> {
        self.edge_attr(edge_id).map(|e| e.dst_vertex_id)
    }
}

impl TryFrom<TomTomGraphConfig> for TomTomGraph {
    type Error = TomTomGraphError;

    /// tries to build a TomTomGraph from a TomTomGraphConfig.
    ///
    /// for both edge and vertex lists, we assume all ids can be used as indices
    /// to an array data structure. to find the size of each array, we pass once
    /// through each file to count the number of rows (minus header) of the CSV.
    /// then we can build a Vec *once* and insert rows as we decode them without
    /// a sort.
    fn try_from(config: TomTomGraphConfig) -> Result<Self, TomTomGraphError> {
        info!("checking file length of edge and vertex input files");
        let (n_edges, n_vertices) = config.read_file_sizes()?;
        info!(
            "creating data structures to hold {} edges, {} vertices",
            n_edges, n_vertices
        );

        info!("reading edge list");

        let e_conf = TomTomEdgeListConfig {
            config: &config,
            n_edges,
            n_vertices,
        };
        let e_result = TomTomEdgeList::try_from(e_conf)?;

        info!("reading vertex list");
        let v_conf = TomTomVertexListConfig {
            config: &config,
            n_vertices,
        };
        let vertices = read_vertex_list(v_conf)?;

        let graph = TomTomGraph {
            adj: e_result.adj,
            rev: e_result.rev,
            edges: e_result.edges,
            vertices,
        };

        info!("{}", graph.print_sizes());

        Ok(graph)
    }
}

impl TomTomGraph {
    fn print_sizes(&self) -> String {
        return format!(
            "TomTomGraph |V| {} |E| {} |Adj| {} |rev| {}",
            self.vertices.len(),
            self.edges.len(),
            self.adj.len(),
            self.rev.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::TomTomGraph;
    use crate::graph::tomtom_graph_config::TomTomGraphConfig;

    // capture logs in test: https://docs.rs/env_logger/latest/env_logger/#capturing-logs-in-tests
    fn init() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn test_read_national_graph() {
        let edges_path = "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/edges_compass.csv.gz";
        let vertices_path =
            "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/vertices_compass.csv.gz";
        let conf = TomTomGraphConfig {
            edge_list_csv: String::from(edges_path),
            vertex_list_csv: String::from(vertices_path),
            n_edges: Some(67198522),
            n_vertices: Some(56306871),
            verbose: false,
        };
        match TomTomGraph::try_from(conf) {
            Ok(graph) => {
                info!("{} rows in adjacency list", graph.adj.len());
                info!("{} rows in reverse list", graph.rev.len());
                info!("{} rows in edge list", graph.edges.len());
                info!("{} rows in vertex list", graph.vertices.len());
                info!("yay!")
            }
            Err(e) => {
                info!("uh oh, no good");
                info!("{}", e.to_string());
                panic!();
            }
        }
    }
}
