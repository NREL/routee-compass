use itertools::Itertools;

use crate::algorithm::search::{
    edge_traversal::EdgeTraversal, search_error::SearchError, search_instance::SearchInstance,
};

/// helper function to address how the reverse route state and costs are assigned.
///
/// the state values in the reverse route monotonically increase in the reverse direction.
/// the marginal state changes need to be extracted for each link and then added to the final
/// state of the forward route.
///
/// finally, the updated route is reversed so that it can be appended to the forward route.
///
/// # Arguments
/// * `fwd_route` - the forward-oriented route of a bidirectional search
/// * `rev_route` - the reverse-oriented route of a bidirectional search (mutated here)
/// * `si`        - the search instance
pub fn reorient_reverse_route(
    fwd_route: &[EdgeTraversal],
    rev_route: &[EdgeTraversal],
    si: &SearchInstance,
) -> Result<Vec<EdgeTraversal>, SearchError> {
    // get the final edge id and state for the forward traversal
    let (final_fwd_edge_id, mut acc_state) = match fwd_route.last() {
        None => (None, si.state_model.initial_state()?),
        Some(last_edge) => (Some(last_edge.edge_id), last_edge.result_state.clone()),
    };

    // get all edge ids along the reverse route when traversed in forward direction,
    // appending the (optional) last edge id of the forward route
    let mut edge_ids = rev_route
        .iter()
        .rev()
        .map(|e| Some(e.edge_id))
        .collect_vec();
    edge_ids.insert(0, final_fwd_edge_id);

    // re-create all EdgeTraversal instances from each successive edge id pair, building
    // from the final state of the forward traversal
    let mut result: Vec<EdgeTraversal> = Vec::with_capacity(rev_route.len());
    for (prev_opt, next_opt) in edge_ids.iter().tuple_windows() {
        let next = next_opt.ok_or_else(|| {
            SearchError::InternalError(String::from("next_opt should never be None"))
        })?;
        let et = EdgeTraversal::forward_traversal(next, *prev_opt, &acc_state, si)?;
        acc_state = et.result_state.clone();
        result.push(et);
    }

    Ok(result)
}

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
        .map(|e| si.graph.src_vertex_id(&e.edge_id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(src_vertices.iter().unique().collect_vec().len() < src_vertices.len())
}
