use std::sync::Arc;

use crate::model::{
    label::{label_model::LabelModel, label_model_error::LabelModelError},
    state::StateModel,
};

/// A [`LabelModelService`] is a persistent builder of [LabelModel] instances.
/// Building a [`LabelModelService`] may require parametrizing the label model
/// based on the incoming query.
/// The service then builds a [LabelModel] instance for each route query.
/// [`LabelModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// The service acts as an intermediate layer between the [LabelModelBuilder] and
/// the [LabelModel]. While the builder creates the service once during application
/// startup, the service can create multiple model instances customized for each
/// individual query.
///
/// [LabelModel]: routee_compass_core::model::label::LabelModel
/// [LabelModelBuilder]: super::label_model_builder::LabelModelBuilder
pub trait LabelModelService: Send + Sync {
    /// Builds a [LabelModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [LabelModel]. For example, a label model
    /// might use query parameters to determine which state variables should be
    /// included in the label generation.
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [LabelModel]
    /// * `state_model` - the state model that defines the structure of the state vector
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
