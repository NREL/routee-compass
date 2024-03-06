use crate::model::{
    cost::cost_model::CostModel,
    frontier::frontier_model::FrontierModel,
    road_network::{graph::Graph, vertex_id::VertexId},
    state::state_model::StateModel,
    termination::termination_model::TerminationModel,
    traversal::{state::state_variable::StateVar, traversal_model::TraversalModel},
    unit::Cost,
};
use std::sync::Arc;

use super::search_error::SearchError;

/// instances of read-only objects used for a search that have
/// been prepared for a specific query.
pub struct SearchInstance {
    pub directed_graph: Arc<Graph>,
    pub state_model: Arc<StateModel>,
    pub traversal_model: Arc<dyn TraversalModel>,
    pub cost_model: CostModel,
    pub frontier_model: Arc<dyn FrontierModel>,
    pub termination_model: Arc<TerminationModel>,
}

impl SearchInstance {
    /// approximates the traversal state delta between two vertices and uses
    /// the result to compute a cost estimate.
    pub fn estimate_traversal_cost(
        &self,
        src: VertexId,
        dst: VertexId,
        state: &[StateVar],
    ) -> Result<Cost, SearchError> {
        let src_vertex = self.directed_graph.get_vertex(src)?;
        let dst_vertex = self.directed_graph.get_vertex(dst)?;
        let state_estimate = self
            .traversal_model
            .estimate_traversal(src_vertex, dst_vertex, state)?;
        let cost_estimate = self.cost_model.cost_estimate(state, &state_estimate)?;
        Ok(cost_estimate)
    }
}
