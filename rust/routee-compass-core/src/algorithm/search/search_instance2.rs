use crate::{algorithm::search::SearchError, model::{
    access::AccessModel,
    cost::CostModel,
    frontier::FrontierModel,
    label::label_model::LabelModel,
    map::MapModel,
    network::{EdgeListId, Graph2},
    state::StateModel,
    termination::TerminationModel,
    traversal::TraversalModel,
}};
use std::sync::Arc;

/// instances of read-only objects used for a search that have
/// been prepared for a specific query.
pub struct SearchInstance2 {
    pub graph: Arc<Graph2>,
    pub frontier_models: Vec<Arc<dyn FrontierModel>>,
    pub access_models: Vec<Arc<dyn AccessModel>>,
    pub traversal_models: Vec<Arc<dyn TraversalModel>>,
    pub map_model: Arc<MapModel>,
    pub state_model: Arc<StateModel>,
    pub cost_model: Arc<CostModel>,
    pub termination_model: Arc<TerminationModel>,
    pub label_model: Arc<dyn LabelModel>,
}

impl SearchInstance2 {

    pub fn get_frontier_model(&self, edge_list_id: &EdgeListId) -> Result<Arc<dyn FrontierModel>, SearchError> {
        self.frontier_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve frontier models for edge list {} that does not exist", edge_list_id)))
            .cloned()
    }

    pub fn get_access_model(&self, edge_list_id: &EdgeListId) -> Result<Arc<dyn AccessModel>, SearchError> {
        self.access_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve access models for edge list {} that does not exist", edge_list_id)))
            .cloned()
    }

    pub fn get_traversal_model(&self, edge_list_id: &EdgeListId) -> Result<Arc<dyn TraversalModel>, SearchError> {
        self.traversal_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve traversal models for edge list {} that does not exist", edge_list_id)))
            .cloned()
    }

}