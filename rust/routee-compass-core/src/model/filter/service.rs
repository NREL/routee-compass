use std::sync::Arc;

use crate::model::state::StateModel;

use super::{error::FilterModelError, FilterModel};

/// A [`FilterModelService`] is a persistent builder of [FilterModel] instances.
/// Building a [`FilterModelService`] may require parametrizing the filter model
/// based on the incoming query.
/// The service then builds a [FilterModel] instance for each route query.
/// [`FilterModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// [FilterModel]: routee_compass_core::model::filter::FilterModel
pub trait FilterModelService: Send + Sync {
    /// Builds a [FilterModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [FilterModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [FilterModel]
    ///
    /// # Returns
    ///
    /// The [FilterModel] instance for this query, or an error
    ///
    /// [FilterModel]: routee_compass_core::model::filter::FilterModel
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn FilterModel>, FilterModelError>;
}
