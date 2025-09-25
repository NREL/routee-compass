use itertools::Itertools;

use crate::algorithm::search::{
    edge_traversal::EdgeTraversal, search_error::SearchError, SearchInstance,
};

/// identifies routes that have loops by checking if any two edges share
/// the same source vertex.
///
/// # Arguments
/// * `route` - the route to check
/// * `si`    - the search instance
///
/// # Returns
///
/// true if there is a loop
pub fn route_contains_loop(
    route: &[EdgeTraversal],
    si: &SearchInstance,
) -> Result<bool, SearchError> {
    let src_vertices = route
        .iter()
        .map(|e| si.graph.src_vertex_id(&e.edge_list_id, &e.edge_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(src_vertices.iter().unique().collect_vec().len() < src_vertices.len())
}
