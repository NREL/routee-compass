use crate::{model::network::{Edge, EdgeId, VertexId}, util::compact_ordered_hash_map::CompactOrderedHashMap};


/// An adjacency list covering some list of edges drawn over the Graph vertex list.
#[derive(Clone, Debug)]
pub struct EdgeList {
    pub adj: Box<[CompactOrderedHashMap<EdgeId, VertexId>]>,
    pub rev: Box<[CompactOrderedHashMap<EdgeId, VertexId>]>,
    pub edges: Box<[Edge]>,
}

impl EdgeList {
    /// number of edges in the Graph2
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }

    
}
