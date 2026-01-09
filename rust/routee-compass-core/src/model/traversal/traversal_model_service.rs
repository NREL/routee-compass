use super::{traversal_model::TraversalModel, traversal_model_error::TraversalModelError};
use crate::model::state::{InputFeature, StateModel, StateVariableConfig};
use std::sync::Arc;

/// A [`TraversalModelService`] is a persistent builder of [TraversalModel] instances.
/// Building a [`TraversalModelService`] may be an expensive operation and often includes
/// file IO on the order of the size of the road network edge list.
/// The service then builds a [TraversalModel] instance for each route query.
/// [`TraversalModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// [TraversalModel]: compass_core::model::traversal::traversal_model::TraversalModel
pub trait TraversalModelService: Send + Sync {
    /// List the state variables required as inputs to this traversal model. For
    /// example, if this traversal model uses a distance metric to compute time, then
    /// it should list the expected distance state variable here.
    fn input_features(&self) -> Vec<InputFeature>;

    /// Lists the state variables produced by this traversal model. For example,
    /// if this traversal model produces leg distances, it should specify that here.
    fn output_features(&self) -> Vec<(String, StateVariableConfig)>;

    /// Builds a [TraversalModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [TraversalModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [TraversalModel]
    /// * `state_model` - the state model used in this search
    ///
    /// # Returns
    ///
    /// The [TraversalModel] instance for this query, or an error
    ///
    /// [TraversalModel]: compass_core::model::traversal::traversal_model::TraversalModel
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn TraversalModel>, TraversalModelError>;
}
