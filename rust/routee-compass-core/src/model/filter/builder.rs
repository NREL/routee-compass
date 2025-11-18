use std::sync::Arc;

use super::{error::FilterModelError, FilterModelService};

/// A [`FilterModelBuilder`] takes a JSON object describing the configuration of a
/// filter model and builds a [FilterModel].
///
/// A [`FilterModelBuilder`] instance should be an empty struct that implements
/// this trait.
///
/// [FilterModel]: routee_compass_core::model::frontier::FilterModel
pub trait FilterModelBuilder {
    /// Builds a [FilterModel] from JSON configuration.
    ///
    /// # Arguments
    ///
    /// * `parameters` - the contents of the "frontier" TOML config section
    ///
    /// # Returns
    ///
    /// A [FilterModel] designed to persist the duration of the CompassApp.
    ///
    /// [FilterModel]: routee_compass_core::model::frontier::FilterModel
    fn build(
        &self,
        parameters: &serde_json::Value,
    ) -> Result<Arc<dyn FilterModelService>, FilterModelError>;
}
