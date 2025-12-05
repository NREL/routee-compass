use crate::{
    algorithm::search::SearchError,
    model::{
        constraint::ConstraintModel,
        cost::CostModel,
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
    pub constraint_models: Vec<Arc<dyn ConstraintModel>>,
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

    pub fn get_constraint_model(
        &self,
        edge_list_id: &EdgeListId,
    ) -> Result<Arc<dyn ConstraintModel>, SearchError> {
        self.constraint_models
            .get(edge_list_id.0)
            .ok_or_else(|| SearchError::InternalError(format!("during search, attempting to retrieve constraint models for edge list {edge_list_id} that does not exist")))
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
