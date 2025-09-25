use super::search_error::SearchError;
use super::SearchInstance;
use crate::algorithm::search::SearchTree;
use crate::model::network::{EdgeId, EdgeListId};
use crate::model::state::StateVariable;
use crate::model::unit::Cost;
use allocative::Allocative;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize, Allocative)]
pub struct EdgeTraversal {
    pub edge_list_id: EdgeListId,
    pub edge_id: EdgeId,
    pub cost: Cost,
    pub result_state: Vec<StateVariable>,
}

impl Display for EdgeTraversal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "edge {} cost:{} state:{:?}",
            self.edge_id, self.cost, self.result_state
        )
    }
}

impl EdgeTraversal {
    /// traverses an edge, possibly after traversing some previous edge,
    /// collecting the access and traversal costs. returns the
    /// accumulated cost and updated search state.
    ///
    /// The traversal and access models for the destination edge list are used
    /// over the following graph trajectory:
    ///   `(v1) -[e1]-> (v2) -[e2]-> (v3)`
    ///     - start_state: at location (v2)
    ///     - next_edge: -[e2]->
    ///     - prev_edge: -[e1]-> (optional, is None at start of a forward search)
    ///     - access model applied from e1 to e2
    ///     - traverse model applied to e2
    ///
    /// # Arguments
    ///
    /// * `next_edge`     - the edge to traverse
    /// * `prev_edge`     - the previously traversed edge, if exists, for access costs
    /// * `prev_state`    - the state before traversal, positioned closer to the destination
    /// * `si`            - the search assets for this query
    ///
    /// # Returns
    ///
    /// An edge traversal summarizing the costs and result state of accessing and traversing the next edge.
    pub fn new(
        next_edge: (EdgeListId, EdgeId),
        tree: &SearchTree,
        prev_state: &[StateVariable],
        si: &SearchInstance,
    ) -> Result<EdgeTraversal, SearchError> {
        let mut result_state = prev_state.to_vec();

        let (next_edge_list_id, next_edge_id) = next_edge;

        // find this traversal in the graph
        let traversal_trajectory = si.graph.edge_triplet(&next_edge_list_id, &next_edge_id)?;
        si.get_traversal_model(&next_edge_list_id)?.traverse_edge(
            traversal_trajectory,
            &mut result_state,
            tree,
            &si.state_model,
        )?;

        let (_, edge, _) = traversal_trajectory;
        let cost = si
            .cost_model
            .traversal_cost(edge, prev_state, &result_state)?;

        let result = EdgeTraversal {
            edge_list_id: next_edge_list_id,
            edge_id: next_edge_id,
            cost,
            result_state,
        };

        Ok(result)
    }

    // /// traverses an edge, possibly after traversing some next edge,
    // /// collecting the access and traversal costs in a reverse-oriented
    // /// tree building process. returns the accumulated cost and updated search state.
    // /// used in bi-directional search algorithms. definition of previous and next
    // /// edges is the same as the forward traversal: (v1)-[prev]->(v2)-[next]->(v3)
    // /// but the "next" edge is now the Optional edge.
    // ///
    // /// # Arguments
    // ///
    // /// * `prev_edge_id`     - the edge to traverse
    // /// * `next_edge_id_opt` - the edge previously traversed that appears closer to the origin
    // ///   of this reverse search
    // /// * `prev_state`       - the state before traversal, positioned closer to the destination
    // /// * `si`               - the search assets for this query
    // ///
    // /// # Returns
    // ///
    // /// An edge traversal summarizing the costs and result state of accessing and traversing the previous edge.
    // pub fn reverse_traversal(
    //     prev_edge: (EdgeListId, EdgeId),
    //     next_edge: Option<(EdgeListId, EdgeId)>,
    //     prev_state: &[StateVariable],
    //     si: &SearchInstance,
    // ) -> Result<EdgeTraversal, SearchError> {
    //     let mut result_state = prev_state.to_vec();
    //     let mut access_cost = Cost::ZERO;

    //     let (prev_edge_list_id, prev_edge_id) = prev_edge;

    //     // find this traversal in the graph
    //     let traversal_trajectory = si.graph.edge_triplet(&prev_edge_list_id, &prev_edge_id)?;

    //     // perform access traversal for (v1)-[prev]->(v2)
    //     // access cost for              (v1)-[prev]->(v2)-[next]->(v3)
    //     if let Some((next_edge_list_id, next_edge_id)) = next_edge {
    //         let e2 = si.graph.get_edge(&next_edge_list_id, &next_edge_id)?;
    //         let v3 = si.graph.get_vertex(&e2.dst_vertex_id)?;

    //         let (v1, e1, v2) = traversal_trajectory;
    //         let access_trajectory = (v1, e1, v2, e2, v3);

    //         si.get_access_model(&next_edge_list_id)?
    //             .access_edge(access_trajectory, &mut result_state, &si.state_model)?;

    //         let ac = si
    //             .cost_model
    //             .access_cost(e1, e2, prev_state, &result_state)?;
    //         access_cost = access_cost + ac;
    //     }

    //     si.get_traversal_model(&prev_edge_list_id)?.traverse_edge(
    //         traversal_trajectory,
    //         &mut result_state,
    //         &si.state_model,
    //     )?;

    //     let (_, edge, _) = traversal_trajectory;
    //     let total_cost = si
    //         .cost_model
    //         .traversal_cost(edge, prev_state, &result_state)?;
    //     let traversal_cost = total_cost - access_cost;

    //     let result = EdgeTraversal {
    //         edge_list_id: prev_edge_list_id,
    //         edge_id: prev_edge_id,
    //         access_cost,
    //         traversal_cost,
    //         result_state,
    //     };

    //     Ok(result)
    // }
}
