use super::{AccessModelError, AccessModelService};
use std::sync::Arc;

/// A [`AccessModelBuilder`] takes a JSON object describing the configuration of a
/// traversal model and builds a [`AccessModelService`].
///
/// A [`AccessModelBuilder`] instance should be an empty struct that implements
/// this trait.
pub trait AccessModelBuilder {
    /// Builds a [`AccessModelService`] from configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "traversal" TOML config section
    ///
    /// # Returns
    ///
    /// A [`AccessModelService`] designed to persist the duration of the CompassApp.
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn AccessModelService>, AccessModelError>;
}
