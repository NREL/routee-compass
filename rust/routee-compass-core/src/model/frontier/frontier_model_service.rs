use std::sync::Arc;

use super::{frontier_model::FrontierModel, frontier_model_error::FrontierModelError};

/// A [`FrontierModelService`] is a persistent builder of [FrontierModel] instances.
/// Building a [`FrontierModelService`] may require parametrizing the frontier model
/// based on the incoming query.
/// The service then builds a [FrontierModel] instance for each route query.
/// [`FrontierModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// [FrontierModel]: routee_compass_core::model::traversal::traversal_model::FrontierModel
pub trait FrontierModelService: Send + Sync {
    /// Builds a [FrontierModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [FrontierModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [FrontierModel]
    ///
    /// # Returns
    ///
    /// The [FrontierModel] instance for this query, or an error
    ///
    /// [FrontierModel]: routee_compass_core::model::traversal::traversal_model::FrontierModel
    fn build(
        &self,
        query: &serde_json::Value,
    ) -> Result<Arc<dyn FrontierModel>, FrontierModelError>;
}
