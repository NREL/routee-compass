use crate::{
    model::{
        property::edge::Edge,
        road_network::{edge_id::EdgeId, graph_error::GraphError, vertex_id::VertexId},
    },
    util::{compact_ordered_hash_map::CompactOrderedHashMap, fs::read_utils},
};
use kdam::Bar;
use kdam::BarExt;
use std::{collections::HashSet, path::PathBuf};

pub struct EdgeLoader {
    pub edges: Box<[Edge]>,
    pub adj: Box<[CompactOrderedHashMap<EdgeId, VertexId>]>,
    pub rev: Box<[CompactOrderedHashMap<EdgeId, VertexId>]>,
}

pub struct EdgeLoaderConfig {
    pub edge_list_csv: PathBuf,
    pub n_edges: usize,
    pub n_vertices: usize,
}

impl TryFrom<EdgeLoaderConfig> for EdgeLoader {
    type Error = GraphError;

    fn try_from(c: EdgeLoaderConfig) -> Result<Self, Self::Error> {
        let mut adj: Vec<CompactOrderedHashMap<EdgeId, VertexId>> =
            vec![CompactOrderedHashMap::empty(); c.n_vertices];
        let mut rev: Vec<CompactOrderedHashMap<EdgeId, VertexId>> =
            vec![CompactOrderedHashMap::empty(); c.n_vertices];

        let mut pb = Bar::builder()
            .total(c.n_edges)
            .animation("fillup")
            .desc("edge list")
            .build()
            .map_err(|e| GraphError::ProgressBarBuildError(String::from("edge list"), e))?;

        let mut missing_vertices: HashSet<VertexId> = HashSet::new();
        let cb = Box::new(|edge: &Edge| {
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
            let _ = pb.update(1);
        });

        let edges = read_utils::from_csv(&c.edge_list_csv, true, Some(cb))?;

        println!();
        let result = EdgeLoader {
            edges,
            adj: adj.into_boxed_slice(),
            rev: rev.into_boxed_slice(),
        };

        Ok(result)
    }
}
