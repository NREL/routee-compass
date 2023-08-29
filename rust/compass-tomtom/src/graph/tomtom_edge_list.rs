use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

use compass_core::{
    model::{
        graph::{edge_id::EdgeId, vertex_id::VertexId},
        property::edge::Edge,
    },
    util::fs::read_utils,
};
use flate2::read::GzDecoder;
use log::debug;

use super::{tomtom_graph_config::TomTomGraphConfig, tomtom_graph_error::TomTomGraphError};
use kdam::Bar;
use kdam::BarExt;

pub struct TomTomEdgeList {
    pub edges: Vec<Edge>,
    pub adj: Vec<HashMap<EdgeId, VertexId>>,
    pub rev: Vec<HashMap<EdgeId, VertexId>>,
}

pub struct TomTomEdgeListConfig<'a> {
    pub config: &'a TomTomGraphConfig,
    pub n_edges: usize,
    pub n_vertices: usize,
}

impl<'a> TryFrom<TomTomEdgeListConfig<'a>> for TomTomEdgeList {
    type Error = TomTomGraphError;

    fn try_from(c: TomTomEdgeListConfig) -> Result<Self, Self::Error> {
        let min_node_connectivity: usize = 1;
        let mut adj: Vec<HashMap<EdgeId, VertexId>> =
            vec![HashMap::with_capacity(min_node_connectivity); c.n_vertices];
        let mut rev: Vec<HashMap<EdgeId, VertexId>> =
            vec![HashMap::with_capacity(min_node_connectivity); c.n_vertices];

        let mut pb = Bar::builder()
            .total(c.n_edges)
            .animation("fillup")
            .desc("edge list")
            .build()
            .map_err(|e| TomTomGraphError::ProgressBarBuildError(String::from("edge list"), e))?;

        let mut missing_vertices: HashSet<VertexId> = HashSet::new();
        let cb = Box::new(|edge: &Edge| {
            // the Edge provides us with all id information to build our adjacency lists as well
            match adj.get_mut(edge.src_vertex_id.0 as usize) {
                None => {
                    missing_vertices.insert(edge.src_vertex_id);
                }
                Some(out_links) => {
                    out_links.insert(edge.edge_id, edge.dst_vertex_id);
                }
            }
            match rev.get_mut(edge.dst_vertex_id.0 as usize) {
                None => {
                    missing_vertices.insert(edge.dst_vertex_id);
                }
                Some(in_links) => {
                    in_links.insert(edge.edge_id, edge.src_vertex_id);
                }
            }
            pb.update(1);
        });

        let edges =
            read_utils::vec_from_csv(&c.config.edge_list_csv, true, Some(c.n_edges), Some(cb))?;

        // for row in edge_rows {
        //     let edge: Edge = row.map_err(|e| TomTomGraphError::CsvError { source: e })?;
        //     edges[edge.edge_id.0 as usize] = edge;
        //     // the Edge provides us with all id information to build our adjacency lists as well

        //     match adj.get_mut(edge.src_vertex_id.0 as usize) {
        //         None => {
        //             return Err(TomTomGraphError::AdjacencyVertexMissing(edge.src_vertex_id));
        //         }
        //         Some(out_links) => {
        //             out_links.insert(edge.edge_id, edge.dst_vertex_id);
        //         }
        //     }
        //     match rev.get_mut(edge.dst_vertex_id.0 as usize) {
        //         None => {
        //             return Err(TomTomGraphError::AdjacencyVertexMissing(edge.dst_vertex_id));
        //         }
        //         Some(in_links) => {
        //             in_links.insert(edge.edge_id, edge.src_vertex_id);
        //         }
        //     }
        //     pb.update(1);
        // }
        print!("\n");
        let result = TomTomEdgeList { edges, adj, rev };

        Ok(result)
    }
}
