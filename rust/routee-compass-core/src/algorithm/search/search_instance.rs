use crate::{
    algorithm::search::SearchError,
    model::{
        cost::CostModel,
        frontier::FrontierModel,
        label::label_model::LabelModel,
        map::MapModel,
        network::{EdgeListId, Graph},
        state::StateModel,
        termination::TerminationModel,
        traversal::TraversalModel,
    },
};
use std::sync::Arc;

/// instances of read-only objects used for a search that have
/// been prepared for a specific query.
pub struct SearchInstance {
    pub graph: Arc<Graph>,
    pub frontier_models: Vec<Arc<dyn FrontierModel>>,
    pub traversal_models: Vec<Arc<dyn TraversalModel>>,
    pub map_model: Arc<MapModel>,
    pub state_model: Arc<StateModel>,
    pub cost_model: Arc<CostModel>,
    pub termination_model: Arc<TerminationModel>,
    pub label_model: Arc<dyn LabelModel>,
    pub default_edge_list: Option<usize>,
}

impl SearchInstance {
    /// in the case of traversal estimation, where no edges are used, divert to the traversal model
    /// associated with the default edge list
    pub fn get_traversal_estimation_model(&self) -> Arc<dyn TraversalModel> {
        self.traversal_models[self.default_edge_list.unwrap_or_default()].clone()
    }

    pub fn get_frontier_model(
        &self,
        edge_list_id: &EdgeListId,
    ) -> Result<Arc<dyn FrontierModel>, SearchError> {
        self.frontier_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve frontier models for edge list {edge_list_id} that does not exist")))
            .cloned()
    }

    pub fn get_traversal_model(
        &self,
        edge_list_id: &EdgeListId,
    ) -> Result<Arc<dyn TraversalModel>, SearchError> {
        self.traversal_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve traversal models for edge list {edge_list_id} that does not exist")))
            .cloned()
    }
}
