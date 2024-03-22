use super::{access_model::AccessModel, access_model_error::AccessModelError};
use std::sync::Arc;

pub trait AccessModelService: Send + Sync {
    /// Builds a [AccessModel] for the incoming query, used as parameters for this
    /// build operation.
    ///
    /// The query is passed as parameters to this operation so that any query-time
    /// coefficients may be applied to the [AccessModel].
    ///
    /// # Arguments
    ///
    /// * `query` - the incoming query which may contain parameters for building the [AccessModel]
    ///
    /// # Returns
    ///
    /// The [AccessModel] instance for this query, or an error
    ///
    /// [AccessModel]: compass_core::model::access::access_model::AccessModel
    fn build(&self, query: &serde_json::Value) -> Result<Arc<dyn AccessModel>, AccessModelError>;
}
