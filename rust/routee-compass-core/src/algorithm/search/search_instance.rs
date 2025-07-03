use super::search_error::SearchError;
use crate::model::{
    access::AccessModel,
    cost::CostModel,
    frontier::FrontierModel,
    label::label_model::LabelModel,
    map::MapModel,
    network::{graph::Graph, vertex_id::VertexId},
    state::{StateModel, StateVariable},
    termination::TerminationModel,
    traversal::TraversalModel,
    unit::Cost,
};
use std::sync::Arc;

/// instances of read-only objects used for a search that have
/// been prepared for a specific query.
pub struct SearchInstance {
    pub graph: Arc<Graph>,
    pub map_model: Arc<MapModel>,
    pub state_model: Arc<StateModel>,
    pub traversal_model: Arc<dyn TraversalModel>,
    pub access_model: Arc<dyn AccessModel>,
    pub cost_model: Arc<CostModel>,
    pub frontier_model: Arc<dyn FrontierModel>,
    pub termination_model: Arc<TerminationModel>,
    pub label_model: Arc<dyn LabelModel>,
}

impl SearchInstance {
    /// approximates the traversal state delta between two vertices and uses
    /// the result to compute a cost estimate.
    pub fn estimate_traversal_cost(
        &self,
        src: VertexId,
        dst: VertexId,
        state: &[StateVariable],
    ) -> Result<Cost, SearchError> {
        let src = self.graph.get_vertex(&src)?;
        let dst = self.graph.get_vertex(&dst)?;
        let mut dst_state = state.to_vec();

        self.traversal_model
            .estimate_traversal((src, dst), &mut dst_state, &self.state_model)?;
        let cost_estimate = self.cost_model.cost_estimate(state, &dst_state)?;
        Ok(cost_estimate)
    }
}
