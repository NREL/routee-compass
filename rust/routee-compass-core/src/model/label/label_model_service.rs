use std::sync::Arc;

use crate::model::{
    label::{label_model::LabelModel, label_model_error::LabelModelError},
    state::StateModel,
};

/// A [`LabelModelService`] is a persistent builder of [FrontierModel] instances.
/// Building a [`LabelModelService`] may require parametrizing the frontier model
/// based on the incoming query.
/// The service then builds a [LabelModel] instance for each route query.
/// [`LabelModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// [LabelModel]: routee_compass_core::model::label::LabelModel
pub trait LabelModelService: Send + Sync {
    /// Builds a [LabelModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [LabelModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [LabelModel]
    ///
    /// # Returns
    ///
    /// The [LabelModel] instance for this query, or an error
    ///
    /// [LabelModel]: routee_compass_core::model::label::LabelModel
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn LabelModel>, LabelModelError>;
}
