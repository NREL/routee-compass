use std::sync::Arc;

use crate::model::state::StateModel;

use super::{error::ConstraintModelError, ConstraintModel};

/// A [`ConstraintModelService`] is a persistent builder of [ConstraintModel] instances.
/// Building a [`ConstraintModelService`] may require parametrizing the constraint model
/// based on the incoming query.
/// The service then builds a [ConstraintModel] instance for each route query.
/// [`ConstraintModelService`] must be read across the thread pool and so it implements
/// Send and Sync.
///
/// [ConstraintModel]: routee_compass_core::model::constraint::ConstraintModel
pub trait ConstraintModelService: Send + Sync {
    /// Builds a [ConstraintModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [ConstraintModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [ConstraintModel]
    ///
    /// # Returns
    ///
    /// The [ConstraintModel] instance for this query, or an error
    ///
    /// [ConstraintModel]: routee_compass_core::model::constraint::ConstraintModel
    fn build(
        &self,
        query: &serde_json::Value,
        state_model: Arc<StateModel>,
    ) -> Result<Arc<dyn ConstraintModel>, ConstraintModelError>;
}
