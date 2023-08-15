use compass_core::{
    algorithm::search::{
        edge_traversal::EdgeTraversal, min_search_tree::a_star::a_star_traversal::AStarTraversal,
    },
    model::graph::vertex_id::VertexId,
};
use std::collections::HashMap;
use std::time::Duration;

pub struct SearchAppResult<T> {
    pub origin: T,
    pub destination: T,
    pub route: Vec<EdgeTraversal>,
    pub tree_size: usize,
    pub search_runtime: Duration,
    pub route_runtime: Duration,
}
