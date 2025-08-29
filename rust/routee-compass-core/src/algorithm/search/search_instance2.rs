use super::search_error::SearchError;
use crate::model::{
    access::AccessModel,
    cost::CostModel,
    frontier::FrontierModel,
    label::label_model::LabelModel,
    map::MapModel,
    network::{Graph, VertexId},
    state::{StateModel, StateVariable},
    termination::TerminationModel,
    traversal::TraversalModel,
    unit::Cost,
};
use std::sync::Arc;

/// instances of read-only objects used for a search that have
/// been prepared for a specific query.
pub struct SearchInstance2 {
    pub graph: Arc<Graph>,
    pub frontier_models: Vec<Arc<dyn FrontierModel>>,
    pub access_models: Vec<Arc<dyn AccessModel>>,
    pub traversal_models: Vec<Arc<dyn TraversalModel>>,
    pub map_model: Arc<MapModel>,
    pub state_model: Arc<StateModel>,
    pub cost_model: Arc<CostModel>,
    pub termination_model: Arc<TerminationModel>,
    pub label_model: Arc<dyn LabelModel>,
}
