use std::{collections::HashMap, fs::File, io::BufReader};

use compass_core::{
    model::{
        graph::{
            directed_graph::DirectedGraph, edge_id::EdgeId, graph_error::GraphError,
            vertex_id::VertexId,
        },
        property::{edge::Edge, vertex::Vertex},
    },
    util::fs_utils::line_count,
};
use flate2::read::GzDecoder;

use super::tomtom_graph_config::TomTomGraphConfig;
use super::tomtom_graph_error::TomTomGraphError;
use csv;

pub struct TomTomGraph {
    pub adj: Vec<HashMap<EdgeId, VertexId>>,
    pub rev: Vec<HashMap<EdgeId, VertexId>>,
    pub edges: Vec<Edge>,
    pub vertices: Vec<Vertex>,
}

impl DirectedGraph for TomTomGraph {
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
        // count file lengths to determine |V| and |E|. remove header from count
        println!("checking file length of edge and vertex input files");
        let edge_file_length = config
            .get_n_edges()
            .map_err(|e| TomTomGraphError::IOError { source: e })?;
        let vertex_file_length = config
            .get_n_vertices()
            .map_err(|e| TomTomGraphError::IOError { source: e })?;
        if edge_file_length < 1 {
            return Err(TomTomGraphError::EmptyFileSource {
                filename: config.edge_list_csv,
            });
        }
        if vertex_file_length < 1 {
            return Err(TomTomGraphError::EmptyFileSource {
                filename: config.vertex_list_csv,
            });
        }
        let n_edges = edge_file_length - 1;
        let n_vertices = vertex_file_length - 1;
        println!(
            "{} edge file len, {} vertex file len",
            edge_file_length, vertex_file_length
        );
        println!("{} edges, {} vertices in file", n_edges, n_vertices);

        // build collections to store in the TomTomGraph
        let mut edges: Vec<Edge> = Vec::with_capacity(n_edges);
        let mut vertices: Vec<Vertex> = Vec::with_capacity(n_vertices);
        let mut adj: Vec<HashMap<EdgeId, VertexId>> = Vec::with_capacity(n_vertices);
        let mut rev: Vec<HashMap<EdgeId, VertexId>> = Vec::with_capacity(n_vertices);

        // set up csv.gz reading and row deserialization
        let edge_list_file = File::open(config.edge_list_csv.clone())
            .map_err(|e| TomTomGraphError::IOError { source: e })?;
        let vertex_list_file = File::open(config.vertex_list_csv.clone())
            .map_err(|e| TomTomGraphError::IOError { source: e })?;
        let mut edge_reader =
            csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(edge_list_file))));
        let edge_rows = edge_reader.deserialize();
        let mut vertex_reader =
            csv::Reader::from_reader(Box::new(BufReader::new(GzDecoder::new(vertex_list_file))));
        let vertex_rows = vertex_reader.deserialize();

        // read each Edge row, updating the Edge table and adjacency lists
        let mut i = 0;
        for row in edge_rows {
            let edge: Edge = row.map_err(|e| TomTomGraphError::CsvError { source: e })?;
            if i < 5 {
                println!("{:?}", edge);
                i = i + 1;
            }
            edges.insert(edge.edge_id.0 as usize, edge);
            // the Edge provides us with all id information to build our adjacency lists as well

            match adj.get_mut(edge.src_vertex_id.0 as usize) {
                None => {
                    let new_map = HashMap::from([(edge.edge_id, edge.dst_vertex_id)]);
                    adj.insert(edge.src_vertex_id.0 as usize, new_map);
                    todo!("add bounds check (0 <= id < vec.len()");
                }
                Some(out_links) => {
                    out_links.insert(edge.edge_id, edge.dst_vertex_id);
                }
            }
            match rev.get_mut(edge.dst_vertex_id.0 as usize) {
                None => {
                    let new_map = HashMap::from([(edge.edge_id, edge.src_vertex_id)]);
                    adj.insert(edge.dst_vertex_id.0 as usize, new_map);
                }
                Some(in_links) => {
                    in_links.insert(edge.edge_id, edge.src_vertex_id);
                }
            }
        }

        // read in each Vertex row
        for row in vertex_rows {
            let vertex: Vertex = row.map_err(|e| TomTomGraphError::CsvError { source: e })?;
            vertices.insert(vertex.vertex_id.0 as usize, vertex);
        }

        let graph = TomTomGraph {
            adj,
            rev,
            edges,
            vertices,
        };

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::tomtom_graph_config::TomTomGraphConfig;

    use super::TomTomGraph;

    #[test]
    fn test_read_national_graph() {
        let edges_path = "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/edges_compass.csv.gz";
        let vertices_path =
            "/Users/rfitzger/data/routee/tomtom/tomtom-condensed/vertices_compass.csv.gz";
        let conf = TomTomGraphConfig {
            edge_list_csv: String::from(edges_path),
            vertex_list_csv: String::from(vertices_path),
            n_edges: Some(67),
            n_vertices: Some(56306871),
        };
        match TomTomGraph::try_from(conf) {
            Ok(graph) => {
                println!("{} rows in adjacency list", graph.adj.len());
                println!("{} rows in reverse list", graph.rev.len());
                println!("{} rows in edge list", graph.edges.len());
                println!("{} rows in vertex list", graph.vertices.len());
                println!("yay!")
            }
            Err(e) => {
                println!("uh oh, no good");
                println!("{}", e.to_string());
                panic!();
            }
        }
    }
}
