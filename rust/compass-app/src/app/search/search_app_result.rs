use compass_core::algorithm::search::edge_traversal::EdgeTraversal;

pub struct SearchAppResult<T> {
    pub origin: T,
    pub destination: T,
    pub route: Vec<EdgeTraversal>,
    pub tree_size: usize,
}
